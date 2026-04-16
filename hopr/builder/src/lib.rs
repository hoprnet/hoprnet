#[cfg(feature = "session-server")]
pub mod exit;

#[cfg(feature = "session-server")]
pub mod config;

#[cfg(any(feature = "testing", test))]
pub mod testing;

use std::sync::Arc;

use hopr_chain_connector::{
    BlockchainConnectorConfig, HoprBlockchainSafeConnector,
    api::HoprChainApi,
    blokli_client::{BlokliClient, BlokliClientConfig},
    create_trustful_hopr_blokli_connector,
};
#[cfg(feature = "runtime-tokio")]
pub use hopr_lib;
#[cfg(feature = "session-server")]
use hopr_lib::traits::HoprSessionServer;
#[cfg(feature = "runtime-tokio")]
use hopr_lib::{
    ChainKeypair, Hopr, HoprLibError, OffchainKeypair,
    config::HoprLibConfig,
};
use hopr_lib::Keypair;
use hopr_network_graph::SharedChannelGraph;
use hopr_transport_p2p::HoprNetwork;
#[cfg(feature = "runtime-tokio")]
use validator::Validate;

#[cfg(feature = "session-server")]
use crate::{config::SessionIpForwardingConfig, exit::HoprServerIpForwardingReactor};

pub type ReferenceHopr = Hopr<Arc<HoprBlockchainSafeConnector<BlokliClient>>, SharedChannelGraph, HoprNetwork, hopr_lib::builder::SharedTicketManager>;

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
) -> anyhow::Result<Arc<Hopr<Chain, SharedChannelGraph, HoprNetwork, hopr_lib::builder::SharedTicketManager>>>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
{
    if let Some(ref pcfg) = probe_cfg {
        pcfg.validate()
            .map_err(|e| anyhow::anyhow!("invalid ProberConfig: {e}"))?;

        let probe_timeout = config.protocol.probe.timeout;
        anyhow::ensure!(
            pcfg.interval >= probe_timeout,
            "ProberConfig interval ({:?}) must be >= ProbeConfig timeout ({:?}) to prevent overlapping probe rounds",
            pcfg.interval,
            probe_timeout,
        );
    }

    #[cfg(feature = "session-server")]
    let builder = hopr_lib::builder::HoprBuilder::default();
    #[cfg(not(feature = "session-server"))]
    let builder = hopr_lib::builder::HoprBuilder::<Chain, ()>::default();

    let mut builder = builder
        .chain(chain_connector)
        .identity(chain_key, packet_key)
        .safe_and_module(&config.safe_module.safe_address, &config.safe_module.module_address)
        .with_config(config);

    if let Some(pcfg) = probe_cfg {
        builder = builder.with_ct_prober_config(pcfg);
    }

    #[cfg(feature = "session-server")]
    let node = builder.session_server(server).build_full().await?;
    #[cfg(not(feature = "session-server"))]
    let node = builder.build_full().await?;

    Ok(Arc::new(node))
}
