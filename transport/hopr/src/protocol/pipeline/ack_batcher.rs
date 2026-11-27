//! Per-peer outgoing acknowledgement batching.
//!
//! Splits the batching state machine ([`PeerAckBatcher`]) from the async driver
//! ([`PerPeerAckBatchStream`]) so expiry ordering, generation-tagged ABA protection,
//! capacity compaction, and EOF draining can be tested deterministically with synthetic
//! [`Instant`]s while the stream itself stays pull-based, runtime-agnostic, and free of
//! detached background tasks.

use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::{FutureExt as _, Stream};
use futures_timer::Delay;
use hopr_api::types::{crypto::prelude::OffchainPublicKey, internal::prelude::VerifiedAcknowledgement};
use hopr_protocol_hopr::prelude::MAX_ACKNOWLEDGEMENTS_BATCH_SIZE;

use crate::trace_context::WithContext;

/// A flushed per-peer batch of at most [`MAX_ACKNOWLEDGEMENTS_BATCH_SIZE`] acknowledgements.
pub(super) type AckBatch = (OffchainPublicKey, Vec<WithContext<VerifiedAcknowledgement>>);

struct PeerBatch {
    generation: u64,
    acks: Vec<WithContext<VerifiedAcknowledgement>>,
}

struct DeadlineEntry {
    peer: OffchainPublicKey,
    deadline: Instant,
    generation: u64,
}

/// Pure synchronous per-peer acknowledgement batcher.
///
/// Each peer's buffered partial batch records a monotonically increasing `generation` and appends a
/// matching [`DeadlineEntry`] to a FIFO [`VecDeque`]. Because every entry shares the same constant
/// `max_wait` and callers supply monotonically non-decreasing `Instant`s, the deque stays ordered by
/// deadline, allowing expired batches to be popped in amortized O(1) time without scanning the map.
///
/// Checking `generation` on pop prevents an ABA bug when a peer reaches
/// [`MAX_ACKNOWLEDGEMENTS_BATCH_SIZE`], flushes early, and starts a fresh batch—even within the
/// same [`Instant`] tick—before its superseded deadline entry reaches the front of the queue.
pub(super) struct PeerAckBatcher {
    groups: halfbrown::HashMap<OffchainPublicKey, PeerBatch, ahash::RandomState>,
    deadlines: VecDeque<DeadlineEntry>,
    max_wait: Duration,
    next_generation: u64,
}

impl PeerAckBatcher {
    /// Creates a batcher whose partial batches expire after `max_wait`, pre-sizing the per-peer
    /// map for `grouping_capacity` concurrently buffered peers.
    pub(super) fn new(max_wait: Duration, grouping_capacity: usize) -> Self {
        Self {
            groups: halfbrown::HashMap::with_capacity_and_hasher(grouping_capacity, ahash::RandomState::default()),
            deadlines: VecDeque::with_capacity(grouping_capacity),
            max_wait,
            next_generation: 0,
        }
    }

    /// Buffers an acknowledgement for `peer` at monotonic timestamp `now`.
    ///
    /// Returns `Some(batch)` immediately when `peer` reaches [`MAX_ACKNOWLEDGEMENTS_BATCH_SIZE`].
    pub(super) fn push(
        &mut self,
        peer: OffchainPublicKey,
        ack: WithContext<VerifiedAcknowledgement>,
        now: Instant,
    ) -> Option<AckBatch> {
        match self.groups.entry(peer) {
            halfbrown::Entry::Occupied(mut occupied) => {
                occupied.get_mut().acks.push(ack);
                if occupied.get().acks.len() >= MAX_ACKNOWLEDGEMENTS_BATCH_SIZE {
                    let batch = occupied.remove();
                    self.compact_deadlines_if_needed();
                    return Some((peer, batch.acks));
                }
                None
            }
            halfbrown::Entry::Vacant(vacant) => {
                let mut acks = Vec::with_capacity(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE);
                acks.push(ack);
                if acks.len() >= MAX_ACKNOWLEDGEMENTS_BATCH_SIZE {
                    return Some((peer, acks));
                }
                let generation = self.next_generation;
                self.next_generation = self.next_generation.wrapping_add(1);
                vacant.insert(PeerBatch { generation, acks });
                if let Some(deadline) = now.checked_add(self.max_wait) {
                    self.deadlines.push_back(DeadlineEntry {
                        peer,
                        deadline,
                        generation,
                    });
                }
                None
            }
        }
    }

