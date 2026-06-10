#[cfg(feature = "session-server")]
pub mod exit;

#[cfg(feature = "session-server")]
pub mod config;

#[cfg(any(feature = "testing", test))]
pub mod testing;

use std::sync::Arc;

use hopr_ticket_manager::{
    HoprTicketManager, RedbStore, RedbTicketQueue,
    ticket_factory_from_chain, ticket_manager_from_chain,
};
#[cfg(feature = "runtime-tokio")]
use {
    futures::StreamExt,
    hopr_chain_connector::api::HoprChainApi,
    hopr_lib::builder::{ChainKeypair, Keypair, OffchainKeypair},
    hopr_lib::{Hopr, config::HoprLibConfig},
    hopr_network_graph::{ChannelGraph, SharedChannelGraph},
    hopr_transport_p2p::{HoprLibp2pNetworkBuilder, HoprNetwork, PeerDiscovery},
    validator::Validate,
};

/// Shareable [`HoprTicketManager`] with [`RedbStore`] backend.
pub type SharedTicketManager = Arc<HoprTicketManager<RedbStore, RedbTicketQueue>>;

/// The canonical full relay HOPR node type using the Blokli chain connector.
#[cfg(feature = "runtime-tokio")]
pub type FullHopr<
    Chain = Arc<hopr_chain_connector::HoprBlockchainSafeConnector<hopr_chain_connector::blokli_client::BlokliClient>>,
    Graph = SharedChannelGraph,
    Net = HoprNetwork,
    TMgr = SharedTicketManager,
> = Hopr<Chain, Graph, Net, TMgr>;

/// The canonical edge (entry/exit) HOPR node type using the Blokli chain connector.
///
/// Unlike [`FullHopr`], this type carries no ticket manager (`TMgr = ()`); edge nodes
/// originate outgoing tickets but never relay incoming ones.
#[cfg(feature = "runtime-tokio")]
pub type EdgeHopr<
    Chain = Arc<hopr_chain_connector::HoprBlockchainSafeConnector<hopr_chain_connector::blokli_client::BlokliClient>>,
    Graph = SharedChannelGraph,
    Net = HoprNetwork,
> = Hopr<Chain, Graph, Net, ()>;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "runtime-tokio")]
fn validate_probe_cfg(
    probe_cfg: &hopr_ct_full_network::ProberConfig,
    probe_timeout: std::time::Duration,
) -> anyhow::Result<()> {
    probe_cfg
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid ProberConfig: {e}"))?;
    anyhow::ensure!(
        probe_cfg.interval >= probe_timeout,
        "ProberConfig interval ({:?}) must be >= ProbeConfig timeout ({:?})",
        probe_cfg.interval,
        probe_timeout,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Level-2 flexible builders (bring your own chain connector)
// ---------------------------------------------------------------------------

/// Builds an edge (entry/exit) HOPR node with a custom chain connector.
///
/// Edge nodes do not relay incoming packets and carry no ticket manager
/// (`TMgr = ()`), which is reflected in the [`EdgeHopr`] return type.
///
/// When the `session-server` feature is enabled a session server must be provided.
/// Callers that do not need to handle incoming sessions can pass a server that discards them.
#[cfg(feature = "runtime-tokio")]
pub async fn build_edge_with_chain<
    Chain,
    #[cfg(feature = "session-server")] Srv: hopr_lib::api::node::HoprSessionServer<
            Session = hopr_lib::exports::transport::IncomingSession,
            Error: std::fmt::Display,
        > + Clone
        + Send
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    chain_connector: Chain,
    #[cfg(feature = "session-server")] server: Srv,
) -> anyhow::Result<Arc<EdgeHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        validate_probe_cfg(pcfg, config.protocol.probe.timeout)?;
    }

    let ticket_factory = ticket_factory_from_chain(&chain_connector)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket factory: {e}"))?;

    let (peer_discovery_tx, peer_discovery_rx) = futures::channel::mpsc::channel(2048);
    let prober_cfg = probe_cfg.unwrap_or_default();
    let path_cfg = config.protocol.path_planner;
    let graph: SharedChannelGraph = Arc::new(ChannelGraph::with_edge_params(
        *packet_key.public(),
        path_cfg.edge_penalty,
        path_cfg.min_ack_rate,
    ));
    let graph_for_ct = graph.clone();
    let safe_address = config.safe_module.safe_address;
    let module_address = config.safe_module.module_address;

    let builder = hopr_lib::builder::HoprBuilder
        .with_identity(chain_key, packet_key)
        .with_config(config)
        .with_safe_module(&safe_address, &module_address)
        .with_chain_api(move |_ctx| chain_connector)
        .with_peer_discovery_tx(peer_discovery_tx)
        .with_graph(move |_ctx| graph)
        .with_network(move |ctx| {
            Box::pin(async move {
                let multiaddresses = vec![
                    (&ctx.cfg.host)
                        .try_into()
                        .expect("host config must be a valid multiaddress"),
                ];
                let nb = HoprLibp2pNetworkBuilder::new(
                    peer_discovery_rx
                        .map(|(peer_id, addrs)| PeerDiscovery::Announce(peer_id, addrs)),
                );
                nb.build(
                    &ctx.packet_key,
                    multiaddresses,
                    "/hopr/mix/1.1.0",
                    ctx.cfg.protocol.transport.prefer_local_addresses,
                )
                .await
                .expect("network must be constructible")
            })
        })
        .with_cover_traffic(move |ctx| {
            hopr_ct_full_network::FullNetworkDiscovery::new(*ctx.packet_key.public(), prober_cfg, graph_for_ct)
        });

    #[cfg(feature = "session-server")]
    let node = builder.with_session_server(server).build_edge(ticket_factory).await?;
    #[cfg(not(feature = "session-server"))]
    let node = builder.build_edge(ticket_factory).await?;

    Ok(Arc::new(node))
}

