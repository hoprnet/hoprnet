use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::time::Duration;

use libp2p::PeerId;

use utils_misc::time::current_timestamp;


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

struct NetworkPeers {
    entries: HashMap<String, Entry>,
    ignored: HashMap<String, u64>,      // timestamp
    excluded: HashSet<String>,
    network_quality_threshold: f64,
    on_peer_offline: Box<dyn Fn(&PeerId)>
}

impl NetworkPeers {
    pub fn new(
        existing: Vec<PeerId>,
        excluded: Vec<PeerId>,
        network_quality_threshold: f64,
        on_peer_offline: Box<dyn Fn(&PeerId)>
    ) -> NetworkPeers {
        if network_quality_threshold < BAD_QUALITY as f64 {
            panic!("Requested quality criteria are too low, expected: {network_quality_threshold}, minimum: {BAD_QUALITY}");
        }

        let mut instance = NetworkPeers {
            entries: HashMap::new(),
            ignored: HashMap::new(),
            excluded: HashSet::from_iter(excluded.iter().map(|x| x.to_string())),
            network_quality_threshold,
            on_peer_offline
        };

        for id in existing.iter() {
            instance.register(id, NetworkPeerOrigin::Initialization);
        }

        instance
    }

    pub fn length(&self) -> usize {
        self.entries.len()
    }

    pub fn has(&self, peer: &PeerId) -> bool {
        self.entries.contains_key(peer.to_string().as_str())
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
                if let Some(_x) = self.entries.insert(peer.to_string(),Entry::new(peer.clone(), origin)) {
                    unreachable!()     // evicting an existing record? This should not happen
                }
            }
        }
    }

    pub fn unregister(&mut self, peer: &PeerId) {
        self.entries.remove(peer.to_string().as_str());
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
                    (self.on_peer_offline)(&entry.id);
                    return
                }

                if entry.quality < BAD_QUALITY {
                    self.ignored.insert(entry.id.to_string(), current_timestamp());
                    self.entries.remove(entry.id.to_string().as_str());
                    return
                }
            } else {
                entry.heartbeats_succeeded = entry.heartbeats_succeeded + 1;
                entry.backoff = MIN_BACKOFF;
                entry.quality = 1.0_f64.min(entry.quality + 0.1)
            }

            self.entries.insert(entry.id.to_string(), entry);
        }
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
    fn test_entry_should_advance_time_on_ping() {
        let entry = Entry::new(PeerId::random(), NetworkPeerOrigin::ManualPing);

        assert_eq!(entry.next_ping(), 2828);
    }

    #[test]
    fn test_peers_should_not_contain_the_excluded_peers() {
        let excluded = PeerId::random();

        let mut peers = NetworkPeers::new(
            vec!(PeerId::random(), excluded.clone()),
            vec!(excluded.clone()),
            0.6, Box::new(|x| { () }));

        peers.register(&excluded, NetworkPeerOrigin::IncomingConnection);

        assert_eq!(1, peers.length());
        assert!(! peers.has(&excluded))
    }

    #[test]
    fn test_peers_should_be_able_to_register_a_succeeded_heartbeat_result() {
        let peer = PeerId::random();
        let mut peers = NetworkPeers::new(
            vec!(peer.clone()),
            vec!(),
            0.6, Box::new(|x| { () }));

        let ts = utils_misc::time::current_timestamp();

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
            vec!(peer.clone()),
            vec!(),
            0.6, Box::new(|x| { () }));

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
    fn test_peers_should_be_listed_for_the_ping_since_if_the_were_recorded_later_than_reference() {
        let first = PeerId::random();
        let second = PeerId::random();
        let mut peers = NetworkPeers::new(
            vec!(first.clone(), second.clone()),
            vec!(),
            0.6, Box::new(|x| { () }));

        let ts = utils_misc::time::current_timestamp();

        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: first.clone(),
            last_seen: Some(ts)
        });
        peers.update_record(heartbeat::HeartbeatPingResult{
            destination: second.clone(),
            last_seen: Some(ts)
        });

        assert_eq!(peers.ping_since(ts + 3000), vec!(first, second));
    }

    #[test]
    fn test_peers_unregistered_peers_should_be_removed_from_the_list() {
        let peer = PeerId::random();
        let mut peers = NetworkPeers::new(
            vec!(),
            vec!(),
            0.6, Box::new(|x| { () }));

        peers.register(&peer, NetworkPeerOrigin::IncomingConnection);
        peers.unregister(&peer);

        assert_eq!(peers.length(), 0)
    }
}
