//! PIX supervisor module for incoming PIX-enabled sessions.
//!
//! Contains the deterministic [`SessionPixSupervisor`] core, the [`ServiceGate`]
//! for bounded predeposit egress, and the per-session actor that serializes
//! lifecycle events.

use std::time::Duration;

use hopr_api::{HoprBalance, types::internal::prelude::HoprPseudonym};
use hopr_protocol_pix::{SsaId, SsaReconstructorConfig, SsaRecoveryProgress};

use crate::errors::TransportSessionError;

mod gate;
mod notify;
mod supervisor;
mod worker;

// ---------------------------------------------------------------------------
// SupervisorConfig
// ---------------------------------------------------------------------------

/// Configuration for the [`SessionPixSupervisor`].
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault)]
pub struct SupervisorConfig {
    /// Maximum time to wait for the SSA to be fully committed.
    ///
    /// Default: 20 s.
    #[default(Duration::from_secs(20))]
    pub max_ssa_delivery_time: Duration,

    /// Maximum time to wait for a deposit after the commitment is verifiable.
    ///
    /// Default: 60 s.
    #[default(Duration::from_secs(60))]
    pub max_deposit_wait: Duration,

    /// Maximum idle time during recovery when service is being consumed.
    ///
    /// Gated on service consumption — if no packets were served, the timer
    /// re-arms instead of closing.
    ///
    /// Default: 60 s.
    #[default(Duration::from_secs(60))]
    pub max_recovery_idle: Duration,

    /// Absolute per-SSA recovery deadline (immutable once set).
    ///
    /// This is a **resource backstop** (session slot + reconstructor memory),
    /// not the anti-drip mechanism. The service-gated idle rule is.
    ///
    /// Default: 1 hour.
    #[default(Duration::from_secs(3600))]
    pub max_recovery_time: Duration,

    /// Maximum tolerated unverifiable shares per SSA before close.
    ///
    /// Default: 3 (4th closes).
    #[default(3)]
    pub max_unverifiable_shares_per_ssa: u64,

    /// Maximum tolerated unverifiable shares across the session lifetime.
    ///
    /// Default: 10 (11th closes).
    #[default(10)]
    pub max_unverifiable_shares_per_session: u64,

    /// Cap on the provisional predeposit service budget.
    ///
    /// Default: 1024 packets.
    #[default(1024)]
    pub max_predeposit_packets: u64,

    /// Maximum packets served without SSA recovery progress before the gate
    /// blocks further service as a defense-in-depth backstop.
    ///
    /// This is a ceiling enforced by [`ServiceGate::acquire`] after the gate is
    /// funded. Each [`RecoveryProgress`] event resets the ceiling counter.
    ///
    /// Default: 256 packets.
    #[default(256)]
    pub max_served_without_progress: u64,

    /// How long to retain recovered-SSA tombstones for late events.
    ///
    /// Must be >= the reconstructor's `max_ack_await_time`.
    ///
    /// Default: 30 s.
    #[default(Duration::from_secs(30))]
    pub tombstone_retention_window: Duration,

    /// Minimum deposit amount required before the gate is released.
    ///
    /// A deposit confirmation below this amount is a no-op (the deposit
    /// deadline keeps running and further top-ups accumulate).  Set to zero
    /// (default) to accept any non-zero deposit — equivalent to the previous
    /// `expected_deposit: None` behaviour.
    ///
    /// Default: zero (accept any).
    #[default(_code = "HoprBalance::new_base(0)")]
    pub min_deposit: HoprBalance,
}

// ---------------------------------------------------------------------------
// SsaDimensions
// ---------------------------------------------------------------------------

/// PIX dimensions agreed upon during session negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsaDimensions {
    pub polys: u16,
    pub threshold: u16,
}

impl SsaDimensions {
    /// Number of useful shares needed for full recovery.
    pub fn target_useful_shares(&self) -> u64 {
        self.polys as u64 * self.threshold as u64
    }
}

// ---------------------------------------------------------------------------
// SessionPixEvent
// ---------------------------------------------------------------------------

