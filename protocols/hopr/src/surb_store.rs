use std::{collections::VecDeque, sync::Arc, time::Duration};

use hopr_api::types::internal::{prelude::HoprPseudonym, routing::SurbMatcher};
use hopr_crypto_packet::prelude::*;
use moka::notification::RemovalCause;
use validator::ValidationError;

use crate::{FoundSurb, traits::SurbStore};

const MINIMUM_SURB_LIFETIME: Duration = Duration::from_secs(30);
const MINIMUM_OPENER_PSEUDONYMS: usize = 1000;
const MINIMUM_OPENERS_PER_PSEUDONYM: usize = 1000;
const MINIMUM_SURBS_PER_PSEUDONYM: usize = 1000;
const MINIMUM_OPENER_LIFETIME: Duration = Duration::from_secs(60);
const MIN_SURB_RB_CAPACITY: usize = 1024;

fn validate_pseudonyms_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_SURB_LIFETIME {
        Err(ValidationError::new("pseudonyms_lifetime is too low"))
    } else {
        Ok(())
    }
}

fn validate_reply_opener_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_OPENER_LIFETIME {
        Err(ValidationError::new("reply_opener_lifetime is too low"))
    } else {
        Ok(())
    }
}

fn default_rb_capacity() -> usize {
    15_000
}

fn default_distress_threshold() -> usize {
    500
}

fn default_max_openers_per_pseudonym() -> usize {
    100_000
}

fn default_max_pseudonyms() -> usize {
    10_000
}

fn default_pseudonyms_lifetime() -> Duration {
    Duration::from_secs(600)
}

fn default_reply_opener_lifetime() -> Duration {
    Duration::from_secs(3600)
}

fn default_pop_order() -> SurbPopOrder {
    SurbPopOrder::Fifo
}

/// Which end of the per-pseudonym buffer a pop consumes from. Replying side only.
///
/// Overflow always evicts the oldest SURB, in either order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SurbPopOrder {
    /// Oldest first. Default; preserves the historical behaviour.
    #[default]
    Fifo,
    /// Newest first, so a return-path change applies immediately instead of only after the
    /// buffered SURBs drain. Stale ones are shed from the other end on overflow.
    Lifo,
}

