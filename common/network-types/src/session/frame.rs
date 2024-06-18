//! This module implements segmentation of [frames][Frame] into [segments][Segment] and
//! their [reassembly](FrameReassembler) back into [`Frames`](Frame) and their sequencing.
//!
//! ## Frames
//! Contain data of arbitrary length up to 65536 bytes, differently sized frames are supported.
//! Each frame carries a [`frame_id`](FrameId) which
//! should be unique within some higher level session. Frame ID ranges from 1 to 2^32-1.
//! Frame ID of 0 is not allowed, and its segments cannot be pushed to the reassembler.
//!
//! ## Segmentation
//! A [frame](Frame) can be [segmented](Frame::segment) into equally sized [`Segments`](Segment),
//! each of them carrying its [sequence number](SeqNum).
//! This operation runs in linear time with respect to the size of the frame.
//! There can be up to 256 segments per frame.
//! Frame segments are uniquely identified via [`SegmentId`].
//!
//! ## Reassembly
//! This is an inverse operation to segmentation. Reassembly is performed by a [`FrameReassembler`]
//! and is implemented lock-free. The reassembler acts as a [`Sink`] for [`Segments`](Segment) and
//! is always paired with a [`Stream`] that outputs the reassembled [`Frames`](Frame).
//!
//! ### Ordering
//! The reassembled frames will always have the segments in correct order, and complete frames emitted
//! from the reassembler will also be ordered correctly according to their frame IDs.
//! If the next frame in sequence cannot be completed within the `max_age` period given
//! upon [construction](FrameReassembler::new) of the reassembler, [`NetworkTypeError::FrameDiscarded`]
//! error will be emitted by the reassembler (see the next section).
//!
//! ### Expiration
//! The reassembler also implements segment expiration. Upon [construction](FrameReassembler::new), the maximum
//! incomplete frame age can be specified. If a frame is not completed in the reassembler within
//! this period, it can be [evicted](FrameReassembler::evict) from the reassembler, so that it will be lost
//! forever.
//! The eviction operation is supposed to be run periodically, so that the space could be freed up in the
//! reassembler, and the reassembler does not wait indefinitely for the next frame in sequence.
//!
//! Beware that once eviction is performed and an incomplete frame with ID `n` is destroyed;
//! the caller should make sure that frames with ID <= `n` will not arrive into the reassembler,
//! otherwise the [NetworkTypeError::OldSegment] error will be thrown.

use bitvec::array::BitArray;
use bitvec::{bitarr, BitArr};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::{Sink, Stream};
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::ops::{Add, Sub};
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::OnceLock;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

use hopr_platform::time::native::current_time;
use hopr_primitive_types::prelude::AsUnixTimestamp;

use crate::errors::NetworkTypeError;

/// ID of a [Frame].
pub type FrameId = u32;
/// Type representing the sequence numbers in a [Frame].
pub type SeqNum = u8;

/// Convenience type that identifies a segment within a frame.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SegmentId(pub FrameId, pub SeqNum);

impl From<&Segment> for SegmentId {
    fn from(value: &Segment) -> Self {
        value.id()
    }
}

impl Display for SegmentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "seg({},{})", self.0, self.1)
    }
}

/// Helper function to segment `data` into segments of given `mtu` length.
/// All segments are tagged with the same `frame_id`.
pub fn segment(data: &[u8], max_segment_size: usize, frame_id: u32) -> crate::errors::Result<Vec<Segment>> {
    if frame_id == 0 {
        return Err(NetworkTypeError::InvalidFrameId);
    }

    let chunks = data.chunks(max_segment_size);
    if chunks.len() > SeqNum::MAX as usize {
        return Err(NetworkTypeError::DataTooLong);
    }

    let seq_len = chunks.len() as SeqNum;
    Ok(chunks
        .enumerate()
        .map(|(idx, data)| Segment {
            frame_id,
            seq_len,
            seq_idx: idx as u8,
            data: data.into(),
        })
        .collect())
}

/// Data frame of arbitrary length.
/// The frame can be segmented into [segments](Segment) and reassembled back
/// via [FrameReassembler].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Identifier of this frame.
    pub frame_id: FrameId,
    /// Frame data.
    pub data: Box<[u8]>,
}

impl Frame {
    /// Segments this frame into a list of [segments](Segment) each of maximum sizes `mtu`.
    #[inline]
    pub fn segment(&self, max_segment_size: usize) -> crate::errors::Result<Vec<Segment>> {
        segment(self.data.as_ref(), max_segment_size, self.frame_id)
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

/// Represents a frame segment.
/// Besides the data, a segment carries information about the total number of
/// segments in the original frame, its index within the frame and
/// ID of that frame.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    /// ID of the [Frame] this segment belongs to.
    pub frame_id: FrameId,
    /// Index of this segment within the segment sequence.
    pub seq_idx: SeqNum,
    /// Total number of segments within this segment sequence.
    pub seq_len: SeqNum,
    /// Data in this segment.
    #[serde(with = "serde_bytes")]
    pub data: Box<[u8]>,
}

impl Segment {
    /// Size of the segment header.
    pub const HEADER_SIZE: usize = mem::size_of::<FrameId>() + 2 * mem::size_of::<SeqNum>();

