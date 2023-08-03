use async_lock::Mutex;
use async_trait::async_trait;
use core_packet::interaction::{ApplicationData, Tag};
use std::time::Duration;

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

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct MessageInboxConfiguration {
    /// Maximum capacity per-each application tag.
    pub capacity: u32,
    pub max_age_sec: u64,
    pub excluded_tags: Vec<Tag>,
}

impl Default for MessageInboxConfiguration {
    fn default() -> Self {
        Self {
            capacity: 1024,
            max_age_sec: 15 * 60_u64,
            excluded_tags: vec![],
        }
    }
}

/// Represents a thread-safe message inbox of messages of type `M`
/// This type is thin frontend over `InboxBackend` using 16-bit unsigned integer tags.
pub struct MessageInbox<B>
where
    B: InboxBackend<Tag, ApplicationData>,
{
    cfg: MessageInboxConfiguration,
    backend: Mutex<B>,
    time: TimestampFn,
}

impl<B> MessageInbox<B>
where
    B: InboxBackend<Tag, ApplicationData>,
{
    #[cfg(any(not(feature = "wasm"), test))]
    pub fn new(cfg: MessageInboxConfiguration) -> Self {
        Self::new_with_time(cfg, || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
        })
    }

    pub fn new_with_time(cfg: MessageInboxConfiguration, time: TimestampFn) -> Self {
        Self {
            backend: Mutex::new(B::new_with_capacity(cfg.capacity as usize, time)),
            time,
            cfg,
        }
    }

    pub async fn push(&self, payload: ApplicationData) -> bool {
        if payload.application_tag.is_none()
            || payload
                .application_tag
                .iter()
                .all(|e| e.eq(&payload.application_tag.unwrap()))
        {
            let mut db = self.backend.lock().await;
            db.push(payload.application_tag, payload).await;
            db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec))
                .await;
            true
        } else {
            false
        }
    }

    pub async fn size(&self, tag: Option<Tag>) -> usize {
        self.backend.lock().await.count(tag).await
    }

    pub async fn pop(&self, tag: Option<Tag>) -> Option<ApplicationData> {
        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec))
            .await;
        db.pop(tag).await
    }

    pub async fn pop_all(&self, tag: Option<Tag>) -> Vec<ApplicationData> {
        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec))
            .await;
        db.pop_all(tag).await
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic_flow() {}
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::inbox::MessageInboxConfiguration;
    use crate::inbox::Tag;
    use crate::ring::RingBufferInboxBackend;
    use core_packet::interaction::ApplicationData;
    use std::time::Duration;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub struct MessageInbox {
        w: super::MessageInbox<RingBufferInboxBackend<Tag, ApplicationData>>,
    }

    #[wasm_bindgen]
    impl MessageInbox {
        #[wasm_bindgen(constructor)]
        pub fn new(cfg: MessageInboxConfiguration) -> Self {
            Self {
                w: super::MessageInbox::new_with_time(cfg, || Duration::from_millis(js_sys::Date::now() as u64)),
            }
        }

        pub async fn push(&self, payload: ApplicationData) -> bool {
            self.w.push(payload).await
        }

        pub async fn pop(&self, tag: Option<u16>) -> Option<ApplicationData> {
            self.w.pop(tag).await
        }

        pub async fn pop_all(&self, tag: Option<u16>) -> JsResult<JsValue> {
            let all = self.w.pop_all(tag).await;
            ok_or_jserr!(serde_wasm_bindgen::to_value(&all))
        }

        pub async fn size(&self, tag: Option<u16>) -> u32 {
            self.w.size(tag).await as u32
        }
    }
}
