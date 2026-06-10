//! Node assembly helpers: canonical type aliases and build functions used by cluster fixtures.

use std::sync::Arc;

use futures::StreamExt;
use hopr_chain_connector::{
    Address, BlockchainConnectorConfig, HoprBlockchainSafeConnector, api::HoprChainApi,
    blokli_client::{BlokliClient, BlokliClientConfig, BlokliDnsOverride},
    create_trustful_hopr_blokli_connector,
};
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

/// Blokli client options for building edge/full nodes from a URL.
///
/// Keep using the hostname-based Blokli URL when setting
/// [`BlokliClientOptions::dns_override`]. The override pins the hostname to a
/// fixed IP address while preserving the original host for HTTP `Host` headers and TLS SNI.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlokliClientOptions {
    /// Optional DNS override for the configured Blokli hostname.
    pub dns_override: Option<BlokliDnsOverride>,
    /// Optional transaction timeout multiplier passed into the chain connector.
    pub tx_timeout_multiplier: Option<u32>,
}

async fn create_blokli_connector(
    chain_key: &ChainKeypair,
    blokli_url: String,
    module_address: Address,
    blokli_options: BlokliClientOptions,
) -> anyhow::Result<Arc<HoprBlockchainSafeConnector<BlokliClient>>> {
    let mut connector = create_trustful_hopr_blokli_connector(
        chain_key,
        BlockchainConnectorConfig {
            connection_sync_timeout: std::time::Duration::from_mins(1),
            sync_tolerance: 90,
            tx_timeout_multiplier: blokli_options
                .tx_timeout_multiplier
                .unwrap_or_else(|| BlockchainConnectorConfig::default().tx_timeout_multiplier),
        },
        BlokliClient::new(
            blokli_url.parse::<hopr_chain_connector::blokli_client::Url>().map_err(anyhow::Error::from)?,
            BlokliClientConfig {
                timeout: std::time::Duration::from_secs(30),
                stream_reconnect_timeout: std::time::Duration::from_secs(30),
                subscription_stream_restart_delay: Some(std::time::Duration::from_secs(1)),
                dns_override: blokli_options.dns_override,
                ..Default::default()
            },
        ),
        module_address,
    )
    .await
    .map_err(anyhow::Error::from)?;

    connector.connect().await.map_err(anyhow::Error::from)?;

    Ok(Arc::new(connector))
}

/// Builds an edge (entry/exit) HOPR node using the default Blokli chain connector
/// configured through [`BlokliClientOptions`].
pub async fn build_edge_with_blokli_options<
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + 'static,
>(
    identity: (&ChainKeypair, &OffchainKeypair),
    config: HoprLibConfig,
    blokli_url: String,
    blokli_options: BlokliClientOptions,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    server: Srv,
) -> anyhow::Result<Arc<EdgeHopr>> {
    let (chain_key, packet_key) = identity;
    let module_address = config.safe_module.module_address;
    let chain_connector = create_blokli_connector(chain_key, blokli_url, module_address, blokli_options).await?;

    build_edge_with_chain(chain_key, packet_key, config, probe_cfg, chain_connector, server).await
}

/// Builds a full relay HOPR node using the default Blokli chain connector
/// configured through [`BlokliClientOptions`].
pub async fn build_full_with_blokli_options<
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + 'static,
>(
    identity: (&ChainKeypair, &OffchainKeypair),
    config: HoprLibConfig,
    blokli_url: String,
    blokli_options: BlokliClientOptions,
    probe_cfg: Option<hopr_ct_full_network::ProberConfig>,
    server: Srv,
) -> anyhow::Result<Arc<FullHopr>> {
    let (chain_key, packet_key) = identity;
    let module_address = config.safe_module.module_address;
    let chain_connector = create_blokli_connector(chain_key, blokli_url, module_address, blokli_options).await?;

    build_full_with_chain(chain_key, packet_key, config, probe_cfg, chain_connector, server).await
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::{BlokliClientOptions, BlokliDnsOverride};

    #[test]
    fn blokli_client_options_default_to_no_dns_override() {
        assert_eq!(
            BlokliClientOptions::default(),
            BlokliClientOptions {
                dns_override: None,
                tx_timeout_multiplier: None,
            }
        );
    }

    #[test]
    fn blokli_client_options_store_dns_override() {
        let options = BlokliClientOptions {
            dns_override: Some(BlokliDnsOverride {
                ip: IpAddr::from([203, 0, 113, 10]),
                port: Some(8443),
            }),
            tx_timeout_multiplier: Some(3),
        };

        assert_eq!(options.tx_timeout_multiplier, Some(3));
        assert_eq!(
            options.dns_override,
            Some(BlokliDnsOverride {
                ip: IpAddr::from([203, 0, 113, 10]),
                port: Some(8443),
            })
        );
    }
}
