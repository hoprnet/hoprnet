use std::str::FromStr;

use hopr_chain_types::ContractAddresses;
use hopr_primitive_types::primitives::Address;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use validator::Validate;

use crate::errors::HoprChainError;

/// Types of HOPR network environments.
#[derive(Debug, Copy, Clone, Deserialize, Serialize, Eq, PartialEq, strum::Display, strum::EnumString)]
#[serde(rename_all(deserialize = "lowercase"))]
#[strum(serialize_all = "lowercase")]
pub enum EnvironmentType {
    Production,
    Staging,
    Development,
    Local,
}

/// Holds all information we need about the blockchain network
/// the client is going to use
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChainOptions {
    pub description: String,
    /// >= 0
    pub chain_id: u32,
    pub live: bool,
    /// a valid HTTP url pointing at a RPC endpoint
    pub default_provider: String,
    /// a valid HTTP url pointing at a RPC endpoint
    pub etherscan_api_url: Option<String>,
    /// The absolute maximum you are willing to pay per unit of gas to get your transaction included in a block, e.g.
    /// '10 gwei'
    pub max_fee_per_gas: String,
    /// Tips paid directly to miners, e.g. '2 gwei'
    pub max_priority_fee_per_gas: String,
    pub native_token_name: String,
    pub hopr_token_name: String,
    /// expected block time on the chain in milliseconds
    pub block_time: u64,
    /// optional maximum number of RPC requests per second for this chain provider
    pub max_rpc_requests_per_sec: Option<u32>,
    pub tags: Option<Vec<String>>,
}

