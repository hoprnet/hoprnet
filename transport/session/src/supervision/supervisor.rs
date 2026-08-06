//! Deterministic [`SessionPixSupervisor`] — the pure state machine for PIX
//! session lifecycle.
//!
//! All methods take explicit [`std::time::Instant`] timestamps and a
//! `served_total: u64` sample from the [`ServiceGate`](super::gate::ServiceGate).
//! No method sleeps, spawns, or performs I/O.

use std::time::{Duration, Instant};

use hopr_api::{HoprBalance, types::internal::prelude::HoprPseudonym};
use hopr_protocol_pix::{SsaId, SsaIndex, SsaRecoveryProgress};

use super::{SessionPixAction, SessionPixCloseReason, SessionPixEvent, SsaDimensions, SupervisorConfig};

// ---------------------------------------------------------------------------
// SsaPhase
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SsaPhase {
    /// Request has been sent; waiting for complete verifiable commitment.
    AwaitingCommitment,
    /// Commitment is verifiable; waiting for a sufficient deposit.
    AwaitingDeposit,
    /// Deposit confirmed; recovering shares.
    Recovering,
    /// SSA fully recovered; tombstone until `tombstone_until`.
    Recovered { tombstone_until: Instant },
    /// Phase that will produce a close action on next deadline check.
    Closing,
}

// ---------------------------------------------------------------------------
// PerSsaState
// ---------------------------------------------------------------------------

/// Internal state for one supervised SSA.
struct PerSsaState {
    ssa_id: SsaId<HoprPseudonym>,
    phase: SsaPhase,

    // Deadlines (None means not set for this phase).
    commitment_deadline: Option<Instant>,
    deposit_deadline: Option<Instant>,
    recovery_idle_deadline: Option<Instant>,
    recovery_hard_deadline: Option<Instant>,

    // Progress tracking.
    largest_useful_shares: u64,
    target_useful_shares: u64,
    recovered_polynomials: u16,

    // Fault tracking.
    per_ssa_invalid_total: u64,

    // Deposit state.
    expected_deposit: Option<HoprBalance>,
    /// Accumulated deposit amount across top-up deposits.
    accumulated_deposit: HoprBalance,

    // Overlap / deferred-request state.
    next_request_pending_deposit: bool,
    next_requested: bool,
    /// Set on the last cycle allocated by [`SessionPixSupervisor::emit_request_next_ssa`], and only
    /// on that one.
    ///
    /// The successor gate: a batch asks for its replacement once, when its *last* cycle is nearly
    /// recovered. Every other flag here is per-cycle, so without this one each member of a batch
    /// would answer its own `AlmostRecovered` with a whole batch of its own — `ssas_per_request`
    /// batches per batch, compounding, each cycle a separate on-chain deposit for the Entry.
    ///
    /// The last cycle rather than the first because the batch is served in index order: gating on
    /// the first would ask for the successor a whole batch too early, which is the exposure
    /// `ssas_per_request` deliberately bounds.
    is_batch_last: bool,
    /// Set when `Recovered` arrives before deposit confirms (i.e. during
    /// `AwaitingCommitment` or `AwaitingDeposit`). Once deposit is confirmed
    /// and the SSA enters `Recovering`, this flag triggers an immediate
    /// tombstone transition — replaying the deferred `Recovered` event.
    recovered_pending: bool,

    // Service gating.
    served_total_at_last_progress: u64,
}

impl PerSsaState {
    fn new(ssa_id: SsaId<HoprPseudonym>, target_useful_shares: u64, _now: Instant) -> Self {
        Self {
            ssa_id,
            phase: SsaPhase::AwaitingCommitment,
            commitment_deadline: None,
            deposit_deadline: None,
            recovery_idle_deadline: None,
            recovery_hard_deadline: None,
            largest_useful_shares: 0,
            target_useful_shares,
            recovered_polynomials: 0,
            per_ssa_invalid_total: 0,
            expected_deposit: None,
            accumulated_deposit: HoprBalance::new_base(0),
            next_request_pending_deposit: false,
            next_requested: false,
            is_batch_last: false,
            recovered_pending: false,
            served_total_at_last_progress: 0,
        }
    }

    /// True if this SSA is past the recovery phase.
    fn is_terminal(&self) -> bool {
        matches!(self.phase, SsaPhase::Recovered { .. } | SsaPhase::Closing)
    }
}

// ---------------------------------------------------------------------------
// SessionPixSupervisor
// ---------------------------------------------------------------------------

/// Deterministic core of the PIX session supervisor.
pub struct SessionPixSupervisor {
    pub(crate) cfg: SupervisorConfig,
    pub(crate) dims: SsaDimensions,
    pub(crate) pseudonym: HoprPseudonym,
    pub(crate) closed: bool,
    next_ssa_index: u32,
    session_invalid_total: u64,
    release_service_emitted: bool,
    /// Ordered SSAs (oldest first, newest last). At most 2 live + 1 tombstone.
    ssas: Vec<PerSsaState>,
    /// Tracks the first failure reason when multiple SSAs fail, so the
    /// earliest cause is used for the final `Close` action rather than the last.
    first_failure_reason: Option<SessionPixCloseReason>,
    /// SSA indices that have been retired (closed and removed).
    /// Prevents stale SsaRequestSent events from resurrecting closed SSAs.
    retired_ssa_indices: Vec<SsaIndex>,
}

impl SessionPixSupervisor {
    /// Create a new supervisor and emit the first `RequestSsa` action.
    pub fn new(
        cfg: SupervisorConfig,
        dims: SsaDimensions,
        pseudonym: HoprPseudonym,
        now: Instant,
    ) -> (Self, Vec<SessionPixAction>) {
        let mut s = Self {
            cfg,
            dims,
            pseudonym,
            next_ssa_index: 1,
            session_invalid_total: 0,
            closed: false,
            release_service_emitted: false,
            ssas: Vec::with_capacity(2),
            first_failure_reason: None,
            retired_ssa_indices: Vec::new(),
        };

        let actions = s.emit_request_next_ssa(now);
        (s, actions)
    }

    /// Handle a lifecycle event.
    pub fn handle_event(&mut self, ev: &SessionPixEvent, now: Instant, served_total: u64) -> Vec<SessionPixAction> {
        if self.closed {
            return Vec::new();
        }

        let actions = match ev {
            SessionPixEvent::SsaRequestSent(ssa_id) => self.on_ssa_request_sent(ssa_id, now),
            SessionPixEvent::CommitmentVerified {
                ssa_id,
                expected_deposit,
            } => self.on_commitment_verified(ssa_id, *expected_deposit, now),
            SessionPixEvent::DepositConfirmed { ssa_id, amount } => {
                self.on_deposit_confirmed(ssa_id, *amount, now, served_total)
            }
            SessionPixEvent::DepositObserverClosed(ssa_id) => self.on_deposit_observer_closed(ssa_id, now),
            SessionPixEvent::RecoveryProgress(progress) => self.on_recovery_progress(progress, now, served_total),
            SessionPixEvent::AlmostRecovered(ssa_id) => self.on_almost_recovered(ssa_id, now),
            SessionPixEvent::Recovered(ssa_id) => self.on_recovered(ssa_id, now),
            SessionPixEvent::UnverifiableShares { ssa_id, observed_total } => {
                self.on_unverifiable_shares(ssa_id, *observed_total, now)
            }
        };

        self.arm_recovery_clocks_for_earliest(now, served_total);
        actions
    }

    /// Starts the recovery clocks of the earliest unrecovered cycle, if they are not running yet.
    ///
    /// A batch's cycles are served strictly in index order — the Entry's emission window is clamped to
    /// one cycle (see [`hopr_protocol_pix::SHARE_EMISSION_WINDOW`]) — so a cycle behind the front of
    /// the batch is *queued*, not stalled. Starting its clocks when its deposit confirmed, as this used
    /// to, measured the queue wait rather than the recovery: at deployed dimensions one cycle takes
    /// ~72 min of emission to exhaust, so every cycle after the first was retired by
    /// `max_recovery_time` — or, worse, by `max_recovery_idle` inside a minute, since the idle gate
    /// compares against *session-wide* service and a queued cycle sees plenty of that without any of it
    /// being its own.
    ///
    /// So the clock starts when the cycle reaches the front, which is when it can actually make
    /// progress, and [`SupervisorConfig::max_recovery_time`] then means what its name says: the ceiling
    /// on recovering a single cycle. `served_total_at_last_progress` is re-baselined at the same moment
    /// for the same reason — otherwise the idle gate would charge the queue wait against it.
    ///
    /// Called after every event and every deadline sweep, so it covers each way a cycle can reach the
    /// front: its predecessor recovering, or its predecessor being retired.
    fn arm_recovery_clocks_for_earliest(&mut self, now: Instant, served_total: u64) {
        if self.closed {
            return;
        }
        let Some(idx) = self
            .ssas
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_terminal())
            .min_by_key(|(_, s)| s.ssa_id.ssa_index())
            .map(|(i, _)| i)
        else {
            return;
        };

        let ssa = &mut self.ssas[idx];
        if ssa.phase != SsaPhase::Recovering || ssa.recovery_hard_deadline.is_some() {
            return;
        }

