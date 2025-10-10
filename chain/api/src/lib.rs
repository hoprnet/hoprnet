//! Crate containing the API object for chain operations used by the HOPRd node.

pub mod errors;
pub mod executors;

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use alloy::{
    rpc::{client::ClientBuilder, types::TransactionRequest},
    transports::{
        http::{Http, ReqwestTransport},
        layers::RetryBackoffLayer,
    },
};
use executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig};
use futures::{
    FutureExt, Stream, StreamExt,
    future::{AbortHandle, BoxFuture},
    stream::BoxStream,
};
use hopr_api::{
    Multiaddr,
    chain::{
        AccountSelector, AnnouncementError, ChainEvents, ChainKeyOperations, ChainReadAccountOperations,
        ChainReadChannelOperations, ChainReceipt, ChainValues, ChainWriteAccountOperations,
        ChainWriteChannelOperations, ChainWriteTicketOperations, ChannelSelector, DomainSeparators,
    },
    db::TicketSelector,
};
use hopr_async_runtime::{prelude::sleep, spawn_as_abortable};
use hopr_chain_actions::{
    ChainActions,
    action_queue::{ActionQueue, ActionQueueConfig},
    action_state::{ActionState, IndexerActionTracker},
    channels::ChannelActions,
    errors::ChainActionsError,
    node::NodeActions,
    payload::SafePayloadGenerator,
    redeem::TicketRedeemActions,
};
pub use hopr_chain_config as config;
pub use hopr_chain_indexer::IndexerConfig;
use hopr_chain_indexer::{block::Indexer, handlers::ContractEventHandlers};
use hopr_chain_rpc::{
    HoprRpcOperations,
    client::DefaultRetryPolicy,
    rpc::{RpcOperations, RpcOperationsConfig},
};
use hopr_chain_types::ContractAddresses;
pub use hopr_chain_types::chain_events::SignificantChainEvent;
use hopr_crypto_types::prelude::*;
use hopr_db_node::HoprNodeDb;
pub use hopr_db_sql::info::IndexerStateInfo;
use hopr_db_sql::{
    HoprIndexerDb, HoprIndexerDbConfig,
    logs::HoprDbLogOperations,
    prelude::{
        HoprDbAccountOperations, HoprDbChannelOperations, HoprDbCorruptedChannelOperations, HoprDbInfoOperations,
    },
};
pub use hopr_internal_types::channels::ChannelEntry;
use hopr_internal_types::{
    account::AccountEntry,
    channels::{ChannelId, CorruptedChannelEntry},
    prelude::{AcknowledgedTicket, AcknowledgedTicketStatus, ChannelStatus, generate_channel_id},
    tickets::WinningProbability,
};
use hopr_primitive_types::prelude::*;
use tracing::{debug, error, info, trace, warn};

use crate::errors::{HoprChainError, Result};

#[cfg(feature = "runtime-tokio")]
pub type DefaultHttpRequestor = hopr_chain_rpc::transport::ReqwestClient;

#[cfg(not(feature = "runtime-tokio"))]
compile_error!("The `runtime-tokio` feature must be enabled");

