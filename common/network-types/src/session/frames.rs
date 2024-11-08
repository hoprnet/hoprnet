use crate::prelude::errors::SessionError;
use crate::prelude::{Frame, FrameId, Segment};
use crate::session::frame::SeqNum;
use bitvec::prelude::BitVec;
use std::time::Instant;

#[derive(Debug)]
pub(crate) struct FrameBuilder {
    segments: Vec<Option<Segment>>,
    frame_id: FrameId,
    seg_remaining: SeqNum,
    recv_bytes: usize,
    pub(crate) last_recv: Instant,
}

impl From<Segment> for FrameBuilder {
    fn from(value: Segment) -> Self {
        let idx = value.seq_idx;
        let mut ret = Self {
            segments: vec![None; value.seq_len as usize],
            frame_id: value.frame_id,
            seg_remaining: value.seq_len - 1,
            recv_bytes: value.data.len(),
            last_recv: Instant::now(),
        };

        ret.segments[idx as usize] = Some(value);

        ret
    }
}

impl TryFrom<FrameBuilder> for Frame {
    type Error = SessionError;

    fn try_from(value: FrameBuilder) -> Result<Self, Self::Error> {
        let mut reassembled = Vec::with_capacity(value.recv_bytes);
        for segment in value.segments {
            match segment {
                Some(segment) => {
                    reassembled.extend_from_slice(&segment.data);
                }
                None => return Err(SessionError::IncompleteFrame(value.frame_id)),
            }
        }

        Ok(Frame {
            frame_id: value.frame_id,
            data: reassembled.into_boxed_slice(),
        })
    }
}

impl FrameBuilder {
    pub fn add_segment(&mut self, segment: Segment) -> Result<(), SessionError> {
        let idx = segment.seq_idx;
        if segment.frame_id != self.frame_id
            || idx as usize >= self.segments.len()
            || segment.seq_len as usize != self.segments.len()
            || self.seg_remaining == 0
        {
            return Err(SessionError::InvalidSegment);
        }

        if self.segments[idx as usize].is_some() {
            return Err(SessionError::InvalidSegment);
        }

        self.recv_bytes += segment.data.len();
        self.seg_remaining -= 1;
        self.segments[idx as usize] = Some(segment);
        self.last_recv = Instant::now();
        Ok(())
    }

    #[cfg(feature = "frame-inspector")]
    pub fn as_missing(&self) -> BitVec {
        self.segments.iter().map(Option::is_none).collect()
    }

    #[inline]
    pub fn frame_id(&self) -> &FrameId {
        &self.frame_id
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        self.seg_remaining == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn frame_builder_should_return_ordered_segments() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 1,
            seq_len: 3,
            data: (*b" new ").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 2,
            seq_len: 3,
            data: (*b"world").into(),
        })?;

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 3,
            data: (*b"hello").into(),
        })?;

        assert!(fb.is_complete());
        assert_eq!(bits![0, 0, 0], fb.as_missing());

        let reassembled: Frame = fb.try_into()?;
        assert_eq!(1, reassembled.frame_id);
        assert_eq!(b"hello new world", reassembled.data.as_ref());

        Ok(())
    }

    #[test]
    fn frame_builder_should_not_accept_invalid_segments() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 2,
            data: (*b"hello world").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 3,
            seq_len: 2,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept invalid segment");

        fb.add_segment(Segment {
            frame_id: 2,
            seq_idx: 1,
            seq_len: 2,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept segment from another frame");

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 1,
            seq_len: 3,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept invalid segment");

        Ok(())
    }

    #[test]
    fn frame_builder_should_not_accept_segments_when_complete() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 1,
            data: (*b"hello world").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 1,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept segment when complete");

        Ok(())
    }

    #[test]
    fn frame_builder_should_not_accept_duplicate_segment() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 2,
            data: (*b"hello world").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 2,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept duplicate segment");

        Ok(())
    }
}
