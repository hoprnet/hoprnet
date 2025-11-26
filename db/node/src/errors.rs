use std::sync::Arc;

use hopr_crypto_packet::errors::TicketValidationError;
use sea_orm::TransactionError;

#[derive(Debug, thiserror::Error)]
pub enum NodeDbError {
    #[error("cannot find a surb: {0}")]
    NoSurbAvailable(String),

    #[error("ticket validation error for {}: {}", .0.ticket, .0.reason)]
    TicketValidationError(#[from] TicketValidationError),

    #[error("failed to process acknowledgement: {0}")]
    AcknowledgementValidationError(String),

    #[error("logical error: {0}")]
    LogicalError(String),

    #[error("orm error: {0}")]
    Orm(#[from] sea_orm::error::DbErr),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] sqlx::Error),

    #[error("other error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),

    #[error("error while inserting into cache: {0}")]
    Cache(#[from] Arc<Self>),

    #[error("custom error: {0}")]
    Custom(String),

    #[error(transparent)]
    EntityError(#[from] hopr_db_entity::errors::DbEntityError),

    #[error(transparent)]
    PacketError(#[from] hopr_crypto_packet::errors::PacketError),

    #[error(transparent)]
    TypeError(#[from] hopr_primitive_types::errors::GeneralError),

    #[error(transparent)]
    CoreTypeError(#[from] hopr_internal_types::errors::CoreTypesError),

    #[error(transparent)]
    CryptoError(#[from] hopr_crypto_types::errors::CryptoError),
}

impl<E: std::error::Error + Send + Sync + 'static> From<TransactionError<E>> for NodeDbError {
    fn from(value: TransactionError<E>) -> Self {
        match value {
            TransactionError::Connection(c) => Self::Orm(c),
            TransactionError::Transaction(e) => Self::Other(e.into()),
        }
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<Arc<TransactionError<E>>> for NodeDbError {
    fn from(value: Arc<TransactionError<E>>) -> Self {
        Self::Custom(format!("in-cache transaction error: {value}"))
    }
}
