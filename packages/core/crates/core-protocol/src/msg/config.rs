
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::Validate;


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[serde_as]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct MsgProtocolConfig {
    /// Maximum duration before the request times out
    #[serde_as(as = "DurationSeconds<u64>")]
    timeout: Duration,      // TODO: with the removal of wasm-bindgen this value can be public
}

impl Default for MsgProtocolConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(15),
        }
    }
}

impl MsgProtocolConfig {
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}