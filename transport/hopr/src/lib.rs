//! The crate aggregates and composes individual transport level objects and functionality
//! into a unified [`crate::HoprTransport`] object with the goal of isolating the transport layer
//! and defining a fully specified transport API.
//!
//! See also the `hopr_protocol_start` crate for details on Start sub-protocol which initiates a Session.
//!
//! As such, the transport layer components should be only those that are directly necessary to:
//!
//! 1. send and receive a packet, acknowledgement or ticket aggregation request
//! 2. send and receive a network telemetry request
//! 3. automate transport level processes
//! 4. algorithms associated with the transport layer operational management
//! 5. interface specifications to allow modular behavioral extensions

/// Configuration of the [crate::HoprTransport].
pub mod config;
/// Constants used and exposed by the crate.
pub mod constants;
/// Errors used by the crate.
pub mod errors;
/// Graph-based path planning for the HOPR transport layer.
pub mod path;
/// Transport binary protocol layer (codec, pipeline, heartbeat, stream).
pub mod protocol;

mod multiaddrs;

#[cfg(feature = "capture")]
mod capture;
mod pipeline;
pub mod socket;

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use constants::MAXIMUM_MSG_OUTGOING_BUFFER_SIZE;
use futures::{FutureExt, SinkExt, StreamExt, channel::mpsc::Sender, stream::select_with_strategy};
pub use hopr_api::{
    Multiaddr, PeerId,
    network::{Health, traits::NetworkView},
    types::{
        crypto::{
            keypairs::{ChainKeypair, Keypair, OffchainKeypair},
            types::{HalfKeyChallenge, Hash, OffchainPublicKey},
        },
        internal::{prelude::HoprPseudonym, routing::RoutingOptions},
    },
};
use hopr_api::{
    chain::{ChainKeyOperations, ChainReadAccountOperations, ChainReadChannelOperations, ChainValues},
    ct::{CoverTrafficGeneration, ProbingTrafficGeneration},
    graph::{NetworkGraphUpdate, NetworkGraphView, traits::EdgeObservableRead},
    network::{BoxedProcessFn, NetworkStreamControl},
    types::primitive::prelude::*,
};
use hopr_crypto_packet::prelude::PacketSignal;
pub use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut, Tag};
use hopr_protocol_hopr::MemorySurbStore;
pub use hopr_transport_probe::{NeighborTelemetry, PathTelemetry, errors::ProbeError, ping::PingQueryReplier};
use hopr_transport_probe::{
    Probe,
    ping::{PingConfig, Pinger},
};
pub use hopr_transport_session as session;
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport_session::transfer_session;
pub use hopr_transport_session::{
    Capabilities as SessionCapabilities, Capability as SessionCapability, HoprSession, IncomingSession, SESSION_MTU,
    SURB_SIZE, ServiceId, SessionClientConfig, SessionId, SessionTarget, SurbBalancerConfig,
    errors::{SessionManagerError, TransportSessionError},
};
use hopr_transport_session::{DispatchResult, SessionManager, SessionManagerConfig};
#[cfg(feature = "telemetry")]
pub use hopr_transport_session::{SessionAckMode, SessionLifecycleState};
pub use hopr_transport_tag_allocator::TagAllocatorConfig;
use hopr_utils::{
    network_types::{
        crossfire_sink::{CrossfireSink, bounded_sink_channel},
        prelude::*,
    },
    runtime::AbortableList,
};
pub use multiaddr::Protocol;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::{Instrument, debug, error, trace, warn};

#[cfg(feature = "runtime-tokio")]
use crate::path::BackgroundPathCacheRefreshable;
pub use crate::{config::HoprProtocolConfig, protocol::PeerProtocolCounterRegistry};
use crate::{
    constants::SESSION_INITIATION_TIMEOUT_BASE,
    errors::HoprTransportError,
    multiaddrs::strip_p2p_protocol,
    path::{HoprGraphPathSelector, PathPlanner},
    pipeline::HoprPacketPipelineBuilder,
    socket::HoprSocket,
};

pub const APPLICATION_TAG_RANGE: std::ops::Range<Tag> = Tag::APPLICATION_TAG_RANGE;

pub use hopr_api as api;
use hopr_api::{
    chain::{ChainReadTicketOperations, ChainWriteTicketOperations},
    tickets::TicketFactory,
    types::internal::routing::DestinationRouting,
};

// Needs lazy-static, since Duration multiplication by a constant is yet not a const-operation.
lazy_static::lazy_static! {
    static ref SESSION_INITIATION_TIMEOUT_MAX: Duration = 2 * SESSION_INITIATION_TIMEOUT_BASE * RoutingOptions::MAX_INTERMEDIATE_HOPS as u32;

    static ref PEER_ID_CACHE: moka::sync::Cache<PeerId, OffchainPublicKey> = moka::sync::Cache::builder()
        .time_to_idle(Duration::from_mins(15))
        .max_capacity(10_000)
        .build();

    static ref RANDOM_DATA: [u8; 400] = hopr_api::types::crypto_random::random_bytes();
}

