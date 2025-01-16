//! The crate aggregates and composes individual transport level objects and functionality
//! into a unified [`crate::HoprTransport`] object with the goal of isolating the transport layer
//! and defining a fully specified transport API.
//!
//! It also implements the Session negotiation via Start sub-protocol.
//! See the [`hopr_transport_session::initiation`] module for details on Start sub-protocol.
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

use async_lock::RwLock;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    future::{select, Either},
    pin_mut, FutureExt, SinkExt, StreamExt,
};
use hopr_transport_identity::multiaddrs::strip_p2p_protocol;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tracing::{debug, error, info, trace, warn};

use core_network::{
    heartbeat::Heartbeat,
    ping::{PingConfig, PingQueryReplier, Pinger, Pinging},
};
use core_path::{
    path::TransportPath,
    selectors::dfs::{DfsPathSelector, DfsPathSelectorConfig, RandomizedEdgeWeighting},
};
use hopr_async_runtime::prelude::{sleep, spawn, JoinHandle};
use hopr_db_sql::{accounts::ChainOrPacketKey, HoprDbAllOperations};
use hopr_internal_types::prelude::*;
use hopr_platform::time::native::current_time;
use hopr_primitive_types::prelude::*;
use hopr_transport_p2p::{
    swarm::{TicketAggregationRequestType, TicketAggregationResponseType},
    HoprSwarm,
};
use hopr_transport_protocol::{
    errors::ProtocolError,
    msg::processor::{MsgSender, PacketInteractionConfig, PacketSendFinalizer},
    ticket_aggregation::processor::{TicketAggregationActions, TicketAggregationInteraction, TicketAggregatorTrait},
};
use hopr_transport_session::{DispatchResult, SessionManager, SessionManagerConfig};

#[cfg(feature = "runtime-tokio")]
pub use hopr_transport_session::types::transfer_session;
pub use {
    core_network::network::{Health, Network, NetworkTriggeredEvent, PeerOrigin, PeerStatus},
    hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{HalfKeyChallenge, Hash, OffchainPublicKey},
    },
    hopr_internal_types::protocol::ApplicationData,
    hopr_network_types::prelude::RoutingOptions,
    hopr_transport_identity::{Multiaddr, PeerId},
    hopr_transport_protocol::{execute_on_tick, PeerDiscovery},
    hopr_transport_session::types::{ServiceId, SessionTarget},
    hopr_transport_session::{
        errors::TransportSessionError, traits::SendMsg, Capability as SessionCapability, IncomingSession, Session,
        SessionClientConfig, SessionId, SESSION_USABLE_MTU_SIZE,
    },
};

use crate::{
    constants::{
        RESERVED_SESSION_TAG_UPPER_LIMIT, RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT, SESSION_INITIATION_TIMEOUT_BASE,
    },
    errors::HoprTransportError,
    helpers::PathPlanner,
};

pub use crate::{
    config::HoprTransportConfig,
    helpers::{PeerEligibility, TicketStatistics},
};

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
    #[strum(to_string = "heartbeat performing the network quality measurements")]
    Heartbeat,
}

/// Currently used implementation of [`PathSelector`](core_path::selectors::PathSelector).
type CurrentPathSelector = DfsPathSelector<RandomizedEdgeWeighting>;

/// Interface into the physical transport mechanism allowing all off-chain HOPR-related tasks on
/// the transport, as well as off-chain ticket manipulation.
pub struct HoprTransport<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    me: OffchainKeypair,
    me_peerid: PeerId, // Cache to avoid an expensive conversion: OffchainPublicKey -> PeerId
    cfg: HoprTransportConfig,
    db: T,
    ping: Arc<OnceLock<Pinger<network_notifier::PingExternalInteractions<T>>>>,
    network: Arc<Network<T>>,
    process_packet_send: Arc<OnceLock<MsgSender>>,
    path_planner: PathPlanner<T, CurrentPathSelector>,
    my_multiaddresses: Vec<Multiaddr>,
    process_ticket_aggregate:
        Arc<OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    smgr: SessionManager<helpers::MessageSender<T, CurrentPathSelector>>,
}

