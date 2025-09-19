use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};

use hopr_crypto_packet::{
    HoprSurb, ReplyOpener,
    prelude::{HoprSenderId, HoprSurbId},
};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::{balance::HoprBalance, prelude::U256};
use moka::{Expiry, future::Cache, notification::RemovalCause};
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::errors::NodeDbError;

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
pub(crate) struct SurbRingBuffer<S>(Arc<parking_lot::Mutex<AllocRingBuffer<(HoprSurbId, S)>>>);

impl<S> Default for SurbRingBuffer<S> {
    fn default() -> Self {
        // With the current packet size, this is almost 10 MB of data budget in SURBs
        Self::new(10_000)
    }
}

impl<S> SurbRingBuffer<S> {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(parking_lot::Mutex::new(AllocRingBuffer::new(capacity))))
    }

    /// Push all SURBs with their IDs into the RB.
    ///
    /// Returns the total number of elements in the RB after the push.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I) -> Result<usize, NodeDbError> {
        let mut rb = self.0.lock();

        rb.extend(surbs);
        Ok(rb.len())
    }

    /// Pop the latest SURB and its IDs from the RB.
    pub fn pop_one(&self) -> Result<PoppedSurb<S>, NodeDbError> {
        let mut rb = self.0.lock();

        let (id, surb) = rb
            .dequeue()
            .ok_or(NodeDbError::NoSurbAvailable("no more surbs".into()))?;
        Ok(PoppedSurb {
            id,
            surb,
            remaining: rb.len(),
        })
    }

    /// Check if the next SURB has the given ID and pop it from the RB.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Result<PoppedSurb<S>, NodeDbError> {
        let mut rb = self.0.lock();

        if rb.peek().is_some_and(|(surb_id, _)| surb_id == id) {
            let (id, surb) = rb
                .dequeue()
                .ok_or(NodeDbError::NoSurbAvailable("no more surbs".into()))?;
            Ok(PoppedSurb {
                id,
                surb,
                remaining: rb.len(),
            })
        } else {
            Err(NodeDbError::NoSurbAvailable("surb does not match the given id".into()))
        }
    }
}

struct ExpiryNever;

impl<K, V> Expiry<K, V> for ExpiryNever {
    fn expire_after_create(&self, _key: &K, _value: &V, _current_time: std::time::Instant) -> Option<Duration> {
        None
    }
}

/// Contains all caches used by the [crate::db::HoprDb].
#[derive(Debug)]
pub struct NodeDbCaches {
    pub(crate) unacked_tickets: Cache<HalfKeyChallenge, PendingAcknowledgement>,
    pub(crate) ticket_index: Cache<Hash, Arc<AtomicU64>>,
    // key is (channel_id, channel_epoch) to ensure calculation of unrealized value does not
    // include tickets from other epochs
    pub(crate) unrealized_value: Cache<(Hash, U256), HoprBalance>,
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    pub(crate) surbs_per_pseudonym: Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
}

impl Default for NodeDbCaches {
    fn default() -> Self {
        Self {
            unacked_tickets: Cache::builder()
                .time_to_live(Duration::from_secs(30))
                .max_capacity(1_000_000_000)
                .build(),
            ticket_index: Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build(),
            unrealized_value: Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build(),
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                .time_to_idle(Duration::from_secs(600))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|sender_id, _reply_opener, cause| {
                    tracing::warn!(?sender_id, ?cause, "evicting reply opener for pseudonym");
                })
                .max_capacity(10_000)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: Cache::builder()
                .time_to_idle(Duration::from_secs(600))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|pseudonym, _reply_opener, cause| {
                    tracing::warn!(%pseudonym, ?cause, "evicting surb for pseudonym");
                })
                .max_capacity(10_000)
                .build(),
        }
    }
}

impl NodeDbCaches {
    pub(crate) fn insert_pseudonym_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), move || {
                moka::sync::Cache::builder()
                    .time_to_live(Duration::from_secs(3600))
                    .eviction_listener(move |id: Arc<HoprSurbId>, _, cause| {
                        if cause != RemovalCause::Explicit {
                            tracing::warn!(pseudonym = %sender_id.pseudonym(), surb_id = hex::encode(id.as_slice()), ?cause, "evicting reply opener for sender id");
                        }
                    })
                    .max_capacity(100_000)
                    .build()
            })
            .insert(sender_id.surb_id(), opener);
    }

    pub(crate) fn extract_pseudonym_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener> {
        self.pseudonym_openers
            .get(&sender_id.pseudonym())
            .and_then(|cache| cache.remove(&sender_id.surb_id()))
    }

    // For future use by the SessionManager
    #[allow(dead_code)]
    pub(crate) fn invalidate_pseudonym_openers(&self, pseudonym: &HoprPseudonym) {
        self.pseudonym_openers.invalidate(pseudonym);
    }

    /// Invalidates all caches.
    pub fn invalidate_all(&self) {
        self.unacked_tickets.invalidate_all();
        self.unrealized_value.invalidate_all();
        self.surbs_per_pseudonym.invalidate_all();
        self.pseudonym_openers.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surb_ring_buffer_must_drop_items_when_capacity_is_reached() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3);
        rb.push([([1u8; 8], 0)])?;
        rb.push([([2u8; 8], 0)])?;
        rb.push([([3u8; 8], 0)])?;
        rb.push([([4u8; 8], 0)])?;

        let popped = rb.pop_one()?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(2, popped.remaining);

        let popped = rb.pop_one()?;
        assert_eq!([3u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_one()?;
        assert_eq!([4u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        assert!(rb.pop_one().is_err());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_must_be_fifo() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        let len = rb.push([([1u8; 8], 0)])?;
        assert_eq!(1, len);

        let len = rb.push([([2u8; 8], 0)])?;
        assert_eq!(2, len);

        let popped = rb.pop_one()?;
        assert_eq!([1u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_one()?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        let len = rb.push([([1u8; 8], 0), ([2u8; 8], 0)])?;
        assert_eq!(2, len);

        assert_eq!([1u8; 8], rb.pop_one()?.id);
        assert_eq!([2u8; 8], rb.pop_one()?.id);

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_must_not_pop_if_id_does_not_match() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        rb.push([([1u8; 8], 0)])?;

        assert!(rb.pop_one_if_has_id(&[2u8; 8]).is_err());
        assert_eq!([1u8; 8], rb.pop_one_if_has_id(&[1u8; 8])?.id);

        Ok(())
    }
}
