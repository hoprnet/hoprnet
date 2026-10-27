//! # `Session` protocol messages
//!
//! The protocol components are built via low-level types of the `frame` module, such as
//! [`Segment`] and [`Frame`].
//! Most importantly, the `Session` protocol fixes the maximum number of segments per frame
//! to 8 (see `SessionMessage::MAX_SEGMENTS_PER_FRAME`).
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
//! which is then encoded as a byte array of a maximum
//! MTU size `C` (which is a generic const argument of the `SessionMessage` type).
//! The header of the `SessionMessage` encoding consists of the [`version`](SessionMessage::VERSION)
//! byte, followed by the discriminator byte of one of the above messages and then followed by
//! the message length and message's encoding itself.
//!
//! ## Segment message ([`Segment`](SessionMessage::Segment))
//!
//! The Segment message contains the payload [`Segment`] of some [`Frame`].
//! The size of this message can range from [`the minimum message size`](SessionMessage::minimum_message_size)
//! up to `C`.
//!
//! ## Retransmission request message ([`Request`](SessionMessage::Request))
//!
//! Contains a request for retransmission of missing segments in a frame. This is sent from
//! the segment recipient to the sender, once it realizes some of the received frames are incomplete
//! (after a certain period of time).
//!
//! The encoding of this message consists of pairs of [frame ID](FrameId) and
//! a single byte bitmap of requested segments in this frame.
//! Each pair is therefore [`ENTRY_SIZE`](SegmentRequest::ENTRY_SIZE) bytes-long.
//! There can be at most [`MAX_ENTRIES`](SegmentRequest::MAX_ENTRIES)
//! in a single Retransmission request message, given `C` as the MTU size. If the message contains
//! fewer entries, it is padded with zeros (0 is not a valid frame ID).
//! If more frames have missing segments, multiple retransmission request messages need to be sent.
//!
//! ## Frame acknowledgement message ([`Acknowledge`](SessionMessage::Acknowledge))
//!
//! This message is sent from the segment recipient to the segment sender to acknowledge that
//! all segments of certain frames have been completely and correctly received by the recipient.
//!
//! The message consists simply of a [frame ID](FrameId) list of the completely received
//! frames. There can be at most [`MAX_ACK_FRAMES`](FrameAcknowledgements::MAX_ACK_FRAMES)
//! per message. If more frames need to be acknowledged, more messages need to be sent.
//! If the message contains fewer entries, it is padded with zeros (0 is not a valid frame ID).

mod frames;
mod messages;

use asynchronous_codec::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
pub use frames::{Frame, FrameId, OrderedFrame, Segment, SegmentId, SeqIndicator, SeqNum};
pub use messages::{FrameAcknowledgements, MissingSegmentsBitmap, SegmentRequest};

use crate::errors::SessionError;

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
    /// This is currently the version byte, the size of [`SessionMessageDiscriminants`] representation
    /// and two bytes for the message length.
    pub const HEADER_SIZE: usize = 1 + size_of::<SessionMessageDiscriminants>() + size_of::<u16>();
    /// Maximum size of the message in v1.
    pub const MAX_MESSAGE_LENGTH: usize = C.saturating_sub(Self::HEADER_SIZE);
    /// Size of the overhead that's added to the raw payload of each [`Segment`].
    ///
    /// This amounts to [`SessionMessage::HEADER_SIZE`] + [`Segment::HEADER_SIZE`].
    pub const SEGMENT_OVERHEAD: usize = Self::HEADER_SIZE + Segment::HEADER_SIZE;
    /// Current version of the protocol.
    pub const VERSION: u8 = 1;

    /// Returns the minimum size of a [`SessionMessage`].
    pub fn minimum_message_size() -> usize {
        // Make this a "const fn" once "min" is const fn too
        Self::HEADER_SIZE
            + Segment::HEADER_SIZE
                .min(SegmentRequest::<C>::SIZE)
                .min(FrameAcknowledgements::<C>::SIZE)
    }

    /// Convenience method to encode the session message.
    pub fn into_encoded(self) -> Box<[u8]> {
        Vec::from(self).into_boxed_slice()
    }
}

