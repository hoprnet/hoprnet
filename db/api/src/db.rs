use migration::{Migrator, MigratorTrait};
use sea_orm::SqlxSqliteConnector;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use std::path::Path;
use std::time::Duration;
use tracing::log::LevelFilter;

#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault)]
pub struct HoprDbConfig {
    #[default(true)]
    pub create_if_missing: bool,
    #[default(false)]
    pub force_create: bool,
    #[default(Duration::from_secs(5))]
    pub log_slow_queries: Duration,
}

#[derive(Debug, Clone)]
pub struct HoprDb {
    pub(crate) db: sea_orm::DatabaseConnection,
}

pub const SQL_DB_FILE_NAME: &str = "hopr_db_1.db";

impl HoprDb {
    pub async fn new(directory: String, cfg: HoprDbConfig) -> Self {
        let dir = Path::new(&directory);
        std::fs::create_dir_all(dir).unwrap_or_else(|_| panic!("cannot create main database directory {directory}")); // hard-failure

        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::default()
                .filename(dir.join(SQL_DB_FILE_NAME))
                .create_if_missing(cfg.create_if_missing)
                .log_slow_statements(LevelFilter::Warn, cfg.log_slow_queries)
                .log_statements(LevelFilter::Debug)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .auto_vacuum(SqliteAutoVacuum::Full)
                .optimize_on_close(true, None)
                .page_size(4096)
                .pragma("cache_size", "-30000"), // 32M
        )
        .await
        .unwrap_or_else(|e| panic!("failed to create main database: {e}"));

        Self::new_sqlx_sqlite(pool).await
    }

    pub async fn new_in_memory() -> Self {
        Self::new_sqlx_sqlite(SqlitePool::connect(":memory:").await.unwrap()).await
    }

    async fn new_sqlx_sqlite(pool: SqlitePool) -> Self {
        let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);

        Migrator::up(&db, None).await.expect("cannot apply database migration");

        Self { db }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::HoprDb;
    use crate::HoprDbGeneralModelOperations;
    use migration::{Migrator, MigratorTrait};

    #[async_std::test]
    async fn test_basic_db_init() {
        let db = HoprDb::new_in_memory().await;

        Migrator::status(db.conn()).await.expect("status must be ok");
    }
}
