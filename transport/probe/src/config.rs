use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use validator::Validate;

/// Configuration for the probing mechanism
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeConfig {
    /// Maximum number of parallel probes performed by the mechanism
    #[serde_as(as = "DurationSeconds<u64>")]
    #[default(default_max_probe_timeout())]
    #[serde(default = "default_max_probe_timeout")]
    pub timeout: std::time::Duration,

    /// Maximum number of parallel probes performed by the mechanism
    #[validate(range(min = 1))]
    #[default(default_max_parallel_probes())]
    #[serde(default = "default_max_parallel_probes")]
    pub max_parallel_probes: usize,

    /// The delay between individual probing rounds for neighbor discovery
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_probing_interval")]
    #[default(default_probing_interval())]
    pub(crate) interval: std::time::Duration,

    /// The time threshold after which it is reasonable to recheck the nearest neighbor
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_recheck_threshold")]
    #[default(default_recheck_threshold())]
    pub recheck_threshold: std::time::Duration,
}

/// The maximum time waiting for a reply from the probe
const DEFAULT_MAX_PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// The maximum number of parallel probes the heartbeat performs
const DEFAULT_MAX_PARALLEL_PROBES: usize = 50;

/// Delay before repeating probing rounds, must include enough time to traverse NATs
const DEFAULT_REPEATED_PROBING_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

/// Time after which the availability of a node gets rechecked
const DEFAULT_PROBE_RECHECK_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(60);

#[inline]
const fn default_max_probe_timeout() -> std::time::Duration {
    DEFAULT_MAX_PROBE_TIMEOUT
}

#[inline]
const fn default_max_parallel_probes() -> usize {
    DEFAULT_MAX_PARALLEL_PROBES
}

#[inline]
const fn default_probing_interval() -> std::time::Duration {
    DEFAULT_REPEATED_PROBING_DELAY
}

#[inline]
const fn default_recheck_threshold() -> std::time::Duration {
    DEFAULT_PROBE_RECHECK_THRESHOLD
}
