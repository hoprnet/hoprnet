//! Snapshot module for HOPR indexer
//!
//! This module provides functionality for downloading, extracting, validating, and managing
//! log database snapshots. Snapshots allow new nodes to quickly synchronize with the network
//! by downloading pre-built database files instead of fetching all historical logs.
//!
//! # Features
//!
//! - **Download**: Secure download of snapshot archives from HTTP/HTTPS URLs with retry logic
//! - **Extraction**: Safe extraction of tar.gz archives with path traversal protection
//! - **Validation**: SQLite database integrity checks and content validation
//! - **Disk Space Management**: Cross-platform disk space verification before operations
//! - **Error Handling**: Comprehensive error types with actionable user guidance
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//!
//! use hopr_chain_indexer::snapshot::SnapshotManager;
//!
//! async fn setup_snapshot() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = SnapshotManager::new();
//!     let snapshot_info = manager
//!         .download_and_setup_snapshot("https://example.com/snapshot.tar.gz", Path::new("/data/hopr"))
//!         .await?;
//!
//!     println!("Snapshot installed: {} logs", snapshot_info.log_count);
//!     Ok(())
//! }
//! ```

pub mod download;
pub mod error;
pub mod extract;
pub mod validate;

// Re-export commonly used types
pub use error::{SnapshotError, SnapshotResult};
pub use validate::SnapshotInfo;

#[cfg(test)]
mod tests;

use std::{fs, path::Path};

use hopr_db_sql::HoprDbGeneralModelOperations;
use tracing::{debug, error, info};

use crate::snapshot::{download::SnapshotDownloader, extract::SnapshotExtractor, validate::SnapshotValidator};

/// Main snapshot management interface for coordinating snapshot operations.
///
/// `SnapshotManager` provides a high-level API for downloading, extracting, and validating
/// database snapshots. It coordinates the individual components (downloader, extractor, validator)
/// to provide a seamless snapshot setup experience.
///
/// # Components
///
/// - **Downloader**: Handles HTTP/HTTPS downloads with retry logic and progress tracking
/// - **Extractor**: Safely extracts tar.gz archives with security validations
/// - **Validator**: Verifies SQLite database integrity and content consistency
pub struct SnapshotManager<Db>
where
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    db: Option<Db>,
    downloader: SnapshotDownloader,
    extractor: SnapshotExtractor,
    validator: SnapshotValidator,
}

impl<Db> SnapshotManager<Db>
where
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    /// Creates a new snapshot manager instance
    pub fn new() -> Self {
        Self {
            db: None,
            downloader: SnapshotDownloader::new(),
            extractor: SnapshotExtractor::new(),
            validator: SnapshotValidator::new(),
        }
    }

    /// Creates a new snapshot manager instance (with a DB)
    pub fn with_db(db: Db) -> Self {
        Self {
            db: Some(db),
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
    pub async fn download_and_setup_snapshot(&self, url: &str, data_dir: &Path) -> SnapshotResult<SnapshotInfo> {
        info!("Starting snapshot download and setup from: {}", url);

        // Create temporary directory for download
        let temp_dir = data_dir.join("snapshot_temp");
        fs::create_dir_all(&temp_dir)?;

        // We'll clean up the temp directory at the end
        let temp_dir_for_cleanup = temp_dir.clone();

        // Download snapshot
        let archive_path = temp_dir.join("snapshot.tar.gz");
        self.downloader.download_snapshot(url, &archive_path).await?;

        // Extract snapshot
        let extracted_files = self.extractor.extract_snapshot(&archive_path, &temp_dir).await?;
        debug!("Extracted snapshot files: {:?}", extracted_files);

        // Validate extracted database
        let db_path = temp_dir.join("hopr_logs.db");
        let snapshot_info = self.validator.validate_snapshot(&db_path).await?;

        // Update database (if db is provided) or install files to data directory
        if let Some(db) = &self.db {
            db.clone()
                .replace_logs_db(&temp_dir, &extracted_files)
                .await
                .map_err(|e| SnapshotError::Installation(e.to_string()))?;
        } else {
            // If no database, install files directly to data directory
            self.install_snapshot_files(&temp_dir, data_dir, &extracted_files)
                .await?;
        }

        // Clean up temporary directory
        if let Err(e) = fs::remove_dir_all(&temp_dir_for_cleanup) {
            error!("Failed to cleanup temp directory: {}", e);
        }

        info!("Snapshot setup completed successfully");
        Ok(snapshot_info)
    }

    /// Installs snapshot files from temporary directory to final location
    async fn install_snapshot_files(&self, temp_dir: &Path, data_dir: &Path, files: &[String]) -> SnapshotResult<()> {
        fs::create_dir_all(data_dir)?;

        for file in files {
            let src = temp_dir.join(file);
            let dst = data_dir.join(file);

            // Remove existing file if it exists
            if dst.exists() {
                fs::remove_file(&dst)?;
            }

            // Copy file to final location
            fs::copy(&src, &dst)?;
            debug!("Installed snapshot file: {} -> {}", file, dst.display());
        }

        Ok(())
    }
}
