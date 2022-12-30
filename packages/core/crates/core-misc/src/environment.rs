use real_base::real;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct SemverResult {
    version: String,
}

#[wasm_bindgen(module = "semver")]
extern "C" {
    // Reads the given file and returns it as array of bytes.
    #[wasm_bindgen(catch)]
    pub fn coerce(version: String) -> Result<SemverResult, JsValue>;

    #[wasm_bindgen(catch)]
    pub fn satisfies(version: String, range: String) -> Result<bool, JsValue>;
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum EnvironmentType {
    Production,
    Staging,
    Development,
}

impl ToString for EnvironmentType {
    fn to_string(&self) -> String {
        match self {
            Self::Production => String::from("production"),
            Self::Staging => String::from("staging"),
            Self::Development => String::from("development"),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NetworkOptions {
    id: String,
    description: String,
    /// >= 0
    chain_id: u32,
    live: bool,
    /// a valid HTTP url pointing at a RPC endpoint
    default_provider: String,
    /// a valid HTTP url pointing at a RPC endpoint
    etherscan_api_url: Option<String>,
    /// The absolute maximum you are willing to pay per unit of gas to get your transaction included in a block, e.g. '10 gwei'
    max_fee_per_gas: String,
    /// Tips paid directly to miners, e.g. '2 gwei'
    max_priority_fee_per_gas: String,
    native_token_name: String,
    hopr_token_name: String,
    tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Environment {
    id: String,
    /// must match one of the Network.id
    network_id: String,
    environment_type: EnvironmentType,
    version_range: String,
    channel_contract_deploy_block: u32,
    /// an Ethereum address
    token_contract_address: String,
    /// an Ethereum address
    channels_contract_address: String,
    /// an Ethereum address
    xhopr_contract_address: String,
    /// an Ethereum address
    boost_contract_address: String,
    /// an Ethereum address
    stake_contract_address: String,
    /// an Ethereum address
    network_registry_proxy_contract_address: String,
    /// an Ethereum address
    network_registry_contract_address: String,
}

#[derive(Deserialize)]
pub struct ProtocolConfig {
    environments: std::collections::HashMap<String, Environment>,
    networks: std::collections::HashMap<String, NetworkOptions>,
}

pub fn get_protocol_config(path: &str) -> Result<ProtocolConfig, JsValue> {
    let data = real::read_file(&path)?;

    serde_json::from_slice::<ProtocolConfig>(&data).map_err(|e| JsValue::from(e.to_string()))
}

#[derive(Deserialize)]
struct PackageJsonFile {
    version: String,
}

fn get_package_version(path: &str) -> Result<String, JsValue> {
    let data = real::read_file(&path)?;

    match serde_json::from_slice::<PackageJsonFile>(&data) {
        Ok(json) => Ok(coerce(json.version).unwrap().version),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
}
pub fn supported_environments(
    package_json_path: &str,
    protocol_config_path: &str,
) -> Result<Vec<Environment>, JsValue> {
    let protocol_config = get_protocol_config(&protocol_config_path)?;
    let version = get_package_version(package_json_path)?;

    let mut allowed: Vec<Environment> = vec![];

    for (_id, env) in protocol_config.environments.iter() {
        let range = env.version_range.to_owned();

        if satisfies(version.to_owned(), range).unwrap() {
            allowed.push(env.to_owned())
        }
    }

    Ok(allowed)
}

pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub fn supported_environments(
        package_json_path: &str,
        protocol_config_path: &str,
    ) -> Result<JsValue, JsValue> {
        let supported_envs =
            super::supported_environments(package_json_path, protocol_config_path)?;

        match serde_wasm_bindgen::to_value(&supported_envs) {
            Ok(string) => Ok(string),
            Err(e) => Err(JsValue::from(e.to_string())),
        }
    }
}

// /**
//  * @param version HOPR version
//  * @returns environments that the given HOPR version should be able to use
//  */
// export function supportedEnvironments(): Environment[] {
//     const environments = Object.entries((protocolConfig as ProtocolConfig).environments)

//     return environments
//       .filter(([_, env]) => {
//         return semver.satisfies(FULL_VERSION_COERCED, env.version_range)
//       })
//       .map(([id, env]) => ({
//         id,
//         ...env
//       }))
//   }
