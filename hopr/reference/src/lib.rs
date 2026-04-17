#[cfg(feature = "session-server")]
pub mod exit;

#[cfg(feature = "session-server")]
pub mod config;

#[cfg(any(feature = "testing", test))]
pub mod testing;

use std::sync::Arc;

#[cfg(feature = "runtime-tokio")]
use hopr_chain_connector::{
    BlockchainConnectorConfig, HoprBlockchainSafeConnector,
    api::HoprChainApi,
    blokli_client::{BlokliClient, BlokliClientConfig},
    create_trustful_hopr_blokli_connector,
};
#[cfg(feature = "runtime-tokio")]
pub use hopr_lib;
#[cfg(feature = "runtime-tokio")]
use hopr_lib::builder::{ChainKeypair, Keypair, OffchainKeypair};
#[cfg(feature = "session-server")]
use hopr_lib::traits::HoprSessionServer;
#[cfg(feature = "runtime-tokio")]
use hopr_lib::{Hopr, config::HoprLibConfig};
#[cfg(feature = "runtime-tokio")]
use hopr_network_graph::{ChannelGraph, SharedChannelGraph};
use hopr_ticket_manager::{HoprTicketManager, RedbStore, RedbTicketQueue};
#[cfg(feature = "runtime-tokio")]
use hopr_transport_p2p::{HoprLibp2pNetworkBuilder, HoprNetwork};
#[cfg(feature = "runtime-tokio")]
use validator::Validate;

#[cfg(feature = "session-server")]
use crate::{config::SessionIpForwardingConfig, exit::HoprServerIpForwardingReactor};

/// Shareable [`HoprTicketManager`] with [`RedbStore`] backend.
pub type SharedTicketManager = Arc<HoprTicketManager<RedbStore, RedbTicketQueue>>;

/// The reference HOPR node type using canonical implementations.
#[cfg(feature = "runtime-tokio")]
pub type ReferenceHopr =
    Hopr<Arc<HoprBlockchainSafeConnector<BlokliClient>>, SharedChannelGraph, HoprNetwork, SharedTicketManager>;

/// Builds a reference HOPR node using canonical implementations:
/// - Blokli blockchain connector
/// - Petgraph-based channel graph
/// - libp2p-based P2P network
/// - Redb-backed ticket management
#[cfg(feature = "runtime-tokio")]
pub async fn build_reference(
    identity: (&ChainKeypair, &OffchainKeypair),
    config: HoprLibConfig,
    blokli_url: String,
    #[cfg(feature = "session-server")] server_config: SessionIpForwardingConfig,
) -> anyhow::Result<Arc<ReferenceHopr>> {
    let (chain_key, packet_key) = identity;

    let mut chain_connector = create_trustful_hopr_blokli_connector(
        chain_key,
        BlockchainConnectorConfig {
            connection_sync_timeout: std::time::Duration::from_mins(1),
            sync_tolerance: 90,
            tx_timeout_multiplier: std::env::var("HOPR_TX_TIMEOUT_MULTIPLIER")
                .ok()
                .and_then(|p| {
                    p.parse()
                        .inspect_err(|error| tracing::warn!(%error, "failed to parse HOPR_TX_TIMEOUT_MULTIPLIER"))
                        .ok()
                })
                .unwrap_or_else(|| BlockchainConnectorConfig::default().tx_timeout_multiplier),
        },
        BlokliClient::new(
            blokli_url.parse()?,
            BlokliClientConfig {
                timeout: std::time::Duration::from_secs(30),
                stream_reconnect_timeout: std::time::Duration::from_secs(30),
                subscription_stream_restart_delay: Some(std::time::Duration::from_secs(1)),
                ..Default::default()
            },
        ),
        config.safe_module.module_address,
    )
    .await?;
    chain_connector.connect().await?;
    let chain_connector = Arc::new(chain_connector);

    #[cfg(feature = "session-server")]
    let session_server = HoprServerIpForwardingReactor::new(packet_key.clone(), server_config);

    build_with_chain(
        chain_key,
        packet_key,
        config,
        None,
        chain_connector,
        #[cfg(feature = "session-server")]
        session_server,
    )
    .await
}

