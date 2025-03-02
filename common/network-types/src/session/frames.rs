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
            || self.segments[idx as usize].is_some()
        {
            return Err(SessionError::InvalidSegment);
        }

        self.recv_bytes += segment.data.len();
        self.seg_remaining -= 1;
        self.segments[idx as usize] = Some(segment);
        self.last_recv = Instant::now();
        Ok(())
    }

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

#[derive(Clone, Debug)]
pub struct FrameInspector(pub(crate) FrameDashMap);

impl FrameInspector {
    pub fn missing_segments(&self, frame_id: &FrameId) -> Option<bitvec::prelude::BitVec> {
        self.0 .0.get(frame_id).map(|f| f.as_missing())
    }
}

pub(crate) trait FrameMapOccupiedEntry {
    fn get_builder_mut(&mut self) -> &mut FrameBuilder;

    fn frame_id(&self) -> &FrameId;

    fn finalize(self) -> FrameBuilder;
}

pub(crate) trait FrameMapVacantEntry {
    fn insert_builder(self, value: FrameBuilder);
}

pub(crate) enum FrameMapEntry<O: FrameMapOccupiedEntry, V: FrameMapVacantEntry> {
    Occupied(O),
    Vacant(V),
}

pub(crate) trait FrameMap {
    type ExistingEntry<'a>: FrameMapOccupiedEntry
    where
        Self: 'a;
    type VacantEntry<'a>: FrameMapVacantEntry
    where
        Self: 'a;

    fn with_capacity(capacity: usize) -> Self;

    fn entry(&mut self, frame_id: FrameId) -> FrameMapEntry<Self::ExistingEntry<'_>, Self::VacantEntry<'_>>;

    fn len(&self) -> usize;

    fn retain(&mut self, f: impl FnMut(&FrameId, &mut FrameBuilder) -> bool);
}

impl<'a> FrameMapOccupiedEntry for dashmap::OccupiedEntry<'a, FrameId, FrameBuilder> {
    fn get_builder_mut(&mut self) -> &mut FrameBuilder {
        self.get_mut()
    }

    fn frame_id(&self) -> &FrameId {
        self.key()
    }

    fn finalize(self) -> FrameBuilder {
        self.remove()
    }
}

impl<'a> FrameMapVacantEntry for dashmap::VacantEntry<'a, FrameId, FrameBuilder> {
    fn insert_builder(self, value: FrameBuilder) {
        self.insert(value);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FrameDashMap(std::sync::Arc<dashmap::DashMap<FrameId, FrameBuilder>>);

impl FrameMap for FrameDashMap {
    type ExistingEntry<'a> = dashmap::OccupiedEntry<'a, FrameId, FrameBuilder>;
    type VacantEntry<'a> = dashmap::VacantEntry<'a, FrameId, FrameBuilder>;

    fn with_capacity(capacity: usize) -> Self {
        Self(std::sync::Arc::new(dashmap::DashMap::with_capacity(capacity)))
    }

    fn entry(&mut self, frame_id: FrameId) -> FrameMapEntry<Self::ExistingEntry<'_>, Self::VacantEntry<'_>> {
        match self.0.entry(frame_id) {
            dashmap::Entry::Occupied(e) => FrameMapEntry::Occupied(e),
            dashmap::Entry::Vacant(v) => FrameMapEntry::Vacant(v),
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn retain(&mut self, f: impl FnMut(&FrameId, &mut FrameBuilder) -> bool) {
        self.0.retain(f)
    }
}

impl<'a> FrameMapOccupiedEntry for std::collections::hash_map::OccupiedEntry<'a, FrameId, FrameBuilder> {
    fn get_builder_mut(&mut self) -> &mut FrameBuilder {
        self.get_mut()
    }

    fn frame_id(&self) -> &FrameId {
        self.key()
    }

    fn finalize(self) -> FrameBuilder {
        self.remove()
    }
}

impl<'a> FrameMapVacantEntry for std::collections::hash_map::VacantEntry<'a, FrameId, FrameBuilder> {
    fn insert_builder(self, value: FrameBuilder) {
        self.insert(value);
    }
}

pub(crate) struct FrameHashMap(std::collections::HashMap<FrameId, FrameBuilder>);

impl FrameMap for FrameHashMap {
    type ExistingEntry<'a> = std::collections::hash_map::OccupiedEntry<'a, FrameId, FrameBuilder>;
    type VacantEntry<'a> = std::collections::hash_map::VacantEntry<'a, FrameId, FrameBuilder>;

    fn with_capacity(capacity: usize) -> Self {
        Self(std::collections::HashMap::with_capacity(capacity))
    }

    fn entry(&mut self, frame_id: FrameId) -> FrameMapEntry<Self::ExistingEntry<'_>, Self::VacantEntry<'_>> {
        match self.0.entry(frame_id) {
            std::collections::hash_map::Entry::Occupied(e) => FrameMapEntry::Occupied(e),
            std::collections::hash_map::Entry::Vacant(v) => FrameMapEntry::Vacant(v),
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn retain(&mut self, f: impl FnMut(&FrameId, &mut FrameBuilder) -> bool) {
        self.0.retain(f)
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
