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

use std::sync::Arc;
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
    p2p::{api, p2p_loop},
    timer::execute_on_tick,
};

use async_lock::RwLock;
use hopr_db_api::{
    peers::HoprDbPeersOperations, registry::HoprDbRegistryOperations, resolver::HoprDbResolverOperations,
    tickets::HoprDbTicketOperations, HoprDbAllOperations,
};
use libp2p::request_response::{OutboundRequestId, ResponseChannel};
use tracing::{debug, error, info, warn};

use core_network::{heartbeat::Heartbeat, messaging::ControlMessage, network::NetworkConfig, ping::Ping};
use core_network::{heartbeat::HeartbeatConfig, ping::PingConfig, PeerId};
use core_path::channel_graph::ChannelGraph;
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

/// Build the ticket aggregation mechanism and processes.
pub fn build_ticket_aggregation<Db>(
    db: Db,
    chain_keypair: &ChainKeypair,
) -> TicketAggregationInteraction<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug + 'static,
{
    TicketAggregationInteraction::new(db, chain_keypair)
}

type HoprPingComponents<T> = (
    Ping<adaptors::ping::PingExternalInteractions<T>>,
    UnboundedReceiver<(PeerId, ControlMessage)>,
    UnboundedSender<(PeerId, std::result::Result<(ControlMessage, String), ()>)>,
);

pub fn build_transport_components<T>(
    proto_cfg: core_protocol::config::ProtocolConfig,
    hb_cfg: HeartbeatConfig,
    network: Arc<Network<T>>,
    addr_resolver: T,
    channel_graph: Arc<RwLock<ChannelGraph>>,
) -> (
    HoprPingComponents<T>,
    HoprHeartbeatComponents<T>,
    Receiver<NetworkTriggeredEvent>,
)
where
    T: HoprDbPeersOperations + hopr_db_api::resolver::HoprDbResolverOperations + std::fmt::Debug + Clone + Sync + Send,
{
    let (network_events_tx, network_events_rx) =
        futures::channel::mpsc::channel::<NetworkTriggeredEvent>(constants::MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE);

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
            addr_resolver.clone(),
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
        adaptors::ping::PingExternalInteractions::new(network.clone(), addr_resolver, channel_graph, network_events_tx),
    );
    let heartbeat = Heartbeat::new(
        hb_cfg,
        hb_pinger,
        core_network::heartbeat::HeartbeatExternalInteractions::new(network),
    );

    (
        (ping, ping_rx, pong_tx),
        (heartbeat, hb_ping_rx, hb_pong_tx),
        network_events_rx,
    )
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

/// Build the interaction object allowing to process messages.
pub fn build_packet_actions<Db>(
    me: &OffchainKeypair,
    me_onchain: &ChainKeypair,
    db: Db,
    tbf: Arc<RwLock<TagBloomFilter>>,
) -> (PacketInteraction, AcknowledgementInteraction)
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone + 'static,
{
    (
        PacketInteraction::new(db.clone(), tbf, PacketInteractionConfig::new(me, me_onchain)),
        AcknowledgementInteraction::new(db, me_onchain),
    )
}

type HoprHearbeat<T> = Heartbeat<
    Ping<adaptors::ping::PingExternalInteractions<T>>,
    core_network::heartbeat::HeartbeatExternalInteractions<T>,
>;

type HoprHeartbeatComponents<T> = (
    HoprHearbeat<T>,
    UnboundedReceiver<(PeerId, ControlMessage)>,
    UnboundedSender<(PeerId, std::result::Result<(ControlMessage, String), ()>)>,
);

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
    pub losing: u128,
    pub win_proportion: f64,
    pub unredeemed: u128,
    pub unredeemed_value: hopr_primitive_types::primitives::Balance,
    pub redeemed: u128,
    pub redeemed_value: hopr_primitive_types::primitives::Balance,
    pub neglected: u128,
    pub neglected_value: hopr_primitive_types::primitives::Balance,
    pub rejected: u128,
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
use hopr_db_api::prelude::HoprDbProtocolOperations;
use hopr_internal_types::channels::ChannelStatus;
use hopr_primitive_types::prelude::*;

