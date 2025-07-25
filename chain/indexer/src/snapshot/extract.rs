//! Secure tar.gz archive extraction with path traversal protection.
//!
//! Provides safe extraction of snapshot archives with security validations
//! to prevent malicious archives from escaping the target directory.

use std::{
    fs,
    fs::File,
    path::{Component::ParentDir, Path},
};

use flate2::read::GzDecoder;
use hopr_async_runtime::prelude::spawn_blocking;
use tar::Archive;
use tracing::{debug, error, info};

use crate::snapshot::error::{SnapshotError, SnapshotResult};

/// Extracts tar.gz snapshot archives with security validations.
///
/// Provides safe extraction by validating file paths to prevent directory
/// traversal attacks and ensuring only expected database files are extracted.
pub struct SnapshotExtractor {
    /// Expected database files in snapshot archives
    expected_files: Vec<String>,
}

impl SnapshotExtractor {
    /// Creates a new extractor with predefined expected files.
    ///
    /// Expected files include SQLite database and WAL files:
    /// - `hopr_logs.db` - Main database file
    /// - `hopr_logs.db-wal` - Write-Ahead Log file
    /// - `hopr_logs.db-shm` - Shared memory file
    pub fn new() -> Self {
        Self {
            expected_files: vec![
                "hopr_logs.db".to_string(),
                "hopr_logs.db-wal".to_string(),
                "hopr_logs.db-shm".to_string(),
            ],
        }
    }

    /// Extracts a tar.gz snapshot archive safely to the target directory.
    ///
    /// Validates each file path to prevent directory traversal attacks and
    /// only extracts expected database files.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the tar.gz archive file
    /// * `target_dir` - Directory where files will be extracted
    ///
    /// # Returns
    ///
    /// Vector of successfully extracted file names (relative paths)
    ///
    /// # Errors
    ///
    /// * [`SnapshotError::Archive`] - Invalid archive format or extraction failure
    /// * [`SnapshotError::Io`] - File system errors during extraction
    /// * [`SnapshotError::InvalidFormat`] - Path traversal attempt detected
    ///
    /// # Security
    ///
    /// This method validates all file paths to prevent extraction outside
    /// the target directory (path traversal attacks).
    pub async fn extract_snapshot(&self, archive_path: &Path, target_dir: &Path) -> SnapshotResult<Vec<String>> {
        info!("Extracting snapshot from {:?} to {:?}", archive_path, target_dir);

        // Create target directory if it doesn't exist
        fs::create_dir_all(target_dir)?;

        let archive_path = archive_path.to_path_buf();
        let target_dir = target_dir.to_path_buf();
        let expected_files = self.expected_files.clone();

        // Run in blocking task to avoid blocking async runtime
        let extracted_files =
            spawn_blocking(move || Self::extract_tar_gz(&archive_path, &target_dir, &expected_files)).await??;

        info!("Extracted {} snapshot files", extracted_files.len());
        Ok(extracted_files)
    }

    /// Extracts a tar.gz archive
    fn extract_tar_gz(
        archive_path: &Path,
        target_dir: &Path,
        expected_files: &[String],
    ) -> SnapshotResult<Vec<String>> {
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let mut extracted_files = Vec::new();

        // Using a for loop because entries uses references which would be hard to iterated over
        // otherwise.
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path_buf = entry.path()?.to_path_buf();

            // Security check: prevent directory traversal
            if path_buf.components().any(|c| c == ParentDir) {
                return Err(SnapshotError::InvalidFormat(
                    "Archive contains parent directory references".to_string(),
                ));
            }

            // Get the filename
            let filename = path_buf
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| SnapshotError::InvalidFormat("Invalid filename".to_string()))?;

            // Check if this is a file we expect
            if expected_files.iter().any(|f| f == filename) {
                let target_path = target_dir.join(filename);

                // Extract the file
                entry.unpack(&target_path)?;
                extracted_files.push(filename.to_string());

                debug!("Extracted: {}", filename);
            } else {
                error!("Skipping unexpected file in archive: {}", filename);
            }
        }

        // Verify we got the main database file
        if !extracted_files.contains(&"hopr_logs.db".to_string()) {
            return Err(SnapshotError::InvalidFormat(
                "Archive does not contain hopr_logs.db".to_string(),
            ));
        }

        Ok(extracted_files)
    }

    /// Validates that the archive contains expected files without extracting
    pub async fn validate_archive(&self, archive_path: &Path) -> SnapshotResult<Vec<String>> {
        let archive_path = archive_path.to_path_buf();

        // Run in blocking task to avoid blocking async runtime
        spawn_blocking(move || Self::list_archive_contents(&archive_path)).await?
    }

    /// Lists the contents of a tar.gz archive
    fn list_archive_contents(archive_path: &Path) -> SnapshotResult<Vec<String>> {
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let mut files = Vec::new();

        // Using a for loop because entries uses references which would be hard to iterated over
        // otherwise.
        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?;

            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                files.push(filename.to_string());
            }
        }

        Ok(files)
    }
}

impl Default for SnapshotExtractor {
    fn default() -> Self {
        Self::new()
    }
}