    /// Returns the [SegmentId] for this segment.
    pub fn id(&self) -> SegmentId {
        SegmentId(self.frame_id, self.seq_idx)
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Segment")
            .field("frame_id", &self.frame_id)
            .field("seq_id", &self.seq_idx)
            .field("seq_len", &self.seq_len)
            .field("data", &hex::encode(&self.data))
            .finish()
    }
}

impl PartialOrd<Segment> for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.frame_id.cmp(&other.frame_id) {
            std::cmp::Ordering::Equal => self.seq_idx.cmp(&other.seq_idx),
            cmp => cmp,
        }
    }
}

impl From<Segment> for Vec<u8> {
    fn from(value: Segment) -> Self {
        let mut ret = Vec::with_capacity(Segment::HEADER_SIZE + value.data.len());
        ret.extend_from_slice(value.frame_id.to_be_bytes().as_ref());
        ret.extend_from_slice(value.seq_idx.to_be_bytes().as_ref());
        ret.extend_from_slice(value.seq_len.to_be_bytes().as_ref());
        ret.extend_from_slice(value.data.as_ref());
        ret
    }
}

impl TryFrom<&[u8]> for Segment {
    type Error = NetworkTypeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let (header, data) = value.split_at(Self::HEADER_SIZE);
        let segment = Segment {
            frame_id: FrameId::from_be_bytes(header[0..4].try_into().map_err(|_| NetworkTypeError::InvalidSegment)?),
            seq_idx: SeqNum::from_be_bytes(header[4..5].try_into().map_err(|_| NetworkTypeError::InvalidSegment)?),
            seq_len: SeqNum::from_be_bytes(header[5..6].try_into().map_err(|_| NetworkTypeError::InvalidSegment)?),
            data: data.into(),
        };
        (segment.frame_id > 0 && segment.seq_idx < segment.seq_len)
            .then_some(segment)
            .ok_or(NetworkTypeError::InvalidSegment)
    }
}

/// Rebuilds the [Frame] from [Segments](Segment).
#[derive(Debug)]
struct FrameBuilder {
    frame_id: FrameId,
    segments: Vec<OnceLock<Box<[u8]>>>,
    remaining: AtomicU8,
    last_ts: AtomicU64,
}

impl FrameBuilder {
    /// Creates a new builder with the given `initial` [Segment] and its timestamp `ts`.
    fn new(initial: Segment, ts: SystemTime) -> Self {
        let ret = Self::empty(initial.frame_id, initial.seq_len);
        ret.put(initial, ts).unwrap();
        ret
    }

    /// Creates a new empty builder for the given frame.
    fn empty(frame_id: FrameId, seq_len: SeqNum) -> Self {
        Self {
            frame_id,
            segments: vec![OnceLock::new(); seq_len as usize],
            remaining: AtomicU8::new(seq_len),
            last_ts: AtomicU64::new(0),
        }
    }

    /// Adds a new [`segment`](Segment) to the builder with a timestamp `ts`.
    /// Returns the number of segments remaining in this builder.
    fn put(&self, segment: Segment, ts: SystemTime) -> crate::errors::Result<SeqNum> {
        if self.frame_id == segment.frame_id {
            if !self.is_complete() {
                if self.segments[segment.seq_idx as usize].set(segment.data).is_ok() {
                    // A new segment has been added, decrease the remaining number and update timestamp
                    self.remaining.fetch_sub(1, Ordering::Relaxed);
                    self.last_ts
                        .fetch_max(ts.as_unix_timestamp().as_millis() as u64, Ordering::Relaxed);
                }
                Ok(self.remaining.load(Ordering::SeqCst))
            } else {
                // Silently throw away segments of a frame that is already complete
                Ok(0)
            }
        } else {
            Err(NetworkTypeError::InvalidFrameId)
        }
    }

    /// Checks if the builder contains all segments of the frame.
    fn is_complete(&self) -> bool {
        self.remaining.load(Ordering::SeqCst) == 0
    }

    /// Checks if the last added segment to this frame happened before `cutoff`.
    /// In other words, the frame under construction is considered expired if the last
    /// segment was added before `cutoff`.
    fn is_expired(&self, cutoff: u64) -> bool {
        self.last_ts.load(Ordering::SeqCst) < cutoff
    }

    /// Returns information about the frame that is being built by this builder.
    pub fn info(&self) -> FrameInfo {
        let mut missing_segments = bitarr![0; 256];
        self.segments
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.get().is_none().then_some(i))
            .for_each(|i| missing_segments.set(i, true));

        FrameInfo {
            frame_id: self.frame_id,
            missing_segments,
            total_segments: self.segments.len() as SeqNum,
            last_update: SystemTime::UNIX_EPOCH.add(Duration::from_millis(self.last_ts.load(Ordering::SeqCst))),
        }
    }

    /// Reassembles the [Frame]. Returns [`NetworkTypeError::IncompleteFrame`] if not [complete](FrameBuilder::is_complete).
    fn reassemble(self) -> crate::errors::Result<Frame> {
        if self.is_complete() {
            Ok(Frame {
                frame_id: self.frame_id,
                data: self
                    .segments
                    .into_iter()
                    .map(|lock| lock.into_inner().unwrap())
                    .collect::<Vec<Box<[u8]>>>()
                    .concat()
                    .into_boxed_slice(),
            })
        } else {
            Err(NetworkTypeError::IncompleteFrame(self.frame_id))
        }
    }
}

