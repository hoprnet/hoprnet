//! Contains implementation of a `Session` message protocol.
//!
//! # What is `Session` protocol?
//! `Session` protocol is a simple protocol for unreliable networks that implements
//! basic TCP-like features, such as segmentation, retransmission and acknowledgement.
//!
//! The goal of this protocol is to establish a one-directional session between two parties,
//! where one is a message sender and the other one is the receiver. The messages are called
//! *frames* which are split and are delivered as *segments* from the sender to the recipient.
//! The session has some reliability guarantees given by the retransmission and acknowledgement
//! capabilities of individual segments.
//!
//! # `Session` protocol messages
//! The protocol components are built via low-level types of the [frame] module, such as
//! [`Segment`](frame::Segment) and [`Frame`](frame::Frame).
//! Most importantly, the `Session` protocol fixes the maximum number of segments per frame
//! to 8. Since each segment must fit within a maximum transmission unit (MTU),
//! a frame can be at most *eight* times the size of the MTU.
//!
//! The [current version](protocol::SessionMessage::VERSION) of the protocol consists of three
//! messages that are sent and received via the underlying transport:
//! - [`Segment message`](frame::Segment)
//! - [`Retransmission request`](protocol::SegmentRequest)
//! - [`Frame acknowledgement`](protocol::FrameAcknowledgements)
//!
//! All of these messages are bundled within the [`SessionMessage`](protocol::SessionMessage) enum,
//! which is then [encoded](protocol::SessionMessage::into_encoded) as a byte array of a maximum
//! MTU size `C` (which is a generic const argument of the `SessionMessage` type).
//! The header of the `SessionMessage` encoding consists of the [`version`](protocol::SessionMessage::VERSION)
//! byte, followed by the discriminator byte of one of the above messages and then followed by
//! the message's encoding itself.
//!
//! ## Segment message ([`Segment`](protocol::SessionMessage::Segment))
//! The Segment message contains the payload [`Segment`](frame::Segment) of some [`Frame`](frame::Frame).
//! The size of this message can range from [`the minimum message size`](protocol::SessionMessage::minimum_message_size)
//! up to `C`.
//!
//! ## Retransmission request message ([`Request`](protocol::SessionMessage::Request))
//! Contains a request for retransmission of missing segments in a frame. This is sent from
//! the segment recipient to the sender, once it realizes some of the received frames are incomplete
//! (after a certain period of time).
//!
//! The encoding of this message consists of pairs of [frame ID](frame::FrameId) and
//! a single byte bitmap of requested segments in this frame.
//! Each pair is therefore [`ENTRY_SIZE`](protocol::SegmentRequest::ENTRY_SIZE) bytes long.
//! There can be at most [`MAX_ENTRIES`](protocol::SegmentRequest::MAX_ENTRIES)
//! in a single Retransmission request message, given `C` as the MTU size. If the message contains
//! fewer entries, it is padded with zeros (0 is not a valid frame ID).
//! If more frames have missing segments, multiple retransmission request messages need to be sent.
//!
//! ## Frame acknowledgement message ([`Acknowledge`](protocol::SessionMessage::Acknowledge))
//! This message is sent from the segment recipient to the segment sender, to acknowledge that
//! all segments of certain frames have been completely and correctly received by the recipient.
//!
//! The message consists simply of a list of [frame IDs](frame::FrameId) of the completely received
//! frames. There can be at most [`MAX_ACK_FRAMES`](protocol::FrameAcknowledgements::MAX_ACK_FRAMES)
//! per message. If more frames need to be acknowledged, more messages need to be sent.
//! If the message contains fewer entries, it is padded with zeros (0 is not a valid frame ID).

pub mod errors;
pub mod protocol;
pub mod state;
mod utils;
