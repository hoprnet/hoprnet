//! The crate aggregates and composes individual transport level objects and functionality
//! into a unified [`crate::HoprTransport`] object with the goal of isolating the transport layer
//! and defining a fully specified transport API.
//!
//! As such, the transport layer components should be only those that are directly needed in
//! order to:
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

use async_lock::RwLock;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    future::{select, Either},
    pin_mut, FutureExt, StreamExt, TryStreamExt,
};
use moka::ops::compute::CompResult;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tracing::{debug, error, info, trace, warn};

use core_network::{
    heartbeat::Heartbeat,
    ping::{PingConfig, PingQueryReplier, Pinger, Pinging},
    PeerId,
};
use core_path::path::TransportPath;
use hopr_async_runtime::prelude::{sleep, spawn, JoinHandle};
use hopr_db_sql::{
    api::tickets::{AggregationPrerequisites, HoprDbTicketOperations},
    HoprDbAllOperations,
};
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
    ticket_aggregation::processor::{
        AwaitingAggregator, TicketAggregationActions, TicketAggregationInteraction, TicketAggregatorTrait,
    },
};
use hopr_transport_session::IpProtocol;
pub use {
    core_network::network::{Health, Network, NetworkTriggeredEvent, PeerOrigin, PeerStatus},
    hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{HalfKeyChallenge, Hash, OffchainPublicKey},
    },
    hopr_internal_types::protocol::ApplicationData,
    hopr_transport_p2p::{
        libp2p, libp2p::swarm::derive_prelude::Multiaddr, multiaddrs::strip_p2p_protocol,
        swarm::HoprSwarmWithProcessors, PeerDiscovery,
    },
    hopr_transport_protocol::execute_on_tick,
    hopr_transport_session::{
        errors::TransportSessionError, traits::SendMsg, Capability as SessionCapability, Session, SessionClientConfig,
        SessionId, SESSION_USABLE_MTU_SIZE,
    },
};

use crate::{
    constants::{RESERVED_SESSION_TAG_UPPER_LIMIT, RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT},
    errors::HoprTransportError,
};

pub use crate::helpers::{IndexerTransportEvent, PeerEligibility, TicketStatistics};
pub use hopr_network_types::prelude::RoutingOptions;
use hopr_transport_session::initiation::{
    StartChallenge, StartErrorReason, StartErrorType, StartEstablished, StartInitiation, StartProtocol,
};
pub use hopr_transport_session::types::SessionTarget;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum HoprTransportProcess {
    Heartbeat,
    Swarm,
    ProtocolAckIn,
    ProtocolAckOut,
    ProtocolMsgIn,
    ProtocolMsgOut,
    SessionsManagement,
    BloomFilterSave,
}

#[derive(Debug, Clone)]
pub struct TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    db: Db,
    maybe_writer:
        Arc<std::sync::OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    agg_timeout: std::time::Duration,
}

impl<Db> TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    pub fn new(
        db: Db,
        maybe_writer: Arc<
            std::sync::OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>,
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
    ) -> hopr_transport_protocol::errors::Result<()> {
        if let Some(writer) = self.maybe_writer.clone().get() {
            AwaitingAggregator::new(self.db.clone(), writer.clone(), self.agg_timeout)
                .aggregate_tickets(channel, prerequisites)
                .await
        } else {
            Err(ProtocolError::TransportError(
                "Ticket aggregation writer not available, the object was not yet initialized".to_string(),
            ))
        }
    }
}

pub struct HoprTransportConfig {
    pub transport: config::TransportConfig,
    pub network: core_network::config::NetworkConfig,
    pub protocol: hopr_transport_protocol::config::ProtocolConfig,
    pub heartbeat: core_network::heartbeat::HeartbeatConfig,
}

