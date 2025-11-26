use hopr_internal_types::prelude::ChannelId;
use hopr_primitive_types::balance::HoprBalance;
use moka::future::Cache;

/// Contains all caches used by the [crate::db::HoprDb].
#[derive(Debug)]
pub struct NodeDbCaches {
    // key is (channel_id, channel_epoch) to ensure calculation of unrealized value does not
    // include tickets from other epochs
    pub(crate) unrealized_value: Cache<(ChannelId, u32), HoprBalance>,
}

impl Default for NodeDbCaches {
    fn default() -> Self {
        Self {
            unrealized_value: Cache::builder().max_capacity(10_000).build(),
        }
    }
}

impl NodeDbCaches {
    /// Invalidates all caches.
    pub fn invalidate_all(&self) {
        self.unrealized_value.invalidate_all();
    }
}
