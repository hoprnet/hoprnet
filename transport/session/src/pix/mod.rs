//! PIX supervisor module for incoming PIX-enabled sessions.
//!
//! Contains the deterministic [`SessionPixSupervisor`] core, the [`ServiceGate`]
//! for bounded predeposit egress, and the per-session actor that serializes
//! lifecycle events.

use std::time::Duration;

use hopr_api::{HoprBalance, types::internal::prelude::HoprPseudonym};
use hopr_protocol_pix::{SsaId, SsaReconstructorConfig, SsaRecoveryProgress};

use crate::errors::TransportSessionError;

pub(crate) mod gate;
pub(crate) mod supervisor;
pub(crate) mod worker;

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

    /// How long to retain recovered-SSA tombstones for late events.
    ///
    /// Must be >= the reconstructor's `max_ack_await_time`.
    ///
    /// Default: 30 s.
    #[default(Duration::from_secs(30))]
    pub tombstone_retention_window: Duration,
}

// ---------------------------------------------------------------------------
// SsaDimensions
// ---------------------------------------------------------------------------

/// PIX dimensions agreed upon during session negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SsaDimensions {
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
pub(crate) enum SessionPixEvent {
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
pub(crate) enum SessionPixAction {
    /// Request a new SSA with the given dimensions.
    RequestSsa {
        ssa_id: SsaId<HoprPseudonym>,
        polys: u16,
        threshold: u16,
    },
    /// Release the service gate (from predeposit to funded mode).
    ReleaseService,
    /// Close the session with the given reason.
    Close(SessionPixCloseReason),
}

// ---------------------------------------------------------------------------
// SessionPixCloseReason
// ---------------------------------------------------------------------------

/// Internal close reasons emitted by the supervisor.
///
/// These are mapped to public [`ClosureReason`] by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub(crate) enum SessionPixCloseReason {
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
    /// The supervisor action driver failed or was dropped.
    SupervisorUnavailable,
}

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
    if cfg.max_recovery_idle < reconstructor_cfg.max_ack_await_time {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be >= max_ack_await_time".into(),
        ));
    }
    if cfg.max_recovery_idle >= reconstructor_cfg.incomplete_ssa_lifetime {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be < incomplete_ssa_lifetime".into(),
        ));
    }
    if reconstructor_cfg.ssa_counter_lifetime_secs
        <= cfg.max_recovery_time.as_secs() + cfg.tombstone_retention_window.as_secs()
    {
        return Err(TransportSessionError::InvalidConfig(
            "ssa_counter_lifetime must be > max_recovery_time + tombstone_retention_window".into(),
        ));
    }
    Ok(())
}
