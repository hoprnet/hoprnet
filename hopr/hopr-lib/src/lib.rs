//! HOPR library creating a unified [`Hopr`] object that can be used on its own,
//! as well as integrated into other systems and libraries.
//!
//! The [`Hopr`] object is standalone, meaning that once it is constructed and run,
//! it will perform its functionality autonomously. The API it offers serves as a
//! high-level integration point for other applications and utilities, but offers
//! a complete and fully featured HOPR node stripped from top level functionality
//! such as the REST API, key management...
//!
//! The intended way to use hopr_lib is for a specific tool to be built on top of it;
//! should the default `hoprd` implementation not be acceptable.
//!
//! For most of the practical use cases, the `hoprd` application should be a preferable
//! choice.
/// Helper functions.
mod helpers;

/// Builder module for the [`Hopr`] object.
pub mod builder;
/// Configuration-related public types
pub mod config;
/// Various public constants.
pub mod constants;
/// Lists all errors thrown from this library.
pub mod errors;
// Re-export peer discovery types from hopr-api.
pub use hopr_api::node::{AnnouncedPeer, AnnouncementOrigin};
/// Utility module with helper types and functionality over hopr-lib behavior.
pub mod utils;

pub use hopr_api as api;

/// Exports of libraries necessary for API and interface operations.
#[doc(hidden)]
pub mod exports {
    pub mod types {
        pub use hopr_api::types::{chain, internal, primitive};
    }

    pub mod crypto {
        pub use hopr_api::types::crypto as types;
        pub use hopr_crypto_keypair as keypair;
    }

    pub mod network {
        pub use hopr_network_types as types;
    }

    pub use hopr_transport as transport;
}

/// Export of relevant types for easier integration.
#[doc(hidden)]
pub mod prelude {
    #[cfg(feature = "runtime-tokio")]
    pub use super::exports::network::types::{
        prelude::ForeignDataMode,
        udp::{ConnectedUdpStream, UdpStreamParallelism},
    };
    pub use super::exports::{
        crypto::{
            keypair::key_pair::HoprKeys,
            types::prelude::{ChainKeypair, Hash, OffchainKeypair},
        },
        transport::{OffchainPublicKey, socket::HoprSocket},
        types::primitive::prelude::Address,
    };
}

use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use futures::{FutureExt, Stream, StreamExt, TryFutureExt, pin_mut};
use futures_time::future::FutureExt as FuturesTimeFutureExt;
#[cfg(feature = "session-client")]
pub use hopr_api::node::HoprSessionClientOperations;
use hopr_api::{
    chain::*,
    graph::HoprGraphApi,
    network::NetworkView as _,
    node::{
        AtomicHoprState, ComponentStatus, ComponentStatusReporter, EitherErrExt, EventWaitResult, HasChainApi,
        HasGraphView, HasNetworkView, HasTicketManagement, HasTransportApi, NodeOnchainIdentity,
    },
};
pub use hopr_api::{
    graph::EdgeLinkObservable,
    network::NetworkStreamControl,
    node::{
        EitherErr, HoprNodeOperations, HoprState, IncentiveChannelOperations, IncentiveRedeemOperations,
        TransportOperations,
    },
    tickets::{ChannelStats, RedemptionResult, TicketManagement, TicketManagementExt},
    types::{crypto::prelude::*, internal::prelude::*, primitive::prelude::*},
};
use hopr_async_runtime::prelude::spawn;
pub use hopr_async_runtime::{Abortable, AbortableList};
pub use hopr_crypto_keypair::key_pair::{HoprKeys, IdentityRetrievalModes};
pub use hopr_network_types::prelude::*;
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport::transfer_session;
pub use hopr_transport::*;
use tracing::debug;

pub use crate::{
    config::SafeModule,
    constants::{MIN_NATIVE_BALANCE, SUGGESTED_NATIVE_BALANCE},
    errors::{HoprLibError, HoprStatusError},
};

/// Public routing configuration for session opening in `hopr-lib`.
///
/// This intentionally exposes only hop-count based routing.
#[cfg(feature = "session-client")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, smart_default::SmartDefault)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HopRouting(
    #[default(hopr_api::types::primitive::bounded::BoundedSize::MIN)]
    hopr_api::types::primitive::bounded::BoundedSize<
        { hopr_api::types::internal::routing::RoutingOptions::MAX_INTERMEDIATE_HOPS },
    >,
);

#[cfg(feature = "session-client")]
impl HopRouting {
    /// Maximum number of hops that can be configured.
    pub const MAX_HOPS: usize = hopr_api::types::internal::routing::RoutingOptions::MAX_INTERMEDIATE_HOPS;

