use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::Validate;

#[serde_as]
#[derive(Debug, Copy, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize, Eq, PartialEq)]
pub struct TicketAggregationProtocolConfig {
    /// Maximum duration before the request times out
    #[serde_as(as = "DurationSeconds<u64>")]
    #[default(Duration::from_secs(15))]
    pub timeout: Duration,
}
