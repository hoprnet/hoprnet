use std::time::Duration;

use async_stream::stream;
use async_trait::async_trait;
use futures::{TryStreamExt, stream::BoxStream};
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_db_api::{
    errors::Result,
    peers::{HoprDbPeersOperations, PeerOrigin, PeerSelector, PeerStatus, Stats},
};
use hopr_db_entity::network_peer;
use hopr_primitive_types::prelude::*;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use sea_query::{Condition, Expr, IntoCondition, Order};
use sqlx::types::chrono::{self, DateTime, Utc};
use tracing::{error, trace};

use crate::{db::HoprDb, prelude::DbSqlError};

const DB_BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
    .with_little_endian()
    .with_variable_int_encoding();

struct WrappedPeerSelector(PeerSelector);

impl From<PeerSelector> for WrappedPeerSelector {
    fn from(selector: PeerSelector) -> Self {
        WrappedPeerSelector(selector)
    }
}

impl IntoCondition for WrappedPeerSelector {
    fn into_condition(self) -> Condition {
        let mut ret = Expr::value(1);

        if let Some(last_seen_l) = self.0.last_seen.0 {
            ret = ret.and(network_peer::Column::LastSeen.gte(chrono::DateTime::<chrono::Utc>::from(last_seen_l)));
        }

        if let Some(last_seen_u) = self.0.last_seen.1 {
            ret = ret.and(network_peer::Column::LastSeen.lte(chrono::DateTime::<chrono::Utc>::from(last_seen_u)));
        }

        if let Some(quality_l) = self.0.quality.0 {
            ret = ret.and(network_peer::Column::Quality.gte(quality_l));
        }

        if let Some(quality_u) = self.0.quality.1 {
            ret = ret.and(network_peer::Column::Quality.lte(quality_u));
        }

        ret.into_condition()
    }
}

