//! Type-state builder for constructing a [`Hopr`] node.
//!
//! The builder guides construction through a series of mandatory phases:
//!
//! 1. **Identity** — `HoprBuilder` → `HoprBuilder::with_identity`
//! 2. **Configuration** — `HoprBuilderWithIdentity::with_config`
//! 3. **Component factories** — chain API, graph, network, and cover-traffic
//! 4. **Session server** (when the `session-server` feature is enabled) — `HoprBuilderConfigured::with_session_server`
//! 5. **Build** — `build_edge` for an entry/exit node or `build_full` for a relay node with ticket management.
//!
//! # Example
//!
//! ```rust,ignore
//! use hopr_lib::{config::HoprLibConfig, builder::{HoprBuilder, ChainKeypair, OffchainKeypair, Keypair}};
//!
//! let chain_key = ChainKeypair::random();
//! let offchain_key = OffchainKeypair::random();
//! let config = HoprLibConfig::default();
//!
//! let builder = HoprBuilder
//!     .with_identity(&chain_key, &offchain_key)
//!     .with_config(config)
//!     .with_chain_api(|_ctx| { /* ... */ todo!() })
//!     .with_graph(|_ctx| { /* ... */ todo!() })
//!     .with_network(|_ctx| Box::pin(async { /* ... */ Ok(todo!()) }))
//!     .with_cover_traffic(|_ctx| { /* ... */ todo!() });
//! ```

mod chain_wiring;

use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use futures::{FutureExt, StreamExt};
pub use hopr_api::types::crypto::{
    keypairs::Keypair,
    prelude::{ChainKeypair, OffchainKeypair},
};
use hopr_api::{
    chain::{AnnouncementError, HoprChainApi, SafeRegistrationError, StateSyncOptions},
    ct::{CoverTrafficGeneration, ProbingTrafficGeneration},
    graph::HoprGraphApi,
    network::{BoxedProcessFn, NetworkStreamControl, NetworkView},
    node::{AtomicHoprState, HoprState, NodeOnchainIdentity, TicketEvent},
    tickets::{TicketFactory, TicketManagement},
    types::{chain::chain_events::ChainEvent, internal::prelude::ChannelDirection, primitive::prelude::Address},
};
use hopr_transport::{HoprTransport, IncomingSession};
use hopr_utils::{
    network_types::{
        addr::is_public_address,
        crossfire_sink::{CrossfireSink, bounded_sink_channel},
    },
    runtime::{AbortableList, prelude::spawn},
};
use validator::Validate;

use crate::{
    Hopr, HoprLibError, HoprLibProcess, MIN_NATIVE_BALANCE, NODE_READY_TIMEOUT, SUGGESTED_NATIVE_BALANCE,
    config::HoprLibConfig, constants,
};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PROCESS_START_TIME:  hopr_api::types::telemetry::SimpleGauge =  hopr_api::types::telemetry::SimpleGauge::new(
        "hopr_start_time",
        "The unix timestamp in seconds at which the process was started"
    ).unwrap();
    static ref METRIC_HOPR_LIB_VERSION:  hopr_api::types::telemetry::MultiGauge =  hopr_api::types::telemetry::MultiGauge::new(
        "hopr_lib_version",
        "Executed version of hopr-lib",
        &["version"]
    ).unwrap();
    static ref METRIC_HOPR_NODE_INFO:  hopr_api::types::telemetry::MultiGauge =  hopr_api::types::telemetry::MultiGauge::new(
        "hopr_node_addresses",
        "Node on-chain and off-chain addresses",
        &["peerid", "address", "safe_address", "module_address"]
    ).unwrap();
}

const PEER_DISCOVERY_CHANNEL_CAPACITY: usize = 2048;

type PeerDiscoveryRx =
    Arc<parking_lot::Mutex<Option<futures::stream::BoxStream<'static, (hopr_api::PeerId, Vec<hopr_api::Multiaddr>)>>>>;

/// Type-erased factory closure producing `T` from a [`BuildCtx`] reference.
type Factory<T> = Box<dyn FnOnce(&BuildCtx) -> T + Send>;
type AsyncFactory<T> = Box<dyn FnOnce(BuildCtx) -> Pin<Box<dyn Future<Output = T> + Send>> + Send>;

/// Context available to factory closures during the build step.
#[derive(Clone)]
pub struct BuildCtx {
    /// Node's on-chain keypair.
    pub chain_key: ChainKeypair,
    /// Node's off-chain (packet) keypair.
    pub packet_key: OffchainKeypair,
    /// Node configuration.
    pub cfg: HoprLibConfig,
    peer_discovery_rx: PeerDiscoveryRx,
}

