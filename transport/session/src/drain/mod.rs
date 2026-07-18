//! Post-closure SURB drain for PIX sessions (Exit side).
//!
//! When a PIX session closes with unused SURBs and funded SSA deposit
//! addresses, the Exit can drain the leftover SURBs by sending keep-alive
//! packets on the return path.  Each keep-alive pops a SURB, the first
//! relay sends back an acknowledgement, and the acknowledgement provides
//! the decryption key for the share encrypted inside that SURB.  Collected
//! shares may fully recover the SSA secret, allowing the Exit to claim
//! the deposit.
//!
//! # Architecture
//!
//! The [`SurbDrainer`] is a standalone component (no dependency on
//! [`SessionManager`](crate::SessionManager)).  It owns:
//!
//! * A configuration ([`SurbDrainConfig`]).
//! * A message sink (`S`) for sending drain keep-alive packets.
//! * A reference to the [`SsaReconstructor`] for progress/fault snapshots.
//! * A closure to query SURB counts.
//! * A closure for the current packet price.
//! * A registry of in-flight drain tasks (pseudonym → abort handle).
//!
//! The [`SessionManager`](crate::SessionManager) hands over closed-session
//! material via [`offer`](SurbDrainer::offer) and forwards post-closure
//! PIX events via [`deliver_event`](SurbDrainer::deliver_event).

mod assess;
mod config;
mod task;

use std::sync::Arc;

use futures::Sink;
use hopr_api::types::{
    internal::{prelude::HoprPseudonym, routing::DestinationRouting},
    primitive::balance::HoprBalance,
};
use hopr_crypto_packet::HoprPixSpec;
use hopr_protocol_pix::{SsaCommitmentGuard, SsaReconstructor};
use hopr_utils::runtime::AbortableList;
use parking_lot::Mutex;
use tracing::info;

pub use assess::evaluate_offer;
pub use config::SurbDrainConfig;

use hopr_protocol_pix::SsaReconstructorConfig;
use crate::errors::TransportSessionError;

use crate::types::{ClosureReason, SessionId};
use crate::supervision::SessionPixCloseReason;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Handover material for one closed session, assembled by `SessionManager`.
pub struct ClosedSessionOffer {
    /// Session identifier (same as pseudonym).
    pub session_id: SessionId,
    /// Return routing for drain keep-alive packets.
    pub routing: DestinationRouting,
    /// Reason the session was closed.
    pub closure_reason: ClosureReason,
    /// Optional PIX-specific close reason.
    pub pix_close_reason: Option<SessionPixCloseReason>,
    /// Per-SSA handover data.
    pub ssas: Vec<SsaHandover>,
}

/// One SSA's handover data from a closed session.
pub struct SsaHandover {
    /// RAII guard — ownership means retire-on-drop stays RAII.
    pub guard: SsaCommitmentGuard<HoprPixSpec>,
    /// Accumulated confirmed deposit for this SSA.
    pub funded: HoprBalance,
}

/// Outcome of a drain (finished or skipped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrainOutcome {
    /// Session this outcome belongs to.
    pub session_id: SessionId,
    /// What happened.
    pub result: DrainResult,
    /// Number of keep-alive packets sent.
    pub packets_sent: u64,
    /// Number of SSAs that were recovered post-closure.
    pub ssas_recovered: u32,
}

/// Why a drain finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum DrainStopReason {
    /// All tracked SSAs were fully recovered.
    AllRecovered,
    /// An unverifiable share delta was observed (zero-tolerance).
    UnverifiableShare,
    /// No SURBs left in the store.
    SurbsExhausted,
    /// No useful progress (acks) received within the grace period.
    NoProgress,
    /// Packet budget (economic limit) exhausted.
    BudgetExhausted,
    /// Drain deadline passed.
    DeadlineReached,
    /// Drainer was shut down.
    Shutdown,
}

/// Why a drain was skipped (never started).
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum SkipReason {
    /// Drainer is disabled in config.
    Disabled,
    /// Session was closed due to a PIX fault.
    FaultClose,
    /// No funded SSA in the offer.
    NoFundedSsa,
    /// All SSAs already have deficit == 0.
    NoDeficit,
    /// Not enough SURBs to cover the total deficit.
    InsufficientSurbs,
    /// Deposit does not cover the expected ticket cost.
    UneconomicalDeposit,
    /// Concurrency limit reached.
    ConcurrencyLimit,
}

