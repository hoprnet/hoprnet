use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::tickets::Ticket;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("DB general error: {0}")]
    General(String),

    #[error("account entry for announcement not found")]
    MissingAccount,

    #[error("channel not found: {0}")]
    ChannelNotFound(Hash),

    #[error("ticket aggregation error: {0}")]
    TicketAggregationError(String),

    #[error("ticket validation error for {0:?}")]
    TicketValidationError(Box<(Ticket, String)>),

    #[error("logical error: {0}")]
    LogicalError(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
