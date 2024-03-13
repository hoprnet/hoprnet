//! This module contains definiation of arguments that specify the environment
//! and networks that a HOPR node runs in.
//!
//! [EnvironmentType] defines the environment type. EnvironmentType of a network is defined in
//! `contracts-address.json` under the foundry contract root. Different environment type uses
//! a different foundry profile.
//!
//! Network is a collection of several major/minor releases.
//!
//! [NetworkDetail] specifies the environment type of the network, the starting block number, and
//! the deployed contract addresses in [NetworkContractAddresses]

use crate::utils::HelperErrors;
use chain_api::{DefaultHttpPostRequestor, JsonRpcClient};
use chain_rpc::{client::SimpleJsonRpcRetryPolicy, errors::RpcError, rpc::RpcOperationsConfig};
use clap::Parser;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::{MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
};
use hopr_crypto_types::keypairs::ChainKeypair;
use hopr_crypto_types::keypairs::Keypair;
use hopr_primitive_types::primitives::Address;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

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
    pub addresses: NetworkContractAddresses,
}

/// Contract addresses (directly from deployment logic)
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContractAddresses {
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
    /// address of contract that manages transport announcements in the hopr network
    #[serde_as(as = "DisplayFromStr")]
    pub announcements: Address,
    /// factory contract to produce Safe instances
    #[serde_as(as = "DisplayFromStr")]
    pub node_stake_v2_factory: Address,
}

/// mapping of networks with its details
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    // #[serde(flatten)]
    networks: HashMap<String, NetworkDetail>,
}

/// Arguments for getting network and ethereum RPC provider.
///
/// RPC provider specifies an endpoint that enables an application to communicate with a blockchain network
/// If not specified, it uses the default value according to the environment config
/// Network specifies a set of contracts used in HOPR network.
#[derive(Debug, Clone, Parser)]
pub struct NetworkProviderArgs {
    /// Name of the network that the node is running on
    #[clap(help = "Network name. E.g. monte_rosa", long, short)]
    network: String,

    /// Path to the root of foundry project (etehereum/contracts), where all the contracts and `contracts-addresses.json` are stored
    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    /// Customized RPC provider endpoint
    #[clap(help = "Blockchain RPC provider endpoint.", long, short = 'r')]
    provider_url: Option<String>,
}

impl Default for NetworkProviderArgs {
    fn default() -> Self {
        Self {
            network: "anvil-localhost".into(),
            contracts_root: None,
            provider_url: None,
        }
    }
}

impl NetworkProviderArgs {
    /// Get the NetworkDetail (contract addresses, environment type) from network names
    pub fn get_network_details_from_name(&self) -> Result<NetworkDetail, HelperErrors> {
        // read `contracts-addresses.json` at make_root_dir_path
        let contract_environment_config_path = self
            .contracts_root
            .as_ref()
            .map_or_else(|| std::env::current_dir().unwrap(), |p| PathBuf::from(OsStr::new(&p)))
            .to_owned()
            .join("contracts-addresses.json");

        let file_read =
            std::fs::read_to_string(contract_environment_config_path).map_err(HelperErrors::UnableToReadFromPath)?;

        let network_config = serde_json::from_str::<NetworkConfig>(&file_read).map_err(HelperErrors::SerdeJson)?;

        network_config
            .networks
            .get(&self.network)
            .cloned()
            .ok_or_else(|| HelperErrors::UnknownNetwork)
    }

    /// get default provider according to environment type of a network
    pub fn get_default_provider_url(&self) -> Result<String, HelperErrors> {
        let network_detail = self.get_network_details_from_name()?.environment_type;

        let default_provider_url = match network_detail {
            EnvironmentType::Production => "https://gnosis-provider.rpch.tech/",
            EnvironmentType::Staging => "https://gnosis-provider.rpch.tech/",
            EnvironmentType::Development => "http://127.0.0.1:8545",
            EnvironmentType::Local => "http://127.0.0.1:8545",
        };

        Ok(default_provider_url.into())
    }

    /// get the provider object
    pub async fn get_provider(&self) -> Result<Arc<Provider<JsonRpcClient>>, HelperErrors> {
        // default values
        let default_rpc_http_config = chain_rpc::client::native::HttpPostRequestorConfig::default();
        let default_rpc_http_retry_policy = SimpleJsonRpcRetryPolicy::default();

        // Build default JSON RPC client
        let default_rpc_client = JsonRpcClient::new(
            &self.get_default_provider_url()?,
            DefaultHttpPostRequestor::new(default_rpc_http_config.clone()),
            default_rpc_http_retry_policy.clone(),
        );
        // Build default JSON RPC provider
        let default_provider =
            Arc::new(Provider::new(default_rpc_client).interval(RpcOperationsConfig::default().tx_polling_interval));

        // validate that the rpc client connects to the expected chain
        let chain_id = default_provider.get_chainid().await.map_err(RpcError::ProviderError)?;

        // if a customized provider is given
        if let Some(customized_provider_url) = &self.provider_url {
            // check if the provided url matches with the network
            let customized_proivder_client = JsonRpcClient::new(
                customized_provider_url,
                DefaultHttpPostRequestor::new(default_rpc_http_config),
                default_rpc_http_retry_policy,
            );
            // Build default JSON RPC provider
            let customized_proivder = Arc::new(
                Provider::new(customized_proivder_client).interval(RpcOperationsConfig::default().tx_polling_interval),
            );

            let customized_chain_id = customized_proivder
                .get_chainid()
                .await
                .map_err(RpcError::ProviderError)?;
            if customized_chain_id.eq(&chain_id) {
                return Ok(customized_proivder);
            }
        }
        Ok(default_provider)
    }

