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
//! (= they are not marked as [BeingRedeemed](hopr_internal_types::tickets::AcknowledgedTicketStatus::BeingRedeemed) or
//! [BeingAggregated](hopr_internal_types::tickets::AcknowledgedTicketStatus::BeingAggregated) in the DB),
//! If they are redeemable, their state is changed to
//! [BeingRedeemed](hopr_internal_types::tickets::AcknowledgedTicketStatus::BeingRedeemed) (while having acquired the
//! exclusive DB write lock). Subsequently, the ticket in such a state is transmitted into the
//! [ActionQueue](crate::action_queue::ActionQueue) so the redemption is soon executed on-chain. The functions return
//! immediately but provide futures that can be awaited in case the callers wish to await the on-chain confirmation of
//! each ticket redemption.
//!
//! See the details in [ActionQueue](crate::action_queue::ActionQueue) on how the confirmation is realized by awaiting
//! the respective [SignificantChainEvent](hopr_chain_types::chain_events::SignificantChainEvent). by the Indexer.
use async_trait::async_trait;
use futures::StreamExt;
use hopr_chain_types::actions::Action;
use hopr_crypto_types::types::Hash;
use hopr_db_sql::{
    api::{
        info::DomainSeparator,
        tickets::{HoprDbTicketOperations, TicketSelector},
    },
    channels::HoprDbChannelOperations,
    prelude::HoprDbInfoOperations,
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, info, warn};

use crate::{
    ChainActions,
    action_queue::PendingAction,
    errors::{
        ChainActionsError::{ChannelDoesNotExist, InvalidState, OldTicket, WrongTicketState},
        Result,
    },
};

lazy_static::lazy_static! {
    /// Used as a placeholder when the redeem transaction has not yet been published on-chain
    static ref EMPTY_TX_HASH: Hash = Hash::default();
}

/// Gathers all the ticket redemption-related on-chain calls.
#[async_trait]
pub trait TicketRedeemActions {
    /// Redeems all redeemable tickets in all channels.
    async fn redeem_all_tickets(&self, min_value: HoprBalance, only_aggregated: bool) -> Result<Vec<PendingAction>>;

