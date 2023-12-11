use async_lock::Mutex;
use async_trait::async_trait;
use core_types::protocol::{ApplicationData, Tag};
use std::time::Duration;

use crate::config::MessageInboxConfiguration;

/// Represents a simple timestamping function.
/// This is useful if used in WASM or environment which might have different means of measuring time.
pub type TimestampFn = fn() -> Duration;

/// Represents a generic backend trait for the message inbox.
/// Messages `M` can be tagged or untagged via the type `T`
#[async_trait(?Send)]
pub trait InboxBackend<T: Copy + Default, M: Clone> {
    /// Create new storage with the given capacity and the timestamping function
    fn new_with_capacity(cap: usize, ts: TimestampFn) -> Self;

    /// Push a new entry with an optional `tag`.
    async fn push(&mut self, tag: Option<T>, payload: M);

    /// Count number of entries with the given `tag`.
    /// If no `tag` is given, returns the total count of all tagged and untagged entries.
    async fn count(&self, tag: Option<T>) -> usize;

    /// Pops oldest entry with the given `tag` or oldest entry in general, if no `tag` was given.
    /// Returns `None` if queue with the given `tag` is empty, or the entire store is empty (if no `tag` was given).
    async fn pop(&mut self, tag: Option<T>) -> Option<(M, Duration)>;

    /// Pops all entries of the given `tag`, or all entries (tagged and untagged) and returns them.
    async fn pop_all(&mut self, tag: Option<T>) -> Vec<(M, Duration)>;

    /// Peeks the oldest entry with the given `tag` or oldest entry in general, if no `tag` was given.
    /// Returns `None` if queue with the given `tag` is empty, or the entire store is empty (if no `tag` was given).
    async fn peek(&mut self, tag: Option<T>) -> Option<(M, Duration)>;

    /// Peeks all entries of the given `tag`, or all entries (tagged and untagged) and returns them.
    async fn peek_all(&mut self, tag: Option<T>) -> Vec<(M, Duration)>;

    // TODO: consider adding a stream version for `pop_all`

    /// Purges all entries strictly older than the given timestamp.
    async fn purge(&mut self, older_than_ts: Duration);
}

/// Represents a thread-safe message inbox of messages of type `M`
/// This type is thin frontend over `InboxBackend` using 16-bit unsigned integer tags.
/// Each operation also performs `purge` of the backend.
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
    /// Creates new instance given the configuration.
    /// Uses `std::time::SystemTime` as timestamping function, therefore
    /// this function cannot be used in WASM runtimes.
    #[cfg(any(not(feature = "wasm"), test))]
    pub fn new(cfg: MessageInboxConfiguration) -> Self {
        Self::new_with_time(cfg, || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
        })
    }

    /// Creates new instance given the configuration and the timestamping function.
    pub fn new_with_time(cfg: MessageInboxConfiguration, time: TimestampFn) -> Self {
        Self {
            backend: Mutex::new(B::new_with_capacity(cfg.capacity as usize, time)),
            time,
            cfg,
        }
    }

    fn is_excluded_tag(&self, tag: &Option<Tag>) -> bool {
        match tag {
            None => false,
            Some(t) => self.cfg.excluded_tags.iter().any(|e| e.eq(t)),
        }
    }

    /// Push message into the inbox. Returns `true` if the message has been enqueued, `false` if it
    /// has been excluded based on the configured exluded tags.
    pub async fn push(&self, payload: ApplicationData) -> bool {
        if self.is_excluded_tag(&payload.application_tag) {
            return false;
        }

        // Push only if there is no tag, or if the tag is not excluded
        let mut db = self.backend.lock().await;
        db.push(payload.application_tag, payload).await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec()))
            .await;

        true
    }

    /// Number of messages in the inbox with the given `tag`, or the total number of all messages
    /// if no `tag` is given.
    pub async fn size(&self, tag: Option<Tag>) -> usize {
        if self.is_excluded_tag(&tag) {
            return 0;
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec()))
            .await;
        db.count(tag).await
    }

    /// Pop the oldest message with the given tag, or the oldest message regardless the tag
    /// if it is not given. Returns `None` if there's no message with such `tag` (if given) in the inbox
    /// or if the whole inbox is empty (if no `tag` is given).
    pub async fn pop(&self, tag: Option<Tag>) -> Option<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return None;
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec()))
            .await;

        return db.pop(tag).await;
    }

    /// Peek the oldest message with the given tag, or the oldest message regardless the tag
    /// if it is not given. Returns `None` if there's no message with such `tag` (if given) in the inbox
    /// or if the whole inbox is empty (if no `tag` is given).
    pub async fn peek(&self, tag: Option<Tag>) -> Option<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return None;
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec()))
            .await;

        return db.peek(tag).await;
    }

    /// Peeks all the messages with the given `tag` (ordered oldest to latest) or
    /// all the messages from the entire inbox (ordered oldest to latest) if no `tag` is given.
    pub async fn peek_all(&self, tag: Option<Tag>) -> Vec<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return Vec::new();
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec()))
            .await;
        db.peek_all(tag).await
    }

    /// Pops all the messages with the given `tag` (ordered oldest to latest) or
    /// all the messages from the entire inbox (ordered oldest to latest) if no `tag` is given.
    pub async fn pop_all(&self, tag: Option<Tag>) -> Vec<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return Vec::new();
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec()))
            .await;
        db.pop_all(tag).await
    }
}

