use ethers::prelude::nonce_manager::NonceManagerError;
use ethers::prelude::signer::SignerMiddlewareError;
use ethers::prelude::ContractError;
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
