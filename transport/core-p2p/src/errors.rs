use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum P2PError {
    #[error("Failed to notify an external process: {0}")]
    Notification(String),

    #[error("Heartbeat protocol failure: {0}")]
    ProtocolHeartbeat(String),

    #[error("Failed on a logical error: {0}")]
    Logic(String),
}

pub type Result<T> = core::result::Result<T, P2PError>;