/// Builds a full relay HOPR node with a custom chain connector and session server.
///
/// Use this when you need a non-default chain connector.
#[cfg(feature = "runtime-tokio")]
pub async fn build_full_with_chain<
    Chain,
    #[cfg(feature = "session-server")] Srv: hopr_lib::api::node::HoprSessionServer<
            Session = hopr_lib::exports::transport::IncomingSession,
            Error: std::fmt::Display,
        > + Clone
        + Send
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    chain_connector: Chain,
    #[cfg(feature = "session-server")] server: Srv,
) -> anyhow::Result<Arc<FullHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        validate_probe_cfg(pcfg, config.protocol.probe.timeout)?;
    }

    let (ticket_manager, ticket_factory) = ticket_manager_from_chain(&chain_connector)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket manager: {e}"))?;

    let (peer_discovery_tx, peer_discovery_rx) = futures::channel::mpsc::channel(2048);
    let prober_cfg = probe_cfg.unwrap_or_default();
    let path_cfg = config.protocol.path_planner;
    let graph: SharedChannelGraph = Arc::new(ChannelGraph::with_edge_params(
        *packet_key.public(),
        path_cfg.edge_penalty,
        path_cfg.min_ack_rate,
    ));
    let graph_for_ct = graph.clone();
    let safe_address = config.safe_module.safe_address;
    let module_address = config.safe_module.module_address;

    let builder = hopr_lib::builder::HoprBuilder
        .with_identity(chain_key, packet_key)
        .with_config(config)
        .with_safe_module(&safe_address, &module_address)
        .with_chain_api(move |_ctx| chain_connector)
        .with_peer_discovery_tx(peer_discovery_tx)
        .with_graph(move |_ctx| graph)
        .with_network(move |ctx| {
            Box::pin(async move {
                let multiaddresses = vec![
                    (&ctx.cfg.host)
                        .try_into()
                        .expect("host config must be a valid multiaddress"),
                ];
                let nb = HoprLibp2pNetworkBuilder::new(
                    peer_discovery_rx
                        .map(|(peer_id, addrs)| PeerDiscovery::Announce(peer_id, addrs)),
                );
                nb.build(
                    &ctx.packet_key,
                    multiaddresses,
                    "/hopr/mix/1.1.0",
                    ctx.cfg.protocol.transport.prefer_local_addresses,
                )
                .await
                .expect("network must be constructible")
            })
        })
        .with_cover_traffic(move |ctx| {
            hopr_ct_full_network::FullNetworkDiscovery::new(*ctx.packet_key.public(), prober_cfg, graph_for_ct)
        });

    #[cfg(feature = "session-server")]
    let node = builder
        .with_session_server(server)
        .build_full(ticket_manager, ticket_factory)
        .await?;
    #[cfg(not(feature = "session-server"))]
    let node = builder.build_full(ticket_manager, ticket_factory).await?;

    Ok(Arc::new(node))
}
