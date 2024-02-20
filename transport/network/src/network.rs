use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::time::Duration;

use hopr_platform::time::native::current_time;
use hopr_primitive_types::traits::AsUnixTimestamp;
use libp2p_identity::PeerId;

use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::Validate;

use crate::backend::{SqliteNetworkBackend, SqliteNetworkBackendConfig};
use crate::constants::DEFAULT_NETWORK_QUALITY_THRESHOLD;
use crate::traits::NetworkBackend;
use hopr_primitive_types::sma::{SingleSumSMA, SMA};
use tracing::{debug, warn};

#[cfg(all(feature = "prometheus", not(test)))]
use {
    crate::traits::Stats,
    hopr_metrics::metrics::{MultiGauge, SimpleGauge},
};

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

/// Configuration for the [Network] object
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    /// Minimum delay will be multiplied by backoff, it will be half the actual minimum value
    #[serde_as(as = "DurationSeconds<u64>")]
    pub min_delay: Duration,

    /// Maximum delay
    #[serde_as(as = "DurationSeconds<u64>")]
    pub max_delay: Duration,

    #[validate(range(min = 0.0, max = 1.0))]
    pub quality_bad_threshold: f64,

    #[validate(range(min = 0.0, max = 1.0))]
    pub quality_offline_threshold: f64,

    pub quality_step: f64,

    /// Size of the window for quality moving average
    #[validate(range(min = 1_u32))]
    pub quality_avg_window_size: u32,

    #[serde_as(as = "DurationSeconds<u64>")]
    pub ignore_timeframe: Duration,

    pub backoff_exponent: f64,

    pub backoff_min: f64,

    pub backoff_max: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let min_delay_in_s = 1;
        let max_delay_in_s = 300;

        Self {
            min_delay: Duration::from_secs(min_delay_in_s),
            max_delay: Duration::from_secs(max_delay_in_s), // 5 minutes
            quality_bad_threshold: 0.2,
            quality_offline_threshold: DEFAULT_NETWORK_QUALITY_THRESHOLD,
            quality_step: 0.1,
            quality_avg_window_size: 25, // TODO: think about some reasonable default
            ignore_timeframe: Duration::from_secs(600), // 10 minutes
            backoff_exponent: 1.5,
            backoff_min: 2.0,
            backoff_max: max_delay_in_s as f64 / min_delay_in_s as f64,
        }
    }
}

/// Actual origin - first occurence of the peer in the network mechanism
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

/// Events emitted by the transport mechanism towards the network monitoring mechanism.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
pub enum NetworkEvent {
    CloseConnection(PeerId),
}

/// Trait defining the operations recognized by the [Network] object allowing it
/// to physically interact with external systems, including the transport mechanism.
#[cfg_attr(test, mockall::automock)]
pub trait NetworkExternalActions {
    fn is_public(&self, peer: &PeerId) -> bool;

    fn emit(&self, event: NetworkEvent);
}

/// Status of the peer as recorded by the [Network].
#[derive(Debug, Clone, PartialEq)]
pub struct PeerStatus {
    pub id: PeerId,
    pub origin: PeerOrigin,
    pub is_public: bool,
    pub last_seen: u64,         // timestamp
    pub last_seen_latency: u64, // duration in ms
    pub heartbeats_sent: u64,
    pub heartbeats_succeeded: u64,
    pub backoff: f64,
    pub ignored: bool,
    pub peer_version: Option<String>,
    pub multiaddresses: Vec<Multiaddr>,
    pub(crate) quality: f64,
    pub(crate) quality_avg: SingleSumSMA<f64>,
}