    /// Returns the configured number of hops.
    pub fn hop_count(self) -> usize {
        self.0.into()
    }
}

#[cfg(feature = "session-client")]
impl TryFrom<usize> for HopRouting {
    type Error = hopr_api::types::primitive::errors::GeneralError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

#[cfg(feature = "session-client")]
impl From<HopRouting> for hopr_api::types::internal::routing::RoutingOptions {
    fn from(value: HopRouting) -> Self {
        Self::Hops(value.0)
    }
}

/// Session client configuration for `hopr-lib`.
///
/// Unlike transport-level configuration, this API intentionally does not expose
/// explicit intermediate paths.
#[cfg(feature = "session-client")]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault)]
pub struct HoprSessionClientConfig {
    /// Forward route selection policy.
    pub forward_path: HopRouting,
    /// Return route selection policy.
    pub return_path: HopRouting,
    /// Capabilities offered by the session.
    #[default(_code = "SessionCapability::Segmentation.into()")]
    pub capabilities: SessionCapabilities,
    /// Optional pseudonym used for the session. Mostly useful for testing only.
    #[default(None)]
    pub pseudonym: Option<hopr_api::types::internal::protocol::HoprPseudonym>,
    /// Enable automatic SURB management for the session.
    #[default(Some(SurbBalancerConfig::default()))]
    pub surb_management: Option<SurbBalancerConfig>,
    /// If set, the maximum number of possible SURBs will always be sent with session data packets.
    #[default(false)]
    pub always_max_out_surbs: bool,
}

#[cfg(feature = "session-client")]
impl From<HoprSessionClientConfig> for hopr_transport::SessionClientConfig {
    fn from(value: HoprSessionClientConfig) -> Self {
        Self {
            forward_path_options: value.forward_path.into(),
            return_path_options: value.return_path.into(),
            capabilities: value.capabilities,
            pseudonym: value.pseudonym,
            surb_management: value.surb_management,
            always_max_out_surbs: value.always_max_out_surbs,
        }
    }
}

/// Long-running tasks that are spawned by the HOPR node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumCount)]
pub(crate) enum HoprLibProcess {
    #[strum(to_string = "transport: {0}")]
    Transport(HoprTransportProcess),
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    #[allow(dead_code)] // constructed only with feature = "session-server"
    SessionServer,
    #[strum(to_string = "subscription for on-chain channel updates")]
    ChannelEvents,
    #[strum(to_string = "on received ticket event (winning or rejected)")]
    TicketEvents,
    #[strum(to_string = "neglecting tickets on closed channels")]
    ChannelClosureNeglect,
}

/// Prepare an optimized version of the tokio runtime setup for hopr-lib specifically.
///
/// Divide the available CPU parallelism by 2, since half of the available threads are
/// to be used for IO-bound and half for CPU-bound tasks.
#[cfg(feature = "runtime-tokio")]
pub fn prepare_tokio_runtime(
    num_cpu_threads: Option<std::num::NonZeroUsize>,
    num_io_threads: Option<std::num::NonZeroUsize>,
) -> anyhow::Result<tokio::runtime::Runtime> {
    use std::str::FromStr;
    let avail_parallelism = std::thread::available_parallelism().ok().map(|v| v.get() / 2);

    hopr_parallelize::cpu::init_thread_pool(
        num_cpu_threads
            .map(|v| v.get())
            .or(avail_parallelism)
            .ok_or(anyhow::anyhow!(
                "Could not determine the number of CPU threads to use. Please set the HOPRD_NUM_CPU_THREADS \
                 environment variable."
            ))?
            .max(1),
    )?;

    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(
            num_io_threads
                .map(|v| v.get())
                .or(avail_parallelism)
                .ok_or(anyhow::anyhow!(
                    "Could not determine the number of IO threads to use. Please set the HOPRD_NUM_IO_THREADS \
                     environment variable."
                ))?
                .max(1),
        )
        .thread_name("hoprd")
        .thread_stack_size(
            std::env::var("HOPRD_THREAD_STACK_SIZE")
                .ok()
                .and_then(|v| usize::from_str(&v).ok())
                .unwrap_or(10 * 1024 * 1024)
                .max(2 * 1024 * 1024),
        )
        .build()?)
}

/// Type alias used to send and receive transport data via a running HOPR node.
pub type HoprTransportIO = socket::HoprSocket<
    futures::channel::mpsc::Receiver<ApplicationDataIn>,
    futures::channel::mpsc::Sender<(DestinationRouting, ApplicationDataOut)>,
