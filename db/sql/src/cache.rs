use std::{
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::Duration,
};

use dashmap::{DashMap, Entry};
use hopr_crypto_packet::{
    HoprSphinxHeaderSpec, HoprSphinxSuite, HoprSurb, ReplyOpener,
    prelude::{HoprSenderId, HoprSurbId},
};
use hopr_crypto_types::prelude::*;
use hopr_db_api::{
    info::{IndexerData, SafeInfo},
    prelude::DbError,
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::{
    balance::HoprBalance,
    prelude::{Address, KeyIdent, U256},
};
use moka::{Expiry, future::Cache};
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::errors::DbSqlError;

/// Lists all singular data that can be cached and
/// cannot be represented by a key. These values can be cached for the long term.
#[derive(Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum CachedValue {
    /// Cached [IndexerData].
    IndexerDataCache(IndexerData),
    /// Cached [SafeInfo].
    SafeInfoCache(Option<SafeInfo>),
}

impl TryFrom<CachedValue> for IndexerData {
    type Error = DbSqlError;

    fn try_from(value: CachedValue) -> Result<Self, Self::Error> {
        match value {
            CachedValue::IndexerDataCache(data) => Ok(data),
            _ => Err(DbSqlError::DecodingError),
        }
    }
}

impl TryFrom<CachedValue> for Option<SafeInfo> {
    type Error = DbSqlError;

    fn try_from(value: CachedValue) -> Result<Self, Self::Error> {
        match value {
            CachedValue::SafeInfoCache(data) => Ok(data),
            _ => Err(DbSqlError::DecodingError),
        }
    }
}

struct ExpiryNever;

impl<K, V> Expiry<K, V> for ExpiryNever {
    fn expire_after_create(&self, _key: &K, _value: &V, _current_time: std::time::Instant) -> Option<Duration> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ChannelParties(pub(crate) Address, pub(crate) Address);

/// Ring buffer containing SURBs along with their IDs.
/// All these SURBs usually belong to the same pseudonym.
#[derive(Clone, Debug)]
pub(crate) struct SurbRingBuffer<S>(Arc<Mutex<AllocRingBuffer<(HoprSurbId, S)>>>);

impl<S> Default for SurbRingBuffer<S> {
    fn default() -> Self {
        // With the current packet size, this is almost 10 MB of data budget in SURBs
        Self::new(10_000)
    }
}

impl<S> SurbRingBuffer<S> {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(Mutex::new(AllocRingBuffer::new(capacity))))
    }

    /// Push all SURBs with their IDs into the RB.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I) -> Result<(), DbError> {
        self.0
            .lock()
            .map_err(|_| DbError::LogicalError("failed to lock surbs".into()))?
            .extend(surbs);
        Ok(())
    }

    /// Pop the latest SURB and its IDs from the RB.
    pub fn pop_one(&self) -> Result<(HoprSurbId, S), DbError> {
        self.0
            .lock()
            .map_err(|_| DbError::LogicalError("failed to lock surbs".into()))?
            .dequeue()
            .ok_or(DbError::NoSurbAvailable("no more surbs".into()))
    }

    /// Check if the next SURB has the given ID and pop it from the RB.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Result<(HoprSurbId, S), DbError> {
        let mut rb = self
            .0
            .lock()
            .map_err(|_| DbError::LogicalError("failed to lock surbs".into()))?;
        if rb.peek().is_some_and(|(surb_id, _)| surb_id == id) {
            rb.dequeue().ok_or(DbError::NoSurbAvailable("no more surbs".into()))
        } else {
            Err(DbError::NoSurbAvailable("surb does not match the given id".into()))
        }
    }
}

/// Contains all caches used by the [crate::db::HoprDb].
#[derive(Debug)]
pub struct HoprDbCaches {
    pub(crate) single_values: Cache<CachedValueDiscriminants, CachedValue>,
    pub(crate) unacked_tickets: Cache<HalfKeyChallenge, PendingAcknowledgement>,
    pub(crate) ticket_index: Cache<Hash, Arc<AtomicU64>>,
    // key is (channel_id, channel_epoch) to ensure calculation of unrealized value does not
    // include tickets from other epochs
    pub(crate) unrealized_value: Cache<(Hash, U256), HoprBalance>,
    pub(crate) chain_to_offchain: Cache<Address, Option<OffchainPublicKey>>,
    pub(crate) offchain_to_chain: Cache<OffchainPublicKey, Option<Address>>,
    pub(crate) src_dst_to_channel: Cache<ChannelParties, Option<ChannelEntry>>,
    // KeyIdMapper must be synchronous because it is used from a sync context.
    pub(crate) key_id_mapper: CacheKeyMapper,
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    pub(crate) surbs_per_pseudonym: Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
}

