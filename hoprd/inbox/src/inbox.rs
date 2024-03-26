use async_lock::Mutex;
use async_trait::async_trait;
use hopr_internal_types::prelude::*;
use std::time::Duration;

use crate::config::MessageInboxConfiguration;

/// Represents a simple timestamping function.
/// This is useful if used in WASM or environment which might have different means of measuring time.
pub type TimestampFn = fn() -> Duration;

/// Represents a generic backend trait for the message inbox.
/// Messages `M` can be tagged or untagged via the type `T`
#[async_trait]
pub trait InboxBackend<T: Copy + Default + std::marker::Send, M: Clone + std::marker::Send> {
    /// Create new storage with the given capacity and the timestamping function
    fn new_with_capacity(cap: usize, ts: TimestampFn) -> Self;

    /// Push a new entry with an optional `tag`.
    async fn push(&mut self, tag: Option<T>, payload: M);

    /// Count number of entries with the given `tag`.
    ///
    /// If no `tag` is given, returns the total count of all tagged and untagged entries.
    async fn count(&self, tag: Option<T>) -> usize;

    /// Pops oldest entry with the given `tag` or oldest entry in general, if no `tag` was given.
    ///
    /// Returns `None` if queue with the given `tag` is empty, or the entire store is empty (if no `tag` was given).
    async fn pop(&mut self, tag: Option<T>) -> Option<(M, Duration)>;

    /// Pops all entries of the given `tag`, or all entries (tagged and untagged) and returns them.
    async fn pop_all(&mut self, tag: Option<T>) -> Vec<(M, Duration)>;

    /// Peeks the oldest entry with the given `tag` or oldest entry in general, if no `tag` was given.
    ///
    /// Returns `None` if queue with the given `tag` is empty, or the entire store is empty (if no `tag` was given).
    async fn peek(&mut self, tag: Option<T>) -> Option<(M, Duration)>;

    /// Peeks all entries of the given `tag`, or all entries (tagged and untagged) and returns them.
    ///
    /// If the optional parameter `timestamp` is provided, only entries more recent than this are returned.
    /// NOTE: the timestamp comparison precision should be at most up to milliseconds.
    async fn peek_all(&mut self, tag: Option<T>, timestamp: Option<Duration>) -> Vec<(M, Duration)>;

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
    ///
    /// Uses `std::time::SystemTime` as timestamping function.
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

    /// Checks whether the tag is in the exclusion list.
    ///
    /// NOTE: If the `tag` is `None`, it will never be considered as excluded.
    /// This has the following implication:
    /// Since [DEFAULT_APPLICATION_TAG](hopr_internal_types::protocol::DEFAULT_APPLICATION_TAG) is also considered as excluded per default,
    /// all the messages without a tag (= with implicit [DEFAULT_APPLICATION_TAG](hopr_internal_types::protocol::DEFAULT_APPLICATION_TAG))
    /// will be allowed into the inbox, whereas the messages which explicitly specify that tag, will not make it into the inbox.
    fn is_excluded_tag(&self, tag: &Option<Tag>) -> bool {
        match tag {
            None => false,
            Some(t) => self.cfg.excluded_tags.iter().any(|e| e.eq(t)),
        }
    }

    /// Push message into the inbox. Returns `true` if the message has been enqueued, `false` if it
    /// has been excluded based on the configured excluded tags.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn push(&self, payload: ApplicationData) -> bool {
        if self.is_excluded_tag(&payload.application_tag) {
            return false;
        }

        // Push only if there is no tag, or if the tag is not excluded
        let mut db = self.backend.lock().await;
        db.push(payload.application_tag, payload).await;
        db.purge((self.time)() - self.cfg.max_age).await;

        true
    }

    /// Number of messages in the inbox with the given `tag`, or the total number of all messages
    /// if no `tag` is given.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn size(&self, tag: Option<Tag>) -> usize {
        if self.is_excluded_tag(&tag) {
            return 0;
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - self.cfg.max_age).await;
        db.count(tag).await
    }

    /// Pop the oldest message with the given tag, or the oldest message regardless the tag
    /// if it is not given.
    ///
    /// Returns `None` if there's no message with such `tag` (if given) in the inbox
    /// or if the whole inbox is empty (if no `tag` is given).
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn pop(&self, tag: Option<Tag>) -> Option<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return None;
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - self.cfg.max_age).await;

        db.pop(tag).await
    }

    /// Peek the oldest message with the given tag, or the oldest message regardless the tag
    /// if it is not given.
    ///
    /// Returns `None` if there's no message with such `tag` (if given) in the inbox
    /// or if the whole inbox is empty (if no `tag` is given).
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn peek(&self, tag: Option<Tag>) -> Option<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return None;
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - self.cfg.max_age).await;

        db.peek(tag).await
    }

    /// Peeks all the messages with the given `tag` (ordered oldest to latest) or
    /// all the messages from the entire inbox (ordered oldest to latest) if no `tag` is given.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn peek_all(&self, tag: Option<Tag>, timestamp: Option<Duration>) -> Vec<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return Vec::new();
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - self.cfg.max_age).await;
        db.peek_all(tag, timestamp).await
    }

    /// Pops all the messages with the given `tag` (ordered oldest to latest) or
    /// all the messages from the entire inbox (ordered oldest to latest) if no `tag` is given.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn pop_all(&self, tag: Option<Tag>) -> Vec<(ApplicationData, Duration)> {
        if self.is_excluded_tag(&tag) {
            return Vec::new();
        }

        let mut db = self.backend.lock().await;
        db.purge((self.time)() - self.cfg.max_age).await;
        db.pop_all(tag).await
    }
}

#[cfg(test)]
mod tests {
    use crate::inbox::{MessageInbox, MessageInboxConfiguration};
    use crate::ring::RingBufferInboxBackend;
    use hopr_internal_types::prelude::*;
    use std::time::Duration;

    #[async_std::test]
    async fn test_basic_flow() {
        let cfg = MessageInboxConfiguration {
            capacity: 4,
            max_age: std::time::Duration::from_secs(2),
            excluded_tags: vec![2],
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
        assert_eq!(b"test msg 0", ad.0.plain_text.as_ref());

        let ad = mi.pop(Some(1)).await.unwrap();
        assert_eq!(b"test msg 1", ad.0.plain_text.as_ref());
        assert_eq!(1, mi.size(Some(1)).await);

        assert_eq!(1, mi.size(None).await);

        async_std::task::sleep(Duration::from_millis(2500)).await;

        assert_eq!(0, mi.size(None).await);
    }
}
