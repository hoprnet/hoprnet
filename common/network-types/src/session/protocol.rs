use fixedbitset::FixedBitSet;
use hopr_primitive_types::prelude::BytesEncodable;
use std::mem;

use crate::session::errors::SessionError;
use crate::frame::{FrameId, FrameInfo, Segment};

// TODO: get this from another crate?
const PACKET_SIZE: usize = 500;

// TODO: should include pub key?
const SESSION_MSG_SIZE: usize = PACKET_SIZE - 2 - 32 - Segment::HEADER_SIZE - 2; // 500 - 44 = 456

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentRequest {
    pub frame_id: FrameId,
    pub missing_segments: FixedBitSet,
}

const BITMAP_SIZE: usize = SESSION_MSG_SIZE - mem::size_of::<FrameId>();

impl From<FrameInfo> for SegmentRequest {
    fn from(value: FrameInfo) -> Self {
        Self {
            frame_id: value.frame_id,
            missing_segments: FixedBitSet::with_capacity_and_blocks(
                BITMAP_SIZE * 8,
                value
                    .missing_segments
                    .into_ones()
                    .take(BITMAP_SIZE / mem::size_of::<fixedbitset::Block>()),
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
                    BITMAP_SIZE * 8,
                    value[mem::size_of::<FrameId>()..]
                        .chunks(mem::size_of::<fixedbitset::Block>())
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
                .into_ones()
                .flat_map(|b| b.to_be_bytes())
                .collect::<Vec<_>>(),
        );
        ret
    }
}

const MAX_ACKS_NUM: usize = SESSION_MSG_SIZE / mem::size_of::<FrameId>();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameAcknowledgements([FrameId; MAX_ACKS_NUM]);

impl From<Vec<FrameId>> for FrameAcknowledgements {
    fn from(value: Vec<FrameId>) -> Self {
        let mut inner = [FrameId::default(); MAX_ACKS_NUM];
        inner[..value.len()].copy_from_slice(&value);
        Self(inner)
    }
}

impl IntoIterator for FrameAcknowledgements {
    type Item = FrameId;
    type IntoIter = std::array::IntoIter<Self::Item, {SESSION_MSG_SIZE / mem::size_of::<FrameId>()}>;

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
                    .collect::<Vec<_>>()
                    .try_into()
                    .map_err(|_| SessionError::ParseError)?,
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
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::FromRepr), repr(u16))]
pub enum SessionMessage {
    /// Represents a message containing a segment.
    Segment(Segment),
    /// Represents a message containing a request for segments.
    Request(SegmentRequest),
    /// Represents a message containing frame acknowledgements.
    Acknowledge(FrameAcknowledgements),
}

impl TryFrom<&[u8]> for SessionMessage {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < SESSION_MSG_SIZE {
            return Err(SessionError::ParseError);
        }

        let disc = u16::from_be_bytes(
            (&value[0..2])
                .try_into()
                .map_err(|_| SessionError::ParseError)?,
        );

        match SessionMessageDiscriminants::from_repr(disc).ok_or(SessionError::UnknownMessageTag)? {
            SessionMessageDiscriminants::Segment => Ok(SessionMessage::Segment((&value[2..]).try_into().map_err(|_| SessionError::ParseError)?)),
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
