/// In-memory stub implementations of chain API traits (no Blokli, no network).
/// Used by benches that need a packet pipeline without a real chain.
pub mod stubs;

/// Shared emulated-peer harness: keypair fixtures, payload generators, per-peer
/// pipeline wiring, and the in-process software transport (`emulate_channel_communication`).
/// Used by both the protocol integration tests and the transport benches.
pub mod harness;
