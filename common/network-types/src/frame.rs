use crate::errors::NetworkTypeError;
use crate::errors::NetworkTypeError::{IncompleteFrame, InvalidFrameId, InvalidSegment, ReassemblerClosed};
use crossbeam_skiplist::SkipSet;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::{Sink, Stream};
use hopr_platform::time::native::current_time;
use hopr_primitive_types::prelude::AsUnixTimestamp;
use serde::{Deserialize, Serialize};
use std::ops::Sub;
use std::pin::Pin;
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::task::{Context, Poll};
use std::time::Duration;

fn segment(data: &[u8], mtu: u16, frame_id: u16) -> Vec<Segment> {
    let chunks = data.chunks(mtu as usize);
    assert!(chunks.len() < u16::MAX as usize, "data too long");

    let seq_len = chunks.len() as u16;

    chunks
        .enumerate()
        .map(|(idx, data)| Segment {
            frame_id,
            seq_len,
            seq_idx: idx as u16,
            data,
        })
        .collect()
}

/// Data frame of arbitrary length.
/// The frame can be segmented into [segments](Segment) and reassembled back
/// via [FrameReassembler].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Identifier of this frame.
    pub frame_id: u16,
    /// Frame data.
    pub data: Box<[u8]>,
}

impl Frame {
    /// Segments this frame into a list of [segments](Segment) each of maximum sizes `mtu`.
    #[inline]
    pub fn segment(&self, mtu: u16) -> Vec<Segment> {
        segment(self.data.as_ref(), mtu, self.frame_id)
    }
}

/// Represents a frame segment.
/// Besides the data, a segment carries information about the total number of
/// segments in the original frame, its index within the frame and
/// ID of that frame.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Segment<'a> {
    /// ID of the [Frame] this segment belongs to.
    pub frame_id: u16,
    /// Index of this segment within the segment sequence.
    pub seq_idx: u16,
    /// Total number of segments within this segment sequence.
    pub seq_len: u16,
    /// Data in this segment.
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
}

impl<'a> PartialOrd<Segment<'a>> for Segment<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.frame_id.cmp(&other.frame_id) {
            std::cmp::Ordering::Equal => self.seq_idx.cmp(&other.seq_idx),
            cmp => cmp,
        }
    }
}

impl From<Segment<'_>> for Box<[u8]> {
    fn from(value: Segment<'_>) -> Self {
        let mut ret = Vec::with_capacity(6 + value.data.len());
        ret.extend_from_slice(value.frame_id.to_be_bytes().as_ref());
        ret.extend_from_slice(value.seq_idx.to_be_bytes().as_ref());
        ret.extend_from_slice(value.seq_len.to_be_bytes().as_ref());
        ret.extend_from_slice(value.data);
        ret.into_boxed_slice()
    }
}

impl<'a> TryFrom<&'a [u8]> for Segment<'a> {
    type Error = NetworkTypeError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let (header, data) = value.split_at(6);
        let segment = Segment {
            frame_id: u16::from_be_bytes(header[0..2].try_into().map_err(|_| InvalidSegment)?),
            seq_idx: u16::from_be_bytes(header[2..4].try_into().map_err(|_| InvalidSegment)?),
            seq_len: u16::from_be_bytes(header[4..6].try_into().map_err(|_| InvalidSegment)?),
            data,
        };
        if segment.seq_idx < segment.seq_len {
            Ok(segment)
        } else {
            Err(InvalidSegment)
        }
    }
}

impl<'a> Segment<'a> {
    /// Identifies the entire segment sequence using the frame id and the length
    /// of the sequence.
    #[inline]
    pub fn seq_id(&self) -> u32 {
        (self.frame_id as u32) << 16 | self.seq_len as u32
    }
}

/// Rebuilds the [Frame] from [Segments](Segment).
#[derive(Debug)]
struct FrameBuilder<'a> {
    frame_id: u16,
    segments: Vec<OnceLock<&'a [u8]>>,
    missing: AtomicU16,
    last_ts: AtomicU64,
}

impl<'a> FrameBuilder<'a> {
    /// Creates a new builder with the given initial segment.
    fn new(initial: Segment<'a>, ts: u64) -> Self {
        let ret = Self::empty(initial.frame_id, initial.seq_len);
        ret.put(initial, ts).unwrap();
        ret
    }

    /// Creates a new empty builder.
    fn empty(frame_id: u16, seq_len: u16) -> Self {
        Self {
            frame_id,
            segments: vec![OnceLock::new(); seq_len as usize],
            missing: AtomicU16::new(seq_len),
            last_ts: AtomicU64::new(0),
        }
    }

