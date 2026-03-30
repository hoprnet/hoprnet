use std::sync::atomic::{AtomicBool, AtomicU64};

use hopr_api::{
    chain::{ChannelId, HoprBalance},
    types::internal::prelude::TicketBuilder,
};

use crate::{OutgoingIndexStore, TicketQueue};

/// Tracks outgoing ticket indices for a channel, starting from 0.
#[derive(Debug)]
struct OutgoingIndexEntry {
    index: AtomicU64,
    is_dirty: AtomicBool,
}

impl Default for OutgoingIndexEntry {
    fn default() -> Self {
        Self::new(0)
    }
}

impl OutgoingIndexEntry {
    /// Creates a new index entry and marks it as dirty.
    fn new(index: u64) -> Self {
        OutgoingIndexEntry {
            index: AtomicU64::new(index),
            is_dirty: AtomicBool::new(true),
        }
    }

    /// Increments the index and marks it as dirty if within bounds.
    ///
    /// The value returned is the value before the increment, saturating at [`TicketBuilder::MAX_TICKET_INDEX`].
    fn increment(&self) -> u64 {
        let v = self.index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if v <= TicketBuilder::MAX_TICKET_INDEX {
            self.is_dirty.store(true, std::sync::atomic::Ordering::Release);
        }
        v.min(TicketBuilder::MAX_TICKET_INDEX)
    }

    /// Sets the index to the maximum of `new_value` and the current index value.
    ///
    /// Marks the index as dirty if the value was increased.
    ///
    /// Returns the new value of the index, saturating at [`TicketBuilder::MAX_TICKET_INDEX`].
    fn set(&self, new_value: u64) -> u64 {
        let current = self.index.fetch_max(new_value, std::sync::atomic::Ordering::Relaxed);
        if current < new_value {
            self.is_dirty.store(true, std::sync::atomic::Ordering::Release);
        }
        new_value.max(current).min(TicketBuilder::MAX_TICKET_INDEX)
    }

    /// Checks if the index is marked as dirty.
    fn is_dirty(&self) -> bool {
        self.is_dirty.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Marks the index as clean.
    fn mark_clean(&self) {
        self.is_dirty.store(false, std::sync::atomic::Ordering::Release);
    }

    /// Gets the index.
    ///
    /// The returned value will always be less than [`TicketBuilder::MAX_TICKET_INDEX`].
    fn get(&self) -> u64 {
        self.index
            .load(std::sync::atomic::Ordering::Relaxed)
            .min(TicketBuilder::MAX_TICKET_INDEX)
    }
}

#[derive(Debug, Default)]
pub struct OutgoingIndexCache {
    cache: dashmap::DashMap<(ChannelId, u32), std::sync::Arc<OutgoingIndexEntry>>,
    removed: dashmap::DashSet<(ChannelId, u32)>,
}

impl OutgoingIndexCache {
    /// Returns the next outgoing index for the given channel and epoch.
    pub fn next(&self, channel_id: &ChannelId, epoch: u32) -> u64 {
        self.cache.entry((*channel_id, epoch)).or_default().increment()
    }

    /// Inserts the index for the given channel and `epoch`, or updates
    /// the existing value if it is less than the provided `index`.
    ///
    /// Returns the index value that is either:
    ///  - equal to `index` if no index for the given channel and epoch existed and the value was inserted, or
    ///  - equal to the existing index value, if the provided `index` is less than the existing index value, or
    ///  - equal to the provided `index` value if it is greater than the existing index value and the value is updated.
    pub fn upsert(&self, channel_id: &ChannelId, epoch: u32, index: u64) -> u64 {
        self.cache
            .entry((*channel_id, epoch))
            .or_insert_with(|| std::sync::Arc::new(OutgoingIndexEntry::new(index)))
            .set(index)
    }

    /// Removes the index for the given channel and `epoch` if it exists.
    ///
    /// Returns `true` if the index was removed, `false` otherwise.
    pub fn remove(&self, channel_id: &ChannelId, epoch: u32) -> bool {
        if let Some(((id, ep), _)) = self.cache.remove(&(*channel_id, epoch)) {
            self.removed.insert((id, ep));
            true
        } else {
            false
        }
    }

