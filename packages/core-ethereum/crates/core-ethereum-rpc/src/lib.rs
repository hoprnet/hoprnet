use crate::errors::Result;
use async_trait::async_trait;
use core_crypto::types::Hash;
use futures::Stream;
use primitive_types::H256;
use utils_types::primitives::{Address, Balance, BalanceType, U256};

pub use ethers::types::transaction::eip2718::TypedTransaction;
pub use ethers::types::TxHash;

pub mod errors;
//pub mod rpc;

//#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
//pub mod nodejs;

/// A type containing selected fields from  the `eth_getBlockByHash`/`eth_getBlockByNumber` RPC
/// calls.
#[derive(Debug, Clone)]
pub struct Block {
    /// Block number or `None` if pending.
    pub number: Option<u64>,
    /// Block hash or `None` if pending.
    pub hash: Option<Hash>,
    /// Block timestamp
    pub timestamp: U256,
    /// Transaction hashes within this block
    pub transactions: Vec<Hash>,
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
    pub address: Address,
    pub topics: Vec<Hash>,
    pub data: Box<[u8]>,
    pub tx_index: Option<u64>,
    pub block_number: Option<u64>,
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

/// Represents a query to extract logs containing specific contract events.
#[derive(Debug, Clone)]
pub struct EventsQuery {
    /// Contract address
    pub address: Address,
    /// Event topics
    pub topics: Vec<TxHash>,
    /// Start block number
    pub from: u64,
    /// End block number
    pub to: u64,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprRpcOperations {
    type BlockStream: Stream<Item = Block>;

    type LogStream: Stream<Item = Log>;

    async fn genesis_block(&self) -> Result<u64>;

    async fn block_number(&self) -> Result<u64>;

    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>>;

    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance>;

    async fn get_transactions_in_block(&self, block_number: u64) -> Result<Vec<Hash>>;

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>>;

    async fn get_safe_from_node_safe_registry(&self, node: Address) -> Result<Address>;

    async fn get_module_target_address(&self) -> Result<Address>;

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash>;

    async fn subscribe_blocks(&self) -> Result<Self::BlockStream>;

    async fn subscribe_logs(&self, query: EventsQuery) -> Result<Self::LogStream>;
}