    /// Pops the oldest live batch whose deadline is at or before `now`, discarding any superseded
    /// deadline entries encountered at the front of the queue.
    pub(super) fn pop_expired(&mut self, now: Instant) -> Option<AckBatch> {
        while let Some(front) = self.deadlines.front() {
            if front.deadline > now {
                return None;
            }
            let entry = self.deadlines.pop_front()?;
            let is_live = self
                .groups
                .get(&entry.peer)
                .is_some_and(|batch| batch.generation == entry.generation);
            if is_live && let Some(batch) = self.groups.remove(&entry.peer) {
                return Some((entry.peer, batch.acks));
            }
        }
        None
    }

    /// Returns the earliest live deadline after pruning any stale entries at the front of the queue.
    pub(super) fn next_deadline(&mut self) -> Option<Instant> {
        self.prune_stale_front();
        self.deadlines.front().map(|entry| entry.deadline)
    }

    /// Returns `true` if the deadline queue contains at least one entry (live or not yet pruned).
    pub(super) fn has_deadlines(&self) -> bool {
        !self.deadlines.is_empty()
    }

    /// Drains all remaining buffered partial batches (used on upstream EOF).
    pub(super) fn drain_all(&mut self) -> Vec<AckBatch> {
        self.deadlines.clear();
        self.groups.drain().map(|(peer, batch)| (peer, batch.acks)).collect()
    }

    fn prune_stale_front(&mut self) {
        while let Some(front) = self.deadlines.front() {
            let is_live = self
                .groups
                .get(&front.peer)
                .is_some_and(|batch| batch.generation == front.generation);
            if is_live {
                break;
            }
            self.deadlines.pop_front();
        }
    }

    /// Keeps stale entries bounded when a hot peer repeatedly flushes full batches behind a
    /// long-lived cold peer. Amortized O(1) because compaction only runs once more than half of
    /// the queue is stale (above a small floor).
    fn compact_deadlines_if_needed(&mut self) {
        if self.deadlines.len() > self.groups.len().saturating_mul(2).max(64) {
            self.deadlines.retain(|entry| {
                self.groups
                    .get(&entry.peer)
                    .is_some_and(|batch| batch.generation == entry.generation)
            });
        }
    }
}

/// Pull-based stream adapter that groups outgoing acknowledgements per peer, flushing each peer's
/// batch as soon as it reaches [`MAX_ACKNOWLEDGEMENTS_BATCH_SIZE`] or its oldest buffered
/// acknowledgement reaches `max_wait`.
///
/// Reuses a single [`Delay`] timer via [`Delay::reset`] and runs inline inside its consumer's poll
/// context, preserving the caller's tracing subscriber/span and dropping the inner stream
/// synchronously when dropped.
pub(super) struct PerPeerAckBatchStream<S> {
    inner: S,
    batcher: PeerAckBatcher,
    timer: Delay,
    armed_deadline: Option<Instant>,
    eof_drain: std::vec::IntoIter<AckBatch>,
    input_closed: bool,
}

impl<S> PerPeerAckBatchStream<S> {
    pub(super) fn new(inner: S, max_wait: Duration, grouping_capacity: usize) -> Self {
        Self {
            inner,
            batcher: PeerAckBatcher::new(max_wait, grouping_capacity),
            // Seed with ZERO (never `max_wait`, which could overflow on `Duration::MAX`);
            // `armed_deadline` starts as `None`, so the timer is always reset before first poll.
            timer: Delay::new(Duration::ZERO),
            armed_deadline: None,
            eof_drain: Vec::new().into_iter(),
            input_closed: false,
        }
    }
}

