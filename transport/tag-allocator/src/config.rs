use std::ops::Range;

use hopr_protocol_app::prelude::ReservedTag;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::{TAG_RANGE_SIZE, Usage};

/// Default number of tags reserved for sessions.
pub const DEFAULT_SESSION_CAPACITY: u64 = 2048;
/// Default number of tags reserved for session terminal telemetry.
pub const DEFAULT_SESSION_PROBING_CAPACITY: u64 = 4000;
/// Default number of tags reserved for probing telemetry (remainder of range).
pub const DEFAULT_PROBING_TELEMETRY_CAPACITY: u64 =
    TAG_RANGE_SIZE - DEFAULT_SESSION_CAPACITY - DEFAULT_SESSION_PROBING_CAPACITY;

/// Configuration for the tag allocator partitions.
///
/// Each field specifies the number of tags reserved for that usage category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct TagAllocatorConfig {
    /// Number of tags reserved for long-lived sessions.
    ///
    /// This also determines the maximum number of concurrent sessions.
    #[default(DEFAULT_SESSION_CAPACITY)]
    pub session: u64,

    /// Number of tags reserved for session terminal telemetry.
    #[default(DEFAULT_SESSION_PROBING_CAPACITY)]
    pub session_probing: u64,

    /// Number of tags reserved for probing telemetry.
    ///
    /// Defaults to the remainder of the available tag range when using the
    /// default values for `session` and `session_probing`. When overriding
    /// those fields, ensure the sum of all three capacities does not exceed
    /// [`TAG_RANGE_SIZE`]; this is validated at allocator creation time.
    #[default(DEFAULT_PROBING_TELEMETRY_CAPACITY)]
    pub probing_telemetry: u64,
}

impl TagAllocatorConfig {
    /// Returns the tag range for the given [`Usage`] partition.
    ///
    /// Partitions are laid out contiguously starting at
    /// [`ReservedTag::UPPER_BOUND`] in the order: Session,
    /// SessionTerminalTelemetry, ProvingTelemetry.
    pub fn range_for(&self, usage: Usage) -> Range<u64> {
        let base = ReservedTag::UPPER_BOUND;
        match usage {
            Usage::Session => base..base + self.session,
            Usage::SessionTerminalTelemetry => {
                let start = base + self.session;
                start..start + self.session_probing
            }
            Usage::ProvingTelemetry => {
                let start = base + self.session + self.session_probing;
                start..start + self.probing_telemetry
            }
        }
    }

    /// The full tag range covered by this configuration.
    ///
    /// Starts at [`ReservedTag::UPPER_BOUND`] and spans the sum of all
    /// configured partition capacities.
    pub fn tag_range(&self) -> Range<u64> {
        let start = ReservedTag::UPPER_BOUND;
        start..start + self.session + self.session_probing + self.probing_telemetry
    }
}

impl Validate for TagAllocatorConfig {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.session == 0 {
            errors.add("session", ValidationError::new("capacity must be greater than zero"));
        }
        if self.session_probing == 0 {
            errors.add(
                "session_probing",
                ValidationError::new("capacity must be greater than zero"),
            );
        }
        if self.probing_telemetry == 0 {
            errors.add(
                "probing_telemetry",
                ValidationError::new("capacity must be greater than zero"),
            );
        }

        let total = self.session + self.session_probing + self.probing_telemetry;
        if total > TAG_RANGE_SIZE {
            let mut err = ValidationError::new("total capacity exceeds available tag range");
            err.add_param(std::borrow::Cow::Borrowed("total"), &total);
            err.add_param(std::borrow::Cow::Borrowed("available"), &TAG_RANGE_SIZE);
            errors.add("probing_telemetry", err);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = TagAllocatorConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn zero_session_capacity_is_invalid() {
        let cfg = TagAllocatorConfig {
            session: 0,
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.field_errors().contains_key("session"));
    }

    #[test]
    fn zero_session_probing_capacity_is_invalid() {
        let cfg = TagAllocatorConfig {
            session_probing: 0,
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.field_errors().contains_key("session_probing"));
    }

    #[test]
    fn zero_probing_telemetry_capacity_is_invalid() {
        let cfg = TagAllocatorConfig {
            probing_telemetry: 0,
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.field_errors().contains_key("probing_telemetry"));
    }

    #[test]
    fn total_exceeding_range_is_invalid() {
        let cfg = TagAllocatorConfig {
            session: TAG_RANGE_SIZE,
            session_probing: 1,
            probing_telemetry: 1,
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.field_errors().contains_key("probing_telemetry"));
    }

    #[test]
    fn custom_config_within_range_is_valid() {
        let cfg = TagAllocatorConfig {
            session: 1000,
            session_probing: 1000,
            probing_telemetry: 1000,
        };
        assert!(cfg.validate().is_ok());
    }
}
