use std::{str::FromStr, sync::Arc};

use async_std::sync::RwLock;
use core_ethereum_actions::{transaction_queue::TransactionQueue, CoreEthereumActions};
use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
use core_path::channel_graph::ChannelGraph;
use core_transport::ChainKeypair;
use futures::channel::mpsc::UnboundedSender;
use utils_db::rusty::RustyLevelDbShim;
use utils_types::primitives::Address;

use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use {core_ethereum_actions::transaction_queue::wasm::WasmTxExecutor, wasm_bindgen::prelude::*};
#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all(deserialize = "lowercase"))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum EnvironmentType {
    Production,
    Staging,
    Development,
    Local,
}

impl ToString for EnvironmentType {
    fn to_string(&self) -> String {
        match self {
            Self::Production => "production".into(),
            Self::Staging => "staging".into(),
            Self::Development => "development".into(),
            Self::Local => "local".into(),
        }
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
    // TODO: why would this be needed in the non-wasm setup?
    /// number of follow-on blocks required until a block is considered confirmed on-chain
    pub confirmations: u32,
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

#[derive(Serialize, Clone)]
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

#[cfg(feature = "wasm")]
pub fn build_chain_components<Db>(
    me: Address,
    db: Arc<RwLock<Db>>,
    tx_executor: WasmTxExecutor,
) -> (TransactionQueue<Db>, CoreEthereumActions<Db>)
where
    Db: HoprCoreEthereumDbActions + Clone + 'static,
{
    let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_executor));

    let chain_actions = CoreEthereumActions::new(me, db, tx_queue.new_sender());

    (tx_queue, chain_actions)
}

pub fn build_chain_api(
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>,
    chain_actions: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>,
    channel_updates: UnboundedSender<core_types::channels::ChannelEntry>,
    channel_graph: Arc<RwLock<ChannelGraph>>,
) -> core_ethereum_api::HoprChain {
    core_ethereum_api::HoprChain::new(
        me_onchain,
        db,
        chain_actions,
        core_ethereum_api::ChannelEventEmitter { tx: channel_updates },
        channel_graph,
    )
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::SmartContractConfig;
    use serde::Deserialize;
    use serde::Serialize;
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen(getter_with_clone)]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChainConfiguration {
        pub chain: String,
        #[wasm_bindgen(js_name = "hoprAnnouncementsAddress")]
        pub hopr_announcements_address: String,
        #[wasm_bindgen(js_name = "hoprTokenAddress")]
        pub hopr_token_address: String,
        #[wasm_bindgen(js_name = "hoprChannelsAddress")]
        pub hopr_channels_address: String,
        #[wasm_bindgen(js_name = "hoprNetworkRegistryAddress")]
        pub hopr_network_registry_address: String,
        #[wasm_bindgen(js_name = "hoprNodeSafeRegistryAddress")]
        pub hopr_node_safe_registry_address: String,
        #[wasm_bindgen(js_name = "hoprTicketPriceOracleAddress")]
        pub hopr_ticket_price_oracle_address: String,
        #[wasm_bindgen(js_name = "moduleAddress")]
        pub module_address: String,
        #[wasm_bindgen(js_name = "safeAddress")]
        pub safe_address: String,
        #[wasm_bindgen(js_name = "noticePeriodChannelClosure")]
        pub notice_period_channel_closure: u32,
    }

    #[wasm_bindgen]
    pub fn get_contract_data(
        network_id: String,
        custom_provider: Option<String>,
    ) -> Result<SmartContractConfig, JsError> {
        let resolved_environment =
            super::ChainNetworkConfig::new(&network_id, custom_provider.as_ref().map(|c| c.as_str()))
                .map_err(|e| JsError::new(e.as_str()))?;

        Ok(SmartContractConfig::from(&resolved_environment))
    }

    #[wasm_bindgen]
    pub fn resolve_network(id: &str, maybe_custom_provider: Option<String>) -> JsResult<JsValue> {
        let resolved_environment =
            super::ChainNetworkConfig::new(id, maybe_custom_provider.as_ref().map(|c| c.as_str()))?;

        ok_or_jserr!(serde_wasm_bindgen::to_value(&resolved_environment))
    }

    #[wasm_bindgen]
    extern "C" {
        /// EventEmitter object used to delegate `on` calls in WSS
        pub type WasmChainQuery;

        #[wasm_bindgen(method)]
        pub fn clone(this: &WasmChainQuery) -> WasmChainQuery;

        #[wasm_bindgen(method, catch)]
        pub async fn startChainSync(this: &WasmChainQuery) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn canRegisterWithSafe(this: &WasmChainQuery) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn waitForFunds(this: &WasmChainQuery) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn isNodeSafeNotRegistered(this: &WasmChainQuery) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn getBalance(this: &WasmChainQuery) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn getNativeBalance(this: &WasmChainQuery) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn getSafeBalance(this: &WasmChainQuery) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn getSafeNativeBalance(this: &WasmChainQuery) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub fn smartContractInfo(this: &WasmChainQuery) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method)]
        pub fn on(this: &WasmChainQuery, event: js_sys::JsString, callback: js_sys::Function);

        #[wasm_bindgen(method)]
        pub fn emit(this: &WasmChainQuery, event: js_sys::JsString, callback: js_sys::Function);
    }
}
