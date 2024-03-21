//! Crate containing the API object for chain operations used by the HOPRd node.

pub mod errors;
pub mod executors;

pub use chain_types::chain_events::SignificantChainEvent;
pub use hopr_internal_types::channels::ChannelEntry;

use async_lock::RwLock;
use std::sync::Arc;
use std::time::Duration;

use chain_actions::ChainActions;
use chain_indexer::{block::Indexer, handlers::ContractEventHandlers, IndexerConfig};
use chain_rpc::rpc::RpcOperations;
use chain_rpc::HoprRpcOperations;
use chain_types::ContractAddresses;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::account::AccountEntry;
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, info, warn};

use crate::errors::{HoprChainError, Result};

use async_std::task::sleep;
use chain_rpc::client::SimpleJsonRpcRetryPolicy;
use hopr_db_api::HoprDbAllOperations;
use hopr_internal_types::prelude::ChannelDirection;

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

    pub async fn sync_chain(&self) -> errors::Result<()> {
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
        Ok(self.db.get_safe_allowance(None).await?)
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
