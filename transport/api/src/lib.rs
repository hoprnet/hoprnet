//! The crate aggregates and composes individual transport level objects and functionality
//! into a unified [`HoprTransport`] object with the goal of isolating the transport layer
//! and defining a fully specified transport API.
//!
//! As such, the transport layer components should be only those that are directly needed in
//! order to:
//! 1. send and receive a packet, acknowledgement or ticket aggregation request
//! 2. send and receive a network telemetry request
//! 3. automate transport level processes
//! 4. algorithms associated with the transport layer operational management
//! 5. interface specifications to allow modular behavioral extensions

/// Configuration of the [HoprTransport].
pub mod config;
/// Constants used and exposed by the crate.
pub mod constants;
/// Errors used by the crate.
pub mod errors;
pub mod network_notifier;
mod processes;
mod timer;

use crate::errors::HoprTransportError;

pub use {
    core_network::network::{Health, Network, NetworkTriggeredEvent, PeerOrigin, PeerStatus},
    hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{HalfKeyChallenge, Hash, OffchainPublicKey},
    },
    hopr_internal_types::protocol::ApplicationData,
    hopr_transport_p2p::{
        libp2p, libp2p::swarm::derive_prelude::Multiaddr, multiaddrs::strip_p2p_protocol,
        swarm::HoprSwarmWithProcessors, PeerTransportEvent, TransportOutput,
    },
    hopr_transport_session::SendOptions,
    timer::execute_on_tick,
};

use async_lock::RwLock;
use constants::RESERVED_TAG_UPPER_LIMIT;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    FutureExt, SinkExt, StreamExt,
};
use hopr_transport_p2p::{
    swarm::{TicketAggregationRequestType, TicketAggregationResponseType},
    HoprSwarm,
};
use hopr_transport_session::{
    traits::{SendMsg, SessionError},
    Session, SessionId,
};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

pub(crate) mod executor {
    #[cfg(any(feature = "runtime-async-std", test))]
    pub use async_std::task::{sleep, spawn, JoinHandle};

    #[cfg(all(feature = "runtime-tokio", not(test)))]
    pub use tokio::{
        task::{spawn, JoinHandle},
        time::sleep,
    };
}

use hopr_db_sql::{
    api::{
        peers::HoprDbPeersOperations,
        tickets::{AggregationPrerequisites, HoprDbTicketOperations},
    },
    HoprDbAllOperations,
};
use tracing::{debug, error, info, warn};

use core_network::{
    config::NetworkConfig,
    heartbeat::Heartbeat,
    ping::{PingQueryReplier, Pinger},
};
use core_network::{heartbeat::HeartbeatConfig, ping::PingConfig, PeerId};
use core_protocol::{
    ack::processor::AcknowledgementInteraction,
    msg::processor::{PacketActions, PacketInteraction, PacketInteractionConfig},
    ticket_aggregation::processor::{
        AwaitingAggregator, TicketAggregationActions, TicketAggregationInteraction, TicketAggregatorTrait,
    },
};
use hopr_internal_types::prelude::*;
use hopr_platform::time::native::current_time;
use hopr_primitive_types::primitives::Address;

#[cfg(all(feature = "prometheus", not(test)))]
use {core_path::path::Path, hopr_metrics::metrics::SimpleHistogram};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PATH_LENGTH: SimpleHistogram = SimpleHistogram::new(
        "hopr_path_length",
        "Distribution of number of hops of sent messages",
        vec![0.0, 1.0, 2.0, 3.0, 4.0]
    ).unwrap();
}

use chain_types::chain_events::NetworkRegistryStatus;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PeerEligibility {
    Eligible,
    Ineligible,
}

impl From<NetworkRegistryStatus> for PeerEligibility {
    fn from(value: NetworkRegistryStatus) -> Self {
        match value {
            NetworkRegistryStatus::Allowed => Self::Eligible,
            NetworkRegistryStatus::Denied => Self::Ineligible,
        }
    }
}