/// Combined drain result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainResult {
    /// Drain ran to completion.
    Finished(DrainStopReason),
    /// Drain was skipped before starting.
    Skipped(SkipReason),
}

// ---------------------------------------------------------------------------
// SurbDrainer
// ---------------------------------------------------------------------------

/// Post-closure SURB drainer (Exit side).
///
/// Spawns one drain task per eligible closed session.  All precondition
/// evaluation, concurrency control, and outcome reporting happen inside
/// this component.
pub struct SurbDrainer<S> {
    cfg: SurbDrainConfig,
    msg_sender: S,
    reconstructor: Arc<SsaReconstructor<HoprPixSpec>>,
    surb_count: Arc<dyn Fn(&HoprPseudonym) -> usize + Send + Sync>,
    packet_price: Arc<dyn Fn() -> HoprBalance + Send + Sync>,
    // Registry of active drain tasks: pseudonym → abort handle.
    registry: Mutex<Vec<(HoprPseudonym, task::DrainTaskHandle)>>,
    // Outcome sender — cloned to each drain task.
    outcome_tx: crossfire::MTx<crossfire::mpsc::List<DrainOutcome>>,
    outcome_rx: Mutex<Option<crossfire::AsyncRx<crossfire::mpsc::List<DrainOutcome>>>>,
    abort_handles: Arc<parking_lot::Mutex<AbortableList<&'static str>>>,
}

