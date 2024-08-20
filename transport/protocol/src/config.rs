use serde::{Deserialize, Serialize};
use validator::Validate;

/// Configuration of the P2P protocols.
#[derive(Debug, Default, Serialize, Deserialize, Validate, Copy, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProtocolConfig {
    /// `heartbeat` protocol config
    #[serde(default)]
    pub heartbeat: crate::heartbeat::config::HeartbeatProtocolConfig,
    /// `ticket_aggregation` protocol config
    #[serde(default)]
    pub ticket_aggregation: crate::ticket_aggregation::config::TicketAggregationProtocolConfig,
}
