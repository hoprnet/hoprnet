use std::{path::Path, time::Duration};

use futures_util::StreamExt;
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{debug, info, warn};

use crate::snapshot::error::{SnapshotError, SnapshotResult};

/// Configuration for snapshot downloads
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_size: u64,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024 * 1024,  // 10GB max
            timeout: Duration::from_secs(1800), // 30 minutes
            max_retries: 3,
        }
    }
}

/// Handles downloading snapshots from HTTP URLs
pub struct SnapshotDownloader {
    client: Client,
    config: DownloadConfig,
}

impl SnapshotDownloader {
    /// Creates a new snapshot downloader with default configuration
    pub fn new() -> Self {
        Self::with_config(DownloadConfig::default())
    }

    /// Creates a new snapshot downloader with custom configuration
    pub fn with_config(config: DownloadConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            config,
        }
    }

    /// Downloads a snapshot from the given URL to the target path
    pub async fn download_snapshot(&self, url: &str, target_path: &Path) -> SnapshotResult<()> {
        self.download_snapshot_with_retry(url, target_path, self.config.max_retries)
            .await
    }

    /// Downloads a snapshot with retry logic
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
                tokio::time::sleep(delay).await;
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
        self.check_disk_space(target_path.parent().unwrap()).await?;

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
            tokio::fs::create_dir_all(parent).await?;
        }

        // Stream download to file
        let mut file = File::create(target_path).await?;
        let mut downloaded = 0u64;
        let total_size = response.content_length().unwrap_or(0);
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            downloaded += chunk.len() as u64;

            if downloaded > self.config.max_size {
                return Err(SnapshotError::TooLarge {
                    size: downloaded,
                    max_size: self.config.max_size,
                });
            }

            file.write_all(&chunk).await?;

            // Progress reporting
            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                if downloaded % (1024 * 1024) == 0 || downloaded == total_size {
                    debug!(
                        "Download progress: {:.1}% ({}/{} bytes)",
                        progress, downloaded, total_size
                    );
                }
            } else {
                if downloaded % (10 * 1024 * 1024) == 0 {
                    debug!("Downloaded: {} bytes", downloaded);
                }
            }
        }

        file.flush().await?;
        info!("Downloaded {} bytes to {:?}", downloaded, target_path);

        Ok(())
    }

    /// Checks if there's sufficient disk space available
    pub async fn check_disk_space(&self, dir: &Path) -> SnapshotResult<()> {
        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(dir).await?;

        // Check if directory exists and is accessible
        let metadata = tokio::fs::metadata(dir).await?;
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
        Self::new()
    }
}

/// Gets available disk space in bytes for the given directory
fn get_available_disk_space(dir: &Path) -> SnapshotResult<u64> {
    use sysinfo::{Disk, Disks};

    let disks = Disks::new_with_refreshed_list();

    // Find the disk that contains the given directory
    let target_path = dir.canonicalize().map_err(|e| SnapshotError::Io(e))?;

    // Find the disk with the longest matching mount point
    let mut best_match: Option<&Disk> = None;
    let mut best_match_len = 0;

    for disk in &disks {
        let mount_point = disk.mount_point();
        if target_path.starts_with(mount_point) {
            let mount_len = mount_point.as_os_str().len();
            if mount_len > best_match_len {
                best_match = Some(disk);
                best_match_len = mount_len;
            }
        }
    }

    match best_match {
        Some(disk) => Ok(disk.available_space()),
        None => {
            // Fallback: use the first disk or return an error
            if let Some(disk) = disks.first() {
                tracing::warn!(
                    "Could not find matching disk for path {:?}, using first available disk",
                    dir
                );
                Ok(disk.available_space())
            } else {
                Err(SnapshotError::InvalidData("No disks found on system".to_string()))
            }
        }
    }
}
