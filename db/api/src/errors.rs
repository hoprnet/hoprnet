use hopr_internal_types::errors::CoreTypesError;
use sea_orm::TransactionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("db contains data which cannot be converted to business object")]
    CorruptedData,

    #[error("transaction error: {0}")]
    TransactionError(String),

    #[error("error while decoding db entity")]
    DecodingError,

    #[error("logical error: {0}")]
    LogicalError(String),

    #[error(transparent)]
    BackendError(#[from] sea_orm::DbErr),

    #[error(transparent)]
    CoreTypesError(#[from] CoreTypesError),

    #[error(transparent)]
    NonSpecificError(#[from] hopr_primitive_types::errors::GeneralError),
}

impl<E: std::error::Error> From<TransactionError<E>> for DbError {
    fn from(value: TransactionError<E>) -> Self {
        DbError::TransactionError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DbError>;