#[async_trait]
impl HoprDbPeersOperations for HoprDb {
    #[tracing::instrument(
        skip(self),
        name = "HoprDbPeersOperations::add_network_peer",
        level = "trace",
        err,
        ret
    )]
    async fn add_network_peer(
        &self,
        peer: &PeerId,
        origin: PeerOrigin,
        mas: Vec<Multiaddr>,
        backoff: f64,
        quality_window: u32,
    ) -> Result<()> {
        let peer = *peer;
        // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
        let pubkey = hopr_parallelize::cpu::spawn_blocking(move || OffchainPublicKey::from_peerid(&peer))
            .await
            .map_err(|_| crate::errors::DbSqlError::DecodingError)?;

        let new_peer = hopr_db_entity::network_peer::ActiveModel {
            packet_key: sea_orm::ActiveValue::Set(Vec::from(pubkey.as_ref())),
            multi_addresses: sea_orm::ActiveValue::Set(
                mas.into_iter().map(|m| m.to_string()).collect::<Vec<String>>().into(),
            ),
            origin: sea_orm::ActiveValue::Set(origin as i8),
            backoff: sea_orm::ActiveValue::Set(Some(backoff)),
            quality_sma: sea_orm::ActiveValue::Set(Some(
                bincode::serde::encode_to_vec(
                    SingleSumSMA::<f64>::new(quality_window as usize),
                    DB_BINCODE_CONFIGURATION,
                )
                .map_err(|_| crate::errors::DbSqlError::DecodingError)?,
            )),
            ..Default::default()
        };

        let _ = new_peer.insert(&self.peers_db).await.map_err(DbSqlError::from)?;

        Ok(())
    }

    #[tracing::instrument(
        skip(self),
        name = "HoprDbPeersOperations::remove_network_peer",
        level = "trace",
        err,
        ret
    )]
    async fn remove_network_peer(&self, peer: &PeerId) -> Result<()> {
        let peer = *peer;
        // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
        let pubkey = hopr_parallelize::cpu::spawn_blocking(move || OffchainPublicKey::from_peerid(&peer))
            .await
            .map_err(|_| crate::errors::DbSqlError::DecodingError)?;

        let res = hopr_db_entity::network_peer::Entity::delete_many()
            .filter(hopr_db_entity::network_peer::Column::PacketKey.eq(Vec::from(pubkey.as_ref())))
            .exec(&self.peers_db)
            .await
            .map_err(DbSqlError::from)?;

        if res.rows_affected > 0 {
            Ok(())
        } else {
            Err(
                crate::errors::DbSqlError::LogicalError("peer cannot be removed because it does not exist".into())
                    .into(),
            )
        }
    }

    #[tracing::instrument(
        skip(self),
        name = "HoprDbPeersOperations::update_network_peer",
        level = "trace",
        err,
        ret
    )]
    async fn update_network_peer(&self, new_status: PeerStatus) -> Result<()> {
        let row = hopr_db_entity::network_peer::Entity::find()
            .filter(hopr_db_entity::network_peer::Column::PacketKey.eq(Vec::from(new_status.id.0.as_ref())))
            .one(&self.peers_db)
            .await
            .map_err(DbSqlError::from)?;

        if let Some(model) = row {
            let mut peer_data: hopr_db_entity::network_peer::ActiveModel = model.into();
            peer_data.packet_key = sea_orm::ActiveValue::Set(Vec::from(new_status.id.0.as_ref()));
            peer_data.multi_addresses = sea_orm::ActiveValue::Set(
                new_status
                    .multiaddresses
                    .into_iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<String>>()
                    .into(),
            );
            peer_data.origin = sea_orm::ActiveValue::Set(new_status.origin as i8);
            peer_data.last_seen = sea_orm::ActiveValue::Set(DateTime::<Utc>::from(new_status.last_seen));
            peer_data.last_seen_latency = sea_orm::ActiveValue::Set(new_status.last_seen_latency.as_millis() as i32);
            peer_data.ignored_until = sea_orm::ActiveValue::Set(new_status.ignored_until.map(DateTime::<Utc>::from));
            peer_data.quality = sea_orm::ActiveValue::Set(new_status.quality);
            peer_data.quality_sma = sea_orm::ActiveValue::Set(Some(
                bincode::serde::encode_to_vec(&new_status.quality_avg, DB_BINCODE_CONFIGURATION)
                    .map_err(|e| crate::errors::DbSqlError::LogicalError(format!("cannot serialize sma: {e}")))?,
            ));
            peer_data.backoff = sea_orm::ActiveValue::Set(Some(new_status.backoff));
            peer_data.heartbeats_sent = sea_orm::ActiveValue::Set(Some(new_status.heartbeats_sent as i32));
            peer_data.heartbeats_successful = sea_orm::ActiveValue::Set(Some(new_status.heartbeats_succeeded as i32));

            peer_data.update(&self.peers_db).await.map_err(DbSqlError::from)?;

            Ok(())
        } else {
            Err(crate::errors::DbSqlError::LogicalError(format!(
                "cannot update a non-existing peer '{}'",
                new_status.id.1
            ))
            .into())
        }
    }

    #[tracing::instrument(
        skip(self),
        name = "HoprDbPeersOperations::get_network_peer",
        level = "trace",
        err,
        ret
    )]
    async fn get_network_peer(&self, peer: &PeerId) -> Result<Option<PeerStatus>> {
        let peer = *peer;
        // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
        let pubkey = hopr_parallelize::cpu::spawn_blocking(move || OffchainPublicKey::from_peerid(&peer))
            .await
            .map_err(|_| crate::errors::DbSqlError::DecodingError)?;
        let row = hopr_db_entity::network_peer::Entity::find()
            .filter(hopr_db_entity::network_peer::Column::PacketKey.eq(Vec::from(pubkey.as_ref())))
            .one(&self.peers_db)
            .await
            .map_err(DbSqlError::from)?;

        if let Some(model) = row {
            let status: WrappedPeerStatus = model.try_into()?;
            Ok(Some(status.0))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(self), name = "HoprDbPeersOperations::get_network_peers", level = "trace", err)]
    async fn get_network_peers<'a>(
        &'a self,
        selector: PeerSelector,
        sort_last_seen_asc: bool,
    ) -> Result<BoxStream<'a, PeerStatus>> {
        let selector: WrappedPeerSelector = selector.into();
        let mut sub_stream = hopr_db_entity::network_peer::Entity::find()
            // .filter(hopr_db_entity::network_peer::Column::Ignored.is_not_null())
            .filter(selector)
            .order_by(
                network_peer::Column::LastSeen,
                if sort_last_seen_asc { Order::Asc } else { Order::Desc },
            )
            .stream(&self.peers_db)
            .await
            .map_err(DbSqlError::from)?;

        Ok(Box::pin(stream! {
            loop {
                match sub_stream.try_next().await {
                    Ok(Some(peer_row)) => {
                        trace!(?peer_row, "got db network row");
                        match WrappedPeerStatus::try_from(peer_row) {
                            Ok(peer_status) => yield peer_status.0,
                            Err(e) => error!(error = %e, "cannot map peer from row"),
                        }
                    },
                    Ok(None) => {
                        trace!("fetched all network results");
                        break;
                    }
                    Err(e) => {
                        error!(error = %e, "failed to retrieve next network row");
                        break;
                    }
                }
            }
        }))
    }

    #[tracing::instrument(
        skip(self),
        name = "HoprDbPeersOperations::network_peer_stats",
        level = "trace",
        err,
        ret
    )]
    async fn network_peer_stats(&self, quality_threshold: f64) -> Result<Stats> {
        Ok(Stats {
            good_quality_public: hopr_db_entity::network_peer::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(hopr_db_entity::network_peer::Column::IgnoredUntil.is_null())
                        .add(hopr_db_entity::network_peer::Column::Quality.gt(quality_threshold)),
                )
                .count(&self.peers_db)
                .await
                .map_err(DbSqlError::from)? as u32,
            good_quality_non_public: 0u32, // TODO: Only public peers supported in v3
            bad_quality_public: hopr_db_entity::network_peer::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(hopr_db_entity::network_peer::Column::IgnoredUntil.is_null())
                        .add(hopr_db_entity::network_peer::Column::Quality.lte(quality_threshold)),
                )
                .count(&self.peers_db)
                .await
                .map_err(DbSqlError::from)? as u32,
            bad_quality_non_public: 0u32, // TODO: Only public peers supported in v3
        })
    }
}

