pub mod skip_queue;

#[cfg(test)]
pub mod test;

use std::{
    cmp::Ordering,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::StreamExt;
use rand::prelude::{Distribution, thread_rng};
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::prelude::FrameId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RetryToken {
    pub num_retry: usize,
    pub started_at: Instant,
    backoff_base: f64,
    created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RetryResult {
    Wait(Duration),
    RetryNow(RetryToken),
    Expired,
}

impl RetryToken {
    pub fn new(backoff_base: f64) -> Self {
        Self::from_instant(Instant::now(), backoff_base)
    }

    pub fn from_instant(now: Instant, backoff_base: f64) -> Self {
        Self {
            num_retry: 0,
            started_at: now,
            created_at: Instant::now(),
            backoff_base,
        }
    }

    pub fn replenish(self) -> Self {
        Self {
            num_retry: 0,
            started_at: Instant::now(),
            created_at: self.created_at,
            backoff_base: self.backoff_base,
        }
    }

    fn retry_in(&self, base: Duration, max_duration: Duration, jitter_dev: f64) -> Option<Duration> {
        let jitter_coeff = if jitter_dev > 0.0 {
            // Should not use jitter with sigma > 0.25
            rand_distr::Normal::new(1.0, jitter_dev.min(0.25))
                .unwrap()
                .sample(&mut thread_rng())
                .abs()
        } else {
            1.0
        };

        // jitter * base * backoff_base ^ num_retry
        let duration = base.mul_f64(jitter_coeff * self.backoff_base.powi(self.num_retry as i32));
        (duration < max_duration).then_some(duration)
    }

    pub fn check(&self, now: Instant, base: Duration, max: Duration, jitter_dev: f64) -> RetryResult {
        match self.retry_in(base, max, jitter_dev) {
            None => RetryResult::Expired,
            Some(retry_in) if self.started_at + retry_in >= now => RetryResult::Wait(self.started_at + retry_in - now),
            _ => RetryResult::RetryNow(Self {
                num_retry: self.num_retry + 1,
                started_at: self.started_at,
                backoff_base: self.backoff_base,
                created_at: self.created_at,
            }),
        }
    }

    pub fn time_since_creation(&self) -> Duration {
        self.created_at.elapsed()
    }
}

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

    pub fn is_open(&self) -> bool {
        !self.0.is_closed()
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
            retry_count: 0,
            max_retries: 0,
        }
    }

    pub fn with_retries(frame_id: FrameId, max_retries: usize) -> Self {
        Self {
            frame_id,
            retry_count: 0,
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
