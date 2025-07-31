use alloy::{
    contract::Error as AlloyContractError,
    providers::{MulticallError, PendingTransactionError},
    transports::{RpcError as AlloyRpcError, TransportErrorKind},
};
use hopr_crypto_types::prelude::Hash;
/// Errors produced by this crate and other error-related types.
use thiserror::Error;

/// Enumerates different errors produced by this crate.
#[derive(Error, Debug)]
pub enum RpcError {
    #[error(transparent)]
    AlloyRpcError(#[from] AlloyRpcError<TransportErrorKind>),

    #[error(transparent)]
    AlloyContractError(#[from] AlloyContractError),

    #[error(transparent)]
    MulticallError(#[from] MulticallError),

    #[error(transparent)]
    LogConversionError(#[from] LogConversionError),

    #[error(transparent)]
    SignerError(#[from] alloy::signers::Error),

    #[error("multicall inner failure at {0}: {1}")]
    MulticallFailure(usize, String),

    #[error(transparent)]
    PendingTransactionError(#[from] PendingTransactionError),

    #[error("transaction with hash {0} failed on-chain")]
    TransactionFailed(Hash),

    #[error("filter does not contain any criteria")]
    FilterIsEmpty,

    #[error("transaction submission to the RPC provider timed out")]
    Timeout,

    #[error("unknown error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RpcError>;

/// Error abstraction for `HttpRequestor`.
#[derive(Error, Clone, Debug, PartialEq)]
pub enum HttpRequestError {
    #[error("connection timed out")]
    Timeout,

    #[error("http error - status {0}")]
    HttpError(http::StatusCode),

    #[error("io error when performing http request: {0}")]
    TransportError(String),

    #[error("unrecognized error: {0}")]
    UnknownError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LogConversionError {
    #[error("Missing transaction index")]
    MissingTransactionIndex,
    #[error("Missing block number")]
    MissingBlockNumber,
    #[error("Missing block hash")]
    MissingBlockHash,
    #[error("Missing log index")]
    MissingLogIndex,
    #[error("Missing transaction hash")]
    MissingTransactionHash,
}
