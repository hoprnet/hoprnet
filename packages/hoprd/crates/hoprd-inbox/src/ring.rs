use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::time::Duration;
use async_trait::async_trait;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use crate::inbox::{InboxBackend, RESERVED_TAG, Tag, TimestampFn};

struct PayloadWrapper<T: PartialEq> {
    payload: T,
    ts: Duration
}

impl<T: PartialEq> Eq for PayloadWrapper<T> {}

impl<T: PartialEq> PartialEq<Self> for PayloadWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.payload.eq(&other.payload)
    }
}

impl<T: PartialEq> PartialOrd<Self> for PayloadWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: PartialEq> Ord for PayloadWrapper<T>{
    fn cmp(&self, other: &Self) -> Ordering {
        let c = self.ts.cmp(&other.ts);
        match &c {
            Ordering::Equal => {
                if !self.eq(other) {
                    // In an unlikely situation when timestamps are equal, but
                    // the payloads are different, prefer the other.
                    // This allows insertion into the BTreeSet and still having the items sorted by TS.
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
            _ => c,
        }
    }
}

/// Ring buffer based heap-allocated backend.
/// The capacity must be a power-of-two due to optimizations.
pub struct RingBufferInboxBackend<T: PartialEq> {
    buffers: HashMap<Tag, AllocRingBuffer<PayloadWrapper<T>>>,
    capacity: usize,
    ts: TimestampFn
}

impl<T: PartialEq> RingBufferInboxBackend<T> {
    #[cfg(not(feature = "wasm"))]
    pub fn new(capacity: usize) -> Self {
        Self::new_with_capacity(capacity, || std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap())
    }

    /// Counts only the untagged entries.
    pub fn count_untagged(&self) -> usize {
        self.buffers.get(&RESERVED_TAG).map(|buf| buf.len()).unwrap_or(0)
    }
}

#[async_trait(? Send)]
impl<T: PartialEq> InboxBackend<T> for RingBufferInboxBackend<T> {
    fn new_with_capacity(capacity: usize, ts: TimestampFn) -> Self {
        assert!(capacity.is_power_of_two(), "capacity must be a power of two");
        Self { capacity, buffers: HashMap::new(), ts}
    }

    async fn push(&mut self, tag: Option<Tag>, payload: T) {
        match self.buffers.entry(tag.unwrap_or(RESERVED_TAG)) {
            Entry::Occupied(mut e) => {
                e.get_mut()
                    .push(PayloadWrapper { payload, ts: (self.ts)() })
            }
            Entry::Vacant(e) => {
                e.insert(AllocRingBuffer::new(self.capacity))
                    .push(PayloadWrapper { payload, ts: (self.ts)() })
            }
        }
    }

    async fn count(&self, tag: Option<Tag>) -> usize {
        match tag {
            None => self.buffers.iter().map(|(_,buf)| buf.len()).sum(),
            Some(specific_tag) => {
                self.buffers
                    .get(&specific_tag)
                    .map(|buf| buf.len())
                    .unwrap_or(0)
            }
        }
    }

    async fn pop(&mut self, tag: Option<Tag>) -> Option<T> {
        // If no tag was given, we need to find a tag which has the oldest entry in it
        let specific_tag = match tag {
            None => self
                .buffers
                .iter()
                .min_by(|(_, a),(_,b)| {
                    // Compare timestamps of the oldest entries in buckets
                    // Empty buckets are moved away
                    let ts_a = a.peek().map(|w|w.ts).unwrap_or(Duration::MAX);
                    let ts_b = b.peek().map(|w|w.ts).unwrap_or(Duration::MAX);
                    ts_a.cmp(&ts_b)
                })
                .map(|(t,_)| *t)?,
            Some(t) => t
        };

        self.buffers
            .get_mut(&specific_tag)
            .and_then(|buf| buf.dequeue().map(|w| w.payload))
    }

    async fn pop_all(&mut self, tag: Option<Tag>) -> Vec<T> {
        match tag {
            Some(specific_tag) => {
                // Pop only all messages of a specific tag
                self.buffers
                    .get_mut(&specific_tag)
                    .map(|buf| buf.drain().map(|w| w.payload).collect::<Vec<_>>())
                    .unwrap_or_else(Vec::<T>::new)
            }
            None => {
                // Pop across all the tags, need to sort again based on timestamp
                self.buffers
                    .drain()
                    .flat_map(|(_, buf)| buf.into_iter())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .map(|w| w.payload)
                    .collect()
            }
        }

    }

    async fn purge(&mut self, older_than: Duration) {
        self.buffers
            .iter_mut()
            .for_each(|(_, buf)| {
                while buf.peek().map(|w| w.ts).unwrap_or(Duration::MAX) < older_than {
                    buf.dequeue();
                }
            });
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::time::Duration;
    use crate::inbox::InboxBackend;
    use crate::ring::RingBufferInboxBackend;

    fn make_backend(capacity: usize) -> RingBufferInboxBackend<i32> {
        RingBufferInboxBackend::new_with_capacity(capacity, ||
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .add(Duration::from_secs(1)) // for testing, ensure the timestamps are at least 1 s apart
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
        assert_eq!(7, rb.count(None).await);
        assert_eq!(1, rb.count_untagged());

        assert_eq!(2, rb.pop(Some(1)).await.unwrap());
        assert_eq!(3, rb.pop(Some(1)).await.unwrap());
        assert_eq!(4, rb.pop(Some(1)).await.unwrap());
        assert_eq!(5, rb.pop(Some(1)).await.unwrap());
        assert_eq!(0, rb.count(Some(1)).await);

        assert_eq!(6, rb.pop(Some(10)).await.unwrap());
        assert_eq!(0, rb.count(Some(10)).await);

        assert_eq!(7, rb.pop(Some(11)).await.unwrap());
        assert_eq!(0, rb.count(Some(11)).await);

        assert_eq!(0, rb.pop(None).await.unwrap());
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

        assert_eq!(vec![1,2,3,4,5], rb.pop_all(None).await);
        assert_eq!(0, rb.count(None).await);
        assert_eq!(0, rb.count_untagged());
    }

    #[async_std::test]
    async fn test_pop_all_specific() {
        let mut rb = make_backend(2);

        rb.push(Some(1), 0).await;
        rb.push(Some(1), 1).await;
        rb.push(Some(1), 2).await;

        rb.push(Some(2), 3).await;
        rb.push(Some(2), 4).await;

        rb.push(None, 5).await;

        assert_eq!(vec![1,2], rb.pop_all(Some(1)).await);
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

        assert_eq!(10, rb.pop(None).await.unwrap());

        assert_eq!(0, rb.count(Some(3)).await);
        assert_eq!(4, rb.count(None).await);

        assert_eq!(1, rb.pop(None).await.unwrap());
        assert_eq!(2, rb.pop(None).await.unwrap());

        assert_eq!(0, rb.count(Some(1)).await);
        assert_eq!(2, rb.count(Some(2)).await);

        assert_eq!(3, rb.pop(None).await.unwrap());
        assert_eq!(4, rb.pop(None).await.unwrap());

        assert_eq!(0, rb.count(Some(2)).await);
        assert_eq!(0, rb.count(None).await);
    }
}