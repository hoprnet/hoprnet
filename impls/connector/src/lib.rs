mod backend;
mod connector;
pub mod errors;
mod reader;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub(crate) mod utils;

#[cfg(any(test, feature = "testing"))]
pub use backend::InMemoryBackend;
pub use backend::{Backend, TempDbBackend, TempDbError};
pub use connector::{BlockchainConnectorConfig, HoprBlockchainConnector};
pub use hopr_api::{
    chain as api,
    types::chain::payload::{BasicPayloadGenerator, SafePayloadGenerator},
};
pub use reader::HoprBlockchainReader;

/// Re-exports of the `blokli_client` crate.
pub mod blokli_client {
    pub use blokli_client::{
        BlokliClient, BlokliClientConfig, BlokliDnsOverride,
        api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient, types},
        exports::Url,
    };
}

/// Configuration for creating a [`blokli_client::BlokliClient`] via [`create_blokli_client`].
///
/// The connector applies its own opinionated defaults for timeouts and reconnection behaviour;
/// only `url` and `dns_override` are caller-controlled.
///
/// # Examples
///
/// Basic usage — no DNS override:
/// ```ignore
/// let client = create_blokli_client(HoprBlokliClientConfig::new(url));
/// ```
///
/// With DNS override (useful when system DNS is unavailable):
/// ```ignore
/// let client = create_blokli_client(HoprBlokliClientConfig {
///     url,
///     dns_override: Some((ip, Some(8545))),
/// });
/// ```
#[derive(Clone, Debug, validator::Validate)]
pub struct HoprBlokliClientConfig {
    /// Blokli service URL.
    pub url: blokli_client::Url,
    /// Optional DNS override: an IP address (and optional port) to use instead of resolving
    /// [`Self::url`]'s host via DNS.
    ///
    /// When `None` (the default) system DNS is used. When set, the connection goes directly to
    /// the given IP, while the original host is kept for the `Host` header and TLS SNI.
    /// `port` defaults to the URL's port when `None`.
    pub dns_override: Option<(std::net::IpAddr, Option<u16>)>,
}

impl HoprBlokliClientConfig {
    /// Creates a config with the given URL and no DNS override.
    pub fn new(url: blokli_client::Url) -> Self {
        Self {
            url,
            dns_override: None,
        }
    }
}

/// Creates a [`blokli_client::BlokliClient`] with the connector's opinionated defaults.
///
/// Applies a 3 s general timeout and 30 s SSE reconnect timeout. Callers that need DNS
/// pinning set [`HoprBlokliClientConfig::dns_override`]; all other settings are fixed.
pub fn create_blokli_client(cfg: HoprBlokliClientConfig) -> blokli_client::BlokliClient {
    blokli_client::BlokliClient::new(
        cfg.url,
        blokli_client::BlokliClientConfig {
            timeout: std::time::Duration::from_secs(3),
            stream_reconnect_timeout: std::time::Duration::from_secs(30),
            auto_compatibility_check: false,
            dns_override: cfg
                .dns_override
                .map(|(ip, port)| ::blokli_client::BlokliDnsOverride { ip, port }),
            ..Default::default()
        },
    )
}

#[doc(hidden)]
pub mod reexports {
    pub use hopr_api::types::chain;
}

use hopr_api::types::crypto::prelude::Keypair;
pub use hopr_api::types::{
    chain::prelude::{ContractAddresses, PayloadGenerator},
    crypto::prelude::ChainKeypair,
    primitive::prelude::Address,
};

/// Connector to HOPR on-chain contracts that uses multisig Safe as a signer and [`TempDbBackend`].
pub type HoprBlockchainSafeConnector<C> = HoprBlockchainConnector<
    C,
    TempDbBackend,
    SafePayloadGenerator,
    <SafePayloadGenerator as PayloadGenerator>::TxRequest,
>;

/// Connector to HOPR on-chain contracts that uses standard EOA as a signer and [`TempDbBackend`].
pub type HoprBlockchainBasicConnector<C> = HoprBlockchainConnector<
    C,
    TempDbBackend,
    BasicPayloadGenerator,
    <BasicPayloadGenerator as PayloadGenerator>::TxRequest,
>;

