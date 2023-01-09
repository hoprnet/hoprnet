use real_base::real;
use serde::{Deserialize, Serialize};

macro_rules! ok_or_str {
    ($v:expr) => {
        $v.map_err(|e| e.to_string())
    };
}

pub trait FromJsonFile: Sized {
    fn from_json_file(mono_repo_path: &str) -> Result<Self, String>;
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
            Self::Production => "production".into(),
            Self::Staging => "staging".into(),
            Self::Development => "development".into(),
        }
    }
}

/// Holds all information we need about the blockchain network
/// the client is going to use
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct NetworkOptions {
    #[serde(skip_deserializing)]
    pub id: String,
    pub description: String,
    /// >= 0
    pub chain_id: u32,
    pub live: bool,
    /// a valid HTTP url pointing at a RPC endpoint
    pub default_provider: String,
    /// a valid HTTP url pointing at a RPC endpoint
    pub etherscan_api_url: Option<String>,
    /// The absolute maximum you are willing to pay per unit of gas to get your transaction included in a block, e.g. '10 gwei'
    pub max_fee_per_gas: String,
    /// Tips paid directly to miners, e.g. '2 gwei'
    pub max_priority_fee_per_gas: String,
    pub native_token_name: String,
    pub hopr_token_name: String,
    pub tags: Option<Vec<String>>,
}

/// Holds all information about the protocol environment
/// to be used by the client
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    #[serde(skip_deserializing)]
    pub id: String,
    /// must match one of the Network.id
    pub network_id: String,
    pub environment_type: EnvironmentType,
    // Node.js-fashioned semver string
    pub version_range: String,
    pub channel_contract_deploy_block: u32,
    /// an Ethereum address
    pub token_contract_address: String,
    /// an Ethereum address
    pub channels_contract_address: String,
    /// an Ethereum address
    pub xhopr_contract_address: String,
    /// an Ethereum address
    pub boost_contract_address: String,
    /// an Ethereum address
    pub stake_contract_address: String,
    /// an Ethereum address
    pub network_registry_proxy_contract_address: String,
    /// an Ethereum address
    pub network_registry_contract_address: String,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolConfig {
    pub environments: std::collections::HashMap<String, Environment>,
    pub networks: std::collections::HashMap<String, NetworkOptions>,
}

impl FromJsonFile for ProtocolConfig {
    /// Reads the protocol config JSON file and returns it
    fn from_json_file(mono_repo_path: &str) -> Result<Self, String> {
        let protocol_config_path = format!("{}/packages/core/protocol-config.json", mono_repo_path);

        let data = ok_or_str!(real::read_file(protocol_config_path.as_str()))?;

        let mut protocol_config = ok_or_str!(serde_json::from_slice::<ProtocolConfig>(&data))?;

        for (id, env) in protocol_config.environments.iter_mut() {
            env.id = id.to_owned();
        }

        for (id, network) in protocol_config.networks.iter_mut() {
            network.id = id.to_owned();
        }

        Ok(protocol_config)
    }
}

impl ProtocolConfig {
    /// Returns a list of environments which the node is able to work with
    fn supported_environments(&self, mono_repo_path: &str) -> Result<Vec<Environment>, String> {
        let version = PackageJsonFile::from_json_file(&mono_repo_path)
            .and_then(|p| ok_or_str!(real::coerce_version(p.version.as_str())))?;

        let mut allowed: Vec<Environment> = vec![];

        for (_, env) in self.environments.iter() {
            let range = env.version_range.to_owned();

            if let Ok(true) = real::satisfies(version.as_str(), range.as_str()) {
                allowed.push(env.to_owned())
            }
        }

        Ok(allowed)
    }
}

#[derive(Deserialize)]
struct PackageJsonFile {
    version: String,
}

impl FromJsonFile for PackageJsonFile {
    fn from_json_file(mono_repo_path: &str) -> Result<Self, String> {
        let package_json_path = format!("{}/packages/hoprd/package.json", mono_repo_path);

        let data = ok_or_str!(real::read_file(package_json_path.as_str()))?;

        ok_or_str!(serde_json::from_slice::<PackageJsonFile>(&data))
    }
}

