//! Collection of objects and functionality allowing building of p2p or stream protocols for the higher business logic
//! layers.
//!
//! ## Contents
//!
//! Supported protocol configurations:
//!
//! - `mix`
//! - `ack`
//! - `heartbeat`

/// Coder and decoder for the transport binary protocol layer
mod codec;

/// Configuration of the protocol components.
pub mod config;
/// Errors produced by the crate.
pub mod errors;

// protocols
/// `heartbeat` p2p protocol
pub mod heartbeat;

/// Stream processing utilities
pub mod stream;

pub mod timer;
mod pipeline;

use hopr_transport_identity::{Multiaddr, PeerId};

pub use timer::execute_on_tick;

pub use pipeline::{run_packet_pipeline, PacketPipelineProcesses, TicketEvent};

const HOPR_PACKET_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::SIZE;

pub type HoprBinaryCodec = crate::codec::FixedLengthCodec<HOPR_PACKET_SIZE>;
pub const CURRENT_HOPR_MSG_PROTOCOL: &str = "/hopr/mix/1.0.0";


/// Processed indexer generated events.
#[derive(Debug, Clone)]
pub enum PeerDiscovery {
    Allow(PeerId),
    Announce(PeerId, Vec<Multiaddr>),
}
