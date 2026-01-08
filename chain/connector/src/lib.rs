mod backend;
mod connector;
pub mod errors;
mod reader;
#[cfg(feature = "testing")]
pub mod testing;

pub(crate) mod utils;

#[cfg(feature = "testing")]
pub use backend::InMemoryBackend;
pub use backend::{Backend, TempDbBackend};
pub use connector::{BlockchainConnectorConfig, HoprBlockchainConnector};
pub use hopr_chain_types::payload::{BasicPayloadGenerator, SafePayloadGenerator};
pub use reader::HoprBlockchainReader;

/// Re-exports of the `blokli_client` crate.
pub mod blokli_client {
    pub use blokli_client::{
        BlokliClient, BlokliClientConfig,
        api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient},
    };
}

#[doc(hidden)]
pub mod reexports {
    pub use hopr_chain_types;
    /// Export of relevant types for easier integration.
    pub use hopr_chain_types::exports::alloy;
}

pub use hopr_chain_types::prelude::{ContractAddresses, PayloadGenerator};
pub use hopr_crypto_types::prelude::ChainKeypair;
use hopr_crypto_types::prelude::Keypair;
pub use hopr_primitive_types::prelude::Address;

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
/// The returned instance uses [`TempDbBackend`] and [`hopr_chain_types::payload::bindings_based::SafePayloadGenerator`]
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
        TempDbBackend::new()?,
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
        TempDbBackend::new()?,
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
        TempDbBackend::new()?,
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
        TempDbBackend::new()?,
        payload_gen,
    ))
}
