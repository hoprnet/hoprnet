use std::time::Instant;

use crate::{
    errors::SessionError,
    frames::{Frame, FrameId, Segment, SeqNum},
    protocol::MissingSegmentsBitmap,
};

/// A helper object that reassembles segments into frames.
#[derive(Debug)]
pub struct FrameBuilder {
    segments: Vec<Option<Segment>>,
    frame_id: FrameId,
    seg_remaining: SeqNum,
    recv_bytes: usize,
    pub(crate) last_recv: Instant,
    #[cfg(all(not(test), feature = "prometheus"))]
    pub(crate) created: Instant,
}

impl From<Segment> for FrameBuilder {
    fn from(value: Segment) -> Self {
        let idx = value.seq_idx;
        let mut ret = Self {
            segments: vec![None; value.seq_flags.seq_len() as usize],
            frame_id: value.frame_id,
            seg_remaining: value.seq_flags.seq_len() - 1,
            recv_bytes: value.data.len(),
            last_recv: Instant::now(),
            #[cfg(all(not(test), feature = "prometheus"))]
            created: Instant::now(),
        };

        ret.segments[idx as usize] = Some(value);

        ret
    }
}

impl FrameBuilder {
    /// Adds a segment to this frame.
    ///
    /// Fails if the segment is invalid for this frame.
    pub fn add_segment(&mut self, segment: Segment) -> Result<(), SessionError> {
        let idx = segment.seq_idx;
        if segment.frame_id != self.frame_id
            || idx as usize >= self.segments.len()
            || segment.seq_flags.seq_len() as usize != self.segments.len()
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

    /// Retrieves the bitmap of missing segments in this frame.
    pub fn as_missing(&self) -> MissingSegmentsBitmap {
        let mut ret = MissingSegmentsBitmap::ZERO;
        self.segments
            .iter()
            .take(SeqNum::BITS as usize)
            .enumerate()
            .for_each(|(i, v)| ret.set(i, v.is_none()));
        ret
    }

    /// Retrieves the frame's ID.
    #[inline]
    pub fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    /// Indicates if all segments of this frame were added.
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.seg_remaining == 0
    }
}

impl TryFrom<FrameBuilder> for Frame {
    type Error = SessionError;

    fn try_from(value: FrameBuilder) -> Result<Self, Self::Error> {
        // The Frame has the terminating flag set
        // if any of its segments had the terminating indicator set
        let mut is_terminating = false;
        value
            .segments
            .into_iter()
            .try_fold(Vec::with_capacity(value.recv_bytes), |mut acc, segment| match segment {
                Some(segment) => {
                    acc.extend_from_slice(&segment.data);
                    is_terminating = is_terminating || segment.seq_flags.is_terminating();
                    Ok(acc)
                }
                None => Err(SessionError::IncompleteFrame(value.frame_id)),
            })
            .map(|data| Frame {
                frame_id: value.frame_id,
                data: data.into_boxed_slice(),
                is_terminating,
            })
    }
}

/// Allows inspecting incomplete frame buffer inside the Reassembler.
// Must use only FrameDashMap, others cannot be reference-cloned
#[derive(Clone, Debug)]
pub struct FrameInspector(pub(crate) FrameDashMap);

impl FrameInspector {
    /// Indicates how many incomplete frames there could be per one complete/discarded frame.
    pub const INCOMPLETE_FRAME_RATIO: usize = 2;

    pub fn new(capacity: usize) -> Self {
        Self(FrameDashMap::with_capacity(Self::INCOMPLETE_FRAME_RATIO * capacity + 1))
    }

    /// Returns a [`MissingSegmentsBitmap`] of missing segments in a frame.
    pub fn missing_segments(&self, frame_id: &FrameId) -> Option<MissingSegmentsBitmap> {
        self.0.0.get(frame_id).map(|f| f.as_missing())
    }
}

/// Trait describing an occupied entry in a [`FrameMap`].
///
/// See [`FrameMap::entry`] for details.
pub trait FrameMapOccupiedEntry {
    /// Gets mutable reference to the builder assigned to this frame.
    fn get_builder_mut(&mut self) -> &mut FrameBuilder;

    #[allow(unused)]
    fn frame_id(&self) -> &FrameId;

    /// Removes the entry and returns the associated [`FrameBuilder`].
    /// The `FrameBuilder` may or may not be [complete](FrameBuilder::is_complete).
    fn finalize(self) -> FrameBuilder;
}

