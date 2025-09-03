use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use futures::channel::mpsc::UnboundedSender;
use hopr_crypto_types::{keypairs::Keypair, prelude::ChainKeypair};
use hopr_db_entity::{
    prelude::{Account, Announcement},
    ticket,
};
use hopr_internal_types::prelude::{AcknowledgedTicket, AcknowledgedTicketStatus};
use hopr_primitive_types::primitives::Address;
use migration::{MigratorChainLogs, MigratorIndex, MigratorPeers, MigratorTickets, MigratorTrait};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, SqlxSqliteConnector};
use sea_query::Expr;
use sqlx::{
    ConnectOptions, SqlitePool,
    pool::PoolOptions,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};
use tracing::{debug, log::LevelFilter};
use validator::Validate;

use crate::{
    HoprDbAllOperations,
    accounts::model_to_account_entry,
    cache::HoprDbCaches,
    errors::{DbSqlError, Result},
    ticket_manager::TicketManager,
};

pub const HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS: u64 = 5 * 60; // 5 minutes

pub const MIN_SURB_RING_BUFFER_SIZE: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
pub struct HoprDbConfig {
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
pub struct HoprDb {
    pub(crate) index_db: DbConnection,
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) peers_db: sea_orm::DatabaseConnection,
    pub(crate) logs_db: sea_orm::DatabaseConnection,

    pub(crate) caches: Arc<HoprDbCaches>,
    pub(crate) chain_key: ChainKeypair,
    pub(crate) me_onchain: Address,
    pub(crate) ticket_manager: Arc<TicketManager>,
    pub(crate) cfg: HoprDbConfig,
}

/// Filename for the blockchain-indexing database.
pub const SQL_DB_INDEX_FILE_NAME: &str = "hopr_index.db";
/// Filename for the network peers database.
pub const SQL_DB_PEERS_FILE_NAME: &str = "hopr_peers.db";
/// Filename for the payment tickets database.
pub const SQL_DB_TICKETS_FILE_NAME: &str = "hopr_tickets.db";
/// Filename for the blockchain logs database (used in snapshots).
pub const SQL_DB_LOGS_FILE_NAME: &str = "hopr_logs.db";

impl HoprDb {
    pub async fn new(directory: &Path, chain_key: ChainKeypair, cfg: HoprDbConfig) -> Result<Self> {
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

        #[cfg(feature = "sqlite")]
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
            false,
            SQL_DB_PEERS_FILE_NAME,
        )
        .await?;

        let tickets = Self::create_pool(
            cfg.clone(),
            directory.to_path_buf(),
            PoolOptions::new(),
            Some(0),
            Some(50),
            false,
            SQL_DB_TICKETS_FILE_NAME,
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
        Self::new_sqlx_sqlite(chain_key, index, index_ro, peers, tickets, logs, cfg).await
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
        chain_key: ChainKeypair,
        index_db_pool: SqlitePool,
        index_db_ro_pool: SqlitePool,
        peers_db_pool: SqlitePool,
        tickets_db_pool: SqlitePool,
        logs_db_pool: SqlitePool,
        cfg: HoprDbConfig,
    ) -> Result<Self> {
        let index_db_rw = SqlxSqliteConnector::from_sqlx_sqlite_pool(index_db_pool);
        let index_db_ro = SqlxSqliteConnector::from_sqlx_sqlite_pool(index_db_ro_pool);

        MigratorIndex::up(&index_db_rw, None)
            .await
            .map_err(|e| DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let tickets_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(tickets_db_pool);

        MigratorTickets::up(&tickets_db, None)
            .await
            .map_err(|e| DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let peers_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(peers_db_pool);

        MigratorPeers::up(&peers_db, None)
            .await
            .map_err(|e| DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let logs_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(logs_db_pool.clone());

        MigratorChainLogs::up(&logs_db, None)
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

        // Initialize KeyId mapping for accounts
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
            chain_key,
            ticket_manager: Arc::new(TicketManager::new(tickets_db.clone(), caches.clone())),
            caches,

            index_db: DbConnection {
                ro: index_db_ro,
                rw: index_db_rw,
            },
            tickets_db,
            peers_db,
            logs_db,
            cfg,
        })
    }

    /// Starts ticket processing by the `TicketManager` with an optional new ticket notifier.
    /// Without calling this method, tickets will not be persisted into the DB.
    ///
    /// If the notifier is given, it will receive notifications once new ticket has been
    /// persisted into the Tickets DB.
    pub fn start_ticket_processing(&self, ticket_notifier: Option<UnboundedSender<AcknowledgedTicket>>) -> Result<()> {
        if let Some(notifier) = ticket_notifier {
            self.ticket_manager.start_ticket_processing(notifier)
        } else {
            self.ticket_manager.start_ticket_processing(futures::sink::drain())
        }
    }

    /// Default SQLite config values for all DBs with RW  (read-write) access.
    fn common_connection_cfg_rw(cfg: HoprDbConfig) -> SqliteConnectOptions {
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
    fn common_connection_cfg_ro(cfg: HoprDbConfig) -> SqliteConnectOptions {
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
        cfg: HoprDbConfig,
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
}

impl HoprDbAllOperations for HoprDb {}

#[cfg(test)]
mod tests {
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, OffchainKeypair},
        prelude::Keypair,
    };
    use hopr_db_api::peers::{HoprDbPeersOperations, PeerOrigin};
    use hopr_primitive_types::sma::SingleSumSMA;
    use libp2p_identity::PeerId;
    use migration::{MigratorChainLogs, MigratorIndex, MigratorPeers, MigratorTickets, MigratorTrait};
    use multiaddr::Multiaddr;
    use rand::{Rng, distributions::Alphanumeric};

    use crate::{HoprDbGeneralModelOperations, TargetDb, db::HoprDb}; // 0.8

    #[tokio::test]
    async fn test_basic_db_init() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        // NOTE: cfg-if this on Postgres to do only `Migrator::status(db.conn(Default::default)).await.expect("status
        // must be ok");`
        MigratorIndex::status(db.conn(TargetDb::Index)).await?;
        MigratorTickets::status(db.conn(TargetDb::Tickets)).await?;
        MigratorPeers::status(db.conn(TargetDb::Peers)).await?;
        MigratorChainLogs::status(db.conn(TargetDb::Logs)).await?;

        Ok(())
    }

