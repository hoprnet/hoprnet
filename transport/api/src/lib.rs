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
mod helpers;

#[cfg(feature = "capture")]
mod capture;
mod pipeline;
pub mod socket;

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use constants::MAXIMUM_MSG_OUTGOING_BUFFER_SIZE;
use futures::{
    FutureExt, SinkExt, StreamExt,
    channel::mpsc::{Sender, channel},
};
use helpers::PathPlanner;
use hopr_api::{
    chain::{AccountSelector, ChainKeyOperations, ChainReadAccountOperations, ChainReadChannelOperations, ChainValues},
    db::HoprDbTicketOperations,
    graph::{NetworkGraphUpdate, NetworkGraphView, traits::EdgeObservableRead},
    network::{NetworkBuilder, NetworkStreamControl},
};
pub use hopr_api::{
    db::ChannelTicketStatistics,
    network::{Health, traits::NetworkView},
};
use hopr_async_runtime::{AbortableList, prelude::spawn, spawn_as_abortable};
use hopr_crypto_packet::prelude::PacketSignal;
pub use hopr_crypto_types::{
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    types::{HalfKeyChallenge, Hash, OffchainPublicKey},
};
pub use hopr_internal_types::prelude::HoprPseudonym;
use hopr_internal_types::prelude::*;
pub use hopr_network_types::prelude::RoutingOptions;
use hopr_network_types::prelude::{DestinationRouting, *};
use hopr_primitive_types::prelude::*;
pub use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut, Tag};
use hopr_protocol_hopr::MemorySurbStore;
use hopr_transport_identity::multiaddrs::strip_p2p_protocol;
pub use hopr_transport_identity::{Multiaddr, PeerId, Protocol};
use hopr_transport_mixer::MixerConfig;
pub use hopr_transport_probe::{NeighborTelemetry, PathTelemetry, errors::ProbeError, ping::PingQueryReplier};
use hopr_transport_probe::{
    Probe, ProbingTrafficGeneration,
    ping::{PingConfig, Pinger},
};
pub use hopr_transport_protocol::{PeerDiscovery, TicketEvent};
pub use hopr_transport_session as session;
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport_session::transfer_session;
pub use hopr_transport_session::{
    Capabilities as SessionCapabilities, Capability as SessionCapability, HoprSession, IncomingSession, SESSION_MTU,
    SURB_SIZE, ServiceId, SessionClientConfig, SessionId, SessionTarget, SurbBalancerConfig,
    errors::{SessionManagerError, TransportSessionError},
};
use hopr_transport_session::{DispatchResult, SessionManager, SessionManagerConfig};
use tracing::{Instrument, debug, error, info, trace, warn};

pub use crate::config::HoprProtocolConfig;
use crate::{
    constants::SESSION_INITIATION_TIMEOUT_BASE, errors::HoprTransportError, pipeline::HoprPipelineComponents,
    socket::HoprSocket,
};

pub const APPLICATION_TAG_RANGE: std::ops::Range<Tag> = Tag::APPLICATION_TAG_RANGE;

pub use hopr_api as api;

// Needs lazy-static, since Duration multiplication by a constant is yet not a const-operation.
lazy_static::lazy_static! {
    static ref SESSION_INITIATION_TIMEOUT_MAX: std::time::Duration = 2 * constants::SESSION_INITIATION_TIMEOUT_BASE * RoutingOptions::MAX_INTERMEDIATE_HOPS as u32;

    static ref PEER_ID_CACHE: moka::future::Cache<PeerId, OffchainPublicKey> = moka::future::Cache::builder()
        .time_to_idle(Duration::from_mins(15))
        .max_capacity(10_000)
        .build();
}

/// PeerId -> OffchainPublicKey is a CPU-intensive blocking operation.
///
/// This helper uses a cached static object to speed up the lookup and avoid blocking the async
/// runtime on repeated conversions for the same [`PeerId`]s.
pub async fn peer_id_to_public_key(peer_id: &PeerId) -> crate::errors::Result<OffchainPublicKey> {
    PEER_ID_CACHE
        .try_get_with_by_ref(peer_id, async {
            OffchainPublicKey::from_peerid(peer_id).map_err(|e| e.into())
        })
        .await
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
    Pipeline(hopr_transport_protocol::PacketPipelineProcesses),
    #[strum(to_string = "session manager sub-process #{0}")]
    SessionsManagement(usize),
    #[strum(to_string = "network probing sub-process: {0}")]
    Probing(hopr_transport_probe::HoprProbeProcess),
    #[strum(to_string = "sync of outgoing ticket indices")]
    OutgoingIndexSync,
    #[cfg(feature = "capture")]
    #[strum(to_string = "packet capture")]
    Capture,
}

