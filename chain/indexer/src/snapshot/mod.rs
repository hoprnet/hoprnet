pub mod download;
pub mod error;
pub mod extract;
pub mod validate;

// Re-export commonly used types
pub use validate::SnapshotInfo;
pub use error::{SnapshotError, SnapshotResult};

#[cfg(test)]
mod tests;

use crate::snapshot::{
    download::SnapshotDownloader,
    extract::SnapshotExtractor,
    validate::SnapshotValidator,
};
use std::path::Path;
use tracing::info;

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
    ) -> crate::snapshot::error::SnapshotResult<crate::snapshot::SnapshotInfo> {
        info!("Starting snapshot download and setup from: {}", url);
        
        // Create temporary directory for download
        let temp_dir = data_dir.join("snapshot_temp");
        tokio::fs::create_dir_all(&temp_dir).await?;
        
        // We'll clean up the temp directory at the end
        let temp_dir_for_cleanup = temp_dir.clone();
        
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
        
        // Clean up temporary directory
        if let Err(e) = tokio::fs::remove_dir_all(&temp_dir_for_cleanup).await {
            tracing::warn!("Failed to cleanup temp directory: {}", e);
        }
        
        info!("Snapshot setup completed successfully");
        Ok(snapshot_info)
    }
    
    /// Installs snapshot files from temporary directory to final location
    async fn install_snapshot_files(
        &self,
        temp_dir: &Path,
        data_dir: &Path,
        files: &[String],
    ) -> crate::snapshot::error::SnapshotResult<()> {
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