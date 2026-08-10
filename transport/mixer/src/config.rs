use std::time::Duration;

use validator::{Validate, ValidationError};

pub const HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS: u64 = 0;
pub const HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS: u64 = 20;
pub const HOPR_MIXER_DEFAULT_MAX_CAP_IN_MS: u64 = 20;
pub const HOPR_MIXER_DELAY_METRIC_WINDOW: u64 = 100;
pub const HOPR_MIXER_CAPACITY: usize = 20_000;

/// Default fraction of packets released before the Poisson hard cap. Fixes the cap:mean ratio
/// (`mean = cap / ln(1 / (1 - p))`), so at the default 20 ms cap the mean is ~4.3 ms.
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
fn default_cap_percentile() -> f64 {
    HOPR_MIXER_CAP_PERCENTILE
}
/// Reject a release percentile outside the open interval `(0, 1)`; the endpoints make the
/// cap:mean ratio zero or infinite.
fn validate_cap_percentile(percentile: f64) -> Result<(), ValidationError> {
    if percentile > 0.0 && percentile < 1.0 {
        Ok(())
    } else {
        Err(ValidationError::new(
            "cap_percentile must be in the open interval (0, 1)",
        ))
    }
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
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, validator::Validate)]
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
    /// Config selecting the uniform-delay engine with the given delay bounds.
    #[cfg(feature = "uniform-channel")]
    pub fn new_uniform(min_delay: Duration, delay_range: Duration) -> Self {
        Self {
            mixer_type: MixerType::Uniform(UniformConfig { min_delay, delay_range }),
            ..Self::default()
        }
    }

    /// Config selecting the dedicated-thread Poisson engine with the given delay anchor.
    #[cfg(feature = "poisson")]
    pub fn new_poisson(delay: PoissonDelay, cap_percentile: f64) -> Self {
        Self {
            mixer_type: MixerType::Poisson(PoissonConfig {
                delay,
                cap_percentile,
                ..PoissonConfig::default()
            }),
            ..Self::default()
        }
    }

    /// Config selecting the shared-pool Poisson engine with the given delay anchor.
    #[cfg(feature = "poisson-shared")]
    pub fn new_poisson_shared(delay: PoissonDelay, cap_percentile: f64) -> Self {
        Self {
            mixer_type: MixerType::PoissonShared(PoissonConfig {
                delay,
                cap_percentile,
                ..PoissonConfig::default()
            }),
            ..Self::default()
        }
    }

    /// The uniform engine's config, so the uniform channel and sink can obtain their
    /// `random_delay` bounds. When the active engine is not `Uniform` (a config built for a
    /// Poisson engine but fed to a uniform primitive), this yields zero delay rather than the
    /// uniform default range — preserving the historical "no Uniform config ⇒ no delay" contract.
    #[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
    pub(crate) fn uniform_config(&self) -> UniformConfig {
        match self.mixer_type {
            #[cfg(feature = "uniform-channel")]
            MixerType::Uniform(uniform) => uniform,
            #[allow(unreachable_patterns)]
            _ => UniformConfig {
                min_delay: Duration::ZERO,
                delay_range: Duration::ZERO,
            },
        }
    }
}

/// Engine-specific mixer configuration; the active variant selects the engine.
///
/// Each variant is gated on the feature of the implementation it configures.
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
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

#[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
impl UniformConfig {
    /// A uniform random delay drawn from `[min_delay, min_delay + delay_range]`.
    pub fn random_delay(&self) -> Duration {
        let max_delay = self.min_delay.saturating_add(self.delay_range);
        let random_delay = if max_delay.as_millis() == 0 {
            0
        } else {
            hopr_types::crypto_random::random_integer(
                self.min_delay.as_millis() as u64,
                Some(max_delay.as_millis() as u64),
            )
        };
        Duration::from_millis(random_delay)
    }
}

/// Which of the two bound quantities (hard cap / mean) the operator fixes; the other is derived
/// from it and [`PoissonConfig::cap_percentile`] via `mean = cap / ln(1 / (1 - percentile))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PoissonDelay {
    /// Hard latency cap; the mean is derived so `cap_percentile` of packets release before it.
    Cap(#[cfg_attr(feature = "serde", serde(with = "humantime_serde"))] Duration),
    /// Mean holding delay; the hard cap is derived so `cap_percentile` of packets release before it.
    Mean(#[cfg_attr(feature = "serde", serde(with = "humantime_serde"))] Duration),
}

impl Default for PoissonDelay {
    fn default() -> Self {
        PoissonDelay::Cap(Duration::from_millis(HOPR_MIXER_DEFAULT_MAX_CAP_IN_MS))
    }
}

impl PoissonDelay {
    /// Resolve to `(hard_cap, mean)`, deriving the unspecified quantity at `percentile`.
    /// A non-`(0, 1)` percentile (rejected by validation) degrades gracefully to zeros.
    pub fn resolve(&self, percentile: f64) -> (Duration, Duration) {
        let factor = (1.0 / (1.0 - percentile)).ln();
        if !factor.is_finite() || factor <= 0.0 {
            return match *self {
                PoissonDelay::Cap(cap) => (cap, Duration::ZERO),
                PoissonDelay::Mean(mean) => (Duration::ZERO, mean),
            };
        }
        match *self {
            PoissonDelay::Cap(cap) => (cap, Duration::from_secs_f64(cap.as_secs_f64() / factor)),
            PoissonDelay::Mean(mean) => (Duration::from_secs_f64(mean.as_secs_f64() * factor), mean),
        }
    }
}

/// Tuning parameters for the exponential (Poisson) release engines.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoissonConfig {
    /// Fixes either the hard cap or the mean; the other is derived (see [`PoissonDelay`]).
    #[default(_code = "PoissonDelay::default()")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub delay: PoissonDelay,
    /// Fraction of packets released before the hard cap, fixing the cap:mean ratio. Must be in
    /// `(0, 1)`.
    #[default(HOPR_MIXER_CAP_PERCENTILE)]
    #[cfg_attr(feature = "serde", serde(default = "default_cap_percentile"))]
    #[validate(custom(function = "validate_cap_percentile"))]
    pub cap_percentile: f64,
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
