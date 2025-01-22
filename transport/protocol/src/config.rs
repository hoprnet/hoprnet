use hopr_internal_types::prelude::DEFAULT_OUTGOING_TICKET_WIN_PROB;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn default_outgoing_win_prob() -> f64 {
    DEFAULT_OUTGOING_TICKET_WIN_PROB
}

/// Configuration of the P2P protocols.
#[derive(Debug, smart_default::SmartDefault, Serialize, Deserialize, Validate, Copy, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProtocolConfig {
    /// Winning probability that gets printed on any outgoing tickets.
    #[validate(range(min = 0.0, max = 1.0))]
    #[default(DEFAULT_OUTGOING_TICKET_WIN_PROB)]
    #[serde(default = "default_outgoing_win_prob")]
    pub outgoing_ticket_winning_prob: f64,
    /// `heartbeat` protocol config
    #[serde(default)]
    pub heartbeat: crate::heartbeat::config::HeartbeatProtocolConfig,
    /// `ticket_aggregation` protocol config
    #[serde(default)]
    pub ticket_aggregation: crate::ticket_aggregation::config::TicketAggregationProtocolConfig,
}
