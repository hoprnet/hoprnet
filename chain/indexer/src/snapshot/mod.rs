//! Fast synchronization using database snapshots.
//!
//! This module enables HOPR nodes to synchronize quickly with the network by downloading
//! and installing pre-built database snapshots instead of processing all historical blockchain logs.
//!
//! # Features
//!
//! - **HTTP/HTTPS Downloads**: Secure download with retry logic and progress tracking
//! - **Local File Support**: Direct installation from local `file://` URLs
//! - **Archive Extraction**: Safe tar.xz extraction with path traversal protection
//! - **Database Validation**: SQLite integrity checks and content verification
//! - **Disk Space Management**: Cross-platform space validation before operations
//! - **Comprehensive Errors**: Actionable error messages with recovery suggestions
//!
//! # URL Support
//!
//! - `https://example.com/snapshot.tar.xz` - Remote HTTP/HTTPS downloads
//! - `file:///path/to/snapshot.tar.xz` - Local file system access
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
//!         "https://snapshots.hoprnet.org/logs.tar.xz",
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

#[cfg(test)]
pub(crate) mod test_utils;

// Re-export commonly used types
pub use error::{SnapshotError, SnapshotResult};
pub use validate::SnapshotInfo;

use std::{fs, path::Path};

use hopr_db_sql::HoprDbGeneralModelOperations;
use tracing::{debug, error, info};

use crate::snapshot::{download::SnapshotDownloader, extract::SnapshotExtractor, validate::SnapshotValidator};

/// Trait for implementing the snapshot installation step.
///
/// This trait abstracts the final installation step of the snapshot workflow,
/// allowing different implementations for production (database integration)
/// and testing (filesystem copy) scenarios.
#[async_trait::async_trait]
trait SnapshotInstaller {
    /// Installs the validated snapshot from the temporary directory.
    ///
    /// # Arguments
    /// * `temp_dir` - Directory containing extracted and validated snapshot files
    /// * `data_dir` - Target directory for installation
    /// * `extracted_files` - List of files that were extracted from the archive
    ///
    /// # Returns
    /// Result indicating success or failure of installation
    async fn install_snapshot(
        &self,
        temp_dir: &Path,
        data_dir: &Path,
        extracted_files: &[String],
    ) -> SnapshotResult<()>;
}

/// Shared snapshot workflow implementation.
///
/// Contains the common download → extract → validate → install workflow
/// shared between SnapshotManager and TestSnapshotManager.
struct SnapshotWorkflow {
    downloader: SnapshotDownloader,
    extractor: SnapshotExtractor,
    validator: SnapshotValidator,
}

impl SnapshotWorkflow {
    /// Creates a new snapshot workflow with default components.
    fn new() -> Result<Self, SnapshotError> {
        Ok(Self {
            downloader: SnapshotDownloader::new()?,
            extractor: SnapshotExtractor::new(),
            validator: SnapshotValidator::new(),
        })
    }

    /// Executes the complete snapshot workflow.
    ///
    /// Downloads, extracts, validates, and installs a snapshot using the provided installer.
    async fn execute_workflow<I: SnapshotInstaller>(
        &self,
        installer: &I,
        url: &str,
        data_dir: &Path,
        use_temp_subdir: bool,
    ) -> SnapshotResult<SnapshotInfo> {
        info!("Starting snapshot download and setup from: {}", url);

        // Create temporary directory - either as subdirectory or using tempfile
        let (temp_dir, _temp_guard) = if use_temp_subdir {
            let temp_dir = data_dir.join("snapshot_temp");
            fs::create_dir_all(&temp_dir)?;
            (temp_dir, None)
        } else {
            let temp_guard = tempfile::tempdir_in(data_dir)?;
            let temp_dir = temp_guard.path().to_path_buf();
            (temp_dir, Some(temp_guard))
        };

        // Download snapshot
        let archive_path = temp_dir.join("snapshot.tar.xz");
        self.downloader.download_snapshot(url, &archive_path).await?;

        // Extract snapshot
        let extracted_files = self.extractor.extract_snapshot(&archive_path, &temp_dir).await?;
        debug!("Extracted snapshot files: {:?}", extracted_files);

        // Validate extracted database
        let db_path = temp_dir.join("hopr_logs.db");
        let snapshot_info = self.validator.validate_snapshot(&db_path).await?;

        // Install using the provided installer
        installer
            .install_snapshot(&temp_dir, data_dir, &extracted_files)
            .await?;

        // Cleanup temporary directory if we created it manually
        if use_temp_subdir {
            if let Err(e) = fs::remove_dir_all(&temp_dir) {
                error!("Failed to cleanup temp directory: {}", e);
            }
        }
        // tempfile cleanup is automatic via Drop

        info!("Snapshot setup completed successfully");
        Ok(snapshot_info)
    }
}

