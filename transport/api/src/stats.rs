use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic counters for tracking packet statistics per peer.
///
/// Uses atomic operations for thread-safe, lock-free updates.
/// Stats are automatically reset when the peer disconnects (the entry is dropped).
#[derive(Debug, Default)]
pub struct PeerPacketStats {
    packets_out: AtomicU64,
    packets_in: AtomicU64,
    bytes_out: AtomicU64,
    bytes_in: AtomicU64,
}

impl PeerPacketStats {
    /// Record an outgoing packet with the given size in bytes.
    #[inline]
    pub fn record_packet_out(&self, bytes: usize) {
        self.packets_out.fetch_add(1, Ordering::Relaxed);
        self.bytes_out.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Record an incoming packet with the given size in bytes.
    #[inline]
    pub fn record_packet_in(&self, bytes: usize) {
        self.packets_in.fetch_add(1, Ordering::Relaxed);
        self.bytes_in.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Create a snapshot of the current stats.
    pub fn snapshot(&self) -> PeerPacketStatsSnapshot {
        PeerPacketStatsSnapshot {
            packets_out: self.packets_out.load(Ordering::Relaxed),
            packets_in: self.packets_in.load(Ordering::Relaxed),
            bytes_out: self.bytes_out.load(Ordering::Relaxed),
            bytes_in: self.bytes_in.load(Ordering::Relaxed),
        }
    }
}

/// A point-in-time snapshot of peer packet statistics.
///
/// This is a non-atomic, serializable copy of the stats for API responses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerPacketStatsSnapshot {
    pub packets_out: u64,
    pub packets_in: u64,
    pub bytes_out: u64,
    pub bytes_in: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_packet_stats_should_start_at_zero() {
        let stats = PeerPacketStats::default();
        let snapshot = stats.snapshot();

        insta::assert_yaml_snapshot!(snapshot);
    }

    #[test]
    fn peer_packet_stats_should_record_outgoing_packets() {
        let stats = PeerPacketStats::default();

        stats.record_packet_out(100);
        stats.record_packet_out(200);

        let snapshot = stats.snapshot();
        insta::assert_yaml_snapshot!(snapshot);
    }

    #[test]
    fn peer_packet_stats_should_record_incoming_packets() {
        let stats = PeerPacketStats::default();

        stats.record_packet_in(50);
        stats.record_packet_in(150);
        stats.record_packet_in(100);

        let snapshot = stats.snapshot();
        insta::assert_yaml_snapshot!(snapshot);
    }

    #[test]
    fn peer_packet_stats_should_record_bidirectional_traffic() {
        let stats = PeerPacketStats::default();

        stats.record_packet_out(1000);
        stats.record_packet_in(500);
        stats.record_packet_out(2000);
        stats.record_packet_in(1500);

        let snapshot = stats.snapshot();
        insta::assert_yaml_snapshot!(snapshot);
    }
}
