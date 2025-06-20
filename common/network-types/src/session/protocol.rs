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
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt::{Display, Formatter},
    mem,
};

use bitvec::field::BitField;

use crate::{
    errors::NetworkTypeError,
    session::{
        errors::SessionError,
        frame::{FrameId, FrameInfo, MissingSegmentsBitmap, Segment, SegmentId, SeqNum},
    },
};

/// Holds the Segment Retransmission Request message.
/// That is an ordered map of frame IDs and a bitmap of missing segments in each frame.
/// The bitmap can cover up a request for up to [`SegmentRequest::MAX_ENTRIES`] segments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SegmentRequest<const C: usize>(BTreeMap<FrameId, SeqNum>);

impl<const C: usize> SegmentRequest<C> {
    /// Size of a single segment retransmission request entry.
    pub const ENTRY_SIZE: usize = mem::size_of::<FrameId>() + mem::size_of::<SeqNum>();
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

impl<const C: usize> IntoIterator for SegmentRequest<C> {
    type IntoIter = std::vec::IntoIter<SegmentId>;
    type Item = SegmentId;

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

impl<const C: usize> FromIterator<FrameInfo> for SegmentRequest<C> {
    fn from_iter<T: IntoIterator<Item = FrameInfo>>(iter: T) -> Self {
        let mut ret = Self::default();
        for frame in iter.into_iter().take(Self::MAX_ENTRIES) {
            let frame_id = frame.frame_id;
            ret.0.insert(frame_id, frame.missing_segments.load());
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
/// This carries an ordered set of up to [`FrameAcknowledgements::MAX_ACK_FRAMES`] [frame IDs](FrameId) that has
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
///
/// The maximum size of the Session sub-protocol message is given by `C`.
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
    /// This is currently the version byte, the size of [SessionMessageDiscriminants] representation
    /// and two bytes for the message length.
    pub const HEADER_SIZE: usize = 1 + mem::size_of::<SessionMessageDiscriminants>() + mem::size_of::<u16>();
    /// Maximum size of the Session protocol message.
    ///
    /// This is equal to the typical Ethernet MTU size minus [`Self::SEGMENT_OVERHEAD`].
    pub const MAX_MESSAGE_SIZE: usize = 1492 - Self::SEGMENT_OVERHEAD;
    /// Maximum number of segments per frame.
    pub const MAX_SEGMENTS_PER_FRAME: usize = SegmentRequest::<C>::MAX_MISSING_SEGMENTS_PER_FRAME;
    /// Size of the overhead that's added to the raw payload of each [`Segment`].
    ///
    /// This amounts to [`SessionMessage::HEADER_SIZE`] + [`Segment::HEADER_SIZE`].
    pub const SEGMENT_OVERHEAD: usize = Self::HEADER_SIZE + Segment::HEADER_SIZE;
    /// Current version of the protocol.
    pub const VERSION: u8 = 1;

    /// Returns the minimum size of a [SessionMessage].
    pub fn minimum_message_size() -> usize {
        // Make this a "const fn" once "min" is const fn too
        Self::HEADER_SIZE
            + Segment::MINIMUM_SIZE
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
        SessionMessageIter::from(value).try_next()
    }
}

impl<const C: usize> From<SessionMessage<C>> for Vec<u8> {
    fn from(value: SessionMessage<C>) -> Self {
        let disc = SessionMessageDiscriminants::from(&value) as u8;

        let msg = match value {
            SessionMessage::Segment(s) => Vec::from(s),
            SessionMessage::Request(r) => Vec::from(r),
            SessionMessage::Acknowledge(a) => Vec::from(a),
        };

        let msg_len = msg.len() as u16;

        let mut ret = Vec::with_capacity(SessionMessage::<C>::HEADER_SIZE + msg_len as usize);
        ret.push(SessionMessage::<C>::VERSION);
        ret.push(disc);
        ret.extend(msg_len.to_be_bytes());
        ret.extend(msg);
        ret
    }
}

/// Allows parsing of multiple [`SessionMessages`](SessionMessage)
/// from a borrowed or an owned binary chunk.
///
/// The iterator will yield [`SessionMessages`](SessionMessage) until all the messages from
/// the underlying data chunk are completely parsed or an error occurs.
///
/// In other words, it keeps yielding `Some(Ok(_))` until it yields either `None`
/// or `Some(Err(_))` immediately followed by `None`.
///
/// This iterator is [fused](std::iter::FusedIterator).
#[derive(Debug, Clone)]
pub struct SessionMessageIter<'a, const C: usize> {
    data: Cow<'a, [u8]>,
    offset: usize,
    last_err: Option<SessionError>,
}

impl<const C: usize> SessionMessageIter<'_, C> {
    /// Determines if there was an error reading the last message.
    ///
    /// If this function returns some error value, the iterator will not
    /// yield any more messages.
    pub fn last_error(&self) -> Option<&SessionError> {
        self.last_err.as_ref()
    }

