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
//! the message's encoding itself.
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
//!
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::mem;

use crate::session::errors::SessionError;
use crate::session::frame::{FrameId, FrameInfo, Segment, SegmentId, SeqNum};

/// Holds the Segment Retransmission Request message.
/// That is an ordered map of frame IDs and a bitmap of missing segments in each frame.
/// The bitmap can cover up a request for up to [`SegmentRequest::MAX_ENTRIES`] segments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SegmentRequest<const C: usize>(BTreeMap<FrameId, SeqNum>);

impl<const C: usize> SegmentRequest<C> {
    /// Size of a single segment retransmission request entry.
    pub const ENTRY_SIZE: usize = mem::size_of::<FrameId>() + mem::size_of::<SeqNum>();

    /// Maximum number of missing segments per frame.
    pub const MAX_MISSING_SEGMENTS_PER_FRAME: usize = mem::size_of::<SeqNum>() * 8;

    /// Maximum number of segment retransmission entries.
    pub const MAX_ENTRIES: usize = Self::SIZE / Self::ENTRY_SIZE;

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
    type Item = SegmentId;
    type IntoIter = SegmentIdIter;

    fn into_iter(self) -> Self::IntoIter {
        let seq_size = mem::size_of::<SeqNum>() * 8;
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

impl<const C: usize> FromIterator<FrameInfo> for SegmentRequest<C> {
    fn from_iter<T: IntoIterator<Item = FrameInfo>>(iter: T) -> Self {
        let mut ret = Self::default();
        for frame in iter.into_iter().take(Self::MAX_ENTRIES) {
            let frame_id = frame.frame_id;
            let missing = frame
                .iter_missing_sequence_indices()
                .filter(|s| *s < Self::MAX_MISSING_SEGMENTS_PER_FRAME as SeqNum)
                .map(|idx| 1 << idx)
                .fold(SeqNum::default(), |acc, n| acc | n);
            ret.0.insert(frame_id, missing);
        }
        ret
    }
}

impl<const C: usize> TryFrom<&[u8]> for SegmentRequest<C> {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            let mut ret = Self::default();
            for (frame_id, missing) in value
                .chunks_exact(Self::ENTRY_SIZE)
                .map(|c| c.split_at(mem::size_of::<FrameId>()))
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
            if offset + mem::size_of::<FrameId>() + mem::size_of::<SeqNum>() < C {
                ret[offset..offset + mem::size_of::<FrameId>()].copy_from_slice(&frame_id.to_be_bytes());
                offset += mem::size_of::<FrameId>();
                ret[offset..offset + mem::size_of::<SeqNum>()].copy_from_slice(&seq_num.to_be_bytes());
                offset += mem::size_of::<SeqNum>();
            } else {
                break;
            }
        }
        ret
    }
}

/// Holds the Frame Acknowledgement message.
/// This carries an ordered set of up to [`FrameAcknowledgements::MAX_ACK_FRAMES`] [frame IDs](FrameId) that have
/// been acknowledged by the counterparty.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameAcknowledgements<const C: usize>(BTreeSet<FrameId>);

impl<const C: usize> FrameAcknowledgements<C> {
    /// Maximum number of [frame IDs](FrameId) that can be accommodated.
    pub const MAX_ACK_FRAMES: usize = Self::SIZE / mem::size_of::<FrameId>();

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
    type Item = FrameId;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

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
                    .chunks_exact(mem::size_of::<FrameId>())
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

/// Contains all messages of the Session sub-protocol.
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants, strum::EnumTryAs)]
#[strum_discriminants(derive(strum::FromRepr), repr(u8))]
pub enum SessionMessage<const C: usize> {
    /// Represents a message containing a segment.
    Segment(Segment),
    /// Represents a message containing a [request](SegmentRequest) for segments.
    Request(SegmentRequest<C>),
    /// Represents a message containing [frame acknowledgements](FrameAcknowledgements).
    Acknowledge(FrameAcknowledgements<C>),
}

impl<const C: usize> Display for SessionMessage<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            SessionMessage::Segment(s) => write!(f, "segment {}", s.id()),
            SessionMessage::Request(r) => write!(f, "retransmission request of {:?}", r.0),
            SessionMessage::Acknowledge(a) => write!(f, "acknowledgement of {:?}", a.0),
        }
    }
}

impl<const C: usize> SessionMessage<C> {
    /// Header size of the session message.
    /// This is currently the version byte and the size of [SessionMessageDiscriminants] representation.
    pub const HEADER_SIZE: usize = 1 + mem::size_of::<SessionMessageDiscriminants>();

    /// Current version of the protocol.
    pub const VERSION: u8 = 0;

    /// Maximum number of segments per frame.
    pub const MAX_SEGMENTS_PER_FRAME: usize = SegmentRequest::<C>::MAX_MISSING_SEGMENTS_PER_FRAME;

    /// Returns the minimum size of a [SessionMessage].
    pub fn minimum_message_size() -> usize {
        // Make this a "const fn" once "min" is const fn too
        (Self::HEADER_SIZE + Segment::HEADER_SIZE + 1)
            .min(SegmentRequest::<C>::SIZE)
            .min(FrameAcknowledgements::<C>::SIZE)
    }