/// Waits until the given address is funded.
///
/// This is done by querying the RPC provider for balance with backoff until `max_delay` argument.
pub async fn wait_for_funds<R: ChainReadAccountOperations>(
    min_balance: XDaiBalance,
    suggested_balance: XDaiBalance,
    max_delay: Duration,
    resolver: &R,
) -> Result<()> {
    info!(
        suggested_minimum_balance = %suggested_balance,
        "Node about to start, checking for funds",
    );

    let multiplier = 1.05;
    let mut current_delay = Duration::from_secs(2).min(max_delay);

    while current_delay <= max_delay {
        match resolver.node_balance::<XDai>().await {
            Ok(current_balance) => {
                info!(balance = %current_balance, "balance status");
                if current_balance.ge(&min_balance) {
                    info!("node is funded");
                    return Ok(());
                } else {
                    warn!("still unfunded, trying again soon");
                }
            }
            Err(error) => error!(%error, "failed to fetch balance from the chain"),
        }

        sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(HoprChainError::Api("timeout waiting for funds".into()))
}

fn build_transport_client(url: &str) -> Result<Http<DefaultHttpRequestor>> {
    let parsed_url = url::Url::parse(url).unwrap_or_else(|_| panic!("failed to parse URL: {url}"));
    Ok(ReqwestTransport::new(parsed_url))
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum HoprChainProcess {
    Indexer,
    OutgoingOnchainActionQueue,
}

const ON_CHAIN_SIG_EVENT_QUEUE_SIZE: usize = 10_000;

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
#[derive(Clone)]
pub struct HoprChain {
    me_onchain: ChainKeypair,
    safe_address: Address,
    contract_addresses: ContractAddresses,
    indexer_cfg: IndexerConfig,
    indexer_events_tx: futures::channel::mpsc::Sender<SignificantChainEvent>,
    indexer_events_rx: Arc<std::sync::Mutex<Option<futures::channel::mpsc::Receiver<SignificantChainEvent>>>>,
    db: HoprIndexerDb,
    node_db: HoprNodeDb,
    hopr_chain_actions: ChainActions<HoprNodeDb>,
    action_queue: ActionQueueType<HoprNodeDb>,
    action_state: Arc<IndexerActionTracker>,
    rpc_operations: RpcOperations<DefaultHttpRequestor>,
}

impl HoprChain {
    #[allow(clippy::too_many_arguments)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
    pub fn new(
        me_onchain: ChainKeypair,
        chain_config: config::ChainNetworkConfig,
        node_db: HoprNodeDb,
        data_dir_path: &str,
        module_address: Address,
        contract_addresses: ContractAddresses,
        safe_address: Address,
        indexer_cfg: IndexerConfig,
    ) -> Result<Self> {
        let db = futures::executor::block_on(HoprIndexerDb::new(
            PathBuf::from_iter([data_dir_path, "index_db"]).as_path(),
            me_onchain.clone(),
            HoprIndexerDbConfig {
                create_if_missing: node_db.config().create_if_missing,
                force_create: node_db.config().force_create,
                log_slow_queries: node_db.config().log_slow_queries,
            },
        ))?;

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
            node_db.clone(),
            IndexerActionTracker::default(),
            ethereum_tx_executor,
            action_queue_cfg,
        );

        let action_state = action_queue.action_state();
        let action_sender = action_queue.new_sender();

        // Instantiate Chain Actions
        let hopr_chain_actions = ChainActions::new(&me_onchain, db.clone(), node_db.clone(), action_sender);

        // The channel can be bounded, since it is used only after the historical on-chain sync has been completed.
        let (indexer_events_tx, indexer_events_rx) =
            futures::channel::mpsc::channel::<SignificantChainEvent>(ON_CHAIN_SIG_EVENT_QUEUE_SIZE);

        Ok(Self {
            me_onchain,
            safe_address,
            contract_addresses,
            indexer_cfg,
            indexer_events_tx,
            indexer_events_rx: Arc::new(std::sync::Mutex::new(Some(indexer_events_rx))),
            db,
            node_db,
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
                    self.node_db.clone(),
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

    pub async fn corrupted_channels(&self) -> errors::Result<Vec<CorruptedChannelEntry>> {
        Ok(self.db.get_all_corrupted_channels(None).await?)
    }

    fn actions_ref(&self) -> &ChainActions<HoprNodeDb> {
        &self.hopr_chain_actions
    }

    fn rpc(&self) -> &RpcOperations<DefaultHttpRequestor> {
        &self.rpc_operations
    }

    pub async fn get_indexer_state(&self) -> errors::Result<IndexerStateInfo> {
        let indexer_state_info = self.db.get_indexer_state_info(None).await?;

        match self.db.get_last_checksummed_log().await? {
            Some(log) => {
                let checksum = match log.checksum {
                    Some(checksum) => Hash::from_hex(checksum.as_str())?,
                    None => Hash::default(),
                };
                Ok(IndexerStateInfo {
                    latest_log_block_number: log.block_number as u32,
                    latest_log_checksum: checksum,
                    ..indexer_state_info
                })
            }
            None => Ok(indexer_state_info),
        }
    }
}

#[async_trait::async_trait]
impl ChainReadAccountOperations for HoprChain {
    type Error = HoprChainError;

    async fn node_balance<C: Currency>(&self) -> std::result::Result<Balance<C>, Self::Error> {
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

    async fn safe_balance<C: Currency>(&self) -> std::result::Result<Balance<C>, Self::Error> {
        let bal = if C::is::<XDai>() {
            self.rpc_operations
                .get_xdai_balance(self.safe_address)
                .await?
                .to_be_bytes()
        } else if C::is::<WxHOPR>() {
            self.rpc_operations
                .get_hopr_balance(self.safe_address)
                .await?
                .to_be_bytes()
        } else {
            return Err(HoprChainError::Api("unsupported currency".into()));
        };

        Ok(Balance::<C>::from(U256::from_be_bytes(bal)))
    }

    async fn safe_allowance<C: Currency>(&self) -> std::result::Result<Balance<C>, Self::Error> {
        let amount = if C::is::<XDai>() {
            return Err(HoprChainError::Api("unsupported currency".into()));
        } else {
            self.rpc_operations
                .get_hopr_allowance(self.safe_address, self.contract_addresses.channels)
                .await?
                .amount()
        };
        Ok(Balance::<C>::from(amount))
    }

    async fn find_account_by_address(
        &self,
        address: &Address,
    ) -> std::result::Result<Option<AccountEntry>, Self::Error> {
        Ok(self.db.get_account(None, *address).await?)
    }

    async fn find_account_by_packet_key(
        &self,
        packet_key: &OffchainPublicKey,
    ) -> std::result::Result<Option<AccountEntry>, Self::Error> {
        Ok(self.db.get_account(None, *packet_key).await?)
    }

    async fn check_node_safe_module_status(&self) -> std::result::Result<bool, Self::Error> {
        let safe_module_configuration = self
            .rpc_operations
            .check_node_safe_module_status(self.me_onchain())
            .await?;
        if !safe_module_configuration.should_pass() {
            error!(
                ?safe_module_configuration,
                "Something is wrong with the safe module configuration",
            );
            Ok(false)
        } else {
            Ok(true)
        }
    }

    async fn can_register_with_safe(&self, safe_address: &Address) -> std::result::Result<bool, Self::Error> {
        let me = self.me_onchain.public().to_address();
        let target_address = self.rpc().get_module_target_address().await?;
        debug!(node_address = %me, %safe_address, %target_address, "can register with safe");

        if &target_address != safe_address {
            // cannot proceed when the safe address is not the target/owner of the given module
            return Err(HoprChainError::Api("safe is not the module target".into()));
        }

        let registered_address = self.rpc().get_safe_from_node_safe_registry(me).await?;
        info!(%registered_address, "currently registered Safe address in NodeSafeRegistry");

        if registered_address.is_zero() {
            info!("Node is not associated with a Safe in NodeSafeRegistry yet");
            Ok(true)
        } else if &registered_address != safe_address {
            Err(HoprChainError::Api(
                "Node is associated with a different Safe in NodeSafeRegistry".into(),
            ))
        } else {
            info!("Node is associated with correct Safe in NodeSafeRegistry");
            Ok(false)
        }
    }

    async fn stream_accounts<'a>(
        &'a self,
        selector: AccountSelector,
    ) -> std::result::Result<BoxStream<'a, AccountEntry>, Self::Error> {
        Ok(self.db.stream_accounts(selector.public_only).await?)
    }

    async fn count_accounts(&self, selector: AccountSelector) -> std::result::Result<usize, Self::Error> {
        Ok(self.db.stream_accounts(selector.public_only).await?.count().await)
    }
}

#[async_trait::async_trait]
impl ChainWriteAccountOperations for HoprChain {
    type Error = HoprChainError;

    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainKeypair,
    ) -> std::result::Result<
        BoxFuture<'_, std::result::Result<ChainReceipt, Self::Error>>,
        AnnouncementError<Self::Error>,
    > {
        Ok(self
            .actions_ref()
            .announce(multiaddrs, key)
            .await
            .map_err(|error| match error {
                hopr_chain_actions::errors::ChainActionsError::AlreadyAnnounced => AnnouncementError::AlreadyAnnounced,
                e => AnnouncementError::ProcessingError(HoprChainError::ActionsError(e)),
            })?
            .map(|r| r.map(|c| c.tx_hash).map_err(HoprChainError::from))
            .boxed())
    }

    async fn withdraw<C: Currency + Send>(
        &self,
        balance: Balance<C>,
        recipient: &Address,
    ) -> std::result::Result<BoxFuture<'_, std::result::Result<ChainReceipt, Self::Error>>, Self::Error> {
        Ok(self
            .actions_ref()
            .withdraw(*recipient, balance)
            .await?
            .map(|r| r.map(|c| c.tx_hash).map_err(HoprChainError::from))
            .boxed())
    }

    async fn register_safe(
        &self,
        safe_address: &Address,
    ) -> std::result::Result<BoxFuture<'_, std::result::Result<ChainReceipt, Self::Error>>, Self::Error> {
        Ok(self
            .actions_ref()
            .register_safe_by_node(*safe_address)
            .await?
            .map(|r| r.map(|c| c.tx_hash).map_err(HoprChainError::from))
            .boxed())
    }
}

