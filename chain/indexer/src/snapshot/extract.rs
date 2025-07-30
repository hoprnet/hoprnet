//! Secure tar.xz archive extraction with path traversal protection.
//!
//! Provides safe extraction of snapshot archives with security validations
//! to prevent malicious archives from escaping the target directory.

use std::{
    fs,
    fs::File,
    path::{Component::ParentDir, Path},
};

use async_compression::futures::bufread::XzDecoder;
use async_tar::Archive;
use futures_util::{
    StreamExt,
    io::{AllowStdIo, BufReader as FuturesBufReader},
};
use tracing::{debug, error, info};

use crate::snapshot::error::{SnapshotError, SnapshotResult};

/// Extracts tar.xz snapshot archives with security validations.
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

    /// Extracts a tar.xz snapshot archive safely to the target directory.
    ///
    /// Validates each file path to prevent directory traversal attacks and
    /// only extracts expected database files.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the tar.xz archive file
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
        info!(from = %archive_path.display(), to = %target_dir.display(), "Extracting snapshot");

        // Create target directory if it doesn't exist
        fs::create_dir_all(target_dir)?;

        let extracted_files = self.extract_tar_xz(archive_path, target_dir).await?;

        info!(nr_of_files = extracted_files.len(), "Extracted snapshot files");
        Ok(extracted_files)
    }

    /// Extracts a tar.xz archive using async operations
    async fn extract_tar_xz(&self, archive_path: &Path, target_dir: &Path) -> SnapshotResult<Vec<String>> {
        // Open file using AllowStdIo to make std::fs::File work with futures-io
        let file = File::open(archive_path).map_err(SnapshotError::Io)?;
        let file_reader = AllowStdIo::new(file);

        // Create XZ decoder with parallel decompression using futures-io
        let buf_reader = FuturesBufReader::new(file_reader);
        let decoder = XzDecoder::new(buf_reader);
        let archive = Archive::new(decoder);

        let mut extracted_files = Vec::new();
        let mut entries = archive.entries().map_err(SnapshotError::Io)?;

        while let Some(entry_result) = entries.next().await {
            let mut entry = entry_result.map_err(SnapshotError::Io)?;
            let path_buf = entry.path().map_err(SnapshotError::Io)?.to_path_buf();

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
            if self.expected_files.iter().any(|f| f == filename) {
                // Extract the file
                entry.unpack_in(target_dir).await.map_err(SnapshotError::Io)?;
                extracted_files.push(filename.to_string());

                debug!(%filename, "Extracted file");
            } else {
                error!(%filename, "Skipping unexpected file in archive");
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
        self.list_archive_contents(archive_path).await
    }

    /// Lists the contents of a tar.xz archive
    async fn list_archive_contents(&self, archive_path: &Path) -> SnapshotResult<Vec<String>> {
        // Open file using AllowStdIo to make std::fs::File work with futures-io
        let file = File::open(archive_path).map_err(SnapshotError::Io)?;
        let file_reader = AllowStdIo::new(file);

        // Create XZ decoder using futures-io
        let buf_reader = FuturesBufReader::new(file_reader);
        let decoder = XzDecoder::new(buf_reader);
        let archive = Archive::new(decoder);

        let mut files = Vec::new();
        let mut entries = archive.entries().map_err(SnapshotError::Io)?;

        while let Some(entry_result) = entries.next().await {
            let entry = entry_result.map_err(SnapshotError::Io)?;
            let path = entry.path().map_err(SnapshotError::Io)?;

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
