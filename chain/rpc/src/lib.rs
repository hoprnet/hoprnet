//! This crate contains types and traits that ensure correct interfacing with Ethereum RPC providers.
//!
//! The most important trait is [HoprRpcOperations] which allows to send arbitrary on-chain transactions
//! and also to perform the selection of HOPR-related smart contract operations.
//! Secondly, the [HoprIndexerRpcOperations] is a trait that contains all operations required by the
//! Indexer to subscribe to the block with logs from the chain.
//!
//! Both of these traits implemented and realized via the [RpcOperations](rpc::RpcOperations) type,
//! so this represents the main entry point to all RPC related operations.

extern crate core;

use alloy::primitives::B256;
use alloy::providers::PendingTransaction;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use futures::Stream;
use http_types::convert::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::time::Duration;

use hopr_crypto_types::types::Hash;
use hopr_primitive_types::prelude::*;

use crate::errors::Result;
use crate::RetryAction::NoRetry;

pub mod client;
pub mod errors;
mod helper;
pub mod indexer;
pub mod rpc;
pub mod transport;

pub use crate::transport::ReqwestClient;

/// A type containing selected fields from  the `eth_getLogs` RPC calls.
///
/// This is further restricted to already mined blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Corresponding block hash
    pub block_hash: Hash,
    /// Corresponding transaction hash
    pub tx_hash: Hash,
    /// Log index
    pub log_index: U256,
    /// Removed flag
    pub removed: bool,
}

impl From<alloy::rpc::types::Log> for Log {
    fn from(value: alloy::rpc::types::Log) -> Self {
        Self {
            address: value.inner.address.into(),
            topics: value.inner.topics().iter().map(|t| Hash::from(t.0)).collect(),
            data: Box::from(value.inner.data.data.as_ref()),
            tx_index: value.transaction_index.expect("tx index must be present"),
            block_number: value.block_number.expect("block id must be present"),
            block_hash: value.block_hash.expect("block hash must be present").0.into(),
            log_index: value.log_index.expect("log index must be present").into(),
            tx_hash: value.transaction_hash.expect("tx hash must be present").0.into(),
            removed: value.removed,
        }
    }
}

// impl From<Log> for alloy::rpc::types::Log {
//     fn from(value: Log) -> Self {
//         alloy::rpc::types::Log {
//             address: value.address.into(),
//             topics: value.inner.topics().into_iter().map(|t| Hash::from(t.0)).collect(),
//             data: Box::from(value.inner.data.data.as_ref()),
//             transaction_index: Some(value.tx_index),
//             block_number: value.block_number.expect("block id must be present"),
//             block_hash: value.block_hash.expect("block hash must be present").0.into(),
//             log_index: Some(Into::<u64>::into(value.log_index)),
//             transaction_hash: Some(B256::from_slice(value.tx_hash.as_ref())),
//             removed: value.removed,
//         }
//     }
// }

impl From<Log> for alloy::rpc::types::RawLog {
    fn from(value: Log) -> Self {
        alloy::rpc::types::RawLog {
            address: value.address.into(),
            topics: value.topics.into_iter().map(|h| B256::from_slice(h.as_ref())).collect(),
            data: value.data.into(),
        }
    }
}

impl From<SerializableLog> for Log {
    fn from(value: SerializableLog) -> Self {
        let topics = value
            .topics
            .into_iter()
            .map(|topic| topic.into())
            .collect::<Vec<Hash>>();

        Self {
            address: value.address,
            topics,
            data: Box::from(value.data.as_ref()),
            tx_index: value.tx_index,
            block_number: value.block_number,
            block_hash: value.block_hash.into(),
            log_index: value.log_index.into(),
            tx_hash: value.tx_hash.into(),
            removed: value.removed,
        }
    }
}

impl From<Log> for SerializableLog {
    fn from(value: Log) -> Self {
        SerializableLog {
            address: value.address,
            topics: value.topics.into_iter().map(|t| t.into()).collect(),
            data: value.data.into_vec(),
            tx_index: value.tx_index,
            block_number: value.block_number,
            block_hash: value.block_hash.into(),
            tx_hash: value.tx_hash.into(),
            log_index: value.log_index.as_u64(),
            removed: value.removed,
            // These fields stay empty for logs coming from the chain and will be populated by the
            // indexer when processing the log.
            processed: None,
            processed_at: None,
            checksum: None,
        }
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "log #{} in tx #{} in block #{} of address {} with {} topics",
            self.log_index,
            self.tx_index,
            self.block_number,
            self.address,
            self.topics.len()
        )
    }
}

impl Ord for Log {
    fn cmp(&self, other: &Self) -> Ordering {
        let blocks = self.block_number.cmp(&other.block_number);
        if blocks == Ordering::Equal {
            let tx_indices = self.tx_index.cmp(&other.tx_index);
            if tx_indices == Ordering::Equal {
                self.log_index.cmp(&other.log_index)
            } else {
                tx_indices
            }
        } else {
            blocks
        }
    }
}

