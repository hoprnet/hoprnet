//! Crate containing the API object for chain operations used by the HOPRd node.

pub mod errors;
pub mod executors;

pub use chain_types::chain_events::SignificantChainEvent;
pub use hopr_internal_types::channels::ChannelEntry;

use async_lock::RwLock;
use chain_db::db::CoreEthereumDb;
use std::sync::Arc;
use std::time::Duration;

use chain_actions::CoreEthereumActions;
use chain_db::traits::HoprCoreEthereumDbActions;
use chain_indexer::block::{Indexer, IndexerConfig};
use chain_indexer::handlers::ContractEventHandlers;
use chain_rpc::rpc::RpcOperations;
use chain_rpc::HoprRpcOperations;
use chain_types::ContractAddresses;
use hopr_internal_types::account::AccountEntry;
use hopr_crypto::keypairs::{ChainKeypair, Keypair};
use log::{debug, error, info, warn};
use utils_db::CurrentDbShim;
use hopr_primitive_types::primitives::{Address, Balance, BalanceType, U256};

use crate::errors::{HoprChainError, Result};

use async_std::task::sleep;
use chain_rpc::client::SimpleJsonRpcRetryPolicy;

pub type DefaultHttpPostRequestor = chain_rpc::client::native::SurfRequestor;

pub type JsonRpcClient = chain_rpc::client::JsonRpcProviderClient<DefaultHttpPostRequestor, SimpleJsonRpcRetryPolicy>;

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
/// This is done by querying the RPC provider for balance with backoff until `max_delay`
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
                if current_balance.gte(&min_balance) {
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

#[derive(Debug, Clone)]
pub struct HoprChain {
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<utils_db::CurrentDbShim>>>,
    indexer: Indexer<
        RpcOperations<JsonRpcClient>,
        ContractEventHandlers<CoreEthereumDb<utils_db::CurrentDbShim>>,
        CoreEthereumDb<utils_db::CurrentDbShim>,
    >,
    chain_actions: CoreEthereumActions<CoreEthereumDb<CurrentDbShim>>,
    rpc_operations: RpcOperations<JsonRpcClient>,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
}

impl HoprChain {
    pub fn new(
        me_onchain: ChainKeypair,
        db: Arc<RwLock<CoreEthereumDb<CurrentDbShim>>>,
        contract_addresses: ContractAddresses,
        safe_address: Address,
        indexer_cfg: IndexerConfig,
        indexer_events_tx: futures::channel::mpsc::UnboundedSender<SignificantChainEvent>,
        chain_actions: CoreEthereumActions<CoreEthereumDb<CurrentDbShim>>,
        rpc_operations: RpcOperations<JsonRpcClient>,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    ) -> Self {
        let db_processor =
            ContractEventHandlers::new(contract_addresses, safe_address, (&me_onchain).into(), db.clone());

        let indexer = Indexer::new(
            rpc_operations.clone(),
            db_processor,
            db.clone(),
            indexer_cfg,
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

    pub async fn ticket_price(&self) -> errors::Result<Option<U256>> {
        Ok(self.db.read().await.get_ticket_price().await?)
    }

    pub async fn safe_allowance(&self) -> errors::Result<Balance> {
        Ok(self.db.read().await.get_staking_safe_allowance().await?)
    }

    pub fn actions_ref(&self) -> &CoreEthereumActions<CoreEthereumDb<CurrentDbShim>> {
        &self.chain_actions
    }

    pub fn actions_mut_ref(&mut self) -> &mut CoreEthereumActions<CoreEthereumDb<CurrentDbShim>> {
        &mut self.chain_actions
    }

    // NOTE: needed early in the initialization to sync
    pub fn channel_graph(&self) -> Arc<RwLock<core_path::channel_graph::ChannelGraph>> {
        self.channel_graph.clone()
    }

    // NOTE: needed early in the initialization to sync
    pub fn db(&self) -> Arc<RwLock<CoreEthereumDb<utils_db::CurrentDbShim>>> {
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
