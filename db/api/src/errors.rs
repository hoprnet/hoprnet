use hopr_crypto_types::{prelude::CryptoError, types::Hash};
use hopr_db_entity::errors::DbEntityError;
use hopr_internal_types::errors::CoreTypesError;
use sea_orm::TransactionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("account entry for announcement not found")]
    MissingAccount,

    #[error("channel not found: {0}")]
    ChannelNotFound(Hash),

    #[error("missing fixed entry in table {0}")]
    MissingFixedTableEntry(String),

    #[error("transaction error: {0}")]
    TransactionError(Box<dyn std::error::Error + Send + Sync>),

    #[error("error while decoding db entity")]
    DecodingError,

    #[error("logical error: {0}")]
    LogicalError(String),

    #[error("ack validation error: {0}")]
    AcknowledgementValidationError(String),

    #[error("ticket validation error: {0}")]
    TicketValidationError(String),

    #[error(transparent)]
    BackendError(#[from] sea_orm::DbErr),

    #[error(transparent)]
    CoreTypesError(#[from] CoreTypesError),

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    EntityError(#[from] DbEntityError),

    #[error(transparent)]
    NonSpecificError(#[from] hopr_primitive_types::errors::GeneralError),
}

impl<E: std::error::Error + Send + Sync + 'static> From<TransactionError<E>> for DbError {
    fn from(value: TransactionError<E>) -> Self {
        match value {
            TransactionError::Connection(e) => Self::BackendError(e),
            TransactionError::Transaction(e) => Self::TransactionError(e.into()),
        }
    }
}

pub type Result<T> = std::result::Result<T, DbError>;
