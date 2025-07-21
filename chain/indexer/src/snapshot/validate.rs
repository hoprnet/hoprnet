use std::{fs, path::Path};

use rusqlite::{Connection, OpenFlags};
use tracing::{info, warn};

use crate::snapshot::error::{SnapshotError, SnapshotResult};

/// Information about a validated snapshot
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    /// Number of log entries in the snapshot
    pub log_count: u64,
    /// Latest block number in the snapshot
    pub latest_block: Option<u64>,
    /// Number of tables in the database
    pub tables: usize,
    /// SQLite database version
    pub sqlite_version: String,
    /// Size of the database file in bytes
    pub db_size: u64,
}

/// Handles validation of snapshot SQLite databases
pub struct SnapshotValidator {
    /// Expected tables that should exist in the logs database
    expected_tables: Vec<String>,
}

impl SnapshotValidator {
    /// Creates a new snapshot validator
    pub fn new() -> Self {
        Self {
            expected_tables: vec!["logs".to_string(), "blocks".to_string()],
        }
    }

    /// Validates a snapshot database
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Returns
    ///
    /// Information about the validated snapshot
    pub async fn validate_snapshot(&self, db_path: &Path) -> SnapshotResult<SnapshotInfo> {
        info!("Validating snapshot database at {:?}", db_path);

        let db_path = db_path.to_path_buf();
        let expected_tables = self.expected_tables.clone();

        let info = Self::validate_sqlite_db(&db_path, &expected_tables)?;

        info!("Snapshot validation successful: {:?}", info);
        Ok(info)
    }

    /// Performs comprehensive validation of the database
    pub async fn comprehensive_validation(&self, db_path: &Path) -> SnapshotResult<SnapshotInfo> {
        // Basic validation
        let info = self.validate_snapshot(db_path).await?;

        // Additional safety checks
        self.check_database_version(db_path).await?;

        Ok(info)
    }

    /// Validates the SQLite database (blocking operation)
    fn validate_sqlite_db(db_path: &Path, expected_tables: &[String]) -> SnapshotResult<SnapshotInfo> {
        // Check if file exists
        if !db_path.exists() {
            return Err(SnapshotError::Validation("Database file does not exist".to_string()));
        }

        // Get file size
        let metadata = fs::metadata(db_path)?;
        let db_size = metadata.len();

        // Open database in read-only mode
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| SnapshotError::Validation(format!("Cannot open database: {e}")))?;

        // Check database integrity
        let integrity_check: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| SnapshotError::Validation(format!("Integrity check failed: {e}")))?;

        if integrity_check != "ok" {
            return Err(SnapshotError::Validation(format!(
                "Database integrity check failed: {integrity_check}",
            )));
        }

        // Get SQLite version
        let sqlite_version: String = conn
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .map_err(|e| SnapshotError::Validation(format!("Cannot get SQLite version: {e}")))?;

        // Check for expected tables
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .map_err(|e| SnapshotError::Validation(format!("Cannot query tables: {e}")))?
            .query_map([], |row| row.get(0))
            .map_err(|e| SnapshotError::Validation(format!("Cannot read tables: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SnapshotError::Validation(format!("Cannot collect tables: {e}")))?;

        // Verify expected tables exist
        for expected in expected_tables {
            if !tables.contains(expected) {
                return Err(SnapshotError::Validation(
                    format!("Missing expected table: {expected}",),
                ));
            }
        }

        // Get snapshot metadata
        let log_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM logs", [], |row| row.get(0))
            .map_err(|e| SnapshotError::Validation(format!("Cannot count logs: {e}")))?;

        let latest_block: Option<i64> = conn
            .query_row("SELECT MAX(block_number) FROM logs", [], |row| row.get(0))
            .map_err(|e| SnapshotError::Validation(format!("Cannot get latest block: {e}")))?;

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

    /// Checks database version compatibility
    async fn check_database_version(&self, db_path: &Path) -> SnapshotResult<()> {
        let db_path = db_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e| SnapshotError::Validation(format!("Cannot open database: {e}")))?;

            // Check SQLite version compatibility
            let version: String = conn
                .query_row("SELECT sqlite_version()", [], |row| row.get(0))
                .map_err(|e| SnapshotError::Validation(format!("Cannot get SQLite version: {e}")))?;

            // Add version compatibility checks as needed
            info!("Snapshot database SQLite version: {}", version);

            Ok(())
        })
        .await?
    }

    /// Checks data consistency
    pub async fn check_data_consistency(&self, db_path: &Path) -> SnapshotResult<()> {
        let db_path = db_path.to_path_buf();
        let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| SnapshotError::Validation(format!("Cannot open database: {e}")))?;

        let invalid_logs: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM logs WHERE block_number IS NULL OR block_number < 0",
                [],
                |row| row.get(0),
            )
            .map_err(|e| SnapshotError::Validation(format!("Cannot check log consistency: {e}")))?;

        if invalid_logs > 0 {
            return Err(SnapshotError::Validation(format!(
                "Found {invalid_logs} logs with invalid block numbers",
            )));
        }

        Ok(())
    }
}

impl Default for SnapshotValidator {
    fn default() -> Self {
        Self::new()
    }
}