/// Events consumed by the [`SessionPixSupervisor`].
#[derive(Debug, Clone)]
pub enum SessionPixEvent {
    /// The initial or next SSA request was successfully sent on the wire.
    SsaRequestSent(SsaId<HoprPseudonym>),
    /// A verifiable commitment was installed in the reconstructor.
    CommitmentVerified {
        ssa_id: SsaId<HoprPseudonym>,
        expected_deposit: Option<HoprBalance>,
    },
    /// Deposit for a specific SSA was confirmed with the given amount.
    DepositConfirmed {
        ssa_id: SsaId<HoprPseudonym>,
        amount: HoprBalance,
    },
    /// The deposit observer channel closed without delivering a confirmation.
    DepositObserverClosed(SsaId<HoprPseudonym>),
    /// Recovery progress snapshot from the reconstructor.
    RecoveryProgress(SsaRecoveryProgress<HoprPseudonym>),
    /// Early-recovery threshold reached.
    AlmostRecovered(SsaId<HoprPseudonym>),
    /// Full SSA recovery completed.
    Recovered(SsaId<HoprPseudonym>),
    /// Absolute per-SSA unverifiable-share count observation.
    UnverifiableShares {
        ssa_id: SsaId<HoprPseudonym>,
        observed_total: u64,
    },
}

// ---------------------------------------------------------------------------
// SessionPixAction
// ---------------------------------------------------------------------------

/// Actions emitted by the [`SessionPixSupervisor`] for the caller to execute.
#[derive(Debug, Clone)]
pub enum SessionPixAction {
    /// Request a new SSA with the given dimensions.
    RequestSsa {
        ssa_id: SsaId<HoprPseudonym>,
        polys: u16,
        threshold: u16,
    },
    /// Release the service gate (from predeposit to funded mode).
    ReleaseService,
    /// Notifies the gate that SSA recovery made progress, resetting the
    /// served-without-progress ceiling.
    ProgressNotification,
    /// Close the session with the given reason.
    Close(SessionPixCloseReason),
    /// Retire a previously-used SSA from the reconstructor (idempotent).
    /// Emitted when an SSA's tombstone period expires so mid-session state
    /// does not accumulate.
    RetireSsa(SsaId<HoprPseudonym>),
}

// ---------------------------------------------------------------------------
// SessionPixCloseReason
// ---------------------------------------------------------------------------

/// Internal close reasons emitted by the supervisor.
///
/// These are mapped to public [`ClosureReason`] by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum SessionPixCloseReason {
    /// The commitment delivery deadline expired.
    CommitmentTimeout,
    /// The deposit deadline expired without a sufficient deposit.
    DepositTimeout,
    /// A confirmed deposit was below the expected amount and never topped up.
    #[expect(dead_code, reason = "will be emitted by supervisor in deposit flow (Step 4)")]
    DepositUnderfundedTimeout,
    /// The deposit observer channel closed without delivering a confirmation.
    DepositObserverClosed,
    /// Service was consumed but no useful progress was made — service-gated idle.
    RecoveryIdle,
    /// The per-SSA hard recovery deadline expired.
    RecoveryDeadline,
    /// Too many unverifiable shares (per-SSA or session-limit exceeded).
    TooManyUnverifiableShares,
    /// A counter observation decreased (protocol violation).
    CounterRegression,
    /// Internal inconsistency (e.g., mismatched target, event on unknown SSA).
    InvalidTransition,
    /// The SSA set drained (all SSAs expired/recovered without a successor).
    NoSsaRemaining,
    /// The supervisor action driver failed or was dropped.
    SupervisorUnavailable,
}

// ---------------------------------------------------------------------------
// Re-exports from submodules
// ---------------------------------------------------------------------------

