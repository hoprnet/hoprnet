//! # `Session` protocol messages
//!
//! The protocol components are built via low-level types of the `frame` module, such as
//! [`Segment`] and [`Frame`](crate::session::Frame).
//! Most importantly, the `Session` protocol fixes the maximum number of segments per frame
//! to 8 (see [`MAX_SEGMENTS_PER_FRAME`](SessionMessage::MAX_SEGMENTS_PER_FRAME)).
//! Since each segment must fit within a maximum transmission unit (MTU),
//! a frame can be at most *eight* times the size of the MTU.
//!
//! The [current version](SessionMessage::VERSION) of the protocol consists of three
//! messages that are sent and received via the underlying transport:
//! - [`Segment message`](Segment)
//! - [`Retransmission request`](SegmentRequest)
//! - [`Frame acknowledgement`](FrameAcknowledgements)
//!
//! All of these messages are bundled within the [`SessionMessage`] enum,
//! which is then [encoded](SessionMessage::into_encoded) as a byte array of a maximum
//! MTU size `C` (which is a generic const argument of the `SessionMessage` type).
//! The header of the `SessionMessage` encoding consists of the [`version`](SessionMessage::VERSION)
//! byte, followed by the discriminator byte of one of the above messages and then followed by
//! the message length and message's encoding itself.
//!
//! Multiple [`SessionMessages`](SessionMessage) can be read from a binary blob using the
//! [`SessionMessageIter`].
//!
//! ## Segment message ([`Segment`](SessionMessage::Segment))
//! The Segment message contains the payload [`Segment`] of some [`Frame`](crate::session::Frame).
//! The size of this message can range from [`the minimum message size`](SessionMessage::minimum_message_size)
//! up to `C`.
//!
//! ## Retransmission request message ([`Request`](SessionMessage::Request))
//! Contains a request for retransmission of missing segments in a frame. This is sent from
//! the segment recipient to the sender, once it realizes some of the received frames are incomplete
//! (after a certain period of time).
//!
//! The encoding of this message consists of pairs of [frame ID](FrameId) and
//! a single byte bitmap of requested segments in this frame.
//! Each pair is therefore [`ENTRY_SIZE`](SegmentRequest::ENTRY_SIZE) bytes long.
//! There can be at most [`MAX_ENTRIES`](SegmentRequest::MAX_ENTRIES)
//! in a single Retransmission request message, given `C` as the MTU size. If the message contains
//! fewer entries, it is padded with zeros (0 is not a valid frame ID).
//! If more frames have missing segments, multiple retransmission request messages need to be sent.
//!
//! ## Frame acknowledgement message ([`Acknowledge`](SessionMessage::Acknowledge))
//! This message is sent from the segment recipient to the segment sender, to acknowledge that
//! all segments of certain frames have been completely and correctly received by the recipient.
//!
//! The message consists simply of a [frame ID](FrameId) list of the completely received
//! frames. There can be at most [`MAX_ACK_FRAMES`](FrameAcknowledgements::MAX_ACK_FRAMES)
//! per message. If more frames need to be acknowledged, more messages need to be sent.
//! If the message contains fewer entries, it is padded with zeros (0 is not a valid frame ID).

use std::collections::{BTreeMap, BTreeSet};

use bitvec::{BitArr, field::BitField, order::Lsb0};

use crate::session::{
    errors::SessionError,
    frames::{FrameId, SegmentId, SeqNum},
    protocol::SessionMessage,
};

/// Holds the Segment Retransmission Request message.
///
/// That is an ordered map of frame IDs and a bitmap of missing segments in each frame.
/// The bitmap can cover up a request for up to [`SegmentRequest::MAX_ENTRIES`] segments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SegmentRequest<const C: usize>(pub(super) BTreeMap<FrameId, SeqNum>);

impl<const C: usize> SegmentRequest<C> {
    /// Size of a single segment retransmission request entry.
    pub const ENTRY_SIZE: usize = size_of::<FrameId>() + size_of::<SeqNum>();
    /// Maximum number of segment retransmission entries.
    pub const MAX_ENTRIES: usize = Self::SIZE / Self::ENTRY_SIZE;
    /// Maximum number of missing segments per frame.
    pub const MAX_MISSING_SEGMENTS_PER_FRAME: usize = SeqNum::BITS as usize;
    pub const SIZE: usize = C - SessionMessage::<C>::HEADER_SIZE;

    /// Returns the number of segments to retransmit.
    pub fn len(&self) -> usize {
        self.0
            .values()
            .take(Self::MAX_ENTRIES)
            .map(|e| e.count_ones() as usize)
            .sum()
    }

    /// Returns true if there are no segments to retransmit in this request.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Iterator over [`SegmentId`] in [`SegmentRequest`].
pub struct SegmentIdIter(Vec<SegmentId>);

impl Iterator for SegmentIdIter {
    type Item = SegmentId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<const C: usize> IntoIterator for SegmentRequest<C> {
    type IntoIter = SegmentIdIter;
    type Item = SegmentId;

    fn into_iter(self) -> Self::IntoIter {
        let seq_size = SeqNum::BITS as usize;
        let mut ret = SegmentIdIter(Vec::with_capacity(seq_size * 8 * self.0.len()));
        for (frame_id, missing) in self.0 {
            for i in (0..seq_size).rev() {
                let mask = (1 << i) as SeqNum;
                if (mask & missing) != 0 {
                    ret.0.push(SegmentId(frame_id, i as SeqNum));
                }
            }
        }
        ret.0.shrink_to_fit();
        ret
    }
}

// From FrameIds and bitmap of missing segments per frame
impl<const C: usize> FromIterator<(FrameId, BitArr!(for 1, in u8, Lsb0))> for SegmentRequest<C> {
    fn from_iter<T: IntoIterator<Item = (FrameId, BitArr!(for 1, in u8, Lsb0))>>(iter: T) -> Self {
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
            if offset + size_of::<FrameId>() + size_of::<SeqNum>() < C {
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

    /// Creates a vector of [`FrameAcknowledgements`] from the given iterator
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

impl<const C: usize> From<Vec<FrameId>> for FrameAcknowledgements<C> {
    fn from(value: Vec<FrameId>) -> Self {
        Self(
            value
                .into_iter()
                .take(Self::MAX_ACK_FRAMES)
                .filter(|v| *v > 0)
                .collect(),
        )
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
}
