//! SQLite database validation for snapshot integrity verification.
//!
//! Validates extracted snapshot databases to ensure they contain expected
//! tables, data, and are not corrupted before installation.

use std::{fs, path::Path};

use sqlx::{
    Connection,
    sqlite::{SqliteConnectOptions, SqliteConnection},
};
use tracing::{info, warn};

use crate::snapshot::error::{SnapshotError, SnapshotResult};

/// Metadata about a validated snapshot database.
///
/// Contains information gathered during validation that describes
/// the contents and state of the snapshot database.
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    /// Total number of log entries in the snapshot
    pub log_count: u64,
    /// Highest block number found in the snapshot (if any)
    pub latest_block: Option<u64>,
    /// Number of database tables found
    pub tables: usize,
    /// SQLite version used to create the database
    pub sqlite_version: String,
    /// Database file size in bytes
    pub db_size: u64,
}

/// Validates SQLite snapshot databases for integrity and expected content.
///
/// Performs comprehensive validation including database connectivity,
/// schema verification, and data integrity checks.
#[derive(Default, PartialEq, Eq)]
pub struct SnapshotValidator {
    /// Required tables that must exist in valid snapshot databases
    expected_tables: Vec<String>,
}

impl SnapshotValidator {
    /// Creates a new validator with predefined expected tables.
    ///
    /// Expected tables include:
    /// - `log` - Blockchain log entries
    /// - `log_status` - Log processing status
    /// - `log_topic_info` - Log topic information
    /// - `seaql_migrations` - Database migration tracking
    ///
    /// Note: `sqlite_sequence` is not required as it's automatically managed by SQLite
    pub fn new() -> Self {
        Self {
            expected_tables: vec![
                "log".to_string(),
                "log_status".to_string(),
                "log_topic_info".to_string(),
                "seaql_migrations".to_string(),
            ],
        }
    }

    /// Validates a snapshot database for integrity and expected content.
    ///
    /// Performs comprehensive validation:
    /// 1. File existence and accessibility
    /// 2. Database connectivity
    /// 3. Schema validation (expected tables)
    /// 4. Data integrity checks
    /// 5. Content statistics gathering
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file to validate
    ///
    /// # Returns
    ///
    /// [`SnapshotInfo`] containing validation results and database metadata
    ///
    /// # Errors
    ///
    /// * [`SnapshotError::Validation`] - Database corruption or schema issues
    /// * [`SnapshotError::Io`] - File access errors
    pub async fn validate_snapshot(&self, db_path: &Path) -> SnapshotResult<SnapshotInfo> {
        info!(db = %db_path.display(), "Validating logs snapshot database");

        let snapshot_info = Self::validate_sqlite_db(db_path, &self.expected_tables).await?;
        info!(?snapshot_info, "Logs snapshot validation successful");

        Ok(snapshot_info)
    }

    /// Validates the SQLite database
    async fn validate_sqlite_db(db_path: &Path, expected_tables: &[String]) -> SnapshotResult<SnapshotInfo> {
        // Check if file exists
        if !db_path.exists() {
            return Err(SnapshotError::Validation("Database file does not exist".to_string()));
        }

        // Get file size
        let metadata = fs::metadata(db_path)?;
        let db_size = metadata.len();

        // Open database in read-only mode using sqlx
        let options = SqliteConnectOptions::new().filename(db_path).read_only(true);

        let mut conn = SqliteConnection::connect_with(&options)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Cannot open database: {e}")))?;

        // Check database integrity
        let integrity_check: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&mut conn)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Integrity check failed: {e}")))?;

        if integrity_check != "ok" {
            return Err(SnapshotError::Validation(format!(
                "Database integrity check failed: {integrity_check}",
            )));
        }

        // Get SQLite version
        let sqlite_version: String = sqlx::query_scalar("SELECT sqlite_version()")
            .fetch_one(&mut conn)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Cannot get SQLite version: {e}")))?;

        // Check for expected tables
        let tables: Vec<String> = sqlx::query_scalar::<_, String>("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&mut conn)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Cannot query tables: {e}")))?;

        // Verify expected tables exist with strict validation
        let missing_tables = expected_tables
            .iter()
            .filter(|t| !tables.contains(t))
            .map(|t| t.to_string())
            .collect::<Vec<_>>();
        if !missing_tables.is_empty() {
            return Err(SnapshotError::Validation(format!(
                "Missing required table(s): {}. Available tables: [{}]",
                missing_tables.join(", "),
                tables.join(", ")
            )));
        }

        // Get snapshot metadata using the log table (block_number stored as 8-byte big-endian blob)
        let log_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM log")
            .fetch_one(&mut conn)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Cannot count logs: {e}")))?;

        // Get latest block number from log table (handling blob format)
        let latest_block: Option<i64> = match sqlx::query_scalar::<_, Vec<u8>>("SELECT MAX(block_number) FROM log")
            .fetch_optional(&mut conn)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Cannot get latest block: {e}")))?
        {
            Some(blob) if blob.len() == 8 => {
                // Convert 8-byte big-endian blob to i64
                let bytes: [u8; 8] = blob
                    .try_into()
                    .map_err(|_| SnapshotError::Validation("Invalid block_number blob length".to_string()))?;
                Some(i64::from_be_bytes(bytes))
            }
            Some(_) => {
                return Err(SnapshotError::Validation(
                    "Invalid block_number blob format".to_string(),
                ));
            }
            None => None,
        };

        // Validate that we have some data
        if log_count == 0 {
            warn!("Snapshot database contains no logs");
        }

        Ok(SnapshotInfo {
            log_count: log_count as u64,
            latest_block: latest_block.map(|b| b as u64),
            tables: tables.len(),
            sqlite_version,
            db_size,
        })
    }

    /// Checks data consistency
    pub async fn check_data_consistency(&self, db_path: &Path) -> SnapshotResult<()> {
        let options = SqliteConnectOptions::new().filename(db_path).read_only(true);

        let mut conn = SqliteConnection::connect_with(&options)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Cannot open database: {e}")))?;

        // Check for NULL block_numbers (blob columns can't be negative in the same way)
        let invalid_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM log WHERE block_number IS NULL")
            .fetch_one(&mut conn)
            .await
            .map_err(|e| SnapshotError::Validation(format!("Cannot check log consistency: {e}")))?;

        if invalid_logs > 0 {
            return Err(SnapshotError::Validation(format!(
                "Found {invalid_logs} logs with invalid block numbers",
            )));
        }

        Ok(())
    }
}