#[cfg(test)]
mod tests {
    use crate::inbox::{MessageInbox, MessageInboxConfiguration};
    use crate::ring::RingBufferInboxBackend;
    use core_types::protocol::{ApplicationData, Tag};
    use std::time::Duration;

    #[async_std::test]
    async fn test_basic_flow() {
        let mut cfg = MessageInboxConfiguration::default();
        cfg.capacity = 4;
        cfg.excluded_tags = vec![2];
        cfg.set_max_age_sec(2);

        let mi = MessageInbox::<RingBufferInboxBackend<Tag, ApplicationData>>::new(cfg);

        assert!(
            mi.push(ApplicationData {
                application_tag: None,
                plain_text: (*b"test msg 0").into()
            })
            .await
        );
        assert!(
            mi.push(ApplicationData {
                application_tag: Some(1),
                plain_text: (*b"test msg 1").into()
            })
            .await
        );
        assert!(
            mi.push(ApplicationData {
                application_tag: Some(1),
                plain_text: (*b"test msg 2").into()
            })
            .await
        );
        assert!(
            !mi.push(ApplicationData {
                application_tag: Some(2),
                plain_text: (*b"test msg").into()
            })
            .await
        );
        assert_eq!(3, mi.size(None).await);
        assert_eq!(2, mi.size(Some(1)).await);
        assert_eq!(0, mi.size(Some(2)).await);

        let ad = mi.pop(None).await.unwrap();
        assert_eq!(b"test msg 0", ad.0.plain_text.as_ref());

        let ad = mi.pop(Some(1)).await.unwrap();
        assert_eq!(b"test msg 1", ad.0.plain_text.as_ref());
        assert_eq!(1, mi.size(Some(1)).await);

        assert_eq!(1, mi.size(None).await);

        async_std::task::sleep(Duration::from_millis(2500)).await;

        assert_eq!(0, mi.size(None).await);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::inbox::MessageInboxConfiguration;
    use crate::ring::RingBufferInboxBackend;
    use core_types::protocol::ApplicationData;
    use core_types::protocol::Tag;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[wasm_bindgen(getter_with_clone)]
    pub struct MessageInboxEntry {
        pub data: ApplicationData,
        pub ts_seconds: u64,
    }

    impl From<(ApplicationData, Duration)> for MessageInboxEntry {
        fn from(value: (ApplicationData, Duration)) -> Self {
            Self {
                data: value.0,
                ts_seconds: value.1.as_secs(),
            }
        }
    }

    #[wasm_bindgen]
    impl MessageInboxConfiguration {
        #[wasm_bindgen(constructor)]
        pub fn _new() -> Self {
            Self::default()
        }
    }

    #[wasm_bindgen]
    pub struct MessageInbox {
        w: super::MessageInbox<RingBufferInboxBackend<Tag, ApplicationData>>,
    }

    #[wasm_bindgen]
    impl MessageInbox {
        #[wasm_bindgen(constructor)]
        pub fn new(cfg: MessageInboxConfiguration) -> Self {
            Self {
                w: super::MessageInbox::new_with_time(cfg, || {
                    Duration::from_millis(utils_misc::time::wasm::current_timestamp())
                }),
            }
        }

        pub async fn push(&self, payload: ApplicationData) -> bool {
            self.w.push(payload).await
        }

        pub async fn pop(&self, tag: Option<u16>) -> Option<MessageInboxEntry> {
            self.w.pop(tag).await.map(MessageInboxEntry::from)
        }

        pub async fn pop_all(&self, tag: Option<u16>) -> JsResult<JsValue> {
            let all = self
                .w
                .pop_all(tag)
                .await
                .into_iter()
                .map(MessageInboxEntry::from)
                .collect::<Vec<MessageInboxEntry>>();

            ok_or_jserr!(serde_wasm_bindgen::to_value(&all))
        }

        pub async fn peek(&self, tag: Option<u16>) -> Option<MessageInboxEntry> {
            self.w.peek(tag).await.map(MessageInboxEntry::from)
        }

        pub async fn peek_all(&self, tag: Option<u16>) -> JsResult<JsValue> {
            let all = self
                .w
                .peek_all(tag)
                .await
                .into_iter()
                .map(MessageInboxEntry::from)
                .collect::<Vec<MessageInboxEntry>>();

            ok_or_jserr!(serde_wasm_bindgen::to_value(&all))
        }
        pub async fn size(&self, tag: Option<u16>) -> u32 {
            self.w.size(tag).await as u32
        }
    }
}
