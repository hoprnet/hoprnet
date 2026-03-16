use std::sync::atomic::{AtomicBool, AtomicU64};

use hopr_api::{chain::ChannelId, types::internal::prelude::TicketBuilder};

use crate::OutgoingIndexStore;

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
    fn increment(&self) -> u64 {
        let v = self.index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if v <= TicketBuilder::MAX_TICKET_INDEX {
            self.is_dirty.store(true, std::sync::atomic::Ordering::Release);
        }
        v.min(TicketBuilder::MAX_TICKET_INDEX)
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
    pub fn next(&self, channel_id: &ChannelId, epoch: u32) -> u64 {
        self.cache.entry((*channel_id, epoch)).or_default().increment()
    }

    pub fn set(&self, channel_id: &ChannelId, epoch: u32, index: u64) {
        self.cache
            .insert((*channel_id, epoch), OutgoingIndexEntry::new(index).into());
    }

    pub fn remove(&self, channel_id: &ChannelId, epoch: u32) {
        if let Some(((id, ep), _)) = self.cache.remove(&(*channel_id, epoch)) {
            self.removed.insert((id, ep));
        }
    }

    pub fn save<S: OutgoingIndexStore + Send + Sync + 'static>(
        &self,
        store: std::sync::Arc<parking_lot::RwLock<S>>,
    ) -> Result<(), S::Error> {
        // Clone entries so that we do not hold any locks
        let cache = self.cache.clone();
        let removed = self.removed.clone();

        for entry in cache.iter().filter(|e| e.value().is_dirty()) {
            let (channel_id, epoch) = entry.key();
            if let Err(error) = store
                .write()
                .save_outgoing_index(channel_id, *epoch, entry.value().get())
            {
                tracing::error!(%error, %channel_id, epoch, "failed to save outgoing index");
            } else {
                entry.value().mark_clean();
            }
        }

        for (channel_id, channel_epoch) in removed.iter().map(|e| (e.0, e.1)) {
            if let Err(error) = store.write().delete_outgoing_index(&channel_id, channel_epoch) {
                tracing::error!(%error, %channel_id, %channel_epoch, "failed to remove outgoing index");
            } else {
                self.removed.remove(&(channel_id, channel_epoch));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ChannelTicketQueue<Q> {
    pub(crate) queue: std::sync::Arc<parking_lot::RwLock<Q>>,
    pub(crate) redeem_lock: std::sync::Arc<parking_lot::Mutex<()>>,
}

impl<Q> From<Q> for ChannelTicketQueue<Q> {
    fn from(queue: Q) -> Self {
        Self {
            queue: std::sync::Arc::new(parking_lot::RwLock::new(queue)),
            redeem_lock: std::sync::Arc::new(parking_lot::Mutex::new(())),
        }
    }
}