// TODO (4.1): implement path selector based on probing
/// Currently used implementation of [`PathSelector`](hopr_path::selectors::PathSelector).
type CurrentPathSelector = NoPathSelector;

/// Interface into the physical transport mechanism allowing all off-chain HOPR-related tasks on
/// the transport.
pub struct HoprTransport<Chain, Db, Graph, Net>
where
    Graph: NetworkGraphView<NodeId = OffchainPublicKey> + NetworkGraphUpdate + Clone + Send + Sync + 'static,
    Net: NetworkView + NetworkStreamControl + Clone + Send + Sync + 'static,
{
    packet_key: OffchainKeypair,
    chain_key: ChainKeypair,
    db: Db,
    chain_api: Chain,
    ping: Arc<OnceLock<Pinger>>,
    network: Arc<OnceLock<Net>>,
    graph: Graph,
    path_planner: PathPlanner<MemorySurbStore, Chain, CurrentPathSelector>,
    my_multiaddresses: Vec<Multiaddr>,
    smgr: SessionManager<Sender<(DestinationRouting, ApplicationDataOut)>, Sender<IncomingSession>>,
    cfg: HoprProtocolConfig,
}

impl<Chain, Db, Graph, Net> HoprTransport<Chain, Db, Graph, Net>
where
    Db: HoprDbTicketOperations + Clone + Send + Sync + 'static,
    Chain: ChainReadChannelOperations
        + ChainReadAccountOperations
        + ChainKeyOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    Graph: NetworkGraphView<NodeId = OffchainPublicKey> + NetworkGraphUpdate + Clone + Send + Sync + 'static,
    Net: NetworkView + NetworkStreamControl + Clone + Send + Sync + 'static,
{
    pub fn new(
        identity: (&ChainKeypair, &OffchainKeypair),
        resolver: Chain,
        db: Db,
        graph: Graph,
        my_multiaddresses: Vec<Multiaddr>,
        cfg: HoprProtocolConfig,
    ) -> Self {
        Self {
            packet_key: identity.1.clone(),
            chain_key: identity.0.clone(),
            ping: Arc::new(OnceLock::new()),
            network: Arc::new(OnceLock::new()),
            graph,
            path_planner: PathPlanner::new(
                *identity.0.as_ref(),
                MemorySurbStore::new(cfg.packet.surb_store),
                resolver.clone(),
                CurrentPathSelector::default(),
            ),
            my_multiaddresses,
            smgr: SessionManager::new(SessionManagerConfig {
                // TODO(v4.0): Use the entire range of tags properly
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
                maximum_surb_buffer_size: cfg.packet.surb_store.rb_capacity,
                // Allow a 10% increase of the target SURB buffer on incoming Sessions
                // if the SURB buffer level has surpassed it by at least 10% in the last 2 minutes.
                growable_target_surb_buffer: Some((Duration::from_secs(120), 0.10)),
            }),
            db,
            chain_api: resolver,
            cfg,
        }
    }

    /// Execute all processes of the [`HoprTransport`] object.
    ///
    /// This method will spawn the [`crate::HoprTransportProcess::Heartbeat`],
    /// [`crate::HoprTransportProcess::BloomFilterSave`], [`crate::HoprTransportProcess::Swarm`] and session-related
    /// processes and return join handles to the calling function. These processes are not started immediately but
    /// are waiting for a trigger from this piece of code.
    pub async fn run<S, T, Ct, NetBuilder>(
        &self,
        cover_traffic: Ct,
        network_builder: NetBuilder,
        discovery_updates: S,
        ticket_events: T,
        on_incoming_session: Sender<IncomingSession>,
    ) -> errors::Result<(
        HoprSocket<
            futures::channel::mpsc::Receiver<ApplicationDataIn>,
            futures::channel::mpsc::Sender<(DestinationRouting, ApplicationDataOut)>,
        >,
        AbortableList<HoprTransportProcess>,
    )>
    where
        S: futures::Stream<Item = PeerDiscovery> + Send + 'static,
        T: futures::Sink<TicketEvent> + Clone + Send + Unpin + 'static,
        T::Error: std::error::Error,
        Ct: ProbingTrafficGeneration + Send + Sync + 'static,
        NetBuilder: NetworkBuilder<Network = Net> + Send + Sync + 'static,
    {
        info!("loading initial peers from the chain");
        let public_nodes = self
            .chain_api
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .await
            .map_err(|e| HoprTransportError::Other(e.into()))?
            .collect::<Vec<_>>()
            .await;

        // Calculate the minimum capacity based on public nodes (each node can generate 2 messages)
        // plus 100 as an additional buffer
        let minimum_capacity = public_nodes.len().saturating_mul(2).saturating_add(100);

        let internal_discovery_updates_capacity = std::env::var("HOPR_INTERNAL_DISCOVERY_UPDATES_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(2048)
            .max(minimum_capacity);

        debug!(
            capacity = internal_discovery_updates_capacity,
            minimum_required = minimum_capacity,
            "creating internal discovery updates channel"
        );
        let (mut internal_discovery_update_tx, internal_discovery_update_rx) =
            futures::channel::mpsc::channel::<PeerDiscovery>(internal_discovery_updates_capacity);

        let discovery_updates =
            futures_concurrency::stream::StreamExt::merge(discovery_updates, internal_discovery_update_rx);

        info!(
            public_nodes = public_nodes.len(),
            "initializing swarm with peers from chain"
        );

        for node_entry in public_nodes {
            if let AccountType::Announced(multiaddresses) = node_entry.entry_type {
                let peer: PeerId = node_entry.public_key.into();

                debug!(%peer, ?multiaddresses, "using initial public node");

                internal_discovery_update_tx
                    .send(PeerDiscovery::Announce(peer, multiaddresses))
                    .await
                    .map_err(|e| HoprTransportError::Api(e.to_string()))?;
            }
        }

        let mut processes = AbortableList::<HoprTransportProcess>::default();

        let (unresolved_routing_msg_tx, unresolved_routing_msg_rx) =
            channel::<(DestinationRouting, ApplicationDataOut)>(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE);

        // -- transport medium

        // NOTE: Private address filtering is implemented at multiple levels for defense-in-depth:
        // 1. Discovery events are filtered before they reach the transport component
        // 2. SwarmEvent::NewExternalAddrOfPeer events are filtered using is_public_address()
        let allow_private_addresses = self.cfg.transport.prefer_local_addresses;
        let (transport_network, transport_layer_process) = network_builder
            .build(
                &self.packet_key,
                self.my_multiaddresses.clone(),
                hopr_transport_protocol::CURRENT_HOPR_MSG_PROTOCOL,
                allow_private_addresses,
                discovery_updates.filter_map(move |event| async move {
                    match event {
                        PeerDiscovery::Announce(peer, multiaddrs) => {
                            let multiaddrs = multiaddrs
                                .into_iter()
                                .filter(|ma| {
                                    hopr_transport_identity::multiaddrs::is_supported(ma)
                                        && (allow_private_addresses || is_public_address(ma))
                                })
                                .collect::<Vec<_>>();
                            if multiaddrs.is_empty() {
                                None
                            } else {
                                Some(PeerDiscovery::Announce(peer, multiaddrs))
                            }
                        }
                    }
                }),
            )
            .await
            .map_err(|e| HoprTransportError::Api(e.to_string()))?;

        self.network
            .clone()
            .set(transport_network.clone())
            .map_err(|_| HoprTransportError::Api("transport network viewer already set".into()))?;

        let msg_codec = hopr_transport_protocol::HoprBinaryCodec {};
        let (wire_msg_tx, wire_msg_rx) =
            hopr_transport_protocol::stream::process_stream_protocol(msg_codec, transport_network.clone()).await?;

        let (mixing_channel_tx, mixing_channel_rx) =
            hopr_transport_mixer::channel::<(PeerId, Box<[u8]>)>(build_mixer_cfg_from_env());

        // the process is terminated, when the input stream runs out
        let _mixing_process_before_sending_out = spawn(
            mixing_channel_rx
                .inspect(|(peer, _)| tracing::trace!(%peer, "moving message from mixer to p2p stream"))
                .map(Ok)
                .forward(wire_msg_tx)
                .inspect(|_| {
                    tracing::warn!(
                        task = "mixer -> egress process",
                        "long-running background task finished"
                    )
                }),
        );

        processes.insert(
            HoprTransportProcess::Medium,
            spawn_as_abortable!(transport_layer_process().inspect(|_| tracing::warn!(
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

        let (on_incoming_data_tx, on_incoming_data_rx) =
            channel::<ApplicationDataIn>(msg_protocol_bidirectional_channel_capacity);

        debug!(
            capacity = msg_protocol_bidirectional_channel_capacity,
            "creating protocol bidirectional channel"
        );
        let (tx_from_protocol, rx_from_protocol) =
            channel::<(HoprPseudonym, ApplicationDataIn)>(msg_protocol_bidirectional_channel_capacity);

        // We have to resolve DestinationRouting -> ResolvedTransportRouting before
        // sending the external packets to the transport pipeline.
        let path_planner = self.path_planner.clone();
        let distress_threshold = self.cfg.packet.surb_store.distress_threshold;
        let all_resolved_external_msg_rx = unresolved_routing_msg_rx.filter_map(move |(unresolved, mut data)| {
            let path_planner = path_planner.clone();
            async move {
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
                        error!(%error, "failed to resolve routing");
                        None
                    }
                }
            }
            .in_current_span()
        });

        let channels_dst = self
            .chain_api
            .domain_separators()
            .await
            .map_err(HoprTransportError::chain)?
            .channel;

        processes.extend_from(pipeline::run_hopr_packet_pipeline(
            (self.packet_key.clone(), self.chain_key.clone()),
            (mixing_channel_tx, wire_msg_rx),
            (tx_from_protocol, all_resolved_external_msg_rx),
            HoprPipelineComponents {
                ticket_events,
                surb_store: self.path_planner.surb_store.clone(),
                chain_api: self.chain_api.clone(),
                db: self.db.clone(),
            },
            channels_dst,
            self.cfg.packet,
        ));

        // -- network probing
        debug!(
            capacity = msg_protocol_bidirectional_channel_capacity,
            note = "same as protocol bidirectional",
            "Creating probing channel"
        );

        let (tx_from_probing, rx_from_probing) =
            channel::<(HoprPseudonym, ApplicationDataIn)>(msg_protocol_bidirectional_channel_capacity);

        let manual_ping_channel_capacity = std::env::var("HOPR_INTERNAL_MANUAL_PING_CHANNEL_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(128);
        debug!(capacity = manual_ping_channel_capacity, "Creating manual ping channel");
        let (manual_ping_tx, manual_ping_rx) =
            channel::<(OffchainPublicKey, PingQueryReplier)>(manual_ping_channel_capacity);

        let probe = Probe::new(self.cfg.probe);

        let probing_processes = probe
            .continuously_scan(
                (unresolved_routing_msg_tx.clone(), rx_from_protocol),
                manual_ping_rx,
                tx_from_probing,
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
        self.smgr
            .start(unresolved_routing_msg_tx.clone(), on_incoming_session)
            .map_err(|_| HoprTransportError::Api("failed to start session manager".into()))?
            .into_iter()
            .enumerate()
            .map(|(i, jh)| (HoprTransportProcess::SessionsManagement(i + 1), jh))
            .for_each(|(k, v)| {
                processes.insert(k, v);
            });

        let smgr = self.smgr.clone();
        processes.insert(
            HoprTransportProcess::SessionsManagement(0),
            spawn_as_abortable!(
                rx_from_probing
                    .filter_map(move |(pseudonym, data)| {
                        let smgr = smgr.clone();
                        async move {
                            match smgr.dispatch_message(pseudonym, data).await {
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
                            }
                        }
                    })
                    .map(Ok)
                    .forward(on_incoming_data_tx)
                    .inspect(|_| tracing::warn!(
                        task = %HoprTransportProcess::SessionsManagement(0),
                        "long-running background task finished"
                    ))
            ),
        );

        Ok(((on_incoming_data_rx, unresolved_routing_msg_tx).into(), processes))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn ping(&self, peer: &OffchainPublicKey) -> errors::Result<(std::time::Duration, Graph::Observed)> {
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
                hopr_transport_identity::multiaddrs::is_supported(ma)
                    && (self.cfg.transport.announce_local_addresses || is_public_address(ma))
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
            match peer_id_to_public_key(&peer_id).await {
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
    pub fn network_peer_observations(&self, peer: &OffchainPublicKey) -> Option<Graph::Observed> {
        self.graph.edge(self.packet_key.public(), peer)
    }

    /// Get connected peers with quality higher than some value.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn all_network_peers(
        &self,
        minimum_score: f64,
    ) -> errors::Result<Vec<(OffchainPublicKey, Graph::Observed)>> {
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
        capacity: {
            let capacity = std::env::var("HOPR_INTERNAL_MIXER_CAPACITY")
                .ok()
                .and_then(|s| s.trim().parse::<usize>().ok())
                .filter(|&c| c > 0)
                .unwrap_or(hopr_transport_mixer::config::HOPR_MIXER_CAPACITY);
            debug!(capacity = capacity, "Setting mixer capacity");
            capacity
        },
        ..MixerConfig::default()
    };
    debug!(?mixer_cfg, "Mixer configuration");

    mixer_cfg
}