/// Indexer events triggered externally from the [`HoprTransport`] object.
pub enum IndexerTransportEvent {
    EligibilityUpdate(PeerId, PeerEligibility),
    Announce(PeerId, Vec<Multiaddr>),
}

/// Build the [Network] object responsible for tracking and holding the
/// observable state of the physical transport network, peers inside the network
/// and telemetry about network connections.
pub fn build_network<T>(peer_id: PeerId, addresses: Vec<Multiaddr>, cfg: NetworkConfig, db: T) -> Arc<Network<T>>
where
    T: HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
{
    Arc::new(Network::new(peer_id, addresses, cfg, db))
}

/// Event emitter used by the indexer to emit events when an on-chain change on a
/// channel is detected.
#[derive(Clone)]
pub struct ChannelEventEmitter {
    pub tx: UnboundedSender<ChannelEntry>,
}

impl ChannelEventEmitter {
    pub async fn send_event(&self, channel: &ChannelEntry) {
        let mut sender = self.tx.clone();
        let _ = sender.send(*channel).await;
    }
}

/// Ticket statistics data exposed by the ticket mechanism.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TicketStatistics {
    pub winning_count: u128,
    pub unredeemed_value: hopr_primitive_types::primitives::Balance,
    pub redeemed_value: hopr_primitive_types::primitives::Balance,
    pub neglected_value: hopr_primitive_types::primitives::Balance,
    pub rejected_value: hopr_primitive_types::primitives::Balance,
}

pub struct PublicNodesResult {
    pub id: String,
    pub address: Address,
    pub multiaddrs: Vec<Multiaddr>,
}

use core_network::ping::Pinging;
use core_path::path::TransportPath;
use core_path::selectors::legacy::LegacyPathSelector;
use core_path::selectors::PathSelector;
use core_protocol::errors::ProtocolError;
use futures::future::{select, Either};
use futures::pin_mut;
use hopr_internal_types::channels::ChannelStatus;
use hopr_primitive_types::prelude::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum HoprTransportProcess {
    Heartbeat,
    Swarm,
    SessionUnwrapper,
}

#[derive(Debug, Clone)]
pub struct AggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    db: Db,
    maybe_writer:
        Arc<std::sync::OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    agg_timeout: std::time::Duration,
}

impl<Db> AggregatorProxy<Db>
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
impl<Db> TicketAggregatorTrait for AggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    async fn aggregate_tickets(
        &self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> core_protocol::errors::Result<()> {
        if let Some(writer) = self.maybe_writer.clone().get() {
            AwaitingAggregator::new(self.db.clone(), writer.clone(), self.agg_timeout)
                .aggregate_tickets(channel, prerequisites)
                .await
        } else {
            Err(core_protocol::errors::ProtocolError::TransportError(
                "Ticket aggregation writer not available, the object was not yet initialized".to_string(),
            ))
        }
    }
}

pub struct PathPlanner<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    db: T,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
}

impl<T> PathPlanner<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    pub fn new(db: T, channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>) -> Self {
        Self { db, channel_graph }
    }

    pub async fn resolve_path(&mut self, destination: PeerId, options: SendOptions) -> errors::Result<TransportPath> {
        let path = match options {
            SendOptions::IntermediatePath(mut path) => {
                path.push(destination);

                debug!(
                    full_path = format!("{path:?}"),
                    "Sending a message using a specific path"
                );

                let cg = self.channel_graph.read().await;

                TransportPath::resolve(path, &self.db, &cg).await.map(|(p, _)| p)?
            }
            SendOptions::Hops(hops) => {
                debug!(hops, "Sending a message using a random path");

                let pk = OffchainPublicKey::try_from(destination)?;

                if let Some(chain_key) = self
                    .db
                    .translate_key(None, pk)
                    .await
                    .map_err(hopr_db_sql::api::errors::DbError::from)?
                {
                    let selector = LegacyPathSelector::default();
                    let target_chain_key: Address = chain_key.try_into()?;
                    let cp = {
                        let cg = self.channel_graph.read().await;
                        selector.select_path(&cg, cg.my_address(), target_chain_key, hops as usize, hops as usize)?
                    };

                    cp.into_path(&self.db, target_chain_key).await?
                } else {
                    return Err(HoprTransportError::Api(
                        "send msg: unknown destination peer id encountered".to_owned(),
                    ));
                }
            }
        };

        #[cfg(all(feature = "prometheus", not(test)))]
        SimpleHistogram::observe(&METRIC_PATH_LENGTH, (path.hops().len() - 1) as f64);

        Ok(path)
    }
}

