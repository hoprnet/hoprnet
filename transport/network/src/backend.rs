use async_trait::async_trait;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_query::{ColumnDef, Query, SqliteQueryBuilder, Table};
use sea_query::ConditionExpression::SimpleExpr;
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};

use crate::network::{PeerOrigin, PeerStatus};
use crate::traits::{NetworkBackend, Stats};
use crate::errors::Result;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SqliteNetworkBackendConfig {
    pub peer_quality_threshold: f64,
}

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
            .col(ColumnDef::new(NetworkPeersTable::PeerId).string().not_null().unique_key())
            .col(ColumnDef::new(NetworkPeersTable::MultiAddresses).string().not_null())
            .col(ColumnDef::new(NetworkPeersTable::Origin).tiny_integer().not_null())
            .col(ColumnDef::new(NetworkPeersTable::PeerVersion).string_len(20))
            .col(ColumnDef::new(NetworkPeersTable::LastSeenLatency).timestamp())
            .col(ColumnDef::new(NetworkPeersTable::LastSeenLatency).integer())
            .col(ColumnDef::new(NetworkPeersTable::Ignored).boolean().not_null().default(false))
            .col(ColumnDef::new(NetworkPeersTable::Quality).float())
            .col(ColumnDef::new(NetworkPeersTable::QualitySMA).binary())
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

#[derive(sea_query::Iden)]
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
                NetworkPeersTable::Origin
            ])
            .values_panic([
                peer.to_base58().into(),
                mas.iter().map(|ma| ma.to_string()).collect::<Vec<_>>().join(",").into(),
                (origin as u8).into()
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.db)
            .await?;

        Ok(())
    }

    async fn remove(&self, peer: &PeerId) -> Result<()> {
        todo!()
    }

    async fn update(&self, peer: &PeerId, new_status: &PeerStatus) -> Result<()> {
        todo!()
    }

    async fn get(&self, peer: &PeerId) -> Result<PeerStatus> {
        todo!()
    }

    async fn get_multiple<F: FnOnce() -> T + Send + Sync, T: Send + Sync>(&self, filter: F) -> Result<Vec<T>> {
        todo!()
    }

    async fn stats(&self) -> Result<Stats> {
        todo!()
    }
}
