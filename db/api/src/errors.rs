use std::sync::Arc;
use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::tickets::Ticket;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("DB general error: {0}")]
    General(String),

    #[error("channel not found: {0}")]
    ChannelNotFound(Hash),

    #[error("cannot find a surb: {0}")]
    NoSurbAvailable(String),

    #[error("ticket validation error for {:?}: {}", 0.0, 0.1)]
    TicketValidationError(Box<(Ticket, String)>),

    #[error("failed to process acknowledgement: {0}")]
    AcknowledgementValidationError(String),

    #[error("logical error: {0}")]
    LogicalError(String),

    // Solves the issue when the message producer can send arbitrary data that cannot be decoded
    // but would then be acknowledged, leading to potentially asymmetrical work on the receiver
    #[error("adversarial behavior detected: {0}")]
    PossibleAdversaryError(String),

    #[error("sql error: {0}")]
    SqlError(Box<dyn std::error::Error + Send + Sync>),

    #[error("error while inserting into cache: {0}")]
    CacheError(#[from] Arc<Self>),
    
    #[error(transparent)]
    TypeError(#[from] hopr_primitive_types::errors::GeneralError),

    #[error(transparent)]
    CoreTypeError(#[from] hopr_internal_types::errors::CoreTypesError),

    // --------------------
    // TODO: (dbmig) remove these once hopr-db-sql crate is removed in 4.0
    #[error("log status not found")]
    MissingLogStatus,

    #[error("log not found")]
    MissingLog,

    #[error("addresses and topics used to fetch logs are inconsistent")]
    InconsistentLogs,

    #[error("account entry for announcement not found")]
    MissingAccount,
}

pub type Result<T> = std::result::Result<T, DbError>;
