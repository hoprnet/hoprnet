use std::time::Duration;

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use validator::Validate;

pub const DEFAULT_AUTO_PATH_QUALITY_THRESHOLD: f64 = 0.95;

pub const DEFAULT_MAX_FIRST_HOP_LATENCY_THRESHOLD: Duration = Duration::from_millis(250);

/// Configuration for the [`crate::network::Network`] object
#[derive(Debug, Clone, Copy, Serialize, Deserialize, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    #[serde(default = "node_score_auto_path_threshold")]
    #[default(node_score_auto_path_threshold())]
    pub node_score_auto_path_threshold: f64,

    #[serde(default = "max_first_hop_latency_threshold", with = "humantime_serde")]
    #[default(max_first_hop_latency_threshold())]
    pub max_first_hop_latency_threshold: Option<Duration>,

    #[serde(default)]
    pub allow_private_addresses_in_store: bool,
}

impl Validate for NetworkConfig {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();
        if !(0.0..=1.0).contains(&self.node_score_auto_path_threshold) {
            errors.add(
                "node_score_auto_path_threshold",
                validator::ValidationError::new("node_score_auto_path_threshold must be between 0 and 1"),
            );
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[inline]
fn node_score_auto_path_threshold() -> f64 {
    DEFAULT_AUTO_PATH_QUALITY_THRESHOLD
}

#[inline]
fn max_first_hop_latency_threshold() -> Option<Duration> {
    Some(DEFAULT_MAX_FIRST_HOP_LATENCY_THRESHOLD)
}