impl PeerStatus {
    fn new(id: PeerId, origin: PeerOrigin, backoff: f64, quality_window: u32) -> PeerStatus {
        PeerStatus {
            id,
            origin,
            is_public: false,
            heartbeats_sent: 0,
            heartbeats_succeeded: 0,
            last_seen: 0,
            last_seen_latency: 0,
            ignored: false,
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
        write!(f, "Entry: [id={}, origin={}, last seen on={}, quality={}, heartbeats sent={}, heartbeats succeeded={}, backoff={}]",
            self.id, self.origin, self.last_seen, self.quality, self.heartbeats_sent, self.heartbeats_succeeded, self.backoff)
    }
}

/// Calculate the health factor for network from the available stats
#[cfg(all(feature = "prometheus", not(test)))]
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

/// The network object storing information about the running observed state of the network,
/// including peers, connection qualities and updates for other parts of the system.
#[derive(Debug)]
pub struct Network<T: NetworkExternalActions> {
    me: PeerId,
    cfg: NetworkConfig,
    db: crate::backend::SqliteNetworkBackend,
    entries: HashMap<PeerId, PeerStatus>,
    last_health: Health,
    network_actions_api: T,
    #[cfg(all(feature = "prometheus", not(test)))]
    started_at: std::time::Duration,
}

impl<T: NetworkExternalActions> Network<T> {
    pub fn new(my_peer_id: PeerId, cfg: NetworkConfig, network_actions_api: T) -> Self {
        if cfg.quality_offline_threshold < cfg.quality_bad_threshold {
            panic!(
                "Strict requirement failed, bad quality threshold {} must be lower than quality offline threshold {}",
                cfg.quality_bad_threshold, cfg.quality_offline_threshold
            );
        }

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
            cfg: cfg.clone(),
            db: async_std::task::block_on(SqliteNetworkBackend::new(SqliteNetworkBackendConfig {
                network_options: cfg.clone(),
            })),
            entries: HashMap::new(),
            last_health: Health::Unknown,
            network_actions_api,
            #[cfg(all(feature = "prometheus", not(test)))]
            started_at: current_time().as_unix_timestamp(),
        }
    }

    /// Check whether the PeerId is present in the network
    pub async fn has(&self, peer: &PeerId) -> bool {
        self.db.get(peer).await.is_ok_and(|p| p.is_some())
    }

