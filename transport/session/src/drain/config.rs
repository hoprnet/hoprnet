use std::time::Duration;

/// Configuration of the post-closure SURB drain (Exit side).
///
/// # Tuning guidance
///
/// | Field | Ties to | Tuning |
/// |---|---|---|
/// | `ack_grace` | `SsaReconstructorConfig::max_ack_await_time` | Must be ≥ `max_ack_await_time` or honest late acks are ignored (validated by [`super::validate_pix_drain`]). |
/// | `max_drain_time` | `SsaReconstructorConfig::incomplete_ssa_lifetime` | Must be < `incomplete_ssa_lifetime` or SSA builders expire mid-drain. |
/// | `cost_safety_factor` | `packet_price` × deficit | Multiplier against the raw ticket cost. 1.0 means break-even; higher values reserve margin against price variance. |
/// | `drain_rate_packets_per_sec` | Network RTT, relay capacity | 100 is conservative for most networks. Increase if the reconstructor accumulates shares faster than the drain sends packets. |
/// | `surplus_slack_packets` | Expected surplus/duplicate share rate | 64 covers typical surplus shares from the PIX protocol. Increase if many relays return duplicates. |
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault)]
pub struct SurbDrainConfig {
    /// Whether the drainer is enabled.
    ///
    /// Default is `false`.
    #[default(false)]
    pub enabled: bool,

    /// Maximum time a single drain may run before being aborted.
    ///
    /// Default is 5 minutes.
    #[default(Duration::from_secs(300))]
    pub max_drain_time: Duration,

    /// Rate of drain packets sent per second.
    ///
    /// Default is 100.
    #[default(100)]
    pub drain_rate_packets_per_sec: usize,

    /// Maximum number of concurrent drains across all closed sessions.
    ///
    /// Default is 8.
    #[default(8)]
    pub max_concurrent_drains: usize,

    /// Grace period after which a drain aborts if no useful progress is observed.
    ///
    /// Default is 60 seconds.
    #[default(Duration::from_secs(60))]
    pub ack_grace: Duration,

    /// Safety factor applied to the economic cost calculation.
    ///
    /// Must be >= 1.0.
    #[default(1.0)]
    pub cost_safety_factor: f64,

    /// Surplus slack — additional packets allocated beyond the deficit
    /// to cover duplicates and surplus shares.
    ///
    /// Default is 64.
    #[default(64)]
    pub surplus_slack_packets: u64,
}
