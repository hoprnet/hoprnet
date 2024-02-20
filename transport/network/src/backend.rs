use async_stream::stream;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::TryStreamExt;
use hopr_primitive_types::prelude::SingleSumSMA;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_query::{Asterisk, ColumnDef, Expr, Order, Query, SimpleExpr, SqliteQueryBuilder, Table};
use sea_query_binder::SqlxBinder;
use std::str::FromStr;
use tracing::{error, trace};

use crate::errors::{NetworkingError, Result};
use crate::network::{NetworkConfig, PeerOrigin, PeerStatus};
use crate::traits::{NetworkBackend, Stats};

/// Configuration object for the Sqlite backend.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SqliteNetworkBackendConfig {
    /// Configuration object of the parent [crate::network::Network].
    pub network_options: NetworkConfig,
}

/// Implementation of the [NetworkBackend] trait that uses in-memory Sqlite database.
#[derive(Debug, Clone)]
pub struct SqliteNetworkBackend {
    db: sqlx::SqlitePool,
    cfg: SqliteNetworkBackendConfig,
}

impl SqliteNetworkBackend {
    /// Construct new backend with in-memory database.
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
            .col(ColumnDef::new(NetworkPeerIden::LastSeen).big_integer().default(0))
            .col(ColumnDef::new(NetworkPeerIden::LastSeenLatency).integer().default(0))
            .col(
                ColumnDef::new(NetworkPeerIden::Ignored)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(NetworkPeerIden::Public).boolean().default(true))
            .col(ColumnDef::new(NetworkPeerIden::Quality).float().default(0.0))
            .col(ColumnDef::new(NetworkPeerIden::QualitySma).binary())
            .col(ColumnDef::new(NetworkPeerIden::Backoff).float().default(0.0))
            .col(ColumnDef::new(NetworkPeerIden::HeartbeatsSent).integer().default(0))
            .col(
                ColumnDef::new(NetworkPeerIden::HeartbeatsSuccessful)
                    .integer()
                    .default(0),
            )
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
            .split(',')
            .map(Multiaddr::try_from)
            .collect::<core::result::Result<Vec<_>, multiaddr::Error>>()
            .map_err(|_| NetworkingError::DecodingError)?;

