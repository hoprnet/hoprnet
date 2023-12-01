pub mod errors;
pub mod executors;

pub use core_ethereum_types::chain_events::SignificantChainEvent;
pub use core_types::channels::ChannelEntry;

use async_lock::RwLock;
use core_ethereum_db::db::CoreEthereumDb;
use std::sync::Arc;

use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_ethereum_actions::CoreEthereumActions;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_indexer::block::{Indexer, IndexerConfig};
use core_ethereum_indexer::handlers::ContractEventHandlers;
use core_ethereum_rpc::rpc::RpcOperations;
use core_ethereum_rpc::HoprRpcOperations;
use core_ethereum_types::ContractAddresses;
use core_types::account::AccountEntry;
use utils_db::rusty::RustyLevelDbShim;
use utils_log::info;
use utils_types::primitives::{Address, Balance};

use crate::errors::{HoprChainError, Result};

#[cfg(feature = "wasm")]
pub type JsonRpcClient =
    core_ethereum_rpc::client::JsonRpcProviderClient<core_ethereum_rpc::nodejs::NodeJsHttpPostRequestor>;

// Alternatively, core_ethereum_rpc::client::JsonRpcProviderClient<ReqwestHttpPostRequestor>
// if more customizability is needed.
#[cfg(not(feature = "wasm"))]
pub type JsonRpcClient = ethers::providers::Http;

#[cfg(feature = "wasm")]
pub fn build_json_rpc_client(base_url: &str) -> JsonRpcClient {
    core_ethereum_rpc::client::JsonRpcProviderClient::new(
        base_url,
        core_ethereum_rpc::nodejs::NodeJsHttpPostRequestor::default(),
    )
}

#[cfg(not(feature = "wasm"))]
pub fn build_json_rpc_client(base_url: &str) -> JsonRpcClient {
    use std::str::FromStr;
    ethers::providers::Http::from_str(base_url).expect("invalid provider URL")
}

pub async fn can_register_with_safe<Rpc: HoprRpcOperations>(
    me: Address,
    safe_address: Address,
    rpc: &Rpc,
) -> Result<bool> {
    let target_address = rpc.get_module_target_address().await?;
    if target_address != safe_address {
        // cannot proceed when the safe address is not the target/owner of given module
        return Err(HoprChainError::Api("safe is not a target of module".into()));
    }

    let registered_address = rpc.get_safe_from_node_safe_registry(me).await?;
    info!("currently registered Safe address in NodeSafeRegistry = {registered_address}");

    if registered_address.is_zero() {
        info!("Node is not associated with a Safe in NodeSafeRegistry yet");
        Ok(true)
    } else if registered_address != safe_address {
        Err(HoprChainError::Api(
            "Node is associated with a different Safe in NodeSafeRegistry".into(),
        ))
    } else {
        info!("Node is associated with correct Safe in NodeSafeRegistry");
        Ok(false)
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HoprChain {
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>>,
    indexer: Indexer<
        RpcOperations<JsonRpcClient>,
        ContractEventHandlers<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>,
        CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>,
    >,
    chain_actions: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>,
    rpc_operations: RpcOperations<JsonRpcClient>,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
}

impl HoprChain {
    pub fn new(
        me_onchain: ChainKeypair,
        db: Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>,
        contract_addresses: ContractAddresses,
        safe_address: Address,
        indexer_events_tx: futures::channel::mpsc::UnboundedSender<SignificantChainEvent>,
        chain_actions: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>,
        rpc_operations: RpcOperations<JsonRpcClient>,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    ) -> Self {
        let db_processor =
            ContractEventHandlers::new(contract_addresses, safe_address, (&me_onchain).into(), db.clone());
        let indexer = Indexer::new(
            rpc_operations.clone(),
            db_processor,
            db.clone(),
            IndexerConfig::default(), // TODO: pass down indexer configuration
            indexer_events_tx,
        );
        Self {
            me_onchain,
            db,
            indexer,
            chain_actions,
            rpc_operations,
            channel_graph,
        }
    }

    pub async fn sync_chain(&mut self) -> errors::Result<()> {
        Ok(self.indexer.start().await?)
    }

    pub fn me_onchain(&self) -> Address {
        self.me_onchain.public().to_address()
    }

    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self.db.read().await.get_accounts().await?)
    }

    pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<ChannelEntry> {
        self.db
            .read()
            .await
            .get_channel_x(src, dest)
            .await
            .map_err(HoprChainError::from)
            .and_then(|v| {
                v.ok_or(errors::HoprChainError::Api(format!(
                    "Channel entry not available {}-{}",
                    src, dest
                )))
            })
    }

    pub async fn channels_from(&self, src: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.read().await.get_channels_from(src).await?)
    }

    pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.read().await.get_channels_to(dest).await?)
    }

    pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.read().await.get_channels().await?)
    }

    pub async fn safe_allowance(&self) -> errors::Result<Balance> {
        Ok(self.db.read().await.get_staking_safe_allowance().await?)
    }

    pub fn actions_ref(&self) -> &CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>> {
        &self.chain_actions
    }

    pub fn actions_mut_ref(&mut self) -> &mut CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>> {
        &mut self.chain_actions
    }

    // NOTE: needed early in the initialization to sync
    pub fn channel_graph(&self) -> Arc<RwLock<core_path::channel_graph::ChannelGraph>> {
        self.channel_graph.clone()
    }

    // NOTE: needed early in the initialization to sync
    pub fn db(&self) -> Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>> {
        self.db.clone()
    }

    pub fn rpc(&self) -> &RpcOperations<JsonRpcClient> {
        &self.rpc_operations
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::wasm::JsLogger;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn core_ethereum_api_initialize_crate() {
        let _ = JsLogger::install(&LOGGER, None);

        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    #[wasm_bindgen]
    pub fn core_ethereum_api_gather_metrics() -> JsResult<String> {
        utils_metrics::metrics::wasm::gather_all_metrics()
    }
}
