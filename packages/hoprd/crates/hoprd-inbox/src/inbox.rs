use async_lock::Mutex;
use async_trait::async_trait;
use core_packet::interaction::{ApplicationData, Tag, DEFAULT_APPLICATION_TAG};
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

    // TODO: consider adding a stream version for `pop_all`

    /// Purges all entries strictly older than the given timestamp.
    async fn purge(&mut self, older_than_ts: Duration);
}

/// Holds basic configuration parameters of the `MessageInbox`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct MessageInboxConfiguration {
    /// Maximum capacity per-each application tag.
    /// In the current implementation, the capacity must be a power of two.
    pub capacity: u32,
    /// Maximum age of a message held in the inbox until it is purged.
    pub max_age_sec: u64, // cannot use std::time::Duration here due to wasm-bindgen
    /// List of tags that are excluded on `push`.
    pub excluded_tags: Vec<Tag>,
}

impl Default for MessageInboxConfiguration {
    fn default() -> Self {
        Self {
            capacity: 512,                                // must be a power of 2 with this implementation
            max_age_sec: 15 * 60,                         // 15 minutes
            excluded_tags: vec![DEFAULT_APPLICATION_TAG], // exclude untagged messages pre default
        }
    }
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
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec))
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
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec))
            .await;
        db.count(tag).await
    }

    /// Pop the oldest message with the given tag, or the oldest message regardless the tag
    /// if it is not given. Returns `None` if there's no message with such `tag` (if given) in the inbox
    /// or if the whole inbox is empty (if no `tag` is given).
    pub async fn pop(&self, tag: Option<Tag>) -> Option<ApplicationData> {
        if self.is_excluded_tag(&tag) {
            return None;
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec))
            .await;
        db.pop(tag).await
    }

    /// Pops all the messages with the given `tag` (ordered oldest to latest) or
    /// all the messages from the entire inbox (ordered oldest to latest) if no `tag` is given.
    pub async fn pop_all(&self, tag: Option<Tag>) -> Vec<ApplicationData> {
        if self.is_excluded_tag(&tag) {
            return Vec::new();
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - Duration::from_secs(self.cfg.max_age_sec))
            .await;
        db.pop_all(tag).await
    }
}

#[cfg(test)]
mod tests {
    use crate::inbox::{MessageInbox, MessageInboxConfiguration};
    use crate::ring::RingBufferInboxBackend;
    use core_packet::interaction::{ApplicationData, Tag};
    use std::time::Duration;

    #[async_std::test]
    async fn test_basic_flow() {
        let cfg = MessageInboxConfiguration {
            capacity: 4,
            excluded_tags: vec![2],
            max_age_sec: 2,
        };

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
        assert_eq!(b"test msg 0", ad.plain_text.as_ref());

        let ad = mi.pop(Some(1)).await.unwrap();
        assert_eq!(b"test msg 1", ad.plain_text.as_ref());
        assert_eq!(1, mi.size(Some(1)).await);

        assert_eq!(1, mi.size(None).await);

        async_std::task::sleep(Duration::from_millis(2500)).await;

        assert_eq!(0, mi.size(None).await);
    }
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
