use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use smart_default::SmartDefault;
use validator::Validate;

/// Network quality threshold since when a node is considered
/// available enough to be used
pub const DEFAULT_NETWORK_OFFLINE_QUALITY_THRESHOLD: f64 = 0.0;
pub const DEFAULT_NETWORK_BAD_QUALITY_THRESHOLD: f64 = 0.1;
pub const DEFAULT_NETWORK_QUALITY_STEP: f64 = 0.1;
pub const DEFAULT_NETWORK_QUALITY_AVERAGE_WINDOW_SIZE: u32 = 25;
pub const DEFAULT_NETWORK_BACKOFF_EXPONENT: f64 = 1.5;
pub const DEFAULT_NETWORK_BACKOFF_MIN: f64 = 2.0;

pub const DEFAULT_AUTO_PATH_QUALITY_THRESHOLD: f64 = 0.95;

pub const DEFAULT_MAX_FIRST_HOP_LATENCY_THRESHOLD: Duration = Duration::from_millis(250);

pub const DEFAULT_CANNOT_DIAL_PENALTY: Duration = Duration::from_secs(60 * 60); // 1 hour

/// Configuration for the [`crate::network::Network`] object
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    /// Minimum delay will be multiplied by backoff, it will be half the actual minimum value
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "duration_1_s")]
    #[default(duration_1_s())]
    pub min_delay: Duration,

    /// Maximum delay
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "duration_5_min")]
    #[default(duration_5_min())]
    pub max_delay: Duration,

    #[serde(default = "quality_bad_threshold")]
    #[default(quality_bad_threshold())]
    pub quality_bad_threshold: f64,

    #[serde(default = "quality_offline_threshold")]
    #[default(quality_offline_threshold())]
    pub quality_offline_threshold: f64,

    #[serde(default = "node_score_auto_path_threshold")]
    #[default(node_score_auto_path_threshold())]
    pub node_score_auto_path_threshold: f64,

    #[serde_as(as = "Option<serde_with::DurationMilliSeconds<u64>>")]
    #[serde(default = "max_first_hop_latency_threshold")]
    #[default(max_first_hop_latency_threshold())]
    pub max_first_hop_latency_threshold: Option<Duration>,

    #[serde(default = "quality_step")]
    #[default(quality_step())]
    pub quality_step: f64,

    #[serde(default = "quality_average_window_size")]
    #[default(quality_average_window_size())]
    pub quality_avg_window_size: u32,

    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "duration_2_min")]
    #[default(duration_2_min())]
    pub ignore_timeframe: Duration,

    #[serde(default = "backoff_exponent")]
    #[default(backoff_exponent())]
    pub backoff_exponent: f64,

    #[serde(default = "backoff_min")]
    #[default(backoff_min())]
    pub backoff_min: f64,

    #[serde(default = "backoff_max")]
    #[default(backoff_max())]
    pub backoff_max: f64,

    #[serde(default)]
    pub allow_private_addresses_in_store: bool,
}

impl Validate for NetworkConfig {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.min_delay >= self.max_delay {
            errors.add(
                "min_delay and max_delay",
                validator::ValidationError::new("min_delay must be less than max_delay"),
            );
        }

        // #[validate(range(min = 0.0, max = 1.0))]
        if !(0.0..=1.0).contains(&self.quality_bad_threshold) {
            errors.add(
                "quality_bad_threshold",
                validator::ValidationError::new("quality_bad_threshold must be between 0 and 1"),
            );
        }

        if !(0.0..=1.0).contains(&self.node_score_auto_path_threshold) {
            errors.add(
                "node_score_auto_path_threshold",
                validator::ValidationError::new("node_score_auto_path_threshold must be between 0 and 1"),
            );
        }

        // #[validate(range(min = 0.0, max = 1.0))]
        if !(0.0..=1.0).contains(&self.quality_offline_threshold) {
            errors.add(
                "quality_offline_threshold",
                validator::ValidationError::new("quality_offline_threshold must be between 0 and 1"),
            );
        }

        if self.quality_bad_threshold < self.quality_offline_threshold {
            errors.add(
                "quality_bad_threshold and quality_offline_threshold",
                validator::ValidationError::new("quality_bad_threshold must be greater than quality_offline_threshold"),
            );
        }

        // #[validate(range(min = 0.0, max = 1.0))]
        if !(0.0..=1.0).contains(&self.quality_step) {
            errors.add(
                "quality_step",
                validator::ValidationError::new("quality_step must be between 0 and 1"),
            );
        }

        // #[validate(range(min = 1_u32))]
        if self.quality_avg_window_size < 1 {
            errors.add(
                "quality_avg_window_size",
                validator::ValidationError::new("quality_avg_window_size must be greater than 0"),
            );
        }

        // #[validate(range(min = 0.0))]
        if self.backoff_min < 0.0 {
            errors.add(
                "backoff_min",
                validator::ValidationError::new("backoff_min must be greater or equal 0"),
            );
        }

        if self.backoff_min >= self.backoff_max {
            errors.add(
                "backoff_min and backoff_max",
                validator::ValidationError::new("backoff_min must be less than backoff_max"),
            );
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[inline]
fn duration_1_s() -> Duration {
    Duration::from_secs(1)
}

#[inline]
fn duration_5_min() -> Duration {
    Duration::from_secs(300)
}

#[inline]
fn quality_bad_threshold() -> f64 {
    DEFAULT_NETWORK_BAD_QUALITY_THRESHOLD
}

#[inline]
fn quality_offline_threshold() -> f64 {
    DEFAULT_NETWORK_OFFLINE_QUALITY_THRESHOLD
}

#[inline]
fn node_score_auto_path_threshold() -> f64 {
    DEFAULT_AUTO_PATH_QUALITY_THRESHOLD
}

#[inline]
fn max_first_hop_latency_threshold() -> Option<Duration> {
    Some(DEFAULT_MAX_FIRST_HOP_LATENCY_THRESHOLD)
}

#[inline]
fn quality_step() -> f64 {
    DEFAULT_NETWORK_QUALITY_STEP
}

#[inline]
fn quality_average_window_size() -> u32 {
    DEFAULT_NETWORK_QUALITY_AVERAGE_WINDOW_SIZE
}

#[inline]
fn duration_2_min() -> Duration {
    Duration::from_secs(2 * 60)
}

#[inline]
fn backoff_exponent() -> f64 {
    DEFAULT_NETWORK_BACKOFF_EXPONENT
}

#[inline]
fn backoff_min() -> f64 {
    DEFAULT_NETWORK_BACKOFF_MIN
}

#[inline]
fn backoff_max() -> f64 {
    duration_5_min().as_millis() as f64 / duration_1_s().as_millis() as f64
}
