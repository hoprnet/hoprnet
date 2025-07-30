#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor, path::Path};

    use async_compression::futures::bufread::XzEncoder;
    use async_tar::Builder;
    use futures_util::io::{AllowStdIo, AsyncReadExt, BufReader as FuturesBufReader};
    use sqlx::{
        Connection, Executor,
        sqlite::{SqliteConnectOptions, SqliteConnection},
    };
    use tempfile::TempDir;
    use tracing::debug;

    use crate::{
        IndexerConfig,
        snapshot::{
            SnapshotInfo, SnapshotInstaller, SnapshotWorkflow, download::SnapshotDownloader, error::SnapshotResult,
            extract::SnapshotExtractor, validate::SnapshotValidator,
        },
    };

    /// Test-only snapshot manager without database dependencies.
    ///
    /// Provides the same snapshot workflow as [`SnapshotManager`] but installs
    /// files directly to the filesystem instead of integrating with a database.
    /// Used in unit tests where database setup would add unnecessary complexity.
    pub struct TestSnapshotManager {
        workflow: SnapshotWorkflow,
    }

    impl TestSnapshotManager {
        /// Creates a test snapshot manager without database dependencies.
        pub fn new() -> SnapshotResult<Self> {
            Ok(Self {
                workflow: SnapshotWorkflow::new()?,
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
            self.workflow.execute_workflow(self, url, data_dir, false).await
        }
    }

    #[async_trait::async_trait]
    impl SnapshotInstaller for TestSnapshotManager {
        async fn install_snapshot(
            &self,
            temp_dir: &Path,
            data_dir: &Path,
            extracted_files: &[String],
        ) -> SnapshotResult<()> {
            // Install files directly to data directory (test mode)
            fs::create_dir_all(data_dir)?;

            for file in extracted_files {
                let src = temp_dir.join(file);
                let dst = data_dir.join(file);

                // Remove existing file if it exists
                if dst.exists() {
                    fs::remove_file(&dst)?;
                }

                // Copy file to final location
                fs::copy(&src, &dst)?;
                debug!(from = %file, to = %dst.display(), "Installed snapshot file");
            }

            Ok(())
        }
    }

    /// Creates a test SQLite database for testing
    async fn create_test_sqlite_db(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let options = SqliteConnectOptions::new().filename(path).create_if_missing(true);

        let mut conn = SqliteConnection::connect_with(&options).await?;

        // Create test tables matching the actual snapshot schema
        conn.execute(
            "CREATE TABLE log (
                transaction_index blob(8) NOT NULL,
                log_index blob(8) NOT NULL,
                block_number blob(8) NOT NULL,
                block_hash blob(32) NOT NULL,
                transaction_hash blob(32) NOT NULL,
                address blob(20) NOT NULL,
                topics blob(1) NOT NULL,
                data blob(1) NOT NULL,
                removed boolean NOT NULL
            )",
        )
        .await?;

        conn.execute(
            "CREATE TABLE log_status (
                id INTEGER PRIMARY KEY,
                status TEXT NOT NULL
            )",
        )
        .await?;

        conn.execute(
            "CREATE TABLE log_topic_info (
                id INTEGER PRIMARY KEY,
                topic_hash TEXT NOT NULL
            )",
        )
        .await?;

        conn.execute(
            "CREATE TABLE seaql_migrations (
                version TEXT PRIMARY KEY,
                applied_at INTEGER NOT NULL
            )",
        )
        .await?;

        // Insert test data with proper blob format (8-byte big-endian for block numbers)
        let block_1_bytes = 1i64.to_be_bytes().to_vec();
        let block_2_bytes = 2i64.to_be_bytes().to_vec();
        let dummy_blob = vec![0u8];
        let dummy_hash32 = vec![0u8; 32];
        let dummy_hash20 = vec![0u8; 20];

        sqlx::query(
            "INSERT INTO log (transaction_index, log_index, block_number, block_hash, transaction_hash, address, \
             topics, data, removed) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&block_1_bytes) // transaction_index
        .bind(&block_1_bytes) // log_index
        .bind(&block_1_bytes) // block_number
        .bind(&dummy_hash32) // block_hash
        .bind(&dummy_hash32) // transaction_hash
        .bind(&dummy_hash20) // address
        .bind(&dummy_blob) // topics
        .bind(&dummy_blob) // data
        .bind(false) // removed
        .execute(&mut conn)
        .await?;

        sqlx::query(
            "INSERT INTO log (transaction_index, log_index, block_number, block_hash, transaction_hash, address, \
             topics, data, removed) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&block_2_bytes) // transaction_index
        .bind(&block_2_bytes) // log_index
        .bind(&block_2_bytes) // block_number
        .bind(&dummy_hash32) // block_hash
        .bind(&dummy_hash32) // transaction_hash
        .bind(&dummy_hash20) // address
        .bind(&dummy_blob) // topics
        .bind(&dummy_blob) // data
        .bind(false) // removed
        .execute(&mut conn)
        .await?;

        conn.execute("INSERT INTO log_status (status) VALUES ('active')")
            .await?;

        conn.execute("INSERT INTO log_topic_info (topic_hash) VALUES ('0x123')")
            .await?;

        conn.execute("INSERT INTO seaql_migrations (version, applied_at) VALUES ('v1', 1234567890)")
            .await?;

        Ok(())
    }

    /// Creates a test tar.xz archive containing a SQLite database
    async fn create_test_archive(temp_dir: &TempDir) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        // Create the database
        let db_path = temp_dir.path().join("hopr_logs.db");
        create_test_sqlite_db(&db_path).await?;

        // First create uncompressed tar in memory
        let mut tar_data = Vec::new();
        {
            let mut tar = Builder::new(&mut tar_data);
            tar.append_path_with_name(&db_path, "hopr_logs.db").await?;
            tar.into_inner().await?;
        }

        // Now compress with xz using async_compression
        let cursor = Cursor::new(tar_data);
        let buf_reader = FuturesBufReader::new(AllowStdIo::new(cursor));
        let mut encoder = XzEncoder::new(buf_reader);

        // Read compressed data
        let mut compressed_data = Vec::new();
        encoder.read_to_end(&mut compressed_data).await?;

        // Write to final archive file
        let archive_path = temp_dir.path().join("test_snapshot.tar.xz");
        fs::write(&archive_path, compressed_data)?;

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
        assert_eq!(info.tables, 4);
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
        let archive_path = temp_dir.path().join("invalid.tar.xz");
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
    async fn test_disk_space_validation() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = SnapshotDownloader::new().expect("Failed to create SnapshotDownloader");

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
        let downloader = SnapshotDownloader::new().expect("Failed to create SnapshotDownloader");

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
        // Test with empty data directory
        let config = IndexerConfig::new(
            0,
            true,
            true,
            Some("https://example.com/snapshot.tar.xz".to_string()),
            "".to_string(),
        );

        assert!(config.data_directory.is_empty());

        // Test with valid data directory
        let config = IndexerConfig::new(
            0,
            true,
            true,
            Some("https://example.com/snapshot.tar.xz".to_string()),
            "/tmp/test_data".to_string(),
        );

        assert!(!config.data_directory.is_empty());
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

    #[tokio::test]
    async fn test_configuration_validation() {
        // Test IndexerConfig::new with all parameters
        let config = IndexerConfig::new(
            100,
            true,
            true,
            Some("https://example.com/snapshot.tar.xz".to_string()),
            "/tmp/hopr_data".to_string(),
        );

        assert_eq!(config.start_block_number, 100);
        assert_eq!(config.fast_sync, true);
        assert_eq!(config.enable_logs_snapshot, true);
        assert_eq!(
            config.logs_snapshot_url,
            Some("https://example.com/snapshot.tar.xz".to_string())
        );
        assert_eq!(config.data_directory, "/tmp/hopr_data");

        // Test validation - valid config
        assert!(config.validate().is_ok());
        assert!(config.is_valid());

        // Test validation - missing URL
        let invalid_url_config = IndexerConfig::new(100, true, true, None, "/tmp/hopr_data".to_string());
        assert!(invalid_url_config.validate().is_err());
        assert!(!invalid_url_config.is_valid());

        // Test validation - snapshots disabled (should be valid even with empty fields)
        let disabled_config = IndexerConfig::new(100, true, false, Some("".to_string()), "".to_string());
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