/// Configuration for the SURB cache.
///
/// The configuration options affect both the sending side (SURB creator) and the
/// replying side (SURB consumer).
///
/// In the classical scenario (`Entry - Relay 1 -... - Exit`), the sending side is
/// the `Entry` and the replying side is the `Exit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
pub struct SurbStoreConfig {
    /// Size of the SURB ring buffer per pseudonym.
    ///
    /// Affects only the replying side.
    ///
    /// This indicates how many SURBs can be at most held to be used to send a reply
    /// back to the sending side.
    ///
    /// Default is 15 000.
    #[default(default_rb_capacity())]
    #[validate(range(min = 1024, message = "rb_capacity must be at least 1024"))]
    #[cfg_attr(feature = "serde", serde(default = "default_rb_capacity"))]
    pub rb_capacity: usize,
    /// Which end of the per-pseudonym buffer a pop consumes from; see [`SurbPopOrder`].
    ///
    /// Affects only the replying side. Default is [`SurbPopOrder::Fifo`].
    #[default(default_pop_order())]
    #[cfg_attr(feature = "serde", serde(default = "default_pop_order"))]
    pub pop_order: SurbPopOrder,
    /// Threshold for the number of SURBs in the ring buffer, below which it is
    /// considered low ("SURB distress").
    ///
    /// Default is 500.
    #[default(default_distress_threshold())]
    #[validate(range(min = 10, message = "distress_threshold must be at least 10"))]
    #[cfg_attr(feature = "serde", serde(default = "default_distress_threshold"))]
    pub distress_threshold: usize,
    /// Maximum number of reply openers (SURB counterparts) per pseudonym.
    ///
    /// Affects only the sending side when decrypting a received reply.
    ///
    /// This mostly affects Sessions, as they use a fixed pseudonym.
    /// It reflects how many reply openers the initiator-side of a Session can hold,
    /// until the oldest ones are dropped. If the other party uses a SURB corresponding
    /// to a dropped reply opener, the reply message will be undecryptable by the initiator-side.
    ///
    /// Default is 100 000.
    #[default(default_max_openers_per_pseudonym())]
    #[validate(range(min = 100, message = "max_openers_per_pseudonym must be at least 100"))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_openers_per_pseudonym"))]
    pub max_openers_per_pseudonym: usize,
    /// The maximum number of distinct pseudonyms for which we hold a SURB ringbuffer.
    ///
    /// Affects only the replying side.
    ///
    /// For each pseudonym, there is a ring-buffer with capacity `rb_capacity`.
    ///
    /// Default is 10 000.
    #[default(default_max_pseudonyms())]
    #[validate(range(min = 100, message = "max_pseudonyms must be at least 100"))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_pseudonyms"))]
    pub max_pseudonyms: usize,
    /// Maximum lifetime of ring-buffer for each pseudonym.
    ///
    /// # Effects on sending side
    /// This is the period for which we hold all reply openers for a pseudonym.
    /// If no more messages carrying SURBs are sent during this period, the entire stash of
    /// reply openers is dropped. Preventing receiving any more replies for that pseudonym.
    ///
    /// # Effects on replying side
    /// If a pseudonym has not received any SURBs for this period,
    /// the entire ring buffer with `rb_capacity` (= all SURBs for this pseudonym) is dropped.
    /// Preventing from sending any more replies for that pseudonym.
    ///
    /// Default is 600 seconds.
    #[default(default_pseudonyms_lifetime())]
    #[validate(custom(function = "validate_pseudonyms_lifetime"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_pseudonyms_lifetime", with = "humantime_serde")
    )]
    pub pseudonyms_lifetime: Duration,
    /// Maximum lifetime of a reply opener.
    ///
    /// Affects only the sending side.
    ///
    /// A reply opener is distinguished using [`HoprSurbId`] and a pseudonym it belongs to.
    /// If a reply opener is not used to decrypt the received packet within this period,
    /// it is dropped. If the replying side uses the corresponding SURB to send a reply,
    /// it won't be possible to decrypt it when received.
    ///
    /// Default is 3600 seconds.
    #[default(default_reply_opener_lifetime())]
    #[validate(custom(function = "validate_reply_opener_lifetime"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_reply_opener_lifetime", with = "humantime_serde")
    )]
    pub reply_opener_lifetime: Duration,
}

/// Basic [`SurbStore`] implementation based on an in-memory cache.
///
/// This SURB store offers no persistence, and all SURBs and Reply Openers are lost once dropped.
///
/// The instance can be cheaply cloned.
#[derive(Clone)]
pub struct MemorySurbStore {
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    surbs_per_pseudonym: moka::sync::Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
    cfg: Arc<SurbStoreConfig>,
}

impl MemorySurbStore {
    /// Creates a new instance with the given configuration.
    pub fn new(cfg: SurbStoreConfig) -> Self {
        Self {
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|sender_id, _reply_opener, cause| {
                    tracing::warn!(?sender_id, ?cause, "evicting reply opener for pseudonym");
                })
                .max_capacity(cfg.max_openers_per_pseudonym.max(MINIMUM_OPENER_PSEUDONYMS) as u64)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|pseudonym, _reply_opener, cause| {
                    tracing::warn!(%pseudonym, ?cause, "evicting surb for pseudonym");
                })
                .max_capacity(cfg.max_pseudonyms.max(MINIMUM_SURBS_PER_PSEUDONYM) as u64)
                .build(),
            cfg: cfg.into(),
        }
    }
}

impl Default for MemorySurbStore {
    fn default() -> Self {
        Self::new(SurbStoreConfig::default())
    }
}

