use std::time::Duration;

use validator::Validate;

pub const HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS: u64 = 0;
pub const HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS: u64 = 20;
pub const HOPR_MIXER_DEFAULT_MAX_CAP_IN_MS: u64 = 20;
pub const HOPR_MIXER_DELAY_METRIC_WINDOW: u64 = 100;
pub const HOPR_MIXER_CAPACITY: usize = 20_000;

/// Percentile of packets released before the Poisson hard cap (`max_cap`) when the mean is
/// derived rather than set explicitly: `mean = max_cap / ln(1 / (1 - CAP_PERCENTILE))`. At the
/// default 20 ms cap this gives a ~4.3 ms mean, i.e. ~99% release before the cap.
pub const HOPR_MIXER_CAP_PERCENTILE: f64 = 0.99;
pub const HOPR_MIXER_DEFAULT_CAP_JITTER_IN_MS: u64 = 2;
pub const HOPR_MIXER_DEFAULT_MIN_MIX_OCCUPANCY: usize = 5;
pub const HOPR_MIXER_DEFAULT_TICK_FLOOR_IN_MS: u64 = 1;
pub const HOPR_MIXER_DEFAULT_SATURATION_MIN_MEAN_IN_MS: u64 = 1;

#[cfg(feature = "serde")]
fn default_metric_delay_window() -> u64 {
    HOPR_MIXER_DELAY_METRIC_WINDOW
}
#[cfg(feature = "serde")]
fn default_max_cap() -> Duration {
    Duration::from_millis(HOPR_MIXER_DEFAULT_MAX_CAP_IN_MS)
}
#[cfg(feature = "serde")]
fn default_cap_jitter() -> Duration {
    Duration::from_millis(HOPR_MIXER_DEFAULT_CAP_JITTER_IN_MS)
}
#[cfg(feature = "serde")]
fn default_min_mix_occupancy() -> usize {
    HOPR_MIXER_DEFAULT_MIN_MIX_OCCUPANCY
}
#[cfg(feature = "serde")]
fn default_high_watermark() -> usize {
    HOPR_MIXER_CAPACITY / 2
}
#[cfg(feature = "serde")]
fn default_tick_floor() -> Duration {
    Duration::from_millis(HOPR_MIXER_DEFAULT_TICK_FLOOR_IN_MS)
}
#[cfg(feature = "serde")]
fn default_saturation_min_mean() -> Duration {
    Duration::from_millis(HOPR_MIXER_DEFAULT_SATURATION_MIN_MEAN_IN_MS)
}

/// Mixer configuration shared by every implementation.
///
/// Fields here apply to all engines; per-engine tuning lives in [`MixerType`], whose active
/// variant also selects which engine [`crate::create`] instantiates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MixerConfig {
    /// Preallocated buffer capacity; more items may still be inserted, triggering reallocation.
    #[default(HOPR_MIXER_CAPACITY)]
    #[validate(range(min = 1))]
    pub capacity: usize,
    /// Packet window over which the average-delay metric is smoothed.
    #[default(HOPR_MIXER_DELAY_METRIC_WINDOW)]
    #[cfg_attr(feature = "serde", serde(skip_serializing, default = "default_metric_delay_window"))]
    #[validate(range(min = 1))]
    pub metric_delay_window: u64,
    /// Engine selection plus its implementation-specific tuning.
    #[default(_code = "MixerType::default()")]
    #[cfg_attr(feature = "serde", serde(default))]
    #[validate(nested)]
    pub mixer_type: MixerType,
}

impl MixerConfig {
    /// Uniform random delay in `[min_delay, min_delay + delay_range]`, used by the uniform
    /// engines. The bounds come from the `Uniform` variant (zero for any other engine).
    pub fn random_delay(&self) -> Duration {
        let (min_delay, delay_range) = self.uniform_delay_bounds();
        let max_delay = min_delay.saturating_add(delay_range);
        let random_delay = if max_delay.as_millis() == 0 {
            0
        } else {
            hopr_types::crypto_random::random_integer(min_delay.as_millis() as u64, Some(max_delay.as_millis() as u64))
        };
        Duration::from_millis(random_delay)
    }

    /// The active engine's nominal maximum delay: the Poisson `max_cap`, or the uniform
    /// `min_delay + delay_range`. Useful for sizing delay-dependent parameters.
    pub fn nominal_max_delay(&self) -> Duration {
        match self.mixer_type {
            #[cfg(feature = "uniform-channel")]
            MixerType::Uniform(uniform) => uniform.min_delay.saturating_add(uniform.delay_range),
            #[cfg(feature = "poisson")]
            MixerType::Poisson(poisson) => poisson.max_cap,
            #[cfg(feature = "poisson-shared")]
            MixerType::PoissonShared(poisson) => poisson.max_cap,
            #[allow(unreachable_patterns)]
            _ => Duration::ZERO,
        }
    }

    /// The uniform engine's `(min_delay, delay_range)`, or zeros when the active engine is not `Uniform`.
    fn uniform_delay_bounds(&self) -> (Duration, Duration) {
        match self.mixer_type {
            #[cfg(feature = "uniform-channel")]
            MixerType::Uniform(uniform) => (uniform.min_delay, uniform.delay_range),
            #[allow(unreachable_patterns)]
            _ => (Duration::ZERO, Duration::ZERO),
        }
    }
}

