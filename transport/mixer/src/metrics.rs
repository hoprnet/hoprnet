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
