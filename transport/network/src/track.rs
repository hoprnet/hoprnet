use std::sync::Arc;

use hopr_api::PeerId;

use crate::utils::ExponentialMovingAverage;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Observations {
    pub msg_sent: u64,
    pub ack_received: u64,
    pub last_update: std::time::Duration,
    latency_average: ExponentialMovingAverage<3>,
    probe_success_rate: ExponentialMovingAverage<5>,
}

impl Observations {
    pub fn record_probe(&mut self, latency: std::result::Result<std::time::Duration, ()>) {
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        if let Ok(latency) = latency {
            self.latency_average.update(latency.as_millis() as f64);
            self.probe_success_rate.update(1.0);
        } else {
            self.probe_success_rate.update(0.0);
        }
    }

    pub fn average_latency(&self) -> Option<std::time::Duration> {
        if self.latency_average.get() <= 0.0 {
            None
        } else {
            Some(std::time::Duration::from_millis(self.latency_average.get() as u64))
        }
    }

    /// A value between 0.0 and 1.0 representing the score of the peer based on probes.
    ///
    /// The higher the value, the better the score.
    pub fn score(&self) -> f64 {
        self.probe_success_rate.get()
    }
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
        self.peers.entry(peer).or_default();
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
    use assertables::{assert_gt, assert_in_delta, assert_lt};
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
        let mut avg = super::ExponentialMovingAverage::<5>::default();

        for i in 1..=10 {
            avg.update(i);
        }

        assertables::assert_in_delta!(avg.get(), 6.6, 0.1);
    }

    #[test]
    fn running_average_should_compute_the_average_from_constant_correctly() {
        let mut avg = super::ExponentialMovingAverage::<5>::default();

        for _ in 1..=10 {
            avg.update(3);
        }

        assertables::assert_f64_eq!(avg.get(), 3.0);
    }

    #[test]
    fn observations_should_update_the_timestamp_on_latency_update() {
        let mut observation = Observations::default();

        assert_eq!(observation.last_update, std::time::Duration::default());

        observation.record_probe(Ok(std::time::Duration::from_millis(50)));

        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        assert_gt!(observation.last_update, std::time::Duration::default());
        assert_lt!(observation.last_update, after);
    }

    #[test]
    fn observations_should_store_an_average_latency_value_after_multiple_updates() -> anyhow::Result<()> {
        let big_latency = std::time::Duration::from_millis(300);
        let small_latency = std::time::Duration::from_millis(10);

        let mut observation = Observations::default();

        for _ in 0..10 {
            observation.record_probe(Ok(small_latency));
        }

        assert_eq!(
            observation.average_latency().context("should contain a value")?,
            small_latency
        );

        observation.record_probe(Ok(big_latency));

        assert_gt!(
            observation.average_latency().context("should contain a value")?,
            small_latency
        );
        assert_lt!(
            observation.average_latency().context("should contain a value")?,
            big_latency
        );

        Ok(())
    }

    #[test]
    fn observations_should_store_the_averaged_success_rate_of_the_probes() {
        let small_latency = std::time::Duration::from_millis(10);

        let mut observation = Observations::default();

        for i in 0..10 {
            if i % 2 == 0 {
                observation.record_probe(Err(()));
            } else {
                observation.record_probe(Ok(small_latency));
            }
        }

        assert_in_delta!(observation.score(), 0.5, 0.05);
    }
}
