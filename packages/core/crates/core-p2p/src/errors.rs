use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2PError {
    #[error("Failed to notify an external process: {0}")]
    Notification(String),

    #[error("Failed on a logical error: {0}")]
    Logic(String),

    #[error("I/O error encountered")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, P2PError>;
