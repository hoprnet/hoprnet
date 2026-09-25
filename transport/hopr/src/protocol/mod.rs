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
    PacketPipelineConfig, PacketPipelineProcesses, PoolArbitrationConfig, Unset,
};

const HOPR_PACKET_SIZE: usize = hopr_crypto_packet::prelude::HoprPacket::SIZE;

pub type HoprBinaryCodec = codec::FixedLengthCodec<HOPR_PACKET_SIZE>;
// The larger fixed-size packets cannot be decoded by peers using the previous wire format.
// Version 1.2.0 uses generation-tagged SURBs with the previous packet payload size.
pub const CURRENT_HOPR_MSG_PROTOCOL: &str = "/hopr/mix/1.3.0";
