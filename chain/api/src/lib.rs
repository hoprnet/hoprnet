//! Crate containing the API object for chain operations used by the HOPRd node.

pub mod config;
pub mod errors;
pub mod executors;

use std::{sync::Arc, time::Duration};

use async_lock::RwLock;
use async_std::task::sleep;
use tracing::{debug, error, info, warn};

use chain_actions::action_queue::{ActionQueue, ActionQueueConfig};
use chain_actions::action_state::IndexerActionTracker;
use chain_actions::payload::SafePayloadGenerator;
use chain_actions::ChainActions;
use chain_indexer::{block::Indexer, handlers::ContractEventHandlers, IndexerConfig};
use chain_rpc::client::SimpleJsonRpcRetryPolicy;
use chain_rpc::rpc::RpcOperationsConfig;
use chain_rpc::HoprRpcOperations;
use chain_rpc::{rpc::RpcOperations, TypedTransaction};
pub use chain_types::chain_events::SignificantChainEvent;
use chain_types::ContractAddresses;
use config::ChainNetworkConfig;
use executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig};
use hopr_crypto_types::prelude::*;
use hopr_db_api::HoprDbAllOperations;
use hopr_internal_types::account::AccountEntry;
pub use hopr_internal_types::channels::ChannelEntry;
use hopr_internal_types::prelude::ChannelDirection;
use hopr_primitive_types::prelude::*;

use crate::errors::{HoprChainError, Result};

/// The default HTTP request engine
///
/// TODO: Should be an internal type, `hopr_lib::chain` must be moved to this package
pub type DefaultHttpPostRequestor = chain_rpc::client::native::SurfRequestor;

/// The default JSON RPC provider client
///
/// TODO: Should be an internal type, `hopr_lib::chain` must be moved to this package
pub type JsonRpcClient = chain_rpc::client::JsonRpcProviderClient<DefaultHttpPostRequestor, SimpleJsonRpcRetryPolicy>;

