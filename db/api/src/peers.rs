use std::{
    str::FromStr,
    time::{Duration, SystemTime},
};

use async_stream::stream;
use async_trait::async_trait;
use futures::{stream::BoxStream, TryStreamExt};
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use sea_query::{Condition, Expr, IntoCondition, Order};
use sqlx::types::chrono::{self, DateTime, Utc};
use tracing::{error, trace, warn};

use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_db_entity::network_peer;
use hopr_primitive_types::prelude::*;

use crate::{db::HoprDb, errors::Result};

/// Actual origin.
///
/// First occurence of the peer in the network mechanism.
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum PeerOrigin {
    #[strum(to_string = "node initialization")]
    Initialization = 0,
    #[strum(to_string = "network registry")]
    NetworkRegistry = 1,
    #[strum(to_string = "incoming connection")]
    IncomingConnection = 2,
    #[strum(to_string = "outgoing connection attempt")]
    OutgoingConnection = 3,
    #[strum(to_string = "strategy monitors existing channel")]
    StrategyExistingChannel = 4,
    #[strum(to_string = "strategy considers opening a channel")]
    StrategyConsideringChannel = 5,
    #[strum(to_string = "strategy decided to open new channel")]
    StrategyNewChannel = 6,
    #[strum(to_string = "manual ping")]
    ManualPing = 7,
    #[strum(to_string = "testing")]
    Testing = 8,
}

/// Statistical observation related to peers in the network. statistics on all peer entries stored
/// in the [crate::network::Network] object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    /// Number of good quality public nodes.
    pub good_quality_public: u32,
    /// Number of bad quality public nodes.
    pub bad_quality_public: u32,
    /// Number of good quality nodes non-public nodes.
    pub good_quality_non_public: u32,
    /// Number of bad quality nodes non-public nodes.
    pub bad_quality_non_public: u32,
}

// #[cfg(all(feature = "prometheus", not(test)))]
impl Stats {
    /// Returns count of all peers.
    pub fn all_count(&self) -> usize {
        self.good_quality_public as usize
            + self.bad_quality_public as usize
            + self.good_quality_non_public as usize
            + self.bad_quality_non_public as usize
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PeerSelector {
    /// Lower and upper bounds (both inclusive) on last seen timestamp.
    pub last_seen: (Option<SystemTime>, Option<SystemTime>),
    /// Lower and upper bounds (both inclusive) on peer quality.
    pub quality: (Option<f64>, Option<f64>),
}

impl PeerSelector {
    pub fn with_last_seen_gte(mut self, lower_bound: SystemTime) -> Self {
        self.last_seen.0 = Some(lower_bound);
        self
    }

    pub fn with_last_seen_lte(mut self, upper_bound: SystemTime) -> Self {
        self.last_seen.1 = Some(upper_bound);
        self
    }

    pub fn with_quality_gte(mut self, lower_bound: f64) -> Self {
        self.quality.0 = Some(lower_bound);
        self
    }

    pub fn with_quality_lte(mut self, upper_bound: f64) -> Self {
        self.quality.1 = Some(upper_bound);
        self
    }
}

impl IntoCondition for PeerSelector {
    fn into_condition(self) -> Condition {
        let mut ret = Expr::value(1);

        if let Some(last_seen_l) = self.last_seen.0 {
            ret = ret.and(network_peer::Column::LastSeen.gte(chrono::DateTime::<chrono::Utc>::from(last_seen_l)));
        }

        if let Some(last_seen_u) = self.last_seen.1 {
            ret = ret.and(network_peer::Column::LastSeen.lte(chrono::DateTime::<chrono::Utc>::from(last_seen_u)));
        }

        if let Some(quality_l) = self.quality.0 {
            ret = ret.and(network_peer::Column::Quality.gte(quality_l));
        }

        if let Some(quality_u) = self.quality.1 {
            ret = ret.and(network_peer::Column::Quality.lte(quality_u));
        }

        ret.into_condition()
    }
}

#[async_trait]
pub trait HoprDbPeersOperations {
    /// Adds a peer to the backend.
    ///
    /// Should fail if the given peer id already exists in the store.
    async fn add_network_peer(
        &self,
        peer: &PeerId,
        origin: PeerOrigin,
        mas: Vec<Multiaddr>,
        backoff: f64,
        quality_window: u32,
    ) -> Result<()>;

    /// Removes the peer from the backend.
    ///
    /// Should fail if the given peer id does not exist.
    async fn remove_network_peer(&self, peer: &PeerId) -> Result<()>;

    /// Updates stored information about the peer.
    /// Should fail if the peer does not exist in the store.
    async fn update_network_peer(&self, new_status: PeerStatus) -> Result<()>;

    /// Gets stored information about the peer.
    ///
    /// Should return `None` if such peer does not exist in the store.
    async fn get_network_peer(&self, peer: &PeerId) -> Result<Option<PeerStatus>>;

    /// Returns a stream of all stored peers, optionally matching the given [SimpleExpr] filter.
    ///
    /// The `sort_last_seen_asc` indicates whether the results should be sorted in ascending
    /// or descending order of the `last_seen` field.
    async fn get_network_peers<'a>(
        &'a self,
        selector: PeerSelector,
        sort_last_seen_asc: bool,
    ) -> Result<BoxStream<'a, PeerStatus>>;

    /// Returns the [statistics](Stats) on the stored peers.
    async fn network_peer_stats(&self, quality_threshold: f64) -> Result<Stats>;
}

#[async_trait]
impl HoprDbPeersOperations for HoprDb {
    async fn add_network_peer(
        &self,
        peer: &PeerId,
        origin: PeerOrigin,
        mas: Vec<Multiaddr>,
        backoff: f64,
        quality_window: u32,
    ) -> Result<()> {
        let new_peer = hopr_db_entity::network_peer::ActiveModel {
            packet_key: sea_orm::ActiveValue::Set(Vec::from(
                OffchainPublicKey::try_from(peer)
                    .map_err(|_| crate::errors::DbError::DecodingError)?
                    .as_ref(),
            )),
            multi_addresses: sea_orm::ActiveValue::Set(
                mas.into_iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            ),
            origin: sea_orm::ActiveValue::Set((origin as u8).into()),
            backoff: sea_orm::ActiveValue::Set(Some(backoff)),
            quality_sma: sea_orm::ActiveValue::Set(Some(
                bincode::serialize(&SingleSumSMA::<f64>::new(quality_window as usize))
                    .map_err(|_| crate::errors::DbError::DecodingError)?,
            )),
            ..Default::default()
        };

        let _ = new_peer.insert(&self.peers_db).await?;

        Ok(())
    }

