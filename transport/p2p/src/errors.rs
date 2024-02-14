use thiserror::Error;

/// All errors raised by the crate.
#[derive(Error, Debug, PartialEq)]
pub enum P2PError {
    #[error("Failed to notify an external process: {0}")]
    Notification(String),

    #[error("Heartbeat protocol failure: {0}")]
    ProtocolHeartbeat(String),

    #[error("Failed on a logical error: {0}")]
    Logic(String),

    #[error("libp2p failed with: {0}")]
    Libp2p(String),
}

/// Result utilizing the [P2PError] as the error type.
pub type Result<T> = core::result::Result<T, P2PError>;
