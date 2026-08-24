//! Contains implementation of a `Session` message protocol.
//!
//! The implementation in this crate follows
//! the HOPR [`RFC-0007`](https://github.com/hoprnet/rfc/tree/main/rfcs/RFC-0007-session-protocol).
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
//! The [`UnreliableSocket`] acts as an unreliable Session protocol socket, taking care only of
//! segmentation, reassembly and sequencing.
//!
//! The [`ReliableSocket`] has (in addition to segmentation, reassembly and sequencing) an
//! internal state that allows acknowledging frames, retransmit unacknowledged frames and/or
//! requesting of missing frame segments. It therefore offers data some delivery guarantees
//! up to the pre-defined frame expiration time.
//!
//! The above sockets can be constructed on top of any transport that implements
//! [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`],
//! also using the [extension](SessionSocketExt) methods.
//!
//! ## Overview of the crate
//! - Protocol messages are defined in the `protocol` submodule.
//! - Socket-like Session interface is defined in `socket` submodule.
//! - Frames and segments are defined in the `frames` module.
//! - Segmentation, reassembly and sequencing are defined in the `processing` submodule.

/// Contains errors thrown from this module.
pub mod errors;
/// Client/ENTRY-side send-window flow control (AIMD send window over the honest delivery clock).
pub mod flow_control;
#[allow(dead_code)]
mod processing;
mod protocol;
mod socket;
pub(crate) mod utils;

pub use processing::types::FrameInspector;
pub use protocol::{FrameAcknowledgements, FrameId, Segment, SegmentId, SegmentRequest, SeqIndicator};
#[cfg(feature = "telemetry")]
pub use socket::telemetry::{NoopTracker, SessionMessageDiscriminants, SessionTelemetryTracker};
pub use socket::{
    SessionSocket, SessionSocketConfig,
    ack_state::{AcknowledgementMode, AcknowledgementState, AcknowledgementStateConfig},
    state::{SocketComponents, SocketState, Stateless},
};

// Enable exports of additional Session protocol types
#[cfg(feature = "session-types")]
pub mod types {
    pub use super::protocol::*;
}

/// Represents a stateless (and therefore unreliable) socket.
pub type UnreliableSocket<const C: usize> = SessionSocket<C, Stateless<C>>;

/// Represents a socket with reliable delivery.
pub type ReliableSocket<const C: usize> = SessionSocket<C, AcknowledgementState<C>>;

const fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// Maximum Session MTU even if the HOPR packet allows for more.
///
/// This value is currently based on the WG packet size plus the WG overhead, as a primary
/// use-case for Session sockets.
pub const MAX_SESSION_MTU: usize = 1452;

/// Computes the Session Socket MTU, given the MTU `C` of the underlying socket.
pub const fn session_socket_mtu<const C: usize>() -> usize {
    min(MAX_SESSION_MTU, C - protocol::SessionMessage::<C>::SEGMENT_OVERHEAD)
}

/// Snaps a requested `frame_size` down to a whole multiple of [`session_socket_mtu`], between one
/// and `max_segments` segments.
///
/// A frame that is not a whole multiple of the segment payload ends in a runt segment, and since the
/// downstream transport is unbuffered by default (`max_buffered_segments = 0`) that runt costs a
/// whole HOPR packet to carry the handful of bytes that did not fit. At `C` = HOPR packet payload
/// that is not a rounding detail: a 1500-byte frame over a 1452-byte segment emits 1452 + 48, so
/// half the packets on the wire would carry 48 bytes.
///
/// Flooring rather than rounding to nearest is deliberate — the frame is the unit of head-of-line
/// loss, so growing it past what the caller asked for would silently widen the blast radius of one
/// missing segment. Flooring only ever narrows it, and costs no extra packets: the same bytes still
/// occupy the same number of segments, just spread over more frames.
pub const fn session_frame_size<const C: usize>(frame_size: usize, max_segments: usize) -> usize {
    let segment = session_socket_mtu::<C>();
    let mut segments = frame_size / segment;
    if segments > max_segments {
        segments = max_segments;
    }
    if segments == 0 {
        segments = 1;
    }
    segments * segment
}

/// Adaptors for [`futures::io::AsyncRead`] + [`futures::io::AsyncWrite`] transport to use Session protocol.
///
/// Use `compat` first when the underlying transport is Tokio-based.
pub trait SessionSocketExt: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin {
    /// Runs a [reliable](ReliableSocket) Session protocol on self.
    fn reliable_session<const MTU: usize>(
        self,
        ack: AcknowledgementState<MTU>,
        cfg: SessionSocketConfig,
    ) -> errors::Result<ReliableSocket<MTU>>
    where
        Self: Sized + 'static,
    {
        #[cfg(feature = "telemetry")]
        {
            SessionSocket::new(self, ack, cfg, NoopTracker)
        }
        #[cfg(not(feature = "telemetry"))]
        {
            SessionSocket::new(self, ack, cfg)
        }
    }

    /// Runs [unreliable](UnreliableSocket) Session protocol on self.
    fn unreliable_session<const MTU: usize>(
        self,
        id: &str,
        cfg: SessionSocketConfig,
    ) -> errors::Result<UnreliableSocket<MTU>>
    where
        Self: Sized + 'static,
    {
        #[cfg(feature = "telemetry")]
        {
            SessionSocket::new_stateless(id, self, cfg, NoopTracker)
        }
        #[cfg(not(feature = "telemetry"))]
        {
            SessionSocket::new_stateless(id, self, cfg)
        }
    }
}

impl<T: ?Sized> SessionSocketExt for T where T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin {}
