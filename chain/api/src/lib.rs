//! Crate containing the API object for chain operations used by the HOPRd node.

pub mod config;
pub mod errors;
pub mod executors;

use std::{collections::HashMap, sync::Arc, time::Duration};

use alloy::{
    rpc::{client::ClientBuilder, types::TransactionRequest},
    transports::{
        http::{Http, ReqwestTransport},
        layers::RetryBackoffLayer,
    },
};
use config::ChainNetworkConfig;
use executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig};
use futures::{FutureExt, future::AbortHandle};
use hopr_async_runtime::{prelude::sleep, spawn_as_abortable};
use hopr_chain_actions::{
    ChainActions,
    action_queue::{ActionQueue, ActionQueueConfig},
    action_state::IndexerActionTracker,
    payload::SafePayloadGenerator,
};
use hopr_chain_indexer::{IndexerConfig, block::Indexer, handlers::ContractEventHandlers};
use hopr_chain_rpc::{
    HoprRpcOperations,
    client::DefaultRetryPolicy,
    rpc::{RpcOperations, RpcOperationsConfig},
    transport::ReqwestClient,
};
use hopr_chain_types::ContractAddresses;
pub use hopr_chain_types::chain_events::SignificantChainEvent;
use hopr_crypto_types::prelude::*;
use hopr_db_sql::HoprDbAllOperations;
pub use hopr_internal_types::channels::ChannelEntry;
use hopr_internal_types::{
    account::AccountEntry, channels::CorruptedChannelEntry, prelude::ChannelDirection, tickets::WinningProbability,
};
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, info, warn};

use crate::errors::{HoprChainError, Result};

pub type DefaultHttpRequestor = hopr_chain_rpc::transport::ReqwestClient;

