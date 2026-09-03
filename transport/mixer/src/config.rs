use std::time::Duration;

use validator::{Validate, ValidationError};

pub const HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS: u64 = 0;
pub const HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS: u64 = 20;
pub const HOPR_MIXER_CAPACITY: usize = 20_000;
/// The average-delay metric window is sized to this many times the active engine's nominal max
/// delay (in ms), so a larger configured delay smooths the EMA over proportionally more packets.
pub const HOPR_MIXER_DELAY_METRIC_WINDOW_FACTOR: u64 = 5;

/// Default hard latency bound. Bounded-latency mode (the default, `target_occupancy = 0`) holds
/// mean delay to `max_delay / ln(1/miss_probability)` regardless of load; constant-privacy mode
/// (`target_occupancy > 0`) uses this only as an anti-starvation ceiling and typically releases
/// well below it.
pub const HOPR_MIXER_DEFAULT_MAX_DELAY_IN_MS: u64 = 20;
/// Default target `P(delay > max_delay)`. Fixes `g_max = ln(1/miss_probability)`, the per-entry
/// release-tag ceiling; at the default this gives `g_max ≈ 4.6` and, at the default `max_delay`,
/// a mean delay of `max_delay/g_max · E[g] ≈ 4.14 ms`.
pub const HOPR_MIXER_DEFAULT_MISS_PROBABILITY: f64 = 0.01;
/// Default `target_occupancy`: `0` disables the arrival term entirely (bounded-latency mode).
pub const HOPR_MIXER_DEFAULT_TARGET_OCCUPANCY: usize = 0;