/// Contains information about a frame that being built.
/// The instances are totally ordered as most recently used first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameInfo {
    /// ID of the frame.
    pub frame_id: FrameId,
    /// Indices of segments that are missing. Empty if the frame is complete.
    pub missing_segments: BitArr!(for 256),
    /// The total number of segments this frame consists from.
    pub total_segments: SeqNum,
    /// Time of the last received segment in this frame.
    pub last_update: SystemTime,
}

impl FrameInfo {
    /// Transform self into iterator of missing segment numbers.
    pub fn iter_missing_sequence_indices(&self) -> impl Iterator<Item = SeqNum> + '_ {
        self.missing_segments
            .iter()
            .by_vals()
            .enumerate()
            .filter(|(i, s)| *s && *i <= SeqNum::MAX as usize)
            .map(|(s, _)| s as SeqNum)
    }

    pub fn into_missing_segments(self) -> impl Iterator<Item = SegmentId> {
        self.missing_segments
            .into_iter()
            .enumerate()
            .filter(|(i, s)| *s && *i <= SeqNum::MAX as usize)
            .map(move |(i, _)| SegmentId(self.frame_id, i as SeqNum))
    }
}

impl PartialOrd for FrameInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FrameInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.last_update.cmp(&other.last_update) {
            std::cmp::Ordering::Equal => self.frame_id.cmp(&self.frame_id),
            cmp => cmp,
        }
        .reverse()
    }
}

/// Represents a frame reassembler.
///
/// The [`FrameReassembler`] behaves as a [`Sink`] for [`Segment`].
/// Upon creation, also [`Stream`] for reassembled [Frames](Frame) is created.
/// The corresponding stream is closed either when the reassembler is dropped or
/// [`futures::SinkExt::close`] is called.
///
/// As new segments are [pushed](FrameReassembler::push_segment) into the reassembler,
/// the frames get reassembled and once they are completed, they are automatically pushed out into
/// the outgoing frame stream.
///
/// The reassembler can also have a `max_age` of frames that are under construction.
/// The [`evict`](FrameReassembler::evict) method then can be called to remove
/// the incomplete frames over `max_age`. The timestamps are measured with millisecond precision.
///
/// Note that the reassembler is also evicted when dropped.
///
/// ````rust
/// # use std::time::Duration;
/// futures::executor::block_on(async {
/// use hopr_network_types::session::{Frame, FrameReassembler};
/// use futures::{pin_mut, StreamExt, TryStreamExt};
///
/// let bytes = b"deadbeefcafe00112233";
///
/// // Build Frame and segment it
/// let frame = Frame { frame_id: 1, data: bytes.as_ref().into() };
/// let segments = frame.segment(2).unwrap();
/// assert_eq!(bytes.len() / 2, segments.len());
///
/// // Create FrameReassembler and feed the segments to it
/// let (fragmented, reassembled) = FrameReassembler::new(Duration::from_secs(10));
///
/// for segment in segments {
///     fragmented.push_segment(segment).unwrap();
/// }
///
/// drop(fragmented);
/// pin_mut!(reassembled);
///
/// assert!(matches!(reassembled.try_next().await, Ok(Some(frame))));
/// # });
/// ````
#[derive(Debug)]
pub struct FrameReassembler {
    sequences: DashMap<FrameId, FrameBuilder>,
    highest_buffered_frame: AtomicU32,
    next_emitted_frame: AtomicU32,
    last_emission: AtomicU64,
    reassembled: futures::channel::mpsc::UnboundedSender<crate::errors::Result<Frame>>,
    max_age: Duration,
}

impl FrameReassembler {
    /// Creates a new frame reassembler and a corresponding stream
    /// for reassembled [Frames](Frame).
    /// An optional `max_age` of segments can be specified,
    /// which allows the [`evict`](FrameReassembler::evict) method to remove stale incomplete segments.
    pub fn new(max_age: Duration) -> (Self, impl Stream<Item = crate::errors::Result<Frame>>) {
        let (reassembled, reassembled_recv) = futures::channel::mpsc::unbounded();
        (
            Self {
                sequences: DashMap::new(),
                highest_buffered_frame: AtomicU32::new(0),
                next_emitted_frame: AtomicU32::new(1),
                last_emission: AtomicU64::new(u64::MAX),
                reassembled,
                max_age,
            },
            reassembled_recv,
        )
    }

    /// Emits the frame if it is the next in sequence and complete.
    /// If it is not next in the sequence or incomplete, it is discarded forever.
    fn emit_if_complete_discard_otherwise(&self, builder: FrameBuilder) -> crate::errors::Result<()> {
        if self.next_emitted_frame.fetch_add(1, Ordering::SeqCst) == builder.frame_id && builder.is_complete() {
            self.reassembled
                .unbounded_send(builder.reassemble())
                .map_err(|_| NetworkTypeError::ReassemblerClosed)?;
        } else {
            self.reassembled
                .unbounded_send(Err(NetworkTypeError::FrameDiscarded(builder.frame_id)))
                .map_err(|_| NetworkTypeError::ReassemblerClosed)?;
        }
        self.last_emission
            .store(current_time().as_unix_timestamp().as_millis() as u64, Ordering::Relaxed);
        Ok(())
    }

