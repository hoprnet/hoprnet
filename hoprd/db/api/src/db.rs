
use hoprd_migration::{MigratorMetadata, MigratorTrait};
use sea_orm::SqlxSqliteConnector;
use sqlx::pool::PoolOptions;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use std::path::Path;
use std::time::Duration;
use tracing::log::LevelFilter;

use crate::HoprdDbAllOperations;

#[derive(Debug, Clone)]
pub struct HoprdDb {
    pub(crate) metadata: sea_orm::DatabaseConnection,
}

pub const SQL_DB_METADATA_FILE_NAME: &str = "hopr_metadata.db";

impl HoprdDb {
    pub async fn new(directory: String) -> Self {
        let dir = Path::new(&directory);
        std::fs::create_dir_all(dir).unwrap_or_else(|_| panic!("cannot create main database directory {directory}")); // hard-failure

        // Default SQLite config values for all 3 DBs.
        // Each DB can customize with its own specific values
        let cfg_template = SqliteConnectOptions::default()
            .create_if_missing(true)
            .log_slow_statements(LevelFilter::Warn, Duration::from_millis(150))
            .log_statements(LevelFilter::Debug)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .auto_vacuum(SqliteAutoVacuum::Full)
            //.optimize_on_close(true, None) // Removed, because it causes optimization on each connection, due to min_connections being set to 0
            .page_size(4096)
            .pragma("cache_size", "-30000") // 32M
            .pragma("busy_timeout", "1000"); // 1000ms

        // Peers database
        let metadata = PoolOptions::new()
            .min_connections(0) // Default is 0
            .acquire_timeout(Duration::from_secs(60)) // Default is 30
            .idle_timeout(Some(Duration::from_secs(10 * 60))) // This is the default
            .max_lifetime(Some(Duration::from_secs(30 * 60))) // This is the default
            .max_connections(300) // Default is 10
            .connect_with(cfg_template.clone().filename(dir.join(SQL_DB_METADATA_FILE_NAME)))
            .await
            .unwrap_or_else(|e| panic!("failed to create main database: {e}"));

        Self::new_sqlx_sqlite(metadata).await
    }

    pub async fn new_in_memory() -> Self {
        Self::new_sqlx_sqlite(SqlitePool::connect(":memory:").await.unwrap()).await
    }

    async fn new_sqlx_sqlite(metadata_db: SqlitePool) -> Self {
        let metadata_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(metadata_db);

        MigratorMetadata::up(&metadata_db, None)
            .await
            .expect("cannot apply database migration");

        Self { metadata: metadata_db }
    }
}

impl HoprdDbAllOperations for HoprdDb {}

#[cfg(test)]
mod tests {
    use crate::db::HoprdDb;
    use hoprd_migration::{MigratorMetadata, MigratorTrait};

    #[async_std::test]
    async fn test_basic_db_init() {
        let db = HoprdDb::new_in_memory().await;

        MigratorMetadata::status(&db.metadata).await.expect("status must be ok");
    }
}