/// Checks whether the node can be registered with the Safe in the NodeSafeRegistry
pub async fn can_register_with_safe<Rpc: HoprRpcOperations>(
    me: Address,
    safe_address: Address,
    rpc: &Rpc,
) -> Result<bool> {
    let target_address = rpc.get_module_target_address().await?;
    debug!("-- node address: {me}");
    debug!("-- safe address: {safe_address}");
    debug!("-- module target address: {target_address}");

    if target_address != safe_address {
        // cannot proceed when the safe address is not the target/owner of given module
        return Err(HoprChainError::Api("safe is not the module target".into()));
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

/// Waits until the given address is funded.
///
/// This is done by querying the RPC provider for balance with backoff until `max_delay` argument.
pub async fn wait_for_funds<Rpc: HoprRpcOperations>(
    address: Address,
    min_balance: Balance,
    max_delay: Duration,
    rpc: &Rpc,
) -> Result<()> {
    let multiplier = 1.05;
    let mut current_delay = Duration::from_secs(2).min(max_delay);

    while current_delay <= max_delay {
        match rpc.get_balance(address, min_balance.balance_type()).await {
            Ok(current_balance) => {
                info!("current balance is {}", current_balance.to_formatted_string());
                if current_balance.ge(&min_balance) {
                    info!("node is funded");
                    return Ok(());
                } else {
                    warn!("still unfunded, trying again soon");
                }
            }
            Err(e) => error!("failed to fetch balance from the chain: {e}"),
        }

        sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(HoprChainError::Api("timeout waiting for funds".into()))
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
    db: Db,
) -> (
    ActionQueue<Db, IndexerActionTracker, ActiveTxExecutor>,
    ChainActions<Db>,
    RpcOperations<JsonRpcClient>,
)
where
    Db: HoprDbAllOperations + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    // TODO: extract this from the global config type
    let rpc_http_config = chain_rpc::client::native::HttpPostRequestorConfig::default();

    // TODO: extract this from the global config type
    let rpc_http_retry_policy = SimpleJsonRpcRetryPolicy {
        min_retries: Some(2),
        ..SimpleJsonRpcRetryPolicy::default()
    };

    // TODO: extract this from the global config type
    let rpc_cfg = RpcOperationsConfig {
        chain_id: chain_config.chain.chain_id as u64,
        contract_addrs,
        module_address,
        expected_block_time: Duration::from_millis(chain_config.chain.block_time),
        tx_polling_interval: Duration::from_millis(chain_config.tx_polling_interval),
        finality: chain_config.confirmations,
        max_block_range_fetch_size: chain_config.max_block_range,
    };

    // TODO: extract this from the global config type
    let rpc_client_cfg = RpcEthereumClientConfig::default();

    // TODO: extract this from the global config type
    let action_queue_cfg = ActionQueueConfig::default();

    // --- Configs done ---

    // Build JSON RPC client
    let rpc_client = JsonRpcClient::new(
        &chain_config.chain.default_provider,
        DefaultHttpPostRequestor::new(rpc_http_config),
        rpc_http_retry_policy,
    );

    // Build RPC operations
    let rpc_operations = RpcOperations::new(rpc_client, me_onchain, rpc_cfg).expect("failed to initialize RPC");

    // Build the Ethereum Transaction Executor that uses RpcOperations as backend
    let ethereum_tx_executor = EthereumTransactionExecutor::new(
        RpcEthereumClient::new(rpc_operations.clone(), rpc_client_cfg),
        SafePayloadGenerator::new(me_onchain, contract_addrs, module_address),
    );

    // Build the Action Queue
    let action_queue = ActionQueue::new(
        db.clone(),
        IndexerActionTracker::default(),
        ethereum_tx_executor,
        action_queue_cfg,
    );

    // Instantiate Chain Actions
    let chain_actions = ChainActions::new(me_onchain.public().to_address(), db, action_queue.new_sender());

    (action_queue, chain_actions, rpc_operations)
}

/// Repsents all chain interactions exported to be used in the hopr-lib
///
/// NOTE: instead of creating a unified interface the [HoprChain] exports
/// some functionality (e.g. the [ChainActions] as a referentially used)
/// object. This behavior will be refactored and hidden behind a trait
/// in the future implementations.
#[derive(Debug, Clone)]
pub struct HoprChain<T: HoprDbAllOperations + Send + Sync + Clone + std::fmt::Debug> {
    me_onchain: ChainKeypair,
    safe_address: Address,
    contract_addresses: ContractAddresses,
    indexer_cfg: IndexerConfig,
    indexer_events_tx: futures::channel::mpsc::UnboundedSender<SignificantChainEvent>,
    db: T,
    chain_actions: ChainActions<T>,
    rpc_operations: RpcOperations<JsonRpcClient>,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
}

impl<T: HoprDbAllOperations + Send + Sync + Clone + std::fmt::Debug + 'static> HoprChain<T> {
    #[allow(clippy::too_many_arguments)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
    pub fn new(
        me_onchain: ChainKeypair,
        db: T,
        contract_addresses: ContractAddresses,
        safe_address: Address,
        indexer_cfg: IndexerConfig,
        indexer_events_tx: futures::channel::mpsc::UnboundedSender<SignificantChainEvent>,
        chain_actions: ChainActions<T>,
        rpc_operations: RpcOperations<JsonRpcClient>,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    ) -> Self {
        Self {
            me_onchain,
            safe_address,
            contract_addresses,
            indexer_cfg,
            indexer_events_tx,
            db,
            chain_actions,
            rpc_operations,
            channel_graph,
        }
    }

    pub async fn sync_chain(&self) -> errors::Result<async_std::task::JoinHandle<()>> {
        let db_processor = ContractEventHandlers::new(
            self.contract_addresses,
            self.safe_address,
            self.me_onchain.clone(),
            self.db.clone(),
        );

        let mut indexer = Indexer::new(
            self.rpc_operations.clone(),
            db_processor,
            self.db.clone(),
            self.indexer_cfg,
            self.indexer_events_tx.clone(),
        );

        Ok(indexer.start().await?)
    }

    pub fn me_onchain(&self) -> Address {
        self.me_onchain.public().to_address()
    }

    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self.db.get_accounts(None, true).await?)
    }

    pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<ChannelEntry> {
        self.db
            .get_channel_by_parties(None, src, dest)
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
        Ok(self.db.get_channels_via(None, ChannelDirection::Outgoing, src).await?)
    }

    pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.get_channels_via(None, ChannelDirection::Incoming, dest).await?)
    }

    pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.get_all_channels(None).await?)
    }

    pub async fn ticket_price(&self) -> errors::Result<Option<U256>> {
        Ok(self.db.get_indexer_data(None).await?.ticket_price.map(|b| b.amount()))
    }

    pub async fn safe_allowance(&self) -> errors::Result<Balance> {
        Ok(self.db.get_safe_hopr_allowance(None).await?)
    }

    pub fn actions_ref(&self) -> &ChainActions<T> {
        &self.chain_actions
    }

    pub fn actions_mut_ref(&mut self) -> &mut ChainActions<T> {
        &mut self.chain_actions
    }

    // NOTE: needed early in the initialization to sync
    pub fn channel_graph(&self) -> Arc<RwLock<core_path::channel_graph::ChannelGraph>> {
        self.channel_graph.clone()
    }

    // NOTE: needed early in the initialization to sync
    pub fn db(&self) -> T {
        self.db.clone()
    }

    pub fn rpc(&self) -> &RpcOperations<JsonRpcClient> {
        &self.rpc_operations
    }

    pub async fn get_balance(&self, balance_type: BalanceType) -> errors::Result<Balance> {
        Ok(self.rpc_operations.get_balance(self.me_onchain(), balance_type).await?)
    }

    pub async fn get_safe_balance(&self, safe_address: Address, balance_type: BalanceType) -> errors::Result<Balance> {
        Ok(self.rpc_operations.get_balance(safe_address, balance_type).await?)
    }

    pub async fn get_channel_closure_notice_period(&self) -> errors::Result<Duration> {
        Ok(self.rpc_operations.get_channel_closure_notice_period().await?)
    }
}
