//! Contains definitions of Session protocol messages.

use std::collections::{BTreeMap, BTreeSet};

use bitvec::{BitArr, field::BitField, prelude::Msb0};

use crate::{
    errors::SessionError,
    protocol::{FrameId, SegmentId, SeqNum, SessionMessage},
};

/// Holds the Segment Retransmission Request message.
///
/// That is an ordered map of frame IDs and a bitmap of missing segments in each frame.
/// The bitmap can cover up a request for up to [`SegmentRequest::MAX_ENTRIES`] segments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SegmentRequest<const C: usize>(pub(super) BTreeMap<FrameId, SeqNum>);

/// Bitmap of segments missing in a frame.
///
/// Represented by `u8`, it can cover up to 8 segments per frame.
/// If a bit is set, the segment is *missing* from the frame.
pub type MissingSegmentsBitmap = BitArr!(for 1, in SeqNum, Msb0);

impl<const C: usize> SegmentRequest<C> {
    /// Size of a single segment retransmission request entry.
    pub const ENTRY_SIZE: usize = size_of::<FrameId>() + size_of::<SeqNum>();
    /// Maximum number of segment retransmission entries.
    pub const MAX_ENTRIES: usize = Self::SIZE / Self::ENTRY_SIZE;
    /// Maximum number of missing segments per frame.
    pub const MAX_MISSING_SEGMENTS_PER_FRAME: usize = SeqNum::BITS as usize;
    /// Size of the message.
    pub const SIZE: usize = C - SessionMessage::<C>::HEADER_SIZE;

    /// Returns the total number of segments to retransmit for all frames in this request.
    pub fn len(&self) -> usize {
        self.0
            .values()
            .take(Self::MAX_ENTRIES)
            .map(|e| e.count_ones() as usize)
            .sum()
    }

    /// Returns true if there are no segments to retransmit in this request.
    pub fn is_empty(&self) -> bool {
        self.0.iter().take(Self::MAX_ENTRIES).all(|(_, e)| e.count_ones() == 0)
    }
}

impl<const C: usize> IntoIterator for SegmentRequest<C> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = SegmentId;

    // An ordered iterator of missing segments in the form of SegmentId tuples.
    fn into_iter(self) -> Self::IntoIter {
        let seq_size = SeqNum::BITS as usize;
        let mut ret = Vec::with_capacity(seq_size * self.0.len());
        for (frame_id, missing) in self.0 {
            ret.extend(
                MissingSegmentsBitmap::from([missing])
                    .iter_ones()
                    .map(|i| SegmentId(frame_id, i as SeqNum)),
            );
        }
        ret.into_iter()
    }
}

// From FrameIds and bitmap of missing segments per frame
impl<const C: usize> FromIterator<(FrameId, MissingSegmentsBitmap)> for SegmentRequest<C> {
    fn from_iter<T: IntoIterator<Item = (FrameId, MissingSegmentsBitmap)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(fid, missing_segments)| (fid, missing_segments.load()))
                .collect(),
        )
    }
}

impl<const C: usize> TryFrom<&[u8]> for SegmentRequest<C> {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            let mut ret = Self::default();
            for (frame_id, missing) in value
                .chunks_exact(Self::ENTRY_SIZE)
                .map(|c| c.split_at(size_of::<FrameId>()))
            {
                let frame_id = FrameId::from_be_bytes(frame_id.try_into().map_err(|_| SessionError::ParseError)?);
                if frame_id > 0 {
                    ret.0.insert(
                        frame_id,
                        SeqNum::from_be_bytes(missing.try_into().map_err(|_| SessionError::ParseError)?),
                    );
                }
            }
            Ok(ret)
        } else {
            Err(SessionError::ParseError)
        }
    }
}

impl<const C: usize> From<SegmentRequest<C>> for Vec<u8> {
    fn from(value: SegmentRequest<C>) -> Self {
        let mut ret = vec![0u8; SegmentRequest::<C>::SIZE];
        let mut offset = 0;
        for (frame_id, seq_num) in value.0 {
            if offset + size_of::<FrameId>() + size_of::<SeqNum>() <= SegmentRequest::<C>::SIZE {
                ret[offset..offset + size_of::<FrameId>()].copy_from_slice(&frame_id.to_be_bytes());
                offset += size_of::<FrameId>();
                ret[offset..offset + size_of::<SeqNum>()].copy_from_slice(&seq_num.to_be_bytes());
                offset += size_of::<SeqNum>();
            } else {
                break;
            }
        }
        ret
    }
}

/// Holds the Frame Acknowledgement message.
/// This carries an ordered set of up to [`FrameAcknowledgements::MAX_ACK_FRAMES`] [frame IDs](FrameId)
/// that has been acknowledged by the counterparty.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameAcknowledgements<const C: usize>(pub(super) BTreeSet<FrameId>);

