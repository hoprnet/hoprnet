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

pub mod adaptors;
/// Configuration of the [HoprTransport].
pub mod config;
/// Constants used and exposed by the crate.
pub mod constants;
/// Errors used by the crate.
pub mod errors;
mod multiaddrs;
mod p2p;
mod processes;
mod timer;

/// Composite output from the transport layer.
#[derive(Clone)]
pub enum TransportOutput {
    Received(ApplicationData),
    Sent(HalfKeyChallenge),
}

use std::{collections::HashMap, sync::Arc};
pub use {
    crate::{
        multiaddrs::decapsulate_p2p_protocol,
        processes::indexer::IndexerProcessed,
        processes::indexer::{IndexerActions, IndexerToProcess, PeerEligibility},
    },
    core_network::network::{Health, Network, NetworkTriggeredEvent, PeerOrigin, PeerStatus},
    core_p2p::libp2p,
    hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{HalfKeyChallenge, Hash, OffchainPublicKey},
    },
    hopr_internal_types::protocol::ApplicationData,
    multiaddr::Multiaddr,
    p2p::{api, SwarmEventLoop},
    timer::execute_on_tick,
};

use async_lock::RwLock;
use async_std::task::{spawn, JoinHandle};
use hopr_db_api::{
    peers::HoprDbPeersOperations, registry::HoprDbRegistryOperations, resolver::HoprDbResolverOperations,
    HoprDbAllOperations,
};
use libp2p::request_response::{OutboundRequestId, ResponseChannel};
use tracing::{debug, error, info, warn};

use core_network::{heartbeat::Heartbeat, messaging::ControlMessage, network::NetworkConfig, ping::Ping};
use core_network::{heartbeat::HeartbeatConfig, ping::PingConfig, PeerId};
use core_protocol::{
    ack::processor::AcknowledgementInteraction,
    msg::processor::{PacketActions, PacketInteraction, PacketInteractionConfig},
    ticket_aggregation::processor::{TicketAggregationActions, TicketAggregationInteraction},
};
use futures::{
    channel::mpsc::{Receiver, UnboundedSender},
    FutureExt, SinkExt,
};
use hopr_internal_types::prelude::*;
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

use {async_std::task::sleep, hopr_platform::time::native::current_time};

/// Build the [Network] object responsible for tracking and holding the
/// observable state of the physical transport network, peers inside the network
/// and telemetry about network connections.
pub fn build_network<T>(peer_id: PeerId, addresses: Vec<Multiaddr>, cfg: NetworkConfig, db: T) -> Arc<Network<T>>
where
    T: HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
{
    Arc::new(Network::new(peer_id, addresses, cfg, db))
}

