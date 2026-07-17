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
        };

        let actions = s.emit_request_next_ssa(now);
        (s, actions)
    }

    /// Handle a lifecycle event.
    pub fn handle_event(&mut self, ev: &SessionPixEvent, now: Instant, served_total: u64) -> Vec<SessionPixAction> {
        if self.closed {
            return Vec::new();
        }

        match ev {
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
        }
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
            actions.push(SessionPixAction::RetireSsa(id));
        }

        // If no SSAs remain, close.
        if self.ssas.is_empty() && !self.closed {
            actions.push(SessionPixAction::Close(SessionPixCloseReason::NoSsaRemaining));
            self.closed = true;
        }

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

    fn on_ssa_request_sent(&mut self, ssa_id: &SsaId<HoprPseudonym>, now: Instant) -> Vec<SessionPixAction> {
        // Guard: ignore if we already have state (idempotent).
        if self.find_ssa(ssa_id).is_some() {
            return Vec::new();
        }

        // Validate pseudonym.
        if ssa_id.pseudonym() != &self.pseudonym {
            return vec![SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)];
        }

        let target = self.dims.target_useful_shares();
        let deadline = now.checked_add(self.cfg.max_ssa_delivery_time);
        let mut state = PerSsaState::new(*ssa_id, target, now);
        state.commitment_deadline = deadline;
        self.ssas.push(state);

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

        let deposit_deadline = now.checked_add(self.cfg.max_deposit_wait);
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

        // Transition to Recovering.
        let ssa = &mut self.ssas[idx];
        ssa.phase = SsaPhase::Recovering;
        ssa.deposit_deadline = None;
        ssa.recovery_idle_deadline = now.checked_add(self.cfg.max_recovery_idle);
        ssa.recovery_hard_deadline = now.checked_add(self.cfg.max_recovery_time);
        ssa.served_total_at_last_progress = served_total;

        let pending = ssa.next_request_pending_deposit;
        if pending {
            ssa.next_request_pending_deposit = false;
        }
        // End the mutable borrow on ssas[idx].
        let _ = ssa;

        let mut actions = Vec::new();

        if !self.release_service_emitted {
            self.release_service_emitted = true;
            actions.push(SessionPixAction::ReleaseService);
        }

        if pending {
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
            // AwaitingCommitment or unknown phase: do not set next_requested.
            // The next SSA will be requested when on_recovered transitions to
            // the tombstone phase, which checks next_requested independently.
            _ => Vec::new(),
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

        // Only accept recovery completion from the Recovering phase.
        // Recovered arriving in AwaitingCommitment or AwaitingDeposit
        // means recovery outpaced commitment/deposit notification —
        // ignore, the normal lifecycle transition will handle it when
        // those events arrive.
        match self.ssas[idx].phase {
            SsaPhase::Recovering => {}
            _ => return Vec::new(),
        }

        let next_requested = self.ssas[idx].next_requested;

        // Transition to tombstone.
        self.ssas[idx].phase = SsaPhase::Recovered {
            tombstone_until: now
                .checked_add(self.cfg.tombstone_retention_window)
                // The config validation caps all durations below overflow,
                // so checked_add should never fail in practice. Belt-and-
                // suspenders fallback: ~1 year in the future.
                .unwrap_or_else(|| now + Duration::from_secs(86400 * 365)),
        };
        self.ssas[idx].commitment_deadline = None;
        self.ssas[idx].deposit_deadline = None;
        self.ssas[idx].recovery_idle_deadline = None;
        self.ssas[idx].recovery_hard_deadline = None;

        let mut actions = Vec::new();
        if !next_requested {
            self.ssas[idx].next_requested = true;
            // Dropping the borrow on ssas[index] before emit_request_next_ssa.
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
        let retired = self.ssas[idx].ssa_id;
        self.ssas.remove(idx);
        vec![SessionPixAction::RetireSsa(retired)]
    }

    fn emit_request_next_ssa(&mut self, _now: Instant) -> Vec<SessionPixAction> {
        let index = self.next_ssa_index;

        let ssa_index = match SsaIndex::try_from(index) {
            Ok(i) => i,
            Err(_) => {
                return vec![SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)];
            }
        };

        match index.checked_add(1) {
            Some(next) => self.next_ssa_index = next,
            None => {
                self.closed = true;
                return vec![SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)];
            }
        }

        let ssa_id = SsaId::new(self.pseudonym, ssa_index);
        vec![SessionPixAction::RequestSsa {
            ssa_id,
            polys: self.dims.polys,
            threshold: self.dims.threshold,
        }]
    }

    fn find_ssa_idx(&self, ssa_id: &SsaId<HoprPseudonym>) -> Option<usize> {
        self.ssas.iter().position(|s| s.ssa_id == *ssa_id)
    }

    fn find_ssa(&self, ssa_id: &SsaId<HoprPseudonym>) -> Option<&PerSsaState> {
        self.ssas.iter().find(|s| s.ssa_id == *ssa_id)
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

    fn default_cfg() -> SupervisorConfig {
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

    fn dims(polys: u16, threshold: u16) -> SsaDimensions {
        SsaDimensions { polys, threshold }
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
                ssa_id,
                polys,
                threshold,
            } => {
                assert_eq!(*polys, 10);
                assert_eq!(*threshold, 5);
                assert_eq!(ssa_id.ssa_index(), SsaIndex::new(1).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }

        assert_eq!(sup.next_ssa_index, 2);
        assert!(!sup.closed);
        assert!(sup.ssas.is_empty());
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

        for secs in [9, 19, 29] {
            sup.handle_event(
                &SessionPixEvent::RecoveryProgress(make_progress(id, secs as u64 + 1, 50, 1)),
                start + Duration::from_secs(secs),
                secs as u64,
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
        sup.handle_event(&SessionPixEvent::RecoveryProgress(progress.clone()), now, 5);

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

    #[test]
    fn fourth_invalid_per_ssa_closes_at_defaults() {
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
            SessionPixAction::RequestSsa { ssa_id, .. } => {
                assert_eq!(ssa_id.ssa_index(), SsaIndex::new(2).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }

        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(id1), now, 0);
        assert!(actions.is_empty());
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
            SessionPixAction::RequestSsa { ssa_id, .. } => {
                assert_eq!(ssa_id.ssa_index(), SsaIndex::new(2).unwrap());
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
            SessionPixAction::RequestSsa { ssa_id, .. } => {
                assert_eq!(ssa_id.ssa_index(), SsaIndex::new(2).unwrap());
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
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        let dl = sup.next_deadline().unwrap();

        let expected = now + Duration::from_secs(20);
        assert!((dl - expected).as_millis() < 10, "expected {expected:?}, got {dl:?}");
    }

    // ---------------------------------------------------------------
    // SsaIndex overflow
    // ---------------------------------------------------------------

    #[test]
    fn ssa_index_overflow_fails_closed() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();

        sup.next_ssa_index = u32::MAX;

        let id1 = ssa_id(p, u32::MAX);
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
        assert!(!actions.is_empty());
        assert!(sup.closed);
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
                ssa_id: ssa_id(p, 1),
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
        sup.handle_event(&SessionPixEvent::Recovered(id), start, 0);

        assert_eq!(sup.ssas.len(), 1);

        // Before tombstone expires — still present.
        let actions = sup.handle_deadline(start + Duration::from_secs(5), 0);
        assert!(actions.is_empty());
        assert_eq!(sup.ssas.len(), 1);

        // After tombstone expires — RetireSsa emitted, then Close because no SSAs remain.
        let actions = sup.handle_deadline(start + Duration::from_secs(11), 5);
        assert_eq!(actions.len(), 2, "expected RetireSsa + Close");
        assert!(
            matches!(&actions[0], SessionPixAction::RetireSsa(rid) if *rid == id),
            "first action should be RetireSsa({id}), got {:?}",
            actions[0]
        );
        assert!(
            matches!(
                actions[1],
                SessionPixAction::Close(SessionPixCloseReason::NoSsaRemaining)
            ),
            "second action should be Close"
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
