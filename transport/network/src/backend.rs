use async_stream::stream;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::TryStreamExt;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_query::{ColumnDef, Expr, Query, SimpleExpr, SqliteQueryBuilder, Table};
use sea_query_binder::SqlxBinder;
use std::str::FromStr;
use tracing::{error, trace};

use crate::errors::{NetworkingError, Result};
use crate::network::{NetworkConfig, PeerOrigin, PeerStatus};
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
            .table(NetworkPeerIden::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(NetworkPeerIden::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(NetworkPeerIden::PeerId).string().not_null().unique_key())
            .col(ColumnDef::new(NetworkPeerIden::MultiAddresses).string().not_null())
            .col(ColumnDef::new(NetworkPeerIden::Origin).tiny_integer().not_null())
            .col(ColumnDef::new(NetworkPeerIden::Version).string_len(20))
            .col(ColumnDef::new(NetworkPeerIden::LastSeen).big_integer())
            .col(ColumnDef::new(NetworkPeerIden::LastSeenLatency).integer())
            .col(
                ColumnDef::new(NetworkPeerIden::Ignored)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(NetworkPeerIden::Public).boolean().default(true))
            .col(ColumnDef::new(NetworkPeerIden::Quality).float())
            .col(ColumnDef::new(NetworkPeerIden::QualitySma).binary())
            .col(ColumnDef::new(NetworkPeerIden::Backoff).float())
            .col(ColumnDef::new(NetworkPeerIden::HeartbeatsSent).integer())
            .col(ColumnDef::new(NetworkPeerIden::HeartbeatsSuccessful).integer())
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
struct NetworkPeer {
    #[allow(dead_code)]
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
    backoff: f64,
    heartbeats_sent: i64,
    heartbeats_successful: i64,
}

impl TryFrom<NetworkPeer> for PeerStatus {
    type Error = NetworkingError;

    fn try_from(value: NetworkPeer) -> std::result::Result<Self, Self::Error> {
        let multiaddresses = value
            .multi_addresses
            .split(",")
            .map(Multiaddr::try_from)
            .collect::<core::result::Result<Vec<_>, multiaddr::Error>>()
            .map_err(|_| NetworkingError::DecodingError)?;

        Ok(PeerStatus {
            id: PeerId::from_str(&value.peer_id).map_err(|_| NetworkingError::DecodingError)?,
            origin: PeerOrigin::try_from(value.origin).map_err(|_| NetworkingError::DecodingError)?,
            is_public: value.public,
            last_seen: value.last_seen as u64,
            last_seen_latency: value.last_seen_latency as u64,
            heartbeats_sent: value.heartbeats_sent as u64,
            heartbeats_succeeded: value.heartbeats_successful as u64,
            backoff: value.backoff,
            ignored: value.ignored,
            peer_version: value.version,
            multiaddresses,
            quality: value.quality,
            quality_avg: bincode::deserialize(&value.quality_sma).map_err(|_| NetworkingError::DecodingError)?,
        })
    }
}

#[async_trait]
impl NetworkBackend for SqliteNetworkBackend {
    async fn add(&self, peer: &PeerId, origin: PeerOrigin, mas: Vec<Multiaddr>) -> Result<()> {
        let (sql, values) = Query::insert()
            .into_table(NetworkPeerIden::Table)
            .columns([
                NetworkPeerIden::PeerId,
                NetworkPeerIden::MultiAddresses,
                NetworkPeerIden::Origin,
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
            .from_table(NetworkPeerIden::Table)
            .and_where(Expr::col(NetworkPeerIden::PeerId).eq(peer.to_base58()))
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        sqlx::query_with(&sql, values).execute(&self.db).await?;

        Ok(())
    }

    async fn update(&self, peer: &PeerId, new_status: &PeerStatus) -> Result<()> {
        let quality_sma = bincode::serialize(&new_status.quality_avg)
            .map_err(|e| NetworkingError::Other(format!("cannot serialize sma: {e}")))?
            .into_boxed_slice();

        let maddrs = new_status
            .multiaddresses
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let origin: u8 = new_status.origin.into();

        let (sql, values) = Query::update()
            .table(NetworkPeerIden::Table)
            .values([
                (NetworkPeerIden::Version, new_status.peer_version.clone().into()),
                (NetworkPeerIden::LastSeen, new_status.last_seen.into()),
                (NetworkPeerIden::LastSeenLatency, new_status.last_seen_latency.into()),
                (NetworkPeerIden::Quality, new_status.quality.into()),
                (NetworkPeerIden::QualitySma, quality_sma.as_ref().into()),
                (NetworkPeerIden::Ignored, new_status.ignored.into()),
                (NetworkPeerIden::Public, new_status.is_public.into()),
                (NetworkPeerIden::MultiAddresses, maddrs.into()),
                (NetworkPeerIden::Backoff, new_status.backoff.into()),
                (NetworkPeerIden::HeartbeatsSent, new_status.heartbeats_sent.into()),
                (
                    NetworkPeerIden::HeartbeatsSuccessful,
                    new_status.heartbeats_succeeded.into(),
                ),
                (NetworkPeerIden::Origin, origin.into()),
            ])
            .and_where(Expr::col(NetworkPeerIden::PeerId).eq(peer.to_base58()))
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        sqlx::query_with(&sql, values).execute(&self.db).await?;

        Ok(())
    }

    async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>> {
        let (sql, values) = Query::select()
            .from(NetworkPeerIden::Table)
            .and_where(Expr::col(NetworkPeerIden::PeerId).eq(peer.to_base58()))
            .limit(1)
            .build_sqlx(SqliteQueryBuilder);

        let peers: Option<NetworkPeer> = sqlx::query_as_with(&sql, values).fetch_optional(&self.db).await?;

        if let Some(peer) = peers {
            Ok(Some(peer.try_into()?))
        } else {
            Ok(None)
        }
    }

    async fn get_multiple<'a>(&'a self, filter: Option<SimpleExpr>) -> Result<BoxStream<'a, PeerStatus>> {
        let (sql, values) = Query::select()
            .from(NetworkPeerIden::Table)
            .and_where(filter.unwrap_or(Expr::value(1)))
            .build_sqlx(SqliteQueryBuilder);

        Ok(Box::pin(stream! {
            let mut sub_stream = sqlx::query_as_with::<_, NetworkPeer, _>(&sql, values.clone())
                .fetch(&self.db);

            loop {
                match sub_stream.try_next().await {
                    Ok(Some(peer_row)) => {
                        trace!("got db network row: {peer_row:?}");
                        match PeerStatus::try_from(peer_row) {
                            Ok(peer_status) => yield peer_status,
                            Err(e) => error!("cannot map peer from row: {e}"),
                        }
                    },
                    Ok(None) => {
                        trace!("fetched all network results");
                        break;
                    }
                    Err(e) => {
                        error!("failed to retrieve next network row: {e}");
                        break;
                    }
                }
            }
        }))
    }

    // TODO: does it make sense to use rather a separate table?
    async fn stats(&self) -> Result<Stats> {
        let (sql, values) = Query::select()
            .expr(Expr::col((NetworkPeerIden::Table, NetworkPeerIden::Id)).count())
            .from(NetworkPeerIden::Table)
            .and_where(
                Expr::col(NetworkPeerIden::Public)
                    .eq(true)
                    .and(Expr::col(NetworkPeerIden::Ignored).eq(false))
                    .and(Expr::col(NetworkPeerIden::Quality).gt(self.cfg.network_options.quality_bad_threshold)),
            )
            .build_sqlx(SqliteQueryBuilder);

        let good_quality_public: u32 = sqlx::query_scalar_with(&sql, values).fetch_one(&self.db).await?;

        let (sql, values) = Query::select()
            .expr(Expr::col((NetworkPeerIden::Table, NetworkPeerIden::Id)).count())
            .from(NetworkPeerIden::Table)
            .and_where(
                Expr::col(NetworkPeerIden::Public)
                    .eq(true)
                    .and(Expr::col(NetworkPeerIden::Ignored).eq(false))
                    .and(Expr::col(NetworkPeerIden::Quality).lte(self.cfg.network_options.quality_bad_threshold)),
            )
            .build_sqlx(SqliteQueryBuilder);

        let bad_quality_public: u32 = sqlx::query_scalar_with(&sql, values).fetch_one(&self.db).await?;

        let (sql, values) = Query::select()
            .expr(Expr::col((NetworkPeerIden::Table, NetworkPeerIden::Id)).count())
            .from(NetworkPeerIden::Table)
            .and_where(
                Expr::col(NetworkPeerIden::Public)
                    .eq(false)
                    .and(Expr::col(NetworkPeerIden::Ignored).eq(false))
                    .and(Expr::col(NetworkPeerIden::Quality).gt(self.cfg.network_options.quality_bad_threshold)),
            )
            .build_sqlx(SqliteQueryBuilder);

        let good_quality_non_public: u32 = sqlx::query_scalar_with(&sql, values).fetch_one(&self.db).await?;

        let (sql, values) = Query::select()
            .expr(Expr::col((NetworkPeerIden::Table, NetworkPeerIden::Id)).count())
            .from(NetworkPeerIden::Table)
            .and_where(
                Expr::col(NetworkPeerIden::Public)
                    .eq(false)
                    .and(Expr::col(NetworkPeerIden::Ignored).eq(false))
                    .and(Expr::col(NetworkPeerIden::Quality).lte(self.cfg.network_options.quality_bad_threshold)),
            )
            .build_sqlx(SqliteQueryBuilder);

        let bad_quality_non_public: u32 = sqlx::query_scalar_with(&sql, values).fetch_one(&self.db).await?;

        Ok(Stats {
            good_quality_public,
            good_quality_non_public,
            bad_quality_public,
            bad_quality_non_public,
        })
    }
}

#[cfg(test)]
mod tests {
    // TODO: missing tests
}
