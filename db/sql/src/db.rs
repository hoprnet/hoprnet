use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use hopr_crypto_types::{keypairs::Keypair, prelude::ChainKeypair};
use hopr_db_entity::prelude::{Account, Announcement};
use hopr_primitive_types::primitives::Address;
use migration::{MigratorChainLogs, MigratorIndex, MigratorTrait};
use sea_orm::{EntityTrait, SqlxSqliteConnector};
use sqlx::{
    ConnectOptions, SqlitePool,
    pool::PoolOptions,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};
use tracing::log::LevelFilter;
use validator::Validate;

use crate::{
    cache::{CacheKeyMapper, HoprIndexerDbCaches},
    errors::{DbSqlError, Result},
    prelude::model_to_account_entry,
};

#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
pub struct HoprIndexerDbConfig {
    #[default(true)]
    pub create_if_missing: bool,
    #[default(false)]
    pub force_create: bool,
    #[default(Duration::from_secs(5))]
    pub log_slow_queries: Duration,
}

#[cfg(feature = "sqlite")]
#[derive(Debug, Clone)]
pub(crate) struct DbConnection {
    ro: sea_orm::DatabaseConnection,
    rw: sea_orm::DatabaseConnection,
}

#[cfg(feature = "sqlite")]
impl DbConnection {
    pub fn read_only(&self) -> &sea_orm::DatabaseConnection {
        &self.ro
    }

    pub fn read_write(&self) -> &sea_orm::DatabaseConnection {
        &self.rw
    }
}

/// Main database handle for HOPR node operations.
///
/// Manages multiple SQLite databases for different data domains to avoid
/// locking conflicts and improve performance:
///
/// - **Index DB**: Blockchain indexing and contract data
/// - **Tickets DB**: Payment tickets and acknowledgments
/// - **Peers DB**: Network peer information and metadata
/// - **Logs DB**: Blockchain logs and processing status
///
/// Supports database snapshot imports for fast synchronization via
/// [`HoprDbGeneralModelOperations::import_logs_db`].
#[derive(Debug, Clone)]
pub struct HoprIndexerDb {
    pub(crate) index_db: DbConnection,
    pub(crate) logs_db: sea_orm::DatabaseConnection,
    pub(crate) me_onchain: Address,
    pub(crate) caches: HoprIndexerDbCaches,
}

/// Filename for the blockchain-indexing database.
pub const SQL_DB_INDEX_FILE_NAME: &str = "hopr_index.db";

/// Filename for the blockchain logs database (used in snapshots).
pub const SQL_DB_LOGS_FILE_NAME: &str = "hopr_logs.db";

impl HoprIndexerDb {
    pub async fn new(directory: &Path, chain_key: ChainKeypair, cfg: HoprIndexerDbConfig) -> Result<Self> {
        cfg.validate()
            .map_err(|e| DbSqlError::Construction(format!("failed configuration validation: {e}")))?;

        fs::create_dir_all(directory)
            .map_err(|_e| DbSqlError::Construction(format!("cannot create main database directory {directory:?}")))?;

        info!("Opening database at {:?}", directory);
        let index = Self::create_pool(
            cfg.clone(),
            directory.to_path_buf(),
            PoolOptions::new(),
            Some(0),
            Some(1),
            false,
            SQL_DB_INDEX_FILE_NAME,
        )
        .await?;

        info!("Creating index RW DB connection pool");
        let index_ro = Self::create_pool(
            cfg.clone(),
            directory.to_path_buf(),
            PoolOptions::new(),
            Some(0),
            Some(30),
            true,
            SQL_DB_INDEX_FILE_NAME,
        )
        .await?;

        let logs = Self::create_pool(
            cfg.clone(),
            directory.to_path_buf(),
            PoolOptions::new(),
            Some(0),
            None,
            false,
            SQL_DB_LOGS_FILE_NAME,
        )
        .await?;

        #[cfg(feature = "sqlite")]
        Self::new_sqlx_sqlite(chain_key, index, index_ro, logs).await
    }