    /// Check if this iterator can yield any more messages.
    ///
    /// Returns `true` only if a [prior error](SessionMessageIter::last_error) occurred or all useful bytes
    /// from the underlying chunk were consumed and all messages were parsed.
    pub fn is_done(&self) -> bool {
        self.last_err.is_some() || self.data.len() - self.offset < SessionMessage::<C>::minimum_message_size()
    }

    /// Attempts to parse the current message and moves the offset if successful.
    fn try_next(&mut self) -> Result<SessionMessage<C>, SessionError> {
        let mut offset = self.offset;

        // Protocol version
        if self.data[offset] != SessionMessage::<C>::VERSION {
            return Err(SessionError::WrongVersion);
        }
        offset += 1;

        // Message discriminant
        let disc = self.data[offset];
        offset += 1;

        // Message length
        let len = u16::from_be_bytes(
            self.data[offset..offset + mem::size_of::<u16>()]
                .try_into()
                .map_err(|_| SessionError::IncorrectMessageLength)?,
        ) as usize;
        offset += mem::size_of::<u16>();

        if len > SessionMessage::<C>::MAX_MESSAGE_SIZE {
            return Err(SessionError::IncorrectMessageLength);
        }

        // The upper 6 bits of the size are reserved for future use,
        // since MAX_MESSAGE_SIZE always fits within 10 bits (<= MAX_MESSAGE_SIZE = 1500)
        let reserved = len & 0b111111_0000000000;

        // In version 1 check that the reserved bits are all 0
        if reserved != 0 {
            return Err(SessionError::ParseError);
        }

        // Read the message
        let res = match SessionMessageDiscriminants::from_repr(disc).ok_or(SessionError::UnknownMessageTag)? {
            SessionMessageDiscriminants::Segment => {
                SessionMessage::Segment(self.data[offset..offset + len].try_into()?)
            }
            SessionMessageDiscriminants::Request => {
                SessionMessage::Request(self.data[offset..offset + len].try_into()?)
            }
            SessionMessageDiscriminants::Acknowledge => {
                SessionMessage::Acknowledge(self.data[offset..offset + len].try_into()?)
            }
        };

        // Move the internal offset only once the message has been fully parsed
        self.offset = offset + len;
        Ok(res)
    }
}

impl<'a, const C: usize, T: Into<Cow<'a, [u8]>>> From<T> for SessionMessageIter<'a, C> {
    fn from(value: T) -> Self {
        Self {
            data: value.into(),
            offset: 0,
            last_err: None,
        }
    }
}

impl<const C: usize> Iterator for SessionMessageIter<'_, C> {
    type Item = Result<SessionMessage<C>, NetworkTypeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_done() {
            self.try_next()
                .inspect_err(|e| self.last_err = Some(e.clone()))
                .map_err(NetworkTypeError::SessionProtocolError)
                .into()
        } else {
            None
        }
    }
}

