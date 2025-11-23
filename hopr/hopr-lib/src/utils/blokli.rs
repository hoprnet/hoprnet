use hopr_chain_connector::{BlockchainConnectorConfig, blokli_client::BlokliClientConfig};
pub use hopr_chain_connector::{HoprBlockchainSafeConnector, blokli_client::BlokliClient};

pub const DEFAULT_BLOKLI_URL: &str = "https://blokli.hoprnet.org";

pub async fn init_blokli_connector(
    chain_key: &hopr_transport::ChainKeypair,
    provider: Option<String>,
    safe_module_address: hopr_api::Address,
) -> anyhow::Result<HoprBlockchainSafeConnector<BlokliClient>> {
    tracing::info!("initiating Blokli connector");
    let mut connector = hopr_chain_connector::create_trustful_hopr_blokli_connector(
        chain_key,
        BlockchainConnectorConfig {
            tx_confirm_timeout: std::time::Duration::from_secs(30),
            ..Default::default()
        },
        BlokliClient::new(
            provider.as_deref().unwrap_or(DEFAULT_BLOKLI_URL).parse()?,
            BlokliClientConfig {
                timeout: std::time::Duration::from_secs(5),
            },
        ),
        safe_module_address,
    )
    .await?;
    connector.connect(std::time::Duration::from_secs(30)).await?;

    Ok(connector)
}
