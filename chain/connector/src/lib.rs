mod backend;
mod connector;
pub mod errors;

pub use connector::{HoprBlockchainConnector};
pub use backend::{Backend, InMemoryBackend, TempDbBackend};

use hopr_chain_types::ContractAddresses;
use hopr_crypto_types::prelude::ChainKeypair;
use hopr_primitive_types::prelude::Address;
use crate::errors::ConnectorError;

pub type HoprBlokliConnector = HoprBlockchainConnector<TempDbBackend, blokli_client::BlokliClient, hopr_chain_types::payload::SafePayloadGenerator>;

pub fn create_hopr_blokli_connector(
    chain_key: &ChainKeypair,
    blokli_url: &str,
    safe_address: Address,
    module_address: Address,
    contracts: ContractAddresses,
) -> Result<HoprBlokliConnector, errors::ConnectorError> {
    let payload_gen = hopr_chain_types::payload::SafePayloadGenerator::new(
        chain_key, contracts, module_address
    );

    let client = blokli_client::BlokliClient::new(
          blokli_url.parse().map_err(|_| ConnectorError::InvalidArguments("invalid blokli url"))?,
          blokli_client::BlokliClientConfig::default()
    );

    Ok(HoprBlokliConnector::new(chain_key.clone(), safe_address, client, TempDbBackend::new()?, payload_gen))
}