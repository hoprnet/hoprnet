use std::time::Duration;

use libp2p::PeerId;

// move to misc-utils crate?
#[derive(Debug,Default, Copy, Clone, PartialEq, Eq)]
struct Timestamp {
    value: u64
}

impl std::ops::Deref for Timestamp {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(not(wasm))]
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 1,
    }
}

#[cfg(wasm)]
fn current_timestamp() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}


/// Minimum delay will be multiplied by backoff, it will be half the actual minimum value
const MIN_DELAY: Duration = Duration::from_secs(1);
const MAX_DELAY: Duration = Duration::from_secs(300); // 5 minutes
const BACKOFF_EXPONENT: f64 = 1.5;
const MAX_BACKOFF: f64 = MAX_DELAY.as_millis() as f64 / MIN_DELAY.as_millis() as f64;
/// Default quality for unknown or offline nodes
const BAD_QUALITY: f64 = 0.2;
const IGNORE_TIMEFRAME: Duration = Duration::from_secs(600); // 10 minutes

// Does not work with enums
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)
#[derive(Debug, Clone)]
enum NetworkPeerOrigin {
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
    heartbeats_sent: u64,
    heartbeats_succeeded: u64,
    last_seen: u64, // timestamp?
    backoff: f64,
    quality: f64,
    origin: NetworkPeerOrigin,
    ignored_at: Option<f32>,
}

impl Entry {
    fn new(id: PeerId, origin: NetworkPeerOrigin) -> Entry {
        Entry {
            id,
            origin,
            heartbeats_sent: 0,
            heartbeats_succeeded: 0,
            last_seen: 0,
            backoff: 1.0,
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

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;

struct NetworkPeers {
    entries: HashMap<String, Entry>,
    ignored: HashMap<String, u64>,      // timestamp
    excluded: HashSet<String>,
    network_quality_threshold: f64,
    on_peer_offline: Box<dyn Fn(&PeerId)>
}

impl NetworkPeers {
    // TODO: vec not needed
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
                    todo!()     // evicting an existing record? This should not happen
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
                entry.backoff = 2.0;
                entry.quality = 1.0_f64.min(entry.quality + 0.1)
            }

            self.entries.insert(entry.id.to_string(), entry);
        }
    }
}


#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_foo() {
        assert_eq!(1, 2 - 1);
    }
}
