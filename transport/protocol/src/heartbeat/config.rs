use std::time::Duration;

use validator::Validate;

/// Configuration for the `heartbeat` protocol.
#[derive(Debug, Copy, Clone, smart_default::SmartDefault, Validate, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct HeartbeatProtocolConfig {
    /// Maximum duration before the request times out
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    #[default(Duration::from_secs(6))]
    pub timeout: Duration,
}
