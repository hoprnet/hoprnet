use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::time::Duration;

use libp2p::PeerId;

use utils_metrics::metrics::{MultiGauge, SimpleGauge};
#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;


/// Minimum delay will be multiplied by backoff, it will be half the actual minimum value
const MIN_DELAY: Duration = Duration::from_secs(1);
const MAX_DELAY: Duration = Duration::from_secs(300); // 5 minutes
const BACKOFF_EXPONENT: f64 = 1.5;
const MIN_BACKOFF: f64 = 2.0;
const MAX_BACKOFF: f64 = MAX_DELAY.as_millis() as f64 / MIN_DELAY.as_millis() as f64;
/// Default quality for unknown or offline nodes
const BAD_QUALITY: f64 = 0.2;
const IGNORE_TIMEFRAME: Duration = Duration::from_secs(600); // 10 minutes


// Does not work with enums
#[derive(Debug, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum NetworkPeerOrigin {
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

impl std::fmt::Display for NetworkPeerOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let description = match self {
            NetworkPeerOrigin::Initialization => "node initialization",
            NetworkPeerOrigin::NetworkRegistry => "registered in network registry",
            NetworkPeerOrigin::IncomingConnection => "incoming connection",
            NetworkPeerOrigin::OutgoingConnection => "outgoing connection attempt",
            NetworkPeerOrigin::StrategyExistingChannel => "strategy monitors existing channel",
            NetworkPeerOrigin::StrategyConsideringChannel => "strategy considers opening a channel",
            NetworkPeerOrigin::StrategyNewChannel => "strategy decided to open new channel",
            NetworkPeerOrigin::ManualPing => "manual ping",
            NetworkPeerOrigin::Testing => "testing",
        };
        write!(f, "{}", description)
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Entry {
    id: PeerId,
    origin: NetworkPeerOrigin,
    last_seen: u64,     // timestamp
    quality: f64,
    heartbeats_sent: u64,
    heartbeats_succeeded: u64,
    backoff: f64,
    ignored_at: Option<f32>,
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Entry: [id={}, origin={}, last seen on={}, quality={}, heartbeats sent={}, heartbeats succeeded={}, backoff={}, ignored at={:#?}]",
               self.id, self.origin, self.last_seen, self.quality, self.heartbeats_sent, self.heartbeats_succeeded, self.backoff, self.ignored_at)
    }
}

impl Entry {
    fn new(id: PeerId, origin: NetworkPeerOrigin) -> Entry {
        Entry {
            id,
            origin,
            heartbeats_sent: 0,
            heartbeats_succeeded: 0,
            last_seen: 0,
            backoff: MIN_BACKOFF,
            quality: 0.0,
            ignored_at: None,
        }
    }

    fn next_ping(&self) -> u64 {
        let backoff = self.backoff.powf(BACKOFF_EXPONENT);
        let delay = std::cmp::min(
            MAX_DELAY,
            Duration::from_millis((MIN_DELAY.as_millis() as f64 * backoff) as u64),
        );
        return self.last_seen + delay.as_millis() as u64;
    }
}

pub struct NetworkPeers {
    entries: HashMap<String, Entry>,
    ignored: HashMap<String, u64>,      // timestamp
    excluded: HashSet<String>,
    me: PeerId,
    network_quality_threshold: f64,
    good_quality: HashSet<PeerId>,
    bad_quality: HashSet<PeerId>,
    last_health: crate::heartbeat::NetworkHealthIndicator,
    on_peer_offline_cb: Box<dyn Fn(&PeerId)>,
    is_public_node_cb: Box<dyn Fn(&PeerId) -> bool>,
    metric_network_health: Option<SimpleGauge>,
    metric_peers_by_quality: Option<MultiGauge>,
    metric_peer_count: Option<SimpleGauge>
}

impl NetworkPeers {
    pub fn new(
        my_peer_id: PeerId,
        network_quality_threshold: f64,
        on_peer_offline: Box<dyn Fn(&PeerId)>,
        is_public_node: Box<dyn Fn(&PeerId) -> bool>
    ) -> NetworkPeers {
        if network_quality_threshold < BAD_QUALITY as f64 {
            panic!("Requested quality criteria are too low, expected: {network_quality_threshold}, minimum: {BAD_QUALITY}");
        }

        let mut excluded = HashSet::new();
        excluded.insert(my_peer_id.to_string());

        let mut instance = NetworkPeers {
            entries: HashMap::new(),
            ignored: HashMap::new(),
            excluded,
            me: my_peer_id,
            network_quality_threshold,
            good_quality: HashSet::new(),
            bad_quality: HashSet::new(),
            last_health: crate::heartbeat::NetworkHealthIndicator::UNKNOWN,
            on_peer_offline_cb: on_peer_offline,
            is_public_node_cb: is_public_node,
            metric_network_health: SimpleGauge::new(
                "core_gauge_network_health", "Connectivity health indicator").ok(),
            metric_peers_by_quality: MultiGauge::new(
                "core_mgauge_peers_by_quality",
                "Number different peer types by quality",
                &["type", "quality"]
            ).ok(),
            metric_peer_count: SimpleGauge::new("core_gauge_num_peers", "Number of all peers").ok()
        };

        instance
    }

