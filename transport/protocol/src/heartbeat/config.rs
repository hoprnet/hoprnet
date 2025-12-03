use std::time::Duration;

use validator::Validate;

/// Configuration for the `heartbeat` protocol.
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug, Copy, Clone, smart_default::SmartDefault, Validate, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct HeartbeatProtocolConfig {
    /// Maximum duration before the request times out
    #[cfg_attr(feature = "serde", serde_as(as = "serde_with::DurationSeconds<u64>"))]
    #[default(Duration::from_secs(6))]
    pub timeout: Duration,
}
