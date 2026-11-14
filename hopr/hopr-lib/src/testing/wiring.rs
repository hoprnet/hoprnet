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

/// Assemble the emulated node factories without installing application-owned reporting policy.
fn make_builder<Chain>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    prober_cfg: hopr_ct_full_network::EgressConfig,
    chain_connector: Chain,
) -> anyhow::Result<FullyConfiguredBuilder<Chain>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    let path_cfg = config.protocol.path_planner;
    let graph: SharedChannelGraph = Arc::new(
        ChannelGraph::with_edge_params(
            *packet_key.public(),
            path_cfg.edge_penalty,
            path_cfg.min_ack_rate,
            path_cfg.max_plausible_loopback_rtt,
        )
        .with_starvation(hopr_network_graph::StarvationParams {
            min_loopbacks_for_rate: path_cfg.min_loopbacks_for_rate,
            min_return_rate: path_cfg.min_loopback_return_rate,
        })
        .map_err(|e| anyhow::anyhow!("invalid loopback starvation config: {e}"))?,
    );
    let graph_for_ct = graph.clone();
    let safe_address = config.safe_module.safe_address;
    let module_address = config.safe_module.module_address;

    let builder = HoprBuilder
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
                    hopr_transport::protocol::CURRENT_HOPR_MSG_PROTOCOL,
                    ctx.cfg.protocol.transport.prefer_local_addresses,
                )
                .await
                .map_err(|e| HoprLibError::GeneralError(e.to_string()))
            })
        })
        .with_cover_traffic(move |ctx| {
            hopr_ct_full_network::FullNetworkDiscovery::new(*ctx.packet_key.public(), prober_cfg, graph_for_ct)
        });
    Ok(builder)
}

/// Builds an edge (entry/exit) HOPR node with a custom chain connector.
pub async fn build_edge_with_chain<
    Chain,
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + Sync
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::EgressConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<EdgeHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let ticket_factory = ticket_factory_from_chain(&chain_connector, RedbStore::new_temp()?)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket factory: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    )?;

    let node = builder.with_session_server(server).build_edge(ticket_factory).await?;

    Ok(Arc::new(node))
}

/// Builds a full relay HOPR node with a custom chain connector.
pub async fn build_full_with_chain<
    Chain,
    Srv: hopr_api::node::HoprSessionServer<Session = hopr_transport::IncomingSession, Error: std::fmt::Display>
        + Clone
        + Send
        + Sync
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::EgressConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<FullHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let (ticket_manager, ticket_factory) = ticket_manager_from_chain(&chain_connector, RedbStore::new_temp()?)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket manager: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    )?;

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
        + Sync
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::EgressConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<EdgeHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let ticket_factory = ticket_factory_from_chain(&chain_connector, RedbStore::new_temp()?)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket factory: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    )?;

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
        + Sync
        + 'static,
>(
    chain_key: &ChainKeypair,
    packet_key: &OffchainKeypair,
    config: HoprLibConfig,
    probe_cfg: Option<hopr_ct_full_network::EgressConfig>,
    chain_connector: Chain,
    server: Srv,
) -> anyhow::Result<Arc<EdgeHopr<Chain>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate_against_probe_timeout(config.protocol.probe.timeout)?;
    }

    let ticket_factory = ticket_factory_from_chain(&chain_connector, RedbStore::new_temp()?)
        .await
        .map_err(|e| anyhow::anyhow!("failed to seed ticket factory: {e}"))?;

    let builder = make_builder(
        chain_key,
        packet_key,
        config,
        probe_cfg.unwrap_or_default(),
        chain_connector,
    )?;

    let node = builder.with_session_server(server).build_exit(ticket_factory).await?;

    Ok(Arc::new(node))
}

#[cfg(all(test, feature = "state-dump"))]
pub(crate) mod tests {
    use anyhow::Context;
    use hopr_api::types::primitive::prelude::Address;
    use hopr_chain_connector::{
        BlockchainConnectorConfig, create_trustful_hopr_blokli_connector, testing::FullStateEmulator,
    };
    use tokio::sync::oneshot;

    use super::*;
    use crate::{
        state_dump::{StateDump, StateDumpFactory, StateDumpSections},
        testing::fixtures::{NODE_CHAIN_KEYS, NODE_OFFCHAIN_KEYS, NODE_SAFES_MODULES, build_blokli_client},
    };

    /// Observe the host's task ownership without implementing a diagnostic collector.
    pub(crate) struct ReporterProbe {
        valid: bool,
        enabled: bool,
        observer: Option<hopr_transport::TransportObserver>,
        started: oneshot::Sender<Vec<(&'static str, serde_json::Value)>>,
        lifetime: oneshot::Sender<()>,
    }

