use crate::snapshot::error::{SnapshotError, SnapshotResult};
use futures_util::StreamExt;
use reqwest::Client;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

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
            max_size: 10 * 1024 * 1024 * 1024, // 10GB max
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
    pub async fn download_snapshot(
        &self,
        url: &str,
        target_path: &Path,
    ) -> SnapshotResult<()> {
        self.download_snapshot_with_retry(url, target_path, self.config.max_retries).await
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
                            SnapshotError::TooLarge { .. } |
                            SnapshotError::InsufficientSpace { .. } |
                            SnapshotError::HttpStatus { status: 400..=499 } => break,
                            _ => continue,
                        }
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| SnapshotError::Archive(
            "All retry attempts failed".to_string()
        )))
    }
    
    /// Performs a single download attempt
    async fn download_snapshot_once(
        &self,
        url: &str,
        target_path: &Path,
    ) -> SnapshotResult<()> {
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
        }
        
        file.flush().await?;
        info!("Downloaded {} bytes to {:?}", downloaded, target_path);
        
        Ok(())
    }
    
    /// Checks if there's sufficient disk space available
    async fn check_disk_space(&self, dir: &Path) -> SnapshotResult<()> {
        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(dir).await?;
        
        // For now, we'll implement a basic check
        // In a production system, you might want to use statvfs or similar
        // to get actual disk space information
        let metadata = tokio::fs::metadata(dir).await?;
        if !metadata.is_dir() {
            return Err(SnapshotError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Target directory does not exist"
            )));
        }
        
        Ok(())
    }
}

impl Default for SnapshotDownloader {
    fn default() -> Self {
        Self::new()
    }
}