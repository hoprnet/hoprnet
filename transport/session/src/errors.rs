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

    #[error("session is closed")]
    Closed,
}

pub type Result<T> = std::result::Result<T, TransportSessionError>;
