use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum EnvironmentType {
    Production,
    Staging,
    Development,
}

impl ToString for EnvironmentType {
    fn to_string(&self) -> String {
        match self {
            Self::Production => "production".into(),
            Self::Staging => "staging".into(),
            Self::Development => "development".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDetail {
    pub stake_season: u8,
    pub indexer_start_block_number: u32,
    pub boost_contract_address: String,
    pub stake_contract_address: String,
    pub network_registry_proxy_contract_address: String,
    pub environment_type: EnvironmentType,
    pub channels_contract_address: String,
    pub xhopr_contract_address: String,
    pub network_registry_contract_address: String,
    pub token_contract_address: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    // #[serde(flatten)]
    environments: HashMap<String, EnvironmentDetail>,
}

pub fn ensure_environment_is_set(
    make_root_dir_path: &PathBuf,
    environment_name: &str,
    environment_type: &str,
) -> Result<bool, String> {
    // read `contracts-addresses.json` at make_root_dir_path
    let contract_environment_config_path = make_root_dir_path.join("contracts-addresses.json");

    let file_read = std::fs::read_to_string(contract_environment_config_path)
        .expect("Unable to read contracts-addresses.json file");

    let env_config = serde_json::from_str::<EnvironmentConfig>(&file_read)
        .expect("Unable to deserialize environment config");

    let env_detail = env_config
        .environments
        .get(environment_name)
        .expect("Unable to find environment details");

    if env_detail.environment_type.to_string() == environment_type {
        return Ok(true);
    } else {
        return Ok(false);
    }
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
            .join("contracts");
        let environment_name = "anvil-localhost";
        let environment_type = "development";
        match ensure_environment_is_set(correct_dir, environment_name, environment_type) {
            Ok(result) => assert_eq!(result, true),
            _ => assert!(false),
        }
    }

    #[test]
    fn read_anvil_localhost_at_wrong_path() {
        let wrong_dir = &std::env::current_dir().unwrap();
        let environment_name = "anvil-localhost";
        let environment_type = "development";
        let result = std::panic::catch_unwind(|| {
            ensure_environment_is_set(wrong_dir, environment_name, environment_type)
        });
        assert!(result.is_err());
    }

    #[test]
    fn read_non_existing_environment_at_right_path() {
        let correct_dir = &std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("contracts");

        let result = std::panic::catch_unwind(|| {
            ensure_environment_is_set(correct_dir, "non-existing", "development")
        });
        assert!(result.is_err());
    }

    #[test]
    fn read_wrong_type_at_right_path() {
        let correct_dir = &std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("contracts");
        let environment_name = "anvil-localhost";
        let environment_type = "production";
        match ensure_environment_is_set(correct_dir, environment_name, environment_type) {
            Ok(result) => assert_eq!(result, false),
            _ => assert!(false),
        }
    }
}
