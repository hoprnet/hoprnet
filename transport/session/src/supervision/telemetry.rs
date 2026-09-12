//! Bounded node-level accounting for the PIX Exit.
//!
//! Everything here is compiled unconditionally, including without the `telemetry` feature. Only the
//! instruments the numbers are eventually written to live behind the feature, in
//! `crate::telemetry::pix`. The split is deliberate: the delta arithmetic below is the part that
//! can be wrong in a way an operator would never notice — a gauge that fails to come back down, a
//! cycle counted twice — and a default `cargo nextest run --lib` must exercise it.
//!
//! # Why aggregates rather than more labels
//!
//! The two per-Session PIX gauges (`hopr_session_pix_gate_mode`, `hopr_session_pix_recovery_progress`)
//! answer questions about one Session. An operator's questions are about the node: how much
//! predeposit service is exposed right now, whether egress is blocked on deposits or on share lag,
//! whether admission is refusing peers because the node is full or because their parameters are
//! wrong. Answering those by adding labels would make the view worse rather than better — every
//! `session_id`-labelled instrument stops reporting new Sessions after ~2000 of them (#8305), and
//! nothing retires the series.
//!
//! So the aggregates carry no identifier at all. Every label value in this module comes from a
//! closed enum, which is what makes that a property of the type system rather than a promise.
//!
//! # The census, and why it is not inc/dec
//!
//! [`SessionPixSupervisor`](super::supervisor::SessionPixSupervisor) recomputes
//! [`PixSessionSnapshot`] from its own live state on every turn, and
//! [`PixSessionTelemetry::publish`] applies the *difference* against what it last published. No
//! transition increments a live-set gauge directly.
//!
//! That is what makes "gauges return to zero on every terminal path" structural instead of a matter
//! of auditing call sites. A cycle that is retired, times out, is rolled back at admission, or
//! disappears because its Session was dropped mid-turn is simply absent from the next census, and
//! the delta that follows is the decrement. There is no path that can forget one, because there is
//! no path that performs one.
//!
//! Cumulative *counters* are the opposite case — they must not be recomputed, because their whole
//! value is that they survive the state they describe. Those are latched at the transition that
//! changes the source of truth ([`PixTurnEvents`]) and drained by the same publish.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use super::gate::{GateBlockReason, ServiceGate};

// ---------------------------------------------------------------------------
// Label enums
// ---------------------------------------------------------------------------

/// Front-gate mode of a live PIX Session.
///
/// Every live PIX Session is in exactly one of these, so the two buckets of
/// `hopr_pix_sessions_active` sum to the number of supervised Sessions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PixGateMode {
    /// The front cycle is unfunded and spending its bounded predeposit allowance.
    ///
    /// The default, because a Session starts here and stays until its first deposit confirms.
    #[default]
    Predeposit,
    /// The front cycle is funded, or a paid recovered predecessor still holds the front.
    Funded,
}

/// Supervisor phase of a retained SSA cycle.
///
/// Recovered tombstones and cycles in `Closing` are deliberately absent. They are retained state
/// rather than live state — a tombstone exists only so late acknowledgements have something to land
/// on — so counting them would leave `hopr_pix_cycles_active` unable to drain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PixCyclePhase {
    /// Requested; waiting for a complete verifiable commitment.
    AwaitingCommitment,
    /// Commitment verified; waiting for the deposit pool's verdict.
    AwaitingDeposit,
    /// Funded and recovering shares.
    Recovering,
    /// A recovered predecessor still authorized to hold the paid service front while its negotiated
    /// FIFO tail drains.
    PaidTail,
}

/// A cycle lifecycle transition worth a cumulative count.
///
/// There is deliberately no `retired` event. Every `RetireSsa` follows either a tombstone expiring
/// or [`close_ssa_and_collect`](super::supervisor::SessionPixSupervisor), both of which are already
/// counted here, so a third event would read the same transition twice. What is left is exact:
/// `requested = recovered + failed + (still live)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PixCycleEvent {
    /// An index was allocated and put into an `SsaRequest`.
    Requested,
    /// Its commitment verified.
    Committed,
    /// Its deposit was confirmed by the pool.
    Funded,
    /// It reconstructed fully.
    Recovered,
    /// It did not recover, whatever the reason.
    ///
    /// Reported for a cycle retired by the supervisor *and* for one still live when its Session
    /// ended, which is not the same as "the supervisor named a close reason for it":
    /// `on_unverifiable_shares` closes a Session outright without routing its siblings through
    /// retirement, and `hopr_session_pix_closures_total{reason}` is where the reasons are. The label
    /// means "did not recover".
    Failed,
}

impl PixCycleEvent {
    /// Whether the census accounts for this transition too, so that emitting it out of step with
    /// the census would read the same cycle twice.
    ///
    /// These three are the terms of `requested = recovered + failed + live`, whose `live` term is
    /// the census. The other two are edges the census has no opinion about: a cycle that commits or
    /// funds moves between census buckets without entering or leaving it, so counting one twice
    /// would be wrong but counting it out of step with the census is not. That is what
    /// [`EventScope::EdgesOnly`] rests on.
    fn is_census_coupled(self) -> bool {
        matches!(self, Self::Requested | Self::Recovered | Self::Failed)
    }
}

/// Why PIX egress stopped, as `hopr_pix_gate_blocks_total` labels it.
///
/// The gate's own two reasons plus the one it does not have a verdict for: a poisoned gate refuses
/// through `Err(GateClosed)` rather than through a `Blocked`, because it is not a stall the Session
/// will come out of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PixGateBlock {
    /// The unfunded front has spent its whole predeposit allowance.
    PredepositExhausted,
    /// Funded service has run its full ceiling ahead of the shares coming back.
    ShareLag,
    /// The gate was poisoned: the Session is being torn down and will serve nothing further.
    Closed,
}

impl From<GateBlockReason> for PixGateBlock {
    fn from(reason: GateBlockReason) -> Self {
        match reason {
            GateBlockReason::PredepositExhausted => Self::PredepositExhausted,
            GateBlockReason::ShareLag => Self::ShareLag,
        }
    }
}