pub use gate::ServiceGate;
pub use worker::{ActionRx, SessionPixSupervisorHandle, spawn_supervisor_worker};

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validates that [`SupervisorConfig`] and [`SsaReconstructorConfig`] are
/// mutually consistent.
///
/// Returns an error if constraints are violated.
pub fn validate_pix_supervision(
    cfg: &SupervisorConfig,
    reconstructor_cfg: &SsaReconstructorConfig,
) -> Result<(), TransportSessionError> {
    if cfg.max_ssa_delivery_time.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_ssa_delivery_time must be non-zero".into(),
        ));
    }
    if cfg.max_deposit_wait.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_deposit_wait must be non-zero".into(),
        ));
    }
    if cfg.max_recovery_idle.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be non-zero".into(),
        ));
    }
    if cfg.max_recovery_time.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_time must be non-zero".into(),
        ));
    }
    if cfg.tombstone_retention_window.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "tombstone_retention_window must be non-zero".into(),
        ));
    }
    if cfg.max_served_without_progress == 0 {
        return Err(TransportSessionError::InvalidConfig(
            "max_served_without_progress must be non-zero".into(),
        ));
    }
    if cfg.max_recovery_idle < reconstructor_cfg.max_ack_await_time {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be >= max_ack_await_time".into(),
        ));
    }
    // Documented invariant: tombstone must outlive the ack window.
    if cfg.tombstone_retention_window < reconstructor_cfg.max_ack_await_time {
        return Err(TransportSessionError::InvalidConfig(
            "tombstone_retention_window must be >= max_ack_await_time (otherwise late acks arrive after the tombstone \
             expires)"
                .into(),
        ));
    }
    if cfg.max_recovery_idle >= reconstructor_cfg.incomplete_ssa_lifetime {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be < incomplete_ssa_lifetime".into(),
        ));
    }
    let supervision_horizon = cfg
        .max_deposit_wait
        .checked_add(cfg.max_recovery_time)
        .and_then(|sum| sum.checked_add(cfg.tombstone_retention_window))
        .unwrap_or(Duration::MAX);
    if Duration::from_secs(reconstructor_cfg.ssa_counter_lifetime_secs) <= supervision_horizon {
        return Err(TransportSessionError::InvalidConfig(
            "ssa_counter_lifetime must be > max_deposit_wait + max_recovery_time + tombstone_retention_window".into(),
        ));
    }
    // Reject durations that would overflow the monotonic clock when used as
    // deadlines. 24 h is a safe upper bound — no supervisor duration should
    // ever be this large, and the cap prevents silent deadline loss via
    // Instant::checked_add returning None.
    const MAX_SUPERVISOR_DURATION: Duration = Duration::from_secs(86400);
    for (name, dur) in [
        ("max_ssa_delivery_time", &cfg.max_ssa_delivery_time),
        ("max_deposit_wait", &cfg.max_deposit_wait),
        ("max_recovery_idle", &cfg.max_recovery_idle),
        ("max_recovery_time", &cfg.max_recovery_time),
        ("tombstone_retention_window", &cfg.tombstone_retention_window),
    ] {
        if *dur > MAX_SUPERVISOR_DURATION {
            return Err(TransportSessionError::InvalidConfig(format!(
                "{name} ({dur:?}) must not exceed {MAX_SUPERVISOR_DURATION:?}"
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[cfg(feature = "runtime-tokio")]
mod tests {
    use std::time::Duration;

    use hopr_protocol_pix::SsaReconstructorConfig;

    use super::*;

    fn valid_cfg() -> SupervisorConfig {
        SupervisorConfig {
            max_ssa_delivery_time: Duration::from_secs(20),
            max_deposit_wait: Duration::from_secs(60),
            max_recovery_idle: Duration::from_secs(60),
            max_recovery_time: Duration::from_secs(3600),
            max_unverifiable_shares_per_ssa: 3,
            max_unverifiable_shares_per_session: 10,
            max_predeposit_packets: 1024,
            max_served_without_progress: 256,
            tombstone_retention_window: Duration::from_secs(30),
            min_deposit: HoprBalance::new_base(0),
        }
    }

    fn valid_rcn_cfg() -> SsaReconstructorConfig {
        SsaReconstructorConfig {
            max_ack_await_time: Duration::from_secs(10),
            incomplete_ssa_lifetime: Duration::from_secs(600),
            ssa_counter_lifetime_secs: 4000,
            ..Default::default()
        }
    }

    #[test]
    fn validation_accepts_valid_configs() {
        assert!(validate_pix_supervision(&valid_cfg(), &valid_rcn_cfg()).is_ok());
    }

    #[test]
    fn validation_rejects_zero_max_ssa_delivery_time() {
        let mut cfg = valid_cfg();
        cfg.max_ssa_delivery_time = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_deposit_wait() {
        let mut cfg = valid_cfg();
        cfg.max_deposit_wait = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_recovery_idle() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_idle = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_recovery_time() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_time = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_tombstone_retention_window() {
        let mut cfg = valid_cfg();
        cfg.tombstone_retention_window = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_served_without_progress() {
        let mut cfg = valid_cfg();
        cfg.max_served_without_progress = 0;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_idle_shorter_than_ack_await() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_idle = Duration::from_secs(5);
        let rcn = valid_rcn_cfg();
        // max_ack_await_time is 10 s, so 5 < 10 should fail.
        assert!(validate_pix_supervision(&cfg, &rcn).is_err());
    }

    #[test]
    fn validation_rejects_idle_exceeds_incomplete_ssa_lifetime() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_idle = Duration::from_secs(700);
        let rcn = valid_rcn_cfg();
        // incomplete_ssa_lifetime is 600 s, so 700 >= 600 should fail.
        assert!(validate_pix_supervision(&cfg, &rcn).is_err());
    }

    #[test]
    fn validation_rejects_counter_lifetime_shorter_than_recovery_horizon() {
        let mut cfg = valid_cfg();
        cfg.max_deposit_wait = Duration::from_secs(60);
        cfg.max_recovery_time = Duration::from_secs(3600);
        cfg.tombstone_retention_window = Duration::from_secs(60);
        let mut rcn = valid_rcn_cfg();
        // horizon = 60 + 3600 + 60 = 3720.  ssa_counter_lifetime must be > 3720.
        rcn.ssa_counter_lifetime_secs = 3720; // equal, not greater → reject
        assert!(validate_pix_supervision(&cfg, &rcn).is_err());

        rcn.ssa_counter_lifetime_secs = 3721;
        assert!(validate_pix_supervision(&cfg, &rcn).is_ok());
    }

    #[test]
    fn subsecond_boundary_preserves_precision() {
        // max_deposit_wait = 60, max_recovery_time = 3600.9 s, tombstone = 30.9 s
        // supervision horizon = 60 + 3600.9 + 30.9 = 3691.8 s.
        // counter_lifetime of 3691 s (= 3691.0 s) is NOT > 3691.8 s → reject
        let mut cfg = valid_cfg();
        cfg.max_deposit_wait = Duration::from_secs(60);
        cfg.max_recovery_time = Duration::new(3600, 900_000_000);
        cfg.tombstone_retention_window = Duration::new(30, 900_000_000);
        let mut rcn = valid_rcn_cfg();
        rcn.ssa_counter_lifetime_secs = 3691;
        assert!(validate_pix_supervision(&cfg, &rcn).is_err());
        // One second beyond the subsecond horizon → accept.
        rcn.ssa_counter_lifetime_secs = 3692;
        assert!(validate_pix_supervision(&cfg, &rcn).is_ok());
    }

    #[test]
    fn overflow_safety_uses_checked_add() {
        // Durations near Duration::MAX should not panic on addition.
        let mut cfg = valid_cfg();
        cfg.max_recovery_time = Duration::MAX;
        cfg.tombstone_retention_window = Duration::from_secs(1);
        let rcn = valid_rcn_cfg();
        // checked_add should saturate to Duration::MAX, so the comparison
        // with ssa_counter_lifetime_secs will fail (Duration::MAX seconds
        // ≫ 4000). The call should not panic.
        assert!(validate_pix_supervision(&cfg, &rcn).is_err());
    }

    #[test]
    fn subsecond_horizon_accepts_large_counter_lifetime() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_time = Duration::new(3600, 1); // 3600 + 1 ns
        cfg.tombstone_retention_window = Duration::new(30, 1);
        let mut rcn = valid_rcn_cfg();
        // horizon = 3630.000000002 s ≪ 4000 s → accept.
        rcn.ssa_counter_lifetime_secs = 4000;
        assert!(validate_pix_supervision(&cfg, &rcn).is_ok());
    }
}