impl BuildCtx {
    /// Take the peer-discovery receiver. Returns `Some` on the first call, `None` afterwards.
    pub fn take_peer_discovery_rx(
        &self,
    ) -> Option<futures::stream::BoxStream<'static, (hopr_api::PeerId, Vec<hopr_api::Multiaddr>)>> {
        self.peer_discovery_rx.lock().take()
    }
}

// ---------------------------------------------------------------------------
// Type-state builder phases
// ---------------------------------------------------------------------------

/// Initial builder — forces `with_identity` first.
#[derive(Default)]
pub struct HoprBuilder;

impl HoprBuilder {
    /// Sets the node's on-chain and off-chain identity.
    pub fn with_identity(self, chain_key: &ChainKeypair, offchain_key: &OffchainKeypair) -> HoprBuilderWithIdentity {
        HoprBuilderWithIdentity {
            chain_key: chain_key.clone(),
            packet_key: offchain_key.clone(),
        }
    }
}

/// Builder with identity set — forces `with_config` next.
pub struct HoprBuilderWithIdentity {
    chain_key: ChainKeypair,
    packet_key: OffchainKeypair,
}

impl HoprBuilderWithIdentity {
    /// Sets the node configuration and produces the configured builder.
    pub fn with_config(self, cfg: HoprLibConfig) -> HoprBuilderConfigured {
        let (peer_discovery_tx, peer_discovery_rx) =
            bounded_sink_channel::<(hopr_api::PeerId, Vec<hopr_api::Multiaddr>)>(PEER_DISCOVERY_CHANNEL_CAPACITY);
        HoprBuilderConfigured {
            ctx: BuildCtx {
                chain_key: self.chain_key,
                packet_key: self.packet_key,
                cfg,
                peer_discovery_rx: Arc::new(parking_lot::Mutex::new(Some(peer_discovery_rx))),
            },
            safe_and_module: None,
            chain_factory: None,
            graph_factory: None,
            network_factory: None,
            ct_factory: None,
            peer_discovery_tx,
        }
    }
}

// ---------------------------------------------------------------------------
// HoprBuilderConfigured — stores factories, no session yet
// ---------------------------------------------------------------------------

/// Configured builder accepting factory closures for components.
///
/// When the `session-server` feature is enabled, `with_session_server`
/// must be called before building — it returns a `HoprBuilderWithSession` which
/// has the `build_edge` / `build_full` methods.
///
/// When the feature is disabled, `build_edge` / `build_full` are available directly.
pub struct HoprBuilderConfigured<Chain = (), Graph = (), Net = (), Ct = ()> {
    ctx: BuildCtx,
    safe_and_module: Option<(Address, Address)>,
    chain_factory: Option<Factory<Chain>>,
    graph_factory: Option<Factory<Graph>>,
    network_factory: Option<AsyncFactory<Result<(Net, BoxedProcessFn), HoprLibError>>>,
    ct_factory: Option<Factory<Ct>>,
    peer_discovery_tx: CrossfireSink<(hopr_api::PeerId, Vec<hopr_api::Multiaddr>)>,
}

impl<Chain, Graph, Net, Ct> HoprBuilderConfigured<Chain, Graph, Net, Ct> {
    /// Sets the node Safe and module addresses.
    pub fn with_safe_module(mut self, safe: &Address, module: &Address) -> Self {
        self.safe_and_module = Some((*safe, *module));
        self
    }

    /// Sets the chain API factory.
    pub fn with_chain_api<NewChain>(
        self,
        f: impl FnOnce(&BuildCtx) -> NewChain + Send + 'static,
    ) -> HoprBuilderConfigured<NewChain, Graph, Net, Ct> {
        HoprBuilderConfigured {
            ctx: self.ctx,
            safe_and_module: self.safe_and_module,
            chain_factory: Some(Box::new(f)),
            graph_factory: self.graph_factory,
            network_factory: self.network_factory,
            ct_factory: self.ct_factory,
            peer_discovery_tx: self.peer_discovery_tx,
        }
    }

