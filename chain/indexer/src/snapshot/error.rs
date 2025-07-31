//! Error types for snapshot operations with user-friendly messages.
//!
//! This module defines comprehensive error types for all snapshot operations,
//! providing clear error messages and actionable suggestions for users.

use thiserror::Error;

/// Comprehensive error type for snapshot operations with actionable guidance.
///
/// Each error variant includes:
/// - A clear description of what went wrong
/// - Suggestions for how users can resolve the issue
/// - Relevant context information (e.g., required vs available space)
///
/// # Error Categories
///
/// - **Network errors**: Download failures, connectivity issues
/// - **IO errors**: File system permissions, disk operations
/// - **Validation errors**: Data integrity, format issues
/// - **Resource errors**: Disk space, file size limits
/// - **Configuration errors**: Invalid settings, missing parameters
#[derive(Error, Debug)]
pub enum SnapshotError {
    /// Network-related errors during download operations.
    /// Includes connection failures, DNS issues, and request timeouts.
    #[error("Network error: {0}.")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}.")]
    Io(#[from] std::io::Error),

    #[error("Archive extraction error: {0}.")]
    Archive(String),

    #[error("SQLite validation error: {0}.")]
    Validation(String),

    #[error("Invalid snapshot format: {0}.")]
    InvalidFormat(String),

    #[error("Insufficient disk space: required {required} MB, available {available} MB.")]
    InsufficientSpace { required: u64, available: u64 },

    #[error("Snapshot too large: {size} bytes exceeds maximum {max_size} bytes.")]
    TooLarge { size: u64, max_size: u64 },

    #[error("HTTP response error: status {status}.")]
    HttpStatus { status: u16 },

    #[error("Invalid data: {0}.")]
    InvalidData(String),

    #[error("Configuration error: {0}.")]
    Configuration(String),

    #[error("Timeout error: {0}.")]
    Timeout(String),

    #[error("Database installation error: {0}.")]
    Installation(String),

    #[error("The snapshot file is missing the required Content-Length header.")]
    ContentLengthMissing(),
}

/// Specialized `Result` type for snapshot operations.
///
/// This type alias simplifies error handling throughout the snapshot module
/// by providing a consistent return type for all operations.
///
/// # Example
///
/// ```no_run
/// use hopr_chain_indexer::snapshot::error::SnapshotResult;
///
/// async fn download_file(url: &str) -> SnapshotResult<Vec<u8>> {
///     // Implementation that may return various SnapshotError variants
///     Ok(vec![])
/// }
/// ```
pub type SnapshotResult<T> = Result<T, SnapshotError>;
