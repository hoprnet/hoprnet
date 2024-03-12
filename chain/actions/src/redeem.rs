//! This module contains the [TicketRedeemActions] trait defining actions regarding
//! ticket redemption.
//!
//! An implementation of this trait is added to [ChainActions] which realizes the redemption
//! operations via [ActionQueue](crate::action_queue::ActionQueue).
//!
//! There are 4 functions that can be used to redeem tickets in the [TicketRedeemActions] trait:
//! - [redeem_all_tickets](TicketRedeemActions::redeem_all_tickets)
//! - [redeem_tickets_in_channel](TicketRedeemActions::redeem_tickets_in_channel)
//! - [redeem_tickets_with_counterparty](TicketRedeemActions::redeem_tickets_with_counterparty)
//! - [redeem_ticket](TicketRedeemActions::redeem_ticket)
//!
//! Each method first checks if the tickets are redeemable.
//! (= they are not marked as [BeingRedeemed](hopr_internal_types::acknowledgement::AcknowledgedTicketStatus::BeingRedeemed) or
//! [BeingAggregated](hopr_internal_types::acknowledgement::AcknowledgedTicketStatus::BeingAggregated) in the DB),
//! If they are redeemable, their state is changed to
//! [BeingRedeemed](hopr_internal_types::acknowledgement::AcknowledgedTicketStatus::BeingRedeemed) (while having acquired the exclusive DB write lock).
//! Subsequently, the ticket in such state is transmitted into the [ActionQueue](crate::action_queue::ActionQueue) so the redemption is soon executed on-chain.
//! The functions return immediately, but provide futures that can be awaited in case the callers wishes to await the on-chain
//! confirmation of each ticket redemption.
//!
//! See the details in [ActionQueue](crate::action_queue::ActionQueue) on how the confirmation is realized by awaiting the respective [SignificantChainEvent](chain_types::chain_events::SignificantChainEvent).
//! by the Indexer.
use async_trait::async_trait;
use chain_types::actions::Action;
use futures::StreamExt;
use hopr_crypto_types::types::Hash;
use hopr_db_api::channels::HoprDbChannelOperations;
use hopr_db_api::tickets::{HoprDbTicketOperations, TicketSelector};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, info, warn};

use crate::action_queue::PendingAction;
use crate::errors::ChainActionsError::ChannelDoesNotExist;
use crate::errors::{ChainActionsError::WrongTicketState, Result};
use crate::ChainActions;

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