#[cfg(feature = "serde")]
fn default_max_delay() -> Duration {
    Duration::from_millis(HOPR_MIXER_DEFAULT_MAX_DELAY_IN_MS)
}
#[cfg(feature = "serde")]
fn default_miss_probability() -> f64 {
    HOPR_MIXER_DEFAULT_MISS_PROBABILITY
}
/// Reject a miss probability outside the open interval `(0, 0.5)`: the lower bound excludes an
/// infinite `g_max`, and the upper bound keeps it a minority tail rather than most of the mass.
fn validate_miss_probability(miss_probability: f64) -> Result<(), ValidationError> {
    if miss_probability > 0.0 && miss_probability < 0.5 {
        Ok(())
    } else {
        Err(ValidationError::new(
            "miss_probability must be in the open interval (0, 0.5)",
        ))
    }
}
#[cfg(feature = "serde")]
fn default_target_occupancy() -> usize {
    HOPR_MIXER_DEFAULT_TARGET_OCCUPANCY
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

    /// Config selecting the Poisson timing-wheel engine with the given hard bound and miss
    /// probability, in bounded-latency mode (`target_occupancy = 0`: load-invariant mean delay).
    #[cfg(feature = "poisson")]
    pub fn new_poisson(max_delay: Duration, miss_probability: f64) -> Self {
        Self::new_poisson_constant_privacy(max_delay, miss_probability, 0)
    }

    /// Config selecting the Poisson timing-wheel engine in constant-privacy mode: the arrival
    /// term locks occupancy toward `target_occupancy` once load clears the crossover
    /// `target_occupancy / mu_max`, trading load-dependent latency (capped at `max_delay`) for a
    /// load-invariant anonymity set.
    #[cfg(feature = "poisson")]
    pub fn new_poisson_constant_privacy(max_delay: Duration, miss_probability: f64, target_occupancy: usize) -> Self {
        Self {
            mixer_type: MixerType::Poisson(PoissonConfig {
                max_delay,
                miss_probability,
                target_occupancy,
            }),
            ..Self::default()
        }
    }

    /// Packet window over which the `hopr_mixer_average_packet_delay` EMA is smoothed. Sized to
    /// [`HOPR_MIXER_DELAY_METRIC_WINDOW_FACTOR`]× the active engine's nominal max delay in ms
    /// (floored at 1), so the smoothing tracks the configured delay instead of a constant.
    pub fn metric_delay_window(&self) -> u64 {
        (HOPR_MIXER_DELAY_METRIC_WINDOW_FACTOR * self.nominal_max_delay().as_millis() as u64).max(1)
    }

    /// The active engine's nominal maximum delay: the timing-wheel hard bound `max_delay`, or the
    /// uniform `min_delay + delay_range`.
    fn nominal_max_delay(&self) -> Duration {
        match self.mixer_type {
            #[cfg(feature = "uniform-channel")]
            MixerType::Uniform(uniform) => uniform.min_delay.saturating_add(uniform.delay_range),
            #[cfg(feature = "poisson")]
            MixerType::Poisson(poisson) => poisson.max_delay,
            #[allow(unreachable_patterns)]
            _ => Duration::ZERO,
        }
    }

    /// The uniform engine's config, so the uniform channel and sink can obtain their
    /// `random_delay` bounds. When the active engine is not `Uniform` (a config built for a
    /// Poisson engine but fed to a uniform primitive), it falls back to the uniform defaults so
    /// the uniform channel still applies its usual mixing delay rather than degenerating to zero.
    #[cfg(any(feature = "uniform-channel", feature = "uniform-adapter"))]
    pub(crate) fn uniform_config(&self) -> UniformConfig {
        match self.mixer_type {
            #[cfg(feature = "uniform-channel")]
            MixerType::Uniform(uniform) => uniform,
            #[allow(unreachable_patterns)]
            _ => UniformConfig::default(),
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
    /// Virtual-clock timing-wheel release engine, pool shared on the consumer task.
    #[cfg(feature = "poisson")]
    Poisson(PoissonConfig),
}

impl Default for MixerType {
    fn default() -> Self {
        #[cfg(feature = "poisson")]
        return MixerType::Poisson(PoissonConfig::default());
        #[cfg(all(feature = "uniform-channel", not(feature = "poisson")))]
        return MixerType::Uniform(UniformConfig::default());
        #[cfg(not(any(feature = "uniform-channel", feature = "poisson")))]
        compile_error!("at least one mixer implementation feature must be enabled");
    }
}

// Hand-written because `validator::Validate` does not derive on enums; delegates to the nested
// `PoissonConfig` for the Poisson variant.
impl validator::Validate for MixerType {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            #[cfg(feature = "uniform-channel")]
            MixerType::Uniform(uniform) => uniform.validate(),
            #[cfg(feature = "poisson")]
            MixerType::Poisson(poisson) => poisson.validate(),
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

/// Tuning parameters for the virtual-clock timing-wheel release engine.
///
/// Every entry is tagged once, at enqueue, with a release threshold in dimensionless virtual
/// time; the pool's virtual clock advances from wall-clock time and, when `target_occupancy > 0`,
/// from arrivals. See `hopr_transport_mixer::pool` for the mechanism and its derivation.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoissonConfig {
    /// Hard latency bound: no entry can be held longer than `max_delay`, by construction of the
    /// release tag (see [`Self::miss_probability`]) — not a force-release rule. `max_delay = 0`
    /// disables mixing entirely (passthrough, FIFO order).
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_MAX_DELAY_IN_MS))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_delay", with = "humantime_serde"))]
    pub max_delay: Duration,
    /// Target `P(delay > max_delay)`. Must be in the open interval `(0, 0.5)`; fixes the
    /// per-entry tag ceiling `g_max = ln(1/miss_probability)` and, with it,
    /// `mu_max = max_delay / g_max` — the slowest the release clock ever holds an entry via
    /// wall-clock time alone.
    #[default(HOPR_MIXER_DEFAULT_MISS_PROBABILITY)]
    #[cfg_attr(feature = "serde", serde(default = "default_miss_probability"))]
    #[validate(custom(function = "validate_miss_probability"))]
    pub miss_probability: f64,
    /// Buffer occupancy the release clock's arrival term targets.
    ///
    /// `0` (the default) disables the arrival term: **bounded-latency mode**, mean delay is
    /// load-invariant and capped at `max_delay`.
    ///
    /// `> 0` blends in load: **constant-privacy mode**. Once the arrival rate `lambda` clears the
    /// crossover `lambda ≈ target_occupancy / mu_max`, occupancy locks toward `target_occupancy`
    /// and mean delay falls as load rises to sustain it — trading load-dependent latency (still
    /// capped at `max_delay`, now typically anti-starvation headroom rather than the working
    /// bound) for a load-invariant anonymity set. Below the crossover it degrades gracefully
    /// rather than stalling: occupancy sags below target and mean delay rises toward, but never
    /// past, `max_delay`.
    #[default(HOPR_MIXER_DEFAULT_TARGET_OCCUPANCY)]
    #[cfg_attr(feature = "serde", serde(default = "default_target_occupancy"))]
    pub target_occupancy: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "poisson")]
    #[test]
    fn default_mixer_type_should_be_poisson() {
        assert!(matches!(MixerType::default(), MixerType::Poisson(_)));
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

    #[test]
    fn miss_probability_outside_open_interval_should_fail_validation() {
        for miss_probability in [0.0, 0.5, -0.1, 1.0] {
            let cfg = PoissonConfig {
                miss_probability,
                ..PoissonConfig::default()
            };
            assert!(
                cfg.validate().is_err(),
                "miss_probability {miss_probability} should be rejected"
            );
        }
    }

    #[test]
    fn default_target_occupancy_should_select_bounded_latency_mode() {
        assert_eq!(PoissonConfig::default().target_occupancy, 0);
    }
}