    async fn remove_network_peer(&self, peer: &PeerId) -> Result<()> {
        let res = hopr_db_entity::network_peer::Entity::delete_many()
            .filter(
                hopr_db_entity::network_peer::Column::PacketKey.eq(Vec::from(
                    OffchainPublicKey::try_from(peer)
                        .map_err(|_| crate::errors::DbError::DecodingError)?
                        .as_ref(),
                )),
            )
            .exec(&self.peers_db)
            .await?;

        if res.rows_affected > 0 {
            Ok(())
        } else {
            Err(crate::errors::DbError::LogicalError(
                "peer cannot be removed because it does not exist".into(),
            ))
        }
    }

    async fn update_network_peer(&self, new_status: PeerStatus) -> Result<()> {
        let row = hopr_db_entity::network_peer::Entity::find()
            .filter(hopr_db_entity::network_peer::Column::PacketKey.eq(Vec::from(new_status.id.0.as_ref())))
            .one(&self.peers_db)
            .await?;

        if let Some(model) = row {
            let mut peer_data: hopr_db_entity::network_peer::ActiveModel = model.into();
            peer_data.packet_key = sea_orm::ActiveValue::Set(Vec::from(new_status.id.0.as_ref()));
            peer_data.multi_addresses = sea_orm::ActiveValue::Set(
                new_status
                    .multiaddresses
                    .into_iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            );
            peer_data.origin = sea_orm::ActiveValue::Set((new_status.origin as u8).into());
            peer_data.version = sea_orm::ActiveValue::Set(new_status.peer_version);
            peer_data.last_seen = sea_orm::ActiveValue::Set(DateTime::<Utc>::from(new_status.last_seen).to_string());
            peer_data.last_seen_latency = sea_orm::ActiveValue::Set(new_status.last_seen_latency.as_millis() as i32);
            peer_data.ignored =
                sea_orm::ActiveValue::Set(new_status.ignored.map(|v| DateTime::<Utc>::from(v).to_string()));
            peer_data.public = sea_orm::ActiveValue::Set(new_status.is_public);
            peer_data.quality = sea_orm::ActiveValue::Set(new_status.quality);
            peer_data.quality_sma = sea_orm::ActiveValue::Set(Some(
                bincode::serialize(&new_status.quality_avg)
                    .map_err(|e| crate::errors::DbError::LogicalError(format!("cannot serialize sma: {e}")))?,
            ));
            peer_data.backoff = sea_orm::ActiveValue::Set(Some(new_status.backoff));
            peer_data.heartbeats_sent = sea_orm::ActiveValue::Set(Some(new_status.heartbeats_sent as i32));
            peer_data.heartbeats_successful = sea_orm::ActiveValue::Set(Some(new_status.heartbeats_succeeded as i32));

            peer_data.update(&self.peers_db).await?;

            Ok(())
        } else {
            Err(crate::errors::DbError::LogicalError(format!(
                "cannot update a non-existing peer '{}'",
                new_status.id.1
            )))
        }
    }