impl<const C: usize> std::iter::FusedIterator for SessionMessageIter<'_, C> {}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use bitvec::bitarr;
    use hex_literal::hex;
    use hopr_platform::time::native::current_time;
    use rand::{Rng, prelude::IteratorRandom, thread_rng};

    use super::*;
    use crate::session::{
        Frame,
        frame::{MissingSegmentsBitmap, NO_MISSING_SEGMENTS},
    };

    pub const ALL_MISSING_SEGMENTS: MissingSegmentsBitmap =
        bitarr![SeqNum, bitvec::prelude::Msb0; 1; SeqNum::BITS as usize];

    #[test]
    fn ensure_session_protocol_version_1_values() {
        // All of these values are independent of C, so we can set C = 0
        assert_eq!(1, SessionMessage::<0>::VERSION);
        assert_eq!(4, SessionMessage::<0>::HEADER_SIZE);
        assert_eq!(10, SessionMessage::<0>::SEGMENT_OVERHEAD);
        assert_eq!(8, SessionMessage::<0>::MAX_SEGMENTS_PER_FRAME);

        const _: () = {
            assert!(SessionMessage::<0>::MAX_MESSAGE_SIZE < 2048);
        };
    }

    #[test]
    fn segment_request_should_be_constructible_from_frame_info() {
        let frames = (1..20)
            .map(|i| {
                let mut missing_segments = NO_MISSING_SEGMENTS;
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
            total_segments: 8,
            missing_segments: [0b10100001].into(),
            last_update: SystemTime::now(),
        };

        let msg_1 = SessionMessage::<466>::Request(SegmentRequest::from_iter(vec![frame_info]));
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        match msg_1 {
            SessionMessage::Request(r) => {
                let missing_segments = r.into_iter().collect::<Vec<_>>();
                let expected = vec![SegmentId(10, 0), SegmentId(10, 2), SegmentId(10, 7)];
                assert_eq!(expected, missing_segments);
            }
            _ => panic!("invalid type"),
        }

        Ok(())
    }

    #[test]
    fn session_message_ack_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let mut rng = thread_rng();
        let frame_ids: Vec<u32> = (0..500).map(|_| rng.r#gen()).collect();

        let msg_1 = SessionMessage::<466>::Acknowledge(frame_ids.into());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn session_message_segment_request_should_yield_correct_bitset_values() {
        let seg_req = SegmentRequest::<466>([(3, 0b01000001), (10, 0b00101000)].into());

        let mut iter = seg_req.into_iter();
        assert_eq!(iter.next(), Some(SegmentId(3, 1)));
        assert_eq!(iter.next(), Some(SegmentId(3, 7)));
        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 4)));
        assert_eq!(iter.next(), None);

        let mut frame_info = FrameInfo {
            frame_id: 10,
            missing_segments: NO_MISSING_SEGMENTS,
            total_segments: 10,
            last_update: current_time(),
        };
        frame_info.missing_segments.set(2, true);
        frame_info.missing_segments.set(4, true);

        let mut iter = frame_info.clone().into_missing_segments();

        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 4)));
        assert_eq!(iter.next(), None);

        let mut iter = SegmentRequest::<466>::from_iter(vec![frame_info]).into_iter();
        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn session_message_iter_should_be_empty_if_slice_has_no_messages() {
        const MTU: usize = 462;

        let mut iter = SessionMessageIter::<MTU>::from(Vec::<u8>::new());
        assert!(iter.next().is_none());
        assert!(iter.is_done());

        let mut iter = SessionMessageIter::<MTU>::from(&[0u8; 2]);
        assert!(iter.next().is_none());
        assert!(iter.is_done());
    }

    #[test]
    fn session_message_iter_should_deserialize_multiple_messages() -> anyhow::Result<()> {
        const MTU: usize = 462;

        let mut messages_1 = Frame {
            frame_id: 10,
            data: hopr_crypto_random::random_bytes::<1500>().into(),
        }
        .segment(MTU - SessionMessage::<MTU>::HEADER_SIZE - Segment::HEADER_SIZE)?
        .into_iter()
        .map(SessionMessage::<MTU>::Segment)
        .collect::<Vec<_>>();

        let frame_info = FrameInfo {
            frame_id: 10,
            total_segments: 255,
            missing_segments: ALL_MISSING_SEGMENTS,
            last_update: SystemTime::now(),
        };

        messages_1.push(SessionMessage::<MTU>::Request(SegmentRequest::from_iter(vec![
            frame_info,
        ])));

        let mut rng = thread_rng();
        let frame_ids: Vec<u32> = (0..100).map(|_| rng.r#gen()).collect();
        messages_1.push(SessionMessage::<MTU>::Acknowledge(frame_ids.into()));

        let iter = SessionMessageIter::<MTU>::from(
            messages_1
                .iter()
                .cloned()
                .flat_map(|m| m.into_encoded().into_vec())
                .chain(std::iter::repeat_n(0, 10))
                .collect::<Vec<u8>>(),
        );

        let messages_2 = iter.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(messages_1, messages_2);

        Ok(())
    }

    #[test]
    fn session_message_iter_should_not_contain_error_when_consuming_everything() -> anyhow::Result<()> {
        const MTU: usize = 462;

        let messages = Frame {
            frame_id: 10,
            data: hopr_crypto_random::random_bytes::<{ 3 * MTU }>().into(),
        }
        .segment(MTU - SessionMessage::<MTU>::HEADER_SIZE - Segment::HEADER_SIZE)?
        .into_iter()
        .map(SessionMessage::<MTU>::Segment)
        .collect::<Vec<_>>();

        assert_eq!(4, messages.len());

        let data = messages
            .iter()
            .cloned()
            .flat_map(|m| m.into_encoded().into_vec())
            .chain(std::iter::repeat_n(0u8, 10))
            .collect::<Vec<_>>();

        let mut iter = SessionMessageIter::<MTU>::from(data);
        assert!(matches!(iter.next(), Some(Ok(m)) if m == messages[0]));
        assert!(matches!(iter.next(), Some(Ok(m)) if m == messages[1]));
        assert!(matches!(iter.next(), Some(Ok(m)) if m == messages[2]));
        assert!(matches!(iter.next(), Some(Ok(m)) if m == messages[3]));

        assert!(iter.next().is_none());
        assert!(iter.last_error().is_none());
        assert!(iter.is_done());

        Ok(())
    }

    #[test]
    fn session_message_iter_should_not_yield_more_after_error() -> anyhow::Result<()> {
        const MTU: usize = 462;

        let messages = Frame {
            frame_id: 10,
            data: hopr_crypto_random::random_bytes::<{ 3 * MTU }>().into(),
        }
        .segment(MTU - SessionMessage::<MTU>::HEADER_SIZE - Segment::HEADER_SIZE)?
        .into_iter()
        .map(SessionMessage::<MTU>::Segment)
        .collect::<Vec<_>>();

        assert_eq!(4, messages.len());

        let data = messages
            .iter()
            .cloned()
            .enumerate()
            .flat_map(|(i, m)| {
                if i == 2 {
                    Vec::from(hopr_crypto_random::random_bytes::<MTU>())
                } else {
                    m.into_encoded().into_vec()
                }
            })
            .collect::<Vec<_>>();

        let mut iter = SessionMessageIter::<MTU>::from(data);
        assert!(matches!(iter.next(), Some(Ok(m)) if m == messages[0]));
        assert!(matches!(iter.next(), Some(Ok(m)) if m == messages[1]));

        let err = iter.next();
        assert!(matches!(err, Some(Err(_))));
        assert!(iter.is_done());
        assert!(iter.last_error().is_some());

        assert!(iter.next().is_none());

        Ok(())
    }
}
