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

impl<'a> Segment<'a> {
    /// Identifies the entire segment sequence using the frame id and the length
    /// of the sequence.
    pub fn seq_id(&self) -> u32 {
        (self.frame_id as u32) << 16 | self.seq_len as u32
    }
}

#[derive(Debug)]
struct SegmentBag<'a> {
    segments: Vec<OnceLock<&'a [u8]>>,
    missing: AtomicU16,
}

impl<'a> SegmentBag<'a> {
    fn new(initial_frame: Segment<'a>) -> Self {
        Self {
            segments: (0..initial_frame.seq_len)
                .map(|i| {
                    if i == initial_frame.seq_idx {
                        OnceLock::from(initial_frame.data)
                    } else {
                        OnceLock::new()
                    }
                })
                .collect(),
            missing: AtomicU16::new(initial_frame.seq_len - 1),
        }
    }

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

    fn is_complete(&self) -> bool {
        self.missing.load(Ordering::SeqCst) == 0
    }

    fn reassemble(self) -> Box<[u8]> {
        assert!(self.is_complete(), "missing frames");
        self.segments
            .into_iter()
            .map(|lock| lock.into_inner().unwrap())
            .collect::<Vec<&[u8]>>()
            .concat()
            .into_boxed_slice()
    }
}

/// Represents a frame reassembler.
/// The [FrameReassembler] behaves as a sink [Sink] for [Segment].
/// Upon creation, also [Stream] for reassembled [Frames](Frame) is created.
/// The corresponding stream is closed either when [FrameReassembler::close] or
/// [futures::SinkExt::close] is called.
#[derive(Debug)]
pub struct FrameReassembler<'a> {
    sequences: DashMap<u32, SegmentBag<'a>>,
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
                earliest_frame: Default::default(),
                reassembled,
            },
            reassembled_recv,
        )
    }

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
                v.insert(SegmentBag::new(segment));
                self.earliest_frame.fetch_min(seq_id, Ordering::SeqCst);
            }
        }

        if self
            .sequences
            .get(&self.earliest_frame.load(Ordering::SeqCst))
            .is_some_and(|t| t.is_complete())
        {
            if let Some((seq_id, table)) = self.sequences.remove(&seq_id) {
                let frame_id = ((seq_id & 0xffff0000) >> 16) as u16;
                let _ = self.reassembled.try_send(Frame {
                    frame_id,
                    data: table.reassemble(),
                });
            }
        }

        Ok(())
    }

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
    use crate::frame::{Frame, FrameReassembler};
    use futures::StreamExt;
    use rand::prelude::SliceRandom;
    use rand::thread_rng;
    use rayon::prelude::*;

    const MTU: u16 = 10;
    const PACKET_COUNT: u16 = 65_535;
    const PACKET_SIZE: usize = 500;

    #[async_std::test]
    async fn test_shuffled_randomized() {
        let packets = (0..PACKET_COUNT)
            .into_par_iter()
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<PACKET_SIZE>().into(),
            })
            .collect::<Vec<_>>();

        let mut segments = packets.par_iter().flat_map(|f| f.segment(MTU)).collect::<Vec<_>>();

        let mut rng = thread_rng();
        segments.shuffle(&mut rng);

        let (fragmented, reassembled) = FrameReassembler::new();

        segments.into_iter().for_each(|b| {
            let _ = fragmented.push_segment(b);
        });
        fragmented.close();

        let reassembled_packets = reassembled.collect::<Vec<_>>().await;

        reassembled_packets
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, packets[i]));
    }

    #[async_std::test]
    async fn test_shuffled_randomized_parallel() {
        let packets = (0..PACKET_COUNT)
            .into_par_iter()
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<PACKET_SIZE>().into(),
            })
            .collect::<Vec<_>>();

        let mut segments = packets.par_iter().flat_map(|f| f.segment(MTU)).collect::<Vec<_>>();

        let mut rng = thread_rng();
        segments.shuffle(&mut rng);

        let (fragmented, reassembled) = FrameReassembler::new();

        segments.into_par_iter().for_each(|b| {
            let _ = fragmented.push_segment(b);
        });
        fragmented.close();

        let reassembled_packets = reassembled.collect::<Vec<_>>().await;

        reassembled_packets
            .into_par_iter()
            .enumerate()
            .for_each(|(i, frame)| assert_eq!(frame, packets[i]));
    }
}
