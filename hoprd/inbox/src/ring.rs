use crate::inbox::{InboxBackend, TimestampFn};
use async_trait::async_trait;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use hopr_platform::time::native::current_timestamp;

/// Acts a simple wrapper of a message with added insertion timestamp.
struct PayloadWrapper<M: std::marker::Send> {
    payload: M,
    ts: Duration,
}

/// Ring buffer based heap-allocated backend.
/// The capacity must be a power-of-two due to optimizations.
/// Tags `T` must be represented by a type that's also a valid key for the `HashMap`
pub struct RingBufferInboxBackend<T, M>
where
    T: Copy + Default + PartialEq + Eq + Hash + std::marker::Send + std::marker::Sync,
    M: Clone + std::marker::Send + std::marker::Sync,
{
    buffers: HashMap<T, AllocRingBuffer<PayloadWrapper<M>>>,
    capacity: usize,
    ts: TimestampFn,
}

impl<T, M> RingBufferInboxBackend<T, M>
where
    T: Copy + Default + PartialEq + Eq + Hash + std::marker::Send + std::marker::Sync,
    M: Clone + std::marker::Send + std::marker::Sync,
{
    /// Creates new backend with default timestamping function from std::time.
    pub fn new(capacity: usize) -> Self {
        Self::new_with_capacity(capacity, current_timestamp)
    }

    /// Counts only the untagged entries.
    pub fn count_untagged(&self) -> usize {
        self.buffers.get(&T::default()).map(|buf| buf.len()).unwrap_or(0)
    }

    fn tag_resolution(&mut self, tag: Option<T>) -> Option<T> {
        let specific_tag = match tag {
            // If no tag was given, we need to find a tag which has the oldest entry in it
            None => self
                .buffers
                .iter()
                .min_by(|(_, a), (_, b)| {
                    // Compare timestamps of the oldest entries in buckets
                    // Empty buckets are moved away
                    let ts_a = a.peek().map(|w| w.ts).unwrap_or(Duration::MAX);
                    let ts_b = b.peek().map(|w| w.ts).unwrap_or(Duration::MAX);
                    ts_a.cmp(&ts_b)
                })
                .map(|(t, _)| *t)?,

            // If a tag was given, just use it
            Some(t) => t,
        };

        Some(specific_tag)
    }
}