    async fn get_network_peer(&self, peer: &PeerId) -> Result<Option<PeerStatus>> {
        let row = hopr_db_entity::network_peer::Entity::find()
            .filter(
                hopr_db_entity::network_peer::Column::PacketKey.eq(Vec::from(
                    OffchainPublicKey::try_from(peer)
                        .map_err(|_| crate::errors::DbError::DecodingError)?
                        .as_ref(),
                )),
            )
            .one(&self.peers_db)
            .await?;

        if let Some(model) = row {
            Ok(Some(model.try_into()?))
        } else {
            Ok(None)
        }
    }

    async fn get_network_peers<'a>(
        &'a self,
        selector: PeerSelector,
        sort_last_seen_asc: bool,
    ) -> Result<BoxStream<'a, PeerStatus>> {
        let mut sub_stream = hopr_db_entity::network_peer::Entity::find()
            // .filter(hopr_db_entity::network_peer::Column::Ignored.is_not_null())
            .filter(selector)
            .order_by(
                network_peer::Column::LastSeen,
                if sort_last_seen_asc { Order::Asc } else { Order::Desc },
            )
            .stream(&self.peers_db)
            .await?;

        Ok(Box::pin(stream! {
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

    async fn network_peer_stats(&self, quality_threshold: f64) -> Result<Stats> {
        Ok(Stats {
            good_quality_public: hopr_db_entity::network_peer::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(hopr_db_entity::network_peer::Column::Public.eq(true))
                        .add(hopr_db_entity::network_peer::Column::Ignored.is_null())
                        .add(hopr_db_entity::network_peer::Column::Quality.gt(quality_threshold)),
                )
                .count(&self.peers_db)
                .await? as u32,
            good_quality_non_public: hopr_db_entity::network_peer::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(hopr_db_entity::network_peer::Column::Public.eq(false))
                        .add(hopr_db_entity::network_peer::Column::Ignored.is_null())
                        .add(hopr_db_entity::network_peer::Column::Quality.gt(quality_threshold)),
                )
                .count(&self.peers_db)
                .await? as u32,
            bad_quality_public: hopr_db_entity::network_peer::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(hopr_db_entity::network_peer::Column::Public.eq(true))
                        .add(hopr_db_entity::network_peer::Column::Ignored.is_null())
                        .add(hopr_db_entity::network_peer::Column::Quality.lte(quality_threshold)),
                )
                .count(&self.peers_db)
                .await? as u32,
            bad_quality_non_public: hopr_db_entity::network_peer::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(hopr_db_entity::network_peer::Column::Public.eq(false))
                        .add(hopr_db_entity::network_peer::Column::Ignored.is_null())
                        .add(hopr_db_entity::network_peer::Column::Quality.lte(quality_threshold)),
                )
                .count(&self.peers_db)
                .await? as u32,
        })
    }
}

/// Status of the peer as recorded by the [Network].
#[derive(Debug, Clone, PartialEq)]
pub struct PeerStatus {
    pub id: (OffchainPublicKey, PeerId),
    pub origin: PeerOrigin,
    pub is_public: bool,
    pub last_seen: SystemTime,
    pub last_seen_latency: Duration,
    pub heartbeats_sent: u64,
    pub heartbeats_succeeded: u64,
    pub backoff: f64,
    pub ignored: Option<SystemTime>,
    pub peer_version: Option<String>,
    pub multiaddresses: Vec<Multiaddr>,
    pub(crate) quality: f64,
    pub(crate) quality_avg: SingleSumSMA<f64>,
}