        let quality_avg = if !value.quality_sma.is_empty() {
            bincode::deserialize(&value.quality_sma).map_err(|_| NetworkingError::DecodingError)?
        } else {
            SingleSumSMA::new(25) // just give some default, will be overriden on `update`
        };

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
            quality_avg,
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
        let res = sqlx::query_with(&sql, values).execute(&self.db).await?;
        if res.rows_affected() > 0 {
            Ok(())
        } else {
            Err(NetworkingError::NonExistingPeer)
        }
    }

    async fn update(&self, new_status: &PeerStatus) -> Result<()> {
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
            .and_where(Expr::col(NetworkPeerIden::PeerId).eq(new_status.id.to_base58()))
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);
        let res = sqlx::query_with(&sql, values).execute(&self.db).await?;

        if res.rows_affected() > 0 {
            Ok(())
        } else {
            Err(NetworkingError::NonExistingPeer)
        }
    }

    async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>> {
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(NetworkPeerIden::Table)
            .and_where(Expr::col(NetworkPeerIden::PeerId).eq(peer.to_base58()))
            .limit(1)
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);

        let peers: Option<NetworkPeer> = sqlx::query_as_with(&sql, values).fetch_optional(&self.db).await?;

        if let Some(peer) = peers {
            Ok(Some(peer.try_into()?))
        } else {
            Ok(None)
        }
    }

    async fn get_multiple<'a>(
        &'a self,
        filter: Option<SimpleExpr>,
        sort_last_seen_asc: bool,
    ) -> Result<BoxStream<'a, PeerStatus>> {
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(NetworkPeerIden::Table)
            .and_where(filter.unwrap_or(Expr::value(1)))
            .order_by(
                NetworkPeerIden::LastSeen,
                if sort_last_seen_asc { Order::Asc } else { Order::Desc },
            )
            .build_sqlx(SqliteQueryBuilder);

        trace!("about to execute network sql {query}", query = sql);

        Ok(Box::pin(stream! {
            let mut sub_stream = sqlx::query_as_with::<_, NetworkPeer, _>(&sql, values)
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

        trace!("about to execute network sql {query}", query = sql);
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

        trace!("about to execute network sql {query}", query = sql);
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

        trace!("about to execute network sql {query}", query = sql);
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

        trace!("about to execute network sql {query}", query = sql);
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
    use crate::backend::{NetworkPeerIden, SqliteNetworkBackend, SqliteNetworkBackendConfig};
    use crate::network::{PeerOrigin, PeerStatus};
    use crate::traits::{NetworkBackend, Stats};
    use futures_lite::StreamExt;
    use hopr_primitive_types::sma::SingleSumSMA;
    use libp2p_identity::PeerId;
    use multiaddr::Multiaddr;
    use sea_query::Expr;

    #[async_std::test]
    async fn test_add_get() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id = PeerId::random();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse().unwrap();

        db.add(
            &peer_id,
            PeerOrigin::IncomingConnection,
            vec![ma_1.clone(), ma_2.clone()],
        )
        .await
        .expect("should add peer");

        let peer_from_db = db
            .get(&peer_id)
            .await
            .expect("should be able to get a peer")
            .expect("peer must exist in the db");

        let expected_peer = PeerStatus {
            id: peer_id,
            origin: PeerOrigin::IncomingConnection,
            is_public: true,
            last_seen: 0,
            last_seen_latency: 0,
            heartbeats_sent: 0,
            heartbeats_succeeded: 0,
            backoff: 0.0,
            ignored: false,
            peer_version: None,
            multiaddresses: vec![ma_1, ma_2],
            quality: 0.0,
            quality_avg: SingleSumSMA::new(25),
        };

        assert_eq!(expected_peer, peer_from_db, "peer states must match");
    }

    #[async_std::test]
    async fn test_should_remove_peer() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id = PeerId::random();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();

        db.add(&peer_id, PeerOrigin::IncomingConnection, vec![ma_1.clone()])
            .await
            .expect("should add peer");
        assert!(
            db.get(&peer_id).await.expect("should get peer").is_some(),
            "must have peer entry"
        );

        db.remove(&peer_id).await.expect("must remove peer");
        assert!(
            db.get(&peer_id).await.expect("should get peer").is_none(),
            "peer entry must be gone"
        );
    }

    #[async_std::test]
    async fn test_should_not_remove_non_existing_peer() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id = PeerId::random();

        db.remove(&peer_id)
            .await
            .expect_err("must not delete non-existent peer");
    }

    #[async_std::test]
    async fn test_should_not_add_duplicate_peers() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id = PeerId::random();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();

        db.add(&peer_id, PeerOrigin::IncomingConnection, vec![ma_1.clone()])
            .await
            .expect("should add peer");
        db.add(&peer_id, PeerOrigin::IncomingConnection, vec![])
            .await
            .expect_err("should fail adding second time");
    }

    #[async_std::test]
    async fn test_should_return_none_on_non_existing_peer() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id = PeerId::random();

        assert!(
            db.get(&peer_id).await.expect("should succeed").is_none(),
            "should return none"
        );
    }

    #[async_std::test]
    async fn test_update() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id = PeerId::random();

        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse().unwrap();

        db.add(
            &peer_id,
            PeerOrigin::IncomingConnection,
            vec![ma_1.clone(), ma_2.clone()],
        )
        .await
        .expect("should add peer");

        let peer_status = PeerStatus {
            id: peer_id,
            origin: PeerOrigin::IncomingConnection,
            is_public: true,
            last_seen: 1,
            last_seen_latency: 2,
            heartbeats_sent: 3,
            heartbeats_succeeded: 4,
            backoff: 1.0,
            ignored: false,
            peer_version: Some("1.2.3".into()),
            multiaddresses: vec![ma_1],
            quality: 0.6,
            quality_avg: SingleSumSMA::new_with_samples(3, vec![0.1_f64, 0.4_64, 0.6_f64]),
        };

        let peer_status_from_db = db
            .get(&peer_id)
            .await
            .expect("get should succeed")
            .expect("entry should exist");

        assert_ne!(peer_status, peer_status_from_db, "entries must not be equal");

        db.update(&peer_status).await.expect("update should succeed");

        let peer_status_from_db = db
            .get(&peer_id)
            .await
            .expect("get should succeed")
            .expect("entry should exist");

        assert_eq!(peer_status, peer_status_from_db, "entries must be equal");
    }

    #[async_std::test]
    async fn test_should_fail_to_update_non_existing_peer() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id = PeerId::random();

        let peer_status = PeerStatus {
            id: peer_id,
            origin: PeerOrigin::IncomingConnection,
            is_public: true,
            last_seen: 1,
            last_seen_latency: 2,
            heartbeats_sent: 3,
            heartbeats_succeeded: 4,
            backoff: 1.0,
            ignored: false,
            peer_version: Some("1.2.3".into()),
            multiaddresses: vec![],
            quality: 0.6,
            quality_avg: SingleSumSMA::new_with_samples(3, vec![0.1_f64, 0.4_64, 0.6_f64]),
        };

        db.update(&peer_status)
            .await
            .expect_err("should fail updating non-existing peer");
    }

    #[async_std::test]
    async fn test_get_multiple_should_return_all_peers() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peers = (0..10).map(|_| PeerId::random()).collect::<Vec<_>>();

        for peer in &peers {
            db.add(peer, PeerOrigin::Initialization, vec![])
                .await
                .expect("should not fail adding peers");
        }

        let peers_from_db: Vec<PeerId> = db
            .get_multiple(None, false)
            .await
            .expect("should get stream")
            .map(|s| s.id)
            .collect()
            .await;

        assert_eq!(peers.len(), peers_from_db.len(), "lengths must match");
        assert_eq!(peers, peers_from_db, "peer ids must match");
    }

    #[async_std::test]
    async fn test_get_multiple_should_return_filtered_peers() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_count = 10;
        let peers = (0..peer_count).map(|_| PeerId::random()).collect::<Vec<_>>();

        for (i, peer) in peers.iter().enumerate() {
            db.add(peer, PeerOrigin::Initialization, vec![])
                .await
                .expect("should not fail adding peers");
            if i >= peer_count / 2 {
                let peer_status = PeerStatus {
                    id: peer.clone(),
                    origin: PeerOrigin::IncomingConnection,
                    is_public: true,
                    last_seen: i as u64,
                    last_seen_latency: 2,
                    heartbeats_sent: 3,
                    heartbeats_succeeded: 4,
                    backoff: 1.0,
                    ignored: false,
                    peer_version: Some("1.2.3".into()),
                    multiaddresses: vec![],
                    quality: 0.6,
                    quality_avg: SingleSumSMA::new_with_samples(3, vec![0.1_f64, 0.4_64, 0.6_f64]),
                };
                db.update(&peer_status).await.expect("must update peer status");
            }
        }

        let peers_from_db: Vec<PeerId> = db
            .get_multiple(Some(Expr::col(NetworkPeerIden::Quality).gt(0.5_f64)), false)
            .await
            .expect("should get stream")
            .map(|s| s.id)
            .collect()
            .await;

        assert_eq!(peer_count / 2, peers_from_db.len(), "lengths must match");
        assert_eq!(
            peers.into_iter().skip(5).rev().collect::<Vec<_>>(),
            peers_from_db,
            "peer ids must match"
        );
    }

    #[async_std::test]
    async fn test_should_update_stats_when_updating_peers() {
        let db = SqliteNetworkBackend::new(SqliteNetworkBackendConfig::default()).await;
        let peer_id_1 = PeerId::random();
        let peer_id_2 = PeerId::random();

        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id_1}").parse().unwrap();
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id_2}").parse().unwrap();

        db.add(&peer_id_1, PeerOrigin::IncomingConnection, vec![ma_1])
            .await
            .expect("should add peer");

        let stats = db.stats().await.expect("must get stats");
        assert_eq!(
            Stats {
                good_quality_public: 0,
                bad_quality_public: 1,
                good_quality_non_public: 0,
                bad_quality_non_public: 0,
            },
            stats,
            "stats must be equal"
        );

        db.add(&peer_id_2, PeerOrigin::IncomingConnection, vec![ma_2])
            .await
            .expect("should add peer");

        let stats = db.stats().await.expect("must get stats");
        assert_eq!(
            Stats {
                good_quality_public: 0,
                bad_quality_public: 2,
                good_quality_non_public: 0,
                bad_quality_non_public: 0,
            },
            stats,
            "stats must be equal"
        );

        let peer_status = PeerStatus {
            id: peer_id_1,
            origin: PeerOrigin::IncomingConnection,
            is_public: true,
            last_seen: 1,
            last_seen_latency: 2,
            heartbeats_sent: 3,
            heartbeats_succeeded: 4,
            backoff: 1.0,
            ignored: false,
            peer_version: Some("1.2.3".into()),
            multiaddresses: vec![],
            quality: 0.8,
            quality_avg: SingleSumSMA::new_with_samples(3, vec![0.1_f64, 0.4_64, 0.6_f64]),
        };
        db.update(&peer_status).await.expect("must be able to update peer");

        let stats = db.stats().await.expect("must get stats");
        assert_eq!(
            Stats {
                good_quality_public: 1,
                bad_quality_public: 1,
                good_quality_non_public: 0,
                bad_quality_non_public: 0,
            },
            stats,
            "stats must be equal"
        );

        let peer_status = PeerStatus {
            id: peer_id_2,
            origin: PeerOrigin::IncomingConnection,
            is_public: false,
            last_seen: 1,
            last_seen_latency: 2,
            heartbeats_sent: 3,
            heartbeats_succeeded: 4,
            backoff: 1.0,
            ignored: false,
            peer_version: Some("1.2.3".into()),
            multiaddresses: vec![],
            quality: 0.0,
            quality_avg: SingleSumSMA::new_with_samples(3, vec![0.1_f64, 0.4_64, 0.6_f64]),
        };
        db.update(&peer_status).await.expect("must be able to update peer");

        let stats = db.stats().await.expect("must get stats");
        assert_eq!(
            Stats {
                good_quality_public: 1,
                bad_quality_public: 0,
                good_quality_non_public: 0,
                bad_quality_non_public: 1,
            },
            stats,
            "stats must be equal"
        );

        db.remove(&peer_id_1).await.expect("should remove peer");

        let stats = db.stats().await.expect("must get stats");
        assert_eq!(
            Stats {
                good_quality_public: 0,
                bad_quality_public: 0,
                good_quality_non_public: 0,
                bad_quality_non_public: 1,
            },
            stats,
            "stats must be equal"
        );
    }
}
