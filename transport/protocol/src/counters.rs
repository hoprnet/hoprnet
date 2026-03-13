use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use dashmap::DashMap;
use hopr_api::types::crypto::types::OffchainPublicKey;

/// Minimal atomic counters for per-peer protocol conformance tracking.
///
/// Tracks the number of messages sent and acknowledgments received for a
/// single peer. All operations are lock-free using relaxed atomic ordering.
#[derive(Debug, Default)]
pub struct PeerProtocolCounters {
    messages_sent: AtomicU64,
    acks_received: AtomicU64,
}

impl PeerProtocolCounters {
    /// Record that a message was sent to this peer.
    #[inline]
    pub fn record_message_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Record that an acknowledgment was received from this peer.
    #[inline]
    pub fn record_ack_received(&self) {
        self.acks_received.fetch_add(1, Ordering::Relaxed);
    }

    /// Atomically swap both counters to 0, returning accumulated values.
    pub fn take(&self) -> (u64, u64) {
        (
            self.messages_sent.swap(0, Ordering::Relaxed),
            self.acks_received.swap(0, Ordering::Relaxed),
        )
    }
}

/// Thread-safe registry of per-peer protocol conformance counters.
///
/// Keyed by [`OffchainPublicKey`] — no PeerId conversion needed since the
/// protocol pipeline already operates on offchain keys.
#[derive(Debug, Default, Clone)]
pub struct PeerProtocolCounterRegistry {
    inner: Arc<DashMap<OffchainPublicKey, Arc<PeerProtocolCounters>>>,
}

impl PeerProtocolCounterRegistry {
    /// Get or create counters for the given peer.
    pub fn get_or_create(&self, peer: &OffchainPublicKey) -> Arc<PeerProtocolCounters> {
        self.inner
            .entry(*peer)
            .or_insert_with(|| Arc::new(PeerProtocolCounters::default()))
            .value()
            .clone()
    }

    /// Swap all counters to 0, returning `(peer, msgs_sent, acks_received)` for non-zero entries.
    pub fn drain(&self) -> Vec<(OffchainPublicKey, u64, u64)> {
        self.inner
            .iter()
            .filter_map(|entry| {
                let (sent, received) = entry.value().take();
                if sent > 0 || received > 0 {
                    Some((*entry.key(), sent, received))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_api::types::crypto::prelude::{Keypair, OffchainKeypair};

    use super::*;

    #[test]
    fn counters_should_start_at_zero() {
        let counters = PeerProtocolCounters::default();
        let (sent, received) = counters.take();

        assert_eq!(sent, 0);
        assert_eq!(received, 0);
    }

    #[test]
    fn counters_should_record_messages_sent() {
        let counters = PeerProtocolCounters::default();

        counters.record_message_sent();
        counters.record_message_sent();
        counters.record_message_sent();

        let (sent, received) = counters.take();
        assert_eq!(sent, 3);
        assert_eq!(received, 0);
    }

    #[test]
    fn counters_should_record_acks_received() {
        let counters = PeerProtocolCounters::default();

        counters.record_ack_received();
        counters.record_ack_received();

        let (sent, received) = counters.take();
        assert_eq!(sent, 0);
        assert_eq!(received, 2);
    }

    #[test]
    fn take_should_reset_counters_to_zero() {
        let counters = PeerProtocolCounters::default();

        counters.record_message_sent();
        counters.record_ack_received();

        let (sent1, received1) = counters.take();
        assert_eq!(sent1, 1);
        assert_eq!(received1, 1);

        let (sent2, received2) = counters.take();
        assert_eq!(sent2, 0);
        assert_eq!(received2, 0);
    }

    #[test]
    fn registry_should_create_and_retrieve_counters() -> anyhow::Result<()> {
        let registry = PeerProtocolCounterRegistry::default();
        let peer = *OffchainKeypair::random().public();

        let counters = registry.get_or_create(&peer);
        counters.record_message_sent();

        let same_counters = registry.get_or_create(&peer);
        same_counters.record_message_sent();

        let (sent, _) = counters.take();
        assert_eq!(sent, 2, "both calls should share the same counter instance");
        Ok(())
    }

    #[test]
    fn drain_should_return_only_nonzero_entries() -> anyhow::Result<()> {
        let registry = PeerProtocolCounterRegistry::default();
        let peer_a = *OffchainKeypair::random().public();
        let peer_b = *OffchainKeypair::random().public();
        let peer_c = *OffchainKeypair::random().public();

        registry.get_or_create(&peer_a).record_message_sent();
        registry.get_or_create(&peer_b); // no activity
        registry.get_or_create(&peer_c).record_ack_received();

        let drained = registry.drain();
        assert_eq!(drained.len(), 2, "only peers with non-zero counters should be drained");

        let a_entry = drained
            .iter()
            .find(|(p, ..)| *p == peer_a)
            .context("peer_a should be in drain results")?;
        assert_eq!(a_entry.1, 1);
        assert_eq!(a_entry.2, 0);

        let c_entry = drained
            .iter()
            .find(|(p, ..)| *p == peer_c)
            .context("peer_c should be in drain results")?;
        assert_eq!(c_entry.1, 0);
        assert_eq!(c_entry.2, 1);

        Ok(())
    }

    #[test]
    fn drain_should_reset_counters() {
        let registry = PeerProtocolCounterRegistry::default();
        let peer = *OffchainKeypair::random().public();

        registry.get_or_create(&peer).record_message_sent();

        let first_drain = registry.drain();
        assert_eq!(first_drain.len(), 1);

        let second_drain = registry.drain();
        assert!(second_drain.is_empty(), "counters should be zero after drain");
    }
}