impl PeerStatus {
    pub fn new(id: PeerId, origin: PeerOrigin, backoff: f64, quality_window: u32) -> PeerStatus {
        PeerStatus {
            id: (OffchainPublicKey::try_from(&id).expect("invalid peer id given"), id),
            origin,
            is_public: true,
            heartbeats_sent: 0,
            heartbeats_succeeded: 0,
            last_seen: SystemTime::UNIX_EPOCH,
            last_seen_latency: Duration::default(),
            ignored: None,
            backoff,
            quality: 0.0,
            peer_version: None,
            quality_avg: SingleSumSMA::new(quality_window as usize),
            multiaddresses: vec![],
        }
    }

    // Update both the immediate last quality and the average windowed quality
    pub fn update_quality(&mut self, new_value: f64) {
        if (0.0f64..=1.0f64).contains(&new_value) {
            self.quality = new_value;
            self.quality_avg.push(new_value);
        } else {
            warn!("Quality failed to update with value outside the [0,1] range")
        }
    }

    /// Gets the average quality of this peer
    pub fn get_average_quality(&self) -> f64 {
        self.quality_avg.average().unwrap_or_default()
    }

    /// Gets the immediate node quality
    pub fn get_quality(&self) -> f64 {
        self.quality
    }
}

impl std::fmt::Display for PeerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Entry: [id={}, origin={}, last seen on={:?}, quality={}, heartbeats sent={}, heartbeats succeeded={}, backoff={}]",
            self.id.1, self.origin, self.last_seen, self.quality, self.heartbeats_sent, self.heartbeats_succeeded, self.backoff)
    }
}

impl TryFrom<hopr_db_entity::network_peer::Model> for PeerStatus {
    type Error = crate::errors::DbError;

    fn try_from(value: hopr_db_entity::network_peer::Model) -> std::result::Result<Self, Self::Error> {
        let key = OffchainPublicKey::from_bytes(value.packet_key.as_slice()).map_err(|_| Self::Error::DecodingError)?;
        Ok(PeerStatus {
            id: (key, key.into()),
            origin: PeerOrigin::try_from(value.origin as u8).map_err(|_| Self::Error::DecodingError)?,
            is_public: value.public,
            last_seen: chrono::DateTime::<chrono::Utc>::from_str(&value.last_seen)
                .map_err(|_| Self::Error::DecodingError)?
                .into(),
            last_seen_latency: Duration::from_millis(value.last_seen_latency as u64),
            heartbeats_sent: value.heartbeats_sent.unwrap_or_default() as u64,
            heartbeats_succeeded: value.heartbeats_successful.unwrap_or_default() as u64,
            backoff: value.backoff.unwrap_or(1.0),
            ignored: if let Some(v) = value.ignored {
                Some(
                    chrono::DateTime::<chrono::Utc>::from_str(&v)
                        .map_err(|_| Self::Error::DecodingError)?
                        .into(),
                )
            } else {
                None
            },
            peer_version: value.version,
            multiaddresses: value
                .multi_addresses
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(Multiaddr::try_from)
                .collect::<core::result::Result<Vec<_>, multiaddr::Error>>()
                .map_err(|_| Self::Error::DecodingError)?,
            quality: value.quality,
            quality_avg: bincode::deserialize(
                value
                    .quality_sma
                    .ok_or_else(|| Self::Error::LogicalError("the SMA should always be present for every peer".into()))?
                    .as_slice(),
            )
            .map_err(|_| Self::Error::DecodingError)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use libp2p_identity::PeerId;
    use multiaddr::Multiaddr;
    use std::ops::Add;
    use std::time::{Duration, SystemTime};

    #[async_std::test]
    async fn test_add_get() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse().unwrap();

        db.add_network_peer(
            &peer_id,
            PeerOrigin::IncomingConnection,
            vec![ma_1.clone(), ma_2.clone()],
            0.0,
            25,
        )
        .await
        .expect("should add peer");

        let peer_from_db = db
            .get_network_peer(&peer_id)
            .await
            .expect("should be able to get a peer")
            .expect("peer must exist in the db");

        let mut expected_peer = PeerStatus::new(peer_id, PeerOrigin::IncomingConnection, 0.0, 25);
        expected_peer.last_seen = SystemTime::UNIX_EPOCH;
        expected_peer.last_seen_latency = Duration::from_secs(0);
        expected_peer.multiaddresses = vec![ma_1, ma_2];

        assert_eq!(expected_peer, peer_from_db, "peer states must match");
    }

    #[async_std::test]
    async fn test_should_remove_peer() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();

        db.add_network_peer(&peer_id, PeerOrigin::IncomingConnection, vec![ma_1.clone()], 0.0, 25)
            .await
            .expect("should add peer");
        assert!(
            db.get_network_peer(&peer_id).await.expect("should get peer").is_some(),
            "must have peer entry"
        );

        db.remove_network_peer(&peer_id).await.expect("must remove peer");
        assert!(
            db.get_network_peer(&peer_id).await.expect("should get peer").is_none(),
            "peer entry must be gone"
        );
    }

    #[async_std::test]
    async fn test_should_not_remove_non_existing_peer() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        db.remove_network_peer(&peer_id)
            .await
            .expect_err("must not delete non-existent peer");
    }