#[async_trait]
impl<T, M> InboxBackend<T, M> for RingBufferInboxBackend<T, M>
where
    T: Copy + Default + PartialEq + Eq + Hash + std::marker::Send + std::marker::Sync,
    M: Clone + std::marker::Send + std::marker::Sync,
{
    fn new_with_capacity(capacity: usize, ts: TimestampFn) -> Self {
        assert!(capacity.is_power_of_two(), "capacity must be a power of two");
        Self {
            capacity,
            buffers: HashMap::new(),
            ts,
        }
    }

    async fn push(&mut self, tag: Option<T>, payload: M) {
        // Either use an existing ringbuffer or initialize a new one, if such tag does not exist yet.
        match self.buffers.entry(tag.unwrap_or_default()) {
            Entry::Occupied(mut e) => e.get_mut().push(PayloadWrapper {
                payload,
                ts: (self.ts)(),
            }),
            Entry::Vacant(e) => e.insert(AllocRingBuffer::new(self.capacity)).push(PayloadWrapper {
                payload,
                ts: (self.ts)(),
            }),
        }
    }

    async fn count(&self, tag: Option<T>) -> usize {
        match tag {
            // Count across all the tags
            None => self.buffers.values().map(|buf| buf.len()).sum(),
            // Count messages with a specific tag only
            Some(specific_tag) => self.buffers.get(&specific_tag).map(|buf| buf.len()).unwrap_or(0),
        }
    }

    async fn pop(&mut self, tag: Option<T>) -> Option<(M, Duration)> {
        let specific_tag = self.tag_resolution(tag).unwrap();

        self.buffers
            .get_mut(&specific_tag)
            .and_then(|buf| buf.dequeue().map(|w| (w.payload, w.ts)))
    }

    async fn pop_all(&mut self, tag: Option<T>) -> Vec<(M, Duration)> {
        match tag {
            Some(specific_tag) => {
                // Pop only all messages of a specific tag
                self.buffers
                    .get_mut(&specific_tag)
                    .map(|buf| buf.drain().map(|w| (w.payload, w.ts)).collect::<Vec<_>>())
                    .unwrap_or_default()
            }
            None => {
                // Pop across all the tags, need to sort again based on the timestamp
                let mut all = self
                    .buffers
                    .drain()
                    .flat_map(|(_, buf)| buf.into_iter())
                    .collect::<Vec<_>>();

                // NOTE: this approach is due to the requirement of considering
                // messages with equal payload and timestamp to be distinct
                // If this requirement was relaxed, the drained entries could be collected into a BTreeSet.
                all.sort_unstable_by(|a, b| a.ts.cmp(&b.ts));

                all.into_iter().map(|w| (w.payload, w.ts)).collect()
            }
        }
    }

    async fn peek(&mut self, tag: Option<T>) -> Option<(M, Duration)> {
        let specific_tag = self.tag_resolution(tag).unwrap();

        self.buffers
            .get_mut(&specific_tag)
            .and_then(|buf| buf.peek().map(|w| (w.payload.clone(), w.ts)))
    }

    async fn peek_all(&mut self, tag: Option<T>, timestamp: Option<Duration>) -> Vec<(M, Duration)> {
        let timestamp = timestamp.unwrap_or(Duration::from_millis(0));

        match tag {
            Some(specific_tag) => {
                // Peek only all messages of a specific tag
                self.buffers
                    .get_mut(&specific_tag)
                    .map(|buf| {
                        buf.iter()
                            .filter_map(|w| {
                                if w.ts >= timestamp {
                                    Some((w.payload.clone(), w.ts))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            }
            None => {
                // Peek across all the tags, need to sort again based on the timestamp
                let mut all = self
                    .buffers
                    .iter()
                    .flat_map(|(_, buf)| buf.into_iter())
                    .collect::<Vec<_>>();

                // NOTE: this approach is due to the requirement of considering
                // messages with equal payload and timestamp to be distinct
                // If this requirement was relaxed, the drained entries could be collected into a BTreeSet.
                all.sort_unstable_by(|a, b| a.ts.cmp(&b.ts));

                all.into_iter().map(|w| (w.payload.clone(), w.ts)).collect()
            }
        }
    }

    async fn purge(&mut self, older_than: Duration) {
        // Remove all the messages across all the tags, which do not satisfy the threshold
        self.buffers.iter_mut().for_each(|(_, buf)| {
            while buf.peek().map(|w| w.ts).unwrap_or(Duration::MAX) < older_than {
                buf.dequeue();
            }
        });
    }
}

#[cfg(test)]
mod test {
    use crate::inbox::InboxBackend;
    use crate::ring::RingBufferInboxBackend;
    use std::ops::Add;
    use std::time::Duration;

    fn make_backend(capacity: usize) -> RingBufferInboxBackend<u16, i32> {
        RingBufferInboxBackend::new_with_capacity(
            capacity,
            || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .add(Duration::from_millis(1))
            }, // for testing, ensure the timestamps are at least 5ms apart
        )
    }

    #[async_std::test]
    async fn test_push_pop_tag() {
        let mut rb = make_backend(4);

        rb.push(Some(1), 1).await;
        rb.push(Some(1), 2).await;
        rb.push(Some(1), 3).await;
        rb.push(Some(1), 4).await;
        rb.push(Some(1), 5).await;

        rb.push(Some(10), 6).await;
        rb.push(Some(11), 7).await;

        rb.push(None, 0).await;

        assert_eq!(4, rb.count(Some(1)).await);
        assert_eq!(1, rb.count(Some(10)).await);
        assert_eq!(1, rb.count(Some(11)).await);
        assert_eq!(0, rb.count(Some(100)).await);
        assert_eq!(0, rb.count(Some(23)).await);
        assert_eq!(7, rb.count(None).await);
        assert_eq!(1, rb.count_untagged());

        assert_eq!(2, rb.pop(Some(1)).await.unwrap().0);
        assert_eq!(3, rb.pop(Some(1)).await.unwrap().0);
        assert_eq!(4, rb.pop(Some(1)).await.unwrap().0);
        assert_eq!(5, rb.pop(Some(1)).await.unwrap().0);
        assert_eq!(0, rb.count(Some(1)).await);
        assert!(rb.pop(Some(1)).await.is_none());

        assert!(rb.pop(Some(100)).await.is_none());
        assert!(rb.pop(Some(23)).await.is_none());

        assert_eq!(6, rb.pop(Some(10)).await.unwrap().0);
        assert_eq!(0, rb.count(Some(10)).await);

        assert_eq!(7, rb.pop(Some(11)).await.unwrap().0);
        assert_eq!(0, rb.count(Some(11)).await);

        assert_eq!(0, rb.pop(None).await.unwrap().0);
        assert_eq!(0, rb.count_untagged());

        assert_eq!(0, rb.count(None).await);

        rb.push(None, 0).await;
        rb.push(None, 0).await;
        assert_eq!(2, rb.count_untagged());
        assert_eq!(2, rb.count(None).await);
    }

    #[async_std::test]
    async fn test_pop_all() {
        let mut rb = make_backend(2);

        rb.push(Some(1), 0).await;
        rb.push(Some(1), 1).await;
        rb.push(Some(1), 2).await;

        rb.push(Some(2), 3).await;
        rb.push(Some(2), 4).await;

        rb.push(None, 5).await;

        assert_eq!(
            vec![1, 2, 3, 4, 5],
            rb.pop_all(None).await.into_iter().map(|(d, _)| d).collect::<Vec<_>>()
        );
        assert_eq!(0, rb.count(None).await);
        assert_eq!(0, rb.count_untagged());
    }

    #[async_std::test]
    async fn test_pop_all_specific() {
        let mut rb = make_backend(2);

        rb.push(Some(1), 0).await;
        rb.push(Some(1), 2).await;
        rb.push(Some(1), 1).await;

        rb.push(Some(2), 3).await;
        rb.push(Some(2), 4).await;

        rb.push(None, 5).await;

        assert_eq!(
            vec![2, 1],
            rb.pop_all(Some(1))
                .await
                .into_iter()
                .map(|(d, _)| d)
                .collect::<Vec<_>>()
        );
        assert_eq!(2, rb.count(Some(2)).await);
        assert_eq!(3, rb.count(None).await);
    }

    #[async_std::test]
    async fn test_pop_oldest() {
        let mut rb = make_backend(2);

        rb.push(Some(3), 10).await;

        rb.push(Some(1), 1).await;
        rb.push(Some(1), 2).await;

        rb.push(Some(2), 3).await;
        rb.push(Some(2), 4).await;

        assert_eq!(5, rb.count(None).await);

        assert_eq!(10, rb.pop(None).await.unwrap().0);

        assert_eq!(0, rb.count(Some(3)).await);
        assert_eq!(4, rb.count(None).await);

        assert_eq!(1, rb.pop(None).await.unwrap().0);
        assert_eq!(2, rb.pop(None).await.unwrap().0);

        assert_eq!(0, rb.count(Some(1)).await);
        assert_eq!(2, rb.count(Some(2)).await);

        assert_eq!(3, rb.pop(None).await.unwrap().0);
        assert_eq!(4, rb.pop(None).await.unwrap().0);

        assert_eq!(0, rb.count(Some(2)).await);
        assert_eq!(0, rb.count(None).await);
    }

    #[async_std::test]
    async fn test_peek_oldest() {
        let mut rb = make_backend(2);

        rb.push(Some(3), 10).await;

        rb.push(Some(1), 1).await;
        rb.push(Some(1), 2).await;

        rb.push(Some(2), 3).await;
        rb.push(Some(2), 4).await;

        assert_eq!(5, rb.count(None).await);

        assert_eq!(10, rb.peek(None).await.unwrap().0);

        assert_eq!(5, rb.count(None).await);
    }

    #[async_std::test]
    async fn test_peek_oldest_specific() {
        let mut rb = make_backend(2);

        rb.push(Some(3), 10).await;

        rb.push(Some(1), 1).await;
        rb.push(Some(1), 2).await;

        rb.push(Some(2), 3).await;
        rb.push(Some(2), 4).await;

        assert_eq!(5, rb.count(None).await);
        assert_eq!(1, rb.count(Some(3)).await);
        assert_eq!(2, rb.count(Some(1)).await);
        assert_eq!(2, rb.count(Some(2)).await);

        assert_eq!(10, rb.peek(Some(3)).await.unwrap().0);
        assert_eq!(1, rb.peek(Some(1)).await.unwrap().0);
        assert_eq!(3, rb.peek(Some(2)).await.unwrap().0);

        assert_eq!(5, rb.count(None).await);
        assert_eq!(1, rb.count(Some(3)).await);
        assert_eq!(2, rb.count(Some(1)).await);
        assert_eq!(2, rb.count(Some(2)).await);
    }

    #[async_std::test]
    async fn test_peek_all() {
        let mut rb = make_backend(4);

        rb.push(Some(1), 0).await;
        rb.push(Some(1), 2).await;
        rb.push(Some(1), 1).await;

        assert_eq!(
            vec![0, 2, 1],
            rb.peek_all(None, None)
                .await
                .into_iter()
                .map(|(d, _)| d)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            vec![0, 2, 1],
            rb.peek_all(None, None)
                .await
                .into_iter()
                .map(|(d, _)| d)
                .collect::<Vec<_>>()
        );
        assert_eq!(3, rb.count(Some(1)).await);

        rb.pop_all(None).await;
    }

    #[async_std::test]
    async fn test_peek_all_specific() {
        let mut rb = make_backend(4);

        rb.push(Some(1), 0).await;
        rb.push(Some(1), 2).await;
        rb.push(Some(1), 1).await;

        rb.push(Some(2), 3).await;
        rb.push(Some(2), 4).await;

        rb.push(None, 5).await;

        assert_eq!(
            vec![0, 2, 1],
            rb.peek_all(Some(1), None)
                .await
                .into_iter()
                .map(|(d, _)| d)
                .collect::<Vec<_>>()
        );
        assert_eq!(3, rb.count(Some(1)).await);
        assert_eq!(2, rb.count(Some(2)).await);
        assert_eq!(6, rb.count(None).await);

        rb.pop_all(None).await;
    }

    #[async_std::test]
    async fn test_peek_all_with_timestamp() {
        let mut rb = make_backend(8);

        rb.push(Some(2), 0).await;
        rb.push(Some(1), 0).await;
        rb.push(Some(1), 1).await;
        rb.push(Some(1), 2).await;
        rb.push(Some(1), 3).await;
        rb.push(Some(1), 4).await;
        rb.push(Some(1), 5).await;

        // sleep for 10ms to ensure a break between timestamps
        async_std::task::sleep(Duration::from_millis(10)).await;
        let close_past = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        rb.push(Some(1), 6).await;
        rb.push(Some(1), 7).await;
        rb.push(Some(2), 1).await;

        assert_eq!(
            vec![6, 7],
            rb.peek_all(Some(1), Some(close_past))
                .await
                .into_iter()
                .map(|(d, _)| d)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            vec![1],
            rb.peek_all(Some(2), Some(close_past))
                .await
                .into_iter()
                .map(|(d, _)| d)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            rb.peek_all(Some(1), None)
                .await
                .into_iter()
                .map(|(d, _)| d)
                .collect::<Vec<_>>()
        );

        rb.pop_all(None).await;
    }

    #[async_std::test]
    async fn test_purge() {
        let mut rb = make_backend(8);

        rb.push(None, 0).await;
        rb.push(None, 1).await;
        rb.push(None, 2).await;
        rb.push(None, 3).await;

        async_std::task::sleep(Duration::from_millis(100)).await;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        rb.push(None, 4).await;
        rb.push(None, 5).await;
        rb.push(None, 6).await;
        rb.push(None, 7).await;

        assert_eq!(8, rb.count(None).await);

        rb.purge(ts).await;

        assert_eq!(4, rb.count(None).await);
        assert_eq!(
            vec![4, 5, 6, 7],
            rb.pop_all(None).await.into_iter().map(|(d, _)| d).collect::<Vec<_>>()
        );
    }

    #[async_std::test]
    async fn test_duplicates() {
        let mut rb = make_backend(4);

        rb.push(None, 1).await;
        rb.push(None, 0).await;
        rb.push(None, 0).await;
        rb.push(None, 0).await;
        rb.push(None, 0).await;

        rb.push(Some(1), 1).await;
        rb.push(Some(1), 0).await;
        rb.push(Some(1), 0).await;
        rb.push(Some(1), 0).await;
        rb.push(Some(1), 0).await;

        assert_eq!(
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            rb.pop_all(None).await.into_iter().map(|(d, _)| d).collect::<Vec<_>>()
        );
    }
}