        ssa.recovery_idle_deadline = now.checked_add(self.cfg.max_recovery_idle);
        ssa.recovery_hard_deadline = now.checked_add(self.cfg.max_recovery_time);
        ssa.served_total_at_last_progress = served_total;
        tracing::debug!(ssa_id = %ssa.ssa_id, "recovery clocks started — cycle is at the front of the batch");
    }

    /// Check all deadlines and emit actions for any that have expired.
    pub fn handle_deadline(&mut self, now: Instant, served_total: u64) -> Vec<SessionPixAction> {
        if self.closed {
            return Vec::new();
        }

        let mut actions = Vec::new();
        let max_recovery_idle = self.cfg.max_recovery_idle;

        let mut i = 0;
        while i < self.ssas.len() {
            if self.closed {
                break;
            }

            let expired = {
                let ssa = &self.ssas[i];
                if ssa.is_terminal() {
                    i += 1;
                    continue;
                }
                ssa.check_deadlines(now)
            };

            if let Some(reason) = expired {
                // Service-gated idle: if no service consumed since last progress,
                // re-arm instead of closing.
                if reason == SessionPixCloseReason::RecoveryIdle
                    && served_total <= self.ssas[i].served_total_at_last_progress
                {
                    self.ssas[i].recovery_idle_deadline = now.checked_add(max_recovery_idle);
                    i += 1;
                    continue;
                }

                actions.extend(self.close_ssa_and_collect(i, reason));
                continue;
            }
            i += 1;
        }

        // If the session is already closing, skip tombstone retirement.
        // Whole-session teardown retires everything via the retirement guard.
        if self.closed {
            return actions;
        }

        // Remove tombstones that have expired and emit RetireSsa so the
        // reconstructor and observer state is released mid-session.
        let retired_ids: Vec<SsaId<_>> = self
            .ssas
            .iter()
            .filter(|ssa| matches!(ssa.phase, SsaPhase::Recovered { tombstone_until } if now >= tombstone_until))
            .map(|ssa| ssa.ssa_id)
            .collect();
        self.ssas
            .retain(|ssa| !matches!(ssa.phase, SsaPhase::Recovered { tombstone_until } if now >= tombstone_until));
        for id in retired_ids {
            self.retired_ssa_indices.push(id.ssa_index());
            actions.push(SessionPixAction::RetireSsa(id));
        }

        // If no SSAs remain, close.
        // Note: a successor request may still be in flight here (RequestSsa
        // emitted, SsaRequestSent not yet observed) — intentionally not
        // tracked. The tombstone retention window is the backstop; a request
        // delayed beyond it (>30 s transport stall on an otherwise healthy
        // session) is theoretical and does not warrant blocking normal
        // tombstone expiry.
        if self.ssas.is_empty() && !self.closed {
            actions.push(SessionPixAction::Close(
                self.first_failure_reason
                    .unwrap_or(SessionPixCloseReason::NoSsaRemaining),
            ));
            self.closed = true;
        }

        // Retiring or tombstoning a cycle can promote the next one to the front of the batch.
        self.arm_recovery_clocks_for_earliest(now, served_total);

        actions
    }

    /// Returns the earliest deadline across all live SSAs, or `None`.
    pub fn next_deadline(&self) -> Option<Instant> {
        if self.closed || self.ssas.is_empty() {
            return None;
        }

        self.ssas
            .iter()
            .filter_map(|ssa| {
                if ssa.is_terminal() {
                    if let SsaPhase::Recovered { tombstone_until } = ssa.phase {
                        return Some(tombstone_until);
                    }
                    return None;
                }
                // Return the earliest set deadline.
                ssa.commitment_deadline
                    .into_iter()
                    .chain(ssa.deposit_deadline)
                    .chain(ssa.recovery_idle_deadline)
                    .chain(ssa.recovery_hard_deadline)
                    .min()
            })
            .min()
    }

    /// Feed back the result of executing an action.
    pub fn action_result(&mut self, action: &SessionPixAction, ok: bool, _now: Instant) -> Vec<SessionPixAction> {
        if self.closed {
            return Vec::new();
        }

        match action {
            SessionPixAction::RequestSsa { .. } if !ok => {
                self.closed = true;
                vec![SessionPixAction::Close(SessionPixCloseReason::SupervisorUnavailable)]
            }
            SessionPixAction::Close(_) => {
                self.closed = true;
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    // ------------------------------------------------------------------
    // Internal event handlers
    // ------------------------------------------------------------------

    /// Arms the commitment deadline once the request this SSA was registered for has gone out.
    ///
    /// The record itself is created by [`emit_request_next_ssa`](Self::emit_request_next_ssa); this
    /// only starts the clock, because the Entry cannot be late answering a request it has not been
    /// sent yet.
    fn on_ssa_request_sent(&mut self, ssa_id: &SsaId<HoprPseudonym>, now: Instant) -> Vec<SessionPixAction> {
        // Validate pseudonym. Checked before the lookup: a foreign pseudonym never matches a record
        // of ours, so looking first would answer a cross-session confusion with silence.
        if ssa_id.pseudonym() != &self.pseudonym {
            return vec![SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)];
        }

        // Guard: reject if this SSA index was already retired.
        if self.retired_ssa_indices.contains(&ssa_id.ssa_index()) {
            return Vec::new();
        }

        let Some(idx) = self.find_ssa_idx(ssa_id) else {
            // A confirmation for an SSA we never asked for.
            return vec![SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)];
        };

        // Idempotent: a repeated confirmation must not extend the deadline, and one arriving after
        // the SSA has moved on must not resurrect a phase it has left.
        if self.ssas[idx].phase == SsaPhase::AwaitingCommitment && self.ssas[idx].commitment_deadline.is_none() {
            self.ssas[idx].commitment_deadline = now.checked_add(crate::supervision::scaled_deadline(
                self.cfg.max_ssa_delivery_time,
                self.cfg.ssas_per_request,
            ));
        }

        Vec::new()
    }

    fn on_commitment_verified(
        &mut self,
        ssa_id: &SsaId<HoprPseudonym>,
        expected_deposit: Option<HoprBalance>,
        now: Instant,
    ) -> Vec<SessionPixAction> {
        let idx = match self.find_ssa_idx(ssa_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        let ssa = &mut self.ssas[idx];
        if ssa.phase != SsaPhase::AwaitingCommitment {
            return Vec::new();
        }

        let deposit_deadline = now.checked_add(crate::supervision::scaled_deadline(
            self.cfg.max_deposit_wait,
            self.cfg.ssas_per_request,
        ));
        ssa.phase = SsaPhase::AwaitingDeposit;
        ssa.expected_deposit = expected_deposit;
        ssa.deposit_deadline = deposit_deadline;
        ssa.commitment_deadline = None;

        Vec::new()
    }

    fn on_deposit_confirmed(
        &mut self,
        ssa_id: &SsaId<HoprPseudonym>,
        amount: HoprBalance,
        now: Instant,
        served_total: u64,
    ) -> Vec<SessionPixAction> {
        let idx = match self.find_ssa_idx(ssa_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        let ssa = &mut self.ssas[idx];
        if ssa.phase != SsaPhase::AwaitingDeposit {
            return Vec::new();
        }

        // Accumulate deposit across top-ups.
        ssa.accumulated_deposit += amount;

        // Check deposit sufficiency against accumulated amount.
        let sufficient = match ssa.expected_deposit {
            Some(expected) => ssa.accumulated_deposit >= expected,
            None => true,
        };

        if !sufficient {
            return Vec::new();
        }

        // Transition to Recovering. The two recovery clocks are *not* started here — see
        // `arm_recovery_clocks_for_earliest`, which starts them when this cycle's turn comes.
        let ssa = &mut self.ssas[idx];
        ssa.phase = SsaPhase::Recovering;
        ssa.deposit_deadline = None;
        ssa.served_total_at_last_progress = served_total;

        // If recovery completed before the deposit arrived, immediately
        // tombstone the SSA — the Recovered event was deferred.
        let recovered_pending = ssa.recovered_pending;

        let pending = ssa.next_request_pending_deposit;
        if pending {
            ssa.next_request_pending_deposit = false;
        }
        // End the mutable borrow on ssas[idx].
        let _ = ssa;

        let mut actions = Vec::new();

        if recovered_pending {
            actions.extend(self.perform_recovered_transition(idx, now));
        }

        if !self.release_service_emitted {
            self.release_service_emitted = true;
            actions.push(SessionPixAction::ReleaseService);
        }

        if pending {
            // When recovered_pending is also set, perform_recovered_transition
            // won't emit RequestSsa (next_requested is already set by
            // on_recovered), so emit it here.
            actions.extend(self.emit_request_next_ssa(now));
        }

        actions
    }

    fn on_deposit_observer_closed(&mut self, ssa_id: &SsaId<HoprPseudonym>, _now: Instant) -> Vec<SessionPixAction> {
        let idx = match self.find_ssa_idx(ssa_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        if self.ssas[idx].phase != SsaPhase::AwaitingDeposit {
            return Vec::new();
        }

        self.close_ssa_and_collect(idx, SessionPixCloseReason::DepositObserverClosed)
    }

    fn on_recovery_progress(
        &mut self,
        progress: &SsaRecoveryProgress<HoprPseudonym>,
        now: Instant,
        served_total: u64,
    ) -> Vec<SessionPixAction> {
        let idx = match self.find_ssa_idx(&progress.ssa_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        // Absorb late progress on tombstones — the SSA is already fully
        // recovered and should not reset the session-wide gate watermark.
        if self.ssas[idx].is_terminal() {
            return Vec::new();
        }

        // Validate target consistency before mutating.
        if progress.target_useful_shares != self.dims.target_useful_shares() {
            return vec![SessionPixAction::Close(SessionPixCloseReason::CounterRegression)];
        }

        let ssa = &mut self.ssas[idx];
        let new_useful = progress.useful_shares;

        // Counter regression check.
        //
        // The relay-as-Exit pipeline processes acknowledgement batches with
        // for_each_concurrent, so absolute progress snapshots from different
        // batches can arrive out of order. Treat a stale snapshot as benign
        // noise rather than a protocol violation.
        if new_useful < ssa.largest_useful_shares {
            return Vec::new();
        }

        // Equal snapshot: no-op.
        if new_useful == ssa.largest_useful_shares {
            return Vec::new();
        }

        // Progress is strictly larger.
        ssa.largest_useful_shares = new_useful;
        ssa.recovered_polynomials = progress.recovered_polynomials;
        ssa.served_total_at_last_progress = served_total;

        // Refresh recovery-idle only in Recovering phase.
        if ssa.phase == SsaPhase::Recovering {
            ssa.recovery_idle_deadline = now.checked_add(self.cfg.max_recovery_idle);
        }

        // Signal the gate to reset its served-without-progress ceiling.
        vec![SessionPixAction::ProgressNotification]
    }

    fn on_almost_recovered(&mut self, ssa_id: &SsaId<HoprPseudonym>, now: Instant) -> Vec<SessionPixAction> {
        let idx = match self.find_ssa_idx(ssa_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        // If already recovered (e.g. a concurrent batch delivered Recovered
        // before this AlmostRecovered), the next SSA is already requested —
        // no-op.
        if self.ssas[idx].is_terminal() {
            return Vec::new();
        }

        // The successor gate. A cycle in the middle of a batch reaching its early-recovery threshold
        // says nothing about the batch as a whole: the ones behind it still have their full quota to
        // serve. Only the last cycle standing in for the batch may ask for a replacement.
        if !self.ssas[idx].is_batch_last {
            return Vec::new();
        }

        let next_requested = self.ssas[idx].next_requested;
        let phase = self.ssas[idx].phase;

        if next_requested {
            return Vec::new();
        }

        match phase {
            SsaPhase::Recovering => {
                self.ssas[idx].next_requested = true;
                self.emit_request_next_ssa(now)
            }
            SsaPhase::AwaitingDeposit => {
                self.ssas[idx].next_requested = true;
                self.ssas[idx].next_request_pending_deposit = true;
                Vec::new()
            }
            // AwaitingCommitment or unknown phase: the next SSA request will be
            // triggered when the recovered transition fires (normal Recovering
            // path or deferred replay via recovered_pending). Set the deferred
            // flag so that commitment_verified can propagate it forward.
            _ => {
                self.ssas[idx].next_request_pending_deposit = true;
                self.ssas[idx].next_requested = true;
                Vec::new()
            }
        }
    }

    fn on_recovered(&mut self, ssa_id: &SsaId<HoprPseudonym>, now: Instant) -> Vec<SessionPixAction> {
        let idx = match self.find_ssa_idx(ssa_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        // Guard against duplicate recovery events (possible with concurrent
        // batch processing). If already recovered, this is a no-op.
        if self.ssas[idx].is_terminal() {
            return Vec::new();
        }

        match self.ssas[idx].phase {
            SsaPhase::Recovering => {
                // Normal path — transition to tombstone directly.
                self.perform_recovered_transition(idx, now)
            }
            SsaPhase::AwaitingDeposit => {
                // Recovery completed before deposit confirmed. Record the
                // pending flag so that when the deposit arrives we replay
                // the transition immediately after entering Recovering.
                self.ssas[idx].recovered_pending = true;
                // Also ensure the next SSA request is deferred — the
                // eventual tombstone will emit it. Subject to the same successor gate as
                // `on_almost_recovered`: a non-last cycle of a batch never asks, deferred or not.
                if !self.ssas[idx].next_requested && self.ssas[idx].is_batch_last {
                    self.ssas[idx].next_request_pending_deposit = true;
                    self.ssas[idx].next_requested = true;
                }
                Vec::new()
            }
            SsaPhase::AwaitingCommitment => {
                // Recovery outpaced commitment verification. Same semantics
                // as AwaitingDeposit: set pending flag so the transition
                // replays once commitment and deposit have both arrived.
                self.ssas[idx].recovered_pending = true;
                Vec::new()
            }
            // Closing / already Recovered: handled by the terminal guard above.
            _ => Vec::new(),
        }
    }

    /// Perform the terminal tombstone transition for a fully-recovered SSA.
    /// Called from `on_recovered` (normal Recovering path) and replayed from
    /// `on_deposit_confirmed`/`on_commitment_verified` when `recovered_pending`
    /// was set earlier.
    fn perform_recovered_transition(&mut self, idx: usize, now: Instant) -> Vec<SessionPixAction> {
        let next_requested = self.ssas[idx].next_requested;

        // Transition to tombstone.
        self.ssas[idx].phase = SsaPhase::Recovered {
            tombstone_until: now
                .checked_add(self.cfg.tombstone_retention_window)
                .unwrap_or_else(|| now + Duration::from_secs(86400 * 365)),
        };
        self.ssas[idx].commitment_deadline = None;
        self.ssas[idx].deposit_deadline = None;
        self.ssas[idx].recovery_idle_deadline = None;
        self.ssas[idx].recovery_hard_deadline = None;
        self.ssas[idx].recovered_pending = false;

        let mut actions = Vec::new();
        // Full recovery is the fallback trigger for the early signal, so it carries the same
        // successor gate: a non-last cycle of a batch is tombstoned without asking for anything.
        if !next_requested && self.ssas[idx].is_batch_last {
            self.ssas[idx].next_requested = true;
            actions.extend(self.emit_request_next_ssa(now));
        }

        actions
    }

    fn on_unverifiable_shares(
        &mut self,
        ssa_id: &SsaId<HoprPseudonym>,
        observed_total: u64,
        _now: Instant,
    ) -> Vec<SessionPixAction> {
        let idx = match self.find_ssa_idx(ssa_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        // Absorb late fault reports on tombstones — the SSA is already
        // terminal so fault totals are no longer relevant.
        if self.ssas[idx].is_terminal() {
            return Vec::new();
        }

        let per_ssa_total = self.ssas[idx].per_ssa_invalid_total;

        // Counter regression (or stale snapshot from concurrent processing).
        // With H-02's aggregate totals this should not happen in normal
        // operation, but remain defensive against out-of-order delivery.
        if observed_total < per_ssa_total {
            return Vec::new();
        }

        let delta = observed_total - per_ssa_total;
        if delta == 0 {
            return Vec::new();
        }

        self.ssas[idx].per_ssa_invalid_total = observed_total;
        self.session_invalid_total += delta;

        if self.ssas[idx].per_ssa_invalid_total > self.cfg.max_unverifiable_shares_per_ssa
            || self.session_invalid_total > self.cfg.max_unverifiable_shares_per_session
        {
            vec![SessionPixAction::Close(
                SessionPixCloseReason::TooManyUnverifiableShares,
            )]
        } else {
            Vec::new()
        }
    }

    // ------------------------------------------------------------------
    // Deadline helpers
    // ------------------------------------------------------------------

    /// Close the SSA at `idx` and return close actions.
    fn close_ssa_and_collect(&mut self, idx: usize, reason: SessionPixCloseReason) -> Vec<SessionPixAction> {
        if matches!(self.ssas[idx].phase, SsaPhase::Closing) {
            return Vec::new();
        }

        // Track the first close reason so it isn't lost when multiple SSAs close.
        self.first_failure_reason.get_or_insert(reason);
        let close_reason = self.first_failure_reason.unwrap_or(reason);

        // Warn-level diagnostic with full SSA state before closing.
        let ssa = &self.ssas[idx];
        tracing::warn!(
            ssa_id = %ssa.ssa_id,
            ?reason,
            phase = ?ssa.phase,
            largest_useful_shares = ssa.largest_useful_shares,
            target_useful_shares = ssa.target_useful_shares,
            recovered_polynomials = ssa.recovered_polynomials,
            per_ssa_invalid_total = ssa.per_ssa_invalid_total,
            served_total_at_last_progress = ssa.served_total_at_last_progress,
            ?ssa.commitment_deadline,
            ?ssa.deposit_deadline,
            ?ssa.recovery_idle_deadline,
            ?ssa.recovery_hard_deadline,
            "closing PIX SSA"
        );

        self.ssas[idx].phase = SsaPhase::Closing;

        if self.ssas.len() == 1 {
            self.closed = true;
            return vec![SessionPixAction::Close(close_reason)];
        }

        // Clear deadlines on this SSA.
        self.ssas[idx].commitment_deadline = None;
        self.ssas[idx].deposit_deadline = None;
        self.ssas[idx].recovery_idle_deadline = None;
        self.ssas[idx].recovery_hard_deadline = None;

        // If all SSAs are terminal, close the session.
        if self
            .ssas
            .iter()
            .all(|s| matches!(s.phase, SsaPhase::Closing | SsaPhase::Recovered { .. }))
        {
            self.closed = true;
            return vec![SessionPixAction::Close(close_reason)];
        }

        // Remove this closing SSA and emit RetireSsa so the reconstructor
        // releases its builder/verifier/counter state mid-session.
        // Record the retired index to prevent stale SsaRequestSent events
        // from resurrecting it.
        let retired = self.ssas[idx].ssa_id;
        self.retired_ssa_indices.push(retired.ssa_index());
        let carried_successor_gate = self.ssas[idx].is_batch_last && !self.ssas[idx].next_requested;
        self.ssas.remove(idx);

        // Retiring the cycle that carried the successor gate would strand the batch: its surviving
        // siblings are barred from asking, so nothing would ever request a replacement and the
        // Session would die on a recovery timeout that names the timer rather than the cause. Hand
        // the gate to the newest cycle still standing, which is now the last of the batch that will
        // actually serve its quota — so the request still cannot come early. `next_requested` guards
        // it: a gate already spent must not be handed on and fire a second time.
        if carried_successor_gate
            && let Some(newest) = self
                .ssas
                .iter_mut()
                .filter(|s| !s.is_terminal())
                .max_by_key(|s| s.ssa_id.ssa_index())
        {
            tracing::debug!(
                %retired, promoted = %newest.ssa_id,
                "successor gate handed on from a retired cycle"
            );
            newest.is_batch_last = true;
        }
        vec![SessionPixAction::RetireSsa(retired)]
    }

    /// Allocates the next batch of SSA indices and asks for all of them in one action.
    ///
    /// The batch size is [`SupervisorConfig::ssas_per_request`], clamped rather than trusted since a
    /// supervisor can be built from a config that never went through `validate_pix_supervision`.
    ///
    /// Exactly one cycle of the batch — the last one allocated — carries
    /// [`PerSsaState::is_batch_last`], and only that cycle may ask for the successor batch. Without
    /// it every member would ask for one of its own, since the request flags are per-cycle; see that
    /// field for why the last rather than the first.
    ///
    /// What batching does change is the exposure *within* a batch: every cycle in it is unfunded at
    /// once, so the ceiling is `ssas_per_request` SSA quotas rather than one. That is the trade the
    /// knob exists to make, and it is why both deadlines are scaled by the same factor.
    fn emit_request_next_ssa(&mut self, now: Instant) -> Vec<SessionPixAction> {
        let batch = self.cfg.ssas_per_request.clamp(1, crate::MAX_SSA_BATCH_SIZE);
        let mut ssa_ids = Vec::with_capacity(batch);

        for _ in 0..batch {
            let index = self.next_ssa_index;

            let Ok(ssa_index) = SsaIndex::try_from(index) else {
                break;
            };
            let Some(next) = index.checked_add(1) else {
                // Index space exhausted mid-batch. Stop here rather than wrapping: a reused index
                // would collide with a live cycle. Whatever was allocated still goes out.
                break;
            };
            self.next_ssa_index = next;

            let ssa_id = SsaId::new(self.pseudonym, ssa_index);

            // Register each SSA now rather than on `SsaRequestSent`. Carrying out the action registers
            // an Exit commitment, which is what makes shares for that SSA processable — so
            // observations about it can reach us before the confirmation that we asked for it does.
            // Every handler ignores an SSA it has no record of, and for `UnverifiableShares` that
            // would mean failing open on exactly the signal that must fail closed.
            //
            // No deadline yet: the request has not gone out, so there is nothing to be late for. That
            // is what `SsaRequestSent` adds, per index.
            self.ssas
                .push(PerSsaState::new(ssa_id, self.dims.target_useful_shares(), now));
            ssa_ids.push(ssa_id);
        }

        // Nothing allocated means the index space is spent, and no further cycle can ever be funded.
        if ssa_ids.is_empty() {
            self.closed = true;
            return vec![SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)];
        }

        // The last cycle *actually* allocated carries the successor gate, which is what makes a
        // batch truncated by index exhaustion still able to ask for the next one.
        self.ssas
            .last_mut()
            .expect("a non-empty batch has just been pushed")
            .is_batch_last = true;

        vec![SessionPixAction::RequestSsa {
            ssa_ids,
            polys: self.dims.polys_per_ssa,
            threshold: self.dims.shares_per_poly,
        }]
    }

    fn find_ssa_idx(&self, ssa_id: &SsaId<HoprPseudonym>) -> Option<usize> {
        self.ssas.iter().position(|s| s.ssa_id == *ssa_id)
    }
}

// ---------------------------------------------------------------------------
// PerSsaState — deadline check (borrows immutably from self)
// ---------------------------------------------------------------------------

impl PerSsaState {
    /// Check which deadline expired, if any.
    fn check_deadlines(&self, now: Instant) -> Option<SessionPixCloseReason> {
        match self.phase {
            SsaPhase::AwaitingCommitment => self
                .commitment_deadline
                .filter(|d| now >= *d)
                .map(|_| SessionPixCloseReason::CommitmentTimeout),
            SsaPhase::AwaitingDeposit => self
                .deposit_deadline
                .filter(|d| now >= *d)
                .map(|_| SessionPixCloseReason::DepositTimeout),
            SsaPhase::Recovering => {
                // Hard deadline is immutable.
                if let Some(d) = self.recovery_hard_deadline
                    && now >= d
                {
                    return Some(SessionPixCloseReason::RecoveryDeadline);
                }
                // Idle deadline — service gating happens in handle_deadline.
                if let Some(d) = self.recovery_idle_deadline
                    && now >= d
                {
                    return Some(SessionPixCloseReason::RecoveryIdle);
                }
                None
            }
            SsaPhase::Recovered { .. } | SsaPhase::Closing => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use hopr_api::{
        HoprBalance,
        types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym},
    };
    use hopr_protocol_pix::{SsaId, SsaIndex, SsaRecoveryProgress};

    use super::*;

    /// A permissive baseline for the state-machine tests — *not* [`SupervisorConfig::default`].
    ///
    /// The shipped fault tolerances are zero, which would close a Session on the first unverifiable
    /// share and so make every multi-fault transition here unreachable. Tests that care about the
    /// shipped values say so by name.
    fn default_cfg() -> SupervisorConfig {
        SupervisorConfig {
            ssas_per_request: 1,
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

    fn dims(polys: u16, threshold: u16) -> SsaDimensions {
        SsaDimensions::new(polys, threshold)
    }

    fn pseudonym() -> HoprPseudonym {
        HoprPseudonym::random()
    }

    fn ssa_id(p: HoprPseudonym, idx: u32) -> SsaId<HoprPseudonym> {
        SsaId::new(p, SsaIndex::new(idx).unwrap())
    }

    fn make_progress(
        ssa_id: SsaId<HoprPseudonym>,
        useful: u64,
        target: u64,
        recovered_polys: u16,
    ) -> SsaRecoveryProgress<HoprPseudonym> {
        SsaRecoveryProgress {
            ssa_id,
            useful_shares: useful,
            target_useful_shares: target,
            recovered_polynomials: recovered_polys,
        }
    }

    fn sufficient_balance() -> HoprBalance {
        HoprBalance::new_base(1000)
    }

    /// Drives every cycle of a batch to `Recovering`, so only the batch's own gates hold anything back.
    fn fund_batch(sup: &mut SessionPixSupervisor, p: HoprPseudonym, batch: u32, now: Instant) {
        for idx in 1..=batch {
            let id = ssa_id(p, idx);
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(
                &SessionPixEvent::CommitmentVerified {
                    ssa_id: id,
                    expected_deposit: None,
                },
                now,
                0,
            );
            sup.handle_event(
                &SessionPixEvent::DepositConfirmed {
                    ssa_id: id,
                    amount: sufficient_balance(),
                },
                now,
                0,
            );
        }
    }

    fn _small_balance() -> HoprBalance {
        HoprBalance::new_base(1)
    }

    // ---------------------------------------------------------------
    // new / initial state
    // ---------------------------------------------------------------

    #[test]
    fn new_emits_initial_request_for_index_one() {
        let p = pseudonym();
        let (sup, actions) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());

        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SessionPixAction::RequestSsa {
                ssa_ids,
                polys,
                threshold,
            } => {
                assert_eq!(*polys, 10);
                assert_eq!(*threshold, 5);
                assert_eq!(ssa_ids.len(), 1, "the default batch is a single SSA");
                assert_eq!(ssa_ids[0].ssa_index(), SsaIndex::new(1).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }

        assert_eq!(sup.next_ssa_index, 2);
        assert!(!sup.closed);

        // The SSA is tracked from the moment it is requested, so that an observation about it
        // cannot arrive before there is anywhere to record it.
        assert_eq!(sup.ssas.len(), 1);
        assert_eq!(sup.ssas[0].phase, SsaPhase::AwaitingCommitment);
        assert!(
            sup.ssas[0].commitment_deadline.is_none(),
            "the delivery clock starts when the request goes out, not when it is queued"
        );
    }

    /// A batch is allocated and asked for as a unit: one action, `ssas_per_request` contiguous
    /// indices, one per-SSA record each, and the index counter advanced past all of them.
    ///
    /// One action rather than N is what puts the whole batch in a single `SsaRequest`, which is the
    /// point of the knob — and what makes the Entry's per-message cap the thing that has to accept it.
    #[test]
    fn a_batch_is_allocated_and_requested_as_one_action() {
        const BATCH: usize = 4;

        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH,
            ..default_cfg()
        };
        let (sup, actions) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());

        assert_eq!(actions.len(), 1, "the batch must travel as a single action");
        match &actions[0] {
            SessionPixAction::RequestSsa { ssa_ids, .. } => {
                assert_eq!(
                    ssa_ids.iter().map(|i| i.ssa_index().get()).collect::<Vec<_>>(),
                    (1..=BATCH as u32).collect::<Vec<_>>(),
                    "the batch must cover contiguous indices from 1"
                );
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }

        assert_eq!(
            sup.next_ssa_index,
            BATCH as u32 + 1,
            "the index counter must advance past the whole batch"
        );
        assert_eq!(
            sup.ssas.len(),
            BATCH,
            "every SSA in the batch needs its own record, or observations about it fail open"
        );
        assert!(sup.ssas.iter().all(|s| s.phase == SsaPhase::AwaitingCommitment));
        assert!(!sup.closed);
    }

    /// Both per-cycle deadlines are multiplied by the batch size.
    ///
    /// The commitment clock must scale because a batch's clocks all start together while the Entry has
    /// that many commitment sets to produce; the deposit clock must scale because an Entry funding a
    /// batch in order finishes the last one that many deposits after the first. Without either, a peer
    /// answering correctly but in sequence is closed for being slow.
    #[test]
    fn batching_scales_both_per_cycle_deadlines() {
        const BATCH: usize = 3;

        let p = pseudonym();
        let unit_commit = default_cfg().max_ssa_delivery_time;
        let unit_deposit = default_cfg().max_deposit_wait;
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH,
            ..default_cfg()
        };
        let now = Instant::now();
        let (mut sup, actions) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);
        let SessionPixAction::RequestSsa { ssa_ids, .. } = &actions[0] else {
            panic!("expected RequestSsa");
        };
        let ssa_ids = ssa_ids.clone();

        // Commitment clock: armed per index when the request goes out, at the scaled duration.
        for id in &ssa_ids {
            sup.handle_event(&SessionPixEvent::SsaRequestSent(*id), now, 0);
        }
        for (i, id) in ssa_ids.iter().enumerate() {
            let idx = sup.find_ssa_idx(id).expect("record must exist");
            assert_eq!(
                sup.ssas[idx].commitment_deadline,
                Some(now + BATCH as u32 * unit_commit),
                "cycle {i} must get the batch-scaled commitment deadline"
            );
        }

        // Deposit clock: armed when that cycle's commitment verifies, also at the scaled duration.
        let later = now + Duration::from_secs(1);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: ssa_ids[0],
                expected_deposit: None,
            },
            later,
            0,
        );
        let idx = sup.find_ssa_idx(&ssa_ids[0]).expect("record must exist");
        assert_eq!(sup.ssas[idx].phase, SsaPhase::AwaitingDeposit);
        assert_eq!(
            sup.ssas[idx].deposit_deadline,
            Some(later + BATCH as u32 * unit_deposit),
            "the deposit deadline must be batch-scaled too"
        );
    }

    /// An unvalidated `ssas_per_request` must not reach the deadline arithmetic or the allocator.
    ///
    /// A supervisor can be built from a config that never went through `validate_pix_supervision`, so
    /// zero must still request one SSA rather than none, and a value above the ceiling must be clamped
    /// rather than allocating an arbitrary batch or scaling a deadline past the duration cap.
    #[test]
    fn an_unvalidated_batch_size_is_clamped() {
        for (configured, expected) in [
            (0usize, 1usize),
            (crate::MAX_SSA_BATCH_SIZE + 5, crate::MAX_SSA_BATCH_SIZE),
        ] {
            let p = pseudonym();
            let cfg = SupervisorConfig {
                ssas_per_request: configured,
                ..default_cfg()
            };
            let (sup, actions) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
            match &actions[0] {
                SessionPixAction::RequestSsa { ssa_ids, .. } => assert_eq!(
                    ssa_ids.len(),
                    expected,
                    "a configured batch of {configured} must be clamped to {expected}"
                ),
                other => panic!("expected RequestSsa, got {other:?}"),
            }
            assert_eq!(sup.ssas.len(), expected);
        }
    }

    // ---------------------------------------------------------------
    // SsaRequestSent
    // ---------------------------------------------------------------

    #[test]
    fn request_sent_starts_commitment_deadline() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        let actions = sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        assert!(actions.is_empty());

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::AwaitingCommitment);
        assert!(ssa.commitment_deadline.is_some());
    }

    #[test]
    fn request_sent_is_idempotent() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        let actions = sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        assert!(actions.is_empty());
        assert_eq!(sup.ssas.len(), 1);
    }

    #[test]
    fn request_sent_wrong_pseudonym_closes() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let wrong_p = pseudonym();
        let now = Instant::now();

        let actions = sup.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id(wrong_p, 1)), now, 0);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], SessionPixAction::Close(_)));
    }

    // ---------------------------------------------------------------
    // CommitmentVerified
    // ---------------------------------------------------------------

    #[test]
    fn commitment_verified_starts_deposit_deadline_and_stores_expected_deposit() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);

        let actions = sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(HoprBalance::new_base(500)),
            },
            now,
            0,
        );
        assert!(actions.is_empty());

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::AwaitingDeposit);
        assert!(ssa.deposit_deadline.is_some());
        assert_eq!(ssa.expected_deposit, Some(HoprBalance::new_base(500)));
        assert!(ssa.commitment_deadline.is_none());
    }

    #[test]
    fn commitment_verified_is_idempotent() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            now,
            0,
        );
        let actions = sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            now,
            0,
        );
        assert!(actions.is_empty());
        assert_eq!(
            sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().phase,
            SsaPhase::AwaitingDeposit
        );
    }

    // ---------------------------------------------------------------
    // DepositConfirmed
    // ---------------------------------------------------------------

    #[test]
    fn sufficient_deposit_enters_recovering_starts_idle_and_hard() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(sufficient_balance()),
            },
            now,
            0,
        );

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], SessionPixAction::ReleaseService));

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::Recovering);
        assert!(ssa.recovery_idle_deadline.is_some());
        assert!(ssa.recovery_hard_deadline.is_some());
        assert!(ssa.deposit_deadline.is_none());
    }

    #[test]
    fn first_funding_emits_release_service_once() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id1 = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: Some(sufficient_balance()),
            },
            now,
            0,
        );

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], SessionPixAction::ReleaseService));

        // Second deposit on a different SSA should not emit ReleaseService again.
        sup.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id(p, 2)), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: ssa_id(p, 2),
                expected_deposit: Some(sufficient_balance()),
            },
            now,
            0,
        );
        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: ssa_id(p, 2),
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert!(actions.iter().all(|a| !matches!(a, SessionPixAction::ReleaseService)));
    }

    #[test]
    fn underfunded_deposit_is_noop_and_deposit_deadline_unchanged() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(HoprBalance::new_base(500)),
            },
            now,
            0,
        );

        let deadline_before = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().deposit_deadline;

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(100),
            },
            now,
            0,
        );
        assert!(actions.is_empty());

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::AwaitingDeposit);
        assert_eq!(ssa.deposit_deadline, deadline_before);
    }

    #[test]
    fn underfunded_then_sufficient_topup_confirms() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(HoprBalance::new_base(500)),
            },
            now,
            0,
        );

        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(100),
            },
            now,
            0,
        );

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(500),
            },
            now,
            0,
        );

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::Recovering);
        assert!(!actions.is_empty());
    }

    #[test]
    fn underfunded_then_sufficient_topup_accumulates() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(HoprBalance::new_base(500)),
            },
            now,
            0,
        );

        // First deposit: 300 < 500 -> accumulated=300, no-op.
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(300),
            },
            now,
            0,
        );

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::AwaitingDeposit);
        assert_eq!(ssa.accumulated_deposit, HoprBalance::new_base(300));

        // Second deposit: 200 + 300 >= 500 -> transitions to Recovering.
        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(200),
            },
            now,
            0,
        );

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::Recovering);
        assert_eq!(ssa.accumulated_deposit, HoprBalance::new_base(500));
        assert!(!actions.is_empty());
    }

    #[test]
    fn expected_deposit_none_accepts_any_amount() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            now,
            0,
        );

        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(1),
            },
            now,
            0,
        );
        assert_eq!(
            sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().phase,
            SsaPhase::Recovering
        );
    }

    #[test]
    fn min_deposit_config_rejects_dust_and_accepts_full() {
        let mut cfg = default_cfg();
        cfg.min_deposit = HoprBalance::new_base(500);
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(HoprBalance::new_base(500)),
            },
            now,
            0,
        );

        // Dust (100 < 500) → no-op, still AwaitingDeposit.
        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(100),
            },
            now,
            0,
        );
        assert!(actions.is_empty());
        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::AwaitingDeposit);

        // Sufficient (500 >= 500) → transitions to Recovering.
        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::new_base(500),
            },
            now,
            0,
        );
        assert!(!actions.is_empty());
        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::Recovering);
    }

    #[test]
    fn duplicate_deposit_confirmation_is_idempotent() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(sufficient_balance()),
            },
            now,
            0,
        );

        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert!(actions.is_empty());
    }

    #[test]
    fn wrong_ssa_deposit_does_not_transition_any_ssa() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();

        sup.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id(p, 1)), now, 0);

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: ssa_id(p, 99),
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert!(actions.is_empty());
    }

    // ---------------------------------------------------------------
    // Deadlines
    // ---------------------------------------------------------------

    #[test]
    fn commitment_deadline_expiry_closes_awaiting_commitment() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);

        let actions = sup.handle_deadline(start + Duration::from_secs(10), 0);
        assert!(actions.is_empty());

        let actions = sup.handle_deadline(start + Duration::from_secs(21), 0);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            SessionPixAction::Close(SessionPixCloseReason::CommitmentTimeout)
        ));
    }

    #[test]
    fn deposit_deadline_expiry_closes_awaiting_deposit() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(sufficient_balance()),
            },
            start,
            0,
        );

        let actions = sup.handle_deadline(start + Duration::from_secs(61), 0);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            SessionPixAction::Close(SessionPixCloseReason::DepositTimeout)
        ));
    }

    // ---------------------------------------------------------------
    // DepositObserverClosed
    // ---------------------------------------------------------------

    #[test]
    fn deposit_observer_closed_closes_with_distinct_reason() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(sufficient_balance()),
            },
            now,
            0,
        );

        let actions = sup.handle_event(&SessionPixEvent::DepositObserverClosed(id), now, 0);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            SessionPixAction::Close(SessionPixCloseReason::DepositObserverClosed)
        ));
    }

    // ---------------------------------------------------------------
    // Stale timer safety
    // ---------------------------------------------------------------

    #[test]
    fn stale_timer_wake_after_transition_does_not_close() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: Some(sufficient_balance()),
            },
            start,
            0,
        );

        // Deposit arrives before deadline.
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );

        let actions = sup.handle_deadline(start + Duration::from_secs(120), 0);
        assert!(
            actions
                .iter()
                .all(|a| { !matches!(a, SessionPixAction::Close(SessionPixCloseReason::DepositTimeout)) })
        );
    }

    // ---------------------------------------------------------------
    // Progress
    // ---------------------------------------------------------------

    #[test]
    fn useful_progress_extends_idle_only_hard_never_moves() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.max_recovery_idle = Duration::from_secs(60);
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );

        let hard_before = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().recovery_hard_deadline;
        let idle_before = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().recovery_idle_deadline;

        sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(id, 10, 50, 1)),
            start + Duration::from_secs(55),
            5,
        );

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.recovery_hard_deadline, hard_before);
        assert!(ssa.recovery_idle_deadline > idle_before);
    }

    #[test]
    fn hard_deadline_immutable_under_trickle_progress() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.max_recovery_idle = Duration::from_secs(10);
        cfg.max_recovery_time = Duration::from_secs(30);
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );

        for secs in [9u64, 19, 29] {
            sup.handle_event(
                &SessionPixEvent::RecoveryProgress(make_progress(id, secs + 1, 50, 1)),
                start + Duration::from_secs(secs),
                secs,
            );
        }

        let actions = sup.handle_deadline(start + Duration::from_secs(31), 30);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SessionPixAction::Close(SessionPixCloseReason::RecoveryDeadline) => {}
            other => panic!("expected RecoveryDeadline, got {other:?}"),
        }
    }

    #[test]
    fn idle_expiry_without_service_since_progress_rearms() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.max_recovery_idle = Duration::from_secs(10);
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );

        let actions = sup.handle_deadline(start + Duration::from_secs(11), 0);
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle)))
        );

        // SSA should still be in Recovering.
        assert_eq!(
            sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().phase,
            SsaPhase::Recovering
        );
    }

    #[test]
    fn idle_expiry_with_service_and_no_progress_closes_recovery_idle() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.max_recovery_idle = Duration::from_secs(10);
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            5,
        );

        // Idle fires with served_total=5 (same as watermark) → re-arm.
        let actions = sup.handle_deadline(start + Duration::from_secs(11), 5);
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle)))
        );

        // Now served_total increased to 10 (5 consumed since progress) → close.
        let actions = sup.handle_deadline(start + Duration::from_secs(22), 10);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle)))
        );
    }

    #[test]
    fn progress_resamples_served_total_watermark() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.max_recovery_idle = Duration::from_secs(10);
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            10,
        );

        sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(id, 5, 50, 1)),
            start + Duration::from_secs(5),
            15,
        );

        let actions = sup.handle_deadline(start + Duration::from_secs(16), 15);
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle)))
        );
    }

    #[test]
    fn equal_snapshot_is_noop() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            now,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        let progress = make_progress(id, 10, 50, 1);
        sup.handle_event(&SessionPixEvent::RecoveryProgress(progress), now, 5);

        let idle_before = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().recovery_idle_deadline;

        let actions = sup.handle_event(&SessionPixEvent::RecoveryProgress(progress), now, 5);
        assert!(actions.is_empty());

        assert_eq!(
            sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().recovery_idle_deadline,
            idle_before
        );
    }

    #[test]
    fn lower_snapshot_is_ignored_as_stale() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(&SessionPixEvent::RecoveryProgress(make_progress(id, 10, 50, 1)), now, 0);

        // Stale snapshot from concurrent processing is silently ignored.
        // Close-on-regression was rejected because ack batches are processed
        // with for_each_concurrent, so out-of-order arrival is possible.
        let actions = sup.handle_event(&SessionPixEvent::RecoveryProgress(make_progress(id, 5, 50, 1)), now, 0);
        assert!(actions.is_empty(), "stale snapshot should be ignored, got: {actions:?}");
    }

    #[test]
    fn inconsistent_target_closes_as_counter_regression() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        // Must register the SSA first.
        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            now,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        // RecoveryProgress with target != dims.target_useful_shares() = 50.
        let actions = sup.handle_event(&SessionPixEvent::RecoveryProgress(make_progress(id, 1, 99, 0)), now, 0);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            SessionPixAction::Close(SessionPixCloseReason::CounterRegression)
        ));
    }

    // ---------------------------------------------------------------
    // Fault tests
    // ---------------------------------------------------------------

    /// A fault reported before the request confirmation still counts.
    ///
    /// Carrying out `RequestSsa` registers the Exit commitment, which is what makes shares for that
    /// SSA processable — so a share can fail verification and be reported before `SsaRequestSent`
    /// gets back. Every handler ignores an SSA it has no record of, which on this event would mean
    /// failing open on the one signal that has to fail closed.
    #[test]
    fn an_unverifiable_share_counts_even_before_the_request_is_confirmed() {
        let p = pseudonym();
        let (mut sup, actions) = SessionPixSupervisor::new(SupervisorConfig::default(), dims(10, 5), p, Instant::now());
        let now = Instant::now();

        let id = match actions.as_slice() {
            [SessionPixAction::RequestSsa { ssa_ids, .. }] => ssa_ids[0],
            other => panic!("expected one RequestSsa, got {other:?}"),
        };

        // No `SsaRequestSent` — the confirmation is still in flight.
        let actions = sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 1,
            },
            now,
            0,
        );

        assert!(
            matches!(
                actions.as_slice(),
                [SessionPixAction::Close(
                    SessionPixCloseReason::TooManyUnverifiableShares
                )]
            ),
            "expected a close, got {actions:?}"
        );
    }

    #[test]
    fn invalid_share_past_a_configured_tolerance_closes() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);

        for i in 1..=3 {
            let actions = sup.handle_event(
                &SessionPixEvent::UnverifiableShares {
                    ssa_id: id,
                    observed_total: i,
                },
                now,
                0,
            );
            assert!(actions.is_empty(), "unexpected close at count {i}");
        }

        let actions = sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 4,
            },
            now,
            0,
        );
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            SessionPixAction::Close(SessionPixCloseReason::TooManyUnverifiableShares)
        ));
    }

    #[test]
    fn duplicate_absolute_counts_do_not_double_charge() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 2,
            },
            now,
            0,
        );
        let actions = sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 2,
            },
            now,
            0,
        );
        assert!(actions.is_empty());
        assert_eq!(sup.session_invalid_total, 2);
    }

    #[test]
    fn decreasing_invalid_count_is_ignored_as_stale() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 3,
            },
            now,
            0,
        );
        // Stale snapshot from concurrent processing is silently ignored.
        // Close-on-regression was rejected because ack batches are processed
        // with for_each_concurrent, so out-of-order arrival is possible.  A
        // fail-closed approach would be a self-inflicted DoS.
        let actions = sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 1,
            },
            now,
            0,
        );
        assert!(actions.is_empty(), "stale snapshot should be ignored, got: {actions:?}");
    }

    #[test]
    fn cross_peer_invalid_shares_accumulates_separately() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        // Advance the single SSA past request.
        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);

        // First peer reports 3 invalid shares (absolute total).
        sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 3,
            },
            now,
            0,
        );
        assert_eq!(sup.session_invalid_total, 3);

        // Second peer independently reports 5 invalid shares for the SAME SSA.
        // The supervisor must observe the *maximum* per-SSA absolute count and
        // charge the delta (5 - 3 = 2) as additional session-level faults.
        sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 5,
            },
            now,
            0,
        );
        assert_eq!(
            sup.session_invalid_total, 5,
            "cross-peer aggregate must track the max total"
        );

        // Third peer reports 7 — another delta of 2.
        sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 7,
            },
            now,
            0,
        );
        assert_eq!(sup.session_invalid_total, 7, "third peer delta must also be charged");

        // Stale report from the first peer (3 < per_ssa_total 7) is ignored.
        sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 3,
            },
            now,
            0,
        );
        assert_eq!(
            sup.session_invalid_total, 7,
            "stale cross-peer snapshot must not regress the aggregate"
        );
    }

    // ---------------------------------------------------------------
    // AlmostRecovered / Recovered
    // ---------------------------------------------------------------

    #[test]
    fn almost_recovered_while_recovering_requests_next_once() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id1 = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: None,
            },
            now,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), now, 0);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SessionPixAction::RequestSsa { ssa_ids, .. } => {
                assert_eq!(ssa_ids[0].ssa_index(), SsaIndex::new(2).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }

        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), now, 0);
        assert!(actions.is_empty());
    }

    /// A cycle queued behind the front of its batch must not be charged for the wait.
    ///
    /// The Entry serves a batch strictly in index order, so at deployed dimensions cycle 2 of a batch
    /// sits idle for the ~72 min it takes cycle 1's shares to be emitted. Starting its recovery clocks
    /// when its deposit confirmed measured that queue wait: `max_recovery_idle` would retire it inside
    /// a minute — the idle gate compares against *session-wide* service, which is plentiful while cycle
    /// 1 is served — and `max_recovery_time` would finish the job. Batching would have paid for
    /// `ssas_per_request` deposits and been able to use one.
    ///
    /// So the clocks start when the cycle reaches the front, and `max_recovery_time` bounds the
    /// recovery of a single cycle rather than its position in a queue.
    #[test]
    fn a_queued_cycle_starts_its_recovery_clocks_only_at_the_front_of_the_batch() {
        const BATCH: u32 = 3;

        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH as usize,
            max_recovery_idle: Duration::from_secs(10),
            max_recovery_time: Duration::from_secs(3600),
            ..default_cfg()
        };
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        fund_batch(&mut sup, p, BATCH, start);
        let target = sup.dims.target_useful_shares();

        let armed = |sup: &SessionPixSupervisor, idx: u32| {
            let ssa = sup
                .ssas
                .iter()
                .find(|s| s.ssa_id == ssa_id(p, idx))
                .expect("cycle must exist");
            (
                ssa.recovery_hard_deadline.is_some(),
                ssa.recovery_idle_deadline.is_some(),
            )
        };

        assert_eq!(
            armed(&sup, 1),
            (true, true),
            "the front cycle's clocks run from funding"
        );
        for queued in 2..=BATCH {
            assert_eq!(
                armed(&sup, queued),
                (false, false),
                "cycle {queued} is queued behind the front and must not be on the clock yet"
            );
        }

        // Cycle 1 is served for five times `max_recovery_idle`, with session-wide service climbing the
        // whole way. Nothing queued may be retired for that.
        for step in 1..=10u64 {
            let at = start + Duration::from_secs(step * 5);
            let served = step * 1000;
            sup.handle_event(
                &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 1), step, target, 1)),
                at,
                served,
            );
            let actions = sup.handle_deadline(at, served);
            assert!(
                !actions
                    .iter()
                    .any(|a| matches!(a, SessionPixAction::Close(_) | SessionPixAction::RetireSsa(_))),
                "no cycle may be closed or retired at +{}s, got {actions:?}",
                step * 5
            );
        }
        assert_eq!(sup.ssas.len(), BATCH as usize, "the whole batch must still be live");

        // Cycle 1 recovers, so cycle 2 reaches the front and only then goes on the clock.
        let handover = start + Duration::from_secs(60);
        sup.handle_event(&SessionPixEvent::Recovered(ssa_id(p, 1)), handover, 10_000);
        assert_eq!(armed(&sup, 2), (true, true), "cycle 2 is at the front now");
        assert_eq!(
            armed(&sup, 3),
            (false, false),
            "cycle 3 is still queued and must stay off the clock"
        );

        let ssa2 = sup.ssas.iter().find(|s| s.ssa_id == ssa_id(p, 2)).expect("cycle 2");
        assert_eq!(
            ssa2.recovery_hard_deadline,
            handover.checked_add(Duration::from_secs(3600)),
            "cycle 2's ceiling must be measured from its own turn, not from its deposit"
        );
        assert_eq!(
            ssa2.served_total_at_last_progress, 10_000,
            "the idle gate's baseline must be re-based at the handover, or the queue wait is charged to it"
        );
    }

    /// At `ssas_per_request > 1`, only the batch's **last** SSA may ask for the successor batch.
    ///
    /// `next_requested` is per-`PerSsaState`, so without a batch-wide gate every member of a batch
    /// answers its own `AlmostRecovered` with a fresh batch of `ssas_per_request`: one batch of 3
    /// becomes three, then nine, each SSA a separate on-chain deposit demanded of the Entry. The
    /// documented invariant on [`SessionPixSupervisor::emit_request_next_ssa`] is the opposite —
    /// "refuses to ask for another batch while any cycle of this one is unfunded" — and at
    /// `ssas_per_request == 1` the two readings coincide, which is why every other test here misses
    /// it.
    ///
    /// All three cycles are funded and `Recovering` before the first event, i.e. the case where the
    /// deferral in [`almost_recovered_while_awaiting_deposit_defers_request`] does *not* apply and
    /// only the batch gate can hold the request back.
    #[test]
    fn only_the_last_ssa_of_a_batch_asks_for_the_next_one() {
        const BATCH: usize = 3;

        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH,
            ..default_cfg()
        };
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let now = Instant::now();

        // Fund every cycle of the batch, so each one is in `Recovering`.
        for idx in 1..=BATCH as u32 {
            let id = ssa_id(p, idx);
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(
                &SessionPixEvent::CommitmentVerified {
                    ssa_id: id,
                    expected_deposit: None,
                },
                now,
                0,
            );
            sup.handle_event(
                &SessionPixEvent::DepositConfirmed {
                    ssa_id: id,
                    amount: sufficient_balance(),
                },
                now,
                0,
            );
        }
        assert_eq!(sup.next_ssa_index, BATCH as u32 + 1, "one batch allocated so far");

        // Every member but the last is silent, in the order the Exit actually reconstructs them.
        for idx in 1..BATCH as u32 {
            let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(ssa_id(p, idx)), now, 0);
            assert!(
                actions.is_empty(),
                "SSA {idx} is not the last of the batch and must not ask for a successor, got {actions:?}"
            );
            assert_eq!(
                sup.next_ssa_index,
                BATCH as u32 + 1,
                "SSA {idx} must not allocate any index"
            );
        }

        // The last member is the one that pipelines the refill.
        let last = ssa_id(p, BATCH as u32);
        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(last), now, 0);
        assert_eq!(actions.len(), 1, "the last SSA of the batch must ask exactly once");
        match &actions[0] {
            SessionPixAction::RequestSsa { ssa_ids, .. } => assert_eq!(
                ssa_ids.iter().map(|i| i.ssa_index().get()).collect::<Vec<_>>(),
                (BATCH as u32 + 1..=2 * BATCH as u32).collect::<Vec<_>>(),
                "the successor batch must be the next contiguous run"
            ),
            other => panic!("expected RequestSsa, got {other:?}"),
        }

        // Once-only is preserved for the last member too.
        assert!(
            sup.handle_event(&SessionPixEvent::AlmostRecovered(last), now, 0)
                .is_empty(),
            "a repeated AlmostRecovered on the last member must not ask again"
        );
        assert_eq!(sup.next_ssa_index, 2 * BATCH as u32 + 1);
    }

    /// Retiring the cycle that holds the successor gate must hand it to the newest survivor.
    ///
    /// Otherwise the batch is stranded: its siblings are barred from asking, so no replacement is
    /// ever requested and the Session dies on a recovery timeout instead of on its actual cause.
    /// The promoted cycle is the last of the batch that will really serve its quota, so the request
    /// still cannot come early.
    #[test]
    fn retiring_the_last_ssa_of_a_batch_hands_the_successor_gate_on() {
        const BATCH: usize = 3;

        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH,
            ..default_cfg()
        };
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let now = Instant::now();

        // Cycles 1 and 2 get funded; 3 — the gate holder — is left awaiting its deposit.
        for idx in 1..BATCH as u32 {
            let id = ssa_id(p, idx);
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(
                &SessionPixEvent::CommitmentVerified {
                    ssa_id: id,
                    expected_deposit: None,
                },
                now,
                0,
            );
            sup.handle_event(
                &SessionPixEvent::DepositConfirmed {
                    ssa_id: id,
                    amount: sufficient_balance(),
                },
                now,
                0,
            );
        }
        let last = ssa_id(p, BATCH as u32);
        sup.handle_event(&SessionPixEvent::SsaRequestSent(last), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: last,
                expected_deposit: Some(sufficient_balance()),
            },
            now,
            0,
        );

        // Its deposit observer gives up, so the gate holder is retired mid-batch.
        let actions = sup.handle_event(&SessionPixEvent::DepositObserverClosed(last), now, 0);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::RetireSsa(id) if *id == last)),
            "the unfunded last cycle must be retired, not close the Session, got {actions:?}"
        );
        assert!(!sup.closed, "surviving siblings must keep the Session alive");

        // The gate is now cycle 2's, and cycle 1 is still barred.
        assert!(
            sup.handle_event(&SessionPixEvent::AlmostRecovered(ssa_id(p, 1)), now, 0)
                .is_empty(),
            "cycle 1 is still not the last survivor and must stay silent"
        );
        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(ssa_id(p, 2)), now, 0);
        assert_eq!(
            actions.len(),
            1,
            "the newest survivor must have inherited the successor gate, got {actions:?}"
        );
        match &actions[0] {
            SessionPixAction::RequestSsa { ssa_ids, .. } => assert_eq!(
                ssa_ids.iter().map(|i| i.ssa_index().get()).collect::<Vec<_>>(),
                (BATCH as u32 + 1..=2 * BATCH as u32).collect::<Vec<_>>(),
                "the successor batch still starts past the whole retired batch"
            ),
            other => panic!("expected RequestSsa, got {other:?}"),
        }
    }

    #[test]
    fn almost_recovered_while_awaiting_deposit_defers_request() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id1 = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: Some(sufficient_balance()),
            },
            now,
            0,
        );

        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), now, 0);
        assert!(actions.is_empty());

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], SessionPixAction::ReleaseService));
        match &actions[1] {
            SessionPixAction::RequestSsa { ssa_ids, .. } => {
                assert_eq!(ssa_ids[0].ssa_index(), SsaIndex::new(2).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }
    }

    #[test]
    fn recovered_without_prior_early_event_falls_back_to_request() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id1 = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: None,
            },
            now,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        let actions = sup.handle_event(&SessionPixEvent::Recovered(id1), now, 0);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SessionPixAction::RequestSsa { ssa_ids, .. } => {
                assert_eq!(ssa_ids[0].ssa_index(), SsaIndex::new(2).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }
    }

    #[test]
    fn recovered_with_prior_early_event_does_not_fallback() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id1 = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: None,
            },
            now,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), now, 0);
        let actions = sup.handle_event(&SessionPixEvent::Recovered(id1), now, 0);
        assert!(actions.is_empty());
    }

    // ---------------------------------------------------------------
    // Close behavior
    // ---------------------------------------------------------------

    #[test]
    fn close_action_emitted_at_most_once() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_deadline(now + Duration::from_secs(100), 0);
        assert!(sup.closed);

        let empty = sup.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id(p, 999)), now, 0);
        assert!(empty.is_empty());
        let empty = sup.handle_deadline(now + Duration::from_secs(200), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn all_events_after_close_are_ignored() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_deadline(now + Duration::from_secs(100), 0);
        assert!(sup.closed);

        for ev in &[
            SessionPixEvent::SsaRequestSent(ssa_id(p, 2)),
            SessionPixEvent::CommitmentVerified {
                ssa_id: ssa_id(p, 2),
                expected_deposit: None,
            },
            SessionPixEvent::DepositConfirmed {
                ssa_id: ssa_id(p, 2),
                amount: sufficient_balance(),
            },
            SessionPixEvent::DepositObserverClosed(ssa_id(p, 2)),
            SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 2), 1, 50, 0)),
            SessionPixEvent::AlmostRecovered(ssa_id(p, 2)),
            SessionPixEvent::Recovered(ssa_id(p, 2)),
            SessionPixEvent::UnverifiableShares {
                ssa_id: ssa_id(p, 2),
                observed_total: 1,
            },
        ] {
            assert!(
                sup.handle_event(ev, now, 0).is_empty(),
                "event should be ignored after close"
            );
        }
    }

    // ---------------------------------------------------------------
    // next_deadline
    // ---------------------------------------------------------------

    #[test]
    fn next_deadline_none_when_no_ssas() {
        let (sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), pseudonym(), Instant::now());
        assert!(sup.next_deadline().is_none());
    }

    #[test]
    fn next_deadline_returns_earliest() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();

        // Two SSAs in flight with deadlines 40 s apart: the later one is armed second, so returning
        // the earliest is a real choice rather than the only candidate.
        let id1 = ssa_id(p, 1);
        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        let id2 = ssa_id(p, 2);
        sup.handle_event(&SessionPixEvent::SsaRequestSent(id2), now + Duration::from_secs(40), 0);

        let dl = sup.next_deadline().unwrap();

        let expected = now + Duration::from_secs(20);
        assert!((dl - expected).as_millis() < 10, "expected {expected:?}, got {dl:?}");
    }

    /// Pipelining a second SSA must not disturb the first one's deadlines.
    ///
    /// This used to be a `SessionManager` test asserting that two per-index `PixKillSwitch` abort
    /// handles coexisted. The deadlines are the supervisor's now, so the invariant is stated here
    /// against the state it actually lives in — and it is stronger, because a shared timer would
    /// satisfy "both handles present" but not "both instants unchanged".
    #[test]
    fn pipelining_a_second_ssa_leaves_the_first_ones_deadlines_alone() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();

        let id1 = ssa_id(p, 1);
        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: None,
            },
            now,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        let before = sup.ssas.iter().find(|s| s.ssa_id == id1).unwrap();
        assert_eq!(before.phase, SsaPhase::Recovering);
        let (hard_before, idle_before) = (before.recovery_hard_deadline, before.recovery_idle_deadline);

        // Early recovery on a funded SSA is what pipelines the next one.
        let later = now + Duration::from_secs(5);
        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), later, 0);
        let id2 = match actions.as_slice() {
            [SessionPixAction::RequestSsa { ssa_ids, .. }] => ssa_ids[0],
            other => panic!("expected exactly one RequestSsa, got {other:?}"),
        };
        assert_eq!(id2.ssa_index().get(), 2, "the pipelined SSA must take the next index");

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id2), later, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id2,
                expected_deposit: None,
            },
            later,
            0,
        );

        let after = sup.ssas.iter().find(|s| s.ssa_id == id1).unwrap();
        assert_eq!(
            after.phase,
            SsaPhase::Recovering,
            "pipelining moved the first SSA's phase"
        );
        assert_eq!(
            after.recovery_hard_deadline, hard_before,
            "pipelining moved the first SSA's hard recovery deadline"
        );
        assert_eq!(
            after.recovery_idle_deadline, idle_before,
            "pipelining moved the first SSA's idle recovery deadline"
        );

        // ...and the second one is independently armed rather than sharing the first's timers.
        let second = sup.ssas.iter().find(|s| s.ssa_id == id2).unwrap();
        assert_eq!(second.phase, SsaPhase::AwaitingDeposit);
        assert!(second.deposit_deadline.is_some());
    }

    // ---------------------------------------------------------------
    // SsaIndex overflow
    // ---------------------------------------------------------------

    #[test]
    fn ssa_index_overflow_fails_closed() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();

        let id1 = ssa_id(p, 1);
        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: None,
            },
            now,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        // Wind the allocator to the last usable index, so pipelining the next one overflows.
        sup.next_ssa_index = u32::MAX;

        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), now, 0);
        assert!(
            matches!(
                actions.as_slice(),
                [SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)]
            ),
            "expected a single Close, got {actions:?}"
        );
        assert!(sup.closed);
        assert_eq!(
            sup.ssas.len(),
            1,
            "an index that could not be allocated must not leave a record behind"
        );
    }

    // ---------------------------------------------------------------
    // action_result
    // ---------------------------------------------------------------

    #[test]
    fn request_failure_result_closes() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();

        let actions = sup.action_result(
            &SessionPixAction::RequestSsa {
                ssa_ids: vec![ssa_id(p, 1)],
                polys: 10,
                threshold: 5,
            },
            false,
            now,
        );
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            SessionPixAction::Close(SessionPixCloseReason::SupervisorUnavailable)
        ));
    }

    // ---------------------------------------------------------------
    // Tombstone and multi-SSA lifecycle
    // ---------------------------------------------------------------

    #[test]
    fn tombstone_expiry_clears_recovered_ssa() {
        let mut cfg = default_cfg();
        cfg.tombstone_retention_window = Duration::from_secs(10);
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );
        // Recovery pipelines the next SSA, which is tracked from the moment it is requested.
        sup.handle_event(&SessionPixEvent::Recovered(id), start, 0);
        assert_eq!(sup.ssas.len(), 2, "the recovered SSA's tombstone plus its successor");

        // Before tombstone expires — still present.
        let actions = sup.handle_deadline(start + Duration::from_secs(5), 0);
        assert!(actions.is_empty());
        assert_eq!(sup.ssas.len(), 2);

        // After tombstone expires — RetireSsa, and nothing else: the successor is still pending, so
        // the Session has not run out of SSAs. Closing here would kill a Session over a request
        // that is merely in flight.
        let actions = sup.handle_deadline(start + Duration::from_secs(11), 5);
        assert_eq!(actions.len(), 1, "expected RetireSsa alone, got {actions:?}");
        assert!(
            matches!(&actions[0], SessionPixAction::RetireSsa(rid) if *rid == id),
            "action should be RetireSsa({id}), got {:?}",
            actions[0]
        );
        assert_eq!(
            sup.ssas.len(),
            1,
            "the successor must survive its predecessor's retirement"
        );
    }

    #[test]
    fn all_ssas_terminal_closes_session_after_multi_ssa_close() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id1 = ssa_id(p, 1);

        // Set up first SSA through recovery.
        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id1,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            start,
            0,
        );
        // AlmostRecovered triggers next SSA request.
        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), start, 0);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], SessionPixAction::RequestSsa { .. }));
        sup.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id(p, 2)), start, 0);

        // Both deadlines have expired by now (SSA 1 hard deadline at start + 3600s,
        // SSA 2 commitment deadline at start + 20s). The loop processes SSA 1 first
        // (RecoveryDeadline → removed), then SSA 2 (CommitmentTimeout → session close).
        let actions = sup.handle_deadline(start + Duration::from_secs(7200), 5);
        assert_eq!(actions.len(), 2, "expected RetireSsa(id1) + Close(RecoveryDeadline)");
        assert!(
            matches!(&actions[0], SessionPixAction::RetireSsa(rid) if *rid == id1),
            "first action should be RetireSsa({id1}), got {:?}",
            actions[0]
        );
        // SSA 1 fails with RecoveryDeadline first, so that reason is surfaced.
        assert!(matches!(
            actions[1],
            SessionPixAction::Close(SessionPixCloseReason::RecoveryDeadline)
        ));
        assert!(sup.closed);
    }

    #[test]
    fn next_deadline_none_when_after_tombstone_expiry() {
        let mut cfg = default_cfg();
        cfg.tombstone_retention_window = Duration::from_secs(10);
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );
        sup.handle_event(&SessionPixEvent::Recovered(id), start, 0);

        // While tombstone is alive, next_deadline returns the tombstone_until.
        assert!(sup.next_deadline().is_some());

        // After tombstone expires, handle_deadline removes it and closes.
        sup.handle_deadline(start + Duration::from_secs(11), 0);

        assert!(sup.next_deadline().is_none());
    }

    #[test]
    fn action_result_close_sets_closed() {
        let p = pseudonym();
        let (mut sup, _actions) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        assert!(!sup.closed);

        // First fake-close via action_result.
        let _ = sup.action_result(&SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle), true, now);
        assert!(sup.closed);

        // After close, all subsequent calls are no-ops.
        let actions = sup.handle_deadline(now, 0);
        assert!(actions.is_empty());
        let actions = sup.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id(p, 1)), now, 0);
        assert!(actions.is_empty());
    }

    // -------------------------------------------------------------------
    // M-02: Event ordering guards
    // -------------------------------------------------------------------

    #[test]
    fn recovered_before_commitment_is_ignored() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);

        // Recovered arrives before CommitmentVerified — should be ignored.
        let actions = sup.handle_event(&SessionPixEvent::Recovered(id), now, 0);
        assert!(actions.is_empty(), "recovered before commitment should be ignored");
        assert!(!sup.closed);
    }

    #[test]
    fn recovered_before_deposit_is_ignored() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            now,
            0,
        );

        // Recovered arrives before DepositConfirmed — should be ignored.
        let actions = sup.handle_event(&SessionPixEvent::Recovered(id), now, 0);
        assert!(actions.is_empty(), "recovered before deposit should be ignored");
    }

    /// A fully recovered and subsequently funded SSA must not lead to the
    /// session being closed with `RecoveryIdle`. The `Recovered` event
    /// arriving during `AwaitingDeposit` is deferred via `recovered_pending`
    /// — once the deposit confirms and the SSA enters `Recovering`, the
    /// deferred tombstone transition fires immediately, retiring the SSA
    /// deadlines cleanly so the idle deadline can never fire.
    #[test]
    fn recovered_before_deposit_then_funded_session_survives() {
        let p = pseudonym();
        let start = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(2, 2), p, start);
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );

        // Entry delivered all shares before the on-chain deposit confirmed.
        // Recovered arrives while still AwaitingDeposit — deferred.
        sup.handle_event(&SessionPixEvent::Recovered(id), start, 0);

        // The deposit confirms — triggers deferred tombstone + RequestSsa.
        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            10,
        );
        assert!(!sup.closed, "session must be alive after funding");

        // The first SSA should have been tombstoned (not stuck in Recovering).
        let ssa1_phase = sup.ssas.iter().find(|s| s.ssa_id == id).map(|s| &s.phase);
        assert!(
            matches!(ssa1_phase, Some(SsaPhase::Recovered { .. })),
            "SSA 1 should be tombstoned, got {ssa1_phase:?}"
        );

        // A RequestSsa for the next SSA must have been emitted (so the
        // driver can start the successor).
        let next_id = ssa_id(p, 2);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::RequestSsa { ssa_ids, .. } if ssa_ids.contains(&next_id))),
            "expected RequestSsa for SSA 2, got actions: {actions:?}"
        );

        // The idle deadline fires well past max_recovery_idle. Since SSA 1
        // is a tombstone (deadlines are None), it should not trigger any
        // close. (No successor SSA state exists yet since the driver hasn't
        // sent back SsaRequestSent, so the drain path would retire the
        // tombstone and close with NoSsaRemaining — but that's a separate
        // concern from the bug we're fixing: the supervisor must not close
        // with RecoveryIdle.)
        let deadline_actions = sup.handle_deadline(start + Duration::from_secs(61), 100);
        assert!(
            !deadline_actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle))),
            "must not close with RecoveryIdle, got: {deadline_actions:?}"
        );
    }

    #[test]
    fn late_tombstone_progress_is_absorbed() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );
        sup.handle_event(&SessionPixEvent::Recovered(id), start, 0);

        // Late progress on tombstone — should be absorbed.
        let actions = sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(id, 50, 50, 10)),
            start,
            100,
        );
        assert!(actions.is_empty(), "late tombstone progress should be absorbed");
    }

    #[test]
    fn late_tombstone_unverifiable_shares_are_absorbed() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::CommitmentVerified {
                ssa_id: id,
                expected_deposit: None,
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );
        sup.handle_event(&SessionPixEvent::Recovered(id), start, 0);

        // Late unverifiable shares on tombstone — should be absorbed.
        let actions = sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: id,
                observed_total: 5,
            },
            start,
            100,
        );
        assert!(
            actions.is_empty(),
            "late tombstone unverifiable shares should be absorbed"
        );
    }
}
