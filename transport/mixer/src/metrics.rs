//! Telemetry gauges and the packet-delay histogram shared by all mixer implementations.
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
    pub static ref METRIC_MIXER_PACKET_DELAY: hopr_types::telemetry::SimpleHistogram =
        hopr_types::telemetry::SimpleHistogram::new(
            "hopr_mixer_packet_delay_ms",
            "Distribution of per-packet mixer output delay in milliseconds",
            vec![1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0],
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
