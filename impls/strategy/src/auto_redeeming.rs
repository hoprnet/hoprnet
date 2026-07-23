//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that
//! ticket.
//!
//! For details on default parameters, see [AutoRedeemingStrategyConfig].
use std::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use futures::{StreamExt, TryStreamExt};
use hopr_api::{
    chain::{ChainEvent, ChainReadChannelOperations, ChainWriteTicketOperations, ChannelSelector},
    node::{
        ActionableEvent, ActionableEventDiscriminant, ActionableEventSource, HasChainApi, HasTicketManagement,
        TicketEvent,
    },
    tickets::TicketManagement,
    types::{
        internal::prelude::{ChannelChange, ChannelDirection, ChannelEntry, ChannelId, ChannelStatus, VerifiedTicket},
        primitive::prelude::HoprBalance,
    },
};
use hopr_utils::runtime::prelude::AbortHandle;
use moka::notification::RemovalCause;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use validator::Validate;

use crate::{
    errors::{StrategyError, StrategyError::CriteriaNotSatisfied},
    just_false, just_true,
    strategy::Strategy as StrategyTrait,
};

/// Maximum time a single channel redemption run is allowed to take.
/// Exceeded entries are logged as errors and their tasks are aborted.
const REDEMPTION_TIMEOUT: Duration = Duration::from_secs(300);

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AUTO_REDEEMS:  hopr_api::types::telemetry::SimpleCounter =
         hopr_api::types::telemetry::SimpleCounter::new("hopr_strategy_auto_redeem_redeem_count", "Count of initiated automatic redemptions").unwrap();
}

fn min_redeem_hopr() -> HoprBalance {
    HoprBalance::from_str("1 wxHOPR").unwrap()
}

/// Configuration object for the `AutoRedeemingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AutoRedeemingStrategyConfig {
    /// If set to true, will redeem all tickets in the channel (which are over the
    /// `minimum_redeem_ticket_value` threshold) once it transitions to `PendingToClose`.
    ///
    /// Default is `true`.
    #[serde(default = "just_true")]
    #[default = true]
    pub redeem_all_on_close: bool,

    /// The strategy will only redeem an acknowledged winning ticket if it has at least this value of HOPR.
    /// If 0 is given, the strategy will redeem tickets regardless of their value.
    ///
    /// Default is `1 wxHOPR`.
    #[serde(default = "min_redeem_hopr")]
    #[serde_as(as = "DisplayFromStr")]
    #[default(min_redeem_hopr())]
    pub minimum_redeem_ticket_value: HoprBalance,

    /// If set, the strategy will redeem each incoming winning ticket.
    /// Otherwise, it will try to redeem tickets in all channels periodically.
    ///
    /// Set this to `true` when winning tickets are not happening too often (e.g., when winning probability
    /// is below 1%).
    /// Set this to `false` when winning tickets are happening very often (e.g., when winning probability
    /// is above 1%).
    ///
    /// Default is `true`
    #[serde(default = "just_false")]
    #[default = false]
    pub redeem_on_winning: bool,
}

/// Builder for [`AutoRedeemingStrategy`].
///
/// Call [`new`](AutoRedeemingStrategy::new) with the strategy configuration,
/// then [`build`](AutoRedeemingStrategy::build) to wire in a node and obtain a
/// runnable `Box<dyn Strategy + Send>`.
pub struct AutoRedeemingStrategy {
    cfg: AutoRedeemingStrategyConfig,
    interval: Duration,
}

impl AutoRedeemingStrategy {
    /// Create a new builder with the given configuration.
    pub fn new(cfg: AutoRedeemingStrategyConfig, interval: Duration) -> Self {
        Self { cfg, interval }
    }

