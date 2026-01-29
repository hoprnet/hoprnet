use std::sync::Arc;

use hopr_api::PeerId;

use super::observation::{Observations, PeerPacketStats, PeerPacketStatsSnapshot};

/// Entry containing both observations and packet stats for a peer.
#[derive(Debug, Default)]
pub struct PeerEntry {
    pub observations: Observations,
    pub packet_stats: Arc<PeerPacketStats>,
}

/// Tracker of [`Observations`] and packet statistics for network peers.
///
/// This structure maintains a mapping between [`PeerId`] and their associated
/// [`PeerEntry`] (containing [`Observations`] and [`PeerPacketStats`]),
/// allowing for efficient tracking and updating of peer telemetry data.
///
/// It can be combined with other objects to offer a complete view of the network state in regards
/// to immediate peer probing.
#[derive(Debug, Default, Clone)]
pub struct NetworkPeerTracker {
    peers: Arc<dashmap::DashMap<PeerId, PeerEntry>>,
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
        self.peers.alter(peer, |peer_id: &PeerId, mut entry: PeerEntry| {
            entry.observations = f(peer_id, entry.observations);
            entry
        });
    }

    #[inline]
    pub fn get(&self, peer: &PeerId) -> Option<Observations> {
        self.peers.get(peer).map(|entry| entry.value().observations)
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

    /// Get the packet stats handle for a peer, for use in instrumenting streams.
    #[inline]
    pub fn get_packet_stats(&self, peer: &PeerId) -> Option<Arc<PeerPacketStats>> {
        self.peers.get(peer).map(|entry| entry.value().packet_stats.clone())
    }

    /// Get a snapshot of packet stats for a specific peer.
    #[inline]
    pub fn packet_stats_snapshot(&self, peer: &PeerId) -> Option<PeerPacketStatsSnapshot> {
        self.peers.get(peer).map(|entry| entry.value().packet_stats.snapshot())
    }

    /// Get packet stats snapshots for all tracked peers.
    pub fn all_packet_stats(&self) -> Vec<(PeerId, PeerPacketStatsSnapshot)> {
        self.peers
            .iter()
            .map(|entry| (*entry.key(), entry.value().packet_stats.snapshot()))
            .collect()
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

        tracker.add(peer.clone());

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

        tracker.add(peer.clone());
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

    #[test]
    fn peer_tracker_should_provide_packet_stats_for_tracked_peer() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();
        let peer = PeerId::random();

        tracker.add(peer);

        let stats = tracker.get_packet_stats(&peer).context("should contain packet stats")?;
        stats.record_packet_out(100);
        stats.record_packet_in(50);

        let snapshot = tracker
            .packet_stats_snapshot(&peer)
            .context("should contain snapshot")?;
        insta::assert_yaml_snapshot!(snapshot);

        Ok(())
    }

    #[test]
    fn peer_tracker_should_return_none_for_packet_stats_of_untracked_peer() {
        let tracker = NetworkPeerTracker::new();
        let peer = PeerId::random();

        assert!(tracker.get_packet_stats(&peer).is_none());
        assert!(tracker.packet_stats_snapshot(&peer).is_none());
    }

    #[test]
    fn peer_tracker_should_reset_packet_stats_on_remove_and_readd() -> anyhow::Result<()> {
        let tracker = NetworkPeerTracker::new();
        let peer = PeerId::random();

        tracker.add(peer);
        let stats = tracker.get_packet_stats(&peer).context("should contain stats")?;
        stats.record_packet_out(1000);

        let before_snapshot = tracker.packet_stats_snapshot(&peer).context("should have snapshot")?;
        insta::assert_yaml_snapshot!("before_remove", before_snapshot);

        tracker.remove(&peer);
        tracker.add(peer);

        let after_snapshot = tracker
            .packet_stats_snapshot(&peer)
            .context("should have new snapshot")?;
        insta::assert_yaml_snapshot!("after_readd", after_snapshot);

        Ok(())
    }

    #[test]
    fn peer_tracker_all_packet_stats_should_return_all_peers() {
        let tracker = NetworkPeerTracker::new();
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        tracker.add(peer1);
        tracker.add(peer2);

        if let Some(stats) = tracker.get_packet_stats(&peer1) {
            stats.record_packet_out(100);
        }
        if let Some(stats) = tracker.get_packet_stats(&peer2) {
            stats.record_packet_in(200);
        }

        let mut all_stats = tracker.all_packet_stats();
        assert_eq!(all_stats.len(), 2);

        // Sort by bytes_out descending so peer1 (100 bytes out) comes first,
        // giving us a deterministic order regardless of DashMap iteration.
        all_stats.sort_by(|(_, a), (_, b)| b.bytes_out.cmp(&a.bytes_out));

        let snapshots: Vec<_> = all_stats.iter().map(|(_, s)| s).collect();
        insta::assert_yaml_snapshot!(snapshots);
    }
}
