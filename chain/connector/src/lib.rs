mod backend;
mod connector;
pub mod errors;

pub use backend::{Backend, InMemoryBackend, TempDbBackend};
pub use connector::HoprBlockchainConnector;

/// Re-exports of the `blokli_client` crate.
pub mod blokli_client {
    pub use blokli_client::{
        BlokliClient,
        api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient},
    };
}

pub use hopr_chain_types::ContractAddresses;
pub use hopr_crypto_types::prelude::ChainKeypair;
pub use hopr_primitive_types::prelude::Address;

/// Type alias for a [`HoprBlockchainConnector`] that uses a [`TempDbBackend`] and a [`blokli_client::BlokliClient`].
pub type HoprBlokliConnector = HoprBlockchainConnector<
    TempDbBackend,
    blokli_client::BlokliClient,
    hopr_chain_types::payload::SafePayloadGenerator,
>;

/// Convenience function to create [`HoprBlokliConnector`] with own contract addresses.
pub fn create_trustless_hopr_blokli_connector(
    chain_key: &ChainKeypair,
    client: blokli_client::BlokliClient,
    module_address: Address,
    contracts: ContractAddresses,
) -> Result<HoprBlokliConnector, errors::ConnectorError> {
    let payload_gen = hopr_chain_types::payload::SafePayloadGenerator::new(chain_key, contracts, module_address);

    Ok(HoprBlockchainConnector::new(
        chain_key.clone(),
        client,
        TempDbBackend::new()?,
        payload_gen,
    ))
}

/// Convenience function to create [`HoprBlokliConnector`] with contract addresses retrieved from the given `client`.
///
/// This instantiation explicitly trusts the contract address information retrieved from the
/// [`blokli_client::BlokliClient`].
pub async fn create_trustful_hopr_blokli_connector(
    _chain_key: &ChainKeypair,
    _client: blokli_client::BlokliClient,
    _module_address: Address,
) -> Result<HoprBlokliConnector, errors::ConnectorError> {
    todo!()
}