    /// Adds a new segment to the builder with a timestamp.
    /// Returns the number of segments missing in this builder.
    fn put(&self, segment: Segment<'a>, ts: u64) -> crate::errors::Result<u16> {
        if self.frame_id == segment.frame_id {
            if !self.is_complete() {
                if self.segments[segment.seq_idx as usize].set(segment.data).is_ok() {
                    self.missing.fetch_sub(1, Ordering::Relaxed);
                    self.last_ts.fetch_max(ts, Ordering::Relaxed);
                }
                Ok(self.missing.load(Ordering::SeqCst))
            } else {
                // Silently throw away segments of a frame that is already complete
                Ok(0)
            }
        } else {
            Err(InvalidFrameId)
        }
    }

    /// Checks if the builder contains all segments of the frame.
    fn is_complete(&self) -> bool {
        self.missing.load(Ordering::SeqCst) == 0
    }

    /// Checks if the last added segment to this frame happened before `cutoff`.
    /// In other words, the frame under construction is considered expired if the last
    /// segment was added before `cutoff`.
    fn is_expired(&self, cutoff: u64) -> bool {
        self.last_ts.load(Ordering::SeqCst) < cutoff
    }

    /// Reassembles the [Frame]. Returns [`IncompleteFrame`] if not [complete](FrameBuilder::is_complete).
    fn reassemble(self) -> crate::errors::Result<Frame> {
        if self.is_complete() {
            Ok(Frame {
                frame_id: self.frame_id,
                data: self
                    .segments
                    .into_iter()
                    .map(|lock| lock.into_inner().unwrap())
                    .collect::<Vec<&[u8]>>()
                    .concat()
                    .into_boxed_slice(),
            })
        } else {
            Err(IncompleteFrame)
        }
    }
}

/// Represents a frame reassembler.
///
/// The [`FrameReassembler`] behaves as a [`Sink`] for [`Segment`].
/// Upon creation, also [`Stream`] for reassembled [Frames](Frame) is created.
/// The corresponding stream is closed either when [`FrameReassembler::close`] or
/// [`futures::SinkExt::close`] is called.
///
/// As new segments are [pushed](FrameReassembler::push_segment) into the reassembler,
/// the frames get reassembled and once they are completed, they are automatically pushed out into
/// the outgoing frame stream.
///
/// The reassembler can also have a `max_age` of frames that are under construction.
/// The [`evict`](FrameReassembler::evict) method then can be called to remove
/// the incomplete frames over `max_age`.
///
/// Note that the reassembler is not flushed when dropped.
///
/// ````rust
/// # futures::executor::block_on(async {
/// use hopr_network_types::frame::{Frame, FrameReassembler};
/// use futures::{pin_mut, StreamExt};
///
/// let bytes = b"deadbeefcafe00112233";
///
/// // Build Frame and segment it
/// let frame = Frame { frame_id: 1, data: bytes.as_ref().into() };
/// let segments = frame.segment(2);
/// assert_eq!(bytes.len() / 2, segments.len());
///
/// // Create FrameReassembler and feed the segments to it
/// let (fragmented, reassembled) = FrameReassembler::new(None);
///
/// for segment in segments {
///     fragmented.push_segment(segment).unwrap();
/// }
///
/// drop(fragmented);
/// pin_mut!(reassembled);
///
/// assert_eq!(Some(frame), reassembled.next().await);
/// # });
/// ````
#[derive(Debug)]
pub struct FrameReassembler<'a> {
    sequences: DashMap<u32, FrameBuilder<'a>>,
    sorted_seq_ids: SkipSet<u32>,
    reassembled: futures::channel::mpsc::UnboundedSender<Frame>,
    max_age: Option<Duration>,
}

impl<'a> FrameReassembler<'a> {
    /// Creates a new frame reassembler and a corresponding stream
    /// for reassembled [Frames](Frame).
    pub fn new(max_age: Option<Duration>) -> (Self, impl Stream<Item = Frame>) {
        let (reassembled, reassembled_recv) = futures::channel::mpsc::unbounded();
        (
            Self {
                sequences: DashMap::new(),
                sorted_seq_ids: SkipSet::new(),
                reassembled,
                max_age,
            },
            reassembled_recv,
        )
    }