    /// Sets the graph factory.
    pub fn with_graph<NewGraph>(
        self,
        f: impl FnOnce(&BuildCtx) -> NewGraph + Send + 'static,
    ) -> HoprBuilderConfigured<Chain, NewGraph, Net, Ct> {
        HoprBuilderConfigured {
            ctx: self.ctx,
            safe_and_module: self.safe_and_module,
            chain_factory: self.chain_factory,
            graph_factory: Some(Box::new(f)),
            network_factory: self.network_factory,
            ct_factory: self.ct_factory,
            peer_discovery_tx: self.peer_discovery_tx,
        }
    }

    /// Sets the network factory. Must resolve to `Ok((Net, BoxedProcessFn))`.
    ///
    /// The factory receives [`BuildCtx`] by value and returns a boxed future,
    /// allowing async network construction without blocking the executor.
    /// Failures are propagated as [`HoprLibError`] during the build step.
    pub fn with_network<NewNet>(
        self,
        f: impl FnOnce(BuildCtx) -> Pin<Box<dyn Future<Output = Result<(NewNet, BoxedProcessFn), HoprLibError>> + Send>>
        + Send
        + 'static,
    ) -> HoprBuilderConfigured<Chain, Graph, NewNet, Ct> {
        HoprBuilderConfigured {
            ctx: self.ctx,
            safe_and_module: self.safe_and_module,
            chain_factory: self.chain_factory,
            graph_factory: self.graph_factory,
            network_factory: Some(Box::new(f)),
            ct_factory: self.ct_factory,
            peer_discovery_tx: self.peer_discovery_tx,
        }
    }

    /// Sets the cover traffic factory.
    pub fn with_cover_traffic<NewCt>(
        self,
        f: impl FnOnce(&BuildCtx) -> NewCt + Send + 'static,
    ) -> HoprBuilderConfigured<Chain, Graph, Net, NewCt> {
        HoprBuilderConfigured {
            ctx: self.ctx,
            safe_and_module: self.safe_and_module,
            chain_factory: self.chain_factory,
            graph_factory: self.graph_factory,
            network_factory: self.network_factory,
            ct_factory: Some(Box::new(f)),
            peer_discovery_tx: self.peer_discovery_tx,
        }
    }

    /// Attaches a session server for handling incoming sessions.
    ///
    /// Eagerly spawns the server task and returns a [`HoprBuilderWithSession`]
    /// that has the `build_edge` / `build_full` methods.
    #[cfg(feature = "session-server")]
    pub fn with_session_server(
        self,
        server: impl hopr_api::node::HoprSessionServer<Session = IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + 'static,
    ) -> HoprBuilderWithSession<Chain, Graph, Net, Ct> {
        let incoming_session_capacity = self.ctx.cfg.incoming_session_capacity.max(1);

        let (session_tx, session_rx) = futures::channel::mpsc::channel::<IncomingSession>(incoming_session_capacity);

        tracing::debug!(capacity = incoming_session_capacity, "spawning session server");
        let session_diag = hopr_utils::runtime::diagnostics::ConcurrentDiagnostics::new(
            "session_server_for_each_concurrent",
            module_path!(),
            file!(),
            line!(),
        );
        let handle = hopr_utils::spawn_as_abortable_named!(
            "hopr_lib_session_server",
            session_rx
                .for_each_concurrent(None, move |session| {
                    let server = server.clone();
                    let session_diag = session_diag.clone();
                    session_diag.wrap(|| async move {
                        let session_id = *session.session.id();
                        match server.process(session).await {
                            Ok(()) => tracing::debug!(?session_id, "session processed successfully"),
                            Err(error) => {
                                tracing::error!(?session_id, %error, "session processing failed")
                            }
                        }
                    })
                })
                .inspect(|_| tracing::warn!(
                    task = %HoprLibProcess::SessionServer,
                    "long-running background task finished"
                ))
        );

        HoprBuilderWithSession {
            inner: self,
            session_tx,
            session_handle: handle,
        }
    }
}

// ---------------------------------------------------------------------------
// HoprBuilderWithSession — session server attached, ready to build
// ---------------------------------------------------------------------------

/// Builder with a session server attached. Has `build_edge` / `build_full`.
///
/// Only exists when the `session-server` feature is enabled.
#[cfg(feature = "session-server")]
pub struct HoprBuilderWithSession<Chain = (), Graph = (), Net = (), Ct = ()> {
    inner: HoprBuilderConfigured<Chain, Graph, Net, Ct>,
    session_tx: futures::channel::mpsc::Sender<IncomingSession>,
    session_handle: hopr_utils::runtime::AbortHandle,
}

// ---------------------------------------------------------------------------
// Intermediate pre-build state
// ---------------------------------------------------------------------------

