use std::{sync::Arc, time::Duration};

use hopr_crypto_packet::{
    HoprSurb, ReplyOpener,
    prelude::{HoprSenderId, HoprSurbId},
};
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::SurbMatcher;
use moka::{future::Cache, notification::RemovalCause};
use ringbuffer::RingBuffer;

use crate::{FoundSurb, traits::SurbStore};

/// Configuration for the SURB cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, smart_default::SmartDefault)]
pub struct SurbStoreConfig {
    /// Size of the SURB ring buffer per pseudonym.
    #[default(10_000)]
    pub rb_capacity: usize,
    /// Threshold for the number of SURBs in the ring buffer, below which it is
    /// considered low ("SURB distress").
    #[default(500)]
    pub distress_threshold: usize,
}

#[derive(Clone)]
pub struct MemorySurbStore {
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    surbs_per_pseudonym: Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
    cfg: SurbStoreConfig,
}

impl MemorySurbStore {
    pub fn new(cfg: SurbStoreConfig) -> Self {
        Self {
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                // TODO: Expose this as a config option
                .time_to_idle(Duration::from_secs(600))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|sender_id, _reply_opener, cause| {
                    tracing::warn!(?sender_id, ?cause, "evicting reply opener for pseudonym");
                })
                // TODO: Expose this as a config option
                .max_capacity(10_000)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: Cache::builder()
                // TODO: Expose this as a config option
                .time_to_idle(Duration::from_secs(600))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|pseudonym, _reply_opener, cause| {
                    tracing::warn!(%pseudonym, ?cause, "evicting surb for pseudonym");
                })
                // TODO: Expose this as a config option
                .max_capacity(10_000)
                .build(),
            cfg,
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
    async fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb> {
        let pseudonym = matcher.pseudonym();
        let surbs_for_pseudonym = self.surbs_per_pseudonym.get(&pseudonym).await?;

        match matcher {
            SurbMatcher::Pseudonym(_) => surbs_for_pseudonym.pop_one().map(|popped_surb| FoundSurb {
                sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                surb: popped_surb.surb,
                remaining: popped_surb.remaining,
            }),
            // The following code intentionally only checks the first SURB in the ring buffer
            // and does not search the entire RB.
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

    async fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize {
        self.surbs_per_pseudonym
            .entry_by_ref(&pseudonym)
            .or_insert_with(futures::future::lazy(|_| {
                SurbRingBuffer::new(self.cfg.rb_capacity.max(1024))
            }))
            .await
            .value()
            .push(surbs)
    }

    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), move || {
                moka::sync::Cache::builder()
                    // TODO: Expose this as a config option
                    .time_to_live(Duration::from_secs(3600))
                    .eviction_listener(move |id: Arc<HoprSurbId>, _, cause| {
                        if cause != RemovalCause::Explicit {
                            tracing::warn!(
                                pseudonym = %sender_id.pseudonym(),
                                surb_id = hex::encode(id.as_slice()),
                                ?cause,
                                "evicting reply opener for sender id"
                            );
                        }
                    })
                    .max_capacity(100_000)
                    .build()
            })
            .insert(sender_id.surb_id(), opener);
    }

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

/// Ring buffer containing SURBs along with their IDs.
///
/// All these SURBs usually belong to the same pseudonym and are therefore identified
/// only by the [`HoprSurbId`].
#[derive(Clone, Debug)]
pub struct SurbRingBuffer<S>(Arc<parking_lot::Mutex<ringbuffer::AllocRingBuffer<(HoprSurbId, S)>>>);

impl<S> SurbRingBuffer<S> {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(parking_lot::Mutex::new(ringbuffer::AllocRingBuffer::new(
            capacity,
        ))))
    }

    /// Push all SURBs with their IDs into the RB.
    ///
    /// Returns the total number of elements in the RB after the push.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I) -> usize {
        let mut rb = self.0.lock();
        rb.extend(surbs);
        rb.len()
    }

    /// Pop the latest SURB and its IDs from the RB.
    pub fn pop_one(&self) -> Option<PoppedSurb<S>> {
        let mut rb = self.0.lock();
        let (id, surb) = rb.dequeue()?;
        Some(PoppedSurb {
            id,
            surb,
            remaining: rb.len(),
        })
    }

    /// Check if the next SURB has the given ID and pop it from the RB.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Option<PoppedSurb<S>> {
        let mut rb = self.0.lock();

        if rb.peek().is_some_and(|(surb_id, _)| surb_id == id) {
            let (id, surb) = rb.dequeue()?;
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
    fn surb_ring_buffer_must_drop_items_when_capacity_is_reached() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3);
        rb.push([([1u8; 8], 0)]);
        rb.push([([2u8; 8], 0)]);
        rb.push([([3u8; 8], 0)]);
        rb.push([([4u8; 8], 0)]);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(2, popped.remaining);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([3u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([4u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        assert!(rb.pop_one().is_none());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_must_be_fifo() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

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
    fn surb_ring_buffer_must_not_pop_if_id_does_not_match() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        rb.push([([1u8; 8], 0)]);

        assert!(rb.pop_one_if_has_id(&[2u8; 8]).is_none());
        assert_eq!(
            [1u8; 8],
            rb.pop_one_if_has_id(&[1u8; 8])
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        Ok(())
    }
}
