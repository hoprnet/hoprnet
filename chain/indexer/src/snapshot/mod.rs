pub mod download;
pub mod error;
pub mod extract;
pub mod validate;

#[cfg(test)]
mod tests;

use crate::snapshot::{
    download::SnapshotDownloader,
    extract::SnapshotExtractor,
    validate::{SnapshotValidator, SnapshotInfo},
    error::{SnapshotError, SnapshotResult},
};
use std::path::Path;
use tracing::{info, warn};
use scopeguard;

/// Main snapshot management interface
pub struct SnapshotManager {
    downloader: SnapshotDownloader,
    extractor: SnapshotExtractor,
    validator: SnapshotValidator,
}

impl SnapshotManager {
    /// Creates a new snapshot manager instance
    pub fn new() -> Self {
        Self {
            downloader: SnapshotDownloader::new(),
            extractor: SnapshotExtractor::new(),
            validator: SnapshotValidator::new(),
        }
    }
    
    /// Downloads and sets up a snapshot from the given URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download the snapshot from
    /// * `data_dir` - The directory to install the snapshot to
    ///
    /// # Returns
    ///
    /// Information about the installed snapshot on success
    pub async fn download_and_setup_snapshot(
        &self,
        url: &str,
        data_dir: &Path,
    ) -> SnapshotResult<SnapshotInfo> {
        info!("Starting snapshot download and setup from: {}", url);
        
        // Create temporary directory for download
        let temp_dir = data_dir.join("snapshot_temp");
        tokio::fs::create_dir_all(&temp_dir).await?;
        
        // Ensure cleanup on exit
        let _cleanup = scopeguard::defer! {
            if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
                warn!("Failed to cleanup temp directory: {}", e);
            }
        };
        
        // Download snapshot
        let archive_path = temp_dir.join("snapshot.tar.gz");
        self.downloader.download_snapshot(url, &archive_path).await?;
        
        // Extract snapshot
        let extracted_files = self.extractor.extract_snapshot(&archive_path, &temp_dir).await?;
        info!("Extracted files: {:?}", extracted_files);
        
        // Validate extracted database
        let db_path = temp_dir.join("hopr_logs.db");
        let snapshot_info = self.validator.validate_snapshot(&db_path).await?;
        
        // Move validated files to final location
        self.install_snapshot_files(&temp_dir, data_dir, &extracted_files).await?;
        
        info!("Snapshot setup completed successfully");
        Ok(snapshot_info)
    }
    
    /// Installs snapshot files from temporary directory to final location
    async fn install_snapshot_files(
        &self,
        temp_dir: &Path,
        data_dir: &Path,
        files: &[String],
    ) -> SnapshotResult<()> {
        for file in files {
            let src = temp_dir.join(file);
            let dst = data_dir.join(file);
            
            // Remove existing file if it exists
            if dst.exists() {
                tokio::fs::remove_file(&dst).await?;
            }
            
            // Move file from temp to final location
            tokio::fs::rename(&src, &dst).await?;
            info!("Installed: {} -> {}", file, dst.display());
        }
        
        Ok(())
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}