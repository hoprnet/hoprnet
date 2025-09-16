use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use futures::TryFutureExt;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, SqlxSqliteConnector, QueryFilter, ColumnTrait};
use sqlx::pool::PoolOptions;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use tracing::debug;
use tracing::log::LevelFilter;
use validator::Validate;
use hopr_crypto_types::keypairs::ChainKeypair;
use hopr_crypto_types::prelude::Keypair;
use hopr_db_api::errors::DbError;
use hopr_db_api::info::SafeInfo;
use hopr_db_entity::{node_info,};
use hopr_primitive_types::prelude::ToHex;
use hopr_primitive_types::primitives::Address;
use migration::{MigratorPeers, MigratorTickets, MigratorTrait};

use crate::cache::{CachedValue, CachedValueDiscriminants, HoprDbCaches};
use crate::{NodeDbTx, OpenTransaction, SINGULAR_TABLE_FIXED_ID};
use crate::ticket_manager::TicketManager;

/// Filename for the network peers database.
pub const SQL_DB_PEERS_FILE_NAME: &str = "hopr_peers.db";
/// Filename for the payment tickets database.
pub const SQL_DB_TICKETS_FILE_NAME: &str = "hopr_tickets.db";

pub const HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS: u64 = 5 * 60; // 5 minutes

pub const MIN_SURB_RING_BUFFER_SIZE: usize = 1024;

#[derive(Clone, Debug, validator::Validate, smart_default::SmartDefault)]
pub struct HoprNodeDbConfig {
    #[default(true)]
    pub create_if_missing: bool,
    #[default(false)]
    pub force_create: bool,
    #[default(Duration::from_secs(5))]
    pub log_slow_queries: Duration,
    #[default(10_000)]
    #[validate(range(min = MIN_SURB_RING_BUFFER_SIZE))]
    pub surb_ring_buffer_size: usize,
    #[default(1000)]
    #[validate(range(min = 2))]
    pub surb_distress_threshold: usize,
}

#[derive(Clone)]
pub struct HoprNodeDb {
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) peers_db: sea_orm::DatabaseConnection,
    pub(crate) ticket_manager: Arc<TicketManager>,
    pub(crate) caches: Arc<HoprDbCaches>,
    pub(crate) me_onchain: ChainKeypair,
    pub(crate) me_address: Address,
    pub(crate) cfg: HoprNodeDbConfig,
}

impl HoprNodeDb {
    pub async fn new(directory: &Path, chain_key: ChainKeypair, cfg: HoprNodeDbConfig) -> Result<Self, DbError> {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&crate::protocol::METRIC_RECEIVED_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_SENT_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_TICKETS_COUNT);
        }

        cfg.validate()
            .map_err(|e| DbError::General(format!("failed configuration validation: {e}")))?;

        fs::create_dir_all(directory)
            .map_err(|_e| DbError::General(format!("cannot create main database directory {directory:?}")))?;

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
        Self::new_sqlx_sqlite(chain_key, tickets, peers, cfg).await
    }

    #[cfg(feature = "sqlite")]
    pub async fn new_in_memory(chain_key: ChainKeypair) -> Result<Self, DbError> {
        Self::new_sqlx_sqlite(
            chain_key,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| DbError::General(e.to_string()))?,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| DbError::General(e.to_string()))?,
            Default::default(),
        )
            .await
    }

    #[cfg(feature = "sqlite")]
    async fn new_sqlx_sqlite(
        me_onchain: ChainKeypair,
        peers_db_pool: SqlitePool,
        tickets_db_pool: SqlitePool,
        cfg: HoprNodeDbConfig,
    ) -> Result<Self, DbError> {
        let tickets_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(tickets_db_pool);

        MigratorTickets::up(&tickets_db, None)
            .await
            .map_err(|e| DbError::General(format!("cannot apply database migration: {e}")))?;

        let peers_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(peers_db_pool);

        MigratorPeers::up(&peers_db, None)
            .await
            .map_err(|e| DbError::General(format!("cannot apply database migration: {e}")))?;

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
            .await
            .map_err(|e| DbError::General(format!("must reset peers on init: {e}")))?;
        debug!(rows = res.rows_affected, "Cleaned up rows from the 'peers' table");

        let caches = Arc::new(HoprDbCaches::default());
        caches.invalidate_all();

        // TODO: (dbmig) initialize key-id mapper via the HoprChain
        // Initialize KeyId mapping for accounts
        /*Account::find()
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
            })?;*/

        Ok(Self {
            ticket_manager: Arc::new(TicketManager::new(tickets_db.clone(), caches.clone())),
            tickets_db,
            peers_db,
            caches,
            me_address: me_onchain.public().to_address(),
            me_onchain,
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
    ) -> Result<SqlitePool, DbError> {
        if let Some(min_conn) = min_conn {
            options = options.min_connections(min_conn);
        }
        if let Some(max_conn) = max_conn {
            options = options.max_connections(max_conn);
        }

        let sqlite_cfg =  SqliteConnectOptions::default()
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

        let pool = options
            .connect_with(sqlite_cfg.filename(directory.join(path)))
            .await
            .map_err(|e| DbError::General(format!("failed to create {path} database: {e}")))?;

        Ok(pool)
    }

    // TODO: move this into separate trait

    async fn get_safe_info<'a>(&'a self, tx: OptTx<'a, TargetNodeDb>) -> crate::errors::Result<Option<SafeInfo>> {
        let myself = self.clone();
        Ok(self
            .caches
            .single_values
            .try_get_with_by_ref(&CachedValueDiscriminants::SafeInfoCache, async move {
                myself
                    .nest_transaction(tx)
                    .and_then(|op| {
                        op.perform(|tx| {
                            Box::pin(async move {
                                let info = node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                                    .one(tx.as_ref())
                                    .await?
                                    .ok_or(MissingFixedTableEntry("node_info".into()))?;
                                Ok::<_, DbSqlError>(info.safe_address.zip(info.module_address))
                            })
                        })
                    })
                    .await
                    .and_then(|addrs| {
                        if let Some((safe_address, module_address)) = addrs {
                            Ok(Some(SafeInfo {
                                safe_address: safe_address.parse()?,
                                module_address: module_address.parse()?,
                            }))
                        } else {
                            Ok(None)
                        }
                    })
                    .map(CachedValue::SafeInfoCache)
            })
            .await?
            .try_into()?)
    }

    async fn set_safe_info<'a>(&'a self, tx: OptTx<'a, TargetNodeDb>, safe_info: SafeInfo) -> crate::errors::Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        safe_address: Set(Some(safe_info.safe_address.to_hex())),
                        module_address: Set(Some(safe_info.module_address.to_hex())),
                        ..Default::default()
                    }
                        .update(tx.as_ref()) // DB is primed in the migration, so only update is needed
                        .await?;
                    Ok::<_, DbSqlError>(())
                })
            })
            .await?;
        self.caches
            .single_values
            .insert(
                CachedValueDiscriminants::SafeInfoCache,
                CachedValue::SafeInfoCache(Some(safe_info)),
            )
            .await;
        Ok(())
    }
}