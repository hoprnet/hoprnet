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
    /// `None` or `Some(0)` fall back to the default, which is computed as
    /// `max(1, pool_thread_count - ENCODE_RESERVED_THREADS)` when the shared Rayon pool has
    /// been initialised, or `available_parallelism * 8` as a fallback when it has not.
    ///
    /// The default is deliberately lower than `output_concurrency` to reserve Rayon threads
    /// for outgoing packet encode (SURB generation). Flooding the pool with decode work would
    /// otherwise starve SURB production and collapse download throughput.
    pub input_concurrency: Option<usize>,
    /// Configuration of the packet acknowledgement processing
    #[validate(nested)]
    pub ack_config: AcknowledgementPipelineConfig,
}
