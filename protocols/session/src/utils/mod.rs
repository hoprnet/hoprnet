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

use crate::{
    errors::SessionError,
    frames::{FrameId, Segment, SeqIndicator, SeqNum},
};

pub(crate) fn to_hex_shortened(data: &impl AsRef<[u8]>, max_chars: usize) -> String {
    let data = data.as_ref();
    if data.len() > max_chars {
        format!(
            "{}..{}",
            hex::encode(&data[0..max_chars / 2]),
            hex::encode(&data[data.len() - max_chars / 2..])
        )
    } else {
        hex::encode(data)
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
}

#[derive(Debug)]
pub(crate) struct RingBufferView<T>(Arc<parking_lot::FairMutex<AllocRingBuffer<T>>>);

impl<T> Clone for RingBufferView<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone> RingBufferView<T> {
    pub fn find<F: FnMut(&T) -> bool>(&self, mut predicate: F) -> Vec<T> {
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
        rb_clone.lock().enqueue(s);
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

/// Helper function to segment `data` into segments of a given ` max_segment_size ` length.
/// All segments are tagged with the same `frame_id` and output into the given ` segments ` buffer.
pub fn segment_into<T: AsRef<[u8]>, E: Extend<Segment>>(
    data: T,
    max_segment_size: usize,
    frame_id: FrameId,
    segments: &mut E,
) -> crate::errors::Result<()> {
    if frame_id == 0 {
        return Err(SessionError::InvalidFrameId);
    }

    if max_segment_size == 0 {
        return Err(SessionError::IncorrectMessageLength);
    }

    let data = data.as_ref();

    let num_chunks = data.len().div_ceil(max_segment_size);
    if num_chunks > SeqNum::MAX as usize {
        return Err(SessionError::DataTooLong);
    }

    let chunks = data.chunks(max_segment_size);

    let seq_len = SeqIndicator::try_from(chunks.len() as SeqNum)?;
    segments.extend(chunks.enumerate().map(|(idx, data)| Segment {
        frame_id,
        seq_flags: seq_len,
        seq_idx: idx as u8,
        data: data.into(),
    }));

    Ok(())
}

/// Convenience wrapper for [`segment_into`] that allocates its own output buffer and returns it.
#[allow(unused)]
pub fn segment<T: AsRef<[u8]>>(data: T, max_segment_size: usize, frame_id: u32) -> crate::errors::Result<Vec<Segment>> {
    let mut out = Vec::with_capacity(data.as_ref().len().div_ceil(max_segment_size));
    segment_into(data, max_segment_size, frame_id, &mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn segment_should_split_data_correctly() -> anyhow::Result<()> {
        let data = hex!("deadbeefcafebabe");

        let segments = segment(data, 3, 1)?;
        assert_eq!(3, segments.len());

        assert_eq!(hex!("deadbe"), segments[0].data.as_ref());
        assert_eq!(0, segments[0].seq_idx);
        assert_eq!(3, segments[0].seq_flags.seq_len());
        assert_eq!(1, segments[0].frame_id);

        assert_eq!(hex!("efcafe"), segments[1].data.as_ref());
        assert_eq!(1, segments[1].seq_idx);
        assert_eq!(3, segments[1].seq_flags.seq_len());
        assert_eq!(1, segments[1].frame_id);

        assert_eq!(hex!("babe"), segments[2].data.as_ref());
        assert_eq!(2, segments[2].seq_idx);
        assert_eq!(3, segments[2].seq_flags.seq_len());
        assert_eq!(1, segments[2].frame_id);

        Ok(())
    }
}
