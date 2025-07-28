//! Secure snapshot downloading with HTTP/HTTPS and local file support.
//!
//! Provides robust download capabilities for snapshot archives with:
//! - **URL Support**: HTTP/HTTPS with retry logic, local file:// URLs
//! - **Safety**: Size limits, timeout protection, disk space validation
//! - **Reliability**: Exponential backoff, progress tracking, error recovery
//! - **Cross-platform**: Uses sysinfo for disk space checking

use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use futures_util::{SinkExt, TryStreamExt};
use hopr_async_runtime::prelude::sleep;
use reqwest::Client;
use sysinfo::Disks;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedWrite};
use tracing::{debug, info, warn};

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
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Maximum allowed file size in bytes
    pub max_size: u64,
    /// HTTP request timeout duration
    pub timeout: Duration,
    /// Maximum number of retry attempts for failed downloads
    pub max_retries: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_size: LOGS_SNAPSHOT_DOWNLOADER_MAX_SIZE,
            timeout: LOGS_SNAPSHOT_DOWNLOADER_TIMEOUT,
            max_retries: LOGS_SNAPSHOT_DOWNLOADER_MAX_RETRIES,
        }
    }
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
///         "https://snapshots.hoprnet.org/logs.tar.gz",
///         Path::new("/tmp/snapshot.tar.gz"),
///     )
///     .await?;
///
/// // Copy from local file
/// downloader
///     .download_snapshot("file:///backups/snapshot.tar.gz", Path::new("/tmp/snapshot.tar.gz"))
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
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = Duration::from_secs(2u64.pow(attempt.min(5))); // Exponential backoff
                info!("Retry attempt {} after {:?}", attempt, delay);
                sleep(delay).await;
            }

            match self.download_snapshot_once(url, target_path).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    warn!("Download attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    // Don't retry on certain errors
                    if let Some(ref err) = last_error {
                        match err {
                            SnapshotError::TooLarge { .. }
                            | SnapshotError::InsufficientSpace { .. }
                            | SnapshotError::HttpStatus { status: 400..=499 } => break,
                            _ => continue,
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| SnapshotError::Timeout("All retry attempts failed".to_string())))
    }

    /// Performs a single download attempt
    async fn download_snapshot_once(&self, url: &str, target_path: &Path) -> SnapshotResult<()> {
        info!("Downloading snapshot from: {}", url);

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

        // Stream download to file
        let file = File::create(target_path).await?;

        // Fail if content length is not available
        let total_size = response
            .content_length()
            .ok_or_else(SnapshotError::ContentLengthMissing)?;

        // Convert file to a sink for streaming using FramedWrite with BytesCodec
        let stream = response.bytes_stream();
        let file_sink = FramedWrite::new(file, BytesCodec::new());

        // Use AtomicU64 for thread-safe progress tracking
        let downloaded = std::sync::Arc::new(AtomicU64::new(0));
        let downloaded_clone = downloaded.clone();

        stream
            .map_err(SnapshotError::Network)
            .try_fold(file_sink, move |mut sink, chunk| {
                let downloaded = downloaded_clone.clone();
                let max_size = self.config.max_size;
                async move {
                    let new_total = downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed) + chunk.len() as u64;

                    // Check size limit and abort if exceeded
                    if new_total > max_size {
                        return Err(SnapshotError::TooLarge {
                            size: new_total,
                            max_size,
                        });
                    }

                    // Progress reporting, only per 1MB or at the end
                    let progress = (new_total as f64 / total_size as f64) * 100.0;
                    if new_total % (1024 * 1024) == 0 || new_total == total_size {
                        debug!(
                            "Snapshot download progress: {:.1}% ({}/{} bytes)",
                            progress, new_total, total_size
                        );
                    }

                    // Send chunk to sink
                    sink.send(chunk)
                        .await
                        .map_err(|e| SnapshotError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

                    Ok(sink)
                }
            })
            .await?;

        let total_downloaded = downloaded.load(Ordering::Relaxed);
        info!("Snapshot downloaded {} bytes to {:?}", total_downloaded, target_path);

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

        // Copy the file
        let copied_bytes = tokio::fs::copy(canonical_path.clone(), target_path).await?;
        info!(
            "Copied local snapshot file {} bytes from {:?} to {:?}",
            copied_bytes, canonical_path, target_path
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

impl Default for SnapshotDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create default SnapshotDownloader")
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