#[async_trait::async_trait]
impl SurbStore for MemorySurbStore {
    #[tracing::instrument(skip_all, level = "trace", fields(?matcher), ret)]
    fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb> {
        let pseudonym = matcher.pseudonym();
        let surbs_for_pseudonym = self.surbs_per_pseudonym.get(&pseudonym)?;

        match matcher {
            SurbMatcher::Pseudonym(_) => surbs_for_pseudonym.pop_one().map(|popped_surb| FoundSurb {
                sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                surb: popped_surb.surb,
                remaining: popped_surb.remaining,
            }),
            // The following code intentionally only checks the SURB at the popping end of the
            // ring buffer and does not search the entire RB.
            // This is because the exact match use-case is suited only for situations
            // when there is a single SURB in the RB.
            SurbMatcher::Exact(id) => {
                surbs_for_pseudonym
                    .pop_one_if_has_id(&id.surb_id())
                    .map(|popped_surb| FoundSurb {
                        sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                        surb: popped_surb.surb,
                        remaining: popped_surb.remaining, // = likely 0
                    })
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace", fields(%pseudonym, num_surbs = surbs.len()))]
    fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize {
        self.surbs_per_pseudonym
            .entry_by_ref(&pseudonym)
            .or_insert_with(|| SurbRingBuffer::new(self.cfg.rb_capacity.max(MIN_SURB_RB_CAPACITY), self.cfg.pop_order))
            .value()
            .push(surbs)
    }

    #[tracing::instrument(skip_all, level = "trace", fields(?sender_id))]
    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        let opener_lifetime = self.cfg.reply_opener_lifetime.max(MINIMUM_OPENER_LIFETIME);
        let max_openers_per_pseudonym = self.cfg.max_openers_per_pseudonym.max(MINIMUM_OPENERS_PER_PSEUDONYM);
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), move || {
                moka::sync::Cache::builder()
                    .time_to_live(opener_lifetime)
                    .eviction_listener(move |id: Arc<HoprSurbId>, _, cause| {
                        if cause != RemovalCause::Explicit {
                            tracing::warn!(
                                pseudonym = %sender_id.pseudonym(),
                                surb_id = const_hex::encode(id.as_slice()),
                                ?cause,
                                "evicting reply opener for sender id"
                            );
                        }
                    })
                    .max_capacity(max_openers_per_pseudonym as u64)
                    .build()
            })
            .insert(sender_id.surb_id(), opener);
    }

    #[tracing::instrument(skip_all, level = "trace", fields(?sender_id), ret)]
    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener> {
        self.pseudonym_openers
            .get(&sender_id.pseudonym())
            .and_then(|cache| cache.remove(&sender_id.surb_id()))
    }
}

/// Represents a single SURB along with its ID popped from the [`SurbRingBuffer`].
#[derive(Debug, Clone)]
pub struct PoppedSurb<S> {
    /// Complete SURB sender ID.
    pub id: HoprSurbId,
    /// The popped SURB.
    pub surb: S,
    /// Number of SURBs left in the RB after the pop.
    pub remaining: usize,
}

/// Ring buffer of SURBs and their IDs, all normally belonging to one pseudonym and therefore
/// identified only by [`HoprSurbId`].
///
/// Backed by a [`VecDeque`] pre-allocated to `capacity` and never allowed to exceed it, so it
/// never reallocates: a push into a full buffer evicts the oldest element first. [`SurbPopOrder`]
/// picks which end a pop consumes from; overflow always evicts the oldest, in either order.
#[derive(Clone, Debug)]
pub struct SurbRingBuffer<S> {
    surbs: Arc<parking_lot::Mutex<VecDeque<(HoprSurbId, S)>>>,
    capacity: usize,
    pop_order: SurbPopOrder,
}

