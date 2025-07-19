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
//! # Overview of the module
//! - Protocol messages are defined in the `protocol` submodule.
//! - Socket-like Session interface is defined in `socket` submodule.
//! - Frames and segments are defined in the `frames` module.
//! - Segmentation, reassembly and sequencing are defined in the `processing` submodule.

/// Contains errors thrown from this module.
pub mod errors;
mod frames;
mod processing;
mod protocol;
mod socket;
pub(crate) mod utils;

pub use socket::{
    SessionSocket, SessionSocketConfig,
    ack_state::{AcknowledgementMode, AcknowledgementState, AcknowledgementStateConfig},
    state::{SocketState, Stateless},
};

// Enable exports of additional Session protocol types
#[cfg(feature = "session-types")]
pub mod types {
    pub use super::{frames::*, protocol::*};
}

use protocol::SessionMessage;

/// Represents a stateless (and therefore unreliable) socket.
pub type StatelessSocket<const C: usize> = SessionSocket<C, Stateless<C>>;

/// Represents a socket with reliable delivery.
pub type ReliableSocket<const C: usize> = SessionSocket<C, AcknowledgementState<C>>;

/// Computes the Session Socket MTU, given the MTU `C` of the underlying socket.
pub const fn session_socket_mtu<const C: usize>() -> usize {
    C - SessionMessage::<C>::SEGMENT_OVERHEAD
}