/// This function will use the given generator to generate an initial seeding key.
/// It will check whether the given cache already contains a value for that key and if not,
/// calls the generator (with the previous value) to generate a new seeding key and retry.
/// The function either finds a suitable free slot, inserting the `value` or terminates with `None`
/// when `gen` returns the initial seed again.
async fn insert_into_next_slot<K, V, F>(cache: &moka::future::Cache<K, V>, gen: F, value: V) -> Option<K>
where
    K: Copy + std::hash::Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(Option<K>) -> K,
{
    let initial = gen(None);
    let mut next = initial;
    loop {
        let insertion_result = cache
            .entry(next)
            .and_try_compute_with(|e| {
                if e.is_none() {
                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Put(value.clone()))
                } else {
                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Nop)
                }
            })
            .await;

        if let Ok(CompResult::Inserted(_)) = insertion_result {
            return Some(next);
        }

        next = gen(Some(next));

        if next == initial {
            return None;
        }
    }
}

// Needs to use an UnboundedSender instead of oneshot
// because Moka cache requires the oneshot Sender to be Clone.
// It also cannot be enclosed in an Arc, since calling `send` consumes the sender.
type SessionInitiationCache =
    moka::future::Cache<StartChallenge, UnboundedSender<Result<StartEstablished<SessionId>, StartErrorType>>>;

/// Interface into the physical transport mechanism allowing all off-chain HOPR-related tasks on
/// the transport, as well as off-chain ticket manipulation.
pub struct HoprTransport<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    me: PeerId,
    me_onchain: Address,
    cfg: HoprTransportConfig,
    db: T,
    ping: Arc<OnceLock<Pinger<network_notifier::PingExternalInteractions<T>>>>,
    network: Arc<Network<T>>,
    path_planner: helpers::PathPlanner<T>,
    my_multiaddresses: Vec<Multiaddr>,
    process_packet_send: Arc<OnceLock<MsgSender>>,
    process_ticket_aggregate:
        Arc<OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    session_initiations: SessionInitiationCache,
    sessions: moka::future::Cache<SessionId, UnboundedSender<Box<[u8]>>>,
}

