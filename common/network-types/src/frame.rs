use crate::errors::NetworkTypeError;
use crate::errors::NetworkTypeError::ReassemblerClosed;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::{Sink, Stream};
use std::pin::Pin;
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::OnceLock;
use std::task::{Context, Poll};

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
    pub fn segment(&self, mtu: u16) -> Vec<Segment> {
        segment(self.data.as_ref(), mtu, self.frame_id)
    }
}

/// Represents a frame segment.
/// Besides the data, a segment carries information about the total number of
/// segments in the original frame, its index within the frame and
/// ID of that frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Segment<'a> {
    frame_id: u16,
    seq_len: u16,
    seq_idx: u16,
    data: &'a [u8],
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

impl<'a> Segment<'a> {
    /// Identifies the entire segment sequence using the frame id and the length
    /// of the sequence.
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
}

impl<'a> FrameBuilder<'a> {
    /// Creates a new builder with the given initial segment.
    fn new(initial: Segment<'a>) -> Self {
        Self {
            frame_id: initial.frame_id,
            segments: (0..initial.seq_len)
                .map(|i| {
                    if i == initial.seq_idx {
                        OnceLock::from(initial.data)
                    } else {
                        OnceLock::new()
                    }
                })
                .collect(),
            missing: AtomicU16::new(initial.seq_len - 1),
        }
    }

    /// Adds a new segment to the builder.
    fn put(&self, frame: Segment<'a>) -> u16 {
        if !self.is_complete() {
            if self.segments[frame.seq_idx as usize].set(frame.data).is_ok() {
                self.missing.fetch_sub(1, Ordering::SeqCst);
            }
            self.missing.load(Ordering::SeqCst)
        } else {
            0
        }
    }

    /// Checks if the builder contains all segments of the frame.
    fn is_complete(&self) -> bool {
        self.missing.load(Ordering::SeqCst) == 0
    }

    /// Reassembles the [Frame]. Panics if not [complete](FrameBuilder::is_complete).
    fn reassemble(self) -> Frame {
        assert!(self.is_complete(), "missing frames");
        Frame {
            frame_id: self.frame_id,
            data: self
                .segments
                .into_iter()
                .map(|lock| lock.into_inner().unwrap())
                .collect::<Vec<&[u8]>>()
                .concat()
                .into_boxed_slice(),
        }
    }
}

/// Represents a frame reassembler.
/// The [FrameReassembler] behaves as a sink [Sink] for [Segment].
/// Upon creation, also [Stream] for reassembled [Frames](Frame) is created.
/// The corresponding stream is closed either when [FrameReassembler::close] or
/// [futures::SinkExt::close] is called.
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
/// let (fragmented, reassembled) = FrameReassembler::new();
/// pin_mut!(reassembled);
///
/// for segment in segments {
///     fragmented.push_segment(segment).unwrap();
/// }
/// fragmented.close();
///
/// assert_eq!(Some(frame), reassembled.next().await);
/// # });
/// ````
#[derive(Debug)]
pub struct FrameReassembler<'a> {
    sequences: DashMap<u32, FrameBuilder<'a>>,
    earliest_frame: AtomicU32,
    reassembled: async_channel::Sender<Frame>,
}

impl<'a> FrameReassembler<'a> {
    /// Creates a new frame reassembler and a corresponding stream
    /// for reassembled [Frames](Frame).
    pub fn new() -> (Self, impl Stream<Item = Frame>) {
        let (reassembled, reassembled_recv) = async_channel::unbounded();
        (
            Self {
                sequences: DashMap::new(),
                earliest_frame: AtomicU32::new(u32::MAX),
                reassembled,
            },
            reassembled_recv,
        )
    }

    /// Pushes a new [Segment] for reassembly.
    pub fn push_segment(&self, segment: Segment<'a>) -> crate::errors::Result<()> {
        if self.reassembled.is_closed() {
            return Err(ReassemblerClosed);
        }

        let seq_id = segment.seq_id();
        match self.sequences.entry(seq_id) {
            Entry::Occupied(e) => {
                e.get().put(segment);
            }
            Entry::Vacant(v) => {
                v.insert(FrameBuilder::new(segment));
                self.earliest_frame.fetch_min(seq_id, Ordering::SeqCst);
            }
        }

        if let Some((seq_id, builder)) = self
            .sequences
            .remove_if(&self.earliest_frame.load(Ordering::SeqCst), |_, table| {
                table.is_complete()
            })
        {
            let _ = self.reassembled.try_send(builder.reassemble());
            self.earliest_frame.fetch_min(seq_id + (1u32 << 16), Ordering::SeqCst);
        }

        Ok(())
    }

    /// Closes the reassembler, so it cannot be used to push new [Segments](Segment) anymore.
    pub fn close(self) {
        self.reassembled.close();
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
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.reassembled.close();
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{Frame, FrameReassembler, Segment};
    use futures::StreamExt;
    use lazy_static::lazy_static;
    use rand::prelude::SliceRandom;
    use rand::thread_rng;
    use rayon::prelude::*;

    const MTU: u16 = 10;
    const PACKET_COUNT: u16 = 65_535;
    const PACKET_SIZE: usize = 500;

    lazy_static! {
        static ref PACKETS: Vec<Frame> = (0..PACKET_COUNT)
            .into_par_iter()
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<PACKET_SIZE>().into(),
            })
            .collect::<Vec<_>>();
        static ref SEGMENTS: Vec<Segment<'static>> = {
            let mut segments = PACKETS.par_iter().flat_map(|f| f.segment(MTU)).collect::<Vec<_>>();
            let mut rng = thread_rng();
            segments.shuffle(&mut rng);
            segments
        };
    }

    #[ctor::ctor]
    fn init() {
        lazy_static::initialize(&PACKETS);
        lazy_static::initialize(&SEGMENTS);
    }

    #[async_std::test]
    async fn test_shuffled_randomized() {
        let (fragmented, reassembled) = FrameReassembler::new();

        SEGMENTS.iter().cloned().for_each(|b| {
            let _ = fragmented.push_segment(b);
        });
        fragmented.close();

        let reassembled_packets = reassembled.collect::<Vec<_>>().await;

        reassembled_packets
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, PACKETS[i]));
    }

    #[async_std::test]
    async fn test_shuffled_randomized_parallel() {
        let (fragmented, reassembled) = FrameReassembler::new();

        SEGMENTS.par_iter().cloned().for_each(|b| {
            let _ = fragmented.push_segment(b);
        });
        fragmented.close();

        let reassembled_packets = reassembled.collect::<Vec<_>>().await;

        reassembled_packets
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, PACKETS[i]));
    }
}