#[async_trait::async_trait]
impl ChainReadChannelOperations for HoprChain {
    type Error = HoprChainError;

    fn me(&self) -> &Address {
        self.me_onchain.public().as_ref()
    }

    async fn channel_by_parties(
        &self,
        src: &Address,
        dst: &Address,
    ) -> std::result::Result<Option<ChannelEntry>, Self::Error> {
        Ok(self.db.get_channel_by_parties(None, src, dst, true).await?)
    }

    async fn channel_by_id(&self, channel_id: &ChannelId) -> std::result::Result<Option<ChannelEntry>, Self::Error> {
        Ok(self.db.get_channel_by_id(None, channel_id).await?)
    }

    async fn stream_channels<'a>(
        &'a self,
        selector: ChannelSelector,
    ) -> std::result::Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        Ok(self
            .db
            .stream_channels(
                selector.source,
                selector.destination,
                &selector.allowed_states,
                (selector.closure_time_range.0, selector.closure_time_range.1),
            )
            .await?)
    }
}

#[async_trait::async_trait]
impl ChainWriteChannelOperations for HoprChain {
    type Error = HoprChainError;

    async fn open_channel<'a>(
        &'a self,
        dst: &'a Address,
        amount: HoprBalance,
    ) -> std::result::Result<BoxFuture<'a, std::result::Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error>
    {
        let me = self.me_onchain();
        Ok(self
            .actions_ref()
            .open_channel(*dst, amount)
            .await?
            .map(move |res| {
                res.map(|c| (generate_channel_id(&me, dst), c.tx_hash))
                    .map_err(HoprChainError::from)
            })
            .boxed())
    }

    async fn fund_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
        amount: HoprBalance,
    ) -> std::result::Result<BoxFuture<'a, std::result::Result<ChainReceipt, Self::Error>>, Self::Error> {
        Ok(self
            .actions_ref()
            .fund_channel(*channel_id, amount)
            .await?
            .map(|res| res.map(|c| c.tx_hash).map_err(HoprChainError::from))
            .boxed())
    }

    async fn close_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
    ) -> std::result::Result<BoxFuture<'a, std::result::Result<(ChannelStatus, ChainReceipt), Self::Error>>, Self::Error>
    {
        let channel = self
            .db
            .get_channel_by_id(None, channel_id)
            .await?
            .ok_or(HoprChainError::Api("channel not found".into()))?;

        Ok(self
            .actions_ref()
            .close_channel(channel)
            .await?
            .map(|res| {
                res.and_then(|c| {
                    let status = match c.event {
                        Some(hopr_chain_types::chain_events::ChainEventType::ChannelClosed(_)) => ChannelStatus::Closed,
                        Some(hopr_chain_types::chain_events::ChainEventType::ChannelClosureInitiated(c)) => c.status,
                        _ => return Err(ChainActionsError::InvalidState("closure must have event type".into())),
                    };

                    Ok((status, c.tx_hash))
                })
                .map_err(HoprChainError::from)
            })
            .boxed())
    }
}

