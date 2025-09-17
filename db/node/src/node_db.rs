use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use sea_orm::{EntityTrait, SqlxSqliteConnector, QueryFilter, ColumnTrait};
use sqlx::pool::PoolOptions;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use tracing::debug;
use tracing::log::LevelFilter;
use validator::Validate;
use hopr_crypto_types::keypairs::ChainKeypair;
use hopr_crypto_types::prelude::Keypair;
use hopr_primitive_types::primitives::Address;
use migration::{MigratorPeers, MigratorTickets, MigratorTrait};

use crate::cache::{NodeDbCaches};
use crate::errors::NodeDbError;
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
    pub(crate) caches: Arc<NodeDbCaches>,
    pub(crate) me_onchain: ChainKeypair,
    pub(crate) me_address: Address,
    pub(crate) cfg: HoprNodeDbConfig,
}

impl HoprNodeDb {
    pub async fn new(directory: &Path, chain_key: ChainKeypair, cfg: HoprNodeDbConfig) -> Result<Self, NodeDbError> {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&crate::protocol::METRIC_RECEIVED_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_SENT_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_TICKETS_COUNT);
        }

        cfg.validate()
            .map_err(|e| NodeDbError::Other(e.into()))?;

        fs::create_dir_all(directory)
            .map_err(|e| NodeDbError::Other(e.into()))?;

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
    pub async fn new_in_memory(chain_key: ChainKeypair) -> Result<Self, NodeDbError> {
        Self::new_sqlx_sqlite(
            chain_key,
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
        me_onchain: ChainKeypair,
        peers_db_pool: SqlitePool,
        tickets_db_pool: SqlitePool,
        cfg: HoprNodeDbConfig,
    ) -> Result<Self, NodeDbError> {
        let tickets_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(tickets_db_pool);
        MigratorTickets::up(&tickets_db, None)
            .await?;

        let peers_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(peers_db_pool);
        MigratorPeers::up(&peers_db, None)
            .await?;

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

        let caches = Arc::new(NodeDbCaches::default());
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
    ) -> Result<SqlitePool, NodeDbError> {
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
            .await?;

        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use hopr_api::{*, db::*};
    use hopr_crypto_types::keypairs::OffchainKeypair;
    use hopr_primitive_types::prelude::SingleSumSMA;
    use super::*;

    #[tokio::test]
    async fn test_basic_db_init() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;
        MigratorTickets::status(&db.tickets_db).await?;
        MigratorPeers::status(&db.peers_db).await?;

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
            let db = HoprNodeDb::new(path, ChainKeypair::random(), HoprNodeDbConfig::default()).await?;

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