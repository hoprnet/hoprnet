use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};

use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::ChannelId;
use hopr_primitive_types::{balance::HoprBalance, prelude::U256};
use moka::{Expiry, future::Cache};

struct ExpiryNever;

impl<K, V> Expiry<K, V> for ExpiryNever {
    fn expire_after_create(&self, _key: &K, _value: &V, _current_time: std::time::Instant) -> Option<Duration> {
        None
    }
}

/// Contains all caches used by the [crate::db::HoprDb].
#[derive(Debug)]
pub struct NodeDbCaches {
    pub(crate) ticket_index: Cache<ChannelId, Arc<AtomicU64>>,
    // key is (channel_id, channel_epoch) to ensure calculation of unrealized value does not
    // include tickets from other epochs
    pub(crate) unrealized_value: Cache<(ChannelId, U256), HoprBalance>,
}

impl Default for NodeDbCaches {
    fn default() -> Self {
        Self {
            ticket_index: Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build(),
            unrealized_value: Cache::builder().expire_after(ExpiryNever).max_capacity(10_000).build(),
        }
    }
}

impl NodeDbCaches {
    /// Invalidates all caches.
    pub fn invalidate_all(&self) {
        self.unrealized_value.invalidate_all();
    }
}