    /// Convenience method to encode the session message.
    pub fn into_encoded(self) -> Box<[u8]> {
        Vec::from(self).into_boxed_slice()
    }
}

impl<const C: usize> TryFrom<&[u8]> for SessionMessage<C> {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::minimum_message_size() || value.len() > C {
            return Err(SessionError::IncorrectMessageLength);
        }

        let version = value[0];
        if version != Self::VERSION {
            return Err(SessionError::WrongVersion);
        }

        match SessionMessageDiscriminants::from_repr(value[1]).ok_or(SessionError::UnknownMessageTag)? {
            SessionMessageDiscriminants::Segment => Ok(SessionMessage::Segment(
                (&value[2..]).try_into().map_err(|_| SessionError::ParseError)?,
            )),
            SessionMessageDiscriminants::Request => Ok(SessionMessage::Request((&value[2..]).try_into()?)),
            SessionMessageDiscriminants::Acknowledge => Ok(SessionMessage::Acknowledge((&value[2..]).try_into()?)),
        }
    }
}

impl<const C: usize> From<SessionMessage<C>> for Vec<u8> {
    fn from(value: SessionMessage<C>) -> Self {
        let mut ret = Vec::with_capacity(C);
        ret.push(SessionMessage::<C>::VERSION);
        ret.push(SessionMessageDiscriminants::from(&value) as u8);

        match value {
            SessionMessage::Segment(s) => ret.extend(Vec::from(s)),
            SessionMessage::Request(b) => ret.extend(Vec::from(b)),
            SessionMessage::Acknowledge(f) => ret.extend(Vec::from(f)),
        };

        ret.shrink_to_fit();
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Frame;
    use bitvec::array::BitArray;
    use bitvec::bitarr;
    use hex_literal::hex;
    use hopr_platform::time::native::current_time;
    use rand::prelude::IteratorRandom;
    use rand::{thread_rng, Rng};
    use std::time::SystemTime;

    #[test]
    fn segment_request_should_be_constructible_from_frame_info() {
        let frames = (1..20)
            .map(|i| {
                let mut missing_segments = BitArray::ZERO;
                (0..7_usize)
                    .choose_multiple(&mut thread_rng(), 4)
                    .into_iter()
                    .for_each(|i| missing_segments.set(i, true));
                FrameInfo {
                    frame_id: i,
                    missing_segments,
                    total_segments: 8,
                    last_update: SystemTime::UNIX_EPOCH,
                }
            })
            .collect::<Vec<_>>();

        let mut req = SegmentRequest::<466>::from_iter(frames.clone())
            .into_iter()
            .collect::<Vec<_>>();
        req.sort();

        assert_eq!(frames.len() * 4, req.len());
        assert_eq!(
            req,
            frames
                .into_iter()
                .flat_map(|f| f.into_missing_segments())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn session_message_segment_should_serialize_and_deserialize() -> anyhow::Result<()> {
        const SEG_SIZE: usize = 8;

        let mut segments = Frame {
            frame_id: 10,
            data: hex!("deadbeefcafebabe").into(),
        }
        .segment(SEG_SIZE)?;

        const MTU: usize = SEG_SIZE + Segment::HEADER_SIZE + 2;

        let msg_1 = SessionMessage::<MTU>::Segment(segments.pop().unwrap());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn session_message_segment_request_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let frame_info = FrameInfo {
            frame_id: 10,
            total_segments: 255,
            missing_segments: bitarr![1; 256],
            last_update: SystemTime::now(),
        };

        let msg_1 = SessionMessage::<466>::Request(SegmentRequest::from_iter(vec![frame_info]));
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        match msg_1 {
            SessionMessage::Request(r) => {
                let missing_segments = r.into_iter().collect::<Vec<_>>();
                let expected = (0..=7).map(|s| SegmentId(10, s)).collect::<Vec<_>>();
                assert_eq!(expected, missing_segments);
            }
            _ => panic!("invalid type"),
        }

        Ok(())
    }

    #[test]
    fn session_message_ack_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let mut rng = thread_rng();
        let frame_ids: Vec<u32> = (0..500).map(|_| rng.gen()).collect();

        let msg_1 = SessionMessage::<466>::Acknowledge(frame_ids.into());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn session_message_segment_request_should_yield_correct_bitset_values() {
        let seg_req = SegmentRequest::<466>([(10, 0b00100100)].into());

        let mut iter = seg_req.into_iter();
        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 5)));
        assert_eq!(iter.next(), None);

        let mut frame_info = FrameInfo {
            frame_id: 10,
            missing_segments: bitarr![0; 256],
            total_segments: 10,
            last_update: current_time(),
        };
        frame_info.missing_segments.set(2, true);
        frame_info.missing_segments.set(5, true);

        let mut iter = SegmentRequest::<466>::from_iter(vec![frame_info]).into_iter();
        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 5)));
        assert_eq!(iter.next(), None);
    }
}