    /// Pushes a new [Segment] for reassembly.
    /// This function also pushes out the reassembled frame if this segment completed it.
    /// Pushing a segment belonging to a frame ID that has been already
    /// previously completed or [evicted](FrameReassembler::evict) will fail.
    pub fn push_segment(&self, segment: Segment) -> crate::errors::Result<()> {
        if self.reassembled.is_closed() {
            return Err(NetworkTypeError::ReassemblerClosed);
        }

        // Check if this frame has not been emitted yet.
        let frame_id = segment.frame_id;
        if frame_id < self.next_emitted_frame.load(Ordering::SeqCst) {
            return Err(NetworkTypeError::OldSegment(frame_id));
        }

        let ts = current_time();
        let mut cascade = false;

        match self.sequences.entry(frame_id) {
            Entry::Occupied(e) => {
                // No more segments missing in this frame, check if it is the next on to emit
                if e.get().put(segment, ts)? == 0
                    && self
                        .next_emitted_frame
                        .compare_exchange(frame_id, frame_id + 1, Ordering::SeqCst, Ordering::Relaxed)
                        .is_ok()
                {
                    // Emit this complete frame
                    self.reassembled
                        .unbounded_send(e.remove().reassemble())
                        .map_err(|_| NetworkTypeError::ReassemblerClosed)?;
                    self.last_emission
                        .store(current_time().as_unix_timestamp().as_millis() as u64, Ordering::Relaxed);
                    cascade = true; // Try to emit next frames in sequence
                }
            }
            Entry::Vacant(v) => {
                let builder = FrameBuilder::new(segment, ts);
                // If this frame is already complete, check if it is the next one to emit
                if builder.is_complete()
                    && self
                        .next_emitted_frame
                        .compare_exchange(frame_id, frame_id + 1, Ordering::SeqCst, Ordering::Relaxed)
                        .is_ok()
                {
                    // Emit this frame if already complete
                    self.reassembled
                        .unbounded_send(builder.reassemble())
                        .map_err(|_| NetworkTypeError::ReassemblerClosed)?;
                    self.last_emission
                        .store(current_time().as_unix_timestamp().as_millis() as u64, Ordering::Relaxed);
                    cascade = true; // Try to emit next frames in sequence
                } else {
                    // If not complete or not the next one to be emitted, just start building it
                    v.insert(builder);
                    self.highest_buffered_frame.fetch_max(frame_id, Ordering::Relaxed);
                }
            }
        }

        // As long as there are more in-sequence frames completed, emit them
        if cascade {
            while let Some((_, builder)) = self
                .sequences
                .remove_if(&self.next_emitted_frame.load(Ordering::SeqCst), |_, b| b.is_complete())
            {
                // If the frame is complete, push it out as reassembled
                self.emit_if_complete_discard_otherwise(builder)?;
            }
        }

        Ok(())
    }

    /// Returns [information](FrameInfo) about the incomplete frames.
    /// The ordered frame IDs are the keys on the returned map.
    pub fn incomplete_frames(&self) -> BinaryHeap<FrameInfo> {
        (self.next_emitted_frame.load(Ordering::SeqCst)..=self.highest_buffered_frame.load(Ordering::SeqCst))
            .filter_map(|frame_id| match self.sequences.get(&frame_id) {
                Some(e) => (!e.is_complete()).then(|| e.info()),
                None => Some({
                    let mut missing_segments = BitArray::ZERO;
                    missing_segments.set(0, true);
                    FrameInfo {
                        frame_id,
                        missing_segments,
                        total_segments: 1,
                        last_update: SystemTime::UNIX_EPOCH,
                    }
                }),
            })
            .collect()
    }

    /// According to the [max_age](FrameReassembler::new) set during construction, evicts
    /// leading incomplete frames that are expired at the time this method was called.
    /// Returns that total number of frames that were evicted.
    pub fn evict(&self) -> crate::errors::Result<usize> {
        if self.reassembled.is_closed() {
            return Err(NetworkTypeError::ReassemblerClosed);
        }

        if self.sequences.is_empty() {
            return Ok(0);
        }

        let cutoff = current_time().sub(self.max_age).as_unix_timestamp().as_millis() as u64;
        let mut count = 0;
        loop {
            let next = self.next_emitted_frame.load(Ordering::SeqCst);
            if let Some((_, builder)) = self
                .sequences
                .remove_if(&next, |_, b| b.is_complete() || b.is_expired(cutoff))
            {
                // If the frame is complete, push it out as reassembled or discard it as expired
                self.emit_if_complete_discard_otherwise(builder)?;
                count += 1;
            } else if !self.sequences.contains_key(&next) && self.last_emission.load(Ordering::SeqCst) < cutoff {
                // Do not stall the sequencer too long if we haven't seen this frame at all
                self.next_emitted_frame.fetch_add(1, Ordering::Relaxed);
                self.last_emission
                    .store(current_time().as_unix_timestamp().as_millis() as u64, Ordering::Relaxed);
                count += 1;
            } else {
                // Break on first incomplete and non-expired frame
                break;
            }
        }

        Ok(count)
    }

