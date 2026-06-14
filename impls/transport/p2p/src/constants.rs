/// Connection idle timeout for all protocols used in the swarm.
pub const HOPR_SWARM_IDLE_CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

// Swarm configuration
/// The maximum number of concurrently dialed (outbound) peers.
pub(crate) const HOPR_SWARM_CONCURRENTLY_DIALED_PEER_COUNT: u8 = 255;
/// The maximum number of concurrently negotiating inbound peers.
pub(crate) const HOPR_SWARM_CONCURRENTLY_NEGOTIATING_INBOUND_PEER_COUNT: usize = 512;
