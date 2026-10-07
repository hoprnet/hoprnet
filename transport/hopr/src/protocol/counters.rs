use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
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

    /// Record that `count` acknowledgments were received from this peer in a batch.
    #[inline]
    pub fn record_acks_received(&self, count: u64) {
        self.acks_received.fetch_add(count, Ordering::Relaxed);
    }

    /// Swap the sent-message counter to 0, returning the accumulated value.
    #[inline]
    pub fn take_sent(&self) -> u64 {
        self.messages_sent.swap(0, Ordering::Relaxed)
    }

    /// Swap the acknowledgment counter to 0, returning the accumulated value.
    #[inline]
    pub fn take_acks(&self) -> u64 {
        self.acks_received.swap(0, Ordering::Relaxed)
    }

    /// Swap both counters to 0, returning accumulated values.
    ///
    /// Each counter is swapped independently via its own atomic operation;
    /// there is no single atomic snapshot of the pair.
    pub fn take(&self) -> (u64, u64) {
        (self.take_sent(), self.take_acks())
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

    /// Drains the counters for one conformance report, giving every packet in it `ack_grace` to be
    /// acknowledged.
    ///
    /// [`Self::drain`] reads both counters at the same instant, so a packet still in flight then is
    /// reported as sent but never acknowledged. In steady traffic that is a sliver of the window. A
    /// burst that starts just before the flush, though, is in flight almost entirely: the report
    /// reads as an acknowledgment rate near zero, the edge falls below the path planner's
    /// `min_ack_rate`, and it stays out of path selection until the next report (hoprnet#8484).
    ///
    /// So the sent counts are taken first, and the acknowledgments only `ack_grace` later. Packets
    /// sent during the grace stay counted for the next report. Their acknowledgments that arrive
    /// within it are credited to this one, a report early. The decayed totals the graph keeps absorb
    /// that: a report later, a burst's acknowledgments still weigh 0.9 against its packets.
    ///
    /// Returns `(peer, msgs_sent, acks_received)` for non-zero entries, like [`Self::drain`].
    pub async fn drain_settled(&self, ack_grace: Duration) -> Vec<(OffchainPublicKey, u64, u64)> {
        let sent = self
            .inner
            .iter()
            .map(|entry| (*entry.key(), entry.value().take_sent()))
            .filter(|(_, sent)| *sent > 0)
            .collect::<HashMap<_, _>>();

        if !ack_grace.is_zero() {
            futures_timer::Delay::new(ack_grace).await;
        }

        self.inner
            .iter()
            .filter_map(|entry| {
                let sent = sent.get(entry.key()).copied().unwrap_or_default();
                let received = entry.value().take_acks();
                (sent > 0 || received > 0).then_some((*entry.key(), sent, received))
            })
            .collect()
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
    use std::time::Duration;

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

    #[tokio::test]
    async fn drain_settled_should_credit_acks_that_arrive_within_the_grace() -> anyhow::Result<()> {
        let registry = PeerProtocolCounterRegistry::default();
        let peer = *OffchainKeypair::random().public();
        let counters = registry.get_or_create(&peer);

        // A burst whose acknowledgments are all still in flight when the flush starts.
        for _ in 0..10 {
            counters.record_message_sent();
        }

        let late_acks = {
            let counters = counters.clone();
            async move {
                futures_timer::Delay::new(Duration::from_millis(20)).await;
                counters.record_acks_received(10);
            }
        };
        let (drained, _) = futures::join!(registry.drain_settled(Duration::from_millis(200)), late_acks);

        let (_, sent, received) = drained.first().context("the peer must be reported")?;
        assert_eq!(
            (*sent, *received),
            (10, 10),
            "acknowledgments arriving within the grace must be credited"
        );
        Ok(())
    }

    #[tokio::test]
    async fn drain_settled_should_carry_packets_sent_during_the_grace_into_the_next_report() -> anyhow::Result<()> {
        let registry = PeerProtocolCounterRegistry::default();
        let peer = *OffchainKeypair::random().public();
        let counters = registry.get_or_create(&peer);

        counters.record_message_sent();
        let in_grace = {
            let counters = counters.clone();
            async move {
                futures_timer::Delay::new(Duration::from_millis(20)).await;
                counters.record_message_sent();
                counters.record_message_sent();
            }
        };
        let (first, _) = futures::join!(registry.drain_settled(Duration::from_millis(200)), in_grace);
        let (_, sent, _) = first.first().context("the peer must be reported")?;
        assert_eq!(*sent, 1, "only the packet sent before the grace belongs to this report");

        let second = registry.drain_settled(Duration::ZERO).await;
        let (_, sent, _) = second.first().context("the carried packets must be reported")?;
        assert_eq!(*sent, 2, "packets sent during the grace must be reported next time");
        Ok(())
    }

    #[tokio::test]
    async fn drain_settled_should_report_only_active_peers() {
        let registry = PeerProtocolCounterRegistry::default();
        registry.get_or_create(OffchainKeypair::random().public());

        assert!(
            registry.drain_settled(Duration::ZERO).await.is_empty(),
            "a peer with nothing sent or received must not be reported"
        );
    }
}
