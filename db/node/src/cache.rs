use std::sync::Arc;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use hopr_crypto_packet::prelude::HoprSurbId;

use std::{
    sync::{atomic::AtomicU64},
    time::Duration,
};

use hopr_crypto_packet::{
    HoprSurb, ReplyOpener,
    prelude::{HoprSenderId},
};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::{
    balance::HoprBalance,
    prelude::U256,
};
use moka::{Expiry, future::Cache, notification::RemovalCause};
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

        let (id, surb) = rb.dequeue().ok_or(NodeDbError::NoSurbAvailable("no more surbs".into()))?;
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
            let (id, surb) = rb.dequeue().ok_or(NodeDbError::NoSurbAvailable("no more surbs".into()))?;
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
pub struct HoprDbCaches {
    pub(crate) unacked_tickets: Cache<HalfKeyChallenge, PendingAcknowledgement>,
    pub(crate) ticket_index: Cache<Hash, Arc<AtomicU64>>,
    // key is (channel_id, channel_epoch) to ensure calculation of unrealized value does not
    // include tickets from other epochs
    pub(crate) unrealized_value: Cache<(Hash, U256), HoprBalance>,
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    pub(crate) surbs_per_pseudonym: Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
}

impl Default for HoprDbCaches {
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

impl HoprDbCaches {
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

// TODO: (dbmig) move this into the implementation of ChainKeyOperations
/*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ChannelParties(pub(crate) Address, pub(crate) Address);

#[derive(Debug)]
pub(crate) struct CacheKeyMapper(
    DashMap<KeyIdent<4>, OffchainPublicKey>,
    DashMap<OffchainPublicKey, KeyIdent<4>>,
);

impl CacheKeyMapper {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(DashMap::with_capacity(capacity), DashMap::with_capacity(capacity))
    }

    /// Creates key id mapping for a public key of an [account](AccountEntry).
    ///
    /// Does nothing if the binding already exists. Returns error if an existing binding
    /// is not consistent.
    pub fn update_key_id_binding(&self, account: &AccountEntry) -> Result<(), DbError> {
        let id = account.key_id();
        let key = account.public_key;

        // Lock entries in the maps to avoid concurrent modifications
        let id_entry = self.0.entry(id);
        let key_entry = self.1.entry(key);

        match (id_entry, key_entry) {
            (Entry::Vacant(v_id), Entry::Vacant(v_key)) => {
                v_id.insert_entry(key);
                v_key.insert_entry(id);
                tracing::debug!(%id, %key, "inserted key-id binding");
                Ok(())
            }
            (Entry::Occupied(v_id), Entry::Occupied(v_key)) => {
                // Check if the existing binding is consistent with the new one.
                if v_id.get() != v_key.key() {
                    Err(DbError::LogicalError(format!(
                        "attempt to insert key {key} with key-id {id}, but key-id already maps to key {} while {} is \
                         expected",
                        v_id.get(),
                        v_key.key(),
                    )))
                } else {
                    Ok(())
                }
            }
            // This only happens on re-announcements:
            // The re-announcement uses the same packet key and chain-key, but the block number (published at)
            // is different, and therefore the id_entry will be vacant.
            (Entry::Vacant(_), Entry::Occupied(v_key)) => {
                tracing::debug!(
                    "attempt to insert key {key} with key-id {id} failed because key is already set as {}",
                    v_key.get()
                );
                Err(DbError::LogicalError("inconsistent key-id binding".into()))
            }
            // This should never happen.
            (Entry::Occupied(v_id), Entry::Vacant(_)) => {
                tracing::debug!(
                    "attempt to insert key {key} with key-id {id} failed because key-id is already set as {}",
                    v_id.get()
                );
                Err(DbError::LogicalError("inconsistent key-id binding".into()))
            }
        }
    }
}

impl hopr_crypto_packet::KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec> for CacheKeyMapper {
    fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<KeyIdent> {
        self.1.get(key).map(|k| *k.value())
    }

    fn map_id_to_public(&self, id: &KeyIdent) -> Option<OffchainPublicKey> {
        self.0.get(id).map(|k| *k.value())
    }
}


*/

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