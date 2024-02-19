use async_trait::async_trait;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_query::{ColumnDef, Expr, Order, Query, SqliteQueryBuilder, Table};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::errors::{NetworkingError, Result};
use crate::network::{PeerGoodness, PeerOrigin, PeerStatus};
use crate::traits::{NetworkBackend, Stats};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SqliteNetworkBackendConfig {
    pub peer_quality_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct SqliteNetworkBackend {
    db: sqlx::SqlitePool,
    cfg: SqliteNetworkBackendConfig,
}

impl SqliteNetworkBackend {
    pub async fn new(cfg: SqliteNetworkBackendConfig) -> Self {
        let db = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("memory driver must be always constructible");

        let sql = Table::create()
            .table(NetworkPeersIden::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(NetworkPeersIden::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(NetworkPeersIden::PeerId)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(NetworkPeersIden::MultiAddresses).string().not_null())
            .col(ColumnDef::new(NetworkPeersIden::Origin).tiny_integer().not_null())
            .col(ColumnDef::new(NetworkPeersIden::Version).string_len(20))
            .col(ColumnDef::new(NetworkPeersIden::LastSeen).big_integer())
            .col(ColumnDef::new(NetworkPeersIden::LastSeenLatency).integer())
            .col(
                ColumnDef::new(NetworkPeersIden::Ignored)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(NetworkPeersIden::Public).boolean().default(true))
            .col(ColumnDef::new(NetworkPeersIden::Quality).float())
            .col(ColumnDef::new(NetworkPeersIden::QualitySma).binary())
            .col(ColumnDef::new(NetworkPeersIden::Goodness).tiny_integer().default(0))
            .col(ColumnDef::new(NetworkPeersIden::Backoff).float())
            .col(ColumnDef::new(NetworkPeersIden::HeartbeatsSent).integer())
            .col(ColumnDef::new(NetworkPeersIden::HeartbeatsSuccessful).integer())
            .build(SqliteQueryBuilder);

        sqlx::query(&sql)
            .execute(&db)
            .await
            .expect("must be able to provision in-memory database");

        Self { db, cfg }
    }
}
#[sea_query::enum_def]
#[derive(Debug, Clone, sqlx::FromRow)]
struct NetworkPeers {
    id: i32,
    peer_id: String,
    multi_addresses: String,
    origin: u8,
    version: String,
    last_seen: i64,
    last_seen_latency: f64,
    ignored: bool,
    public: bool,
    quality: f64,
    quality_sma: Box<[u8]>,
    goodness: u8,
    backoff: f64,
    heartbeats_sent: i64,
    heartbeats_successful: i64
}

impl From<NetworkPeers> for PeerStatus {
    fn from(value: NetworkPeers) -> Self {
        todo!()
    }
}

#[async_trait]
impl NetworkBackend for SqliteNetworkBackend {
    async fn add(&self, peer: &PeerId, origin: PeerOrigin, mas: Vec<Multiaddr>) -> Result<()> {

        let (sql, values) = Query::insert()
            .into_table(NetworkPeersIden::Table)
            .columns([
                NetworkPeersIden::PeerId,
                NetworkPeersIden::MultiAddresses,
                NetworkPeersIden::Origin,
            ])
            .values_panic([
                peer.to_base58().into(),
                mas.iter().map(|ma| ma.to_string()).collect::<Vec<_>>().join(",").into(),
                (origin as u8).into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        sqlx::query_with(&sql, values).execute(&self.db).await?;

        Ok(())
    }

    async fn remove(&self, peer: &PeerId) -> Result<()> {
        let (sql, values) = Query::delete()
            .from_table(NetworkPeersIden::Table)
            .and_where(Expr::col(NetworkPeersIden::PeerId).eq(peer.to_base58()))
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        sqlx::query_with(&sql, values).execute(&self.db).await?;

        Ok(())
    }

    async fn update(&self, peer: &PeerId, new_status: &PeerStatus) -> Result<()> {
        let quality_sma = bincode::serialize(&Vec::from_iter((&new_status.quality_avg).into_iter()))
            .map_err(|e| NetworkingError::Other("cannot serialize sma".into()))?
            .into_boxed_slice();

        let maddrs = new_status.multiaddresses.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",");

        // Override the goodness based on the quality value
        let goodness = if new_status.get_quality() <= self.cfg.peer_quality_threshold {
            PeerGoodness::Bad
        } else {
            PeerGoodness::Good
        };

        let (sql, values) = Query::update()
            .table(NetworkPeersIden::Table)
            .values([
                (NetworkPeersIden::Version, new_status.peer_version.clone().into()),
                (NetworkPeersIden::LastSeen, new_status.last_seen.into()),
                (NetworkPeersIden::LastSeenLatency, new_status.last_seen_latency.into()),
                (NetworkPeersIden::Quality, new_status.quality.into()),
                (NetworkPeersIden::QualitySma, quality_sma.as_ref().into()),
                (NetworkPeersIden::Ignored, new_status.ignored.into()),
                (NetworkPeersIden::Public, new_status.is_public.into()),
                (NetworkPeersIden::MultiAddresses, maddrs.into()),
                (NetworkPeersIden::Goodness, (new_status.peer_goodness as u8).into()),
                (NetworkPeersIden::Backoff, new_status.backoff.into()),
                (NetworkPeersIden::HeartbeatsSent, new_status.heartbeats_sent.into()),
                (NetworkPeersIden::HeartbeatsSuccessful, new_status.heartbeats_succeeded.into()),
                (NetworkPeersIden::Origin, (new_status.origin as u8).into())
            ])
            .and_where(Expr::col(NetworkPeersIden::PeerId).eq(peer.to_base58()))
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        sqlx::query_with(&sql, values).execute(&self.db).await?;

        Ok(())
    }

    async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>> {
        let (sql, values) = Query::select()
            .from(NetworkPeersIden::Table)
            .and_where(Expr::col(NetworkPeersIden::PeerId).eq(peer.to_base58()))
            .limit(1)
            .build_sqlx(SqliteQueryBuilder);

        let peer: Option<NetworkPeers> = sqlx::query_as_with(&sql, values).fetch_optional(&self.db).await?;
        Ok(peer.map(|p| p.into()))
    }

    async fn get_multiple<F: FnOnce() -> T + Send + Sync, T: Send + Sync>(&self, filter: F) -> Result<Vec<T>> {
        todo!()
    }

    async fn stats(&self) -> Result<Stats> {
        todo!()
    }
}