impl PackageJsonFile {
    fn coerced_version(&self) -> Result<String, String> {
        /*
         * Coerced full version using
         * coerce_version('42.6.7.9.3-alpha') // '42.6.7'
         */
        ok_or_str!(real::coerce_version(self.version.as_str()))
    }
}

#[derive(Serialize, Clone)]
pub struct ResolvedEnvironment {
    /// the environment identifier, e.g. monte_rosa
    pub id: String,
    pub network: NetworkOptions,
    pub environment_type: EnvironmentType,
    /// an Ethereum address
    pub channels_contract_address: String,
    pub channel_contract_deploy_block: u32,
    /// an Ethereum address
    pub token_contract_address: String,
    /// an Ethereum address
    pub xhopr_contract_address: String,
    /// an Ethereum address
    pub boost_contract_address: String,
    /// an Ethereum address
    pub stake_contract_address: String,
    /// an Ethereum address
    pub network_registry_proxy_contract_address: String,
    /// an Ethereum address
    pub network_registry_contract_address: String,
}

impl ResolvedEnvironment {
    /// Returns the environment details, returns an error if environment is not supported
    fn new(
        mono_repo_path: &str,
        environment_id: &str,
        maybe_custom_provider: Option<&str>,
    ) -> Result<Self, String> {
        let mut protocol_config = ProtocolConfig::from_json_file(mono_repo_path)?;
        let version =
            PackageJsonFile::from_json_file(mono_repo_path).and_then(|c| c.coerced_version())?;

        let environment = protocol_config
            .environments
            .get_mut(environment_id)
            .ok_or(format!(
                "Could not find environment {} in protocol config",
                environment_id
            ))?;

        let network = protocol_config
            .networks
            .get_mut(&environment.network_id)
            .ok_or(format!(
                "Invalid network_id {} for environment {}",
                environment.network_id, environment_id
            ))?;

        if let Some(custom_provider) = maybe_custom_provider {
            network.default_provider = custom_provider.into();
        }

        match real::satisfies(version.as_str(), environment.version_range.as_str()) {
            Ok(true) => Ok(ResolvedEnvironment {
                id: environment_id.into(),
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
            Ok(false) => protocol_config
                .supported_environments(mono_repo_path)
                .and_then(|envs| {
                    Err(format!(
                        "environment {} is not supported, supported environments {:?}",
                        environment_id,
                        envs.iter()
                            .map(|e| e.id.to_owned())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                    .into())
                }),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub mod wasm {
    use super::FromJsonFile;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    pub type JsResult<T> = Result<T, JsValue>;

    macro_rules! ok_or_jserr {
        ($v:expr) => {
            $v.map_err(|e| JsValue::from(e.to_string()))
        };
    }

    macro_rules! clean_mono_repo_path {
        ($v:expr,$r:ident) => {
            let $r = $v.strip_suffix("/").unwrap_or($v);
        };
    }

    #[wasm_bindgen]
    pub fn supported_environments(mono_repo_path: &str) -> JsResult<JsValue> {
        clean_mono_repo_path!(mono_repo_path, cleaned_mono_repo_path);

        let supported_envs = super::ProtocolConfig::from_json_file(cleaned_mono_repo_path)
            .and_then(|c| c.supported_environments(cleaned_mono_repo_path))?;

        ok_or_jserr!(serde_wasm_bindgen::to_value(&supported_envs))
    }

    #[wasm_bindgen]
    pub fn resolve_environment(
        mono_repo_path: &str,
        environment_id: &str,
        maybe_custom_provider: Option<String>,
    ) -> JsResult<JsValue> {
        clean_mono_repo_path!(mono_repo_path, cleaned_mono_repo_path);

        let resolved_environment = super::ResolvedEnvironment::new(
            cleaned_mono_repo_path,
            environment_id,
            maybe_custom_provider.as_ref().map(|c| c.as_str()),
        )?;

        ok_or_jserr!(serde_wasm_bindgen::to_value(&resolved_environment))
    }
}
