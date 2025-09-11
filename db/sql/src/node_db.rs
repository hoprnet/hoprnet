use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use futures::channel::mpsc::UnboundedSender;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, SqlxSqliteConnector};
use sea_query::Expr;
use sqlx::pool::PoolOptions;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use tracing::debug;
use tracing::log::LevelFilter;
use validator::Validate;
use hopr_crypto_types::keypairs::ChainKeypair;
use hopr_db_api::info::SafeInfo;
use hopr_db_entity::{node_info, ticket};
use hopr_db_entity::prelude::{Account, Announcement};
use hopr_internal_types::prelude::{AcknowledgedTicket, AcknowledgedTicketStatus};
use hopr_primitive_types::prelude::ToHex;
use migration::{MigratorPeers, MigratorTickets, MigratorTrait};

use crate::accounts::model_to_account_entry;
use crate::cache::{CachedValue, CachedValueDiscriminants, HoprDbCaches};
use crate::db::{DbConnection, HoprDbConfig, HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS, SQL_DB_INDEX_FILE_NAME, SQL_DB_LOGS_FILE_NAME, SQL_DB_PEERS_FILE_NAME, SQL_DB_TICKETS_FILE_NAME};
use crate::errors::DbSqlError;
use crate::{OptTx, SINGULAR_TABLE_FIXED_ID};
use crate::errors::DbSqlError::MissingFixedTableEntry;
use crate::ticket_manager::TicketManager;

#[derive(Clone, Debug, validator::Validate)]
pub struct HoprNodeDbConfig {

}

#[derive(Clone)]
pub struct HoprNodeDb {
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) peers_db: sea_orm::DatabaseConnection,
    pub(crate) ticket_manager: Arc<TicketManager>,
    pub(crate) caches: Arc<HoprDbCaches>,
    me_onchain: ChainKeypair,
    cfg: HoprNodeDbConfig,
}

impl HoprNodeDb {
    pub async fn new(directory: &Path, chain_key: ChainKeypair, cfg: HoprNodeDbConfig) -> crate::errors::Result<Self> {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&crate::protocol::METRIC_RECEIVED_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_SENT_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_TICKETS_COUNT);
        }

        cfg.validate()
            .map_err(|e| DbSqlError::Construction(format!("failed configuration validation: {e}")))?;

        fs::create_dir_all(directory)
            .map_err(|_e| DbSqlError::Construction(format!("cannot create main database directory {directory:?}")))?;

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
    pub async fn new_in_memory(chain_key: ChainKeypair) -> crate::errors::Result<Self> {
        Self::new_sqlx_sqlite(
            chain_key,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| DbSqlError::Construction(e.to_string()))?,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| DbSqlError::Construction(e.to_string()))?,
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
    ) -> crate::errors::Result<Self> {
        let tickets_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(tickets_db_pool);

        MigratorTickets::up(&tickets_db, None)
            .await
            .map_err(|e| DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let peers_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(peers_db_pool);

        MigratorPeers::up(&peers_db, None)
            .await
            .map_err(|e| DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

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
            .map_err(|e| DbSqlError::Construction(format!("must reset peers on init: {e}")))?;
        debug!(rows = res.rows_affected, "Cleaned up rows from the 'peers' table");

        // Reset all BeingAggregated ticket states to Untouched
        ticket::Entity::update_many()
            .filter(ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .col_expr(
                ticket::Column::State,
                Expr::value(AcknowledgedTicketStatus::Untouched as u8),
            )
            .exec(&tickets_db)
            .await?;

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
            me_onchain,
            cfg,
        })
    }

    pub async fn create_pool(
        cfg: HoprNodeDbConfig,
        directory: PathBuf,
        mut options: PoolOptions<sqlx::Sqlite>,
        min_conn: Option<u32>,
        max_conn: Option<u32>,
        path: &str,
    ) -> crate::errors::Result<SqlitePool> {
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
            .pragma("busy_timeout", "1000") // 1000ms

        let pool = options
            .connect_with(sqlite_cfg.filename(directory.join(path)))
            .await
            .map_err(|e| DbSqlError::Construction(format!("failed to create {path} database: {e}")))?;

        Ok(pool)
    }


    // TODO: move this into separate trait

    async fn get_safe_info<'a>(&'a self, tx: OptTx<'a>) -> crate::errors::Result<Option<SafeInfo>> {
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

    async fn set_safe_info<'a>(&'a self, tx: OptTx<'a>, safe_info: SafeInfo) -> crate::errors::Result<()> {
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