use thiserror::Error;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug, PartialEq)]
pub enum TransportSessionError {
    #[error("Connection timed out")]
    Timeout,
    #[error("Application tag from unallowed range")]
    Tag,
    #[error("Incorrect data size")]
    PayloadSize,
    #[error("Invalid peer id")]
    PeerId,
    #[error("Impossible transport path")]
    Path,
    #[error("Session is closed")]
    Closed,
}

pub type Result<T> = std::result::Result<T, TransportSessionError>;
