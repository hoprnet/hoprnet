use async_lock::RwLock;
use async_trait::async_trait;
use chain_db::traits::HoprCoreEthereumDbActions;
use chain_types::actions::Action;
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use log::{debug, error, info, warn};
use std::ops::DerefMut;
use std::sync::Arc;
use utils_db::errors::DbError;

use crate::action_queue::{ActionSender, PendingAction};
use crate::errors::CoreEthereumActionsError::ChannelDoesNotExist;
use crate::errors::{
    CoreEthereumActionsError::{NotAWinningTicket, WrongTicketState},
    Result,
};
use crate::CoreEthereumActions;

lazy_static::lazy_static! {
    /// Used as a placeholder when the redeem transaction has not yet been published on-chain
    static ref EMPTY_TX_HASH: Hash = Hash::default();
}

/// Gathers all the ticket redemption related on-chain calls.
#[async_trait]
pub trait TicketRedeemActions {
    /// Redeems all redeemable tickets in all channels.
    async fn redeem_all_tickets(&self, only_aggregated: bool) -> Result<Vec<PendingAction>>;

    /// Redeems all redeemable tickets in the incoming channel from the given counterparty.
    async fn redeem_tickets_with_counterparty(
        &self,
        counterparty: &Address,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>>;

    /// Redeems all redeemable tickets in the given channel.
    async fn redeem_tickets_in_channel(
        &self,
        channel: &ChannelEntry,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>>;

    /// Tries to redeem the given ticket. If the ticket is not redeemable, returns an error.
    /// Otherwise, the transaction hash of the on-chain redemption is returned.
    async fn redeem_ticket(&self, ack: AcknowledgedTicket) -> Result<PendingAction>;
}

async fn set_being_redeemed<Db>(db: &mut Db, ack_ticket: &mut AcknowledgedTicket, tx_hash: Hash) -> Result<()>
where
    Db: HoprCoreEthereumDbActions,
{
    match ack_ticket.status {
        AcknowledgedTicketStatus::Untouched => {
            let dst = db
                .get_channels_domain_separator()
                .await
                .and_then(|separator| separator.ok_or(DbError::NotFound))?;

            // Check if we're going to redeem a winning ticket
            if !ack_ticket.is_winning_ticket(&dst) {
                return Err(NotAWinningTicket);
            }
        }
        AcknowledgedTicketStatus::BeingAggregated => return Err(WrongTicketState(ack_ticket.to_string())),
        AcknowledgedTicketStatus::BeingRedeemed => {}
    }

    ack_ticket.status = AcknowledgedTicketStatus::BeingRedeemed;
    debug!(
        "setting a winning {} as being redeemed with TX hash {tx_hash}",
        ack_ticket.ticket
    );
    Ok(db.update_acknowledged_ticket(ack_ticket).await?)
}

async fn unchecked_ticket_redeem<Db>(
    db: Arc<RwLock<Db>>,
    mut ack_ticket: AcknowledgedTicket,
    on_chain_tx_sender: ActionSender,
) -> Result<PendingAction>
where
    Db: HoprCoreEthereumDbActions,
{
    set_being_redeemed(db.write().await.deref_mut(), &mut ack_ticket, *EMPTY_TX_HASH).await?;
    on_chain_tx_sender.send(Action::RedeemTicket(ack_ticket)).await
}

#[async_trait]
impl<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> TicketRedeemActions for CoreEthereumActions<Db> {
    async fn redeem_all_tickets(&self, only_aggregated: bool) -> Result<Vec<PendingAction>> {
        let incoming_channels = self.db.read().await.get_incoming_channels().await?;
        debug!(
            "starting to redeem all tickets in {} incoming channels to us.",
            incoming_channels.len()
        );

        let mut receivers: Vec<PendingAction> = vec![];

        // Must be synchronous because underlying Ethereum transactions are sequential
        for incoming_channel in incoming_channels {
            match self.redeem_tickets_in_channel(&incoming_channel, only_aggregated).await {
                Ok(mut successful_txs) => {
                    receivers.append(&mut successful_txs);
                }
                Err(e) => {
                    warn!(
                        "Failed to redeem tickets in channel {} due to {}",
                        generate_channel_id(&incoming_channel.source, &incoming_channel.destination),
                        e
                    );
                }
            }
        }

        Ok(receivers)
    }

    /// Redeems all redeemable tickets in the incoming channel from the given counterparty.
    async fn redeem_tickets_with_counterparty(
        &self,
        counterparty: &Address,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>> {
        let ch = self.db.read().await.get_channel_from(counterparty).await?;
        if let Some(channel) = ch {
            self.redeem_tickets_in_channel(&channel, only_aggregated).await
        } else {
            Err(ChannelDoesNotExist)
        }
    }

    /// Redeems all redeemable tickets in the given channel.
    async fn redeem_tickets_in_channel(
        &self,
        channel: &ChannelEntry,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>> {
        let channel_id = channel.get_id();

        let count_redeemable_tickets = self
            .db
            .read()
            .await
            .get_acknowledged_tickets(Some(*channel))
            .await?
            .into_iter()
            .filter(|t| {
                t.status == AcknowledgedTicketStatus::Untouched
                    && channel.channel_epoch == U256::from(t.ticket.channel_epoch)
                    && (!only_aggregated || t.ticket.is_aggregated())
            })
            .count();
        info!(
            "there are {count_redeemable_tickets} acknowledged tickets in channel {channel_id} which can be redeemed"
        );

        // Return fast if there are no redeemable tickets
        if count_redeemable_tickets == 0 {
            return Ok(vec![]);
        }

        // Keep holding the DB write lock until we mark all the eligible tickets as BeginRedeemed
        let mut to_redeem = Vec::new();
        {
            // Lock the database and retrieve again all the redeemable tickets
            let mut db = self.db.write().await;
            let redeemable = db
                .get_acknowledged_tickets(Some(*channel))
                .await?
                .into_iter()
                .filter(|t| {
                    AcknowledgedTicketStatus::Untouched == t.status
                        && channel.channel_epoch == U256::from(t.ticket.channel_epoch)
                        && (!only_aggregated || t.ticket.is_aggregated())
                });

            for mut avail_to_redeem in redeemable {
                if let Err(e) = set_being_redeemed(&mut *db, &mut avail_to_redeem, *EMPTY_TX_HASH).await {
                    error!("failed to update state of {}: {e}", avail_to_redeem.ticket)
                } else {
                    to_redeem.push(avail_to_redeem);
                }
            }
        }

        info!(
            "{} acknowledged tickets are still available to redeem in {channel_id}",
            to_redeem.len()
        );

        let mut receivers: Vec<PendingAction> = vec![];

        for acked_ticket in to_redeem {
            let ticket_index = acked_ticket.ticket.index;
            match unchecked_ticket_redeem(self.db.clone(), acked_ticket, self.tx_sender.clone()).await {
                Ok(successful_tx) => {
                    receivers.push(successful_tx);
                }
                Err(e) => {
                    warn!(
                        "Failed to submit transaction that redeem ticket with index {} in channel {} due to {}",
                        ticket_index, channel_id, e
                    );
                }
            }
        }

        Ok(receivers)
    }

    /// Tries to redeem the given ticket. If the ticket is not redeemable, returns an error.
    /// Otherwise, the transaction hash of the on-chain redemption is returned.
    async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> Result<PendingAction> {
        let ch = self.db.read().await.get_channel(&ack_ticket.ticket.channel_id).await?;
        if let Some(channel) = ch {
            if ack_ticket.status == AcknowledgedTicketStatus::Untouched
                && channel.channel_epoch == U256::from(ack_ticket.ticket.channel_epoch)
            {
                unchecked_ticket_redeem(self.db.clone(), ack_ticket, self.tx_sender.clone()).await
            } else {
                Err(WrongTicketState(ack_ticket.to_string()))
            }
        } else {
            Err(ChannelDoesNotExist)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_db::db::CoreEthereumDb;
    use chain_db::traits::HoprCoreEthereumDbActions;
    use chain_types::chain_events::ChainEventType::TicketRedeemed;
    use chain_types::chain_events::SignificantChainEvent;
    use futures::FutureExt;
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use utils_db::constants::ACKNOWLEDGED_TICKETS_PREFIX;
    use utils_db::db::DB;
    use utils_db::CurrentDbShim;

    use crate::action_queue::{ActionQueue, MockTransactionExecutor};
    use crate::action_state::MockActionState;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
        static ref CHARLIE: ChainKeypair = ChainKeypair::from_secret(&hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26")).unwrap();
    }

    fn generate_random_ack_ticket(idx: u32, counterparty: &ChainKeypair, channel_epoch: U256) -> AcknowledgedTicket {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &Balance::new(price_per_packet.div_f64(1.0f64).unwrap() * 5u32, BalanceType::HOPR),
            idx.into(),
            U256::one(),
            1.0f64,
            channel_epoch,
            Challenge::from(cp_sum).to_ethereum_challenge(),
            counterparty,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, counterparty.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    fn to_acknowledged_ticket_key(ack: &AcknowledgedTicket) -> utils_db::db::Key {
        let mut ack_key = Vec::new();

        ack_key.extend_from_slice(&ack.ticket.channel_id.to_bytes());
        ack_key.extend_from_slice(&ack.ticket.channel_epoch.to_be_bytes());
        ack_key.extend_from_slice(&ack.ticket.index.to_be_bytes());

        utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap()
    }

    async fn set_domain_separator(rdb: CurrentDbShim) {
        let inner_db = DB::new(rdb);
        let mut db = CoreEthereumDb::new(inner_db, ALICE.public().to_address());

        db.set_channels_domain_separator(&Hash::default(), &Snapshot::default())
            .await
            .unwrap();
    }

    async fn create_channel_with_ack_tickets(
        rdb: CurrentDbShim,
        ticket_count: usize,
        counterparty: &ChainKeypair,
        channel_epoch: U256,
    ) -> (ChannelEntry, Vec<AcknowledgedTicket>) {
        let mut inner_db = DB::new(rdb);
        let mut input_tickets = Vec::new();

        for i in 0..ticket_count {
            let ack_ticket = generate_random_ack_ticket(i as u32, counterparty, channel_epoch);
            inner_db
                .set(to_acknowledged_ticket_key(&ack_ticket), &ack_ticket)
                .await
                .unwrap();
            input_tickets.push(ack_ticket);
        }

        let mut db = CoreEthereumDb::new(inner_db, ALICE.public().to_address());
        let channel = ChannelEntry::new(
            counterparty.public().to_address(),
            ALICE.public().to_address(),
            Balance::zero(BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            channel_epoch,
            U256::zero(),
        );
        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Default::default())
            .await
            .unwrap();
        db.set_channels_domain_separator(&Hash::default(), &Snapshot::default())
            .await
            .unwrap();

        (channel, input_tickets)
    }

    #[async_std::test]
    async fn test_ticket_redeem_flow() {
        let _ = env_logger::builder().is_test(true).try_init();
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let ticket_count = 5;
        let rdb = CurrentDbShim::new_in_memory().await;

        // all the tickets can be redeemed, coz they are issued with the same epoch as channel
        let (channel_from_bob, bob_tickets) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_count, &BOB, U256::from(4u32)).await;
        let (channel_from_charlie, charlie_tickets) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_count, &CHARLIE, U256::from(4u32)).await;

        // ticket redemption requires a domain separator
        set_domain_separator(rdb.clone()).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        let mut indexer_action_tracker = MockActionState::new();
        let mut seq2 = mockall::Sequence::new();

        for tkt in bob_tickets.iter().cloned() {
            indexer_action_tracker
                .expect_register_expectation()
                .once()
                .in_sequence(&mut seq2)
                .return_once(move |_| {
                    Ok(futures::future::ok(SignificantChainEvent {
                        tx_hash: random_hash,
                        event_type: TicketRedeemed(channel_from_bob, Some(tkt)),
                    })
                    .boxed())
                });
        }

        for tkt in charlie_tickets.iter().cloned() {
            indexer_action_tracker
                .expect_register_expectation()
                .once()
                .in_sequence(&mut seq2)
                .return_once(move |_| {
                    Ok(futures::future::ok(SignificantChainEvent {
                        tx_hash: random_hash,
                        event_type: TicketRedeemed(channel_from_charlie, Some(tkt)),
                    })
                    .boxed())
                });
        }

        let mut tx_exec = MockTransactionExecutor::new();
        let mut seq = mockall::Sequence::new();

        // Expect all Bob's tickets get redeemed first
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count)
            .in_sequence(&mut seq)
            .withf(move |t, _| bob_tickets.iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_, _| Ok(random_hash));

        // and then all Charlie's tickets get redeemed
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count)
            .in_sequence(&mut seq)
            .withf(move |t, _| charlie_tickets.iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_, _| Ok(random_hash));

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = CoreEthereumActions::new(ALICE.public().to_address(), db.clone(), tx_sender.clone());

        let confirmations = futures::future::try_join_all(
            actions
                .redeem_all_tickets(false)
                .await
                .expect("redeem_all_tickets should succeed")
                .into_iter(),
        )
        .await
        .expect("must resolve confirmations");

        assert_eq!(2 * ticket_count, confirmations.len(), "must have all confirmations");
        assert!(
            confirmations.into_iter().all(|c| c.tx_hash == random_hash),
            "tx hashes must be equal"
        );

        let db_acks_bob = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_bob))
            .await
            .unwrap();

        let db_acks_charlie = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_charlie))
            .await
            .unwrap();

