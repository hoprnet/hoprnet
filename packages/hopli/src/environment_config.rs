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
}

impl fmt::Display for EnvironmentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Production => write!(f, "production"),
            Self::Staging => write!(f, "staging"),
            Self::Development => write!(f, "development"),
        }
    }
}

/// Contract addresses and configuration associated with a environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDetail {
    /// number of the staking season used in the environment
    pub stake_season: u8,
    /// block number from which the indexer starts
    pub indexer_start_block_number: u32,
    /// HoprBoost NFT contract address
    pub boost_contract_address: String,
    /// HoprStake contract address
    pub stake_contract_address: String,
    /// NetworkRegistryProxy contract address
    pub network_registry_proxy_contract_address: String,
    /// Type of environment
    pub environment_type: EnvironmentType,
    /// HoprChannel contract address
    pub channels_contract_address: String,
    /// xHOPR contract address
    pub xhopr_contract_address: String,
    /// NetworkRegistry contract address
    pub network_registry_contract_address: String,
    /// HOPR token contract address
    pub token_contract_address: String,
}

/// mapping of networks with its details
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    // #[serde(flatten)]
    networks: HashMap<String, NetworkDetail>,
}

/// ensures that the network_name and environment_type exist
/// in `contracts-addresses.json` and are matched
pub fn ensure_environment_is_set(
    make_root_dir_path: &PathBuf,
    network_name: &str,
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
        .get(network_name)
        .expect("Unable to find environment details");

    if env_detail.environment_type.to_string() == environment_type {
        return Ok(true);
    } else {
        return Ok(false);
    }
}

/// Returns the environment type from the environment name
/// according to `contracts-addresses.json`
pub fn get_environment_type_from_name(
    make_root_dir_path: &PathBuf,
    network_name: &str,
) -> Result<EnvironmentType, String> {
    // read `contracts-addresses.json` at make_root_dir_path
    let contract_environment_config_path = make_root_dir_path.join("contracts-addresses.json");

    let file_read = std::fs::read_to_string(contract_environment_config_path)
        .expect("Unable to read contracts-addresses.json file");

    let env_config =
        serde_json::from_str::<NetworkConfig>(&file_read).expect("Unable to deserialize environment config");

    let env_detail = env_config
        .networks
        .get(network_name)
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
        let network_name = "anvil-localhost";
        let environment_type = "development";
        match ensure_environment_is_set(correct_dir, network_name, environment_type) {
            Ok(result) => assert_eq!(result, true),
            _ => assert!(false),
        }
    }

    #[test]
    fn read_anvil_localhost_at_wrong_path() {
        let wrong_dir = &std::env::current_dir().unwrap();
        let network_name = "anvil-localhost";
        let environment_type = "development";
        let result = std::panic::catch_unwind(|| ensure_environment_is_set(wrong_dir, network_name, environment_type));
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

        let result = std::panic::catch_unwind(|| ensure_environment_is_set(correct_dir, "non-existing", "development"));
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
        let network_name = "anvil-localhost";
        let environment_type = "production";
        match ensure_environment_is_set(correct_dir, network_name, environment_type) {
            Ok(result) => assert_eq!(result, false),
            _ => assert!(false),
        }
    }
}
