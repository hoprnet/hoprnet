use std::marker::PhantomData;
use std::time::Duration;
use async_lock::{Mutex};
use async_trait::async_trait;

/// Represents a simple timestamping function.
/// This is useful if used in WASM or environment which might have different means of measuring time.
pub type TimestampFn = fn() -> Duration;

/// Represents a generic backend trait for the message inbox.
/// Messages `M` can be tagged or untagged via the type `T`
#[async_trait(?Send)]
pub trait InboxBackend<T: Copy + Default, M> {
    /// Create new storage with the given capacity and the timestamping function
    fn new_with_capacity(cap: usize, ts: TimestampFn) -> Self;

    /// Push a new entry with an optional `tag`.
    async fn push(&mut self, tag: Option<T>, payload: M);

    /// Count number of entries with the given `tag`.
    /// If no `tag` is given, returns the total count of all tagged and untagged entries.
    async fn count(&self, tag: Option<T>) -> usize;

    /// Pops oldest entry with the given `tag` or oldest entry in general, if no `tag` was given.
    /// Returns `None` if queue with the given `tag` is empty, or the entire store is empty (if no `tag` was given).
    async fn pop(&mut self, tag: Option<T>) -> Option<M>;

    /// Pops all entries of the given `tag`, or all entries (tagged and untagged) and returns them.
    async fn pop_all(&mut self, tag: Option<T>) -> Vec<M>;

    /// Purges all entries strictly older than the given timestamp.
    async fn purge(&mut self, older_than_ts: Duration);
}

/// Tags are currently 16-bit unsigned integers
type Tag = u16;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct MessageInboxConfiguration {
    pub capacity: u32,
    pub max_age_sec: u64,
    pub excluded_tags: Vec<Tag>,
}

impl Default for MessageInboxConfiguration {
    fn default() -> Self {
        Self {
            capacity: 1024,
            max_age_sec: 15*60_u64,
            excluded_tags: vec![]
        }
    }
}

/// Represents a thread-safe message inbox of messages `M`
pub struct MessageInbox<M, B>
where M: AsRef<[u8]>, B: InboxBackend<Tag, M> {
    cfg: MessageInboxConfiguration,
    backend: Mutex<B>,
    time: TimestampFn,
    _data: PhantomData<M>
}

impl<M, B> MessageInbox<M, B>
where M: AsRef<[u8]>, B: InboxBackend<Tag, M> {
    #[cfg(any(not(feature = "wasm"), test))]
    pub fn new(cfg: MessageInboxConfiguration) -> Self {
        Self::new_with_time(cfg, || std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap())
    }

    pub fn new_with_time(cfg: MessageInboxConfiguration, time: TimestampFn) -> Self {
        Self {
            backend: Mutex::new(B::new_with_capacity(cfg.capacity as usize, time)),
            time, cfg, _data: PhantomData
        }
    }

    fn extract_tag(payload: &M) -> Option<Tag> {
        // TODO: specify application-layer format
        let data: [u8; 2] = payload.as_ref()[0..2].try_into().ok()?;
        Some(Tag::from_be_bytes(data))
    }

    pub async fn push(&mut self, payload: M) {
        let tag = Self::extract_tag(&payload);
        let mut db = self.backend.lock().await;
        db.push(tag, payload).await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec)).await;
    }

    pub async fn size(&self, tag: Option<Tag>) -> usize {
        self.backend.lock().await.count(tag).await
    }

    pub async fn pop(&mut self, tag: Option<Tag>) -> Option<M> {
        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec)).await;
        db.pop(tag).await
    }

    pub async fn pop_all(&mut self, tag: Option<Tag>) -> Vec<M> {
        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec)).await;
        db.pop_all(tag).await
    }
}

#[cfg(test)]
mod tests {

}

#[cfg(feature = "wasm")]
pub mod wasm {

}