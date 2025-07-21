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
    #[error(
        "Network error: {0}.\nSuggestion: Check your internet connection and verify the snapshot URL is accessible."
    )]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}.\nSuggestion: Ensure the target directory exists and has proper write permissions.")]
    Io(#[from] std::io::Error),

    #[error(
        "Archive extraction error: {0}.\nSuggestion: The snapshot file may be corrupted. Try downloading it again or \
         use a different snapshot URL."
    )]
    Archive(String),

    #[error(
        "SQLite validation error: {0}.\nSuggestion: The snapshot database may be corrupted or incompatible. Try using \
         a different snapshot or disable snapshots with --noLogSnapshot."
    )]
    Validation(String),

    #[error(
        "Invalid snapshot format: {0}.\nSuggestion: The snapshot file format is not supported. Ensure you're using a \
         valid tar.gz snapshot file."
    )]
    InvalidFormat(String),

    #[error(
        "Insufficient disk space: required {required} MB, available {available} MB.\nSuggestion: Free up disk space \
         or use a different data directory with more available space."
    )]
    InsufficientSpace { required: u64, available: u64 },

    #[error(
        "Snapshot too large: {size} bytes exceeds maximum {max_size} bytes.\nSuggestion: The snapshot file is \
         unusually large. Verify the snapshot URL or increase size limits if this is expected."
    )]
    TooLarge { size: u64, max_size: u64 },

    #[error("Task join error: {0}.\nSuggestion: Internal error occurred. Check system resources and try again.")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error(
        "HTTP response error: status {status}.\nSuggestion: Server returned error {status}. Check if the snapshot URL \
         is correct and the server is accessible."
    )]
    HttpStatus { status: u16 },

    #[error(
        "Invalid data: {0}.\nSuggestion: The snapshot data is invalid or corrupted. Try downloading again or use a \
         different snapshot source."
    )]
    InvalidData(String),

    #[error(
        "Configuration error: {0}.\nSuggestion: Check your configuration settings and ensure all required parameters \
         are set correctly."
    )]
    Configuration(String),

    #[error(
        "Timeout error: {0}.\nSuggestion: The operation timed out. Check your network connection or increase timeout \
         values."
    )]
    Timeout(String),
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