/// Builds a HOPR node with a custom chain connector using canonical implementations
/// for all other components.
#[cfg(feature = "runtime-tokio")]
pub async fn build_with_chain<
    Chain,
    #[cfg(feature = "session-server")] Srv: HoprSessionServer + Clone + Send + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    chain_connector: Chain,
    #[cfg(feature = "session-server")] server: Srv,
) -> anyhow::Result<Arc<Hopr<Chain, SharedChannelGraph, HoprNetwork, SharedTicketManager>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate()
            .map_err(|e| anyhow::anyhow!("invalid ProberConfig: {e}"))?;
        let probe_timeout = config.protocol.probe.timeout;
        anyhow::ensure!(
            pcfg.interval >= probe_timeout,
            "ProberConfig interval ({:?}) must be >= ProbeConfig timeout ({:?})",
            pcfg.interval,
            probe_timeout,
        );
    }

    // Create concrete components
    let graph: SharedChannelGraph = Arc::new(ChannelGraph::new(*packet_key.public()));

    // Wire chain announcement events → network peer discovery
    let (peer_discovery_tx, peer_discovery_rx) = futures::channel::mpsc::channel(2048);
    {
        use futures::{SinkExt, StreamExt};
        use hopr_lib::api::chain::StateSyncOptions;
        let chain_events = chain_connector
            .subscribe_with_state_sync([StateSyncOptions::PublicAccounts])
            .map_err(|e| anyhow::anyhow!("failed to subscribe to chain events: {e}"))?;
        let tx = peer_discovery_tx;
        tokio::spawn(async move {
            chain_events
                .for_each(|event| {
                    let mut tx = tx.clone();
                    async move {
                        if let hopr_lib::api::types::chain::chain_events::ChainEvent::Announcement(account) = event {
                            let peer_id: hopr_lib::api::PeerId = account.public_key.into();
                            if let Err(error) = tx
                                .send(hopr_transport_p2p::PeerDiscovery::Announce(
                                    peer_id,
                                    account.get_multiaddrs().to_vec(),
                                ))
                                .await
                            {
                                tracing::error!(%peer_id, %error, "failed to send peer discovery event");
                            }
                        }
                    }
                })
                .await;
        });
    }

    let network_builder = HoprLibp2pNetworkBuilder::new(peer_discovery_rx);

    // Wire network events (peer connected/disconnected) → graph updates
    {
        use futures::StreamExt;
        use hopr_lib::api::graph::NetworkGraphUpdate;
        let network_events = network_builder.subscribe_network_events();
        let graph_updater = graph.clone();
        tokio::spawn(async move {
            network_events
                .for_each(|event| {
                    let graph_updater = graph_updater.clone();
                    async move {
                        let (peer_id, connected) = match event {
                            hopr_lib::api::network::NetworkEvent::PeerConnected(p) => (p, true),
                            hopr_lib::api::network::NetworkEvent::PeerDisconnected(p) => (p, false),
                        };
                        if let Ok(opk) = hopr_lib::api::OffchainPublicKey::from_peerid(&peer_id) {
                            graph_updater.record_edge(hopr_lib::api::graph::MeasurableEdge::<
                                hopr_transport::NeighborTelemetry,
                                hopr_transport::PathTelemetry,
                            >::ConnectionStatus {
                                peer: opk,
                                connected,
                            });
                        }
                    }
                })
                .await;
        });
    }

    let prober_cfg = probe_cfg.unwrap_or_default();
    let cover_traffic =
        hopr_ct_full_network::FullNetworkDiscovery::new(*packet_key.public(), prober_cfg, graph.clone());

    let backend = RedbStore::new_temp().map_err(hopr_ticket_manager::TicketManagerError::store)?;
    let (ticket_manager, ticket_factory) = HoprTicketManager::new_with_factory(backend);
    let ticket_manager = Arc::new(ticket_manager);
    let ticket_factory = Arc::new(ticket_factory);

    // Use the abstract builder
    let mut builder = hopr_lib::builder::HoprBuilder::default()
        .with_chain_api(chain_connector)
        .with_graph(graph)
        .with_network_builder(network_builder)
        .with_cover_traffic(cover_traffic)
        .with_ticket_factory(ticket_factory)
        .with_ticket_management(ticket_manager)
        .with_identity(chain_key, packet_key)
        .with_safe_module(&config.safe_module.safe_address, &config.safe_module.module_address)
        .with_config(config);

    #[cfg(feature = "session-server")]
    {
        builder = builder.with_session_server(server);
    }

    #[cfg(not(feature = "session-server"))]
    {
        builder = builder.with_session_server(());
    }

    let node = builder.build().await?;

    Ok(Arc::new(node))
}
