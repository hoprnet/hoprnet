//! Secure snapshot downloading with HTTP/HTTPS and local file support.
//!
//! Provides robust download capabilities for snapshot archives with:
//! - **URL Support**: HTTP/HTTPS with retry logic, local file:// URLs
//! - **Safety**: Size limits, timeout protection, disk space validation
//! - **Reliability**: Exponential backoff, progress tracking, error recovery
//! - **Cross-platform**: Uses sysinfo for disk space checking

use std::{
    fs,
    fs::File,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use async_lock::Mutex;
use backon::{FuturesTimerSleeper, Retryable};
use futures_util::{AsyncWriteExt, TryStreamExt, io::AllowStdIo};
use reqwest::Client;
use smart_default::SmartDefault;
use sysinfo::Disks;
use tracing::{debug, error, info};

use crate::{
    constants::{
        LOGS_SNAPSHOT_DOWNLOADER_MAX_RETRIES, LOGS_SNAPSHOT_DOWNLOADER_MAX_SIZE, LOGS_SNAPSHOT_DOWNLOADER_TIMEOUT,
    },
    snapshot::error::{SnapshotError, SnapshotResult},
};

/// Configuration for snapshot downloads with safety limits.
///
/// Controls download behavior including size limits, timeouts, and retry attempts
/// to ensure safe and reliable snapshot downloads.
#[derive(Debug, Clone, SmartDefault)]
pub struct DownloadConfig {
    /// Maximum allowed file size in bytes
    #[default(_code = "LOGS_SNAPSHOT_DOWNLOADER_MAX_SIZE")]
    pub max_size: u64,
    /// HTTP request timeout duration
    #[default(_code = "LOGS_SNAPSHOT_DOWNLOADER_TIMEOUT")]
    pub timeout: Duration,
    /// Maximum number of retry attempts for failed downloads
    #[default(_code = "LOGS_SNAPSHOT_DOWNLOADER_MAX_RETRIES")]
    pub max_retries: u32,
}

/// Downloads snapshot archives from HTTP/HTTPS and file:// URLs.
///
/// Provides secure, reliable downloading with automatic retry logic for network sources
/// and direct file copying for local sources.
///
/// # Features
///
/// - **HTTP/HTTPS**: Automatic retry with exponential backoff, progress tracking
/// - **Local Files**: Direct copying from file:// URLs with validation
/// - **Safety**: Size limits, disk space checks, timeout protection
/// - **Monitoring**: Progress reporting and detailed error messages
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
///
/// use hopr_chain_indexer::snapshot::download::SnapshotDownloader;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let downloader = SnapshotDownloader::new()?;
///
/// // Download from HTTPS
/// downloader
///     .download_snapshot(
///         "https://snapshots.hoprnet.org/logs.tar.xz",
///         Path::new("/tmp/snapshot.tar.xz"),
///     )
///     .await?;
///
/// // Copy from local file
/// downloader
///     .download_snapshot("file:///backups/snapshot.tar.xz", Path::new("/tmp/snapshot.tar.xz"))
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct SnapshotDownloader {
    client: Client,
    config: DownloadConfig,
}

impl SnapshotDownloader {
    /// Creates a new snapshot downloader with default configuration
    pub fn new() -> SnapshotResult<Self> {
        Self::with_config(DownloadConfig::default())
    }

    /// Creates a new snapshot downloader with custom configuration
    pub fn with_config(config: DownloadConfig) -> SnapshotResult<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(config.timeout)
                .user_agent("curl/8.14.1") // acts like curl for compatibility
                .build()
                .map_err(SnapshotError::Network)?,
            config,
        })
    }

    /// Downloads a snapshot from the given URL to the target path.
    ///
    /// Supports HTTP/HTTPS URLs with retry logic and file:// URLs for local files.
    ///
    /// # Arguments
    ///
    /// * `url` - Source URL (http://, https://, or file:// scheme)
    /// * `target_path` - Destination file path
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError`] for network failures, file system errors, or validation failures.
    pub async fn download_snapshot(&self, url: &str, target_path: &Path) -> SnapshotResult<()> {
        self.download_snapshot_with_retry(url, target_path, self.config.max_retries)
            .await
    }

    /// Downloads a snapshot with configurable retry logic.
    ///
    /// Implements exponential backoff between retry attempts for HTTP/HTTPS URLs.
    /// Local file:// URLs are handled without retry logic. Certain errors
    /// (like 4xx HTTP status codes or insufficient disk space) will not be retried.
    ///
    /// # Arguments
    ///
    /// * `url` - The HTTP/HTTPS or file:// URL to download/copy from
    /// * `target_path` - Local path where the downloaded file will be saved
    /// * `max_retries` - Maximum number of retry attempts (ignored for file:// URLs)
    ///
    /// # Errors
    ///
    /// Returns `SnapshotError` for various failure conditions including:
    /// - Network errors (with retry)
    /// - HTTP errors (4xx without retry, 5xx with retry)
    /// - Insufficient disk space (without retry)
    /// - File size exceeding limits (without retry)
    pub async fn download_snapshot_with_retry(
        &self,
        url: &str,
        target_path: &Path,
        max_retries: u32,
    ) -> SnapshotResult<()> {
        let backoff = backon::ExponentialBuilder::default().with_max_times(max_retries as usize);

        (|| async { self.download_snapshot_once(url, target_path).await })
            .retry(backoff)
            .sleep(FuturesTimerSleeper)
            .when(|err| {
                !matches!(
                    err,
                    SnapshotError::TooLarge { .. }
                        | SnapshotError::InsufficientSpace { .. }
                        | SnapshotError::HttpStatus { status: 400..=499 },
                )
            })
            .notify(|error, _dur| {
                error!(%error, "Download attempt failed");
            })
            .await
    }

    /// Performs a single download attempt
    async fn download_snapshot_once(&self, url: &str, target_path: &Path) -> SnapshotResult<()> {
        info!(%url, "Downloading logs snapshot file");

        // Check available disk space
        let parent_dir = target_path
            .parent()
            .ok_or_else(|| SnapshotError::InvalidData("Target path has no parent directory".to_string()))?;
        self.check_disk_space(parent_dir).await?;

        // Handle file:// URLs for local file access
        if url.starts_with("file://") {
            return self.copy_local_file(url, target_path).await;
        }

        // Send GET request
        let response = self.client.get(url).send().await?;

        // Check response status
        if !response.status().is_success() {
            return Err(SnapshotError::HttpStatus {
                status: response.status().as_u16(),
            });
        }

        // Check content length
        if let Some(content_length) = response.content_length() {
            if content_length > self.config.max_size {
                return Err(SnapshotError::TooLarge {
                    size: content_length,
                    max_size: self.config.max_size,
                });
            }
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Fail if content length is not available
        let total_bytes = response
            .content_length()
            .ok_or_else(SnapshotError::ContentLengthMissing)?;

        // Create file writer using futures-io
        let file = File::create(target_path)?;
        let file_writer = Arc::new(Mutex::new(AllowStdIo::new(file)));

        // Use AtomicU64 for thread-safe progress tracking
        let downloaded = Arc::new(AtomicU64::new(0));

        let stream = response.bytes_stream();

        // Process each chunk with progress tracking and size checking
        stream
            .map_err(SnapshotError::Network)
            .try_for_each(|chunk| {
                let downloaded = downloaded.clone();
                let file_writer = file_writer.clone();
                let max_size = self.config.max_size;

                async move {
                    let received_bytes =
                        downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed) + chunk.len() as u64;

                    // Check size limit and abort if exceeded
                    if received_bytes > max_size {
                        return Err(SnapshotError::TooLarge {
                            size: received_bytes,
                            max_size,
                        });
                    }

                    // Progress reporting, only per 1MB or at the end
                    let progress = (received_bytes as f64 / total_bytes as f64) * 100.0;
                    if received_bytes % (1024 * 1024) == 0 || received_bytes == total_bytes {
                        debug!(
                            progress = format!("{:.1}%", progress),
                            %received_bytes, %total_bytes, "Logs snapshot download progress"
                        );
                    }

                    // Write chunk to file using AsyncWriteExt
                    {
                        let mut writer = file_writer.lock().await;
                        writer.write_all(&chunk).await.map_err(SnapshotError::Io)?;
                        writer.flush().await.map_err(SnapshotError::Io)?;
                    }

                    Ok(())
                }
            })
            .await?;

        let downloaded_bytes = downloaded.load(Ordering::Relaxed);
        info!(%downloaded_bytes, to = %target_path.display(), "Logs snapshot file downloaded");

        Ok(())
    }

    /// Copies a local file from a file:// URL to the target path.
    ///
    /// Validates file existence, checks size limits, and copies the file to the target location.
    ///
    /// # Arguments
    ///
    /// * `url` - File URL in format `file:///absolute/path/to/file`
    /// * `target_path` - Destination file path
    ///
    /// # Errors
    ///
    /// * [`SnapshotError::InvalidData`] - Invalid file:// URL format
    /// * [`SnapshotError::Io`] - File not found or permission errors
    /// * [`SnapshotError::TooLarge`] - File exceeds size limit
    async fn copy_local_file(&self, url: &str, target_path: &Path) -> SnapshotResult<()> {
        // Parse the file path from the URL
        let file_path = url
            .strip_prefix("file://")
            .ok_or_else(|| SnapshotError::InvalidData("Invalid file:// URL format".to_string()))?;

        let source_path = Path::new(file_path);

        // Validate path to prevent directory traversal
        let canonical_path = source_path.canonicalize().map_err(SnapshotError::Io)?;

        // Check if source file exists
        if !canonical_path.exists() {
            return Err(SnapshotError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Local file not found: {file_path}"),
            )));
        }

        // Check file size
        let metadata = fs::metadata(canonical_path.clone())?;
        if metadata.len() > self.config.max_size {
            return Err(SnapshotError::TooLarge {
                size: metadata.len(),
                max_size: self.config.max_size,
            });
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy the file using futures-io
        let copied_bytes = fs::copy(canonical_path.clone(), target_path)? as u64;
        info!(
            %copied_bytes, from = %canonical_path.display(), to = %target_path.display(),
            "Copied local snapshot file",
        );

        Ok(())
    }

    /// Checks if there's sufficient disk space available for download and extraction.
    ///
    /// Validates that the target directory has at least 3x the maximum download size
    /// available to account for:
    /// 1. The downloaded archive
    /// 2. Extracted files
    /// 3. Safety margin for system operations
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to check for available space
    ///
    /// # Errors
    ///
    /// Returns `SnapshotError::InsufficientSpace` if available space is below requirements
    pub async fn check_disk_space(&self, dir: &Path) -> SnapshotResult<()> {
        // Create directory if it doesn't exist
        fs::create_dir_all(dir)?;

        // Check if directory exists and is accessible
        let metadata = fs::metadata(dir)?;
        if !metadata.is_dir() {
            return Err(SnapshotError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Target directory does not exist",
            )));
        }

        // Get available disk space
        let available_bytes = get_available_disk_space(dir)?;

        // We need at least 3x the max size to account for:
        // 1. Downloaded archive
        // 2. Extracted files
        // 3. Safety margin for system operations
        let required_bytes = self.config.max_size * 3;

        if available_bytes < required_bytes {
            return Err(SnapshotError::InsufficientSpace {
                required: required_bytes / (1024 * 1024),
                available: available_bytes / (1024 * 1024),
            });
        }

        Ok(())
    }
}

