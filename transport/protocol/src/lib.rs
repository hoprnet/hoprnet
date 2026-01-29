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

/// Errors produced by the crate.
pub mod errors;

// protocols
/// `heartbeat` p2p protocol
pub mod heartbeat;

/// Packet pipeline for the HOPR protocol.
mod pipeline;
/// Stream processing utilities
pub mod stream;

use hopr_transport_identity::{Multiaddr, PeerId};
pub use pipeline::{AcknowledgementPipelineConfig, PacketPipelineProcesses, TicketEvent, run_packet_pipeline};

const HOPR_PACKET_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::SIZE;

pub type HoprBinaryCodec = codec::FixedLengthCodec<HOPR_PACKET_SIZE>;
pub const CURRENT_HOPR_MSG_PROTOCOL: &str = "/hopr/mix/1.1.0";

/// Processed indexer generated events.
#[derive(Debug, Clone)]
pub enum PeerDiscovery {
    Announce(PeerId, Vec<Multiaddr>),
}