/// Trait describing a vacant entry in a [`FrameMap`].
///
/// See [`FrameMap::entry`] for details.
pub trait FrameMapVacantEntry {
    /// Insert a new [`FrameBuilder`] into the empty entry.
    fn insert_builder(self, value: FrameBuilder);
}

#[derive(strum::EnumTryAs)]
pub enum FrameMapEntry<O: FrameMapOccupiedEntry, V: FrameMapVacantEntry> {
    Occupied(O),
    Vacant(V),
}

/// An abstraction of a Hash Map, suitable for reassembling frames.
pub trait FrameMap {
    type ExistingEntry<'a>: FrameMapOccupiedEntry
    where
        Self: 'a;
    type VacantEntry<'a>: FrameMapVacantEntry
    where
        Self: 'a;

    /// Creates a new map with the given capacity.
    fn with_capacity(capacity: usize) -> Self;

    /// Returns the [`FrameMapEntry`] API for a vacant or existing entry.
    fn entry(&mut self, frame_id: FrameId) -> FrameMapEntry<Self::ExistingEntry<'_>, Self::VacantEntry<'_>>;

    /// Number of elements in the map.
    fn len(&self) -> usize;

    /// Removes all elements from the map, for which the given predicate evaluates to `false`.
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

/// A [`FrameMap`] implementation using reference-counted `dashmap::DashMap` as a backend.
#[derive(Clone, Debug)]
pub struct FrameDashMap(std::sync::Arc<dashmap::DashMap<FrameId, FrameBuilder>>);

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

/// A [`FrameMap`] implementation using `std::collections::HashMap` as backend.
#[cfg(not(feature = "hashbrown"))]
pub struct FrameHashMap(std::collections::HashMap<FrameId, FrameBuilder>);

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

/// A [`FrameMap`] implementation using `hashbrown::HashMap` as backend.
#[cfg(feature = "hashbrown")]
pub struct FrameHashMap(hashbrown::HashMap<FrameId, FrameBuilder>);

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
    use crate::frames::SeqIndicator;

    #[test]
    fn frame_builder_should_return_ordered_segments() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 1,
            seq_flags: 3.try_into()?,
            data: (*b" new ").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 2,
            seq_flags: 3.try_into()?,
            data: (*b"world").into(),
        })?;

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: 3.try_into()?,
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
    fn frame_builder_should_correctly_mark_terminating_flag() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 1,
            seq_flags: 3.try_into()?,
            data: (*b" new ").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 2,
            seq_flags: SeqIndicator::new_with_flags(3, true),
            data: (*b"world").into(),
        })?;

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: 3.try_into()?,
            data: (*b"hello").into(),
        })?;

        assert!(fb.is_complete());
        assert_eq!(MissingSegmentsBitmap::ZERO, fb.as_missing());

        let reassembled: Frame = fb.try_into()?;
        assert_eq!(1, reassembled.frame_id);
        assert_eq!(b"hello new world", reassembled.data.as_ref());
        assert!(reassembled.is_terminating);

        Ok(())
    }

    #[test]
    fn frame_builder_should_correctly_allow_empty_segments() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 1,
            seq_flags: 4.try_into()?,
            data: (*b" new ").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 2,
            seq_flags: 4.try_into()?,
            data: (*b"world").into(),
        })?;

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: 4.try_into()?,
            data: (*b"hello").into(),
        })?;

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 3,
            seq_flags: SeqIndicator::new_with_flags(4, true),
            data: Box::new([]),
        })?;

        assert!(fb.is_complete());
        assert_eq!(MissingSegmentsBitmap::ZERO, fb.as_missing());

        let reassembled: Frame = fb.try_into()?;
        assert_eq!(1, reassembled.frame_id);
        assert_eq!(b"hello new world", reassembled.data.as_ref());
        assert!(reassembled.is_terminating);

        Ok(())
    }

    #[test]
    fn frame_builder_should_not_accept_invalid_segments() -> anyhow::Result<()> {
        let mut fb = FrameBuilder::from(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: 2.try_into()?,
            data: (*b"hello world").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 3,
            seq_flags: 2.try_into()?,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept invalid segment");

        fb.add_segment(Segment {
            frame_id: 2,
            seq_idx: 1,
            seq_flags: 2.try_into()?,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept segment from another frame");

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 1,
            seq_flags: 3.try_into()?,
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
            seq_flags: 1.try_into()?,
            data: (*b"hello world").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: 1.try_into()?,
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
            seq_flags: 2.try_into()?,
            data: (*b"hello world").into(),
        });

        fb.add_segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: 2.try_into()?,
            data: (*b"foo").into(),
        })
        .expect_err("should not accept duplicate segment");

        Ok(())
    }
}