struct PreHopr<Chain, Graph, Net, Ct> {
    chain_id: ChainKeypair,
    transport_id: OffchainKeypair,
    cfg: HoprLibConfig,
    state: Arc<AtomicHoprState>,
    transport_api: HoprTransport<Chain, Graph, Net>,
    chain_api: Chain,

    ticket_event_subscribers: (
        async_broadcast::Sender<TicketEvent>,
        async_broadcast::InactiveReceiver<TicketEvent>,
    ),
    processes: AbortableList<HoprLibProcess>,
    session_tx: futures::channel::mpsc::Sender<IncomingSession>,
    cover_traffic: Ct,
    network: Net,
    network_process: BoxedProcessFn,
}

// ---------------------------------------------------------------------------
// Shared pre_build logic
// ---------------------------------------------------------------------------

/// Drains a HoprSocket reader, discarding all packets and logging throughput every ~60 seconds.
/// Runs until the stream ends (sender side dropped).
async fn drain_incoming_data<S: futures::Stream + Unpin>(mut reader: S) {
    let mut received: u64 = 0;
    let mut last_report = std::time::Instant::now();
    while reader.next().await.is_some() {
        received += 1;
        if last_report.elapsed().as_secs() >= 60 {
            tracing::info!(
                received,
                "incoming-data drain: unrelated packets discarded in last ~1 min"
            );
            received = 0;
            last_report = std::time::Instant::now();
        }
    }
}

