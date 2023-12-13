use std::fmt::Display;
use std::time::Duration;
use std::{str::FromStr, sync::Arc};

use async_std::sync::RwLock;
use core_ethereum_actions::{action_queue::ActionQueue, CoreEthereumActions};
use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
use core_path::channel_graph::ChannelGraph;
use core_transport::{ChainKeypair, Keypair};
use serde::{Deserialize, Serialize};
use utils_db::rusty::RustyLevelDbShim;
use utils_types::primitives::Address;

use core_ethereum_actions::action_queue::ActionQueueConfig;
use core_ethereum_actions::action_state::IndexerActionTracker;
use core_ethereum_actions::payload::SafePayloadGenerator;
use core_ethereum_api::executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig};
use core_ethereum_api::{DefaultHttpPostRequestor, JsonRpcClient};
use core_ethereum_rpc::client::SimpleJsonRpcRetryPolicy;
use core_ethereum_rpc::rpc::{RpcOperations, RpcOperationsConfig};
use core_ethereum_types::chain_events::SignificantChainEvent;
use core_ethereum_types::{ContractAddresses, TypedTransaction};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all(deserialize = "lowercase"))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum EnvironmentType {
    Production,
    Staging,
    Development,
    Local,
}

impl Display for EnvironmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Production => "production",
                Self::Staging => "staging",
                Self::Development => "development",
                Self::Local => "local",
            }
        )
    }
}

