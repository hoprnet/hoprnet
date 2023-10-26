use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {

    #[error(transparent)]
    ProviderError(#[from] ethers_providers::ProviderError)
}

pub type Result<T> = std::result::Result<T, RpcError>;