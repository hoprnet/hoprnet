use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use hopr_primitive_types::prelude::SingleSumSMA;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_query::{ColumnDef, Expr, Func, Order, Query, SimpleExpr, SqliteQueryBuilder, Table};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::trace;

use crate::errors::{NetworkingError, Result};
use crate::network::{NetworkConfig, PeerGoodness, PeerOrigin, PeerStatus};
use crate::traits::{NetworkBackend, Stats};

#[derive(Clone, Debug, PartialEq)]
pub struct SqliteNetworkBackendConfig {
    pub network_options: NetworkConfig,
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
    version: Option<String>,
    last_seen: i64,
    last_seen_latency: i64,
    ignored: bool,
    public: bool,
    quality: f64,
    quality_sma: Box<[u8]>,
    goodness: u8,
    backoff: f64,
    heartbeats_sent: i64,
    heartbeats_successful: i64,
}

fn peer_status_from_row(row: NetworkPeers, quality_avg_window_size: u32) -> Result<PeerStatus> {
    let multiaddresses = row
        .multi_addresses
        .split(",")
        .map(Multiaddr::try_from)
        .collect::<core::result::Result<Vec<_>, multiaddr::Error>>()
        .map_err(|_| NetworkingError::DecodingError)?;

    let samples: Vec<f64> = bincode::deserialize(&row.quality_sma).map_err(|_| NetworkingError::DecodingError)?;

    Ok(PeerStatus {
        id: PeerId::from_str(&row.peer_id).map_err(|_| NetworkingError::DecodingError)?,
        origin: PeerOrigin::try_from(row.origin).map_err(|_| NetworkingError::DecodingError)?,
        is_public: row.public,
        last_seen: row.last_seen as u64,
        last_seen_latency: row.last_seen_latency as u64,
        heartbeats_sent: row.heartbeats_sent as u64,
        heartbeats_succeeded: row.heartbeats_successful as u64,
        backoff: row.backoff,
        ignored: row.ignored,
        peer_goodness: PeerGoodness::try_from(row.goodness).map_err(|_| NetworkingError::DecodingError)?,
        peer_version: row.version,
        multiaddresses,
        quality: row.quality,
        quality_avg: SingleSumSMA::new_with_samples(quality_avg_window_size, samples),
    })
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

        let maddrs = new_status
            .multiaddresses
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Override the goodness based on the quality value
        let goodness: u8 = if new_status.get_quality() <= self.cfg.network_options.quality_bad_threshold {
            PeerGoodness::Bad
        } else {
            PeerGoodness::Good
        }
        .into();

        let origin: u8 = new_status.origin.into();

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
                (NetworkPeersIden::Goodness, goodness.into()),
                (NetworkPeersIden::Backoff, new_status.backoff.into()),
                (NetworkPeersIden::HeartbeatsSent, new_status.heartbeats_sent.into()),
                (
                    NetworkPeersIden::HeartbeatsSuccessful,
                    new_status.heartbeats_succeeded.into(),
                ),
                (NetworkPeersIden::Origin, origin.into()),
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

        if let Some(peer) = sqlx::query_as_with(&sql, values).fetch_optional(&self.db).await? {
            Ok(Some(peer_status_from_row(
                peer,
                self.cfg.network_options.quality_avg_window_size,
            )?))
        } else {
            Ok(None)
        }
    }

    async fn get_multiple<'a>(&'a self, filter: Option<SimpleExpr>) -> Result<BoxStream<'a, PeerStatus>> {
        /*let qry = Query::select()
            .from(NetworkPeersIden::Table);

        if let Some(f) = filter {
            qry = qry.and_where(f);
        }

        let (sql, values) = qry.build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).fetch_many(&self.db)
            .map(|item| match item {
                Ok(Ei)
            })*/
        todo!()
    }

    async fn stats(&self) -> Result<Stats> {
        /*let (sql, values) = Query::select()
            .expr(Expr::col((NetworkPeersIden::Table, NetworkPeersIden::Id)).count())
            .from(NetworkPeersIden::Table)
            .and_where(Expr::col(NetworkPeersIden::Public).eq(true)
                .and(Expr::col(NetworkPeersIden::Ignored).eq(false))
                .and(Expr::col(Ne)))
            .build_sqlx(SqliteQueryBuilder);

        let count_public: u32 = sqlx::query_scalar_with(&sql, values).fetch_one(&self.db).await?;

        Err(NetworkingError::DecodingError)*/
        todo!()
    }
}
