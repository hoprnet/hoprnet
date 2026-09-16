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

/// Per-peer protocol conformance counters.
pub mod counters;

/// Errors produced by the crate.
pub mod errors;

// protocols
/// `heartbeat` p2p protocol
pub mod heartbeat;

/// Packet pipeline for the HOPR protocol.
mod pipeline;
/// Stream processing utilities
pub mod stream;

/// Sequences re-planning ahead of refilling when a return path goes silent.
pub mod return_path_recovery;
/// Records SURB round-trips as network graph edge telemetry.
pub mod surb_telemetry;

pub use counters::{PeerProtocolCounterRegistry, PeerProtocolCounters};
pub use pipeline::{
    AcknowledgementPipelineConfig, NodeType, NopExitAcknowledgementShareProcessor, PacketPipelineBuilder,
    PacketPipelineConfig, PacketPipelineProcesses, Unset,
};

const HOPR_PACKET_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::SIZE;

pub type HoprBinaryCodec = codec::FixedLengthCodec<HOPR_PACKET_SIZE>;
// Bumped 1.1.0 -> 1.2.0 for the generation-tagged SURB: the SURB grew by one byte (401 -> 402)
// while the fixed HOPR packet frame size is unchanged, so a pre-bump node would negotiate the same
// protocol and silently misparse the trailing generation byte. The exact-string match on this
// identifier makes the wire-format boundary explicit — nodes across it simply find no common `mix`
// protocol instead of exchanging incompatible payloads.
pub const CURRENT_HOPR_MSG_PROTOCOL: &str = "/hopr/mix/1.2.0";
