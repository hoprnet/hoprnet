use crate::initiation::StartErrorReason;
use thiserror::Error;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum TransportSessionError {
    #[error("connection timed out")]
    Timeout,

    #[error("application tag from disallowed range")]
    Tag,

    #[error("incorrect data size")]
    PayloadSize,

    #[error("serializer error: {0}")]
    Serializer(#[from] bincode::Error),

    #[error("invalid peer id")]
    PeerId,

    #[error("impossible transport path")]
    Path,

    #[error("the other party rejected session initiation with error: {0}")]
    Rejected(StartErrorReason),

    #[error("session manager error: {0}")]
    Manager(String),

    #[error(transparent)]
    Network(#[from] hopr_network_types::errors::NetworkTypeError),

    #[error("session is closed")]
    Closed,
}

pub type Result<T> = std::result::Result<T, TransportSessionError>;
