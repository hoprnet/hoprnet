//! ## Aggregating Strategy
//! This strategy automates ticket aggregation on different channel/ticket events.
//! Note that the aggregating strategy can be combined with the Auto Redeem Strategy above.
//!
//! Ticket aggregation is an interactive process and requires cooperation of the ticket issuer, the aggregation
//! will fail if the aggregation takes more than `aggregation_timeout` (in seconds). This does not affect runtime of the
//! strategy, since the ticket aggregation and awaiting it is performed on a separate thread.
//!
//! This strategy listens for two distinct channel events and triggers the interactive aggregation based on different criteria:
//!
//! ### 1) New winning acknowledged ticket event
//!
//! This strategy listens to newly added acknowledged winning tickets and once the amount of tickets in a certain channel reaches
//! an `aggregation_threshold`, the strategy will initiate ticket aggregation in that channel.
//! The strategy can independently also check if the unrealized balance (current balance _minus_ total unredeemed unaggregated tickets value) in a certain channel
//! has not gone over `unrelalized_balance_ratio` percent of the current balance in that channel. If that happens, the strategy will also initiate
//! ticket aggregation.
//!
//! ### 2) Channel transition from `Open` to `PendingToClose` event
//!
//! If the `aggregate_on_channel_close` flag is set, the aggregation will be triggered once a channel transitions from `Open` to `PendingToClose` state.
//! This behavior does not have any additional criteria, unlike in the previous event.
//!
//! The aggregation on channel closure slightly differs from the one that happens on a new winning ticket.
//! The difference lies in the on-failure behaviour.
//! If the aggregation on channel closure fails, the unaggregated tickets in that channel are automatically send for redeeming.
//! When this strategy is triggered from the
//!
//! For details on default parameters see [AggregatingStrategyConfig].
use async_trait::async_trait;
pub use core_protocol::ticket_aggregation::processor::AwaitingAggregator;
use hopr_crypto_types::prelude::Hash;
use hopr_db_api::channels::HoprDbChannelOperations;
use hopr_db_api::tickets::{AggregationPrerequisites, HoprDbTicketOperations};
use hopr_internal_types::prelude::*;

use async_lock::RwLock;
use async_std::task::JoinHandle;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};
use tracing::{debug, error, info, warn};
use validator::Validate;

use crate::{strategy::SingularStrategy, Strategy};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AGGREGATIONS: SimpleCounter =
        SimpleCounter::new("hopr_strategy_aggregating_aggregation_count", "Count of initiated automatic aggregations").unwrap();
}

use hopr_platform::time::native::current_time;

/// Configuration object for the `AggregatingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AggregatingStrategyConfig {
    /// Number of acknowledged winning tickets in a channel that triggers the ticket aggregation
    /// in that channel when exceeded.
    ///
    /// This condition is independent of `unrealized_balance_ratio`.
    ///
    /// Default is 100.
    #[validate(range(min = 2))]
    #[default(Some(100))]
    pub aggregation_threshold: Option<u32>,

    /// Percentage of unrealized balance in unaggregated tickets in a channel
    /// that triggers the ticket aggregation when exceeded.
    ///
    /// The unrealized balance in this case is the proportion of the channel balance allocated in unredeemed unaggregated tickets.
    /// This condition is independent of `aggregation_threshold`.
    ///
    /// Default is 0.9
    #[validate(range(min = 0_f32, max = 1.0_f32))]
    #[default(Some(0.9))]
    pub unrealized_balance_ratio: Option<f32>,

    /// If set, the strategy will automatically aggregate tickets in channel that has transitioned
    /// to the `PendingToClose` state.
    ///
    /// This happens regardless if `aggregation_threshold` or `unrealized_balance_ratio` thresholds are met on that channel.
    /// If the aggregation on-close fails, the tickets are automatically sent for redeeming instead.
    ///
    /// Default is true.
    #[default = true]
    pub aggregate_on_channel_close: bool,
}

impl From<AggregatingStrategyConfig> for AggregationPrerequisites {
    fn from(value: AggregatingStrategyConfig) -> Self {
        AggregationPrerequisites {
            min_ticket_count: value.aggregation_threshold.map(|x| x as usize),
            min_unaggregated_ratio: value.unrealized_balance_ratio.map(|x| x as f64),
        }
    }
}

