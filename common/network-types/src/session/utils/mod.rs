pub mod skip_queue;

#[cfg(test)]
pub mod test;

use std::{
    cmp::Ordering,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::StreamExt;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::session::frames::FrameId;

#[derive(Debug)]
pub(crate) struct RingBufferProducer<T>(futures::channel::mpsc::Sender<T>);

impl<T> Clone for RingBufferProducer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> RingBufferProducer<T> {
    pub fn push(&mut self, item: T) -> bool {
        self.0.try_send(item).is_ok()
    }
}

#[derive(Debug)]
pub(crate) struct RingBufferView<T>(Arc<parking_lot::FairMutex<AllocRingBuffer<T>>>);

impl<T> Clone for RingBufferView<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone> RingBufferView<T> {
    pub fn find<F: Fn(&T) -> bool>(&self, predicate: F) -> Vec<T> {
        self.0.lock().iter().filter(|item| predicate(item)).cloned().collect()
    }
}

pub(crate) fn searchable_ringbuffer<T: Send + Sync + 'static>(
    capacity: usize,
) -> (RingBufferProducer<T>, RingBufferView<T>) {
    let (rb_tx, rb_rx) = futures::channel::mpsc::channel(capacity * 2);
    let rb = Arc::new(parking_lot::FairMutex::new(AllocRingBuffer::new(capacity)));

    let rb_clone = rb.clone();
    hopr_async_runtime::prelude::spawn(rb_rx.for_each(move |s| {
        rb_clone.lock().push(s);
        futures::future::ready(())
    }));

    (RingBufferProducer(rb_tx), RingBufferView(rb))
}

pub(crate) fn next_deadline_with_backoff(n: usize, base: f64, duration: Duration) -> Instant {
    Instant::now() + duration.mul_f64(base.powi(n as i32 + 1))
}

#[derive(Debug, Copy, Clone, Eq)]
pub(crate) struct RetriedFrameId {
    pub frame_id: FrameId,
    pub retry_count: usize,
    max_retries: usize,
}

impl RetriedFrameId {
    pub fn new(frame_id: FrameId) -> Self {
        Self {
            frame_id,
            retry_count: 1,
            max_retries: 1,
        }
    }

    pub fn with_retries(frame_id: FrameId, max_retries: usize) -> Self {
        Self {
            frame_id,
            retry_count: 1,
            max_retries,
        }
    }

    pub fn next(self) -> Option<Self> {
        if self.retry_count < self.max_retries {
            Some(Self {
                frame_id: self.frame_id,
                retry_count: self.retry_count + 1,
                max_retries: self.max_retries,
            })
        } else {
            None
        }
    }
}

impl PartialEq<Self> for RetriedFrameId {
    fn eq(&self, other: &Self) -> bool {
        self.frame_id.eq(&other.frame_id)
    }
}

impl PartialOrd<Self> for RetriedFrameId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RetriedFrameId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.frame_id.cmp(&other.frame_id)
    }
}