    /// Redeems all redeemable tickets in the incoming channel from the given counterparty.
    async fn redeem_tickets_with_counterparty(
        &self,
        counterparty: &Address,
        min_value: HoprBalance,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>>;

    /// Redeems all redeemable tickets in the given channel.
    async fn redeem_tickets_in_channel(
        &self,
        channel: &ChannelEntry,
        min_value: HoprBalance,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>>;

    /// Redeems all tickets based on the given [`TicketSelector`].
    async fn redeem_tickets(&self, selector: TicketSelector) -> Result<Vec<PendingAction>>;

    /// Tries to redeem the given ticket. If the ticket is not redeemable, returns an error.
    /// Otherwise, the transaction hash of the on-chain redemption is returned.
    async fn redeem_ticket(&self, ack: AcknowledgedTicket) -> Result<PendingAction>;
}

#[async_trait]
impl<Db> TicketRedeemActions for ChainActions<Db>
where
    Db: HoprDbChannelOperations + HoprDbTicketOperations + HoprDbInfoOperations + Clone + Send + Sync + std::fmt::Debug,
{
    #[tracing::instrument(level = "debug", skip(self, min_value, only_aggregated))]
    async fn redeem_all_tickets(&self, min_value: HoprBalance, only_aggregated: bool) -> Result<Vec<PendingAction>> {
        let incoming_channels = self
            .db
            .get_channels_via(None, ChannelDirection::Incoming, &self.self_address())
            .await?;
        debug!(
            channel_count = incoming_channels.len(),
            "starting to redeem all tickets in channels to self"
        );

        let mut receivers: Vec<PendingAction> = vec![];

        // Must be synchronous because underlying Ethereum transactions are sequential
        for incoming_channel in incoming_channels {
            match self
                .redeem_tickets_in_channel(&incoming_channel, min_value, only_aggregated)
                .await
            {
                Ok(mut successful_txs) => {
                    receivers.append(&mut successful_txs);
                }
                Err(e) => {
                    warn!(
                        channel = %generate_channel_id(&incoming_channel.source, &incoming_channel.destination),
                        error = %e,
                        "Failed to redeem tickets in channel",
                    );
                }
            }
        }

        Ok(receivers)
    }

    #[tracing::instrument(level = "debug", skip(self, min_value, only_aggregated))]
    async fn redeem_tickets_with_counterparty(
        &self,
        counterparty: &Address,
        min_value: HoprBalance,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>> {
        let maybe_channel = self
            .db
            .get_channel_by_parties(None, counterparty, &self.self_address(), false)
            .await?;
        if let Some(channel) = maybe_channel {
            self.redeem_tickets_in_channel(&channel, min_value, only_aggregated)
                .await
        } else {
            Err(ChannelDoesNotExist)
        }
    }

    #[tracing::instrument(level = "debug", skip(self, min_value, only_aggregated))]
    async fn redeem_tickets_in_channel(
        &self,
        channel: &ChannelEntry,
        min_value: HoprBalance,
        only_aggregated: bool,
    ) -> Result<Vec<PendingAction>> {
        self.redeem_tickets(
            TicketSelector::from(channel)
                .with_aggregated_only(only_aggregated)
                .with_index_range(channel.ticket_index.as_u64()..)
                .with_amount(min_value..)
                .with_state(AcknowledgedTicketStatus::Untouched),
        )
        .await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn redeem_tickets(&self, selector: TicketSelector) -> Result<Vec<PendingAction>> {
        let (count_redeemable_tickets, _) = self.db.get_tickets_value(selector.clone()).await?;

        info!(
            count_redeemable_tickets, %selector,
            "acknowledged tickets in channel that can be redeemed"
        );

        // Return fast if there are no redeemable tickets
        if count_redeemable_tickets == 0 {
            return Ok(vec![]);
        }

        let channel_dst = self
            .db
            .get_indexer_data(None)
            .await?
            .domain_separator(DomainSeparator::Channel)
            .ok_or(InvalidState("missing channel dst".into()))?;

        let selector_id = selector.to_string();

        // Collect here, so we don't hold-up the stream open for too long
        let redeem_stream = self
            .db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect::<Vec<_>>()
            .await;

        let mut receivers: Vec<PendingAction> = vec![];
        for ack_ticket in redeem_stream {
            let ticket_id = ack_ticket.to_string();

            if let Ok(redeemable) = ack_ticket.into_redeemable(&self.chain_key, &channel_dst) {
                let action = self.tx_sender.send(Action::RedeemTicket(redeemable)).await;
                match action {
                    Ok(successful_tx) => {
                        receivers.push(successful_tx);
                    }
                    Err(e) => {
                        error!(ticket_id, error = %e, "Failed to submit transaction that redeems ticket",);
                    }
                }
            } else {
                error!("failed to extract redeemable ticket");
            }
        }

        info!(
            count = receivers.len(),
            selector = selector_id,
            "acknowledged tickets were submitted to redeem in channel",
        );

        Ok(receivers)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> Result<PendingAction> {
        if let Some(channel) = self
            .db
            .get_channel_by_id(None, &ack_ticket.verified_ticket().channel_id)
            .await?
        {
            // Check if not trying to redeem a ticket that cannot be redeemed.
            // Such tickets are automatically cleaned up (neglected) after successful redemption.
            if ack_ticket.verified_ticket().index < channel.ticket_index.as_u64() {
                return Err(OldTicket);
            }

            debug!(%ack_ticket, %channel, "redeeming single ticket");

            let selector = TicketSelector::from(&channel)
                .with_index(ack_ticket.verified_ticket().index)
                .with_state(AcknowledgedTicketStatus::Untouched);

            // Do not hold up the stream open for too long
            let maybe_ticket = self
                .db
                .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
                .await?
                .next()
                .await;

            if let Some(ticket) = maybe_ticket {
                let channel_dst = self
                    .db
                    .get_indexer_data(None)
                    .await?
                    .domain_separator(DomainSeparator::Channel)
                    .ok_or(InvalidState("missing channel dst".into()))?;

                let redeemable = ticket.into_redeemable(&self.chain_key, &channel_dst)?;

                debug!(%ack_ticket, "ticket is redeemable");
                Ok(self.tx_sender.send(Action::RedeemTicket(redeemable)).await?)
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
    use futures::FutureExt;
    use hex_literal::hex;
    use hopr_chain_types::chain_events::{ChainEventType::TicketRedeemed, SignificantChainEvent};
    use hopr_crypto_random::{Randomizable, random_bytes};
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::{
        HoprDbGeneralModelOperations, TargetDb, api::info::DomainSeparator, db::HoprDb, errors::DbSqlError,
        info::HoprDbInfoOperations,
    };

    use super::*;
    use crate::{
        action_queue::{ActionQueue, MockTransactionExecutor},
        action_state::MockActionState,
    };

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be constructible");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be constructible");
        static ref CHARLIE: ChainKeypair = ChainKeypair::from_secret(&hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26")).expect("lazy static keypair should be constructible");
    }

    const PRICE_PER_PACKET: u128 = 10000000000000000u128; // 0.01 HOPR

    fn generate_random_ack_ticket(
        idx: u64,
        counterparty: &ChainKeypair,
        channel_epoch: u32,
    ) -> anyhow::Result<AcknowledgedTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let resp = Response::from_half_keys(&hk1, &hk2)?;

        Ok(TicketBuilder::default()
            .addresses(counterparty, &*ALICE)
            .amount(U256::from(PRICE_PER_PACKET).div_f64(1.0f64)? * 5u32)
            .index(idx)
            .index_offset(1)
            .win_prob(WinningProbability::ALWAYS)
            .channel_epoch(channel_epoch)
            .challenge(resp.to_challenge()?)
            .build_signed(counterparty, &Hash::default())?
            .into_acknowledged(resp))
    }

    async fn create_channel_with_ack_tickets(
        db: HoprDb,
        ticket_count: usize,
        counterparty: &ChainKeypair,
        channel_epoch: u32,
    ) -> anyhow::Result<(ChannelEntry, Vec<AcknowledgedTicket>)> {
        let ckp = counterparty.clone();
        let db_clone = db.clone();
        let channel = db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;

                    let channel = ChannelEntry::new(
                        ckp.public().to_address(),
                        ALICE.public().to_address(),
                        0.into(),
                        U256::zero(),
                        ChannelStatus::Open,
                        channel_epoch.into(),
                    );
                    db_clone.upsert_channel(Some(tx), channel).await?;
                    Ok::<_, DbSqlError>(channel)
                })
            })
            .await?;

        let ckp = counterparty.clone();
        let input_tickets = db
            .begin_transaction_in_db(TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let mut input_tickets = Vec::new();
                    for i in 0..ticket_count {
                        let ack_ticket = generate_random_ack_ticket(i as u64, &ckp, channel_epoch)
                            .map_err(|e| hopr_db_sql::errors::DbSqlError::LogicalError(e.to_string()))?;
                        db.upsert_ticket(Some(tx), ack_ticket.clone()).await?;
                        input_tickets.push(ack_ticket);
                    }
                    Ok::<_, DbSqlError>(input_tickets)
                })
            })
            .await?;

