use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use smart_default::SmartDefault;
use validator::Validate;

/// Network quality threshold since which a node is considered
/// available enough to be used
pub const DEFAULT_NETWORK_OFFLINE_QUALITY_THRESHOLD: f64 = 0.5;
pub const DEFAULT_NETWORK_BAD_QUALITY_THRESHOLD: f64 = 0.2;
pub const DEFAULT_NETWORK_QUALITY_STEP: f64 = 0.1;
pub const DEFAULT_NETWORK_QUALITY_AVERAGE_WINDOW_SIZE: u32 = 25;
pub const DEFAULT_NETWORK_BACKOFF_EXPONENT: f64 = 1.5;
pub const DEFAULT_NETWORK_BACKOFF_MIN: f64 = 2.0;

/// Configuration for the [`Network`] object
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

    #[serde(default = "quality_step")]
    #[default(quality_step())]
    pub quality_step: f64,

    #[serde(default = "quality_average_window_size")]
    #[default(quality_average_window_size())]
    pub quality_avg_window_size: u32,

    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "duration_10_min")]
    #[default(duration_10_min())]
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
        if !(0.0..1.0).contains(&self.quality_bad_threshold) {
            errors.add(
                "quality_bad_threshold",
                validator::ValidationError::new("quality_bad_threshold must be between 0 and 1"),
            );
        }

        // #[validate(range(min = 0.0, max = 1.0))]
        if !(0.0..1.0).contains(&self.quality_offline_threshold) {
            errors.add(
                "quality_offline_threshold",
                validator::ValidationError::new("quality_offline_threshold must be between 0 and 1"),
            );
        }

        // if self.quality_bad_threshold > self.quality_offline_threshold {
        //     errors.add(
        //         "quality_bad_threshold and quality_offline_threshold",
        //         validator::ValidationError::new("quality_bad_threshold must be less than quality_offline_threshold"),
        //     );
        // }

        // #[validate(range(min = 0.0, max = 1.0))]
        if !(0.0..1.0).contains(&self.quality_step) {
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

        if self.backoff_min >= self.backoff_max {
            errors.add(
                "backoff_min and backoff_max",
                validator::ValidationError::new("backoff_min must be less than backoff_max"),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
fn quality_step() -> f64 {
    DEFAULT_NETWORK_QUALITY_STEP
}

#[inline]
fn quality_average_window_size() -> u32 {
    DEFAULT_NETWORK_QUALITY_AVERAGE_WINDOW_SIZE
}

#[inline]
fn duration_10_min() -> Duration {
    Duration::from_secs(600)
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