/// PeerId -> OffchainPublicKey is a CPU-intensive blocking operation.
///
/// This helper uses a cached static object to speed up the lookup and avoid blocking the async
/// runtime on repeated conversions for the same [`PeerId`]s.
pub fn peer_id_to_public_key(peer_id: &PeerId) -> crate::errors::Result<OffchainPublicKey> {
    PEER_ID_CACHE
        .try_get_with_by_ref(peer_id, move || {
            OffchainPublicKey::from_peerid(peer_id).map_err(|e| e.into())
        })
        .map_err(|e: Arc<HoprTransportError>| {
            crate::errors::HoprTransportError::Other(anyhow::anyhow!(
                "failed to convert peer_id ({:?}) to an offchain public key: {e}",
                peer_id
            ))
        })
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, strum::Display)]
pub enum HoprTransportProcess {
    #[strum(to_string = "component responsible for the transport medium (libp2p swarm)")]
    Medium,
    #[strum(to_string = "HOPR packet pipeline ({0})")]
    Pipeline(protocol::PacketPipelineProcesses),
    #[strum(to_string = "session manager sub-process #{0}")]
    SessionsManagement(usize),
    #[strum(to_string = "network probing sub-process: {0}")]
    Probing(hopr_transport_probe::HoprProbeProcess),
    #[cfg(feature = "runtime-tokio")]
    #[strum(to_string = "path cache refresh")]
    PathRefresh,
    #[strum(to_string = "sync of outgoing ticket indices")]
    OutgoingIndexSync,
    #[strum(to_string = "periodic protocol counter flush")]
    CounterFlush,
    #[strum(to_string = "mixer→wire forwarder")]
    MixerForwarder,
    #[cfg(feature = "capture")]
    #[strum(to_string = "packet capture")]
    Capture,
}

/// HOPR protocol specific instantiation of the SessionManager.
type HoprSessionManager = SessionManager<CrossfireSink<(DestinationRouting, ApplicationDataOut)>>;

/// Allows configuration of one specific [`HoprSession`].
///
/// The configurator does not prevent the Session from being closed
/// or the Session manager from being dropped.
#[derive(Debug, Clone)]
pub struct HoprSessionConfigurator {
    id: SessionId,
    // Makes sure configurator does not extend lifetime of the SessionManager.
    smgr: std::sync::Weak<HoprSessionManager>,
}

impl HoprSessionConfigurator {
    /// [`SessionId`] of the session this object can configure.
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Sends a Session Keep-Alive packet over the Session.
    ///
    /// NOTE: This usually carries at least 2 SURBs on the HOPR protocol level and can be
    /// used for manual SURB balancing.
    ///
    /// NOTE: This operation only sends the Session Keep-Alive packet and **DOES NOT** guarantee the other side
    /// has received it.
    pub async fn ping(&self) -> errors::Result<()> {
        Ok(self
            .smgr
            .upgrade()
            .ok_or(HoprTransportError::Other(anyhow::anyhow!("session manager is dropped")))?
            .ping_session(&self.id)
            .await?)
    }

    /// Gets the configuration of the SURB balancer.
    ///
    /// Returns an error if the Session is closed, the Session manager is gone.
    ///
    /// Returns `Ok(None)` if the Session has been created without a SURB balancer.
    pub fn get_surb_balancer_config(&self) -> errors::Result<Option<SurbBalancerConfig>> {
        Ok(self
            .smgr
            .upgrade()
            .ok_or(HoprTransportError::Other(anyhow::anyhow!("session manager is dropped")))?
            .get_surb_balancer_config(&self.id)?)
    }

    /// Updates the configuration of the SURB balancer.
    ///
    /// Returns an error if the Session is closed, the Session manager is gone, or the
    /// Session has been created without a SURB balancer.
    pub fn update_surb_balancer_config(&self, config: SurbBalancerConfig) -> errors::Result<()> {
        Ok(self
            .smgr
            .upgrade()
            .ok_or(HoprTransportError::Other(anyhow::anyhow!("session manager is dropped")))?
            .update_surb_balancer_config(&self.id, config)?)
    }

    /// Explicitly closes the underlying Session in the [`SessionManager`].
    ///
    /// Returns `true` if the session was found and closed, `false` if it was
    /// already gone (or the manager is dropped). Frees the per-session state
    /// (frame reassembly buffers, control channels, …) immediately rather than
    /// waiting for the manager's idle-timeout eviction.
    pub fn close(&self) -> bool {
        match self.smgr.upgrade() {
            Some(smgr) => smgr.close_session(&self.id),
            None => false,
        }
    }
}

/// Interface into the physical transport mechanism allowing all off-chain HOPR-related tasks on
/// the transport.
pub struct HoprTransport<Chain, Graph, Net> {
    packet_key: OffchainKeypair,
    chain_key: ChainKeypair,
    chain_api: Chain,
    ping: Arc<OnceLock<Pinger>>,
    network: Arc<OnceLock<Net>>,
    graph: Graph,
    path_planner: PathPlanner<MemorySurbStore, Chain, HoprGraphPathSelector<Graph>>,
    my_multiaddresses: Vec<Multiaddr>,
    smgr: Arc<HoprSessionManager>,
    session_telemetry_tag_allocator: Arc<dyn hopr_transport_tag_allocator::TagAllocator + Send + Sync>,
    probing_tag_allocator: Arc<dyn hopr_transport_tag_allocator::TagAllocator + Send + Sync>,
    counters: PeerProtocolCounterRegistry,
    cfg: HoprProtocolConfig,
}

