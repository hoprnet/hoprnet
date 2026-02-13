use std::sync::Arc;

use hopr_api::PeerId;

use super::observation::Observations;

/// Tracker of [`Observations`] for network peers.
///
/// This structure maintains a mapping between [`PeerId`] and their associated
/// [`Observations`], allowing for efficient tracking and updating of peer telemetry data.
///
/// It can be combined with other objects to offer a complete view of the network state in regards
/// to immediate peer probing.
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

    /// The number of currently tracked peers.
    #[inline]
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// Check whether there are no tracked peers.
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

    use super::*;

    #[test]
    fn peer_tracker_adding_a_peer_adds_a_default_observation() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();

        let peer = PeerId::random();

        tracker.add(peer);

        assert_eq!(
            tracker.get(&peer).context("should contain a value")?,
            Observations::default()
        );

        Ok(())
    }

    #[test]
    fn peer_tracker_adding_multiple_different_peers_results_in_higher_count() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();

        const NUM_PEERS: usize = 10;

        for _ in 0..NUM_PEERS {
            tracker.add(PeerId::random());
        }

        assert_eq!(tracker.len(), NUM_PEERS);

        Ok(())
    }

    #[test]
    fn peer_tracker_should_reflect_the_alteration_changes() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();

        let peer = PeerId::random();

        tracker.add(peer);
        tracker.alter(&peer, |_, mut o| {
            o.msg_sent += 1;
            o
        });

        let obs = tracker.get(&peer).context("should contain a value")?;
        assert_eq!(obs.msg_sent, 1);
        assert_eq!(obs.ack_received, 0);

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
}
