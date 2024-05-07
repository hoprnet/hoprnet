use std::sync::Arc;

use crate::cache::HoprDbCaches;
use hopr_crypto_types::keypairs::Keypair;
use hopr_crypto_types::prelude::ChainKeypair;
use hopr_db_entity::ticket;
use hopr_internal_types::prelude::AcknowledgedTicketStatus;
use hopr_primitive_types::primitives::Address;
use migration::{MigratorIndex, MigratorPeers, MigratorTickets, MigratorTrait};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, SqlxSqliteConnector};
use sea_query::Expr;
use sqlx::pool::PoolOptions;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use std::path::Path;
use std::time::Duration;
use tracing::debug;
use tracing::log::LevelFilter;

use crate::ticket_manager::TicketManager;
use crate::HoprDbAllOperations;
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
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) peers_db: sea_orm::DatabaseConnection,
    pub(crate) ticket_manager: Arc<TicketManager>,
    pub(crate) chain_key: ChainKeypair, // TODO: remove this once chain keypairs are not needed to reconstruct tickets
    pub(crate) me_onchain: Address,
    pub(crate) caches: Arc<HoprDbCaches>,
}

pub const SQL_DB_INDEX_FILE_NAME: &str = "hopr_index.db";
pub const SQL_DB_PEERS_FILE_NAME: &str = "hopr_peers.db";
pub const SQL_DB_TICKETS_FILE_NAME: &str = "hopr_tickets.db";

impl HoprDb {
    pub async fn new(directory: String, chain_key: ChainKeypair, cfg: HoprDbConfig) -> Self {
        let dir = Path::new(&directory);
        std::fs::create_dir_all(dir).unwrap_or_else(|_| panic!("cannot create main database directory {directory}")); // hard-failure

        // Default SQLite config values for all 3 DBs.
        // Each DB can customize with its own specific values
        let cfg_template = SqliteConnectOptions::default()
            .create_if_missing(cfg.create_if_missing)
            .log_slow_statements(LevelFilter::Warn, cfg.log_slow_queries)
            .log_statements(LevelFilter::Debug)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .auto_vacuum(SqliteAutoVacuum::Full)
            .optimize_on_close(true, None)
            .page_size(4096)
            .pragma("cache_size", "-30000") // 32M
            .pragma("busy_timeout", "1000"); // 1000ms

        // Indexer database
        let index = PoolOptions::new()
            .min_connections(3)
            .max_connections(30)
            .connect_with(cfg_template.clone().filename(dir.join(SQL_DB_INDEX_FILE_NAME)))
            .await
            .unwrap_or_else(|e| panic!("failed to create main database: {e}"));

        // Peers database
        let peers = PoolOptions::new()
            .min_connections(10) // Default is 0
            .acquire_timeout(Duration::from_secs(60)) // Default is 30
            .idle_timeout(Some(Duration::from_secs(10 * 60))) // This is the default
            .max_lifetime(Some(Duration::from_secs(30 * 60))) // This is the default
            .max_connections(300) // Default is 10
            .connect_with(cfg_template.clone().filename(dir.join(SQL_DB_PEERS_FILE_NAME)))
            .await
            .unwrap_or_else(|e| panic!("failed to create main database: {e}"));

        // Tickets database
        let tickets = PoolOptions::new()
            .min_connections(5)
            .max_connections(50)
            .connect_with(cfg_template.clone().filename(dir.join(SQL_DB_TICKETS_FILE_NAME)))
            .await
            .unwrap_or_else(|e| panic!("failed to create main database: {e}"));

        Self::new_sqlx_sqlite(chain_key, index, peers, tickets).await
    }

    pub async fn new_in_memory(chain_key: ChainKeypair) -> Self {
        Self::new_sqlx_sqlite(
            chain_key,
            SqlitePool::connect(":memory:").await.unwrap(),
            SqlitePool::connect(":memory:").await.unwrap(),
            SqlitePool::connect(":memory:").await.unwrap(),
        )
        .await
    }

    async fn new_sqlx_sqlite(
        chain_key: ChainKeypair,
        index_db: SqlitePool,
        peers_db: SqlitePool,
        tickets_db: SqlitePool,
    ) -> Self {
        let index_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(index_db);

        MigratorIndex::up(&index_db, None)
            .await
            .expect("cannot apply database migration");

        let tickets_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(tickets_db);

        MigratorTickets::up(&tickets_db, None)
            .await
            .expect("cannot apply database migration");

        let peers_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(peers_db);

        MigratorPeers::up(&peers_db, None)
            .await
            .expect("cannot apply database migration");

        // Reset the peer network information
        let res = hopr_db_entity::network_peer::Entity::delete_many()
            .filter(sea_orm::Condition::all())
            .exec(&peers_db)
            .await
            .expect("must reset peers on init");
        debug!("Cleaned up {} rows from the 'peers' table", res.rows_affected);

        // Reset all BeingAggregated ticket states to Untouched
        ticket::Entity::update_many()
            .filter(ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .col_expr(
                ticket::Column::State,
                Expr::value(AcknowledgedTicketStatus::Untouched as u8),
            )
            .exec(&tickets_db)
            .await
            .expect("must reset ticket state on init");

        let caches = Arc::new(HoprDbCaches::default());
        caches.invalidate_all();

        Self {
            me_onchain: chain_key.public().to_address(),
            chain_key,
            db: index_db,
            peers_db,
            ticket_manager: Arc::new(TicketManager::new(tickets_db.clone(), caches.clone())),
            tickets_db,
            caches,
        }
    }
}

impl HoprDbAllOperations for HoprDb {}

#[cfg(test)]
mod tests {
    use crate::db::HoprDb;
    use crate::peers::{HoprDbPeersOperations, PeerOrigin};
    use crate::{HoprDbGeneralModelOperations, TargetDb};
    use hopr_crypto_types::keypairs::{ChainKeypair, OffchainKeypair};
    use hopr_crypto_types::prelude::Keypair;
    use libp2p_identity::PeerId;
    use migration::{MigratorIndex, MigratorPeers, MigratorTickets, MigratorTrait};
    use multiaddr::Multiaddr;
    use rand::{distributions::Alphanumeric, Rng}; // 0.8

    #[async_std::test]
    async fn test_basic_db_init() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        // TODO: cfg-if this on Postgres to do only `Migrator::status(db.conn(Default::default)).await.expect("status must be ok");`
        MigratorIndex::status(db.conn(TargetDb::Index))
            .await
            .expect("status must be ok");
        MigratorTickets::status(db.conn(TargetDb::Tickets))
            .await
            .expect("status must be ok");
        MigratorPeers::status(db.conn(TargetDb::Peers))
            .await
            .expect("status must be ok");
    }

    #[async_std::test]
    async fn test_peer_cleanup_on_database_start() {
        let random_filename: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();
        let random_tmp_file = format!("/tmp/{random_filename}.sqlite");

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse().unwrap();

        {
            let db = HoprDb::new(
                random_tmp_file.clone(),
                ChainKeypair::random(),
                crate::db::HoprDbConfig::default(),
            )
            .await;

            db.add_network_peer(
                &peer_id,
                PeerOrigin::IncomingConnection,
                vec![ma_1.clone(), ma_2.clone()],
                0.0,
                25,
            )
            .await
            .expect("should add peer");
        }
        {
            let db = HoprDb::new(
                random_tmp_file,
                ChainKeypair::random(),
                crate::db::HoprDbConfig::default(),
            )
            .await;

            let not_found_peer = db
                .get_network_peer(&peer_id)
                .await
                .expect("should not encounter a DB issue");

            assert_eq!(not_found_peer, None);
        }
    }
}