impl<const C: usize> From<SessionMessage<C>> for Vec<u8> {
    fn from(message: SessionMessage<C>) -> Self {
        debug_assert!(
            C > SessionMessage::<C>::HEADER_SIZE && SessionMessage::<C>::MAX_MESSAGE_LENGTH <= u16::MAX as usize
        );

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
        debug_assert!(
            C > SessionMessage::<C>::HEADER_SIZE && SessionMessage::<C>::MAX_MESSAGE_LENGTH <= u16::MAX as usize
        );

        let disc = SessionMessageDiscriminants::from(&item) as u8;

        let msg = match item {
            SessionMessage::Segment(s) => Vec::from(s),
            SessionMessage::Request(r) => Vec::from(r),
            SessionMessage::Acknowledge(a) => Vec::from(a),
        };

        if msg.len() > SessionMessage::<C>::MAX_MESSAGE_LENGTH {
            return Err(SessionError::IncorrectMessageLength);
        }

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

    /// Decodes one message from `src`.
    ///
    /// On error, the bad bytes are always removed from `src`. The framed reader calls `decode`
    /// again on the same buffer after an error, so bytes left in place would fail again on
    /// every poll and the reader would never move on.
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        debug_assert!(C > SessionMessage::<C>::HEADER_SIZE);

        tracing::trace!(msg_len = src.len(), "decoding message");
        if src.len() < SessionMessage::<C>::minimum_message_size() {
            return Ok(None);
        }

        // Protocol version
        if src[0] != SessionMessage::<C>::VERSION {
            // The header cannot be trusted, so the message length is unknown: drop everything.
            src.clear();
            return Err(SessionError::WrongVersion);
        }

        // Message discriminant
        let disc = src[1];

        // Message length
        let payload_len = u16::from_be_bytes([src[2], src[3]]) as usize;

        // Check the maximum message length for version 1
        if payload_len > SessionMessage::<C>::MAX_MESSAGE_LENGTH {
            // The length cannot be trusted either: drop everything.
            src.clear();
            return Err(SessionError::IncorrectMessageLength);
        }

        // Check if there's enough data so that we can read the rest of the message
        let message_len = SessionMessage::<C>::HEADER_SIZE + payload_len;
        if src.len() < message_len {
            return Ok(None);
        }

        // Take the whole message out of the buffer first, so a message that fails to parse
        // below is skipped and the next one can still be read.
        let message = src.split_to(message_len);
        let payload = &message[SessionMessage::<C>::HEADER_SIZE..];

        let res = match SessionMessageDiscriminants::from_repr(disc).ok_or(SessionError::UnknownMessageTag)? {
            SessionMessageDiscriminants::Segment => SessionMessage::Segment(payload.try_into()?),
            SessionMessageDiscriminants::Request => SessionMessage::Request(payload.try_into()?),
            SessionMessageDiscriminants::Acknowledge => SessionMessage::Acknowledge(payload.try_into()?),
        };

        Ok(Some(res))
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_protocol_app::prelude::ApplicationData;
    use rand::{RngExt, rngs::ThreadRng};

    use super::*;
    use crate::{
        protocol::{FrameId, SegmentId},
        session_socket_mtu,
        utils::segment,
    };

    #[test]
    fn ensure_session_protocol_version_1_values() {
        // All of these values are independent of C, so we can set C = 0
        assert_eq!(1, SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::VERSION);
        assert_eq!(4, SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::HEADER_SIZE);
        assert_eq!(
            10,
            SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::SEGMENT_OVERHEAD
        );
        assert!(
            session_socket_mtu::<{ ApplicationData::PAYLOAD_SIZE }>()
                + SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::SEGMENT_OVERHEAD
                - SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::HEADER_SIZE
                <= SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::MAX_MESSAGE_LENGTH
        );
    }

    #[test]
    fn session_message_segment_should_serialize_and_deserialize() -> anyhow::Result<()> {
        const SEG_SIZE: usize = 8;

        let mut segments = segment(hex!("deadbeefcafebabe"), SEG_SIZE, 10)?;

        const MTU: usize = SEG_SIZE + SessionMessage::<0>::SEGMENT_OVERHEAD;

        let msg_1 = SessionMessage::<MTU>::Segment(segments.pop().unwrap());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn session_message_segment_request_should_serialize_and_deserialize() -> anyhow::Result<()> {
        // The first 8 segments are missing in Frame 10
        let msg_1 = SessionMessage::<466>::Request(SegmentRequest::from_iter([
            (2 as FrameId, [0b11000001].into()),
            (10 as FrameId, [0b01000100].into()),
        ]));
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        match msg_1 {
            SessionMessage::Request(r) => {
                let missing_segments = r.into_iter().collect::<Vec<_>>();
                let expected = vec![
                    SegmentId(2, 0),
                    SegmentId(2, 1),
                    SegmentId(2, 7),
                    SegmentId(10, 1),
                    SegmentId(10, 5),
                ];
                assert_eq!(expected, missing_segments);
            }
            _ => panic!("invalid type"),
        }

        Ok(())
    }

    #[test]
    fn session_message_ack_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let mut rng = ThreadRng::default();
        let frame_ids: Vec<u32> = (0..FrameAcknowledgements::<466>::MAX_ACK_FRAMES)
            .map(|_| rng.random())
            .collect();

        let msg_1 = SessionMessage::<466>::Acknowledge(frame_ids.try_into()?);
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..])?;

        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    /// Encodes a valid acknowledgement message, used after a bad message in the tests below.
    fn valid_ack<const C: usize>() -> (SessionMessage<C>, Vec<u8>) {
        let msg = SessionMessage::<C>::Acknowledge(vec![1 as FrameId, 2].try_into().expect("ack fits"));
        let data = Vec::from(msg.clone());
        (msg, data)
    }

