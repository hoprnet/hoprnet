use crate::snapshot::error::{SnapshotError, SnapshotResult};
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;
use tracing::{info, warn};

/// Handles extraction of snapshot archives
pub struct SnapshotExtractor {
    /// List of files we expect to find in the archive
    expected_files: Vec<String>,
}

impl SnapshotExtractor {
    /// Creates a new snapshot extractor
    pub fn new() -> Self {
        Self {
            expected_files: vec![
                "hopr_logs.db".to_string(),
                "hopr_logs.db-wal".to_string(),
                "hopr_logs.db-shm".to_string(),
            ],
        }
    }
    
    /// Extracts a snapshot archive to the target directory
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the tar.gz archive
    /// * `target_dir` - Directory to extract files to
    ///
    /// # Returns
    ///
    /// List of files that were successfully extracted
    pub async fn extract_snapshot(
        &self,
        archive_path: &Path,
        target_dir: &Path,
    ) -> SnapshotResult<Vec<String>> {
        info!("Extracting snapshot from {:?} to {:?}", archive_path, target_dir);
        
        // Create target directory if it doesn't exist
        tokio::fs::create_dir_all(target_dir).await?;
        
        // Extract in blocking task to avoid blocking async runtime
        let archive_path = archive_path.to_path_buf();
        let target_dir = target_dir.to_path_buf();
        let expected_files = self.expected_files.clone();
        
        let extracted_files = tokio::task::spawn_blocking(move || {
            Self::extract_tar_gz(&archive_path, &target_dir, &expected_files)
        }).await??;
        
        info!("Extracted {} files", extracted_files.len());
        Ok(extracted_files)
    }
    
    /// Extracts a tar.gz archive (blocking operation)
    fn extract_tar_gz(
        archive_path: &Path,
        target_dir: &Path,
        expected_files: &[String],
    ) -> SnapshotResult<Vec<String>> {
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        
        let mut extracted_files = Vec::new();
        
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            
            // Security check: prevent directory traversal
            if path.components().any(|c| c == std::path::Component::ParentDir) {
                return Err(SnapshotError::InvalidFormat(
                    "Archive contains parent directory references".to_string()
                ));
            }
            
            // Get the filename
            let filename = path.file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| SnapshotError::InvalidFormat("Invalid filename".to_string()))?;
            
            // Check if this is a file we expect
            if expected_files.iter().any(|f| f == filename) {
                let target_path = target_dir.join(filename);
                
                // Extract the file
                entry.unpack(&target_path)?;
                extracted_files.push(filename.to_string());
                
                info!("Extracted: {}", filename);
            } else {
                warn!("Skipping unexpected file in archive: {}", filename);
            }
        }
        
        // Verify we got the main database file
        if !extracted_files.contains(&"hopr_logs.db".to_string()) {
            return Err(SnapshotError::InvalidFormat(
                "Archive does not contain hopr_logs.db".to_string()
            ));
        }
        
        Ok(extracted_files)
    }
    
    /// Validates that the archive contains expected files without extracting
    pub async fn validate_archive(&self, archive_path: &Path) -> SnapshotResult<Vec<String>> {
        let archive_path = archive_path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            Self::list_archive_contents(&archive_path)
        }).await?
    }
    
    /// Lists the contents of a tar.gz archive
    fn list_archive_contents(archive_path: &Path) -> SnapshotResult<Vec<String>> {
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        
        let mut files = Vec::new();
        
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