async fn handle_start_protocol_message<T>(
    tag: Tag,
    data: Box<[u8]>,
    me: PeerId,
    new_session_notifier: UnboundedSender<(Session, SessionTarget)>,
    message_sender: Arc<helpers::MessageSender<T>>,
    sessions: moka::future::Cache<SessionId, UnboundedSender<Box<[u8]>>>,
    session_initiations: SessionInitiationCache,
) -> errors::Result<()>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    match StartProtocol::<SessionId>::decode(tag, &data)? {
        StartProtocol::StartSession(session_req) => {
            // Back-routing information is mandatory until the Return Path is introduced
            let (route, peer) = session_req.back_routing.ok_or(errors::HoprTransportError::Api(
                "no back-routing information given".into(),
            ))?;

            debug!(
                peer = tracing::field::display(peer),
                "got new session request, searching for a free session slot"
            );

            // Construct the session
            let (tx, rx) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
            if let Some(session_id) = insert_into_next_slot(
                &sessions,
                |sid| {
                    let next_tag = if let Some(session_id) = sid {
                        ((session_id.tag() + 1) % RESERVED_SESSION_TAG_UPPER_LIMIT)
                            .max(RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT)
                    } else {
                        hopr_crypto_random::random_integer(
                            RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT as u64,
                            Some(RESERVED_SESSION_TAG_UPPER_LIMIT as u64),
                        ) as u16
                    };
                    SessionId::new(next_tag, peer)
                },
                tx,
            )
            .await
            {
                debug!(
                    session_id = tracing::field::debug(session_id),
                    "assigning a new session"
                );

                let session = Session::new(
                    session_id,
                    me,
                    route.clone(),
                    session_req.capabilities,
                    message_sender.clone(),
                    rx,
                );

                // Notify that a new session has been created
                if let Err(e) = new_session_notifier.unbounded_send((session, session_req.target)) {
                    warn!("failed to send session to incoming session queue: {e}");
                }

                trace!(
                    session_id = tracing::field::debug(session_id),
                    "session notification sent"
                );

                // Notify the sender that the session has been established
                let data = StartProtocol::SessionEstablished(StartEstablished {
                    orig_challenge: session_req.challenge,
                    session_id,
                })
                .encode_as_app_data()?;

                message_sender.send_message(data, peer, route).await.map_err(|e| {
                    HoprTransportError::Api(format!("failed to send session establishment message: {e}"))
                })?;

                info!(
                    session_id = tracing::field::display(session_id),
                    "new session established"
                );
            } else {
                error!(
                    peer = tracing::field::display(peer),
                    "failed to reserve a new session slot"
                );

                // Notify the sender that the session could not be established
                let data = StartProtocol::<SessionId>::SessionError(StartErrorType {
                    challenge: session_req.challenge,
                    reason: StartErrorReason::NoSlotsAvailable,
                })
                .encode_as_app_data()?;

                message_sender.send_message(data, peer, route).await.map_err(|e| {
                    HoprTransportError::Api(format!("failed to send session establishment error message: {e}"))
                })?;

                trace!(
                    peer = tracing::field::display(peer),
                    "session establishment failure message sent"
                );
            }
        }
        StartProtocol::SessionEstablished(est) => {
            let challenge = est.orig_challenge;
            if let Some(tx_est) = session_initiations.remove(&est.orig_challenge).await {
                if let Err(e) = tx_est.unbounded_send(Ok(est)) {
                    return Err(
                        GeneralError::NonSpecificError(format!("could not notify session establishment: {e}")).into(),
                    );
                }
                debug!(challenge, "session establishment complete");
            } else {
                error!(challenge, "session establishment attempt expired");
            }
        }
        StartProtocol::SessionError(err) => {
            // Currently, we don't distinguish the error types
            // and just discard the initiation attempt and pass on the error.
            if let Some(tx_est) = session_initiations.remove(&err.challenge).await {
                if let Err(e) = tx_est.unbounded_send(Err(err)) {
                    return Err(GeneralError::NonSpecificError(format!(
                        "could not notify session establishment error {err:?}: {e}"
                    ))
                    .into());
                }
                error!(
                    challenge = err.challenge,
                    "session establishment error received: {}", err.reason
                );
            } else {
                error!(
                    challenge = err.challenge,
                    "session establishment attempt expired before error could be delivered: {}", err.reason
                );
            }
        }
        StartProtocol::CloseSession(session_id) => {
            if let Some(sender) = sessions.remove(&session_id).await {
                sender.close_channel();
                debug!(
                    session_id = tracing::field::debug(session_id),
                    "session has been closed by the other party"
                );
            } else {
                error!(
                    session_id = tracing::field::debug(session_id),
                    "session to be closed not found"
                );
            }
        }
    }

    Ok(())
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
        let identity: libp2p::identity::Keypair = (me).into();

        Self {
            me: identity.public().to_peer_id(),
            me_onchain: me_onchain.public().to_address(),
            db: db.clone(),
            ping: Arc::new(OnceLock::new()),
            network: Arc::new(Network::new(
                me.public().into(),
                my_multiaddresses.clone(),
                cfg.network.clone(),
                db.clone(),
            )),
            cfg,
            path_planner: helpers::PathPlanner::new(db, channel_graph),
            my_multiaddresses,
            process_packet_send: Arc::new(OnceLock::new()),
            process_ticket_aggregate: Arc::new(OnceLock::new()),
            session_initiations: moka::future::Cache::builder()
                .max_capacity((RESERVED_SESSION_TAG_UPPER_LIMIT - RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT + 1) as u64)
                .time_to_live(constants::SESSION_INITIATION_TIMEOUT)
                .build(),
            sessions: moka::future::Cache::builder()
                .max_capacity(u16::MAX as u64)
                .time_to_idle(constants::SESSION_LIFETIME)
                .build(),
        }
    }

    pub fn me(&self) -> &PeerId {
        &self.me
    }

    pub fn network(&self) -> Arc<Network<T>> {
        self.network.clone()
    }

    /// Execute all processes of the [`crate::HoprTransport`] object.
    ///
    /// This method will spawn the [`crate::HoprTransportProcess::Heartbeat`], [`crate::HoprTransportProcess::BloomFilterSave`],
    /// [`crate::HoprTransportProcess::Swarm`] and [`crate::HoprTransportProcess::SessionsRouter`] processes and return
    /// join handles to the calling function. These processes are not started immediately but are
    /// waiting for a trigger from this piece of code.
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        &self,
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
        version: String,
        network: Arc<Network<T>>,
        tbf_path: String,
        on_transport_output: UnboundedSender<ApplicationData>,
        on_acknowledged_ticket: UnboundedSender<AcknowledgedTicket>,
        transport_updates: UnboundedReceiver<PeerDiscovery>,
        new_session_notifier: UnboundedSender<(Session, SessionTarget)>,
    ) -> HashMap<HoprTransportProcess, JoinHandle<()>> {
        let mut processes: HashMap<HoprTransportProcess, JoinHandle<()>> = HashMap::new();

        // network event processing channel
        let (network_events_tx, network_events_rx) = futures::channel::mpsc::channel::<NetworkTriggeredEvent>(
            constants::MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE,
        );

        // manual ping
        let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let ping_cfg = PingConfig {
            timeout: self.cfg.protocol.heartbeat.timeout,
            ..PingConfig::default()
        };

        let ping: Pinger<network_notifier::PingExternalInteractions<T>> = Pinger::new(
            ping_cfg,
            ping_tx.clone(),
            network_notifier::PingExternalInteractions::new(
                network.clone(),
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

        let transport_layer = HoprSwarm::new(
            me.into(),
            network_events_rx,
            transport_updates,
            ping_rx,
            ticket_agg_proc,
            self.my_multiaddresses.clone(),
            self.cfg.protocol,
        )
        .await;

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
            core_network::heartbeat::HeartbeatExternalInteractions::new(network.clone()),
            Box::new(|dur| Box::pin(sleep(dur))),
        );

        // initiate the libp2p transport layer
        let (ack_to_send_tx, ack_to_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();
        let (ack_received_tx, ack_received_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();

        let (msg_to_send_tx, msg_to_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (msg_received_tx, msg_received_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();

        let transport_layer = transport_layer.with_processors(
            ack_to_send_rx,
            ack_received_tx,
            msg_to_send_rx,
            msg_received_tx,
            tkt_agg_writer,
        );

        processes.insert(
            HoprTransportProcess::Swarm,
            spawn(transport_layer.run(version, on_acknowledged_ticket.clone())),
        );

        // initiate the msg-ack protocol stack over the wire transport
        let packet_cfg = PacketInteractionConfig::new(me, me_onchain);

        let (tx_from_protocol, rx_from_protocol) = futures::channel::mpsc::unbounded::<ApplicationData>();
        for (k, v) in hopr_transport_protocol::run_msg_ack_protocol(
            packet_cfg,
            self.db.clone(),
            me_onchain,
            Some(tbf_path),
            on_acknowledged_ticket,
            (ack_to_send_tx, ack_received_rx),
            (msg_to_send_tx, msg_received_rx),
            (tx_from_protocol, external_msg_rx),
        )
        .await
        .into_iter()
        {
            processes.insert(
                match k {
                    hopr_transport_protocol::ProtocolProcesses::AckIn => HoprTransportProcess::ProtocolAckIn,
                    hopr_transport_protocol::ProtocolProcesses::AckOut => HoprTransportProcess::ProtocolAckOut,
                    hopr_transport_protocol::ProtocolProcesses::MsgIn => HoprTransportProcess::ProtocolMsgIn,
                    hopr_transport_protocol::ProtocolProcesses::MsgOut => HoprTransportProcess::ProtocolMsgOut,
                    hopr_transport_protocol::ProtocolProcesses::BloomPersist => HoprTransportProcess::BloomFilterSave,
                },
                v,
            );
        }

        // initiate session handling over the msg-ack protocol stack
        let sessions = self.sessions.clone();
        let session_initiations = self.session_initiations.clone();
        let me = self.me;
        let message_sender = Arc::new(helpers::MessageSender::new(
            self.process_packet_send.clone(),
            self.path_planner.clone(),
        ));

        processes.insert(
            HoprTransportProcess::SessionsManagement,
            spawn(async move {
                let _the_process_should_not_end = StreamExt::filter_map(rx_from_protocol, move |data| {
                    let me = me;
                    let sessions = sessions.clone();
                    let session_initiations = session_initiations.clone();
                    let message_sender = message_sender.clone();
                    let new_session_notifier = new_session_notifier.clone();

                    async move {
                        if let Some(app_tag) = data.application_tag {
                            match app_tag {
                                tag @ 0..RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT => {
                                    if let Err(e) = handle_start_protocol_message(
                                        tag,
                                        data.plain_text,
                                        me,
                                        new_session_notifier,
                                        message_sender,
                                        sessions,
                                        session_initiations,
                                    )
                                    .await
                                    {
                                        error!("failed to handle Start protocol message: {e}");
                                    }
                                    None
                                }
                                RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT..RESERVED_SESSION_TAG_UPPER_LIMIT => {
                                    if let Ok((peer, data)) =
                                        hopr_transport_session::types::unwrap_offchain_key(data.plain_text.clone())
                                    {
                                        let session_id = SessionId::new(app_tag, peer);
                                        if let Some(session_data_sender) = sessions.get(&session_id).await {
                                            trace!(
                                                session_id = tracing::field::debug(session_id),
                                                "received data for a registered session"
                                            );
                                            if let Err(e) = session_data_sender.unbounded_send(data) {
                                                error!(
                                                    session_id = tracing::field::debug(session_id),
                                                    "failed to send received data to session: {e}"
                                                );
                                            }
                                        } else {
                                            error!(
                                                session_id = tracing::field::debug(session_id),
                                                "received data from an unestablished session"
                                            )
                                        }
                                    }
                                    None
                                }
                                _ => Some(data),
                            }
                        } else {
                            Some(data)
                        }
                    }
                })
                .map(Ok)
                .forward(on_transport_output)
                .await;
            }),
        );

        // initiate the network telemetry
        processes.insert(
            HoprTransportProcess::Heartbeat,
            spawn(async move { heartbeat.heartbeat_loop().await }),
        );

        processes
    }

    pub fn ticket_aggregator(&self) -> Arc<dyn TicketAggregatorTrait + Send + Sync + 'static> {
        Arc::new(TicketAggregatorProxy::new(
            self.db.clone(),
            self.process_ticket_aggregate.clone(),
            self.cfg.protocol.ticket_aggregation.timeout,
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<Option<std::time::Duration>> {
        if !self.is_allowed_to_access_network(peer).await? {
            return Err(HoprTransportError::Api(format!(
                "ping to '{peer}' not allowed due to network registry"
            )));
        }

        if peer == &self.me {
            return Err(HoprTransportError::Api("ping to self does not make sense".into()));
        }

        let pinger = self
            .ping
            .get()
            .ok_or_else(|| HoprTransportError::Api("ping processing is not yet initialized".into()))?;

        let timeout = sleep(std::time::Duration::from_secs(30)).fuse();
        let ping = (*pinger).ping(vec![*peer]).fuse();

        pin_mut!(timeout, ping);

        if let Err(e) = self.network.add(peer, PeerOrigin::ManualPing, vec![]).await {
            error!("Failed to store the peer observation: {e}");
        }

        let start = current_time().as_unix_timestamp();

        match select(timeout, ping).await {
            Either::Left(_) => {
                warn!(peer = peer.to_string(), "Manual ping to peer timed out");
                return Err(ProtocolError::Timeout.into());
            }
            Either::Right(_) => info!("Manual ping succeeded"),
        };

        Ok(self
            .network
            .get(peer)
            .await?
            .map(|status| status.last_seen.as_unix_timestamp().saturating_sub(start)))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn new_session(&self, cfg: SessionClientConfig) -> errors::Result<Session> {
        let (tx_initiation_done, rx_initiation_done) = futures::channel::mpsc::unbounded();

        const MIN_CHALLENGE: StartChallenge = 1;

        let challenge = insert_into_next_slot(
            &self.session_initiations,
            |ch| {
                if let Some(challenge) = ch {
                    ((challenge + 1) % hopr_crypto_random::MAX_RANDOM_INTEGER).max(MIN_CHALLENGE)
                } else {
                    hopr_crypto_random::random_integer(MIN_CHALLENGE, None)
                }
            },
            tx_initiation_done,
        )
        .await
        .ok_or(HoprTransportError::Api("all challenge slots are occupied".into()))?; // almost impossible with u64

        // Prepare the session initiation message in the Start protocol
        debug!(challenge, "initiating session with config {cfg:?}");
        let (tag, msg) = StartProtocol::<SessionId>::StartSession(StartInitiation {
            challenge,
            target: match cfg.target_protocol {
                IpProtocol::TCP => SessionTarget::TcpStream(cfg.target),
                IpProtocol::UDP => SessionTarget::UdpStream(cfg.target),
            },
            capabilities: cfg.capabilities.iter().copied().collect(),
            // Back-routing currently uses the same (inverted) route as session initiation
            back_routing: Some((cfg.path_options.clone().invert(), self.me)),
        })
        .encode()?;

        // Send the Session initiation message
        self.send_message(msg, cfg.peer, cfg.path_options.clone(), tag.into())
            .await?;

        // Await session establishment response from the Exit node or timeout
        pin_mut!(rx_initiation_done);
        let initiation_done = TryStreamExt::try_next(&mut rx_initiation_done);

        let timeout = hopr_async_runtime::prelude::sleep(constants::SESSION_INITIATION_TIMEOUT);
        pin_mut!(timeout);

        trace!(challenge, "awaiting session establishment");
        match futures::future::select(initiation_done, timeout).await {
            Either::Left((Ok(Some(est)), _)) => {
                // Session has been established, construct it
                let session_id = est.session_id;
                debug!(
                    challenge = est.orig_challenge,
                    session_id = tracing::field::debug(session_id),
                    "started a new session"
                );

                // Insert the Session object, forcibly overwrite any other session with the same ID
                let (tx, rx) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
                self.sessions.insert(session_id, tx).await;

                Ok(Session::new(
                    session_id,
                    self.me,
                    cfg.path_options,
                    cfg.capabilities.into_iter().collect(),
                    Arc::new(helpers::MessageSender::new(
                        self.process_packet_send.clone(),
                        self.path_planner.clone(),
                    )),
                    rx,
                ))
            }
            Either::Left((Ok(None), _)) => Err(errors::HoprTransportError::Api(
                "internal error: sender has been closed without completing the session establishment".into(),
            )),
            Either::Left((Err(e), _)) => {
                // The other side didn't allow us to establish a session
                Err(errors::HoprTransportError::Api(format!(
                    "exit node didn't allow session initiation with challenge {}: {}",
                    e.challenge, e.reason
                )))
            }
            Either::Right(_) => Err(errors::HoprTransportError::Api("timeout establishing session".into())),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn close_session(&self, session: Session) -> errors::Result<()> {
        let session_id = *session.id();
        if let Some(session_tx) = self.sessions.remove(&session_id).await {
            let (tag, msg) = StartProtocol::CloseSession(session_id).encode()?;

            trace!(
                session_id = tracing::field::debug(session_id),
                "sending session termination"
            );
            self.send_message(msg, *session_id.peer(), session.routing_options().clone(), tag.into())
                .await?;
            session_tx.close_channel();

            info!(
                session_id = tracing::field::debug(session_id),
                "session has been closed"
            );
            Ok(())
        } else {
            Err(HoprTransportError::Api("cannot find session".into()))
        }
    }

    #[tracing::instrument(level = "info", skip(self, msg), fields(uuid = uuid::Uuid::new_v4().to_string()))]
    pub async fn send_message(
        &self,
        msg: Box<[u8]>,
        destination: PeerId,
        options: RoutingOptions,
        application_tag: Option<u16>,
    ) -> errors::Result<()> {
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

        Ok(Arc::new(TicketAggregatorProxy::new(
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
            .get(&self.me)
            .await
            .unwrap_or(None)
            .map(|peer| peer.multiaddresses)
            .unwrap_or(vec![])
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn announceable_multiaddresses(&self) -> Vec<Multiaddr> {
        let mut mas = self
            .local_multiaddresses()
            .into_iter()
            .filter(|ma| {
                hopr_transport_p2p::multiaddrs::is_supported(ma)
                    && (self.cfg.transport.announce_local_addresses || !hopr_transport_p2p::multiaddrs::is_private(ma))
            })
            .map(|ma| strip_p2p_protocol(&ma))
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();

        mas.sort_by(|l, r| {
            let is_left_dns = hopr_transport_p2p::multiaddrs::is_dns(l);
            let is_right_dns = hopr_transport_p2p::multiaddrs::is_dns(r);

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
        Ok(self.network.peer_filter(|peer| async move { Some(peer.id.1) }).await?)
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
            if channel.destination == self.me_onchain {
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
