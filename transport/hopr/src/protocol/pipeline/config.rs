//! Configuration structures for the HOPR packet processing pipeline.

use validator::{Validate, ValidationError, ValidationErrors};

fn default_ack_buffer_interval() -> std::time::Duration {
    std::time::Duration::from_millis(200)
}

fn default_ack_grouping_capacity() -> usize {
    5
}

fn default_ticket_ack_buffer_size() -> usize {
    50_000
}

fn default_ack_out_buffer_size() -> usize {
    50_000
}

/// Configuration for the acknowledgement processing pipeline.
#[derive(Debug, Copy, Clone, smart_default::SmartDefault, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AcknowledgementPipelineConfig {
    /// Interval for which to wait to buffer acknowledgements before sending them out.
    ///
    /// Default is 200 ms.
    #[default(default_ack_buffer_interval())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_ack_buffer_interval", with = "humantime_serde")
    )]
    pub ack_buffer_interval: std::time::Duration,
    /// Initial capacity when grouping outgoing acknowledgements.
    ///
    /// If set too low, it causes additional reallocations in the outgoing acknowledgement processing pipeline.
    /// The value should grow if `ack_buffer_interval` grows.
    ///
    /// Default is 5.
    #[default(default_ack_grouping_capacity())]
    #[cfg_attr(feature = "serde", serde(default = "default_ack_grouping_capacity"))]
    pub ack_grouping_capacity: usize,
    /// Capacity of the `incoming_ack` MPSC channel carrying received acknowledgements
    /// to the ticket-ack processing pipeline.
    ///
    /// The previous hardcoded value of 1_000_000 pre-allocated ~MBs of ring buffer per node even
    /// though real-world throughput rarely saturates more than a few thousand entries. Let the
    /// 50 ms sink timeouts (`QUEUE_SEND_TIMEOUT`) propagate backpressure instead.
    ///
    /// The default is 50 000.
    #[default(default_ticket_ack_buffer_size())]
    #[cfg_attr(feature = "serde", serde(default = "default_ticket_ack_buffer_size"))]
    pub ticket_ack_buffer_size: usize,
    /// Capacity of the `outgoing_ack` MPSC channel carrying acknowledgements to be sent back
    /// to the previous hop.
    ///
    /// The default is 50 000. See [`ticket_ack_buffer_size`](Self::ticket_ack_buffer_size) for the
    /// rationale on why this is smaller than the original hardcoded 1_000_000.
    #[default(default_ack_out_buffer_size())]
    #[cfg_attr(feature = "serde", serde(default = "default_ack_out_buffer_size"))]
    pub ack_out_buffer_size: usize,
    /// Maximum concurrency when processing incoming (received) acknowledgements.
    ///
    /// `None` or `Some(0)` both fall back to a default of 10.
    pub ack_input_concurrency: Option<usize>,
    /// Maximum concurrency when processing outgoing (sent-back) acknowledgements.
    ///
    /// `None` or `Some(0)` both fall back to a default of 10.
    pub ack_output_concurrency: Option<usize>,
}

