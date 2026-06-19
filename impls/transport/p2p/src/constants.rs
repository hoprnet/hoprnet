/// Connection idle timeout for all protocols used in the swarm.
pub const HOPR_SWARM_IDLE_CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// Default interval between keep-alive pings sent to a connected peer.
///
/// Must be short enough to detect half-open connections (NAT rebinding,
/// middlebox idle drops) before the application layer notices the stall.
/// Overridable via `HOPR_INTERNAL_LIBP2P_PING_INTERVAL_SECS`.
pub(crate) const HOPR_PING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);

/// Timeout for a single ping round-trip.
///
/// A peer that does not respond within this window has its connection closed,
/// generating a `ConnectionClosed` → `PeerDisconnected` event → liveness flag
/// cleared → cached streams self-error on next poll.
/// Overridable via `HOPR_INTERNAL_LIBP2P_PING_TIMEOUT_SECS`.
pub(crate) const HOPR_PING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

// Swarm configuration
/// The maximum number of concurrently dialed (outbound) peers.
pub(crate) const HOPR_SWARM_CONCURRENTLY_DIALED_PEER_COUNT: u8 = 255;
/// The maximum number of concurrently negotiating inbound peers.
pub(crate) const HOPR_SWARM_CONCURRENTLY_NEGOTIATING_INBOUND_PEER_COUNT: usize = 512;
