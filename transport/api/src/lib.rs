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
pub mod helpers;
pub mod network_notifier;
/// Objects used and possibly exported by the crate for re-use for transport functionality
pub mod proxy;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
    time::Duration,
};

use async_lock::RwLock;
use constants::MAXIMUM_MSG_OUTGOING_BUFFER_SIZE;
use futures::{
    FutureExt, SinkExt, StreamExt,
    channel::mpsc::{self, Sender, UnboundedReceiver, UnboundedSender, unbounded},
};
use helpers::PathPlanner;
use hopr_async_runtime::{AbortHandle, prelude::spawn, spawn_as_abortable};
use hopr_crypto_packet::prelude::HoprPacket;
pub use hopr_crypto_types::{
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    types::{HalfKeyChallenge, Hash, OffchainPublicKey},
};
use hopr_db_sql::{
    HoprDbAllOperations,
    accounts::ChainOrPacketKey,
    api::tickets::{AggregationPrerequisites, HoprDbTicketOperations},
};
pub use hopr_internal_types::prelude::HoprPseudonym;
use hopr_internal_types::prelude::*;
pub use hopr_network_types::prelude::RoutingOptions;
use hopr_network_types::prelude::{DestinationRouting, ResolvedTransportRouting};
use hopr_path::{
    PathAddressResolver,
    selectors::dfs::{DfsPathSelector, DfsPathSelectorConfig, RandomizedEdgeWeighting},
};
use hopr_primitive_types::prelude::*;
pub use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut, Tag};
use hopr_transport_identity::multiaddrs::strip_p2p_protocol;
pub use hopr_transport_identity::{Multiaddr, PeerId};
use hopr_transport_mixer::MixerConfig;
pub use hopr_transport_network::network::{Health, Network, PeerOrigin, PeerStatus};
use hopr_transport_p2p::{
    HoprSwarm,
    swarm::{TicketAggregationRequestType, TicketAggregationResponseType},
};
use hopr_transport_probe::{
    DbProxy, Probe,
    ping::{PingConfig, Pinger},
};
pub use hopr_transport_probe::{errors::ProbeError, ping::PingQueryReplier};
pub use hopr_transport_protocol::{PeerDiscovery, execute_on_tick};
use hopr_transport_protocol::{
    errors::ProtocolError,
    processor::{MsgSender, PacketInteractionConfig, PacketSendFinalizer, SendMsgInput},
};
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport_session::transfer_session;
pub use hopr_transport_session::{
    Capabilities as SessionCapabilities, Capability as SessionCapability, HoprSession, IncomingSession, SESSION_MTU,
    SURB_SIZE, ServiceId, SessionClientConfig, SessionId, SessionTarget, SurbBalancerConfig,
    errors::{SessionManagerError, TransportSessionError},
};
use hopr_transport_session::{DispatchResult, SessionManager, SessionManagerConfig};
use hopr_transport_ticket_aggregation::{
    AwaitingAggregator, TicketAggregationActions, TicketAggregationError, TicketAggregationInteraction,
    TicketAggregatorTrait,
};
use rand::seq::SliceRandom;
#[cfg(feature = "mixer-stream")]
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::{debug, error, info, trace, warn};

pub use crate::{
    config::HoprTransportConfig,
    helpers::{PeerEligibility, TicketStatistics},
};
use crate::{constants::SESSION_INITIATION_TIMEOUT_BASE, errors::HoprTransportError, helpers::run_packet_planner};

pub const APPLICATION_TAG_RANGE: std::ops::Range<Tag> = Tag::APPLICATION_TAG_RANGE;

#[cfg(any(
    all(feature = "mixer-channel", feature = "mixer-stream"),
    all(not(feature = "mixer-channel"), not(feature = "mixer-stream"))
))]
compile_error!("Exactly one of the 'mixer-channel' or 'mixer-stream' features must be specified");