async fn pre_build_inner<Chain, Graph, Net, Ct>(
    configured: HoprBuilderConfigured<Chain, Graph, Net, Ct>,
    session_tx: futures::channel::mpsc::Sender<IncomingSession>,
    mut processes: AbortableList<HoprLibProcess>,
) -> Result<PreHopr<Chain, Graph, Net, Ct>, HoprLibError>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = hopr_api::OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
{
    let peer_discovery_tx = Some(configured.peer_discovery_tx);
    let ctx = configured.ctx;
    ctx.cfg.validate()?;

    let chain_api = (configured
        .chain_factory
        .ok_or(HoprLibError::BuilderError("missing chain factory"))?)(&ctx);
    let graph = (configured
        .graph_factory
        .ok_or(HoprLibError::BuilderError("missing graph factory"))?)(&ctx);
    let (network, network_process) =
        (configured
            .network_factory
            .ok_or(HoprLibError::BuilderError("missing network factory"))?)(ctx.clone())
        .await?;
    let cover_traffic = (configured
        .ct_factory
        .ok_or(HoprLibError::BuilderError("missing cover traffic factory"))?)(&ctx);

    let (chain_id, transport_id) = (ctx.chain_key.clone(), ctx.packet_key.clone());

    let transport_api = HoprTransport::new(
        (&chain_id, &transport_id),
        chain_api.clone(),
        graph.clone(),
        vec![(&ctx.cfg.host).try_into().map_err(HoprLibError::TransportError)?],
        ctx.cfg.protocol.clone(),
    )
    .map_err(HoprLibError::TransportError)?;

    #[cfg(all(feature = "telemetry", not(test)))]
    {
        use hopr_api::types::primitive::traits::AsUnixTimestamp;
        METRIC_PROCESS_START_TIME.set(std::time::SystemTime::now().as_unix_timestamp().as_secs_f64());
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

    let me_onchain = chain_id.public().to_address();

    #[cfg(feature = "testing")]
    tracing::warn!("!! FOR TESTING ONLY !! Node is running with some safety checks disabled!");

    tracing::info!(
        address = %me_onchain,
        minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
        "node is not started, please fund this node",
    );

    tracing::info!(
        suggested_minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
        "node about to start, checking for funds",
    );
    let funding_timeout = Duration::from_secs(200);
    crate::helpers::wait_for_balance(*MIN_NATIVE_BALANCE, funding_timeout, me_onchain, &chain_api)
        .await
        .map_err(|_| {
            HoprLibError::InsufficientFunds(format!(
                "failed to fund the node within {} seconds",
                funding_timeout.as_secs()
            ))
        })?;

    tracing::info!("starting HOPR node...");
    let balance: hopr_api::types::primitive::prelude::XDaiBalance =
        chain_api.balance(me_onchain).await.map_err(HoprLibError::chain)?;
    let minimum_balance = *constants::MIN_NATIVE_BALANCE;

    tracing::info!(address = %me_onchain, %balance, %minimum_balance, "node information");

    if balance.le(&minimum_balance) {
        return Err(HoprLibError::InsufficientFunds(
            "cannot start the node without a sufficiently funded wallet".into(),
        ));
    }

    #[cfg(debug_assertions)]
    let skip_protocol_checks = ctx.cfg.disable_protocol_checks;
    #[cfg(not(debug_assertions))]
    let skip_protocol_checks = false;

    let network_min_ticket_price = chain_api.minimum_ticket_price().await.map_err(HoprLibError::chain)?;
    let configured_ticket_price = ctx.cfg.protocol.packet.codec.outgoing_ticket_price;
    if !skip_protocol_checks && configured_ticket_price.is_some_and(|c| c < network_min_ticket_price) {
        return Err(HoprLibError::GeneralError(format!(
            "configured outgoing ticket price < network minimum: {configured_ticket_price:?} < \
             {network_min_ticket_price}"
        )));
    }

    let network_min_win_prob = chain_api
        .minimum_incoming_ticket_win_prob()
        .await
        .map_err(HoprLibError::chain)?;
    let configured_win_prob = ctx.cfg.protocol.packet.codec.outgoing_win_prob;

    if !skip_protocol_checks && configured_win_prob.is_some_and(|c| c.approx_cmp(&network_min_win_prob).is_lt()) {
        return Err(HoprLibError::GeneralError(format!(
            "configured outgoing win probability < network minimum: {configured_win_prob:?} < {network_min_win_prob}"
        )));
    }

    tracing::info!(
        peer_id = %transport_id.public().to_peerid_str(),
        address = %me_onchain,
        version = constants::APP_VERSION,
        "Node information"
    );

    let safe_addr = ctx.cfg.safe_module.safe_address;
    if me_onchain == safe_addr {
        return Err(HoprLibError::GeneralError(
            "cannot use self as staking safe address".into(),
        ));
    }

    tracing::info!(%safe_addr, "registering safe with this node");
    match chain_api.register_safe(&safe_addr).await {
        Ok(awaiter) => {
            awaiter.await.map_err(|error| {
                tracing::error!(%safe_addr, %error, "safe registration failed");
                HoprLibError::chain(error)
            })?;
            tracing::info!(%safe_addr, "safe successfully registered with this node");
        }
        Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) => {
            if registered_safe == safe_addr {
                tracing::info!(%safe_addr, "this safe is already registered with this node");
            } else {
                tracing::error!(%safe_addr, %registered_safe, "node registered with different safe");
                return Err(HoprLibError::GeneralError("node registered with different safe".into()));
            }
        }
        Err(error) => {
            tracing::error!(%safe_addr, %error, "safe registration failed");
            return Err(HoprLibError::chain(error));
        }
    }

    let multiaddresses_to_announce = if ctx.cfg.publish {
        transport_api.announceable_multiaddresses()
    } else {
        Vec::new()
    };

    multiaddresses_to_announce
        .iter()
        .filter(|a| !is_public_address(a))
        .for_each(|multi_addr| tracing::warn!(?multi_addr, "announcing private multiaddress"));

    // Preflight for the on-chain announcement
    if ctx.cfg.publish {
        let key_binding_fee = chain_api.key_binding_fee().await.map_err(HoprLibError::chain)?;
        if !key_binding_fee.is_zero() {
            let safe_hopr_balance: hopr_api::types::primitive::prelude::HoprBalance =
                chain_api.balance(safe_addr).await.map_err(HoprLibError::chain)?;
            // The key-binding is a persistent on-chain state: if this node was already bound in a
            // previous run, the fee has already been paid and we can announce even if the safe has
            // since been drained below the fee. Hence we only wait for funds when the safe is
            // underfunded *and* the node is not yet key-bound.
            if safe_hopr_balance < key_binding_fee
                && chain_api
                    .await_key_binding(transport_id.public(), Duration::from_secs(1))
                    .await
                    .is_err()
            {
                tracing::warn!(
                    %safe_addr,
                    safe_balance = %safe_hopr_balance,
                    required = %key_binding_fee,
                    "the safe does not hold enough wxHOPR to pay the node announcement (key-binding) fee, waiting for \
                     it to be funded before announcing the node",
                );
                let announce_timeout = Duration::from_secs(200);
                crate::helpers::wait_for_balance(key_binding_fee, announce_timeout, safe_addr, &chain_api)
                    .await
                    .map_err(|_| {
                        HoprLibError::InsufficientFunds(format!(
                            "the safe {safe_addr} does not hold enough wxHOPR (needs at least {key_binding_fee}) to \
                             announce the node on chain; fund the safe with wxHOPR and restart the node"
                        ))
                    })?;
            }
        }
    }

    let chain_api_clone = chain_api.clone();
    let me_offchain = *transport_id.public();
    let node_ready = spawn(async move {
        chain_api_clone
            .await_key_binding(&me_offchain, NODE_READY_TIMEOUT)
            .await
    });

    tracing::info!(?multiaddresses_to_announce, "announcing node on chain");
    match chain_api.announce(&multiaddresses_to_announce, &transport_id).await {
        Ok(awaiter) => {
            awaiter.await.map_err(|error| {
                tracing::error!(
                    ?multiaddresses_to_announce,
                    %error,
                    "node announcement failed; this is commonly caused by the safe not holding enough wxHOPR to pay \
                     the key-binding fee",
                );
                HoprLibError::chain(error)
            })?;
            tracing::info!(?multiaddresses_to_announce, "node announced successfully");
        }
        Err(AnnouncementError::AlreadyAnnounced) => {
            tracing::info!("node already announced on chain");
        }
        Err(error) => {
            tracing::error!(%error, "failed to transmit node announcement");
            return Err(HoprLibError::chain(error));
        }
    }

    let this_node_account = node_ready
        .await
        .map_err(HoprLibError::other)?
        .map_err(HoprLibError::chain)?;
    if this_node_account.chain_addr != me_onchain || this_node_account.safe_address.is_none_or(|a| a != safe_addr) {
        tracing::error!(%this_node_account, "account key-binding mismatch");
        return Err(HoprLibError::GeneralError("account key-binding mismatch".into()));
    }

    tracing::info!(%this_node_account, "node account is ready");

    // Network → graph event wiring (subscribe before transport starts)
    {
        let network_events = network.subscribe_network_events();
        let graph_updater = graph.clone();
        spawn(async move {
            network_events
                .for_each(|event| {
                    let graph_updater = graph_updater.clone();
                    async move {
                        let (peer_id, connected) = match event {
                            hopr_api::network::NetworkEvent::PeerConnected(p) => (p, true),
                            hopr_api::network::NetworkEvent::PeerDisconnected(p) => (p, false),
                        };
                        if let Ok(opk) = hopr_api::OffchainPublicKey::from_peerid(&peer_id) {
                            graph_updater.record_edge(hopr_api::graph::MeasurableEdge::<
                                hopr_transport::NeighborTelemetry,
                                hopr_transport::PathTelemetry,
                            >::ConnectionStatus {
                                peer: opk,
                                connected,
                            });
                        } else {
                            tracing::error!(%peer_id, "failed to convert peer ID to public key for graph update");
                        }
                    }
                })
                .await;
        });
    }

    // Chain → graph event wiring
    {
        let chain_events = chain_api
            .subscribe_with_state_sync([StateSyncOptions::PublicAccounts, StateSyncOptions::OpenedChannels])
            .map_err(HoprLibError::chain)?;

        let graph_updater = graph.clone();
        let chain_reader = chain_api.clone();

        let own_chain_addr = me_onchain;
        let own_packet_key = *transport_id.public();

        let ticket_price = Arc::new(parking_lot::RwLock::new(
            chain_reader.minimum_ticket_price().await.unwrap_or_default(),
        ));
        let win_probability = Arc::new(parking_lot::RwLock::new(
            chain_reader
                .minimum_incoming_ticket_win_prob()
                .await
                .unwrap_or_default(),
        ));

        let proc = chain_wiring::process_chain_events(
            chain_reader,
            graph_updater,
            chain_events,
            own_chain_addr,
            own_packet_key,
            ticket_price,
            win_probability,
            peer_discovery_tx,
        )
        .inspect(|_| {
            tracing::warn!(
                task = "chain-to-graph event wiring",
                "long-running background task finished"
            )
        });
        processes.insert(HoprLibProcess::ChannelEvents, hopr_utils::spawn_as_abortable!(proc));
    }

    Ok(PreHopr {
        chain_id,
        transport_id,
        cfg: ctx.cfg,
        state: Arc::new(AtomicHoprState::new(HoprState::Uninitialized)),
        transport_api,
        chain_api,
        ticket_event_subscribers: (new_tickets_tx, new_tickets_rx.deactivate()),
        processes,
        session_tx,
        cover_traffic,
        network,
        network_process,
    })
}

