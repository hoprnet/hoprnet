use async_trait::async_trait;
use core_crypto::types::Hash;
use primitive_types::H256;
use std::fmt::{Display, Formatter};
use utils_types::primitives::{Address, Balance, BalanceType, U256};

use crate::errors::Result;

pub use ethers::types::transaction::eip2718::TypedTransaction;
pub use ethers::types::TxHash;
use futures::channel::mpsc::UnboundedReceiver;
use utils_types::traits::BinarySerializable;

pub mod errors;
pub mod rpc;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod nodejs;
mod indexer;

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
            "block {} ({}) with {} txs",
            self.number.map(|i| i.to_string()).unwrap_or("pending".into()),
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
        write!(f, "log of contract address {} with {} topics", self.address, self.topics.len())
    }
}

#[derive(Debug, Clone)]
pub struct BlockWithLogs {
    pub block: Block,
    pub logs: Vec<Log>
}

/// Represents a query to extract logs containing specific contract events.
#[derive(Debug, Clone)]
pub struct EventsQuery {
    /// Contract address
    pub address: Address,
    /// Event topics
    pub topics: Vec<TxHash>,
}

impl From<EventsQuery> for ethers::types::Filter {
    fn from(value: EventsQuery) -> Self {
        let addr: ethers::types::H160 = value.address.into();
        let mut ret = ethers::types::Filter::new()
            .address::<ethers::types::H160>(addr.into());

        for i in 0..4.min(value.topics.len()) {
            ret.topics[i] = Some(value.topics[i].into())
        }

        ret
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprRpcOperations {
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>>;

    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance>;

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>>;

    async fn get_safe_from_node_safe_registry(&self, node: Address) -> Result<Address>;

    async fn get_module_target_address(&self) -> Result<Address>;

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprIndexerRpcOperations: HoprRpcOperations {
    async fn block_number(&self) -> Result<u64>;

    async fn poll_blocks_with_logs(&self, start_block_number: Option<u64>, filter: EventsQuery) -> Result<UnboundedReceiver<BlockWithLogs>>;

}