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
//! the deployed contract addresses in [ContractAddresses]
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, sync::Arc};

use clap::Parser;
use hopr_bindings::exports::alloy::{
    network::EthereumWallet,
    providers::{
        Identity, ProviderBuilder, RootProvider,
        fillers::{
            BlobGasFiller, CachedNonceManager, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
    },
    rpc::client::ClientBuilder,
    signers::local::PrivateKeySigner,
    transports::http::ReqwestTransport,
};
use hopr_chain_types::ContractAddresses;
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

use crate::utils::HelperErrors;

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

type SharedFillerChain = JoinFill<
    JoinFill<JoinFill<JoinFill<Identity, ChainIdFiller>, NonceFiller<CachedNonceManager>>, GasFiller>,
    BlobGasFiller,
>;
pub type RpcProvider = FillProvider<JoinFill<SharedFillerChain, WalletFiller<EthereumWallet>>, RootProvider>;
pub type RpcProviderWithoutSigner = FillProvider<SharedFillerChain, RootProvider>;

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
    pub addresses: ContractAddresses,
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

    /// Path to the root of foundry project (ethereum/contracts), where all the contracts and
    /// `contracts-addresses.json` are stored Default to "./ethereum/contracts", which is the path to the
    /// `contracts` folder from the root of monorepo
    #[clap(
        env = "HOPLI_CONTRACTS_ROOT",
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = "./ethereum/contracts"
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
            contracts_root: Some("./ethereum/contracts".into()),
            provider_url: "http://127.0.0.1:8545".into(),
        }
    }
}

impl NetworkProviderArgs {
    /// Get the NetworkDetail (contract addresses, environment type) from network names
    pub fn get_network_details_from_name(&self) -> Result<NetworkDetail, HelperErrors> {
        // read `contracts-addresses.json` at make_root_dir_path
        let contract_root = self.contracts_root.to_owned().unwrap_or(
            NetworkProviderArgs::default()
                .contracts_root
                .ok_or(HelperErrors::UnableToSetFoundryRoot)?,
        );
        let contract_environment_config_path =
            PathBuf::from(OsStr::new(&contract_root)).join("contracts-addresses.json");

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
    pub async fn get_provider_with_signer(&self, chain_key: &ChainKeypair) -> Result<Arc<RpcProvider>, HelperErrors>
// ) -> Result<Arc<NonceManagerMiddleware<SignerMiddleware<Provider<JsonRpcClient>, Wallet<SigningKey>>>>, HelperErrors>
    {
        // Build transport
        let parsed_url = url::Url::parse(self.provider_url.as_str()).unwrap();
        let transport_client = ReqwestTransport::new(parsed_url);

        // Build JSON RPC client
        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        if rpc_client.is_local() {
            rpc_client.set_poll_interval(std::time::Duration::from_millis(10));
        };

        // build wallet
        let wallet = PrivateKeySigner::from_slice(chain_key.secret().as_ref()).expect("failed to construct wallet");

        // Build default JSON RPC provider
        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            .filler(ChainIdFiller::default())
            .filler(NonceFiller::new(CachedNonceManager::default()))
            .filler(GasFiller)
            .filler(BlobGasFiller)
            .wallet(wallet)
            .connect_client(rpc_client);

        Ok(Arc::new(provider))
    }

    /// get the provider object without signer
    pub async fn get_provider_without_signer(&self) -> Result<Arc<RpcProviderWithoutSigner>, HelperErrors> {
        // Build transport
        let parsed_url = url::Url::parse(self.provider_url.as_str()).unwrap();
        let transport_client = ReqwestTransport::new(parsed_url);

        // Build JSON RPC client
        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        if rpc_client.is_local() {
            rpc_client.set_poll_interval(std::time::Duration::from_millis(10));
        };

        // Build default JSON RPC provider
        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            // .wallet(wallet)
            .filler(ChainIdFiller::default())
            .filler(NonceFiller::new(CachedNonceManager::default()))
            .filler(GasFiller)
            .filler(BlobGasFiller)
            .connect_client(rpc_client);

        Ok(Arc::new(provider))
    }
}

/// ensures that the network and environment_type exist
/// in `contracts-addresses.json` and are matched
pub fn ensure_environment_and_network_are_set(network: &str, environment_type: &str) -> Result<bool, HelperErrors> {
    let network_detail = hopr_bindings::config::NetworksWithContractAddresses::default()
        .networks
        .get(network)
        .cloned()
        .ok_or_else(|| HelperErrors::UnknownNetwork)?;

    Ok(network_detail.environment_type.to_string() == environment_type)
}

#[cfg(test)]
mod tests {
    use hopr_bindings::exports::alloy::{
        node_bindings::{Anvil, AnvilInstance},
        providers::Provider,
    };

    use super::*;

    fn create_anvil_at_port(default: bool) -> AnvilInstance {
        let mut anvil = Anvil::new();

        if !default {
            let listener =
                std::net::TcpListener::bind("127.0.0.1:0").unwrap_or_else(|_| panic!("Failed to bind localhost"));
            let random_port = listener
                .local_addr()
                .unwrap_or_else(|_| panic!("Failed to get local address"))
                .port();
            anvil = anvil.port(random_port);
            anvil = anvil.chain_id(random_port.into());
        } else {
            anvil = anvil.port(8545u16);
        }
        anvil.spawn()
    }

    #[test]
    fn read_anvil_localhost_at_right_path() -> anyhow::Result<()> {
        let network = "anvil-localhost";
        let environment_type = "local";
        assert!(ensure_environment_and_network_are_set(network, environment_type)?);
        Ok(())
    }

    #[test]
    fn read_non_existing_environment_at_right_path() -> anyhow::Result<()> {
        assert!(ensure_environment_and_network_are_set("non-existing", "development").is_err());
        Ok(())
    }

    #[test]
    fn read_wrong_type_at_right_path() -> anyhow::Result<()> {
        let network = "anvil-localhost";
        let environment_type = "production";
        assert!(!ensure_environment_and_network_are_set(network, environment_type)?);
        Ok(())
    }

    #[tokio::test]
    async fn test_network_provider_with_signer() -> anyhow::Result<()> {
        // create an identity
        let chain_key = ChainKeypair::random();

        // launch local anvil instance
        let anvil = create_anvil_at_port(false);

        let network_provider_args = NetworkProviderArgs {
            network: "anvil-localhost".into(),
            contracts_root: Some("../ethereum/contracts".into()),
            provider_url: anvil.endpoint(),
        };

        let provider = network_provider_args.get_provider_with_signer(&chain_key).await?;

        let chain_id = provider.get_chain_id().await?;
        assert_eq!(chain_id, anvil.chain_id());
        Ok(())
    }

    #[tokio::test]
    async fn test_default_contracts_root() -> anyhow::Result<()> {
        // create an identity
        let chain_key = ChainKeypair::random();

        // launch local anvil instance
        let anvil = create_anvil_at_port(false);

        let network_provider_args = NetworkProviderArgs {
            network: "anvil-localhost".into(),
            contracts_root: None,
            provider_url: anvil.endpoint(),
        };

        let provider = network_provider_args.get_provider_with_signer(&chain_key).await?;

        let chain_id = provider.get_chain_id().await?;
        assert_eq!(chain_id, anvil.chain_id());
        Ok(())
    }
}
