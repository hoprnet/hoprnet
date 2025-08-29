//! ## Aggregating Strategy
//! This strategy automates ticket aggregation on different channel/ticket events.
//! Note that the aggregating strategy can be combined with the Auto Redeem Strategy above.
//!
//! Ticket aggregation is an interactive process and requires cooperation of the ticket issuer, the aggregation
//! will fail if the aggregation takes more than `aggregation_timeout` (in seconds). This does not affect runtime of the
//! strategy, since the ticket aggregation and awaiting it is performed on a separate thread.
//!
//! This strategy listens for two distinct channel events and triggers the interactive aggregation based on different
//! criteria:
//!
//! ### 1) New winning acknowledged ticket event
//!
//! This strategy listens to newly added acknowledged winning tickets and once the amount of tickets in a certain
//! channel reaches an `aggregation_threshold`, the strategy will initiate ticket aggregation in that channel.
//! The strategy can independently also check if the unrealized balance (current balance _minus_ total unredeemed
//! unaggregated tickets value) in a certain channel has not gone over `unrelalized_balance_ratio` percent of the
//! current balance in that channel. If that happens, the strategy will also initiate ticket aggregation.
//!
//! ### 2) Channel transition from `Open` to `PendingToClose` event
//!
//! If the `aggregate_on_channel_close` flag is set, the aggregation will be triggered once a channel transitions from
//! `Open` to `PendingToClose` state. This behavior does not have any additional criteria, unlike in the previous event,
//! but there must be at least 2 tickets in the channel.
//!
//!
//! For details on default parameters see [AggregatingStrategyConfig].
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

use async_lock::{RwLock, RwLockUpgradableReadGuardArc};
use async_trait::async_trait;
use hopr_async_runtime::prelude::{JoinHandle, spawn};
use hopr_crypto_types::prelude::Hash;
use hopr_db_sql::{
    api::tickets::{AggregationPrerequisites, HoprDbTicketOperations},
    channels::HoprDbChannelOperations,
};
use hopr_internal_types::prelude::*;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;
use hopr_transport_ticket_aggregation::TicketAggregatorTrait;
use serde::{Deserialize, Serialize, Serializer};
use serde_with::serde_as;
use tracing::{debug, error, info, warn};
use validator::Validate;

use crate::{Strategy, strategy::SingularStrategy};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AGGREGATIONS: SimpleCounter =
        SimpleCounter::new("hopr_strategy_aggregating_aggregation_count", "Count of initiated automatic aggregations").unwrap();
}

use hopr_platform::time::native::current_time;

const MAX_AGGREGATABLE_TICKET_COUNT: u32 = hopr_db_sql::tickets::MAX_TICKETS_TO_AGGREGATE_BATCH as u32;

#[inline]
fn default_aggregation_threshold() -> Option<u32> {
    Some(250)
}

#[inline]
fn just_true() -> bool {
    true
}

#[inline]
fn default_unrealized_balance_ratio() -> Option<f64> {
    Some(0.9)
}

fn serialize_optional_f64<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(v) = x {
        s.serialize_f64(*v)
    } else {
        s.serialize_none()
    }
}

/// Configuration object for the `AggregatingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AggregatingStrategyConfig {
    /// Number of acknowledged winning tickets in a channel that triggers the ticket aggregation
    /// in that channel when exceeded.
    ///
    /// This condition is independent of `unrealized_balance_ratio`.
    ///
    /// Default is 250.
    #[validate(range(min = 2, max = MAX_AGGREGATABLE_TICKET_COUNT))]
    #[serde(default = "default_aggregation_threshold")]
    #[default(default_aggregation_threshold())]
    pub aggregation_threshold: Option<u32>,

    /// Percentage of unrealized balance in unaggregated tickets in a channel
    /// that triggers the ticket aggregation when exceeded.
    ///
    /// The unrealized balance in this case is the proportion of the channel balance allocated in unredeemed
    /// unaggregated tickets. This condition is independent of `aggregation_threshold`.
    ///
    /// Default is 0.9
    #[validate(range(min = 0_f64, max = 1.0_f64))]
    #[default(default_unrealized_balance_ratio())]
    #[serde(serialize_with = "serialize_optional_f64")]
    pub unrealized_balance_ratio: Option<f64>,

    /// If set, the strategy will automatically aggregate tickets in channel that has transitioned
    /// to the `PendingToClose` state.
    ///
    /// This happens regardless if `aggregation_threshold` or `unrealized_balance_ratio` thresholds are met on that
    /// channel. If the aggregation on-close fails, the tickets are automatically sent for redeeming instead.
    ///
    /// Default is true.
    #[default(just_true())]
    pub aggregate_on_channel_close: bool,
}