    type ReportedSections = Vec<(&'static str, serde_json::Value)>;

    impl ReporterProbe {
        pub(crate) fn new(
            valid: bool,
            enabled: bool,
            observer: Option<hopr_transport::TransportObserver>,
        ) -> (Self, oneshot::Receiver<ReportedSections>, oneshot::Receiver<()>) {
            let (started, started_rx) = oneshot::channel();
            let (lifetime, lifetime_rx) = oneshot::channel();
            (
                Self {
                    valid,
                    enabled,
                    observer,
                    started,
                    lifetime,
                },
                started_rx,
                lifetime_rx,
            )
        }
    }

    // Specialize for the final component types to check that registering a reporter before
    // those component factories does not constrain the builder's intermediate types.
    impl StateDumpFactory<crate::testing::TestingConnector, SharedChannelGraph, HoprNetwork> for ReporterProbe {
        fn prepare(&mut self) -> Result<(), HoprLibError> {
            if self.valid {
                Ok(())
            } else {
                Err(HoprLibError::GeneralError("invalid reporter configuration".into()))
            }
        }

        fn create(
            self,
            _: crate::testing::TestingConnector,
            _: SharedChannelGraph,
            _: HoprNetwork,
            _: Address,
            sections: StateDumpSections,
        ) -> Option<StateDump> {
            self.enabled.then(|| {
                StateDump::new(self.observer, async move {
                    // Dropping this sender signals cancellation, including when the task is aborted.
                    let _lifetime = self.lifetime;
                    let _ = self
                        .started
                        .send(sections.into_iter().map(|(name, read)| (name, read())).collect());
                    futures::future::pending::<()>().await;
                })
            })
        }
    }

    #[rstest::rstest]
    #[case::with_observer(true, true)]
    #[case::without_observer(true, false)]
    #[case::disabled(false, true)]
    #[tokio::test]
    async fn node_owns_the_optional_reporter_lifetime(
        #[case] enabled: bool,
        #[case] with_observer: bool,
    ) -> anyhow::Result<()> {
        let chain_key = &NODE_CHAIN_KEYS[0];
        let packet_key = &NODE_OFFCHAIN_KEYS[0];
        let (safe, module) = NODE_SAFES_MODULES[0];
        let client = build_blokli_client().with_mutator(FullStateEmulator::new(module));
        let mut connector =
            create_trustful_hopr_blokli_connector(chain_key, BlockchainConnectorConfig::default(), client, module)
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        let config = HoprLibConfig {
            safe_module: crate::config::SafeModule {
                safe_address: safe,
                module_address: module,
            },
            ..Default::default()
        };
        let ticket_factory = ticket_factory_from_chain(&connector, RedbStore::new_temp()?)
            .await
            .map_err(|error| anyhow::anyhow!("create ticket factory: {error}"))?;
        let observer: hopr_transport::TransportObserver = Arc::new(|_| {});
        let observer_ref = Arc::downgrade(&observer);
        let (reporter, started_rx, mut lifetime_rx) =
            ReporterProbe::new(true, enabled, with_observer.then_some(observer));
        let graph = Arc::new(ChannelGraph::new(*packet_key.public()));
        let graph_for_ct = graph.clone();
        let node = HoprBuilder
            .with_identity(chain_key, packet_key)
            .with_config(config)
            .with_state_dump(reporter)
            .with_state_dump_section("application", || serde_json::json!({ "ready": true }))
            .with_chain_api(move |_| connector)
            .with_graph(move |_| graph)
            // Reporting needs a real network handle, but no listening sockets or peers.
            .with_network(|ctx| {
                Box::pin(async move {
                    HoprLibp2pNetworkBuilder::new(futures::stream::empty())
                        .build(
                            &ctx.packet_key,
                            vec![],
                            hopr_transport::protocol::CURRENT_HOPR_MSG_PROTOCOL,
                            true,
                        )
                        .await
                        .map_err(|error| HoprLibError::GeneralError(error.to_string()))
                })
            })
            .with_cover_traffic(move |ctx| {
                hopr_ct_full_network::FullNetworkDiscovery::new(
                    *ctx.packet_key.public(),
                    Default::default(),
                    graph_for_ct,
                )
            })
            .with_session_server(crate::testing::dummies::EchoServer::default())
            .build_entry(ticket_factory)
            .await?;

        let started = tokio::time::timeout(std::time::Duration::from_secs(5), started_rx)
            .await
            .context("reporter should start or be disabled during node construction")?;
        assert_eq!(
            observer_ref.upgrade().is_some(),
            enabled && with_observer,
            "transport must retain the enabled observer after the factory is consumed"
        );
        if enabled {
            assert_eq!(started?, vec![("application", serde_json::json!({ "ready": true }))]);
            assert_eq!(
                lifetime_rx.try_recv(),
                Err(oneshot::error::TryRecvError::Empty),
                "the reporter must remain alive while the node owns it"
            );
        } else {
            assert!(started.is_err(), "disabled reporting must never poll the task");
        }
        drop(node);
        assert!(
            tokio::time::timeout(std::time::Duration::from_secs(5), lifetime_rx)
                .await
                .context("dropping the node must cancel its reporter")?
                .is_err()
        );
        Ok(())
    }
}