/// One episode of PIX egress being blocked, measured from first refusal until it ends.
///
/// # Why an episode rather than a refusal
///
/// A blocked gate is polled again by whatever is trying to send, so counting refusals would measure
/// how hard the caller retries rather than how long the Exit was stalled — the issue's "a tight
/// retry loop must not increment blocked on every failed poll". The entry into the blocked state is
/// counted once, here, and how long it lasted is measured separately.
///
/// # Why `Drop`
///
/// The episode ends three ways, and only one of them is a return value: the gate resumes and the
/// parked writer proceeds, the gate is poisoned and the writer fails, or the whole future is
/// dropped because the Session was torn down underneath it. `Drop` is the one mechanism that covers
/// all three, which is what makes "block timers close exactly once on every terminal path" hold
/// without any of those paths having to remember.
pub struct PixGateBlockEpisode {
    reason: GateBlockReason,
    began: Instant,
}

impl PixGateBlockEpisode {
    /// Counts one episode beginning, and starts its clock.
    pub fn begin(reason: GateBlockReason) -> Self {
        emit_gate_block(reason.into());
        Self {
            reason,
            began: Instant::now(),
        }
    }
}

impl Drop for PixGateBlockEpisode {
    fn drop(&mut self) {
        emit_gate_block_duration(self.reason, self.began.elapsed().as_secs_f64());
    }
}

/// Counts one refusal by a poisoned gate.
///
/// No episode and no duration: a poisoned gate never resumes, so the only thing a clock on it would
/// measure is how long the Session's teardown took to reach its last writer.
pub fn record_gate_closed() {
    emit_gate_block(PixGateBlock::Closed);
}

/// Why an incoming PIX Session was refused before it was established.
///
/// Bounded by construction, and finer than the wire reason it maps to. `NoSessionSlot` and
/// `LiveCycleCapacity` are both `StartErrorReason::NoSlotsAvailable` on the wire, so
/// `hopr_session_sent_error_count{kind}` cannot tell "this Exit is healthy but its reconstructor
/// budget is full" from "this Exit is at its Session limit" — which are different problems with
/// different fixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PixAdmissionRejection {
    /// The peer offered PIX to a node with no `PixToolbox` installed.
    NoPixSupport,
    /// The offer could not be decoded, or fell outside the accepted quota range.
    UnacceptableParams,
    /// The session server refused this target, or refused it on the offered terms.
    TargetPolicy,
    /// The node's `max_live_cycle_bytes` budget could not cover this Session's reconstructor state.
    LiveCycleCapacity,
    /// The node is already managing `maximum_sessions`, or this pseudonym already holds a slot.
    NoSessionSlot,
    /// This node could not decide right now — the session server did not answer in time, or its
    /// request channel was full.
    Busy,
}

impl PixAdmissionRejection {
    /// Counts this refusal, if the peer was actually asking for a PIX Session.
    ///
    /// The capability check is here rather than at the six call sites because it is the part that
    /// can silently be wrong: every one of those sites also refuses ordinary non-PIX Sessions, and a
    /// `hopr_pix_admission_rejections_total` that counted those would report a PIX problem this node
    /// does not have. `hopr_session_sent_error_count{kind}` already counts refusals of every kind —
    /// this one exists precisely to be narrower.
    ///
    /// Always compiled, so the gate is exercised by the default test configuration even though the
    /// emission behind it is not. Returns whether the refusal was counted, which is how a test
    /// without that feature can still assert the gate rather than merely run it.
    pub fn record_if_pix(self, capabilities: crate::Capabilities) -> bool {
        if !capabilities.contains(crate::Capability::UsePIX) {
            return false;
        }
        #[cfg(feature = "telemetry")]
        crate::telemetry::pix::record_admission_rejection(self);
        true
    }
}

// ---------------------------------------------------------------------------
// Snapshot and event latch
// ---------------------------------------------------------------------------

/// One PIX Session's contribution to the node-level live-set gauges.
///
/// Recomputed from the supervisor's own state on every turn; see the module documentation for why
/// it is a census rather than a set of increments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PixSessionSnapshot {
    /// Which bucket of `hopr_pix_sessions_active` this Session is in.
    pub gate_mode: PixGateMode,
    /// Cycles requested, whose commitment has not verified yet.
    pub awaiting_commitment: u32,
    /// Cycles with a verified commitment, waiting for a deposit.
    pub awaiting_deposit: u32,
    /// Funded cycles still recovering.
    pub recovering: u32,
    /// Whether a recovered predecessor is still holding the paid front (0 or 1).
    pub paid_tail: u32,
    /// Packets this Session has been served against an unfunded front's predeposit allowance and
    /// which no deposit has yet converted into paid service.
    pub predeposit_exposure_packets: u64,
}

impl PixSessionSnapshot {
    /// Cycles this Session currently holds in a live (non-terminal, non-tombstone) phase.
    pub fn live_cycles(&self) -> u32 {
        self.awaiting_commitment
            .saturating_add(self.awaiting_deposit)
            .saturating_add(self.recovering)
    }

    /// The per-phase counts, in label order.
    fn by_phase(&self) -> [(PixCyclePhase, u32); 4] {
        [
            (PixCyclePhase::AwaitingCommitment, self.awaiting_commitment),
            (PixCyclePhase::AwaitingDeposit, self.awaiting_deposit),
            (PixCyclePhase::Recovering, self.recovering),
            (PixCyclePhase::PaidTail, self.paid_tail),
        ]
    }
}

/// Cycle lifecycle transitions latched by the supervisor since the last publish.
///
/// The supervisor is a pure state machine and cannot emit a metric, so it latches the edge and the
/// worker drains it — the same arrangement as
/// [`take_fill_stall`](super::supervisor::SessionPixSupervisor::take_fill_stall).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PixTurnEvents {
    /// Indices allocated into an `SsaRequest`.
    pub requested: u32,
    /// Commitments verified.
    pub committed: u32,
    /// Deposits confirmed.
    pub funded: u32,
    /// Cycles fully reconstructed.
    pub recovered: u32,
    /// Cycles retired without recovering.
    pub failed: u32,
}

impl PixTurnEvents {
    /// True when nothing happened, so the publish can skip the counters entirely.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The non-zero counts, in label order.
    fn counts(&self) -> [(PixCycleEvent, u32); 5] {
        [
            (PixCycleEvent::Requested, self.requested),
            (PixCycleEvent::Committed, self.committed),
            (PixCycleEvent::Funded, self.funded),
            (PixCycleEvent::Recovered, self.recovered),
            (PixCycleEvent::Failed, self.failed),
        ]
    }
}