/// Engine-specific mixer configuration; the active variant selects the engine.
///
/// Each variant is gated on the feature of the implementation it configures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MixerType {
    /// Uniform-delay min-heap channel.
    #[cfg(feature = "uniform-channel")]
    Uniform(UniformConfig),
    /// Exponential (Poisson) release engine on a dedicated thread.
    #[cfg(feature = "poisson")]
    Poisson(PoissonConfig),
    /// Exponential (Poisson) release engine sharing the pool on the consumer task.
    #[cfg(feature = "poisson-shared")]
    PoissonShared(PoissonConfig),
}

impl Default for MixerType {
    fn default() -> Self {
        #[cfg(feature = "poisson-shared")]
        return MixerType::PoissonShared(PoissonConfig::default());
        #[cfg(all(feature = "poisson", not(feature = "poisson-shared")))]
        return MixerType::Poisson(PoissonConfig::default());
        #[cfg(all(
            feature = "uniform-channel",
            not(feature = "poisson"),
            not(feature = "poisson-shared")
        ))]
        return MixerType::Uniform(UniformConfig::default());
        #[cfg(not(any(feature = "uniform-channel", feature = "poisson", feature = "poisson-shared")))]
        compile_error!("at least one mixer implementation feature must be enabled");
    }
}

// Hand-written because `validator::Validate` does not derive on enums; delegates to the nested
// `PoissonConfig` for the Poisson variants.
impl validator::Validate for MixerType {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            #[cfg(feature = "uniform-channel")]
            MixerType::Uniform(uniform) => uniform.validate(),
            #[cfg(feature = "poisson")]
            MixerType::Poisson(poisson) => poisson.validate(),
            #[cfg(feature = "poisson-shared")]
            MixerType::PoissonShared(poisson) => poisson.validate(),
            #[allow(unreachable_patterns)]
            _ => Ok(()),
        }
    }
}

/// Tuning parameters for the uniform-delay engines.
///
/// The delay is drawn uniformly from `[min_delay, min_delay + delay_range]`; a deterministic
/// `min_delay` floor is a uniform-only concept (it does not aid a memoryless Poisson mixer).
#[cfg(feature = "uniform-channel")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UniformConfig {
    /// Minimum delay before any packet is eligible for release.
    #[default(Duration::from_millis(HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub min_delay: Duration,
    /// Range above `min_delay`; the delay is uniform in `[min_delay, min_delay + delay_range]`.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub delay_range: Duration,
}

/// Tuning parameters for the exponential (Poisson) release engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoissonConfig {
    /// Hard latency cap: no packet waits longer than this, and the derived mean places
    /// [`HOPR_MIXER_CAP_PERCENTILE`] (~99%) of releases within it.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_MAX_CAP_IN_MS))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_cap", with = "humantime_serde"))]
    pub max_cap: Duration,
    /// Explicit mean holding delay; when zero the mean is derived from `max_cap` and
    /// [`HOPR_MIXER_CAP_PERCENTILE`]. Keep well below the cap: a mean at or above it collapses
    /// the exponential holding time onto the hard-cap release.
    #[default(Duration::from_millis(0))]
    #[cfg_attr(feature = "serde", serde(default, with = "humantime_serde"))]
    pub target_mean_delay: Duration,
    /// Width of the window over which hard-cap force-releases are smeared, removing the
    /// deterministic release instant at exactly the cap.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_CAP_JITTER_IN_MS))]
    #[cfg_attr(feature = "serde", serde(default = "default_cap_jitter", with = "humantime_serde"))]
    pub cap_jitter: Duration,
    /// Occupancy at or below which the coin is replaced by a deterministic minimum dwell of one
    /// mean, so a tiny buffer's packets overlap and mix instead of escaping the fast tail.
    #[default(HOPR_MIXER_DEFAULT_MIN_MIX_OCCUPANCY)]
    #[cfg_attr(feature = "serde", serde(default = "default_min_mix_occupancy"))]
    pub min_mix_occupancy: usize,
    /// Occupancy above which the overload valve shrinks the effective mean toward
    /// `saturation_min_mean` as the buffer nears `capacity`.
    #[default(HOPR_MIXER_CAPACITY / 2)]
    #[cfg_attr(feature = "serde", serde(default = "default_high_watermark"))]
    #[validate(range(min = 1))]
    pub high_watermark: usize,
    /// Lower bound on the adaptive wake interval, keeping the tick frequency low under load.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_TICK_FLOOR_IN_MS))]
    #[cfg_attr(feature = "serde", serde(default = "default_tick_floor", with = "humantime_serde"))]
    pub tick_floor: Duration,
    /// Floor the overload valve shrinks the mean toward, preserving a minimum mixing delay at
    /// saturation instead of collapsing to pass-through.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_SATURATION_MIN_MEAN_IN_MS))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_saturation_min_mean", with = "humantime_serde")
    )]
    pub saturation_min_mean: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "poisson-shared")]
    #[cfg(feature = "poisson-shared")]
    #[test]
    fn default_mixer_type_should_be_poisson_shared() {
        assert!(matches!(MixerType::default(), MixerType::PoissonShared(_)));
    }

    #[test]
    fn default_config_should_pass_validation() -> anyhow::Result<()> {
        MixerConfig::default().validate()?;
        Ok(())
    }

    #[test]
    fn zero_capacity_should_fail_validation() {
        let cfg = MixerConfig {
            capacity: 0,
            ..MixerConfig::default()
        };
        assert!(cfg.validate().is_err());
    }
}