/// Convenience function to create [`HoprBlockchainConnector`] with own contract addresses.
///
/// The returned instance uses [`TempDbBackend`] and
/// `SafePayloadGenerator`
pub fn create_trustless_hopr_blokli_connector<C>(
    chain_key: &ChainKeypair,
    cfg: BlockchainConnectorConfig,
    client: C,
    module_address: Address,
    contracts: ContractAddresses,
) -> Result<HoprBlockchainSafeConnector<C>, errors::ConnectorError>
where
    C: blokli_client::BlokliSubscriptionClient
        + blokli_client::BlokliQueryClient
        + blokli_client::BlokliTransactionClient
        + Send
        + Sync
        + 'static,
{
    let payload_gen = SafePayloadGenerator::new(chain_key, contracts, module_address);

    Ok(HoprBlockchainConnector::new(
        chain_key.clone(),
        cfg,
        client,
        TempDbBackend::new().map_err(errors::ConnectorError::backend)?,
        payload_gen,
    ))
}

/// Convenience function to create [`HoprBlockchainConnector`] with own contract addresses.
///
/// The transactions generated using this Connector are simply signed using the `chain_key` EOA.
///
/// The returned instance uses [`TempDbBackend`] and [`BasicPayloadGenerator`]
pub fn create_trustless_safeless_hopr_blokli_connector<C>(
    chain_key: &ChainKeypair,
    cfg: BlockchainConnectorConfig,
    client: C,
    contracts: ContractAddresses,
) -> Result<HoprBlockchainBasicConnector<C>, errors::ConnectorError>
where
    C: blokli_client::BlokliSubscriptionClient
        + blokli_client::BlokliQueryClient
        + blokli_client::BlokliTransactionClient
        + Send
        + Sync
        + 'static,
{
    let payload_gen = BasicPayloadGenerator::new(chain_key.public().to_address(), contracts);

    Ok(HoprBlockchainConnector::new(
        chain_key.clone(),
        cfg,
        client,
        TempDbBackend::new().map_err(errors::ConnectorError::backend)?,
        payload_gen,
    ))
}

/// Convenience function to create [`HoprBlockchainConnector`] with contract addresses retrieved from the given
/// `client`.
///
/// This instantiation explicitly trusts the contract address information retrieved from the
/// [`blokli_client::BlokliClient`].
/// If you wish to provide your own deployment information, use the [`create_trustless_hopr_blokli_connector`] function.
///
/// The returned instance uses [`TempDbBackend`] and [`SafePayloadGenerator`].
pub async fn create_trustful_hopr_blokli_connector<C>(
    chain_key: &ChainKeypair,
    cfg: BlockchainConnectorConfig,
    client: C,
    module_address: Address,
) -> Result<HoprBlockchainSafeConnector<C>, errors::ConnectorError>
where
    C: blokli_client::BlokliSubscriptionClient
        + blokli_client::BlokliQueryClient
        + blokli_client::BlokliTransactionClient
        + Send
        + Sync
        + 'static,
{
    let info = client.query_chain_info().await?;
    let contract_addrs = serde_json::from_str(&info.contract_addresses.0)
        .map_err(|e| errors::ConnectorError::TypeConversion(format!("contract addresses not a valid JSON: {e}")))?;

    let payload_gen = SafePayloadGenerator::new(chain_key, contract_addrs, module_address);

    Ok(HoprBlockchainConnector::new(
        chain_key.clone(),
        cfg,
        client,
        TempDbBackend::new().map_err(errors::ConnectorError::backend)?,
        payload_gen,
    ))
}

/// Convenience function to create [`HoprBlockchainConnector`] with contract addresses retrieved from the given
/// `client`.
///
/// The transactions generated using this Connector are simply signed using the `chain_key` EOA.
///
/// This instantiation explicitly trusts the contract address information retrieved from the
/// [`blokli_client::BlokliClient`].
/// If you wish to provide your own deployment information, use the [`create_trustless_safeless_hopr_blokli_connector`]
/// function.
///
/// The returned instance uses [`TempDbBackend`] and [`BasicPayloadGenerator`].
pub async fn create_trustful_safeless_hopr_blokli_connector<C>(
    chain_key: &ChainKeypair,
    cfg: BlockchainConnectorConfig,
    client: C,
) -> Result<HoprBlockchainBasicConnector<C>, errors::ConnectorError>
where
    C: blokli_client::BlokliSubscriptionClient
        + blokli_client::BlokliQueryClient
        + blokli_client::BlokliTransactionClient
        + Send
        + Sync
        + 'static,
{
    let info = client.query_chain_info().await?;
    let contract_addrs = serde_json::from_str(&info.contract_addresses.0)
        .map_err(|e| errors::ConnectorError::TypeConversion(format!("contract addresses not a valid JSON: {e}")))?;

    let payload_gen = BasicPayloadGenerator::new(chain_key.public().to_address(), contract_addrs);

    Ok(HoprBlockchainConnector::new(
        chain_key.clone(),
        cfg,
        client,
        TempDbBackend::new().map_err(errors::ConnectorError::backend)?,
        payload_gen,
    ))
}