// ---------------------------------------------------------------------------
// Build methods — shared via macro to avoid duplicating edge/full logic
// ---------------------------------------------------------------------------

macro_rules! impl_build_methods {
    () => {
        /// Builds an entry [`Hopr`] node.
        pub async fn build_edge<TFact>(
            self,
            ticket_factory: TFact,
        ) -> Result<Hopr<Chain, Graph, Net, ()>, HoprLibError>
        where
            TFact: TicketFactory + Clone + Send + Sync + 'static,
        {
            let (configured, session_tx, processes) = self.into_parts();
            let pre = pre_build_inner(configured, session_tx, processes).await?;

            tracing::info!("starting transport for edge (entry) node");
            let (socket, transport_processes) = pre
                .transport_api
                .run_entry(
                    pre.cover_traffic,
                    pre.network,
                    pre.network_process,
                    ticket_factory,
                )
                .await?;

            // Drain unrelated packets to avoid missing blackhole
            spawn(drain_incoming_data(socket.reader()));

            let mut processes = pre.processes;
            processes.flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

            let hopr = Hopr {
                chain_id: NodeOnchainIdentity {
                    node_address: pre.chain_id.public().to_address(),
                    safe_address: pre.cfg.safe_module.safe_address,
                    module_address: pre.cfg.safe_module.module_address,
                },
                cfg: pre.cfg,
                state: pre.state.clone(),
                ticket_event_subscribers: pre.ticket_event_subscribers,
                transport_id: pre.transport_id,
                transport_api: pre.transport_api,
                chain_api: pre.chain_api,
                processes,
                ticket_manager: (),
            };

            hopr.state.store(HoprState::Running, std::sync::atomic::Ordering::Relaxed);
            tracing::info!(
                id = %hopr.transport_id.public().to_peerid_str(),
                version = constants::APP_VERSION,
                "EDGE NODE STARTED AND RUNNING"
            );

            Ok(hopr)
        }

        /// Builds a full (relay) [`Hopr`] node.
        pub async fn build_full<TMgr, TFact>(
            self,
            ticket_manager: TMgr,
            ticket_factory: TFact,
        ) -> Result<Hopr<Chain, Graph, Net, TMgr>, HoprLibError>
        where
            TMgr: TicketManagement + Clone + Send + Sync + 'static,
            TFact: TicketFactory + Clone + Send + Sync + 'static,
        {
            let (configured, session_tx, processes) = self.into_parts();
            let pre = pre_build_inner(configured, session_tx, processes).await?;
            let mut processes = pre.processes;

            tracing::info!("starting ticket events processor");
            let (tickets_tx, tickets_rx) = bounded_sink_channel::<TicketEvent>(8192);

            // Need to use DropAbortable, so that the receiver is dropped when aborted and no new items can be sent by the senders.
            let (tickets_rx_stream, tickets_handle) = hopr_utils::runtime::DropAbortable::new(tickets_rx);

            processes.insert(HoprLibProcess::TicketEvents, tickets_handle);
            let new_ticket_tx = pre.ticket_event_subscribers.0.clone();
            let tmgr_clone = ticket_manager.clone();
            spawn(
                hopr_utils::runtime::diagnostics::instrument(
                    tickets_rx_stream
                    .for_each(move |event| {
                        if let TicketEvent::WinningTicket(ticket) = &event
                            && let Err(error) = tmgr_clone.insert_incoming_ticket(**ticket)
                        {
                            tracing::error!(%error, "failed to insert incoming ticket");
                        }
                        if let Err(error) = new_ticket_tx.try_broadcast(event) {
                            tracing::error!(%error, "failed to broadcast ticket event");
                        }
                        futures::future::ready(())
                    })
                    .inspect(|_| {
                        tracing::warn!(task = %HoprLibProcess::TicketEvents, "long-running background task finished")
                    }),
                    "hopr_lib_ticket_events",
                    module_path!(),
                    file!(),
                    line!(),
                ),
            );

            {
                let chain_for_neglect = pre.chain_api.clone();
                let tmgr_for_neglect = ticket_manager.clone();
                let events = pre.chain_api.subscribe().map_err(HoprLibError::chain)?;
                let (neglect_handle, neglect_reg) = hopr_utils::runtime::AbortHandle::new_pair();
                let neglect_task = futures::stream::Abortable::new(
                    events.filter_map(move |event| {
                        futures::future::ready(match event {
                            ChainEvent::ChannelClosed(ch) => Some(ch),
                            _ => None,
                        })
                    }),
                    neglect_reg,
                )
                .for_each(move |closed_channel| {
                    let chain = chain_for_neglect.clone();
                    let tmgr = tmgr_for_neglect.clone();
                    async move {
                        match closed_channel.direction(chain.me()) {
                            Some(ChannelDirection::Incoming) => {
                                match tmgr.neglect_tickets(closed_channel.get_id(), None) {
                                    Ok(neglected) if !neglected.is_empty() => {
                                        tracing::warn!(
                                            num_neglected = neglected.len(),
                                            %closed_channel,
                                            "tickets on incoming closed channel were neglected"
                                        );
                                    }
                                    Ok(_) => {}
                                    Err(error) => {
                                        tracing::error!(
                                            %error, %closed_channel,
                                            "failed to neglect tickets on closed channel"
                                        );
                                    }
                                }
                            }
                            Some(ChannelDirection::Outgoing) => {}
                            _ => {}
                        }
                    }
                })
                .inspect(|_| {
                    tracing::warn!(
                        task = %HoprLibProcess::ChannelClosureNeglect,
                        "channel closure ticket neglect task finished"
                    )
                });
                spawn(hopr_utils::runtime::diagnostics::instrument(
                    neglect_task,
                    "hopr_lib_channel_closure_neglect",
                    module_path!(),
                    file!(),
                    line!(),
                ));
                processes.insert(HoprLibProcess::ChannelClosureNeglect, neglect_handle);
            }

            tracing::info!("starting transport for full (relay) node");
            let (socket, transport_processes) = pre
                .transport_api
                .run_relay(
                    pre.cover_traffic,
                    pre.network,
                    pre.network_process,
                    tickets_tx,
                    ticket_factory,
                    pre.session_tx,
                )
                .await?;
            // Drain unrelated packets to avoid missing blackhole
            spawn(drain_incoming_data(socket.reader()));
            processes.flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

            let hopr = Hopr {
                chain_id: NodeOnchainIdentity {
                    node_address: pre.chain_id.public().to_address(),
                    safe_address: pre.cfg.safe_module.safe_address,
                    module_address: pre.cfg.safe_module.module_address,
                },
                cfg: pre.cfg,
                state: pre.state.clone(),
                ticket_event_subscribers: pre.ticket_event_subscribers,
                transport_id: pre.transport_id,
                transport_api: pre.transport_api,
                chain_api: pre.chain_api,
                processes,
                ticket_manager,
            };

            hopr.state.store(HoprState::Running, std::sync::atomic::Ordering::Relaxed);

            tracing::info!(
                id = %hopr.transport_id.public().to_peerid_str(),
                version = constants::APP_VERSION,
                "FULL NODE STARTED AND RUNNING"
            );

            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_HOPR_NODE_INFO.set(
                &[
                    &hopr.transport_id.public().to_peerid_str(),
                    &hopr.chain_id.node_address.to_string(),
                    &hopr.chain_id.safe_address.to_string(),
                    &hopr.chain_id.module_address.to_string(),
                ],
                1.0,
            );

            Ok(hopr)
        }
    };
}

