use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
    mem,
    time::Instant,
};

use crate::{prelude::errors::SessionError, session::protocol::MissingSegmentsBitmap};

/// ID of a [Frame].
pub type FrameId = u32;
/// Type representing the sequence numbers in a [Frame].
pub type SeqNum = u8;

/// Convenience type that identifies a segment within a frame.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
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

/// Data frame of arbitrary length.
/// The frame can be segmented into [segments](Segment) and reassembled back
/// via [FrameReassembler].
#[derive(Clone, PartialEq, Eq)]
pub struct Frame {
    /// Identifier of this frame.
    pub frame_id: FrameId,
    /// Frame data.
    pub data: Box<[u8]>,
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const DBG_LEN: usize = 16;
        let excerpt = if self.data.len() > DBG_LEN {
            format!(
                "{}..{}",
                hex::encode(&self.data[0..DBG_LEN / 2]),
                hex::encode(&self.data[self.data.len() - DBG_LEN / 2..])
            )
        } else {
            hex::encode(&self.data)
        };

        f.debug_struct("Frame")
            .field("frame_id", &self.frame_id)
            .field("len", &self.data.len())
            .field("data", &excerpt)
            .finish()
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

/// Wrapper for [`Frame`] that implements comparison and total ordering based on [`FrameId`].
#[derive(Clone)]
pub struct OrderedFrame(pub Frame);

impl Eq for OrderedFrame {}

impl PartialEq<Self> for OrderedFrame {
    fn eq(&self, other: &Self) -> bool {
        self.0.frame_id == other.0.frame_id
    }
}

impl PartialEq<FrameId> for OrderedFrame {
    fn eq(&self, other: &FrameId) -> bool {
        self.0.frame_id == *other
    }
}

impl PartialOrd<Self> for OrderedFrame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<FrameId> for OrderedFrame {
    fn partial_cmp(&self, other: &FrameId) -> Option<Ordering> {
        Some(self.0.frame_id.cmp(other))
    }
}

impl Ord for OrderedFrame {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.frame_id.cmp(&other.0.frame_id)
    }
}

impl From<Frame> for OrderedFrame {
    fn from(value: Frame) -> Self {
        Self(value)
    }
}

impl From<OrderedFrame> for Frame {
    fn from(value: OrderedFrame) -> Self {
        value.0
    }
}

/// Represents a frame segment.
/// Besides the data, a segment carries information about the total number of
/// segments in the original frame, its index within the frame and
/// ID of that frame.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Segment {
    /// ID of the [Frame] this segment belongs to.
    pub frame_id: FrameId,
    /// Index of this segment within the segment sequence.
    pub seq_idx: SeqNum,
    /// Total number of segments within this segment sequence.
    pub seq_len: SeqNum,
    /// Data in this segment.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub data: Box<[u8]>,
}

impl Segment {
    /// Size of the segment header.
    pub const HEADER_SIZE: usize = mem::size_of::<FrameId>() + 2 * mem::size_of::<SeqNum>();
    /// The minimum size of a segment: [`Segment::HEADER_SIZE`] + 1 byte of data.
    pub const MINIMUM_SIZE: usize = Self::HEADER_SIZE + 1;

    /// Returns the [SegmentId] for this segment.
    pub fn id(&self) -> SegmentId {
        SegmentId(self.frame_id, self.seq_idx)
    }

