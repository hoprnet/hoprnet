mod messages;

use asynchronous_codec::{Decoder, Encoder};
use bytes::{Buf, BufMut, BytesMut};

pub use messages::{FrameAcknowledgements, SegmentRequest, MissingSegmentsBitmap};

use crate::session::{errors::SessionError, frames::Segment};
use crate::session::frames::SeqIndicator;

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

impl<const C: usize> std::fmt::Display for SessionMessage<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    pub const HEADER_SIZE: usize = 1 + size_of::<SessionMessageDiscriminants>() + size_of::<u16>();
    /// Maximum size of the Session protocol message.
    ///
    /// This is equal to the typical Ethernet MTU size minus [`Self::SEGMENT_OVERHEAD`].
    // TODO: parameterize this
    pub const MAX_MESSAGE_SIZE: usize = 1492 - Self::SEGMENT_OVERHEAD;
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

    /// Maximum number of segments per frame.
    pub fn max_segments_per_frame() -> usize {
        SegmentRequest::<C>::MAX_MISSING_SEGMENTS_PER_FRAME.min(SeqIndicator::MAX as usize + 1)
    }

    /// Convenience method to encode the session message.
    pub fn into_encoded(self) -> Box<[u8]> {
        Vec::from(self).into_boxed_slice()
    }
}

impl<const C: usize> From<SessionMessage<C>> for Vec<u8> {
    fn from(message: SessionMessage<C>) -> Self {
        let mut result = BytesMut::new();
        SessionCodec::<C>
            .encode(message, &mut result)
            .expect("encoding never fails");

        result.to_vec()
    }
}

impl<const C: usize> TryFrom<&[u8]> for SessionMessage<C> {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        SessionCodec
            .decode(&mut BytesMut::from(value))?
            .ok_or(SessionError::IncorrectMessageLength)
    }
}

#[derive(Clone, Copy, Default)]
pub struct SessionCodec<const C: usize>;

impl<const C: usize> Encoder for SessionCodec<C> {
    type Error = SessionError;
    type Item<'a> = SessionMessage<C>;

    fn encode(&mut self, item: Self::Item<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let disc = SessionMessageDiscriminants::from(&item) as u8;

        let msg = match item {
            SessionMessage::Segment(s) => Vec::from(s),
            SessionMessage::Request(r) => Vec::from(r),
            SessionMessage::Acknowledge(a) => Vec::from(a),
        };

        let msg_len = msg.len() as u16;
        dst.put_u8(SessionMessage::<C>::VERSION);
        dst.put_u8(disc);
        dst.put_u16(msg_len);
        dst.extend_from_slice(&msg);

        tracing::trace!(disc, msg_len, "encoded message");
        Ok(())
    }
}

impl<const C: usize> Decoder for SessionCodec<C> {
    type Error = SessionError;
    type Item = SessionMessage<C>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        tracing::trace!(msg_len = src.len(), "decoding message");
        if src.len() < SessionMessage::<C>::minimum_message_size() {
            return Ok(None);
        }

        // Protocol version
        if src.get_u8() != SessionMessage::<C>::VERSION {
            return Err(SessionError::WrongVersion);
        }

        // Message discriminant
        let disc = src.get_u8();

        // Message length
        let len = src.get_u16() as usize;

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
            SessionMessageDiscriminants::Segment => SessionMessage::Segment(src[..len].try_into()?),
            SessionMessageDiscriminants::Request => SessionMessage::Request(src[..len].try_into()?),
            SessionMessageDiscriminants::Acknowledge => SessionMessage::Acknowledge(src[..len].try_into()?),
        };

        src.advance(len);
        Ok(Some(res))
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use rand::{Rng, thread_rng};

    use super::*;
    use crate::session::{
        frames::{FrameId, SegmentId},
    };
    use crate::session::utils::test::segment;

    #[test]
    fn ensure_session_protocol_version_1_values() {
        // All of these values are independent of C, so we can set C = 0
        assert_eq!(1, SessionMessage::<0>::VERSION);
        assert_eq!(4, SessionMessage::<0>::HEADER_SIZE);
        assert_eq!(10, SessionMessage::<0>::SEGMENT_OVERHEAD);
        assert_eq!(8, SessionMessage::<0>::max_segments_per_frame());

        const _: () = {
            assert!(SessionMessage::<0>::MAX_MESSAGE_SIZE < 2048);
        };
    }

    #[test]
    fn session_message_segment_should_serialize_and_deserialize() -> anyhow::Result<()> {
        const SEG_SIZE: usize = 8;

        let mut segments = segment(hex!("deadbeefcafebabe"), SEG_SIZE, 10)?;

        const MTU: usize = SEG_SIZE + Segment::HEADER_SIZE + 2;

        let msg_1 = SessionMessage::<MTU>::Segment(segments.pop().unwrap());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn session_message_segment_request_should_serialize_and_deserialize() -> anyhow::Result<()> {
        // The first 8 segments are missing in Frame 10
        let msg_1 =
            SessionMessage::<466>::Request(SegmentRequest::from_iter([
                (2 as FrameId,  [0b11000001].into()),
                (10 as FrameId, [0b01000100].into())
                ]
            ));
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        match msg_1 {
            SessionMessage::Request(r) => {
                let missing_segments = r.into_iter().collect::<Vec<_>>();
                let expected = vec![
                    SegmentId(2, 0), SegmentId(2, 1), SegmentId(2, 7),
                    SegmentId(10, 1), SegmentId(10, 5),
                ];
                assert_eq!(expected, missing_segments);
            }
            _ => panic!("invalid type"),
        }

        Ok(())
    }

    #[test]
    fn session_message_ack_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let mut rng = thread_rng();
        let frame_ids: Vec<u32> = (0..FrameAcknowledgements::<466>::MAX_ACK_FRAMES).map(|_| rng.gen()).collect();

        let msg_1 = SessionMessage::<466>::Acknowledge(frame_ids.try_into()?);
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn session_message_segment_request_should_yield_correct_bitset_values() {
        let seg_req = SegmentRequest::<466>::from_iter([(10, MissingSegmentsBitmap::from([0b00101000]))]);

        let mut iter = seg_req.into_iter();
        assert_eq!(iter.next(), Some(SegmentId(10, 2)));
        assert_eq!(iter.next(), Some(SegmentId(10, 4)));
        assert_eq!(iter.next(), None);
    }
}
