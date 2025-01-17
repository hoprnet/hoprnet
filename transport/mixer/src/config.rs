use std::time::Duration;

pub const HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS: u64 = 0;
pub const HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS: u64 = 200;
pub const HOPR_MIXER_DELAY_METRIC_WINDOW: u64 = 100;
pub const HOPR_MIXER_CAPACITY: usize = 20_000;

/// Mixer configuration.
#[derive(Debug, Clone, Copy, Eq, PartialEq, smart_default::SmartDefault)]
pub struct MixerConfig {
    /// The minimum delay introduced during mixing.
    #[default(Duration::from_millis(HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS))]
    pub min_delay: Duration,
    /// The range from the minimum delay to the maximum possible delay.
    #[default(Duration::from_millis(HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS))]
    pub delay_range: Duration,
    /// The capacity of the preallocated mixer buffer.
    ///
    /// It is possible to insert more items past the capacity, triggering
    /// a possible buffer reallocation.
    #[default(HOPR_MIXER_CAPACITY)]
    pub capacity: usize,
    #[default(HOPR_MIXER_DELAY_METRIC_WINDOW)]
    pub metric_delay_window: u64,
}

impl MixerConfig {
    /// Get a random delay duration from the specified minimum and maximum delay available
    /// inside the configuration.
    pub fn random_delay(&self) -> Duration {
        let max_delay = self.min_delay.saturating_add(self.delay_range);

        let random_delay = if max_delay.as_millis() == 0 {
            max_delay.as_millis() as u64
        } else {
            hopr_crypto_random::random_integer(self.min_delay.as_millis() as u64, Some(max_delay.as_millis() as u64))
        };

        Duration::from_millis(random_delay)
    }
}