    /// Wire in a node and return a running-ready strategy.
    ///
    /// The generic `N` is erased at construction time; the returned
    /// `Box<dyn Strategy + Send>` can be held and spawned without knowledge
    /// of the concrete node type.
    pub fn build<N>(self, node: Arc<N>) -> Box<dyn StrategyTrait + Send>
    where
        N: HasChainApi + HasTicketManagement + ActionableEventSource + Send + Sync + 'static,
        N::ChainApi: ChainReadChannelOperations + ChainWriteTicketOperations + Clone + Send + Sync + 'static,
        N::TicketManager: TicketManagement + Clone + Send + Sync + 'static,
    {
        Box::new(AutoRedeemingStrategyInner {
            cfg: self.cfg,
            interval: self.interval,
            node,
            running_redemptions: new_redemption_cache(),
        })
    }
}

/// Construct the moka cache used to track in-progress per-channel redemptions.
fn new_redemption_cache() -> moka::sync::Cache<ChannelId, AbortHandle> {
    moka::sync::CacheBuilder::new(1024)
        .time_to_live(REDEMPTION_TIMEOUT)
        .eviction_listener(|key: Arc<ChannelId>, value: AbortHandle, cause| {
            match cause {
                RemovalCause::Expired => {
                    // TTL elapsed — log an error and abort the stalled redemption task.
                    tracing::error!(%key, "redemption timed out after {:?}; aborting", REDEMPTION_TIMEOUT);
                    value.abort();
                }
                RemovalCause::Size => {
                    // Cache capacity exceeded — let the task run to natural completion
                    // rather than aborting work that is likely still making progress.
                    tracing::warn!(%key, "redemption cache at capacity; entry evicted without abort");
                }
                _ => {
                    // Explicit invalidation (task completed) or replacement.
                    // The abortable future is already resolved; abort() is a no-op.
                    value.abort();
                }
            }
        })
        .build()
}

/// Private generic runner — constructed by [`AutoRedeemingStrategy::build`].
///
/// The strategy does not await the result of the redemption.
struct AutoRedeemingStrategyInner<N: HasChainApi + HasTicketManagement> {
    node: Arc<N>,
    cfg: AutoRedeemingStrategyConfig,
    interval: Duration,
    /// Bookkeeping for in-progress per-channel redemptions.
    ///
    /// Entries are removed when a redemption completes naturally (via [`moka::sync::Cache::invalidate`]).
    /// Entries that exceed [`REDEMPTION_TIMEOUT`] are evicted by the cache; the eviction listener
    /// logs an error and aborts the background task.
    running_redemptions: moka::sync::Cache<ChannelId, AbortHandle>,
}

impl<N: HasChainApi + HasTicketManagement> Drop for AutoRedeemingStrategyInner<N> {
    fn drop(&mut self) {
        // Moka does not invoke the eviction listener when the cache is dropped.
        // Abort all in-flight redemption tasks manually to prevent runaway background work.
        self.running_redemptions.iter().for_each(|(_, handle)| handle.abort());
    }
}