impl<S> Stream for PerPeerAckBatchStream<S>
where
    S: Stream<Item = (OffchainPublicKey, WithContext<VerifiedAcknowledgement>)> + Unpin,
{
    type Item = AckBatch;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.input_closed {
            return Poll::Ready(this.eof_drain.next());
        }

        loop {
            // Always flush already-expired partial batches before polling `inner` so a continuously
            // ready hot peer cannot starve low-rate peers past `max_wait`.
            if this.batcher.has_deadlines() {
                let now = Instant::now();
                if let Some(batch) = this.batcher.pop_expired(now) {
                    if this.armed_deadline.is_some_and(|d| d <= now) {
                        this.armed_deadline = None;
                    }
                    return Poll::Ready(Some(batch));
                }
            }

            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some((peer, ack))) => {
                    if let Some(batch) = this.batcher.push(peer, ack, Instant::now()) {
                        return Poll::Ready(Some(batch));
                    }
                }
                Poll::Ready(None) => {
                    this.input_closed = true;
                    this.armed_deadline = None;
                    this.eof_drain = this.batcher.drain_all().into_iter();
                    return Poll::Ready(this.eof_drain.next());
                }
                Poll::Pending => {
                    let Some(deadline) = this.batcher.next_deadline() else {
                        this.armed_deadline = None;
                        return Poll::Pending;
                    };

                    let now = Instant::now();
                    if let Some(batch) = this.batcher.pop_expired(now) {
                        if this.armed_deadline.is_some_and(|d| d <= now) {
                            this.armed_deadline = None;
                        }
                        return Poll::Ready(Some(batch));
                    }

                    if this.armed_deadline != Some(deadline) {
                        this.timer.reset(deadline.saturating_duration_since(now));
                        this.armed_deadline = Some(deadline);
                    }

                    match this.timer.poll_unpin(cx) {
                        Poll::Ready(()) => {
                            this.armed_deadline = None;
                            if let Some(batch) = this.batcher.pop_expired(Instant::now()) {
                                return Poll::Ready(Some(batch));
                            }
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Context as _;
    use hopr_api::types::crypto::prelude::{Keypair, OffchainKeypair};

    use super::*;

    fn test_ack(key: &OffchainKeypair) -> WithContext<VerifiedAcknowledgement> {
        WithContext::new(VerifiedAcknowledgement::random(key), tracing::Span::none())
    }

    #[test]
    fn fifo_expiry_ordering_flushes_oldest_peer_first() -> anyhow::Result<()> {
        let key = OffchainKeypair::random();
        let peer_a = *OffchainKeypair::random().public();
        let peer_b = *OffchainKeypair::random().public();
        let peer_c = *OffchainKeypair::random().public();

        let max_wait = Duration::from_millis(100);
        let mut batcher = PeerAckBatcher::new(max_wait, 4);
        let t0 = Instant::now();

        assert!(batcher.push(peer_a, test_ack(&key), t0).is_none());
        assert!(
            batcher
                .push(peer_b, test_ack(&key), t0 + Duration::from_millis(30))
                .is_none()
        );
        assert!(
            batcher
                .push(peer_b, test_ack(&key), t0 + Duration::from_millis(40))
                .is_none()
        );
        assert!(
            batcher
                .push(peer_c, test_ack(&key), t0 + Duration::from_millis(60))
                .is_none()
        );

        assert_eq!(batcher.next_deadline(), Some(t0 + Duration::from_millis(100)));
        assert!(batcher.pop_expired(t0 + Duration::from_millis(99)).is_none());

        let (peer, acks) = batcher
            .pop_expired(t0 + Duration::from_millis(100))
            .context("peer A must expire at t0 + 100ms")?;
        assert_eq!(peer, peer_a);
        assert_eq!(acks.len(), 1);
        assert!(batcher.pop_expired(t0 + Duration::from_millis(100)).is_none());

        // Peer B's deadline is anchored to its first buffered ack (t0 + 30ms), not its second.
        assert_eq!(batcher.next_deadline(), Some(t0 + Duration::from_millis(130)));

        // When both B and C are past their deadlines, pop_expired emits B before C.
        let (peer, acks) = batcher
            .pop_expired(t0 + Duration::from_millis(170))
            .context("peer B must expire before peer C")?;
        assert_eq!(peer, peer_b);
        assert_eq!(acks.len(), 2);

        let (peer, acks) = batcher
            .pop_expired(t0 + Duration::from_millis(170))
            .context("peer C must expire after peer B")?;
        assert_eq!(peer, peer_c);
        assert_eq!(acks.len(), 1);

        assert!(batcher.pop_expired(t0 + Duration::from_millis(170)).is_none());
        assert_eq!(batcher.next_deadline(), None);
        Ok(())
    }

    #[test]
    fn same_instant_aba_refill_after_cap_flush_respects_generation() -> anyhow::Result<()> {
        let key = OffchainKeypair::random();
        let peer_a = *OffchainKeypair::random().public();
        let peer_b = *OffchainKeypair::random().public();

        let max_wait = Duration::from_millis(100);
        let mut batcher = PeerAckBatcher::new(max_wait, 4);
        let t0 = Instant::now();

        // Fill peer A to cap at t0 -> leaves stale (peer_a, t0 + 100ms, gen 0) in deadlines.
        for _ in 0..(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE - 1) {
            assert!(batcher.push(peer_a, test_ack(&key), t0).is_none());
        }
        let (flushed_peer, flushed_acks) = batcher
            .push(peer_a, test_ack(&key), t0)
            .context("33rd ack must flush immediately")?;
        assert_eq!(flushed_peer, peer_a);
        assert_eq!(flushed_acks.len(), MAX_ACKNOWLEDGEMENTS_BATCH_SIZE);

        // Refill peer A in the exact same Instant tick t0 (gen 1, same deadline t0 + 100ms),
        // then flush it again at t0 + 40ms and refill (gen 2, deadline t0 + 140ms).
        assert!(batcher.push(peer_a, test_ack(&key), t0).is_none());
        assert!(
            batcher
                .push(peer_b, test_ack(&key), t0 + Duration::from_millis(10))
                .is_none()
        );
        for _ in 0..(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE - 2) {
            assert!(
                batcher
                    .push(peer_a, test_ack(&key), t0 + Duration::from_millis(40))
                    .is_none()
            );
        }
        let (flushed_peer, flushed_acks) = batcher
            .push(peer_a, test_ack(&key), t0 + Duration::from_millis(40))
            .context("second 33-ack batch must flush immediately")?;
        assert_eq!(flushed_peer, peer_a);
        assert_eq!(flushed_acks.len(), MAX_ACKNOWLEDGEMENTS_BATCH_SIZE);

        // Refill peer A (gen 2) at t0 + 40ms -> must NOT expire until t0 + 140ms.
        assert!(
            batcher
                .push(peer_a, test_ack(&key), t0 + Duration::from_millis(40))
                .is_none()
        );

        // At t0 + 100ms, both stale gen 0 and gen 1 entries for peer A must be discarded without
        // touching gen 2.
        assert!(
            batcher.pop_expired(t0 + Duration::from_millis(100)).is_none(),
            "stale deadline entries for peer A must not prematurely flush gen 2"
        );
        assert_eq!(batcher.next_deadline(), Some(t0 + Duration::from_millis(110)));

        let (peer, acks) = batcher
            .pop_expired(t0 + Duration::from_millis(110))
            .context("peer B must expire at t0 + 110ms")?;
        assert_eq!(peer, peer_b);
        assert_eq!(acks.len(), 1);

        assert!(batcher.pop_expired(t0 + Duration::from_millis(139)).is_none());
        let (peer, acks) = batcher
            .pop_expired(t0 + Duration::from_millis(140))
            .context("peer A gen 2 must expire at t0 + 140ms")?;
        assert_eq!(peer, peer_a);
        assert_eq!(acks.len(), 1);
        assert!(batcher.pop_expired(t0 + Duration::from_millis(140)).is_none());

        // Also verify same-Instant refill without a second flush expires cleanly exactly once.
        let t1 = t0 + Duration::from_millis(200);
        for _ in 0..(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE - 1) {
            assert!(batcher.push(peer_a, test_ack(&key), t1).is_none());
        }
        assert!(batcher.push(peer_a, test_ack(&key), t1).is_some());
        assert!(batcher.push(peer_a, test_ack(&key), t1).is_none());

        assert_eq!(batcher.next_deadline(), Some(t1 + max_wait));
        let (peer, acks) = batcher
            .pop_expired(t1 + max_wait)
            .context("same-Instant refilled batch must expire once at t1 + max_wait")?;
        assert_eq!(peer, peer_a);
        assert_eq!(acks.len(), 1);
        assert!(batcher.pop_expired(t1 + max_wait).is_none());
        Ok(())
    }

    #[test]
    fn duration_max_never_arms_deadline_and_still_flushes_full_and_eof() -> anyhow::Result<()> {
        let key = OffchainKeypair::random();
        let peer_a = *OffchainKeypair::random().public();
        let peer_b = *OffchainKeypair::random().public();

        let mut batcher = PeerAckBatcher::new(Duration::MAX, 4);
        let t0 = Instant::now();

        assert!(batcher.push(peer_b, test_ack(&key), t0).is_none());
        assert!(!batcher.has_deadlines());
        assert_eq!(batcher.next_deadline(), None);
        assert!(batcher.pop_expired(t0 + Duration::from_secs(3600)).is_none());

        for _ in 0..(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE - 1) {
            assert!(batcher.push(peer_a, test_ack(&key), t0).is_none());
        }
        let (peer, acks) = batcher
            .push(peer_a, test_ack(&key), t0)
            .context("full batch must flush even with Duration::MAX")?;
        assert_eq!(peer, peer_a);
        assert_eq!(acks.len(), MAX_ACKNOWLEDGEMENTS_BATCH_SIZE);

        let drained = batcher.drain_all();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].0, peer_b);
        assert_eq!(drained[0].1.len(), 1);
        assert!(batcher.drain_all().is_empty());
        Ok(())
    }

    #[test]
    fn hot_peer_full_batches_do_not_starve_cold_peer_or_grow_deadlines_unboundedly() -> anyhow::Result<()> {
        let key = OffchainKeypair::random();
        let cold_peer = *OffchainKeypair::random().public();
        let hot_peer = *OffchainKeypair::random().public();

        let max_wait = Duration::from_millis(100);
        let mut batcher = PeerAckBatcher::new(max_wait, 4);
        let t0 = Instant::now();

        assert!(batcher.push(cold_peer, test_ack(&key), t0).is_none());

        for batch_idx in 0u64..100 {
            let tick = t0 + Duration::from_millis(batch_idx);
            for _ in 0..(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE - 1) {
                assert!(batcher.push(hot_peer, test_ack(&key), tick).is_none());
            }
            let (peer, acks) = batcher
                .push(hot_peer, test_ack(&key), tick)
                .context("hot peer batch must flush at cap")?;
            assert_eq!(peer, hot_peer);
            assert_eq!(acks.len(), MAX_ACKNOWLEDGEMENTS_BATCH_SIZE);
        }

        // Compaction keeps stale entries bounded behind the cold peer.
        assert!(
            batcher.deadlines.len() <= 65,
            "stale deadline entries grew unboundedly: {}",
            batcher.deadlines.len()
        );

        let (peer, acks) = batcher
            .pop_expired(t0 + max_wait)
            .context("cold peer must expire on schedule despite hot peer traffic")?;
        assert_eq!(peer, cold_peer);
        assert_eq!(acks.len(), 1);
        assert_eq!(batcher.next_deadline(), None);
        Ok(())
    }

    #[test]
    fn eof_drain_emits_all_remaining_partial_batches_and_clears_state() -> anyhow::Result<()> {
        let key = OffchainKeypair::random();
        let peer_a = *OffchainKeypair::random().public();
        let peer_b = *OffchainKeypair::random().public();
        let peer_c = *OffchainKeypair::random().public();
        let peer_full = *OffchainKeypair::random().public();

        let mut batcher = PeerAckBatcher::new(Duration::from_secs(10), 4);
        let t0 = Instant::now();

        for _ in 0..2 {
            assert!(batcher.push(peer_a, test_ack(&key), t0).is_none());
        }
        for _ in 0..5 {
            assert!(batcher.push(peer_b, test_ack(&key), t0).is_none());
        }
        assert!(batcher.push(peer_c, test_ack(&key), t0).is_none());
        for _ in 0..(MAX_ACKNOWLEDGEMENTS_BATCH_SIZE - 1) {
            assert!(batcher.push(peer_full, test_ack(&key), t0).is_none());
        }
        assert!(batcher.push(peer_full, test_ack(&key), t0).is_some());

        let drained: HashMap<OffchainPublicKey, usize> = batcher
            .drain_all()
            .into_iter()
            .map(|(peer, acks)| (peer, acks.len()))
            .collect();
        assert_eq!(drained.len(), 3);
        assert_eq!(drained.get(&peer_a), Some(&2));
        assert_eq!(drained.get(&peer_b), Some(&5));
        assert_eq!(drained.get(&peer_c), Some(&1));
        assert!(!drained.contains_key(&peer_full));

        assert!(!batcher.has_deadlines());
        assert_eq!(batcher.next_deadline(), None);
        assert!(batcher.drain_all().is_empty());
        Ok(())
    }

    #[test]
    fn dropping_per_peer_ack_batch_stream_drops_inner_stream_immediately() {
        let (drop_tx, mut drop_rx) = futures::channel::oneshot::channel::<()>();
        struct GuardedPendingStream {
            _drop_tx: futures::channel::oneshot::Sender<()>,
        }
        impl Stream for GuardedPendingStream {
            type Item = (OffchainPublicKey, WithContext<VerifiedAcknowledgement>);

            fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                Poll::Pending
            }
        }

        let stream = PerPeerAckBatchStream::new(GuardedPendingStream { _drop_tx: drop_tx }, Duration::from_secs(10), 4);
        drop(stream);

        assert!(
            matches!(drop_rx.try_recv(), Err(futures::channel::oneshot::Canceled)),
            "dropping PerPeerAckBatchStream must synchronously drop its inner stream"
        );
    }
}