>;

type TicketEvents = (
    async_broadcast::Sender<hopr_api::node::TicketEvent>,
    async_broadcast::InactiveReceiver<hopr_api::node::TicketEvent>,
);

/// Time to wait until the node's keybinding appears on-chain
const NODE_READY_TIMEOUT: Duration = Duration::from_secs(120);

/// HOPR main object providing the entire HOPR node functionality
///
/// Instantiating this object creates all processes and objects necessary for
/// running the HOPR node. Once created, the node can be started using the
/// `run()` method.
///
/// Externally offered API should be enough to perform all necessary tasks
/// with the HOPR node manually, but it is advised to create such a configuration
/// that manual interaction is unnecessary.
///
/// As such, the `hopr_lib` serves mainly as an integration point into Rust programs.
pub struct Hopr<Chain, Graph, Net, TMgr> {
    pub(crate) transport_id: OffchainKeypair,
    pub(crate) chain_id: NodeOnchainIdentity,
    pub(crate) cfg: config::HoprLibConfig,
    pub(crate) state: Arc<AtomicHoprState>,
    pub(crate) transport_api: HoprTransport<Chain, Graph, Net>,
    pub(crate) chain_api: Chain,
    pub(crate) ticket_event_subscribers: TicketEvents,
    pub(crate) ticket_manager: TMgr,
    #[allow(dead_code)] // Handles must stay alive to keep background tasks running
    pub(crate) processes: AbortableList<HoprLibProcess>,
}

impl<Chain, Graph, Net, TMgr> Hopr<Chain, Graph, Net, TMgr>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    pub fn config(&self) -> &config::HoprLibConfig {
        &self.cfg
    }

    /// Returns a reference to the network graph.
    pub fn graph(&self) -> &Graph {
        self.transport_api.graph()
    }

    #[cfg(feature = "session-client")]
    fn error_if_not_in_state(&self, state: HoprState, error: String) -> errors::Result<()> {
        if HoprNodeOperations::status(self) == state {
            Ok(())
        } else {
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(state, error)))
        }
    }
}

#[cfg(feature = "session-client")]
#[async_trait::async_trait]
impl<Chain, Graph, Net, TMgr> hopr_api::node::HoprSessionClientOperations for Hopr<Chain, Graph, Net, TMgr>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: hopr_api::network::NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    TMgr: Send + Sync + 'static,
{
    type Config = HoprSessionClientConfig;
    type Error = HoprLibError;
    type Session = HoprSession;
    type SessionConfigurator = HoprSessionConfigurator;
    type Target = SessionTarget;

    async fn connect_to(
        &self,
        destination: Address,
        target: Self::Target,
        cfg: Self::Config,
    ) -> Result<(Self::Session, Self::SessionConfigurator), Self::Error> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let backoff = backon::ConstantBuilder::default()
            .with_max_times(self.cfg.protocol.session.establish_max_retries as usize)
            .with_delay(self.cfg.protocol.session.establish_retry_timeout)
            .with_jitter();

        use backon::Retryable;

        Ok((|| {
            let cfg = hopr_transport::SessionClientConfig::from(cfg.clone());
            let target = target.clone();
            async { self.transport_api.new_session(destination, target, cfg).await }
        })
        .retry(backoff)
        .sleep(backon::FuturesTimerSleeper)
        .await?)
    }
}

// ---------------------------------------------------------------------------
// Has* accessor trait implementations
// ---------------------------------------------------------------------------

/// Maps [`Health`] into a [`ComponentStatus`] for a named component.
///
/// Assumes the node is already in the `Running` state.
fn network_health_to_status(health: Health, component: &str) -> ComponentStatus {
    match health {
        Health::Green | Health::Yellow => ComponentStatus::Ready,
        Health::Orange => ComponentStatus::Degraded(format!("{component}: low connectivity (1 peer)")),
        Health::Red => ComponentStatus::Degraded(format!("{component}: no connected peers")),
        Health::Unknown => ComponentStatus::Unavailable(format!("{component}: network not initialized")),
    }
}