/// How much of a latched batch a publish may emit.
///
/// The worker drains [`PixTurnEvents`] from the supervisor and *then* calls
/// [`publish`](PixSessionTelemetry::publish), so a `release` from `close_session` — which runs on
/// the manager's task, not the worker's — can land between the two. The batch is in the worker's
/// hand at that point and the supervisor's latch is already empty, so whatever the publish declines
/// to emit is gone for good. This decides what "declines to emit" means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EventScope {
    /// All of it. The census is live and moves with the batch, so the two agree by construction.
    Everything,
    /// Only the counts nothing reads back.
    ///
    /// `release` has already charged this Session's outstanding cycles from its last census, so
    /// re-emitting a [census-coupled](PixCycleEvent::is_census_coupled) count here would account
    /// the same cycle twice. What is left is an edge in no invariant and no gauge, and dropping it
    /// would understate the phase-transition rate for no reason at all.
    EdgesOnly,
}

// ---------------------------------------------------------------------------
// PixSessionTelemetry
// ---------------------------------------------------------------------------

/// One PIX Session's share of the node-level aggregates, and the bookkeeping that keeps it exact.
///
/// Created with its supervisor worker and carried on
/// [`SessionPixSupervisorHandle`](super::worker::SessionPixSupervisorHandle), so it reaches both the
/// worker (which publishes) and `close_session` (which releases) without a new `SessionSlot` field.
///
/// # Why `release` is explicit rather than only `Drop`
///
/// The same reason `CycleBudgetReservation` gives: a `SessionSlot` lives in a `moka` cache that
/// drops its own clone during a later maintenance pass, and the supervisor worker is spawned
/// detached and exits only when the last `cmd_tx` goes with it. A purely refcount-driven release
/// would therefore zero these gauges at an unpredictable time, and until then the node would report
/// Sessions and cycles that no longer exist. The flag is what lets both the explicit call and `Drop`
/// run without decrementing twice.
pub struct PixSessionTelemetry {
    /// The gate this Session's egress passes through, read for its served split.
    ///
    /// Held rather than passed in, because the two callers do not both have it at the moment they
    /// need it: `release` runs from `close_session`, which does have the gate, but `Drop` runs from
    /// wherever the last `Arc` happens to go.
    gate: Arc<ServiceGate>,
    state: parking_lot::Mutex<Published>,
    released: AtomicBool,
}

/// What this handle has already accounted for.
#[derive(Clone, Copy, Debug, Default)]
struct Published {
    /// The last census, or `None` before the first publish and after a release.
    census: Option<PixSessionSnapshot>,
    /// The gate's `(predeposit, funded)` split as of the last flush, so the next one emits the
    /// difference. Cumulative counters, unlike the census, are never returned on release.
    egress: (u64, u64),
}

impl PixSessionTelemetry {
    /// A handle that has published nothing yet, reading egress from `gate`.
    pub fn new(gate: Arc<ServiceGate>) -> Self {
        Self {
            gate,
            state: parking_lot::Mutex::new(Published::default()),
            released: AtomicBool::new(false),
        }
    }

    /// Publishes the difference between `snapshot` and what was last published, and drains `events`.
    ///
    /// The census half is a no-op once [`release`](Self::release) has run, so a worker still
    /// draining its command channel after its Session was torn down cannot re-inflate a gauge that
    /// has already been returned to zero. The events are a different matter: the worker took them
    /// out of the supervisor's latch before calling this, so they exist nowhere else, and a
    /// released handle emits what it safely can of them rather than dropping the batch on the floor.
    /// [`EventScope`] draws that line.
    pub fn publish(&self, snapshot: PixSessionSnapshot, events: PixTurnEvents) {
        if self.released.load(Ordering::Acquire) {
            record_events(events, EventScope::EdgesOnly);
            return;
        }

        let mut state = self.state.lock();
        // Re-checked under the lock: `release` takes it to apply its own decrements, so without this
        // a publish that passed the check above could otherwise be applied after them.
        if self.released.load(Ordering::Acquire) {
            drop(state);
            record_events(events, EventScope::EdgesOnly);
            return;
        }

        apply_delta(state.census.as_ref(), Some(&snapshot));
        state.census = Some(snapshot);
        let egress = self.flush_egress(&mut state);
        drop(state);

        record_egress(egress);
        record_events(events, EventScope::Everything);
    }

    /// Returns every gauge this Session holds to zero, and flushes its last egress. Idempotent.
    ///
    /// Cycles still live in the last published census are counted as
    /// [`Failed`](PixCycleEvent::Failed), which is what keeps `requested = recovered + failed` exact
    /// for a Session that ends with cycles in flight. `on_unverifiable_shares` does exactly that on
    /// purpose — it closes the Session outright rather than retiring each cycle — and so does any
    /// close that arrives from outside the supervisor.
    ///
    /// The egress flush is why `close_session` poisons the gate before calling this: a poisoned gate
    /// admits nothing further, so the split read here is final rather than a moving target.
    ///
    /// If a publish is racing this, one census applies and the other is suppressed, and either
    /// ordering counts each live cycle exactly once: a publish that wins first shrinks the census
    /// this then charges, and a publish that loses leaves the older census for this to charge
    /// instead. What the losing publish carried is not thrown away with it — it emits the part of
    /// its batch this does not account for, which [`EventScope`] defines.
    pub fn release(&self) {
        if self.released.swap(true, Ordering::AcqRel) {
            return;
        }

        let mut state = self.state.lock();
        let last = state.census.take();
        apply_delta(last.as_ref(), None);
        let egress = self.flush_egress(&mut state);
        drop(state);

        record_egress(egress);

        let live = last.map(|census| census.live_cycles()).unwrap_or(0);
        if live > 0 {
            record_events(
                PixTurnEvents {
                    failed: live,
                    ..Default::default()
                },
                EventScope::Everything,
            );
        }
    }

