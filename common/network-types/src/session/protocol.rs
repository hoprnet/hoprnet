use fixedbitset::FixedBitSet;
use hopr_primitive_types::prelude::BytesEncodable;
use std::collections::BTreeSet;
use std::mem;

use crate::frame::{FrameId, FrameInfo, Segment, SegmentId};
use crate::session::errors::SessionError;

// TODO: get this from another crate?
const PACKET_SIZE: usize = 500;

// TODO: should include pub key?
const SESSION_MSG_SIZE: usize = PACKET_SIZE - 2 - 32 - Segment::HEADER_SIZE - 2; // 500 - 44 = 456

/// Holds the Segment Retransmission Request message.
/// That is an ID of a frame and a bitmap of missing segments in this frame.
/// The bitmap can cover up a request for up to [`SegmentRequest::MAX_ERROR_SEGMENTS`] segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentRequest {
    /// Frame ID that requires segment retransmission.
    pub frame_id: FrameId,
    /// Bitmap of segments missing from the frame.
    /// This can express up to [`SegmentRequest::MAX_ERROR_SEGMENTS`] segment positions.
    pub missing_segments: FixedBitSet,
}

/// Size of the block inside the bitmap.
const BITMAP_BLOCK_SIZE: usize = mem::size_of::<fixedbitset::Block>();
/// Size of the `missing_segments` bitmap in bytes, rounded to the multiple of its Block.
const BITMAP_SIZE: usize = ((SESSION_MSG_SIZE - mem::size_of::<FrameId>()) / BITMAP_BLOCK_SIZE) * BITMAP_BLOCK_SIZE;

impl SegmentRequest {
    /// Maximum number of segments that can be requested.
    /// This is calculated so that the request fits within a single segment itself.
    pub const MAX_ERROR_SEGMENTS: usize = BITMAP_SIZE * 8;

    /// Returns the number of missing segments.
    pub fn len(&self) -> usize {
        self.missing_segments.count_ones(..)
    }
}

pub struct SegmentIdIter(Vec<SegmentId>);

impl Iterator for SegmentIdIter {
    type Item = SegmentId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl IntoIterator for SegmentRequest {
    type Item = SegmentId;
    type IntoIter = SegmentIdIter;

    fn into_iter(self) -> Self::IntoIter {
        SegmentIdIter(
            self.missing_segments
                .into_ones()
                .map(|b| SegmentId(self.frame_id, b as u16))
                .rev()
                .collect(),
        )
    }
}

impl From<FrameInfo> for SegmentRequest {
    fn from(value: FrameInfo) -> Self {
        Self {
            frame_id: value.frame_id,
            missing_segments: FixedBitSet::with_capacity_and_blocks(
                Self::MAX_ERROR_SEGMENTS,
                value.missing_segments.as_slice().iter().copied(),
            ),
        }
    }
}

impl BytesEncodable<SESSION_MSG_SIZE, SessionError> for SegmentRequest {}

impl TryFrom<&[u8]> for SegmentRequest {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == SESSION_MSG_SIZE {
            Ok(Self {
                frame_id: FrameId::from_be_bytes(value[..mem::size_of::<FrameId>()].try_into().unwrap()),
                missing_segments: FixedBitSet::with_capacity_and_blocks(
                    Self::MAX_ERROR_SEGMENTS,
                    value[mem::size_of::<FrameId>()..]
                        .chunks(BITMAP_BLOCK_SIZE)
                        .map(|c| fixedbitset::Block::from_be_bytes(c.try_into().unwrap())),
                ),
            })
        } else {
            Err(SessionError::ParseError)
        }
    }
}

impl From<SegmentRequest> for [u8; SESSION_MSG_SIZE] {
    fn from(value: SegmentRequest) -> Self {
        let mut ret = [0u8; SESSION_MSG_SIZE];
        ret[0..mem::size_of::<FrameId>()].copy_from_slice(&value.frame_id.to_be_bytes());
        ret[mem::size_of::<FrameId>()..].copy_from_slice(
            &value
                .missing_segments
                .as_slice()
                .into_iter()
                .flat_map(|b| b.to_be_bytes())
                .chain(std::iter::repeat(0_u8))
                .take(SESSION_MSG_SIZE - mem::size_of::<FrameId>())
                .collect::<Vec<_>>(),
        );
        ret
    }
}

/// Holds the Frame Acknowledgement message.
/// This carries an ordered set of up to [`FrameAcknowledgements::MAX_ACK_FRAMES`] [frame IDs](FrameId) that have
/// been acknowledged by the counterparty.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameAcknowledgements(BTreeSet<FrameId>);

impl FrameAcknowledgements {
    /// Maximum number of [frame IDs](FrameId) that can be accommodated.
    pub const MAX_ACK_FRAMES: usize = SESSION_MSG_SIZE / mem::size_of::<FrameId>();

    /// Pushes the frame ID.
    /// Returns true if the value has been pushed or false it the container is full or already
    /// contains that value.
    pub fn push(&mut self, frame_id: FrameId) -> bool {
        if !self.is_full() {
            self.0.insert(frame_id)
        } else {
            false
        }
    }

    /// Number of acknowledged frame IDs in this instance.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Indicates whether the [maximum number of frame IDs](FrameAcknowledgements::MAX_ACK_FRAMES)
    /// has been reached.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.0.len() == Self::MAX_ACK_FRAMES
    }
}