impl<Chain, Graph, Net, TMgr> HasChainApi for Hopr<Chain, Graph, Net, TMgr>
where
    Chain: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
{
    type ChainApi = Chain;
    type ChainError = HoprLibError;

    fn identity(&self) -> &NodeOnchainIdentity {
        &self.chain_id
    }

    fn chain_api(&self) -> &Chain {
        &self.chain_api
    }

    fn status(&self) -> ComponentStatus {
        self.chain_api.component_status()
    }

    fn wait_for_on_chain_event<F>(
        &self,
        predicate: F,
        context: String,
        timeout: Duration,
    ) -> EventWaitResult<<Self::ChainApi as HoprChainApi>::ChainError, Self::ChainError>
    where
        F: Fn(&ChainEvent) -> bool + Send + Sync + 'static,
    {
        debug!(%context, "registering wait for on-chain event");
        let (event_stream, handle) = futures::stream::abortable(
            self.chain_api
                .subscribe()?
                .skip_while(move |event| futures::future::ready(!predicate(event))),
        );

        let ctx = context.clone();

        Ok((
            spawn(async move {
                pin_mut!(event_stream);
                let res = event_stream
                    .next()
                    .timeout(futures_time::time::Duration::from(timeout))
                    .map_err(|_| HoprLibError::GeneralError(format!("{ctx} timed out after {timeout:?}")).into_right())
                    .await?
                    .ok_or(
                        HoprLibError::GeneralError(format!("failed to yield an on-chain event for {ctx}")).into_right(),
                    );
                debug!(%ctx, ?res, "on-chain event waiting done");
                res
            })
            .map_err(move |_| HoprLibError::GeneralError(format!("failed to spawn future for {context}")).into_right())
            .and_then(futures::future::ready)
            .boxed(),
            handle,
        ))
    }
}

impl<Chain, Graph, Net, TMgr> HasNetworkView for Hopr<Chain, Graph, Net, TMgr>
where
    Chain: Send + Sync + 'static,
    Graph: Send + Sync + 'static,
    Net: hopr_api::network::NetworkView + Send + Sync + 'static,
{
    type NetworkView = HoprTransport<Chain, Graph, Net>;

    fn network_view(&self) -> &Self::NetworkView {
        &self.transport_api
    }

    fn status(&self) -> ComponentStatus {
        network_health_to_status(self.transport_api.health(), "network")
    }
}

impl<Chain, Graph, Net, TMgr> HasGraphView for Hopr<Chain, Graph, Net, TMgr>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: hopr_api::network::NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    type Graph = Graph;

    fn graph(&self) -> &Graph {
        self.transport_api.graph()
    }

    fn status(&self) -> ComponentStatus {
        ComponentStatus::Ready
    }
}

impl<Chain, Graph, Net, TMgr> HasTransportApi for Hopr<Chain, Graph, Net, TMgr>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: hopr_api::network::NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    TMgr: Send + Sync + 'static,
{
    type Transport = HoprTransport<Chain, Graph, Net>;

    fn transport(&self) -> &Self::Transport {
        &self.transport_api
    }

    fn status(&self) -> ComponentStatus {
        network_health_to_status(self.transport_api.health(), "transport")
    }
}

// Available only on Relay nodes that specify `TMgr` that implements TicketManagement
impl<Chain, Graph, Net, TMgr> HasTicketManagement for Hopr<Chain, Graph, Net, TMgr>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    TMgr: TicketManagement + Clone + Send + Sync + 'static,
{
    type TicketManager = TMgr;

    fn ticket_management(&self) -> &TMgr {
        &self.ticket_manager
    }

    fn subscribe_ticket_events(&self) -> impl Stream<Item = hopr_api::node::TicketEvent> + Send + 'static {
        self.ticket_event_subscribers.1.activate_cloned()
    }

    fn status(&self) -> ComponentStatus {
        ComponentStatus::Ready
    }
}

/// Per-component status report for the HOPR node.
#[derive(Debug, Clone)]
pub struct NodeComponentStatuses {
    /// Overall node lifecycle state.
    pub node_state: HoprState,
    /// Chain/blokli connector status.
    pub chain: ComponentStatus,
    /// P2P network layer status.
    pub network: ComponentStatus,
    /// Transport layer status.
    pub transport: ComponentStatus,
}

impl NodeComponentStatuses {
    /// Worst-case aggregation: the overall status is the worst of any component.
    pub fn aggregate(&self) -> ComponentStatus {
        let statuses = [&self.chain, &self.network, &self.transport];
        if statuses.iter().any(|s| s.is_unavailable()) {
            ComponentStatus::Unavailable("one or more components unavailable".into())
        } else if statuses.iter().any(|s| s.is_degraded()) {
            ComponentStatus::Degraded("one or more components degraded".into())
        } else if statuses.iter().any(|s| s.is_initializing()) {
            ComponentStatus::Initializing("one or more components initializing".into())
        } else {
            ComponentStatus::Ready
        }
    }
}

