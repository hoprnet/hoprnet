use std::sync::atomic::{AtomicU64, Ordering};

use hopr_api::network::Observable;
use hopr_statistics::ExponentialMovingAverage;

/// Observations related to a specific peer in the network.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Observations {
    pub msg_sent: u64,
    pub ack_received: u64,
    last_update: std::time::Duration,
    latency_average: ExponentialMovingAverage<3>,
    probe_success_rate: ExponentialMovingAverage<5>,
}

impl Observable for Observations {
    fn record_probe(&mut self, latency: std::result::Result<std::time::Duration, ()>) {
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

    #[inline]
    fn last_update(&self) -> std::time::Duration {
        self.last_update
    }

    fn average_latency(&self) -> Option<std::time::Duration> {
        if self.latency_average.get() <= 0.0 {
            None
        } else {
            Some(std::time::Duration::from_millis(self.latency_average.get() as u64))
        }
    }

    fn average_probe_rate(&self) -> f64 {
        self.probe_success_rate.get()
    }

    fn score(&self) -> f64 {
        self.probe_success_rate.get()
    }
}

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
    use anyhow::Context;
    use assertables::{assert_gt, assert_in_delta, assert_lt};

    use super::*;

    #[test]
    fn observations_should_update_the_timestamp_on_latency_update() {
        let mut observation = Observations::default();

        assert_eq!(observation.last_update, std::time::Duration::default());

        observation.record_probe(Ok(std::time::Duration::from_millis(50)));

        std::thread::sleep(std::time::Duration::from_millis(10));

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

    #[test]
    fn peer_packet_stats_should_start_at_zero() {
        let stats = PeerPacketStats::default();
        let snapshot = stats.snapshot();

        assert_eq!(snapshot.packets_out, 0);
        assert_eq!(snapshot.packets_in, 0);
        assert_eq!(snapshot.bytes_out, 0);
        assert_eq!(snapshot.bytes_in, 0);
    }

    #[test]
    fn peer_packet_stats_should_record_outgoing_packets() {
        let stats = PeerPacketStats::default();

        stats.record_packet_out(100);
        stats.record_packet_out(200);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.packets_out, 2);
        assert_eq!(snapshot.bytes_out, 300);
        assert_eq!(snapshot.packets_in, 0);
        assert_eq!(snapshot.bytes_in, 0);
    }

    #[test]
    fn peer_packet_stats_should_record_incoming_packets() {
        let stats = PeerPacketStats::default();

        stats.record_packet_in(50);
        stats.record_packet_in(150);
        stats.record_packet_in(100);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.packets_in, 3);
        assert_eq!(snapshot.bytes_in, 300);
        assert_eq!(snapshot.packets_out, 0);
        assert_eq!(snapshot.bytes_out, 0);
    }

    #[test]
    fn peer_packet_stats_should_record_bidirectional_traffic() {
        let stats = PeerPacketStats::default();

        stats.record_packet_out(1000);
        stats.record_packet_in(500);
        stats.record_packet_out(2000);
        stats.record_packet_in(1500);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.packets_out, 2);
        assert_eq!(snapshot.packets_in, 2);
        assert_eq!(snapshot.bytes_out, 3000);
        assert_eq!(snapshot.bytes_in, 2000);
    }
}
