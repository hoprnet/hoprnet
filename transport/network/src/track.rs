use std::sync::Arc;

use hopr_api::PeerId;

// TODO: make this a streamable telemetry instead:
// static ref METRIC_PEERS_BY_QUALITY:  hopr_metrics::MultiGauge =
//      hopr_metrics::MultiGauge::new("hopr_peers_by_quality", "Number different peer types by quality",
//         &["type", "quality"],
//     ).unwrap();

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct ExponentialMovingAverage<const FACTOR: usize> {
    count: usize,
    average: f64,
}

impl<const FACTOR: usize> ExponentialMovingAverage<FACTOR> {
    pub fn new() -> Self {
        Self { count: 0, average: 0.0 }
    }

    pub fn update(&mut self, value: u128) {
        self.count += 1;
        self.average = self.average + (value as f64 - self.average) / (std::cmp::min(self.count, FACTOR) as f64);
    }

    pub fn get(&self) -> u128 {
        self.average as u128
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Observations {
    pub msg_sent: u64,
    pub ack_received: u64,
    pub latency_average: ExponentialMovingAverage<3>,
    pub probes_sent: u64,
    pub probes_failed: u64,
}

#[derive(Debug, Default, Clone)]
pub struct NetworkPeerTracker {
    peers: Arc<dashmap::DashMap<PeerId, Observations>>,
}

impl NetworkPeerTracker {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(dashmap::DashMap::new()),
        }
    }

    #[inline]
    pub fn add(&self, peer: PeerId) {
        if !self.peers.contains_key(&peer) {
            self.peers.insert(peer, Observations::default());
        }
    }

    #[inline]
    pub fn alter<F>(&self, peer: &PeerId, f: F)
    where
        F: FnOnce(&PeerId, Observations) -> Observations,
    {
        self.peers.alter(peer, f);
    }

    #[inline]
    pub fn get(&self, peer: &PeerId) -> Option<Observations> {
        self.peers.get(peer).map(|o| *o.value())
    }

    #[inline]
    pub fn remove(&self, peer: &PeerId) {
        self.peers.remove(peer);
    }

    /// The number of currently tracked peers with results.
    #[inline]
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.peers.len() == 0
    }

    #[inline]
    pub fn iter_keys(&self) -> impl Iterator<Item = PeerId> + '_ {
        self.peers.iter().map(|entry| *entry.key())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_api::PeerId;

    use super::{NetworkPeerTracker, Observations};

    #[test]
    fn peer_tracker_adding_a_peer_adds_a_default_observation() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();

        let peer = PeerId::random();

        tracker.add(peer.clone());

        assert_eq!(
            tracker.get(&peer).context("should contain a value")?,
            Observations::default()
        );

        Ok(())
    }

    #[test]
    fn peer_tracker_should_reflect_the_alteration_changes() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();

        let peer = PeerId::random();

        tracker.add(peer.clone());
        tracker.alter(&peer, |_, mut o| {
            o.msg_sent += 1;
            o
        });

        assert_eq!(
            tracker.get(&peer).context("should contain a value")?,
            Observations {
                msg_sent: 1,
                ack_received: 0,
                ..Default::default()
            }
        );

        Ok(())
    }

    #[test]
    fn peer_tracker_should_not_reflect_alterations_on_non_existent_peers() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();

        let peer = PeerId::random();

        tracker.alter(&peer, |_, mut o| {
            o.msg_sent += 1;
            o
        });

        assert!(tracker.get(&peer).is_none());

        Ok(())
    }

    #[test]
    fn running_average_should_compute_the_windowed_average_correctly() {
        let mut avg = super::ExponentialMovingAverage::<5>::new();

        for i in 1..=12 {
            avg.update(i);
        }

        assert_eq!(avg.get(), 8);
    }

    #[test]
    fn running_average_should_compute_the_average_from_constant_correctly() {
        let mut avg = super::ExponentialMovingAverage::<5>::new();

        for _ in 1..=10 {
            avg.update(3);
        }

        assert_eq!(avg.get(), 3);
    }
}