impl<Chain, Graph, Net, TMgr> Hopr<Chain, Graph, Net, TMgr>
where
    Chain: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    Net: hopr_api::network::NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    Graph: HoprGraphApi<HoprNodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    TMgr: Send + Sync + 'static,
{
    /// Returns per-component health statuses for the node.
    ///
    /// When the node has reached `Running`, the aggregate `node_state` is
    /// derived from component statuses (Running → Degraded → Failed).
    pub fn component_statuses(&self) -> NodeComponentStatuses {
        let base = self.state.load(Ordering::Relaxed);
        let statuses = NodeComponentStatuses {
            node_state: base,
            chain: HasChainApi::status(self),
            network: HasNetworkView::status(self),
            transport: HasTransportApi::status(self),
        };

        // Derive aggregate HoprState from component statuses once Running
        if base == HoprState::Running {
            NodeComponentStatuses {
                node_state: match statuses.aggregate() {
                    ComponentStatus::Unavailable(_) => HoprState::Failed,
                    ComponentStatus::Degraded(_) | ComponentStatus::Initializing(_) => HoprState::Degraded,
                    ComponentStatus::Ready => HoprState::Running,
                },
                ..statuses
            }
        } else {
            statuses
        }
    }
}

impl<Chain, Graph, Net, TMgr> Hopr<Chain, Graph, Net, TMgr> {
    /// Prometheus formatted metrics collected by the hopr-lib components.
    pub fn collect_hopr_metrics() -> errors::Result<String> {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "telemetry", not(test)))] {
                hopr_metrics::gather_all_metrics().map_err(HoprLibError::other)
            } else {
                Err(HoprLibError::GeneralError("BUILT WITHOUT METRICS SUPPORT".into()))
            }
        }
    }
}

impl<Chain, Graph, Net, TMgr> HoprNodeOperations for Hopr<Chain, Graph, Net, TMgr> {
    fn status(&self) -> HoprState {
        self.state.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_health_green_is_ready() {
        assert_eq!(network_health_to_status(Health::Green, "test"), ComponentStatus::Ready);
    }

    #[test]
    fn network_health_yellow_is_ready() {
        assert_eq!(network_health_to_status(Health::Yellow, "test"), ComponentStatus::Ready);
    }

    #[test]
    fn network_health_orange_is_degraded() {
        assert!(network_health_to_status(Health::Orange, "network").is_degraded());
    }

    #[test]
    fn network_health_red_is_degraded() {
        assert!(network_health_to_status(Health::Red, "network").is_degraded());
    }

    #[test]
    fn network_health_unknown_is_unavailable() {
        assert!(network_health_to_status(Health::Unknown, "network").is_unavailable());
    }

    #[test]
    fn aggregate_all_ready() {
        let statuses = NodeComponentStatuses {
            node_state: HoprState::Running,
            chain: ComponentStatus::Ready,
            network: ComponentStatus::Ready,
            transport: ComponentStatus::Ready,
        };
        assert_eq!(statuses.aggregate(), ComponentStatus::Ready);
    }

    #[test]
    fn aggregate_one_degraded() {
        let statuses = NodeComponentStatuses {
            node_state: HoprState::Running,
            chain: ComponentStatus::Ready,
            network: ComponentStatus::Degraded("low peers".into()),
            transport: ComponentStatus::Ready,
        };
        assert!(statuses.aggregate().is_degraded());
    }

    #[test]
    fn aggregate_one_unavailable() {
        let statuses = NodeComponentStatuses {
            node_state: HoprState::Running,
            chain: ComponentStatus::Unavailable("blokli down".into()),
            network: ComponentStatus::Ready,
            transport: ComponentStatus::Ready,
        };
        assert!(statuses.aggregate().is_unavailable());
    }

    #[test]
    fn aggregate_unavailable_wins_over_degraded() {
        let statuses = NodeComponentStatuses {
            node_state: HoprState::Running,
            chain: ComponentStatus::Unavailable("blokli down".into()),
            network: ComponentStatus::Degraded("low peers".into()),
            transport: ComponentStatus::Ready,
        };
        assert!(statuses.aggregate().is_unavailable());
    }
}

/// Converts a PeerId to an OffchainPublicKey.
///
/// This is a standalone utility function, not part of the API traits.
pub fn peer_id_to_offchain_key(peer_id: &PeerId) -> errors::Result<OffchainPublicKey> {
    Ok(hopr_transport::peer_id_to_public_key(peer_id)?)
}
