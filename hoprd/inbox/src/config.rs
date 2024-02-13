use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::{Validate, ValidationError};

use hopr_internal_types::prelude::*;

pub fn validate_is_power_of_two(value: u32) -> Result<(), ValidationError> {
    ((value & (value - 1)) == 0)
        .then_some(())
        .ok_or(ValidationError::new("The value is not power of 2"))
}

/// Holds basic configuration parameters of the `MessageInbox`.
#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct MessageInboxConfiguration {
    /// Maximum capacity per-each application tag.
    ///
    /// In the current implementation, the capacity must be a power of two.
    ///
    /// Defaults to 512.
    #[validate(custom = "validate_is_power_of_two")]
    #[serde(default = "default_capacity")]
    #[default(default_capacity())]
    pub capacity: u32,
    /// Maximum age of a message held in the inbox until it is purged.
    ///
    /// Defaults to 15 minutes.
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "just_15_minutes")]
    #[default(just_15_minutes())]
    pub max_age: Duration,
    /// List of tags that are excluded on `push`.
    ///
    /// Defaults to \[[DEFAULT_APPLICATION_TAG]\]
    #[serde(default = "default_excluded_tags")]
    #[default(default_excluded_tags())]
    pub excluded_tags: Vec<Tag>,
}

#[inline]
fn just_15_minutes() -> Duration {
    const RAW_15_MINUTES: Duration = Duration::from_secs(15 * 60);
    RAW_15_MINUTES
}

#[inline]
fn default_capacity() -> u32 {
    512
}

#[inline]
fn default_excluded_tags() -> Vec<Tag> {
    vec![DEFAULT_APPLICATION_TAG]
}
