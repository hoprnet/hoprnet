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

use crate::snapshot::{SnapshotInfo, SnapshotInstaller, SnapshotWorkflow, error::SnapshotResult};

/// Test-only snapshot manager without database dependencies.
///
/// Provides the same snapshot workflow as [`SnapshotManager`] but installs
/// files directly to the filesystem instead of integrating with a database.
/// Used in unit tests where database setup would add unnecessary complexity.
pub struct TestSnapshotManager {
    pub(crate) workflow: SnapshotWorkflow,
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
pub async fn create_test_sqlite_db(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
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
        "INSERT INTO log (transaction_index, log_index, block_number, block_hash, transaction_hash, address, topics, \
         data, removed) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        "INSERT INTO log (transaction_index, log_index, block_number, block_hash, transaction_hash, address, topics, \
         data, removed) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
pub(crate) async fn create_test_archive(temp_dir: &TempDir) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
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
