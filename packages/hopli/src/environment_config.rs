use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, path::PathBuf};

/// Type of environment that HOPR node is running in
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum EnvironmentType {
    /// Production environment, on Gnosis chain
    Production,
    /// Staging environment, on Gnosis chain
    Staging,
    /// Development environment, on Gnosis chain
    Development,
    /// Local environment, on anvil-localhost
    Local,
}

impl fmt::Display for EnvironmentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Production => write!(f, "production"),
            Self::Staging => write!(f, "staging"),
            Self::Development => write!(f, "development"),
            Self::Local => write!(f, "local"),
        }
    }
}

/// Contract addresses and configuration associated with a environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDetail {
    /// block number from which the indexer starts
    pub indexer_start_block_number: u32,
    /// Type of environment
    pub environment_type: EnvironmentType,
    /// Contract addresses
    pub addresses: Addresses,
}

/// Contract addresses (directly from deployment logic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Addresses {
    /// address of contract that manages authorization to access the Hopr network
    pub network_registry: String,
    /// address of contract that maps to the requirements that need to be fulfilled
    /// in order to access the network, upgradeable
    pub network_registry_proxy: String,
    /// HoprChannels contract address, implementation of mixnet incentives
    pub channels: String, 
    /// Hopr token contract address
    pub token: String,
    /// contract address of Safe capability module implementation 
    pub module_implementation: String,
    /// address of contract that maps between Safe instances and node addresses
    pub node_safe_registry: String,
    /// address of contract that allows Hopr Association to dictate price per packet in Hopr
    pub ticket_price_oracle: String,
    /// address of contract that manages transport announcements in the hopr network
    pub announcements: String,
    /// factory contract to produce Safe instances
    pub node_stake_v2_factory: String,
}

/// mapping of networks with its details
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    // #[serde(flatten)]
    networks: HashMap<String, NetworkDetail>,
}

/// ensures that the network and environment_type exist
/// in `contracts-addresses.json` and are matched
pub fn ensure_environment_and_network_are_set(
    make_root_dir_path: &PathBuf,
    network: &str,
    environment_type: &str,
) -> Result<bool, String> {
    // read `contracts-addresses.json` at make_root_dir_path
    let contract_environment_config_path = make_root_dir_path.join("contracts-addresses.json");

    let file_read = std::fs::read_to_string(contract_environment_config_path)
        .expect("Unable to read contracts-addresses.json file");

    let env_config =
        serde_json::from_str::<NetworkConfig>(&file_read).expect("Unable to deserialize environment config");

    let env_detail = env_config
        .networks
        .get(network)
        .expect("Unable to find environment details");

    if env_detail.environment_type.to_string() == environment_type {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Returns the environment type from the network name
/// according to `contracts-addresses.json`
pub fn get_environment_type_from_name(make_root_dir_path: &PathBuf, network: &str) -> Result<EnvironmentType, String> {
    // read `contracts-addresses.json` at make_root_dir_path
    let contract_environment_config_path = make_root_dir_path.join("contracts-addresses.json");

    let file_read = std::fs::read_to_string(contract_environment_config_path)
        .expect("Unable to read contracts-addresses.json file");

    let env_config =
        serde_json::from_str::<NetworkConfig>(&file_read).expect("Unable to deserialize environment config");

    let env_detail = env_config
        .networks
        .get(network)
        .expect("Unable to find environment details");

    return Ok(env_detail.environment_type);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_anvil_localhost_at_right_path() {
        let correct_dir = &std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("ethereum")
            .join("contracts");
        let network = "anvil-localhost";
        let environment_type = "local";
        match ensure_environment_and_network_are_set(correct_dir, network, environment_type) {
            Ok(result) => assert_eq!(result, true),
            _ => assert!(false),
        }
    }

    #[test]
    fn read_anvil_localhost_at_wrong_path() {
        let wrong_dir = &std::env::current_dir().unwrap();
        let network = "anvil-localhost";
        let environment_type = "local";
        let result =
            std::panic::catch_unwind(|| ensure_environment_and_network_are_set(wrong_dir, network, environment_type));
        assert!(result.is_err());
    }

    #[test]
    fn read_non_existing_environment_at_right_path() {
        let correct_dir = &std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("ethereum")
            .join("contracts");

        let result = std::panic::catch_unwind(|| {
            ensure_environment_and_network_are_set(correct_dir, "non-existing", "development")
        });
        assert!(result.is_err());
    }

    #[test]
    fn read_wrong_type_at_right_path() {
        let correct_dir = &std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("ethereum")
            .join("contracts");
        let network = "anvil-localhost";
        let environment_type = "production";
        match ensure_environment_and_network_are_set(correct_dir, network, environment_type) {
            Ok(result) => assert_eq!(result, false),
            _ => assert!(false),
        }
    }
}
