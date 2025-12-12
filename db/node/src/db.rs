use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use hopr_internal_types::channels::ChannelId;
use hopr_primitive_types::prelude::HoprBalance;
use migration::{MigratorPeers, MigratorTickets, MigratorTrait};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, SqlxSqliteConnector};
use sqlx::{
    ConnectOptions, SqlitePool,
    pool::PoolOptions,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};
use tracing::{debug, log::LevelFilter};
use validator::Validate;

use crate::errors::NodeDbError;

/// Filename for the network peers database.
pub const SQL_DB_PEERS_FILE_NAME: &str = "hopr_peers.db";
/// Filename for the payment tickets database.
pub const SQL_DB_TICKETS_FILE_NAME: &str = "hopr_tickets.db";

pub const HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS: u64 = 5 * 60; // 5 minutes

#[derive(Clone, Debug, validator::Validate, smart_default::SmartDefault)]
pub struct HoprNodeDbConfig {
    #[default(true)]
    pub create_if_missing: bool,
    #[default(false)]
    pub force_create: bool,
    #[default(Duration::from_secs(5))]
    pub log_slow_queries: Duration,
}

#[derive(Clone)]
pub struct HoprNodeDb {
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) tickets_write_lock: Arc<async_lock::Mutex<()>>,
    pub(crate) cfg: HoprNodeDbConfig,
    // This value must be cached here, due to complicated invalidation logic.
    pub(crate) unrealized_value: moka::future::Cache<(ChannelId, u32), HoprBalance>,
}

impl HoprNodeDb {
    pub async fn new(directory: &Path, cfg: HoprNodeDbConfig) -> Result<Self, NodeDbError> {
        cfg.validate().map_err(|e| NodeDbError::Other(e.into()))?;

        fs::create_dir_all(directory).map_err(|e| NodeDbError::Other(e.into()))?;

        let peers_options = PoolOptions::new()
            .acquire_timeout(Duration::from_secs(60)) // Default is 30
            .idle_timeout(Some(Duration::from_secs(10 * 60))) // This is the default
            .max_lifetime(Some(Duration::from_secs(30 * 60))); // This is the default

        let peers = Self::create_pool(
            cfg.clone(),
            directory.to_path_buf(),
            peers_options,
            Some(0),
            Some(300),
            SQL_DB_PEERS_FILE_NAME,
        )
        .await?;

        let tickets = Self::create_pool(
            cfg.clone(),
            directory.to_path_buf(),
            PoolOptions::new(),
            Some(0),
            Some(50),
            SQL_DB_TICKETS_FILE_NAME,
        )
        .await?;

        #[cfg(feature = "sqlite")]
        Self::new_sqlx_sqlite(tickets, peers, cfg).await
    }

    #[cfg(feature = "sqlite")]
    pub async fn new_in_memory() -> Result<Self, NodeDbError> {
        Self::new_sqlx_sqlite(
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| NodeDbError::Other(e.into()))?,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| NodeDbError::Other(e.into()))?,
            Default::default(),
        )
        .await
    }

    #[cfg(feature = "sqlite")]
    async fn new_sqlx_sqlite(
        peers_db_pool: SqlitePool,
        tickets_db_pool: SqlitePool,
        cfg: HoprNodeDbConfig,
    ) -> Result<Self, NodeDbError> {
        let tickets_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(tickets_db_pool);
        MigratorTickets::up(&tickets_db, None).await?;

        let peers_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(peers_db_pool);
        MigratorPeers::up(&peers_db, None).await?;

        // Reset the peer network information
        let res = hopr_db_entity::network_peer::Entity::delete_many()
            .filter(
                sea_orm::Condition::all().add(
                    hopr_db_entity::network_peer::Column::LastSeen.lt(chrono::DateTime::<chrono::Utc>::from(
                        hopr_platform::time::native::current_time()
                            .checked_sub(std::time::Duration::from_secs(
                                std::env::var("HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS")
                                    .unwrap_or_else(|_| {
                                        HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS.to_string()
                                    })
                                    .parse::<u64>()
                                    .unwrap_or(HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS),
                            ))
                            .unwrap_or_else(hopr_platform::time::native::current_time),
                    )),
                ),
            )
            .exec(&peers_db)
            .await?;
        debug!(rows = res.rows_affected, "Cleaned up rows from the 'peers' table");

        Ok(Self {
            tickets_write_lock: Arc::new(async_lock::Mutex::new(())),
            unrealized_value: moka::future::CacheBuilder::new(10_000)
                .time_to_idle(std::time::Duration::from_secs(30))
                .build(),
            tickets_db,
            cfg,
        })
    }

    async fn create_pool(
        cfg: HoprNodeDbConfig,
        directory: PathBuf,
        mut options: PoolOptions<sqlx::Sqlite>,
        min_conn: Option<u32>,
        max_conn: Option<u32>,
        path: &str,
    ) -> Result<SqlitePool, NodeDbError> {
        if let Some(min_conn) = min_conn {
            options = options.min_connections(min_conn);
        }
        if let Some(max_conn) = max_conn {
            options = options.max_connections(max_conn);
        }

        let sqlite_cfg = SqliteConnectOptions::default()
            .create_if_missing(cfg.create_if_missing)
            .log_slow_statements(LevelFilter::Warn, cfg.log_slow_queries)
            .log_statements(LevelFilter::Debug)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .auto_vacuum(SqliteAutoVacuum::Full)
            //.optimize_on_close(true, None) // Removed, because it causes optimization on each connection, due to min_connections being set to 0
            .page_size(4096)
            .pragma("cache_size", "-30000") // 32M
            .pragma("busy_timeout", "1000"); // 1000ms

        let pool = options.connect_with(sqlite_cfg.filename(directory.join(path))).await?;

        Ok(pool)
    }

    pub fn config(&self) -> &HoprNodeDbConfig {
        &self.cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_db_init() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;
        MigratorTickets::status(&db.tickets_db).await?;

        Ok(())
    }
}
