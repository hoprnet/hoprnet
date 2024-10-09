use sea_orm::TransactionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("alias not found: {0}")]
    AliasNotFound(String),

    #[error("transaction error: {0}")]
    TransactionError(Box<dyn std::error::Error + Send + Sync>),

    #[error("logical error: {0}")]
    LogicalError(String),

    #[error(transparent)]
    BackendError(#[from] sea_orm::DbErr),
}

impl<E: std::error::Error + Send + Sync + 'static> From<TransactionError<E>> for DbError {
    fn from(value: TransactionError<E>) -> Self {
        match value {
            TransactionError::Connection(e) => Self::BackendError(e),
            TransactionError::Transaction(e) => Self::TransactionError(Box::new(e)),
        }
    }
}

pub type Result<T> = std::result::Result<T, DbError>;