    /// Packets admitted since the last flush, and advances the watermark.
    ///
    /// Returned rather than emitted so the caller can release the lock first: these are cumulative
    /// counters, and holding this Session's lock across a global instrument would serialize every
    /// other Session's publish behind it for no benefit.
    fn flush_egress(&self, state: &mut Published) -> (u64, u64) {
        let (predeposit, funded) = self.gate.served_split();
        // `served_split`'s two loads are not atomic with respect to each other, so a predeposit
        // permit landing between them is seen by one and not the other. Its publication order rules
        // out the direction that would misattribute the packet; what remains is a `served_predeposit`
        // from after the permit against a `served` from before it, which makes the *derived* funded
        // figure come out one lower than the truth.
        //
        // Saturating the subtraction keeps that from wrapping a counter to 2^64. Clamping the stored
        // watermark is what keeps it from double-counting: without the `max`, the regressed value
        // would be stored, and every funded packet between it and the real figure would be reported
        // a second time when the next flush crossed that ground again. The reading corrects itself
        // on the following flush either way — this only ensures the correction is not paid for
        // twice.
        let delta = (
            predeposit.saturating_sub(state.egress.0),
            funded.saturating_sub(state.egress.1),
        );
        state.egress = (predeposit.max(state.egress.0), funded.max(state.egress.1));
        delta
    }

    /// The snapshot this handle last published, for tests that assert its bookkeeping without
    /// reading the process-wide metric registry.
    #[cfg(test)]
    pub fn published(&self) -> Option<PixSessionSnapshot> {
        self.state.lock().census
    }
}

impl Drop for PixSessionTelemetry {
    fn drop(&mut self) {
        self.release();
    }
}

// ---------------------------------------------------------------------------
// Emission
// ---------------------------------------------------------------------------

/// Moves the live-set gauges from `from` to `to`, either of which may be "no Session at all".
///
/// Signed on purpose. A gauge is only ever nudged by the difference between two censuses, so an
/// arithmetic slip cannot make it drift: the next publish computes its delta from the same stored
/// snapshot and lands on the correct absolute value regardless.
fn apply_delta(from: Option<&PixSessionSnapshot>, to: Option<&PixSessionSnapshot>) {
    for mode in [PixGateMode::Predeposit, PixGateMode::Funded] {
        let delta = sessions_in(to, mode) - sessions_in(from, mode);
        if delta != 0 {
            emit_sessions_active(mode, delta);
        }
    }

    let before = from.copied().unwrap_or_default().by_phase();
    let after = to.copied().unwrap_or_default().by_phase();
    for ((phase, after), (_, before)) in after.into_iter().zip(before) {
        let delta = i64::from(after) - i64::from(before);
        if delta != 0 {
            emit_cycles_active(phase, delta);
        }
    }

    // Widened before subtracting, because both sides are `u64` and either may be the larger: a
    // Session whose front rotates goes from a spent allowance to a restored one in a single turn.
    let exposure = i128::from(to.map(|s| s.predeposit_exposure_packets).unwrap_or(0))
        - i128::from(from.map(|s| s.predeposit_exposure_packets).unwrap_or(0));
    if exposure != 0 {
        emit_predeposit_exposure(exposure);
    }
}

/// 1 if `snapshot` is a live Session in `mode`, else 0.
fn sessions_in(snapshot: Option<&PixSessionSnapshot>, mode: PixGateMode) -> i64 {
    i64::from(snapshot.is_some_and(|s| s.gate_mode == mode))
}

fn record_events(events: PixTurnEvents, scope: EventScope) {
    if events.is_empty() {
        return;
    }
    for (event, count) in events.counts() {
        if count > 0 && (scope == EventScope::Everything || !event.is_census_coupled()) {
            emit_cycles_total(event, u64::from(count));
        }
    }
}

/// Counts packets admitted since the last flush, by what paid for them.
fn record_egress((predeposit, funded): (u64, u64)) {
    if predeposit > 0 {
        emit_egress_packets(PixGateMode::Predeposit, predeposit);
    }
    if funded > 0 {
        emit_egress_packets(PixGateMode::Funded, funded);
    }
}

// The four shims below are the whole of this module's dependence on the `telemetry` feature. Keeping
// the `cfg` here rather than around the arithmetic above means that arithmetic is compiled, linted
// and tested in both configurations — with the feature off and outside a test build, each of these
// is an empty function the optimizer removes along with the loop that calls it.

fn emit_sessions_active(mode: PixGateMode, delta: i64) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::pix::add_sessions_active(mode, delta);
    #[cfg(test)]
    probe::add(&format!("sessions_active/{mode}"), delta);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = (mode, delta);
}

fn emit_cycles_active(phase: PixCyclePhase, delta: i64) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::pix::add_cycles_active(phase, delta);
    #[cfg(test)]
    probe::add(&format!("cycles_active/{phase}"), delta);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = (phase, delta);
}

fn emit_predeposit_exposure(delta: i128) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::pix::add_predeposit_exposure(delta);
    #[cfg(test)]
    probe::add("predeposit_exposure", delta as i64);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = delta;
}

fn emit_cycles_total(event: PixCycleEvent, count: u64) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::pix::add_cycles_total(event, count);
    #[cfg(test)]
    probe::add(&format!("cycles_total/{event}"), count as i64);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = (event, count);
}

fn emit_egress_packets(mode: PixGateMode, count: u64) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::pix::add_egress_packets(mode, count);
    #[cfg(test)]
    probe::add(&format!("egress_packets/{mode}"), count as i64);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = (mode, count);
}

fn emit_gate_block(reason: PixGateBlock) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::pix::record_gate_block(reason);
    #[cfg(test)]
    probe::add(&format!("gate_blocks/{reason}"), 1);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = reason;
}

fn emit_gate_block_duration(reason: GateBlockReason, seconds: f64) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::pix::record_gate_block_duration(reason, seconds);
    // The one argument the probe does not consume: a test can assert that an episode was closed,
    // but not how long a wall clock said it took.
    #[cfg(not(feature = "telemetry"))]
    let _ = seconds;

    #[cfg(test)]
    probe::add(&format!("gate_block_observations/{reason}"), 1);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = reason;
}

/// An in-process mirror of the aggregates, so tests can assert what was emitted.
///
/// Exists because the instruments themselves are behind the `telemetry` feature, which no CI job
/// enables — a test written against them would never run. What it makes testable is the part that
/// most needs it: whether a gauge comes back to zero. That is invisible in production until an
/// operator compares the metric against a reality they have no access to.
///
/// Not a general mocking layer. It mirrors deltas exactly as the shims above pass them, so a test
/// asserting zero here is asserting that the same sequence of calls reached the real instrument.
#[cfg(test)]
pub(crate) mod probe {
    use std::{cell::RefCell, collections::BTreeMap};

    thread_local! {
        static STATE: RefCell<BTreeMap<String, i64>> = const { RefCell::new(BTreeMap::new()) };
    }

