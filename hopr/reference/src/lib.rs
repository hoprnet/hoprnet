#[cfg(feature = "session-server")]
pub mod exit;

#[cfg(feature = "session-server")]
pub mod config;

#[cfg(any(feature = "testing", test))]
pub mod testing;

use std::sync::Arc;

use futures::{FutureExt, StreamExt as _};
use futures_concurrency::stream::StreamExt;
use hopr_chain_connector::{
    BlockchainConnectorConfig, HoprBlockchainSafeConnector,
    api::{AccountSelector, ChainEvent, HoprChainApi, StateSyncOptions},
    blokli_client::{BlokliClient, BlokliClientConfig},
    create_trustful_hopr_blokli_connector,
};
use hopr_db_node::{HoprNodeDb, HoprNodeDbApi, init_hopr_node_db};
#[cfg(feature = "runtime-tokio")]
pub use hopr_lib;
use hopr_lib::Keypair;
#[cfg(feature = "session-server")]
use hopr_lib::traits::HoprSessionServer;
#[cfg(feature = "runtime-tokio")]
use hopr_lib::{
    ChainKeypair, Hopr, HoprLibError, HoprTransportIO, OffchainKeypair, api::network::NetworkEvent,
    config::HoprLibConfig,
};
use hopr_network_graph::{ChannelGraph, SharedChannelGraph};
use hopr_transport_p2p::{HoprLibp2pNetworkBuilder, HoprNetwork, PeerDiscovery};
#[cfg(feature = "runtime-tokio")]
use {
    futures::SinkExt,
    hopr_lib::{
        api::{PeerId, graph::EdgeCapacityUpdate, graph::NetworkGraphUpdate},
        {ChannelStatus, NeighborTelemetry, PathTelemetry},
    },
};

#[cfg(feature = "session-server")]
use crate::{config::SessionIpForwardingConfig, exit::HoprServerIpForwardingReactor};

pub type ReferenceHopr =
    Hopr<Arc<HoprBlockchainSafeConnector<BlokliClient>>, HoprNodeDb, SharedChannelGraph, HoprNetwork>;

#[cfg(feature = "runtime-tokio")]
pub async fn build_reference(
    identity: (&ChainKeypair, &OffchainKeypair),
    config: HoprLibConfig,
    blokli_url: String,
    db_data_path: String,
    #[cfg(feature = "session-server")] server_config: SessionIpForwardingConfig,
) -> anyhow::Result<(
    Arc<ReferenceHopr>,
    impl Future<Output = std::result::Result<HoprTransportIO, HoprLibError>>,
)> {
    let (chain_key, packet_key) = identity;
    let node_db = init_hopr_node_db(&db_data_path, true, false).await?;

    let mut chain_connector = create_trustful_hopr_blokli_connector(
        chain_key,
        BlockchainConnectorConfig {
            connection_sync_timeout: std::time::Duration::from_mins(1),
            sync_tolerance: 90,
        },
        BlokliClient::new(
            blokli_url.parse()?,
            BlokliClientConfig {
                timeout: std::time::Duration::from_secs(30),
                stream_reconnect_timeout: std::time::Duration::from_secs(30),
            },
        ),
        config.safe_module.module_address,
    )
    .await?;
    chain_connector.connect().await?;
    let chain_connector = Arc::new(chain_connector);

    #[cfg(feature = "session-server")]
    let session_server = HoprServerIpForwardingReactor::new(packet_key.clone(), server_config);

    build_from_chain_and_db(
        chain_key,
        packet_key,
        config,
        chain_connector,
        node_db,
        #[cfg(feature = "session-server")]
        session_server,
    )
    .await
}

