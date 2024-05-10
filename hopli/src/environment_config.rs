//! This module contains definiation of arguments that specify the environment
//! and networks that a HOPR node runs in.
//!
//! [EnvironmentType] defines the environment type. EnvironmentType of a network is defined in
//! `contracts-addresses.json` under the foundry contract root. Different environment type uses
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
use hopr_lib::{EnvironmentType, NetworkContractAddresses};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

// replace NetworkConfig with ProtocolConfig
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NetworkDetail {
    /// block number to start the indexer from
    pub indexer_start_block_number: u32,
    /// Type of environment
    #[serde_as(as = "DisplayFromStr")]
    pub environment_type: EnvironmentType,
    /// contract addresses used by the network
    pub addresses: NetworkContractAddresses,
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
        env = "HOPLI_CONTRACTS_ROOT",
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    /// Customized RPC provider endpoint
    #[clap(help = "Blockchain RPC provider endpoint.", long, short = 'r')]
    provider_url: String,
}

impl Default for NetworkProviderArgs {
    fn default() -> Self {
        Self {
            network: "anvil-localhost".into(),
            contracts_root: None,
            provider_url: "http://127.0.0.1:8545".into(),
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

    /// get the provider object
    pub async fn get_provider_with_signer(
        &self,
        chain_key: &ChainKeypair,
    ) -> Result<Arc<NonceManagerMiddleware<SignerMiddleware<Provider<JsonRpcClient>, Wallet<SigningKey>>>>, HelperErrors>
    {
        // Build JSON RPC client
        let rpc_client = JsonRpcClient::new(
            self.provider_url.as_str(),
            DefaultHttpPostRequestor::new(chain_rpc::client::native::HttpPostRequestorConfig::default()),
            SimpleJsonRpcRetryPolicy::default(),
        );

        // Build default JSON RPC provider
        let mut provider = Provider::new(rpc_client);

        let chain_id = provider.get_chainid().await.map_err(RpcError::ProviderError)?;
        let default_tx_polling_interval = if chain_id.eq(&ethers::types::U256::from(31337u32)) {
            std::time::Duration::from_millis(10)
        } else {
            RpcOperationsConfig::default().tx_polling_interval
        };
        provider.set_interval(default_tx_polling_interval);

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
        let mut anvil = ethers::utils::Anvil::new();

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