    /// Add a new peer into the network
    ///
    /// Each PeerId must have an origin specification.
    pub async fn add(&self, peer: &PeerId, origin: PeerOrigin, mut addrs: Vec<Multiaddr>) -> crate::errors::Result<()> {
        if peer == &self.me {
            return Err(crate::errors::NetworkingError::DisallowedOperationOnOwnPeerIdError);
        }

        debug!("Adding '{peer}' from {origin} with multiaddresses {addrs:?}");
        // let now = current_time().as_unix_timestamp().as_millis() as u64;

        // if !is_excluded {

        //     if !has_entry && !is_ignored {
        //         let mut entry = PeerStatus::new(*peer, origin, self.cfg.backoff_min, self.cfg.quality_avg_window_size);
        //         entry.is_public = self.network_actions_api.is_public(peer);
        //         self.refresh_network_status().await;

        //         if let Some(x) = self.entries.insert(*peer, entry) {
        //             warn!("Evicting an existing record for {}, this should not happen!", &x);
        //         }
        //     }
        // }

        if let Some(mut peer_status) = self.db.get(&peer).await? {
            let is_ignored = false;
            // TODO: check if should still be ignored
            //     let is_ignored = if !has_entry && self.ignored.contains_key(peer) {
            //         let timestamp = self.ignored.get(peer).unwrap();
            //         if Duration::from_millis(now - timestamp) > self.cfg.ignore_timeframe {
            //             self.ignored.remove(peer);
            //             false
            //         } else {
            //             true
            //         }
            //     } else {
            //         false
            //     };
            if !is_ignored && addrs.len() > 0 {
                peer_status.multiaddresses.append(&mut addrs);
                peer_status.multiaddresses = peer_status
                    .multiaddresses
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                self.db.update(&peer_status).await?;
            }
        } else {
            self.db.add(&peer, origin, addrs).await?;
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        let _ = self.refresh_metrics();

        Ok(())
    }

    /// Remove PeerId from the network
    pub async fn remove(&self, peer: &PeerId) -> crate::errors::Result<()> {
        if peer == &self.me {
            return Err(crate::errors::NetworkingError::DisallowedOperationOnOwnPeerIdError);
        }

        self.db.remove(peer).await?;

        #[cfg(all(feature = "prometheus", not(test)))]
        let _ = self.refresh_metrics();

        Ok(())
    }

    /// Get all registered multiaddresses for a specific peer
    pub async fn get_peer_multiaddresses(&self, peer: &PeerId) -> crate::errors::Result<Vec<Multiaddr>> {
        Ok(self.db.get(peer).await?.map(|p| p.multiaddresses).unwrap_or(vec![]))
    }

    /// Update the PeerId record in the network
    pub async fn update_from_ping(
        &self,
        peer: &PeerId,
        ping_result: crate::ping::PingResult,
        version: Option<String>,
    ) -> crate::errors::Result<Option<PeerStatus>> {
        if peer == &self.me {
            return Err(crate::errors::NetworkingError::DisallowedOperationOnOwnPeerIdError);
        }

        if let Some(mut entry) = self.db.get(peer).await? {
            entry.heartbeats_sent += 1;
            entry.is_public = self.network_actions_api.is_public(peer);
            entry.peer_version = version;

            if let Ok(latency) = ping_result {
                entry.last_seen = current_time().as_unix_timestamp().as_millis() as u64;
                entry.last_seen_latency = latency;
                entry.heartbeats_succeeded += 1;
                entry.backoff = self.cfg.backoff_min;
                entry.update_quality(1.0_f64.min(entry.quality + self.cfg.quality_step));
            } else {
                entry.backoff = self.cfg.backoff_max.max(entry.backoff.powf(self.cfg.backoff_exponent));
                entry.update_quality(0.0_f64.max(entry.quality - self.cfg.quality_step));

                if entry.quality < (self.cfg.quality_step / 2.0) {
                    self.network_actions_api.emit(NetworkEvent::CloseConnection(entry.id));
                    self.db.remove(&entry.id).await?;
                    return Ok(Some(entry));
                } else if entry.quality < self.cfg.quality_bad_threshold {
                    // TODO: solve the ignored case in the DB
                    // self.ignored
                    //     .insert(entry.id, current_time().as_unix_timestamp().as_millis() as u64);
                }
            }

            self.db.update(&entry).await?;

            #[cfg(all(feature = "prometheus", not(test)))]
            let _ = self.refresh_metrics().await;

            Ok(Some(entry))
        } else {
            debug!("Ignoring update request for unknown peer {}", peer);
            Ok(None)
        }
    }

    /// Update the internally perceived network status that is processed to the network health
    #[cfg(all(feature = "prometheus", not(test)))]
    async fn refresh_metrics(&self) -> crate::errors::Result<()> {
        let stats = self.db.stats().await?;
        let health = health_from_stats(&stats, self.network_actions_api.is_public(&self.me));

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

        Ok(())
    }

    pub fn get_peer_status(&self, peer: &PeerId) -> Option<PeerStatus> {
        self.entries.get(peer).cloned()
    }

    pub fn get_all_peers(&self) -> Vec<PeerId> {
        self.entries
            .values()
            .map(|peer_status| peer_status.id)
            .collect::<Vec<_>>()
    }

    /// Perform arbitrary predicate filtering operation on the network entries
    fn filter<F>(&self, f: F) -> Vec<PeerId>
    where
        F: FnMut(&&PeerStatus) -> bool,
    {
        self.entries.values().filter(f).map(|x| x.id).collect::<Vec<_>>()
    }

    pub fn all_peers(&self) -> Vec<&PeerStatus> {
        self.entries.values().collect()
    }

    pub fn find_peers_to_ping(&self, threshold: u64) -> Vec<PeerId> {
        let mut data: Vec<PeerId> = self.filter(|v| {
            let backoff = v.backoff.powf(self.cfg.backoff_exponent);
            let delay = std::cmp::min(self.cfg.min_delay * (backoff as u32), self.cfg.max_delay);

            (v.last_seen + (delay.as_millis() as u64)) < threshold
        });
        data.sort_by(|a, b| {
            if self.entries.get(a).unwrap().last_seen < self.entries.get(b).unwrap().last_seen {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        data
    }

    /// Returns the quality of the network as a network health indicator.
    pub fn health(&self) -> Health {
        self.last_health
    }
}

#[cfg(test)]
mod tests {
    use crate::network::{
        Health, MockNetworkExternalActions, Network, NetworkConfig, NetworkEvent, NetworkExternalActions, PeerOrigin,
    };
    use hopr_platform::time::native::current_time;
    use hopr_primitive_types::prelude::AsUnixTimestamp;
    use libp2p_identity::PeerId;

    #[test]
    fn test_network_health_should_serialize_to_a_proper_string() {
        assert_eq!(format!("{}", Health::Orange), "Orange".to_owned())
    }

    #[test]
    fn test_network_health_should_deserialize_from_proper_string() -> Result<(), Box<dyn std::error::Error>> {
        let parsed: Health = "Orange".parse()?;
        Ok(assert_eq!(parsed, Health::Orange))
    }

    struct DummyNetworkAction {}

    impl NetworkExternalActions for DummyNetworkAction {
        fn is_public(&self, _: &PeerId) -> bool {
            false
        }

        fn emit(&self, _: NetworkEvent) {}
    }

    fn basic_network(my_id: &PeerId) -> Network<DummyNetworkAction> {
        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;
        Network::new(*my_id, cfg, DummyNetworkAction {})
    }

    #[test]
    fn test_network_health_should_be_ordered_numerically_for_hopr_metrics_output() {
        assert_eq!(Health::Unknown as i32, 0);
        assert_eq!(Health::Red as i32, 1);
        assert_eq!(Health::Orange as i32, 2);
        assert_eq!(Health::Yellow as i32, 3);
        assert_eq!(Health::Green as i32, 4);
    }

    #[async_std::test]
    async fn test_network_should_not_contain_the_self_reference() {
        let me = PeerId::random();

        let peers = basic_network(&me);

        peers.add(&me, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        assert_eq!(0, peers.get_all_peers().len());
        assert!(!peers.has(&me).await)
    }

    #[async_std::test]
    async fn test_network_should_contain_a_registered_peer() {
        let expected = PeerId::random();

        let peers = basic_network(&PeerId::random());

        peers
            .add(&expected, PeerOrigin::IncomingConnection, vec![])
            .await
            .unwrap();

        assert_eq!(1, peers.get_all_peers().len());
        assert!(peers.has(&expected).await)
    }

    #[async_std::test]
    async fn test_network_should_remove_a_peer_on_unregistration() {
        let peer = PeerId::random();

        let peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        peers.remove(&peer).await.expect("should not fail on DB remove");

        assert_eq!(0, peers.get_all_peers().len());
        assert!(!peers.has(&peer).await)
    }

    #[async_std::test]
    async fn test_network_should_ingore_heartbeat_updates_for_peers_that_were_not_registered() {
        let peer = PeerId::random();

        let peers = basic_network(&PeerId::random());

        peers
            .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
            .await
            .expect("no error should occur");

        assert_eq!(0, peers.get_all_peers().len());
        assert!(!peers.has(&peer).await)
    }

    #[async_std::test]
    async fn test_network_should_be_able_to_register_a_succeeded_heartbeat_result() {
        let peer = PeerId::random();

        let peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        let ts = current_time().as_unix_timestamp().as_millis() as u64;

        peers
            .update_from_ping(&peer, Ok(ts), None)
            .await
            .expect("no error should occur");

        std::thread::sleep(std::time::Duration::from_millis(100));

        let actual = peers.get_peer_status(&peer).expect("peer record should be present");

        assert_eq!(actual.heartbeats_sent, 1);
        assert_eq!(actual.heartbeats_succeeded, 1);
        assert_eq!(actual.last_seen, ts);
    }

    #[async_std::test]
    async fn test_network_update_should_merge_metadata() {
        let peer = PeerId::random();

        let peers = basic_network(&PeerId::random());

        let expected_version = Some("1.2.4".to_string());

        {
            peers
                .add(&peer, PeerOrigin::IncomingConnection, vec![])
                .await
                .expect("should not fail on DB add");
            peers
                .update_from_ping(
                    &peer,
                    Ok(current_time().as_unix_timestamp().as_millis() as u64),
                    expected_version.clone(),
                )
                .await
                .expect("no error should occur");

            let status = peers.get_peer_status(&peer).unwrap();

            assert_eq!(status.peer_version, expected_version);
        }

        let ts = current_time().as_unix_timestamp().as_millis() as u64;

        {
            let expected_version = Some("2.0.0".to_string());

            peers
                .update_from_ping(&peer, Ok(ts), expected_version.clone())
                .await
                .expect("no error should occur");

            let status = peers.get_peer_status(&peer).expect("the peer status should be preent");

            assert_eq!(status.peer_version, expected_version);
        }
    }

    #[async_std::test]
    async fn test_network_should_ignore_a_peer_that_has_reached_lower_thresholds_a_specified_amount_of_time() {
        let peer = PeerId::random();

        let peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        peers
            .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
            .await
            .expect("no error should occur");
        peers
            .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
            .await
            .expect("no error should occur");
        peers
            .update_from_ping(&peer, Err(()), None)
            .await
            .expect("no error should occur"); // should drop to ignored
        peers
            .update_from_ping(&peer, Err(()), None)
            .await
            .expect("no error should occur"); // should drop from network

        assert!(!peers.has(&peer).await);

        // peer should remain ignored and not be added
        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        assert!(!peers.has(&peer).await)
    }

    #[async_std::test]
    async fn test_network_should_be_able_to_register_a_failed_heartbeat_result() {
        let peer = PeerId::random();
        let peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        peers
            .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
            .await
            .expect("no error should occur");
        peers
            .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
            .await
            .expect("no error should occur");
        peers
            .update_from_ping(&peer, Err(()), None)
            .await
            .expect("no error should occur");

        let actual = peers.get_peer_status(&peer).expect("the peer record should be preent");

        assert_eq!(actual.heartbeats_succeeded, 2);
        assert_eq!(actual.backoff, 300f64);
    }

    #[async_std::test]
    async fn test_network_should_be_listed_for_the_ping_if_last_recorded_later_than_reference() {
        let first = PeerId::random();
        let second = PeerId::random();
        let peers = basic_network(&PeerId::random());

        peers.add(&first, PeerOrigin::IncomingConnection, vec![]).await.unwrap();
        peers
            .add(&second, PeerOrigin::IncomingConnection, vec![])
            .await
            .unwrap();

        let ts = current_time().as_unix_timestamp().as_millis() as u64;

        let mut expected = vec![first, second];
        expected.sort();

        peers
            .update_from_ping(&first, Ok(ts), None)
            .await
            .expect("no error should occur");
        peers
            .update_from_ping(&second, Ok(ts), None)
            .await
            .expect("no error should occur");

        let mut actual = peers.find_peers_to_ping(ts + 3000);
        actual.sort();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_network_should_have_no_knowledge_about_health_without_any_registered_peers() {
        let peers = basic_network(&PeerId::random());

        assert_eq!(peers.health(), Health::Unknown);
    }

    #[async_std::test]
    async fn test_network_should_be_unhealthy_without_any_heartbeat_updates() {
        let peer = PeerId::random();

        let peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        assert_eq!(peers.health(), Health::Red);
    }

    #[async_std::test]
    async fn test_network_should_be_unhealthy_without_any_peers_once_the_health_was_known() {
        let peer = PeerId::random();

        let peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();
        let _ = peers.health();
        peers.remove(&peer).await.expect("should not fail on DB remove");

        assert_eq!(peers.health(), Health::Red);
    }

    #[async_std::test]
    async fn test_network_should_notify_the_callback_for_every_health_change() {
        let peer = PeerId::random();

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(1).returning(|_| false);
        let peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        assert_eq!(peers.health(), Health::Red);
    }

    #[async_std::test]
    async fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_low_quality() {
        let peer = PeerId::random();
        let public = peer;

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(2).returning(move |x| x == &public);
        let peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        peers
            .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
            .await
            .expect("no error should occur");

        assert_eq!(peers.health(), Health::Orange);
    }

    #[async_std::test]
    async fn test_network_should_remove_the_peer_once_it_reaches_the_lowest_possible_quality() {
        let peer = PeerId::random();
        let public = peer;

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(3).returning(move |x| x == &public);
        mock.expect_emit()
            .with(mockall::predicate::eq(NetworkEvent::CloseConnection(peer)))
            .return_const(());
        let peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        peers
            .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
            .await
            .expect("no error should occur");
        peers
            .update_from_ping(&peer, Err(()), None)
            .await
            .expect("no error should occur");

        assert!(!peers.has(&public).await);
    }

    #[async_std::test]
    async fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_i_am_public() {
        let me = PeerId::random();
        let peer = PeerId::random();
        let public = [peer, me];

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.3;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(5).returning(move |x| public.contains(x));
        let peers = Network::new(me, cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        for _ in 0..3 {
            peers
                .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
                .await
                .expect("no error should occur");
        }

        assert_eq!(peers.health(), Health::Green);
    }

    #[async_std::test]
    async fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_another_high_quality_non_public(
    ) {
        let peer = PeerId::random();
        let peer2 = PeerId::random();
        let public = [peer];

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.3;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(8).returning(move |x| public.contains(x));
        let peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection, vec![]).await.unwrap();
        peers.add(&peer2, PeerOrigin::IncomingConnection, vec![]).await.unwrap();

        for _ in 0..3 {
            peers
                .update_from_ping(&peer2, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
                .await
                .expect("no error should occur");
            peers
                .update_from_ping(&peer, Ok(current_time().as_unix_timestamp().as_millis() as u64), None)
                .await
                .expect("no error should occur");
        }

        assert_eq!(peers.health(), Health::Green);
    }
}
