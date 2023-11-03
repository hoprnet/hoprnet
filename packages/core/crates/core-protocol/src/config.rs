use serde::{Deserialize, Serialize};
use validator::Validate;

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Default, Serialize, Deserialize, Validate, Copy, Clone, PartialEq)]
pub struct ProtocolConfig {
    /// `ack` protocol config
    pub ack: crate::ack::config::AckProtocolConfig,
    /// `heartbeat` protocol config
    pub heartbeat: crate::heartbeat::config::HeartbeatProtocolConfig,
    /// `msg` protocol config
    pub msg: crate::msg::config::MsgProtocolConfig,
    /// `ticket_aggregation` protocol config
    pub ticket_aggregation: crate::ticket_aggregation::config::TicketAggregationProtocolConfig,
}
