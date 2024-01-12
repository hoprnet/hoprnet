use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::collections::VecDeque;
use std::time::Duration;

use libp2p_identity::PeerId;

use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::Validate;

use crate::constants::DEFAULT_NETWORK_QUALITY_THRESHOLD;
use log::{info, warn};
use utils_types::sma::{SingleSumSMA, SMA};

#[cfg(all(feature = "prometheus", not(test)))]
use {
    metrics::metrics::{MultiGauge, SimpleGauge, SimpleHistogram},
    platform::time::native::current_timestamp,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_NETWORK_HEALTH: SimpleGauge =
        SimpleGauge::new("core_gauge_network_health", "Connectivity health indicator").unwrap();
    static ref METRIC_PEERS_BY_QUALITY: MultiGauge =
        MultiGauge::new("core_mgauge_peers_by_quality", "Number different peer types by quality",
            &["type", "quality"],
        ).unwrap();
    static ref METRIC_PEER_COUNT: SimpleGauge =
        SimpleGauge::new("core_gauge_num_peers", "Number of all peers").unwrap();
    static ref METRIC_NETWORK_HEALTH_TIME_TO_GREEN: SimpleHistogram = SimpleHistogram::new(
        "hoprd_histogram_time_to_green_seconds",
        "Time it takes for a node to transition to the GREEN network state",
        vec![30.0, 60.0, 90.0, 120.0, 180.0, 240.0, 300.0, 420.0, 600.0, 900.0, 1200.0]
    ).unwrap();
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct NetworkConfig {
    /// Minimum delay will be multiplied by backoff, it will be half the actual minimum value
    #[serde_as(as = "DurationSeconds<u64>")]
    min_delay: Duration,

    /// Maximum delay
    #[serde_as(as = "DurationSeconds<u64>")]
    max_delay: Duration,

    #[validate(range(min = 0.0, max = 1.0))]
    quality_bad_threshold: f64,

    #[validate(range(min = 0.0, max = 1.0))]
    pub quality_offline_threshold: f64,
    quality_step: f64,

    /// Size of the window for quality moving average
    #[validate(range(min = 1_u32))]
    pub quality_avg_window_size: u32,

    #[serde_as(as = "DurationSeconds<u64>")]
    ignore_timeframe: Duration,

    backoff_exponent: f64,

    backoff_min: f64,

    backoff_max: f64,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PeerOrigin {
    Initialization = 0,
    NetworkRegistry = 1,
    IncomingConnection = 2,
    OutgoingConnection = 3,
    StrategyExistingChannel = 4,
    StrategyConsideringChannel = 5,
    StrategyNewChannel = 6,
    ManualPing = 7,
    Testing = 8,
}

impl std::fmt::Display for PeerOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let description = match self {
            PeerOrigin::Initialization => "node initialization",
            PeerOrigin::NetworkRegistry => "network registry",
            PeerOrigin::IncomingConnection => "incoming connection",
            PeerOrigin::OutgoingConnection => "outgoing connection attempt",
            PeerOrigin::StrategyExistingChannel => "strategy monitors existing channel",
            PeerOrigin::StrategyConsideringChannel => "strategy considers opening a channel",
            PeerOrigin::StrategyNewChannel => "strategy decided to open new channel",
            PeerOrigin::ManualPing => "manual ping",
            PeerOrigin::Testing => "testing",
        };
        write!(f, "{}", description)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl std::fmt::Display for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for Health {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Red" => Ok(Self::Red),
            "Orange" => Ok(Self::Orange),
            "Yellow" => Ok(Self::Yellow),
            "Green" => Ok(Self::Green),
            "Unknown" => Ok(Self::Unknown),
            _ => Err(format!("Unsupported Health string '{s}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEvent {
    CloseConnection(PeerId),
}

impl std::fmt::Display for NetworkEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait NetworkExternalActions {
    fn is_public(&self, peer: &PeerId) -> bool;

    fn emit(&self, event: NetworkEvent);

    fn create_timestamp(&self) -> u64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeerStatus {
    id: PeerId,
    pub origin: PeerOrigin,
    pub is_public: bool,
    pub last_seen: u64,         // timestamp
    pub last_seen_latency: u64, // duration in ms
    quality: f64,
    pub heartbeats_sent: u64,
    pub heartbeats_succeeded: u64,
    pub backoff: f64,
    quality_avg: SingleSumSMA<f64>,
    metadata: HashMap<String, String>,
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
            backoff,
            quality: 0.0,
            metadata: HashMap::new(),
            quality_avg: SingleSumSMA::new(quality_window),
        }
    }

    pub fn update_quality(&mut self, new_value: f64) {
        self.quality = new_value;
        self.quality_avg.push(new_value);
    }

    /// Gets metadata associated with the peer
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
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

#[derive(Debug)]
pub struct Network<T: NetworkExternalActions> {
    me: PeerId,
    cfg: NetworkConfig,
    events_to_emit: VecDeque<NetworkEvent>,
    entries: HashMap<PeerId, PeerStatus>,
    ignored: HashMap<PeerId, u64>, // timestamp
    excluded: HashSet<PeerId>,
    known_multiaddresses: HashMap<PeerId, Vec<Multiaddr>>,
    good_quality_public: HashSet<PeerId>,
    bad_quality_public: HashSet<PeerId>,
    good_quality_non_public: HashSet<PeerId>,
    bad_quality_non_public: HashSet<PeerId>,
    last_health: Health,
    network_actions_api: T,
    #[cfg(all(feature = "prometheus", not(test)))]
    started_at: Option<std::time::Duration>,
}

impl<T: NetworkExternalActions> Network<T> {
    pub fn new(my_peer_id: PeerId, cfg: NetworkConfig, network_actions_api: T) -> Self {
        if cfg.quality_offline_threshold < cfg.quality_bad_threshold {
            panic!(
                "Strict requirement failed, bad quality threshold {} must be lower than quality offline threshold {}",
                cfg.quality_bad_threshold, cfg.quality_offline_threshold
            );
        }

        let mut excluded = HashSet::new();
        excluded.insert(my_peer_id);

        Network {
            me: my_peer_id,
            cfg,
            events_to_emit: VecDeque::new(),
            entries: HashMap::new(),
            ignored: HashMap::new(),
            excluded,
            known_multiaddresses: HashMap::new(),
            good_quality_public: HashSet::new(),
            bad_quality_public: HashSet::new(),
            good_quality_non_public: HashSet::new(),
            bad_quality_non_public: HashSet::new(),
            last_health: Health::Unknown,
            network_actions_api,
            #[cfg(all(feature = "prometheus", not(test)))]
            started_at: Some(current_timestamp()),
        }
    }

    /// Set all registered multiaddresses for a specific peer
    pub fn store_peer_multiaddresses(&mut self, peer: &PeerId, addrs: Vec<Multiaddr>) {
        self.known_multiaddresses.insert(*peer, addrs);
    }

    /// Get all registered multiaddresses for a specific peer
    pub fn get_peer_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
        match self.known_multiaddresses.get(peer) {
            Some(addrs) => addrs.clone(),
            None => vec![],
        }
    }

    /// Check whether the PeerId is present in the network
    pub fn has(&self, peer: &PeerId) -> bool {
        self.entries.contains_key(peer)
    }

    /// Add a new PeerId into the network
    ///
    /// Each PeerId must have an origin specification.
    pub fn add(&mut self, peer: &PeerId, origin: PeerOrigin) {
        self.add_with_metadata(peer, origin, None)
    }

    /// Add a new PeerId into the network (with optional metadata entries)
    ///
    /// Each PeerId must have an origin specification.
    pub fn add_with_metadata(&mut self, peer: &PeerId, origin: PeerOrigin, metadata: Option<HashMap<String, String>>) {
        let now = self.network_actions_api.create_timestamp();
        log::debug!("Registering peer '{}' with origin {}", peer, origin);

        // assumes disjoint sets
        let has_entry = self.entries.contains_key(peer);
        let is_excluded = !has_entry && self.excluded.contains(peer);

        if !is_excluded {
            let is_ignored = if !has_entry && self.ignored.contains_key(peer) {
                let timestamp = self.ignored.get(peer).unwrap();
                if Duration::from_millis(now - timestamp) > self.cfg.ignore_timeframe {
                    self.ignored.remove(peer);
                    false
                } else {
                    true
                }
            } else {
                false
            };

            if !has_entry && !is_ignored {
                let mut entry = PeerStatus::new(*peer, origin, self.cfg.backoff_min, self.cfg.quality_avg_window_size);
                entry.is_public = self.network_actions_api.is_public(peer);
                if let Some(m) = metadata {
                    entry.metadata.extend(m);
                }
                self.refresh_network_status(&entry);

                if let Some(x) = self.entries.insert(*peer, entry) {
                    warn!("Evicting an existing record for {}, this should not happen!", &x);
                }
            }
        }
    }

    /// Remove PeerId from the network
    pub fn remove(&mut self, peer: &PeerId) {
        self.prune_from_network_status(peer);
        self.entries.remove(peer);
    }

    /// Update the PeerId record in the network
    pub fn update(&mut self, peer: &PeerId, ping_result: crate::types::Result) {
        self.update_with_metadata(peer, ping_result, None);
    }

    /// Update the PeerId record in the network (with optional metadata entries that will be merged into the existing ones)
    pub fn update_with_metadata(
        &mut self,
        peer: &PeerId,
        ping_result: crate::types::Result,
        metadata: Option<HashMap<String, String>>,
    ) -> Option<PeerStatus> {
        if let Some(existing) = self.entries.get(peer) {
            let mut entry = existing.clone();
            entry.heartbeats_sent += 1;
            entry.is_public = self.network_actions_api.is_public(peer);

            // Upsert metadata if any
            if let Some(mm) = metadata {
                mm.into_iter().for_each(|(k, v)| match entry.metadata.entry(k) {
                    Entry::Occupied(val) => {
                        *val.into_mut() = v.clone();
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(v);
                    }
                });
            }

            if let Ok(latency) = ping_result {
                entry.last_seen = self.network_actions_api.create_timestamp();
                entry.last_seen_latency = latency;
                entry.heartbeats_succeeded += 1;
                entry.backoff = self.cfg.backoff_min;
                entry.update_quality(1.0_f64.min(entry.quality + self.cfg.quality_step));
            } else {
                entry.backoff = self.cfg.backoff_max.max(entry.backoff.powf(self.cfg.backoff_exponent));
                entry.update_quality(0.0_f64.max(entry.quality - self.cfg.quality_step));

                if entry.quality < (self.cfg.quality_step / 2.0) {
                    self.network_actions_api.emit(NetworkEvent::CloseConnection(entry.id));
                    self.prune_from_network_status(&entry.id);
                    self.entries.remove(&entry.id);
                    return Some(entry);
                } else if entry.quality < self.cfg.quality_bad_threshold {
                    self.ignored
                        .insert(entry.id, self.network_actions_api.create_timestamp());
                }
            }

            self.refresh_network_status(&entry);
            self.entries.insert(entry.id, entry.clone());

            Some(entry)
        } else {
            info!("Ignoring update request for unknown peer {}", peer);
            None
        }
    }

    /// Update the internally perceived network status that is processed to the network health
    fn refresh_network_status(&mut self, entry: &PeerStatus) {
        self.prune_from_network_status(&entry.id);

        if entry.quality < self.cfg.quality_offline_threshold {
            if entry.is_public {
                self.bad_quality_public.insert(entry.id);
            } else {
                self.bad_quality_non_public.insert(entry.id);
            }
        } else if entry.is_public {
            self.good_quality_public.insert(entry.id);
        } else {
            self.good_quality_non_public.insert(entry.id);
        }

        let good_public = self.good_quality_public.len();
        let good_non_public = self.good_quality_non_public.len();
        let bad_public = self.bad_quality_public.len();

        #[cfg(all(feature = "prometheus", not(test)))]
        let bad_non_public = self.bad_quality_non_public.len();

        let mut health = Health::Red;

        if bad_public > 0 {
            health = Health::Orange;
        }

        if good_public > 0 {
            health = if good_non_public > 0 || self.network_actions_api.is_public(&self.me) {
                Health::Green
            } else {
                Health::Yellow
            };
        }

        if health != self.last_health {
            info!("Network health indicator changed {} -> {}", self.last_health, health);
            info!("NETWORK HEALTH: {}", health);

            #[cfg(all(feature = "prometheus", not(test)))]
            if self.started_at.is_some() {
                if let Some(ts) = current_timestamp().checked_sub(self.started_at.take().unwrap()) {
                    METRIC_NETWORK_HEALTH_TIME_TO_GREEN.observe(ts.as_secs() as f64);
                }
            }

            self.last_health = health;
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_PEER_COUNT.set((good_public + good_non_public + bad_public + bad_non_public) as f64);
            METRIC_PEERS_BY_QUALITY.set(&["public", "high"], good_public as f64);
            METRIC_PEERS_BY_QUALITY.set(&["public", "low"], bad_public as f64);
            METRIC_PEERS_BY_QUALITY.set(&["nonPublic", "high"], good_non_public as f64);
            METRIC_PEERS_BY_QUALITY.set(&["nonPublic", "low"], bad_non_public as f64);
            METRIC_NETWORK_HEALTH.set((health as i32).into());
        }
    }

    /// Remove the PeerId from network status observed variables
    fn prune_from_network_status(&mut self, peer: &PeerId) {
        self.good_quality_public.remove(peer);
        self.good_quality_non_public.remove(peer);
        self.good_quality_public.remove(peer);
        self.bad_quality_non_public.remove(peer);
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
    pub fn filter<F>(&self, f: F) -> Vec<PeerId>
    where
        F: FnMut(&&PeerStatus) -> bool,
    {
        self.entries.values().filter(f).map(|x| x.id).collect::<Vec<_>>()
    }

    pub fn all_peers_with_quality(&self) -> Vec<(PeerId, f64)> {
        self.entries
            .values()
            .map(|status: &PeerStatus| (status.id, status.get_quality()))
            .collect::<Vec<_>>()
    }

    pub fn all_peers_with_avg_quality(&self) -> Vec<(PeerId, f64)> {
        self.entries
            .values()
            .map(|status: &PeerStatus| (status.id, status.get_average_quality()))
            .collect::<Vec<_>>()
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

    pub fn events_since_last_poll(&mut self) -> Vec<NetworkEvent> {
        self.events_to_emit.drain(..).collect()
    }

    /// Total count of the peers observed withing the network
    pub fn length(&self) -> usize {
        self.entries.len()
    }

    /// Returns the quality of the network as a network health indicator.
    pub fn health(&self) -> Health {
        self.last_health
    }

    pub fn debug_output(&self) -> String {
        let mut output = "".to_string();

        for entry in self.entries.values() {
            output.push_str(format!("{}\n", entry).as_str());
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use crate::network::{
        Health, MockNetworkExternalActions, Network, NetworkConfig, NetworkEvent, NetworkExternalActions, PeerOrigin,
    };
    use libp2p_identity::PeerId;
    use platform::time::native::current_timestamp;

    struct DummyNetworkAction {}

    impl NetworkExternalActions for DummyNetworkAction {
        fn is_public(&self, _: &PeerId) -> bool {
            false
        }

        fn emit(&self, _: NetworkEvent) {}

        fn create_timestamp(&self) -> u64 {
            current_timestamp().as_millis() as u64
        }
    }

    fn basic_network(my_id: &PeerId) -> Network<DummyNetworkAction> {
        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;
        Network::new(my_id.clone(), cfg, DummyNetworkAction {})
    }

    #[test]
    fn test_network_health_should_be_ordered_numerically_for_metrics_output() {
        assert_eq!(Health::Unknown as i32, 0);
        assert_eq!(Health::Red as i32, 1);
        assert_eq!(Health::Orange as i32, 2);
        assert_eq!(Health::Yellow as i32, 3);
        assert_eq!(Health::Green as i32, 4);
    }

    #[test]
    fn test_network_should_not_contain_the_self_reference() {
        let me = PeerId::random();

        let mut peers = basic_network(&me);

        peers.add(&me, PeerOrigin::IncomingConnection);

        assert_eq!(0, peers.length());
        assert!(!peers.has(&me))
    }

    #[test]
    fn test_network_should_contain_a_registered_peer() {
        let expected = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        peers.add(&expected, PeerOrigin::IncomingConnection);

        assert_eq!(1, peers.length());
        assert!(peers.has(&expected))
    }

    #[test]
    fn test_network_should_add_metadata() {
        let expected = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        let proto_version = ("protocol_version".to_string(), "1.2.3".to_string());

        peers.add_with_metadata(
            &expected,
            PeerOrigin::IncomingConnection,
            Some([proto_version.clone()].into()),
        );

        assert_eq!(1, peers.length());
        assert!(peers.has(&expected));

        let status = peers.get_peer_status(&expected).unwrap();
        assert!(status.metadata().contains_key(&proto_version.0));
        assert_eq!(&proto_version.1, status.metadata().get(&proto_version.0).unwrap());
    }

    #[test]
    fn test_network_should_remove_a_peer_on_unregistration() {
        let peer = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection);

        peers.remove(&peer);

        assert_eq!(0, peers.length());
        assert!(!peers.has(&peer))
    }

    #[test]
    fn test_network_should_ingore_heartbeat_updates_for_peers_that_were_not_registered() {
        let peer = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));

        assert_eq!(0, peers.length());
        assert!(!peers.has(&peer))
    }

    #[test]
    fn test_network_should_be_able_to_register_a_succeeded_heartbeat_result() {
        let peer = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection);

        let ts = peers.network_actions_api.create_timestamp();

        peers.update(&peer, Ok(ts.clone()));

        std::thread::sleep(std::time::Duration::from_millis(100));

        let actual = peers.debug_output();

        assert!(actual.contains("heartbeats sent=1"));
        assert!(actual.contains("heartbeats succeeded=1"));
        assert!(actual.contains(format!("last seen on={}", ts).as_str()))
    }

    #[test]
    fn test_network_update_should_merge_metadata() {
        let peer = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        let other_metadata_1 = ("other_1".to_string(), "efgh".to_string());
        let other_metadata_2 = ("other_2".to_string(), "abcd".to_string());

        {
            let proto_version = ("protocol_version".to_string(), "1.2.3".to_string());

            peers.add_with_metadata(
                &peer,
                PeerOrigin::IncomingConnection,
                Some([proto_version.clone(), other_metadata_1.clone()].into()),
            );

            let status = peers.get_peer_status(&peer).unwrap();

            assert_eq!(2, status.metadata().len());
            assert_eq!(&proto_version.1, status.metadata().get(&proto_version.0).unwrap());
            assert_eq!(&other_metadata_1.1, status.metadata().get(&other_metadata_1.0).unwrap());
            assert!(status.metadata().get(&other_metadata_2.0).is_none());
        }

        let ts = peers.network_actions_api.create_timestamp();

        {
            let proto_version = ("protocol_version".to_string(), "1.2.4".to_string());

            peers.update_with_metadata(
                &peer,
                Ok(ts.clone()),
                Some([proto_version.clone(), other_metadata_2.clone()].into()),
            );

            let status = peers.get_peer_status(&peer).unwrap();

            assert_eq!(3, status.metadata().len());
            assert_eq!(&proto_version.1, status.metadata().get(&proto_version.0).unwrap());
            assert_eq!(&other_metadata_1.1, status.metadata().get(&other_metadata_1.0).unwrap());
            assert_eq!(&other_metadata_2.1, status.metadata().get(&other_metadata_2.0).unwrap());
        }
    }

    #[test]
    fn test_network_should_ignore_a_peer_that_has_reached_lower_thresholds_a_specified_amount_of_time() {
        let peer = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection);

        peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));
        peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));
        peers.update(&peer, Err(())); // should drop to ignored
        peers.update(&peer, Err(())); // should drop from network

        assert!(!peers.has(&peer));

        // peer should remain ignored and not be added
        peers.add(&peer, PeerOrigin::IncomingConnection);

        assert!(!peers.has(&peer))
    }

    #[test]
    fn test_network_should_be_able_to_register_a_failed_heartbeat_result() {
        let peer = PeerId::random();
        let mut peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection);

        peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));
        peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));
        peers.update(&peer, Err(()));

        let actual = peers.debug_output();

        assert!(actual.contains("heartbeats succeeded=2"));
        assert!(actual.contains("backoff=300"));
    }

    #[test]
    fn test_network_should_be_listed_for_the_ping_if_last_recorded_later_than_reference() {
        let first = PeerId::random();
        let second = PeerId::random();
        let mut peers = basic_network(&PeerId::random());

        peers.add(&first, PeerOrigin::IncomingConnection);
        peers.add(&second, PeerOrigin::IncomingConnection);

        let ts = peers.network_actions_api.create_timestamp();

        let mut expected = vec![first, second];
        expected.sort();

        peers.update(&first, Ok(ts));
        peers.update(&second, Ok(ts));

        let mut actual = peers.find_peers_to_ping(ts + 3000);
        actual.sort();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_network_should_have_no_knowledge_about_health_without_any_registered_peers() {
        let peers = basic_network(&PeerId::random());

        assert_eq!(peers.health(), Health::Unknown);
    }

    #[test]
    fn test_network_should_be_unhealthy_without_any_heartbeat_updates() {
        let peer = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection);

        assert_eq!(peers.health(), Health::Red);
    }

    #[test]
    fn test_network_should_be_unhealthy_without_any_peers_once_the_health_was_known() {
        let peer = PeerId::random();

        let mut peers = basic_network(&PeerId::random());

        peers.add(&peer, PeerOrigin::IncomingConnection);
        let _ = peers.health();
        peers.remove(&peer);

        assert_eq!(peers.health(), Health::Red);
    }

    #[test]
    fn test_network_should_notify_the_callback_for_every_health_change() {
        let peer = PeerId::random();

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(1).returning(|_| false);
        mock.expect_create_timestamp()
            .returning(|| current_timestamp().as_millis() as u64);
        let mut peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection);

        assert_eq!(peers.health(), Health::Red);
    }

    #[test]
    fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_low_quality() {
        let peer = PeerId::random();
        let public = peer.clone();

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(2).returning(move |x| x == &public);
        mock.expect_create_timestamp()
            .returning(|| current_timestamp().as_millis() as u64);
        let mut peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection);

        peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));

        assert_eq!(peers.health(), Health::Orange);
    }

    #[test]
    fn test_network_should_remove_the_peer_once_it_reaches_the_lowest_possible_quality() {
        let peer = PeerId::random();
        let public = peer.clone();

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.6;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(3).returning(move |x| x == &public);
        mock.expect_emit()
            .with(mockall::predicate::eq(NetworkEvent::CloseConnection(peer.clone())))
            .return_const(());
        mock.expect_create_timestamp()
            .returning(|| current_timestamp().as_millis() as u64);
        let mut peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection);

        peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));
        peers.update(&peer, Err(()));

        assert!(!peers.has(&public));
    }

    #[test]
    fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_i_am_public() {
        let me = PeerId::random();
        let peer = PeerId::random();
        let public = vec![peer.clone(), me.clone()];

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.3;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(5).returning(move |x| public.contains(&x));
        mock.expect_create_timestamp()
            .returning(|| current_timestamp().as_millis() as u64);
        let mut peers = Network::new(me, cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection);

        for _ in 0..3 {
            peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));
        }

        assert_eq!(peers.health(), Health::Green);
    }

    #[test]
    fn test_network_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_another_high_quality_non_public(
    ) {
        let peer = PeerId::random();
        let peer2 = PeerId::random();
        let public = vec![peer.clone()];

        let mut cfg = NetworkConfig::default();
        cfg.quality_offline_threshold = 0.3;

        let mut mock = MockNetworkExternalActions::new();
        mock.expect_is_public().times(8).returning(move |x| public.contains(&x));
        mock.expect_create_timestamp()
            .returning(|| current_timestamp().as_millis() as u64);
        let mut peers = Network::new(PeerId::random(), cfg, mock);

        peers.add(&peer, PeerOrigin::IncomingConnection);
        peers.add(&peer2, PeerOrigin::IncomingConnection);

        for _ in 0..3 {
            peers.update(&peer2, Ok(peers.network_actions_api.create_timestamp()));
            peers.update(&peer, Ok(peers.network_actions_api.create_timestamp()));
        }

        assert_eq!(peers.health(), Health::Green);
    }
}