impl PartialOrd<Self> for Log {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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

impl From<LogFilter> for alloy::rpc::types::Filter {
    fn from(value: LogFilter) -> Self {
        alloy::rpc::types::Filter::new()
            .address(
                value
                    .address
                    .into_iter()
                    .map(alloy::primitives::Address::from)
                    .collect::<Vec<_>>(),
            )
            .event_signature(
                value
                    .topics
                    .into_iter()
                    .map(|h| alloy::primitives::B256::from_slice(h.as_ref()))
                    .collect::<Vec<_>>(),
            )
    }
}

/// Indicates what retry action should be taken, as result of a `RetryPolicy` implementation.
pub enum RetryAction {
    /// Request should not be retried
    NoRetry,
    /// Request should be retried after the given duration has elapsed.
    RetryAfter(Duration),
}

/// Simple retry policy trait
pub trait RetryPolicy<E> {
    /// Indicates whether a client should retry the request given the last error, current number of retries
    /// of this request and the number of other requests being retried by the client at this time.
    fn is_retryable_error(&self, _err: &E, _retry_number: u32, _retry_queue_size: u32) -> RetryAction {
        NoRetry
    }
}

// /// Performs no retries.
// #[derive(Clone, Debug)]
// pub struct ZeroRetryPolicy<E>(PhantomData<E>);

// impl<E> Default for ZeroRetryPolicy<E> {
//     fn default() -> Self {
//         Self(PhantomData)
//     }
// }

// impl<E> RetryPolicy<E> for ZeroRetryPolicy<E> {}

// /// Abstraction for an HTTP client that performs HTTP POST with serializable request data.
// #[async_trait]
// pub trait HttpRequestor: std::fmt::Debug + Send + Sync {
//     /// Performs HTTP request with optional JSON data to the given URL
//     /// and gets the JSON response.
//     async fn http_query<T>(
//         &self,
//         method: http_types::Method,
//         url: &str,
//         data: Option<T>,
//     ) -> std::result::Result<Box<[u8]>, HttpRequestError>
//     where
//         T: Serialize + Send + Sync;

//     /// Performs HTTP POST of JSON data to the given URL
//     /// and gets the JSON response.
//     async fn http_post<T>(&self, url: &str, data: T) -> std::result::Result<Box<[u8]>, HttpRequestError>
//     where
//         T: Serialize + Send + Sync,
//     {
//         self.http_query(http_types::Method::Post, url, Some(data)).await
//     }

//     /// Performs HTTP GET query to the given URL
//     /// and gets the JSON response.
//     async fn http_get(&self, url: &str) -> std::result::Result<Box<[u8]>, HttpRequestError> {
//         self.http_query(http_types::Method::Get, url, Option::<()>::None).await
//     }
// }

/// Common configuration for all native `HttpPostRequestor`s
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, smart_default::SmartDefault)]
pub struct HttpPostRequestorConfig {
    /// Timeout for HTTP POST request
    ///
    /// Defaults to 30 seconds.
    #[default(Duration::from_secs(30))]
    pub http_request_timeout: Duration,

    /// Maximum number of HTTP redirects to follow
    ///
    /// Defaults to 3
    #[default(3)]
    pub max_redirects: u8,

    /// Maximum number of requests per second.
    /// If set to Some(0) or `None`, there will be no limit.
    ///
    /// Defaults to 10
    #[default(Some(10))]
    pub max_requests_per_sec: Option<u32>,
}

/// Represents the on-chain status for the Node Safe module.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct NodeSafeModuleStatus {
    pub is_node_included_in_module: bool,
    pub is_module_enabled_in_safe: bool,
    pub is_safe_owner_of_module: bool,
}

impl NodeSafeModuleStatus {
    /// Determines if the node passes all status checks.
    pub fn should_pass(&self) -> bool {
        self.is_node_included_in_module && self.is_module_enabled_in_safe && self.is_safe_owner_of_module
    }
}

/// Trait defining a general set of operations an RPC provider
/// must provide to the HOPR node.
#[async_trait]
pub trait HoprRpcOperations {
    /// Retrieves the timestamp from the given block number.
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>>;

    /// Retrieves the node's account balance of the given type.
    async fn get_balance(&self, address: Address, balance_type: BalanceType) -> Result<Balance>;

    /// Retrieves the minimum incoming ticket winning probability by directly
    /// calling the network's winning probability oracle.
    async fn get_minimum_network_winning_probability(&self) -> Result<f64>;

    /// Retrieves the minimum ticket prices by directly calling the network's
    /// ticket price oracle.
    async fn get_minimum_network_ticket_price(&self) -> Result<Balance>;

    /// Retrieves the node's eligibility status
    async fn get_eligibility_status(&self, address: Address) -> Result<bool>;

    /// Retrieves information of the given node module's target.
    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>>;

    /// Retrieves the safe address of the given node address from the registry.
    async fn get_safe_from_node_safe_registry(&self, node: Address) -> Result<Address>;

    /// Retrieves the target address of the node module.
    async fn get_module_target_address(&self) -> Result<Address>;

    /// Retrieves the notice period of channel closure from the Channels contract.
    async fn get_channel_closure_notice_period(&self) -> Result<Duration>;

    /// Retrieves the on-chain status of node, safe, and module.
    async fn check_node_safe_module_status(&self, node_address: Address) -> Result<NodeSafeModuleStatus>;

    /// Sends transaction to the RPC provider.
    async fn send_transaction(&self, tx: TransactionRequest) -> Result<PendingTransaction>;
}

/// Structure containing filtered logs that all belong to the same block.
#[derive(Debug, Clone, Default)]
pub struct BlockWithLogs {
    /// Block number
    pub block_id: u64,
    /// Filtered logs belonging to this block.
    pub logs: BTreeSet<SerializableLog>,
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
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HoprIndexerRpcOperations {
    /// Retrieves the latest block number.
    async fn block_number(&self) -> Result<u64>;

    /// Starts streaming logs from the given `start_block_number`.
    /// If no `start_block_number` is given, the stream starts from the latest block.
    /// The given `filter` are applied to retrieve the logs, the function fails if the filter is empty.
    /// The streaming stops only when the corresponding channel is closed by the returned receiver.
    fn try_stream_logs<'a>(
        &'a self,
        start_block_number: u64,
        filter: LogFilter,
    ) -> Result<Pin<Box<dyn Stream<Item = BlockWithLogs> + Send + 'a>>>;
}
