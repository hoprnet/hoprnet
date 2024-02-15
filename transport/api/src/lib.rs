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

use std::ops::Add;
pub use {
    crate::{
        adaptors::network::ExternalNetworkInteractions,
        multiaddrs::decapsulate_p2p_protocol,
        processes::indexer::IndexerProcessed,
        processes::indexer::{IndexerActions, IndexerToProcess, PeerEligibility},
    },
    core_network::network::{Health, Network, NetworkEvent, NetworkExternalActions, PeerOrigin, PeerStatus},
    core_p2p::libp2p,
    hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{HalfKeyChallenge, Hash, OffchainPublicKey},
    },
    hopr_internal_types::protocol::ApplicationData,
    multiaddr::Multiaddr,
    p2p::{api, p2p_loop},
    timer::execute_on_tick,
};

use async_lock::RwLock;
use chain_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
use core_network::{heartbeat::Heartbeat, messaging::ControlMessage, network::NetworkConfig, ping::Ping};
use core_network::{heartbeat::HeartbeatConfig, ping::PingConfig, PeerId};
use core_path::{channel_graph::ChannelGraph, DbPeerAddressResolver};
use core_protocol::{
    ack::processor::AcknowledgementInteraction,
    msg::processor::{PacketActions, PacketInteraction, PacketInteractionConfig},
    ticket_aggregation::processor::{TicketAggregationActions, TicketAggregationInteraction},
};
use futures::{
    channel::mpsc::{Receiver, UnboundedReceiver, UnboundedSender},
    FutureExt, SinkExt,
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::Address;
use libp2p::request_response::{OutboundRequestId, ResponseChannel};
use log::{debug, info, warn};
use std::sync::Arc;

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
pub fn build_network(
    peer_id: PeerId,
    cfg: NetworkConfig,
) -> (
    Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
    Receiver<NetworkEvent>,
) {
    let (network_events_tx, network_events_rx) =
        futures::channel::mpsc::channel::<NetworkEvent>(constants::MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE);

    let network = Arc::new(RwLock::new(Network::new(
        peer_id,
        cfg,
        adaptors::network::ExternalNetworkInteractions::new(network_events_tx),
    )));

    (network, network_events_rx)
}

/// Build the ticket aggregation mechanism and processes.
pub fn build_ticket_aggregation<Db>(
    db: Arc<RwLock<Db>>,
    chain_keypair: &ChainKeypair,
) -> TicketAggregationInteraction<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>
where
    Db: HoprCoreEthereumDbActions + Send + Sync + 'static,
{
    TicketAggregationInteraction::new(db, chain_keypair)
}

type HoprPingComponents = (
    Ping<adaptors::ping::PingExternalInteractions<DbPeerAddressResolver>>,
    UnboundedReceiver<(PeerId, ControlMessage)>,
    UnboundedSender<(PeerId, std::result::Result<(ControlMessage, String), ()>)>,
);

/// Build the ping infrastructure to run manual pings from the interface.
pub fn build_manual_ping(
    cfg: core_protocol::config::ProtocolConfig,
    network: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
    addr_resolver: DbPeerAddressResolver,
    channel_graph: Arc<RwLock<ChannelGraph>>,
) -> HoprPingComponents {
    let (ping_tx, ping_rx) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
    let (pong_tx, pong_rx) =
        futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

    let ping_cfg = PingConfig {
        timeout: cfg.heartbeat.timeout,
        ..PingConfig::default()
    };

    // manual ping explicitly called by the API
    let ping: Ping<adaptors::ping::PingExternalInteractions<DbPeerAddressResolver>> = Ping::new(
        ping_cfg,
        ping_tx,
        pong_rx,
        adaptors::ping::PingExternalInteractions::new(network, addr_resolver, channel_graph),
    );

    (ping, ping_rx, pong_tx)
}

/// Build the index updater mechanism for indexer generated behavior inclusion.
pub fn build_index_updater<Db>(
    db: Arc<RwLock<Db>>,
    network: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
) -> (processes::indexer::IndexerActions, Receiver<IndexerProcessed>)
where
    Db: HoprCoreEthereumDbActions + Send + Sync + 'static,
{
    let (indexer_update_tx, indexer_update_rx) =
        futures::channel::mpsc::channel::<IndexerProcessed>(constants::INDEXER_UPDATE_QUEUE_SIZE);
    let indexer_updater = processes::indexer::IndexerActions::new(db, network, indexer_update_tx);

    (indexer_updater, indexer_update_rx)
}

/// Build the interaction object allowing to process messages.
pub fn build_packet_actions<Db>(
    me: &OffchainKeypair,
    me_onchain: &ChainKeypair,
    db: Arc<RwLock<Db>>,
    tbf: Arc<RwLock<TagBloomFilter>>,
) -> (PacketInteraction, AcknowledgementInteraction)
where
    Db: HoprCoreEthereumDbActions + std::marker::Send + std::marker::Sync + 'static,
{
    (
        PacketInteraction::new(db.clone(), tbf, PacketInteractionConfig::new(me, me_onchain)),
        AcknowledgementInteraction::new(db, me_onchain),
    )
}

type HoprHearbeat = Heartbeat<
    Ping<adaptors::ping::PingExternalInteractions<DbPeerAddressResolver>>,
    adaptors::heartbeat::HeartbeatExternalInteractions,
>;

type HoprHeartbeatComponents = (
    HoprHearbeat,
    UnboundedReceiver<(PeerId, ControlMessage)>,
    UnboundedSender<(PeerId, std::result::Result<(ControlMessage, String), ()>)>,
);

/// Build the heartbeat mechanism for network probing.
pub fn build_heartbeat(
    proto_cfg: core_protocol::config::ProtocolConfig,
    hb_cfg: HeartbeatConfig,
    network: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
    addr_resolver: DbPeerAddressResolver,
    channel_graph: Arc<RwLock<ChannelGraph>>,
) -> HoprHeartbeatComponents {
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
        adaptors::ping::PingExternalInteractions::new(network.clone(), addr_resolver, channel_graph),
    );
    let heartbeat = Heartbeat::new(
        hb_cfg,
        hb_pinger,
        adaptors::heartbeat::HeartbeatExternalInteractions::new(network),
    );

    (heartbeat, hb_ping_rx, hb_pong_tx)
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
    pub losing: u64,
    pub win_proportion: f64,
    pub unredeemed: u64,
    pub unredeemed_value: hopr_primitive_types::primitives::Balance,
    pub redeemed: u64,
    pub redeemed_value: hopr_primitive_types::primitives::Balance,
    pub neglected: u64,
    pub neglected_value: hopr_primitive_types::primitives::Balance,
    pub rejected: u64,
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
use core_protocol::ticket_aggregation::processor::AggregationList;
use futures::future::{select, Either};
use futures::pin_mut;
use hopr_internal_types::channels::ChannelStatus;
use hopr_primitive_types::prelude::*;

/// Interface into the physical transport mechanism allowing all HOPR related tasks on
/// the transport mechanism, as well as off-chain ticket manipulation.
#[derive(Debug, Clone)]
pub struct HoprTransport {
    me: PeerId,
    me_onchain: Address,
    cfg: config::TransportConfig,
    db: Arc<RwLock<CoreEthereumDb<utils_db::CurrentDbShim>>>,
    ping: Arc<RwLock<Ping<adaptors::ping::PingExternalInteractions<DbPeerAddressResolver>>>>,
    network: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
    indexer: processes::indexer::IndexerActions,
    pkt_sender: PacketActions,
    ticket_aggregate_actions: TicketAggregationActions<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    my_multiaddresses: Vec<Multiaddr>,
}

impl HoprTransport {
    #[allow(clippy::too_many_arguments)] // TODO: Needs refactoring and cleanup once rearchitected
    pub fn new(
        identity: libp2p::identity::Keypair,
        me_onchain: ChainKeypair,
        cfg: config::TransportConfig,
        db: Arc<RwLock<CoreEthereumDb<utils_db::CurrentDbShim>>>,
        ping: Ping<adaptors::ping::PingExternalInteractions<DbPeerAddressResolver>>,
        network: Arc<RwLock<Network<adaptors::network::ExternalNetworkInteractions>>>,
        indexer: processes::indexer::IndexerActions,
        pkt_sender: PacketActions,
        ticket_aggregate_actions: TicketAggregationActions<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
        my_multiaddresses: Vec<Multiaddr>,
    ) -> Self {
        Self {
            me: identity.public().to_peer_id(),
            me_onchain: me_onchain.public().to_address(),
            cfg,
            db,
            ping: Arc::new(RwLock::new(ping)),
            network,
            indexer,
            pkt_sender,
            ticket_aggregate_actions,
            channel_graph,
            my_multiaddresses,
        }
    }

    pub fn me(&self) -> &PeerId {
        &self.me
    }

    pub fn index_updater(&self) -> IndexerActions {
        self.indexer.clone()
    }

    pub async fn init_from_db(&self) -> errors::Result<()> {
        info!("Loading initial peers from the storage");

        let index_updater = self.index_updater();

        for (peer, _address, multiaddresses) in self.get_public_nodes().await? {
            if self.is_allowed_to_access_network(&peer).await {
                debug!("Using initial public node '{peer}' with '{:?}'", multiaddresses);
                index_updater
                    .emit_indexer_update(IndexerToProcess::EligibilityUpdate(peer, PeerEligibility::Eligible))
                    .await;

                index_updater
                    .emit_indexer_update(IndexerToProcess::Announce(peer, multiaddresses.clone()))
                    .await;

                let mut net = self.network.write().await;
                if !net.has(&peer) {
                    net.add(&peer, PeerOrigin::Initialization);
                    net.store_peer_multiaddresses(&peer, multiaddresses);
                }
            }
        }

        Ok(())
    }

    pub async fn ping(&self, peer: &PeerId) -> errors::Result<Option<std::time::Duration>> {
        if !self.is_allowed_to_access_network(peer).await {
            return Err(errors::HoprTransportError::Api(format!(
                "ping to {peer} not allowed due to network registry"
            )));
        }

        let mut pinger = self.ping.write().await;

        // TODO: add timeout on the p2p transport layer
        let timeout = sleep(std::time::Duration::from_secs(30)).fuse();
        let ping = (*pinger).ping(vec![*peer]).fuse();

        pin_mut!(timeout, ping);

        let has_peer = self.network.read().await.has(peer);
        if !has_peer {
            self.network.write().await.add(peer, PeerOrigin::ManualPing)
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
            .read()
            .await
            .get_peer_status(peer)
            .map(|status| std::time::Duration::from_millis(status.last_seen).saturating_sub(start)))
    }

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

            let cg = self.channel_graph.read().await;

            TransportPath::resolve(full_path, &DbPeerAddressResolver(self.db.clone()), &cg)
                .await
                .map(|(p, _)| p)?
        } else if let Some(hops) = hops {
            let pk = OffchainPublicKey::try_from(destination)?;

            if let Some(chain_key) = self.db.read().await.get_chain_key(&pk).await? {
                let selector = LegacyPathSelector::default();
                let cp = {
                    let cg = self.channel_graph.read().await;
                    // Sends 1-hop packet if > 1-hop does not work
                    selector.select_path(&cg, cg.my_address(), chain_key, 1, hops as usize)?
                };

                cp.into_path(&DbPeerAddressResolver(self.db.clone()), chain_key).await?
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
                log::trace!("Awaiting the HalfKeyChallenge");
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

    pub async fn aggregate_tickets(&self, channel: &Hash) -> errors::Result<()> {
        let entry = self
            .db
            .read()
            .await
            .get_channel(channel)
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
            .aggregate_tickets(AggregationList::WholeChannel(entry))?
            .consume_and_wait(std::time::Duration::from_millis(60000))
            .await?)
    }

    pub async fn get_public_nodes(&self) -> errors::Result<Vec<(PeerId, Address, Vec<Multiaddr>)>> {
        let db = self.db.read().await;

        let mut public_nodes = vec![];

        for node in db.get_public_node_accounts().await?.into_iter() {
            if let Ok(Some(v)) = db.get_packet_key(&node.chain_addr).await {
                public_nodes.push((
                    v.into(),
                    node.chain_addr,
                    if let Some(ma) = node.get_multiaddr() {
                        vec![ma]
                    } else {
                        vec![]
                    },
                ))
            }
        }

        Ok(public_nodes)
    }

    pub async fn is_allowed_to_access_network(&self, peer: &PeerId) -> bool {
        let db = self.db.read().await;

        if let Ok(pk) = OffchainPublicKey::try_from(peer) {
            if let Some(address) = db.get_chain_key(&pk).await.unwrap_or(None) {
                return db.is_allowed_to_access_network(&address).await.unwrap_or(false);
            }
        }

        false
    }

    pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        self.network.read().await.get_peer_multiaddresses(&self.me)
    }

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

    pub async fn multiaddresses_announced_to_dht(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.network.read().await.get_peer_multiaddresses(peer)
    }

    pub async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.network.read().await.get_peer_multiaddresses(peer)
    }

    pub async fn network_health(&self) -> Health {
        self.network.read().await.health()
    }

    pub async fn network_connected_peers(&self) -> Vec<PeerId> {
        self.network.read().await.get_all_peers()
    }

    pub async fn network_peer_info(&self, peer: &PeerId) -> Option<PeerStatus> {
        self.network.read().await.get_peer_status(peer)
    }

    pub async fn ticket_statistics(&self) -> errors::Result<TicketStatistics> {
        let db = self.db.read().await;

        let acked_ticket_amounts = db
            .get_acknowledged_tickets(None)
            .await
            .map(|v| v.into_iter().map(|at| at.ticket.amount).collect::<Vec<_>>())?;

        let losing = db.get_losing_tickets_count().await?;

        let total_value = acked_ticket_amounts
            .iter()
            .fold(Balance::zero(BalanceType::HOPR), |sum, val| sum.add(val));

        Ok(TicketStatistics {
            win_proportion: if (acked_ticket_amounts.len() + losing) > 0 {
                acked_ticket_amounts.len() as f64 / (acked_ticket_amounts.len() + losing) as f64
            } else {
                0f64
            },
            losing: losing as u64,
            unredeemed: acked_ticket_amounts.len() as u64,
            unredeemed_value: total_value,
            redeemed: db.get_redeemed_tickets_count().await? as u64,
            redeemed_value: db.get_redeemed_tickets_value().await?,
            neglected: db.get_neglected_tickets_count().await? as u64,
            neglected_value: db.get_neglected_tickets_value().await?,
            rejected: db.get_rejected_tickets_count().await? as u64,
            rejected_value: db.get_rejected_tickets_value().await?,
        })
    }

    pub async fn tickets_in_channel(&self, channel: &Hash) -> errors::Result<Option<Vec<AcknowledgedTicket>>> {
        let db = self.db.read().await;

        let channel = db.get_channel(channel).await?;

        if let Some(channel) = channel {
            if channel.destination == self.me_onchain {
                Ok(Some(db.get_acknowledged_tickets(Some(channel)).await?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn all_tickets(&self) -> errors::Result<Vec<Ticket>> {
        Ok(self
            .db
            .read()
            .await
            .get_acknowledged_tickets(None)
            .await
            .map(|tickets| {
                tickets
                    .into_iter()
                    .map(|acked_ticket| acked_ticket.ticket)
                    .collect::<Vec<_>>()
            })?)
    }
}