/// Coordinates snapshot download, extraction, validation, and database integration.
///
/// The main interface for snapshot operations in production environments.
/// Manages the complete workflow from download to database installation.
///
/// # Architecture
///
/// - [`SnapshotDownloader`] - HTTP/HTTPS and file:// URL handling with retry logic
/// - [`SnapshotExtractor`] - Secure tar.xz extraction with path validation
/// - [`SnapshotValidator`] - SQLite integrity and content verification
/// - Database integration via [`HoprDbGeneralModelOperations::import_logs_db`]
pub struct SnapshotManager<Db>
where
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    db: Db,
    workflow: SnapshotWorkflow,
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
            workflow: SnapshotWorkflow::new()?,
        })
    }

    /// Downloads, extracts, validates, and installs a snapshot.
    ///
    /// Performs the complete snapshot setup workflow:
    /// 1. Downloads archive from URL (HTTP/HTTPS/file://)
    /// 2. Extracts tar.xz archive safely
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
    ///     .download_and_setup_snapshot("https://snapshots.hoprnet.org/logs.tar.xz", Path::new("/data"))
    ///     .await?;
    ///
    /// // Use local file
    /// let info = manager
    ///     .download_and_setup_snapshot("file:///backups/snapshot.tar.xz", Path::new("/data"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_and_setup_snapshot(&self, url: &str, data_dir: &Path) -> SnapshotResult<SnapshotInfo> {
        self.workflow.execute_workflow(self, url, data_dir, true).await
    }
}

#[async_trait::async_trait]
impl<Db> SnapshotInstaller for SnapshotManager<Db>
where
    Db: HoprDbGeneralModelOperations + Clone + Send + Sync + 'static,
{
    async fn install_snapshot(
        &self,
        temp_dir: &Path,
        _data_dir: &Path,
        _extracted_files: &[String],
    ) -> SnapshotResult<()> {
        // Update database using the imported logs database
        self.db
            .clone()
            .import_logs_db(temp_dir.to_path_buf())
            .await
            .map_err(|e| SnapshotError::Installation(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use super::test_utils::*;

    #[tokio::test]
    async fn test_snapshot_manager_integration() {
        let temp_dir = TempDir::new().unwrap();

        // Create test archive
        let archive_path = create_test_archive(&temp_dir).await.unwrap();

        // Use TestSnapshotManager for testing
        let manager = TestSnapshotManager::new().expect("Failed to create TestSnapshotManager");
        let data_dir = temp_dir.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        // Test file:// URL using TestSnapshotManager
        let file_url = format!("file://{}", archive_path.display());
        let result = manager.download_and_setup_snapshot(&file_url, &data_dir).await;

        assert!(result.is_ok(), "TestSnapshotManager should handle file:// URLs");
        let info = result.unwrap();
        assert_eq!(info.log_count, 2);

        // Verify the database file was installed
        assert!(data_dir.join("hopr_logs.db").exists());
    }

    #[tokio::test]
    async fn test_snapshot_manager_with_data_directory() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("hopr_data");
        fs::create_dir_all(&data_dir).unwrap();

        // Create a test archive
        let archive_path = create_test_archive(&temp_dir).await.unwrap();

        // Test file:// URL support using TestSnapshotManager
        let manager = TestSnapshotManager::new().expect("Failed to create TestSnapshotManager");

        // Test with file:// URL for local testing
        let file_url = format!("file://{}", archive_path.display());

        // Test the full workflow through TestSnapshotManager
        let result = manager.download_and_setup_snapshot(&file_url, &data_dir).await;
        assert!(result.is_ok(), "TestSnapshotManager should handle complete workflow");

        let info = result.unwrap();
        assert_eq!(info.log_count, 2);

        // Verify the database file exists in the data directory
        assert!(data_dir.join("hopr_logs.db").exists());

        // Also test individual component access
        let downloader = manager.workflow.downloader;
        let downloaded_archive = data_dir.join("test_download.tar.xz");
        let download_result = downloader.download_snapshot(&file_url, &downloaded_archive).await;
        assert!(download_result.is_ok(), "file:// URL download should succeed");
        assert!(downloaded_archive.exists(), "Downloaded archive should exist");
    }
}
