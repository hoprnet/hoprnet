use thiserror::Error;

/// Error representing all possible erroneous states of the entire HOPR
/// on-chain interactions.
#[derive(Error, Debug)]
pub enum HoprChainError {
    #[error("API error: {0}")]
    Api(String),

    #[error("rpc error: {0}")]
    Rpc(#[from] hopr_chain_rpc::errors::RpcError),

    #[error("indexer error: {0}")]
    Indexer(#[from] hopr_chain_indexer::errors::CoreEthereumIndexerError),

    #[error(transparent)]
    DbError(#[from] hopr_db_sql::errors::DbSqlError),

    #[error("configuration error: {0}")]
    Configuration(String),
}

/// The default [Result] object translating errors in the [HoprChainError] type
pub type Result<T> = core::result::Result<T, HoprChainError>;
