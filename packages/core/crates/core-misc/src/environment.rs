use real_base::real;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen(module = "@hoprnet/hopr-real")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn coerce_version(version: String) -> Result<String, JsValue>;

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

/// Holds all information we need about the blockchain network
/// the client is going to use
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct NetworkOptions {
    #[serde(skip_deserializing)]
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
    tags: Option<Vec<String>>,
}

/// Holds all information about the protocol environment
/// to be used by the client
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    #[serde(skip_deserializing)]
    id: String,
    /// must match one of the Network.id
    network_id: String,
    environment_type: EnvironmentType,
    // Node.js-fashioned semver string
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
    tags: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct ResolvedEnvironment {
    /// the environment identifier, e.g. monte_rosa
    id: String,
    network: NetworkOptions,
    environment_type: EnvironmentType,
    /// an Ethereum address
    channels_contract_address: String,
    channel_contract_deploy_block: u32,
    /// an Ethereum address
    token_contract_address: String,
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
#[serde(deny_unknown_fields)]
pub struct ProtocolConfig {
    environments: std::collections::HashMap<String, Environment>,
    networks: std::collections::HashMap<String, NetworkOptions>,
}

/// Reads current protocol config and returns it
pub fn get_protocol_config(mono_repo_path: &str) -> Result<ProtocolConfig, JsValue> {
    let protocol_config_path = format!("{}/packages/core/protocol-config.json", mono_repo_path);

    let data = real::read_file(protocol_config_path.as_str())?;

    serde_json::from_slice::<ProtocolConfig>(&data).map_err(|e| JsValue::from(e.to_string()))
}

#[derive(Deserialize)]
struct PackageJsonFile {
    version: String,
}

/// Reads `hoprd` package.json file and returns its version
fn get_package_version(mono_repo_path: &str) -> Result<String, JsValue> {
    let package_json_path = format!("{}/packages/hoprd/package.json", mono_repo_path);

    let data = real::read_file(package_json_path.as_str())?;

    let package_json = serde_json::from_slice::<PackageJsonFile>(&data)
        .map_err(|e| JsValue::from(e.to_string()))?;

    /*
     * Coerced full version using
     * coerce_version('42.6.7.9.3-alpha') // '42.6.7'
     */
    coerce_version(package_json.version)
}

/// Returns a list of environments which the node is able to work with
pub fn supported_environments(mono_repo_path: &str) -> Result<Vec<Environment>, JsValue> {
    let mut protocol_config_path = get_protocol_config(&mono_repo_path)?;
    let version = get_package_version(mono_repo_path)?;

    let mut allowed: Vec<Environment> = vec![];

    for (id, env) in protocol_config_path.environments.iter_mut() {
        let range = env.version_range.to_owned();

        if satisfies(version.to_owned(), range).unwrap() {
            env.id = id.to_owned();
            allowed.push(env.to_owned())
        }
    }

    Ok(allowed)
}
/// Returns the environment details, returns an error if environment is not supported
pub fn resolve_environment(
    mono_repo_path: &str,
    environment_id: &str,
    maybe_custom_provider: Option<&str>,
) -> Result<ResolvedEnvironment, JsValue> {
    let mut protocol_config = get_protocol_config(mono_repo_path)?;
    let version = get_package_version(mono_repo_path)?;

    let environment = protocol_config
        .environments
        .get_mut(environment_id)
        .ok_or(JsValue::from(format!(
            "Could not find environment {} in protocol config",
            environment_id
        )))?;

    let network = protocol_config
        .networks
        .get_mut(&environment.network_id)
        .ok_or(JsValue::from(format!(
            "Invalid network_id {} for environment {}",
            environment.network_id, environment_id
        )))?;

    network.id = environment.network_id.to_owned();

    if let Some(custom_provider) = maybe_custom_provider {
        network.default_provider = String::from(custom_provider);
    }

    match satisfies(version, environment.version_range.to_owned()) {
        Ok(true) => Ok(ResolvedEnvironment {
            id: String::from(environment_id),
            network: network.to_owned(),
            environment_type: environment.environment_type,
            channel_contract_deploy_block: environment.channel_contract_deploy_block,
            token_contract_address: environment.token_contract_address.to_owned(),
            channels_contract_address: environment.channels_contract_address.to_owned(),
            xhopr_contract_address: environment.xhopr_contract_address.to_owned(),
            boost_contract_address: environment.boost_contract_address.to_owned(),
            stake_contract_address: environment.stake_contract_address.to_owned(),
            network_registry_contract_address: environment
                .network_registry_contract_address
                .to_owned(),
            network_registry_proxy_contract_address: environment
                .network_registry_proxy_contract_address
                .to_owned(),
        }),
        Ok(false) => match supported_environments(mono_repo_path) {
            Ok(envs) => Err(JsValue::from(format!(
                "environment {} is not supported, supported environments {:?}",
                environment_id,
                envs.iter()
                    .map(|e| e.id.to_owned())
                    .collect::<Vec<String>>()
                    .join(", ")
            ))),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    macro_rules! clean_mono_repo_path {
        ($v:expr,$r:ident) => {
            let $r = $v.strip_suffix("/").unwrap_or($v);
        };
    }

    #[wasm_bindgen]
    pub fn supported_environments(mono_repo_path: &str) -> Result<JsValue, JsValue> {
        clean_mono_repo_path!(mono_repo_path, cleaned_mono_repo_path);

        let supported_envs = super::supported_environments(cleaned_mono_repo_path)?;

        serde_wasm_bindgen::to_value(&supported_envs).map_err(|e| JsValue::from(e.to_string()))
    }

    #[wasm_bindgen]
    pub fn resolve_environment(
        mono_repo_path: &str,
        environment_id: &str,
        maybe_custom_provider: Option<String>,
    ) -> Result<JsValue, JsValue> {
        clean_mono_repo_path!(mono_repo_path, cleaned_mono_repo_path);

        let resolved_environment = super::resolve_environment(
            cleaned_mono_repo_path,
            environment_id,
            maybe_custom_provider.as_ref().map(|c| c.as_str()),
        )?;

        serde_wasm_bindgen::to_value(&resolved_environment)
            .map_err(|e| JsValue::from(e.to_string()))
    }
}
