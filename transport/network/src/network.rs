use std::{
    collections::hash_set::HashSet,
    time::{Duration, SystemTime},
};

use futures::StreamExt;
pub use hopr_db_api::peers::{HoprDbPeersOperations, PeerOrigin, PeerSelector, PeerStatus, Stats};
use hopr_platform::time::current_time;
use hopr_primitive_types::sma::SMA;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use tracing::debug;
#[cfg(all(feature = "prometheus", not(test)))]
use {
    hopr_metrics::metrics::{MultiGauge, SimpleGauge},
    hopr_primitive_types::prelude::*,
};

use crate::{config::NetworkConfig, errors::Result};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_NETWORK_HEALTH: SimpleGauge =
        SimpleGauge::new("hopr_network_health", "Connectivity health indicator").unwrap();
    static ref METRIC_PEERS_BY_QUALITY: MultiGauge =
        MultiGauge::new("hopr_peers_by_quality", "Number different peer types by quality",
            &["type", "quality"],
        ).unwrap();
    static ref METRIC_PEER_COUNT: SimpleGauge =
        SimpleGauge::new("hopr_peer_count", "Number of all peers").unwrap();
    static ref METRIC_NETWORK_HEALTH_TIME_TO_GREEN: SimpleGauge = SimpleGauge::new(
        "hopr_time_to_green_sec",
        "Time it takes for a node to transition to the GREEN network state"
    ).unwrap();
}

/// Network health represented with colors, where green is the best and red
/// is the worst possible observed nework quality.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, strum::Display, strum::EnumString)]
pub enum Health {
    /// Unknown health, on application startup
    Unknown = 0,
    /// No connection, default
    Red = 1,
    /// Low quality connection to at least 1 public relay
    Orange = 2,
    /// High quality connection to at least 1 public relay
    Yellow = 3,
    /// High quality connection to at least 1 public relay and 1 NAT node
    Green = 4,
}