pub struct MessageSender<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    process_packet_send: Arc<OnceLock<PacketActions>>,
    resolver: Arc<RwLock<PathPlanner<T>>>,
}

impl<T> MessageSender<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    pub fn new(process_packet_send: Arc<OnceLock<PacketActions>>, resolver: Arc<RwLock<PathPlanner<T>>>) -> Self {
        Self {
            process_packet_send,
            resolver,
        }
    }
}

#[async_trait::async_trait]
impl<T> SendMsg for MessageSender<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: PeerId,
        options: SendOptions,
    ) -> std::result::Result<(), SessionError> {
        if let Some(application_tag) = data.application_tag {
            if application_tag < RESERVED_TAG_UPPER_LIMIT {
                return Err(SessionError::Tag);
            }
        } else {
            error!("Application tag must be set for an outgoing message inside a session");
            return Err(SessionError::Tag);
        }

        let path = self
            .resolver
            .write()
            .await
            .resolve_path(destination, options)
            .await
            .map_err(|_| SessionError::Path)?;

        let mut sender = self
            .process_packet_send
            .get()
            .ok_or_else(|| SessionError::Closed)?
            .clone();

        match sender.send_packet(data, path) {
            Ok(mut awaiter) => {
                awaiter
                    .consume_and_wait(crate::constants::PACKET_QUEUE_TIMEOUT_MILLISECONDS)
                    .await
                    .map_err(|_e| SessionError::Timeout)?;
                Ok(())
            }
            Err(_e) => Err(SessionError::Closed),
        }
    }
}

/// Interface into the physical transport mechanism allowing all HOPR related tasks on
/// the transport mechanism, as well as off-chain ticket manipulation.
pub struct HoprTransport<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    me: PeerId,
    me_onchain: Address,
    cfg: config::TransportConfig,
    hb_cfg: HeartbeatConfig,
    proto_cfg: core_protocol::config::ProtocolConfig,
    db: T,
    ping: Arc<OnceLock<Pinger<network_notifier::PingExternalInteractions<T>>>>,
    network: Arc<Network<T>>,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    my_multiaddresses: Vec<Multiaddr>,
    process_packet_send: Arc<OnceLock<PacketActions>>,
    process_ticket_aggregate:
        Arc<OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    sessions: moka::future::Cache<SessionId, UnboundedSender<ApplicationData>>,
}

