use std::collections::hash_map::Entry;
use std::collections::HashMap;
use async_trait::async_trait;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use crate::inbox::{InboxBackend, RESERVED_TAG, Tag};

struct PayloadWrapper<T> {
    payload: T,
    ts: u64
}

pub struct RingBufferInboxBackend<T> {
    buffers: HashMap<Tag, AllocRingBuffer<PayloadWrapper<T>>>,
    capacity: usize
}

impl<T> RingBufferInboxBackend<T> {
    fn ts() -> u64 {
        // TODO:
        0u64
    }
}

#[async_trait(? Send)]
impl<T> InboxBackend<T> for RingBufferInboxBackend<T> {
    fn new_with_capacity(capacity: usize) -> Self {
        let mut buffers = HashMap::new();
        buffers.insert(RESERVED_TAG, AllocRingBuffer::new(capacity));
        Self { capacity, buffers }
    }

    async fn push(&mut self, tag: Option<Tag>, payload: T) {
        match self.buffers.entry(tag.unwrap_or(RESERVED_TAG)) {
            Entry::Occupied(mut e) => {
                e.get_mut()
                    .push(PayloadWrapper { payload, ts: Self::ts() })
            }
            Entry::Vacant(e) => {
                e.insert(AllocRingBuffer::new(self.capacity))
                    .push(PayloadWrapper { payload, ts: Self::ts() })
            }
        }
    }

    async fn count(&self, tag: Option<Tag>) -> usize {
        self.buffers
            .get(&tag.unwrap_or(RESERVED_TAG))
            .map(|buf| buf.len())
            .unwrap_or(0)
    }

    async fn pop(&mut self, tag: Option<Tag>) -> Option<T> {
        self.buffers
            .get_mut(&tag.unwrap_or(RESERVED_TAG))
            .and_then(|buf| buf.dequeue().map(|w| w.payload))
    }

    async fn pop_all(&mut self, tag: Option<Tag>) -> Vec<T> {
        self.buffers
            .get_mut(&tag.unwrap_or(RESERVED_TAG))
            .map(|buf| buf.drain().map(|w| w.payload).collect::<Vec<_>>())
            .unwrap_or_else(Vec::<T>::new)
    }

    async fn purge(&mut self, older_than_ts: u64) {
        self.buffers
            .iter_mut()
            .for_each(|(_, buf)| {
                while buf.peek().map(|w| w.ts).unwrap_or(u64::MAX) < older_than_ts {
                    buf.dequeue();
                }
            });
    }
}