    /// Synchronizes the current state with the provided store.
    ///
    /// Saves only those values that were changed since the last save operation.
    pub fn save<S: OutgoingIndexStore + Send + Sync + 'static>(
        &self,
        store: std::sync::Arc<parking_lot::RwLock<S>>,
    ) -> Result<(), anyhow::Error> {
        // Clone entries so that we do not hold any locks
        let cache = self.cache.clone();
        let removed = self.removed.clone();
        let mut failed = 0;

        for entry in cache.iter().filter(|e| e.value().is_dirty()) {
            let (channel_id, epoch) = entry.key();
            let index = entry.value().get();
            if let Err(error) = store.write().save_outgoing_index(channel_id, *epoch, index) {
                tracing::error!(%error, %channel_id, epoch, "failed to save outgoing index");
                failed += 1;
            } else {
                tracing::trace!(%channel_id, epoch, index, "saved outgoing index");
                entry.value().mark_clean();
            }
        }

        for (channel_id, channel_epoch) in removed.iter().map(|e| (e.0, e.1)) {
            if let Err(error) = store.write().delete_outgoing_index(&channel_id, channel_epoch) {
                tracing::error!(%error, %channel_id, %channel_epoch, "failed to remove outgoing index");
                failed += 1;
            } else {
                tracing::trace!(%channel_id, %channel_epoch, "removed outgoing index");
                self.removed.remove(&(channel_id, channel_epoch));
            }
        }

        if failed > 0 {
            anyhow::bail!("failed to save {} outgoing index entries", failed);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChannelTicketQueue<Q> {
    pub(crate) queue: std::sync::Arc<parking_lot::RwLock<(Q, ChannelTicketStats)>>,
    pub(crate) redeem_lock: std::sync::Arc<AtomicBool>,
}

impl<Q: TicketQueue> From<Q> for ChannelTicketQueue<Q> {
    fn from(queue: Q) -> Self {
        let stats = ChannelTicketStats {
            winning_tickets: queue.len().unwrap_or(0) as u128,
            ..Default::default()
        };
        Self {
            queue: std::sync::Arc::new(parking_lot::RwLock::new((queue, stats))),
            redeem_lock: std::sync::Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct ChannelTicketStats {
    pub winning_tickets: u128,
    pub redeemed_value: HoprBalance,
    pub neglected_value: HoprBalance,
    pub rejected_value: HoprBalance,
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::*;
    use crate::MemoryStore;

    const MAX: u64 = TicketBuilder::MAX_TICKET_INDEX;

    fn store() -> Arc<parking_lot::RwLock<MemoryStore>> {
        Arc::new(parking_lot::RwLock::new(MemoryStore::default()))
    }

    #[test]
    fn default_initializes_to_zero_and_dirty() {
        let e = OutgoingIndexEntry::default();
        assert_eq!(e.get(), 0);
        assert!(e.is_dirty());
    }

    #[test]
    fn increment_saturates_return_value_at_max() {
        let e = OutgoingIndexEntry::new(MAX);
        assert_eq!(e.increment(), MAX);
        assert_eq!(e.get(), MAX);
    }

    #[test]
    fn set_does_not_decrease_value_when_new_value_is_lower() {
        let e = OutgoingIndexEntry::new(20);
        e.mark_clean();
        assert_eq!(e.set(10), 20);
        assert_eq!(e.get(), 20);
        assert!(!e.is_dirty());
    }

    #[test]
    fn concurrent_set_uses_max_semantics() {
        let e = Arc::new(OutgoingIndexEntry::new(0));
        e.mark_clean();

        let vals = [1u64, 7, 3, 42, 9];
        let mut handles = vec![];

        for v in vals {
            let e2 = Arc::clone(&e);
            handles.push(thread::spawn(move || {
                e2.set(v);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(e.get(), 42.min(MAX));
        assert!(e.is_dirty());
    }

    #[test]
    fn next_creates_entry_with_zero_and_increments_sequentially() {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();
        let epoch = 1;

        assert_eq!(cache.next(&channel_id, epoch), 0);
        assert_eq!(cache.next(&channel_id, epoch), 1);
        assert_eq!(cache.next(&channel_id, epoch), 2);
    }

    #[test]
    fn next_is_scoped_by_channel_and_epoch() {
        let cache = OutgoingIndexCache::default();
        let channel_a = Default::default();
        let channel_b = ChannelId::create(&[b"other"]);

        assert_eq!(cache.next(&channel_a, 1), 0);
        assert_eq!(cache.next(&channel_a, 2), 0);
        assert_eq!(cache.next(&channel_b, 1), 0);
        assert_eq!(cache.next(&channel_a, 1), 1);
    }

    #[test]
    fn set_inserts_when_key_is_missing() {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();

        assert_eq!(cache.upsert(&channel_id, 1, 17), 17);
        assert_eq!(cache.next(&channel_id, 1), 17);
        assert_eq!(cache.next(&channel_id, 1), 18);
    }

    #[test]
    fn set_does_not_decrease_existing_value() {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();

        assert_eq!(cache.upsert(&channel_id, 1, 10), 10);
        assert_eq!(cache.upsert(&channel_id, 1, 7), 10);
        assert_eq!(cache.next(&channel_id, 1), 10);
        assert_eq!(cache.next(&channel_id, 1), 11);
    }

    #[test]
    fn next_saturates_at_max() {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();

        assert_eq!(cache.upsert(&channel_id, 1, MAX), MAX);
        assert_eq!(cache.next(&channel_id, 1), MAX);
        assert_eq!(cache.next(&channel_id, 1), MAX);
    }

    #[test]
    fn remove_existing_entry_returns_true_and_persists_deletion_on_save() -> anyhow::Result<()> {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();
        let epoch = 1;
        let store = store();

        assert_eq!(cache.upsert(&channel_id, epoch, 5), 5);
        cache.save(store.clone())?;

        assert_eq!(store.read().load_outgoing_index(&channel_id, epoch)?, Some(5));

        assert!(cache.remove(&channel_id, epoch));
        assert!(cache.save(store.clone()).is_ok());

        assert_eq!(store.read().load_outgoing_index(&channel_id, epoch)?, None);
        assert!(!cache.remove(&channel_id, epoch));

        Ok(())
    }

    #[test]
    fn remove_missing_entry_returns_false() {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();

        assert!(!cache.remove(&channel_id, 1));
    }

    #[test]
    fn save_persists_only_dirty_entries() -> anyhow::Result<()> {
        let cache = OutgoingIndexCache::default();
        let channel_a = Default::default();
        let channel_b = ChannelId::create(&[b"other"]);
        let store = store();

        cache.upsert(&channel_a, 1, 10);
        cache.upsert(&channel_b, 2, 20);

        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_a, 1)?, Some(10));
        assert_eq!(store.read().load_outgoing_index(&channel_b, 2)?, Some(20));

        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_a, 1)?, Some(10));
        assert_eq!(store.read().load_outgoing_index(&channel_b, 2)?, Some(20));

        cache.next(&channel_a, 1);
        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_a, 1)?, Some(11));
        assert_eq!(store.read().load_outgoing_index(&channel_b, 2)?, Some(20));

        Ok(())
    }

    #[test]
    fn save_persists_removed_entries_only_once() -> anyhow::Result<()> {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();
        let store = store();

        cache.upsert(&channel_id, 1, 3);
        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_id, 1)?, Some(3));

        assert!(cache.remove(&channel_id, 1));
        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_id, 1)?, None);

        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_id, 1)?, None);

        Ok(())
    }

    #[test]
    fn save_is_idempotent_after_success() -> anyhow::Result<()> {
        let cache = OutgoingIndexCache::default();
        let channel_id = Default::default();
        let store = store();

        cache.upsert(&channel_id, 1, 9);
        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_id, 1)?, Some(9));

        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_id, 1)?, Some(9));

        cache.next(&channel_id, 1);
        cache.save(store.clone())?;
        assert_eq!(store.read().load_outgoing_index(&channel_id, 1)?, Some(10));

        Ok(())
    }

    #[test]
    fn concurrent_next_on_same_key_is_monotonic() -> anyhow::Result<()> {
        use std::thread;

        let cache = Arc::new(OutgoingIndexCache::default());
        let channel_id = Default::default();
        let epoch = 1;

        let mut handles = vec![];
        for _ in 0..8 {
            let cache = Arc::clone(&cache);
            let channel_id = channel_id;
            handles.push(thread::spawn(move || cache.next(&channel_id, epoch)));
        }

        let mut values = handles
            .into_iter()
            .map(|h| h.join())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| anyhow::anyhow!("join error"))?;
        values.sort_unstable();

        assert_eq!(values, (0..8).collect::<Vec<_>>());
        assert_eq!(cache.next(&channel_id, epoch), 8);

        Ok(())
    }

    #[test]
    fn save_handles_multiple_keys_and_removals_together() -> anyhow::Result<()> {
        let cache = OutgoingIndexCache::default();
        let channel_a = Default::default();
        let channel_b = ChannelId::create(&[b"other"]);
        let store = store();

        cache.upsert(&channel_a, 1, 4);
        cache.upsert(&channel_b, 2, 7);
        cache.save(store.clone())?;

        assert!(cache.remove(&channel_a, 1));
        cache.next(&channel_b, 2);
        cache.save(store.clone())?;

        assert_eq!(store.read().load_outgoing_index(&channel_a, 1)?, None);
        assert_eq!(store.read().load_outgoing_index(&channel_b, 2)?, Some(8));

        Ok(())
    }
}