    #[tokio::test]
    async fn peers_without_any_recent_updates_should_be_discarded_on_restarts() -> anyhow::Result<()> {
        let random_filename: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();
        let random_tmp_file = format!("/tmp/{random_filename}.sqlite");

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse()?;
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse()?;

        let path = std::path::Path::new(&random_tmp_file);

        {
            let db = HoprDb::new(path, ChainKeypair::random(), crate::db::HoprDbConfig::default()).await?;

            db.add_network_peer(
                &peer_id,
                PeerOrigin::IncomingConnection,
                vec![ma_1.clone(), ma_2.clone()],
                0.0,
                25,
            )
            .await?;
        }

        {
            let db = HoprDb::new(path, ChainKeypair::random(), crate::db::HoprDbConfig::default()).await?;

            let not_found_peer = db.get_network_peer(&peer_id).await?;

            assert_eq!(not_found_peer, None);
        }

        Ok(())
    }

    #[tokio::test]
    async fn peers_with_a_recent_update_should_be_retained_in_the_database() -> anyhow::Result<()> {
        let random_filename: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();
        let random_tmp_file = format!("/tmp/{random_filename}.sqlite");

        let ofk = OffchainKeypair::random();
        let peer_id: PeerId = (*ofk.public()).into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse()?;
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse()?;

        let path = std::path::Path::new(&random_tmp_file);

        {
            let db = HoprDb::new(path, ChainKeypair::random(), crate::db::HoprDbConfig::default()).await?;

            db.add_network_peer(
                &peer_id,
                PeerOrigin::IncomingConnection,
                vec![ma_1.clone(), ma_2.clone()],
                0.0,
                25,
            )
            .await?;

            let ten_seconds_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(10);

            db.update_network_peer(hopr_db_api::peers::PeerStatus {
                id: (*ofk.public(), peer_id),
                origin: PeerOrigin::Initialization,
                last_seen: ten_seconds_ago,
                last_seen_latency: std::time::Duration::from_millis(10),
                heartbeats_sent: 1,
                heartbeats_succeeded: 1,
                backoff: 1.0,
                ignored_until: None,
                multiaddresses: vec![ma_1.clone(), ma_2.clone()],
                quality: 1.0,
                quality_avg: SingleSumSMA::new(2),
            })
            .await?;
        }
        {
            let db = HoprDb::new(path, ChainKeypair::random(), crate::db::HoprDbConfig::default()).await?;

            let found_peer = db.get_network_peer(&peer_id).await?.map(|p| p.id.1);

            assert_eq!(found_peer, Some(peer_id));
        }

        Ok(())
    }
}