    /// Pushes a new [Segment] for reassembly.
    pub fn push_segment(&self, segment: Segment<'a>) -> crate::errors::Result<()> {
        if self.reassembled.is_closed() {
            return Err(ReassemblerClosed);
        }

        let ts = current_time().as_unix_timestamp().as_millis() as u64;
        let seq_id = segment.seq_id();

        match self.sequences.entry(seq_id) {
            Entry::Occupied(e) => {
                if e.get().put(segment, ts)? == 0 {
                    // No more segments missing in this frame, check if this is the earliest frame
                    if let Some(earliest_seq_id) = self.sorted_seq_ids.front() {
                        if earliest_seq_id.eq(e.key()) {
                            self.reassembled
                                .unbounded_send(e.remove().reassemble()?)
                                .map_err(|_| ReassemblerClosed)?;
                            earliest_seq_id.remove();
                        }
                    }
                }
            }
            Entry::Vacant(v) => {
                v.insert(FrameBuilder::new(segment, ts));
                self.sorted_seq_ids.insert(seq_id);
            }
        }

        Ok(())
    }

    /// If [max_age](FrameReassembler::new) was set during construction, evicts
    /// leading incomplete frames that are expired at the time this method was called.
    /// Returns that total number of frames that were evicted.
    pub fn evict(&self) -> crate::errors::Result<usize> {
        if self.reassembled.is_closed() || self.sequences.is_empty() {
            self.sorted_seq_ids.clear();
            return Ok(0);
        }

        // Start evicting the earliest incomplete expired frames.
        // This might also uncover queued complete frames that couldn't be pushed out
        // due to prior incomplete frames.

        let mut count = 0;
        if let Some(cutoff) = self
            .max_age
            .map(|max_age| current_time().sub(max_age).as_unix_timestamp().as_millis() as u64)
        {
            // Iterate from lowest seq_ids first, since they are more likely to be completed or expired
            for seq_id in self.sorted_seq_ids.iter() {
                // Remove each frame that is either completed (or expired if max age was given).
                if let Some((_, builder)) = self.sequences.remove_if(seq_id.value(), |_, builder| {
                    builder.is_complete() || builder.is_expired(cutoff)
                }) {
                    if builder.is_complete() {
                        self.reassembled
                            .unbounded_send(builder.reassemble()?)
                            .map_err(|_| ReassemblerClosed)?;
                    }

                    seq_id.remove();
                    count += 1;
                } else if !self.sequences.contains_key(seq_id.value()) {
                    // If removal failed, because the entry is no longer there, just remove its seq_id
                    seq_id.remove();
                } else {
                    // Stop on the first non expired and non completed frame.
                    break;
                }
            }

            Ok(count)
        } else {
            Ok(0)
        }
    }
}

impl Drop for FrameReassembler<'_> {
    fn drop(&mut self) {
        let _ = self.evict();
        self.reassembled.close_channel();
    }
}

impl<'a> Sink<Segment<'a>> for FrameReassembler<'a> {
    type Error = NetworkTypeError;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Segment<'a>) -> Result<(), Self::Error> {
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
mod tests {
    use crate::frame::{Frame, FrameReassembler, Segment};
    use futures::{pin_mut, StreamExt};
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use rand::prelude::SliceRandom;
    use rand::{seq::IteratorRandom, thread_rng, Rng};
    use rayon::prelude::*;
    use std::collections::HashSet;
    use std::convert::identity;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    const MTU: u16 = 448;
    const FRAME_COUNT: u16 = 65_535;
    const FRAME_SIZE: usize = 2048;