    #[async_std::test]
    async fn test_should_not_add_duplicate_peers() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();

        db.add_network_peer(&peer_id, PeerOrigin::IncomingConnection, vec![ma_1.clone()], 0.0, 25)
            .await
            .expect("should add peer");
        db.add_network_peer(&peer_id, PeerOrigin::IncomingConnection, vec![], 0.0, 25)
            .await
            .expect_err("should fail adding second time");
    }

    #[async_std::test]
    async fn test_should_return_none_on_non_existing_peer() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        assert!(
            db.get_network_peer(&peer_id).await.expect("should succeed").is_none(),
            "should return none"
        );
    }

    #[async_std::test]
    async fn test_update() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id}").parse().unwrap();
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id}").parse().unwrap();

        db.add_network_peer(
            &peer_id,
            PeerOrigin::IncomingConnection,
            vec![ma_1.clone(), ma_2.clone()],
            0.0,
            25,
        )
        .await
        .expect("should add peer");

        let mut peer_status = PeerStatus::new(peer_id, PeerOrigin::IncomingConnection, 0.2, 25);
        peer_status.last_seen = SystemTime::UNIX_EPOCH;
        peer_status.last_seen_latency = Duration::from_secs(2);
        peer_status.multiaddresses = vec![ma_1, ma_2];
        peer_status.backoff = 2.0;
        peer_status.ignored = None;
        peer_status.peer_version = Some("1.2.3".into());
        for i in [0.1_f64, 0.4_64, 0.6_f64].into_iter() {
            peer_status.update_quality(i);
        }

        let peer_status_from_db = db
            .get_network_peer(&peer_id)
            .await
            .expect("get should succeed")
            .expect("entry should exist");

        assert_ne!(peer_status, peer_status_from_db, "entries must not be equal");

        db.update_network_peer(peer_status.clone())
            .await
            .expect("update should succeed");

        let peer_status_from_db = db
            .get_network_peer(&peer_id)
            .await
            .expect("get should succeed")
            .expect("entry should exist");

        assert_eq!(peer_status, peer_status_from_db, "entries must be equal");
    }

    #[async_std::test]
    async fn test_should_fail_to_update_non_existing_peer() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id: PeerId = OffchainKeypair::random().public().into();

        let mut peer_status = PeerStatus::new(peer_id, PeerOrigin::IncomingConnection, 0.2, 25);
        peer_status.last_seen = SystemTime::UNIX_EPOCH;
        peer_status.last_seen_latency = Duration::from_secs(2);
        peer_status.backoff = 2.0;
        peer_status.ignored = None;
        peer_status.peer_version = Some("1.2.3".into());
        peer_status.multiaddresses = vec![];
        for i in [0.1_f64, 0.4_64, 0.6_f64].into_iter() {
            peer_status.update_quality(i);
        }

        db.update_network_peer(peer_status)
            .await
            .expect_err("should fail updating non-existing peer");
    }

    #[async_std::test]
    async fn test_get_multiple_should_return_all_peers() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peers = (0..10)
            .map(|_| {
                let peer_id: PeerId = OffchainKeypair::random().public().into();
                peer_id
            })
            .collect::<Vec<_>>();

        for peer in &peers {
            db.add_network_peer(peer, PeerOrigin::Initialization, vec![], 0.0, 25)
                .await
                .expect("should not fail adding peers");
        }

        let peers_from_db: Vec<PeerId> = db
            .get_network_peers(Default::default(), false)
            .await
            .expect("should get stream")
            .map(|s| s.id.1)
            .collect()
            .await;

        assert_eq!(peers.len(), peers_from_db.len(), "lengths must match");
        assert_eq!(peers, peers_from_db, "peer ids must match");
    }

    #[async_std::test]
    async fn test_get_multiple_should_return_filtered_peers() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_count = 10;
        let peers = (0..peer_count)
            .map(|_| {
                let peer_id: PeerId = OffchainKeypair::random().public().into();
                peer_id
            })
            .collect::<Vec<_>>();

        for (i, peer) in peers.iter().enumerate() {
            db.add_network_peer(peer, PeerOrigin::Initialization, vec![], 0.2, 25)
                .await
                .expect("should not fail adding peers");
            if i >= peer_count / 2 {
                let mut peer_status = PeerStatus::new(*peer, PeerOrigin::IncomingConnection, 0.2, 25);
                peer_status.last_seen = SystemTime::UNIX_EPOCH.add(Duration::from_secs(i as u64));
                peer_status.last_seen_latency = Duration::from_secs(2);
                peer_status.multiaddresses = vec![];
                peer_status.heartbeats_sent = 3;
                peer_status.heartbeats_succeeded = 4;
                peer_status.backoff = 1.0;
                peer_status.ignored = None;
                peer_status.peer_version = Some("1.2.3".into());
                for i in [0.1_f64, 0.4_64, 0.6_f64].into_iter() {
                    peer_status.update_quality(i);
                }

                db.update_network_peer(peer_status)
                    .await
                    .expect("must update peer status");
            }
        }

        let peers_from_db: Vec<PeerId> = db
            .get_network_peers(PeerSelector::default().with_quality_gte(0.501_f64), false)
            .await
            .expect("should get stream")
            .map(|s| s.id.1)
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
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id_1: PeerId = OffchainKeypair::random().public().into();
        let peer_id_2: PeerId = OffchainKeypair::random().public().into();

        let ma_1: Multiaddr = format!("/ip4/127.0.0.1/tcp/10000/p2p/{peer_id_1}").parse().unwrap();
        let ma_2: Multiaddr = format!("/ip4/127.0.0.1/tcp/10002/p2p/{peer_id_2}").parse().unwrap();

        db.add_network_peer(&peer_id_1, PeerOrigin::IncomingConnection, vec![ma_1], 0.0, 25)
            .await
            .expect("should add peer");

        let stats = db.network_peer_stats(0.2).await.expect("must get stats");
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
            .await
            .expect("should add peer");

        let stats = db.network_peer_stats(0.2).await.expect("must get stats");
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
        peer_status.ignored = None;
        peer_status.peer_version = Some("1.2.3".into());
        for i in [0.1_f64, 0.4_64, 0.6_f64].into_iter() {
            peer_status.update_quality(i);
        }

        db.update_network_peer(peer_status)
            .await
            .expect("must be able to update peer");

        let stats = db.network_peer_stats(0.2).await.expect("must get stats");
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

        let mut peer_status = PeerStatus::new(peer_id_2, PeerOrigin::IncomingConnection, 0.2, 25);
        peer_status.last_seen = SystemTime::UNIX_EPOCH.add(Duration::from_secs(2));
        peer_status.last_seen_latency = Duration::from_secs(2);
        peer_status.multiaddresses = vec![];
        peer_status.is_public = false;
        peer_status.heartbeats_sent = 3;
        peer_status.heartbeats_succeeded = 4;
        peer_status.backoff = 2.0;
        peer_status.ignored = None;
        peer_status.peer_version = Some("1.2.3".into());

        db.update_network_peer(peer_status)
            .await
            .expect("must be able to update peer");

        let stats = db.network_peer_stats(0.2).await.expect("must get stats");
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

        db.remove_network_peer(&peer_id_1).await.expect("should remove peer");

        let stats = db.network_peer_stats(0.2).await.expect("must get stats");
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