impl From<Vec<FrameId>> for FrameAcknowledgements {
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

impl IntoIterator for FrameAcknowledgements {
    type Item = FrameId;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl BytesEncodable<SESSION_MSG_SIZE, SessionError> for FrameAcknowledgements {}

impl<'a> TryFrom<&'a [u8]> for FrameAcknowledgements {
    type Error = SessionError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == SESSION_MSG_SIZE {
            Ok(Self(
                value
                    .chunks(mem::size_of::<FrameId>())
                    .map(|v| FrameId::from_be_bytes(v.try_into().unwrap()))
                    .filter(|f| *f > 0)
                    .collect(),
            ))
        } else {
            Err(SessionError::ParseError)
        }
    }
}

impl From<FrameAcknowledgements> for [u8; SESSION_MSG_SIZE] {
    fn from(value: FrameAcknowledgements) -> Self {
        value
            .0
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .chain(std::iter::repeat(0_u8))
            .take(SESSION_MSG_SIZE)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

/// Contains all messages of the Session sub-protocol.
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants, strum::EnumTryAs)]
#[strum_discriminants(derive(strum::FromRepr), repr(u16))]
pub enum SessionMessage {
    /// Represents a message containing a segment.
    Segment(Segment),
    /// Represents a message containing a request for segments.
    Request(SegmentRequest),
    /// Represents a message containing frame acknowledgements.
    Acknowledge(FrameAcknowledgements),
}

impl SessionMessage {
    /// Header size of the session message.
    /// This is currently just the size of [SessionMessageDiscriminants] representation.
    pub const HEADER_SIZE: usize = mem::size_of::<SessionMessageDiscriminants>();

    /// Convenience method to encode the session message.
    pub fn into_encoded(self) -> Box<[u8]> {
        Vec::from(self).into_boxed_slice()
    }
}

impl TryFrom<&[u8]> for SessionMessage {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 2 + Segment::HEADER_SIZE {
            return Err(SessionError::ParseError);
        }

        let disc = u16::from_be_bytes((&value[0..2]).try_into().map_err(|_| SessionError::ParseError)?);

        match SessionMessageDiscriminants::from_repr(disc).ok_or(SessionError::UnknownMessageTag)? {
            SessionMessageDiscriminants::Segment => Ok(SessionMessage::Segment(
                (&value[2..]).try_into().map_err(|_| SessionError::ParseError)?,
            )),
            SessionMessageDiscriminants::Request => Ok(SessionMessage::Request((&value[2..]).try_into()?)),
            SessionMessageDiscriminants::Acknowledge => Ok(SessionMessage::Acknowledge((&value[2..]).try_into()?)),
        }
    }
}

impl From<SessionMessage> for Vec<u8> {
    fn from(value: SessionMessage) -> Self {
        let disc = SessionMessageDiscriminants::from(&value) as u16;

        let mut ret = Vec::with_capacity(500);
        ret.extend(disc.to_be_bytes());

        match value {
            SessionMessage::Segment(s) => ret.extend(Vec::from(s)),
            SessionMessage::Request(b) => ret.extend(b.into_encoded()),
            SessionMessage::Acknowledge(f) => ret.extend(f.into_encoded()),
        };

        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{Frame, FrameInfo, SegmentId};
    use crate::session::protocol::{SegmentRequest, SessionMessage};
    use fixedbitset::FixedBitSet;
    use hex_literal::hex;
    use hopr_platform::time::native::current_time;
    use rand::prelude::IteratorRandom;
    use rand::{thread_rng, Rng};
    use std::time::SystemTime;

    #[test]
    fn test_session_message_segment() {
        let mut segments = Frame {
            frame_id: 10,
            data: hex!("deadbeefcafebabe").into(),
        }
        .segment(8);

        let msg_1 = SessionMessage::Segment(segments.pop().unwrap());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..]).unwrap();

        assert_eq!(msg_1, msg_2);
    }

    #[test]
    fn test_session_message_segment_req() {
        let mut missing_segments = FixedBitSet::with_capacity(10_000);
        (0..10_000_usize)
            .choose_multiple(&mut thread_rng(), 2000)
            .into_iter()
            .for_each(|i| {
                missing_segments.put(i);
            });

        let frame_info = FrameInfo {
            frame_id: 10,
            total_segments: missing_segments.len() as u16,
            missing_segments,
            last_update: SystemTime::now(),
        };

        let msg_1 = SessionMessage::Request(frame_info.into());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..]).unwrap();

        assert_eq!(msg_1, msg_2);

        assert!(matches!(msg_1, SessionMessage::Request(r) if  r.missing_segments.len() < 10_000));
    }

    #[test]
    fn test_session_message_frame_ack() {
        let mut rng = thread_rng();
        let frame_ids: Vec<u32> = (0..500).map(|_| rng.gen()).collect();

        let msg_1 = SessionMessage::Acknowledge(frame_ids.into());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..]).unwrap();

        assert_eq!(msg_1, msg_2);
    }

    #[test]
    fn session_message_segment_request_bitset_test() {
        let mut seg_req = SegmentRequest {
            frame_id: 10,
            missing_segments: FixedBitSet::with_capacity(1024),
        };

        seg_req.missing_segments.insert(2);
        seg_req.missing_segments.insert(5);

        let mut iter = seg_req.into_iter();
        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 5)));
        assert_eq!(iter.next(), None);

        let mut frame_info = FrameInfo {
            frame_id: 10,
            missing_segments: FixedBitSet::with_capacity(1024),
            total_segments: 10,
            last_update: current_time(),
        };
        frame_info.missing_segments.insert(2);
        frame_info.missing_segments.insert(5);

        let mut iter = SegmentRequest::from(frame_info).into_iter();
        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 5)));
        assert_eq!(iter.next(), None);
    }
}
