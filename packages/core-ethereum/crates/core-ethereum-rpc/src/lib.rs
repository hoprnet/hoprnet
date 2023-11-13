use async_trait::async_trait;
use core_crypto::types::Hash;
use primitive_types::H256;
use std::fmt::{Display, Formatter};
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use utils_types::traits::BinarySerializable;

use crate::errors::Result;

pub use ethers::types::transaction::eip2718::TypedTransaction;
pub use ethers::types::TxHash;
pub use futures::channel::mpsc::UnboundedReceiver;

pub mod errors;
pub mod indexer;
pub mod rpc;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod nodejs;

/// A type containing selected fields from  the `eth_getBlockByHash`/`eth_getBlockByNumber` RPC
/// calls.
#[derive(Debug, Clone)]
pub struct Block {
    /// Block number
    pub number: Option<u64>,
    /// Block hash if any.
    pub hash: Option<Hash>,
    /// Block timestamp
    pub timestamp: U256,
    /// Transaction hashes within this block
    pub transactions: Vec<Hash>,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (@ {}) with {} txs",
            self.number
                .map(|i| format!("block #{i}"))
                .unwrap_or("pending block".into()),
            self.timestamp.as_u64(),
            self.transactions.len()
        )
    }
}

impl From<ethers::types::Block<H256>> for Block {
    fn from(value: ethers::prelude::Block<H256>) -> Self {
        Self {
            number: value.number.map(|u| u.as_u64()),
            hash: value.hash.map(|h| h.0.into()),
            timestamp: value.timestamp.into(),
            transactions: value.transactions.into_iter().map(|h| Hash::from(h.0)).collect(),
        }
    }
}

/// A type containing selected fields from  the `eth_getLogs` RPC calls.
#[derive(Debug, Clone)]
pub struct Log {
    /// Contract address
    pub address: Address,
    /// Topics
    pub topics: Vec<Hash>,
    /// Raw log data
    pub data: Box<[u8]>,
    /// Transaction index
    pub tx_index: Option<u64>,
    /// Corresponding block number
    pub block_number: Option<u64>,
    /// Log index
    pub log_index: Option<U256>,
}

impl From<ethers::types::Log> for Log {
    fn from(value: ethers::prelude::Log) -> Self {
        Self {
            address: value.address.into(),
            topics: value.topics.into_iter().map(|h| Hash::from(h.0)).collect(),
            data: Box::from(value.data.as_ref()),
            tx_index: value.transaction_index.map(|u| u.as_u64()),
            block_number: value.block_number.map(|u| u.as_u64()),
            log_index: value.log_index.map(|u| u.into()),
        }
    }
}

impl From<Log> for ethers::abi::RawLog {
    fn from(value: Log) -> Self {
        ethers::abi::RawLog {
            topics: value.topics.iter().map(|h| H256::from_slice(&h.to_bytes())).collect(),
            data: value.data.into(),
        }
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "log of {} with {} topics", self.address, self.topics.len())
    }
}

/// Represents a mined block with also corresponding logs fetched (according to some `EventsQuery` filter).
#[derive(Debug, Clone)]
pub struct BlockWithLogs {
    /// Block with TX hashes.
    pub block: Block,
    /// Logs of interest corresponding to the block
    pub logs: Vec<Log>,
}

impl Display for BlockWithLogs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} and {} logs", self.block, self.logs.len())
    }
}

/// Represents a query to extract logs containing specific contract events.
#[derive(Debug, Clone)]
pub struct EventsQuery {
    /// Contract address
    pub address: Address,
    /// Event topics
    pub topics: Vec<TxHash>,
}

impl Display for EventsQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "filter of {} with {} topics", self.address, self.topics.len())
    }
}

impl From<EventsQuery> for Vec<ethers::types::Filter> {
    fn from(value: EventsQuery) -> Self {
        let mut ret = Vec::new();
        let addr: ethers::types::H160 = value.address.into();

        let mut filter = ethers::types::Filter::new().address::<ethers::types::H160>(addr.into());
        for (i, topic) in value.topics.into_iter().enumerate() {
            if i > 0 && i % 4 == 0 {
                ret.push(filter);
                filter = ethers::types::Filter::new().address::<ethers::types::H160>(addr.into());
            }
            filter.topics[i % 4] = Some(topic.into());
        }
        ret.push(filter);

        ret
    }
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

/// Extension of `HoprRpcOperations` trait with functionality required by the Indexer.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprIndexerRpcOperations: HoprRpcOperations {
    /// Retrieves the latest block number.
    async fn block_number(&self) -> Result<u64>;

    /// Starts streaming the blocks with logs from the given `start_block_number`.
    /// If no `start_block_number` is given, the stream starts from the latest block.
    /// The given `filters` are applied to retrieve the logs for each retrieved block.
    /// The streaming stops only when the corresponding channel is closed by the returned receiver.
    async fn poll_blocks_with_logs(
        &self,
        start_block_number: Option<u64>,
        filters: Vec<EventsQuery>,
    ) -> Result<UnboundedReceiver<BlockWithLogs>>;
}
