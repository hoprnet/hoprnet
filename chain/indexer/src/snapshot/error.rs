use thiserror::Error;

/// Error types for snapshot operations
#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Archive extraction error: {0}")]
    Archive(String),

    #[error("SQLite validation error: {0}")]
    Validation(String),

    #[error("Invalid snapshot format: {0}")]
    InvalidFormat(String),

    #[error("Insufficient disk space: required {required}, available {available}")]
    InsufficientSpace { required: u64, available: u64 },

    #[error("Snapshot too large: {size} bytes exceeds maximum {max_size} bytes")]
    TooLarge { size: u64, max_size: u64 },

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("HTTP response error: status {status}")]
    HttpStatus { status: u16 },
}

/// Result type for snapshot operations
pub type SnapshotResult<T> = Result<T, SnapshotError>;