impl<S> SurbDrainer<S>
where
    S: Sink<(DestinationRouting, hopr_protocol_app::v1::ApplicationDataOut)> + Clone + Unpin + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(
        cfg: SurbDrainConfig,
        msg_sender: S,
        reconstructor: Arc<SsaReconstructor<HoprPixSpec>>,
        surb_count: Arc<dyn Fn(&HoprPseudonym) -> usize + Send + Sync>,
        packet_price: Arc<dyn Fn() -> HoprBalance + Send + Sync>,
    ) -> Self {
        let (outcome_tx, outcome_rx) = crossfire::mpsc::unbounded_async();
        Self {
            cfg,
            msg_sender,
            reconstructor,
            surb_count,
            packet_price,
            registry: Mutex::new(Vec::new()),
            outcome_tx,
            outcome_rx: Mutex::new(Some(outcome_rx)),
            abort_handles: Default::default(),
        }
    }

    /// Take the outcome receiver (only callable once).
    pub fn outcome_rx(&self) -> Option<crossfire::AsyncRx<crossfire::mpsc::List<DrainOutcome>>> {
        self.outcome_rx.lock().take()
    }

    /// Check whether a session is currently being drained.
    pub fn is_draining(&self, pseudonym: &HoprPseudonym) -> bool {
        self.registry.lock().iter().any(|(p, _)| p == pseudonym)
    }

    /// Submit a closed-session offer for evaluation.
    ///
    /// Non-blocking.  If eligible, spawns a drain task and registers the
    /// pseudonym.  If ineligible, drops the guards (→ retire_ssa) and
    /// emits a `DrainOutcome::Skipped`.
    pub fn offer(&self, offer: ClosedSessionOffer) {
        let active = self.registry.lock().len();

        // Build snapshots from the reconstructor using the handover guards.
        let snapshots: Vec<_> = offer
            .ssas
            .iter()
            .map(|ssa| self.reconstructor.drain_snapshot(ssa.guard.ssa_id()))
            .collect();

        let surb_count = (self.surb_count)(&offer.session_id.into());
        let packet_price = (self.packet_price)();

        let verdict = evaluate_offer(&self.cfg, &offer, snapshots.as_slice(), surb_count, packet_price, active);

        match verdict {
            assess::DrainVerdict::Drain(params) => {
                info!(
                    session_id = %offer.session_id,
                    max_packets = params.max_packets,
                    deficits = ?params.deficits,
                    "drainer: spawning drain task"
                );
                let deficits: Vec<(usize, u64)> = params
                    .deficits
                    .into_iter()
                    .map(|d| (d.guard_index, d.deficit))
                    .collect();

                let pseudonym: HoprPseudonym = offer.session_id.into();
                let handle = task::spawn_drain_task(
                    self.msg_sender.clone(),
                    self.cfg,
                    offer,
                    self.surb_count.clone(),
                    self.packet_price.clone(),
                    self.reconstructor.clone(),
                    params.max_packets,
                    deficits,
                    self.outcome_tx.clone(),
                );
                self.registry.lock().push((pseudonym, handle));
            }
            assess::DrainVerdict::Skip(reason) => {
                // Guards drop here → retire_ssa.
                let outcome = DrainOutcome {
                    session_id: offer.session_id,
                    result: DrainResult::Skipped(reason),
                    packets_sent: 0,
                    ssas_recovered: 0,
                };
                info!(session_id = %offer.session_id, %reason, "drainer: skipping");
                let _ = self.outcome_tx.try_send(outcome);
            }
        }
    }

    /// Forward a PIX event to a running drain task (post-closure).
    pub fn deliver_event(&self, event: &crate::types::HoprSessionInPixEvent) {
        use crate::types::HoprSessionInPixEvent;

        let relevant = match event {
            HoprSessionInPixEvent::SsaRecovered(_) => true,
            _ => return,
        };

        if !relevant {
            return;
        }

        let pseudonym = event.pseudonym();
        let mut registry = self.registry.lock();
        if let Some((_, handle)) = registry.iter_mut().find(|(p, _)| p == pseudonym) {
            let event = match event {
                HoprSessionInPixEvent::SsaRecovered(ssa_id) => Some(task::DrainEvent::SsaRecovered(*ssa_id)),
                _ => None,
            };
            if let Some(evt) = event {
                let _ = handle.event_tx.try_send(evt);
            }
        }
    }

    /// Abort all running drain tasks and clear the registry.
    pub fn shutdown(&self) {
        let mut registry = self.registry.lock();
        for (_, handle) in registry.drain(..) {
            handle.abort_handle.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// Config validation
// ---------------------------------------------------------------------------

/// Validate drain configuration against the reconstructor config.
///
/// Checks the invariants from §8 of the drain spec:
/// - `ack_grace >= max_ack_await_time` (otherwise honest late acks abort)
/// - `max_drain_time > ack_grace`
/// - `max_drain_time < incomplete_ssa_lifetime` (builders must not expire mid-drain)
/// - `cost_safety_factor >= 1.0`
/// - `drain_rate_packets_per_sec > 0`
/// - `max_concurrent_drains > 0`
/// - Durations capped at 24 h
pub fn validate_pix_drain(
    cfg: &SurbDrainConfig,
    reconstructor_cfg: &SsaReconstructorConfig,
) -> Result<(), TransportSessionError> {
    const MAX_DURATION: std::time::Duration = std::time::Duration::from_secs(86400); // 24 h

    if !cfg.enabled {
        // Disabled drainer is always valid.
        return Ok(());
    }
    if cfg.max_drain_time.is_zero() {
        return Err(TransportSessionError::InvalidConfig("max_drain_time must be non-zero".into()));
    }
    if cfg.max_drain_time > MAX_DURATION {
        return Err(TransportSessionError::InvalidConfig(
            "max_drain_time must not exceed 24 hours".into(),
        ));
    }
    if cfg.drain_rate_packets_per_sec == 0 {
        return Err(TransportSessionError::InvalidConfig(
            "drain_rate_packets_per_sec must be > 0".into(),
        ));
    }
    if cfg.max_concurrent_drains == 0 {
        return Err(TransportSessionError::InvalidConfig(
            "max_concurrent_drains must be > 0".into(),
        ));
    }
    if cfg.ack_grace.is_zero() {
        return Err(TransportSessionError::InvalidConfig("ack_grace must be non-zero".into()));
    }
    if cfg.ack_grace > MAX_DURATION {
        return Err(TransportSessionError::InvalidConfig("ack_grace must not exceed 24 hours".into()));
    }
    if cfg.ack_grace < reconstructor_cfg.max_ack_await_time {
        return Err(TransportSessionError::InvalidConfig(
            "ack_grace must be >= max_ack_await_time (otherwise honest late acks abort the drain)".into(),
        ));
    }
    if cfg.max_drain_time <= cfg.ack_grace {
        return Err(TransportSessionError::InvalidConfig(
            "max_drain_time must be > ack_grace".into(),
        ));
    }
    if cfg.max_drain_time >= reconstructor_cfg.incomplete_ssa_lifetime {
        return Err(TransportSessionError::InvalidConfig(
            "max_drain_time must be < incomplete_ssa_lifetime (SSA builders must not expire mid-drain)".into(),
        ));
    }
    if cfg.cost_safety_factor < 1.0 {
        return Err(TransportSessionError::InvalidConfig(
            "cost_safety_factor must be >= 1.0".into(),
        ));
    }

    Ok(())
}