#[async_trait::async_trait]
impl ChainKeyOperations for HoprChain {
    type Error = HoprChainError;
    type Mapper = hopr_db_sql::CacheKeyMapper;

    async fn chain_key_to_packet_key(
        &self,
        chain: &Address,
    ) -> std::result::Result<Option<OffchainPublicKey>, Self::Error> {
        match self.db.translate_key(None, *chain).await? {
            None => Ok(None),
            Some(key) => Ok(Some(key.try_into()?)),
        }
    }

    async fn packet_key_to_chain_key(
        &self,
        packet: &OffchainPublicKey,
    ) -> std::result::Result<Option<Address>, Self::Error> {
        match self.db.translate_key(None, *packet).await? {
            None => Ok(None),
            Some(key) => Ok(Some(key.try_into()?)),
        }
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        self.db.key_id_mapper_ref()
    }
}

#[async_trait::async_trait]
impl ChainValues for HoprChain {
    type Error = HoprChainError;

    async fn domain_separators(&self) -> std::result::Result<DomainSeparators, Self::Error> {
        let indexer_data = self.db.get_indexer_data(None).await?;
        Ok(DomainSeparators {
            ledger: indexer_data
                .ledger_dst
                .ok_or(HoprChainError::Api("missing ledger dst".into()))?,
            safe_registry: indexer_data
                .safe_registry_dst
                .ok_or(HoprChainError::Api("missing safe registry dst".into()))?,
            channel: indexer_data
                .channels_dst
                .ok_or(HoprChainError::Api("missing channel dst".into()))?,
        })
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> std::result::Result<WinningProbability, Self::Error> {
        let indexer_data = self.db.get_indexer_data(None).await?;
        Ok(indexer_data.minimum_incoming_ticket_winning_prob)
    }

    async fn minimum_ticket_price(&self) -> std::result::Result<HoprBalance, Self::Error> {
        let indexer_data = self.db.get_indexer_data(None).await?;
        // The default minimum ticket price is 0
        Ok(indexer_data.ticket_price.unwrap_or_default())
    }

    async fn channel_closure_notice_period(&self) -> std::result::Result<Duration, Self::Error> {
        Ok(self.rpc_operations.get_channel_closure_notice_period().await?)
    }
}

#[async_trait::async_trait]
impl ChainWriteTicketOperations for HoprChain {
    type Error = HoprChainError;

