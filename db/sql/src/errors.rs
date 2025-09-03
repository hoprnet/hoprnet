use std::sync::Arc;

use hopr_crypto_packet::errors::TicketValidationError;
use hopr_crypto_types::{prelude::CryptoError, types::Hash};
use hopr_db_entity::errors::DbEntityError;
use hopr_internal_types::{errors::CoreTypesError, tickets::Ticket};
use sea_orm::TransactionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbSqlError {
    #[error("failed to construct the database: {0}")]
    Construction(String),

    #[error("log status not found")]
    MissingLogStatus,

    #[error("log not found")]
    MissingLog,

    #[error("list of logs is empty")]
    EmptyLogsList,

    #[error("log status could not be updated")]
    UpdateLogStatusError,

    #[error("account entry for announcement not found")]
    MissingAccount,

    #[error("channel not found: {0}")]
    ChannelNotFound(Hash),

    #[error("can't edit a corrupted channel: {0}")]
    CorruptedChannelEntry(Hash),

    #[error("missing fixed entry in table {0}")]
    MissingFixedTableEntry(String),

    #[error("ticket aggregation error: {0}")]
    TicketAggregationError(String),

    #[error("ticket validation error for {}: {}", .0.0, .0.1)]
    TicketValidationError(Box<(Ticket, String)>),

    #[error("transaction error: {0}")]
    TransactionError(Box<dyn std::error::Error + Send + Sync>),

    #[error("error while decoding db entity")]
    DecodingError,

    #[error("logical error: {0}")]
    LogicalError(String),

    #[error("ack validation error: {0}")]
    AcknowledgementValidationError(String),

    #[error(transparent)]
    BackendError(#[from] sea_orm::DbErr),

    #[error(transparent)]
    CoreTypesError(#[from] CoreTypesError),

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    EntityError(#[from] DbEntityError),

    #[error("error while inserting into cache: {0}")]
    CacheError(#[from] Arc<Self>),

    #[error(transparent)]
    NonSpecificError(#[from] hopr_primitive_types::errors::GeneralError),

    #[error(transparent)]
    ApiError(#[from] hopr_db_api::errors::DbError),
}

impl From<TicketValidationError> for DbSqlError {
    fn from(value: TicketValidationError) -> Self {
        DbSqlError::TicketValidationError(Box::new((*value.ticket, value.reason)))
    }
}

impl From<DbSqlError> for hopr_db_api::errors::DbError {
    fn from(value: DbSqlError) -> Self {
        hopr_db_api::errors::DbError::General(value.to_string())
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<TransactionError<E>> for DbSqlError {
    fn from(value: TransactionError<E>) -> Self {
        match value {
            TransactionError::Connection(e) => Self::BackendError(e),
            TransactionError::Transaction(e) => Self::TransactionError(e.into()),
        }
    }
}

pub type Result<T> = std::result::Result<T, DbSqlError>;