// Needs lazy-static, since Duration multiplication by a constant is yet not a const-operation.
lazy_static::lazy_static! {
    static ref SESSION_INITIATION_TIMEOUT_MAX: std::time::Duration = 2 * constants::SESSION_INITIATION_TIMEOUT_BASE * RoutingOptions::MAX_INTERMEDIATE_HOPS as u32;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, strum::Display)]
pub enum HoprTransportProcess {
    #[strum(to_string = "component responsible for the transport medium (libp2p swarm)")]
    Medium,
    #[strum(to_string = "HOPR protocol ({0})")]
    Protocol(hopr_transport_protocol::ProtocolProcesses),
    #[strum(to_string = "session manager sub-process #{0}")]
    SessionsManagement(usize),
    #[strum(to_string = "network probing sub-process: {0}")]
    Probing(hopr_transport_probe::HoprProbeProcess),
}

#[derive(Debug, Clone)]
pub struct TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    db: Db,
    maybe_writer: Arc<OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    agg_timeout: std::time::Duration,
}

impl<Db> TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    pub fn new(
        db: Db,
        maybe_writer: Arc<
            OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>,
        >,
        agg_timeout: std::time::Duration,
    ) -> Self {
        Self {
            db,
            maybe_writer,
            agg_timeout,
        }
    }
}

#[async_trait::async_trait]
impl<Db> TicketAggregatorTrait for TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    async fn aggregate_tickets(
        &self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> hopr_transport_ticket_aggregation::Result<()> {
        if let Some(writer) = self.maybe_writer.clone().get() {
            AwaitingAggregator::new(self.db.clone(), writer.clone(), self.agg_timeout)
                .aggregate_tickets(channel, prerequisites)
                .await
        } else {
            Err(TicketAggregationError::TransportError(
                "Ticket aggregation writer not available, the object was not yet initialized".to_string(),
            ))
        }
    }
}

/// Currently used implementation of [`PathSelector`](hopr_path::selectors::PathSelector).
type CurrentPathSelector = DfsPathSelector<RandomizedEdgeWeighting>;

/// Interface into the physical transport mechanism allowing all off-chain HOPR-related tasks on
/// the transport, as well as off-chain ticket manipulation.
pub struct HoprTransport<T>
where
    T: HoprDbAllOperations + PathAddressResolver + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    me: OffchainKeypair,
    me_peerid: PeerId, // Cache to avoid an expensive conversion: OffchainPublicKey -> PeerId
    me_address: Address,
    cfg: HoprTransportConfig,
    db: T,
    ping: Arc<OnceLock<Pinger>>,
    network: Arc<Network<T>>,
    process_packet_send: Arc<OnceLock<MsgSender<Sender<SendMsgInput>>>>,
    path_planner: PathPlanner<T, CurrentPathSelector>,
    my_multiaddresses: Vec<Multiaddr>,
    process_ticket_aggregate:
        Arc<OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    smgr: SessionManager<Sender<(DestinationRouting, ApplicationDataOut)>>,
}

