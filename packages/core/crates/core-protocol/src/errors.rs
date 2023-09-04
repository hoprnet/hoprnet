use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ProtocolError {
    #[error("tx queue is full, retry later")]
    Retry,

    #[error("underlying transport error while sending packet: {0}")]
    TransportError(String),

    #[error("Failed to notify an external process: {0}")]
    Notification(String),

    #[error("Ticket aggregation error: {0}")]
    ProtocolTicketAggregation(String),

    #[error("Failed on a logical error: {0}")]
    Logic(String),
}

pub type Result<T> = core::result::Result<T, ProtocolError>;
