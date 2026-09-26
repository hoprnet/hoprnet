//! Telemetry gauges and the packet-delay histogram shared by all mixer implementations.
//!
//! Only present with the `telemetry` feature (and never under `test`); callers guard their
//! use sites with the same `cfg`.

/// Packet count the window-miss ratio EMA is smoothed over. Independent of the configurable
/// `metric_delay_window` (which tracks the active `max_delay` and would make the miss-ratio EMA's
/// responsiveness swing with configuration); a fixed window keeps this gauge comparable across
/// configs.
#[cfg(all(feature = "telemetry", not(test)))]
const WINDOW_MISS_EMA_PACKETS: u64 = 200;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    pub static ref METRIC_QUEUE_SIZE: hopr_types::telemetry::SimpleGauge =
        hopr_types::telemetry::SimpleGauge::new("hopr_mixer_queue_size", "Current mixer queue size").unwrap();
    pub static ref METRIC_MIXER_AVERAGE_DELAY: hopr_types::telemetry::SimpleGauge =
        hopr_types::telemetry::SimpleGauge::new(
            "hopr_mixer_average_packet_delay",
            "Average mixer packet delay averaged over a packet window"
        )
        .unwrap();
    pub static ref METRIC_MIXER_PACKET_DELAY: hopr_types::telemetry::SimpleHistogram =
        hopr_types::telemetry::SimpleHistogram::new(
            "hopr_mixer_packet_delay_ms",
            "Distribution of per-packet mixer output delay in milliseconds",
            vec![1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0],
        )
        .unwrap();
    /// `e * queue_size`, the live effective-anonymity-set estimate (`2^H`) for a memoryless-shaped
    /// release distribution — the operational signal that the mixer has stopped buying anything
    /// once traffic is too thin (see the PR description for the derivation).
    pub static ref METRIC_MIXER_ANONYMITY_SET: hopr_types::telemetry::SimpleGauge =
        hopr_types::telemetry::SimpleGauge::new(
            "hopr_mixer_effective_anonymity_set",
            "Estimated effective anonymity set (2^H) of the mixer's current occupancy"
        )
        .unwrap();
    /// EMA of `P(delay > max_delay)`, the live check that the configured `miss_probability`
    /// actually holds in production.
    pub static ref METRIC_MIXER_WINDOW_MISS_RATIO: hopr_types::telemetry::SimpleGauge =
        hopr_types::telemetry::SimpleGauge::new(
            "hopr_mixer_window_miss_ratio",
            "EMA fraction of packets whose realized delay exceeded the configured hard bound"
        )
        .unwrap();
}

/// Record one packet's output delay (ms): observe it into the `hopr_mixer_packet_delay_ms`
/// distribution and feed the `hopr_mixer_average_packet_delay` EMA gauge (weight `1 / window`).
/// `window` is clamped to at least 1 so a misconfigured zero can't divide-by-zero into NaN.
#[cfg(all(feature = "telemetry", not(test)))]
pub(crate) fn record_packet_delay(delay_ms: f64, window: u64) {
    METRIC_MIXER_PACKET_DELAY.observe(delay_ms);
    let weight = 1.0f64 / window.max(1) as f64;
    METRIC_MIXER_AVERAGE_DELAY.set(weight * delay_ms + (1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get());
}

/// Set `hopr_mixer_effective_anonymity_set` from the current queue size. Euler's number, not the
/// queue-size-derived shape factor `k(x)`, is used as the multiplier: `e` is the memoryless-clock
/// (`x -> infinity`) limit and a conservative (slight over-)estimate across the configured range.
#[cfg(all(feature = "telemetry", not(test)))]
pub(crate) fn record_anonymity_set(queue_size: usize) {
    METRIC_MIXER_ANONYMITY_SET.set(std::f64::consts::E * queue_size as f64);
}

/// Feed one release's window-miss outcome into the `hopr_mixer_window_miss_ratio` EMA.
#[cfg(all(feature = "telemetry", not(test)))]
pub(crate) fn record_window_miss(exceeded_window: bool) {
    let weight = 1.0f64 / WINDOW_MISS_EMA_PACKETS as f64;
    let sample = if exceeded_window { 1.0 } else { 0.0 };
    METRIC_MIXER_WINDOW_MISS_RATIO.set(weight * sample + (1.0f64 - weight) * METRIC_MIXER_WINDOW_MISS_RATIO.get());
}
