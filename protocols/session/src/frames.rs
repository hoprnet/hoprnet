//! Contains basic types for the Session protocol.

use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
};

use hopr_primitive_types::prelude::GeneralError;

use crate::{errors::SessionError, utils::to_hex_shortened};

/// ID of a [Frame].
pub type FrameId = u32;

/// Type representing the sequence numbers in a [Frame].
pub type SeqNum = u8;

/// Convenience type that identifies a segment within a frame.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct SegmentId(pub FrameId, pub SeqNum);

impl Display for SegmentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "seg({},{})", self.0, self.1)
    }
}

/// Data frame of arbitrary length.
///
/// The frame can be segmented into [segments](Segment) and reassembled back
/// via [`FrameBuilder`].
#[derive(Clone, PartialEq, Eq)]
pub struct Frame {
    /// Identifier of this frame.
    pub frame_id: FrameId,
    /// Frame data.
    pub data: Box<[u8]>,
    /// Indicates whether the frame is the last one of the frame sequence.
    ///
    /// This indicates that the Session is over.
    pub is_terminating: bool,
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("frame_id", &self.frame_id)
            .field("len", &self.data.len())
            .field("data", &to_hex_shortened(&self.data, 16))
            .field("is_terminating", &self.is_terminating)
            .finish()
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

/// Wrapper for [`Frame`] that implements comparison and total ordering based on [`FrameId`].
#[derive(Clone, Debug)]
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

/// Carries segment flags and the length of the segment sequence.
#[derive(Copy, Clone, Eq, PartialEq, Default, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct SeqIndicator(SeqNum);

impl SeqIndicator {
    /// Maximum length of a segment sequence.
    pub const MAX: SeqNum = 0b0011_1111;

    #[inline]
    pub const fn new_with_flags(seq_len: SeqNum, is_terminating: bool) -> Self {
        let flags = ((is_terminating as u8) << 7) | (seq_len & Self::MAX);
        Self(flags)
    }

    #[inline]
    pub const fn new(seq_len: SeqNum) -> Self {
        Self::new_with_flags(seq_len, false)
    }

    #[inline]
    const fn new_unchecked(seq_ind: SeqNum) -> Self {
        Self(seq_ind)
    }

    #[inline]
    pub fn with_terminating_bit(self, is_terminating: bool) -> Self {
        Self::new_with_flags(self.0, is_terminating)
    }

    #[inline]
    pub const fn is_terminating(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    #[inline]
    pub const fn seq_len(&self) -> SeqNum {
        self.0 & Self::MAX
    }

    #[inline]
    pub const fn value(&self) -> SeqNum {
        self.0
    }
}

impl Debug for SeqIndicator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeqIndicator")
            .field("seq_len", &self.seq_len())
            .field("is_terminating", &self.is_terminating())
            .finish()
    }
}

impl Display for SeqIndicator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.seq_len(), if self.is_terminating() { "*" } else { "" })
    }
}

impl TryFrom<SeqNum> for SeqIndicator {
    type Error = GeneralError;

    fn try_from(value: SeqNum) -> Result<Self, Self::Error> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(GeneralError::InvalidInput)
        }
    }
}

/// Represents a frame segment.
///
/// Besides the data, a segment carries information about the total number of
/// segments in the original frame ([`SeqIndicator`]), its index within the frame ([`SeqNum`]), and
/// ID of that frame ([`FrameId`]).
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Segment {
    /// ID of the [Frame] this segment belongs to.
    pub frame_id: FrameId,
    /// Index of this segment within the segment sequence.
    pub seq_idx: SeqNum,
    /// Flags of the segment sequence (includes sequence length).
    pub seq_flags: SeqIndicator,
    /// Data in this segment.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub data: Box<[u8]>,
}

impl Segment {
    /// Size of the segment header.
    pub const HEADER_SIZE: usize = size_of::<FrameId>() + 2 * size_of::<SeqNum>();

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
        self.seq_idx == self.seq_flags.seq_len() - 1
    }

    /// Short-cut to check if this segment is a terminating segment.
    #[inline]
    pub fn is_terminating(&self) -> bool {
        self.seq_flags.is_terminating()
    }

    /// Creates an empty `Segment` with the terminating flag set.
    pub fn terminating(frame_id: FrameId) -> Self {
        Self {
            frame_id,
            seq_idx: 0,
            seq_flags: SeqIndicator::new_with_flags(1, true),
            data: Box::default(),
        }
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Segment")
            .field("frame_id", &self.frame_id)
            .field("seq_id", &self.seq_idx)
            .field("seq_flags", &self.seq_flags)
            .field("data", &to_hex_shortened(&self.data, 16))
            .finish()
    }
}

impl PartialOrd<Segment> for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.frame_id.cmp(&other.frame_id) {
            Ordering::Equal => self.seq_idx.cmp(&other.seq_idx),
            cmp => cmp,
        }
    }
}

impl From<&Segment> for SegmentId {
    fn from(value: &Segment) -> Self {
        value.id()
    }
}

impl From<Segment> for Vec<u8> {
    fn from(value: Segment) -> Self {
        let mut ret = Vec::with_capacity(Segment::HEADER_SIZE + value.data.len());
        ret.extend_from_slice(value.frame_id.to_be_bytes().as_ref());
        ret.extend_from_slice(value.seq_idx.to_be_bytes().as_ref());
        ret.push(value.seq_flags.value());
        ret.extend_from_slice(value.data.as_ref());
        ret
    }
}

impl TryFrom<&[u8]> for Segment {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::HEADER_SIZE {
            return Err(SessionError::InvalidSegment);
        }

        let (header, data) = value.split_at(Self::HEADER_SIZE);
        let segment = Segment {
            frame_id: FrameId::from_be_bytes(header[0..4].try_into().map_err(|_| SessionError::InvalidSegment)?),
            seq_idx: SeqNum::from_be_bytes(header[4..5].try_into().map_err(|_| SessionError::InvalidSegment)?),
            seq_flags: SeqIndicator::new_unchecked(header[5]),
            data: data.into(),
        };
        (segment.frame_id > 0 && segment.seq_idx < segment.seq_flags.seq_len())
            .then_some(segment)
            .ok_or(SessionError::InvalidSegment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminating_sequence_indicator_should_be_greater_than_non_terminating() -> anyhow::Result<()> {
        let ind_1 = SeqIndicator::new_with_flags(1, true);
        let ind_2 = SeqIndicator::new_with_flags(1, false);

        assert!(ind_1 > ind_2);
        Ok(())
    }

    #[test]
    fn segment_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let seg_1 = Segment {
            frame_id: 10,
            seq_idx: 0,
            seq_flags: 2.try_into()?,
            data: Box::new([123u8]),
        };

        let seg_2 = Segment::try_from(Vec::from(seg_1.clone()).as_slice())?;
        assert_eq!(seg_1, seg_2);
        Ok(())
    }
}
