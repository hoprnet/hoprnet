use async_trait::async_trait;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_query::{ColumnDef, Expr, Query, SqliteQueryBuilder, Table};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::errors::{NetworkingError, Result};
use crate::network::{PeerOrigin, PeerStatus};
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
            .table(NetworkPeersTable::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(NetworkPeersTable::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(NetworkPeersTable::PeerId)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(NetworkPeersTable::MultiAddresses).string().not_null())
            .col(ColumnDef::new(NetworkPeersTable::Origin).tiny_integer().not_null())
            .col(ColumnDef::new(NetworkPeersTable::PeerVersion).string_len(20))
            .col(ColumnDef::new(NetworkPeersTable::LastSeen).big_integer())
            .col(ColumnDef::new(NetworkPeersTable::LastSeenLatency).integer())
            .col(
                ColumnDef::new(NetworkPeersTable::Ignored)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(NetworkPeersTable::Quality).float())
            .col(ColumnDef::new(NetworkPeersTable::QualitySMA).binary())
            .col(ColumnDef::new(NetworkPeersTable::QualityEval).tiny_integer().default(0))
            .col(ColumnDef::new(NetworkPeersTable::Backoff).float())
            .col(ColumnDef::new(NetworkPeersTable::HeartbeatsSent).integer())
            .col(ColumnDef::new(NetworkPeersTable::HeartbeatsSuccessful).integer())
            .build(SqliteQueryBuilder);

        sqlx::query(&sql)
            .execute(&db)
            .await
            .expect("must be able to provision in-memory database");

        Self { db, cfg }
    }
}

#[derive(Debug, Clone, Copy, sea_query::Iden)]
pub enum NetworkPeersTable {
    Table,
    Id,
    PeerId,
    MultiAddresses,
    Origin,
    PeerVersion,
    LastSeen,
    LastSeenLatency,
    Ignored,
    Quality,
    QualitySMA,
    QualityEval,
    Backoff,
    HeartbeatsSent,
    HeartbeatsSuccessful,
}

#[async_trait]
impl NetworkBackend for SqliteNetworkBackend {
    async fn add(&self, peer: &PeerId, origin: PeerOrigin, mas: Vec<Multiaddr>) -> Result<()> {
        let (sql, values) = Query::insert()
            .into_table(NetworkPeersTable::Table)
            .columns([
                NetworkPeersTable::PeerId,
                NetworkPeersTable::MultiAddresses,
                NetworkPeersTable::Origin,
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
            .from_table(NetworkPeersTable::Table)
            .and_where(Expr::col(NetworkPeersTable::PeerId).eq(peer.to_base58()))
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        sqlx::query_with(&sql, values).execute(&self.db).await?;

        Ok(())
    }

    async fn update(&self, peer: &PeerId, new_status: &PeerStatus) -> Result<()> {
        let quality_sma = bincode::serialize(&Vec::from_iter((&new_status.quality_avg).into_iter()))
            .map_err(|e| NetworkingError::Other("cannot serialize sma".into()))?
            .into_boxed_slice();

        let (sql, values) = Query::update()
            .table(NetworkPeersTable::Table)
            .values([
                (NetworkPeersTable::PeerVersion, new_status.peer_version.clone().into()),
                (NetworkPeersTable::LastSeen, new_status.last_seen.into()),
                (NetworkPeersTable::LastSeenLatency, new_status.last_seen_latency.into()),
                (NetworkPeersTable::Quality, new_status.quality.into()),
                (NetworkPeersTable::QualitySMA, quality_sma.as_ref().into()),
                (NetworkPeersTable::Ignored, new_status.ignored.into()),
            ])
            .and_where(Expr::col(NetworkPeersTable::PeerId).eq(peer.to_base58()))
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        sqlx::query_with(&sql, values).execute(&self.db).await?;

        Ok(())
    }

    async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>> {
        todo!()
    }

    async fn get_multiple<F: FnOnce() -> T + Send + Sync, T: Send + Sync>(&self, filter: F) -> Result<Vec<T>> {
        todo!()
    }

    async fn stats(&self) -> Result<Stats> {
        todo!()
    }
}