    pub fn length(&self) -> usize {
        self.entries.len()
    }

    pub fn has(&self, peer: &PeerId) -> bool {
        self.entries.contains_key(peer.to_string().as_str())
    }

    fn rebalance_network_status(&mut self, entry: &Entry) {
        if entry.quality < self.network_quality_threshold {
            self.good_quality.remove(&entry.id);
            self.bad_quality.insert(entry.id.clone());
        } else {
            self.bad_quality.remove(&entry.id);
            self.good_quality.insert(entry.id.clone());
        }
    }

    fn prune_from_network_status(&mut self, peer: &PeerId) {
        self.good_quality.remove(&peer);
        self.bad_quality.remove(&peer);
    }

    pub fn register(&mut self, peer: &PeerId, origin: NetworkPeerOrigin) {
        let id = peer.to_string();
        let now = current_timestamp();

        // assumes disjoint sets
        let has_entry = self.entries.contains_key(id.as_str());
        let is_excluded = !has_entry && self.excluded.contains(id.as_str());

        if ! is_excluded {
            let is_ignored = if !has_entry && self.ignored.contains_key(id.as_str()) {
                let timestamp = self.ignored.get(id.as_str()).unwrap();
                if timestamp + (IGNORE_TIMEFRAME.as_millis() as u64) < now {
                    self.ignored.remove(id.as_str());
                    false
                } else {
                    true
                }
            } else {
                false
            };

            if ! has_entry && ! is_ignored {
                let entry = Entry::new(peer.clone(), origin);
                self.rebalance_network_status(&entry);

                if let Some(_x) = self.entries.insert(peer.to_string(), entry) {
                    unreachable!()     // evicting an existing record? This should not happen
                }
            }
        }
    }

    pub fn unregister(&mut self, peer: &PeerId) {
        self.prune_from_network_status(&peer);
        self.entries.remove(peer.to_string().as_str());
        // TODO: remove from ignored a nd excluded as well?
    }

    /// Returns the quality of the node
    ///
    /// # Arguments
    ///  id: The desired PeerId
    ///
    /// # Returns
    /// A float value from the range <0,1>, where:
    /// - 0.0 => completely unreliable / offline or unknown
    /// - 1.0 => completely reliable / online
    pub fn quality_of(&self, id: &PeerId) -> f64 {
        return match self.entries.get(id.to_string().as_str()) {
            Some(entry) => if entry.heartbeats_sent > 0 {entry.quality} else {BAD_QUALITY},
            None => 0.0,
        }
    }

    pub fn get_connection_info(&self, id: &PeerId) -> Option<Entry> {
        return match self.entries.get(id.to_string().as_str()) {
            Some(entry) => Some(entry.clone()),
            None => None
        }
    }

