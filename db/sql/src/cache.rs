use crate::errors::DbSqlError;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{Address, Balance};
use moka::future::Cache;
use moka::Expiry;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use hopr_db_api::info::{IndexerData, SafeInfo};

/// Enumerates all singular data that can be cached and
/// cannot be represented by a key. These values can be cached for long term.
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
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

/// Contains all caches used by the [crate::db::HoprDb].
#[derive(Debug, Clone)]
pub struct HoprDbCaches {
    pub(crate) single_values: Cache<CachedValueDiscriminants, CachedValue>,
    pub(crate) unacked_tickets: Cache<HalfKeyChallenge, PendingAcknowledgement>,
    pub(crate) ticket_index: Cache<Hash, Arc<AtomicU64>>,
    pub(crate) unrealized_value: Cache<Hash, Balance>,
    pub(crate) chain_to_offchain: Cache<Address, Option<OffchainPublicKey>>,
    pub(crate) offchain_to_chain: Cache<OffchainPublicKey, Option<Address>>,
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
            .time_to_live(Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

        let offchain_to_chain = Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

        Self {
            single_values,
            unacked_tickets,
            ticket_index,
            unrealized_value,
            chain_to_offchain,
            offchain_to_chain,
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
    }
}