impl From<AggregatingStrategyConfig> for AggregationPrerequisites {
    fn from(value: AggregatingStrategyConfig) -> Self {
        AggregationPrerequisites {
            min_ticket_count: value.aggregation_threshold.map(|x| x as usize),
            min_unaggregated_ratio: value.unrealized_balance_ratio,
        }
    }
}

/// Represents a strategy that starts aggregating tickets in a certain
/// channel, once the number of acknowledged tickets in that channel goes
/// above the given threshold.
/// Optionally, the strategy can also redeem the aggregated ticket, if the aggregation
/// was successful.
pub struct AggregatingStrategy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    db: Db,
    ticket_aggregator: Arc<dyn TicketAggregatorTrait + Send + Sync + 'static>,
    cfg: AggregatingStrategyConfig,
    #[allow(clippy::type_complexity)]
    agg_tasks: Arc<RwLock<HashMap<Hash, (bool, JoinHandle<()>)>>>,
}

impl<Db> Debug for AggregatingStrategy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::Aggregating(self.cfg))
    }
}

impl<Db> Display for AggregatingStrategy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Aggregating(self.cfg))
    }
}

impl<Db> AggregatingStrategy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    pub fn new(
        cfg: AggregatingStrategyConfig,
        db: Db,
        ticket_aggregator: Arc<dyn TicketAggregatorTrait + Send + Sync + 'static>,
    ) -> Self {
        Self {
            db,
            cfg,
            ticket_aggregator,
            agg_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<Db> AggregatingStrategy<Db>
where
    Db: HoprDbChannelOperations + HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug + 'static,
{
    async fn try_start_aggregation(
        &self,
        channel_id: Hash,
        criteria: AggregationPrerequisites,
    ) -> crate::errors::Result<()> {
        if !self.is_strategy_aggregating_in_channel(channel_id).await {
            debug!("checking aggregation in {channel_id} with criteria {criteria:?}...");

            let agg_tasks_clone = self.agg_tasks.clone();
            let aggregator_clone = self.ticket_aggregator.clone();
            let (can_remove_tx, can_remove_rx) = futures::channel::oneshot::channel();
            let task = spawn(async move {
                match aggregator_clone.aggregate_tickets(&channel_id, criteria).await {
                    Ok(_) => {
                        debug!("tried ticket aggregation in channel {channel_id} without any issues");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_AGGREGATIONS.increment();
                    }
                    Err(e) => {
                        error!("cannot complete aggregation in channel {channel_id}: {e}");
                    }
                }

                // Wait until we're added to the aggregation tasks table
                let _ = can_remove_rx.await;
                if let Some((done, _)) = agg_tasks_clone.write_arc().await.get_mut(&channel_id) {
                    *done = true;
                }
            });

            self.agg_tasks.write_arc().await.insert(channel_id, (false, task));
            let _ = can_remove_tx.send(()); // Allow the task to mark itself as done
        } else {
            warn!(channel = %channel_id, "this strategy already aggregates in channel");
        }

        Ok(())
    }

    async fn is_strategy_aggregating_in_channel(&self, channel_id: Hash) -> bool {
        let tasks_read_locked = self.agg_tasks.upgradable_read_arc().await;
        let existing = tasks_read_locked.get(&channel_id).map(|(done, _)| *done);
        if let Some(done) = existing {
            // Task exists, check if it has been completed
            if done {
                let mut tasks_write_locked = RwLockUpgradableReadGuardArc::upgrade(tasks_read_locked).await;

                if let Some((_, task)) = tasks_write_locked.remove(&channel_id) {
                    // Task has been completed, remove it and await its join handle
                    let _ = task.await;
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
impl<Db> SingularStrategy for AggregatingStrategy<Db>
where
    Db: HoprDbChannelOperations + HoprDbTicketOperations + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    async fn on_tick(&self) -> crate::errors::Result<()> {
        let incoming = self
            .db
            .get_incoming_channels(None)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?
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

            info!(%channel, "going to aggregate tickets in channel because it transitioned to PendingToClose");

            // On closing there must be at least 2 tickets to justify aggregation
            let on_close_agg_prerequisites = AggregationPrerequisites {
                min_ticket_count: Some(2),
                min_unaggregated_ratio: None,
            };

            Ok(self
                .try_start_aggregation(channel.get_id(), on_close_agg_prerequisites)
                .await?)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{pin::pin, sync::Arc, time::Duration};

    use anyhow::Context;
    use futures::{FutureExt, StreamExt, pin_mut};
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::{
        HoprDbGeneralModelOperations, TargetDb,
        accounts::HoprDbAccountOperations,
        api::{info::DomainSeparator, tickets::HoprDbTicketOperations},
        channels::HoprDbChannelOperations,
        db::HoprDb,
        errors::DbSqlError,
        info::HoprDbInfoOperations,
    };
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use hopr_transport_ticket_aggregation::{
        AwaitingAggregator, TicketAggregationInteraction, TicketAggregationProcessed,
    };
    use lazy_static::lazy_static;
    use tokio::time::timeout;
    use tracing::{debug, error};

    use crate::{
        aggregating::{MAX_AGGREGATABLE_TICKET_COUNT, default_aggregation_threshold},
        strategy::SingularStrategy,
    };

    #[test]
    fn default_ticket_aggregation_count_is_lower_than_maximum_allowed_ticket_count() -> anyhow::Result<()> {
        assert!(default_aggregation_threshold().unwrap() < MAX_AGGREGATABLE_TICKET_COUNT);

        Ok(())
    }

    lazy_static! {
        static ref PEERS: Vec<OffchainKeypair> = [
            hex!("b91a28ff9840e9c93e5fafd581131f0b9f33f3e61b02bf5dd83458aa0221f572"),
            hex!("82283757872f99541ce33a47b90c2ce9f64875abf08b5119a8a434b2fa83ea98")
        ]
        .iter()
        .map(|private| OffchainKeypair::from_secret(private).expect("lazy static keypair should be valid"))
        .collect();
        static ref PEERS_CHAIN: Vec<ChainKeypair> = [
            hex!("51d3003d908045a4d76d0bfc0d84f6ff946b5934b7ea6a2958faf02fead4567a"),
            hex!("e1f89073a01831d0eed9fe2c67e7d65c144b9d9945320f6d325b1cccc2d124e9")
        ]
        .iter()
        .map(|private| ChainKeypair::from_secret(private).expect("lazy static keypair should be valid"))
        .collect();
    }

    fn mock_acknowledged_ticket(
        signer: &ChainKeypair,
        destination: &ChainKeypair,
        index: u64,
        index_offset: u32,
    ) -> anyhow::Result<AcknowledgedTicket> {
        let price_per_packet: U256 = 20_u32.into();
        let ticket_win_prob = 1.0f64;

        let channel_id = generate_channel_id(&signer.into(), &destination.into());

        let channel_epoch = 1u64;
        let domain_separator = Hash::default();

        let response = Response::try_from(
            Hash::create(&[channel_id.as_ref(), &channel_epoch.to_be_bytes(), &index.to_be_bytes()]).as_ref(),
        )?;

        Ok(TicketBuilder::default()
            .addresses(signer, destination)
            .amount(price_per_packet.div_f64(ticket_win_prob)?)
            .index(index)
            .index_offset(index_offset)
            .win_prob(ticket_win_prob.try_into()?)
            .channel_epoch(1)
            .challenge(response.to_challenge()?)
            .build_signed(signer, &domain_separator)?
            .into_acknowledged(response))
    }

    async fn populate_db_with_ack_tickets(
        db: HoprDb,
        amount: usize,
    ) -> anyhow::Result<(Vec<AcknowledgedTicket>, ChannelEntry)> {
        let db_clone = db.clone();
        let (acked_tickets, total_value) = db
            .begin_transaction_in_db(TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let mut acked_tickets = Vec::new();
                    let mut total_value = HoprBalance::zero();

                    for i in 0..amount {
                        let acked_ticket = mock_acknowledged_ticket(&PEERS_CHAIN[0], &PEERS_CHAIN[1], i as u64, 1)
                            .expect("should be able to create an ack ticket");
                        debug!("inserting {acked_ticket}");

                        db_clone.upsert_ticket(Some(tx), acked_ticket.clone()).await?;

                        total_value += acked_ticket.verified_ticket().amount;
                        acked_tickets.push(acked_ticket);
                    }

                    Ok::<_, DbSqlError>((acked_tickets, total_value))
                })
            })
            .await?;

        let channel = ChannelEntry::new(
            (&PEERS_CHAIN[0]).into(),
            (&PEERS_CHAIN[1]).into(),
            total_value,
            0_u32.into(),
            ChannelStatus::Open,
            1u32.into(),
        );

        Ok((acked_tickets, channel))
    }

    async fn init_db(db: HoprDb) -> anyhow::Result<()> {
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Hash::default())
                        .await?;
                    for i in 0..PEERS_CHAIN.len() {
                        debug!(
                            "linking {} with {}",
                            PEERS[i].public(),
                            PEERS_CHAIN[i].public().to_address()
                        );
                        db_clone
                            .insert_account(
                                Some(tx),
                                AccountEntry {
                                    public_key: *PEERS[i].public(),
                                    chain_addr: PEERS_CHAIN[i].public().to_address(),
                                    entry_type: AccountType::NotAnnounced,
                                    published_at: 1,
                                },
                            )
                            .await?;
                    }
                    Ok::<_, DbSqlError>(())
                })
            })
            .await?;

        Ok(())
    }

    fn spawn_aggregation_interaction(
        db_alice: HoprDb,
        db_bob: HoprDb,
        key_alice: &ChainKeypair,
        key_bob: &ChainKeypair,
    ) -> anyhow::Result<(
        AwaitingAggregator<(), (), HoprDb>,
        futures::channel::oneshot::Receiver<()>,
    )> {
        let mut alice = TicketAggregationInteraction::<(), ()>::new(db_alice, key_alice);
        let mut bob = TicketAggregationInteraction::<(), ()>::new(db_bob.clone(), key_bob);

        let (tx, awaiter) = futures::channel::oneshot::channel::<()>();
        let bob_aggregator = bob.writer();

        tokio::task::spawn(async move {
            let mut finalizer = None;

            match bob.next().await {
                Some(TicketAggregationProcessed::Send(_, acked_tickets, request_finalizer)) => {
                    let _ = finalizer.insert(request_finalizer);
                    match alice.writer().receive_aggregation_request(
                        PEERS[1].public().into(),
                        acked_tickets.into_iter().collect(),
                        (),
                    ) {
                        Ok(_) => {}
                        Err(e) => error!(error = %e, "Failed to received aggregation ticket"),
                    }
                }
                //  alice.ack_event_queue.0.start_send(super::TicketAggregationToProcess::ToProcess(destination,
                // acked_tickets)),
                _ => panic!("unexpected action happened"),
            };

            match alice.next().await {
                Some(TicketAggregationProcessed::Reply(_, aggregated_ticket, ())) => {
                    match bob
                        .writer()
                        .receive_ticket(PEERS[0].public().into(), aggregated_ticket, ())
                    {
                        Ok(_) => {}
                        Err(e) => error!(error = %e, "Failed to receive a ticket"),
                    }
                }

                _ => panic!("unexpected action happened"),
            };

            match bob.next().await {
                Some(TicketAggregationProcessed::Receive(_destination, _ticket, ())) => (),
                _ => panic!("unexpected action happened"),
            };

            finalizer.expect("should have a value present").finalize();
            let _ = tx.send(());
        });

        Ok((
            AwaitingAggregator::new(db_bob, bob_aggregator, Duration::from_secs(5)),
            awaiter,
        ))
    }

    #[tokio::test]
    async fn test_strategy_aggregation_on_tick() -> anyhow::Result<()> {
        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await?;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await?;

        init_db(db_alice.clone()).await?;
        init_db(db_bob.clone()).await?;

        let (bob_notify_tx, bob_notify_rx) = futures::channel::mpsc::unbounded();
        db_bob.start_ticket_processing(bob_notify_tx.into())?;

        let (_, channel) = populate_db_with_ack_tickets(db_bob.clone(), 5).await?;

        db_alice.upsert_channel(None, channel).await?;
        db_bob.upsert_channel(None, channel).await?;

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1])?;

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: Some(5),
            unrealized_balance_ratio: None,
            aggregate_on_channel_close: false,
        };

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_bob.clone(), Arc::new(bob_aggregator));

        // let threshold_ticket = acked_tickets.last().unwrap();
        aggregation_strategy.on_tick().await?;

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(tokio::time::sleep(Duration::from_secs(5)).fuse());
        let _ = futures::future::select(f1, f2).await;

        pin_mut!(bob_notify_rx);
        let notified_ticket = bob_notify_rx.next().await.expect("should have a ticket");

        let tickets = db_bob.get_tickets((&channel).into()).await?;
        assert_eq!(tickets.len(), 1, "there should be a single aggregated ticket");
        assert_eq!(notified_ticket, tickets[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_strategy_aggregation_on_tick_when_unrealized_balance_exceeded() -> anyhow::Result<()> {
        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await?;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await?;

        init_db(db_alice.clone()).await?;
        init_db(db_bob.clone()).await?;

        let (bob_notify_tx, bob_notify_rx) = futures::channel::mpsc::unbounded();
        db_bob.start_ticket_processing(bob_notify_tx.into())?;

        let (_, channel) = populate_db_with_ack_tickets(db_bob.clone(), 4).await?;

        db_alice.upsert_channel(None, channel).await?;
        db_bob.upsert_channel(None, channel).await?;

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1])?;

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: None,
            unrealized_balance_ratio: Some(0.75),
            aggregate_on_channel_close: false,
        };

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_bob.clone(), Arc::new(bob_aggregator));

        // let threshold_ticket = acked_tickets.last().unwrap();
        aggregation_strategy.on_tick().await?;

        // Wait until aggregation has finished
        let f1 = pin!(awaiter);
        let f2 = pin!(tokio::time::sleep(Duration::from_secs(5)));
        let _ = futures::future::select(f1, f2).await;

        pin_mut!(bob_notify_rx);
        let notified_ticket = bob_notify_rx.next().await.expect("should have a ticket");

        let tickets = db_bob.get_tickets((&channel).into()).await?;
        assert_eq!(tickets.len(), 1, "there should be a single aggregated ticket");
        assert_eq!(notified_ticket, tickets[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_strategy_aggregation_on_tick_should_not_agg_when_unrealized_balance_exceeded_via_aggregated_tickets()
    -> anyhow::Result<()> {
        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await?;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await?;

        init_db(db_alice.clone()).await?;
        init_db(db_bob.clone()).await?;

        db_bob.start_ticket_processing(None)?;

        const NUM_TICKETS: usize = 4;
        let (mut acked_tickets, mut channel) = populate_db_with_ack_tickets(db_bob.clone(), NUM_TICKETS).await?;

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1])?;

        // Make this ticket aggregated
        acked_tickets[0] = mock_acknowledged_ticket(&PEERS_CHAIN[0], &PEERS_CHAIN[1], 0, 2)?;

        debug!("upserting {}", acked_tickets[0]);
        db_bob.upsert_ticket(None, acked_tickets[0].clone()).await?;

        let tickets = db_bob.get_tickets((&channel).into()).await?;
        assert_eq!(tickets.len(), NUM_TICKETS, "nothing should be aggregated");

        channel.balance = HoprBalance::from(100_u32);

        db_alice.upsert_channel(None, channel).await?;
        db_bob.upsert_channel(None, channel).await?;

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: None,
            unrealized_balance_ratio: Some(0.75),
            aggregate_on_channel_close: false,
        };

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_bob.clone(), Arc::new(bob_aggregator));

        // let threshold_ticket = acked_tickets.last().unwrap();
        aggregation_strategy.on_tick().await?;

        let tickets = db_bob.get_tickets((&channel).into()).await?;
        assert_eq!(tickets.len(), NUM_TICKETS, "nothing should be aggregated");
        std::mem::drop(awaiter);

        Ok(())
    }

    #[tokio::test]
    async fn test_strategy_aggregation_on_channel_close() -> anyhow::Result<()> {
        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await?;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await?;

        init_db(db_alice.clone()).await?;
        init_db(db_bob.clone()).await?;

        let (bob_notify_tx, bob_notify_rx) = futures::channel::mpsc::unbounded();
        db_bob.start_ticket_processing(bob_notify_tx.into())?;

        let (_, mut channel) = populate_db_with_ack_tickets(db_bob.clone(), 5).await?;

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: Some(100),
            unrealized_balance_ratio: None,
            aggregate_on_channel_close: true,
        };

        channel.status = ChannelStatus::PendingToClose(std::time::SystemTime::now());

        db_alice.upsert_channel(None, channel).await?;
        db_bob.upsert_channel(None, channel).await?;

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1])?;

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_alice.clone(), Arc::new(bob_aggregator));

        aggregation_strategy
            .on_own_channel_changed(
                &channel,
                ChannelDirection::Incoming,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: ChannelStatus::PendingToClose(std::time::SystemTime::now()),
                },
            )
            .await?;

        // Wait until aggregation has finished
        timeout(Duration::from_secs(5), awaiter).await.context("Timeout")??;

        pin_mut!(bob_notify_rx);
        let notified_ticket = bob_notify_rx.next().await.expect("should have a ticket");

        let tickets = db_bob.get_tickets((&channel).into()).await?;
        assert_eq!(tickets.len(), 1, "there should be a single aggregated ticket");
        assert_eq!(notified_ticket, tickets[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_strategy_aggregation_on_tick_should_not_agg_on_channel_close_if_only_single_ticket()
    -> anyhow::Result<()> {
        // db_0: Alice (channel source)
        // db_1: Bob (channel destination)
        let db_alice = HoprDb::new_in_memory(PEERS_CHAIN[0].clone()).await?;
        let db_bob = HoprDb::new_in_memory(PEERS_CHAIN[1].clone()).await?;

        init_db(db_alice.clone()).await?;
        init_db(db_bob.clone()).await?;

        db_bob.start_ticket_processing(None)?;

        const NUM_TICKETS: usize = 1;
        let (_, channel) = populate_db_with_ack_tickets(db_bob.clone(), NUM_TICKETS).await?;

        let (bob_aggregator, awaiter) =
            spawn_aggregation_interaction(db_alice.clone(), db_bob.clone(), &PEERS_CHAIN[0], &PEERS_CHAIN[1])?;

        let tickets = db_bob.get_tickets((&channel).into()).await?;
        assert_eq!(tickets.len(), NUM_TICKETS, "should have a single ticket");

        db_alice.upsert_channel(None, channel).await?;
        db_bob.upsert_channel(None, channel).await?;

        let cfg = super::AggregatingStrategyConfig {
            aggregation_threshold: Some(100),
            unrealized_balance_ratio: Some(0.75),
            aggregate_on_channel_close: true,
        };

        let aggregation_strategy = super::AggregatingStrategy::new(cfg, db_bob.clone(), Arc::new(bob_aggregator));

        aggregation_strategy
            .on_own_channel_changed(
                &channel,
                ChannelDirection::Incoming,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: ChannelStatus::PendingToClose(std::time::SystemTime::now()),
                },
            )
            .await?;

        timeout(Duration::from_millis(500), awaiter)
            .await
            .expect_err("should timeout");

        let tickets = db_bob.get_tickets((&channel).into()).await?;
        assert_eq!(tickets.len(), NUM_TICKETS, "nothing should be aggregated");
        Ok(())
    }
}
