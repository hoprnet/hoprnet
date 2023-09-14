use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::{Validate, ValidationError};

use core_types::protocol::{Tag, DEFAULT_APPLICATION_TAG};

pub fn validate_is_power_of_two(value: u32) -> Result<(), ValidationError> {
    if (value & (value - 1)) != 0 {
        Err(ValidationError::new("The value is not power of 2"))
    } else {
        Ok(())
    }
}

/// Holds basic configuration parameters of the `MessageInbox`.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[serde_as]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct MessageInboxConfiguration {
    /// Maximum capacity per-each application tag.
    /// In the current implementation, the capacity must be a power of two.
    #[validate(custom = "validate_is_power_of_two")]
    pub capacity: u32,
    /// Maximum age of a message held in the inbox until it is purged.
    #[serde_as(as = "DurationSeconds<u64>")]
    max_age: Duration, // TODO: with the removal of wasm-bindgen this value can be public
    /// List of tags that are excluded on `push`.
    pub excluded_tags: Vec<Tag>,
}

const RAW_15_MINUTES: Duration = Duration::from_secs(15 * 60);

impl Default for MessageInboxConfiguration {
    fn default() -> Self {
        Self {
            capacity: 512,
            max_age: RAW_15_MINUTES,
            excluded_tags: vec![DEFAULT_APPLICATION_TAG],
        }
    }
}

impl MessageInboxConfiguration {
    pub fn max_age_sec(&self) -> u64 {
        self.max_age.as_secs()
    }

    pub fn set_max_age_sec(&mut self, age: u64) {
        self.max_age = Duration::from_secs(age)
    }
}
