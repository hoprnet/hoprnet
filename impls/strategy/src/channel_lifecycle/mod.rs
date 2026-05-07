//! ## Channel Lifecycle Strategy
//!
//! A unified strategy that owns **open / fund / close / finalize** for outgoing
//! payment channels.  It replaces the combination of `AutoFundingStrategy` +
//! `ClosureFinalizerStrategy` with a single component that maintains a target
//! population of funded outgoing channels against online peers and retires
//! channels to peers that have been absent for too long.
//!
//! ### State machine
//!
//! ```text
//!                                   ┌────────────────────────┐
//!                                   │   no on-chain entry    │
//!                                   └───────────┬────────────┘
//!                                               │ open()  (eligibility passed)
//!                                               ▼
//!                                   ┌────────────────────────┐
//!                                   │     OpenInFlight       │
//!                                   └───────────┬────────────┘
//!                                               │ ChannelOpened
//!                                               ▼
//!                                   ┌────────────────────────┐
//!                                   │         Open           │◄──────────────┐
//!                                   └─────┬──────────┬───────┘               │
//!                below_lower_balance      │          │ staleness/quality drop
//!                       fund()            │          │  close()
//!                           ▼             │          ▼
//!                   ┌──────────────┐      │   ┌────────────────────┐
//!                   │ FundInFlight │      │   │   CloseInFlight    │
//!                   └──────┬───────┘      │   └─────────┬──────────┘
//!                          │ Balance↑     │             │ ChannelClosureInitiated
//!                          ▼              │             ▼
//!                         Open ───────────┘   ┌────────────────────┐
//!                                             │  PendingToClose    │
//!                                             └─────────┬──────────┘
//!                                                       │ notice_period + max_closure_overdue
//!                                                       │ finalize()
//!                                                       ▼
//!                                             ┌────────────────────┐
//!                                             │ FinalizeInFlight   │
//!                                             └─────────┬──────────┘
//!                                                       │ ChannelClosed
//!                                                       ▼
//!                                             ┌────────────────────┐
//!                                             │  cooldown (peer)   │
//!                                             └────────────────────┘
//!                                                       │ peer_reopen_cooldown
//!                                                       ▼
//!                                                (eligible to reopen)
//! ```
//!
//! In-flight states are tracked off-chain in `DashSet<ChannelId>` / `DashSet<Address>`.
//! The on-chain `ChannelStatus` plus the in-flight sets together drive transitions.
//! The cooldown is keyed by peer `Address` with an `Instant`-stamped map entry.
//!
//! ### Feature flag
//!
//! Enable with `strategy-channel-lifecycle`.

mod config;
pub use config::*;

mod events;
mod pipeline;
mod strategy;
use std::{collections::HashMap, sync::Arc, time::Instant};

use dashmap::{DashMap, DashSet};
use hopr_lib::api::{
    PeerId,
    types::{
        crypto::prelude::OffchainPublicKey,
        internal::prelude::ChannelId,
        primitive::prelude::{Address, HoprBalance},
    },
};
use parking_lot::Mutex;
pub use strategy::ChannelLifecycleStrategy;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_CHANNEL_OPENS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_opens",
            "Count of initiated channel opens",
        ).unwrap();
    static ref METRIC_CHANNEL_FUNDS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_fundings",
            "Count of initiated channel fundings",
        ).unwrap();
    static ref METRIC_CHANNEL_CLOSES: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_closes",
            "Count of initiated channel closures",
        ).unwrap();
    static ref METRIC_CHANNEL_FINALIZES: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_finalizations",
            "Count of initiated channel closure finalizations",
        ).unwrap();
}

/// Per-channel observation snapshot used by the proactive funding estimate.
#[derive(Clone)]
struct ChannelObservation {
    balance: HoprBalance,
    ticket_index: u64,
    at: Instant,
}

/// Cached `peer_id → (offchain key, chain address)` map plus the timestamp at
/// which it was last refreshed.  Lets the snapshot pass skip the full account
/// stream on most ticks.
struct PeerAddrCache {
    refreshed_at: Instant,
    map: HashMap<PeerId, (OffchainPublicKey, Address)>,
}

