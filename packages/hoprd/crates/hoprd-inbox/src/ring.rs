use std::collections::hash_map::Entry;
use std::collections::HashMap;
use async_trait::async_trait;
use ringbuffer::{AllocRingBuffer};
use crate::inbox::{InboxBackend, Tag};

pub struct RingBufferInboxBackend<T> {
    buffers: HashMap<Tag, AllocRingBuffer<T>>,
    capacity: usize
}

impl<T> RingBufferInboxBackend<T> {
    fn add_or_get_buffer(&mut self, tag: Tag) -> &mut AllocRingBuffer<T> {
        match self.buffers.entry(tag) {
            Entry::Occupied(mut e) => {
                e.get_mut()
            }
            Entry::Vacant(e) => {
                e.insert(AllocRingBuffer::new(self.capacity))
            }
        }
    }
}

#[async_trait(? Send)]
impl<T> InboxBackend<T> for RingBufferInboxBackend<T> {
    fn new_with_capacity(capacity: usize) -> Self {
        Self { capacity, buffers: HashMap::new() }
    }

    async fn push(&mut self, tag: crate::inbox::Tag, payload: T) {
        todo!()
    }

    async fn count(&self, tag: Option<Tag>) -> usize {
        todo!()
    }

    async fn pop(&mut self, tag: Option<crate::inbox::Tag>) -> Option<T> {
        todo!()
    }

    async fn pop_all(&mut self, tag: Option<crate::inbox::Tag>) -> Vec<T> {
        todo!()
    }

    async fn purge(&mut self, older_than_ts: u64) {
        todo!()
    }
}