#[async_trait]
impl<Db> TicketRedeemActions for ChainActions<Db>
where
    Db: HoprDbChannelOperations + HoprDbTicketOperations + Clone + Send + Sync + std::fmt::Debug,
{
    #[tracing::instrument(level = "debug", skip(self))]
    async fn redeem_all_tickets(&self, only_aggregated: bool) -> Result<Vec<PendingAction>> {
        let incoming_channels = self
            .db
            .get_channels_via(None, ChannelDirection::Incoming, self.self_address())
            .await?;
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
    #[tracing::instrument(level = "debug", skip(self))]
    async fn redeem_tickets_with_counterparty(
        &self,
        counterparty: &Address,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>> {
        let maybe_channel = self
            .db
            .get_channel_by_id(None, generate_channel_id(counterparty, &self.self_address()))
            .await?;
        if let Some(channel) = maybe_channel {
            self.redeem_tickets_in_channel(&channel, only_aggregated).await
        } else {
            Err(ChannelDoesNotExist)
        }
    }

    /// Redeems all redeemable tickets in the given channel.
    #[tracing::instrument(level = "debug", skip(self))]
    async fn redeem_tickets_in_channel(
        &self,
        channel: &ChannelEntry,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>> {
        let channel_id = channel.get_id();

        let mut selector: TicketSelector = channel.into();
        selector.only_aggregated = only_aggregated;
        selector.state = Some(AcknowledgedTicketStatus::Untouched);

        let (count_redeemable_tickets, _) = self.db.get_tickets_value(None, selector).await?;

        info!(
            "there are {count_redeemable_tickets} acknowledged tickets in channel {channel_id} which can be redeemed"
        );

        // Return fast if there are no redeemable tickets
        if count_redeemable_tickets == 0 {
            return Ok(vec![]);
        }

        let mut redeem_stream = self
            .db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed, &self.me)
            .await?;
        let mut receivers: Vec<PendingAction> = vec![];
        while let Some(ack_ticket) = redeem_stream.next().await {
            let ticket_id = ack_ticket.to_string();
            let action = self.tx_sender.send(Action::RedeemTicket(ack_ticket)).await;
            match action {
                Ok(successful_tx) => {
                    receivers.push(successful_tx);
                }
                Err(e) => {
                    error!("Failed to submit transaction that redeems {ticket_id}: {e}",);
                }
            }
        }

        info!(
            "{} acknowledged tickets were submitted to redeem in {channel_id}",
            receivers.len()
        );

        Ok(receivers)
    }

    /// Tries to redeem the given ticket. If the ticket is not redeemable, returns an error.
    /// Otherwise, the transaction hash of the on-chain redemption is returned.
    #[tracing::instrument(level = "debug", skip(self))]
    async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> Result<PendingAction> {
        if let Some(channel) = self.db.get_channel_by_id(None, ack_ticket.ticket.channel_id).await? {
            let mut selector: TicketSelector = (&channel).into();
            selector.state = Some(AcknowledgedTicketStatus::Untouched);
            selector.index = Some(ack_ticket.ticket.index);

            if let Some(ticket) = self
                .db
                .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed, &self.me)
                .await?
                .next()
                .await
            {
                Ok(self.tx_sender.send(Action::RedeemTicket(ticket)).await?)
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
    use chain_types::chain_events::ChainEventType::TicketRedeemed;
    use chain_types::chain_events::SignificantChainEvent;
    use futures::FutureExt;
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_api::db::HoprDb;
    use hopr_db_api::errors::DbError;
    use hopr_db_api::info::{DomainSeparator, HoprDbInfoOperations};
    use hopr_db_api::{HoprDbGeneralModelOperations, TargetDb};

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

    async fn create_channel_with_ack_tickets(
        db: HoprDb,
        ticket_count: usize,
        counterparty: &ChainKeypair,
        channel_epoch: U256,
    ) -> (ChannelEntry, Vec<AcknowledgedTicket>) {
        let ckp = counterparty.clone();
        let db_clone = db.clone();
        let channel = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;

                    let channel = ChannelEntry::new(
                        ckp.public().to_address(),
                        ALICE.public().to_address(),
                        Balance::zero(BalanceType::HOPR),
                        U256::zero(),
                        ChannelStatus::Open,
                        channel_epoch,
                    );
                    db_clone.upsert_channel(Some(tx), channel).await?;
                    Ok::<_, DbError>(channel)
                })
            })
            .await
            .unwrap();

        let ckp = counterparty.clone();
        let input_tickets = db
            .begin_transaction_in_db(TargetDb::Tickets)
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    let mut input_tickets = Vec::new();
                    for i in 0..ticket_count {
                        let ack_ticket = generate_random_ack_ticket(i as u32, &ckp, channel_epoch);
                        db.upsert_ticket(Some(tx), ack_ticket.clone()).await?;
                        input_tickets.push(ack_ticket);
                    }
                    Ok::<_, DbError>(input_tickets)
                })
            })
            .await
            .unwrap();

        (channel, input_tickets)
    }

    #[async_std::test]
    async fn test_ticket_redeem_flow() {
        let _ = env_logger::builder().is_test(true).try_init();
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let ticket_count = 5;
        let db = HoprDb::new_in_memory().await;

        // all the tickets can be redeemed, coz they are issued with the same epoch as channel
        let (channel_from_bob, bob_tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, U256::from(4u32)).await;
        let (channel_from_charlie, charlie_tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &CHARLIE, U256::from(4u32)).await;

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

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());

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

        let db_acks_bob = db.get_tickets(None, (&channel_from_bob).into(), &ALICE).await.unwrap();

        let db_acks_charlie = db
            .get_tickets(None, (&channel_from_charlie).into(), &ALICE)
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
        let db = HoprDb::new_in_memory().await;

        // all the tickets can be redeemed, coz they are issued with the same epoch as channel
        let (channel_from_bob, bob_tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, U256::from(4u32)).await;
        let (channel_from_charlie, _) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &CHARLIE, U256::from(4u32)).await;

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

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());

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

        let db_acks_bob = db.get_tickets(None, (&channel_from_bob).into(), &ALICE).await.unwrap();

        let db_acks_charlie = db
            .get_tickets(None, (&channel_from_charlie).into(), &ALICE)
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
        let db = HoprDb::new_in_memory().await;

        let (channel_from_bob, mut tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, U256::from(4u32)).await;

        // Make the first ticket unredeemable
        tickets[0].status = AcknowledgedTicketStatus::BeingAggregated;
        let mut selector: TicketSelector = (&tickets[0]).into();
        selector.state = None;
        db.update_ticket_states(selector, AcknowledgedTicketStatus::BeingAggregated)
            .await
            .unwrap();

        // Make the second ticket unredeemable
        tickets[1].status = AcknowledgedTicketStatus::BeingRedeemed;
        let mut selector: TicketSelector = (&tickets[1]).into();
        selector.state = None;
        db.update_ticket_states(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await
            .unwrap();

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

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());

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
        let db = HoprDb::new_in_memory().await;
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        // Create 4 tickets in Epoch
        let (channel_from_bob, mut tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, U256::from(4u32)).await;

        // Update the first 2 to be in Epoch 3
        tickets[0].ticket.channel_epoch = 3;
        db.upsert_ticket(None, tickets[0].clone()).await.unwrap();
        tickets[1].ticket.channel_epoch = 3;
        db.upsert_ticket(None, tickets[1].clone()).await.unwrap();

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

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());

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
        let db = HoprDb::new_in_memory().await;
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        // Create 4 tickets in Epoch
        let (channel_from_bob, mut tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, U256::from(4u32)).await;

        // Update the first 2 to be in Epoch 5
        tickets[0].ticket.channel_epoch = 4;
        db.upsert_ticket(None, tickets[0].clone()).await.unwrap();
        tickets[1].ticket.channel_epoch = 4;
        db.upsert_ticket(None, tickets[1].clone()).await.unwrap();

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

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());

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