impl<N> AutoRedeemingStrategyInner<N>
where
    N: HasChainApi + HasTicketManagement + ActionableEventSource + Send + Sync + 'static,
    <N as HasChainApi>::ChainApi:
        ChainReadChannelOperations + ChainWriteTicketOperations + Clone + Send + Sync + 'static,
    <N as HasTicketManagement>::TicketManager: TicketManagement + Clone + Send + Sync + 'static,
{
    fn enqueue_redemption(&self, channel_id: &ChannelId) -> Result<(), StrategyError> {
        if self.running_redemptions.contains_key(channel_id) {
            tracing::debug!(%channel_id, "existing on-going redemption");
            return Err(StrategyError::InProgress);
        }

        tracing::debug!(%channel_id, "attempting to start redemption in channel");

        let tmgr = self.node.ticket_management().clone();
        let client = self.node.chain_api().clone();
        let min_value = self.cfg.minimum_redeem_ticket_value;
        let channel_id = *channel_id;
        let redemptions = self.running_redemptions.clone();

        let abort_handle = hopr_utils::spawn_as_abortable!(async move {
            let redeem_result = match tmgr
                .redeem_stream(client.clone(), channel_id, min_value.into())
                .map_err(StrategyError::other)
            {
                Ok(stream) => {
                    stream
                        .map_err(StrategyError::other)
                        .try_for_each(|res| {
                            tracing::debug!(?res, %channel_id, "ticket redemption completed");
                            futures::future::ok(())
                        })
                        .await
                }
                err => err.map(|_| ()),
            };

            tracing::debug!(?redeem_result, %channel_id, "redemption in channel complete");
            // Remove the entry so the cache does not eventually time it out.
            redemptions.invalidate(&channel_id);
        });

        self.running_redemptions.insert(channel_id, abort_handle);
        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_COUNT_AUTO_REDEEMS.increment();
        Ok(())
    }

    /// Handle an acknowledged winning ticket. Called from `run()` on `TicketEvent::WinningTicket`.
    async fn on_acknowledged_winning_ticket(&self, ack: &VerifiedTicket) -> crate::errors::Result<()> {
        if self.cfg.redeem_on_winning && ack.verified_ticket().amount.ge(&self.cfg.minimum_redeem_ticket_value) {
            let chain_api = self.node.chain_api().clone();
            let channel_id = *ack.channel_id();
            let maybe_channel = hopr_utils::runtime::prelude::spawn_blocking(move || {
                chain_api.channel_by_id(&channel_id).map_err(StrategyError::other)
            })
            .await
            .map_err(StrategyError::other)??;

            if let Some(channel) = maybe_channel {
                tracing::info!(%ack, "redeeming");

                if ack.verified_ticket().index < channel.ticket_index {
                    tracing::error!(%ack, "acknowledged ticket is older than channel ticket index");
                    return Err(CriteriaNotSatisfied);
                }

                self.enqueue_redemption(channel.get_id())?;
                Ok(())
            } else {
                Err(CriteriaNotSatisfied)
            }
        } else {
            Err(CriteriaNotSatisfied)
        }
    }

    /// Handle a channel status change. Called from `run()` on `ChainEvent::ChannelClosureInitiated`.
    async fn on_own_channel_changed(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        change: ChannelChange,
    ) -> crate::errors::Result<()> {
        if direction != ChannelDirection::Incoming || !self.cfg.redeem_all_on_close {
            return Ok(());
        }

        if let ChannelChange::Status { left: old, right: new } = change {
            if old != ChannelStatus::Open || !matches!(new, ChannelStatus::PendingToClose(_)) {
                tracing::debug!(?channel, "ignoring channel state change that's not in PendingToClose");
                return Ok(());
            }
            tracing::info!(%channel, "channel transitioned to PendingToClose, checking if it has tickets to redeem");
            self.enqueue_redemption(channel.get_id())?;
            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }

    /// Periodic scan: redeem tickets in all eligible channels.
    async fn on_tick(&self) -> crate::errors::Result<()> {
        if !self.cfg.redeem_on_winning {
            tracing::debug!("trying to redeem all tickets in all channels");

            let chain = self.node.chain_api();
            chain
                .stream_channels(
                    ChannelSelector::default()
                        .with_destination(*chain.me())
                        .with_redeemable_channels(Duration::from_secs(30).into()),
                )
                .map_err(StrategyError::other)?
                .for_each(|channel| {
                    if let Err(error) = self.enqueue_redemption(channel.get_id()) {
                        tracing::error!(
                            %error,
                            channel_id = %channel.get_id(),
                            "cannot start redemption in channel"
                        );
                    }
                    futures::future::ready(())
                })
                .await;

            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }
}

impl<N: HasChainApi + HasTicketManagement> Debug for AutoRedeemingStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AutoRedeemingStrategy({:?})", self.cfg)
    }
}

impl<N: HasChainApi + HasTicketManagement> Display for AutoRedeemingStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "auto_redeeming")
    }
}

