use std::sync::atomic::{AtomicBool, AtomicU64};
use hopr_api::chain::ChannelId;
use futures::{FutureExt, StreamExt};
use crate::OutgoingIndexStore;

const OUT_INDEX_SYNC_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

// Contains map of outgoing indices and indicator flag if the map is "dirty"
// (not yet synced to the persistent storage).
type OutIdxCache = (dashmap::DashMap<(ChannelId, u32), AtomicU64>, AtomicBool);

/// Stores outgoing ticket indices in a cache and periodically syncs them to the persistent storage.
pub struct OutgoingIndexTracker {
    out_indices: std::sync::Arc<OutIdxCache>,
    sync_handle: hopr_async_runtime::AbortHandle,
}

impl OutgoingIndexTracker {
    pub fn new<T>(store: std::sync::Arc<parking_lot::RwLock<T>>) -> Self
    where T: OutgoingIndexStore + Send + Sync + 'static {
        let out_indices = std::sync::Arc::new((
            dashmap::DashMap::<(ChannelId, u32), AtomicU64>::new(),
             AtomicBool::new(false)
        ));

        let out_indices_1 = out_indices.clone();
        let out_indices_2 = out_indices.clone();

        // Start syncing of the outgoing ticket indices back to the storage
        // This is needed, because the outgoing ticket indices change rapidly, and therefore
        // the sync cannot be done per call to `next_outgoing_ticket_index`.
        let (stream, sync_handle) =
            futures::stream::abortable(futures_time::stream::interval(OUT_INDEX_SYNC_INTERVAL.into()));
        hopr_async_runtime::prelude::spawn(stream
            .filter(move |_| futures::future::ready(out_indices_1.1.load(std::sync::atomic::Ordering::Acquire)))
            .for_each(move |_| {
                let out_indices_2 = out_indices_2.clone();
                let store = store.clone();
                let all_items = out_indices_2
                    .0
                    .iter()
                    .map(|e| {
                        (
                            e.key().0,
                            e.key().1,
                            e.value().load(std::sync::atomic::Ordering::Relaxed),
                        )
                    })
                    .collect::<Vec<_>>();
                out_indices_2.1.store(false, std::sync::atomic::Ordering::Release);

                async move {
                    let res = hopr_async_runtime::prelude::spawn_blocking(move || {
                        let mut store = store.write();
                        for (channel_id, channel_epoch, index) in all_items {
                            if let Err(error) = store.save_outgoing_index(&channel_id, channel_epoch, index) {
                                tracing::error!(%error, %channel_id, %channel_epoch, %index, "failed to save outgoing index");
                            }
                        }
                    }).await;
                    if let Err(error) = res {
                        tracing::error!(%error, "failed to sync outgoing indices");
                    }
                }
            }).inspect(|_| tracing::debug!("syncing outgoing indices done")));

        Self {
            out_indices,
            sync_handle,
        }
    }

    /// Get the cache of outgoing ticket indices.
    pub fn index_cache(&self) -> &dashmap::DashMap<(ChannelId, u32), AtomicU64> {
        &self.out_indices.0
    }

    /// Indicate that the outgoing indices have been modified and need to be synced.
    pub fn set_dirty(&self) {
        self.out_indices.1.store(true, std::sync::atomic::Ordering::Release);
    }
}

impl Drop for OutgoingIndexTracker {
    fn drop(&mut self) {
        self.sync_handle.abort();
    }
}

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