    #[test]
    fn decode_should_skip_a_message_with_an_unknown_tag() -> anyhow::Result<()> {
        const MTU: usize = 466;
        let (valid, valid_data) = valid_ack::<MTU>();

        // Correct version and length, but a tag that does not exist
        let mut src = BytesMut::from(&[SessionMessage::<MTU>::VERSION, 0xff, 0, 2, 0xaa, 0xbb][..]);
        src.extend_from_slice(&valid_data);

        let mut codec = SessionCodec::<MTU>;
        assert!(matches!(codec.decode(&mut src), Err(SessionError::UnknownMessageTag)));
        assert_eq!(src.len(), valid_data.len(), "the bad message must be consumed");
        assert_eq!(codec.decode(&mut src)?, Some(valid));
        assert!(src.is_empty());

        Ok(())
    }

    #[test]
    fn decode_should_skip_a_message_that_cannot_be_parsed() -> anyhow::Result<()> {
        const MTU: usize = 466;
        let (valid, valid_data) = valid_ack::<MTU>();

        // A valid tag and length, but a segment payload that is too short to be a segment
        let disc = SessionMessageDiscriminants::Segment as u8;
        let mut src = BytesMut::from(&[SessionMessage::<MTU>::VERSION, disc, 0, 1, 0xaa][..]);
        src.extend_from_slice(&valid_data);

        let mut codec = SessionCodec::<MTU>;
        assert!(codec.decode(&mut src).is_err());
        assert_eq!(src.len(), valid_data.len(), "the bad message must be consumed");
        assert_eq!(codec.decode(&mut src)?, Some(valid));

        Ok(())
    }

    #[test]
    fn decode_should_drop_the_buffer_when_the_header_is_bad() {
        const MTU: usize = 466;
        let mut codec = SessionCodec::<MTU>;

        let mut wrong_version = BytesMut::from(&[0xee_u8; 16][..]);
        assert!(matches!(
            codec.decode(&mut wrong_version),
            Err(SessionError::WrongVersion)
        ));
        assert!(wrong_version.is_empty());

        // Header claims a payload longer than the protocol allows
        let mut too_long = BytesMut::from(&[SessionMessage::<MTU>::VERSION, 0, 0xff, 0xff][..]);
        too_long.extend_from_slice(&[0; 12]);
        assert!(matches!(
            codec.decode(&mut too_long),
            Err(SessionError::IncorrectMessageLength)
        ));
        assert!(too_long.is_empty());
    }

    #[test]
    fn framed_reader_should_move_on_after_a_bad_message() {
        use futures::StreamExt;

        const MTU: usize = 466;
        let (valid, valid_data) = valid_ack::<MTU>();

        let mut data = vec![SessionMessage::<MTU>::VERSION, 0xff, 0, 2, 0xaa, 0xbb];
        data.extend_from_slice(&valid_data);

        // Before the fix, the reader returned the same error forever; `take` keeps the test from hanging
        let items = futures::executor::block_on(
            asynchronous_codec::FramedRead::new(futures::io::Cursor::new(data), SessionCodec::<MTU>)
                .take(10)
                .collect::<Vec<_>>(),
        );

        assert_eq!(items.len(), 2, "expected one error, then the valid message");
        assert!(matches!(items[0], Err(SessionError::UnknownMessageTag)));
        assert_eq!(items[1].as_ref().ok(), Some(&valid));
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