#[async_trait::async_trait]
impl<N> StrategyTrait for AutoRedeemingStrategyInner<N>
where
    N: HasChainApi + HasTicketManagement + ActionableEventSource + Send + Sync + 'static,
    <N as HasChainApi>::ChainApi:
        ChainReadChannelOperations + ChainWriteTicketOperations + Clone + Send + Sync + 'static,
    <N as HasTicketManagement>::TicketManager: TicketManagement + Clone + Send + Sync + 'static,
{
    async fn run(&mut self) -> crate::errors::Result<()> {
        enum Event {
            Tick,
            Actionable(Box<ActionableEvent>),
        }

        // Run the first scan immediately at startup without waiting for the initial interval.
        if let Err(e) = self.on_tick().await
            && !matches!(e, StrategyError::CriteriaNotSatisfied)
        {
            tracing::error!(%e, "auto-redeeming tick failed");
        }

        let tick_stream = futures_time::stream::interval(self.interval.into()).map(|_| Event::Tick);
        let event_stream = self
            .node
            .subscribe_to_actionable_events(Some(&[
                ActionableEventDiscriminant::Chain,
                ActionableEventDiscriminant::Ticket,
            ]))
            .map_err(|e| StrategyError::Other(anyhow::anyhow!(e)))?
            .map(|e| Event::Actionable(Box::new(e)));

        let mut combined = futures_concurrency::stream::Merge::merge((tick_stream, event_stream));
        let me = *self.node.chain_api().me();

        while let Some(event) = combined.next().await {
            match event {
                Event::Tick => {
                    if let Err(e) = self.on_tick().await
                        && !matches!(e, StrategyError::CriteriaNotSatisfied)
                    {
                        tracing::error!(%e, "auto-redeeming tick failed");
                    }
                }
                Event::Actionable(ev) => match *ev {
                    ActionableEvent::Ticket(TicketEvent::WinningTicket(rt)) => {
                        if let Err(e) = self.on_acknowledged_winning_ticket(&rt.ticket).await
                            && !matches!(e, StrategyError::CriteriaNotSatisfied)
                        {
                            tracing::error!(%e, "auto-redeeming failed on winning ticket");
                        }
                    }
                    ActionableEvent::Chain(ChainEvent::ChannelClosureInitiated(channel)) => {
                        if let Some(dir) = channel.direction(&me)
                            && let Err(e) = self
                                .on_own_channel_changed(
                                    &channel,
                                    dir,
                                    ChannelChange::Status {
                                        left: ChannelStatus::Open,
                                        right: channel.status,
                                    },
                                )
                                .await
                            && !matches!(e, StrategyError::CriteriaNotSatisfied | StrategyError::InProgress)
                        {
                            tracing::error!(%e, "auto-redeeming failed on channel closure");
                        }
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Add,
        sync::Arc,
        time::{Duration, SystemTime},
    };

    use futures::stream::BoxStream;
    use futures_time::future::FutureExt as TimeExt;
    use hex_literal::hex;
    use hopr_api::{
        chain::{ChainEvent, ChainEvents, ChainWriteTicketOperations, HoprChainApi},
        node::{
            ActionableEvent, ComponentStatus, ComponentStatusReporter, EventWaitResult, HasChainApi,
            HasTicketManagement, NodeOnchainIdentity, TicketEvent,
        },
        tickets::{ChannelStats, RedemptionResult},
        types::{
            crypto::{
                keypairs::Keypair,
                prelude::{ChainKeypair, HalfKey, Hash, Response},
            },
            crypto_random::Randomizable,
            internal::prelude::{RedeemableTicket, TicketBuilder, WinningProbability},
            primitive::prelude::{Address, BytesRepresentable, HoprBalance, UnitaryFloatOps, XDaiBalance},
        },
    };
    use hopr_chain_connector::{HoprBlockchainSafeConnector, create_trustful_hopr_blokli_connector, testing::*};

    use super::*;

    mockall::mock! {
        pub TicketMgmt {}
         #[allow(refining_impl_trait)]
        impl TicketManagement for TicketMgmt {
            type Error = std::io::Error;
            fn redeem_stream<C: ChainWriteTicketOperations + Send + Sync + 'static>(
                &self,
                client: C,
                channel_id: ChannelId,
                min_amount: Option<HoprBalance>,
            ) -> Result<BoxStream<'static, Result<RedemptionResult, std::io::Error>>, std::io::Error>;

            fn neglect_tickets(
                &self,
                channel_id: &ChannelId,
                max_ticket_index: Option<u64>,
            ) -> Result<Vec<VerifiedTicket>, std::io::Error>;

            fn ticket_stats<'a>(&self, channel_id: Option<&'a ChannelId>) -> Result<ChannelStats, std::io::Error>;

            fn insert_incoming_ticket(
                &self,
                ticket: hopr_api::types::internal::prelude::RedeemableTicket,
            ) -> Result<Vec<VerifiedTicket>, std::io::Error>;
        }
    }

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
        static ref CHARLIE: ChainKeypair = ChainKeypair::from_secret(&hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26")).expect("lazy static keypair should be constructible");
        static ref PRICE_PER_PACKET: HoprBalance = 10000000000000000_u128.into(); // 0.01 HOPR

        static ref CHANNEL_1: ChannelEntry = ChannelEntry::builder()
            .between(&*ALICE, &*BOB)
            .balance(*PRICE_PER_PACKET * 10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(4)
            .build()
            .unwrap();

        static ref CHANNEL_2: ChannelEntry = ChannelEntry::builder()
            .between(&*CHARLIE, &*BOB)
            .balance(*PRICE_PER_PACKET * 11)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(4)
            .build()
            .unwrap();

        static ref CHAIN_CLIENT: BlokliTestClient<StaticState> = BlokliTestStateBuilder::default()
            .with_generated_accounts(&[ALICE.public().as_ref(), BOB.public().as_ref(), CHARLIE.public().as_ref()], false, XDaiBalance::new_base(1), HoprBalance::new_base(1000))
            .with_channels([*CHANNEL_1, *CHANNEL_2])
            .build_static_client();
    }

    type TestConnector = Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>;

    fn generate_random_ack_ticket(index: u64, worth_packets: u32) -> anyhow::Result<RedeemableTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        Ok(TicketBuilder::default()
            .counterparty(&*BOB)
            .amount(PRICE_PER_PACKET.div_f64(1.0f64)?.amount() * worth_packets)
            .index(index)
            .win_prob(WinningProbability::ALWAYS)
            .channel_epoch(4)
            .challenge(challenge)
            .build_signed(&ALICE, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?)
            .into_redeemable(&BOB, &Hash::default())?)
    }

    /// Minimal node wrapper combining a chain connector and a ticket manager for tests.
    struct TestNode<C, T> {
        chain: C,
        tmgr: T,
    }

    impl<C, T> HasChainApi for TestNode<C, T>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        type ChainApi = C;
        type ChainError = <C as HoprChainApi>::ChainError;

        fn identity(&self) -> &NodeOnchainIdentity {
            static IDENTITY: std::sync::OnceLock<NodeOnchainIdentity> = std::sync::OnceLock::new();
            IDENTITY.get_or_init(NodeOnchainIdentity::default)
        }

        fn chain_api(&self) -> &C {
            &self.chain
        }

        fn status(&self) -> ComponentStatus {
            self.chain.component_status()
        }

        fn wait_for_on_chain_event<F>(
            &self,
            _predicate: F,
            _context: String,
            _timeout: std::time::Duration,
        ) -> EventWaitResult<<C as HoprChainApi>::ChainError, <C as HoprChainApi>::ChainError>
        where
            F: Fn(&ChainEvent) -> bool + Send + Sync + 'static,
        {
            unimplemented!("tests do not call wait_for_on_chain_event")
        }
    }

    impl<C, T> HasTicketManagement for TestNode<C, T>
    where
        C: Send + Sync + 'static,
        T: TicketManagement + Clone + Send + Sync + 'static,
    {
        type TicketManager = T;

        fn ticket_management(&self) -> &T {
            &self.tmgr
        }

        fn subscribe_ticket_events(&self) -> impl futures::Stream<Item = TicketEvent> + Send + 'static {
            futures::stream::empty()
        }

        fn status(&self) -> ComponentStatus {
            ComponentStatus::Ready
        }
    }

    impl<C, T> ActionableEventSource for TestNode<C, T>
    where
        C: ChainEvents + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        fn subscribe_to_actionable_events(
            &self,
            _filter: Option<&[ActionableEventDiscriminant]>,
        ) -> Result<futures::stream::BoxStream<'static, ActionableEvent>, String> {
            Ok(self
                .chain
                .subscribe()
                .map_err(|e| e.to_string())?
                .map(ActionableEvent::Chain)
                .boxed())
        }
    }

    async fn await_redemption_queue_empty(redeems: moka::sync::Cache<ChannelId, AbortHandle>) {
        loop {
            hopr_utils::runtime::prelude::sleep(Duration::from_millis(100)).await;
            redeems.run_pending_tasks();
            if redeems.entry_count() == 0 {
                break;
            }
        }
    }

    fn make_strategy<T: TicketManagement + Clone + Send + Sync + 'static>(
        cfg: AutoRedeemingStrategyConfig,
        connector: TestConnector,
        tmgr: T,
    ) -> AutoRedeemingStrategyInner<TestNode<TestConnector, T>> {
        AutoRedeemingStrategyInner {
            cfg,
            interval: Duration::from_secs(60),
            node: Arc::new(TestNode { chain: connector, tmgr }),
            running_redemptions: new_redemption_cache(),
        }
    }

    #[test_log::test(tokio::test)]
    async fn test_auto_redeeming_strategy_redeem() -> anyhow::Result<()> {
        let ack_ticket = generate_random_ack_ticket(0, 5)?;

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(move |_: TestConnector, _, _| {
                Ok(futures::stream::once(futures::future::ok(RedemptionResult::Redeemed(ack_ticket.ticket))).boxed())
            });

        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(mock_tmgr));

        ars.on_acknowledged_winning_ticket(&ack_ticket.ticket).await?;
        assert!(ars.on_tick().await.is_err());

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_auto_redeeming_strategy_redeem_on_tick() -> anyhow::Result<()> {
        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_on_winning: false,
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(|_: TestConnector, _, _| Ok(futures::stream::empty().boxed()));

        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_2.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(|_: TestConnector, _, _| Ok(futures::stream::empty().boxed()));

        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(mock_tmgr));
        ars.on_tick().await?;

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_should_redeem_singular_ticket_on_close() -> anyhow::Result<()> {
        let mut channel = *CHANNEL_1;
        channel.status = ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100)));

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_all_on_close: true,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(move |_: TestConnector, _, _| Ok(futures::stream::empty().boxed()));

        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(mock_tmgr));
        ars.on_own_channel_changed(
            &channel,
            ChannelDirection::Incoming,
            ChannelChange::Status {
                left: ChannelStatus::Open,
                right: channel.status,
            },
        )
        .await?;

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_multiple_times_in_same_channel() -> anyhow::Result<()> {
        let ack_ticket_1 = generate_random_ack_ticket(0, 5)?;

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(move |_: TestConnector, _, _| {
                Ok(futures::stream::once(
                    futures::future::ok(RedemptionResult::Redeemed(ack_ticket_1.ticket))
                        .delay(futures_time::time::Duration::from_millis(500)),
                )
                .boxed())
            });

        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(mock_tmgr));
        ars.on_acknowledged_winning_ticket(&ack_ticket_1.ticket).await?;
        assert!(matches!(
            ars.on_acknowledged_winning_ticket(&ack_ticket_1.ticket).await,
            Err(StrategyError::InProgress)
        ));

        let mut channel = *CHANNEL_1;
        channel.status = ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100)));

        assert!(matches!(
            ars.on_own_channel_changed(
                &channel,
                ChannelDirection::Incoming,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: channel.status,
                }
            )
            .await,
            Err(StrategyError::InProgress)
        ));
        assert!(ars.on_tick().await.is_err());

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_on_tick_returns_criteria_not_satisfied_when_redeem_on_winning_true() -> anyhow::Result<()> {
        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_on_winning: true,
            ..Default::default()
        };
        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(MockTicketMgmt::new()));

        assert!(matches!(ars.on_tick().await, Err(StrategyError::CriteriaNotSatisfied)));
        Ok(())
    }

    #[tokio::test]
    async fn test_on_acknowledged_winning_ticket_skips_when_redeem_on_winning_false() -> anyhow::Result<()> {
        let ack_ticket = generate_random_ack_ticket(0, 5)?;

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_on_winning: false,
            ..Default::default()
        };
        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(MockTicketMgmt::new()));

        assert!(matches!(
            ars.on_acknowledged_winning_ticket(&ack_ticket.ticket).await,
            Err(StrategyError::CriteriaNotSatisfied)
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_on_acknowledged_winning_ticket_skips_below_minimum_value() -> anyhow::Result<()> {
        // Ticket worth 1 packet, but minimum is 10 packets.
        let ack_ticket = generate_random_ack_ticket(0, 1)?;

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_on_winning: true,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 10),
            ..Default::default()
        };
        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(MockTicketMgmt::new()));

        assert!(matches!(
            ars.on_acknowledged_winning_ticket(&ack_ticket.ticket).await,
            Err(StrategyError::CriteriaNotSatisfied)
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_on_own_channel_changed_skips_outgoing_direction() -> anyhow::Result<()> {
        let pending_status = ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100)));

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_all_on_close: true,
            ..Default::default()
        };
        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(MockTicketMgmt::new()));

        assert!(
            ars.on_own_channel_changed(
                &CHANNEL_1,
                ChannelDirection::Outgoing,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: pending_status
                },
            )
            .await
            .is_ok()
        );
        assert_eq!(ars.running_redemptions.entry_count(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_on_own_channel_changed_skips_when_redeem_all_on_close_false() -> anyhow::Result<()> {
        let pending_status = ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100)));

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_all_on_close: false,
            ..Default::default()
        };
        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(MockTicketMgmt::new()));

        assert!(
            ars.on_own_channel_changed(
                &CHANNEL_1,
                ChannelDirection::Incoming,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: pending_status
                },
            )
            .await
            .is_ok()
        );
        assert_eq!(ars.running_redemptions.entry_count(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_on_own_channel_changed_returns_error_for_non_status_change() -> anyhow::Result<()> {
        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_all_on_close: true,
            ..Default::default()
        };
        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(MockTicketMgmt::new()));

        assert!(matches!(
            ars.on_own_channel_changed(
                &CHANNEL_1,
                ChannelDirection::Incoming,
                ChannelChange::Balance {
                    left: 5.into(),
                    right: 3.into()
                },
            )
            .await,
            Err(StrategyError::CriteriaNotSatisfied)
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_on_own_channel_changed_skips_non_open_to_pending_transition() -> anyhow::Result<()> {
        let pending_status = ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100)));

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_all_on_close: true,
            ..Default::default()
        };
        let ars = make_strategy(cfg, Arc::new(connector), Arc::new(MockTicketMgmt::new()));

        // Transition from PendingToClose → something (not from Open), should be ignored.
        assert!(
            ars.on_own_channel_changed(
                &CHANNEL_1,
                ChannelDirection::Incoming,
                ChannelChange::Status {
                    left: pending_status,
                    right: ChannelStatus::Closed
                },
            )
            .await
            .is_ok()
        );
        assert_eq!(ars.running_redemptions.entry_count(), 0);
        Ok(())
    }

    /// Tests the public builder API: `AutoRedeemingStrategy::new(...).build(node)` must
    /// return a `Box<dyn Strategy + Send>` with the expected Display string.
    #[tokio::test]
    async fn test_build_returns_strategy_trait_object() -> anyhow::Result<()> {
        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let mock_tmgr = MockTicketMgmt::new();
        let node = Arc::new(TestNode {
            chain: Arc::new(connector),
            tmgr: Arc::new(mock_tmgr),
        });

        let strategy: Box<dyn crate::strategy::Strategy + Send> =
            super::AutoRedeemingStrategy::new(AutoRedeemingStrategyConfig::default(), Duration::from_secs(60))
                .build(node);

        assert_eq!(strategy.to_string(), "auto_redeeming");
        // Verify the box is Send (compile-time check via trait object)
        fn assert_send<T: Send>(_: T) {}
        assert_send(strategy);

        Ok(())
    }
}
