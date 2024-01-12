use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::Validate;

#[serde_as]
#[derive(Debug, Copy, Clone, Validate, Serialize, Deserialize, Eq, PartialEq)]
pub struct MsgProtocolConfig {
    /// Maximum duration before the request times out
    #[serde_as(as = "DurationSeconds<u64>")]
    pub timeout: Duration,
}

impl Default for MsgProtocolConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(15),
        }
    }
}