        assert!(
            db_acks_bob
                .into_iter()
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::BeingRedeemed),
            "all bob's tickets must be in BeingRedeemed state"
        );
        assert!(
            db_acks_charlie
                .into_iter()
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::BeingRedeemed),
            "all charlie's tickets must be in BeingRedeemed state"
        );
    }

    #[async_std::test]
    async fn test_ticket_redeem_in_channel() {
        let _ = env_logger::builder().is_test(true).try_init();
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let ticket_count = 5;
        let rdb = CurrentDbShim::new_in_memory().await;

        // all the tickets can be redeemed, coz they are issued with the same epoch as channel
        let (channel_from_bob, bob_tickets) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_count, &BOB, U256::from(4u32)).await;
        let (channel_from_charlie, _) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_count, &CHARLIE, U256::from(4u32)).await;

        // ticket redemption requires a domain separator
        set_domain_separator(rdb.clone()).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        let mut indexer_action_tracker = MockActionState::new();
        let mut seq2 = mockall::Sequence::new();

        for tkt in bob_tickets.iter().cloned() {
            indexer_action_tracker
                .expect_register_expectation()
                .once()
                .in_sequence(&mut seq2)
                .return_once(move |_| {
                    Ok(futures::future::ok(SignificantChainEvent {
                        tx_hash: random_hash,
                        event_type: TicketRedeemed(channel_from_bob, Some(tkt)),
                    })
                    .boxed())
                });
        }

        // Expect only Bob's tickets to get redeemed
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count)
            .withf(move |t, _| bob_tickets.iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_, _| Ok(random_hash));

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = CoreEthereumActions::new(ALICE.public().to_address(), db.clone(), tx_sender.clone());

        let confirmations = futures::future::try_join_all(
            actions
                .redeem_tickets_with_counterparty(&BOB.public().to_address(), false)
                .await
                .expect("redeem_tickets_with_counterparty should succeed")
                .into_iter(),
        )
        .await
        .expect("must resolve all confirmations");

        assert_eq!(ticket_count, confirmations.len(), "must have all confirmations");
        assert!(
            confirmations.into_iter().all(|c| c.tx_hash == random_hash),
            "tx hashes must be equal"
        );

        let db_acks_bob = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_bob))
            .await
            .unwrap();

        let db_acks_charlie = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_charlie))
            .await
            .unwrap();

        assert!(
            db_acks_bob
                .into_iter()
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::BeingRedeemed),
            "all bob's tickets must be in BeingRedeemed state"
        );
        assert!(
            db_acks_charlie
                .into_iter()
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::Untouched),
            "all charlie's tickets must be in Untouched state"
        );
    }

    #[async_std::test]
    async fn test_redeem_must_not_work_for_tickets_being_aggregated_and_being_redeemed() {
        let _ = env_logger::builder().is_test(true).try_init();
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let ticket_count = 3;
        let rdb = CurrentDbShim::new_in_memory().await;

        let (channel_from_bob, mut tickets) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_count, &BOB, U256::from(4u32)).await;

        // ticket redemption requires a domain separator
        set_domain_separator(rdb.clone()).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        // Make the first ticket unredeemable
        tickets[0].status = AcknowledgedTicketStatus::BeingAggregated;
        db.write().await.update_acknowledged_ticket(&tickets[0]).await.unwrap();

        // Make the second ticket unredeemable
        tickets[1].status = AcknowledgedTicketStatus::BeingRedeemed;
        db.write().await.update_acknowledged_ticket(&tickets[1]).await.unwrap();

        // Expect only the redeemable tickets get redeemed
        let tickets_clone = tickets.clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count - 2)
            .withf(move |t, _| tickets_clone[2..].iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_, _| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        for tkt in tickets.iter().skip(2).cloned() {
            indexer_action_tracker
                .expect_register_expectation()
                .once()
                .return_once(move |_| {
                    Ok(futures::future::ok(SignificantChainEvent {
                        tx_hash: random_hash,
                        event_type: TicketRedeemed(channel_from_bob, Some(tkt)),
                    })
                    .boxed())
                });
        }

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = CoreEthereumActions::new(ALICE.public().to_address(), db.clone(), tx_sender.clone());

        let confirmations = futures::future::try_join_all(
            actions
                .redeem_tickets_in_channel(&channel_from_bob, false)
                .await
                .expect("redeem_tickets_in_channel should succeed")
                .into_iter(),
        )
        .await
        .expect("must resolve all confirmations");

        assert_eq!(
            ticket_count - 2,
            confirmations.len(),
            "must redeem only redeemable tickets in channel"
        );

        assert!(
            actions.redeem_ticket(tickets[0].clone()).await.is_err(),
            "cannot redeem a ticket that's being aggregated"
        );

        assert!(
            actions.redeem_ticket(tickets[1].clone()).await.is_err(),
            "cannot redeem a ticket that's being redeemed"
        );
    }

    #[async_std::test]
    async fn test_redeem_must_not_work_for_tickets_of_previous_epoch_being_aggregated_and_being_redeemed() {
        let _ = env_logger::builder().is_test(true).try_init();

        let ticket_count = 3;
        let ticket_from_previous_epoch_count = 1;
        let rdb = CurrentDbShim::new_in_memory().await;
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        // Make the first ticket from the previous epoch
        let (_, tickets_from_previous_epoch) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_from_previous_epoch_count, &BOB, U256::from(3u32))
                .await;
        // remaining tickets are from the current epoch
        let (channel_from_bob, mut tickets_from_current_epoch) = create_channel_with_ack_tickets(
            rdb.clone(),
            ticket_count - ticket_from_previous_epoch_count,
            &BOB,
            U256::from(4u32),
        )
        .await;

        // ticket redemption requires a domain separator
        set_domain_separator(rdb.clone()).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        // Expect only the redeemable tickets get redeemed
        let mut tickets = tickets_from_previous_epoch.clone();
        tickets.append(&mut tickets_from_current_epoch);

        let tickets_clone = tickets.clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count - ticket_from_previous_epoch_count)
            .withf(move |t, _| {
                tickets_clone[ticket_from_previous_epoch_count..]
                    .iter()
                    .any(|tk| tk.ticket.eq(&t.ticket))
            })
            .returning(move |_, _| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        for tkt in tickets.iter().skip(ticket_from_previous_epoch_count).cloned() {
            indexer_action_tracker
                .expect_register_expectation()
                .once()
                .return_once(move |_| {
                    Ok(futures::future::ok(SignificantChainEvent {
                        tx_hash: random_hash,
                        event_type: TicketRedeemed(channel_from_bob, Some(tkt)),
                    })
                    .boxed())
                });
        }

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = CoreEthereumActions::new(ALICE.public().to_address(), db.clone(), tx_sender.clone());

        futures::future::join_all(
            actions
                .redeem_tickets_in_channel(&channel_from_bob, false)
                .await
                .expect("redeem_tickets_in_channel should succeed")
                .into_iter(),
        )
        .await;

        assert!(
            actions.redeem_ticket(tickets[0].clone()).await.is_err(),
            "cannot redeem a ticket that's from the previous epoch"
        );
    }

    #[async_std::test]
    async fn test_redeem_must_not_work_for_tickets_of_next_epoch_being_redeemed() {
        let _ = env_logger::builder().is_test(true).try_init();

        let ticket_count = 4;
        let ticket_from_next_epoch_count = 2;
        let rdb = CurrentDbShim::new_in_memory().await;
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        // Make the first few tickets from the next epoch
        let (_, tickets_from_next_epoch) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_from_next_epoch_count, &BOB, U256::from(5u32)).await;
        // remaining tickets are from the current epoch
        let (channel_from_bob, mut tickets_from_current_epoch) = create_channel_with_ack_tickets(
            rdb.clone(),
            ticket_count - ticket_from_next_epoch_count,
            &BOB,
            U256::from(4u32),
        )
        .await;

        // ticket redemption requires a domain separator
        set_domain_separator(rdb.clone()).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        // Expect only the redeemable tickets get redeemed
        let mut tickets = tickets_from_next_epoch.clone();
        tickets.append(&mut tickets_from_current_epoch);

        let tickets_clone = tickets.clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count - ticket_from_next_epoch_count)
            .withf(move |t, _| {
                tickets_clone[ticket_from_next_epoch_count..]
                    .iter()
                    .any(|tk| tk.ticket.eq(&t.ticket))
            })
            .returning(move |_, _| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        for tkt in tickets.iter().skip(ticket_from_next_epoch_count).cloned() {
            indexer_action_tracker
                .expect_register_expectation()
                .once()
                .return_once(move |_| {
                    Ok(futures::future::ok(SignificantChainEvent {
                        tx_hash: random_hash,
                        event_type: TicketRedeemed(channel_from_bob, Some(tkt)),
                    })
                    .boxed())
                });
        }

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = CoreEthereumActions::new(ALICE.public().to_address(), db.clone(), tx_sender.clone());

        futures::future::join_all(
            actions
                .redeem_tickets_in_channel(&channel_from_bob, false)
                .await
                .expect("redeem_tickets_in_channel should succeed")
                .into_iter(),
        )
        .await;

        for unredeemable_index in 0..ticket_from_next_epoch_count {
            assert!(
                actions
                    .redeem_ticket(tickets[unredeemable_index].clone())
                    .await
                    .is_err(),
                "cannot redeem a ticket that's from the next epoch"
            );
        }
    }
}
