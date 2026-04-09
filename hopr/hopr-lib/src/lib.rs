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

/// Configuration-related public types
pub mod config;
/// Various public constants.
pub mod constants;
/// Lists all errors thrown from this library.
pub mod errors;
/// Public traits for interactions with this library.
pub mod traits;
/// Public domain types for peer discovery.
pub mod types;
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
    convert::identity,
    future::Future,
    sync::{Arc, OnceLock, atomic::Ordering},
    time::Duration,
};

use anyhow::anyhow;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt, channel::mpsc::channel, pin_mut};
use futures_time::future::FutureExt as FuturesTimeFutureExt;
use hopr_api::{
    chain::*,
    ct::{CoverTrafficGeneration, ProbingTrafficGeneration},
    node::{ChainInfo, CloseChannelResult, OpenChannelResult, SafeModuleConfig, state::AtomicHoprState},
};
pub use hopr_api::{
    graph::EdgeLinkObservable,
    network::{NetworkBuilder, NetworkStreamControl},
    node::{HoprNodeNetworkOperations, HoprNodeOperations, state::HoprState},
    tickets::{ChannelStats, RedemptionResult, TicketManagement, TicketManagementExt},
    types::{crypto::prelude::*, internal::prelude::*, primitive::prelude::*},
};
use hopr_async_runtime::prelude::spawn;
pub use hopr_async_runtime::{Abortable, AbortableList};
pub use hopr_crypto_keypair::key_pair::{HoprKeys, IdentityRetrievalModes};
pub use hopr_network_types::prelude::*;
#[cfg(all(feature = "telemetry", not(test)))]
use hopr_platform::time::native::current_time;
use hopr_ticket_manager::{HoprTicketManager, RedbStore, RedbTicketQueue};
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport::transfer_session;
pub use hopr_transport::*;
use tracing::{debug, error, info, warn};
use validator::Validate;

pub use crate::{
    config::SafeModule,
    constants::{MIN_NATIVE_BALANCE, SUGGESTED_NATIVE_BALANCE},
    errors::{HoprLibError, HoprStatusError},
    types::{AnnouncedPeer, AnnouncementOrigin},
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
pub enum HoprLibProcess {
    #[strum(to_string = "transport: {0}")]
    Transport(HoprTransportProcess),
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    SessionServer,
    #[strum(to_string = "ticket redemption queue driver")]
    TicketRedemptions,
    #[strum(to_string = "subscription for on-chain channel updates")]
    ChannelEvents,
    #[strum(to_string = "on received ticket event (winning or rejected)")]
    TicketEvents,
    #[strum(to_string = "persisting of outgoing ticket indices")]
    OutIndexSync,
}

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PROCESS_START_TIME:  hopr_metrics::SimpleGauge =  hopr_metrics::SimpleGauge::new(
        "hopr_start_time",
        "The unix timestamp in seconds at which the process was started"
    ).unwrap();
    static ref METRIC_HOPR_LIB_VERSION:  hopr_metrics::MultiGauge =  hopr_metrics::MultiGauge::new(
        "hopr_lib_version",
        "Executed version of hopr-lib",
        &["version"]
    ).unwrap();
    static ref METRIC_HOPR_NODE_INFO:  hopr_metrics::MultiGauge =  hopr_metrics::MultiGauge::new(
        "hopr_node_addresses",
        "Node on-chain and off-chain addresses",
        &["peerid", "address", "safe_address", "module_address"]
    ).unwrap();
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
    async_broadcast::Sender<TicketEvent>,
    async_broadcast::InactiveReceiver<TicketEvent>,
);

/// Time to wait until the node's keybinding appears on-chain
const NODE_READY_TIMEOUT: Duration = Duration::from_secs(120);

/// Timeout to wait until an on-chain event is received in response to a successful on-chain operation resolution.
// TODO: use the value from ChainInfo instead (once available via https://github.com/hoprnet/blokli/issues/200)
const ON_CHAIN_RESOLUTION_EVENT_TIMEOUT: Duration = Duration::from_secs(90);

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
pub struct Hopr<Chain, Graph, Net>
where
    Graph: hopr_api::graph::NetworkGraphView<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    me: OffchainKeypair,
    cfg: config::HoprLibConfig,
    state: Arc<api::node::state::AtomicHoprState>,
    transport_api: HoprTransport<Chain, Graph, Net>,
    chain_api: Chain,
    ticket_event_subscribers: TicketEvents,
    ticket_manager: OnceLock<Arc<HoprTicketManager<RedbStore, RedbTicketQueue>>>,
    processes: OnceLock<AbortableList<HoprLibProcess>>,
}

