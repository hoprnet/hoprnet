pub const HOPR_HEARTBEAT_PROTOCOL_V_0_1_0: &str = "/hopr/heartbeat/0.1.0";
pub const HOPR_MESSAGE_PROTOCOL_V_0_1_0: &str = "/hopr/msg/0.1.0";
pub const HOPR_ACKNOWLEDGE_PROTOCOL_V_0_1_0: &str = "/hopr/ack/0.1.0";
pub const HOPR_TICKET_AGGREGATION_PROTOCOL_V_0_1_0: &str = "/hopr/ticket-aggregation/0.1.0";

pub const HOPR_HEARTBEAT_CONNECTION_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3600);          // 1 hour
pub const HOPR_MESSAGE_CONNECTION_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3600);            // 1 hour
pub const HOPR_ACKNOWLEDGEMENT_CONNECTION_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3600);    // 1 hour
pub const HOPR_TICKET_AGGREGATION_CONNECTION_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3600); // 1 hour