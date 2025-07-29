//! Fast synchronization using database snapshots.
//!
//! This module enables HOPR nodes to synchronize quickly with the network by downloading
//! and installing pre-built database snapshots instead of processing all historical blockchain logs.
//!
//! # Features
//!
//! - **HTTP/HTTPS Downloads**: Secure download with retry logic and progress tracking
//! - **Local File Support**: Direct installation from local `file://` URLs
//! - **Archive Extraction**: Safe tar.gz extraction with path traversal protection
//! - **Database Validation**: SQLite integrity checks and content verification
//! - **Disk Space Management**: Cross-platform space validation before operations
//! - **Comprehensive Errors**: Actionable error messages with recovery suggestions
//!
//! # URL Support
//!
//! - `https://example.com/snapshot.tar.gz` - Remote HTTP/HTTPS downloads
//! - `file:///path/to/snapshot.tar.gz` - Local file system access
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use hopr_chain_indexer::snapshot::{SnapshotResult, SnapshotManager};
//!
//! # async fn example(db: impl hopr_db_sql::HoprDbGeneralModelOperations + Clone + Send + Sync + 'static) -> SnapshotResult<()> {
//! let manager = SnapshotManager::with_db(db)?;
//! let info = manager
//!     .download_and_setup_snapshot(
//!         "https://snapshots.hoprnet.org/logs.tar.gz",
//!         Path::new("/data/hopr")
//!     )
//!     .await?;
//!
//! println!("Installed snapshot: {} logs, latest block {}", info.log_count, info.latest_block.unwrap_or(0));
//! # Ok(())
//! # }
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

/// Coordinates snapshot download, extraction, validation, and database integration.
///
/// The main interface for snapshot operations in production environments.
/// Manages the complete workflow from download to database installation.
///
/// # Architecture
///
/// - [`SnapshotDownloader`] - HTTP/HTTPS and file:// URL handling with retry logic
/// - [`SnapshotExtractor`] - Secure tar.gz extraction with path validation
/// - [`SnapshotValidator`] - SQLite integrity and content verification
/// - Database integration via [`HoprDbGeneralModelOperations::import_logs_db`]
pub struct SnapshotManager<Db>
where
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    db: Db,
    downloader: SnapshotDownloader,
    extractor: SnapshotExtractor,
    validator: SnapshotValidator,
}