/// Interface into the physical transport mechanism allowing all HOPR related tasks on
/// the transport mechanism, as well as off-chain ticket manipulation.
#[derive(Debug, Clone)]
pub struct HoprTransport<T>
where
    T: HoprDbAllOperations + hopr_db_api::resolver::HoprDbResolverOperations + std::fmt::Debug + Clone + Send + Sync,
{
    me: PeerId,
    me_onchain: Address,
    cfg: config::TransportConfig,
    db: T,
    ping: Arc<RwLock<Ping<adaptors::ping::PingExternalInteractions<T>>>>,
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
        identity: libp2p::identity::Keypair,
        me_onchain: ChainKeypair,
        cfg: config::TransportConfig,
        db: T,
        ping: Ping<adaptors::ping::PingExternalInteractions<T>>,
        network: Arc<Network<T>>,
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
            if self.is_allowed_to_access_network(&peer).await? {
                debug!("Using initial public node '{peer}' with '{:?}'", multiaddresses);
                index_updater
                    .emit_indexer_update(IndexerToProcess::EligibilityUpdate(peer, PeerEligibility::Eligible))
                    .await;

                index_updater
                    .emit_indexer_update(IndexerToProcess::Announce(peer, multiaddresses.clone()))
                    .await;

                if let Err(e) = self
                    .network
                    .add(&peer, PeerOrigin::Initialization, multiaddresses)
                    .await
                {
                    error!("Failed to store the peer observation: {e}");
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
            .aggregate_tickets(&entry.get_id(), None)?
            .consume_and_wait(std::time::Duration::from_millis(60000))
            .await?)
    }

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

    pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        // TODO: can fail with the Result?
        self.network
            .get(&self.me)
            .await
            .unwrap_or(None)
            .map(|peer| peer.multiaddresses)
            .unwrap_or(vec![])
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
        self.network
            .get(peer)
            .await
            .unwrap_or(None)
            .map(|peer| peer.multiaddresses)
            .unwrap_or(vec![])
    }

    pub async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.network
            .get(peer)
            .await
            .unwrap_or(None)
            .map(|peer| peer.multiaddresses)
            .unwrap_or(vec![])
    }

    pub async fn network_health(&self) -> Health {
        self.network.health().await
    }

    pub async fn network_connected_peers(&self) -> errors::Result<Vec<PeerId>> {
        Ok(self.network.peer_filter(|peer| async move { Some(peer.id) }).await?)
    }

    pub async fn network_peer_info(&self, peer: &PeerId) -> errors::Result<Option<PeerStatus>> {
        Ok(self.network.get(peer).await?)
    }

    pub async fn ticket_statistics(&self) -> errors::Result<TicketStatistics> {
        // TODO: add parameter to specify which channel are we interested in
        let ticket_stats = self.db.get_ticket_statistics(None, None).await?;
        let received_tickets = ticket_stats.unredeemed_tickets + ticket_stats.losing_tickets;

        Ok(TicketStatistics {
            win_proportion: if received_tickets > 0 {
                ticket_stats.unredeemed_tickets as f64 / received_tickets as f64
            } else {
                0f64
            },
            losing: ticket_stats.losing_tickets,
            unredeemed: ticket_stats.unredeemed_tickets,
            unredeemed_value: ticket_stats.unredeemed_value,
            redeemed: ticket_stats.redeemed_tickets,
            redeemed_value: ticket_stats.redeemed_value,
            neglected: ticket_stats.neglected_tickets,
            neglected_value: ticket_stats.neglected_value,
            rejected: ticket_stats.rejected_tickets,
            rejected_value: ticket_stats.rejected_value,
        })
    }

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

    pub async fn all_tickets(&self) -> errors::Result<Vec<Ticket>> {
        Ok(self.db.get_all_tickets().await?.into_iter().map(|v| v.ticket).collect())
    }
}
