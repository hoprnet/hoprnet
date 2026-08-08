use std::time::Duration;

pub const HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS: u64 = 0;
pub const HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS: u64 = 20;
pub const HOPR_MIXER_DELAY_METRIC_WINDOW: u64 = 100;
pub const HOPR_MIXER_CAPACITY: usize = 20_000;

/// Percentile of packets that should be released before the hard cap when the
/// mean holding delay of the exponential (Poisson) release engine is derived
/// rather than set explicitly. `mean = delay_range / ln(1 / (1 - CAP_PERCENTILE))`.
///
/// The window is `delay_range`, not the whole cap: the memoryless clock only runs after the
/// `min_delay` floor (see `pool::sweep`), so the percentile must be solved over the eligible
/// window `cap - min_delay = delay_range`. At the default 20 ms range this yields a mean of
/// ~6.7 ms with ~5% of packets force-released at the cap. Override by setting
/// [`MixerConfig::target_mean_delay`].
pub const HOPR_MIXER_CAP_PERCENTILE: f64 = 0.95;
/// Default width of the jitter window smearing the hard-cap force-release.
pub const HOPR_MIXER_DEFAULT_CAP_JITTER_IN_MS: u64 = 2;
/// Default buffer occupancy at or below which the memoryless coin is replaced by a
/// deterministic minimum dwell of one mean.
///
/// When the buffer is this small, the exponential coin's fast tail could release a packet
/// before any anonymity set forms around it. Below this threshold each packet instead dwells
/// at least `mean`, guaranteeing the few packets present overlap and mix; above it, the clean
/// memoryless coin applies with no added latency floor.
pub const HOPR_MIXER_DEFAULT_MIN_MIX_OCCUPANCY: usize = 5;
/// Default lower bound on the adaptive wake interval of the Poisson engine.
pub const HOPR_MIXER_DEFAULT_TICK_FLOOR_IN_MS: u64 = 1;
/// Default floor the overload safety valve shrinks the effective mean toward (instead of
/// zero), so packets keep a minimum mixing delay even at capacity saturation.
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

/// Mixer configuration.
#[derive(Debug, Clone, Copy, Eq, PartialEq, smart_default::SmartDefault)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MixerConfig {
    /// The minimum delay introduced during mixing.
    #[default(Duration::from_millis(HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub min_delay: Duration,
    /// The range from the minimum delay to the maximum possible delay.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub delay_range: Duration,
    /// The capacity of the preallocated mixer buffer.
    ///
    /// It is possible to insert more items past the capacity, triggering
    /// a possible buffer reallocation.
    #[default(HOPR_MIXER_CAPACITY)]
    pub capacity: usize,
    #[default(HOPR_MIXER_DELAY_METRIC_WINDOW)]
    #[cfg_attr(feature = "serde", serde(skip_serializing, default = "default_metric_delay_window"))]
    pub metric_delay_window: u64,

    // --- Poisson (exponential-release) engine parameters; ignored by the uniform `channel`. ---
    /// Explicit mean holding delay of the exponential release engine.
    ///
    /// When zero (the default), the mean is derived from `delay_range` and
    /// [`HOPR_MIXER_CAP_PERCENTILE`] so that the configured percentile of packets is released
    /// before the cap. When set explicitly, keep it well below the cap (`min_delay +
    /// delay_range`): a mean at or above the cap pushes almost every packet onto the hard-cap
    /// branch, collapsing the exponential holding time this engine exists to produce.
    #[default(Duration::from_millis(0))]
    #[cfg_attr(feature = "serde", serde(default, with = "humantime_serde"))]
    pub target_mean_delay: Duration,
    /// Width of the jitter window over which hard-cap force-releases are smeared,
    /// removing the deterministic release instant at exactly the cap.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_CAP_JITTER_IN_MS))]
    #[cfg_attr(feature = "serde", serde(default = "default_cap_jitter", with = "humantime_serde"))]
    pub cap_jitter: Duration,
    /// Buffer occupancy at or below which the memoryless coin is replaced by a deterministic
    /// minimum dwell of one `mean`.
    ///
    /// Guarantees that when the buffer is small, packets dwell long enough to overlap and mix
    /// rather than escaping through the exponential's fast tail. Above this occupancy the
    /// memoryless coin applies. Set to `0` to always use the coin.
    #[default(HOPR_MIXER_DEFAULT_MIN_MIX_OCCUPANCY)]
    #[cfg_attr(feature = "serde", serde(default = "default_min_mix_occupancy"))]
    pub min_mix_occupancy: usize,
    /// Buffer occupancy above which the overload safety valve engages, shrinking
    /// the effective mean toward immediate release as occupancy nears `capacity`.
    #[default(HOPR_MIXER_CAPACITY / 2)]
    #[cfg_attr(feature = "serde", serde(default = "default_high_watermark"))]
    pub high_watermark: usize,
    /// Lower bound on the adaptive wake interval, keeping the actual tick
    /// frequency low even under load.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_TICK_FLOOR_IN_MS))]
    #[cfg_attr(feature = "serde", serde(default = "default_tick_floor", with = "humantime_serde"))]
    pub tick_floor: Duration,
    /// Floor for the effective mean under the overload safety valve.
    ///
    /// At capacity saturation the valve shrinks the mean toward this value instead of zero,
    /// so packets keep a minimum (exponential, memoryless) mixing delay rather than passing
    /// straight through. Bounded above by the base `mean`, so it never adds delay in normal
    /// operation.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_SATURATION_MIN_MEAN_IN_MS))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_saturation_min_mean", with = "humantime_serde")
    )]
    pub saturation_min_mean: Duration,
}

