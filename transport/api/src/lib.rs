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
pub mod network_notifier;

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
    db::{HoprDbPeersOperations, HoprDbProtocolOperations, HoprDbTicketOperations, PeerOrigin, PeerStatus},
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
use hopr_transport_identity::multiaddrs::strip_p2p_protocol;
pub use hopr_transport_identity::{Multiaddr, PeerId, Protocol};
use hopr_transport_mixer::MixerConfig;
pub use hopr_transport_network::network::{Health, Network};
use hopr_transport_p2p::HoprSwarm;
use hopr_transport_probe::{
    Probe,
    neighbors::ImmediateNeighborProber,
    ping::{PingConfig, Pinger},
};
pub use hopr_transport_probe::{
    errors::ProbeError,
    ping::PingQueryReplier,
    traits::TrafficGeneration,
    types::{NeighborTelemetry, Telemetry},
};
pub use hopr_transport_protocol::PeerDiscovery;
use hopr_transport_protocol::processor::PacketInteractionConfig;
pub use hopr_transport_session as session;
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport_session::transfer_session;
pub use hopr_transport_session::{
    Capabilities as SessionCapabilities, Capability as SessionCapability, HoprSession, IncomingSession, SESSION_MTU,
    SURB_SIZE, ServiceId, SessionClientConfig, SessionId, SessionTarget, SurbBalancerConfig,
    errors::{SessionManagerError, TransportSessionError},
};
use hopr_transport_session::{DispatchResult, SessionManager, SessionManagerConfig};
#[cfg(feature = "mixer-stream")]
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::{Instrument, debug, error, info, trace, warn};

pub use crate::{config::HoprTransportConfig, helpers::TicketStatistics};
use crate::{constants::SESSION_INITIATION_TIMEOUT_BASE, errors::HoprTransportError, socket::HoprSocket};

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

// TODO (4.1): implement path selector based on probing
/// Currently used implementation of [`PathSelector`](hopr_path::selectors::PathSelector).
type CurrentPathSelector = NoPathSelector;

/// Interface into the physical transport mechanism allowing all off-chain HOPR-related tasks on
/// the transport, as well as off-chain ticket manipulation.
pub struct HoprTransport<Db, R> {
    me: OffchainKeypair,
    me_peerid: PeerId, // Cache to avoid an expensive conversion: OffchainPublicKey -> PeerId
    me_address: Address,
    cfg: HoprTransportConfig,
    db: Db,
    resolver: R,
    ping: Arc<OnceLock<Pinger>>,
    network: Arc<Network<Db>>,
    path_planner: PathPlanner<Db, R, CurrentPathSelector>,
    my_multiaddresses: Vec<Multiaddr>,
    smgr: SessionManager<Sender<(DestinationRouting, ApplicationDataOut)>, Sender<IncomingSession>>,
}