#[cfg(feature = "runtime-tokio")]
pub async fn build_from_chain_and_db<
    Chain,
    Db,
    #[cfg(feature = "session-server")] Srv: HoprSessionServer + Clone + Send + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    chain_connector: Chain,
    db: Db,
    #[cfg(feature = "session-server")] server: Srv,
) -> anyhow::Result<(
    Arc<Hopr<Chain, Db, SharedChannelGraph, HoprNetwork>>,
    impl Future<Output = std::result::Result<HoprTransportIO, HoprLibError>>,
)>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Db: HoprNodeDbApi + Clone + Send + Sync + 'static,
{
    // Calculate the minimum capacity based on accounts (each account can generate 2 messages),
    // plus 100 as an additional buffer
    let minimum_capacity = chain_connector
        .count_accounts(AccountSelector {
            public_only: true,
            ..Default::default()
        })
        .await
        .map_err(hopr_lib::HoprLibError::chain)?
        .saturating_mul(2)
        .saturating_add(100);

    let chain_discovery_events_capacity = std::env::var("HOPR_INTERNAL_CHAIN_DISCOVERY_CHANNEL_CAPACITY")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&c| c > 0)
        .unwrap_or(2048)
        .max(minimum_capacity);

    tracing::debug!(
        capacity = chain_discovery_events_capacity,
        minimum_required = minimum_capacity,
        "creating chain discovery events channel"
    );
    let (indexer_peer_update_tx, indexer_peer_update_rx) =
        futures::channel::mpsc::channel::<PeerDiscovery>(chain_discovery_events_capacity);

    // create network
    let network_builder = HoprLibp2pNetworkBuilder::new(indexer_peer_update_rx);
    // create graph
    let graph = std::sync::Arc::new(ChannelGraph::new(*packet_key.public()));

    // END = implementation definitions

    // START = process chain and network events into graph updates
    let chain_events = chain_connector
        .subscribe_with_state_sync([StateSyncOptions::PublicAccounts, StateSyncOptions::OpenedChannels])?;
    let network_events = network_builder.subscribe_network_events();
    let graph_updater = graph.clone();
    let chain_reader = chain_connector.clone();
    let indexer_peer_update_tx = indexer_peer_update_tx.clone();

    let proc =
        async move {
            enum Event {
                Chain(ChainEvent),
                Network(NetworkEvent),
            }

            network_events
                .map(Event::Network)
                .merge(chain_events.map(Event::Chain))
                .for_each(|event| {
                    let mut indexer_peer_update_tx = indexer_peer_update_tx.clone();
                    let chain_reader = chain_reader.clone();
                    let graph_updater = graph_updater.clone();
                    async move {

                        let ticket_price = std::sync::Arc::new(parking_lot::RwLock::new(chain_reader.minimum_ticket_price().await.unwrap_or_default()));
                        let win_probability = std::sync::Arc::new(parking_lot::RwLock::new(chain_reader.minimum_incoming_ticket_win_prob().await.unwrap_or_default()));

                        match event {
                            Event::Chain(chain_event) => {
                                match chain_event {
                                    ChainEvent::Announcement(account) =>{
                                        graph_updater.record_node(account.public_key);
                                        let peer_id: PeerId = account.public_key.into();
                                        if let Err(error) = indexer_peer_update_tx.send(PeerDiscovery::Announce(peer_id, account.get_multiaddrs().to_vec())).await {
                                            tracing::error!(peer = %peer_id, %error, reason = "failed to propagate the record", "ignoring announced peer")
                                        }
                                },
                                ChainEvent::ChannelOpened(channel) |
                                ChainEvent::ChannelClosed(channel) |
                                ChainEvent::ChannelBalanceIncreased(channel, _) |
                                ChainEvent::ChannelBalanceDecreased(channel, _) => {
                                    let from = chain_reader.chain_key_to_packet_key(&channel.source).await;
                                    let to = chain_reader.chain_key_to_packet_key(&channel.destination).await;

                                    match (from, to) {
                                        (Ok(Some(from)), Ok(Some(to))) => {
                                            let capacity =  if matches!(channel.status, ChannelStatus::Closed) {
                                                None
                                            } else {
                                                Some(channel.balance.amount().low_u128().saturating_div(ticket_price.read().amount().low_u128()).saturating_mul(win_probability.read().as_luck() as u128))
                                            };

                                            graph_updater.record_edge(hopr_lib::api::graph::MeasurableEdge::<NeighborTelemetry, PathTelemetry>::Capacity(Box::new(EdgeCapacityUpdate{
                                                capacity,
                                                src: from,
                                                dest: to
                                        })));
                                        },
                                        (Ok(_), Ok(_)) => {
                                            tracing::error!(%channel, "could not find packet keys for the channel endpoints");
                                        },
                                        (Err(e), _) | (_, Err(e)) => {
                                            tracing::error!(%e, %channel, "failed to convert chain keys to packet keys for graph update");
                                        }
                                    }
                                },
                                ChainEvent::ChannelClosureInitiated(_channel) => {},
                                ChainEvent::WinningProbabilityIncreased(probability) |
                                ChainEvent::WinningProbabilityDecreased(probability) => {
                                    *win_probability.write() = probability;
                                }
                                ChainEvent::TicketPriceChanged(price) => {
                                    *ticket_price.write() = price;
                                },
                                _ => {}
                            }
                        }
                        Event::Network(network_event) => {
                            match network_event {
                                NetworkEvent::PeerConnected(peer_id) =>
                                    if let Ok(opk) = hopr_lib::peer_id_to_public_key(&peer_id).await {
                                        graph_updater.record_node(opk);
                                    } else {
                                        tracing::error!(%peer_id, "failed to convert peer ID to public key for graph update");
                                    },
                                NetworkEvent::PeerDisconnected(peer_id) =>
                                    if let Ok(opk) = hopr_lib::peer_id_to_public_key(&peer_id).await {
                                        graph_updater.record_node(opk);
                                    } else {
                                        tracing::error!(%peer_id, "failed to convert peer ID to public key for graph update");
                                    },
                            };
                        }
                    }
                    }
                })
                .await;
        }
        .inspect(|_| tracing::warn!(task = "Interconnecting chain, graph and network", "long-running background task finished"));
    let _jh = tokio::spawn(proc);
    // END = process chain and network events into graph updates

    // create the node
    let node = Arc::new(
        hopr_lib::Hopr::new(
            (chain_key, packet_key),
            chain_connector.clone(),
            db,
            graph.clone(),
            config,
        )
        .await?,
    );

    let node_for_run = node.clone();
    let start = async move {
        node_for_run
            .run(
                hopr_ct_telemetry::ImmediateNeighborProber::new(Default::default(), graph),
                network_builder,
                #[cfg(feature = "session-server")]
                server,
            )
            .await
    };

    Ok((node, start))
}
