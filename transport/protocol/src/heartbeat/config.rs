use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use validator::Validate;

/// Configuration for the `heartbeat` protocol.
#[serde_as]
#[derive(Debug, Copy, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize, Eq, PartialEq)]
pub struct HeartbeatProtocolConfig {
    /// Maximum duration before the request times out
    #[serde_as(as = "DurationSeconds<u64>")]
    #[default(Duration::from_secs(6))]
    pub timeout: Duration,
}
