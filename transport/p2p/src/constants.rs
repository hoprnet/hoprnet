/// Connection idle timeout for all protocols used in the swarm.
pub const HOPR_SWARM_IDLE_CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600); // 10 minutes

/// P2P protocol identifiers
pub(crate) const HOPR_HEARTBEAT_PROTOCOL_V_0_1_0: &str = "/hopr/heartbeat/0.1.0";
pub(crate) const HOPR_MESSAGE_PROTOCOL_V_0_1_0: &str = "/hopr/msg/0.1.0";
pub(crate) const HOPR_ACKNOWLEDGE_PROTOCOL_V_0_1_0: &str = "/hopr/ack/0.1.0";
pub(crate) const HOPR_TICKET_AGGREGATION_PROTOCOL_V_0_1_0: &str = "/hopr/ticket-aggregation/0.1.0";

// Swarm configuration
/// The maximum number of concurrently dialed (outbound) peers.
pub const HOPR_SWARM_CONCURRENTLY_DIALED_PEER_COUNT: u8 = 255;
/// The maximum number of concurrently negotiating inbound peers.
pub const HOPR_SWARM_CONCURRENTLY_NEGOTIATING_INBOUND_PEER_COUNT: usize = 255;