    /// Length of the segment data plus header.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        Self::HEADER_SIZE + self.data.len()
    }

    /// Indicates whether this segment is the last one from the frame.
    #[inline]
    pub fn is_last(&self) -> bool {
        self.seq_idx == self.seq_len - 1
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
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let (header, data) = value.split_at(Self::HEADER_SIZE);
        let segment = Segment {
            frame_id: FrameId::from_be_bytes(header[0..4].try_into().map_err(|_| SessionError::InvalidSegment)?),
            seq_idx: SeqNum::from_be_bytes(header[4..5].try_into().map_err(|_| SessionError::InvalidSegment)?),
            seq_len: SeqNum::from_be_bytes(header[5..6].try_into().map_err(|_| SessionError::InvalidSegment)?),
            data: data.into(),
        };
        (segment.frame_id > 0 && segment.seq_idx < segment.seq_len)
            .then_some(segment)
            .ok_or(SessionError::InvalidSegment)
    }
}

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
        value
            .segments
            .into_iter()
            .try_fold(Vec::with_capacity(value.recv_bytes), |mut acc, segment| match segment {
                Some(segment) => {
                    acc.extend_from_slice(&segment.data);
                    Ok(acc)
                }
                None => Err(SessionError::IncompleteFrame(value.frame_id)),
            })
            .map(|data| Frame {
                frame_id: value.frame_id,
                data: data.into_boxed_slice(),
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

    pub fn as_missing(&self) -> MissingSegmentsBitmap {
        let mut ret = MissingSegmentsBitmap::ZERO;
        self.segments
            .iter()
            .take(SeqNum::BITS as usize)
            .enumerate()
            .for_each(|(i, v)| ret.set(i, v.is_none()));
        ret
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        self.seg_remaining == 0
    }
}

// Must use only FrameDashMap, others cannot be reference-cloned
#[derive(Clone, Debug)]
pub struct FrameInspector(pub(crate) FrameDashMap);

impl FrameInspector {
    /// Returns a [`MissingSegmentsBitmap`] of missing segments in a frame.
    pub fn missing_segments(&self, frame_id: &FrameId) -> Option<MissingSegmentsBitmap> {
        self.0.0.get(frame_id).map(|f| f.as_missing())
    }
}

pub(crate) trait FrameMapOccupiedEntry {
    fn get_builder_mut(&mut self) -> &mut FrameBuilder;

    #[allow(unused)]
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

impl FrameMapOccupiedEntry for dashmap::OccupiedEntry<'_, FrameId, FrameBuilder> {
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

impl FrameMapVacantEntry for dashmap::VacantEntry<'_, FrameId, FrameBuilder> {
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

#[cfg(not(feature = "hashbrown"))]
pub(crate) struct FrameHashMap(std::collections::HashMap<FrameId, FrameBuilder>);

#[cfg(not(feature = "hashbrown"))]
impl FrameMapOccupiedEntry for std::collections::hash_map::OccupiedEntry<'_, FrameId, FrameBuilder> {
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

#[cfg(not(feature = "hashbrown"))]
impl FrameMapVacantEntry for std::collections::hash_map::VacantEntry<'_, FrameId, FrameBuilder> {
    fn insert_builder(self, value: FrameBuilder) {
        self.insert(value);
    }
}

#[cfg(not(feature = "hashbrown"))]
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

#[cfg(feature = "hashbrown")]
pub(crate) struct FrameHashMap(hashbrown::HashMap<FrameId, FrameBuilder>);

#[cfg(feature = "hashbrown")]
impl FrameMapOccupiedEntry for hashbrown::hash_map::OccupiedEntry<'_, FrameId, FrameBuilder> {
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

#[cfg(feature = "hashbrown")]
impl FrameMapVacantEntry for hashbrown::hash_map::VacantEntry<'_, FrameId, FrameBuilder> {
    fn insert_builder(self, value: FrameBuilder) {
        self.insert(value);
    }
}

#[cfg(feature = "hashbrown")]
impl FrameMap for FrameHashMap {
    type ExistingEntry<'a> = hashbrown::hash_map::OccupiedEntry<'a, FrameId, FrameBuilder>;
    type VacantEntry<'a> = hashbrown::hash_map::VacantEntry<'a, FrameId, FrameBuilder>;

    fn with_capacity(capacity: usize) -> Self {
        Self(hashbrown::HashMap::with_capacity(capacity))
    }

    fn entry(&mut self, frame_id: FrameId) -> FrameMapEntry<Self::ExistingEntry<'_>, Self::VacantEntry<'_>> {
        match self.0.entry(frame_id) {
            hashbrown::hash_map::Entry::Occupied(e) => FrameMapEntry::Occupied(e),
            hashbrown::hash_map::Entry::Vacant(v) => FrameMapEntry::Vacant(v),
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
        assert_eq!(MissingSegmentsBitmap::ZERO, fb.as_missing());

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