// Requires manual implementation due to https://github.com/Keats/validator/issues/285
impl Validate for AcknowledgementPipelineConfig {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.ack_grouping_capacity == 0 {
            errors.add("ack_grouping_capacity", ValidationError::new("must be greater than 0"));
        }
        if self.ack_buffer_interval < std::time::Duration::from_millis(10) {
            errors.add("ack_buffer_interval", ValidationError::new("must be at least 10 ms"));
        }
        if self.ticket_ack_buffer_size == 0 {
            errors.add("ticket_ack_buffer_size", ValidationError::new("must be greater than 0"));
        }
        if self.ack_out_buffer_size == 0 {
            errors.add("ack_out_buffer_size", ValidationError::new("must be greater than 0"));
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn default_arbitration_enabled() -> bool {
    true
}
fn default_arbitration_occupancy_pct() -> u32 {
    75
}
fn default_arbitration_encode_reserve_pct() -> u32 {
    50
}

/// Arbitration of the shared Rayon pool, protecting the encode path (SPHINX wrap + SURB generation)
/// from decode floods (SPHINX peel — relay forwarding + exit termination).
///
/// Asymmetric and occupancy-gated: only decode is ever throttled, and only when the pool is
/// saturated *and* encode work is present, so pure forwarding and unsaturated nodes are untouched.
/// Enforced inside `spawn_decode_blocking` (see `hopr_utils::parallelize::cpu::configure_arbitration`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct PoolArbitrationConfig {
    /// When `false`, decode is never throttled (the pool is shared first-come-first-served).
    #[default(default_arbitration_enabled())]
    #[cfg_attr(feature = "serde", serde(default = "default_arbitration_enabled"))]
    pub enabled: bool,
    /// Pool occupancy (percent of threads actually running) at or above which decode admission may
    /// engage. Below it, decode is never throttled.
    #[default(default_arbitration_occupancy_pct())]
    #[validate(range(min = 1, max = 100))]
    #[cfg_attr(feature = "serde", serde(default = "default_arbitration_occupancy_pct"))]
    pub occupancy_pct: u32,
    /// Share of the pool (percent) that decode yields to encode when both contend under saturation.
    #[default(default_arbitration_encode_reserve_pct())]
    #[validate(range(min = 1, max = 100))]
    #[cfg_attr(feature = "serde", serde(default = "default_arbitration_encode_reserve_pct"))]
    pub encode_reserve_pct: u32,
}

impl PoolArbitrationConfig {
    /// Maps this flat (serde-friendly) config onto the pool arbiter's [`ArbitrationConfig`] enum,
    /// where the disabled state carries no tuning percentages.
    pub fn to_arbitration(&self) -> hopr_utils::parallelize::cpu::ArbitrationConfig {
        use hopr_utils::parallelize::cpu::ArbitrationConfig;
        if self.enabled {
            ArbitrationConfig::Enabled {
                occupancy_pct: self.occupancy_pct,
                encode_reserve_pct: self.encode_reserve_pct,
            }
        } else {
            ArbitrationConfig::Disabled
        }
    }
}

/// Overall configuration of the input/output packet processing pipeline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct PacketPipelineConfig {
    /// Maximum concurrency when processing outgoing packets.
    ///
    /// `None` or `Some(0)` both fall back to the default (available parallelism * 8).
    pub output_concurrency: Option<usize>,
    /// Maximum concurrency when processing incoming packets (SPHINX decode).
    ///
    /// `None` or `Some(0)` both fall back to the default (available parallelism * 8), the same as
    /// `output_concurrency`. Encode is no longer protected by throttling this queue depth below
    /// output's (which regressed relay forwarding — #8246); protection now lives in the shared-pool
    /// arbiter (see [`PoolArbitrationConfig`]), which throttles decode *admission* only under a
    /// genuine flood, leaving pure forwarding at full concurrency.
    pub input_concurrency: Option<usize>,
    /// How long routing resolution keeps waiting for a return path's SURBs before giving up on the
    /// packet.
    ///
    /// `None` falls back to the default of 6 s. `Some(0)` disables the wait entirely, dropping a
    /// return packet the first time its SURBs are missing.
    ///
    /// The right value trades two failures against each other. Too short loses data on a session
    /// whose SURB pool is only momentarily empty, which is why the wait exists at all. Too long
    /// stalls **every** packet the node originates, not just this one: resolution preserves
    /// submission order, so an unresolvable packet withholds everything behind it, and a
    /// counterparty that has gone away never sends another SURB. An unbounded wait here took a
    /// production exit's entire egress down for 1h44m while it still forwarded and acknowledged
    /// normally.
    ///
    /// **Set this if you know your session's frame timeout.** The default errs long, because a
    /// library cannot know it; a packet held past that timeout is discarded by the receiver anyway,
    /// so the wait is pure stall from then on. hoprd, whose sessions time frames out at 3 s,
    /// configures 1 s.
    #[cfg_attr(feature = "serde", serde(default, with = "humantime_serde"))]
    pub surb_resolution_wait: Option<std::time::Duration>,
    /// Configuration of the packet acknowledgement processing
    #[validate(nested)]
    pub ack_config: AcknowledgementPipelineConfig,
    /// Arbitration of the shared Rayon pool between encode and decode.
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub arbitration: PoolArbitrationConfig,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    /// Everything this struct requires in a document, minus the wait under test.
    ///
    /// Only `surb_resolution_wait` carries `serde(default)`, so the surrounding fields have to be
    /// written out; `deny_unknown_fields` means the document must otherwise be exact.
    const REQUIRED_FIELDS: &str = "output_concurrency: null\ninput_concurrency: null\nack_config:\n  \
                                   ack_input_concurrency: null\n  ack_output_concurrency: null\n";

    fn parse(wait: Option<&str>) -> PacketPipelineConfig {
        let doc = match wait {
            Some(value) => format!("{REQUIRED_FIELDS}surb_resolution_wait: {value}\n"),
            None => REQUIRED_FIELDS.to_string(),
        };
        serde_saphyr::from_str(&doc).unwrap_or_else(|e| panic!("config must parse:\n{doc}\n{e}"))
    }

    /// The wait has to survive a config file, in the human-readable form the rest of this config
    /// uses. `Option<Duration>` through `humantime_serde` is easy to get wrong in a way that only
    /// shows up when someone's YAML is silently ignored.
    #[test]
    fn the_surb_resolution_wait_should_round_trip_through_yaml() {
        assert_eq!(
            Some(std::time::Duration::from_secs(2)),
            parse(Some("2s")).surb_resolution_wait,
            "a duration string must reach the field"
        );
        assert_eq!(
            None,
            parse(None).surb_resolution_wait,
            "an omitted wait must stay unset so the default applies"
        );
        assert_eq!(
            Some(std::time::Duration::ZERO),
            parse(Some("0s")).surb_resolution_wait,
            "zero must reach the code as zero, not as unset"
        );
    }

    /// The `SmartDefault` derive and the serde field defaults share the `default_arbitration_*` fns,
    /// so they can't diverge — pin the values the arbiter ships with, and that a default config
    /// validates (the percentages sit inside the `1..=100` range).
    #[test]
    fn pool_arbitration_config_defaults_are_enabled_75_50_and_valid() {
        let cfg = PoolArbitrationConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.occupancy_pct, 75);
        assert_eq!(cfg.encode_reserve_pct, 50);
        assert!(
            cfg.validate().is_ok(),
            "the default arbitration config must pass validation"
        );
    }
}