/// Build the index updater mechanism for indexer generated behavior inclusion.
pub fn build_index_updater<T>(
    db: T,
    network: Arc<Network<T>>,
) -> (processes::indexer::IndexerActions, Receiver<IndexerProcessed>)
where
    T: HoprDbPeersOperations
        + HoprDbResolverOperations
        + HoprDbRegistryOperations
        + Send
        + Sync
        + 'static
        + std::fmt::Debug
        + Clone,
{
    let (indexer_update_tx, indexer_update_rx) =
        futures::channel::mpsc::channel::<IndexerProcessed>(constants::INDEXER_UPDATE_QUEUE_SIZE);
    let indexer_updater = processes::indexer::IndexerActions::new(db, network, indexer_update_tx);

    (indexer_updater, indexer_update_rx)
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
use hopr_db_api::errors::DbError;
use hopr_internal_types::channels::ChannelStatus;
use hopr_primitive_types::prelude::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum HoprTransportProcess {
    Heartbeat,
    Swarm,
}

// reason for using `async_channel` is that the rx can be cloned
type HoprTransportProcessType<T> = (
    async_channel::Sender<HoprSwarmArgs<T>>,
    async_channel::Receiver<HashMap<HoprTransportProcess, JoinHandle<()>>>,
);

/// An object used to collect and pass along the arguments necessary for runtime creation
/// of the HoprSwarm process inside the `HoprTransport` object.
struct HoprSwarmArgs<T>
where
    T: hopr_db_api::peers::HoprDbPeersOperations + Sync + Send + std::fmt::Debug + 'static,
{
    pub version: String,
    pub network: Arc<Network<T>>,
    pub protocol_cfg: crate::config::ProtocolConfig,
    pub on_transport_output: UnboundedSender<TransportOutput>,
    pub on_acknowledged_ticket: UnboundedSender<AcknowledgedTicket>,
}

/// Interface into the physical transport mechanism allowing all HOPR related tasks on
/// the transport mechanism, as well as off-chain ticket manipulation.
#[derive(Debug)]
pub struct HoprTransport<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    me: PeerId,
    me_onchain: Address,
    cfg: config::TransportConfig,
    db: T,
    ping: Arc<RwLock<Ping<adaptors::ping::PingExternalInteractions<T>>>>,
    processes: HoprTransportProcessType<T>,
    network: Arc<Network<T>>,
    indexer: processes::indexer::IndexerActions,
    pkt_sender: PacketActions,
    ticket_aggregate_actions: TicketAggregationActions<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    my_multiaddresses: Vec<Multiaddr>,
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
        tbf: Arc<RwLock<TagBloomFilter>>,
        network: Arc<Network<T>>,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
        my_multiaddresses: Vec<Multiaddr>,
    ) -> Self {
        // protocol section
        // - actions on processed packets
        let packet_actions = PacketInteraction::new(db.clone(), tbf, PacketInteractionConfig::new(me, me_onchain));
        let ack_actions = AcknowledgementInteraction::new(db.clone(), me_onchain);
        // - ticket aggregation protocol
        let ticket_aggregation = TicketAggregationInteraction::new(db.clone(), me_onchain);

        // index updater
        let (indexer_updater, indexer_update_rx) = build_index_updater(db.clone(), network.clone());

        // network event processing channel
        let (network_events_tx, network_events_rx) = futures::channel::mpsc::channel::<NetworkTriggeredEvent>(
            constants::MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE,
        );

        // manual ping
        let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (pong_tx, pong_rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let ping_cfg = PingConfig {
            timeout: proto_cfg.heartbeat.timeout,
            ..PingConfig::default()
        };

        let ping: Ping<adaptors::ping::PingExternalInteractions<T>> = Ping::new(
            ping_cfg,
            ping_tx,
            pong_rx,
            adaptors::ping::PingExternalInteractions::new(
                network.clone(),
                db.clone(),
                channel_graph.clone(),
                network_events_tx.clone(),
            ),
        );

        // heartbeat
        let (hb_ping_tx, hb_ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (hb_pong_tx, hb_pong_rx) = futures::channel::mpsc::unbounded::<(
            libp2p::identity::PeerId,
            std::result::Result<(ControlMessage, String), ()>,
        )>();

        let ping_cfg = PingConfig {
            timeout: proto_cfg.heartbeat.timeout,
            ..PingConfig::default()
        };

        let hb_pinger = Ping::new(
            ping_cfg,
            hb_ping_tx,
            hb_pong_rx,
            adaptors::ping::PingExternalInteractions::new(
                network.clone(),
                db.clone(),
                channel_graph.clone(),
                network_events_tx,
            ),
        );
        let heartbeat = Heartbeat::new(
            hb_cfg,
            hb_pinger,
            core_network::heartbeat::HeartbeatExternalInteractions::new(network.clone()),
        );

        let pkt_sender = packet_actions.writer();
        let ticket_aggregate_actions = ticket_aggregation.writer();

        let identity: libp2p::identity::Keypair = (me).into();

        let swarm_loop = SwarmEventLoop::new(
            identity.clone(),
            my_multiaddresses.clone(),
            network_events_rx,
            indexer_update_rx,
            ack_actions,
            packet_actions,
            ticket_aggregation,
            core_p2p::api::HeartbeatRequester::new(hb_ping_rx),
            core_p2p::api::HeartbeatResponder::new(hb_pong_tx),
            core_p2p::api::ManualPingRequester::new(ping_rx),
            core_p2p::api::HeartbeatResponder::new(pong_tx),
        );

        let (push_tx, push_rx) = async_channel::bounded::<HoprSwarmArgs<T>>(1);
        let (pull_tx, pull_rx) =
            async_channel::bounded::<std::collections::HashMap<HoprTransportProcess, JoinHandle<()>>>(1);

        // NOTE: This spawned task does not need to be explicitly canceled, since it will
        // be automatically dropped when the event sender object is dropped.
        spawn(async move {
            let mut heartbeat = heartbeat;
            let swarm_loop = swarm_loop;

            match push_rx.recv().await {
                Ok(args) => {
                    let mut r = HashMap::new();
                    r.insert(
                        HoprTransportProcess::Heartbeat,
                        spawn(async move { heartbeat.heartbeat_loop().await }),
                    );
                    r.insert(
                        HoprTransportProcess::Swarm,
                        spawn(swarm_loop.run(
                            args.version,
                            args.network,
                            args.protocol_cfg,
                            args.on_transport_output,
                            args.on_acknowledged_ticket,
                        )),
                    );
                    pull_tx
                        .send(r)
                        .await
                        .expect("Failed to send process handles to the invocation site");
                }
                Err(e) => panic!(
                    "Failed to receive the push message from HoprTransport object to initiate process spawning: {e}"
                ),
            }
        });

        Self {
            me: identity.public().to_peer_id(),
            me_onchain: me_onchain.public().to_address(),
            cfg,
            db,
            processes: (push_tx, pull_rx),
            ping: Arc::new(RwLock::new(ping)),
            network,
            indexer: indexer_updater,
            pkt_sender,
            ticket_aggregate_actions,
            channel_graph,
            my_multiaddresses,
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
    ///
    /// This is a hack around the fact that `futures::channel::mpsc::oneshot`
    pub async fn run(
        &self,
        version: String,
        network: Arc<Network<T>>,
        protocol_cfg: crate::config::ProtocolConfig,
        on_transport_output: UnboundedSender<TransportOutput>,
        on_acknowledged_ticket: UnboundedSender<AcknowledgedTicket>,
    ) -> std::collections::HashMap<HoprTransportProcess, JoinHandle<()>> {
        let rx = self.processes.1.clone();
        self.processes
            .0
            .clone()
            .send(HoprSwarmArgs {
                version,
                network,
                protocol_cfg,
                on_transport_output,
                on_acknowledged_ticket,
            })
            .await
            .unwrap();
        rx.recv().await.unwrap()
    }

    pub fn ticket_aggregator_writer(
        &self,
    ) -> TicketAggregationActions<ResponseChannel<Result<Ticket, String>>, OutboundRequestId> {
        self.ticket_aggregate_actions.clone()
    }

    pub fn index_updater(&self) -> IndexerActions {
        self.indexer.clone()
    }

    pub async fn init_from_db(&self) -> errors::Result<()> {
        let index_updater = self.index_updater();

        for (peer, _address, multiaddresses) in self.get_public_nodes().await? {
            if self.is_allowed_to_access_network(&peer).await? {
                debug!("Using initial public node '{peer}' with '{:?}'", multiaddresses);
                index_updater
                    .emit_indexer_update(IndexerToProcess::EligibilityUpdate(peer, PeerEligibility::Eligible))
                    .await;

                index_updater
                    .emit_indexer_update(IndexerToProcess::Announce(peer, multiaddresses.clone()))
                    .await;

                // Self-reference is not needed in the network storage
                if &peer != self.me() {
                    if let Err(e) = self
                        .network
                        .add(&peer, PeerOrigin::Initialization, multiaddresses)
                        .await
                    {
                        error!("Failed to store the peer observation: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<Option<std::time::Duration>> {
        if !self.is_allowed_to_access_network(peer).await? {
            return Err(errors::HoprTransportError::Api(format!(
                "ping to {peer} not allowed due to network registry"
            )));
        }

        let mut pinger = self.ping.write().await;

        // TODO: add timeout on the p2p transport layer
        let timeout = sleep(std::time::Duration::from_secs(30)).fuse();
        let ping = (*pinger).ping(vec![*peer]).fuse();

        pin_mut!(timeout, ping);

        if let Err(e) = self.network.add(peer, PeerOrigin::ManualPing, vec![]).await {
            error!("Failed to store the peer observation: {e}");
        }

        let start = current_time().as_unix_timestamp();

        match select(timeout, ping).await {
            Either::Left(_) => {
                warn!("Manual ping to peer '{}' timed out", peer);
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

    #[tracing::instrument(level = "info", skip(self, msg), fields(uuid = uuid::Uuid::new_v4().to_string()))]
    pub async fn send_message(
        &self,
        msg: Box<[u8]>,
        destination: PeerId,
        intermediate_path: Option<Vec<PeerId>>,
        hops: Option<u16>,
        application_tag: Option<u16>,
    ) -> crate::errors::Result<HalfKeyChallenge> {
        let app_data = ApplicationData::new(application_tag, &msg)?;

        let path: TransportPath = if let Some(intermediate_path) = intermediate_path {
            let mut full_path = intermediate_path;
            full_path.push(destination);

            debug!(
                full_path = format!("{:?}", full_path),
                "Sending a message using full path"
            );

            let cg = self.channel_graph.read().await;

            TransportPath::resolve(full_path, &self.db, &cg).await.map(|(p, _)| p)?
        } else if let Some(hops) = hops {
            debug!(hops, "Sending a message using hops");

            let pk = OffchainPublicKey::try_from(destination)?;

            if let Some(chain_key) = self.db.translate_key(None, pk).await? {
                let selector = LegacyPathSelector::default();
                let target_chain_key: Address = chain_key.try_into()?;
                let cp = {
                    let cg = self.channel_graph.read().await;
                    selector.select_path(&cg, cg.my_address(), target_chain_key, hops as usize, hops as usize)?
                };

                cp.into_path(&self.db, target_chain_key).await?
            } else {
                return Err(crate::errors::HoprTransportError::Api(
                    "send msg: unknown destination peer id encountered".to_owned(),
                ));
            }
        } else {
            return Err(crate::errors::HoprTransportError::Api(
                "send msg: one of either hops or intermediate path must be specified".to_owned(),
            ));
        };

        #[cfg(all(feature = "prometheus", not(test)))]
        SimpleHistogram::observe(&METRIC_PATH_LENGTH, (path.hops().len() - 1) as f64);

        match self.pkt_sender.clone().send_packet(app_data, path) {
            Ok(mut awaiter) => {
                tracing::trace!("Awaiting the HalfKeyChallenge");
                Ok(awaiter
                    .consume_and_wait(crate::constants::PACKET_QUEUE_TIMEOUT_MILLISECONDS)
                    .await?)
            }
            Err(e) => Err(crate::errors::HoprTransportError::Api(format!(
                "send msg: failed to enqueue msg send: {}",
                e
            ))),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn aggregate_tickets(&self, channel_id: &Hash) -> errors::Result<()> {
        let entry = self
            .db
            .get_channel_by_id(None, channel_id)
            .await
            .map_err(errors::HoprTransportError::from)
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

        Ok(self
            .ticket_aggregate_actions
            .clone()
            .aggregate_tickets(&entry.get_id(), Default::default())?
            .consume_and_wait(std::time::Duration::from_millis(60000))
            .await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_public_nodes(&self) -> errors::Result<Vec<(PeerId, Address, Vec<Multiaddr>)>> {
        Ok(self
            .db
            .get_accounts(None, true)
            .await?
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
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let pk = OffchainPublicKey::try_from(peer)?;
                    if let Some(address) = db_clone.translate_key(Some(tx), pk).await? {
                        db_clone
                            .is_allowed_in_network_registry(Some(tx), address.try_into()?)
                            .await
                    } else {
                        Err(DbError::LogicalError("cannot translate off-chain key".into()))
                    }
                })
            })
            .await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        // TODO: can fail with the Result?
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
                crate::multiaddrs::is_supported(ma)
                    && (self.cfg.announce_local_addresses || !crate::multiaddrs::is_private(ma))
            })
            .map(|ma| crate::multiaddrs::decapsulate_p2p_protocol(&ma))
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

    pub fn local_multiaddresses(&self) -> Vec<Multiaddr> {
        self.my_multiaddresses.clone()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn multiaddresses_announced_to_dht(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.network
            .get(peer)
            .await
            .unwrap_or(None)
            .map(|peer| peer.multiaddresses)
            .unwrap_or(vec![])
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
        // TODO: add parameter to specify which channel are we interested in
        let ticket_stats = self.db.get_ticket_statistics(None, None).await?;

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
        if let Some(channel) = self.db.get_channel_by_id(None, channel_id).await? {
            if channel.destination == self.me_onchain {
                Ok(Some(self.db.get_tickets(None, (&channel).into()).await?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn all_tickets(&self) -> errors::Result<Vec<Ticket>> {
        Ok(self.db.get_all_tickets().await?.into_iter().map(|v| v.ticket).collect())
    }
}
