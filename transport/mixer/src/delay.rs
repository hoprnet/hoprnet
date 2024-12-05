use std::time::Duration;

pub const HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS: u64 = 20;
pub const HOPR_MIXER_DEFAULT_DELAY_DIFFERENCE_IN_MS: u64 = 180;

/// Get a random delay duration from the specified minimum and maximum delay specified in milliseconds
/// by the environment variables `HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS` and `HOPR_INTERNAL_MIXER_MAXIMUM_DELAY_IN_MS`.
pub fn random_delay() -> Duration {
    let minimum = std::env::var("HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS")
        .map(|v| {
            v.trim()
                .parse::<u64>()
                .unwrap_or(HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS)
        })
        .unwrap_or(HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS);

    let random_delay = hopr_crypto_random::random_integer(
        minimum,
        Some(
            std::env::var("HOPR_INTERNAL_MIXER_MAXIMUM_DELAY_IN_MS")
                .map(|v| {
                    v.trim()
                        .parse::<u64>()
                        .unwrap_or(minimum + HOPR_MIXER_DEFAULT_DELAY_DIFFERENCE_IN_MS)
                })
                .unwrap_or(minimum + HOPR_MIXER_DEFAULT_DELAY_DIFFERENCE_IN_MS),
        ),
    );

    Duration::from_millis(random_delay)
}
