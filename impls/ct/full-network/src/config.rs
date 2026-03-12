#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Configuration for the probing mechanism
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
pub struct ProberConfig {
    /// The delay between individual probing rounds for neighbor discovery
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_probing_interval", with = "humantime_serde")
    )]
    #[default(default_probing_interval())]
    pub interval: std::time::Duration,

    /// Weight for staleness factor in probe priority (0.0–1.0).
    ///
    /// Higher values prioritize probing edges that haven't been measured recently.
    #[cfg_attr(feature = "serde", serde(default = "default_staleness_weight"))]
    #[default(default_staleness_weight())]
    pub staleness_weight: f64,

    /// Weight for inverse quality factor in probe priority (0.0–1.0).
    ///
    /// Higher values prioritize probing edges with poor quality scores.
    #[cfg_attr(feature = "serde", serde(default = "default_quality_weight"))]
    #[default(default_quality_weight())]
    pub quality_weight: f64,

    /// Base priority ensuring all peers have a nonzero chance of being probed (0.0–1.0).
    #[cfg_attr(feature = "serde", serde(default = "default_base_priority"))]
    #[default(default_base_priority())]
    pub base_priority: f64,

    /// TTL for the cached weighted shuffle order.
    ///
    /// When expired, the graph is re-traversed and a new priority-ordered shuffle is computed.
    #[cfg_attr(feature = "serde", serde(default = "default_shuffle_ttl", with = "humantime_serde"))]
    #[default(default_shuffle_ttl())]
    pub shuffle_ttl: std::time::Duration,
}

#[inline]
const fn default_staleness_weight() -> f64 {
    0.4
}

#[inline]
const fn default_quality_weight() -> f64 {
    0.3
}

#[inline]
const fn default_base_priority() -> f64 {
    0.3
}

#[inline]
const fn default_shuffle_ttl() -> std::time::Duration {
    std::time::Duration::from_secs(600)
}

#[inline]
const fn default_probing_interval() -> std::time::Duration {
    std::time::Duration::from_secs(30)
}