impl MixerConfig {
    /// Get a random delay duration from the specified minimum and maximum delay available
    /// inside the configuration.
    pub fn random_delay(&self) -> Duration {
        let max_delay = self.min_delay.saturating_add(self.delay_range);

        let random_delay = if max_delay.as_millis() == 0 {
            max_delay.as_millis() as u64
        } else {
            hopr_types::crypto_random::random_integer(
                self.min_delay.as_millis() as u64,
                Some(max_delay.as_millis() as u64),
            )
        };

        Duration::from_millis(random_delay)
    }

    /// The hard latency cap: no packet waits longer than this. Equals
    /// `min_delay + delay_range`.
    pub fn cap(&self) -> Duration {
        self.min_delay.saturating_add(self.delay_range)
    }

    /// Whether the mixer degenerates into an order-preserving pass-through
    /// (no delay configured at all).
    pub fn is_passthrough(&self) -> bool {
        self.cap().is_zero()
    }

    /// Mean holding delay of the exponential release engine.
    ///
    /// Returns [`Self::target_mean_delay`] when set explicitly; otherwise derives it from
    /// `delay_range` so that [`HOPR_MIXER_CAP_PERCENTILE`] of packets are released before the
    /// cap: `mean = delay_range / ln(1 / (1 - percentile))`. The window is `delay_range` (not the
    /// whole cap) because the memoryless clock only runs over the eligible age past `min_delay`
    /// (see `pool::sweep`); deriving from the full cap would undershoot the target percentile
    /// whenever `min_delay > 0`.
    pub fn mean(&self) -> Duration {
        if !self.target_mean_delay.is_zero() {
            return self.target_mean_delay;
        }

        let window = self.delay_range.as_secs_f64();
        let factor = (1.0 / (1.0 - HOPR_MIXER_CAP_PERCENTILE)).ln();
        if window <= 0.0 || factor <= 0.0 {
            return Duration::ZERO;
        }

        Duration::from_secs_f64(window / factor)
    }

    /// Effective mean under the overload safety valve.
    ///
    /// Equals [`Self::mean`] up to `high_watermark`; beyond it the mean shrinks
    /// linearly as occupancy approaches `capacity`, providing back-pressure relief under
    /// bursts. It is floored at [`Self::saturation_min_mean`] (capped by the base mean) so a
    /// minimum mixing delay is preserved even at capacity saturation, rather than collapsing
    /// to immediate pass-through.
    pub fn mean_for(&self, occupancy: usize) -> Duration {
        let base = self.mean();
        // Clamp the watermark below `capacity`: a config that sets a small `capacity` but leaves
        // the default `high_watermark` (e.g. capacity 100, watermark 10_000) would otherwise put
        // the trip point above any reachable occupancy, silently disabling the valve.
        let watermark = self.high_watermark.min(self.capacity.saturating_sub(1));
        if occupancy <= watermark {
            return base;
        }

        let low = watermark.max(1);
        let high = self.capacity.max(low + 1);
        let fraction = (occupancy.saturating_sub(low) as f64 / (high - low) as f64).clamp(0.0, 1.0);

        let scaled = Duration::from_secs_f64(base.as_secs_f64() * (1.0 - fraction));
        // Never let the valve reduce the mean below the configured saturation floor, but the
        // floor must never exceed the base mean (so it is inert in normal operation).
        scaled.max(self.saturation_min_mean.min(base))
    }