    lazy_static! {
        static ref FRAMES: Vec<Frame> = (0..FRAME_COUNT)
            .into_par_iter()
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<FRAME_SIZE>().into(),
            })
            .collect::<Vec<_>>();
        static ref SEGMENTS: Vec<Segment<'static>> = {
            let mut segments = FRAMES.par_iter().flat_map(|f| f.segment(MTU)).collect::<Vec<_>>();
            let mut rng = thread_rng();
            segments.shuffle(&mut rng);
            segments
        };
    }

    #[ctor::ctor]
    fn init() {
        lazy_static::initialize(&FRAMES);
        lazy_static::initialize(&SEGMENTS);
    }

    #[test]
    fn test_segmentation() {
        let data = hex!("deadbeefcafebabe");
        let frame = Frame {
            frame_id: 0,
            data: data.as_ref().into(),
        };

        let segments = frame.segment(3);
        assert_eq!(3, segments.len());

        assert_eq!(hex!("deadbe"), segments[0].data);
        assert_eq!(0, segments[0].seq_idx);
        assert_eq!(3, segments[0].seq_len);
        assert_eq!(frame.frame_id, segments[0].frame_id);

        assert_eq!(hex!("efcafe"), segments[1].data);
        assert_eq!(1, segments[1].seq_idx);
        assert_eq!(3, segments[1].seq_len);
        assert_eq!(frame.frame_id, segments[1].frame_id);

        assert_eq!(hex!("babe"), segments[2].data);
        assert_eq!(2, segments[2].seq_idx);
        assert_eq!(3, segments[2].seq_len);
        assert_eq!(frame.frame_id, segments[2].frame_id);
    }

    #[test]
    fn test_segment_serialization() {
        let data = hopr_crypto_random::random_bytes::<128>();

        let segment = Segment {
            frame_id: 1234,
            seq_len: 123,
            seq_idx: 12,
            data: &data,
        };

        let boxed: Box<[u8]> = segment.clone().into();
        let recovered: Segment = (&boxed[..]).try_into().unwrap();

        assert_eq!(segment, recovered);
    }

    #[async_std::test]
    async fn test_shuffled_randomized() {
        let (fragmented, reassembled) = FrameReassembler::new(None);

        SEGMENTS.iter().cloned().for_each(|b| {
            let _ = fragmented.push_segment(b);
        });

        assert_eq!(0, fragmented.evict().unwrap());
        drop(fragmented);

        let reassembled_frames = reassembled.collect::<Vec<_>>().await;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, FRAMES[i]));
    }

    #[async_std::test]
    async fn test_shuffled_randomized_parallel() {
        let (fragmented, reassembled) = FrameReassembler::new(None);

        SEGMENTS.par_iter().cloned().for_each(|b| {
            let _ = fragmented.push_segment(b);
        });

        assert_eq!(0, fragmented.evict().unwrap());
        drop(fragmented);

        let reassembled_frames = reassembled.collect::<Vec<_>>().await;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, FRAMES[i]));
    }

    #[async_std::test]
    async fn test_flush() {
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

        let mut segments = frames.iter().flat_map(|f| f.segment(3)).collect::<Vec<_>>();
        segments.remove(4); // Remove 2nd segment of Frame 3

        let mut rng = thread_rng();
        segments.shuffle(&mut rng);

        let (fragmented, reassembled) = FrameReassembler::new(Some(Duration::from_millis(10)));

        segments.into_iter().for_each(|b| {
            let _ = fragmented.push_segment(b);
        });

        let flushed = Arc::new(AtomicBool::new(false));

        let flushed_cpy = flushed.clone();
        let frames_cpy = frames.clone();
        let jh = async_std::task::spawn(async move {
            pin_mut!(reassembled);

            // The first frame should yield immediately
            assert_eq!(Some(frames_cpy[0].clone()), reassembled.next().await);

            assert!(!flushed_cpy.load(Ordering::SeqCst));

            // The next frame is the third one
            assert_eq!(Some(frames_cpy[2].clone()), reassembled.next().await);

            // and it must've happened only after pruning
            assert!(flushed_cpy.load(Ordering::SeqCst));
        });

        async_std::task::sleep(Duration::from_millis(50)).await;

        // Prune the expired entry, which is Frame 2 (that is missing a segment)
        flushed.store(true, Ordering::SeqCst);
        assert_eq!(2, fragmented.evict().unwrap()); // One expired, one complete

        jh.await;
    }

    #[async_std::test]
    async fn test_randomized_delayed_parallel() {
        let frames = FRAMES.iter().take(100).collect::<Vec<_>>();

        let mut segments = frames.iter().flat_map(|frame| frame.segment(MTU)).collect::<Vec<_>>();

        let mut rng = thread_rng();
        segments.shuffle(&mut rng);

        let (fragmented, reassembled) = FrameReassembler::new(None);

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

        let reassembled_frames = reassembled.collect::<Vec<_>>().await;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(&frame, frames[i]));
    }

    #[async_std::test]
    async fn test_randomized_delayed_parallel_with_eviction() {
        let frames = FRAMES.iter().take(100).collect::<Vec<_>>();

        let mut segments = frames.iter().flat_map(|frame| frame.segment(MTU)).collect::<Vec<_>>();

        let mut rng = thread_rng();
        segments.shuffle(&mut rng);

        let (fragmented, reassembled) = FrameReassembler::new(None);

        let fragmented = Arc::new(fragmented);

        futures::stream::iter(segments)
            .map(|segment| {
                let mut rng = thread_rng();
                let delay = Duration::from_millis(rng.gen_range(0..10u64));
                let should_evict = rng.gen_bool(0.5);
                let reassembler = fragmented.clone();

                async_std::task::spawn(async move {
                    async_std::task::sleep(delay).await;

                    if should_evict {
                        reassembler.evict().unwrap();
                    }

                    reassembler.push_segment(segment).unwrap();
                })
            })
            .for_each_concurrent(4, identity)
            .await;

        drop(fragmented);

        let reassembled_frames = reassembled.collect::<Vec<_>>().await;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(&frame, frames[i]));
    }

    /// Creates `num_frames` out of which `num_corrupted` will have missing segments.
    fn corrupt_frames(num_frames: u16, corrupted_ratio: f32) -> (Vec<Segment<'static>>, Vec<&'static Frame>) {
        assert!((0.0..=1.0).contains(&corrupted_ratio));

        #[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
        struct SegmentId(u16, u16);

        let excluded_segments = (0..num_frames)
            .choose_multiple(&mut thread_rng(), ((num_frames as f32) * corrupted_ratio) as usize)
            .into_par_iter()
            .map(|frame_id| {
                SegmentId(
                    frame_id,
                    thread_rng().gen_range(0..SEGMENTS.iter().find(|s| s.frame_id == frame_id).unwrap().seq_len),
                )
            })
            .collect::<HashSet<_>>();

        let excluded_frame_ids = excluded_segments.par_iter().map(|s| s.0).collect::<HashSet<_>>();

        let segments = SEGMENTS
            .par_iter()
            .filter(|s| s.frame_id < num_frames && !excluded_segments.contains(&SegmentId(s.frame_id, s.seq_idx)))
            .cloned()
            .collect::<Vec<_>>();

        let expected_frames = FRAMES
            .par_iter()
            .filter(|f| f.frame_id < num_frames && !excluded_frame_ids.contains(&f.frame_id))
            .collect::<Vec<_>>();

        (segments, expected_frames)
    }

    #[async_std::test]
    async fn test_random_corrupted_frames_parallel() {
        // Corrupt 30% of the frames, by removing a random segment from them
        let (segments, expected_frames) = corrupt_frames(FRAME_COUNT / 4, 0.3);

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(100).into());

        segments.into_par_iter().for_each(|s| {
            fragmented.push_segment(s).unwrap();
        });

        async_std::task::sleep(Duration::from_millis(100)).await;
        drop(fragmented);

        let reassembled_frames = reassembled.collect::<Vec<_>>().await;

        reassembled_frames
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(&frame, expected_frames[i]));
    }

    #[async_std::test]
    async fn test_corrupted_frames_should_yield_no_frames() {
        // Corrupt each frame
        let (segments, expected_frames) = corrupt_frames(1000, 1.0);
        assert!(expected_frames.is_empty());

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(100).into());

        segments.into_par_iter().for_each(|s| {
            fragmented.push_segment(s).unwrap();
        });
        drop(fragmented);

        let reassembled_frames = reassembled.collect::<Vec<_>>().await;

        assert!(reassembled_frames.is_empty());
    }

    /*#[async_std::test]
    async fn test_random_corrupted_frames_with_parallel_evictions() {
        // Corrupt 40% of the frames, by removing a random segment from them
        let (segments, expected_frames) = corrupt_frames(100, 0.4);

        let (fragmented, reassembled) = FrameReassembler::new(Duration::from_millis(40).into());
        let fragmented = Arc::new(fragmented);

        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let frag_clone = fragmented.clone();
        let eviction_jh = async_std::task::spawn(async move {
            while !done_clone.load(Ordering::SeqCst) {
                frag_clone.evict().unwrap();
                async_std::task::sleep(Duration::from_millis(500)).await;
            }
        });

        futures::stream::iter(segments)
            .map(|segment| {
                let delay = Duration::from_millis(thread_rng().gen_range(0..20u64));
                let frag_clone = fragmented.clone();
                async_std::task::spawn(async move {
                    async_std::task::sleep(delay).await;
                    frag_clone.push_segment(segment).unwrap();
                })
            })
            .for_each_concurrent(2, identity)
            .await;

        done.store(true, Ordering::SeqCst);
        eviction_jh.await;

        drop(fragmented);

        let reassembled_frames = reassembled.collect::<Vec<_>>().await;

        reassembled_frames
            .into_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(&frame, expected_frames[i]));

    }*/
}