    async fn redeem_ticket(
        &self,
        ticket: AcknowledgedTicket,
    ) -> std::result::Result<BoxFuture<'_, std::result::Result<ChainReceipt, Self::Error>>, Self::Error> {
        Ok(self
            .actions_ref()
            .redeem_ticket(ticket)
            .await?
            .map(|r| r.map(|c| c.tx_hash).map_err(HoprChainError::from))
            .boxed())
    }

    async fn redeem_tickets_via_selector(
        &self,
        selector: TicketSelector,
    ) -> std::result::Result<Vec<BoxFuture<'_, std::result::Result<ChainReceipt, Self::Error>>>, Self::Error> {
        Ok(self
            .actions_ref()
            .redeem_tickets(selector.with_state(AcknowledgedTicketStatus::Untouched))
            .await?
            .into_iter()
            .map(|r| r.map(|c| c.map(|ac| ac.tx_hash).map_err(HoprChainError::from)).boxed())
            .collect())
    }
}

impl ChainEvents for HoprChain {
    type Error = HoprChainError;

    fn subscribe(
        &self,
    ) -> std::result::Result<impl Stream<Item = SignificantChainEvent> + Send + 'static, Self::Error> {
        if let Some(stream) = self
            .indexer_events_rx
            .lock()
            .map_err(|_| HoprChainError::Api("failed to lock mutex".into()))?
            .take()
        {
            let indexer_action_tracker = self.action_state.clone();
            Ok(stream.then(move |event| {
                let indexer_action_tracker = indexer_action_tracker.clone();
                async move {
                    let resolved = indexer_action_tracker.match_and_resolve(&event).await;
                    if resolved.is_empty() {
                        trace!(%event, "No indexer expectations resolved for the event");
                    } else {
                        debug!(count = resolved.len(), %event, "resolved indexer expectations");
                    }
                    event
                }
            }))
        } else {
            Err(HoprChainError::Api("cannot subscribe more than once".into()))
        }
    }
}