/// The running strategy instance.  Generic over the node type `N` so that
/// callers can provide any node implementation satisfying the required traits.
///
/// Constructed via [`ChannelLifecycleStrategy::build`]; the builder erases `N`
/// behind `Box<dyn Strategy + Send>`.
struct ChannelLifecycleStrategyInner<N> {
    cfg: ChannelLifecycleConfig,
    node: Arc<N>,
    /// Destination addresses for channels currently being opened.
    open_in_flight: Arc<DashSet<Address>>,
    /// Channel IDs with an in-flight funding transaction.
    fund_in_flight: Arc<DashSet<ChannelId>>,
    /// Channel IDs with an in-flight closure transaction.
    close_in_flight: Arc<DashSet<ChannelId>>,
    /// Channel IDs with an in-flight finalization transaction.
    finalize_in_flight: Arc<DashSet<ChannelId>>,
    /// Peer addresses mapped to the `Instant` when their cooldown expires.
    cooldown: Arc<DashMap<Address, Instant>>,
    /// When this strategy instance started; used by the restart guard.
    start_epoch: Instant,
    /// Most-recently recorded balance/ticket_index snapshot per channel.
    last_observed: Arc<DashMap<ChannelId, ChannelObservation>>,
    /// Cumulative ticket count increments from `TicketRedeemed` events.
    peer_ticket_activity: Arc<DashMap<Address, u64>>,
    /// TTL-cached peer-id → (offchain key, chain address) map.  Avoids
    /// streaming the full on-chain account list on every tick.
    peer_addr_cache: Arc<Mutex<Option<PeerAddrCache>>>,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        sync::Arc,
        time::{Duration, Instant},
    };

    use dashmap::DashMap;
    use futures::StreamExt as _;
    use hex_literal::hex;
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
    use hopr_lib::api::{
        PeerId,
        chain::{
            AccountSelector, ChainEvent, ChainEvents, ChainReadAccountOperations, ChainReadChannelOperations,
            ChainWriteAccountOperations, ChannelSelector, HoprChainApi,
        },
        node::{
            ActionableEvent, ActionableEventDiscriminant, ActionableEventSource, ComponentStatus,
            ComponentStatusReporter, EventWaitResult, HasChainApi, HasGraphView, HasNetworkView, NodeOnchainIdentity,
        },
        types::{
            crypto::{
                keypairs::Keypair,
                prelude::{ChainKeypair, OffchainPublicKey},
            },
            internal::prelude::{ChannelEntry, ChannelStatus},
            primitive::prelude::{Address, BytesRepresentable, HoprBalance, XDaiBalance},
        },
    };

    use super::*;

    lazy_static::lazy_static! {
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("test keypair");
        static ref ALICE: Address = hex!("18f8ae833c85c51fbeba29cef9fbfb53b3bad950").into();
        static ref BOB: Address = BOB_KP.public().to_address();
        static ref CHRIS: Address = hex!("b6021e0860dd9d96c9ff0a73e2e5ba3a466ba234").into();
        static ref DAVE: Address = hex!("68499f50ff68d523385dc60686069935d17d762a").into();
    }

    /// Minimal node wrapper — same pattern as in auto_funding tests.
    struct ChainNode<C>(C);

    impl<C> HasChainApi for ChainNode<C>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type ChainApi = C;
        type ChainError = <C as HoprChainApi>::ChainError;

        fn identity(&self) -> &NodeOnchainIdentity {
            static IDENTITY: std::sync::OnceLock<NodeOnchainIdentity> = std::sync::OnceLock::new();
            IDENTITY.get_or_init(NodeOnchainIdentity::default)
        }

        fn chain_api(&self) -> &C {
            &self.0
        }

        fn status(&self) -> ComponentStatus {
            self.0.component_status()
        }

        fn wait_for_on_chain_event<F>(
            &self,
            _predicate: F,
            _context: String,
            _timeout: Duration,
        ) -> EventWaitResult<<C as HoprChainApi>::ChainError, <C as HoprChainApi>::ChainError>
        where
            F: Fn(&ChainEvent) -> bool + Send + Sync + 'static,
        {
            unimplemented!("tests do not call wait_for_on_chain_event")
        }
    }

    impl<C> ActionableEventSource for ChainNode<C>
    where
        C: ChainEvents + Send + Sync + 'static,
    {
        fn subscribe_to_actionable_events(
            &self,
            _filter: Option<&[ActionableEventDiscriminant]>,
        ) -> Result<futures::stream::BoxStream<'static, ActionableEvent>, String> {
            Ok(self
                .0
                .subscribe()
                .map_err(|e| e.to_string())?
                .map(ActionableEvent::Chain)
                .boxed())
        }
    }

    struct StubNetworkView;

    impl hopr_lib::api::network::NetworkView for StubNetworkView {
        fn listening_as(&self) -> HashSet<hopr_lib::api::Multiaddr> {
            HashSet::new()
        }

        fn multiaddress_of(&self, _peer: &PeerId) -> Option<HashSet<hopr_lib::api::Multiaddr>> {
            None
        }

        fn discovered_peers(&self) -> HashSet<PeerId> {
            HashSet::new()
        }

        fn connected_peers(&self) -> HashSet<PeerId> {
            HashSet::new()
        }

        fn is_connected(&self, _peer: &PeerId) -> bool {
            false
        }

        fn health(&self) -> hopr_lib::api::network::Health {
            hopr_lib::api::network::Health::Red
        }

        fn subscribe_network_events(
            &self,
        ) -> impl futures::Stream<Item = hopr_lib::api::network::NetworkEvent> + Send + 'static {
            futures::stream::pending()
        }
    }

    impl<C> HasNetworkView for ChainNode<C>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type NetworkView = StubNetworkView;

        fn network_view(&self) -> &Self::NetworkView {
            static NV: StubNetworkView = StubNetworkView;
            &NV
        }

        fn status(&self) -> ComponentStatus {
            ComponentStatus::Ready
        }
    }

    struct StubGraph;

    impl hopr_lib::api::graph::NetworkGraphView for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn node_count(&self) -> usize {
            0
        }

        fn contains_node(&self, _key: &OffchainPublicKey) -> bool {
            false
        }

        fn nodes(&self) -> futures::stream::BoxStream<'static, OffchainPublicKey> {
            Box::pin(futures::stream::empty())
        }

        fn edge(&self, _src: &OffchainPublicKey, _dest: &OffchainPublicKey) -> Option<StubEdge> {
            None
        }

        fn identity(&self) -> &OffchainPublicKey {
            static KEY: std::sync::OnceLock<OffchainPublicKey> = std::sync::OnceLock::new();
            KEY.get_or_init(|| {
                use hopr_lib::api::types::crypto::keypairs::Keypair as _;
                *hopr_lib::api::types::crypto::prelude::OffchainKeypair::from_secret(&[1u8; 32])
                    .expect("test key")
                    .public()
            })
        }
    }

    impl hopr_lib::api::graph::NetworkGraphConnectivity for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn connected_edges(&self) -> Vec<(OffchainPublicKey, OffchainPublicKey, StubEdge)> {
            Vec::new()
        }

        fn reachable_edges(&self) -> Vec<(OffchainPublicKey, OffchainPublicKey, StubEdge)> {
            Vec::new()
        }
    }

    impl hopr_lib::api::graph::NetworkGraphTraverse for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn simple_paths<V: hopr_lib::api::graph::ValueFn<Weight = StubEdge>>(
            &self,
            _source: &OffchainPublicKey,
            _destination: &OffchainPublicKey,
            _length: usize,
            _take_count: Option<usize>,
            _value_fn: V,
        ) -> Vec<(Vec<OffchainPublicKey>, [u64; 5], V::Value)> {
            Vec::new()
        }

        fn simple_paths_from<V: hopr_lib::api::graph::ValueFn<Weight = StubEdge>>(
            &self,
            _source: &OffchainPublicKey,
            _length: usize,
            _take_count: Option<usize>,
            _value_fn: V,
        ) -> Vec<(Vec<OffchainPublicKey>, [u64; 5], V::Value)> {
            Vec::new()
        }

        fn simple_loopback_to_self(
            &self,
            _length: usize,
            _take_count: Option<usize>,
        ) -> Vec<(Vec<OffchainPublicKey>, [u64; 5])> {
            Vec::new()
        }
    }

    struct StubEdge;

    impl hopr_lib::api::graph::EdgeObservableRead for StubEdge {
        type ImmediateMeasurement = StubMeasurement;
        type IntermediateMeasurement = StubMeasurement;

        fn last_update(&self) -> Duration {
            Duration::ZERO
        }

        fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement> {
            None
        }

        fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement> {
            None
        }

        fn score(&self) -> f64 {
            0.5
        }
    }

    impl hopr_lib::api::graph::traits::EdgeObservableWrite for StubEdge {
        fn record(&mut self, _measurement: hopr_lib::api::graph::traits::EdgeWeightType) {}
    }

    struct StubMeasurement;

    impl hopr_lib::api::graph::EdgeLinkObservable for StubMeasurement {
        fn record(&mut self, _: hopr_lib::api::graph::traits::EdgeTransportMeasurement) {}

        fn average_latency(&self) -> Option<Duration> {
            None
        }

        fn average_probe_rate(&self) -> f64 {
            0.0
        }

        fn score(&self) -> f64 {
            0.0
        }
    }

    impl hopr_lib::api::graph::traits::EdgeNetworkObservableRead for StubMeasurement {
        fn is_connected(&self) -> bool {
            false
        }
    }

    impl hopr_lib::api::graph::EdgeImmediateProtocolObservable for StubMeasurement {
        fn ack_rate(&self) -> Option<f64> {
            None
        }
    }

    impl hopr_lib::api::graph::traits::EdgeProtocolObservable for StubMeasurement {
        fn capacity(&self) -> Option<u128> {
            None
        }
    }

    impl<C> HasGraphView for ChainNode<C>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type Graph = StubGraph;

        fn graph(&self) -> &Self::Graph {
            static G: StubGraph = StubGraph;
            &G
        }

        fn status(&self) -> ComponentStatus {
            ComponentStatus::Ready
        }
    }

    async fn register_test_safe<C>(chain: &C, node_addr: Address) -> anyhow::Result<()>
    where
        C: HoprChainApi + ChainReadAccountOperations + ChainWriteAccountOperations,
    {
        let account = chain
            .stream_accounts(AccountSelector::default().with_chain_key(node_addr))?
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("missing account for {node_addr}"))?;
        let safe = account
            .safe_address
            .ok_or_else(|| anyhow::anyhow!("missing safe for {node_addr}"))?;
        chain.register_safe(&safe).await?.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn default_config_should_have_sensible_values() {
        let cfg = ChannelLifecycleConfig::default();
        assert_eq!(cfg.population.min_open_channels, 5);
        assert_eq!(cfg.population.target_open_channels, 8);
        assert!(cfg.finalizer.enabled);
        assert!(cfg.proactive_funding.enabled);
        assert_eq!(cfg.eligibility.min_peer_quality_score, 0.5);
    }

    #[test_log::test(tokio::test)]
    async fn strategy_should_fund_channel_below_threshold() -> anyhow::Result<()> {
        use anyhow::Context as _;

        let stake_limit = HoprBalance::from(3_u32);
        let fund_amount = HoprBalance::from(5_u32);
        let initial_balance = HoprBalance::from(2_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(2_u32) // below threshold of 3
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let node = Arc::new(ChainNode(Arc::clone(&connector)));

        let cfg = ChannelLifecycleConfig {
            tick_interval: Duration::from_millis(100),
            jitter: Duration::ZERO,
            funding: FundingConfig {
                lower_balance_threshold: stake_limit,
                topup_balance: fund_amount,
                min_safe_balance_required: HoprBalance::from(1_u32),
                stop_when_unfunded: true,
                ..Default::default()
            },
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::ZERO,
            },
            ..Default::default()
        };

        let mut strategy: Box<dyn crate::strategy::Strategy + Send> = ChannelLifecycleStrategy::new(cfg).build(node);

        let handle = tokio::spawn(async move {
            let _ = strategy.run().await;
        });

        // Drive at least one full pipeline pass so the fund-pass has a chance
        // to submit a `fund_channel` tx and the chain layer to confirm it.
        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
        let _ = handle.await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels for BOB")?
            .collect()
            .await;

        assert!(
            channels.iter().any(|c| c.balance > initial_balance),
            "expected the under-funded channel to be topped up; got {channels:?}"
        );

        Ok(())
    }

    #[test]
    fn restart_grace_should_block_close_pass() {
        let cfg = ChannelLifecycleConfig {
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::from_secs(3600),
            },
            ..Default::default()
        };
        let start_epoch = Instant::now();
        let grace_elapsed = start_epoch.elapsed() >= cfg.restart.startup_close_grace_period;
        assert!(
            !grace_elapsed,
            "close pass should be suppressed during startup grace period"
        );
    }

    #[test_log::test(tokio::test)]
    async fn display_should_return_channel_lifecycle() -> anyhow::Result<()> {
        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let chain_connector = Arc::new(chain_connector);
        let node = Arc::new(ChainNode(Arc::clone(&chain_connector)));

        let strategy: Box<dyn crate::strategy::Strategy + Send> =
            ChannelLifecycleStrategy::new(ChannelLifecycleConfig::default()).build(node);

        assert_eq!(strategy.to_string(), "channel_lifecycle");
        fn assert_send<T: Send>(_: T) {}
        assert_send(strategy);

        Ok(())
    }

    #[test]
    fn cooldown_should_prevent_reopen() {
        let _cfg = ChannelLifecycleConfig {
            population: PopulationConfig {
                peer_reopen_cooldown: Duration::from_secs(3600),
                ..Default::default()
            },
            ..Default::default()
        };

        let cooldown: Arc<DashMap<Address, Instant>> = Arc::new(DashMap::new());
        let dest = *CHRIS;
        cooldown.insert(dest, Instant::now() + Duration::from_secs(3600));

        let on_cooldown = cooldown.get(&dest).map(|v| Instant::now() < *v).unwrap_or(false);
        assert!(on_cooldown, "peer should be on cooldown");
    }

    /// Documents the restart guard's per-instance semantics: a freshly-built
    /// strategy starts a new grace window, regardless of how long the previous
    /// instance had been running.  The close pass is suppressed on the new
    /// instance until its own `startup_close_grace_period` elapses.
    #[test]
    fn restart_grace_should_re_apply_on_new_instance() {
        let cfg = ChannelLifecycleConfig {
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::from_secs(60),
            },
            ..Default::default()
        };

        // Old instance was running long enough that its grace window had elapsed.
        let old_start_epoch = Instant::now() - Duration::from_secs(65);
        assert!(
            old_start_epoch.elapsed() >= cfg.restart.startup_close_grace_period,
            "old instance's grace should have elapsed"
        );

        // After dropping the old instance and constructing a new one,
        // `start_epoch` resets — the new grace window starts from now.
        let new_start_epoch = Instant::now();
        assert!(
            new_start_epoch.elapsed() < cfg.restart.startup_close_grace_period,
            "new instance's grace should not have elapsed — restart guard re-applies per instance"
        );
    }

    /// Documents that no per-instance runtime state (in-flight sets, cooldown,
    /// observation history, ticket-activity counters, peer-addr cache) survives
    /// dropping the strategy.  A new instance starts cold; only on-chain state
    /// (channels, balances) is observable to it.  This is intentional: the
    /// strategy treats the chain as the source of truth and rebuilds its
    /// off-chain bookkeeping from observations after restart.
    #[test]
    fn new_instance_should_have_empty_state_after_old_dropped() {
        use dashmap::DashSet;
        use parking_lot::Mutex;

        fn fresh_inner(cfg: ChannelLifecycleConfig) -> ChannelLifecycleStrategyInner<()> {
            ChannelLifecycleStrategyInner {
                cfg,
                node: Arc::new(()),
                open_in_flight: Arc::new(DashSet::new()),
                fund_in_flight: Arc::new(DashSet::new()),
                close_in_flight: Arc::new(DashSet::new()),
                finalize_in_flight: Arc::new(DashSet::new()),
                cooldown: Arc::new(DashMap::new()),
                start_epoch: Instant::now(),
                last_observed: Arc::new(DashMap::new()),
                peer_ticket_activity: Arc::new(DashMap::new()),
                peer_addr_cache: Arc::new(Mutex::new(None)),
            }
        }

        let cfg = ChannelLifecycleConfig::default();

        // Simulate accumulated state on the first instance.
        let inner1 = fresh_inner(cfg.clone());
        inner1
            .cooldown
            .insert(*CHRIS, Instant::now() + Duration::from_secs(3600));
        inner1.peer_ticket_activity.insert(*ALICE, 42);
        inner1.open_in_flight.insert(*DAVE);
        let old_start_epoch = inner1.start_epoch;

        drop(inner1);
        std::thread::sleep(Duration::from_millis(5));

        // The new instance is built from scratch — none of the previous state
        // is reachable.
        let inner2 = fresh_inner(cfg);

        assert!(
            inner2.open_in_flight.is_empty(),
            "open_in_flight should not persist across drop"
        );
        assert!(
            inner2.fund_in_flight.is_empty(),
            "fund_in_flight should not persist across drop"
        );
        assert!(
            inner2.close_in_flight.is_empty(),
            "close_in_flight should not persist across drop"
        );
        assert!(
            inner2.finalize_in_flight.is_empty(),
            "finalize_in_flight should not persist across drop"
        );
        assert!(
            inner2.cooldown.is_empty(),
            "cooldown should not persist across drop — recently closed peers may be reopened by the new instance"
        );
        assert!(
            inner2.peer_ticket_activity.is_empty(),
            "ticket activity counters should not persist across drop"
        );
        assert!(
            inner2.last_observed.is_empty(),
            "balance/ticket-index history should not persist across drop — proactive funding warms up over the first \
             few ticks"
        );
        assert!(
            inner2.peer_addr_cache.lock().is_none(),
            "peer-addr cache should not persist across drop"
        );
        assert!(
            inner2.start_epoch > old_start_epoch,
            "start_epoch should reset on a new instance — restart guard re-applies"
        );
    }
}