impl<Db> SnapshotManager<Db>
where
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    /// Creates a snapshot manager with database integration.
    ///
    /// # Arguments
    ///
    /// * `db` - Database instance implementing [`HoprDbGeneralModelOperations`]
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hopr_chain_indexer::snapshot::{SnapshotResult, SnapshotManager};
    ///
    /// # fn example(db: impl hopr_db_sql::HoprDbGeneralModelOperations + Clone + Send + Sync + 'static) -> SnapshotResult<()> {
    /// let manager = SnapshotManager::with_db(db)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_db(db: Db) -> Result<Self, SnapshotError> {
        Ok(Self {
            db,
            downloader: SnapshotDownloader::new()?,
            extractor: SnapshotExtractor::new(),
            validator: SnapshotValidator::new(),
        })
    }

    /// Downloads, extracts, validates, and installs a snapshot.
    ///
    /// Performs the complete snapshot setup workflow:
    /// 1. Downloads archive from URL (HTTP/HTTPS/file://)
    /// 2. Extracts tar.gz archive safely
    /// 3. Validates database integrity
    /// 4. Installs via [`HoprDbGeneralModelOperations::import_logs_db`]
    /// 5. Cleans up temporary files
    ///
    /// # Arguments
    ///
    /// * `url` - Snapshot URL (`https://`, `http://`, or `file://` scheme)
    /// * `data_dir` - Target directory for temporary files during installation
    ///
    /// # Returns
    ///
    /// [`SnapshotInfo`] containing log count, block count, and metadata on success
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError`] for network failures, validation errors, or installation issues
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::path::Path;
    /// # use hopr_chain_indexer::snapshot::SnapshotManager;
    /// # async fn example(manager: SnapshotManager<impl hopr_db_sql::HoprDbGeneralModelOperations + Clone + Send + Sync + 'static>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Download from HTTPS
    /// let info = manager
    ///     .download_and_setup_snapshot("https://snapshots.hoprnet.org/logs.tar.gz", Path::new("/data"))
    ///     .await?;
    ///
    /// // Use local file
    /// let info = manager
    ///     .download_and_setup_snapshot("file:///backups/snapshot.tar.gz", Path::new("/data"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
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

        // Update database
        self.db
            .clone()
            .import_logs_db(temp_dir.clone())
            .await
            .map_err(|e| SnapshotError::Installation(e.to_string()))?;

        // Clean up temporary directory
        if let Err(e) = fs::remove_dir_all(&temp_dir_for_cleanup) {
            error!("Failed to cleanup temp directory: {}", e);
        }

        info!("Snapshot setup completed successfully");
        Ok(snapshot_info)
    }
}

/// Test-only snapshot manager without database dependencies.
///
/// Provides the same snapshot workflow as [`SnapshotManager`] but installs
/// files directly to the filesystem instead of integrating with a database.
/// Used in unit tests where database setup would add unnecessary complexity.
#[cfg(test)]
pub struct TestSnapshotManager {
    downloader: SnapshotDownloader,
    extractor: SnapshotExtractor,
    validator: SnapshotValidator,
}

#[cfg(test)]
impl TestSnapshotManager {
    /// Creates a test snapshot manager without database dependencies.
    pub fn new() -> Result<Self, SnapshotError> {
        Ok(Self {
            downloader: SnapshotDownloader::new()?,
            extractor: SnapshotExtractor::new(),
            validator: SnapshotValidator::new(),
        })
    }

    /// Downloads, extracts, validates, and installs a snapshot (test mode).
    ///
    /// Performs the same workflow as [`SnapshotManager::download_and_setup_snapshot`]
    /// but installs files directly to the filesystem instead of database integration.
    ///
    /// # Arguments
    ///
    /// * `url` - Snapshot URL (`https://`, `http://`, or `file://` scheme)
    /// * `data_dir` - Target directory for extracted files
    ///
    /// # Returns
    ///
    /// [`SnapshotInfo`] containing validation results
    pub async fn download_and_setup_snapshot(&self, url: &str, data_dir: &Path) -> SnapshotResult<SnapshotInfo> {
        info!("Starting test snapshot download and setup from: {}", url);

        let temp_dir = tempfile::tempdir_in(data_dir)?;
        let temp_path = temp_dir.path();

        // Download snapshot
        let archive_path = temp_path.join("snapshot.tar.gz");
        self.downloader.download_snapshot(url, &archive_path).await?;

        // Extract snapshot
        let extracted_files = self.extractor.extract_snapshot(&archive_path, &temp_path).await?;
        debug!("Extracted snapshot files: {:?}", extracted_files);

        // Validate extracted database
        let db_path = temp_path.join("hopr_logs.db");
        let snapshot_info = self.validator.validate_snapshot(&db_path).await?;

        // Install files directly to data directory (test mode)
        self.install_snapshot_files(&temp_path, data_dir, &extracted_files)
            .await?;

        info!("Test snapshot setup completed successfully");
        Ok(snapshot_info)
    }

    /// Installs snapshot files from temporary directory to final location.
    ///
    /// Copies extracted files to the target directory, replacing existing files.
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

    /// Returns a reference to the downloader for component testing.
    pub fn downloader(&self) -> &SnapshotDownloader {
        &self.downloader
    }

    /// Returns a reference to the extractor for component testing.
    pub fn extractor(&self) -> &SnapshotExtractor {
        &self.extractor
    }

    /// Returns a reference to the validator for component testing.
    pub fn validator(&self) -> &SnapshotValidator {
        &self.validator
    }
}