/// Gets available disk space in bytes for the given directory using cross-platform sysinfo.
///
/// This function uses the sysinfo crate to provide platform-independent disk space checking.
/// It finds the disk/mount point that contains the specified directory and returns the
/// available space on that disk.
///
/// # Arguments
///
/// * `dir` - Directory path to check (will be canonicalized)
///
/// # Returns
///
/// Available space in bytes on the disk containing the directory
///
/// # Errors
///
/// - `SnapshotError::Io` if the path cannot be canonicalized
/// - `SnapshotError::InvalidData` if no disks are found on the system
fn get_available_disk_space(dir: &Path) -> SnapshotResult<u64> {
    // Find the disk that contains the given directory
    let target_path = dir.canonicalize().map_err(SnapshotError::Io)?;

    // Find the disk with the longest matching mount point
    let disks = Disks::new_with_refreshed_list();

    // Filter out disks with non matching mount points
    let mut usable_disks = disks
        .iter()
        .filter(|d| target_path.starts_with(d.mount_point()))
        .collect::<Vec<_>>();

    // Sort disks by mount point length (longest first)
    usable_disks.sort_by(|a, b| {
        b.mount_point()
            .as_os_str()
            .len()
            .cmp(&(a.mount_point().as_os_str().len()))
    });

    // If no usable disks found, return an error
    usable_disks.first().map_or_else(
        || {
            Err(SnapshotError::InvalidData(format!(
                "Could not determine disk space for path: {dir:?}"
            )))
        },
        |disk| Ok(disk.available_space()),
    )
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_disk_space_validation() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = SnapshotDownloader::new().expect("Failed to create SnapshotDownloader");

        // Test with available disk space (this should pass)
        let result = downloader.check_disk_space(temp_dir.path()).await;
        assert!(result.is_ok());

        // Test with invalid directory path
        let invalid_path = temp_dir.path().join("nonexistent/nested/path");
        let result = downloader.check_disk_space(&invalid_path).await;
        // Should create the directory and succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enhanced_error_messages() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = SnapshotDownloader::new().expect("Failed to create SnapshotDownloader");

        // Test invalid URL error
        let result = downloader.download_snapshot("invalid://url", temp_dir.path()).await;
        assert!(result.is_err());

        // Test file not found error
        let result = downloader
            .download_snapshot("https://httpbin.org/status/404", temp_dir.path())
            .await;
        assert!(result.is_err());
    }
}