    #[cfg(feature = "sqlite")]
    pub async fn new_in_memory(chain_key: ChainKeypair) -> Result<Self> {
        let index_db = SqlitePool::connect(":memory:")
            .await
            .map_err(|e| DbSqlError::Construction(e.to_string()))?;

        Self::new_sqlx_sqlite(
            chain_key,
            index_db.clone(),
            index_db,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| DbSqlError::Construction(e.to_string()))?,
        )
        .await
    }

    #[cfg(feature = "sqlite")]
    async fn new_sqlx_sqlite(
        chain_key: ChainKeypair,
        index_db_pool: SqlitePool,
        index_db_ro_pool: SqlitePool,
        logs_db_pool: SqlitePool,
    ) -> Result<Self> {
        let index_db_rw = SqlxSqliteConnector::from_sqlx_sqlite_pool(index_db_pool);
        let index_db_ro = SqlxSqliteConnector::from_sqlx_sqlite_pool(index_db_ro_pool);

        MigratorIndex::up(&index_db_rw, None)
            .await
            .map_err(|e| DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let logs_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(logs_db_pool.clone());

        MigratorChainLogs::up(&logs_db, None)
            .await
            .map_err(|e| DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let caches = HoprIndexerDbCaches::default();

        Account::find()
            .find_with_related(Announcement)
            .all(&index_db_rw)
            .await?
            .into_iter()
            .try_for_each(|(a, b)| match model_to_account_entry(a, b) {
                Ok(account) => caches.key_id_mapper.update_key_id_binding(&account),
                Err(error) => {
                    // Undecodeable accounts are skipped and will be unreachable
                    tracing::error!(%error, "undecodeable account");
                    Ok(())
                }
            })?;

        Ok(Self {
            me_onchain: chain_key.public().to_address(),
            index_db: DbConnection {
                ro: index_db_ro,
                rw: index_db_rw,
            },
            logs_db,
            caches,
        })
    }

    /// Default SQLite config values for all DBs with RW  (read-write) access.
    fn common_connection_cfg_rw(cfg: HoprIndexerDbConfig) -> SqliteConnectOptions {
        SqliteConnectOptions::default()
            .create_if_missing(cfg.create_if_missing)
            .log_slow_statements(LevelFilter::Warn, cfg.log_slow_queries)
            .log_statements(LevelFilter::Debug)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .auto_vacuum(SqliteAutoVacuum::Full)
            //.optimize_on_close(true, None) // Removed, because it causes optimization on each connection, due to min_connections being set to 0
            .page_size(4096)
            .pragma("cache_size", "-30000") // 32M
            .pragma("busy_timeout", "1000") // 1000ms
    }

    /// Default SQLite config values for all DBs with RO (read-only) access.
    fn common_connection_cfg_ro(cfg: HoprIndexerDbConfig) -> SqliteConnectOptions {
        SqliteConnectOptions::default()
            .create_if_missing(cfg.create_if_missing)
            .log_slow_statements(LevelFilter::Warn, cfg.log_slow_queries)
            .log_statements(LevelFilter::Debug)
            //.optimize_on_close(true, None) // Removed, because it causes optimization on each connection, due to min_connections being set to 0
            .page_size(4096)
            .pragma("cache_size", "-30000") // 32M
            .pragma("busy_timeout", "1000") // 1000ms
            .read_only(true)
    }

    pub async fn create_pool(
        cfg: HoprIndexerDbConfig,
        directory: PathBuf,
        mut options: PoolOptions<sqlx::Sqlite>,
        min_conn: Option<u32>,
        max_conn: Option<u32>,
        read_only: bool,
        path: &str,
    ) -> Result<SqlitePool> {
        if let Some(min_conn) = min_conn {
            options = options.min_connections(min_conn);
        }
        if let Some(max_conn) = max_conn {
            options = options.max_connections(max_conn);
        }

        let cfg = if read_only {
            Self::common_connection_cfg_ro(cfg)
        } else {
            Self::common_connection_cfg_rw(cfg)
        };

        let pool = options
            .connect_with(cfg.filename(directory.join(path)))
            .await
            .map_err(|e| DbSqlError::Construction(format!("failed to create {path} database: {e}")))?;

        Ok(pool)
    }

    pub fn key_id_mapper_ref(&self) -> &CacheKeyMapper {
        &self.caches.key_id_mapper
    }
}

#[cfg(test)]
mod tests {
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use migration::{MigratorChainLogs, MigratorIndex, MigratorTrait};

    use super::*;
    use crate::{HoprDbGeneralModelOperations, TargetDb}; // 0.8

    #[tokio::test]
    async fn test_basic_db_init() -> anyhow::Result<()> {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await?;

        // NOTE: cfg-if this on Postgres to do only `Migrator::status(db.conn(Default::default)).await.expect("status
        // must be ok");`
        MigratorIndex::status(db.conn(TargetDb::Index)).await?;
        MigratorChainLogs::status(db.conn(TargetDb::Logs)).await?;

        Ok(())
    }
}