/// Represents a strategy that starts aggregating tickets in a certain
/// channel, once the amount of acknowledged tickets in that channel goes
/// above the given threshold.
/// Optionally, the strategy can also redeem the aggregated ticket, if the aggregation
/// was successful.
pub struct AggregatingStrategy<Db, T, U>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
    T: Send + 'static,
    U: Send + 'static,
{
    db: Db,
    ticket_aggregator: Arc<AwaitingAggregator<T, U, Db>>,
    cfg: AggregatingStrategyConfig,
    #[allow(clippy::type_complexity)]
    agg_tasks: Arc<RwLock<HashMap<Hash, (bool, JoinHandle<()>)>>>,
}

impl<Db, T, U> Debug for AggregatingStrategy<Db, T, U>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
    T: Send,
    U: Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::Aggregating(self.cfg))
    }
}

impl<Db, T, U> Display for AggregatingStrategy<Db, T, U>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
    T: Send,
    U: Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Aggregating(self.cfg))
    }
}

impl<Db, T, U> AggregatingStrategy<Db, T, U>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
    T: Send,
    U: Send,
{
    pub fn new(cfg: AggregatingStrategyConfig, db: Db, ticket_aggregator: AwaitingAggregator<T, U, Db>) -> Self {
        Self {
            db,
            cfg,
            ticket_aggregator: Arc::new(ticket_aggregator),
            agg_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<Db, T, U> AggregatingStrategy<Db, T, U>
where
    Db: HoprDbChannelOperations + HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug + 'static,
    T: Send + 'static,
    U: Send + 'static,
{
    async fn try_start_aggregation(
        &self,
        channel_id: Hash,
        criteria: AggregationPrerequisites,
    ) -> crate::errors::Result<()> {
        debug!("starting aggregation in {channel_id} with criteria {criteria:?}");

        if !self.is_strategy_aggregating_in_channel(channel_id).await {
            let agg_tasks_clone = self.agg_tasks.clone();
            let aggregator_clone = self.ticket_aggregator.clone();
            let (can_remove_tx, can_remove_rx) = futures::channel::oneshot::channel();
            let task = async_std::task::spawn(async move {
                match aggregator_clone
                    .aggregate_tickets_in_the_channel(&channel_id, criteria)
                    .await
                {
                    Ok(_) => {
                        info!("completed ticket aggregation in channel {channel_id}");
                    }
                    Err(e) => {
                        error!("cannot complete aggregation in channel {channel_id}: {e}");
                    }
                }

                // Wait until we're added to the aggregation tasks table
                let _ = can_remove_rx.await;
                if let Some((done, _)) = agg_tasks_clone.write().await.get_mut(&channel_id) {
                    *done = true;
                }
            });

            self.agg_tasks.write().await.insert(channel_id, (false, task));
            let _ = can_remove_tx.send(()); // Allow the task to mark itself as done
        } else {
            warn!("this strategy already aggregates in channel {channel_id}");
        }

        Ok(())
    }

    async fn is_strategy_aggregating_in_channel(&self, channel_id: Hash) -> bool {
        let existing = self.agg_tasks.read().await.get(&channel_id).map(|(done, _)| *done);
        if let Some(done) = existing {
            // Task exists, check if it has been completed
            if done {
                if let Some((_, task)) = self.agg_tasks.write().await.remove(&channel_id) {
                    // Task has been completed, remove it and await its join handle
                    task.await;
                    false
                } else {
                    // Should not happen, but means there's no more aggregation task for the channel
                    false
                }
            } else {
                // There's still a running aggregation task for this channel
                true
            }
        } else {
            // No aggregation task found for this channel
            false
        }
    }
}

#[async_trait]
impl<Db, T, U> SingularStrategy for AggregatingStrategy<Db, T, U>
where
    Db: HoprDbChannelOperations + HoprDbTicketOperations + Clone + Send + Sync + std::fmt::Debug + 'static,
    T: Send,
    U: Send,
{
    async fn on_tick(&self) -> crate::errors::Result<()> {
        let incoming = self
            .db
            .get_incoming_channels(None)
            .await?
            .into_iter()
            .filter(|c| !c.closure_time_passed(current_time()))
            .map(|c| c.get_id());

        for channel_id in incoming {
            if let Err(e) = self.try_start_aggregation(channel_id, self.cfg.into()).await {
                debug!("skipped aggregation in channel {channel_id}: {e}");
            }
        }

        Ok(())
    }

    async fn on_own_channel_changed(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        change: ChannelChange,
    ) -> crate::errors::Result<()> {
        if !self.cfg.aggregate_on_channel_close || direction != ChannelDirection::Incoming {
            return Ok(());
        }

        if let ChannelChange::Status { left: old, right: new } = change {
            if old != ChannelStatus::Open || !matches!(new, ChannelStatus::PendingToClose(_)) {
                debug!("ignoring channel {channel} state change that's not in PendingToClose");
                return Ok(());
            }

            info!("going to aggregate tickets in {channel} because it transitioned to PendingToClose");

            Ok(self.try_start_aggregation(channel.get_id(), self.cfg.into()).await?)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::strategy::SingularStrategy;
    use core_protocol::ticket_aggregation::processor::{
        AwaitingAggregator, TicketAggregationInteraction, TicketAggregationProcessed,
    };
    use futures::{FutureExt, StreamExt};
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_db_api::accounts::HoprDbAccountOperations;
    use hopr_db_api::channels::HoprDbChannelOperations;
    use hopr_db_api::db::HoprDb;
    use hopr_db_api::errors::DbError;
    use hopr_db_api::info::{DomainSeparator, HoprDbInfoOperations};
    use hopr_db_api::tickets::HoprDbTicketOperations;
    use hopr_db_api::{HoprDbGeneralModelOperations, TargetDb};
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use std::ops::Add;
    use std::pin::pin;
    use std::time::Duration;
    use tracing::{debug, error};

    lazy_static! {
        static ref PEERS: Vec<OffchainKeypair> = vec![
            hex!("b91a28ff9840e9c93e5fafd581131f0b9f33f3e61b02bf5dd83458aa0221f572"),
            hex!("82283757872f99541ce33a47b90c2ce9f64875abf08b5119a8a434b2fa83ea98")
        ]
        .iter()
        .map(|private| OffchainKeypair::from_secret(private).unwrap())
        .collect();
        static ref PEERS_CHAIN: Vec<ChainKeypair> = vec![
            hex!("51d3003d908045a4d76d0bfc0d84f6ff946b5934b7ea6a2958faf02fead4567a"),
            hex!("e1f89073a01831d0eed9fe2c67e7d65c144b9d9945320f6d325b1cccc2d124e9"),
        ]
        .iter()
        .map(|private| ChainKeypair::from_secret(private).unwrap())
        .collect();
    }

    fn mock_acknowledged_ticket(signer: &ChainKeypair, destination: &ChainKeypair, index: u64) -> AcknowledgedTicket {
        let price_per_packet: U256 = 20_u32.into();
        let ticket_win_prob = 1.0f64;

        let channel_id = generate_channel_id(&signer.into(), &destination.into());

        let channel_epoch = 1u64;
        let domain_separator = Hash::default();

        let response = Response::new(
            &Hash::create(&[
                &channel_id.to_bytes(),
                &channel_epoch.to_be_bytes(),
                &index.to_be_bytes(),
            ])
            .to_bytes(),
        );

        let ticket = Ticket::new(
            &destination.into(),
            &Balance::new(price_per_packet.div_f64(ticket_win_prob).unwrap(), BalanceType::HOPR),
            index.into(),
            1u64.into(),
            ticket_win_prob,
            1u64.into(),
            response.to_challenge().into(),
            signer,
            &domain_separator,
        )
        .unwrap();

        AcknowledgedTicket::new(ticket, response, signer.into(), destination, &domain_separator).unwrap()
    }

    async fn populate_db_with_ack_tickets(db: HoprDb, amount: usize) -> (Vec<AcknowledgedTicket>, ChannelEntry) {
        let db_clone = db.clone();
        let (acked_tickets, total_value) = db
            .begin_transaction_in_db(TargetDb::Tickets)
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    let mut acked_tickets = Vec::new();
                    let mut total_value = Balance::zero(BalanceType::HOPR);

                    for i in 0..amount {
                        let acked_ticket = mock_acknowledged_ticket(&PEERS_CHAIN[0], &PEERS_CHAIN[1], i as u64);
                        debug!("inserting {acked_ticket}");

                        db_clone.upsert_ticket(Some(tx), acked_ticket.clone()).await?;

                        total_value = total_value.add(&acked_ticket.ticket.amount);
                        acked_tickets.push(acked_ticket);
                    }

                    Ok::<_, DbError>((acked_tickets, total_value))
                })
            })
            .await
            .unwrap();

        let channel = ChannelEntry::new(
            (&PEERS_CHAIN[0]).into(),
            (&PEERS_CHAIN[1]).into(),
            total_value,
            (amount as u32).into(),
            ChannelStatus::Open,
            1u32.into(),
        );

        (acked_tickets, channel)
    }

    async fn init_db(db: HoprDb) {
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Hash::default())
                        .await
                        .unwrap();
                    for i in 0..PEERS_CHAIN.len() {
                        debug!(
                            "linking {} with {}",
                            PEERS[i].public(),
                            PEERS_CHAIN[i].public().to_address()
                        );
                        db_clone
                            .insert_account(
                                Some(tx),
                                AccountEntry::new(
                                    *PEERS[i].public(),
                                    PEERS_CHAIN[i].public().to_address(),
                                    AccountType::NotAnnounced,
                                ),
                            )
                            .await
                            .unwrap();
                    }
                    Ok::<_, DbError>(())
                })
            })
            .await
            .unwrap();
    }

    fn spawn_aggregation_interaction(
        db_alice: HoprDb,
        db_bob: HoprDb,
        key_alice: &ChainKeypair,
        key_bob: &ChainKeypair,
    ) -> (
        AwaitingAggregator<(), (), HoprDb>,
        futures::channel::oneshot::Receiver<()>,
    ) {
        let mut alice = TicketAggregationInteraction::<(), ()>::new(db_alice, key_alice);
        let mut bob = TicketAggregationInteraction::<(), ()>::new(db_bob.clone(), key_bob);

        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let bob_aggregator = bob.writer();

        async_std::task::spawn(async move {
            let mut finalizer = None;

            match bob.next().await {
                Some(TicketAggregationProcessed::Send(_, acked_tickets, request_finalizer)) => {
                    let _ = finalizer.insert(request_finalizer);
                    match alice
                        .writer()
                        .receive_aggregation_request(PEERS[1].public().into(), acked_tickets, ())
                    {
                        Ok(_) => {}
                        Err(e) => error!("{e}"),
                    }
                }
                //  alice.ack_event_queue.0.start_send(super::TicketAggregationToProcess::ToProcess(destination, acked_tickets)),
                _ => panic!("unexpected action happened"),
            };

            match alice.next().await {
                Some(TicketAggregationProcessed::Reply(_, aggregated_ticket, ())) => {
                    match bob
                        .writer()
                        .receive_ticket(PEERS[0].public().into(), aggregated_ticket, ())
                    {
                        Ok(_) => {}
                        Err(e) => error!("{e}"),
                    }
                }

                _ => panic!("unexpected action happened"),
            };

            match bob.next().await {
                Some(TicketAggregationProcessed::Receive(_destination, _ticket, ())) => (),
                _ => panic!("unexpected action happened"),
            };

            finalizer.unwrap().finalize();
            let _ = tx.send(());
        });

        (
            AwaitingAggregator::new(db_bob, key_bob.clone(), bob_aggregator, Duration::from_secs(5)),
            awaiter,
        )
    }

    #[async_std::test]
    async fn test_strategy_aggregation_on_tick() {
        let _ = env_logger::builder().is_test(true).try_init();

        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await;

        init_db(db_alice.clone()).await;
        init_db(db_bob.clone()).await;

        let (_, channel) = populate_db_with_ack_tickets(db_bob.clone(), 5).await;

        db_alice.upsert_channel(None, channel).await.unwrap();
        db_bob.upsert_channel(None, channel).await.unwrap();

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1]);

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: Some(5),
            unrealized_balance_ratio: None,
            aggregate_on_channel_close: false,
        };

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_bob.clone(), bob_aggregator);

        //let threshold_ticket = acked_tickets.last().unwrap();
        aggregation_strategy
            .on_tick()
            .await
            .expect("strategy call should succeed");

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(async_std::task::sleep(Duration::from_secs(5)).fuse());
        let _ = futures::future::select(f1, f2).await;

        let tickets = db_bob.get_tickets(None, (&channel).into()).await.unwrap();
        assert_eq!(tickets.len(), 1, "there should be a single aggregated ticket");
    }

    #[async_std::test]
    async fn test_strategy_aggregation_on_tick_when_unrealized_balance_exceeded() {
        let _ = env_logger::builder().is_test(true).try_init();

        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await;

        init_db(db_alice.clone()).await;
        init_db(db_bob.clone()).await;

        let (_, channel) = populate_db_with_ack_tickets(db_bob.clone(), 4).await;

        db_alice.upsert_channel(None, channel).await.unwrap();
        db_bob.upsert_channel(None, channel).await.unwrap();

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1]);

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: None,
            unrealized_balance_ratio: Some(0.75),
            aggregate_on_channel_close: false,
        };

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_bob.clone(), bob_aggregator);

        //let threshold_ticket = acked_tickets.last().unwrap();
        aggregation_strategy
            .on_tick()
            .await
            .expect("strategy call should succeed");

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(async_std::task::sleep(Duration::from_secs(5)));
        let _ = futures::future::select(f1, f2).await;

        let tickets = db_bob.get_tickets(None, (&channel).into()).await.unwrap();
        assert_eq!(tickets.len(), 1, "there should be a single aggregated ticket");
    }

    #[async_std::test]
    async fn test_strategy_aggregation_on_tick_should_not_agg_when_unrealized_balance_exceeded_via_aggregated_tickets()
    {
        let _ = env_logger::builder().is_test(true).try_init();

        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await;

        init_db(db_alice.clone()).await;
        init_db(db_bob.clone()).await;

        const NUM_TICKETS: usize = 4;
        let (mut acked_tickets, mut channel) = populate_db_with_ack_tickets(db_bob.clone(), NUM_TICKETS).await;

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1]);

        // Make this ticket aggregated
        acked_tickets[0].ticket.index_offset = 2;
        acked_tickets[0].ticket.sign(&PEERS_CHAIN[0], &Hash::default());

        debug!("upserting {}", acked_tickets[0]);
        db_bob.upsert_ticket(None, acked_tickets[0].clone()).await.unwrap();

        let tickets = db_bob.get_tickets(None, (&channel).into()).await.unwrap();
        assert_eq!(tickets.len(), NUM_TICKETS, "nothing should be aggregated");

        channel.balance = Balance::new(100_u32, BalanceType::HOPR);

        db_alice.upsert_channel(None, channel).await.unwrap();
        db_bob.upsert_channel(None, channel).await.unwrap();

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: None,
            unrealized_balance_ratio: Some(0.75),
            aggregate_on_channel_close: false,
        };

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_bob.clone(), bob_aggregator);

        //let threshold_ticket = acked_tickets.last().unwrap();
        aggregation_strategy
            .on_tick()
            .await
            .expect("strategy call should succeed");

        let tickets = db_bob.get_tickets(None, (&channel).into()).await.unwrap();
        assert_eq!(tickets.len(), NUM_TICKETS, "nothing should be aggregated");
        std::mem::drop(awaiter);
    }

    #[async_std::test]
    async fn test_strategy_aggregation_on_channel_close() {
        let _ = env_logger::builder().is_test(true).try_init();

        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await;

        init_db(db_alice.clone()).await;
        init_db(db_bob.clone()).await;

        let (_, mut channel) = populate_db_with_ack_tickets(db_bob.clone(), 5).await;

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: None,
            unrealized_balance_ratio: None,
            aggregate_on_channel_close: true,
        };

        channel.status = ChannelStatus::PendingToClose(std::time::SystemTime::now());

        db_alice.upsert_channel(None, channel).await.unwrap();
        db_bob.upsert_channel(None, channel).await.unwrap();

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1]);

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_alice.clone(), bob_aggregator);

        aggregation_strategy
            .on_own_channel_changed(
                &channel,
                ChannelDirection::Incoming,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: ChannelStatus::PendingToClose(std::time::SystemTime::now()),
                },
            )
            .await
            .expect("strategy call should not fail");

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(async_std::task::sleep(Duration::from_secs(5)));
        let _ = futures::future::select(f1, f2).await;

        let tickets = db_bob.get_tickets(None, (&channel).into()).await.unwrap();
        assert_eq!(tickets.len(), 1, "there should be a single aggregated ticket");
    }
}
