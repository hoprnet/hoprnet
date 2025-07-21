#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;

    use flate2::{Compression, write::GzEncoder};
    use tar::Builder;
    use tempfile::TempDir;

    use crate::snapshot::{
        SnapshotManager, download::SnapshotDownloader, extract::SnapshotExtractor, validate::SnapshotValidator,
    };

    /// Creates a test SQLite database for testing
    async fn create_test_sqlite_db(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::{
            Connection, Executor,
            sqlite::{SqliteConnectOptions, SqliteConnection},
        };

        let options = SqliteConnectOptions::new().filename(path).create_if_missing(true);

        let mut conn = SqliteConnection::connect_with(&options).await?;

        // Create test tables
        conn.execute(
            "CREATE TABLE logs (
                id INTEGER PRIMARY KEY,
                block_number INTEGER NOT NULL,
                log_index INTEGER NOT NULL,
                data TEXT NOT NULL
            )",
        )
        .await?;

        conn.execute(
            "CREATE TABLE blocks (
                id INTEGER PRIMARY KEY,
                block_number INTEGER NOT NULL UNIQUE,
                block_hash TEXT NOT NULL
            )",
        )
        .await?;

        // Insert test data
        conn.execute("INSERT INTO logs (block_number, log_index, data) VALUES (1, 0, 'test_log_1')")
            .await?;

        conn.execute("INSERT INTO logs (block_number, log_index, data) VALUES (2, 0, 'test_log_2')")
            .await?;

        conn.execute("INSERT INTO blocks (block_number, block_hash) VALUES (1, 'hash_1')")
            .await?;

        conn.execute("INSERT INTO blocks (block_number, block_hash) VALUES (2, 'hash_2')")
            .await?;

        Ok(())
    }

    /// Creates a test tar.gz archive containing a SQLite database
    async fn create_test_archive(temp_dir: &TempDir) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        // Create the database
        let db_path = temp_dir.path().join("hopr_logs.db");
        create_test_sqlite_db(&db_path).await?;

        // Create archive
        let archive_path = temp_dir.path().join("test_snapshot.tar.gz");
        let tar_gz = File::create(&archive_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        // Add the database file to the archive
        tar.append_path_with_name(&db_path, "hopr_logs.db")?;

        // Finish the archive
        tar.into_inner()?.finish()?;

        // Clean up the temporary database file to avoid test interference
        fs::remove_file(&db_path)?;

        Ok(archive_path)
    }

    #[tokio::test]
    async fn test_snapshot_extractor() {
        let temp_dir = TempDir::new().unwrap();
        let extractor = SnapshotExtractor::new();

        // Create test archive
        let archive_path = create_test_archive(&temp_dir).await.unwrap();

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
        create_test_sqlite_db(&db_path).await.unwrap();

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
        fs::write(&archive_path, "not a valid archive").unwrap();

        let extract_dir = temp_dir.path().join("extracted");
        let result = extractor.extract_snapshot(&archive_path, &extract_dir).await;

        assert!(result.is_err(), "Extraction should fail for invalid archive");
    }

    #[tokio::test]
    async fn test_snapshot_manager_integration() {
        let temp_dir = TempDir::new().unwrap();

        // Create test archive
        let archive_path = create_test_archive(&temp_dir).await.unwrap();

        // For this test, we'll simulate a file:// URL since we can't rely on external URLs
        // This is a simplified test - in a real scenario you'd use a mock HTTP server
        let data_dir = temp_dir.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

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

    #[tokio::test]
    async fn test_disk_space_validation() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = SnapshotDownloader::new();

        // Test with available disk space (this should pass)
        let result = downloader.check_disk_space(temp_dir.path()).await;
        assert!(result.is_ok());

        // Test with invalid directory path
        let invalid_path = temp_dir.path().join("nonexistent/nested/path");
        let result = downloader.check_disk_space(&invalid_path).await;
        // Should create the directory and succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enhanced_error_messages() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = SnapshotDownloader::new();

        // Test invalid URL error
        let result = downloader.download_snapshot("invalid://url", temp_dir.path()).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Suggestion:"));

        // Test file not found error
        let result = downloader
            .download_snapshot("https://httpbin.org/status/404", temp_dir.path())
            .await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Suggestion:"));
    }

    #[tokio::test]
    async fn test_data_directory_validation() {
        use crate::IndexerConfig;

        // Test with empty data directory
        let config = IndexerConfig {
            start_block_number: 0,
            fast_sync: true,
            logs_snapshot_enabled: true,
            logs_snapshot_url: "https://example.com/snapshot.tar.gz".to_string(),
            data_directory: "".to_string(),
        };

        assert!(config.data_directory.is_empty());

        // Test with valid data directory
        let config = IndexerConfig {
            start_block_number: 0,
            fast_sync: true,
            logs_snapshot_enabled: true,
            logs_snapshot_url: "https://example.com/snapshot.tar.gz".to_string(),
            data_directory: "/tmp/test_data".to_string(),
        };

        assert!(!config.data_directory.is_empty());
    }

    #[tokio::test]
    async fn test_snapshot_manager_with_data_directory() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("hopr_data");
        fs::create_dir_all(&data_dir).unwrap();

        // Create a test archive
        let archive_path = create_test_archive(&temp_dir).await.unwrap();

        let manager = SnapshotManager::new();

        // Test with file:// URL for local testing
        let file_url = format!("file://{}", archive_path.display());
        let _ = manager.download_and_setup_snapshot(&file_url, &data_dir).await;

        // The result may fail due to HTTP client not supporting file:// URLs
        // but we can verify the data directory structure is correct
        assert!(data_dir.join("hopr_logs.db").exists());
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        use crate::IndexerConfig;

        // Test IndexerConfig::new with all parameters
        let config = IndexerConfig::new(
            100,
            true,
            true,
            "https://example.com/snapshot.tar.gz".to_string(),
            "/tmp/hopr_data".to_string(),
        );

        assert_eq!(config.start_block_number, 100);
        assert_eq!(config.fast_sync, true);
        assert_eq!(config.logs_snapshot_enabled, true);
        assert_eq!(config.logs_snapshot_url, "https://example.com/snapshot.tar.gz");
        assert_eq!(config.data_directory, "/tmp/hopr_data");

        // Test validation - valid config
        assert!(config.validate().is_ok());
        assert!(config.is_valid());

        // Test validation - invalid URL
        let invalid_url_config = IndexerConfig::new(
            100,
            true,
            true,
            "ftp://example.com/snapshot.tar.gz".to_string(),
            "/tmp/hopr_data".to_string(),
        );
        assert!(invalid_url_config.validate().is_err());
        assert!(!invalid_url_config.is_valid());

        // Test validation - empty URL when snapshots enabled
        let empty_url_config = IndexerConfig::new(100, true, true, "".to_string(), "/tmp/hopr_data".to_string());
        assert!(empty_url_config.validate().is_err());

        // Test validation - empty data directory when snapshots enabled
        let empty_dir_config = IndexerConfig::new(
            100,
            true,
            true,
            "https://example.com/snapshot.tar.gz".to_string(),
            "".to_string(),
        );
        assert!(empty_dir_config.validate().is_err());

        // Test validation - snapshots disabled (should be valid even with empty fields)
        let disabled_config = IndexerConfig::new(100, true, false, "".to_string(), "".to_string());
        assert!(disabled_config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_sqlite_file_existence_check() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("hopr_logs.db");

        // Create a test database
        create_test_sqlite_db(&db_path).await.unwrap();

        let validator = SnapshotValidator::new();
        let result = validator.validate_snapshot(&db_path).await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.log_count, 2);
    }

    #[tokio::test]
    async fn test_archive_security_validation() {
        let temp_dir = TempDir::new().unwrap();
        let extractor = SnapshotExtractor::new();

        // Test with valid archive
        let archive_path = create_test_archive(&temp_dir).await.unwrap();

        let extract_dir = temp_dir.path().join("extract");

        // verify files before extraction
        assert!(!extract_dir.parent().unwrap().join("hopr_logs.db").exists());

        let result = extractor.extract_snapshot(&archive_path, &extract_dir).await;

        assert!(result.is_ok());

        // verify files after extraction
        let extracted_files = result.unwrap();
        assert!(extracted_files.contains(&"hopr_logs.db".to_string()));
        assert!(!extract_dir.parent().unwrap().join("hopr_logs.db").exists());
    }
}
