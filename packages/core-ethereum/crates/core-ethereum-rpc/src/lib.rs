use std::fmt::{Display, Formatter};
use std::pin::Pin;

use async_trait::async_trait;
pub use ethers::types::transaction::eip2718::TypedTransaction;
pub use futures::channel::mpsc::UnboundedReceiver;
use futures::Stream;
use primitive_types::H256;

use core_crypto::types::Hash;
use utils_types::primitives::{Address, Balance, BalanceType, U256};

use crate::errors::{HttpRequestError, Result};

pub mod errors;
pub mod indexer;
pub mod rpc;

#[cfg(feature = "wasm")]
mod nodejs;

mod client;
mod helper;

/// A type containing selected fields from  the `eth_getLogs` RPC calls.
/// This is further restritect to already mined blocks.
#[derive(Debug, Clone)]
pub struct Log {
    /// Contract address
    pub address: Address,
    /// Topics
    pub topics: Vec<Hash>,
    /// Raw log data
    pub data: Box<[u8]>,
    /// Transaction index
    pub tx_index: u64,
    /// Corresponding block number
    pub block_number: u64,
    /// Log index
    pub log_index: U256,
}

impl From<ethers::types::Log> for Log {
    fn from(value: ethers::prelude::Log) -> Self {
        Self {
            address: value.address.into(),
            topics: value.topics.into_iter().map(Hash::from).collect(),
            data: Box::from(value.data.as_ref()),
            tx_index: value.transaction_index.expect("tx index must be present").as_u64(),
            block_number: value.block_number.expect("block id must be present").as_u64(),
            log_index: value.log_index.expect("log index must be present").into(),
        }
    }
}

impl From<Log> for ethers::abi::RawLog {
    fn from(value: Log) -> Self {
        ethers::abi::RawLog {
            topics: value.topics.into_iter().map(H256::from).collect(),
            data: value.data.into(),
        }
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "log in block #{} of {} with {} topics",
            self.block_number,
            self.address,
            self.topics.len()
        )
    }
}

/// Represents a filter to extract logs containing specific contract events from a block.
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Contract addresses
    pub address: Vec<Address>,
    /// Event topics
    pub topics: Vec<Hash>,
}

impl LogFilter {
    /// Indicates if this filter filters anything.
    pub fn is_empty(&self) -> bool {
        self.address.is_empty() && self.topics.is_empty()
    }
}

impl Display for LogFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "filter of {} contracts with {} topics",
            self.address.len(),
            self.topics.len()
        )
    }
}

impl From<LogFilter> for ethers::types::Filter {
    fn from(value: LogFilter) -> Self {
        ethers::types::Filter::new()
            .address(
                value
                    .address
                    .into_iter()
                    .map(ethers::types::Address::from)
                    .collect::<Vec<_>>(),
            )
            .topic0(value.topics)
    }
}

/// Abstraction for HTTP client that perform HTTP POST with JSON data.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HttpPostRequestor: Send + Sync {
    /// Performs HTTP POST of JSON data to the given URL
    /// and obtains the JSON response.
    async fn http_post(&self, url: &str, json_data: &str) -> std::result::Result<String, HttpRequestError>;
}

/// Short-hand for creating new EIP1559 transaction object.
pub fn create_eip1559_transaction() -> TypedTransaction {
    TypedTransaction::Eip1559(ethers::types::Eip1559TransactionRequest::new())
}

/// Trait defining general set of operations an RPC provider
/// must provide to the HOPR node.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprRpcOperations {
    /// Retrieves the timestamp from the given block number.
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>>;

    /// Retrieves the node's account balance of the given type.
    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance>;

    /// Retrieves info of the given node module's target.
    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>>;

    /// Retrieves safe address of the given node address from the registry.
    async fn get_safe_from_node_safe_registry(&self, node: Address) -> Result<Address>;

    /// Retrieves target address of the node module.
    async fn get_module_target_address(&self) -> Result<Address>;

    /// Sends transaction to the RPC provider.
    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash>;
}

/// Structure containing filtered logs that all belong to the same block.
#[derive(Debug, Clone, Default)]
pub struct BlockWithLogs {
    /// Block number
    pub block_id: u64,
    /// Filtered logs belonging to this block.
    pub logs: Vec<Log>,
}
impl Display for BlockWithLogs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "block #{} with {} logs", self.block_id, self.logs.len())
    }
}

impl BlockWithLogs {
    /// Returns `true` if no logs are contained within this block.
    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }

    /// Returns the number of logs within this block.
    pub fn len(&self) -> usize {
        self.logs.len()
    }
}

/// Trait with RPC provider functionality required by the Indexer.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprIndexerRpcOperations {
    /// Retrieves the latest block number.
    async fn block_number(&self) -> Result<u64>;

    /// Starts streaming logs from the given `start_block_number`.
    /// If no `start_block_number` is given, the stream starts from the latest block.
    /// The given `filter` are applied to retrieve the logs, the function fails if the filter is empty.
    /// The streaming stops only when the corresponding channel is closed by the returned receiver.
    fn try_stream_logs<'a>(
        &'a self,
        start_block_number: Option<u64>,
        filter: LogFilter,
    ) -> Result<Pin<Box<dyn Stream<Item = BlockWithLogs> + 'a>>>;
}