impl<S> SurbRingBuffer<S> {
    /// Creates a buffer holding at most `capacity` (min 1, so a push is never a no-op) SURBs,
    /// popped in the given order.
    pub fn new(capacity: usize, pop_order: SurbPopOrder) -> Self {
        let capacity = capacity.max(1);
        Self {
            surbs: Arc::new(parking_lot::Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
            pop_order,
        }
    }

    /// Pushes all SURBs with their IDs, evicting the oldest ones past capacity.
    ///
    /// Returns the number of elements held after the push.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I) -> usize {
        let mut rb = self.surbs.lock();
        for surb in surbs {
            // Evict before inserting, so that the length never exceeds the pre-allocated
            // capacity and the backing allocation stays put.
            if rb.len() == self.capacity {
                rb.pop_front();
            }
            rb.push_back(surb);
        }
        rb.len()
    }

    /// Pops the next SURB and its ID, in the buffer's [`SurbPopOrder`].
    pub fn pop_one(&self) -> Option<PoppedSurb<S>> {
        self.pop_next_valid(|_, _| true)
    }

    /// Pops the next SURB that `is_valid` accepts, in the buffer's [`SurbPopOrder`].
    ///
    /// **Destructive:** rejected entries are discarded, not skipped, so an unusable SURB neither is
    /// handed out nor blocks those behind it. Pass only a validity test — a selective predicate
    /// (say, a routing preference) would drain the buffer. `None` once it is exhausted without a
    /// match.
    ///
    /// `is_valid` runs *outside* the lock: it is caller-supplied and may take locks of its own, so
    /// calling it inside the critical section would invite lock-order inversion.
    pub fn pop_next_valid<F>(&self, is_valid: F) -> Option<PoppedSurb<S>>
    where
        F: Fn(&HoprSurbId, &S) -> bool,
    {
        loop {
            let (id, surb, remaining) = {
                let mut rb = self.surbs.lock();
                let (id, surb) = match self.pop_order {
                    SurbPopOrder::Fifo => rb.pop_front()?,
                    SurbPopOrder::Lifo => rb.pop_back()?,
                };
                (id, surb, rb.len())
            };

            if is_valid(&id, &surb) {
                return Some(PoppedSurb { id, surb, remaining });
            }
        }
    }

    /// Pops the next SURB (in the buffer's [`SurbPopOrder`]) only if it has the given ID.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Option<PoppedSurb<S>> {
        let mut rb = self.surbs.lock();

        let next = match self.pop_order {
            SurbPopOrder::Fifo => rb.front(),
            SurbPopOrder::Lifo => rb.back(),
        };