    pub fn ping_since(&self, threshold: u64) -> Vec<PeerId> {
        let mut data: Vec<PeerId> = Vec::new();
        for v in self.entries.values() {
            if v.next_ping() < threshold {
                data.push(v.id);
            }
        }
        data.sort_by(|a, b| {
            if self.entries.get(a.to_string().as_str()).unwrap().last_seen < self.entries.get(b.to_string().as_str()).unwrap().last_seen {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        data
    }

    pub fn update_record(&mut self, ping_result: crate::heartbeat::HeartbeatPingResult) {
        if let Some(existing) = self.entries.get(ping_result.destination.to_string().as_str()) {
            let mut entry = existing.clone();
            entry.last_seen = current_timestamp();
            entry.heartbeats_sent = entry.heartbeats_sent + 1;

            let ping_failed = ping_result.last_seen.is_none();

            if ping_failed {
                entry.backoff = MAX_BACKOFF.max(entry.backoff.powf(BACKOFF_EXPONENT));
                entry.quality = 0.0_f64.max(entry.quality - 0.1);

                if entry.quality < self.network_quality_threshold {
                    (self.on_peer_offline_cb)(&entry.id);
                    return
                }

                if entry.quality < BAD_QUALITY {
                    self.ignored.insert(entry.id.to_string(), current_timestamp());
                    self.entries.remove(entry.id.to_string().as_str());
                    self.prune_from_network_status(&entry.id);
                    return
                }
            } else {
                entry.heartbeats_succeeded = entry.heartbeats_succeeded + 1;
                entry.backoff = MIN_BACKOFF;
                entry.quality = 1.0_f64.min(entry.quality + 0.1)
            }

            self.rebalance_network_status(&entry);
            self.entries.insert(entry.id.to_string(), entry);
        }
    }

    /// Returns the quality of the network information as a tuple of (good, bad) vectors containing
    /// data.
    pub fn health(&mut self) -> crate::heartbeat::NetworkHealthIndicator {
        if self.entries.len() == 0 {
            return self.last_health;
        }
        let mut health = crate::heartbeat::NetworkHealthIndicator::RED;

        let good_public = self.good_quality.iter().filter(|&x| { (*self.is_public_node_cb)(&x) }).count();
        let good_non_public = self.good_quality.len() - good_public;
        let bad_public = self.bad_quality.iter().filter(|&x| { (*self.is_public_node_cb)(&x) }).count();
        let bad_non_public = self.bad_quality.len() - bad_public;

        if bad_public > 0 {
            health = crate::heartbeat::NetworkHealthIndicator::ORANGE;
        }

        if good_public > 0 {
            health = if good_non_public > 0 || (*self.is_public_node_cb)(&self.me) {
                crate::heartbeat::NetworkHealthIndicator::GREEN
            } else {
                crate::heartbeat::NetworkHealthIndicator::YELLOW
            };
        }

        // metrics

        if health != self.last_health {
            // (*self.on_network_health_change(self.last_health, health));  // TODO: emit
            self.last_health = health;
        }


        if let Some(metric_peer_count) = &self.metric_peer_count {
            metric_peer_count.set((good_public + good_non_public + bad_public + bad_non_public) as f64);
        }

        if let Some(metric_peers_by_quality) = &self.metric_peers_by_quality {
            metric_peers_by_quality.set(&["public", "high"], good_public as f64);
            metric_peers_by_quality.set(&["public", "low"], bad_public as f64);
            metric_peers_by_quality.set(&["nonPublic", "high"], good_non_public as f64);
            metric_peers_by_quality.set(&["nonPublic", "low"], bad_non_public as f64);
        }

        if let Some(metric_network_health) = &self.metric_network_health {
            metric_network_health.set((health as i32).into());
        }

        // TODO: add logs, what framework?

        health
    }

    pub fn debug_output(&self) -> String {
        let mut output = "".to_string();

        for (_, entry) in &self.entries {
            output.push_str(format!("{}\n", entry).as_str());
        }

        output
    }
}


#[cfg(test)]
mod tests {
    use crate::heartbeat;
    use super::*;

    #[test]
    fn test_network_health_should_be_ordered_numerically_for_metrics_output() {
        assert_eq!(crate::heartbeat::NetworkHealthIndicator::UNKNOWN as i32, 0);
        assert_eq!(crate::heartbeat::NetworkHealthIndicator::RED as i32, 1);
        assert_eq!(crate::heartbeat::NetworkHealthIndicator::ORANGE as i32, 2);
        assert_eq!(crate::heartbeat::NetworkHealthIndicator::YELLOW as i32, 3);
        assert_eq!(crate::heartbeat::NetworkHealthIndicator::GREEN as i32, 4);
    }

    #[test]
    fn test_entry_should_advance_time_on_ping() {
        let entry = Entry::new(PeerId::random(), NetworkPeerOrigin::ManualPing);

        assert_eq!(entry.next_ping(), 2828);
    }

    #[test]
    fn test_peers_should_not_contain_the_self_reference() {
        let me = PeerId::random();

        let mut peers = NetworkPeers::new(
            me.clone(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&me, NetworkPeerOrigin::IncomingConnection);

        assert_eq!(0, peers.length());
        assert!(! peers.has(&me))
    }

    #[test]
    fn test_peers_should_contain_a_registered_peer() {
        let expected = PeerId::random();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&expected, NetworkPeerOrigin::IncomingConnection);

        assert_eq!(1, peers.length());
        assert!(peers.has(&expected))
    }

    #[test]
    fn test_peers_should_remove_a_peer_on_unregistration() {
        let peer = PeerId::random();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);

        peers.unregister(&peer);

        assert_eq!(0, peers.length());
        assert!(! peers.has(&peer))
    }

    #[test]
    fn test_peers_should_ingore_heartbeat_updates_for_peers_that_were_not_registered() {
        let peer = PeerId::random();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: peer.clone(),
            last_seen: Some(utils_misc::time::current_timestamp())
        });

        assert_eq!(0, peers.length());
        assert!(! peers.has(&peer))
    }