/// Holds all information about the protocol network
/// to be used by the client
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Network {
    /// must match one of the Network.id
    pub chain: String,
    #[serde_as(as = "DisplayFromStr")]
    pub environment_type: EnvironmentType,
    /// Node.js-fashioned semver string
    pub version_range: String,
    /// block number to start the indexer from
    pub indexer_start_block_number: u32,
    pub tags: Vec<String>,
    /// contract addresses used by the network
    pub addresses: Addresses,
    /// number of follow-on blocks required until a block is considered confirmed on-chain
    pub confirmations: u32,
    /// milliseconds between polling the RPC for new transactions
    pub tx_polling_interval: u64,
    /// maximum block range to fetch while indexing logs
    pub max_block_range: u64,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Addresses {
    /// address of contract that manages authorization to access the Hopr network
    #[serde_as(as = "DisplayFromStr")]
    pub network_registry: Address,
    /// address of contract that maps to the requirements that need to be fulfilled
    /// in order to access the network, upgradeable
    #[serde_as(as = "DisplayFromStr")]
    pub network_registry_proxy: Address,
    /// HoprChannels contract address, implementation of mixnet incentives
    #[serde_as(as = "DisplayFromStr")]
    pub channels: Address,
    /// Hopr token contract address
    #[serde_as(as = "DisplayFromStr")]
    pub token: Address,
    /// contract address of Safe capability module implementation
    #[serde_as(as = "DisplayFromStr")]
    pub module_implementation: Address,
    /// address of contract that maps between Safe instances and node addresses
    #[serde_as(as = "DisplayFromStr")]
    pub node_safe_registry: Address,
    /// address of contract that allows Hopr Association to dictate price per packet in Hopr
    #[serde_as(as = "DisplayFromStr")]
    pub ticket_price_oracle: Address,
    /// address of contract that allows Hopr Association to dictate the minimum ticket winning probability in Hopr
    #[serde_as(as = "DisplayFromStr")]
    pub winning_probability_oracle: Address,
    /// address of contract that manages transport announcements in the hopr network
    #[serde_as(as = "DisplayFromStr")]
    pub announcements: Address,
    /// factory contract to produce Safe instances
    #[serde_as(as = "DisplayFromStr")]
    pub node_stake_v2_factory: Address,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNetworkConfig {
    /// the network identifier, e.g. monte_rosa
    pub id: String,
    pub chain: ChainOptions,
    pub environment_type: EnvironmentType,
    pub channel_contract_deploy_block: u32,
    /// address of contract that manages authorization to access the Hopr network
    #[serde_as(as = "DisplayFromStr")]
    pub network_registry: Address,
    /// address of contract that maps to the requirements that need to be fulfilled
    /// in order to access the network, upgradeable
    #[serde_as(as = "DisplayFromStr")]
    pub network_registry_proxy: Address,
    /// HoprChannels contract address, implementation of mixnet incentives
    #[serde_as(as = "DisplayFromStr")]
    pub channels: Address,
    /// Hopr token contract address
    #[serde_as(as = "DisplayFromStr")]
    pub token: Address,
    /// contract address of Safe capability module implementation
    #[serde_as(as = "DisplayFromStr")]
    pub module_implementation: Address,
    /// address of contract that maps between Safe instances and node addresses
    #[serde_as(as = "DisplayFromStr")]
    pub node_safe_registry: Address,
    /// address of contract that allows Hopr Association to dictate price per packet in Hopr
    #[serde_as(as = "DisplayFromStr")]
    pub ticket_price_oracle: Address,
    /// address of contract that allows Hopr Association to dictate the minimum ticket winning probability in Hopr
    #[serde_as(as = "DisplayFromStr")]
    pub winning_probability_oracle: Address,
    /// address of contract that manages transport announcements in the hopr network
    #[serde_as(as = "DisplayFromStr")]
    pub announcements: Address,
    /// factory contract to produce Safe instances
    #[serde_as(as = "DisplayFromStr")]
    pub node_stake_v2_factory: Address,
    /// number of follow-on blocks required until a block is considered confirmed on-chain
    pub confirmations: u32,
    /// milliseconds between polling the RPC for new transactions
    pub tx_polling_interval: u64,
    /// maximum block range to fetch when indexing logs
    pub max_block_range: u64,
    /// maximum number of RPC requests per second
    pub max_requests_per_sec: Option<u32>,
}

/// Check whether the version is allowed
fn satisfies(version: &str, allowed_versions: &str) -> crate::errors::Result<bool> {
    let allowed_versions = VersionReq::parse(allowed_versions)
        .map_err(|e| HoprChainError::Configuration(format!("failed to deserialize allowed version string: {e}")))?;

    let version = Version::from_str(version)
        .map_err(|e| HoprChainError::Configuration(format!("failed to deserialize current lib version string: {e}")))?;

    Ok(allowed_versions.matches(&version))
}

impl ChainNetworkConfig {
    /// Returns the network details, returns an error if network is not supported
    pub fn new(
        id: &str,
        version: &str,
        maybe_custom_provider: Option<&str>,
        max_rpc_requests_per_sec: Option<u32>,
        protocol_config: &mut ProtocolsConfig,
    ) -> Result<Self, String> {
        let network = protocol_config
            .networks
            .get_mut(id)
            .ok_or(format!("Could not find network {id} in protocol config"))?;

        let chain = protocol_config
            .chains
            .get_mut(&network.chain)
            .ok_or(format!("Invalid chain {} for network {id}", network.chain))?;

        if let Some(custom_provider) = maybe_custom_provider {
            chain.default_provider = custom_provider.into();
        }

        match satisfies(version, network.version_range.as_str()) {
            Ok(true) => Ok(ChainNetworkConfig {
                announcements: network.addresses.announcements.to_owned(),
                chain: chain.to_owned(),
                channel_contract_deploy_block: network.indexer_start_block_number,
                channels: network.addresses.channels.to_owned(),
                confirmations: network.confirmations,
                environment_type: network.environment_type,
                id: network.chain.to_owned(),
                module_implementation: network.addresses.module_implementation.to_owned(),
                network_registry: network.addresses.network_registry.to_owned(),
                network_registry_proxy: network.addresses.network_registry_proxy.to_owned(),
                node_safe_registry: network.addresses.node_safe_registry.to_owned(),
                node_stake_v2_factory: network.addresses.node_stake_v2_factory.to_owned(),
                ticket_price_oracle: network.addresses.ticket_price_oracle.to_owned(),
                winning_probability_oracle: network.addresses.winning_probability_oracle.to_owned(),
                token: network.addresses.token.to_owned(),
                tx_polling_interval: network.tx_polling_interval,
                max_block_range: network.max_block_range,
                max_requests_per_sec: max_rpc_requests_per_sec.or(chain.max_rpc_requests_per_sec),
            }),
            Ok(false) => Err(format!(
                "network {id} is not supported, supported networks {:?}",
                protocol_config.supported_networks(version).join(", ")
            )),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl From<&ChainNetworkConfig> for ContractAddresses {
    fn from(network: &ChainNetworkConfig) -> Self {
        Self {
            token: network.token,
            channels: network.channels,
            announcements: network.announcements,
            network_registry: network.network_registry,
            network_registry_proxy: network.network_registry_proxy,
            safe_registry: network.node_safe_registry,
            price_oracle: network.ticket_price_oracle,
            win_prob_oracle: network.winning_probability_oracle,
            stake_factory: network.node_stake_v2_factory,
            module_implementation: network.module_implementation,
        }
    }
}

/// The entire on-chain protocol configuration containing the information about
/// usable networks and chains.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ProtocolsConfig {
    pub networks: std::collections::BTreeMap<String, Network>,
    pub chains: std::collections::BTreeMap<String, ChainOptions>,
}

impl Default for ProtocolsConfig {
    fn default() -> Self {
        Self::from_str(include_str!("../../../hopr/hopr-lib/data/protocol-config.json"))
            .expect("bundled protocol config should be always valid")
    }
}

impl FromStr for ProtocolsConfig {
    type Err = String;

    /// Reads the protocol config JSON file and returns it
    fn from_str(data: &str) -> Result<Self, Self::Err> {
        serde_json::from_str::<ProtocolsConfig>(data).map_err(|e| e.to_string())
    }
}

impl std::cmp::PartialEq for ProtocolsConfig {
    fn eq(&self, other: &Self) -> bool {
        Vec::from_iter(self.networks.clone()) == Vec::from_iter(other.networks.clone())
            && Vec::from_iter(self.chains.clone()) == Vec::from_iter(self.chains.clone())
    }
}

impl ProtocolsConfig {
    /// Returns a list of environments which the node is able to work with
    /// TODO: crate::constants::APP_VERSION_COERCED
    pub fn supported_networks(&self, version: &str) -> Vec<String> {
        let mut allowed = vec![];

        for (name, env) in self.networks.iter() {
            let range = env.version_range.to_owned();

            if let Ok(true) = satisfies(version, range.as_str()) {
                allowed.push(name.clone())
            }
        }

        allowed
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_protocol_config_can_be_deserialized() {
        let _ = ProtocolsConfig::default();
    }

    #[test]
    fn test_version_is_satisfied_should_work_on_ranges() {
        let actual = satisfies("1.90.0", ">=1.89, <1.93");
        assert!(actual.is_ok());
        assert!(actual.expect("should be contained"))
    }

    #[test]
    fn test_version_is_satisfied_should_work_for_glob() {
        let actual = satisfies("1.2.3", "*");
        assert!(actual.is_ok());
        assert!(actual.expect("should be contained"))
    }
}