impl<Chain, Graph, Net> Hopr<Chain, Graph, Net>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: hopr_api::graph::NetworkGraphView<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    pub async fn new(
        identity: (&ChainKeypair, &OffchainKeypair),
        hopr_chain_api: Chain,
        graph: Graph,
        cfg: config::HoprLibConfig,
    ) -> errors::Result<Self> {
        if hopr_api::types::crypto_random::is_rng_fixed() {
            warn!("!! FOR TESTING ONLY !! THIS BUILD IS USING AN INSECURE FIXED RNG !!")
        }

        cfg.validate()?;

        let hopr_transport_api = HoprTransport::new(
            identity,
            hopr_chain_api.clone(),
            graph,
            vec![(&cfg.host).try_into().map_err(HoprLibError::TransportError)?],
            cfg.protocol.clone(),
        )
        .map_err(HoprLibError::TransportError)?;

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            METRIC_PROCESS_START_TIME.set(current_time().as_unix_timestamp().as_secs_f64());
            METRIC_HOPR_LIB_VERSION.set(
                &[const_format::formatcp!("{}", constants::APP_VERSION)],
                const_format::formatcp!(
                    "{}.{}",
                    env!("CARGO_PKG_VERSION_MAJOR"),
                    env!("CARGO_PKG_VERSION_MINOR")
                )
                .parse()
                .unwrap_or(0.0),
            );
        }

        let (mut new_tickets_tx, new_tickets_rx) = async_broadcast::broadcast(2048);
        new_tickets_tx.set_await_active(false);
        new_tickets_tx.set_overflow(true);

        Ok(Self {
            me: identity.1.clone(),
            cfg,
            state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
            transport_api: hopr_transport_api,
            chain_api: hopr_chain_api,
            processes: OnceLock::new(),
            ticket_event_subscribers: (new_tickets_tx, new_tickets_rx.deactivate()),
            ticket_manager: Default::default(),
        })
    }

    fn error_if_not_in_state(&self, state: HoprState, error: String) -> errors::Result<()> {
        if HoprNodeOperations::status(self) == state {
            Ok(())
        } else {
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(state, error)))
        }
    }

    pub fn config(&self) -> &config::HoprLibConfig {
        &self.cfg
    }

    /// Returns a reference to the network graph.
    pub fn graph(&self) -> &Graph {
        self.transport_api.graph()
    }

    #[inline]
    fn is_public(&self) -> bool {
        self.cfg.publish
    }

    pub async fn run<
        Ct,
        NetBuilder,
        #[cfg(feature = "session-server")] T: traits::HoprSessionServer + Clone + Send + 'static,
    >(
        &self,
        cover_traffic: Ct,
        network_builder: NetBuilder,
        #[cfg(feature = "session-server")] serve_handler: T,
    ) -> errors::Result<HoprTransportIO>
    where
        Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
        NetBuilder: NetworkBuilder<Network = Net> + Send + Sync + 'static,
    {
        self.error_if_not_in_state(
            HoprState::Uninitialized,
            "cannot start the hopr node multiple times".into(),
        )?;

        #[cfg(feature = "testing")]
        warn!("!! FOR TESTING ONLY !! Node is running with some safety checks disabled!");

        let me_onchain = *self.chain_api.me();
        info!(
            address = %me_onchain, minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
            "node is not started, please fund this node",
        );

        self.state.store(HoprState::WaitingForFunds, Ordering::Relaxed);
        helpers::wait_for_funds(
            *MIN_NATIVE_BALANCE,
            *SUGGESTED_NATIVE_BALANCE,
            Duration::from_secs(200),
            me_onchain,
            &self.chain_api,
        )
        .await?;

        let mut processes = AbortableList::<HoprLibProcess>::default();

        info!("starting HOPR node...");
        self.state.store(HoprState::CheckingBalance, Ordering::Relaxed);

        let balance: XDaiBalance = self.chain_api.balance(me_onchain).await.map_err(HoprLibError::chain)?;
        let minimum_balance = *constants::MIN_NATIVE_BALANCE;

        info!(
            address = %me_onchain,
            %balance,
            %minimum_balance,
            "node information"
        );

        if balance.le(&minimum_balance) {
            return Err(HoprLibError::GeneralError(
                "cannot start the node without a sufficiently funded wallet".into(),
            ));
        }

        self.state.store(HoprState::ValidatingNetworkConfig, Ordering::Relaxed);

        // Once we are able to query the chain,
        // check if the ticket price is configured correctly.
        let network_min_ticket_price = self
            .chain_api
            .minimum_ticket_price()
            .await
            .map_err(HoprLibError::chain)?;
        let configured_ticket_price = self.cfg.protocol.packet.codec.outgoing_ticket_price;
        if configured_ticket_price.is_some_and(|c| c < network_min_ticket_price) {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing ticket price is lower than the network minimum ticket price: \
                 {configured_ticket_price:?} < {network_min_ticket_price}"
            )));
        }
        // Once we are able to query the chain,
        // check if the winning probability is configured correctly.
        let network_min_win_prob = self
            .chain_api
            .minimum_incoming_ticket_win_prob()
            .await
            .map_err(HoprLibError::chain)?;
        let configured_win_prob = self.cfg.protocol.packet.codec.outgoing_win_prob;
        if !std::env::var("HOPR_TEST_DISABLE_CHECKS").is_ok_and(|v| v.to_lowercase() == "true")
            && configured_win_prob.is_some_and(|c| c.approx_cmp(&network_min_win_prob).is_lt())
        {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing ticket winning probability is lower than the network minimum winning \
                 probability: {configured_win_prob:?} < {network_min_win_prob}"
            )));
        }

        self.state.store(HoprState::CheckingOnchainAddress, Ordering::Relaxed);

        info!(peer_id = %self.me_peer_id(), address = %self.me_onchain(), version = constants::APP_VERSION, "Node information");

        let safe_addr = self.cfg.safe_module.safe_address;

        if self.me_onchain() == safe_addr {
            return Err(HoprLibError::GeneralError(
                "cannot use self as staking safe address".into(),
            ));
        }

        self.state.store(HoprState::RegisteringSafe, Ordering::Relaxed);
        info!(%safe_addr, "registering safe with this node");
        match self.chain_api.register_safe(&safe_addr).await {
            Ok(awaiter) => {
                // Wait until the registration is confirmed on-chain, otherwise we cannot proceed.
                awaiter.await.map_err(|error| {
                    error!(%safe_addr, %error, "safe registration failed with error");
                    HoprLibError::chain(error)
                })?;
                info!(%safe_addr, "safe successfully registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) if registered_safe == safe_addr => {
                info!(%safe_addr, "this safe is already registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) if registered_safe != safe_addr => {
                // TODO: support safe deregistration flow
                error!(%safe_addr, %registered_safe, "this node is currently registered with different safe");
                return Err(HoprLibError::GeneralError("node registered with different safe".into()));
            }
            Err(error) => {
                error!(%safe_addr, %error, "safe registration failed");
                return Err(HoprLibError::chain(error));
            }
        }

        // Only public nodes announce multiaddresses
        let multiaddresses_to_announce = if self.is_public() {
            // The multiaddresses are filtered for the non-private ones,
            // unless `announce_local_addresses` is set to `true`.
            self.transport_api.announceable_multiaddresses()
        } else {
            Vec::with_capacity(0)
        };

        // Warn when announcing a private multiaddress, which is acceptable in certain scenarios
        multiaddresses_to_announce
            .iter()
            .filter(|a| !is_public_address(a))
            .for_each(|multi_addr| warn!(?multi_addr, "announcing private multiaddress"));

        self.state.store(HoprState::AnnouncingNode, Ordering::Relaxed);

        let chain_api = self.chain_api.clone();
        let me_offchain = *self.me.public();
        let node_ready = spawn(async move { chain_api.await_key_binding(&me_offchain, NODE_READY_TIMEOUT).await });

        // At this point the node is already registered with Safe, so
        // we can announce via Safe-compliant TX
        info!(?multiaddresses_to_announce, "announcing node on chain");
        match self.chain_api.announce(&multiaddresses_to_announce, &self.me).await {
            Ok(awaiter) => {
                // Wait until the announcement is confirmed on-chain, otherwise we cannot proceed.
                awaiter.await.map_err(|error| {
                    error!(?multiaddresses_to_announce, %error, "node announcement failed");
                    HoprLibError::chain(error)
                })?;
                info!(?multiaddresses_to_announce, "node has been successfully announced");
            }
            Err(AnnouncementError::AlreadyAnnounced) => {
                info!(multiaddresses_announced = ?multiaddresses_to_announce, "node already announced on chain")
            }
            Err(error) => {
                error!(%error, ?multiaddresses_to_announce, "failed to transmit node announcement");
                return Err(HoprLibError::chain(error));
            }
        }

        self.state.store(HoprState::AwaitingKeyBinding, Ordering::Relaxed);

        // Wait for the node key-binding readiness to return
        let this_node_account = node_ready
            .await
            .map_err(HoprLibError::other)?
            .map_err(HoprLibError::chain)?;
        if this_node_account.chain_addr != self.me_onchain()
            || this_node_account.safe_address.is_none_or(|a| a != safe_addr)
        {
            error!(%this_node_account, "account bound to offchain key does not match this node");
            return Err(HoprLibError::GeneralError("account key-binding mismatch".into()));
        }

        info!(%this_node_account, "node account is ready");

        self.state.store(HoprState::InitializingServices, Ordering::Relaxed);

        info!("initializing session infrastructure");
        let incoming_session_channel_capacity = std::env::var("HOPR_INTERNAL_SESSION_INCOMING_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(256);

        let (session_tx, _session_rx) = channel::<IncomingSession>(incoming_session_channel_capacity);
        #[cfg(feature = "session-server")]
        {
            debug!(capacity = incoming_session_channel_capacity, "creating session server");
            processes.insert(
                HoprLibProcess::SessionServer,
                hopr_async_runtime::spawn_as_abortable!(
                    _session_rx
                        .for_each_concurrent(None, move |session| {
                            let serve_handler = serve_handler.clone();
                            async move {
                                let session_id = *session.session.id();
                                match serve_handler.process(session).await {
                                    Ok(_) => debug!(?session_id, "client session processed successfully"),
                                    Err(error) => error!(
                                        ?session_id,
                                        %error,
                                        "client session processing failed"
                                    ),
                                }
                            }
                        })
                        .inspect(|_| tracing::warn!(
                            task = %HoprLibProcess::SessionServer,
                            "long-running background task finished"
                        ))
                ),
            );
        }

        info!("starting ticket manager & factory");
        let store = self
            .cfg
            .ticket_storage_file
            .as_ref()
            .map(RedbStore::new)
            .unwrap_or_else(RedbStore::new_temp)
            .map_err(HoprLibError::ticket_manager)?;

        let (ticket_manager, ticket_factory) = HoprTicketManager::new_with_factory(store);
        let ticket_manager = Arc::new(ticket_manager);
        let ticket_factory = Arc::new(ticket_factory);

        // Synchronize the ticket manager with the chain before starting the packet pipeline
        ticket_manager
            .sync_from_incoming_channels(
                &self
                    .chain_api
                    .stream_channels(ChannelSelector::default().with_destination(me_onchain))
                    .map_err(HoprLibError::chain)?
                    .collect::<Vec<_>>()
                    .await,
            )
            .map_err(HoprLibError::ticket_manager)?;

        // Synchronize the ticket factory with the chain before starting the packet pipeline
        ticket_factory
            .sync_from_outgoing_channels(
                &self
                    .chain_api
                    .stream_channels(ChannelSelector::default().with_source(me_onchain))
                    .map_err(HoprLibError::chain)?
                    .collect::<Vec<_>>()
                    .await,
            )
            .map_err(HoprLibError::ticket_manager)?;

        // Make sure outgoing ticket indices in the ticket factory are periodically persisted to disk
        let (index_sync_handle, index_sync_reg) = futures::future::AbortHandle::new_pair();
        let tfact = ticket_factory.clone();
        let tfact2 = ticket_factory.clone();
        spawn(
            futures::stream::Abortable::new(
                futures_time::stream::interval(self.cfg.out_index_sync_period.into()),
                index_sync_reg,
            )
            .for_each(move |_| {
                let tfact = tfact.clone();
                async move {
                    if let Err(error) =
                        hopr_async_runtime::prelude::spawn_blocking(move || tfact.save_outgoing_indices())
                            .map_err(hopr_ticket_manager::TicketManagerError::store)
                            .and_then(futures::future::ready)
                            .await
                    {
                        tracing::error!(%error, "failed to sync ticket indices to persistent storage:");
                    } else {
                        tracing::trace!("successfully synced ticket indices");
                    }
                }
            })
            .inspect(move |_| {
                // Do an extra save here on graceful shutdown
                if let Err(error) = tfact2.save_outgoing_indices() {
                    tracing::error!(%error, "failed to sync ticket indices to persistent storage on shutdown");
                }
                tracing::warn!(
                    task = %HoprLibProcess::OutIndexSync,
                    "long-running background task finished"
                )
            }),
        );
        processes.insert(HoprLibProcess::OutIndexSync, index_sync_handle);

        info!("starting ticket events processor");
        let (tickets_tx, tickets_rx) = channel(8192);
        let (tickets_rx, tickets_handle) = futures::stream::abortable(tickets_rx);
        processes.insert(HoprLibProcess::TicketEvents, tickets_handle);
        let new_ticket_tx = self.ticket_event_subscribers.0.clone();
        let tmgr = ticket_manager.clone();
        spawn(
            tickets_rx
                .for_each(move |event| {
                    // Ticket manager processes new winning tickets
                    if let TicketEvent::WinningTicket(ticket) = &event
                        && let Err(error) = tmgr.insert_incoming_ticket(**ticket)
                    {
                        tracing::error!(%error, "failed to insert incoming ticket into manager");
                    }
                    if let Err(error) = new_ticket_tx.try_broadcast(event) {
                        tracing::error!(%error, "failed to broadcast new ticket event to subscribers");
                    }
                    futures::future::ready(())
                })
                .inspect(|_| {
                    tracing::warn!(
                        task = %HoprLibProcess::TicketEvents,
                        "long-running background task finished"
                    )
                }),
        );

        info!("starting transport");
        let (hopr_socket, transport_processes) = self
            .transport_api
            .run(cover_traffic, network_builder, tickets_tx, ticket_factory, session_tx)
            .await?;
        processes.flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

        info!("subscribing to channel events");
        let (chain_events_sub_handle, chain_events_sub_reg) = hopr_async_runtime::AbortHandle::new_pair();
        processes.insert(HoprLibProcess::ChannelEvents, chain_events_sub_handle);
        let chain = self.chain_api.clone();
        let events = chain.subscribe().map_err(HoprLibError::chain)?;
        let tmgr = ticket_manager.clone();

        spawn(
            futures::stream::Abortable::new(
                events
                    .filter_map(move |event|
                        futures::future::ready(event.try_as_channel_closed())
                    ),
                chain_events_sub_reg
            )
            .for_each(move |closed_channel| {
                let chain = chain.clone();
                let tmgr = tmgr.clone();
                async move {
                    match closed_channel.direction(chain.me()) {
                        Some(ChannelDirection::Incoming) => {
                            // Ticket manager neglects tickets on incoming channel closure
                            match tmgr.neglect_tickets(closed_channel.get_id(), None) {
                                Ok(neglected) if !neglected.is_empty() => {
                                    warn!(num_neglected = neglected.len(), %closed_channel, "tickets on incoming closed channel were neglected");
                                },
                                Ok(_) => {
                                    debug!(%closed_channel, "no neglected tickets on incoming closed channel");
                                },
                                Err(error) => {
                                    error!(%error, %closed_channel, "failed to mark tickets on incoming closed channel as neglected");
                                }
                            }
                        },
                        Some(ChannelDirection::Outgoing) => {
                            // Removal of outgoing ticket index is done automatically be the ticket manager
                            // when new epoch is encountered
                        }
                        _ => {} // Event for a channel that is not our own
                    }
                }
            })
            .inspect(|_| tracing::warn!(task = %HoprLibProcess::ChannelEvents, "long-running background task finished"))
        );

        self.state.store(HoprState::Running, Ordering::Relaxed);

        self.ticket_manager
            .set(ticket_manager)
            .map_err(|_| HoprLibError::other(anyhow!("cannot set ticket manager")))?;

        info!(
            id = %self.me_peer_id(),
            version = constants::APP_VERSION,
            "NODE STARTED AND RUNNING"
        );

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_HOPR_NODE_INFO.set(
            &[
                &self.me.public().to_peerid_str(),
                &me_onchain.to_string(),
                &self.cfg.safe_module.safe_address.to_string(),
                &self.cfg.safe_module.module_address.to_string(),
            ],
            1.0,
        );

        let _ = self.processes.set(processes);

        Ok(hopr_socket)
    }

    /// Used to practically shut down all node's processes without dropping the instance.
    ///
    /// This means that the instance can be used to retrieve some information, but all
    /// active operations will stop and new will be impossible to perform.
    /// Such operations will return [`HoprStatusError::NotThereYet`].
    ///
    /// This is the final state and cannot be reversed by calling `run` again.
    pub fn shutdown(&self) -> Result<(), HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "node is not running".into())?;
        if let Some(processes) = self.processes.get() {
            processes.abort_all();
        }
        self.state.store(HoprState::Terminated, Ordering::Relaxed);
        info!("NODE SHUTDOWN COMPLETE");
        Ok(())
    }

    /// Create a client session connection returning a session object that implements
    /// [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`] and can bu used as a read/write binary session.
    #[cfg(feature = "session-client")]
    pub async fn connect_to(
        &self,
        destination: Address,
        target: SessionTarget,
        cfg: HoprSessionClientConfig,
    ) -> errors::Result<HoprSession> {
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

    /// Sends keep-alive to the given [`SessionId`], making sure the session is not
    /// closed due to inactivity.
    #[cfg(feature = "session-client")]
    pub async fn keep_alive_session(&self, id: &SessionId) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for session operations".into())?;
        Ok(self.transport_api.probe_session(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn get_session_surb_balancer_config(&self, id: &SessionId) -> errors::Result<Option<SurbBalancerConfig>> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for session operations".into())?;
        Ok(self.transport_api.session_surb_balancing_cfg(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn update_session_surb_balancer_config(
        &self,
        id: &SessionId,
        cfg: SurbBalancerConfig,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for session operations".into())?;
        Ok(self.transport_api.update_session_surb_balancing_cfg(id, cfg).await?)
    }

    /// Spawns a one-shot awaiter that hooks up to the [`ChainEvent`] bus and either matching the given `predicate`
    /// successfully or timing out after `timeout`.
    fn spawn_wait_for_on_chain_event(
        &self,
        context: impl std::fmt::Display,
        predicate: impl Fn(&ChainEvent) -> bool + Send + Sync + 'static,
        timeout: Duration,
    ) -> errors::Result<(
        impl Future<Output = errors::Result<ChainEvent>>,
        hopr_async_runtime::AbortHandle,
    )> {
        debug!(%context, "registering wait for on-chain event");
        let (event_stream, handle) = futures::stream::abortable(
            self.chain_api
                .subscribe()
                .map_err(HoprLibError::chain)?
                .skip_while(move |event| futures::future::ready(!predicate(event))),
        );

        let ctx = context.to_string();

        Ok((
            spawn(async move {
                pin_mut!(event_stream);
                let res = event_stream
                    .next()
                    .timeout(futures_time::time::Duration::from(timeout))
                    .map_err(|_| HoprLibError::GeneralError(format!("{ctx} timed out after {timeout:?}")))
                    .await?
                    .ok_or(HoprLibError::GeneralError(format!(
                        "failed to yield an on-chain event for {ctx}"
                    )));
                debug!(%ctx, ?res, "on-chain event waiting done");
                res
            })
            .map_err(move |_| HoprLibError::GeneralError(format!("failed to spawn future for {context}")))
            .and_then(futures::future::ready),
            handle,
        ))
    }
}

impl<Chain, Graph, Net> Hopr<Chain, Graph, Net>
where
    Graph: hopr_api::graph::NetworkGraphView<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    // === telemetry
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

// === Trait implementations for the high-level node API ===

impl<Chain, Graph, Net> HoprNodeOperations for Hopr<Chain, Graph, Net>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: hopr_api::graph::NetworkGraphView<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    fn status(&self) -> HoprState {
        self.state.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl<Chain, Graph, Net> HoprNodeNetworkOperations for Hopr<Chain, Graph, Net>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: hopr_api::graph::NetworkGraphView<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    type Error = HoprLibError;
    type TransportObservable = <Graph as hopr_api::graph::NetworkGraphView>::Observed;

    fn me_peer_id(&self) -> PeerId {
        (*self.me.public()).into()
    }

    async fn get_public_nodes(&self) -> Result<Vec<(PeerId, Address, Vec<Multiaddr>)>, Self::Error> {
        Ok(self
            .chain_api
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .map_err(HoprLibError::chain)?
            .map(|entry| {
                (
                    PeerId::from(entry.public_key),
                    entry.chain_addr,
                    entry.get_multiaddrs().to_vec(),
                )
            })
            .collect()
            .await)
    }

    async fn network_health(&self) -> hopr_api::network::Health {
        self.transport_api.network_health().await
    }

    async fn network_connected_peers(&self) -> Result<Vec<PeerId>, Self::Error> {
        Ok(self
            .transport_api
            .network_connected_peers()
            .await?
            .into_iter()
            .map(PeerId::from)
            .collect())
    }

    fn network_peer_info(&self, peer: &PeerId) -> Option<Self::TransportObservable> {
        let pubkey = OffchainPublicKey::from_peerid(peer).ok()?;
        self.transport_api.network_peer_observations(&pubkey)
    }

    async fn all_network_peers(
        &self,
        minimum_score: f64,
    ) -> Result<Vec<(Option<Address>, PeerId, Self::TransportObservable)>, Self::Error> {
        Ok(
            futures::stream::iter(self.transport_api.all_network_peers(minimum_score).await?)
                .filter_map(|(pubkey, info)| async move {
                    let peer_id = PeerId::from(pubkey);
                    let address = self.peerid_to_chain_key(&peer_id).ok().flatten();
                    Some((address, peer_id, info))
                })
                .collect::<Vec<_>>()
                .await,
        )
    }

    fn local_multiaddresses(&self) -> Vec<Multiaddr> {
        self.transport_api.local_multiaddresses()
    }

    async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        self.transport_api.listening_multiaddresses().await
    }

    async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
        let Ok(pubkey) = hopr_transport::peer_id_to_public_key(peer) else {
            return vec![];
        };
        self.transport_api.network_observed_multiaddresses(&pubkey).await
    }

    async fn multiaddresses_announced_on_chain(&self, peer: &PeerId) -> Result<Vec<Multiaddr>, Self::Error> {
        let pubkey = hopr_transport::peer_id_to_public_key(peer).map_err(HoprLibError::TransportError)?;

        match self
            .chain_api
            .stream_accounts(AccountSelector {
                public_only: false,
                offchain_key: Some(pubkey),
                ..Default::default()
            })
            .map_err(HoprLibError::chain)?
            .next()
            .await
        {
            Some(entry) => Ok(entry.get_multiaddrs().to_vec()),
            None => {
                error!(%peer, "no information");
                Ok(vec![])
            }
        }
    }

    async fn ping(&self, peer: &PeerId) -> Result<(Duration, Self::TransportObservable), Self::Error> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;
        let pubkey = hopr_transport::peer_id_to_public_key(peer).map_err(HoprLibError::TransportError)?;
        Ok(self.transport_api.ping(&pubkey).await?)
    }
}

impl<Chain, Graph, Net> Hopr<Chain, Graph, Net>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: hopr_api::graph::NetworkGraphView<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
{
    // Minimum grace period left in PendingToClose channel, so
    // that ticket redemption should still be attempted.
    const PENDING_TO_CLOSE_TOLERANCE: Duration = Duration::from_secs(5);

    pub fn me_onchain(&self) -> Address {
        *self.chain_api.me()
    }

    pub fn get_safe_config(&self) -> SafeModuleConfig {
        SafeModuleConfig {
            safe_address: self.cfg.safe_module.safe_address,
            module_address: self.cfg.safe_module.module_address,
        }
    }

    pub async fn get_balance<C: Currency + Send>(&self) -> Result<Balance<C>, HoprLibError> {
        self.chain_api
            .balance(self.me_onchain())
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn get_safe_balance<C: Currency + Send>(&self) -> Result<Balance<C>, HoprLibError> {
        self.chain_api
            .balance(self.cfg.safe_module.safe_address)
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn safe_allowance(&self) -> Result<HoprBalance, HoprLibError> {
        self.chain_api
            .safe_allowance(self.cfg.safe_module.safe_address)
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn chain_info(&self) -> Result<ChainInfo, HoprLibError> {
        self.chain_api.chain_info().await.map_err(HoprLibError::chain)
    }

    pub async fn get_ticket_price(&self) -> Result<HoprBalance, HoprLibError> {
        self.chain_api.minimum_ticket_price().await.map_err(HoprLibError::chain)
    }

    pub async fn get_minimum_incoming_ticket_win_probability(&self) -> Result<WinningProbability, HoprLibError> {
        self.chain_api
            .minimum_incoming_ticket_win_prob()
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn get_channel_closure_notice_period(&self) -> Result<Duration, HoprLibError> {
        self.chain_api
            .channel_closure_notice_period()
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn announced_peers(&self) -> Result<Vec<AnnouncedPeer>, HoprLibError> {
        Ok(self
            .chain_api
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .map_err(HoprLibError::chain)?
            .map(|entry| AnnouncedPeer {
                address: entry.chain_addr,
                multiaddresses: entry.get_multiaddrs().to_vec(),
                origin: AnnouncementOrigin::Chain,
            })
            .collect()
            .await)
    }

    pub fn peerid_to_chain_key(&self, peer_id: &PeerId) -> Result<Option<Address>, HoprLibError> {
        let pubkey = hopr_transport::peer_id_to_public_key(peer_id).map_err(HoprLibError::TransportError)?;

        self.chain_api
            .packet_key_to_chain_key(&pubkey)
            .map_err(HoprLibError::chain)
    }

    pub fn chain_key_to_peerid(&self, address: &Address) -> Result<Option<PeerId>, HoprLibError> {
        self.chain_api
            .chain_key_to_packet_key(address)
            .map(|pk| pk.map(|v| v.into()))
            .map_err(HoprLibError::chain)
    }

    pub fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, HoprLibError> {
        self.chain_api.channel_by_id(channel_id).map_err(HoprLibError::chain)
    }

    pub fn channel(&self, src: &Address, dest: &Address) -> Result<Option<ChannelEntry>, HoprLibError> {
        self.chain_api
            .channel_by_parties(src, dest)
            .map_err(HoprLibError::chain)
    }

    pub async fn channels_from(&self, src: &Address) -> Result<Vec<ChannelEntry>, HoprLibError> {
        Ok(self
            .chain_api
            .stream_channels(ChannelSelector::default().with_source(*src).with_allowed_states(&[
                ChannelStatusDiscriminants::Closed,
                ChannelStatusDiscriminants::Open,
                ChannelStatusDiscriminants::PendingToClose,
            ]))
            .map_err(HoprLibError::chain)?
            .collect()
            .await)
    }

    pub async fn channels_to(&self, dest: &Address) -> Result<Vec<ChannelEntry>, HoprLibError> {
        Ok(self
            .chain_api
            .stream_channels(
                ChannelSelector::default()
                    .with_destination(*dest)
                    .with_allowed_states(&[
                        ChannelStatusDiscriminants::Closed,
                        ChannelStatusDiscriminants::Open,
                        ChannelStatusDiscriminants::PendingToClose,
                    ]),
            )
            .map_err(HoprLibError::chain)?
            .collect()
            .await)
    }

    pub async fn all_channels(&self) -> Result<Vec<ChannelEntry>, HoprLibError> {
        Ok(self
            .chain_api
            .stream_channels(ChannelSelector::default().with_allowed_states(&[
                ChannelStatusDiscriminants::Closed,
                ChannelStatusDiscriminants::Open,
                ChannelStatusDiscriminants::PendingToClose,
            ]))
            .map_err(HoprLibError::chain)?
            .collect()
            .await)
    }

    pub async fn open_channel(
        &self,
        destination: &Address,
        amount: HoprBalance,
    ) -> Result<OpenChannelResult, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let channel_id = generate_channel_id(&self.me_onchain(), destination);

        // Subscribe to chain events BEFORE sending the transaction to avoid
        // a race where the event is broadcast before the subscriber activates.
        let (event_awaiter, event_abort) = self.spawn_wait_for_on_chain_event(
            format!("open channel to {destination} ({channel_id})"),
            move |event| matches!(event, ChainEvent::ChannelOpened(c) if c.get_id() == &channel_id),
            ON_CHAIN_RESOLUTION_EVENT_TIMEOUT,
        )?;

        let confirm_awaiter = self
            .chain_api
            .open_channel(destination, amount)
            .await
            .map_err(HoprLibError::chain)?;

        let tx_hash = confirm_awaiter.await.map_err(|e| {
            event_abort.abort();
            HoprLibError::chain(e)
        })?;

        let event = event_awaiter.await?;
        debug!(%event, "open channel event received");

        Ok(OpenChannelResult { tx_hash, channel_id })
    }

    pub async fn fund_channel(&self, channel_id: &ChannelId, amount: HoprBalance) -> Result<Hash, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let channel_id = *channel_id;

        // Subscribe to chain events BEFORE sending the transaction to avoid
        // a race where the event is broadcast before the subscriber activates.
        let (event_awaiter, event_abort) = self.spawn_wait_for_on_chain_event(
            format!("fund channel {channel_id}"),
            move |event| matches!(event, ChainEvent::ChannelBalanceIncreased(c, a) if c.get_id() == &channel_id && a == &amount),
            ON_CHAIN_RESOLUTION_EVENT_TIMEOUT
        )?;

        let confirm_awaiter = self
            .chain_api
            .fund_channel(&channel_id, amount)
            .await
            .map_err(HoprLibError::chain)?;

        let res = confirm_awaiter.await.map_err(|e| {
            event_abort.abort();
            HoprLibError::chain(e)
        })?;

        let event = event_awaiter.await?;
        debug!(%event, "fund channel event received");

        Ok(res)
    }

    pub async fn close_channel_by_id(&self, channel_id: &ChannelId) -> Result<CloseChannelResult, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let channel_id = *channel_id;

        // Subscribe to chain events BEFORE sending the transaction to avoid
        // a race where the event is broadcast before the subscriber activates.
        let (event_awaiter, event_abort) = self.spawn_wait_for_on_chain_event(
            format!("close channel {channel_id}"),
            move |event| {
                matches!(event, ChainEvent::ChannelClosed(c) if c.get_id() == &channel_id)
                    || matches!(event, ChainEvent::ChannelClosureInitiated(c) if c.get_id() == &channel_id)
            },
            ON_CHAIN_RESOLUTION_EVENT_TIMEOUT,
        )?;

        let confirm_awaiter = self
            .chain_api
            .close_channel(&channel_id)
            .await
            .map_err(HoprLibError::chain)?;

        let tx_hash = confirm_awaiter.await.map_err(|e| {
            event_abort.abort();
            HoprLibError::chain(e)
        })?;

        let event = event_awaiter.await?;
        debug!(%event, "close channel event received");

        Ok(CloseChannelResult { tx_hash })
    }

    pub fn ticket_management(&self) -> Result<impl TicketManagement + Clone + Send + 'static, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for transport operations".into())?;

        self.ticket_manager
            .get()
            .cloned()
            .ok_or(HoprLibError::StatusError(HoprStatusError::NotThereYet(
                HoprState::Running,
                "Node is not ready for transport operations".into(),
            )))
    }

    // TODO: this method can be sync unless we allow querying of the redeemed value from Blokli
    pub async fn ticket_statistics(&self) -> Result<ChannelStats, HoprLibError> {
        self.ticket_management()?
            .ticket_stats(None)
            .map_err(HoprLibError::ticket_manager)
    }

    pub async fn redeem_all_tickets<B: Into<HoprBalance> + Send>(
        &self,
        min_value: B,
    ) -> Result<Vec<RedemptionResult>, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let min_value = min_value.into();

        self.ticket_management()?
            .redeem_in_channels(
                self.chain_api.clone(),
                None,
                min_value.into(),
                Some(Self::PENDING_TO_CLOSE_TOLERANCE),
            )
            .await
            .map_err(HoprLibError::ticket_manager)?
            .try_collect::<Vec<_>>()
            .map_err(HoprLibError::ticket_manager)
            .await
    }

    pub async fn redeem_tickets_with_counterparty<B: Into<HoprBalance> + Send>(
        &self,
        counterparty: &Address,
        min_value: B,
    ) -> Result<Vec<RedemptionResult>, HoprLibError> {
        self.redeem_tickets_in_channel(&generate_channel_id(counterparty, &self.me_onchain()), min_value)
            .await
    }

    pub async fn redeem_tickets_in_channel<B: Into<HoprBalance> + Send>(
        &self,
        channel_id: &ChannelId,
        min_value: B,
    ) -> Result<Vec<RedemptionResult>, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let min_value = min_value.into();

        let chain_api = self.chain_api.clone();
        let channel_id = *channel_id;
        let channel = hopr_async_runtime::prelude::spawn_blocking(move || {
            chain_api
                .channel_by_id(&channel_id)
                .map_err(HoprLibError::chain)?
                .ok_or(HoprLibError::GeneralError("channel not found".into()))
        })
        .await
        .map_err(HoprLibError::other)??;

        self.ticket_management()?
            .redeem_in_channels(
                self.chain_api.clone(),
                ChannelSelector::default()
                    .with_source(channel.source)
                    .with_destination(channel.destination)
                    .into(),
                min_value.into(),
                Some(Self::PENDING_TO_CLOSE_TOLERANCE),
            )
            .await
            .map_err(HoprLibError::ticket_manager)?
            .map_err(HoprLibError::ticket_manager)
            .try_collect::<Vec<_>>()
            .await
    }

    pub fn subscribe_ticket_events(&self) -> impl Stream<Item = TicketEvent> + Send + 'static {
        self.ticket_event_subscribers.1.activate_cloned()
    }

    pub async fn withdraw_tokens(&self, recipient: Address, amount: HoprBalance) -> Result<Hash, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.chain_api
            .withdraw(amount, &recipient)
            .and_then(identity)
            .map_err(HoprLibError::chain)
            .await
    }

    pub async fn withdraw_native(&self, recipient: Address, amount: XDaiBalance) -> Result<Hash, HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.chain_api
            .withdraw(amount, &recipient)
            .and_then(identity)
            .map_err(HoprLibError::chain)
            .await
    }
}
