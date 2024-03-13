use thiserror::Error;

/// Error representing all possible erroneous states of the entire HOPR
/// on-chain interactions.
#[derive(Error, Debug)]
pub enum HoprChainError {
    #[error("API error: {0}")]
    Api(String),

    #[error("rpc error: {0}")]
    Rpc(#[from] chain_rpc::errors::RpcError),

    #[error("indexer error: {0}")]
    Indexer(#[from] chain_indexer::errors::CoreEthereumIndexerError),

    #[error("Db error: {0}")]
    Db(#[from] utils_db::errors::DbError),
}

/// The default [Result] object translating errors in the [HoprChainError] type
pub type Result<T> = core::result::Result<T, HoprChainError>;
