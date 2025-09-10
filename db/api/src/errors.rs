use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::tickets::Ticket;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("DB general error: {0}")]
    General(String),

    #[error("log status not found")]
    MissingLogStatus,

    #[error("log not found")]
    MissingLog,

    #[error("addresses and topics used to fetch logs are inconsistent")]
    InconsistentLogs,

    #[error("account entry for announcement not found")]
    MissingAccount,

    #[error("channel not found: {0}")]
    ChannelNotFound(Hash),

    #[error("cannot find a surb: {0}")]
    NoSurbAvailable(String),

    #[error("ticket aggregation error: {0}")]
    TicketAggregationError(String),

    #[error("ticket validation error for {:?}: {}", 0.0, 0.1)]
    TicketValidationError(Box<(Ticket, String)>),

    #[error("logical error: {0}")]
    LogicalError(String),

    // Solves the issue when the message producer can send arbitrary data that cannot be decoded
    // but would then be acknowledged, leading to potentially asymmetrical work on the receiver
    #[error("adversarial behavior detected: {0}")]
    PossibleAdversaryError(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
