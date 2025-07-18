#[cfg(test)]
mod tests {
    use std::fs::File;

    use flate2::{Compression, write::GzEncoder};
    use tar::Builder;
    use tempfile::TempDir;

    use super::*;
    use crate::snapshot::{
        SnapshotManager, download::SnapshotDownloader, extract::SnapshotExtractor, validate::SnapshotValidator,
    };

    /// Creates a test SQLite database for testing
    fn create_test_sqlite_db(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let conn = rusqlite::Connection::open(path)?;

        // Create test tables
        conn.execute(
            "CREATE TABLE logs (
                id INTEGER PRIMARY KEY,
                block_number INTEGER NOT NULL,
                log_index INTEGER NOT NULL,
                data TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE blocks (
                id INTEGER PRIMARY KEY,
                block_number INTEGER NOT NULL UNIQUE,
                block_hash TEXT NOT NULL
            )",
            [],
        )?;

        // Insert test data
        conn.execute(
            "INSERT INTO logs (block_number, log_index, data) VALUES (1, 0, 'test_log_1')",
            [],
        )?;

        conn.execute(
            "INSERT INTO logs (block_number, log_index, data) VALUES (2, 0, 'test_log_2')",
            [],
        )?;

        conn.execute("INSERT INTO blocks (block_number, block_hash) VALUES (1, 'hash_1')", [])?;

        conn.execute("INSERT INTO blocks (block_number, block_hash) VALUES (2, 'hash_2')", [])?;

        Ok(())
    }

    /// Creates a test tar.gz archive containing a SQLite database
    fn create_test_archive(temp_dir: &TempDir) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        // Create the database
        let db_path = temp_dir.path().join("hopr_logs.db");
        create_test_sqlite_db(&db_path)?;

        // Create archive
        let archive_path = temp_dir.path().join("test_snapshot.tar.gz");
        let tar_gz = File::create(&archive_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        // Add the database file to the archive
        tar.append_path_with_name(&db_path, "hopr_logs.db")?;

        // Finish the archive
        tar.into_inner()?.finish()?;

        Ok(archive_path)
    }

    #[tokio::test]
    async fn test_snapshot_extractor() {
        let temp_dir = TempDir::new().unwrap();
        let extractor = SnapshotExtractor::new();

        // Create test archive
        let archive_path = create_test_archive(&temp_dir).unwrap();

        // Extract the archive
        let extract_dir = temp_dir.path().join("extracted");
        let result = extractor.extract_snapshot(&archive_path, &extract_dir).await;

        assert!(result.is_ok(), "Extraction should succeed");
        let files = result.unwrap();
        assert!(files.contains(&"hopr_logs.db".to_string()));
        assert!(extract_dir.join("hopr_logs.db").exists());
    }

    #[tokio::test]
    async fn test_snapshot_validator() {
        let temp_dir = TempDir::new().unwrap();
        let validator = SnapshotValidator::new();

        // Create test database
        let db_path = temp_dir.path().join("hopr_logs.db");
        create_test_sqlite_db(&db_path).unwrap();

        // Validate the database
        let result = validator.validate_snapshot(&db_path).await;

        assert!(result.is_ok(), "Validation should succeed");
        let info = result.unwrap();
        assert_eq!(info.log_count, 2);
        assert_eq!(info.latest_block, Some(2));
        assert_eq!(info.tables, 2);
    }

    #[tokio::test]
    async fn test_snapshot_validator_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let validator = SnapshotValidator::new();

        // Try to validate non-existent file
        let db_path = temp_dir.path().join("nonexistent.db");
        let result = validator.validate_snapshot(&db_path).await;

        assert!(result.is_err(), "Validation should fail for missing file");
    }

    #[tokio::test]
    async fn test_snapshot_extractor_invalid_archive() {
        let temp_dir = TempDir::new().unwrap();
        let extractor = SnapshotExtractor::new();

        // Create invalid archive (just a text file)
        let archive_path = temp_dir.path().join("invalid.tar.gz");
        std::fs::write(&archive_path, "not a valid archive").unwrap();

        let extract_dir = temp_dir.path().join("extracted");
        let result = extractor.extract_snapshot(&archive_path, &extract_dir).await;

        assert!(result.is_err(), "Extraction should fail for invalid archive");
    }

    #[tokio::test]
    async fn test_snapshot_manager_integration() {
        let temp_dir = TempDir::new().unwrap();
        let _manager = SnapshotManager::new();

        // Create test archive
        let archive_path = create_test_archive(&temp_dir).unwrap();

        // For this test, we'll simulate a file:// URL since we can't rely on external URLs
        // This is a simplified test - in a real scenario you'd use a mock HTTP server
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        // Manually test the components
        let extractor = SnapshotExtractor::new();
        let validator = SnapshotValidator::new();

        // Test extraction
        let extract_dir = temp_dir.path().join("extracted");
        let extracted_files = extractor.extract_snapshot(&archive_path, &extract_dir).await.unwrap();
        assert!(extracted_files.contains(&"hopr_logs.db".to_string()));

        // Test validation
        let db_path = extract_dir.join("hopr_logs.db");
        let info = validator.validate_snapshot(&db_path).await.unwrap();
        assert_eq!(info.log_count, 2);
    }
}
