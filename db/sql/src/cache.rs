use crate::errors::DbSqlError;
use dashmap::{DashMap, Entry};
use hopr_crypto_packet::{HoprPseudonym, HoprSphinxHeaderSpec, HoprSphinxSuite, HoprSurb, ReplyOpener};
use hopr_crypto_types::prelude::*;
use hopr_db_api::info::{IndexerData, SafeInfo};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{Address, Balance, KeyIdent, U256};
use moka::future::Cache;
use moka::Expiry;
use ringbuffer::AllocRingBuffer;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Enumerates all singular data that can be cached and
/// cannot be represented by a key. These values can be cached for the long term.
#[derive(Debug, Clone, PartialEq, strum::EnumDiscriminants)]
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

/// Contains all caches used by the [crate::db::HoprDb].
#[derive(Debug)]
pub struct HoprDbCaches {
    pub(crate) single_values: Cache<CachedValueDiscriminants, CachedValue>,
    pub(crate) unacked_tickets: Cache<HalfKeyChallenge, PendingAcknowledgement>,
    pub(crate) ticket_index: Cache<Hash, Arc<AtomicU64>>,
    // key is (channel_id, channel_epoch) to ensure calculation of unrealized value does not
    // include tickets from other epochs
    pub(crate) unrealized_value: Cache<(Hash, U256), Balance>,
    pub(crate) chain_to_offchain: Cache<Address, Option<OffchainPublicKey>>,
    pub(crate) offchain_to_chain: Cache<OffchainPublicKey, Option<Address>>,
    pub(crate) src_dst_to_channel: Cache<ChannelParties, Option<ChannelEntry>>,
    // KeyIdMapper must be synchronous because it is used from a sync context.
    pub(crate) key_id_mapper: CacheKeyMapper,
    pub(crate) pseudonym_openers: moka::sync::Cache<HoprPseudonym, ReplyOpener>,
    pub(crate) surbs_per_pseudonym: Cache<HoprPseudonym, Arc<Mutex<AllocRingBuffer<HoprSurb>>>>,
}

impl Default for HoprDbCaches {
    fn default() -> Self {
        let single_values = Cache::builder().time_to_idle(Duration::from_secs(1800)).build();

        let unacked_tickets = Cache::builder()
            .time_to_live(Duration::from_secs(30))
            .max_capacity(1_000_000_000)
            .build();

        let ticket_index = Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build();

        let unrealized_value = Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build();

        let chain_to_offchain = Cache::builder()
            .time_to_idle(Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

        let offchain_to_chain = Cache::builder()
            .time_to_idle(Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

        let src_dst_to_channel = Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .max_capacity(10_000)
            .build();

        let pseudonym_openers = moka::sync::Cache::builder()
            .time_to_live(Duration::from_secs(60))
            .max_capacity(100_000)
            .build();

        let surbs_per_pseudonym = Cache::builder()
            .time_to_idle(Duration::from_secs(60))
            .max_capacity(100_000)
            .build();

        Self {
            single_values,
            unacked_tickets,
            ticket_index,
            unrealized_value,
            chain_to_offchain,
            offchain_to_chain,
            src_dst_to_channel,
            pseudonym_openers,
            surbs_per_pseudonym,
            key_id_mapper: CacheKeyMapper::with_capacity(10_000),
        }
    }
}

impl HoprDbCaches {
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
        match self.0.entry(id) {
            Entry::Vacant(v) => {
                v.insert(account.public_key);
                self.1.insert(account.public_key, id);
                tracing::debug!(%id, %account.public_key, "inserted key-id binding");
                Ok(())
            }
            Entry::Occupied(v) => {
                // Check if the existing binding is consistent with the new one.
                if v.get() != &account.public_key {
                    Err(DbSqlError::LogicalError(format!(
                        "attempt to insert key {} with key-id {id} already exists for the key {}",
                        account.public_key,
                        v.get()
                    )))
                } else {
                    Ok(())
                }
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