    /// Clears the probe for this thread.
    ///
    /// Thread-local rather than global, and that is what makes these tests independent rather than
    /// merely serialized. Every `publish` and `release` anywhere in the crate reaches `add` through
    /// the emission shims, so a shared map would let a test that never reads the probe still
    /// perturb one that asserts an exact value — and `cargo test` runs tests concurrently on
    /// threads, so a lock would have to be taken by every test rather than only the asserting ones.
    /// A thread-local needs no discipline from either.
    ///
    /// The consequence to know about: a test whose emissions happen on a *different* thread — a
    /// multi-thread tokio runtime, or a `std::thread::spawn` — will not see them here. Every probe
    /// test drives the code directly or through a current-thread runtime, which is the same thread
    /// throughout. A future test that needs otherwise should assert through `gather_all_metrics`
    /// under `--features telemetry` instead.
    pub(crate) fn reset() {
        STATE.with_borrow_mut(|state| state.clear());
    }

    pub(crate) fn add(series: &str, delta: i64) {
        STATE.with_borrow_mut(|state| *state.entry(series.to_string()).or_default() += delta);
    }

    pub(crate) fn get(series: &str) -> i64 {
        STATE.with_borrow(|state| state.get(series).copied().unwrap_or_default())
    }

    /// Every series that is not currently zero, for an assertion that wants to name the offenders.
    pub(crate) fn non_zero() -> BTreeMap<String, i64> {
        STATE.with_borrow(|state| {
            state
                .iter()
                .filter(|(_, value)| **value != 0)
                .map(|(name, value)| (name.clone(), *value))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{super::gate::GateVerdict, *};

    /// A handle over a gate that serves nothing, for tests about the census alone.
    ///
    /// `ServiceGate::new(0, 0)` is a strict-prepay gate with a zero ceiling: it admits nothing on
    /// either branch, so `served_split` stays `(0, 0)` and the egress flush contributes nothing to
    /// what these tests assert.
    fn handle() -> PixSessionTelemetry {
        PixSessionTelemetry::new(ServiceGate::new(0, 0))
    }

    fn snapshot(mode: PixGateMode, recovering: u32) -> PixSessionSnapshot {
        PixSessionSnapshot {
            gate_mode: mode,
            recovering,
            ..Default::default()
        }
    }

    #[test]
    fn a_fresh_handle_has_published_nothing() {
        assert_eq!(None, handle().published());
    }

    #[test]
    fn publish_stores_the_latest_census() {
        let telemetry = handle();

        telemetry.publish(snapshot(PixGateMode::Predeposit, 0), PixTurnEvents::default());
        assert_eq!(Some(snapshot(PixGateMode::Predeposit, 0)), telemetry.published());

        let funded = snapshot(PixGateMode::Funded, 2);
        telemetry.publish(funded, PixTurnEvents::default());
        assert_eq!(Some(funded), telemetry.published());
    }

    #[test]
    fn release_clears_the_census_and_is_idempotent() {
        let telemetry = handle();
        telemetry.publish(snapshot(PixGateMode::Funded, 3), PixTurnEvents::default());

        telemetry.release();
        assert_eq!(None, telemetry.published());

        // A second release must not decrement anything a second time; there is nothing left to
        // assert on this handle, so the guarantee is that it neither panics nor resurrects state.
        telemetry.release();
        assert_eq!(None, telemetry.published());
    }

    #[test]
    fn publishing_after_release_is_ignored() {
        let telemetry = handle();
        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        telemetry.release();

        telemetry.publish(snapshot(PixGateMode::Funded, 9), PixTurnEvents::default());
        assert_eq!(
            None,
            telemetry.published(),
            "a worker still draining its channel must not re-inflate a released Session"
        );
    }

    /// A publish that loses the race to `release` keeps the counts `release` cannot rebuild.
    ///
    /// The worker takes the batch out of the supervisor's latch *before* it calls `publish`, so a
    /// `release` from `close_session` landing in between leaves the worker holding the only copy.
    /// Dropping the batch whole would lose the phase-transition counts for good; emitting it whole
    /// would let `release`'s census charge and the batch's own terms account for the same cycle
    /// twice.
    #[test]
    fn a_publish_that_loses_to_release_still_books_the_uncoupled_counts() {
        let telemetry = handle();
        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        telemetry.release();
        probe::reset();

        telemetry.publish(
            snapshot(PixGateMode::Funded, 1),
            PixTurnEvents {
                requested: 3,
                committed: 2,
                funded: 1,
                recovered: 1,
                failed: 1,
            },
        );

        assert_eq!(2, probe::get("cycles_total/committed"));
        assert_eq!(1, probe::get("cycles_total/funded"));
        for coupled in ["requested", "recovered", "failed"] {
            assert_eq!(
                0,
                probe::get(&format!("cycles_total/{coupled}")),
                "{coupled} is settled against the census, which `release` has already charged"
            );
        }
        assert_eq!(
            None,
            telemetry.published(),
            "a worker still draining its channel must not re-inflate a released Session"
        );
    }

    /// `requested = recovered + failed + live` survives `release` beating the final publish.
    ///
    /// This is the property the split in [`EventScope`] exists to protect: the losing batch's
    /// `recovered` is not counted, because `release` has already charged that same cycle from the
    /// census it was still in.
    #[test]
    fn the_lifecycle_invariant_holds_when_release_beats_the_final_publish() {
        probe::reset();
        let telemetry = handle();
        telemetry.publish(
            snapshot(PixGateMode::Funded, 2),
            PixTurnEvents {
                requested: 2,
                ..Default::default()
            },
        );

        // The worker has drained a turn in which one of the two recovered — and loses the race.
        telemetry.release();
        telemetry.publish(
            PixSessionSnapshot {
                gate_mode: PixGateMode::Funded,
                recovering: 1,
                paid_tail: 1,
                ..Default::default()
            },
            PixTurnEvents {
                committed: 1,
                recovered: 1,
                ..Default::default()
            },
        );

        let requested = probe::get("cycles_total/requested");
        let recovered = probe::get("cycles_total/recovered");
        let failed = probe::get("cycles_total/failed");
        assert_eq!(2, requested);
        assert_eq!(
            requested,
            recovered + failed,
            "every requested cycle must be accounted exactly once, whichever side of the race it landed on \
             (recovered={recovered}, failed={failed})"
        );
        assert_eq!(1, probe::get("cycles_total/committed"), "the edge count still lands");
        assert!(
            probe::non_zero()
                .keys()
                .all(|series| !series.starts_with("cycles_active/"))
        );
    }

    #[test]
    fn releasing_a_handle_that_published_nothing_is_a_no_op() {
        let telemetry = handle();
        telemetry.release();
        assert_eq!(None, telemetry.published());
    }

    /// The live-cycle count that `release` charges as `failed` excludes the paid tail.
    ///
    /// A paid recovered predecessor has already been counted `recovered`; charging it again on the
    /// way out would make `requested = recovered + failed` overshoot by one per rollover.
    #[test]
    fn live_cycles_counts_the_three_unfinished_phases_only() {
        let snapshot = PixSessionSnapshot {
            gate_mode: PixGateMode::Funded,
            awaiting_commitment: 2,
            awaiting_deposit: 3,
            recovering: 4,
            paid_tail: 1,
            predeposit_exposure_packets: 17,
        };
        assert_eq!(9, snapshot.live_cycles());
    }

    #[test]
    fn label_values_are_lowercase_and_stable() {
        assert_eq!("predeposit", PixGateMode::Predeposit.to_string());
        assert_eq!("funded", PixGateMode::Funded.to_string());
        assert_eq!("awaiting_commitment", PixCyclePhase::AwaitingCommitment.to_string());
        assert_eq!("awaiting_deposit", PixCyclePhase::AwaitingDeposit.to_string());
        assert_eq!("recovering", PixCyclePhase::Recovering.to_string());
        assert_eq!("paid_tail", PixCyclePhase::PaidTail.to_string());
        assert_eq!("requested", PixCycleEvent::Requested.to_string());
        assert_eq!("failed", PixCycleEvent::Failed.to_string());
        assert_eq!(
            "live_cycle_capacity",
            PixAdmissionRejection::LiveCycleCapacity.to_string()
        );
        assert_eq!("no_pix_support", PixAdmissionRejection::NoPixSupport.to_string());
    }

    /// A refusal is only a PIX admission refusal if the peer was asking for PIX.
    ///
    /// The six call sites all sit on paths that refuse ordinary Sessions too, so without this gate
    /// `hopr_pix_admission_rejections_total` would report a PIX problem on a node serving no PIX at
    /// all. Exercised here rather than at the instruments because this is the part that is compiled
    /// — and therefore tested — whether or not the `telemetry` feature is on.
    #[test]
    fn only_a_pix_offer_counts_as_a_pix_admission_refusal() {
        let pix: crate::Capabilities = crate::Capability::UsePIX | crate::Capability::Segmentation;
        let plain: crate::Capabilities = crate::Capability::Segmentation.into();

        for reason in [
            PixAdmissionRejection::NoPixSupport,
            PixAdmissionRejection::UnacceptableParams,
            PixAdmissionRejection::TargetPolicy,
            PixAdmissionRejection::LiveCycleCapacity,
            PixAdmissionRejection::NoSessionSlot,
            PixAdmissionRejection::Busy,
        ] {
            assert!(reason.record_if_pix(pix), "{reason} refused a PIX offer");
            assert!(
                !reason.record_if_pix(plain),
                "{reason} refused an ordinary Session and must not be counted as a PIX refusal"
            );
            assert!(
                !reason.record_if_pix(crate::Capabilities::default()),
                "{reason} refused a Session offering nothing at all"
            );
        }
    }

    #[test]
    fn an_empty_event_latch_is_recognised() {
        assert!(PixTurnEvents::default().is_empty());
        assert!(
            !PixTurnEvents {
                requested: 1,
                ..Default::default()
            }
            .is_empty()
        );
    }

    // -----------------------------------------------------------------------
    // What actually reaches the aggregates
    // -----------------------------------------------------------------------

    /// A publish moves each gauge by the difference between two censuses, never by the census.
    #[test]
    fn successive_publishes_emit_differences_not_totals() {
        probe::reset();
        let telemetry = handle();

        telemetry.publish(
            PixSessionSnapshot {
                gate_mode: PixGateMode::Predeposit,
                awaiting_commitment: 3,
                predeposit_exposure_packets: 100,
                ..Default::default()
            },
            PixTurnEvents::default(),
        );
        assert_eq!(1, probe::get("sessions_active/predeposit"));
        assert_eq!(3, probe::get("cycles_active/awaiting_commitment"));
        assert_eq!(100, probe::get("predeposit_exposure"));

        // Two of the three commitments verify and the third is still waiting; the exposure grows.
        telemetry.publish(
            PixSessionSnapshot {
                gate_mode: PixGateMode::Predeposit,
                awaiting_commitment: 1,
                awaiting_deposit: 2,
                predeposit_exposure_packets: 250,
                ..Default::default()
            },
            PixTurnEvents {
                committed: 2,
                ..Default::default()
            },
        );
        assert_eq!(1, probe::get("cycles_active/awaiting_commitment"));
        assert_eq!(2, probe::get("cycles_active/awaiting_deposit"));
        assert_eq!(250, probe::get("predeposit_exposure"), "absolute, not 100 + 250");
        assert_eq!(
            1,
            probe::get("sessions_active/predeposit"),
            "a Session that stayed in one bucket must not be counted twice"
        );
        assert_eq!(2, probe::get("cycles_total/committed"));
    }

    /// A Session moving between gate modes leaves one bucket and enters the other.
    #[test]
    fn a_funding_session_moves_between_the_two_buckets() {
        probe::reset();
        let telemetry = handle();

        telemetry.publish(snapshot(PixGateMode::Predeposit, 1), PixTurnEvents::default());
        assert_eq!(1, probe::get("sessions_active/predeposit"));
        assert_eq!(0, probe::get("sessions_active/funded"));

        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        assert_eq!(0, probe::get("sessions_active/predeposit"), "the old bucket must drain");
        assert_eq!(1, probe::get("sessions_active/funded"));

        // Rotating back to an unfunded successor moves it back, rather than counting a new Session.
        telemetry.publish(snapshot(PixGateMode::Predeposit, 1), PixTurnEvents::default());
        assert_eq!(1, probe::get("sessions_active/predeposit"));
        assert_eq!(0, probe::get("sessions_active/funded"));
    }

    /// `release` returns every gauge to zero and charges what was still live.
    #[test]
    fn release_zeroes_the_gauges_and_charges_the_live_cycles() {
        probe::reset();
        let telemetry = handle();

        telemetry.publish(
            PixSessionSnapshot {
                gate_mode: PixGateMode::Funded,
                awaiting_commitment: 1,
                awaiting_deposit: 2,
                recovering: 3,
                paid_tail: 1,
                predeposit_exposure_packets: 64,
            },
            PixTurnEvents {
                requested: 6,
                ..Default::default()
            },
        );

        telemetry.release();

        assert_eq!(
            std::collections::BTreeMap::from([
                ("cycles_total/requested".to_string(), 6),
                ("cycles_total/failed".to_string(), 6),
            ]),
            probe::non_zero(),
            "only the cumulative counters may survive a release"
        );
        assert_eq!(
            6,
            probe::get("cycles_total/failed"),
            "the paid tail has already been counted as recovered and must not be charged again"
        );
    }

    /// A Session that ends with nothing live charges nothing.
    #[test]
    fn releasing_a_drained_session_charges_no_failure() {
        probe::reset();
        let telemetry = handle();

        telemetry.publish(
            PixSessionSnapshot {
                gate_mode: PixGateMode::Funded,
                paid_tail: 1,
                ..Default::default()
            },
            PixTurnEvents {
                requested: 1,
                recovered: 1,
                ..Default::default()
            },
        );
        telemetry.release();

        assert_eq!(0, probe::get("cycles_total/failed"));
        assert_eq!(1, probe::get("cycles_total/recovered"));
        assert!(probe::non_zero().keys().all(|k| k.starts_with("cycles_total/")));
    }

    /// Dropping a handle that was never released still returns its gauges.
    #[test]
    fn drop_is_a_backstop_for_a_handle_nobody_released() {
        probe::reset();

        {
            let telemetry = handle();
            telemetry.publish(snapshot(PixGateMode::Funded, 2), PixTurnEvents::default());
            assert_eq!(1, probe::get("sessions_active/funded"));
        }

        assert_eq!(0, probe::get("sessions_active/funded"));
        assert_eq!(0, probe::get("cycles_active/recovering"));
    }

    /// A double release decrements exactly once.
    #[test]
    fn a_second_release_does_not_decrement_again() {
        probe::reset();
        let telemetry = handle();

        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        telemetry.release();
        telemetry.release();
        // And the `Drop` backstop on top of the two explicit calls.
        drop(telemetry);

        assert_eq!(
            0,
            probe::get("sessions_active/funded"),
            "a repeated release must not drive the gauge negative"
        );
        assert_eq!(1, probe::get("cycles_total/failed"), "the loss is charged once");
    }

    // -----------------------------------------------------------------------
    // Gate pressure
    // -----------------------------------------------------------------------

    /// An episode is counted once when it begins and closed once when it ends.
    #[test]
    fn a_block_episode_is_counted_once_and_closed_once() {
        probe::reset();

        {
            let _episode = PixGateBlockEpisode::begin(GateBlockReason::PredepositExhausted);
            assert_eq!(1, probe::get("gate_blocks/predeposit_exhausted"));
            assert_eq!(
                0,
                probe::get("gate_block_observations/predeposit_exhausted"),
                "the duration must not be observed while the episode is still open"
            );
        }

        assert_eq!(1, probe::get("gate_block_observations/predeposit_exhausted"));
        assert_eq!(1, probe::get("gate_blocks/predeposit_exhausted"), "still one episode");
    }

    /// Successive episodes are separate, and each reason is counted under its own label.
    #[test]
    fn each_reason_is_counted_separately() {
        probe::reset();

        drop(PixGateBlockEpisode::begin(GateBlockReason::PredepositExhausted));
        drop(PixGateBlockEpisode::begin(GateBlockReason::ShareLag));
        drop(PixGateBlockEpisode::begin(GateBlockReason::ShareLag));

        assert_eq!(1, probe::get("gate_blocks/predeposit_exhausted"));
        assert_eq!(2, probe::get("gate_blocks/share_lag"));
        assert_eq!(1, probe::get("gate_block_observations/predeposit_exhausted"));
        assert_eq!(2, probe::get("gate_block_observations/share_lag"));
    }

    /// A poisoned gate is counted but never timed.
    ///
    /// It is not a stall the Session recovers from, so a duration on it would measure how long the
    /// teardown took to reach the last writer — which is not egress pressure and would drag the
    /// histogram's tail for a reason that is not one.
    #[test]
    fn a_closed_gate_is_counted_without_a_duration() {
        probe::reset();

        record_gate_closed();
        record_gate_closed();

        assert_eq!(2, probe::get("gate_blocks/closed"));
        assert_eq!(
            std::collections::BTreeMap::from([("gate_blocks/closed".to_string(), 2)]),
            probe::non_zero(),
            "a closed gate must observe no duration under any reason"
        );
    }

    /// Egress is flushed as a delta from the gate, never as its running total.
    #[test]
    fn egress_is_flushed_as_a_delta_from_the_gate() -> anyhow::Result<()> {
        probe::reset();
        let gate = ServiceGate::new(2, 10);
        let telemetry = PixSessionTelemetry::new(gate.clone());

        // Two packets on the allowance, then a publish.
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        telemetry.publish(snapshot(PixGateMode::Predeposit, 1), PixTurnEvents::default());
        assert_eq!(2, probe::get("egress_packets/predeposit"));
        assert_eq!(0, probe::get("egress_packets/funded"));

        // A publish with nothing served in between adds nothing.
        telemetry.publish(snapshot(PixGateMode::Predeposit, 1), PixTurnEvents::default());
        assert_eq!(
            2,
            probe::get("egress_packets/predeposit"),
            "totals must not be re-added"
        );

        // Funded service lands in the other bucket, and the allowance total is untouched.
        gate.release_service();
        for _ in 0..3 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        }
        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        assert_eq!(2, probe::get("egress_packets/predeposit"));
        assert_eq!(3, probe::get("egress_packets/funded"));
        Ok(())
    }

    /// `release` flushes the packets served since the last publish rather than losing them.
    ///
    /// This is the tail between a Session's final supervisor turn and its teardown. It is not
    /// hypothetical: the worker publishes per turn, and a Session passing data serves thousands of
    /// packets between turns.
    #[test]
    fn release_flushes_the_final_egress_tail() -> anyhow::Result<()> {
        probe::reset();
        let gate = ServiceGate::new(10, 10);
        let telemetry = PixSessionTelemetry::new(gate.clone());

        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        telemetry.publish(snapshot(PixGateMode::Predeposit, 1), PixTurnEvents::default());
        assert_eq!(1, probe::get("egress_packets/predeposit"));

        // Four more packets with no publish in between — the window `release` has to cover.
        for _ in 0..4 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        }
        telemetry.release();

        assert_eq!(
            5,
            probe::get("egress_packets/predeposit"),
            "packets served after the last publish must still be counted"
        );
        assert_eq!(
            0,
            probe::get("sessions_active/predeposit"),
            "the gauges still come back to zero"
        );
        Ok(())
    }

    /// A flush never lowers the egress watermark, so ground already counted is not counted again.
    ///
    /// The rule rather than the race. `served_split` derives its funded component by subtraction
    /// from two non-atomic loads, and a predeposit permit increments `served_predeposit` before it
    /// publishes `served` — so a permit landing between the loads is counted in the split but not
    /// yet in the total, and yields a funded figure *below* the previous one. That is the tear the
    /// publication order deliberately leaves reachable, because it emits nothing; reproducing the
    /// interleaving would be a timing test, and what has to hold is simply that a reading below the
    /// watermark leaves the watermark where it is. The watermark is put ahead of the gate here to
    /// reach the same state deterministically.
    #[test]
    fn a_flush_never_rewinds_the_egress_watermark() -> anyhow::Result<()> {
        probe::reset();
        let gate = ServiceGate::new(0, 100);
        let telemetry = PixSessionTelemetry::new(gate.clone());
        gate.release_service();

        for _ in 0..6 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        }
        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        assert_eq!(6, probe::get("egress_packets/funded"));

        // The watermark now claims more than the gate will report — the shape a torn read leaves.
        telemetry.state.lock().egress.1 = 9;

        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        assert_eq!(
            6,
            probe::get("egress_packets/funded"),
            "a reading below the watermark adds nothing"
        );
        assert_eq!(
            9,
            telemetry.state.lock().egress.1,
            "and must not drag the watermark back down to it"
        );

        // Four more packets take the gate to 10, of which exactly one is above the watermark.
        for _ in 0..4 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        }
        telemetry.publish(snapshot(PixGateMode::Funded, 1), PixTurnEvents::default());
        assert_eq!(
            7,
            probe::get("egress_packets/funded"),
            "only the packet past the watermark is new; a rewound one would report four"
        );
        Ok(())
    }

    /// Cumulative counters are not returned on release, only gauges are.
    #[test]
    fn release_does_not_decrement_the_cumulative_counters() -> anyhow::Result<()> {
        probe::reset();
        let gate = ServiceGate::new(4, 10);
        let telemetry = PixSessionTelemetry::new(gate.clone());

        for _ in 0..4 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        }
        telemetry.publish(snapshot(PixGateMode::Predeposit, 1), PixTurnEvents::default());
        telemetry.release();
        drop(telemetry);

        assert_eq!(
            4,
            probe::get("egress_packets/predeposit"),
            "a monotonic counter must never be walked back"
        );
        Ok(())
    }

    /// More Sessions than the OpenTelemetry per-instrument cardinality limit, opened and closed.
    ///
    /// This is the shape that breaks the per-Session instruments (#8305): past ~2000 distinct label
    /// tuples the SDK folds everything further into one `otel.metric.overflow` bucket and quietly
    /// stops describing reality. These aggregates carry no per-Session label at all, so the property
    /// to prove is the other half — that the *values* stay exact over that many lifecycles, with
    /// every gauge back at zero and every cycle accounted for exactly once.
    #[test]
    fn two_thousand_session_lifecycles_leave_the_aggregates_exact() {
        const SESSIONS: usize = 2_001;
        const CYCLES_PER_SESSION: u32 = 3;

        probe::reset();

        for i in 0..SESSIONS {
            let telemetry = handle();

            // Request three cycles, fund them, and let two of the three recover.
            telemetry.publish(
                PixSessionSnapshot {
                    gate_mode: PixGateMode::Predeposit,
                    awaiting_commitment: CYCLES_PER_SESSION,
                    predeposit_exposure_packets: 512,
                    ..Default::default()
                },
                PixTurnEvents {
                    requested: CYCLES_PER_SESSION,
                    ..Default::default()
                },
            );
            telemetry.publish(
                PixSessionSnapshot {
                    gate_mode: PixGateMode::Funded,
                    recovering: CYCLES_PER_SESSION,
                    ..Default::default()
                },
                PixTurnEvents {
                    committed: CYCLES_PER_SESSION,
                    funded: CYCLES_PER_SESSION,
                    ..Default::default()
                },
            );
            telemetry.publish(
                PixSessionSnapshot {
                    gate_mode: PixGateMode::Funded,
                    recovering: 1,
                    paid_tail: 1,
                    ..Default::default()
                },
                PixTurnEvents {
                    recovered: 2,
                    ..Default::default()
                },
            );

            // Half the Sessions are closed cleanly and half die with that last cycle still live, so
            // both the retire-then-release and the release-charges-it paths are exercised.
            if i % 2 == 0 {
                telemetry.publish(
                    PixSessionSnapshot {
                        gate_mode: PixGateMode::Funded,
                        ..Default::default()
                    },
                    PixTurnEvents {
                        recovered: 1,
                        ..Default::default()
                    },
                );
            }
            telemetry.release();
        }

        assert_eq!(
            std::collections::BTreeMap::new(),
            probe::non_zero()
                .into_iter()
                .filter(|(name, _)| !name.starts_with("cycles_total/"))
                .collect::<std::collections::BTreeMap<_, _>>(),
            "every live-set gauge must be back at zero after {SESSIONS} lifecycles"
        );

        let requested = probe::get("cycles_total/requested");
        let recovered = probe::get("cycles_total/recovered");
        let failed = probe::get("cycles_total/failed");
        assert_eq!(SESSIONS as i64 * i64::from(CYCLES_PER_SESSION), requested);
        assert_eq!(
            requested,
            recovered + failed,
            "every requested cycle must end as exactly one of recovered or failed, got {recovered} recovered and \
             {failed} failed against {requested} requested"
        );
        assert_eq!(
            SESSIONS as i64 / 2,
            failed,
            "exactly the odd-numbered Sessions died with a cycle in flight"
        );
    }
}