        if next.is_some_and(|(surb_id, _)| surb_id == id) {
            let (id, surb) = match self.pop_order {
                SurbPopOrder::Fifo => rb.pop_front()?,
                SurbPopOrder::Lifo => rb.pop_back()?,
            };
            Some(PoppedSurb {
                id,
                surb,
                remaining: rb.len(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surb_store_config_should_default_to_fifo() {
        assert_eq!(SurbPopOrder::Fifo, SurbStoreConfig::default().pop_order);
        assert_eq!(SurbPopOrder::Fifo, SurbPopOrder::default());
    }

    #[test]
    fn surb_ring_buffer_should_drop_oldest_items_when_capacity_is_reached() -> anyhow::Result<()> {
        // Eviction always removes the oldest, so both orders see the same surviving set {2,3,4},
        // but consume it from opposite ends.
        for (order, expected) in [
            (SurbPopOrder::Fifo, [[2u8; 8], [3u8; 8], [4u8; 8]]),
            (SurbPopOrder::Lifo, [[4u8; 8], [3u8; 8], [2u8; 8]]),
        ] {
            let rb = SurbRingBuffer::new(3, order);
            rb.push([([1u8; 8], 0)]);
            rb.push([([2u8; 8], 0)]);
            rb.push([([3u8; 8], 0)]);
            rb.push([([4u8; 8], 0)]);

            for (i, expected_id) in expected.into_iter().enumerate() {
                let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop in {order} order"))?;
                assert_eq!(expected_id, popped.id, "{order}: unexpected id at index {i}");
                assert_eq!(
                    expected.len() - 1 - i,
                    popped.remaining,
                    "{order}: unexpected remaining"
                );
            }

            assert!(rb.pop_one().is_none(), "{order}: buffer should be drained");
        }

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_pop_fifo_by_default() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, SurbPopOrder::default());

        let len = rb.push([([1u8; 8], 0)]);
        assert_eq!(1, len);

        let len = rb.push([([2u8; 8], 0)]);
        assert_eq!(2, len);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([1u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        let len = rb.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert_eq!(2, len);

        assert_eq!([1u8; 8], rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!([2u8; 8], rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_pop_lifo_when_configured() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, SurbPopOrder::Lifo);

        let len = rb.push([([1u8; 8], 0)]);
        assert_eq!(1, len);

        let len = rb.push([([2u8; 8], 0)]);
        assert_eq!(2, len);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([1u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        let len = rb.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert_eq!(2, len);

        assert_eq!([2u8; 8], rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!([1u8; 8], rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_skip_entries_failing_the_predicate() -> anyhow::Result<()> {
        for order in [SurbPopOrder::Fifo, SurbPopOrder::Lifo] {
            let rb = SurbRingBuffer::new(5, order);
            rb.push([([1u8; 8], 0), ([2u8; 8], 0), ([3u8; 8], 0)]);

            // Only the middle entry is acceptable, so the two rejected ones must be discarded.
            let popped = rb
                .pop_next_valid(|id, _| id == &[2u8; 8])
                .ok_or(anyhow::anyhow!("expected pop in {order} order"))?;
            assert_eq!([2u8; 8], popped.id, "{order}");

            // The rejected entries are gone, not merely skipped over.
            assert_eq!(1, popped.remaining, "{order}");
            assert!(rb.pop_next_valid(|id, _| id == &[2u8; 8]).is_none(), "{order}");
        }

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_return_none_when_no_entry_satisfies_the_predicate() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, SurbPopOrder::Lifo);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)]);

        assert!(rb.pop_next_valid(|_, _| false).is_none());
        // The buffer is fully drained by the exhaustive search.
        assert!(rb.pop_one().is_none());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_not_reallocate_under_steady_overflow() -> anyhow::Result<()> {
        for order in [SurbPopOrder::Fifo, SurbPopOrder::Lifo] {
            let rb = SurbRingBuffer::new(8, order);
            let initial_capacity = rb.surbs.lock().capacity();

            for i in 0..1_000u32 {
                rb.push([(((i as u64).to_be_bytes()), 0)]);
                if i % 3 == 0 {
                    rb.pop_one();
                }
                assert!(rb.surbs.lock().len() <= 8, "{order}: length exceeded capacity");
            }

            assert_eq!(
                initial_capacity,
                rb.surbs.lock().capacity(),
                "{order}: buffer reallocated"
            );
        }

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_not_pop_if_id_does_not_match() -> anyhow::Result<()> {
        for order in [SurbPopOrder::Fifo, SurbPopOrder::Lifo] {
            let rb = SurbRingBuffer::new(5, order);

            rb.push([([1u8; 8], 0)]);

            assert!(rb.pop_one_if_has_id(&[2u8; 8]).is_none(), "{order}");
            assert_eq!(
                [1u8; 8],
                rb.pop_one_if_has_id(&[1u8; 8])
                    .ok_or(anyhow::anyhow!("expected pop in {order} order"))?
                    .id,
                "{order}"
            );
        }

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_check_the_popping_end_for_an_exact_id() -> anyhow::Result<()> {
        let fifo = SurbRingBuffer::new(5, SurbPopOrder::Fifo);
        fifo.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert!(fifo.pop_one_if_has_id(&[2u8; 8]).is_none());
        assert_eq!(
            [1u8; 8],
            fifo.pop_one_if_has_id(&[1u8; 8])
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        let lifo = SurbRingBuffer::new(5, SurbPopOrder::Lifo);
        lifo.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert!(lifo.pop_one_if_has_id(&[1u8; 8]).is_none());
        assert_eq!(
            [2u8; 8],
            lifo.pop_one_if_has_id(&[2u8; 8])
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        Ok(())
    }
}
