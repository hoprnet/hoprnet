//! Deterministic [`SessionPixSupervisor`] — the pure state machine for PIX
//! session lifecycle.
//!
//! All methods take explicit [`std::time::Instant`] timestamps and a
//! `served_total: u64` sample from the [`ServiceGate`](super::gate::ServiceGate).
//! No method sleeps, spawns, or performs I/O.

use std::time::{Duration, Instant};

use hopr_api::{HoprBalance, types::internal::prelude::HoprPseudonym};
use hopr_protocol_pix::{SsaId, SsaIndex, SsaRecoveryProgress};

use super::{PixParams, SessionPixAction, SessionPixCloseReason, SessionPixEvent, SupervisorConfig};

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
    /// First SSA index allocated in this request batch.
    batch_id: u32,
    phase: SsaPhase,

    // Deadlines (None means not set for this phase).
    commitment_deadline: Option<Instant>,
    deposit_deadline: Option<Instant>,
    recovery_idle_deadline: Option<Instant>,
    recovery_hard_deadline: Option<Instant>,

    // Progress tracking.
    largest_useful_shares: u64,
    /// Largest `shares_seen` observed for this cycle — the liveness counter, which moves for surplus
    /// shares too. Also the staleness discriminator for snapshots, since it moves for every share
    /// while `largest_useful_shares` does not.
    largest_shares_seen: u64,
    target_useful_shares: u64,
    recovered_polynomials: u16,

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
    ///
    /// If the two-generation admission bound defers that request, `next_requested` remains set and
    /// the supervisor-level `successor_request_deferred` flag owns the retry; the gate must not be
    /// handed to another cycle and spend the request twice.
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
    fn new(ssa_id: SsaId<HoprPseudonym>, batch_id: u32, target_useful_shares: u64, _now: Instant) -> Self {
        Self {
            ssa_id,
            batch_id,
            phase: SsaPhase::AwaitingCommitment,
            commitment_deadline: None,
            deposit_deadline: None,
            recovery_idle_deadline: None,
            recovery_hard_deadline: None,
            largest_useful_shares: 0,
            largest_shares_seen: 0,
            target_useful_shares,
            recovered_polynomials: 0,
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
    pub(crate) dims: PixParams,
    pub(crate) pseudonym: HoprPseudonym,
    pub(crate) closed: bool,
    next_ssa_index: u32,
    /// The cycle the two share-order counters below are measured against — the earliest unrecovered
    /// cycle, i.e. the one the Entry should currently be serving. `None` before the first batch exists.
    share_order_front: Option<SsaIndex>,
    /// Useful shares booked on [`Self::share_order_front`] since it took the front.
    front_useful: u64,
    /// Useful shares booked on any *other* live cycle over that same span.
    ///
    /// `off_front_useful / (front_useful + off_front_useful)` is the share-order ratio — see
    /// [`SupervisorConfig::max_off_front_share_fraction`] for what it detects and why it is a fraction
    /// rather than a count. Both counters reset when the front changes, which is what keeps the window
    /// bounded to a single cycle's service.
    off_front_useful: u64,
    /// Whether the service gate currently follows a funded front cycle.
    service_open: bool,
    /// Front cycle for which the current gate mode was emitted.
    ///
    /// Tracking the identity as well as the mode lets a funded-to-funded handoff rebaseline the
    /// served-without-progress ceiling for the newly promoted cycle.
    service_front: Option<SsaIndex>,
    /// A front cycle became paid and terminal in one event, so an unfunded successor still earns a
    /// freshly restored predeposit allowance even though the gate never entered funded mode.
    paid_front_handoff: bool,
    /// A successor batch was earned but could not yet fit the two-generation reservation.
    successor_request_deferred: bool,
    /// Ordered SSAs (oldest first, newest last), including short-lived recovered tombstones.
    ssas: Vec<PerSsaState>,
    /// Tracks the first failure reason when multiple SSAs fail, so the
    /// earliest cause is used for the final `Close` action rather than the last.
    first_failure_reason: Option<SessionPixCloseReason>,
    /// Cycles this Session has lost without recovering them, over its whole life.
    ///
    /// Cumulative, and that is the whole point: retiring a failed member leaves no trace on its
    /// siblings, so without a counter that survives the retirement an Entry can lose one cycle per
    /// batch indefinitely while a single funded sibling holds the Session open. Bounded by
    /// [`SupervisorConfig::max_failed_cycles`].
    failed_cycles: usize,
    /// Greatest SSA index that has been retired (closed and removed).
    ///
    /// Prevents a stale `SsaRequestSent` from resurrecting a closed SSA. A high-watermark rather
    /// than the set of every retired index: the set grew for the whole life of the Session and was
    /// scanned linearly on every request, one entry per cycle, without bound.
    ///
    /// A watermark alone would be wrong if it were consulted first — retirement is not monotone,
    /// since a later batch member can lose its deposit while an earlier one is still recovering, and
    /// "at or below the greatest retired index" would then condemn that live earlier cycle. The
    /// guard is therefore placed *after* the live-record lookup, so a cycle that still exists always
    /// wins and the watermark only ever answers for indices that have none.
    highest_retired_ssa_index: Option<SsaIndex>,
}

impl SessionPixSupervisor {
    /// Create a new supervisor and emit the first `RequestSsa` action.
    pub fn new(
        cfg: SupervisorConfig,
        dims: PixParams,
        pseudonym: HoprPseudonym,
        now: Instant,
    ) -> (Self, Vec<SessionPixAction>) {
        let mut s = Self {
            cfg,
            dims,
            pseudonym,
            next_ssa_index: 1,
            share_order_front: None,
            front_useful: 0,
            off_front_useful: 0,
            closed: false,
            service_open: false,
            service_front: None,
            paid_front_handoff: false,
            successor_request_deferred: false,
            ssas: Vec::with_capacity(2),
            first_failure_reason: None,
            failed_cycles: 0,
            highest_retired_ssa_index: None,
        };

        let actions = s.emit_request_next_ssa(now);
        s.refresh_share_order_front();
        (s, actions)
    }

    /// Handle a lifecycle event.
    pub fn handle_event(&mut self, ev: &SessionPixEvent, now: Instant, served_total: u64) -> Vec<SessionPixAction> {
        if self.closed {
            return Vec::new();
        }

        let actions = match ev {
            SessionPixEvent::SsaRequestSent(ssa_id) => self.on_ssa_request_sent(ssa_id, now),
            SessionPixEvent::CommitmentVerified(ssa_id) => self.on_commitment_verified(ssa_id, now),
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

        let mut lifecycle_actions = actions;
        lifecycle_actions.extend(self.retry_deferred_successor_request(now));
        self.arm_recovery_clocks_for_earliest(now, served_total);
        let mut actions = if lifecycle_actions
            .iter()
            .any(|action| matches!(action, SessionPixAction::Close(_)))
        {
            Vec::new()
        } else {
            self.sync_service_gate()
        };
        actions.extend(lifecycle_actions);
        self.refresh_share_order_front();
        actions
    }

    /// Raises the retired-index watermark.
    ///
    /// Takes the maximum rather than assuming the caller retires in order, because it does not: a
    /// later batch member losing its deposit is retired before an earlier one that is still
    /// recovering.
    fn note_retired_index(&mut self, index: SsaIndex) {
        self.highest_retired_ssa_index = Some(match self.highest_retired_ssa_index {
            Some(highest) if highest >= index => highest,
            _ => index,
        });
    }

    /// Index of the earliest cycle that has not finished recovering — the one the Entry should be
    /// serving, given that emission is clamped to a single cycle.
    fn earliest_live_idx(&self) -> Option<usize> {
        self.ssas
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_terminal())
            .min_by_key(|(_, s)| s.ssa_id.ssa_index())
            .map(|(i, _)| i)
    }

    /// Keeps the service gate aligned with the funding state of the earliest live cycle.
    ///
    /// A funded-to-unfunded handoff restores the configured predeposit allowance. A
    /// funded-to-funded handoff emits `ReleaseService` too: the mode does not change, but the new
    /// front must receive its own served-without-progress ceiling rather than inheriting the tail of
    /// its predecessor's. An unfunded cycle that fails does not earn another allowance.
    fn sync_service_gate(&mut self) -> Vec<SessionPixAction> {
        if self.closed {
            return Vec::new();
        }

        let front = self.earliest_live_idx();
        let front_id = front.map(|idx| self.ssas[idx].ssa_id.ssa_index());
        let front_changed = front_id != self.service_front;
        let front_funded = front.is_some_and(|idx| self.ssas[idx].phase == SsaPhase::Recovering);
        let paid_front_handoff = std::mem::take(&mut self.paid_front_handoff);
        self.service_front = front_id;

        match (front_funded, self.service_open, front_changed) {
            (true, false, _) | (true, true, true) => {
                self.service_open = true;
                tracing::debug!(?front_id, "aligning service gate with funded front cycle");
                vec![SessionPixAction::ReleaseService]
            }
            (false, true, _) => {
                self.service_open = false;
                tracing::debug!(?front_id, "withholding service for unfunded front cycle");
                vec![SessionPixAction::WithholdService]
            }
            (false, false, _) if paid_front_handoff => {
                tracing::debug!(?front_id, "restoring predeposit service after paid front handoff");
                vec![SessionPixAction::WithholdService]
            }
            _ => Vec::new(),
        }
    }

    fn live_cycle_count(&self) -> usize {
        self.ssas.iter().filter(|ssa| !ssa.is_terminal()).count()
    }

    fn live_batch_count(&self) -> usize {
        let mut batches = Vec::with_capacity(crate::MAX_OVERLAPPING_BATCHES as usize);
        for ssa in self.ssas.iter().filter(|ssa| !ssa.is_terminal()) {
            if !batches.contains(&ssa.batch_id) {
                batches.push(ssa.batch_id);
            }
        }
        batches.len()
    }

    fn reserved_cycle_slots(&self) -> usize {
        self.cfg
            .ssas_per_request
            .clamp(1, crate::MAX_SSA_BATCH_SIZE)
            .saturating_mul(crate::MAX_OVERLAPPING_BATCHES as usize)
    }

    fn retry_deferred_successor_request(&mut self, now: Instant) -> Vec<SessionPixAction> {
        if self.closed || !self.successor_request_deferred {
            return Vec::new();
        }

        self.successor_request_deferred = false;
        self.emit_request_next_ssa(now)
    }

    /// Re-points the share-order accounting at the current front of the batch, clearing it if the front
    /// moved.
    ///
    /// Clearing on a change of front is what bounds the measurement window to one cycle's service, and
    /// it is why an Entry cannot launder a spree by eventually finishing a cycle: the reset is only
    /// reached by the front cycle *completing*, which is the thing the Exit wanted all along.
    ///
    /// Called after every event and every deadline sweep, so it covers both ways the front can move —
    /// the front cycle recovering, or being retired.
    fn refresh_share_order_front(&mut self) {
        let front = self.earliest_live_idx().map(|i| self.ssas[i].ssa_id.ssa_index());
        if front != self.share_order_front {
            self.share_order_front = front;
            self.front_useful = 0;
            self.off_front_useful = 0;
        }
    }

    /// Starts the recovery clocks of the earliest unrecovered cycle, if they are not running yet.
    ///
    /// A batch's cycles are served strictly in index order — the Entry's emission window is clamped to
    /// one cycle (see [`hopr_protocol_pix::SHARE_EMISSION_WINDOW`]) — so a cycle behind the front of
    /// the batch is *queued*, not stalled. Starting its clocks when its deposit confirmed, as this used
    /// to, measured the queue wait rather than the recovery: at deployed dimensions one cycle takes
    /// ~61 min of emission to exhaust, so every cycle after the first was retired by
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
        let Some(idx) = self.earliest_live_idx() else {
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
            self.note_retired_index(id.ssa_index());
            actions.push(SessionPixAction::RetireSsa(id));
        }

        // A terminal cycle may have released the older of two live batches. Keep retirement before
        // the replacement request in the action stream so the reconstructor drops the old guard
        // before allocating the successor.
        actions.extend(self.retry_deferred_successor_request(now));

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

        // Retiring or tombstoning a cycle can promote the next one to the front of the batch — and if
        // that one is already funded, promotion is the only moment left at which service can be
        // released, since its deposit event is long past.
        self.arm_recovery_clocks_for_earliest(now, served_total);
        let mut gate_actions = self.sync_service_gate();
        gate_actions.extend(actions);
        self.refresh_share_order_front();

        gate_actions
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

        // The live-record lookup comes first, and the staleness guard only answers for indices that
        // have no record. Retirement is not monotone — a later batch member can lose its deposit while
        // an earlier one is still recovering — so a watermark consulted first would silently ignore a
        // legitimate request for a cycle that is very much alive. Asking the records first removes the
        // assumption instead of relying on it.
        let Some(idx) = self.find_ssa_idx(ssa_id) else {
            // No live record. Either this index was retired and the event is a straggler, which is
            // benign and ignored, or we were never asked for it at all, which is a protocol fault.
            if self
                .highest_retired_ssa_index
                .is_some_and(|highest| ssa_id.ssa_index() <= highest)
            {
                return Vec::new();
            }
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

    fn on_commitment_verified(&mut self, ssa_id: &SsaId<HoprPseudonym>, now: Instant) -> Vec<SessionPixAction> {
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
        ssa.deposit_deadline = deposit_deadline;
        ssa.commitment_deadline = None;

        Vec::new()
    }

    /// Funds the cycle on the deposit pool's verdict. The supervisor does not second-guess the amount.
    ///
    /// Sufficiency is priced and judged outside this workspace. The Exit's pool is handed the
    /// [`DepositUpdated`](hopr_api::node::DepositUpdated) sender together with the byte quota when the
    /// deposit address arrives, and it sends on that channel once the deposit clears the price it
    /// charges for that quota — so by the time this runs, the question has been answered by the only
    /// component that can answer it. Nothing in the Session layer prices a byte.
    ///
    /// This used to compare the running total against `max(expected_deposit, min_deposit)`. Both are
    /// gone, and not because they were merely unused: a configured floor here is not a backstop behind
    /// the pool but a second authority alongside it, and one set above the pool's price fails a cycle
    /// that has already been paid for — silently, on `max_deposit_wait`, with neither side able to
    /// break the tie. `expected_deposit` was the same idea arriving over the wire, and the commitment
    /// never carried an amount to fill it.
    ///
    /// What remains is the one check that needs no price: a zero balance asserts that nothing was
    /// deposited, so it is not a verdict and cannot release service. The deadline keeps running and a
    /// later confirmation can still fund the cycle.
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
        let was_front = self.earliest_live_idx() == Some(idx);

        let ssa = &mut self.ssas[idx];
        if ssa.phase != SsaPhase::AwaitingDeposit {
            return Vec::new();
        }

        if amount == HoprBalance::zero() {
            return Vec::new();
        }

        // Transition to Recovering. The two recovery clocks are *not* started here — see
        // `arm_recovery_clocks_for_earliest`, which starts them when this cycle's turn comes.
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
            if was_front {
                self.paid_front_handoff = true;
            }
            actions.extend(self.perform_recovered_transition(idx, now));
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
        let new_seen = progress.shares_seen;

        // Counter regression check, on the liveness counter because it is the one that moves for every
        // share — a snapshot can be newer than the last one while carrying the same `useful_shares`.
        //
        // The relay-as-Exit pipeline processes acknowledgement batches with
        // for_each_concurrent, so absolute progress snapshots from different
        // batches can arrive out of order. Treat a stale snapshot as benign
        // noise rather than a protocol violation.
        if new_seen <= ssa.largest_shares_seen {
            return Vec::new();
        }
        ssa.largest_shares_seen = new_seen;

        // Liveness tier. A share arrived, useful or not, so the Entry is serving this cycle — which is
        // the question the egress gate and the recovery-idle deadline are asking. Keying either of them
        // on `useful_shares` instead makes a conforming Entry's surplus run look like silence, and for
        // the gate that is self-inflicted: withholding service removes the only thing that could
        // produce the next useful share. See `SsaRecoveryProgress::shares_seen`.
        ssa.served_total_at_last_progress = served_total;
        // Refreshing is not arming: only `arm_recovery_clocks_for_earliest` starts a cycle's clocks,
        // and `recovery_hard_deadline` is how a cycle records that it has reached the front. Without
        // that second condition a stray reordered share *starts* the idle clock on a cycle still
        // queued behind the front — which the Entry cannot then feed, because emission is clamped to
        // one cycle — and 60 s later the idle gate sees session-wide service climbing and retires it.
        // That is the queue-wait failure `arm_recovery_clocks_for_earliest` documents, reached
        // through the refresh path instead of the deposit one. The clocks arm together or not at all.
        if ssa.phase == SsaPhase::Recovering && ssa.recovery_hard_deadline.is_some() {
            ssa.recovery_idle_deadline = now.checked_add(self.cfg.max_recovery_idle);
        }

        // Payment tier. Only shares that advanced reconstruction are worth anything, so only they move
        // the recovery counters or count towards the share-order ratio.
        if new_useful > ssa.largest_useful_shares {
            let delta = new_useful - ssa.largest_useful_shares;
            ssa.largest_useful_shares = new_useful;
            ssa.recovered_polynomials = progress.recovered_polynomials;

            if let Some(reason) = self.book_share_order_progress(&progress.ssa_id, delta) {
                return vec![SessionPixAction::Close(reason)];
            }
        }

        // Only funded progress on the cycle service is currently charged against may reopen the
        // Session-wide ceiling. Unfunded progress is the H11 bypass; off-front progress belongs to a
        // queued cycle and must not buy service for the front either.
        if self.ssas[idx].phase == SsaPhase::Recovering && self.earliest_live_idx() == Some(idx) {
            vec![SessionPixAction::ProgressNotification]
        } else {
            Vec::new()
        }
    }

    /// Attributes `delta` useful shares to the front of the batch or behind it, and judges the ratio.
    ///
    /// The Entry serves a batch strictly in index order, so progress on anything but the front cycle is
    /// service the Exit cannot yet be paid for. See
    /// [`SupervisorConfig::max_off_front_share_fraction`] for the threat and for why this is measured
    /// as a fraction; the judgement waits for
    /// [`SupervisorConfig::min_share_order_sample`] shares of evidence.
    fn book_share_order_progress(
        &mut self,
        ssa_id: &SsaId<HoprPseudonym>,
        delta: u64,
    ) -> Option<SessionPixCloseReason> {
        let front = self.share_order_front?;
        if ssa_id.ssa_index() == front {
            self.front_useful = self.front_useful.saturating_add(delta);
        } else {
            self.off_front_useful = self.off_front_useful.saturating_add(delta);
        }

        // Judged on every event, not only off-front ones: the sample can cross the floor on a share
        // that lands *on* the front cycle, and the ratio is already whatever it is by then. Waiting for
        // the next off-front share would only delay the same verdict.
        let total = self.front_useful.saturating_add(self.off_front_useful);
        if total < self.cfg.min_share_order_sample || total == 0 {
            return None;
        }

        let fraction = self.off_front_useful as f64 / total as f64;
        if fraction > self.cfg.max_off_front_share_fraction {
            tracing::error!(
                %ssa_id, %front, fraction,
                limit = self.cfg.max_off_front_share_fraction,
                front_useful = self.front_useful,
                off_front_useful = self.off_front_useful,
                "entry is serving the batch out of order"
            );
            return Some(SessionPixCloseReason::BatchServedOutOfOrder);
        }

        tracing::trace!(%ssa_id, %front, fraction, "progress behind the front of the batch");
        None
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

    /// Closes the Session on the first unverifiable-share report. There is no tolerance to configure.
    ///
    /// Shares are not checked on arrival, so a report is not "a bad share" — it means a whole
    /// polynomial's share set failed to open its commitment, which surfaces only once `threshold` of
    /// them have been interpolated. `SsaPart::add_share` then marks that part failed and releases its
    /// shares, and nothing ever clears the flag; since the SSA is the sum of *every* polynomial's
    /// constant term, the cycle can no longer be reconstructed by any means and will never pay.
    ///
    /// So serving on is not tolerance, it is donation: every packet past this point is unpaid with no
    /// recovery to preserve at the end of it. That holds however the failure arose, which is why this
    /// is not a knob — a false positive from a verification bug leaves the part just as permanently
    /// failed, so a tolerance would buy unpaid service rather than the cycle it was raised to save.
    /// Closing here caps the exposure at the `threshold` packets already served rather than a multiple
    /// of it.
    ///
    /// `observed_total` is the reconstructor's cross-peer aggregate, and with no limit to compare it
    /// against it is logged rather than accumulated: it says how many polynomials the batch that
    /// triggered this took down, which is the difference between one bad relayer and a peer sending
    /// garbage wholesale.
    ///
    /// Note this closes the Session directly rather than through
    /// [`close_ssa_and_collect`](Self::close_ssa_and_collect), so a batch's surviving siblings do not
    /// keep it alive: unlike a lost deposit, this fault is evidence about the peer rather than about
    /// one cycle.
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

        // Absorb late reports on a tombstoned or already-closing SSA: there is no service left to
        // stop, and the cycle they condemn has already left. Concurrent ack batches make these
        // ordinary rather than exceptional.
        if self.ssas[idx].is_terminal() {
            return Vec::new();
        }

        tracing::warn!(
            %ssa_id,
            observed_total,
            phase = ?self.ssas[idx].phase,
            "closing PIX session: a polynomial's share set failed to open its commitment"
        );

        vec![SessionPixAction::Close(SessionPixCloseReason::UnverifiableShares)]
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

        // Charged before the branches below, because the whole point is a count that outlives the
        // record being retired. Every path into this function is a cycle lost without recovering —
        // a recovered cycle leaves through tombstone expiry in `handle_deadline`, not through here.
        self.failed_cycles += 1;

        // Warn-level diagnostic with full SSA state before closing.
        let ssa = &self.ssas[idx];
        tracing::warn!(
            ssa_id = %ssa.ssa_id,
            ?reason,
            phase = ?ssa.phase,
            largest_useful_shares = ssa.largest_useful_shares,
            target_useful_shares = ssa.target_useful_shares,
            recovered_polynomials = ssa.recovered_polynomials,
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

        // A batch has siblings to fall back on, which is what makes retiring one member survivable —
        // but only up to a point. Past the limit the Session goes, with the *first* failure's reason
        // rather than this one's, so the report names the cause instead of the last symptom.
        //
        // Strictly greater: the field is how many failures are *tolerated*, so the shipping value of
        // one keeps the existing "retire the member and carry on" behaviour for a batch's first loss
        // — including handing on the successor gate below — and closes on the second.
        if self.failed_cycles > self.cfg.max_failed_cycles.max(1) {
            tracing::warn!(
                failed_cycles = self.failed_cycles,
                max_failed_cycles = self.cfg.max_failed_cycles,
                ?close_reason,
                "closing PIX session: too many cycles lost without recovering"
            );
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
        self.note_retired_index(retired.ssa_index());
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
    /// Before allocating, this enforces both halves of the admission reservation: no more than
    /// [`crate::MAX_OVERLAPPING_BATCHES`] live generations and no more than that many full batches'
    /// live cycles. A request earned while the reservation is full is retried after a generation
    /// releases its reconstructor state.
    ///
    /// What batching does change is the exposure *within* a batch: every cycle in it is unfunded at
    /// once, so the ceiling is `ssas_per_request` SSA quotas rather than one. That is the trade the
    /// knob exists to make, and it is why both deadlines are scaled by the same factor.
    fn emit_request_next_ssa(&mut self, now: Instant) -> Vec<SessionPixAction> {
        let batch = self.cfg.ssas_per_request.clamp(1, crate::MAX_SSA_BATCH_SIZE);
        let live_cycles = self.live_cycle_count();
        let live_batches = self.live_batch_count();
        if live_batches >= crate::MAX_OVERLAPPING_BATCHES as usize
            || live_cycles.saturating_add(batch) > self.reserved_cycle_slots()
        {
            self.successor_request_deferred = true;
            tracing::debug!(
                live_cycles,
                live_batches,
                requested = batch,
                reserved = self.reserved_cycle_slots(),
                "deferring successor SSA batch until an older batch is released"
            );
            return Vec::new();
        }

        self.successor_request_deferred = false;
        let batch_id = self.next_ssa_index;
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
            self.ssas.push(PerSsaState::new(
                ssa_id,
                batch_id,
                self.dims.target_useful_shares(),
                now,
            ));
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
            params: self.dims,
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

    /// A compact baseline for the state-machine tests — *not* [`SupervisorConfig::default`].
    ///
    /// Deadlines and budgets are shrunk to keep the tests short; the tolerances that remain are at
    /// their shipped values. Tests that care about a specific one say so by name.
    fn default_cfg() -> SupervisorConfig {
        SupervisorConfig {
            ssas_per_request: 1,
            max_failed_cycles: 1,
            max_ssa_delivery_time: Duration::from_secs(20),
            max_deposit_wait: Duration::from_secs(60),
            max_recovery_idle: Duration::from_secs(60),
            max_recovery_time: Duration::from_secs(3600),
            max_off_front_share_fraction: 0.25,
            min_share_order_sample: 16384,
            max_predeposit_packets: 1024,
            max_served_without_progress: 256,
            tombstone_retention_window: Duration::from_secs(30),
        }
    }

    /// Test dimensions, with a deliberately non-zero surplus.
    ///
    /// The supervisor must ignore the surplus entirely — it counts *useful* shares, and a surplus
    /// share is by definition one that arrives after its polynomial is already complete. A non-zero
    /// value here is what would make a leak into `target_useful_shares` visible; zero would hide it.
    fn dims(polys: u16, threshold: u8) -> PixParams {
        PixParams::try_new(polys, threshold, 7, crate::types::LOCAL_PIX_SUITE).expect("test dimensions must be valid")
    }

    fn pseudonym() -> HoprPseudonym {
        HoprPseudonym::random()
    }

    fn ssa_id(p: HoprPseudonym, idx: u32) -> SsaId<HoprPseudonym> {
        SsaId::new(p, SsaIndex::new(idx).unwrap())
    }

    /// A snapshot in which every share seen was useful — the shape of a cycle's first
    /// `threshold` shares per polynomial, and what every test predating the liveness split means.
    fn make_progress(
        ssa_id: SsaId<HoprPseudonym>,
        useful: u64,
        target: u64,
        recovered_polys: u16,
    ) -> SsaRecoveryProgress<HoprPseudonym> {
        make_progress_seen(ssa_id, useful, useful, target, recovered_polys)
    }

    /// A snapshot where `seen` and `useful` can differ, i.e. the Entry is serving surplus.
    fn make_progress_seen(
        ssa_id: SsaId<HoprPseudonym>,
        useful: u64,
        seen: u64,
        target: u64,
        recovered_polys: u16,
    ) -> SsaRecoveryProgress<HoprPseudonym> {
        SsaRecoveryProgress {
            ssa_id,
            useful_shares: useful,
            shares_seen: seen,
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
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
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

    /// Drives a range of a batch's cycles to `AwaitingDeposit` and leaves them there.
    ///
    /// The complement of [`fund_batch`]: this is the state a cycle is *lost* from, and a funded one
    /// cannot be — `DepositObserverClosed` on a recovering cycle is a no-op.
    fn commit_unfunded(
        sup: &mut SessionPixSupervisor,
        p: HoprPseudonym,
        indices: std::ops::RangeInclusive<u32>,
        now: Instant,
    ) {
        for idx in indices {
            let id = ssa_id(p, idx);
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
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
            SessionPixAction::RequestSsa { ssa_ids, params } => {
                assert_eq!(*params, dims(10, 5));
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(ssa_ids[0]), later, 0);
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
    fn commitment_verified_starts_the_deposit_deadline() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);

        let actions = sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
        assert!(actions.is_empty());

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::AwaitingDeposit);
        assert!(ssa.deposit_deadline.is_some());
        assert!(ssa.commitment_deadline.is_none());
    }

    #[test]
    fn commitment_verified_is_idempotent() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
        let actions = sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), now, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(ssa_id(p, 2)), now, 0);
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
    fn a_funded_successor_gets_a_fresh_service_ceiling_when_it_reaches_the_front() {
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 2;
        let p = pseudonym();
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);

        fund_batch(&mut sup, p, 2, now);
        let actions = sup.handle_event(&SessionPixEvent::Recovered(ssa_id(p, 1)), now, 0);

        assert!(
            matches!(actions.as_slice(), [SessionPixAction::ReleaseService]),
            "a funded-to-funded handoff must rebaseline the ceiling, got {actions:?}"
        );
    }

    /// Any non-zero amount funds the cycle, because the amount is not the supervisor's to judge.
    ///
    /// The pool prices the quota and sends only once the deposit clears that price, so a confirmation
    /// is a verdict rather than evidence. A dust amount arriving here means the pool decided dust was
    /// enough — a pool bug, or an operator's deliberate pricing, and either way not something a floor
    /// in this crate could correct without overruling the component that knows the price.
    #[test]
    fn any_nonzero_deposit_funds_the_cycle() {
        for amount in [HoprBalance::new_base(1), sufficient_balance()] {
            let p = pseudonym();
            let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
            let now = Instant::now();
            let id = ssa_id(p, 1);

            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);

            let actions = sup.handle_event(&SessionPixEvent::DepositConfirmed { ssa_id: id, amount }, now, 0);

            assert_eq!(
                sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().phase,
                SsaPhase::Recovering,
                "a confirmation of {amount} is the pool's verdict and must fund the cycle"
            );
            assert!(!actions.is_empty());
        }
    }

    /// A zero balance is the one confirmation that asserts nothing, and must not release service.
    ///
    /// It also must not consume the cycle's chance to be funded: the deadline keeps running, and a
    /// later confirmation still counts. That is what makes it safe for the observer to forward every
    /// message the pool sends rather than latching on the first.
    #[test]
    fn a_zero_deposit_is_not_a_verdict_and_leaves_the_cycle_fundable() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);

        let deadline_before = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().deposit_deadline;

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: HoprBalance::zero(),
            },
            now,
            0,
        );
        assert!(actions.is_empty());

        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).unwrap();
        assert_eq!(ssa.phase, SsaPhase::AwaitingDeposit);
        assert_eq!(
            ssa.deposit_deadline, deadline_before,
            "a zero confirmation must not extend or clear the deadline"
        );

        // The next confirmation still funds it.
        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert_eq!(
            sup.ssas.iter().find(|s| s.ssa_id == id).unwrap().phase,
            SsaPhase::Recovering
        );
        assert!(!actions.is_empty());
    }

    /// Funding a queued cycle must not release service while an unfunded predecessor is live, but
    /// that funding must take effect if the predecessor is later retired. Otherwise the one-shot
    /// Session gate remains in predeposit mode even though the newly promoted front is funded, and
    /// no further deposit event exists to wake it.
    #[test]
    fn funded_successor_releases_service_when_unfunded_front_is_retired() {
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 2;
        let p = pseudonym();
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);
        let front = ssa_id(p, 1);
        let successor = ssa_id(p, 2);

        for id in [front, successor] {
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
        }

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: successor,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert!(
            actions.iter().all(|a| !matches!(a, SessionPixAction::ReleaseService)),
            "funding a cycle behind an unfunded front must not release service"
        );

        let actions = sup.handle_event(&SessionPixEvent::DepositObserverClosed(front), now, 0);
        assert!(
            actions.iter().any(|a| matches!(a, SessionPixAction::ReleaseService)),
            "retiring the unfunded front must release service for its already-funded successor; got {actions:?}"
        );
    }

    /// A cycle that funds and goes terminal in the same call must re-arm rather than open service.
    ///
    /// `Recovered` can arrive while the cycle is still `AwaitingDeposit` — the Exit finishes
    /// reconstructing before the chain observer reports the deposit — and is deferred until the
    /// deposit lands. That one call then both funds the cycle and retires it. Opening the gate at
    /// that point would serve against its unfunded successor; withholding restores the successor's
    /// allowance, and its own deposit opens the gate normally.
    #[test]
    fn a_cycle_that_funds_and_recovers_at_once_rearms_for_its_unfunded_successor() {
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 2;
        let p = pseudonym();
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);
        let front = ssa_id(p, 1);
        let successor = ssa_id(p, 2);

        for id in [front, successor] {
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
        }

        // Recovery completes before the deposit is observed, so the event is held.
        let actions = sup.handle_event(&SessionPixEvent::Recovered(front), now, 0);
        assert!(
            actions.iter().all(|a| !matches!(a, SessionPixAction::ReleaseService)),
            "recovering an unfunded cycle must not release service"
        );

        // The deposit lands: the cycle funds and is retired in the same call. Its successor remains
        // unfunded, so the next allowance is restored without opening funded service.
        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: front,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert!(
            matches!(actions.as_slice(), [SessionPixAction::WithholdService]),
            "the paid handoff must re-arm without opening service, got {actions:?}"
        );

        let actions = sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: successor,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert!(
            actions.iter().any(|a| matches!(a, SessionPixAction::ReleaseService)),
            "the successor's own deposit must open service, got {actions:?}"
        );
    }

    #[test]
    fn progress_on_an_unfunded_front_does_not_notify_the_service_gate() {
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 2;
        cfg.max_off_front_share_fraction = 1.0;
        let p = pseudonym();
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);
        let funded_front = ssa_id(p, 1);
        let unfunded_successor = ssa_id(p, 2);

        for id in [funded_front, unfunded_successor] {
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
        }
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: funded_front,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        sup.handle_event(&SessionPixEvent::Recovered(funded_front), now, 0);

        let actions = sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(
                unfunded_successor,
                1,
                sup.dims.target_useful_shares(),
                0,
            )),
            now,
            1,
        );

        assert!(
            actions
                .iter()
                .all(|action| !matches!(action, SessionPixAction::ProgressNotification)),
            "an unfunded front must not reopen the funded service ceiling, got {actions:?}"
        );
    }

    #[test]
    fn progress_behind_the_front_does_not_notify_the_service_gate() {
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 2;
        cfg.max_off_front_share_fraction = 1.0;
        let p = pseudonym();
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);

        fund_batch(&mut sup, p, 2, now);
        let actions = sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 2), 1, sup.dims.target_useful_shares(), 0)),
            now,
            1,
        );

        assert!(
            actions.is_empty(),
            "off-front progress must not reopen the gate, got {actions:?}"
        );
    }

    /// A live cycle below the retired watermark must still accept its own request event.
    ///
    /// Retirement is not monotone: a later batch member can lose its deposit — timeout, observer
    /// closure — while an earlier one is still recovering, which puts the watermark *above* a live
    /// index. A staleness guard consulted before the record lookup would then silently drop the
    /// earlier cycle's `SsaRequestSent`, leaving its commitment deadline unarmed and the Entry free
    /// to never answer. Looking the record up first is what removes the assumption.
    #[test]
    fn a_live_cycle_below_the_retired_watermark_still_arms_its_deadline() {
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 2;
        let p = pseudonym();
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);
        let earlier = ssa_id(p, 1);
        let later = ssa_id(p, 2);

        // The later member is retired first, which puts the watermark above the earlier one.
        sup.handle_event(&SessionPixEvent::SsaRequestSent(later), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(later), now, 0);
        sup.handle_event(&SessionPixEvent::DepositObserverClosed(later), now, 0);
        assert_eq!(
            Some(later.ssa_index()),
            sup.highest_retired_ssa_index,
            "the later member must have set the watermark"
        );

        // The earlier member is still live, and its request event must still arm the clock.
        sup.handle_event(&SessionPixEvent::SsaRequestSent(earlier), now, 0);
        let armed = sup
            .ssas
            .iter()
            .find(|s| s.ssa_id == earlier)
            .expect("the earlier cycle must still be live")
            .commitment_deadline;
        assert!(
            armed.is_some(),
            "a live cycle below the watermark must still have its commitment deadline armed"
        );
    }

    /// And a straggler for an index that really is gone stays benign rather than closing the Session.
    ///
    /// A batch of two, so retiring one member leaves the Session alive — retiring the last live cycle
    /// closes it outright, and a closed supervisor ignores everything, which would make the second
    /// half of this test vacuous.
    #[test]
    fn a_request_event_for_a_retired_index_is_ignored() {
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 2;
        let p = pseudonym();
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);
        let retired = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(retired), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(retired), now, 0);
        sup.handle_event(&SessionPixEvent::DepositObserverClosed(retired), now, 0);
        assert!(!sup.closed, "the second batch member must keep the Session alive");

        let actions = sup.handle_event(&SessionPixEvent::SsaRequestSent(retired), now, 0);
        assert!(
            actions.iter().all(|a| !matches!(a, SessionPixAction::Close(_))),
            "a straggling request for a retired index must be ignored, not fatal; got {actions:?}"
        );

        // An index we were never asked for is a different matter — above the watermark and with no
        // record, it can only be an SSA this supervisor never registered.
        let actions = sup.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id(p, 9)), now, 0);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::InvalidTransition))),
            "an index above the watermark with no record is a protocol fault; got {actions:?}"
        );
    }

    #[test]
    fn duplicate_deposit_confirmation_is_idempotent() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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

    /// A snapshot carrying only surplus must keep the cycle alive without paying it anything.
    ///
    /// This is the whole of C1 at the supervisor. A conforming Entry drains an emission window's
    /// surplus in one contiguous run — `surplus × min(polys, 256)` packets, 4096 at the shipped
    /// dimensions — during which `useful_shares` does not move at all. Every one of those packets is
    /// service the Exit is consuming, so a supervisor that keyed liveness on `useful_shares` would
    /// let the run exhaust `max_served_without_progress`, park the writer, and then close an honest
    /// Session with `RecoveryIdle` — the re-arm branch cannot save it, because service *was*
    /// consumed. Liveness must therefore move on `shares_seen`, and payment must not.
    #[test]
    fn a_surplus_only_snapshot_is_liveness_without_payment() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.max_recovery_idle = Duration::from_secs(10);
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );

        // One useful share, to establish a baseline both counters agree on.
        sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(id, 1, 50, 0)),
            start + Duration::from_secs(1),
            10,
        );

        // Then a run of pure surplus: `shares_seen` climbs, `useful_shares` is pinned, and service is
        // being consumed throughout.
        for (i, seen) in (2..=6u64).enumerate() {
            let actions = sup.handle_event(
                &SessionPixEvent::RecoveryProgress(make_progress_seen(id, 1, seen, 50, 0)),
                start + Duration::from_secs(2 + i as u64),
                20 + 10 * i as u64,
            );
            assert!(
                actions
                    .iter()
                    .any(|a| matches!(a, SessionPixAction::ProgressNotification)),
                "a surplus share must reset the gate's served-without-progress ceiling"
            );
            assert!(
                !actions.iter().any(|a| matches!(a, SessionPixAction::Close(_))),
                "a surplus share must never close the Session"
            );
        }

        // The idle deadline was refreshed by the surplus run, so it has not expired — even though
        // `served_total` has grown well past the watermark of the last *useful* share.
        let actions = sup.handle_deadline(start + Duration::from_secs(12), 70);
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle))),
            "an Entry serving its negotiated surplus is not idle"
        );

        // Payment did not move: the surplus bought the Entry time, not credit.
        let ssa = sup.ssas.iter().find(|s| s.ssa_id == id).expect("cycle must be live");
        assert_eq!(1, ssa.largest_useful_shares, "surplus must not count as useful shares");
        assert_eq!(6, ssa.largest_shares_seen, "every share must count as liveness");
    }

    /// Once the surplus run stops, the idle rule must still fire.
    ///
    /// The liveness split widens what counts as "the Entry is alive"; it must not make the anti-drip
    /// bound unreachable. With no snapshot of any kind and service still being consumed, the Session
    /// closes exactly as it did before.
    #[test]
    fn silence_after_a_surplus_run_still_closes_the_session() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.max_recovery_idle = Duration::from_secs(10);
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let start = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id,
                amount: sufficient_balance(),
            },
            start,
            0,
        );
        sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress_seen(id, 1, 4, 50, 0)),
            start + Duration::from_secs(1),
            10,
        );

        // Nothing further arrives, and service keeps being consumed.
        let actions = sup.handle_deadline(start + Duration::from_secs(12), 500);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::RecoveryIdle))),
            "a genuinely silent Entry must still be caught"
        );
    }

    #[test]
    fn equal_snapshot_is_noop() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
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
                [SessionPixAction::Close(SessionPixCloseReason::UnverifiableShares)]
            ),
            "expected a close, got {actions:?}"
        );
    }

    /// The report closes whatever it says, because the count is not what makes it fatal.
    ///
    /// `observed_total` is a cross-peer aggregate assembled from concurrently processed ack batches,
    /// so it can arrive as any value and out of order. It used to be charged as a delta against a
    /// running maximum and compared to a tolerance; with no tolerance left it is a log field, and the
    /// close must not become conditional on it again — a total of one is a doomed cycle exactly as
    /// much as a total of a thousand.
    #[test]
    fn any_observed_total_closes_on_the_first_report() {
        for observed_total in [1, 2, u64::MAX] {
            let p = pseudonym();
            let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
            let now = Instant::now();
            let id = ssa_id(p, 1);

            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);

            let actions = sup.handle_event(
                &SessionPixEvent::UnverifiableShares {
                    ssa_id: id,
                    observed_total,
                },
                now,
                0,
            );

            assert!(
                matches!(
                    actions.as_slice(),
                    [SessionPixAction::Close(SessionPixCloseReason::UnverifiableShares)]
                ),
                "observed_total {observed_total} must close on the first report, got {actions:?}"
            );
        }
    }

    /// A batch's healthy siblings must not keep the Session open.
    ///
    /// Every other per-cycle fault routes through `close_ssa_and_collect`, which retires the member
    /// and lets `max_failed_cycles` siblings carry on. This one does not, and deliberately: a
    /// polynomial that fails to open its commitment is evidence about the *peer*, and the same peer
    /// holds every other cycle in the batch.
    #[test]
    fn an_unverifiable_share_closes_the_session_even_with_live_siblings() {
        let p = pseudonym();
        let mut cfg = default_cfg();
        cfg.ssas_per_request = 3;
        cfg.max_failed_cycles = 10;
        let (mut sup, actions) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let now = Instant::now();

        let ids = match actions.as_slice() {
            [SessionPixAction::RequestSsa { ssa_ids, .. }] => ssa_ids.clone(),
            other => panic!("expected one RequestSsa, got {other:?}"),
        };
        assert_eq!(3, ids.len());
        for id in &ids {
            sup.handle_event(&SessionPixEvent::SsaRequestSent(*id), now, 0);
        }

        // The fault lands on the *last* member, so two healthy cycles remain in front of it.
        let actions = sup.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id: ids[2],
                observed_total: 1,
            },
            now,
            0,
        );

        assert!(
            matches!(
                actions.as_slice(),
                [SessionPixAction::Close(SessionPixCloseReason::UnverifiableShares)]
            ),
            "a live batch must not absorb an unverifiable-share report, got {actions:?}"
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), now, 0);
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

    /// Config for the share-order tests: a batch of three, generous dimensions so a cycle can carry a
    /// meaningful sample, and a small floor so the tests need few events.
    fn share_order_cfg(sample: u64, fraction: f64) -> SupervisorConfig {
        SupervisorConfig {
            ssas_per_request: 3,
            min_share_order_sample: sample,
            max_off_front_share_fraction: fraction,
            ..default_cfg()
        }
    }

    /// Serving the front cycle, with only a trace of progress behind it, must never trip the ratio.
    ///
    /// The trace stands in for what the mixnet produces on its own: SURBs are permuted outbound and
    /// their acknowledgements again inbound, so a conforming Entry still shows *some* off-front
    /// progress at a cycle boundary. The point of measuring a fraction rather than a count is that a
    /// bounded permutation stays a vanishing share of a growing window however the mixer is tuned.
    #[test]
    fn conforming_service_never_trips_the_share_order_ratio() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(share_order_cfg(1000, 0.25), dims(100, 10), p, Instant::now());
        let now = Instant::now();
        fund_batch(&mut sup, p, 3, now);
        let target = sup.dims.target_useful_shares();

        for useful in [200, 400, 600, 800] {
            let actions = sup.handle_event(
                &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 1), useful, target, 10)),
                now,
                useful,
            );
            assert!(
                matches!(actions.as_slice(), [SessionPixAction::ProgressNotification]),
                "front-cycle service is what is supposed to happen, got {actions:?}"
            );
        }

        // A boundary's worth of reordering: 50 shares against 800, far inside the ceiling.
        let actions = sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 2), 50, target, 1)),
            now,
            850,
        );
        assert!(
            actions.is_empty(),
            "off-front progress must neither convict nor reopen the gate"
        );
        assert!(!sup.closed);
    }

    /// An Entry spreading a batch across all its cycles must be caught once the evidence is in.
    ///
    /// Three cycles served in parallel put two thirds of all progress behind the front — the signature
    /// the ratio exists for, and one that neither the service gate nor `max_recovery_idle` can see,
    /// since the front cycle keeps progressing and keeps refreshing its own idle timer.
    #[test]
    fn spreading_a_batch_across_its_cycles_trips_the_share_order_ratio() {
        const SAMPLE: u64 = 1000;

        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(share_order_cfg(SAMPLE, 0.25), dims(100, 10), p, Instant::now());
        let now = Instant::now();
        fund_batch(&mut sup, p, 3, now);
        let target = sup.dims.target_useful_shares();

        let mut closed_at = None;
        'outer: for round in 1..=5u64 {
            for cycle in 1..=3u32 {
                let actions = sup.handle_event(
                    &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, cycle), round * 100, target, 10)),
                    now,
                    round * 300,
                );
                let sample = sup.front_useful + sup.off_front_useful;
                if let [SessionPixAction::Close(reason)] = actions.as_slice() {
                    assert_eq!(*reason, SessionPixCloseReason::BatchServedOutOfOrder);
                    closed_at = Some(sample);
                    break 'outer;
                }
                assert!(
                    sample < SAMPLE,
                    "at {sample} shares the evidence was in and the 2:1 split should have closed the Session"
                );
            }
        }

        let closed_at = closed_at.expect("spreading across three cycles must close the Session");
        assert!(
            closed_at >= SAMPLE,
            "conviction at {closed_at} shares is below the evidence floor of {SAMPLE}"
        );
    }

    /// The accounting is scoped to one cycle's service: completing the front cycle clears it.
    ///
    /// That bound is what stops an Entry laundering a spree of out-of-order service — the only way to
    /// reach the reset is to finish the front cycle, which is to say to earn the traffic it took.
    #[test]
    fn share_order_accounting_resets_when_the_front_cycle_completes() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(share_order_cfg(1000, 0.25), dims(100, 10), p, Instant::now());
        let now = Instant::now();
        fund_batch(&mut sup, p, 3, now);
        let target = sup.dims.target_useful_shares();

        sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 1), 500, target, 50)),
            now,
            500,
        );
        sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 2), 100, target, 10)),
            now,
            600,
        );
        assert_eq!(sup.share_order_front, Some(SsaIndex::new(1).unwrap()));
        assert_eq!((sup.front_useful, sup.off_front_useful), (500, 100));

        sup.handle_event(&SessionPixEvent::Recovered(ssa_id(p, 1)), now, 600);
        assert_eq!(
            sup.share_order_front,
            Some(SsaIndex::new(2).unwrap()),
            "cycle 2 is the front now"
        );
        assert_eq!(
            (sup.front_useful, sup.off_front_useful),
            (0, 0),
            "the window is one cycle's service, so it starts empty on the new front"
        );
    }

    /// Below the evidence floor nothing is judged, however lopsided the split.
    ///
    /// This is what keeps a cycle made unrecoverable by loss from convicting the Entry: it stops
    /// progressing while later cycles continue, so the fraction goes to 1.0 — and the floor outlasts
    /// the service-gated `max_recovery_idle` window that retires it, after which the front moves and
    /// the accounting resets. The ceiling here is 0.0, the strictest possible, so only the floor can be
    /// what holds the verdict back.
    #[test]
    fn no_verdict_is_reached_below_the_evidence_floor() {
        const SAMPLE: u64 = 1000;

        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(share_order_cfg(SAMPLE, 0.0), dims(100, 10), p, Instant::now());
        let now = Instant::now();
        fund_batch(&mut sup, p, 3, now);
        let target = sup.dims.target_useful_shares();

        // Every share lands behind the front, and still nothing is decided while the sample is thin.
        for useful in [300, 600, 900] {
            let actions = sup.handle_event(
                &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 3), useful, target, 10)),
                now,
                useful,
            );
            assert!(actions.is_empty(), "{useful} shares is below the floor and off-front");
        }

        let actions = sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 3), SAMPLE, target, 10)),
            now,
            SAMPLE,
        );
        assert!(
            matches!(
                actions.as_slice(),
                [SessionPixAction::Close(SessionPixCloseReason::BatchServedOutOfOrder)]
            ),
            "at the floor the verdict must be reached, got {actions:?}"
        );
    }

    /// A stray reordered share must not put a *queued* cycle on the idle clock.
    ///
    /// The sibling test below fixes the arming path: clocks start when a cycle reaches the front. But
    /// refreshing is not arming, and `on_recovery_progress` used to set `recovery_idle_deadline` for
    /// any cycle merely in `Recovering`. One share crossing a cycle boundary out of order — the exact
    /// thing [`SupervisorConfig::min_share_order_sample`] exists to tolerate — therefore *started* the
    /// idle clock on a cycle the Entry cannot serve, since emission is clamped to one cycle. A
    /// `max_recovery_idle` later the gate saw session-wide service climbing on the front cycle's
    /// behalf and retired the queued one, charging it for a queue wait it had no way to end.
    ///
    /// The two clocks arm together or not at all, so the hard deadline is the witness for "has been at
    /// the front", and a queued cycle must come out of this with neither.
    #[test]
    fn a_stray_share_on_a_queued_cycle_must_not_start_its_idle_clock() {
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

        // One share for queued cycle 2 arrives while cycle 1 holds the front.
        sup.handle_event(
            &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 2), 1, target, 0)),
            start + Duration::from_secs(1),
            1_000,
        );

        let queued = sup.ssas.iter().find(|s| s.ssa_id == ssa_id(p, 2)).expect("cycle 2");
        assert_eq!(
            (queued.recovery_hard_deadline, queued.recovery_idle_deadline),
            (None, None),
            "an off-front share must leave a queued cycle off both clocks, not just the hard one"
        );

        // Cycle 1 is served normally, so session-wide service climbs past cycle 2's baseline — which is
        // what would convict it if the idle clock were running.
        for step in 1..=3u64 {
            sup.handle_event(
                &SessionPixEvent::RecoveryProgress(make_progress(ssa_id(p, 1), step, target, 0)),
                start + Duration::from_secs(1 + step * 3),
                1_000 + step * 1_000,
            );
        }

        let actions = sup.handle_deadline(start + Duration::from_secs(12), 5_000);
        assert!(
            actions.is_empty(),
            "no cycle may be retired or closed while the front is being served, got {actions:?}"
        );
        assert_eq!(sup.ssas.len(), BATCH as usize, "the whole batch must still be live");
        assert_eq!(sup.failed_cycles, 0, "nothing may be charged as a failed cycle");
    }

    /// A cycle queued behind the front of its batch must not be charged for the wait.
    ///
    /// The Entry serves a batch strictly in index order, so at deployed dimensions cycle 2 of a batch
    /// sits idle for the ~61 min it takes cycle 1's shares to be emitted. Starting its recovery clocks
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
    /// becomes three, then nine, each SSA a separate on-chain deposit demanded of the Entry. At
    /// `ssas_per_request == 1`, "one requesting cycle" and "one requesting batch" coincide, which is
    /// why every unbatched test misses the distinction.
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
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
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
        assert_eq!(
            sup.live_cycle_count(),
            sup.reserved_cycle_slots(),
            "two full batches sit exactly on the admission reservation"
        );
    }

    #[test]
    fn a_third_live_batch_is_deferred_even_when_share_order_policy_allows_it() {
        const BATCH: usize = 2;

        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH,
            max_off_front_share_fraction: 1.0,
            ..default_cfg()
        };
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);

        fund_batch(&mut sup, p, BATCH as u32, now);
        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(ssa_id(p, 2)), now, 0);
        let second_batch = match actions.as_slice() {
            [SessionPixAction::RequestSsa { ssa_ids, .. }] => ssa_ids.clone(),
            other => panic!("expected the second batch, got {other:?}"),
        };
        for id in second_batch {
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
            sup.handle_event(
                &SessionPixEvent::DepositConfirmed {
                    ssa_id: id,
                    amount: sufficient_balance(),
                },
                now,
                0,
            );
        }

        let actions = sup.handle_event(&SessionPixEvent::AlmostRecovered(ssa_id(p, 4)), now, 0);
        assert!(
            actions
                .iter()
                .all(|action| !matches!(action, SessionPixAction::RequestSsa { .. })),
            "two live batches already consume the reservation, got {actions:?}"
        );
        assert_eq!(sup.ssas.len(), 2 * BATCH, "a third batch must not be allocated");
    }

    #[test]
    fn a_deferred_batch_retries_only_after_the_older_generation_is_released() {
        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: 2,
            max_failed_cycles: 2,
            max_off_front_share_fraction: 1.0,
            ..default_cfg()
        };
        let now = Instant::now();
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, now);

        // Leave cycle 1 unfunded and fund cycle 2, whose early signal earns batch 2.
        commit_unfunded(&mut sup, p, 1..=1, now);
        let second = ssa_id(p, 2);
        sup.handle_event(&SessionPixEvent::SsaRequestSent(second), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(second), now, 0);
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: second,
                amount: sufficient_balance(),
            },
            now,
            0,
        );
        assert!(
            sup.handle_event(&SessionPixEvent::AlmostRecovered(second), now, 0)
                .iter()
                .any(|action| matches!(action, SessionPixAction::RequestSsa { .. }))
        );

        for idx in 3..=4 {
            let id = ssa_id(p, idx);
            sup.handle_event(&SessionPixEvent::SsaRequestSent(id), now, 0);
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
            sup.handle_event(
                &SessionPixEvent::DepositConfirmed {
                    ssa_id: id,
                    amount: sufficient_balance(),
                },
                now,
                0,
            );
        }
        assert!(
            sup.handle_event(&SessionPixEvent::AlmostRecovered(ssa_id(p, 4)), now, 0)
                .iter()
                .all(|action| !matches!(action, SessionPixAction::RequestSsa { .. }))
        );

        // Releasing only one member of the old generation is insufficient.
        let actions = sup.handle_event(&SessionPixEvent::DepositObserverClosed(ssa_id(p, 1)), now, 0);
        assert!(
            actions
                .iter()
                .all(|action| !matches!(action, SessionPixAction::RequestSsa { .. }))
        );

        // The remaining old member times out. Its RetireSsa must precede the now-admissible request,
        // otherwise the action driver would allocate the replacement before dropping the old guard.
        let actions = sup.handle_deadline(now + default_cfg().max_recovery_idle, 1);
        let retire_position = actions
            .iter()
            .position(|action| matches!(action, SessionPixAction::RetireSsa(id) if *id == second));
        let request_position = actions.iter().position(|action| {
            matches!(action, SessionPixAction::RequestSsa { ssa_ids, .. }
                if ssa_ids.iter().map(|id| id.ssa_index().get()).eq(5..=6))
        });
        assert!(
            matches!((retire_position, request_position), (Some(retire), Some(request)) if retire < request),
            "the old generation must retire before batch 3 is requested, got {actions:?}"
        );
        assert_eq!(sup.live_batch_count(), 2);
        assert_eq!(sup.live_cycle_count(), 4);
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
            sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(last), now, 0);

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

    /// A second lost cycle closes a batched Session, however many funded siblings are still standing.
    ///
    /// Retiring a member and carrying on is what keeps one bad cycle from costing a whole Session,
    /// but it leaves no trace on the survivors — so without a count that outlives the retirement an
    /// Entry can lose one cycle per batch forever, paying for a fraction of what it is served. The
    /// counter is cumulative for exactly that reason: here the two losses are in *different* batches,
    /// and nothing in a per-batch view would connect them.
    ///
    /// The close carries the *first* failure's reason, so the report names the cause rather than the
    /// last symptom.
    #[test]
    fn the_second_lost_cycle_closes_a_batched_session() {
        const BATCH: usize = 4;

        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH,
            max_failed_cycles: 1,
            ..default_cfg()
        };
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let now = Instant::now();

        // Cycles 1 and 2 are funded and recovering; 3 and 4 are stuck awaiting their deposits, which
        // is the state an unpaid cycle is lost from.
        fund_batch(&mut sup, p, 2, now);
        commit_unfunded(&mut sup, p, 3..=BATCH as u32, now);

        // First loss: survivable, and the batch carries on.
        let actions = sup.handle_event(&SessionPixEvent::DepositObserverClosed(ssa_id(p, 3)), now, 0);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::RetireSsa(id) if *id == ssa_id(p, 3))),
            "the first lost cycle must be retired rather than close the Session, got {actions:?}"
        );
        assert!(!sup.closed, "one loss is inside the limit");

        // Second loss, with two funded siblings still recovering: over the limit, so the Session
        // goes anyway — which is the point, since those siblings are what would otherwise keep an
        // Entry losing one cycle per batch alive indefinitely.
        let actions = sup.handle_event(&SessionPixEvent::DepositObserverClosed(ssa_id(p, 4)), now, 0);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::DepositObserverClosed))),
            "the second lost cycle must close the Session, got {actions:?}"
        );
        assert!(sup.closed);
        assert_eq!(2, sup.failed_cycles);
    }

    /// Raising the limit buys exactly the extra losses it names, and no more.
    ///
    /// The counter has to survive the retirement of the cycle that incremented it, so this drives
    /// three separate losses through a batch large enough that a survivor always remains — a
    /// per-batch or per-cycle count would never reach the limit.
    #[test]
    fn a_raised_failure_limit_tolerates_exactly_that_many_losses() {
        const BATCH: usize = 6;

        let p = pseudonym();
        let cfg = SupervisorConfig {
            ssas_per_request: BATCH,
            max_failed_cycles: 3,
            ..default_cfg()
        };
        let (mut sup, _) = SessionPixSupervisor::new(cfg, dims(10, 5), p, Instant::now());
        let now = Instant::now();

        // One funded cycle, so the Session always has a survivor and only the limit can close it.
        fund_batch(&mut sup, p, 1, now);
        commit_unfunded(&mut sup, p, 2..=BATCH as u32, now);

        for idx in 2..=4 {
            sup.handle_event(&SessionPixEvent::DepositObserverClosed(ssa_id(p, idx)), now, 0);
            assert!(!sup.closed, "loss {} of 3 is inside the limit", idx - 1);
        }

        sup.handle_event(&SessionPixEvent::DepositObserverClosed(ssa_id(p, 5)), now, 0);
        assert!(sup.closed, "the fourth loss is over a limit of three");
    }

    #[test]
    fn almost_recovered_while_awaiting_deposit_defers_request() {
        let p = pseudonym();
        let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(10, 5), p, Instant::now());
        let now = Instant::now();
        let id1 = ssa_id(p, 1);

        sup.handle_event(&SessionPixEvent::SsaRequestSent(id1), now, 0);
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), now, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), now, 0);
        sup.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: id1,
                amount: sufficient_balance(),
            },
            now,
            0,
        );

        let actions = sup.handle_event(&SessionPixEvent::Recovered(id1), now, 0);
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], SessionPixAction::WithholdService));
        match &actions[1] {
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), now, 0);
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
        assert!(matches!(actions.as_slice(), [SessionPixAction::WithholdService]));
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
            SessionPixEvent::CommitmentVerified(ssa_id(p, 2)),
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), now, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id2), later, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), now, 0);
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
                params: dims(10, 5),
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id1), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), now, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);

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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
        sup.handle_event(&SessionPixEvent::CommitmentVerified(id), start, 0);
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