impl<Db, R> HoprTransport<Db, R>
where
    Db: HoprDbTicketOperations + HoprDbPeersOperations + HoprDbProtocolOperations + Clone + Send + Sync + 'static,
    R: ChainReadChannelOperations
        + ChainReadAccountOperations
        + ChainKeyOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub fn new(
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
        cfg: HoprTransportConfig,
        db: Db,
        resolver: R,
        my_multiaddresses: Vec<Multiaddr>,
    ) -> Self {
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
            path_planner: PathPlanner::new(
                me_chain_addr,
                db.clone(),
                resolver.clone(),
                CurrentPathSelector::default(),
            ),
            my_multiaddresses,
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
            resolver,
            cfg,
        }
    }

    /// Execute all processes of the [`HoprTransport`] object.
    ///
    /// This method will spawn the [`crate::HoprTransportProcess::Heartbeat`],
    /// [`crate::HoprTransportProcess::BloomFilterSave`], [`crate::HoprTransportProcess::Swarm`] and session-related
    /// processes and return join handles to the calling function. These processes are not started immediately but
    /// are waiting for a trigger from this piece of code.
    #[allow(clippy::too_many_arguments)]
    pub async fn run<S, Ct>(
        &self,
        cover_traffic: Option<Ct>,
        discovery_updates: S,
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
        Ct: TrafficGeneration + Send + Sync + 'static,
    {
        info!("Loading initial peers from the chain");
        let public_nodes = self
            .resolver
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .await
            .map_err(|e| HoprTransportError::Other(e.into()))?
            .collect::<Vec<_>>()
            .await;

        // Calculate the minimum capacity based on public nodes (each node can generate 2 messages)
        // plus 100 as additional buffer
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
            "Creating internal discovery updates channel"
        );
        let (mut internal_discovery_update_tx, internal_discovery_update_rx) =
            futures::channel::mpsc::channel::<PeerDiscovery>(internal_discovery_updates_capacity);

        let me_peerid = self.me_peerid;
        let network = self.network.clone();
        let discovery_updates = futures_concurrency::stream::StreamExt::merge(
            discovery_updates,
            internal_discovery_update_rx,
        )
        .filter_map(move |event| {
            let network = network.clone();
            async move {
                match event {
                    PeerDiscovery::Announce(peer, multiaddresses) => {
                        debug!(%peer, ?multiaddresses, "processing peer discovery event: Announce");
                        if peer != me_peerid {
                            // decapsulate the `p2p/<peer_id>` to remove duplicities
                            let mas = multiaddresses
                                .into_iter()
                                .map(|ma| strip_p2p_protocol(&ma))
                                .filter(|v| !v.is_empty())
                                .collect::<Vec<_>>();

                            if !mas.is_empty() {
                                if let Err(error) = network.add(&peer, PeerOrigin::NetworkRegistry, mas.clone()).await {
                                    error!(%peer, %error, "failed to add peer to the network");
                                    None
                                } else {
                                    Some(PeerDiscovery::Announce(peer, mas))
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        });

        info!(
            public_nodes = public_nodes.len(),
            "Initializing swarm with peers from chain"
        );

        for node_entry in public_nodes {
            if let AccountType::Announced(multiaddresses) = node_entry.entry_type {
                let peer: PeerId = node_entry.public_key.into();

                debug!(%peer, ?multiaddresses, "Using initial public node");

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

        let transport_layer = HoprSwarm::new(
            (&self.me).into(),
            discovery_updates,
            self.my_multiaddresses.clone(),
            self.cfg.transport.prefer_local_addresses,
        )
        .await;

        let msg_proto_control =
            transport_layer.build_protocol_control(hopr_transport_protocol::CURRENT_HOPR_MSG_PROTOCOL);
        let msg_codec = hopr_transport_protocol::HoprBinaryCodec {};
        let (wire_msg_tx, wire_msg_rx) =
            hopr_transport_protocol::stream::process_stream_protocol(msg_codec, msg_proto_control).await?;

        let _mixing_process_before_sending_out = hopr_async_runtime::prelude::spawn(
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

        let (transport_events_tx, transport_events_rx) =
            futures::channel::mpsc::channel::<hopr_transport_p2p::DiscoveryEvent>(2048);

        let network_clone = self.network.clone();
        spawn(
            transport_events_rx
                .for_each(move |event| {
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
                })
                .inspect(|_| {
                    tracing::warn!(
                        task = "transport events recording",
                        "long-running background task finished"
                    )
                }),
        );

        processes.insert(
            HoprTransportProcess::Medium,
            spawn_as_abortable!(transport_layer.run(transport_events_tx).inspect(|_| tracing::warn!(
                task = %HoprTransportProcess::Medium,
                "long-running background task finished"
            ))),
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
            "Creating protocol bidirectional channel"
        );
        let (tx_from_protocol, rx_from_protocol) =
            channel::<(HoprPseudonym, ApplicationDataIn)>(msg_protocol_bidirectional_channel_capacity);

        // We have to resolve DestinationRouting -> ResolvedTransportRouting before
        // sending the external packets to the transport pipeline.
        let path_planner = self.path_planner.clone();
        let distress_threshold = self.db.get_surb_config().distress_threshold;
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

        for (k, v) in hopr_transport_protocol::run_msg_ack_protocol(
            packet_cfg,
            self.db.clone(),
            self.resolver.clone(),
            (
                mixing_channel_tx.with(|(peer, msg): (PeerId, Box<[u8]>)| {
                    trace!(%peer, "sending message to peer");
                    futures::future::ok::<_, hopr_transport_mixer::channel::SenderError>((peer, msg))
                }),
                wire_msg_rx.inspect(|(peer, _)| trace!(%peer, "received message from peer")),
            ),
            (tx_from_protocol, all_resolved_external_msg_rx),
        )
        .await
        .into_iter()
        {
            processes.insert(HoprTransportProcess::Protocol(k), v);
        }

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
        let (manual_ping_tx, manual_ping_rx) = channel::<(PeerId, PingQueryReplier)>(manual_ping_channel_capacity);

        let probe = Probe::new(self.cfg.probe);
        let probing_processes = if let Some(ct) = cover_traffic {
            probe
                .continuously_scan(
                    (unresolved_routing_msg_tx.clone(), rx_from_protocol),
                    manual_ping_rx,
                    tx_from_probing,
                    ct,
                )
                .await
        } else {
            probe
                .continuously_scan(
                    (unresolved_routing_msg_tx.clone(), rx_from_protocol),
                    manual_ping_rx,
                    tx_from_probing,
                    ImmediateNeighborProber::new(
                        self.cfg.probe,
                        network_notifier::ProbeNetworkInteractions::new(self.network.clone()),
                    ),
                )
                .await
        };

        for (k, v) in probing_processes.into_iter() {
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
        self.smgr
            .start(unresolved_routing_msg_tx.clone(), on_incoming_session)
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
            spawn_as_abortable!(
                StreamExt::filter_map(rx_from_probing, move |(pseudonym, data)| {
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
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<(std::time::Duration, PeerStatus)> {
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
        let ticket_stats = self
            .db
            .get_ticket_statistics(None)
            .await
            .map_err(|e| HoprTransportError::Other(e.into()))?;

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
            .resolver
            .channel_by_id(channel_id)
            .await
            .map_err(|e| HoprTransportError::Other(e.into()))?
        {
            if channel.destination == self.me_address {
                Ok(Some(
                    self.db
                        .stream_tickets(Some((&channel).into()))
                        .await
                        .map_err(|e| HoprTransportError::Other(e.into()))?
                        .collect()
                        .await,
                ))
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
            .stream_tickets(None)
            .await
            .map_err(|e| HoprTransportError::Other(e.into()))?
            .map(|v| v.ticket.leak())
            .collect()
            .await)
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