    /// Per-evaluation release probability of the memoryless exponential clock:
    /// `1 - e^(-delta / mean)`, where `delta` is the time since the item was last
    /// evaluated. Independent of the wake cadence, which is the property that
    /// lets the tick be low and adaptive without distorting the distribution.
    pub fn release_probability(&self, delta: Duration, mean: Duration) -> f64 {
        let mean = mean.as_secs_f64();
        if mean <= 0.0 {
            return 1.0;
        }

        1.0 - (-delta.as_secs_f64() / mean).exp()
    }

    /// Adaptive wake interval as a function of buffer occupancy: `mean / occupancy`
    /// clamped to `[tick_floor, mean]`. More buffered packets ⇒ shorter interval.
    pub fn adaptive_interval(&self, occupancy: usize) -> Duration {
        let mean = self.mean();
        let floor = self.tick_floor;
        let ceil = mean.max(floor);
        let raw = Duration::from_secs_f64(mean.as_secs_f64() / occupancy.max(1) as f64);

        raw.clamp(floor, ceil)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: Duration, b_ms: f64, tol_ms: f64) -> bool {
        (a.as_secs_f64() * 1000.0 - b_ms).abs() <= tol_ms
    }

    #[test]
    fn mean_should_be_derived_from_cap_and_percentile_by_default() {
        // cap = 20 ms, mean = cap / ln(1 / (1 - 0.95)) = 20 / ln(20) ≈ 6.68 ms.
        let cfg = MixerConfig::default();
        assert!(approx(cfg.cap(), 20.0, 0.001));
        assert!(approx(cfg.mean(), 6.68, 0.05), "derived mean was {:?}", cfg.mean());
    }

    #[test]
    fn explicit_target_mean_delay_should_override_derivation() {
        let cfg = MixerConfig {
            target_mean_delay: Duration::from_millis(10),
            ..MixerConfig::default()
        };
        assert_eq!(cfg.mean(), Duration::from_millis(10));
    }

    #[test]
    fn safety_valve_should_shrink_mean_above_high_watermark() {
        let cfg = MixerConfig::default();
        let base = cfg.mean();

        assert_eq!(
            cfg.mean_for(cfg.high_watermark),
            base,
            "mean is unchanged up to the high watermark"
        );
        assert!(
            cfg.mean_for(cfg.high_watermark + 1) < base,
            "mean shrinks past the high watermark"
        );
        // The valve floors at the saturation minimum instead of collapsing to zero.
        let floor = cfg.saturation_min_mean;
        assert!(
            floor > Duration::ZERO,
            "a minimum mixing delay is preserved under saturation"
        );
        assert_eq!(
            cfg.mean_for(cfg.capacity),
            floor,
            "mean is floored at the saturation minimum at capacity"
        );
        assert_eq!(
            cfg.mean_for(cfg.capacity * 2),
            floor,
            "mean stays floored beyond capacity"
        );
    }

    #[test]
    fn release_probability_should_follow_the_exponential_clock() {
        let cfg = MixerConfig::default();
        let mean = Duration::from_millis(10);

        assert_eq!(cfg.release_probability(Duration::ZERO, mean), 0.0);
        // 1 - e^-1 ≈ 0.632 after one mean has elapsed.
        assert!((cfg.release_probability(mean, mean) - 0.6321).abs() < 1e-3);
        // Degenerate mean releases unconditionally.
        assert_eq!(cfg.release_probability(mean, Duration::ZERO), 1.0);
    }

    #[test]
    fn adaptive_interval_should_shorten_with_occupancy() {
        let cfg = MixerConfig::default();
        let mean = cfg.mean();

        // One item: interval == mean (clamped to the ceiling).
        assert_eq!(cfg.adaptive_interval(1), mean);
        // Many items: interval clamps down to the tick floor.
        assert_eq!(cfg.adaptive_interval(100_000), cfg.tick_floor);
        // Monotonic non-increasing in occupancy.
        assert!(cfg.adaptive_interval(10) >= cfg.adaptive_interval(100));
    }

    #[test]
    fn passthrough_should_be_detected_when_no_delay_configured() {
        assert!(!MixerConfig::default().is_passthrough());
        let cfg = MixerConfig {
            min_delay: Duration::ZERO,
            delay_range: Duration::ZERO,
            ..MixerConfig::default()
        };
        assert!(cfg.is_passthrough());
    }
}