impl<const C: usize> FrameAcknowledgements<C> {
    /// Maximum number of [`FrameIds`](FrameId) that can be accommodated.
    pub const MAX_ACK_FRAMES: usize = Self::SIZE / size_of::<FrameId>();
    /// Size of the message.
    pub const SIZE: usize = C - SessionMessage::<C>::HEADER_SIZE;

    /// Pushes the frame ID.
    /// Returns true if the value has been pushed or false it the container is full or already
    /// contains that value.
    #[inline]
    pub fn push(&mut self, frame_id: FrameId) -> bool {
        !self.is_full() && self.0.insert(frame_id)
    }

    /// Number of acknowledged frame IDs in this instance.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no frame IDs in this instance.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Indicates whether the [maximum number of frame IDs](FrameAcknowledgements::MAX_ACK_FRAMES)
    /// has been reached.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.0.len() == Self::MAX_ACK_FRAMES
    }

    /// Creates a vector of [`FrameAcknowledgements`](FrameAcknowledgements) from the given iterator
    /// of acknowledged [`FrameIds`](FrameId).
    pub fn new_multiple<T: IntoIterator<Item = FrameId>>(items: T) -> Vec<Self> {
        let mut out = Vec::with_capacity(2);
        let mut frame_ack = Self::default();
        for frame_id in items {
            if frame_ack.is_full() {
                out.push(frame_ack);
                frame_ack = Self::default();
            }

            frame_ack.push(frame_id);
        }
        out.push(frame_ack);
        out
    }
}

impl<const C: usize> TryFrom<Vec<FrameId>> for FrameAcknowledgements<C> {
    type Error = SessionError;

    fn try_from(value: Vec<FrameId>) -> Result<Self, Self::Error> {
        if value.len() <= Self::MAX_ACK_FRAMES {
            value
                .into_iter()
                .map(|v| {
                    if v > 0 {
                        Ok(v)
                    } else {
                        Err(SessionError::InvalidFrameId)
                    }
                })
                .collect::<Result<BTreeSet<_>, _>>()
                .map(Self)
        } else {
            Err(SessionError::DataTooLong)
        }
    }
}

impl<const C: usize> IntoIterator for FrameAcknowledgements<C> {
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;
    type Item = FrameId;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, const C: usize> TryFrom<&'a [u8]> for FrameAcknowledgements<C> {
    type Error = SessionError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self(
                // chunks_exact discards the remainder bytes
                value
                    .chunks_exact(size_of::<FrameId>())
                    .map(|v| FrameId::from_be_bytes(v.try_into().unwrap()))
                    .filter(|f| *f > 0)
                    .collect(),
            ))
        } else {
            Err(SessionError::ParseError)
        }
    }
}

impl<const C: usize> From<FrameAcknowledgements<C>> for Vec<u8> {
    fn from(value: FrameAcknowledgements<C>) -> Self {
        value
            .0
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .chain(std::iter::repeat(0_u8))
            .take(FrameAcknowledgements::<C>::SIZE)
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_acks_multiple_single() {
        let mut acks = FrameAcknowledgements::<1024>::new_multiple(vec![1, 2, 3]);
        assert_eq!(acks.len(), 1);

        let ids = acks.remove(0).into_iter().collect::<Vec<_>>();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_frame_acks_multiple_many() {
        const MAX: usize = FrameAcknowledgements::<1024>::MAX_ACK_FRAMES;

        let expected = (0..(2 * MAX + 2) as FrameId).collect::<Vec<_>>();
        let acks = FrameAcknowledgements::<1024>::new_multiple(expected.clone());
        assert_eq!(3, acks.len());

        assert_eq!(MAX, acks[0].len());
        assert_eq!(MAX, acks[1].len());
        assert_eq!(2, acks[2].len());

        let actual = acks.into_iter().flat_map(|a| a.into_iter()).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_missing_segments_in_segment_request() {
        let frame_1_missing: MissingSegmentsBitmap = [0b00000000_u8].into();
        let frame_2_missing: MissingSegmentsBitmap = [0b00100000_u8].into();
        let frame_3_missing: MissingSegmentsBitmap = [0b00111001_u8].into();
        let frame_4_missing: MissingSegmentsBitmap = [0b11111111_u8].into();

        let req = SegmentRequest::<1000>::from_iter([
            (4, frame_4_missing),
            (1, frame_1_missing),
            (3, frame_3_missing),
            (2, frame_2_missing),
        ]);

        // Iterator of SegmentIds is guaranteed to be sorted
        let missing = req.into_iter().collect::<Vec<SegmentId>>();
        let missing_seg_ids = [
            SegmentId(2, 2),
            SegmentId(3, 2),
            SegmentId(3, 3),
            SegmentId(3, 4),
            SegmentId(3, 7),
            SegmentId(4, 0),
            SegmentId(4, 1),
            SegmentId(4, 2),
            SegmentId(4, 3),
            SegmentId(4, 4),
            SegmentId(4, 5),
            SegmentId(4, 6),
            SegmentId(4, 7),
        ];

        assert_eq!(missing, missing_seg_ids);
    }
}