    #[test]
    fn test_peers_should_be_able_to_register_a_succeeded_heartbeat_result() {
        let peer = PeerId::random();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);

        let ts = current_timestamp();

        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: peer.clone(),
            last_seen: Some(ts.clone())
        });

        let actual = peers.debug_output();

        assert!(actual.contains("heartbeats sent=1"));
        assert!(actual.contains("heartbeats succeeded=1"));
        assert!(actual.contains(format!("last seen on={}", ts).as_str()))
    }

    #[test]
    fn test_peers_should_be_able_to_register_a_failed_heartbeat_result() {
        let peer = PeerId::random();
        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);

        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: peer.clone(),
            last_seen: None
        });

        let actual = peers.debug_output();

        assert!(actual.contains("last seen on=0"));
        assert!(actual.contains("heartbeats succeeded=0"));
        assert!(actual.contains("backoff=2"));
    }

    #[test]
    fn test_peers_should_be_listed_for_the_ping_if_last_recorded_later_than_reference() {
        let first = PeerId::random();
        let second = PeerId::random();

        let mut expected = vec!(first, second);

        let mut peers = NetworkPeers::new(
            expected.clone(),
            vec!(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&first, NetworkPeerOrigin::IncomingConnection);
        peers.register(&second, NetworkPeerOrigin::IncomingConnection);

        let ts = current_timestamp();

        let mut expected = vec!(first, second);
        expected.sort();

        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: first.clone(),
            last_seen: Some(ts)
        });
        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: second.clone(),
            last_seen: Some(ts)
        });

        let mut actual = peers.ping_since(ts + 3000);

        expected.sort();
        actual.sort();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_peers_should_have_no_knowledge_about_health_without_any_registered_peers() {
        let peer = PeerId::random();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.3,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        assert_eq!(peers.health(), crate::heartbeat::NetworkHealthIndicator::UNKNOWN);
    }

    #[test]
    fn test_peers_should_be_unhealthy_without_any_heartbeat_updates() {
        let peer = PeerId::random();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);

        assert_eq!(peers.health(), crate::heartbeat::NetworkHealthIndicator::RED);
    }

    #[test]
    fn test_peers_should_be_unhealthy_without_any_peers_once_the_health_was_known() {
        let peer = PeerId::random();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(|x| -> bool { false } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);
        let _ = peers.health();
        peers.unregister(&peer);

        assert_eq!(peers.health(), crate::heartbeat::NetworkHealthIndicator::RED);
    }

    #[test]
    fn test_peers_should_be_healthy_when_a_public_peer_is_pingable_with_low_quality() {
        let peer = PeerId::random();
        let public = peer.clone();

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.6,
            Box::new(|x| { () }),
            Box::new(move |x| -> bool { x == &public } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);

        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: peer.clone(),
            last_seen: Some(current_timestamp())
        });

        assert_eq!(peers.health(), crate::heartbeat::NetworkHealthIndicator::ORANGE);
    }

    #[test]
    fn test_peers_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_I_am_public() {
        let me = PeerId::random();
        let peer = PeerId::random();
        let public = vec![peer.clone(), me.clone()];

        let mut peers = NetworkPeers::new(
            me,
            0.3,
            Box::new(|x| { () }),
            Box::new(move |x| -> bool { public.contains(&x) } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);

        for _ in 0..3 {
            peers.update_record(heartbeat::HeartbeatPingResult {
                destination: peer.clone(),
                last_seen: Some(current_timestamp())
            });
        }

        assert_eq!(peers.health(), crate::heartbeat::NetworkHealthIndicator::GREEN);
    }

    #[test]
    fn test_peers_should_be_healthy_when_a_public_peer_is_pingable_with_high_quality_and_another_high_quality_non_public() {
        let peer = PeerId::random();
        let peer2 = PeerId::random();
        let public = vec![peer.clone()];

        let mut peers = NetworkPeers::new(
            PeerId::random(),
            0.3,
            Box::new(|x| { () }),
            Box::new(move |x| -> bool { public.contains(&x) } ));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);
        peers.register(&peer2, NetworkPeerOrigin::IncomingConnection);

        for _ in 0..3 {
            peers.update_record(heartbeat::HeartbeatPingResult {
                destination: peer2.clone(),
                last_seen: Some(current_timestamp())
            });
            peers.update_record(heartbeat::HeartbeatPingResult {
                destination: peer.clone(),
                last_seen: Some(current_timestamp())
            });
        }

        assert_eq!(peers.health(), crate::heartbeat::NetworkHealthIndicator::GREEN);
    }
}