struct WrappedPeerStatus(PeerStatus);

impl From<PeerStatus> for WrappedPeerStatus {
    fn from(status: PeerStatus) -> Self {
        WrappedPeerStatus(status)
    }
}

impl TryFrom<hopr_db_entity::network_peer::Model> for WrappedPeerStatus {
    type Error = crate::errors::DbSqlError;

    fn try_from(value: hopr_db_entity::network_peer::Model) -> std::result::Result<Self, Self::Error> {
        let key = OffchainPublicKey::try_from(value.packet_key.as_slice()).map_err(|_| Self::Error::DecodingError)?;
        Ok(PeerStatus {
            id: (key, key.into()),
            origin: PeerOrigin::try_from(value.origin as u8).map_err(|_| Self::Error::DecodingError)?,
            last_seen: value.last_seen.into(),
            last_seen_latency: Duration::from_millis(value.last_seen_latency as u64),
            heartbeats_sent: value.heartbeats_sent.unwrap_or_default() as u64,
            heartbeats_succeeded: value.heartbeats_successful.unwrap_or_default() as u64,
            backoff: value.backoff.unwrap_or(1.0f64),
            ignored_until: value.ignored_until.map(|v| v.into()),
            multiaddresses: {
                if let sea_orm::query::JsonValue::Array(mas) = value.multi_addresses {
                    mas.into_iter()
                        .filter_map(|s| {
                            if let sea_orm::query::JsonValue::String(s) = s {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .filter(|s| !s.trim().is_empty())
                        .map(Multiaddr::try_from)
                        .collect::<std::result::Result<Vec<_>, multiaddr::Error>>()
                        .map_err(|_| Self::Error::DecodingError)
                } else {
                    Err(Self::Error::DecodingError)
                }?
            },
            quality: value.quality,
            quality_avg: bincode::serde::borrow_decode_from_slice(
                value
                    .quality_sma
                    .ok_or_else(|| Self::Error::LogicalError("the SMA should always be present for every peer".into()))?
                    .as_slice(),
                DB_BINCODE_CONFIGURATION,
            )
            .map(|(v, _bytes)| v)
            .map_err(|_| Self::Error::DecodingError)?,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Add,
        time::{Duration, SystemTime},
    };

    use futures::StreamExt;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use libp2p_identity::PeerId;
    use multiaddr::Multiaddr;

    use super::*;

    #[tokio::test]
    async fn test_add_get() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse()?;
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse()?;

        db.add_network_peer(
            &peer_id,
            PeerOrigin::IncomingConnection,
            vec![ma_1.clone(), ma_2.clone()],
            0.0,
            25,
        )
        .await?;

        let peer_from_db = db.get_network_peer(&peer_id).await?.expect("peer must exist in the db");

        let mut expected_peer = PeerStatus::new(peer_id, PeerOrigin::IncomingConnection, 0.0, 25);
        expected_peer.last_seen = SystemTime::UNIX_EPOCH;
        expected_peer.last_seen_latency = Duration::from_secs(0);
        expected_peer.multiaddresses = vec![ma_1, ma_2];

        assert_eq!(expected_peer, peer_from_db, "peer states must match");
        Ok(())
    }

    #[tokio::test]
    async fn test_should_remove_peer() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse()?;

        db.add_network_peer(&peer_id, PeerOrigin::IncomingConnection, vec![ma_1.clone()], 0.0, 25)
            .await?;
        assert!(db.get_network_peer(&peer_id).await?.is_some(), "must have peer entry");

        db.remove_network_peer(&peer_id).await?;
        assert!(
            db.get_network_peer(&peer_id).await?.is_none(),
            "peer entry must be gone"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_should_not_remove_non_existing_peer() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        db.remove_network_peer(&peer_id)
            .await
            .expect_err("must not delete non-existent peer");

        Ok(())
    }

    #[tokio::test]
    async fn test_should_not_add_duplicate_peers() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse()?;

        db.add_network_peer(&peer_id, PeerOrigin::IncomingConnection, vec![ma_1.clone()], 0.0, 25)
            .await?;
        db.add_network_peer(&peer_id, PeerOrigin::IncomingConnection, vec![], 0.0, 25)
            .await
            .expect_err("should fail adding second time");

        Ok(())
    }

    #[tokio::test]
    async fn test_should_return_none_on_non_existing_peer() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        assert!(db.get_network_peer(&peer_id).await?.is_none(), "should return none");
        Ok(())
    }

    #[tokio::test]
    async fn test_update() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse()?;
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse()?;

        db.add_network_peer(
            &peer_id,
            PeerOrigin::IncomingConnection,
            vec![ma_1.clone(), ma_2.clone()],
            0.0,
            25,
        )
        .await?;

        let mut peer_status = PeerStatus::new(peer_id, PeerOrigin::IncomingConnection, 0.2, 25);
        peer_status.last_seen = SystemTime::UNIX_EPOCH;
        peer_status.last_seen_latency = Duration::from_secs(2);
        peer_status.multiaddresses = vec![ma_1, ma_2];
        peer_status.backoff = 2.0;
        peer_status.ignored_until = None;
        for i in [0.1_f64, 0.4_f64, 0.6_f64].into_iter() {
            peer_status.update_quality(i);
        }
        peer_status.quality = peer_status.quality as f32 as f64;

        let peer_status_from_db = db.get_network_peer(&peer_id).await?.expect("entry should exist");

        assert_ne!(peer_status, peer_status_from_db, "entries must not be equal");

        db.update_network_peer(peer_status.clone()).await?;

        let peer_status_from_db = db.get_network_peer(&peer_id).await?.expect("entry should exist");

        assert_eq!(peer_status, peer_status_from_db, "entries must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_should_fail_to_update_non_existing_peer() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        let mut peer_status = PeerStatus::new(peer_id, PeerOrigin::IncomingConnection, 0.2, 25);
        peer_status.last_seen = SystemTime::UNIX_EPOCH;
        peer_status.last_seen_latency = Duration::from_secs(2);
        peer_status.backoff = 2.0;
        peer_status.ignored_until = None;
        peer_status.multiaddresses = vec![];
        for i in [0.1_f64, 0.4_f64, 0.6_f64].into_iter() {
            peer_status.update_quality(i);
        }

        db.update_network_peer(peer_status)
            .await
            .expect_err("should fail updating non-existing peer");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_multiple_should_return_all_peers() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peers = (0..10)
            .map(|_| {
                let peer_id: PeerId = OffchainKeypair::random().public().into();
                peer_id
            })
            .collect::<Vec<_>>();

        for peer in &peers {
            db.add_network_peer(peer, PeerOrigin::Initialization, vec![], 0.0, 25)
                .await?;
        }

        let peers_from_db: Vec<PeerId> = db
            .get_network_peers(Default::default(), false)
            .await?
            .map(|s| s.id.1)
            .collect()
            .await;

        assert_eq!(peers.len(), peers_from_db.len(), "lengths must match");
        assert_eq!(peers, peers_from_db, "peer ids must match");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_multiple_should_return_filtered_peers() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_count = 10;
        let peers = (0..peer_count)
            .map(|_| {
                let peer_id: PeerId = OffchainKeypair::random().public().into();
                peer_id
            })
            .collect::<Vec<_>>();

        for (i, peer) in peers.iter().enumerate() {
            db.add_network_peer(peer, PeerOrigin::Initialization, vec![], 0.2, 25)
                .await?;
            if i >= peer_count / 2 {
                let mut peer_status = PeerStatus::new(*peer, PeerOrigin::IncomingConnection, 0.2, 25);
                peer_status.last_seen = SystemTime::UNIX_EPOCH.add(Duration::from_secs(i as u64));
                peer_status.last_seen_latency = Duration::from_secs(2);
                peer_status.multiaddresses = vec![];
                peer_status.heartbeats_sent = 3;
                peer_status.heartbeats_succeeded = 4;
                peer_status.backoff = 1.0;
                peer_status.ignored_until = None;
                for i in [0.1_f64, 0.4_f64, 0.6_f64].into_iter() {
                    peer_status.update_quality(i);
                }

                db.update_network_peer(peer_status).await?;
            }
        }

        let peers_from_db: Vec<PeerId> = db
            .get_network_peers(PeerSelector::default().with_quality_gte(0.501_f64), false)
            .await?
            .map(|s| s.id.1)
            .collect()
            .await;

        assert_eq!(peer_count / 2, peers_from_db.len(), "lengths must match");
        assert_eq!(
            peers.into_iter().skip(5).rev().collect::<Vec<_>>(),
            peers_from_db,
            "peer ids must match"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_should_update_stats_when_updating_peers() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let peer_id_1: PeerId = OffchainKeypair::random().public().into();
        let peer_id_2: PeerId = OffchainKeypair::random().public().into();

        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id_1}").parse()?;
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id_2}").parse()?;

        db.add_network_peer(&peer_id_1, PeerOrigin::IncomingConnection, vec![ma_1], 0.0, 25)
            .await?;

        let stats = db.network_peer_stats(0.2).await?;
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

        db.add_network_peer(&peer_id_2, PeerOrigin::IncomingConnection, vec![ma_2], 0.0, 25)
            .await?;

        let stats = db.network_peer_stats(0.2).await?;
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

        let mut peer_status = PeerStatus::new(peer_id_1, PeerOrigin::IncomingConnection, 0.2, 25);
        peer_status.last_seen = SystemTime::UNIX_EPOCH.add(Duration::from_secs(2));
        peer_status.last_seen_latency = Duration::from_secs(2);
        peer_status.multiaddresses = vec![];
        peer_status.heartbeats_sent = 3;
        peer_status.heartbeats_succeeded = 4;
        peer_status.backoff = 1.0;
        peer_status.ignored_until = None;
        for i in [0.1_f64, 0.4_f64, 0.6_f64].into_iter() {
            peer_status.update_quality(i);
        }

        db.update_network_peer(peer_status).await?;

        let stats = db.network_peer_stats(0.2).await?;
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

        db.remove_network_peer(&peer_id_1).await?;

        let stats = db.network_peer_stats(0.2).await?;
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

        Ok(())
    }
}
