use ethers::prelude::nonce_manager::NonceManagerError;
use ethers::prelude::signer::SignerMiddlewareError;
use ethers::prelude::ContractError;
use ethers_providers::{JsonRpcError, ProviderError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("contract error: {0}")]
    ContractError(String),

    #[error("middleware error: {0}")]
    MiddlewareError(String),

    #[error("block with such id does not (yet) exist")]
    NoSuchBlock,

    #[error("filter does not contain any criteria")]
    FilterIsEmpty,

    #[error("non-specific RPC error occurred: {0}")]
    GeneralError(String),

    #[error(transparent)]
    KeypairError(#[from] ethers::signers::WalletError),

    #[error(transparent)]
    ProviderError(#[from] ethers_providers::ProviderError),
}

pub type Result<T> = std::result::Result<T, RpcError>;

impl<M> From<NonceManagerError<M>> for RpcError
where
    M: ethers::middleware::Middleware,
{
    fn from(value: NonceManagerError<M>) -> Self {
        Self::MiddlewareError(value.to_string())
    }
}

impl<M, S> From<SignerMiddlewareError<M, S>> for RpcError
where
    M: ethers::middleware::Middleware,
    S: ethers::signers::Signer,
{
    fn from(value: SignerMiddlewareError<M, S>) -> Self {
        Self::MiddlewareError(value.to_string())
    }
}

impl<M> From<ContractError<M>> for RpcError
where
    M: ethers::middleware::Middleware,
{
    fn from(value: ContractError<M>) -> Self {
        Self::ContractError(value.to_string())
    }
}

/// Error abstraction for `HttpRequestor`.
#[derive(Error, Debug)]
pub enum HttpRequestError {
    #[error("error on js-wasm interface: {0}")]
    InterfaceError(String),

    #[error("connection timed out")]
    Timeout,

    #[error("http error - status {0}")]
    HttpError(u16),

    #[error("unrecognized error: {0}")]
    UnknownError(String),
}

/// Errors for `JsonRpcProviderClient`
#[derive(Error, Debug)]
pub enum JsonRpcProviderClientError {
    #[error("Deserialization Error: {err}. Response: {text}")]
    /// Serde JSON Error
    SerdeJson {
        /// Underlying error
        err: serde_json::Error,
        /// The contents of the HTTP response that could not be deserialized
        text: String,
    },

    #[error(transparent)]
    JsonRpcError(#[from] JsonRpcError),

    #[error(transparent)]
    BackendError(#[from] HttpRequestError),
}

impl From<JsonRpcProviderClientError> for ProviderError {
    fn from(src: JsonRpcProviderClientError) -> Self {
        match src {
            // Because we cannot use `ProviderError::HTTPError`, due to `request::Error` having private constructor
            // we must resolve connectivity error within our `RetryPolicy<JsonRpcProviderClientError>`
            JsonRpcProviderClientError::BackendError(err) => ProviderError::CustomError(err.to_string()),
            _ => ProviderError::JsonRpcClientError(Box::new(src)),
        }
    }
}

impl ethers::providers::RpcError for JsonRpcProviderClientError {
    fn as_error_response(&self) -> Option<&JsonRpcError> {
        if let JsonRpcProviderClientError::JsonRpcError(err) = self {
            Some(err)
        } else {
            None
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            JsonRpcProviderClientError::SerdeJson { err, .. } => Some(err),
            _ => None,
        }
    }
}
