//! Node assembly helpers: canonical type aliases and build functions used by cluster fixtures.

use std::sync::Arc;

use futures::StreamExt;
use hopr_chain_connector::api::HoprChainApi;
use hopr_network_graph::{ChannelGraph, SharedChannelGraph};
use hopr_ticket_manager::{
    HoprTicketManager, RedbStore, RedbTicketQueue, ticket_factory_from_chain, ticket_manager_from_chain,
};
use hopr_transport_p2p::{HoprLibp2pNetworkBuilder, HoprNetwork, PeerDiscovery};

use crate::{
    Hopr,
    builder::{ChainKeypair, HoprBuilder, Keypair, OffchainKeypair},
    config::HoprLibConfig,
    errors::HoprLibError,
};

pub type SharedTicketManager = Arc<HoprTicketManager<RedbStore, RedbTicketQueue>>;

pub type FullHopr<
    Chain = Arc<hopr_chain_connector::HoprBlockchainSafeConnector<hopr_chain_connector::blokli_client::BlokliClient>>,
    Graph = SharedChannelGraph,
    Net = HoprNetwork,
    TMgr = SharedTicketManager,
> = Hopr<Chain, Graph, Net, TMgr>;

pub type EdgeHopr<
    Chain = Arc<hopr_chain_connector::HoprBlockchainSafeConnector<hopr_chain_connector::blokli_client::BlokliClient>>,
    Graph = SharedChannelGraph,
    Net = HoprNetwork,
> = Hopr<Chain, Graph, Net, ()>;

type FullyConfiguredBuilder<Chain> = crate::builder::HoprBuilderConfigured<
    Chain,
    SharedChannelGraph,
    HoprNetwork,
    hopr_ct_full_network::FullNetworkDiscovery<SharedChannelGraph>,
>;

fn make_builder<Chain>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    prober_cfg: hopr_ct_full_network::ProberConfig,
    chain_connector: Chain,
) -> FullyConfiguredBuilder<Chain>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    let path_cfg = config.protocol.path_planner;
    let graph: SharedChannelGraph = Arc::new(ChannelGraph::with_edge_params(
        *packet_key.public(),
        path_cfg.edge_penalty,
        path_cfg.min_ack_rate,
        path_cfg.max_plausible_loopback_rtt,
    ));
    let graph_for_ct = graph.clone();
    let safe_address = config.safe_module.safe_address;
    let module_address = config.safe_module.module_address;

    HoprBuilder
        .with_identity(chain_key, packet_key)
        .with_config(config)
        .with_safe_module(&safe_address, &module_address)
        .with_chain_api(move |_ctx| chain_connector)
        .with_graph(move |_ctx| graph)
        .with_network(move |ctx| {
            Box::pin(async move {
                let peer_discovery_rx = ctx
                    .take_peer_discovery_rx()
                    .ok_or(HoprLibError::BuilderError("peer_discovery_rx already taken"))?;
                let multiaddresses = vec![(&ctx.cfg.host).try_into().map_err(HoprLibError::TransportError)?];
                let nb = HoprLibp2pNetworkBuilder::new(
                    peer_discovery_rx.map(|(peer_id, addrs)| PeerDiscovery::Announce(peer_id, addrs)),
                );
                nb.build(
                    &ctx.packet_key,
                    multiaddresses,
                    "/hopr/mix/1.1.0",
                    ctx.cfg.protocol.transport.prefer_local_addresses,
                )
                .await
                .map_err(|e| HoprLibError::GeneralError(e.to_string()))
            })
        })
        .with_cover_traffic(move |ctx| {
            hopr_ct_full_network::FullNetworkDiscovery::new(*ctx.packet_key.public(), prober_cfg, graph_for_ct)
        })
}

/// Builds an edge (entry/exit) HOPR node with a custom chain connector.
pub async fn build_edge_with_chain<
    Chain,
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<EdgeHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let ticket_factory = ticket_factory_from_chain(&chain_connector)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket factory: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    );

    let node = builder.with_session_server(server).build_edge(ticket_factory).await?;

    Ok(Arc::new(node))
}

/// Builds a full relay HOPR node with a custom chain connector.
pub async fn build_full_with_chain<
    Chain,
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<FullHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let (ticket_manager, ticket_factory) = ticket_manager_from_chain(&chain_connector)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket manager: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    );

    let node = builder
        .with_session_server(server)
        .build_full(ticket_manager, ticket_factory)
        .await?;

    Ok(Arc::new(node))
}

/// Builds an entry HOPR node with a custom chain connector.
///
/// Entry nodes do not process tickets, do not have ticket manager state,
/// and do not accept incoming sessions.
pub async fn build_entry_with_chain<
    Chain,
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<EdgeHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let ticket_factory = ticket_factory_from_chain(&chain_connector)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket factory: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    );

    let node = builder.with_session_server(server).build_entry(ticket_factory).await?;

    Ok(Arc::new(node))
}

/// Builds an exit HOPR node with a custom chain connector.
///
/// Exit nodes accept incoming sessions and process PIX acknowledgements,
/// but do not process tickets (no ticket manager state).
pub async fn build_exit_with_chain<
    Chain,
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<EdgeHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let ticket_factory = ticket_factory_from_chain(&chain_connector)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket factory: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    );

    let node = builder.with_session_server(server).build_exit(ticket_factory).await?;

    Ok(Arc::new(node))
}
