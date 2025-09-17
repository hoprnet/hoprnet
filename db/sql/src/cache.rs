use std::time::Duration;

use dashmap::{DashMap, Entry};
use hopr_crypto_packet::{HoprSphinxHeaderSpec, HoprSphinxSuite};
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::{AccountEntry, ChannelEntry};
use hopr_primitive_types::prelude::{Address, KeyIdent};
use moka::future::Cache;

use crate::{errors::DbSqlError, info::IndexerData};

/// Lists all singular data that can be cached and
/// cannot be represented by a key. These values can be cached for the long term.
#[derive(Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum CachedValue {
    /// Cached [IndexerData].
    IndexerDataCache(IndexerData),
}

impl TryFrom<CachedValue> for IndexerData {
    type Error = DbSqlError;

    fn try_from(value: CachedValue) -> Result<Self, Self::Error> {
        match value {
            CachedValue::IndexerDataCache(data) => Ok(data),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ChannelParties(pub(crate) Address, pub(crate) Address);


// TODO: (dbmig) move this into the implementation of ChainKeyOperations
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

/// Contains all caches used by the [crate::db::HoprDb].
#[derive(Debug, Clone)]
pub struct HoprDbCaches {
    pub(crate) single_values: Cache<CachedValueDiscriminants, CachedValue>,
    pub(crate) chain_to_offchain: Cache<Address, Option<OffchainPublicKey>>,
    pub(crate) offchain_to_chain: Cache<OffchainPublicKey, Option<Address>>,
    pub(crate) src_dst_to_channel: Cache<ChannelParties, Option<ChannelEntry>>,
    // KeyIdMapper must be synchronous because it is used from a sync context.
    pub(crate) key_id_mapper: std::sync::Arc<CacheKeyMapper>,
}

impl Default for HoprDbCaches {
    fn default() -> Self {
        Self {
            single_values: Cache::builder().time_to_idle(Duration::from_secs(1800)).build(),
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
            key_id_mapper: std::sync::Arc::new(CacheKeyMapper::with_capacity(10_000)),
        }
    }
}

#[cfg(test)]
impl HoprDbCaches {
    pub fn invalidate_all(&self) {
        self.src_dst_to_channel.invalidate_all();
        self.chain_to_offchain.invalidate_all();
        self.offchain_to_chain.invalidate_all();
        self.single_values.invalidate_all();
    }
}
