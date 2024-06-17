use hopr_primitive_types::prelude::BytesEncodable;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use crate::frame::{FrameId, FrameInfo, Segment, SegmentId, SeqNum};
use crate::session::errors::SessionError;

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
    pub const MAX_ENTRIES: usize = C / Self::ENTRY_SIZE;

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

impl<const C: usize> BytesEncodable<C, SessionError> for SegmentRequest<C> {}

impl<const C: usize> TryFrom<&[u8]> for SegmentRequest<C> {
    type Error = SessionError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == C {
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

impl<const C: usize> From<SegmentRequest<C>> for [u8; C] {
    fn from(value: SegmentRequest<C>) -> Self {
        let mut ret = [0u8; C];
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
    pub const MAX_ACK_FRAMES: usize = (C - SessionMessage::<C>::HEADER_SIZE) / mem::size_of::<FrameId>();

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

impl<const C: usize> BytesEncodable<C, SessionError> for FrameAcknowledgements<C> {}

impl<'a, const C: usize> TryFrom<&'a [u8]> for FrameAcknowledgements<C> {
    type Error = SessionError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == C {
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

impl<const C: usize> From<FrameAcknowledgements<C>> for [u8; C] {
    fn from(value: FrameAcknowledgements<C>) -> Self {
        value
            .0
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .chain(std::iter::repeat(0_u8))
            .take(C)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
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
        if value.len() < Self::minimum_message_size()
        /*|| value.len() > C*/
        {
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
            SessionMessage::Request(b) => ret.extend(b.into_encoded()),
            SessionMessage::Acknowledge(f) => ret.extend(f.into_encoded()),
        };

        ret.shrink_to_fit();
        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{Frame, FrameInfo, SegmentId};
    use crate::session::protocol::{SegmentRequest, SessionMessage};
    use bitvec::bitarr;
    use hex_literal::hex;
    use hopr_platform::time::native::current_time;
    use rand::{thread_rng, Rng};
    use std::time::SystemTime;

    #[test]
    fn test_session_message_segment() {
        let mut segments = Frame {
            frame_id: 10,
            data: hex!("deadbeefcafebabe").into(),
        }
        .segment(8);

        let msg_1 = SessionMessage::<10>::Segment(segments.pop().unwrap());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..]).unwrap();

        assert_eq!(msg_1, msg_2);
    }

    #[test]
    fn test_session_message_segment_req() {
        let frame_info = FrameInfo {
            frame_id: 10,
            total_segments: 255,
            missing_segments: bitarr![1; 256],
            last_update: SystemTime::now(),
        };

        let msg_1 = SessionMessage::<466>::Request(SegmentRequest::from_iter(vec![frame_info]));
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..]).unwrap();

        assert_eq!(msg_1, msg_2);

        match msg_1 {
            SessionMessage::Request(r) => {
                let missing_segments = r.into_iter().collect::<Vec<_>>();
                let expected = (0..=7).map(|s| SegmentId(10, s)).collect::<Vec<_>>();
                assert_eq!(expected, missing_segments);
            }
            _ => panic!("invalid type"),
        }
    }

    #[test]
    fn test_session_message_frame_ack() {
        let mut rng = thread_rng();
        let frame_ids: Vec<u32> = (0..500).map(|_| rng.gen()).collect();

        let msg_1 = SessionMessage::<466>::Acknowledge(frame_ids.into());
        let data = Vec::from(msg_1.clone());
        let msg_2 = SessionMessage::try_from(&data[..]).unwrap();

        assert_eq!(msg_1, msg_2);
    }

    #[test]
    fn session_message_segment_request_bitset_test() {
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