impl<T> HoprTransport<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    #[allow(clippy::too_many_arguments)] // TODO: Needs refactoring and cleanup once rearchitected
    pub fn new(
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
        cfg: config::TransportConfig,
        proto_cfg: core_protocol::config::ProtocolConfig,
        hb_cfg: HeartbeatConfig,
        db: T,
        network: Arc<Network<T>>,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
        my_multiaddresses: Vec<Multiaddr>,
    ) -> Self {
        let identity: libp2p::identity::Keypair = (me).into();

        Self {
            me: identity.public().to_peer_id(),
            me_onchain: me_onchain.public().to_address(),
            cfg,
            hb_cfg,
            proto_cfg,
            db,
            ping: Arc::new(OnceLock::new()),
            network,
            channel_graph,
            my_multiaddresses,
            process_packet_send: Arc::new(OnceLock::new()),
            process_ticket_aggregate: Arc::new(OnceLock::new()),
            sessions: moka::future::Cache::new(RESERVED_TAG_UPPER_LIMIT as u64),
        }
    }

    pub fn me(&self) -> &PeerId {
        &self.me
    }

    /// Execute all processes of the [`HoprTransport`] object.
    ///
    /// This method will spawn the [`HoprTransportProcessType::Heartbeat`] and [`HoprTransportProcessType::SwarmEventLoop`] processes and return
    /// join handles to the calling function. Both processes are not started immediately, but are
    /// waiting for a trigger from this piece of code.
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        &self,
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
        version: String,
        network: Arc<Network<T>>,
        tbf: Arc<RwLock<TagBloomFilter>>,
        on_transport_output: UnboundedSender<TransportOutput>,
        on_acknowledged_ticket: UnboundedSender<AcknowledgedTicket>,
        transport_updates: UnboundedReceiver<PeerTransportEvent>,
    ) -> HashMap<HoprTransportProcess, executor::JoinHandle<()>> {
        // network event processing channel
        let (network_events_tx, network_events_rx) = futures::channel::mpsc::channel::<NetworkTriggeredEvent>(
            constants::MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE,
        );

        // manual ping
        let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let ping_cfg = PingConfig {
            timeout: self.proto_cfg.heartbeat.timeout,
            ..PingConfig::default()
        };

        let ping: Pinger<network_notifier::PingExternalInteractions<T>> = Pinger::new(
            ping_cfg,
            ping_tx.clone(),
            network_notifier::PingExternalInteractions::new(
                network.clone(),
                self.db.clone(),
                self.channel_graph.clone(),
                network_events_tx.clone(),
            ),
        );

        self.ping
            .clone()
            .set(ping)
            .expect("must set the ping executor only once");

        let transport_layer = HoprSwarm::new(me.into(), self.my_multiaddresses.clone(), self.proto_cfg).await;

        let ack_proc = AcknowledgementInteraction::new(self.db.clone(), me_onchain);

        let packet_proc = PacketInteraction::new(self.db.clone(), tbf, PacketInteractionConfig::new(me, me_onchain));
        self.process_packet_send
            .clone()
            .set(packet_proc.writer())
            .expect("must set the packet processing writer only once");

        let ticket_agg_proc = TicketAggregationInteraction::new(self.db.clone(), me_onchain);
        self.process_ticket_aggregate
            .clone()
            .set(ticket_agg_proc.writer())
            .expect("must set the ticket aggregation writer only once");

        // heartbeat
        let mut heartbeat = Heartbeat::new(
            self.hb_cfg,
            self.ping
                .get()
                .expect("Ping should be initialized at this point")
                .clone(),
            core_network::heartbeat::HeartbeatExternalInteractions::new(network.clone()),
            Box::new(|dur| Box::pin(executor::sleep(dur))),
        );

        let transport_layer = transport_layer.with_processors(
            network_events_rx,
            transport_updates,
            ack_proc,
            packet_proc,
            ticket_agg_proc,
            ping_rx,
        );

        let mut processes: HashMap<HoprTransportProcess, executor::JoinHandle<()>> = HashMap::new();

        processes.insert(
            HoprTransportProcess::Heartbeat,
            executor::spawn(async move { heartbeat.heartbeat_loop().await }),
        );

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<TransportOutput>();
        let sessions = self.sessions.clone();

        processes.insert(
            HoprTransportProcess::SessionUnwrapper,
            executor::spawn(async move {
                let mut on_transport_output = on_transport_output;

                while let Some(output) = rx.next().await {
                    match output {
                        TransportOutput::Received((peer, data)) => {
                            if let Some(app_tag) = data.application_tag {
                                if let Some(sender) = sessions.get(&SessionId::new(app_tag, peer)).await {
                                    // if the data does not get into the session, it can recover
                                    let _ = sender.unbounded_send(data);
                                    continue;
                                }
                            }

                            let _ = on_transport_output
                                .send(TransportOutput::Received((peer, data)))
                                .await
                                .map_err(|e| error!("Failed to send transport output: {e}"));
                        }
                        TransportOutput::ConnectionClosed(peer) => {
                            if let Err(e) = sessions.invalidate_entries_if(move |k, _v| k.peer() == &peer) {
                                error!("Failed to invalidate session entries: {e}")
                            };
                            continue;
                        }
                        _ => {
                            let _ = on_transport_output
                                .send(output)
                                .await
                                .map_err(|e| error!("Failed to send transport output: {e}"));
                        }
                    }
                }
            }),
        );

        processes.insert(
            HoprTransportProcess::Swarm,
            executor::spawn(transport_layer.run(version, tx, on_acknowledged_ticket)),
        );

        processes
    }

    pub fn ticket_aggregator(&self) -> Arc<dyn TicketAggregatorTrait + Send + Sync + 'static> {
        Arc::new(AggregatorProxy::new(
            self.db.clone(),
            self.process_ticket_aggregate.clone(),
            self.proto_cfg.ticket_aggregation.timeout,
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

        let timeout = executor::sleep(std::time::Duration::from_secs(30)).fuse();
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

    // #[cfg(feature = "session")]
    pub async fn new_session(&self, destination: PeerId, options: SendOptions) -> errors::Result<Session> {
        let session_id = SessionId::new(0u16, destination);

        let (tx, rx) = futures::channel::mpsc::unbounded::<ApplicationData>();

        self.sessions.insert(session_id, tx).await;

        Ok(Session::new(
            session_id,
            options,
            rx,
            Box::new(MessageSender::new(
                self.process_packet_send.clone(),
                Arc::new(RwLock::new(PathPlanner::new(
                    self.db.clone(),
                    self.channel_graph.clone(),
                ))),
            )),
        ))
    }

    #[tracing::instrument(level = "info", skip(self, msg), fields(uuid = uuid::Uuid::new_v4().to_string()))]
    pub async fn send_message(
        &self,
        msg: Box<[u8]>,
        destination: PeerId,
        options: SendOptions,
        application_tag: Option<u16>,
    ) -> errors::Result<HalfKeyChallenge> {
        if let Some(application_tag) = application_tag {
            if application_tag < RESERVED_TAG_UPPER_LIMIT {
                return Err(HoprTransportError::Api(format!(
                    "Application tag must not be lower than {RESERVED_TAG_UPPER_LIMIT}"
                )));
            }
        }

        if msg.len() > PAYLOAD_SIZE {
            return Err(HoprTransportError::Api(format!(
                "Message exceeds the maximum allowed size of {PAYLOAD_SIZE} bytes"
            )));
        }

        let app_data = ApplicationData::new_from_owned(application_tag, msg)?;

        let path = PathPlanner::new(self.db.clone(), self.channel_graph.clone())
            .resolve_path(destination, options)
            .await?;

        let mut sender = self
            .process_packet_send
            .get()
            .ok_or_else(|| {
                HoprTransportError::Api(
                    "send msg: failed to send a message, because message processing is not yet initialized".into(),
                )
            })?
            .clone();

        match sender.send_packet(app_data, path) {
            Ok(mut awaiter) => {
                tracing::trace!("Awaiting the HalfKeyChallenge");
                Ok(awaiter
                    .consume_and_wait(crate::constants::PACKET_QUEUE_TIMEOUT_MILLISECONDS)
                    .await?)
            }
            Err(e) => Err(HoprTransportError::Api(format!(
                "send msg: failed to enqueue msg send: {e}"
            ))),
        }
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
                    Err(core_protocol::errors::ProtocolError::ChannelNotFound.into())
                }
            })?;

        if entry.status != ChannelStatus::Open {
            return Err(core_protocol::errors::ProtocolError::ChannelClosed.into());
        }

        Ok(Arc::new(AggregatorProxy::new(
            self.db.clone(),
            self.process_ticket_aggregate.clone(),
            self.proto_cfg.ticket_aggregation.timeout,
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
                    && (self.cfg.announce_local_addresses || !hopr_transport_p2p::multiaddrs::is_private(ma))
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