    /// Closes the reassembler.
    /// Any subsequent calls to [`FrameReassembler::push_segment`] will fail.
    pub fn close(&self) {
        self.reassembled.close_channel();
    }
}

impl Drop for FrameReassembler {
    fn drop(&mut self) {
        let _ = self.evict();
        self.reassembled.close_channel();
    }
}

impl Extend<Segment> for FrameReassembler {
    fn extend<T: IntoIterator<Item = Segment>>(&mut self, iter: T) {
        iter.into_iter()
            .try_for_each(|s| self.push_segment(s))
            .expect("failed to extend")
    }
}

impl Sink<Segment> for FrameReassembler {
    type Error = NetworkTypeError;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Segment) -> Result<(), Self::Error> {
        self.push_segment(item)
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(self.evict().map(|_| ()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.reassembled.close_channel();
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::session::protocol::SegmentRequest;
    use async_stream::stream;
    use bitvec::array::BitArray;
    use futures::{pin_mut, Stream, StreamExt, TryStreamExt};
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use rand::prelude::{Distribution, SliceRandom};
    use rand::{seq::IteratorRandom, thread_rng, Rng, SeedableRng};
    use rand_distr::Normal;
    use rayon::prelude::*;
    use std::collections::{HashSet, VecDeque};
    use std::convert::identity;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    const MTU: usize = 448;
    const FRAME_COUNT: u32 = 65_535;
    const FRAME_SIZE: usize = 4096;
    const MIXING_FACTOR: f64 = 4.0;

    lazy_static! {
        // bd8e89c13f96c29377528865424efa7380a8c8e5cdd486b0fc508c9130ab39ef
        static ref RAND_SEED: [u8; 32] = hopr_crypto_random::random_bytes();
        static ref FRAMES: Vec<Frame> = (0..FRAME_COUNT)
            .into_par_iter()
            .map(|frame_id| Frame {
                frame_id: frame_id + 1,
                data: hopr_crypto_random::random_bytes::<FRAME_SIZE>().into(),
            })
            .collect::<Vec<_>>();
        static ref SEGMENTS: Vec<Segment> = {
            let vec = FRAMES.par_iter().flat_map(|f| f.segment(MTU).unwrap()).collect::<VecDeque<_>>();
            let mut rng = rand::rngs::StdRng::from_seed(RAND_SEED.clone());
            linear_half_normal_shuffle(&mut rng, vec, MIXING_FACTOR)
        };
    }

    /// Sample an index between `0` and `len - 1` using the given distribution and RNG.
    pub fn sample_index<T: Distribution<f64>, R: Rng>(dist: &mut T, rng: &mut R, len: usize) -> usize {
        let f: f64 = dist.sample(rng);
        (f.max(0.0).round() as usize).min(len - 1)
    }

    /// Shuffles the given `vec` by taking a next element with index `|N(0,factor^2)`|, where
    /// `N` denotes normal distribution.
    /// When used on frame segments vector, it will shuffle the segments in a controlled manner;
    /// such that an entire frame can unlikely swap position with another, if `factor` ~ frame length.
    fn linear_half_normal_shuffle<T, R: Rng>(rng: &mut R, mut vec: VecDeque<T>, factor: f64) -> Vec<T> {
        if factor == 0.0 || vec.is_empty() {
            return vec.into(); // no mixing
        }

        let mut dist = Normal::new(0.0, factor).unwrap();
        let mut ret = Vec::new();
        while !vec.is_empty() {
            ret.push(vec.remove(sample_index(&mut dist, rng, vec.len())).unwrap());
        }
        ret
    }

    #[ctor::ctor]
    fn init() {
        lazy_static::initialize(&FRAMES);
        lazy_static::initialize(&SEGMENTS);
    }

    #[test]
    fn test_frame_info() {
        let frames = (1..20)
            .map(|i| {
                let mut missing_segments = BitArray::ZERO;
                (0..7_usize)
                    .choose_multiple(&mut thread_rng(), 4)
                    .into_iter()
                    .for_each(|i| missing_segments.set(i, true));
                FrameInfo {
                    frame_id: i,
                    missing_segments,
                    total_segments: 8,
                    last_update: SystemTime::UNIX_EPOCH,
                }
            })
            .collect::<Vec<_>>();

        let mut req = SegmentRequest::<466>::from_iter(frames.clone())
            .into_iter()
            .collect::<Vec<_>>();
        req.sort();

        assert_eq!(frames.len() * 4, req.len());
        assert_eq!(
            req,
            frames
                .into_iter()
                .flat_map(|f| f.into_missing_segments())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_segmentation() -> anyhow::Result<()> {
        let data = hex!("deadbeefcafebabe");
        let frame = Frame {
            frame_id: 1,
            data: data.as_ref().into(),
        };

        let segments = frame.segment(3)?;
        assert_eq!(3, segments.len());

        assert_eq!(hex!("deadbe"), segments[0].data.as_ref());
        assert_eq!(0, segments[0].seq_idx);
        assert_eq!(3, segments[0].seq_len);
        assert_eq!(frame.frame_id, segments[0].frame_id);

        assert_eq!(hex!("efcafe"), segments[1].data.as_ref());
        assert_eq!(1, segments[1].seq_idx);
        assert_eq!(3, segments[1].seq_len);
        assert_eq!(frame.frame_id, segments[1].frame_id);

        assert_eq!(hex!("babe"), segments[2].data.as_ref());
        assert_eq!(2, segments[2].seq_idx);
        assert_eq!(3, segments[2].seq_len);
        assert_eq!(frame.frame_id, segments[2].frame_id);

        Ok(())
    }

    #[test]
    fn test_segment_serialization() {
        let data = hopr_crypto_random::random_bytes::<128>();

        let segment = Segment {
            frame_id: 1234,
            seq_len: 123,
            seq_idx: 12,
            data: data.into(),
        };

        let boxed: Vec<u8> = segment.clone().into();
        let recovered: Segment = (&boxed[..]).try_into().unwrap();

        assert_eq!(segment, recovered);
    }

    #[async_std::test]
    async fn test_ordered() -> anyhow::Result<()> {
        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_secs(30));

        FRAMES
            .iter()
            .flat_map(|f| f.segment(MTU).unwrap())
            .try_for_each(|s| fragmented.push_segment(s))?;

        drop(fragmented);
        let reassembled_frames = reassembled.try_collect::<Vec<_>>().await?;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, FRAMES[i]));

        Ok(())
    }

    #[async_std::test]
    async fn test_one_segment_frame() -> anyhow::Result<()> {
        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_secs(10));

        let data = hex!("cafe");

        let segment = Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 1,
            data: hex!("cafe").clone().into(),
        };

        fragmented.push_segment(segment)?;
        drop(fragmented);
        let mut reassembled_frames = reassembled.try_collect::<Vec<_>>().await?;

        assert_eq!(1, reassembled_frames.len());
        let frame = reassembled_frames.pop().ok_or(NetworkTypeError::InvalidSegment)?;

        assert_eq!(1, frame.frame_id);
        assert_eq!(&data, frame.data.as_ref());

        Ok(())
    }

    #[test]
    fn test_should_not_push_frame_id_0() -> anyhow::Result<()> {
        let frame = Frame {
            frame_id: 1,
            data: hex!("deadbeefcafe").into(),
        };

        let mut segments = frame.segment(2)?;
        segments[0].frame_id = 0;

        let (fragmented, _reassembled) = FrameReassembler::new(Duration::from_secs(30));
        fragmented
            .push_segment(segments[0].clone())
            .expect_err("must not push frame id 0");

        Ok(())
    }

    #[test]
    fn test_pushing_segment_of_completed_frame_should_fail() -> anyhow::Result<()> {
        let (fragmented, _reassembled) = FrameReassembler::new(Duration::from_secs(30));

        let segments = FRAMES[0].segment(MTU)?;
        let segment_1 = segments[0].clone();

        segments.into_iter().try_for_each(|s| fragmented.push_segment(s))?;

        fragmented
            .push_segment(segment_1)
            .expect_err("must fail pushing segment of a completed frame");

        Ok(())
    }

    #[async_std::test]
    async fn test_pushing_segment_of_evicted_frame_should_fail() -> anyhow::Result<()> {
        let (fragmented, _reassembled) = FrameReassembler::new(Duration::from_millis(5).into());

        let mut segments = FRAMES[0].segment(MTU)?;
        let segment_1 = segments.pop().unwrap(); // Remove the first segment

        segments.into_iter().try_for_each(|s| fragmented.push_segment(s))?;

        async_std::task::sleep(Duration::from_millis(10)).await;
        assert_eq!(1, fragmented.evict()?);

        fragmented
            .push_segment(segment_1)
            .expect_err("must fail pushing segment of an evicted frame");

        Ok(())
    }

    #[async_std::test]
    async fn test_reassemble_single_frame() -> anyhow::Result<()> {
        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_secs(30));

        let mut rng = thread_rng();

        let frame = FRAMES[0].clone();
        let mut segments = frame.segment(MTU)?;
        segments.shuffle(&mut rng);

        segments.into_iter().try_for_each(|s| fragmented.push_segment(s))?;

        drop(fragmented);
        let reassembled_frames = reassembled.try_collect::<Vec<_>>().await?;

        assert_eq!(1, reassembled_frames.len());
        assert_eq!(frame, reassembled_frames[0]);

        Ok(())
    }

    #[async_std::test]
    async fn test_shuffled_randomized() -> anyhow::Result<()> {
        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_secs(30));

        SEGMENTS.iter().cloned().try_for_each(|b| fragmented.push_segment(b))?;

        assert_eq!(0, fragmented.evict().unwrap());
        drop(fragmented);

        let reassembled_frames = reassembled.try_collect::<Vec<_>>().await?;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, FRAMES[i]));

        Ok(())
    }

    #[async_std::test]
    async fn test_shuffled_randomized_parallel() -> anyhow::Result<()> {
        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_secs(30));

        SEGMENTS
            .par_iter()
            .cloned()
            .try_for_each(|b| fragmented.push_segment(b))?;

        assert_eq!(0, fragmented.evict()?);
        drop(fragmented);

        let reassembled_frames = reassembled.try_collect::<Vec<_>>().await?;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, FRAMES[i]));

        Ok(())
    }

    #[async_std::test]
    async fn test_should_evict_expired_incomplete_frames() -> anyhow::Result<()> {
        let frames = vec![
            Frame {
                frame_id: 1,
                data: hex!("deadbeefcafebabe").into(),
            },
            Frame {
                frame_id: 2,
                data: hex!("feedbeefbaadcafe").into(),
            },
            Frame {
                frame_id: 3,
                data: hex!("00112233abcd").into(),
            },
        ];

        let mut segments = frames
            .iter()
            .flat_map(|f| f.segment(3).unwrap())
            .collect::<VecDeque<_>>();
        segments.retain(|s| s.frame_id != 2 || s.seq_idx != 2); // Remove 2nd segment of Frame 2

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(10));

        segments.into_iter().try_for_each(|b| fragmented.push_segment(b))?;

        let frames_cpy = frames.clone();
        let jh = async_std::task::spawn(async move {
            pin_mut!(reassembled);

            // Frame #1 should yield immediately
            assert_eq!(Some(frames_cpy[0].clone()), reassembled.try_next().await?);

            // Frame #2 will yield an error once `evict` has been called
            assert!(matches!(
                reassembled.try_next().await,
                Err(NetworkTypeError::FrameDiscarded(2))
            ));

            // Frame #3 will yield normally
            assert_eq!(Some(frames_cpy[2].clone()), reassembled.try_next().await?);

            Ok(())
        });

        async_std::task::sleep(Duration::from_millis(20)).await;

        assert_eq!(2, fragmented.evict()?); // One expired, one complete

        jh.await
    }

    #[async_std::test]
    async fn test_should_evict_frame_that_never_arrived() -> anyhow::Result<()> {
        let frames = vec![
            Frame {
                frame_id: 1,
                data: hex!("deadbeefcafebabe").into(),
            },
            Frame {
                frame_id: 3,
                data: hex!("00112233abcd").into(),
            },
        ];

        let segments = frames
            .iter()
            .flat_map(|f| f.segment(3).unwrap())
            .collect::<VecDeque<_>>();

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(10));

        segments.into_iter().try_for_each(|b| fragmented.push_segment(b))?;

        let flushed = Arc::new(AtomicBool::new(false));

        let flushed_cpy = flushed.clone();
        let frames_cpy = frames.clone();
        let jh = async_std::task::spawn(async move {
            pin_mut!(reassembled);

            // The first frame should yield immediately
            assert_eq!(Some(frames_cpy[0].clone()), reassembled.try_next().await?);

            assert!(!flushed_cpy.load(Ordering::SeqCst));

            // The next frame is the third one
            assert_eq!(Some(frames_cpy[1].clone()), reassembled.try_next().await?);

            // and it must've happened only after pruning
            assert!(flushed_cpy.load(Ordering::SeqCst));

            Ok(())
        });

        async_std::task::sleep(Duration::from_millis(20)).await;

        // Prune the expired entry, which is Frame 2 (that is missing a segment)
        flushed.store(true, Ordering::SeqCst);
        assert_eq!(2, fragmented.evict()?); // One expired, one complete

        jh.await
    }

    #[async_std::test]
    async fn test_randomized_delayed_parallel() -> anyhow::Result<()> {
        let frames = FRAMES.iter().take(100).collect::<Vec<_>>();

        let segments = frames
            .iter()
            .flat_map(|frame| frame.segment(MTU).unwrap())
            .collect::<Vec<_>>();

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_secs(30));

        futures::stream::iter(segments)
            .map(|segment| {
                let delay = Duration::from_millis(thread_rng().gen_range(0..10u64));
                async_std::task::spawn(async move {
                    async_std::task::sleep(delay).await;
                    Ok(segment)
                })
            })
            .buffer_unordered(4)
            .forward(fragmented)
            .await
            .unwrap();

        let reassembled_frames = reassembled.try_collect::<Vec<_>>().await?;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(&frame, frames[i]));

        Ok(())
    }

    /// Creates `num_frames` out of which `num_corrupted` will have missing segments.
    fn corrupt_frames(
        num_frames: u32,
        corrupted_ratio: f32,
    ) -> (Vec<Segment>, Vec<&'static Frame>, HashSet<SegmentId>) {
        assert!((0.0..=1.0).contains(&corrupted_ratio));

        let mut rng = rand::rngs::StdRng::from_seed(RAND_SEED.clone());

        let (excluded_frame_ids, excluded_segments): (HashSet<FrameId>, HashSet<SegmentId>) = (1..num_frames + 1)
            .choose_multiple(&mut rng, ((num_frames as f32) * corrupted_ratio) as usize)
            .into_iter() // Must be sequentially generated due RNG determinism
            .map(|frame_id| {
                (
                    frame_id,
                    SegmentId(
                        frame_id,
                        rng.gen_range(0..SEGMENTS.iter().find(|s| s.frame_id == frame_id).unwrap().seq_len),
                    ),
                )
            })
            .unzip();

        let segments = SEGMENTS
            .par_iter()
            .filter(|s| s.frame_id < num_frames && !excluded_segments.contains(&SegmentId(s.frame_id, s.seq_idx)))
            .cloned()
            .collect::<Vec<_>>();

        let expected_frames = FRAMES
            .par_iter()
            .filter(|f| f.frame_id < num_frames && !excluded_frame_ids.contains(&f.frame_id))
            .collect::<Vec<_>>();

        (segments, expected_frames, excluded_segments)
    }

    #[async_std::test]
    async fn test_random_corrupted_frames() -> anyhow::Result<()> {
        // Corrupt 30% of the frames, by removing a random segment from them
        let (segments, expected_frames, excluded) = corrupt_frames(FRAME_COUNT / 4, 0.3);

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(25).into());

        segments.into_iter().try_for_each(|s| fragmented.push_segment(s))?;

        let computed_missing = fragmented
            .incomplete_frames()
            .into_par_iter()
            .flat_map_iter(|e| e.into_missing_segments())
            .collect::<HashSet<_>>();

        assert!(computed_missing.par_iter().all(|s| excluded.contains(&s)));
        assert!(
            excluded.par_iter().all(|s| computed_missing.contains(&s)),
            "seed {}",
            hex::encode(RAND_SEED.clone())
        );

        async_std::task::sleep(Duration::from_millis(25)).await;
        drop(fragmented);

        let (reassembled_frames, discarded_frames) = reassembled
            .map(|f| match f {
                Ok(f) => (Some(f), None),
                Err(e) => (None, Some(e)),
            })
            .unzip::<_, _, Vec<_>, Vec<_>>()
            .await;

        let reassembled_frames = reassembled_frames
            .into_par_iter()
            .filter_map(identity)
            .collect::<Vec<_>>();

        (reassembled_frames, expected_frames)
            .into_par_iter()
            .all(|(a, b)| a.eq(b));

        let discarded_frames = discarded_frames
            .into_par_iter()
            .filter_map(|s| match s {
                Some(NetworkTypeError::FrameDiscarded(f)) => Some(f),
                _ => None,
            })
            .collect::<Vec<_>>();

        let expected_discarded_frames = excluded.into_par_iter().map(|s| s.0).collect::<Vec<_>>();

        (discarded_frames, expected_discarded_frames)
            .into_par_iter()
            .all(|(a, b)| a == b);

        Ok(())
    }

    #[async_std::test]
    async fn test_corrupted_frames_should_yield_no_frames() -> anyhow::Result<()> {
        // Corrupt each frame
        let (segments, expected_frames, _) = corrupt_frames(1000, 1.0);
        assert!(expected_frames.is_empty());

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(100).into());

        segments.into_par_iter().try_for_each(|s| fragmented.push_segment(s))?;
        drop(fragmented);

        let reassembled_frames = reassembled.try_collect::<Vec<_>>().await?;

        assert!(reassembled_frames.is_empty());

        Ok(())
    }

    fn create_unreliable_segment_stream(
        num_frames: usize,
        max_latency: Duration,
        mixing_factor: f64,
        corruption_ratio: f64,
    ) -> (impl Stream<Item = Segment>, Vec<&'static Frame>) {
        let mut segments = FRAMES
            .par_iter()
            .take(num_frames)
            .flat_map(|f| f.segment(MTU).unwrap())
            .collect::<VecDeque<_>>();

        let (corrupted_frames, corrupted_segments): (HashSet<FrameId>, HashSet<SegmentId>) = segments
            .iter()
            .choose_multiple(
                &mut thread_rng(),
                (segments.len() as f64 * corruption_ratio).round() as usize,
            )
            .into_par_iter()
            .map(|s| (s.frame_id, SegmentId(s.frame_id, s.seq_idx)))
            .unzip();

        (
            stream! {
                let mut rng = thread_rng();
                let mut distr = Normal::new(0.0, mixing_factor).unwrap();
                while !segments.is_empty() {
                    let segment = segments.remove(sample_index(&mut distr, &mut rng, segments.len())).unwrap();

                    if !corrupted_segments.contains(&SegmentId(segment.frame_id, segment.seq_idx)) {
                        async_std::task::sleep(max_latency.mul_f64(rng.gen())).await;
                        yield segment;
                    }
                }
            },
            FRAMES
                .par_iter()
                .filter(|f| !corrupted_frames.contains(&f.frame_id))
                .collect(),
        )
    }

    #[async_std::test]
    async fn test_unreliable_network_with_parallel_evictions() -> anyhow::Result<()> {
        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(25).into());
        let fragmented = Arc::new(fragmented);

        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let frag_clone = fragmented.clone();
        let eviction_jh = async_std::task::spawn(async move {
            while !done_clone.load(Ordering::SeqCst) {
                async_std::task::sleep(Duration::from_millis(25)).await;
                frag_clone.evict().unwrap();
            }
        });

        // Corrupt 20% of the frames
        let (stream, expected_frames) =
            create_unreliable_segment_stream(200, Duration::from_millis(2), MIXING_FACTOR, 0.2);
        stream
            .map(Ok)
            .try_for_each(|s| futures::future::ready(fragmented.push_segment(s)))
            .await?;

        done.store(true, Ordering::SeqCst);
        eviction_jh.await;
        drop(fragmented);

        let reassembled_frames = reassembled
            .filter(|f| futures::future::ready(f.is_ok())) // Skip the discarded frames
            .try_collect::<Vec<_>>()
            .await?;
        reassembled_frames
            .into_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(&frame, expected_frames[i]));

        Ok(())
    }
}