impl<T> HoprTransport<T>
where
    T: HoprDbAllOperations + PathAddressResolver + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    pub fn new(
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
        cfg: HoprTransportConfig,
        db: T,
        channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
        my_multiaddresses: Vec<Multiaddr>,
    ) -> Self {
        let process_packet_send = Arc::new(OnceLock::new());

        let me_peerid: PeerId = me.into();
        let me_chain_addr = me_onchain.public().to_address();

        Self {
            me: me.clone(),
            me_peerid,
            me_address: me_chain_addr,
            ping: Arc::new(OnceLock::new()),
            network: Arc::new(Network::new(
                me_peerid,
                my_multiaddresses.clone(),
                cfg.network.clone(),
                db.clone(),
            )),
            process_packet_send,
            path_planner: PathPlanner::new(
                me_chain_addr,
                db.clone(),
                CurrentPathSelector::new(
                    channel_graph.clone(),
                    DfsPathSelectorConfig {
                        node_score_threshold: cfg.network.node_score_auto_path_threshold,
                        max_first_hop_latency: cfg.network.max_first_hop_latency_threshold,
                        ..Default::default()
                    },
                ),
                channel_graph.clone(),
            ),
            my_multiaddresses,
            process_ticket_aggregate: Arc::new(OnceLock::new()),
            smgr: SessionManager::new(SessionManagerConfig {
                // TODO(v3.1): Use the entire range of tags properly
                session_tag_range: (16..65535),
                maximum_sessions: cfg.session.maximum_sessions as usize,
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
                initiation_timeout_base: SESSION_INITIATION_TIMEOUT_BASE,
                idle_timeout: cfg.session.idle_timeout,
                balancer_sampling_interval: cfg.session.balancer_sampling_interval,
                initial_return_session_egress_rate: 10,
                minimum_surb_buffer_duration: Duration::from_secs(5),
                maximum_surb_buffer_size: db.get_surb_config().rb_capacity,
                // Allow a 10% increase of the target SURB buffer on incoming Sessions
                // if the SURB buffer level has surpassed it by at least 10% in the last 2 minutes.
                growable_target_surb_buffer: Some((Duration::from_secs(120), 0.10)),
            }),
            db,
            cfg,
        }
    }

    /// Execute all processes of the [`crate::HoprTransport`] object.
    ///
    /// This method will spawn the [`crate::HoprTransportProcess::Heartbeat`],
    /// [`crate::HoprTransportProcess::BloomFilterSave`], [`crate::HoprTransportProcess::Swarm`] and session-related
    /// processes and return join handles to the calling function. These processes are not started immediately but
    /// are waiting for a trigger from this piece of code.
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        &self,
        me_onchain: &ChainKeypair,
        public_nodes: Vec<AccountEntry>,
        on_incoming_data: UnboundedSender<ApplicationDataIn>,
        discovery_updates: UnboundedReceiver<PeerDiscovery>,
        on_incoming_session: UnboundedSender<IncomingSession>,
    ) -> crate::errors::Result<HashMap<HoprTransportProcess, AbortHandle>> {
        let (mut internal_discovery_update_tx, internal_discovery_update_rx) =
            futures::channel::mpsc::unbounded::<PeerDiscovery>();

        let network_clone = self.network.clone();
        let db_clone = self.db.clone();
        let me_peerid = self.me_peerid;
        let discovery_updates =
            futures_concurrency::stream::StreamExt::merge(discovery_updates, internal_discovery_update_rx)
                .filter_map(move |event| {
                    let network = network_clone.clone();
                    let db = db_clone.clone();
                    let me = me_peerid;

                    async move {
                        match event {
                            PeerDiscovery::Allow(peer_id) | PeerDiscovery::Ban(peer_id) => {
                                debug!(%peer_id, "processed peer discovery event as a no-op")
                            }
                            PeerDiscovery::Announce(peer, multiaddresses) => {
                                debug!(peer = %peer, ?multiaddresses, "Processing peer discovery event: Announce");
                                if peer != me {
                                    // decapsulate the `p2p/<peer_id>` to remove duplicities
                                    let mas = multiaddresses
                                        .into_iter()
                                        .map(|ma| strip_p2p_protocol(&ma))
                                        .filter(|v| !v.is_empty())
                                        .collect::<Vec<_>>();

                                    if !mas.is_empty() {
                                        return Some(PeerDiscovery::Announce(peer, mas))
                                    }
                                }
                            }
                        }

                        None
                    }
                });

        info!("Loading initial peers from the storage");

        let mut addresses: HashSet<Multiaddr> = HashSet::new();
        for node_entry in public_nodes {
            if let AccountType::Announced { multiaddr, ..} = node_entry.entry_type {
                let peer: PeerId = node_entry.public_key.into();
                let multiaddresses = vec![multiaddr];

                debug!(%peer, ?multiaddresses, "Using initial public node");
                addresses.extend(multiaddresses.clone());

                internal_discovery_update_tx
                    .send(PeerDiscovery::Announce(peer, multiaddresses.clone()))
                    .await
                    .map_err(|e| HoprTransportError::Api(e.to_string()))?;

                internal_discovery_update_tx
                    .send(PeerDiscovery::Allow(peer))
                    .await
                    .map_err(|e| HoprTransportError::Api(e.to_string()))?;
            }
        }

        let mut processes: HashMap<HoprTransportProcess, AbortHandle> = HashMap::new();

        let (external_msg_send, external_msg_rx) =
            mpsc::channel::<(ApplicationDataOut, ResolvedTransportRouting, PacketSendFinalizer)>(
                MAXIMUM_MSG_OUTGOING_BUFFER_SIZE,
            );

        self.process_packet_send
            .clone()
            .set(MsgSender::new(external_msg_send.clone()))
            .expect("must set the packet processing writer only once");

        // -- transport medium
        let mixer_cfg = build_mixer_cfg_from_env();

        #[cfg(feature = "mixer-channel")]
        let (mixing_channel_tx, mixing_channel_rx) = hopr_transport_mixer::channel::<(PeerId, Box<[u8]>)>(mixer_cfg);

        #[cfg(feature = "mixer-stream")]
        let (mixing_channel_tx, mixing_channel_rx) = {
            let (tx, rx) = futures::channel::mpsc::channel::<(PeerId, Box<[u8]>)>(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE);
            let rx = rx.then_concurrent(move |v| {
                let cfg = mixer_cfg;

                async move {
                    let random_delay = cfg.random_delay();
                    trace!(delay_in_ms = random_delay.as_millis(), "Created random mixer delay",);

                    #[cfg(all(feature = "prometheus", not(test)))]
                    hopr_transport_mixer::channel::METRIC_QUEUE_SIZE.decrement(1.0f64);

                    sleep(random_delay).await;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        hopr_transport_mixer::channel::METRIC_QUEUE_SIZE.decrement(1.0f64);

                        let weight = 1.0f64 / cfg.metric_delay_window as f64;
                        hopr_transport_mixer::channel::METRIC_MIXER_AVERAGE_DELAY.set(
                            (weight * random_delay.as_millis() as f64)
                                + ((1.0f64 - weight) * hopr_transport_mixer::channel::METRIC_MIXER_AVERAGE_DELAY.get()),
                        );
                    }

                    v
                }
            });

            (tx, rx)
        };

        let mut transport_layer =
            HoprSwarm::new((&self.me).into(), discovery_updates, self.my_multiaddresses.clone()).await;

        if let Some(port) = self.cfg.protocol.autonat_port {
            transport_layer.run_nat_server(port);
        }

        if addresses.is_empty() {
            warn!("No addresses found in the database, not dialing any NAT servers");
        } else {
            info!(num_addresses = addresses.len(), "Found addresses from the database");
            let mut randomized_addresses: Vec<_> = addresses.into_iter().collect();
            randomized_addresses.shuffle(&mut rand::thread_rng());
            transport_layer.dial_nat_server(randomized_addresses);
        }

        let msg_proto_control =
            transport_layer.build_protocol_control(hopr_transport_protocol::CURRENT_HOPR_MSG_PROTOCOL);
        let msg_codec = hopr_transport_protocol::HoprBinaryCodec {};
        let (wire_msg_tx, wire_msg_rx) =
            hopr_transport_protocol::stream::process_stream_protocol(msg_codec, msg_proto_control).await?;

        let _mixing_process_before_sending_out = hopr_async_runtime::prelude::spawn(
            mixing_channel_rx
                .inspect(|(peer, _)| tracing::trace!(%peer, "moving message from mixer to p2p stream"))
                .map(Ok)
                .forward(wire_msg_tx),
        );

        let (transport_events_tx, transport_events_rx) =
            futures::channel::mpsc::channel::<hopr_transport_p2p::DiscoveryEvent>(1000);

        let network_clone = self.network.clone();
        spawn(transport_events_rx.for_each(move |event| {
            let network = network_clone.clone();

            async move {
                match event {
                    hopr_transport_p2p::DiscoveryEvent::IncomingConnection(peer, multiaddr) => {
                        if let Err(error) = network
                            .add(&peer, PeerOrigin::IncomingConnection, vec![multiaddr])
                            .await
                        {
                            tracing::error!(%peer, %error, "Failed to add incoming connection peer");
                        }
                    }
                    hopr_transport_p2p::DiscoveryEvent::FailedDial(peer) => {
                        if let Err(error) = network
                            .update(&peer, Err(hopr_transport_network::network::UpdateFailure::DialFailure))
                            .await
                        {
                            tracing::error!(%peer, %error, "Failed to update peer status after failed dial");
                        }
                    }
                }
            }
            .inspect(|_| {
                info!(
                    task = "transport event notifier",
                    "long-running background task finished"
                )
            })
        }));

        processes.insert(
            HoprTransportProcess::Medium,
            spawn_as_abortable!(transport_layer.run(transport_events_tx)),
        );

        // -- msg-ack protocol over the wire transport
        let packet_cfg = PacketInteractionConfig {
            packet_keypair: self.me.clone(),
            outgoing_ticket_win_prob: self
                .cfg
                .protocol
                .outgoing_ticket_winning_prob
                .map(WinningProbability::try_from)
                .transpose()?,
            outgoing_ticket_price: self.cfg.protocol.outgoing_ticket_price,
        };

        let (tx_from_protocol, rx_from_protocol) = unbounded::<(HoprPseudonym, ApplicationDataIn)>();
        for (k, v) in hopr_transport_protocol::run_msg_ack_protocol(
            packet_cfg,
            self.db.clone(),
            (
                mixing_channel_tx.with(|(peer, msg): (PeerId, Box<[u8]>)| {
                    trace!(%peer, "sending message to peer");
                    futures::future::ok::<_, hopr_transport_mixer::channel::SenderError>((peer, msg))
                }),
                wire_msg_rx.inspect(|(peer, _)| trace!(%peer, "received message from peer")),
            ),
            (tx_from_protocol, external_msg_rx),
        )
        .await
        .into_iter()
        {
            processes.insert(HoprTransportProcess::Protocol(k), v);
        }

        // -- network probing
        let (tx_from_probing, rx_from_probing) = unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let (manual_ping_tx, manual_ping_rx) = unbounded::<(PeerId, PingQueryReplier)>();

        let probe = Probe::new((*self.me.public(), self.me_address), self.cfg.probe);
        for (k, v) in probe
            .continuously_scan(
                (external_msg_send, rx_from_protocol),
                manual_ping_rx,
                network_notifier::ProbeNetworkInteractions::new(
                    self.network.clone(),
                    self.db.clone(),
                    self.path_planner.channel_graph(),
                ),
                DbProxy::new(self.db.clone()),
                tx_from_probing,
            )
            .await
            .into_iter()
        {
            processes.insert(HoprTransportProcess::Probing(k), v);
        }

        // manual ping
        self.ping
            .clone()
            .set(Pinger::new(
                PingConfig {
                    timeout: self.cfg.probe.timeout,
                },
                manual_ping_tx,
            ))
            .expect("must set the ticket aggregation writer only once");

        // -- session management
        let packet_planner = run_packet_planner(
            self.path_planner.clone(),
            self.process_packet_send
                .get()
                .cloned()
                .expect("packet sender must be set"),
        );

        self.smgr
            .start(packet_planner, on_incoming_session)
            .expect("failed to start session manager")
            .into_iter()
            .enumerate()
            .map(|(i, jh)| (HoprTransportProcess::SessionsManagement(i + 1), jh))
            .for_each(|(k, v)| {
                processes.insert(k, v);
            });

        let smgr = self.smgr.clone();
        processes.insert(
            HoprTransportProcess::SessionsManagement(0),
            spawn_as_abortable!(async move {
                let _the_process_should_not_end = StreamExt::filter_map(rx_from_probing, |(pseudonym, data)| {
                    let smgr = smgr.clone();
                    async move {
                        match smgr.dispatch_message(pseudonym, data).await {
                            Ok(DispatchResult::Processed) => {
                                trace!("message dispatch completed");
                                None
                            }
                            Ok(DispatchResult::Unrelated(data)) => {
                                trace!("unrelated message dispatch completed");
                                Some(data)
                            }
                            Err(e) => {
                                error!(error = %e, "error while processing packet");
                                None
                            }
                        }
                    }
                })
                .map(Ok)
                .forward(on_incoming_data)
                .await;
            }),
        );

        Ok(processes)
    }

    pub fn ticket_aggregator(&self) -> Arc<dyn TicketAggregatorTrait + Send + Sync + 'static> {
        Arc::new(proxy::TicketAggregatorProxy::new(
            self.db.clone(),
            self.process_ticket_aggregate.clone(),
            std::time::Duration::from_secs(15),
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<(std::time::Duration, PeerStatus)> {
        if !self.is_allowed_to_access_network(either::Left(peer)).await? {
            return Err(HoprTransportError::Api(format!(
                "ping to '{peer}' not allowed due to network registry"
            )));
        }

        if peer == &self.me_peerid {
            return Err(HoprTransportError::Api("ping to self does not make sense".into()));
        }

        let pinger = self
            .ping
            .get()
            .ok_or_else(|| HoprTransportError::Api("ping processing is not yet initialized".into()))?;

        if let Err(e) = self.network.add(peer, PeerOrigin::ManualPing, vec![]).await {
            error!(error = %e, "Failed to store the peer observation");
        }

        let latency = (*pinger).ping(*peer).await?;

        let peer_status = self.network.get(peer).await?.ok_or(HoprTransportError::Probe(
            hopr_transport_probe::errors::ProbeError::NonExistingPeer,
        ))?;

        Ok((latency, peer_status))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn new_session(
        &self,
        destination: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
    ) -> errors::Result<HoprSession> {
        Ok(self.smgr.new_session(destination, target, cfg).await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn probe_session(&self, id: &SessionId) -> errors::Result<()> {
        Ok(self.smgr.ping_session(id).await?)
    }

    pub async fn session_surb_balancing_cfg(&self, id: &SessionId) -> errors::Result<Option<SurbBalancerConfig>> {
        Ok(self.smgr.get_surb_balancer_config(id).await?)
    }

    pub async fn update_session_surb_balancing_cfg(
        &self,
        id: &SessionId,
        cfg: SurbBalancerConfig,
    ) -> errors::Result<()> {
        Ok(self.smgr.update_surb_balancer_config(id, cfg).await?)
    }

    #[tracing::instrument(level = "info", skip(self, msg), fields(uuid = uuid::Uuid::new_v4().to_string()))]
    pub async fn send_message(&self, msg: Box<[u8]>, routing: DestinationRouting, tag: Tag) -> errors::Result<()> {
        if let Tag::Reserved(reserved_tag) = tag {
            return Err(HoprTransportError::Api(format!(
                "Application tag must not from range: {:?}, but was {reserved_tag:?}",
                Tag::APPLICATION_TAG_RANGE
            )));
        }

        if msg.len() > HoprPacket::PAYLOAD_SIZE {
            return Err(HoprTransportError::Api(format!(
                "Message exceeds the maximum allowed size of {} bytes",
                HoprPacket::PAYLOAD_SIZE
            )));
        }

        let app_data = ApplicationData::new(tag, msg.into_vec())?;
        let routing = self
            .path_planner
            .resolve_routing(app_data.total_len(), usize::MAX, routing)
            .await?
            .0;

        // Here we do not use msg_sender directly,
        // since it internally follows Session-oriented logic
        let sender = self.process_packet_send.get().ok_or_else(|| {
            HoprTransportError::Api("send msg: failed because message processing is not yet initialized".into())
        })?;

        sender
            .send_packet(ApplicationDataOut::with_no_packet_info(app_data), routing)
            .await
            .map_err(|e| HoprTransportError::Api(format!("send msg failed to enqueue msg: {e}")))?
            .consume_and_wait(crate::constants::PACKET_QUEUE_TIMEOUT_MILLISECONDS)
            .await
            .map_err(|e| HoprTransportError::Api(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        self.network
            .get(&self.me_peerid)
            .await
            .unwrap_or_else(|e| {
                error!(error = %e, "failed to obtain listening multi-addresses");
                None
            })
            .map(|peer| peer.multiaddresses)
            .unwrap_or_default()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn announceable_multiaddresses(&self) -> Vec<Multiaddr> {
        let mut mas = self
            .local_multiaddresses()
            .into_iter()
            .filter(|ma| {
                hopr_transport_identity::multiaddrs::is_supported(ma)
                    && (self.cfg.transport.announce_local_addresses
                        || !hopr_transport_identity::multiaddrs::is_private(ma))
            })
            .map(|ma| strip_p2p_protocol(&ma))
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();

        mas.sort_by(|l, r| {
            let is_left_dns = hopr_transport_identity::multiaddrs::is_dns(l);
            let is_right_dns = hopr_transport_identity::multiaddrs::is_dns(r);

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

    pub fn local_multiaddresses(&self) -> Vec<Multiaddr> {
        self.my_multiaddresses.clone()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.network
            .get(peer)
            .await
            .unwrap_or(None)
            .map(|peer| peer.multiaddresses)
            .unwrap_or(vec![])
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn network_health(&self) -> Health {
        self.network.health().await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn network_connected_peers(&self) -> errors::Result<Vec<PeerId>> {
        Ok(self.network.connected_peers().await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn network_peer_info(&self, peer: &PeerId) -> errors::Result<Option<PeerStatus>> {
        Ok(self.network.get(peer).await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn ticket_statistics(&self) -> errors::Result<TicketStatistics> {
        let ticket_stats = self.db.get_ticket_statistics(None).await?;

        Ok(TicketStatistics {
            winning_count: ticket_stats.winning_tickets,
            unredeemed_value: ticket_stats.unredeemed_value,
            redeemed_value: ticket_stats.redeemed_value,
            neglected_value: ticket_stats.neglected_value,
            rejected_value: ticket_stats.rejected_value,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn tickets_in_channel(&self, channel_id: &Hash) -> errors::Result<Option<Vec<AcknowledgedTicket>>> {
        if let Some(channel) = self
            .db
            .get_channel_by_id(None, channel_id)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?
        {
            let own_address: Address = self
                .db
                .translate_key(None, ChainOrPacketKey::PacketKey(*self.me.public()))
                .await?
                .ok_or_else(|| {
                    HoprTransportError::Api("Failed to translate the off-chain key to on-chain address".into())
                })?
                .try_into()?;

            if channel.destination == own_address {
                Ok(Some(self.db.get_tickets((&channel).into()).await?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn all_tickets(&self) -> errors::Result<Vec<Ticket>> {
        Ok(self
            .db
            .get_all_tickets()
            .await?
            .into_iter()
            .map(|v| v.ticket.leak())
            .collect())
    }
}

fn build_mixer_cfg_from_env() -> MixerConfig {
    let mixer_cfg = MixerConfig {
        min_delay: std::time::Duration::from_millis(
            std::env::var("HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS")
                .map(|v| {
                    v.trim()
                        .parse::<u64>()
                        .unwrap_or(hopr_transport_mixer::config::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS)
                })
                .unwrap_or(hopr_transport_mixer::config::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS),
        ),
        delay_range: std::time::Duration::from_millis(
            std::env::var("HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS")
                .map(|v| {
                    v.trim()
                        .parse::<u64>()
                        .unwrap_or(hopr_transport_mixer::config::HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS)
                })
                .unwrap_or(hopr_transport_mixer::config::HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS),
        ),
        capacity: std::env::var("HOPR_INTERNAL_MIXER_CAPACITY")
            .map(|v| {
                v.trim()
                    .parse::<usize>()
                    .unwrap_or(hopr_transport_mixer::config::HOPR_MIXER_CAPACITY)
            })
            .unwrap_or(hopr_transport_mixer::config::HOPR_MIXER_CAPACITY),
        ..MixerConfig::default()
    };
    debug!(?mixer_cfg, "Mixer configuration");

    mixer_cfg
}