// When session-server is ON: build methods only on HoprBuilderWithSession
#[cfg(feature = "session-server")]
impl<Chain, Graph, Net, Ct> HoprBuilderWithSession<Chain, Graph, Net, Ct>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = hopr_api::OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
{
    impl_build_methods!();

    fn into_parts(
        self,
    ) -> (
        HoprBuilderConfigured<Chain, Graph, Net, Ct>,
        futures::channel::mpsc::Sender<IncomingSession>,
        AbortableList<HoprLibProcess>,
    ) {
        let mut processes = AbortableList::<HoprLibProcess>::default();
        processes.insert(HoprLibProcess::SessionServer, self.session_handle);
        (self.inner, self.session_tx, processes)
    }
}

// When session-server is OFF: build methods directly on HoprBuilderConfigured
#[cfg(not(feature = "session-server"))]
impl<Chain, Graph, Net, Ct> HoprBuilderConfigured<Chain, Graph, Net, Ct>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = hopr_api::OffchainPublicKey> + Clone + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static,
    Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
{
    impl_build_methods!();

    fn into_parts(
        self,
    ) -> (
        HoprBuilderConfigured<Chain, Graph, Net, Ct>,
        futures::channel::mpsc::Sender<IncomingSession>,
        AbortableList<HoprLibProcess>,
    ) {
        let (tx, _rx) = futures::channel::mpsc::channel::<IncomingSession>(1);
        let processes = AbortableList::<HoprLibProcess>::default();
        (self, tx, processes)
    }
}
