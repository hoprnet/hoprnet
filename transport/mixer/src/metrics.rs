//! Telemetry gauges shared by all mixer implementations.
//!
//! Only present with the `telemetry` feature (and never under `test`); callers guard their
//! use sites with the same `cfg`.

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
}

/// Feed one packet delay (ms) into the `hopr_mixer_average_packet_delay` gauge as an exponential
/// moving average with weight `1 / window`, so the smoothing lives in one place. `window` is
/// clamped to at least 1 so a misconfigured zero window can't divide-by-zero and poison the gauge
/// with NaN.
#[cfg(all(feature = "telemetry", not(test)))]
pub(crate) fn record_average_delay(delay_ms: f64, window: u64) {
    let weight = 1.0f64 / window.max(1) as f64;
    METRIC_MIXER_AVERAGE_DELAY.set(weight * delay_ms + (1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get());
}
