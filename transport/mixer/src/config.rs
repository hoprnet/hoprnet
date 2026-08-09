use std::time::Duration;

pub const HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS: u64 = 0;
pub const HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS: u64 = 20;
pub const HOPR_MIXER_DELAY_METRIC_WINDOW: u64 = 100;
pub const HOPR_MIXER_CAPACITY: usize = 20_000;

/// Percentile of packets released before the hard cap when the Poisson mean is derived rather
/// than set explicitly: `mean = delay_range / ln(1 / (1 - CAP_PERCENTILE))`.
///
/// The window is `delay_range`, not the whole cap, because the memoryless clock only runs after
/// the `min_delay` floor; deriving from the cap would undershoot the percentile when
/// `min_delay > 0`. At the default 20 ms range this gives a ~6.7 ms mean with ~5% at the cap.
pub const HOPR_MIXER_CAP_PERCENTILE: f64 = 0.95;
pub const HOPR_MIXER_DEFAULT_CAP_JITTER_IN_MS: u64 = 2;
pub const HOPR_MIXER_DEFAULT_MIN_MIX_OCCUPANCY: usize = 5;
pub const HOPR_MIXER_DEFAULT_TICK_FLOOR_IN_MS: u64 = 1;
pub const HOPR_MIXER_DEFAULT_SATURATION_MIN_MEAN_IN_MS: u64 = 1;

#[cfg(feature = "serde")]
fn default_metric_delay_window() -> u64 {
    HOPR_MIXER_DELAY_METRIC_WINDOW
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MixerConfig {
    /// Preallocated buffer capacity; more items may still be inserted, triggering reallocation.
    #[default(HOPR_MIXER_CAPACITY)]
    pub capacity: usize,
    /// Packet window over which the average-delay metric is smoothed.
    #[default(HOPR_MIXER_DELAY_METRIC_WINDOW)]
    #[cfg_attr(feature = "serde", serde(skip_serializing, default = "default_metric_delay_window"))]
    pub metric_delay_window: u64,
    /// Minimum delay before any packet is eligible for release.
    #[default(Duration::from_millis(HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub min_delay: Duration,
    /// Range added to `min_delay` to form the hard latency cap.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub delay_range: Duration,
    /// Engine selection plus its implementation-specific tuning.
    #[default(_code = "MixerType::default()")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub mixer_type: MixerType,
}

impl MixerConfig {
    /// Hard latency cap: no packet waits longer than `min_delay + delay_range`.
    pub fn cap(&self) -> Duration {
        self.min_delay.saturating_add(self.delay_range)
    }

    /// Whether the mixer degenerates into an order-preserving pass-through (no delay at all).
    pub fn is_passthrough(&self) -> bool {
        self.cap().is_zero()
    }

    /// Uniform random delay in `[min_delay, min_delay + delay_range]`, used by the uniform engine.
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

/// Engine-specific mixer configuration; the active variant selects the engine.
///
/// Each variant is gated on the feature of the implementation it configures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MixerType {
    /// Uniform-delay min-heap channel.
    #[cfg(feature = "uniform-channel")]
    Uniform,
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
        return MixerType::Uniform;
        #[cfg(not(any(feature = "uniform-channel", feature = "poisson", feature = "poisson-shared")))]
        compile_error!("at least one mixer implementation feature must be enabled");
    }
}

/// Tuning parameters for the exponential (Poisson) release engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoissonConfig {
    /// Explicit mean holding delay; when zero the mean is derived from `delay_range` and
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

    #[test]
    fn default_mixer_type_should_be_poisson_shared() {
        assert!(matches!(MixerType::default(), MixerType::PoissonShared(_)));
    }

    #[test]
    fn cap_should_be_min_delay_plus_range() {
        let cfg = MixerConfig::default();
        assert_eq!(cfg.cap(), cfg.min_delay + cfg.delay_range);
    }

    #[test]
    fn passthrough_should_be_detected_only_when_no_delay_is_configured() {
        assert!(!MixerConfig::default().is_passthrough());
        let cfg = MixerConfig {
            min_delay: Duration::ZERO,
            delay_range: Duration::ZERO,
            ..MixerConfig::default()
        };
        assert!(cfg.is_passthrough());
    }
}