/// Holds all information we need about the blockchain network
/// the client is going to use
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[cfg(not(feature = "wasm"))]
pub struct ChainOptions {
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

// duplicate due to issue of wasm_bindgen with proc macros on struct properties
#[cfg(feature = "wasm")]
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[wasm_bindgen(getter_with_clone)]
pub struct ChainOptions {
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
    #[wasm_bindgen(skip)] // no tags in Typescript
    pub tags: Option<Vec<String>>,
}

/// Holds all information about the protocol network
/// to be used by the client
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[cfg(not(feature = "wasm"))]
pub struct Network {
    #[serde(skip_deserializing)]
    pub id: String,
    /// must match one of the Network.id
    pub chain: String,
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
}

// duplicate due to issue of wasm_bindgen with proc macros on struct properties
#[cfg(feature = "wasm")]
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[wasm_bindgen(getter_with_clone)]
pub struct Network {
    #[serde(skip_deserializing)]
    pub id: String,
    /// must match one of the Network.id
    pub chain: String,
    pub environment_type: EnvironmentType,
    /// Node.js-fashioned semver string
    pub version_range: String,
    /// block number to start the indexer from
    pub indexer_start_block_number: u32,
    /// contract addresses used by the network
    pub addresses: Addresses,
    /// Number of blockchain block to wait until an on-chain state-change is considered to be final
    ///
    /// Note that the probability that on-chain state changes will get pruned due to
    /// block reorganizations increases exponentially in the number of confirmations, e.g.
    /// after one block it is `0.5` whereas after two blocks it is `0.25 = 0.5^2`  etc.
    pub confirmations: u32,
    #[wasm_bindgen(skip)] // no tags in Typescript
    pub tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
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

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct ChainNetworkConfig {
    /// the network identifier, e.g. monte_rosa
    pub id: String,
    pub chain: ChainOptions,
    pub environment_type: EnvironmentType,
    pub channel_contract_deploy_block: u32,
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
    /// number of follow-on blocks required until a block is considered confirmed on-chain
    pub confirmations: u32,
}

impl ChainNetworkConfig {
    /// Returns the network details, returns an error if network is not supported
    pub fn new(id: &str, maybe_custom_provider: Option<&str>) -> Result<Self, String> {
        let mut protocol_config = ProtocolConfig::from_str(include_str!("../data/protocol-config.json"))?;

        let network = protocol_config
            .networks
            .get_mut(id)
            .ok_or(format!("Could not find network {} in protocol config", id))?;

        let chain = protocol_config
            .chains
            .get_mut(&network.chain)
            .ok_or(format!("Invalid chain {} for network {}", network.chain, id))?;

        if let Some(custom_provider) = maybe_custom_provider {
            chain.default_provider = custom_provider.into();
        }

        match real_base::real::satisfies(crate::constants::APP_VERSION_COERCED, network.version_range.as_str()) {
            Ok(true) => Ok(ChainNetworkConfig {
                announcements: network.addresses.announcements.to_owned(),
                chain: chain.to_owned(),
                channel_contract_deploy_block: network.indexer_start_block_number,
                channels: network.addresses.channels.to_owned(),
                confirmations: network.confirmations,
                environment_type: network.environment_type,
                id: id.into(),
                module_implementation: network.addresses.module_implementation.to_owned(),
                network_registry: network.addresses.network_registry.to_owned(),
                network_registry_proxy: network.addresses.network_registry_proxy.to_owned(),
                node_safe_registry: network.addresses.node_safe_registry.to_owned(),
                node_stake_v2_factory: network.addresses.node_stake_v2_factory.to_owned(),
                ticket_price_oracle: network.addresses.ticket_price_oracle.to_owned(),
                token: network.addresses.token.to_owned(),
            }),
            Ok(false) => Err(format!(
                "network {} is not supported, supported networks {:?}",
                id,
                protocol_config
                    .supported_networks()
                    .iter()
                    .map(|e| e.id.to_owned())
                    .collect::<Vec<String>>()
                    .join(", ")
            )),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct SmartContractConfig {
    pub hopr_announcements_address: String,
    pub hopr_token_address: String,
    pub hopr_channels_address: String,
    pub hopr_network_registry_address: String,
    pub hopr_node_safe_registry_address: String,
    pub hopr_ticket_price_oracle_address: String,
    pub indexer_start_block_number: u32,
}

impl From<&ChainNetworkConfig> for SmartContractConfig {
    fn from(network: &ChainNetworkConfig) -> Self {
        Self {
            hopr_announcements_address: network.announcements.clone(),
            hopr_token_address: network.token.clone(),
            hopr_channels_address: network.channels.clone(),
            hopr_network_registry_address: network.network_registry.clone(),
            hopr_node_safe_registry_address: network.node_safe_registry.clone(),
            hopr_ticket_price_oracle_address: network.ticket_price_oracle.clone(),
            indexer_start_block_number: network.channel_contract_deploy_block,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolConfig {
    pub networks: std::collections::HashMap<String, Network>,
    pub chains: std::collections::HashMap<String, ChainOptions>,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self::from_str(include_str!("../data/protocol-config.json"))
            .expect("bundled protocol config should be always valid")
    }
}

impl FromStr for ProtocolConfig {
    type Err = String;

    /// Reads the protocol config JSON file and returns it
    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let mut protocol_config =
            (serde_json::from_slice::<ProtocolConfig>(data.as_bytes())).map_err(|e| e.to_string())?;

        for (id, env) in protocol_config.networks.iter_mut() {
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
    pub fn supported_networks(&self) -> Vec<Network> {
        let mut allowed: Vec<Network> = vec![];

        for (_, env) in self.networks.iter() {
            let range = env.version_range.to_owned();

            if let Ok(true) = real_base::real::satisfies(crate::constants::APP_VERSION_COERCED, range.as_str()) {
                allowed.push(env.to_owned())
            }
        }

        allowed
    }
}

type ActiveTxExecutor = EthereumTransactionExecutor<
    TypedTransaction,
    RpcEthereumClient<RpcOperations<JsonRpcClient>>,
    SafePayloadGenerator,
>;

pub fn build_chain_components<Db>(
    me_onchain: &ChainKeypair,
    chain_config: ChainNetworkConfig,
    contract_addrs: ContractAddresses,
    module_address: Address,
    db: Arc<RwLock<Db>>,
) -> (
    ActionQueue<Db, IndexerActionTracker, ActiveTxExecutor>,
    CoreEthereumActions<Db>,
    RpcOperations<JsonRpcClient>,
)
where
    Db: HoprCoreEthereumDbActions + Clone + 'static,
{
    let rpc_client = JsonRpcClient::new(
        &chain_config.chain.default_provider,
        DefaultHttpPostRequestor::default(),
    );

    // TODO: extract these configs from the global config type
    let rpc_cfg = RpcOperationsConfig {
        chain_id: chain_config.chain.chain_id as u64,
        contract_addrs,
        module_address,
        max_http_retries: 10,
        expected_block_time: Duration::from_secs(7),
        ..RpcOperationsConfig::default()
    };
    let rpc_client_cfg = RpcEthereumClientConfig::default();
    let action_queue_cfg = ActionQueueConfig::default();

    let rpc_operations = RpcOperations::new(rpc_client, me_onchain, rpc_cfg, SimpleJsonRpcRetryPolicy)
        .expect("failed to initialize RPC");

    let ethereum_tx_executor = EthereumTransactionExecutor::new(
        RpcEthereumClient::new(rpc_operations.clone(), rpc_client_cfg),
        SafePayloadGenerator::new(me_onchain, contract_addrs, module_address),
    );

    let tx_queue = ActionQueue::new(
        db.clone(),
        IndexerActionTracker::default(),
        ethereum_tx_executor,
        action_queue_cfg,
    );

    let chain_actions = CoreEthereumActions::new(me_onchain.public().to_address(), db, tx_queue.new_sender());

    (tx_queue, chain_actions, rpc_operations)
}

pub fn build_chain_api(
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>,
    contract_addrs: ContractAddresses,
    safe_address: Address,
    indexer_start_block: u64,
    indexer_events_tx: futures::channel::mpsc::UnboundedSender<SignificantChainEvent>,
    chain_actions: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>,
    rpc_operations: RpcOperations<JsonRpcClient>,
    channel_graph: Arc<RwLock<ChannelGraph>>,
) -> core_ethereum_api::HoprChain {
    let indexer_cfg = core_ethereum_indexer::block::IndexerConfig {
        start_block_number: indexer_start_block,
        ..Default::default()
    };

    core_ethereum_api::HoprChain::new(
        me_onchain,
        db,
        contract_addrs,
        safe_address,
        indexer_cfg,
        indexer_events_tx,
        chain_actions,
        rpc_operations,
        channel_graph,
    )
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::SmartContractConfig;
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub fn get_contract_data(
        network_id: String,
        custom_provider: Option<String>,
    ) -> Result<SmartContractConfig, JsError> {
        let resolved_environment = super::ChainNetworkConfig::new(&network_id, custom_provider.as_deref())
            .map_err(|e| JsError::new(e.as_str()))?;

        Ok(SmartContractConfig::from(&resolved_environment))
    }

    #[wasm_bindgen]
    pub fn resolve_network(id: &str, maybe_custom_provider: Option<String>) -> JsResult<JsValue> {
        let resolved_environment = super::ChainNetworkConfig::new(id, maybe_custom_provider.as_deref())?;

        ok_or_jserr!(serde_wasm_bindgen::to_value(&resolved_environment))
    }
}