        Ok((channel, input_tickets))
    }

    #[tokio::test]
    async fn test_ticket_redeem_flow() -> anyhow::Result<()> {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let ticket_count = 5;
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        // All the tickets can be redeemed because they are issued with the same channel epoch
        let (channel_from_bob, bob_tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, 4u32).await?;
        let (channel_from_charlie, charlie_tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &CHARLIE, 4u32).await?;

        // Add extra ticket to Charlie that has very low value
        let resp = Response::from_half_keys(&HalfKey::random(), &HalfKey::random())?;
        let low_value_ack_ticket = TicketBuilder::default()
            .addresses(&*CHARLIE, &*ALICE)
            .amount(PRICE_PER_PACKET)
            .index((ticket_count + 1) as u64)
            .index_offset(1)
            .win_prob(WinningProbability::ALWAYS)
            .channel_epoch(4u32)
            .challenge(resp.to_challenge()?)
            .build_signed(&CHARLIE, &Hash::default())?
            .into_acknowledged(resp);
        db.upsert_ticket(None, low_value_ack_ticket).await?;

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
            .withf(move |t| bob_tickets.iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_| Ok(random_hash));

        // And then all Charlie's tickets get redeemed except the one that does not meet the minimum value
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count)
            .in_sequence(&mut seq)
            .withf(move |t| charlie_tickets.iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_| Ok(random_hash));

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE, db.clone(), tx_sender.clone());

        // Make sure that the low value ticket does not pass the min_value check
        let confirmations = futures::future::try_join_all(
            actions
                .redeem_all_tickets((PRICE_PER_PACKET * 5).into(), false)
                .await?
                .into_iter(),
        )
        .await?;

        assert_eq!(2 * ticket_count, confirmations.len(), "must have all confirmations");
        assert!(
            confirmations.into_iter().all(|c| c.tx_hash == random_hash),
            "tx hashes must be equal"
        );

        let db_acks_bob = db.get_tickets((&channel_from_bob).into()).await?;

        let db_acks_charlie = db.get_tickets((&channel_from_charlie).into()).await?;

        assert!(
            db_acks_bob
                .into_iter()
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::BeingRedeemed),
            "all bob's tickets must be in BeingRedeemed state"
        );
        assert!(
            db_acks_charlie
                .iter()
                .take(ticket_count)
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::BeingRedeemed),
            "all valuable charlie's tickets must be in BeingRedeemed state"
        );
        assert!(
            db_acks_charlie
                .iter()
                .skip(ticket_count)
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::Untouched),
            "all non-valuable charlie's tickets must be in Untouched state"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_redeem_in_channel() -> anyhow::Result<()> {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let ticket_count = 5;
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        // All the tickets can be redeemed because they are issued with the same channel epoch
        let (mut channel_from_bob, bob_tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, 4u32).await?;
        let (channel_from_charlie, _) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &CHARLIE, 4u32).await?;

        // Tickets with index 0 will be skipped, as that is already past
        channel_from_bob.ticket_index = 1_u32.into();
        db.upsert_channel(None, channel_from_bob).await?;

        let mut indexer_action_tracker = MockActionState::new();
        let mut seq2 = mockall::Sequence::new();

        // Skipping ticket with index 0
        for tkt in bob_tickets.iter().skip(1).cloned() {
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

        let mut tx_exec = MockTransactionExecutor::new();
        let mut seq = mockall::Sequence::new();

        // Expect only Bob's tickets to get redeemed
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count - 1)
            .in_sequence(&mut seq)
            .withf(move |t| bob_tickets.iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_| Ok(random_hash));

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE, db.clone(), tx_sender.clone());

        let confirmations = futures::future::try_join_all(
            actions
                .redeem_tickets_with_counterparty(&BOB.public().to_address(), 0.into(), false)
                .await?
                .into_iter(),
        )
        .await?;

        // First ticket is skipped, because its index is lower than the index on the channel entry
        assert_eq!(ticket_count - 1, confirmations.len(), "must have all confirmations");
        assert!(
            confirmations.into_iter().all(|c| c.tx_hash == random_hash),
            "tx hashes must be equal"
        );

        let db_acks_bob = db.get_tickets((&channel_from_bob).into()).await?;

        let db_acks_charlie = db.get_tickets((&channel_from_charlie).into()).await?;

        assert!(
            db_acks_bob
                .into_iter()
                .take_while(|tkt| tkt.verified_ticket().index != 0)
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::BeingRedeemed),
            "all bob's tickets must be in BeingRedeemed state"
        );
        assert!(
            db_acks_charlie
                .into_iter()
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::Untouched),
            "all charlie's tickets must be in Untouched state"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_redeem_must_not_work_for_tickets_being_aggregated_and_being_redeemed() -> anyhow::Result<()> {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let ticket_count = 3;
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        let (channel_from_bob, mut tickets) =
            create_channel_with_ack_tickets(db.clone(), ticket_count, &BOB, 4u32).await?;

        // Make the first ticket unredeemable
        tickets[0].status = AcknowledgedTicketStatus::BeingAggregated;
        let selector = TicketSelector::from(&tickets[0]).with_no_state();
        db.update_ticket_states(selector, AcknowledgedTicketStatus::BeingAggregated)
            .await?;

        // Make the second ticket unredeemable
        tickets[1].status = AcknowledgedTicketStatus::BeingRedeemed;
        let selector = TicketSelector::from(&tickets[1]).with_no_state();
        db.update_ticket_states(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await?;

        // Expect only the redeemable tickets get redeemed
        let tickets_clone = tickets.clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count - 2)
            .withf(move |t| tickets_clone[2..].iter().any(|tk| tk.ticket.eq(&t.ticket)))
            .returning(move |_| Ok(random_hash));

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
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE, db.clone(), tx_sender.clone());

        let confirmations = futures::future::try_join_all(
            actions
                .redeem_tickets_in_channel(&channel_from_bob, 0.into(), false)
                .await?
                .into_iter(),
        )
        .await?;

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

        Ok(())
    }

    #[tokio::test]
    async fn test_redeem_must_not_work_for_tickets_of_previous_epoch_being_aggregated_and_being_redeemed()
    -> anyhow::Result<()> {
        let ticket_count = 3;
        let ticket_from_previous_epoch_count = 2;
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        // Create 1 ticket in Epoch 4
        let (channel_from_bob, mut tickets) = create_channel_with_ack_tickets(db.clone(), 1, &BOB, 4u32).await?;

        // Insert another 2 tickets in Epoch 3
        let ticket = generate_random_ack_ticket(0, &BOB, 3)?;
        db.upsert_ticket(None, ticket.clone()).await?;
        tickets.insert(0, ticket);

        let ticket = generate_random_ack_ticket(1, &BOB, 3)?;
        db.upsert_ticket(None, ticket.clone()).await?;
        tickets.insert(1, ticket);

        let tickets_clone = tickets.clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count - ticket_from_previous_epoch_count)
            .withf(move |t| {
                tickets_clone[ticket_from_previous_epoch_count..]
                    .iter()
                    .any(|tk| tk.ticket.eq(&t.ticket))
            })
            .returning(move |_| Ok(random_hash));

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
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE, db.clone(), tx_sender.clone());

        futures::future::join_all(
            actions
                .redeem_tickets_in_channel(&channel_from_bob, 0.into(), false)
                .await?
                .into_iter(),
        )
        .await;

        assert!(
            actions.redeem_ticket(tickets[0].clone()).await.is_err(),
            "cannot redeem a ticket that's from the previous epoch"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_redeem_must_not_work_for_tickets_of_next_epoch_being_redeemed() -> anyhow::Result<()> {
        let ticket_count = 3;
        let ticket_from_next_epoch_count = 2;
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        // Create 1 ticket in Epoch 4
        let (channel_from_bob, mut tickets) = create_channel_with_ack_tickets(db.clone(), 1, &BOB, 4u32).await?;

        // Insert another 2 tickets in Epoch 5
        let ticket = generate_random_ack_ticket(0, &BOB, 5)?;
        db.upsert_ticket(None, ticket.clone()).await?;
        tickets.insert(0, ticket);

        let ticket = generate_random_ack_ticket(1, &BOB, 5)?;
        db.upsert_ticket(None, ticket.clone()).await?;
        tickets.insert(1, ticket);

        let tickets_clone = tickets.clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_redeem_ticket()
            .times(ticket_count - ticket_from_next_epoch_count)
            .withf(move |t| {
                tickets_clone[ticket_from_next_epoch_count..]
                    .iter()
                    .any(|tk| tk.ticket.eq(&t.ticket))
            })
            .returning(move |_| Ok(random_hash));

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
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE, db.clone(), tx_sender.clone());

        futures::future::join_all(
            actions
                .redeem_tickets_in_channel(&channel_from_bob, 0.into(), false)
                .await?
                .into_iter(),
        )
        .await;

        for ticket in tickets.iter().take(ticket_from_next_epoch_count) {
            assert!(
                actions.redeem_ticket(ticket.clone()).await.is_err(),
                "cannot redeem a ticket that's from the next epoch"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_should_redeem_single_ticket() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let (channel_from_bob, tickets) = create_channel_with_ack_tickets(db.clone(), 1, &BOB, 1u32).await?;

        let ticket = tickets.into_iter().next().unwrap();

        let mut tx_exec = MockTransactionExecutor::new();
        let ticket_clone = ticket.clone();
        tx_exec
            .expect_redeem_ticket()
            .once()
            .withf(move |t| ticket_clone.ticket.eq(&t.ticket))
            .returning(move |_| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        let ticket_clone = ticket.clone();
        indexer_action_tracker
            .expect_register_expectation()
            .once()
            .return_once(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: TicketRedeemed(channel_from_bob, Some(ticket_clone)),
                })
                .boxed())
            });

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE, db.clone(), tx_sender.clone());

        let confirmation = actions.redeem_ticket(ticket).await?.await?;

        assert_eq!(confirmation.tx_hash, random_hash);

        assert!(
            db.get_tickets((&channel_from_bob).into())
                .await?
                .into_iter()
                .all(|tkt| tkt.status == AcknowledgedTicketStatus::BeingRedeemed),
            "all bob's tickets must be in BeingRedeemed state"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_should_not_redeem_single_ticket_with_lower_index_than_channel_index() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let (mut channel_from_bob, tickets) = create_channel_with_ack_tickets(db.clone(), 1, &BOB, 1u32).await?;

        channel_from_bob.ticket_index = 2_u32.into();
        db.upsert_channel(None, channel_from_bob).await?;

        let ticket = tickets.into_iter().next().unwrap();

        let mut tx_exec = MockTransactionExecutor::new();
        let ticket_clone = ticket.clone();
        tx_exec
            .expect_redeem_ticket()
            .never()
            .withf(move |t| ticket_clone.ticket.eq(&t.ticket))
            .returning(move |_| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        let ticket_clone = ticket.clone();
        indexer_action_tracker
            .expect_register_expectation()
            .never()
            .return_once(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: TicketRedeemed(channel_from_bob, Some(ticket_clone)),
                })
                .boxed())
            });

        // Start the ActionQueue with the mock TransactionExecutor
        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE, db.clone(), tx_sender.clone());

        assert!(matches!(actions.redeem_ticket(ticket).await, Err(OldTicket)));

        Ok(())
    }
}