impl<Chain, Graph, Net> HoprTransport<Chain, Graph, Net>
where
    Chain: ChainReadChannelOperations
        + ChainReadAccountOperations
        + ChainWriteTicketOperations
        + ChainKeyOperations
        + ChainReadTicketOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    Graph: NetworkGraphView<NodeId = OffchainPublicKey>
        + NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as NetworkGraphView>::Observed: hopr_api::graph::traits::EdgeObservableRead + Send,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Clone + Send + Sync + 'static,
{
    pub fn new(
        identity: (&ChainKeypair, &OffchainKeypair),
        resolver: Chain,
        graph: Graph,
        my_multiaddresses: Vec<Multiaddr>,
        cfg: HoprProtocolConfig,
    ) -> errors::Result<Self> {
        let me_offchain = *identity.1.public();
        let planner_config = cfg.path_planner;
        let selector = HoprGraphPathSelector::new(
            me_offchain,
            graph.clone(),
            planner_config.max_cached_paths,
            planner_config.edge_penalty,
            planner_config.min_ack_rate,
            planner_config.min_paths_anonymity_floor,
        );

        let tag_allocators = hopr_transport_tag_allocator::create_allocators_from_config(&cfg.session.tag_allocator)?;

        let mut session_telemetry_tag_allocator = None;
        let mut probing_tag_allocator = None;
        for (usage, alloc) in tag_allocators {
            match usage {
                // TODO: cleanup of Session tag allocators needed * (#8199)
                hopr_transport_tag_allocator::Usage::Session => {}
                hopr_transport_tag_allocator::Usage::SessionTerminalTelemetry => {
                    session_telemetry_tag_allocator = Some(alloc)
                }
                hopr_transport_tag_allocator::Usage::ProvingTelemetry => probing_tag_allocator = Some(alloc),
            }
        }
        let session_telemetry_tag_allocator = session_telemetry_tag_allocator
            .ok_or_else(|| HoprTransportError::Api("session telemetry tag allocator missing".into()))?;
        let probing_tag_allocator =
            probing_tag_allocator.ok_or_else(|| HoprTransportError::Api("probing tag allocator missing".into()))?;

        Ok(Self {
            packet_key: identity.1.clone(),
            chain_key: identity.0.clone(),
            ping: Arc::new(OnceLock::new()),
            network: Arc::new(OnceLock::new()),
            graph,
            path_planner: PathPlanner::new(
                me_offchain,
                MemorySurbStore::new(cfg.packet.surb_store),
                resolver.clone(),
                selector,
                planner_config,
            ),
            my_multiaddresses,
            smgr: Arc::new(SessionManager::new(SessionManagerConfig {
                frame_mtu: std::env::var("HOPR_SESSION_FRAME_SIZE")
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or_else(|| SessionManagerConfig::default().frame_mtu)
                    .max(ApplicationData::PAYLOAD_SIZE),
                max_frame_timeout: std::env::var("HOPR_SESSION_FRAME_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok().map(Duration::from_millis))
                    .unwrap_or_else(|| SessionManagerConfig::default().max_frame_timeout)
                    .max(Duration::from_millis(100)),
                max_buffered_segments: std::env::var("HOPR_SESSION_MAX_BUFFERED_SEGMENTS")
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or_else(|| SessionManagerConfig::default().max_buffered_segments),
                initiation_timeout_base: SESSION_INITIATION_TIMEOUT_BASE,
                idle_timeout: cfg.session.idle_timeout,
                balancer_sampling_interval: cfg.session.balancer_sampling_interval,
                initial_return_session_egress_rate: 10,
                minimum_surb_buffer_duration: cfg.session.balancer_minimum_surb_buffer_duration,
                maximum_surb_buffer_size: cfg.packet.surb_store.rb_capacity,
                surb_balance_notify_period: cfg.session.surb_balance_notify_period,
                surb_target_notify: true,
                maximum_sessions: cfg.session.maximum_managed_sessions,
                ..Default::default()
            })),
            chain_api: resolver,
            session_telemetry_tag_allocator,
            probing_tag_allocator,
            counters: PeerProtocolCounterRegistry::default(),
            cfg,
        })
    }

    /// Execute all processes of the [`HoprTransport`] object as a **Relay** node.
    ///
    /// Relay nodes run the full packet pipeline including incoming ticket/acknowledgement
    /// processing and require a [`futures::Sink`] for ticket events as well as an
    /// `on_incoming_session` channel from the SessionManager (they can accept incoming sessions).
    pub async fn run_relay<T, TFact, Ct>(
        &self,
        cover_traffic: Ct,
        network: Net,
        network_process: BoxedProcessFn,
        ticket_events: T,
        ticket_factory: TFact,
        on_incoming_session: Sender<IncomingSession>,
    ) -> errors::Result<(
        HoprSocket<
            futures::stream::BoxStream<'static, ApplicationDataIn>,
            CrossfireSink<(DestinationRouting, ApplicationDataOut)>,
        >,
        AbortableList<HoprTransportProcess>,
    )>
    where
        T: futures::Sink<hopr_api::node::TicketEvent> + Clone + Send + Unpin + 'static,
        T::Error: std::error::Error + Clone + Send,
        Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
        TFact: TicketFactory + Clone + Send + Sync + 'static,
    {
        self.run_inner(
            protocol::NodeType::Relay,
            cover_traffic,
            network,
            network_process,
            ticket_events,
            ticket_factory,
            Some(on_incoming_session),
        )
        .await
    }

    /// Execute all processes of the [`HoprTransport`] object as an **Exit** (destination) node.
    ///
    /// Exit nodes do not process tickets but keep the incoming acknowledgement
    /// pipeline running and can accept incoming sessions via SessionManager.
    pub async fn run_exit<TFact, Ct>(
        &self,
        cover_traffic: Ct,
        network: Net,
        network_process: BoxedProcessFn,
        ticket_factory: TFact,
        on_incoming_session: Sender<IncomingSession>,
    ) -> errors::Result<(
        HoprSocket<
            futures::stream::BoxStream<'static, ApplicationDataIn>,
            CrossfireSink<(DestinationRouting, ApplicationDataOut)>,
        >,
        AbortableList<HoprTransportProcess>,
    )>
    where
        Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
        TFact: TicketFactory + Clone + Send + Sync + 'static,
    {
        self.run_inner(
            protocol::NodeType::Exit,
            cover_traffic,
            network,
            network_process,
            futures::sink::drain(),
            ticket_factory,
            Some(on_incoming_session),
        )
        .await
    }

    /// Execute all processes of the [`HoprTransport`] object as an **Entry** (source) node.
    ///
    /// Entry nodes do not process tickets, do not start the incoming acknowledgement
    /// pipeline, and do not accept incoming sessions — therefore, they require neither a
    /// `ticket_events` sink nor an `on_incoming_session` channel.
    pub async fn run_entry<TFact, Ct>(
        &self,
        cover_traffic: Ct,
        network: Net,
        network_process: BoxedProcessFn,
        ticket_factory: TFact,
    ) -> errors::Result<(
        HoprSocket<
            futures::stream::BoxStream<'static, ApplicationDataIn>,
            CrossfireSink<(DestinationRouting, ApplicationDataOut)>,
        >,
        AbortableList<HoprTransportProcess>,
    )>
    where
        Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
        TFact: TicketFactory + Clone + Send + Sync + 'static,
    {
        self.run_inner(
            protocol::NodeType::Entry,
            cover_traffic,
            network,
            network_process,
            futures::sink::drain(),
            ticket_factory,
            None,
        )
        .await
    }

    /// Internal worker driving all node-type variants of `HoprTransport::run_*`.
    ///
    /// Branches on `role`:
    /// - [`protocol::NodeType::Relay`]: full packet pipeline + SessionManager.
    /// - [`protocol::NodeType::Exit`]: ack-drain pipeline + incoming Sessions.
    /// - [`protocol::NodeType::Entry`]: no ack pipeline, no incoming Sessions.
    #[allow(clippy::too_many_arguments)]
    async fn run_inner<T, TFact, Ct>(
        &self,
        role: protocol::NodeType,
        cover_traffic: Ct,
        network: Net,
        network_process: BoxedProcessFn,
        ticket_events: T,
        ticket_factory: TFact,
        on_incoming_session: Option<Sender<IncomingSession>>,
    ) -> errors::Result<(
        HoprSocket<
            futures::stream::BoxStream<'static, ApplicationDataIn>,
            CrossfireSink<(DestinationRouting, ApplicationDataOut)>,
        >,
        AbortableList<HoprTransportProcess>,
    )>
    where
        T: futures::Sink<hopr_api::node::TicketEvent> + Clone + Send + Unpin + 'static,
        T::Error: std::error::Error + Clone + Send,
        Ct: ProbingTrafficGeneration + CoverTrafficGeneration + Send + Sync + 'static,
        TFact: TicketFactory + Clone + Send + Sync + 'static,
    {
        let mut processes = AbortableList::<HoprTransportProcess>::default();

        let (unresolved_routing_msg_tx, unresolved_routing_msg_rx) =
            bounded_sink_channel::<(DestinationRouting, ApplicationDataOut)>(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE);

        // -- transport medium

        let transport_network = network;
        let transport_layer_process = network_process;

        let msg_codec = crate::protocol::HoprBinaryCodec {};
        let (wire_msg_tx, wire_msg_rx) =
            protocol::stream::process_stream_protocol(msg_codec, transport_network.clone(), self.cfg.stream).await?;

        // Shared mixing channel: all per-destination clones of `mixing_channel_tx` push into one
        // heap, so cross-destination packets are mixed together rather than each destination
        // getting its own independent delay queue. The single forwarder task owns the receiver
        // (and therefore the heap timer) — no per-clone waker coordination is needed.
        let mut mixer_cfg = self.cfg.mixer;
        mixer_cfg.metric_delay_window = u64::try_from(5 * mixer_cfg.delay_range.as_millis())
            .unwrap_or(u64::MAX)
            .max(1);
        let (mixing_channel_tx, mix_rx) = hopr_transport_mixer::channel(mixer_cfg);
        let transit_latency_cfg = self.cfg.transit_latency;
        processes.insert(
            HoprTransportProcess::MixerForwarder,
            hopr_utils::spawn_as_abortable!(async move {
                let mut mix_rx = mix_rx;
                let mut wire_sink = wire_msg_tx;

                if let Some(lat) = transit_latency_cfg {
                    // Concurrent transit-latency fan-out: spawn one short-lived task per
                    // packet so each packet ages through its own ~mean delay independently.
                    // A burst of N packets at mean=50 ms takes ~50 ms total, not N×50 ms.
                    //
                    // Without a tokio runtime the latency is silently skipped (pass-through).
                    #[cfg(feature = "runtime-tokio")]
                    {
                        let (tx, rx) = futures::channel::mpsc::unbounded();
                        let fan_out = async move {
                            while let Some(item) = futures::StreamExt::next(&mut mix_rx).await {
                                let mean_us = lat.mean.as_micros() as f64;
                                let std_us = lat.std_dev.as_micros() as f64;
                                let delay_us = if std_us > 0.0 {
                                    use rand_distr::{Distribution, Normal};
                                    Normal::new(mean_us, std_us)
                                        .expect("transit latency Normal params are valid")
                                        .sample(&mut rand::rng())
                                        .max(0.0_f64)
                                } else {
                                    mean_us.max(0.0)
                                };
                                let delay = Duration::from_micros(delay_us as u64);
                                let item_tx = tx.clone();
                                hopr_utils::runtime::prelude::spawn(async move {
                                    if !delay.is_zero() {
                                        futures_timer::Delay::new(delay).await;
                                    }
                                    let _ = item_tx.unbounded_send(item);
                                });
                            }
                            // `tx` drops here; the channel closes once all per-packet tasks send
                        };
                        let fan_in = async move {
                            let mut rx = rx;
                            while let Some(item) = futures::StreamExt::next(&mut rx).await {
                                if wire_sink.send(item).await.is_err() {
                                    tracing::error!(
                                        task = %HoprTransportProcess::MixerForwarder,
                                        "wire sink dropped — discarding transit-delayed packet"
                                    );
                                    break;
                                }
                            }
                        };
                        futures::join!(fan_out, fan_in);
                    }
                    #[cfg(not(feature = "runtime-tokio"))]
                    {
                        let _ = lat;
                        while let Some(item) = futures::StreamExt::next(&mut mix_rx).await {
                            if wire_sink.send(item).await.is_err() {
                                tracing::error!(
                                    task = %HoprTransportProcess::MixerForwarder,
                                    "wire sink dropped — discarding mixed packet"
                                );
                            }
                        }
                    }
                } else {
                    while let Some(item) = futures::StreamExt::next(&mut mix_rx).await {
                        if wire_sink.send(item).await.is_err() {
                            tracing::error!(
                                task = %HoprTransportProcess::MixerForwarder,
                                "wire sink dropped — discarding mixed packet"
                            );
                        }
                    }
                }

                tracing::warn!(
                    task = %HoprTransportProcess::MixerForwarder,
                    "long-running background task finished"
                );
            }),
        );

        // -- path cache background refresh (only when tokio runtime is available)
        #[cfg(feature = "runtime-tokio")]
        processes.insert(
            HoprTransportProcess::PathRefresh,
            hopr_utils::spawn_as_abortable!(self.path_planner.run_background_refresh()),
        );

        processes.insert(
            HoprTransportProcess::Medium,
            hopr_utils::spawn_as_abortable!(transport_layer_process().inspect(|_| tracing::warn!(
                task = %HoprTransportProcess::Medium,
                "long-running background task finished"
            ))),
        );

        let msg_protocol_bidirectional_channel_capacity =
            std::env::var("HOPR_INTERNAL_PROTOCOL_BIDIRECTIONAL_CHANNEL_CAPACITY")
                .ok()
                .and_then(|s| s.trim().parse::<usize>().ok())
                .filter(|&c| c > 0)
                .unwrap_or(16_384);

        debug!(
            capacity = msg_protocol_bidirectional_channel_capacity,
            "creating protocol bidirectional channel"
        );
        let (tx_from_protocol, rx_from_protocol) =
            bounded_sink_channel::<(HoprPseudonym, ApplicationDataIn)>(msg_protocol_bidirectional_channel_capacity);

        // === START === cover traffic control
        // Allocate a cover traffic tag from the session telemetry partition to avoid
        // collisions with session and probing tags.
        let cover_traffic_allocated_tag = self
            .session_telemetry_tag_allocator
            .allocate()
            .ok_or_else(|| HoprTransportError::Api("failed to allocate cover traffic tag".into()))?;
        let cover_traffic_tag: Tag = cover_traffic_allocated_tag.value().into();

        // filter out the known cover traffic not to lose processing time with it
        // The allocated tag is moved into the closure to keep it alive for the transport lifetime.
        let rx_from_protocol = rx_from_protocol.filter_map(move |(pseudonym, data)| {
            let _keep_alive = &cover_traffic_allocated_tag;
            async move { (data.data.application_tag != cover_traffic_tag).then_some((pseudonym, data)) }
        });

        // prepare a cover traffic stream
        let cover_traffic_stream = CoverTrafficGeneration::build(&cover_traffic).filter_map(move |routing| {
            let start =
                hopr_api::types::crypto_random::random_integer(0, Some((RANDOM_DATA.len() - 100) as u64)) as usize;
            let data = &RANDOM_DATA[start..start + 100];

            futures::future::ready(if let Ok(data) = ApplicationData::new(cover_traffic_tag, data) {
                Some((routing, ApplicationDataOut::with_no_packet_info(data)))
            } else {
                tracing::error!("failed to construct cover traffic packet");
                None
            })
        });

        // merge cover traffic with other outgoing data
        let merged_unresolved_output_data =
            select_with_strategy(unresolved_routing_msg_rx, cover_traffic_stream, |_: &mut ()| {
                futures::stream::PollNext::Left
            });

        // === END === cover traffic control

        // We have to resolve DestinationRouting -> ResolvedTransportRouting before
        // sending the external packets to the transport pipeline. Concurrency matches
        // the encoder stage (output_concurrency) to avoid head-of-line blocking on
        // cache-miss path lookups.
        let path_planner = self.path_planner.clone();
        let distress_threshold = self.cfg.packet.surb_store.distress_threshold;
        let routing_concurrency = {
            let avail = std::thread::available_parallelism()
                .ok()
                .map(|n| n.get())
                .unwrap_or(1)
                .max(1)
                * 8;
            self.cfg
                .packet
                .pipeline
                .output_concurrency
                .filter(|&n| n > 0)
                .unwrap_or(avail)
        };
        let all_resolved_external_msg_rx = merged_unresolved_output_data
            .then_concurrent(
                move |(unresolved, mut data)| {
                    let path_planner = path_planner.clone();
                    async move {
                        hopr_utils::parallelize::ROUTING_RESOLUTION_ATTEMPTS
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        trace!(?unresolved, "resolving routing for packet");
                        match path_planner
                            .resolve_routing(data.data.total_len(), data.estimate_surbs_with_msg(), unresolved)
                            .await
                        {
                            Ok((resolved, rem_surbs)) => {
                                // Set the SURB distress/out-of-SURBs flag if applicable.
                                // These flags are translated into HOPR protocol packet signals and are
                                // applicable only on the return path.
                                let mut signals_to_dst = data
                                    .packet_info
                                    .as_ref()
                                    .map(|info| info.signals_to_destination)
                                    .unwrap_or_default();

                                if resolved.is_return() {
                                    signals_to_dst = match rem_surbs {
                                        Some(rem) if (1..distress_threshold.max(2)).contains(&rem) => {
                                            signals_to_dst | PacketSignal::SurbDistress
                                        }
                                        Some(0) => signals_to_dst | PacketSignal::OutOfSurbs,
                                        _ => signals_to_dst - (PacketSignal::OutOfSurbs | PacketSignal::SurbDistress),
                                    };
                                } else {
                                    // Unset these flags as they make no sense on the forward path.
                                    signals_to_dst -= PacketSignal::SurbDistress | PacketSignal::OutOfSurbs;
                                }

                                data.packet_info.get_or_insert_default().signals_to_destination = signals_to_dst;
                                trace!(?resolved, "resolved routing for packet");
                                Some((resolved, data))
                            }
                            Err(error) => {
                                hopr_utils::parallelize::ROUTING_RESOLUTION_FAILURES
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                error!(%error, "failed to resolve routing");
                                None
                            }
                        }
                    }
                    .in_current_span()
                },
                routing_concurrency,
            )
            .filter_map(futures::future::ready);

        let channels_dst = self
            .chain_api
            .domain_separators()
            .await
            .map_err(HoprTransportError::chain)?
            .channel;

        let pipeline_builder = HoprPacketPipelineBuilder::new()
            .identity((&self.chain_key, &self.packet_key))
            .transport((mixing_channel_tx, wire_msg_rx))
            .api((tx_from_protocol, all_resolved_external_msg_rx))
            .surb_store(self.path_planner.surb_store.clone())
            .chain_api(self.chain_api.clone())
            .ticket_factory(ticket_factory)
            .channels_dst(channels_dst)
            .with_counters(self.counters.clone())
            .with_config(self.cfg.packet);

        let pipeline_processes = match role {
            protocol::NodeType::Relay => pipeline_builder.with_ticket_events(ticket_events).build_for_relay(),
            protocol::NodeType::Exit => pipeline_builder.build_for_exit(),
            protocol::NodeType::Entry => pipeline_builder.build_for_entry(),
        };
        processes.extend_from(pipeline_processes);

        // -- periodic counter flush
        let flush_counters = self.counters.clone();
        let flush_graph = self.graph.clone();
        let flush_me = *self.packet_key.public();
        let flush_interval = self.cfg.counter_flush_interval;
        processes.insert(
            HoprTransportProcess::CounterFlush,
            hopr_utils::spawn_as_abortable!(async move {
                use hopr_api::graph::traits::{EdgeObservableWrite, EdgeWeightType};

                futures_time::stream::interval(futures_time::time::Duration::from(flush_interval))
                    .for_each(|_| {
                        for (peer, num_packets, num_acks) in flush_counters.drain() {
                            tracing::trace!(
                                %peer,
                                num_packets,
                                num_acks,
                                "flushing protocol conformance counters"
                            );
                            flush_graph.upsert_edge(&flush_me, &peer, |obs| {
                                obs.record(EdgeWeightType::ImmediateProtocolConformance { num_packets, num_acks });
                            });
                        }
                        futures::future::ready(())
                    })
                    .await;
            }),
        );

        // -- network probing
        let manual_ping_channel_capacity = std::env::var("HOPR_INTERNAL_MANUAL_PING_CHANNEL_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(128);
        debug!(capacity = manual_ping_channel_capacity, "Creating manual ping channel");
        let (manual_ping_tx, manual_ping_rx_raw) =
            crossfire::mpsc::bounded_async::<(OffchainPublicKey, PingQueryReplier)>(manual_ping_channel_capacity);
        let manual_ping_rx = manual_ping_rx_raw.into_stream();

        let probe = Probe::new(self.cfg.probe, self.probing_tag_allocator.clone());

        let (probing_processes, probe_classifier) = probe
            .continuously_scan(
                unresolved_routing_msg_tx.clone(),
                manual_ping_rx,
                cover_traffic,
                self.graph.clone(),
            )
            .await;

        processes.flat_map_extend_from(probing_processes, HoprTransportProcess::Probing);

        // manual ping
        self.ping
            .clone()
            .set(Pinger::new(
                PingConfig {
                    timeout: self.cfg.probe.timeout,
                },
                manual_ping_tx,
            ))
            .map_err(|_| HoprTransportError::Api("must set the ticket aggregation writer only once".into()))?;

        // -- session management
        let smgr_start_res = if role != protocol::NodeType::Entry {
            // Relays and Exits can accept incoming Sessions
            self.smgr.start(
                unresolved_routing_msg_tx.clone(),
                on_incoming_session.ok_or_else(|| {
                    HoprTransportError::Api("on_incoming_session channel is required for relay/exit nodes".into())
                })?,
            )
        } else {
            // Entry nodes cannot accept incoming Sessions
            self.smgr
                .start(unresolved_routing_msg_tx.clone(), futures::sink::drain())
        };

        smgr_start_res
            .map_err(|_| HoprTransportError::Api("failed to start session manager".into()))?
            .into_iter()
            .enumerate()
            .map(|(i, jh)| (HoprTransportProcess::SessionsManagement(i + 1), jh))
            .for_each(|(k, v)| {
                processes.insert(k, v);
            });

        // Wire incoming: cover-traffic-filtered stream → probe classify → (session dispatch).
        // This stage must run in a background task, so the pipeline drains even when the
        // caller discards the returned HoprSocket (e.g. edge-node builder).
        //
        // The channel uses a resilient for_each rather than .forward() so that a disconnected
        // receiver (HoprSocket dropped without consuming) logs an error and continues rather
        // than collapsing the entire ingress pipeline. Callers should use HoprSocket::reader()
        // and actively drain the stream; see hopr-lib builder for the reference drain.
        let (on_incoming_data_tx, on_incoming_data_rx) =
            bounded_sink_channel::<ApplicationDataIn>(msg_protocol_bidirectional_channel_capacity);
        let smgr = self.smgr.clone();
        let unresolved_routing_msg_tx_for_task = unresolved_routing_msg_tx.clone();
        processes.insert(
            HoprTransportProcess::SessionsManagement(0),
            hopr_utils::spawn_as_abortable!(async move {
                probe_classifier
                    .filter_stream(unresolved_routing_msg_tx_for_task, rx_from_protocol)
                    .filter_map(move |(pseudonym, data)| {
                        hopr_utils::parallelize::DISPATCH_MESSAGE_CALLS
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        futures::future::ready(match smgr.dispatch_message(pseudonym, data) {
                            Ok(DispatchResult::Processed) => {
                                tracing::trace!("message dispatch completed");
                                None
                            }
                            Ok(DispatchResult::Unrelated(data)) => {
                                tracing::trace!("unrelated message dispatch completed");
                                Some(data)
                            }
                            Err(error) => {
                                tracing::error!(%error, "error while dispatching packet in the session manager");
                                None
                            }
                        })
                    })
                    .fold(on_incoming_data_tx, |mut tx, data| async move {
                        if tx.send(data).await.is_err() {
                            tracing::error!(
                                task = %HoprTransportProcess::SessionsManagement(0),
                                "incoming-data channel disconnected — dropping unrelated packet; \
                                 HoprSocket must be consumed or drained by the caller"
                            );
                        }
                        tx
                    })
                    .await;
                tracing::warn!(
                    task = %HoprTransportProcess::SessionsManagement(0),
                    "long-running background task finished"
                );
            }),
        );

        // Populate the OnceLock at the end, making sure everything before didn't fail.
        self.network
            .clone()
            .set(transport_network)
            .map_err(|_| HoprTransportError::Api("transport network viewer already set".into()))?;

        Ok((
            (on_incoming_data_rx.boxed(), unresolved_routing_msg_tx).into(),
            processes,
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn ping(
        &self,
        peer: &OffchainPublicKey,
    ) -> errors::Result<(std::time::Duration, <Graph as NetworkGraphView>::Observed)> {
        let me: &OffchainPublicKey = self.packet_key.public();
        if peer == me {
            return Err(HoprTransportError::Api("ping to self does not make sense".into()));
        }

        let pinger = self
            .ping
            .get()
            .ok_or_else(|| HoprTransportError::Api("ping processing is not yet initialized".into()))?;

        let latency = (*pinger).ping(peer).await?;

        if let Some(observations) = self.graph.edge(me, peer) {
            Ok((latency, observations))
        } else {
            Err(HoprTransportError::Api(format!(
                "no observations available for peer {peer}",
            )))
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn new_session(
        &self,
        destination: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
    ) -> errors::Result<(HoprSession, HoprSessionConfigurator)> {
        let session = self.smgr.new_session(destination, target, cfg).await?;
        let id = *session.id();
        Ok((
            session,
            HoprSessionConfigurator {
                id,
                smgr: Arc::downgrade(&self.smgr),
            },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        self.network
            .get()
            .ok_or_else(|| HoprTransportError::Api("transport network is not yet initialized".into()))
            .map(|network| network.listening_as().into_iter().collect())
            .unwrap_or_default()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn announceable_multiaddresses(&self) -> Vec<Multiaddr> {
        let mut mas = self
            .local_multiaddresses()
            .into_iter()
            .filter(|ma| {
                crate::multiaddrs::is_supported(ma)
                    && (self.cfg.transport.announce_local_addresses || is_public_address(ma))
            })
            .map(|ma| strip_p2p_protocol(&ma))
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();

        mas.sort_by(|l, r| {
            let is_left_dns = crate::multiaddrs::is_dns(l);
            let is_right_dns = crate::multiaddrs::is_dns(r);

            if !(is_left_dns ^ is_right_dns) {
                std::cmp::Ordering::Equal
            } else if is_left_dns {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        mas
    }

    /// Returns a reference to the network graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn local_multiaddresses(&self) -> Vec<Multiaddr> {
        self.network
            .get()
            .map(|network| network.listening_as().into_iter().collect())
            .unwrap_or_else(|| {
                tracing::error!("transport network is not yet initialized, cannot fetch announced multiaddresses");
                self.my_multiaddresses.clone()
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn network_observed_multiaddresses(&self, peer: &OffchainPublicKey) -> Vec<Multiaddr> {
        match self
            .network
            .get()
            .ok_or_else(|| HoprTransportError::Api("transport network is not yet initialized".into()))
        {
            Ok(network) => network
                .multiaddress_of(&peer.into())
                .unwrap_or_default()
                .into_iter()
                .collect(),
            Err(error) => {
                tracing::error!(%error, "failed to get observed multiaddresses");
                return vec![];
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn network_health(&self) -> Health {
        self.network
            .get()
            .ok_or_else(|| HoprTransportError::Api("transport network is not yet initialized".into()))
            .map(|network| network.health())
            .unwrap_or(Health::Red)
    }

    pub async fn network_connected_peers(&self) -> errors::Result<Vec<OffchainPublicKey>> {
        Ok(futures::stream::iter(
            self.network
                .get()
                .ok_or_else(|| {
                    tracing::error!("transport network is not yet initialized");
                    HoprTransportError::Api("transport network is not yet initialized".into())
                })?
                .connected_peers(),
        )
        .filter_map(|peer_id| async move {
            match peer_id_to_public_key(&peer_id) {
                Ok(key) => Some(key),
                Err(error) => {
                    tracing::warn!(%peer_id, %error, "failed to convert PeerId to OffchainPublicKey");
                    None
                }
            }
        })
        .collect()
        .await)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn network_peer_observations(&self, peer: &OffchainPublicKey) -> Option<<Graph as NetworkGraphView>::Observed> {
        self.graph.edge(self.packet_key.public(), peer)
    }

    /// Get connected peers with quality higher than some value.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn all_network_peers(
        &self,
        minimum_score: f64,
    ) -> errors::Result<Vec<(OffchainPublicKey, <Graph as NetworkGraphView>::Observed)>> {
        let me = self.packet_key.public();
        Ok(self
            .network_connected_peers()
            .await?
            .into_iter()
            .filter_map(|peer| {
                let observation = self.graph.edge(me, &peer);
                if let Some(info) = observation {
                    if info.score() >= minimum_score {
                        Some((peer, info))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>())
    }
}

// ---------------------------------------------------------------------------
// NetworkView impl for HoprTransport — wraps OnceLock<Net> access
// ---------------------------------------------------------------------------

impl<Chain, Graph, Net> NetworkView for HoprTransport<Chain, Graph, Net>
where
    Net: NetworkView + Send + Sync + 'static,
{
    fn listening_as(&self) -> std::collections::HashSet<Multiaddr> {
        self.network.get().map(|n| n.listening_as()).unwrap_or_default()
    }

    fn multiaddress_of(&self, peer: &PeerId) -> Option<std::collections::HashSet<Multiaddr>> {
        self.network.get()?.multiaddress_of(peer)
    }

    fn discovered_peers(&self) -> std::collections::HashSet<PeerId> {
        self.network.get().map(|n| n.discovered_peers()).unwrap_or_default()
    }

    fn connected_peers(&self) -> std::collections::HashSet<PeerId> {
        self.network.get().map(|n| n.connected_peers()).unwrap_or_default()
    }

    fn is_connected(&self, peer: &PeerId) -> bool {
        self.network.get().map(|n| n.is_connected(peer)).unwrap_or(false)
    }

    fn health(&self) -> Health {
        self.network.get().map(|n| n.health()).unwrap_or(Health::Red)
    }

    fn subscribe_network_events(
        &self,
    ) -> impl futures::Stream<Item = hopr_api::network::NetworkEvent> + Send + 'static {
        match self.network.get() {
            Some(n) => futures::future::Either::Left(n.subscribe_network_events()),
            None => futures::future::Either::Right(futures::stream::empty()),
        }
    }
}

// ---------------------------------------------------------------------------
// TransportOperations impl for HoprTransport
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl<Chain, Graph, Net> hopr_api::node::TransportOperations for HoprTransport<Chain, Graph, Net>
where
    Chain: ChainReadChannelOperations
        + ChainReadAccountOperations
        + hopr_api::chain::ChainWriteTicketOperations
        + ChainKeyOperations
        + hopr_api::chain::ChainReadTicketOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    Graph: NetworkGraphView<NodeId = OffchainPublicKey>
        + NetworkGraphUpdate
        + hopr_api::graph::NetworkGraphWrite<NodeId = OffchainPublicKey>
        + hopr_api::graph::NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <Graph as NetworkGraphView>::Observed: EdgeObservableRead + Send,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
    <Graph as hopr_api::graph::NetworkGraphWrite>::Observed: hopr_api::graph::traits::EdgeObservableWrite + Send,
    Net: NetworkView + NetworkStreamControl + Clone + Send + Sync + 'static,
{
    type Error = errors::HoprTransportError;
    type Observable = <Graph as NetworkGraphView>::Observed;

    async fn ping(&self, key: &OffchainPublicKey) -> Result<(Duration, Self::Observable), Self::Error> {
        self.ping(key).await
    }

    async fn observed_multiaddresses(&self, key: &OffchainPublicKey) -> Vec<Multiaddr> {
        self.network_observed_multiaddresses(key).await
    }
}