/// Calculate the health factor for network from the available stats
fn health_from_stats(stats: &Stats, is_public: bool) -> Health {
    let mut health = Health::Red;

    if stats.bad_quality_public > 0 {
        health = Health::Orange;
    }

    if stats.good_quality_public > 0 {
        health = if is_public || stats.good_quality_non_public > 0 {
            Health::Green
        } else {
            Health::Yellow
        };
    }

    health
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateFailure {
    /// Check timed out
    Timeout,
    /// Dial failure
    DialFailure,
}

/// The network object storing information about the running observed state of the network,
/// including peers, connection qualities and updates for other parts of the system.
#[derive(Debug)]
pub struct Network<T>
where
    T: HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
{
    me: PeerId,
    me_addresses: Vec<Multiaddr>,
    am_i_public: bool,
    cfg: NetworkConfig,
    db: T,
    #[cfg(all(feature = "prometheus", not(test)))]
    started_at: Duration,
}

impl<T> Network<T>
where
    T: HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
{
    pub fn new(my_peer_id: PeerId, my_multiaddresses: Vec<Multiaddr>, cfg: NetworkConfig, db: T) -> Self {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_NETWORK_HEALTH.set(0.0);
            METRIC_NETWORK_HEALTH_TIME_TO_GREEN.set(0.0);
            METRIC_PEERS_BY_QUALITY.set(&["public", "high"], 0.0);
            METRIC_PEERS_BY_QUALITY.set(&["public", "low"], 0.0);
            METRIC_PEERS_BY_QUALITY.set(&["nonPublic", "high"], 0.0);
            METRIC_PEERS_BY_QUALITY.set(&["nonPublic", "low"], 0.0);
        }

        Self {
            me: my_peer_id,
            me_addresses: my_multiaddresses,
            am_i_public: true,
            cfg,
            db,
            #[cfg(all(feature = "prometheus", not(test)))]
            started_at: current_time().as_unix_timestamp(),
        }
    }

    /// Check whether the PeerId is present in the network.
    #[tracing::instrument(level = "debug", skip(self), ret(Display))]
    pub async fn has(&self, peer: &PeerId) -> bool {
        peer == &self.me || self.db.get_network_peer(peer).await.is_ok_and(|p| p.is_some())
    }

    /// Add a new peer into the network.
    ///
    /// Each peer must have an origin specification.
    #[tracing::instrument(level = "debug", skip(self), ret(level = "trace"), err)]
    pub async fn add(&self, peer: &PeerId, origin: PeerOrigin, mut addrs: Vec<Multiaddr>) -> Result<()> {
        if peer == &self.me {
            return Err(crate::errors::NetworkingError::DisallowedOperationOnOwnPeerIdError);
        }

        if let Some(mut peer_status) = self.db.get_network_peer(peer).await? {
            debug!(%peer, %origin, multiaddresses = ?addrs, "Updating existing peer in the store");

            if !peer_status.is_ignored() || matches!(origin, PeerOrigin::IncomingConnection) {
                peer_status.ignored_until = None;
            }

            peer_status.multiaddresses.append(&mut addrs);
            peer_status.multiaddresses = peer_status
                .multiaddresses
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            self.db.update_network_peer(peer_status).await?;
        } else {
            debug!(%peer, %origin, multiaddresses = ?addrs, "Adding peer to the store");

            self.db
                .add_network_peer(
                    peer,
                    origin,
                    addrs,
                    self.cfg.backoff_exponent,
                    self.cfg.quality_avg_window_size,
                )
                .await?;
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let stats = self.db.network_peer_stats(self.cfg.quality_bad_threshold).await?;
            self.refresh_metrics(&stats)
        }

        Ok(())
    }

    /// Get peer information and status.
    #[tracing::instrument(level = "debug", skip(self), ret(level = "trace"), err)]
    pub async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>> {
        if peer == &self.me {
            Ok(Some({
                let mut ps = PeerStatus::new(*peer, PeerOrigin::Initialization, 0.0f64, 2u32);
                ps.multiaddresses.clone_from(&self.me_addresses);
                ps
            }))
        } else {
            Ok(self.db.get_network_peer(peer).await?)
        }
    }

    /// Remove peer from the network
    #[tracing::instrument(level = "debug", skip(self), ret(level = "trace"), err)]
    pub async fn remove(&self, peer: &PeerId) -> Result<()> {
        if peer == &self.me {
            return Err(crate::errors::NetworkingError::DisallowedOperationOnOwnPeerIdError);
        }

        self.db.remove_network_peer(peer).await?;

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let stats = self.db.network_peer_stats(self.cfg.quality_bad_threshold).await?;
            self.refresh_metrics(&stats);
            tracing::info!(
                health = %health_from_stats(&stats, self.am_i_public),
                trigger = "peer removal",
                "Network health updated"
            );
        }

        Ok(())
    }

    /// Updates a peer's record with the result of a heartbeat ping.
    ///
    /// Adjusts the peer's quality, backoff, and ignore status based on the ping outcome. If the peer's quality drops
    /// below configured thresholds, may trigger a connection close or quality update event. Returns an error if called
    /// on the local peer.
    ///
    /// # Returns
    /// - `Ok(Some(NetworkTriggeredEvent))` if the peer's status changed and an event should be triggered.
    /// - `Ok(None)` if the peer is unknown.
    /// - `Err(NetworkingError)` if the operation is disallowed or a database error occurs.
    #[tracing::instrument(level = "debug", skip(self), ret(level = "trace"), err)]
    pub async fn update(&self, peer: &PeerId, ping_result: std::result::Result<Duration, UpdateFailure>) -> Result<()> {
        if peer == &self.me {
            return Err(crate::errors::NetworkingError::DisallowedOperationOnOwnPeerIdError);
        }

        if let Some(mut entry) = self.db.get_network_peer(peer).await? {
            entry.heartbeats_sent += 1;

            match ping_result {
                Ok(latency) => {
                    if !entry.is_ignored() {
                        entry.ignored_until = None;
                    }
                    entry.last_seen = current_time();
                    entry.last_seen_latency = latency;
                    entry.heartbeats_succeeded += 1;
                    // reset backoff in case of a successful ping
                    entry.backoff = self.cfg.backoff_min;
                    entry.update_quality(1.0_f64.min(entry.get_quality() + self.cfg.quality_step));
                }
                Err(error) => match error {
                    UpdateFailure::Timeout => {
                        tracing::trace!("Update failed with timeout");
                        // increase backoff in case of a failed ping, but cap it at the max backoff to
                        // prevent entries from being shut out
                        entry.backoff = self.cfg.backoff_max.min(entry.backoff.powf(self.cfg.backoff_exponent));
                        entry.update_quality(0.0_f64.max(entry.get_quality() - self.cfg.quality_step));

                        let q = entry.get_quality();

                        if q < self.cfg.quality_bad_threshold {
                            entry.ignored_until = Some(current_time() + self.cfg.ignore_timeframe);
                        }
                    }
                    UpdateFailure::DialFailure => {
                        tracing::trace!("Update failed with dial failure");
                        entry.update_quality(0.0_f64);
                        entry.ignored_until = Some(
                            current_time()
                                + crate::config::DEFAULT_CANNOT_DIAL_PENALTY
                                + std::time::Duration::from_secs(hopr_crypto_random::random_integer(0, Some(600))),
                        );
                    }
                },
            }

            tracing::trace!(%peer, quality = entry.quality, quality_avg = entry.quality_avg.average(), "Updating peer status in the store");
            self.db.update_network_peer(entry).await?;

            #[cfg(all(feature = "prometheus", not(test)))]
            {
                let stats = self.db.network_peer_stats(self.cfg.quality_bad_threshold).await?;
                self.refresh_metrics(&stats);
                tracing::info!(
                    health = %health_from_stats(&stats, self.am_i_public),
                    trigger = "peer update",
                    "Network health updated"
                );
            }

            Ok(())
        } else {
            debug!(%peer, "Ignoring update request for unknown peer");
            Ok(())
        }
    }

    /// Returns the quality of the network as a network health indicator.
    pub async fn health(&self) -> Health {
        self.db
            .network_peer_stats(self.cfg.quality_bad_threshold)
            .await
            .map(|stats| health_from_stats(&stats, self.am_i_public))
            .unwrap_or(Health::Unknown)
    }

    /// Update the internally perceived network status that is processed to the network health
    #[cfg(all(feature = "prometheus", not(test)))]
    fn refresh_metrics(&self, stats: &Stats) {
        let health = health_from_stats(stats, self.am_i_public);

        if METRIC_NETWORK_HEALTH_TIME_TO_GREEN.get() < 0.5f64 {
            if let Some(ts) = current_time().checked_sub(self.started_at) {
                METRIC_NETWORK_HEALTH_TIME_TO_GREEN.set(ts.as_unix_timestamp().as_secs_f64());
            }
        }
        METRIC_PEER_COUNT.set(stats.all_count() as f64);
        METRIC_PEERS_BY_QUALITY.set(&["public", "high"], stats.good_quality_public as f64);
        METRIC_PEERS_BY_QUALITY.set(&["public", "low"], stats.bad_quality_public as f64);
        METRIC_PEERS_BY_QUALITY.set(&["nonPublic", "high"], stats.good_quality_non_public as f64);
        METRIC_PEERS_BY_QUALITY.set(&["nonPublic", "low"], stats.bad_quality_non_public as f64);
        METRIC_NETWORK_HEALTH.set((health as i32).into());
    }

    pub async fn connected_peers(&self) -> Result<Vec<PeerId>> {
        let minimum_quality = self.cfg.quality_offline_threshold;
        self.peer_filter(|peer| async move { (peer.get_quality() > minimum_quality).then_some(peer.id.1) })
            .await
    }

    // ======
    pub(crate) async fn peer_filter<Fut, V, F>(&self, filter: F) -> Result<Vec<V>>
    where
        F: FnMut(PeerStatus) -> Fut,
        Fut: std::future::Future<Output = Option<V>>,
    {
        let stream = self.db.get_network_peers(Default::default(), false).await?;
        futures::pin_mut!(stream);
        Ok(stream.filter_map(filter).collect().await)
    }

    /// Returns a list of peer IDs eligible for pinging based on last seen time, ignore status, and backoff delay.
    ///
    /// Peers are filtered to exclude self, those currently within their ignore timeframe, and those whose
    /// backoff-adjusted delay has not yet elapsed. The resulting peers are sorted by last seen time in ascending order.
    ///
    /// # Parameters
    /// - `threshold`: The cutoff `SystemTime`; only peers whose next ping is due before this time are considered.
    ///
    /// # Returns
    /// A vector of peer IDs that should be pinged.
    #[tracing::instrument(level = "debug", skip(self, threshold), ret(level = "trace"), err, fields(since = ?threshold))]
    pub async fn find_peers_to_ping(&self, threshold: SystemTime) -> Result<Vec<PeerId>> {
        let stream = self
            .db
            .get_network_peers(PeerSelector::default().with_last_seen_lte(threshold), false)
            .await?;
        futures::pin_mut!(stream);
        let mut data: Vec<PeerStatus> = stream
            .filter_map(|v| async move {
                if v.id.1 == self.me {
                    return None;
                }

                if let Some(ignore_start) = v.ignored_until {
                    let should_be_ignored = ignore_start
                        .checked_add(self.cfg.ignore_timeframe)
                        .is_some_and(|v| v > threshold);

                    if should_be_ignored {
                        return None;
                    }
                }

                let backoff = v.backoff.powf(self.cfg.backoff_exponent);
                let delay = std::cmp::min(self.cfg.min_delay * (backoff as u32), self.cfg.max_delay);

                if (v.last_seen + delay) < threshold {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
            .await;

        data.sort_by(|a, b| {
            if a.last_seen < b.last_seen {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        Ok(data.into_iter().map(|peer| peer.id.1).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Add, time::Duration};

    use anyhow::Context;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_platform::time::native::current_time;
    use hopr_primitive_types::prelude::AsUnixTimestamp;
    use libp2p_identity::PeerId;
    use more_asserts::*;

    use super::*;
    use crate::network::{Health, Network, NetworkConfig, PeerOrigin};

    impl<T> Network<T>
    where
        T: HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
    {
        /// Checks if the peer is present in the network, but it is being currently ignored.
        async fn is_ignored(&self, peer: &PeerId) -> bool {
            peer != &self.me && self.get(peer).await.is_ok_and(|ps| ps.is_some_and(|p| p.is_ignored()))
        }
    }

    #[test]
    fn test_network_health_should_serialize_to_a_proper_string() {
        assert_eq!(format!("{}", Health::Orange), "Orange".to_owned())
    }

    #[test]
    fn test_network_health_should_deserialize_from_proper_string() -> anyhow::Result<()> {
        let parsed: Health = "Orange".parse()?;
        assert_eq!(parsed, Health::Orange);

        Ok(())
    }

    async fn basic_network(my_id: &PeerId) -> anyhow::Result<Network<hopr_db_sql::db::HoprDb>> {
        let cfg = NetworkConfig {
            quality_offline_threshold: 0.6,
            ..Default::default()
        };
        Ok(Network::new(
            *my_id,
            vec![],
            cfg,
            hopr_db_sql::db::HoprDb::new_in_memory(ChainKeypair::random()).await?,
        ))
    }

    #[test]
    fn test_network_health_should_be_ordered_numerically_for_hopr_metrics_output() {
        assert_eq!(Health::Unknown as i32, 0);
        assert_eq!(Health::Red as i32, 1);
        assert_eq!(Health::Orange as i32, 2);
        assert_eq!(Health::Yellow as i32, 3);
        assert_eq!(Health::Green as i32, 4);
    }

    #[tokio::test]
    async fn test_network_should_not_be_able_to_add_self_reference() -> anyhow::Result<()> {
        let me = PeerId::random();

        let peers = basic_network(&me).await?;

        assert!(peers.add(&me, PeerOrigin::IncomingConnection, vec![]).await.is_err());

        assert_eq!(
            0,
            peers
                .peer_filter(|peer| async move { Some(peer.id) })
                .await
                .unwrap_or(vec![])
                .len()
        );
        assert!(peers.has(&me).await);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_contain_a_registered_peer() -> anyhow::Result<()> {
        let expected: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&expected, PeerOrigin::IncomingConnection, vec![]).await?;

        assert_eq!(
            1,
            peers
                .peer_filter(|peer| async move { Some(peer.id) })
                .await
                .unwrap_or(vec![])
                .len()
        );
        assert!(peers.has(&expected).await);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_remove_a_peer_on_unregistration() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        peers.remove(&peer).await?;

        assert_eq!(
            0,
            peers
                .peer_filter(|peer| async move { Some(peer.id) })
                .await
                .unwrap_or(vec![])
                .len()
        );
        assert!(!peers.has(&peer).await);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_ignore_heartbeat_updates_for_peers_that_were_not_registered() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;

        assert_eq!(
            0,
            peers
                .peer_filter(|peer| async move { Some(peer.id) })
                .await
                .unwrap_or(vec![])
                .len()
        );
        assert!(!peers.has(&peer).await);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_be_able_to_register_a_succeeded_heartbeat_result() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        let latency = 123u64;

        peers
            .update(&peer, Ok(std::time::Duration::from_millis(latency)))
            .await?;

        let actual = peers.get(&peer).await?.expect("peer record should be present");

        assert_eq!(actual.heartbeats_sent, 1);
        assert_eq!(actual.heartbeats_succeeded, 1);
        assert_eq!(actual.last_seen_latency, std::time::Duration::from_millis(latency));

        Ok(())
    }

    #[tokio::test]
    async fn test_network_update_should_merge_metadata() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        let ts = Duration::from_millis(100);

        {
            peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;
            peers.update(&peer, Ok(ts)).await?;

            let status = peers.get(&peer).await?.context("peer should be present")?;

            assert_eq!(status.last_seen_latency, ts);
        }

        let ts = Duration::from_millis(200);

        {
            peers.update(&peer, Ok(ts)).await?;

            let status = peers.get(&peer).await?.context("peer should be present")?;

            assert_eq!(status.last_seen_latency, ts);
        }

        Ok(())
    }

    #[tokio::test]
    async fn network_should_ignore_a_peer_that_has_reached_lower_thresholds_a_specified_amount_of_time()
    -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::NetworkRegistry, vec![]).await?;

        peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;
        peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;
        peers.update(&peer, Err(UpdateFailure::Timeout)).await?; // should drop to ignored

        peers
            .update(&peer, Err(UpdateFailure::Timeout))
            .await
            .expect("no error should occur"); // should drop from network

        assert!(peers.is_ignored(&peer).await);

        // peer should remain ignored and not be added
        peers.add(&peer, PeerOrigin::ManualPing, vec![]).await?;

        assert!(peers.is_ignored(&peer).await);

        Ok(())
    }

    #[tokio::test]
    async fn network_should_stop_ignoring_a_peer_that_has_reached_lower_thresholds_but_connected_back()
    -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::NetworkRegistry, vec![]).await?;

        peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;
        peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;
        peers.update(&peer, Err(UpdateFailure::Timeout)).await?; // should drop to ignored

        peers
            .update(&peer, Err(UpdateFailure::Timeout))
            .await
            .expect("no error should occur"); // should drop from network

        assert!(peers.is_ignored(&peer).await);

        // peer should remain ignored and not be added
        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        assert!(!peers.is_ignored(&peer).await);

        Ok(())
    }

    #[tokio::test]
    async fn network_should_ignore_a_peer_that_could_not_be_dialed() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::NetworkRegistry, vec![]).await?;

        peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;
        peers.update(&peer, Err(UpdateFailure::DialFailure)).await?; // should drop to ignored

        assert!(peers.is_ignored(&peer).await);

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        assert!(!peers.is_ignored(&peer).await);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_be_able_to_register_a_failed_heartbeat_result() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        // Needs to do 3 pings, so we get over the ignore threshold limit
        // when doing the 4th failed ping
        peers
            .update(&peer, Ok(std::time::Duration::from_millis(123_u64)))
            .await?;
        peers
            .update(&peer, Ok(std::time::Duration::from_millis(200_u64)))
            .await?;
        peers
            .update(&peer, Ok(std::time::Duration::from_millis(200_u64)))
            .await?;

        peers.update(&peer, Err(UpdateFailure::Timeout)).await?;

        let actual = peers.get(&peer).await?.expect("the peer record should be present");

        assert_eq!(actual.heartbeats_succeeded, 3);
        assert_lt!(actual.backoff, 3f64);
        assert_gt!(actual.backoff, 2f64);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_not_overflow_max_backoff() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        for latency in [123_u64, 200_u64, 200_u64] {
            peers
                .update(&peer, Ok(std::time::Duration::from_millis(latency)))
                .await?;
        }

        // iterate until max backoff is reached
        loop {
            let updated_peer = peers.get(&peer).await?.expect("the peer record should be present");
            if updated_peer.backoff == peers.cfg.backoff_max {
                break;
            }

            peers.update(&peer, Err(UpdateFailure::Timeout)).await?;
        }

        // perform one more failing heartbeat update and ensure max backoff is not exceeded
        peers.update(&peer, Err(UpdateFailure::Timeout)).await?;
        let actual = peers.get(&peer).await?.expect("the peer record should be present");

        assert_eq!(actual.backoff, peers.cfg.backoff_max);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_peer_should_be_listed_for_the_ping_if_last_recorded_later_than_reference()
    -> anyhow::Result<()> {
        let first: PeerId = OffchainKeypair::random().public().into();
        let second: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&first, PeerOrigin::IncomingConnection, vec![]).await?;
        peers.add(&second, PeerOrigin::IncomingConnection, vec![]).await?;

        let latency = 77_u64;

        let mut expected = vec![first, second];
        expected.sort();

        peers
            .update(&first, Ok(std::time::Duration::from_millis(latency)))
            .await?;
        peers
            .update(&second, Ok(std::time::Duration::from_millis(latency)))
            .await?;

        // assert_eq!(
        //     format!(
        //         "{:?}",
        //         peers.should_still_be_ignored(&peers.get(&first).await.unwrap().unwrap())
        //     ),
        //     ""
        // );
        // assert_eq!(format!("{:?}", peers.get(&first).await), "");

        let mut actual = peers
            .find_peers_to_ping(current_time().add(Duration::from_secs(2u64)))
            .await?;
        actual.sort();

        assert_eq!(actual, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_have_red_health_without_any_registered_peers() -> anyhow::Result<()> {
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        assert_eq!(peers.health().await, Health::Red);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_be_unhealthy_without_any_heartbeat_updates() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        // all peers are public
        assert_eq!(peers.health().await, Health::Orange);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_be_unhealthy_without_any_peers_once_the_health_was_known() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let peers = basic_network(&me).await?;

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;
        let _ = peers.health().await;
        peers.remove(&peer).await?;

        assert_eq!(peers.health().await, Health::Red);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_low_quality() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let me: PeerId = OffchainKeypair::random().public().into();

        let cfg = NetworkConfig {
            quality_offline_threshold: 0.6,
            ..Default::default()
        };

        let peers = Network::new(
            me,
            vec![],
            cfg,
            hopr_db_sql::db::HoprDb::new_in_memory(ChainKeypair::random()).await?,
        );

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;

        assert_eq!(peers.health().await, Health::Orange);

        Ok(())
    }

    #[tokio::test]
    async fn network_should_allow_the_quality_to_go_to_0() -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let public = peer;
        let me: PeerId = OffchainKeypair::random().public().into();

        let cfg = NetworkConfig {
            quality_offline_threshold: 0.6,
            ..Default::default()
        };

        let peers = Network::new(
            me,
            vec![],
            cfg,
            hopr_db_sql::db::HoprDb::new_in_memory(ChainKeypair::random()).await?,
        );

        peers.add(&peer, PeerOrigin::NetworkRegistry, vec![]).await?;

        assert!(peers.update(&peer, Err(UpdateFailure::Timeout)).await.is_ok());

        assert!(peers.is_ignored(&public).await);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_i_am_public()
    -> anyhow::Result<()> {
        let me: PeerId = OffchainKeypair::random().public().into();
        let peer: PeerId = OffchainKeypair::random().public().into();

        let cfg = NetworkConfig {
            quality_offline_threshold: 0.3,
            ..Default::default()
        };

        let peers = Network::new(
            me,
            vec![],
            cfg,
            hopr_db_sql::db::HoprDb::new_in_memory(ChainKeypair::random()).await?,
        );

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;

        for _ in 0..3 {
            peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;
        }

        assert_eq!(peers.health().await, Health::Green);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_another_high_quality_non_public()
    -> anyhow::Result<()> {
        let peer: PeerId = OffchainKeypair::random().public().into();
        let peer2: PeerId = OffchainKeypair::random().public().into();

        let cfg = NetworkConfig {
            quality_offline_threshold: 0.3,
            ..Default::default()
        };

        let peers = Network::new(
            OffchainKeypair::random().public().into(),
            vec![],
            cfg,
            hopr_db_sql::db::HoprDb::new_in_memory(ChainKeypair::random()).await?,
        );

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await?;
        peers.add(&peer2, PeerOrigin::IncomingConnection, vec![]).await?;

        for _ in 0..3 {
            peers.update(&peer2, Ok(current_time().as_unix_timestamp())).await?;
            peers.update(&peer, Ok(current_time().as_unix_timestamp())).await?;
        }

        assert_eq!(peers.health().await, Health::Green);

        Ok(())
    }
}
