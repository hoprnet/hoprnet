use std::{path::Path, sync::Arc, time::Duration};

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

use crate::{
    HoprDbAllOperations, accounts::model_to_account_entry, cache::HoprDbCaches, errors::Result,
    ticket_manager::TicketManager,
};

pub const HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS: u64 = 5 * 60; // 5 minutes

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
    pub(crate) index_db: sea_orm::DatabaseConnection,
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) peers_db: sea_orm::DatabaseConnection,
    pub(crate) logs_db: sea_orm::DatabaseConnection,
    pub(crate) ticket_manager: Arc<TicketManager>,
    pub(crate) chain_key: ChainKeypair,
    pub(crate) me_onchain: Address,
    pub(crate) caches: Arc<HoprDbCaches>,
}

pub const SQL_DB_INDEX_FILE_NAME: &str = "hopr_index.db";
pub const SQL_DB_PEERS_FILE_NAME: &str = "hopr_peers.db";
pub const SQL_DB_TICKETS_FILE_NAME: &str = "hopr_tickets.db";
pub const SQL_DB_LOGS_FILE_NAME: &str = "hopr_logs.db";

impl HoprDb {
    pub async fn new(directory: &Path, chain_key: ChainKeypair, cfg: HoprDbConfig) -> Result<Self> {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&crate::protocol::METRIC_RECEIVED_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_SENT_ACKS);
            lazy_static::initialize(&crate::protocol::METRIC_TICKETS_COUNT);
        }

        std::fs::create_dir_all(directory).map_err(|_e| {
            crate::errors::DbSqlError::Construction(format!("cannot create main database directory {directory:?}"))
        })?;

        // Default SQLite config values for all 3 DBs.
        // Each DB can customize with its own specific values
        let cfg_template = SqliteConnectOptions::default()
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

        // Indexer database
        let index = PoolOptions::new()
            .min_connections(0)
            .max_connections(1)
            .connect_with(cfg_template.clone().filename(directory.join(SQL_DB_INDEX_FILE_NAME)))
            .await
            .map_err(|e| crate::errors::DbSqlError::Construction(e.to_string()))?;

        // Peers database
        let peers = PoolOptions::new()
            .min_connections(0) // Default is 0
            .acquire_timeout(Duration::from_secs(60)) // Default is 30
            .idle_timeout(Some(Duration::from_secs(10 * 60))) // This is the default
            .max_lifetime(Some(Duration::from_secs(30 * 60))) // This is the default
            .max_connections(300) // Default is 10
            .connect_with(cfg_template.clone().filename(directory.join(SQL_DB_PEERS_FILE_NAME)))
            .await
            .map_err(|e| crate::errors::DbSqlError::Construction(e.to_string()))?;

        // Tickets database
        let tickets = PoolOptions::new()
            .min_connections(0)
            .max_connections(50)
            .connect_with(cfg_template.clone().filename(directory.join(SQL_DB_TICKETS_FILE_NAME)))
            .await
            .map_err(|e| crate::errors::DbSqlError::Construction(e.to_string()))?;

        let logs = PoolOptions::new()
            .min_connections(0)
            .connect_with(cfg_template.clone().filename(directory.join(SQL_DB_LOGS_FILE_NAME)))
            .await
            .unwrap_or_else(|e| panic!("failed to create logs database: {e}"));

        Self::new_sqlx_sqlite(chain_key, index, peers, tickets, logs).await
    }

    pub async fn new_in_memory(chain_key: ChainKeypair) -> Result<Self> {
        Self::new_sqlx_sqlite(
            chain_key,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| crate::errors::DbSqlError::Construction(e.to_string()))?,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| crate::errors::DbSqlError::Construction(e.to_string()))?,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| crate::errors::DbSqlError::Construction(e.to_string()))?,
            SqlitePool::connect(":memory:")
                .await
                .map_err(|e| crate::errors::DbSqlError::Construction(e.to_string()))?,
        )
        .await
    }

    async fn new_sqlx_sqlite(
        chain_key: ChainKeypair,
        index_db: SqlitePool,
        peers_db: SqlitePool,
        tickets_db: SqlitePool,
        logs_db: SqlitePool,
    ) -> Result<Self> {
        let index_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(index_db);

        MigratorIndex::up(&index_db, None)
            .await
            .map_err(|e| crate::errors::DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let tickets_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(tickets_db);

        MigratorTickets::up(&tickets_db, None)
            .await
            .map_err(|e| crate::errors::DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let peers_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(peers_db);

        MigratorPeers::up(&peers_db, None)
            .await
            .map_err(|e| crate::errors::DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

        let logs_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(logs_db);

        MigratorChainLogs::up(&logs_db, None)
            .await
            .map_err(|e| crate::errors::DbSqlError::Construction(format!("cannot apply database migration: {e}")))?;

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
            .map_err(|e| crate::errors::DbSqlError::Construction(format!("must reset peers on init: {e}")))?;
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
            .all(&index_db)
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
            index_db,
            peers_db,
            logs_db,
            ticket_manager: Arc::new(TicketManager::new(tickets_db.clone(), caches.clone())),
            tickets_db,
            caches,
        })
    }

    /// Starts ticket processing by the [TicketManager] with an optional new ticket notifier.
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