/// Checks whether the node can be registered with the Safe in the NodeSafeRegistry
pub async fn can_register_with_safe<Rpc: HoprRpcOperations>(
    me: Address,
    safe_address: Address,
    rpc: &Rpc,
) -> Result<bool> {
    let target_address = rpc.get_module_target_address().await?;
    debug!(node_address = %me, %safe_address, %target_address, "can register with safe");

    if target_address != safe_address {
        // cannot proceed when the safe address is not the target/owner of given module
        return Err(HoprChainError::Api("safe is not the module target".into()));
    }

    let registered_address = rpc.get_safe_from_node_safe_registry(me).await?;
    info!(%registered_address, "currently registered Safe address in NodeSafeRegistry");

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
    min_balance: XDaiBalance,
    max_delay: Duration,
    rpc: &Rpc,
) -> Result<()> {
    let multiplier = 1.05;
    let mut current_delay = Duration::from_secs(2).min(max_delay);

    while current_delay <= max_delay {
        match rpc.get_xdai_balance(address).await {
            Ok(current_balance) => {
                info!(balance = %current_balance, "balance status");
                if current_balance.ge(&min_balance) {
                    info!("node is funded");
                    return Ok(());
                } else {
                    warn!("still unfunded, trying again soon");
                }
            }
            Err(e) => error!(error = %e, "failed to fetch balance from the chain"),
        }

        sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(HoprChainError::Api("timeout waiting for funds".into()))
}

fn build_transport_client(url: &str) -> Result<Http<ReqwestClient>> {
    let parsed_url = url::Url::parse(url).unwrap_or_else(|_| panic!("failed to parse URL: {url}"));
    Ok(ReqwestTransport::new(parsed_url))
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum HoprChainProcess {
    Indexer,
    OutgoingOnchainActionQueue,
}

type ActionQueueType<T> = ActionQueue<
    T,
    IndexerActionTracker,
    EthereumTransactionExecutor<
        TransactionRequest,
        RpcEthereumClient<RpcOperations<DefaultHttpRequestor>>,
        SafePayloadGenerator,
    >,
>;

/// Represents all chain interactions exported to be used in the hopr-lib
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
    indexer_events_tx: async_channel::Sender<SignificantChainEvent>,
    db: T,
    hopr_chain_actions: ChainActions<T>,
    action_queue: ActionQueueType<T>,
    action_state: Arc<IndexerActionTracker>,
    rpc_operations: RpcOperations<DefaultHttpRequestor>,
}

impl<T: HoprDbAllOperations + Send + Sync + Clone + std::fmt::Debug + 'static> HoprChain<T> {
    #[allow(clippy::too_many_arguments)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
    pub fn new(
        me_onchain: ChainKeypair,
        db: T,
        // --
        chain_config: ChainNetworkConfig,
        module_address: Address,
        // --
        contract_addresses: ContractAddresses,
        safe_address: Address,
        indexer_cfg: IndexerConfig,
        indexer_events_tx: async_channel::Sender<SignificantChainEvent>,
    ) -> Result<Self> {
        // TODO: extract this from the global config type
        let mut rpc_http_config = hopr_chain_rpc::HttpPostRequestorConfig::default();
        if let Some(max_rpc_req) = chain_config.max_requests_per_sec {
            rpc_http_config.max_requests_per_sec = Some(max_rpc_req); // override the default if set
        }

        // TODO(#7140): replace this DefaultRetryPolicy with a custom one that computes backoff with the number of
        // retries
        let rpc_http_retry_policy = DefaultRetryPolicy::default();

        // TODO: extract this from the global config type
        let rpc_cfg = RpcOperationsConfig {
            chain_id: chain_config.chain.chain_id as u64,
            contract_addrs: contract_addresses,
            module_address,
            safe_address,
            expected_block_time: Duration::from_millis(chain_config.chain.block_time),
            tx_polling_interval: Duration::from_millis(chain_config.tx_polling_interval),
            finality: chain_config.confirmations,
            max_block_range_fetch_size: chain_config.max_block_range,
            ..Default::default()
        };

        // TODO: extract this from the global config type
        let rpc_client_cfg = RpcEthereumClientConfig::default();

        // TODO: extract this from the global config type
        let action_queue_cfg = ActionQueueConfig::default();

        // --- Configs done ---

        let transport_client = build_transport_client(&chain_config.chain.default_provider)?;

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new_with_policy(2, 100, 100, rpc_http_retry_policy))
            .transport(transport_client.clone(), transport_client.guess_local());

        let requestor = DefaultHttpRequestor::new();

        // Build RPC operations
        let rpc_operations =
            RpcOperations::new(rpc_client, requestor, &me_onchain, rpc_cfg, None).expect("failed to initialize RPC");

        // Build the Ethereum Transaction Executor that uses RpcOperations as backend
        let ethereum_tx_executor = EthereumTransactionExecutor::new(
            RpcEthereumClient::new(rpc_operations.clone(), rpc_client_cfg),
            SafePayloadGenerator::new(&me_onchain, contract_addresses, module_address),
        );

        // Build the Action Queue
        let action_queue = ActionQueue::new(
            db.clone(),
            IndexerActionTracker::default(),
            ethereum_tx_executor,
            action_queue_cfg,
        );

        let action_state = action_queue.action_state();
        let action_sender = action_queue.new_sender();

        // Instantiate Chain Actions
        let hopr_chain_actions = ChainActions::new(&me_onchain, db.clone(), action_sender);

        Ok(Self {
            me_onchain,
            safe_address,
            contract_addresses,
            indexer_cfg,
            indexer_events_tx,
            db,
            hopr_chain_actions,
            action_queue,
            action_state,
            rpc_operations,
        })
    }

    /// Execute all processes of the [`HoprChain`] object.
    ///
    /// This method will spawn the [`HoprChainProcess::Indexer`] and [`HoprChainProcess::OutgoingOnchainActionQueue`]
    /// processes and return join handles to the calling function.
    pub async fn start(&self) -> errors::Result<HashMap<HoprChainProcess, AbortHandle>> {
        let mut processes: HashMap<HoprChainProcess, AbortHandle> = HashMap::new();

        processes.insert(
            HoprChainProcess::OutgoingOnchainActionQueue,
            spawn_as_abortable!(self.action_queue.clone().start().inspect(|_| tracing::warn!(
                task = "action queue - outgoing",
                "long-running background task finished"
            ))),
        );
        processes.insert(
            HoprChainProcess::Indexer,
            Indexer::new(
                self.rpc_operations.clone(),
                ContractEventHandlers::new(
                    self.contract_addresses,
                    self.safe_address,
                    self.me_onchain.clone(),
                    self.db.clone(),
                    self.rpc_operations.clone(),
                ),
                self.db.clone(),
                self.indexer_cfg.clone(),
                self.indexer_events_tx.clone(),
            )
            .start()
            .await?,
        );
        Ok(processes)
    }

    pub fn me_onchain(&self) -> Address {
        self.me_onchain.public().to_address()
    }

    pub fn action_state(&self) -> Arc<IndexerActionTracker> {
        self.action_state.clone()
    }

    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self.db.get_accounts(None, true).await?)
    }

    pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<ChannelEntry> {
        self.db
            .get_channel_by_parties(None, src, dest, false)
            .await
            .map_err(HoprChainError::from)
            .and_then(|v| {
                v.ok_or(errors::HoprChainError::Api(format!(
                    "Channel entry not available {src}-{dest}"
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

    pub async fn corrupted_channels(&self) -> errors::Result<Vec<CorruptedChannelEntry>> {
        Ok(self.db.get_all_corrupted_channels(None).await?)
    }

    pub async fn ticket_price(&self) -> errors::Result<Option<HoprBalance>> {
        Ok(self.db.get_indexer_data(None).await?.ticket_price)
    }

    pub async fn nr_enabled(&self) -> errors::Result<bool> {
        Ok(self.db.get_indexer_data(None).await?.nr_enabled)
    }

    pub async fn safe_allowance(&self) -> errors::Result<HoprBalance> {
        Ok(self.db.get_safe_hopr_allowance(None).await?)
    }

    pub fn actions_ref(&self) -> &ChainActions<T> {
        &self.hopr_chain_actions
    }

    pub fn actions_mut_ref(&mut self) -> &mut ChainActions<T> {
        &mut self.hopr_chain_actions
    }

    pub fn rpc(&self) -> &RpcOperations<DefaultHttpRequestor> {
        &self.rpc_operations
    }

    /// Retrieves the balance of the node's on-chain account for the specified currency.
    ///
    /// This method queries the on-chain balance of the node's account for the given currency.
    /// It supports querying balances for XDai and WxHOPR currencies. If the currency is unsupported,
    /// an error is returned.
    ///
    /// # Returns
    /// * `Result<Balance<C>>` - The balance of the node's account for the specified currency, or an error if the query
    ///   fails.
    pub async fn get_balance<C: Currency + Send>(&self) -> errors::Result<Balance<C>> {
        let bal = if C::is::<XDai>() {
            self.rpc_operations
                .get_xdai_balance(self.me_onchain())
                .await?
                .to_be_bytes()
        } else if C::is::<WxHOPR>() {
            self.rpc_operations
                .get_hopr_balance(self.me_onchain())
                .await?
                .to_be_bytes()
        } else {
            return Err(HoprChainError::Api("unsupported currency".into()));
        };

        Ok(Balance::<C>::from(U256::from_be_bytes(bal)))
    }

    /// Retrieves the balance of the specified address for the given currency.
    ///
    /// This method queries the on-chain balance of the provided address for the specified currency.
    /// It supports querying balances for XDai and WxHOPR currencies. If the currency is unsupported,
    /// an error is returned.
    ///
    /// # Arguments
    /// * `address` - The address whose balance is to be retrieved.
    ///
    /// # Returns
    /// * `Result<Balance<C>>` - The balance of the specified address for the given currency, or an error if the query
    ///   fails.
    pub async fn get_safe_balance<C: Currency + Send>(&self, safe_address: Address) -> errors::Result<Balance<C>> {
        let bal = if C::is::<XDai>() {
            self.rpc_operations.get_xdai_balance(safe_address).await?.to_be_bytes()
        } else if C::is::<WxHOPR>() {
            self.rpc_operations.get_hopr_balance(safe_address).await?.to_be_bytes()
        } else {
            return Err(HoprChainError::Api("unsupported currency".into()));
        };

        Ok(Balance::<C>::from(U256::from_be_bytes(bal)))
    }

    /// Retrieves the HOPR token allowance granted by the safe address to the channels contract.
    ///
    /// This method queries the on-chain HOPR token contract to determine how many tokens
    /// the safe address has approved the channels contract to spend on its behalf.
    ///
    /// # Returns
    /// * `Result<HoprBalance>` - The current allowance amount, or an error if the query fails
    pub async fn get_safe_hopr_allowance(&self) -> Result<HoprBalance> {
        Ok(self
            .rpc_operations
            .get_hopr_allowance(self.safe_address, self.contract_addresses.channels)
            .await?)
    }

    pub async fn get_channel_closure_notice_period(&self) -> errors::Result<Duration> {
        Ok(self.rpc_operations.get_channel_closure_notice_period().await?)
    }

    pub async fn get_eligibility_status(&self) -> errors::Result<bool> {
        Ok(self.rpc_operations.get_eligibility_status(self.me_onchain()).await?)
    }

    pub async fn get_minimum_winning_probability(&self) -> errors::Result<WinningProbability> {
        Ok(self.rpc_operations.get_minimum_network_winning_probability().await?)
    }

    pub async fn get_minimum_ticket_price(&self) -> errors::Result<HoprBalance> {
        Ok(self.rpc_operations.get_minimum_network_ticket_price().await?)
    }
}