impl Default for HoprDbCaches {
    fn default() -> Self {
        Self {
            single_values: Cache::builder().time_to_idle(Duration::from_secs(1800)).build(),
            unacked_tickets: Cache::builder()
                .time_to_live(Duration::from_secs(30))
                .max_capacity(1_000_000_000)
                .build(),
            ticket_index: Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build(),
            unrealized_value: Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build(),
            chain_to_offchain: Cache::builder()
                .time_to_idle(Duration::from_secs(600))
                .max_capacity(100_000)
                .build(),
            offchain_to_chain: Cache::builder()
                .time_to_idle(Duration::from_secs(600))
                .max_capacity(100_000)
                .build(),
            src_dst_to_channel: Cache::builder()
                .time_to_live(Duration::from_secs(600))
                .max_capacity(10_000)
                .build(),
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                .time_to_idle(Duration::from_secs(600))
                .eviction_listener(|sender_id, _reply_opener, cause| {
                    tracing::trace!(?sender_id, ?cause, "evicting reply opener for pseudonym");
                })
                .max_capacity(10_000)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: Cache::builder()
                .time_to_idle(Duration::from_secs(600))
                .eviction_listener(|pseudonym, _reply_opener, cause| {
                    tracing::trace!(%pseudonym, ?cause, "evicting surb for pseudonym");
                })
                .max_capacity(10_000)
                .build(),
            key_id_mapper: CacheKeyMapper::with_capacity(10_000),
        }
    }
}

impl HoprDbCaches {
    pub(crate) fn insert_pseudonym_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), || {
                moka::sync::Cache::builder()
                    .time_to_live(Duration::from_secs(3600))
                    .eviction_listener(|sender_id, _reply_opener, cause| {
                        tracing::trace!(?sender_id, ?cause, "evicting reply opener for sender id");
                    })
                    .max_capacity(10_000)
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
        self.single_values.invalidate_all();
        self.unacked_tickets.invalidate_all();
        self.unrealized_value.invalidate_all();
        self.chain_to_offchain.invalidate_all();
        self.offchain_to_chain.invalidate_all();
        self.src_dst_to_channel.invalidate_all();
        // NOTE: key_id_mapper intentionally not invalidated
    }
}

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
    pub fn update_key_id_binding(&self, account: &AccountEntry) -> Result<(), DbSqlError> {
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
                    Err(DbSqlError::LogicalError(format!(
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
                Err(DbSqlError::LogicalError("inconsistent key-id binding".into()))
            }
            // This should never happen.
            (Entry::Occupied(v_id), Entry::Vacant(_)) => {
                tracing::debug!(
                    "attempt to insert key {key} with key-id {id} failed because key-id is already set as {}",
                    v_id.get()
                );
                Err(DbSqlError::LogicalError("inconsistent key-id binding".into()))
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

        assert_eq!([2u8; 8], rb.pop_one()?.0);
        assert_eq!([3u8; 8], rb.pop_one()?.0);
        assert_eq!([4u8; 8], rb.pop_one()?.0);
        assert!(rb.pop_one().is_err());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_must_be_fifo() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        rb.push([([1u8; 8], 0)])?;
        rb.push([([2u8; 8], 0)])?;

        assert_eq!([1u8; 8], rb.pop_one()?.0);
        assert_eq!([2u8; 8], rb.pop_one()?.0);

        rb.push([([1u8; 8], 0), ([2u8; 8], 0)])?;

        assert_eq!([1u8; 8], rb.pop_one()?.0);
        assert_eq!([2u8; 8], rb.pop_one()?.0);

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_must_not_pop_if_id_does_not_match() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        rb.push([([1u8; 8], 0)])?;

        assert!(rb.pop_one_if_has_id(&[2u8; 8]).is_err());
        assert_eq!([1u8; 8], rb.pop_one_if_has_id(&[1u8; 8])?.0);

        Ok(())
    }
}
