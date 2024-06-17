//! Contains implementation of a `Session` message protocol.
//!
//! # What is `Session` protocol?
//! `Session` protocol is a simple protocol for unreliable networks that implements
//! basic TCP-like features, such as segmentation, retransmission and acknowledgement.
//!
//! The goal of this protocol is to establish a read-write session between two parties,
//! where one is a message sender and the other one is the receiver. The messages are called
//! *frames* which are split and are delivered as *segments* from the sender to the recipient.
//! The session has some reliability guarantees given by the retransmission and acknowledgement
//! capabilities of individual segments.
//!
//! # `Session` protocol messages
//! The protocol components are built via low-level types of the [`frame`](crate::frame) module, such as
//! [`Segment`](crate::frame::Segment) and [`Frame`](crate::frame::Frame).
//! Most importantly, the `Session` protocol fixes the maximum number of segments per frame
//! to 8 (see [`MAX_SEGMENTS_PER_FRAME`](protocol::SessionMessage::MAX_SEGMENTS_PER_FRAME)).
//! Since each segment must fit within a maximum transmission unit (MTU),
//! a frame can be at most *eight* times the size of the MTU.
//!
//! The [current version](protocol::SessionMessage::VERSION) of the protocol consists of three
//! messages that are sent and received via the underlying transport:
//! - [`Segment message`](crate::frame::Segment)
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
//! The Segment message contains the payload [`Segment`](crate::frame::Segment) of some [`Frame`](crate::frame::Frame).
//! The size of this message can range from [`the minimum message size`](protocol::SessionMessage::minimum_message_size)
//! up to `C`.
//!
//! ## Retransmission request message ([`Request`](protocol::SessionMessage::Request))
//! Contains a request for retransmission of missing segments in a frame. This is sent from
//! the segment recipient to the sender, once it realizes some of the received frames are incomplete
//! (after a certain period of time).
//!
//! The encoding of this message consists of pairs of [frame ID](crate::frame::FrameId) and
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
//! The message consists simply of a list of [frame IDs](crate::frame::FrameId) of the completely received
//! frames. There can be at most [`MAX_ACK_FRAMES`](protocol::FrameAcknowledgements::MAX_ACK_FRAMES)
//! per message. If more frames need to be acknowledged, more messages need to be sent.
//! If the message contains fewer entries, it is padded with zeros (0 is not a valid frame ID).
//!
//! # `Session` protocol state machine
//! The protocol always forms a middle layer between a *lower layer* transport (such as an unreliable
//! UDP-like network) and any upstream protocol.
//! The communication with the *lower layer* is done via [`SessionState`](state::SessionState);
//! the *upper layer* is using the [`SessionSocket`](state::SessionSocket) to pass data with the
//! `Session` protocol.
//!
//! ## Instantiation
//! The instantiation of the protocol state machine is done by creating the [`SessionSocket`](state::SessionSocket)
//! object, by [providing it](state::SessionSocket::new) an underlying transport writer and its MTU `C`.
//! The protocol can be instantiated over any transport that implements [`futures::io::AsyncWrite`]
//! for sending raw data packets.
//!
//! ## Passing data between the protocol and the upper layer
//! The [`SessionSocket`](state::SessionSocket) exposes as [`AsyncRead`](futures::io::AsyncRead) +
//! [`AsyncWrite`](futures::io::AsyncWrite) and can be used to read and write arbitrary data
//! to the protocol. If the writer is [closed](futures::AsyncWrite::poll_close), the session is closed
//! as well.
//!
//! ## Passing of data from the protocol *to* the lower layer
//!
//! Writes to the underlying transport happen automatically as needed. The amount of data written
//! per each [`write`](futures::io::AsyncWrite::poll_write) does not exceed the size of the set MTU.
//!
//! ## Passing of data to the protocol *from* the lower layer
//!
//! The user is responsible for polling the underlying transport for any incoming data,
//! and updating the [state of the socket](state::SessionSocket::state).
//! This is done by passing the data to the [`SessionState`](state::SessionState), which implements
//! [`futures::io::Sink`] for any `AsRef<[u8]>`. The size of each [`send`](futures::Sink::start_send)
//! to the `Sink` also must not exceed the size of the MTU.
//!
//! ## Protocol features
//!
//! ### Data segmentation
//! Once data is written to the [`SessionSocket`](state::SessionSocket), it is segmented and written
//! automatically to the underlying transport. Every write to the `SessionSocket` corresponds to
//! a [`Frame`](crate::frame::Frame).
//!
//! ## Frame reassembly
//! The receiving side performs frame reassembly and sequencing of the frames.
//! Frames are never emitted to the upper layer transport out of order, but frames
//! can be skipped if they exceed the [`frame_expiration_age`](state::SessionConfig).
//!
//! ## Frame acknowledgement
//!
//! The recipient can acknowledge frames to the sender, once all its segments have been received.
//! This is done with a [`FrameAcknowledgements`](protocol::FrameAcknowledgements) message sent back
//! to the sender.
//!
//! ## Segment retransmission
//!
//! There are two means of segment retransmission:
//!
//! ### Recipient requested retransmission
//! This is useful in situations when the recipient has received only some segments of a frame.
//! At this point, the recipient knows which segments are missing in a frame and can initiate
//! [`SegmentRequest`](protocol::SegmentRequest) sent back to the sender.
//! This method is more targeted, as it requests only those segments of a frame that are needed.
//! Once the sender receives the segment request, it will retransmit the segments in question
//! over to the receiver.
//! The recipient can make repeating requests on retransmission, based on the network reliability.
//! However, retransmission requests decay with an exponential backoff given by `backoff_base`
//! and `rto_base_receiver` timeout in [`SessionConfig`](state::SessionConfig) up
//! until the `frame_expiration_age`.
//!
//!
//! ### Sender initiated retransmission
//! The frame sender can also automatically retransmit entire frames (= all their segments)
//! to the recipient. This happens if the sender (within a time period) did not receive the
//! frame acknowledgement *and* the recipient also did not request retransmission of any segment in
//! that frame.
//! This is useful in situations when the recipient did not receive any segment of a frame. Once
//! the recipient receives at least one segment of a frame, the recipient requested retransmission
//! is the preferred way.
//!
//! The sender can make repeating frame retransmissions, based on the network reliability.
//! However, retransmissions decay with an exponential backoff given by `backoff_base`
//! and `rto_base_sender` timeout in [`SessionConfig`](state::SessionConfig) up until
//! the `frame_expiration_age`.
//! The retransmissions of a frame by the sender stop, if the frame has been acknowledged by the
//! recipient *or* the recipient started requesting segment retransmission.
//!
//! ### Retransmission timing
//! Both retransmission methods will work up until `frame_expiration_age`. Since the
//! recipient request based method is more targeted, at least one should be allowed to happen
//! before the sender initiated retransmission kicks in. Therefore, it is recommended to set
//! the `rto_base_sender` at least twice the `rto_base_receiver`.
//!
//! ## State stepping
//!
//! As mentioned in the above text, the only responsibility of the user so far is to
//! pass the data received by the transport to the `SessionState` of the socket.
//!
//! This, however, takes care only of the basic payload transmission function.
//! If the user wishes to use the retransmission and acknowledgement features of the protocol,
//! it must periodically call (in order) the corresponding methods of [`SessionState`](state::SessionState):
//! 1) [`acknowledge_segments`](state::SessionState::acknowledge_segments)
//! 2) [`request_missing_segments`](state::SessionState::request_missing_segments)
//! 3) [`retransmit_unacknowledged_frames`](state::SessionState::retransmit_unacknowledged_frames)
//!
//! These should be called multiple times within the `frame_expiration_age` period, in order
//! for the session state to advance.

pub mod errors;
pub mod protocol;
pub mod state;
mod utils;