impl<T> HoprTransport<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    pub fn new(
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
        cfg: HoprTransportConfig,
        db: T,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
        my_multiaddresses: Vec<Multiaddr>,
    ) -> Self {
        let process_packet_send = Arc::new(OnceLock::new());

        let me_peerid: PeerId = me.into();

        Self {
            me: me.clone(),
            me_peerid,
            ping: Arc::new(OnceLock::new()),
            network: Arc::new(Network::new(
                me_peerid,
                my_multiaddresses.clone(),
                cfg.network.clone(),
                db.clone(),
            )),
            process_packet_send,
            path_planner: PathPlanner::new(
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
                me_onchain.public().to_address(),
            ),
            db,
            my_multiaddresses,
            process_ticket_aggregate: Arc::new(OnceLock::new()),
            smgr: SessionManager::new(
                me_peerid,
                SessionManagerConfig {
                    session_tag_range: RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT..RESERVED_SESSION_TAG_UPPER_LIMIT,
                    initiation_timeout_base: SESSION_INITIATION_TIMEOUT_BASE,
                    idle_timeout: cfg.session.idle_timeout,
                },
            ),
            cfg,
        }
    }

    /// Execute all processes of the [`crate::HoprTransport`] object.
    ///
    /// This method will spawn the [`crate::HoprTransportProcess::Heartbeat`], [`crate::HoprTransportProcess::BloomFilterSave`],
    /// [`crate::HoprTransportProcess::Swarm`] and session-related processes and return
    /// join handles to the calling function. These processes are not started immediately but are
    /// waiting for a trigger from this piece of code.
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        &self,
        me_onchain: &ChainKeypair,
        version: String,
        tbf_path: String,
        on_incoming_data: UnboundedSender<ApplicationData>,
        discovery_updates: UnboundedReceiver<PeerDiscovery>,
        on_incoming_session: UnboundedSender<IncomingSession>,
    ) -> crate::errors::Result<HashMap<HoprTransportProcess, JoinHandle<()>>> {
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
                            PeerDiscovery::Allow(peer_id) => {
                                if let Ok(pk) = OffchainPublicKey::try_from(peer_id) {
                                    if !network.has(&peer_id).await {
                                        let mas = db
                                            .get_account(None, hopr_db_sql::accounts::ChainOrPacketKey::PacketKey(pk))
                                            .await
                                            .map(|entry| {
                                                entry
                                                    .map(|v| Vec::from_iter(v.get_multiaddr().into_iter()))
                                                    .unwrap_or_default()
                                            })
                                            .unwrap_or_default();

                                        if let Err(e) = network.add(&peer_id, PeerOrigin::NetworkRegistry, mas).await {
                                            error!(peer = %peer_id, error = %e, "Failed to allow locally (already allowed on-chain)");
                                            return None;
                                        }
                                    }

                                    return Some(PeerDiscovery::Allow(peer_id))
                                } else {
                                    error!(peer = %peer_id, "Failed to allow locally (already allowed on-chain): peer id not convertible to off-chain public key")
                                }
                            }
                            PeerDiscovery::Ban(peer_id) => {
                                if let Err(e) = network.remove(&peer_id).await {
                                    error!(peer = %peer_id, error = %e, "Failed to ban locally (already banned on-chain)")
                                } else {
                                    return Some(PeerDiscovery::Ban(peer_id))
                                }
                            }
                            PeerDiscovery::Announce(peer, multiaddresses) => {
                                if peer != me {
                                    // decapsulate the `p2p/<peer_id>` to remove duplicities
                                    let mas = multiaddresses
                                        .into_iter()
                                        .map(|ma| strip_p2p_protocol(&ma))
                                        .filter(|v| !v.is_empty())
                                        .collect::<Vec<_>>();

                                    if ! mas.is_empty() {
                                        if let Ok(pk) = OffchainPublicKey::try_from(peer) {
                                            if let Ok(Some(key)) = db.translate_key(None, hopr_db_sql::accounts::ChainOrPacketKey::PacketKey(pk)).await {
                                                let key: Result<Address, _> = key.try_into();

                                                if let Ok(key) = key {
                                                    if db
                                                        .is_allowed_in_network_registry(None, key)
                                                        .await
                                                        .unwrap_or(false)
                                                    {
                                                        if let Err(e) = network.add(&peer, PeerOrigin::NetworkRegistry, mas.clone()).await
                                                        {
                                                            error!(%peer, error = %e, "failed to record peer from the NetworkRegistry");
                                                        } else {
                                                            return Some(PeerDiscovery::Announce(peer, mas))
                                                        }
                                                    }
                                                }
                                            } else {
                                                error!(%peer, "Failed to announce peer due to convertibility error");
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        None
                    }
                });

        info!("Loading initial peers from the storage");

        let nodes = self.get_public_nodes().await?;
        for (peer, _address, multiaddresses) in nodes {
            if self.is_allowed_to_access_network(&peer).await? {
                debug!(%peer, ?multiaddresses, "Using initial public node");

                internal_discovery_update_tx
                    .send(PeerDiscovery::Allow(peer))
                    .await
                    .map_err(|e| HoprTransportError::Api(e.to_string()))?;

                internal_discovery_update_tx
                    .send(PeerDiscovery::Announce(peer, multiaddresses.clone()))
                    .await
                    .map_err(|e| HoprTransportError::Api(e.to_string()))?;
            }
        }

        let mut processes: HashMap<HoprTransportProcess, JoinHandle<()>> = HashMap::new();

        // network event processing channel
        let (network_events_tx, network_events_rx) = futures::channel::mpsc::channel::<NetworkTriggeredEvent>(
            constants::MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE,
        );

        // manual ping
        let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let ping_cfg = PingConfig {
            timeout: self.cfg.protocol.heartbeat.timeout,
            max_parallel_pings: self.cfg.heartbeat.max_parallel_probes,
        };

        let ping: Pinger<network_notifier::PingExternalInteractions<T>> = Pinger::new(
            ping_cfg,
            ping_tx.clone(),
            network_notifier::PingExternalInteractions::new(
                self.network.clone(),
                self.db.clone(),
                self.path_planner.channel_graph(),
                network_events_tx,
            ),
        );

        self.ping
            .clone()
            .set(ping)
            .expect("must set the ping executor only once");

        let ticket_agg_proc = TicketAggregationInteraction::new(self.db.clone(), me_onchain);
        let tkt_agg_writer = ticket_agg_proc.writer();

        let (external_msg_send, external_msg_rx) =
            futures::channel::mpsc::unbounded::<(ApplicationData, TransportPath, PacketSendFinalizer)>();

        self.process_packet_send
            .clone()
            .set(MsgSender::new(external_msg_send))
            .expect("must set the packet processing writer only once");

        self.process_ticket_aggregate
            .clone()
            .set(tkt_agg_writer.clone())
            .expect("must set the ticket aggregation writer only once");

        // heartbeat
        let mut heartbeat = Heartbeat::new(
            self.cfg.heartbeat,
            self.ping
                .get()
                .expect("Ping should be initialized at this point")
                .clone(),
            core_network::heartbeat::HeartbeatExternalInteractions::new(self.network.clone()),
            Box::new(|dur| Box::pin(sleep(dur))),
        );

        // initiate the transport layer
        let (ack_to_send_tx, ack_to_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();
        let (ack_received_tx, ack_received_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();

        let (msg_to_send_tx, msg_to_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (msg_received_tx, msg_received_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();

        let transport_layer = HoprSwarm::new(
            (&self.me).into(),
            network_events_rx,
            discovery_updates,
            ping_rx,
            ticket_agg_proc,
            self.my_multiaddresses.clone(),
            self.cfg.protocol,
        )
        .await;

        let transport_layer = transport_layer.with_processors(
            ack_to_send_rx,
            ack_received_tx,
            msg_to_send_rx,
            msg_received_tx,
            tkt_agg_writer,
        );

        processes.insert(HoprTransportProcess::Medium, spawn(transport_layer.run(version)));

        // initiate the msg-ack protocol stack over the wire transport
        let packet_cfg = PacketInteractionConfig::new(
            &self.me,
            me_onchain,
            self.cfg.protocol.outgoing_ticket_winning_prob,
            self.cfg.protocol.outgoing_ticket_price,
        );

        let (tx_from_protocol, rx_from_protocol) = futures::channel::mpsc::unbounded::<ApplicationData>();
        for (k, v) in hopr_transport_protocol::run_msg_ack_protocol(
            packet_cfg,
            self.db.clone(),
            Some(tbf_path),
            (ack_to_send_tx, ack_received_rx),
            (msg_to_send_tx, msg_received_rx),
            (tx_from_protocol, external_msg_rx),
        )
        .await
        .into_iter()
        {
            processes.insert(HoprTransportProcess::Protocol(k), v);
        }

        let msg_sender = helpers::MessageSender::new(self.process_packet_send.clone(), self.path_planner.clone());

        self.smgr
            .start(msg_sender, on_incoming_session)
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
            spawn(async move {
                let _the_process_should_not_end = StreamExt::filter_map(rx_from_protocol, |data| {
                    let smgr = smgr.clone();
                    async move {
                        match smgr.dispatch_message(data).await {
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

        // initiate the network telemetry
        let half_the_hearbeat_interval = self.cfg.heartbeat.interval / 4;
        processes.insert(
            HoprTransportProcess::Heartbeat,
            spawn(async move {
                // present to make sure that the heartbeat does not start immediately
                hopr_async_runtime::prelude::sleep(half_the_hearbeat_interval).await;
                heartbeat.heartbeat_loop().await
            }),
        );

        Ok(processes)
    }

    pub fn ticket_aggregator(&self) -> Arc<dyn TicketAggregatorTrait + Send + Sync + 'static> {
        Arc::new(proxy::TicketAggregatorProxy::new(
            self.db.clone(),
            self.process_ticket_aggregate.clone(),
            self.cfg.protocol.ticket_aggregation.timeout,
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<(std::time::Duration, PeerStatus)> {
        if !self.is_allowed_to_access_network(peer).await? {
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

        let timeout = sleep(Duration::from_secs(30)).fuse();
        let ping = (*pinger).ping(vec![*peer]);

        pin_mut!(timeout, ping);

        if let Err(e) = self.network.add(peer, PeerOrigin::ManualPing, vec![]).await {
            error!(error = %e, "Failed to store the peer observation");
        }

        let start = current_time().as_unix_timestamp();

        match select(timeout, ping.next().fuse()).await {
            Either::Left(_) => {
                warn!(%peer, "Manual ping to peer timed out");
                return Err(ProtocolError::Timeout.into());
            }
            Either::Right((v, _)) => {
                match v
                    .into_iter()
                    .map(|r| r.map_err(HoprTransportError::NetworkError))
                    .collect::<errors::Result<Vec<Duration>>>()
                {
                    Ok(d) => info!(%peer, rtt = ?d, "Manual ping succeeded"),
                    Err(e) => {
                        error!(%peer, error = %e, "Manual ping failed");
                        return Err(e);
                    }
                }
            }
        };

        let peer_status = self.network.get(peer).await?.ok_or(HoprTransportError::NetworkError(
            errors::NetworkingError::NonExistingPeer,
        ))?;

        Ok((
            peer_status.last_seen.as_unix_timestamp().saturating_sub(start),
            peer_status,
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn new_session(&self, cfg: SessionClientConfig) -> errors::Result<Session> {
        Ok(self.smgr.new_session(cfg).await?)
    }

    #[tracing::instrument(level = "info", skip(self, msg), fields(uuid = uuid::Uuid::new_v4().to_string()))]
    pub async fn send_message(
        &self,
        msg: Box<[u8]>,
        destination: PeerId,
        options: RoutingOptions,
        application_tag: Option<u16>,
    ) -> errors::Result<()> {
        // The send_message logic will be entirely removed in 3.0
        if let Some(application_tag) = application_tag {
            if application_tag < RESERVED_SESSION_TAG_UPPER_LIMIT {
                return Err(HoprTransportError::Api(format!(
                    "Application tag must not be lower than {RESERVED_SESSION_TAG_UPPER_LIMIT}"
                )));
            }
        }

        if msg.len() > PAYLOAD_SIZE {
            return Err(HoprTransportError::Api(format!(
                "Message exceeds the maximum allowed size of {PAYLOAD_SIZE} bytes"
            )));
        }

        let app_data = ApplicationData::new_from_owned(application_tag, msg)?;

        // Here we do not use msg_sender directly,
        // since it internally follows Session-oriented logic
        let path = self.path_planner.resolve_path(destination, options).await?;
        let sender = self.process_packet_send.get().ok_or_else(|| {
            HoprTransportError::Api("send msg: failed because message processing is not yet initialized".into())
        })?;

        sender
            .send_packet(app_data, path)
            .await
            .map_err(|e| HoprTransportError::Api(format!("send msg failed to enqueue msg: {e}")))?
            .consume_and_wait(crate::constants::PACKET_QUEUE_TIMEOUT_MILLISECONDS)
            .await
            .map_err(|e| HoprTransportError::Api(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn aggregate_tickets(&self, channel_id: &Hash) -> errors::Result<()> {
        let entry = self
            .db
            .get_channel_by_id(None, channel_id)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)
            .map_err(HoprTransportError::from)
            .and_then(|c| {
                if let Some(c) = c {
                    Ok(c)
                } else {
                    Err(ProtocolError::ChannelNotFound.into())
                }
            })?;

        if entry.status != ChannelStatus::Open {
            return Err(ProtocolError::ChannelClosed.into());
        }

        Ok(Arc::new(proxy::TicketAggregatorProxy::new(
            self.db.clone(),
            self.process_ticket_aggregate.clone(),
            self.cfg.protocol.ticket_aggregation.timeout,
        ))
        .aggregate_tickets(&entry.get_id(), Default::default())
        .await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_public_nodes(&self) -> errors::Result<Vec<(PeerId, Address, Vec<Multiaddr>)>> {
        Ok(self
            .db
            .get_accounts(None, true)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?
            .into_iter()
            .map(|entry| {
                (
                    PeerId::from(entry.public_key),
                    entry.chain_addr,
                    Vec::from_iter(entry.get_multiaddr().into_iter()),
                )
            })
            .collect())
    }

    pub async fn is_allowed_to_access_network<'a>(&self, peer: &'a PeerId) -> errors::Result<bool>
    where
        T: 'a,
    {
        let db_clone = self.db.clone();
        let peer = *peer;
        Ok(self
            .db
            .begin_transaction()
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?
            .perform(|tx| {
                Box::pin(async move {
                    let pk = OffchainPublicKey::try_from(peer)?;
                    if let Some(address) = db_clone.translate_key(Some(tx), pk).await? {
                        db_clone
                            .is_allowed_in_network_registry(Some(tx), address.try_into()?)
                            .await
                    } else {
                        Err(hopr_db_sql::errors::DbSqlError::LogicalError(
                            "cannot translate off-chain key".into(),
                        ))
                    }
                })
            })
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?)
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