    /// get the provider object
    pub async fn get_provider_with_signer(
        &self,
        chain_key: &ChainKeypair,
    ) -> Result<Arc<NonceManagerMiddleware<SignerMiddleware<Provider<JsonRpcClient>, Wallet<SigningKey>>>>, HelperErrors>
    {
        let default_provider_url = &self.get_default_provider_url()?;
        let rpc_provider_url = self.provider_url.as_ref().unwrap_or(default_provider_url);
        // Build JSON RPC client
        let rpc_client = JsonRpcClient::new(
            rpc_provider_url,
            DefaultHttpPostRequestor::new(chain_rpc::client::native::HttpPostRequestorConfig::default()),
            SimpleJsonRpcRetryPolicy::default(),
        );
        // Build default JSON RPC provider
        let provider = Provider::new(rpc_client).interval(RpcOperationsConfig::default().tx_polling_interval);

        let chain_id = provider.get_chainid().await.map_err(RpcError::ProviderError)?;

        let wallet = LocalWallet::from_bytes(chain_key.secret().as_ref())?.with_chain_id(chain_id.as_u64());

        Ok(Arc::new(
            provider
                .with_signer(wallet)
                .nonce_manager(chain_key.public().to_address().into()),
        ))
    }
}

/// ensures that the network and environment_type exist
/// in `contracts-addresses.json` and are matched
pub fn ensure_environment_and_network_are_set(
    make_root_dir_path: &Path,
    network: &str,
    environment_type: &str,
) -> Result<bool, HelperErrors> {
    let network_detail = get_network_details_from_name(make_root_dir_path, network)?;

    if network_detail.environment_type.to_string() == environment_type {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Returns the environment type from the network name
/// according to `contracts-addresses.json`
pub fn get_environment_type_from_name(
    make_root_dir_path: &Path,
    network: &str,
) -> Result<EnvironmentType, HelperErrors> {
    let network_detail = get_network_details_from_name(make_root_dir_path, network)?;
    Ok(network_detail.environment_type)
}

/// Get the NetworkDetail (contract addresses, environment type) from network names
pub fn get_network_details_from_name(make_root_dir_path: &Path, network: &str) -> Result<NetworkDetail, HelperErrors> {
    // read `contracts-addresses.json` at make_root_dir_path
    let contract_environment_config_path = make_root_dir_path.join("contracts-addresses.json");

    let file_read =
        std::fs::read_to_string(contract_environment_config_path).map_err(HelperErrors::UnableToReadFromPath)?;

    let network_config = serde_json::from_str::<NetworkConfig>(&file_read).map_err(HelperErrors::SerdeJson)?;

    network_config
        .networks
        .get(network)
        .cloned()
        .ok_or_else(|| HelperErrors::UnknownNetwork)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_anvil_at_port(default: bool) -> ethers::utils::AnvilInstance {
        let output = std::process::Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;
        let cargo_path = std::path::Path::new(std::str::from_utf8(&output).unwrap().trim());
        let workspace_dir = cargo_path.parent().unwrap().to_path_buf();

        let mut anvil = ethers::utils::Anvil::new().path(workspace_dir.join(".foundry/bin/anvil"));

        if !default {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let random_port = listener.local_addr().unwrap().port();
            anvil = anvil.port(random_port);
            anvil = anvil.chain_id(random_port);
        } else {
            anvil = anvil.port(8545u16);
        }
        anvil.spawn()
    }

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
            Ok(result) => assert!(result),
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
        assert!(result.unwrap().is_err());
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
        assert!(result.unwrap().is_err());
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
            Ok(result) => assert!(!result),
            _ => assert!(false),
        }
    }

    #[async_std::test]
    async fn use_default_rpc_url_for_anvil_when_no_provide_is_given() {
        let anvil = create_anvil_at_port(true);

        let network_provider_args = NetworkProviderArgs {
            network: "anvil-localhost".into(),
            contracts_root: Some("../ethereum/contracts".into()),
            provider_url: None,
        };

        let provider = network_provider_args.get_provider().await.unwrap();

        let chain_id = provider.get_chainid().await.unwrap();
        assert_eq!(chain_id, anvil.chain_id().into());
    }

    #[async_std::test]
    async fn test_network_provider_with_signer() {
        // create an identity
        let chain_key = ChainKeypair::random();

        // launch local anvil instance
        let anvil = create_anvil_at_port(false);

        let network_provider_args = NetworkProviderArgs {
            network: "anvil-localhost".into(),
            contracts_root: Some("../ethereum/contracts".into()),
            provider_url: anvil.endpoint().into(),
        };

        let provider = network_provider_args
            .get_provider_with_signer(&chain_key)
            .await
            .unwrap();

        let chain_id = provider.get_chainid().await.unwrap();
        assert_eq!(chain_id, anvil.chain_id().into());
    }
}
