//! Per-session actor for the [`SessionPixSupervisor`].
//!
//! The worker serializes lifecycle events through the deterministic core,
//! manages the deadline timer, applies safety-critical gate actions locally,
//! and forwards actions to an external I/O driver.
//!
//! Runtime-agnostic: uses crossfire channels and the runtime prelude from
//! `hopr_utils` so no direct tokio dependency (tests use tokio freely).

use std::{sync::Arc, time::Instant};

use crossfire::{AsyncRx, MAsyncTx, SendError, TrySendError, mpsc::Array};
use futures_time::future::FutureExt as TimeExt;
use hopr_api::types::internal::prelude::HoprPseudonym;
use hopr_protocol_pix::SsaRecoveryProgress;
use hopr_utils::runtime::prelude::spawn;

use super::{
    PixParams, SessionPixAction, SessionPixCloseReason, SessionPixEvent, SupervisorConfig, gate::ServiceGate,
    supervisor::SessionPixSupervisor,
};

// ---------------------------------------------------------------------------
// Channel type aliases
// ---------------------------------------------------------------------------

type CmdChannel = Array<WorkerCommand>;
type CmdTx = MAsyncTx<CmdChannel>;
type CmdRx = AsyncRx<CmdChannel>;

type ActionChannel = Array<SessionPixAction>;
pub type ActionTx = MAsyncTx<ActionChannel>;
pub type ActionRx = AsyncRx<ActionChannel>;

// ---------------------------------------------------------------------------
// SessionPixSupervisorHandle
// ---------------------------------------------------------------------------

/// Cloneable handle to a running `SessionPixWorker`.
#[derive(Clone)]
pub struct SessionPixSupervisorHandle {
    cmd_tx: CmdTx,
    pub(crate) gate: Arc<ServiceGate>,
}

impl SessionPixSupervisorHandle {
    /// Send a PIX event to the supervisor, awaiting capacity if the channel is
    /// full.
    ///
    /// Returns `Err` if the worker is no longer running. Backpressures instead
    /// of dropping events, so overflow cannot occur by construction.
    pub async fn send_event(&self, ev: SessionPixEvent) -> Result<(), ()> {
        match self.cmd_tx.send(WorkerCommand::Event(ev)).await {
            Ok(()) => Ok(()),
            Err(SendError(_)) => {
                tracing::warn!("PIX supervisor command channel closed");
                Err(())
            }
        }
    }

    /// Send an action result feedback to the supervisor, awaiting capacity if
    /// the channel is full.
    pub async fn send_action_result(&self, action: SessionPixAction, ok: bool) -> Result<(), ()> {
        match self.cmd_tx.send(WorkerCommand::ActionResult { action, ok }).await {
            Ok(()) => Ok(()),
            Err(SendError(_)) => {
                tracing::warn!("PIX supervisor result channel closed");
                Err(())
            }
        }
    }

    /// Deliver a recovery-progress snapshot without blocking, dropping it if the channel is full.
    ///
    /// Returns `true` if the snapshot was queued.
    ///
    /// Unlike every other event, progress arrives once per acknowledgement batch rather than once
    /// per SSA lifecycle transition, so it is the one input whose rate is set by traffic. Awaiting
    /// capacity for it would put the supervisor's scheduling latency on the acknowledgement path —
    /// where a stall costs share verification throughput for the whole Session.
    ///
    /// Dropping is safe by construction rather than by tolerance: snapshots carry absolute counters,
    /// [`on_recovery_progress`](super::supervisor::SessionPixSupervisor) keeps the maximum it has
    /// seen, and a lower or repeated `useful_shares` is already treated as benign — concurrent
    /// acknowledgement batches make out-of-order snapshots ordinary. A dropped snapshot is therefore
    /// indistinguishable from one that arrived out of order, and the next batch supersedes it.
    ///
    /// The one thing a drop delays is the [`ProgressNotification`](SessionPixAction::ProgressNotification)
    /// that resets the gate's served-without-progress ceiling, so `max_served_without_progress` has
    /// to leave room for a few missed snapshots. Since the command channel is drained by a state
    /// machine that performs no I/O, drops require a genuinely wedged worker.
    pub fn try_send_progress(&self, progress: SsaRecoveryProgress<HoprPseudonym>) -> bool {
        match self
            .cmd_tx
            .try_send(WorkerCommand::Event(SessionPixEvent::RecoveryProgress(progress)))
        {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                tracing::debug!(
                    ssa_id = %progress.ssa_id,
                    "supervisor command channel full — dropping progress snapshot"
                );
                false
            }
            Err(TrySendError::Disconnected(_)) => {
                tracing::warn!("PIX supervisor command channel closed");
                false
            }
        }
    }
}

// ---------------------------------------------------------------------------
// WorkerCommand
// ---------------------------------------------------------------------------

pub enum WorkerCommand {
    Event(SessionPixEvent),
    ActionResult { action: SessionPixAction, ok: bool },
}

// ---------------------------------------------------------------------------
// spawn_supervisor_worker
// ---------------------------------------------------------------------------

/// Spawn a supervisor worker and return its handle and action driver receiver.
pub fn spawn_supervisor_worker(
    cfg: SupervisorConfig,
    dims: PixParams,
    pseudonym: hopr_api::types::internal::prelude::HoprPseudonym,
    now: Instant,
) -> (SessionPixSupervisorHandle, ActionRx) {
    let (cmd_tx, cmd_rx) = crossfire::mpsc::bounded_async::<WorkerCommand>(64);
    let (action_tx, action_rx) = crossfire::mpsc::bounded_async::<SessionPixAction>(64);

    let predeposit_budget = std::cmp::min(
        dims.target_useful_shares().saturating_sub(1),
        cfg.max_predeposit_packets,
    );
    let gate = ServiceGate::new(predeposit_budget, cfg.max_served_without_progress);

    let handle = SessionPixSupervisorHandle {
        cmd_tx,
        gate: gate.clone(),
    };

    let (supervisor, initial_actions) = SessionPixSupervisor::new(cfg, dims, pseudonym, now);

    spawn(worker_loop(supervisor, cmd_rx, action_tx, gate, initial_actions));

    (handle, action_rx)
}

// ---------------------------------------------------------------------------
// Worker loop
// ---------------------------------------------------------------------------

async fn worker_loop(
    mut supervisor: SessionPixSupervisor,
    cmd_rx: CmdRx,
    action_tx: ActionTx,
    gate: Arc<ServiceGate>,
    initial_actions: Vec<SessionPixAction>,
) {
    // Emit initial actions. `false` says only that a freshly built supervisor has not flagged
    // itself, which it cannot have: `new` does not close. It is not a claim that these actions are
    // non-terminal — `dispatch` reads the payload for that, so a construction-time `Close` would
    // still fail closed here rather than being waved through on the strength of the flag.
    if !dispatch(&initial_actions, false, &action_tx, &gate) {
        return;
    }

    loop {
        let deadline = supervisor.next_deadline();

        if let Some(dl) = deadline {
            let now = Instant::now();
            if now >= dl {
                let actions = supervisor.handle_deadline(now, gate.served_total());
                if !dispatch(&actions, supervisor.closed, &action_tx, &gate) {
                    return;
                }
                continue;
            }

            let duration = dl.saturating_duration_since(Instant::now());

            match cmd_rx
                .recv()
                .timeout(futures_time::time::Duration::from(duration))
                .await
            {
                Ok(result) => {
                    if !process_cmd(result.ok(), &mut supervisor, &action_tx, &gate).await {
                        return;
                    }
                }
                Err(_) => {
                    let now = Instant::now();
                    let actions = supervisor.handle_deadline(now, gate.served_total());
                    if !dispatch(&actions, supervisor.closed, &action_tx, &gate) {
                        return;
                    }
                }
            }
        } else {
            let cmd = cmd_rx.recv().await.ok();
            if !process_cmd(cmd, &mut supervisor, &action_tx, &gate).await {
                return;
            }
        }
    }
}

/// Handle a received command from the handle.
///
/// Returns `false` to signal the worker loop to stop.
async fn process_cmd(
    cmd: Option<WorkerCommand>,
    supervisor: &mut SessionPixSupervisor,
    action_tx: &ActionTx,
    gate: &Arc<ServiceGate>,
) -> bool {
    let cmd = match cmd {
        Some(c) => c,
        None => {
            // All senders dropped — close. Terminal by construction, so `dispatch` is told the
            // supervisor is closed and its `false` is the verdict.
            let actions = vec![SessionPixAction::Close(SessionPixCloseReason::SupervisorUnavailable)];
            return dispatch(&actions, true, action_tx, gate);
        }
    };

    match cmd {
        WorkerCommand::Event(ev) => {
            let now = Instant::now();
            let actions = supervisor.handle_event(&ev, now, gate.served_total());
            if !dispatch(&actions, supervisor.closed, action_tx, gate) {
                return false;
            }
        }
        WorkerCommand::ActionResult { action, ok } => {
            let now = Instant::now();
            let actions = supervisor.action_result(&action, ok, now);
            if !dispatch(&actions, supervisor.closed, action_tx, gate) {
                return false;
            }
        }
    }
    true
}

/// Apply gate control, forward `actions`, then report whether the worker should keep running.
///
/// The four places that produce actions — the two deadline paths in [`worker_loop`] and the two
/// command arms in [`process_cmd`] — all had the same four-way branch spelled out: send, check
/// whether the supervisor closed, poison, return. Fail-close is the property that matters most here
/// and it was stated four times, so a correction to it had to be made four times.
///
/// Poisons the gate on every terminal path, so no caller has to remember to. `closed` is passed
/// rather than read from the supervisor because the callers hold it by different borrows.
///
/// Terminality is derived from the *actions* as well as from `closed`, because the two do not
/// always agree: several handlers return [`Close`](SessionPixAction::Close) without setting the
/// supervisor's flag — both `InvalidTransition` paths in `on_ssa_request_sent`, `CounterRegression`
/// and the share-order verdict in `on_recovery_progress`, and `on_unverifiable_shares`. Two of those
/// are the anti-abuse verdicts. Reading only the flag would send the close, return `true`, and leave
/// the gate open for as long as it takes the action driver to poison it on receipt — which is a real
/// but bounded window, not a correct one. `Close` is also the one action the driver never reports
/// back, so `action_result`'s own close path cannot cover for it.
fn dispatch(actions: &[SessionPixAction], closed: bool, action_tx: &ActionTx, gate: &Arc<ServiceGate>) -> bool {
    // Gate control is local state, not I/O. Apply it before touching the action channel so a safety
    // transition cannot wait behind commitment generation, deposit-data lookup, or a network send in
    // the action driver. Stop at Close to preserve the driver's rule that actions after a close are
    // unreachable; poisoning below is stronger than every gate mode.
    for action in actions
        .iter()
        .take_while(|action| !matches!(action, SessionPixAction::Close(_)))
    {
        match action {
            SessionPixAction::ReleaseService => gate.release_service(),
            SessionPixAction::WithholdService => gate.withhold_service(),
            SessionPixAction::ProgressNotification => gate.notify_progress(),
            _ => {}
        }
    }

    let terminal = closed || actions.iter().any(|a| matches!(a, SessionPixAction::Close(_)));

    // Sent before the verdict either way: a closing supervisor's last actions carry the reason it
    // closed, and dropping them would leave the driver to infer it.
    let delivered = send_actions(actions, action_tx);
    if terminal || !delivered {
        gate.poison();
        return false;
    }
    true
}

/// Non-blocking forward of actions to the driver.
///
/// On `Disconnected`, the driver is gone — returns `false` so the caller
/// can fail-close.
///
/// On `Full`:
/// - **Coalescible** actions (`ProgressNotification`) are logged + skipped. They are idempotent and safe to drop — the
///   next notification will replace them, and dropping here prevents transient load from killing a healthy session.
/// - **Non-coalescible** actions (every other variant) are treated as fatal. If these cannot be delivered the channel
///   is genuinely wedged.
fn send_actions(actions: &[SessionPixAction], action_tx: &ActionTx) -> bool {
    for action in actions {
        match action_tx.try_send(action.clone()) {
            Ok(()) => continue,
            Err(TrySendError::Full(item)) => {
                if is_coalescible(&item) {
                    tracing::trace!(?action, "action channel full, dropping coalescible action");
                    continue;
                }
                tracing::warn!(?action, "non-coalescible action dropped — channel full");
                return false;
            }
            Err(TrySendError::Disconnected(_item)) => {
                tracing::warn!(?action, "action driver disconnected");
                return false;
            }
        }
    }
    true
}

/// Returns `true` for actions that are safe to drop when the action channel
/// is transiently full.
fn is_coalescible(action: &SessionPixAction) -> bool {
    matches!(action, SessionPixAction::ProgressNotification)
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
            ssas_per_request: 1,
            allow_dynamic_ssa_batches: true,
            max_failed_cycles: 1,
            max_ssa_delivery_time: Duration::from_secs(20),
            max_deposit_wait: Duration::from_secs(60),
            max_recovery_idle: Duration::from_secs(10),
            max_recovery_time: Duration::from_secs(3600),
            max_off_front_share_fraction: 0.25,
            min_share_order_sample: 16384,
            max_predeposit_packets: 1024,
            max_served_without_progress: 256,
            tombstone_retention_window: Duration::from_secs(30),
        }
    }

    /// See the identically-named helper in [`super::supervisor`] for why the surplus is non-zero.
    fn dims() -> PixParams {
        PixParams::try_new(10, 5, 7, crate::types::LOCAL_PIX_SUITE).expect("test dimensions must be valid")
    }

    /// Wait for `condition` to hold, failing the test if it never does.
    ///
    /// The worker is a detached task, so every assertion about its effects is really an assertion
    /// that it has been scheduled. A fixed sleep encodes a guess about the scheduler, and the guess
    /// is wrong under CI load or on a busy single-core runner — the test then fails with nothing
    /// wrong in the code. Polling asserts the same property and can only fail if it genuinely never
    /// happens, which is why the deadline is generous: it is reached only on a real failure.
    async fn poll_until(what: &str, mut condition: impl FnMut() -> bool) {
        const DEADLINE: Duration = Duration::from_secs(5);
        tokio::time::timeout(DEADLINE, async {
            while !condition() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("{what} — not observed within {DEADLINE:?}"));
    }

    #[tokio::test]
    async fn worker_creates_and_forwards_initial_request() {
        let (_handle, action_rx) =
            spawn_supervisor_worker(default_cfg(), dims(), HoprPseudonym::random(), Instant::now());

        let action = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout waiting for initial action")
            .expect("action stream ended");

        match action {
            SessionPixAction::RequestSsa { ssa_ids, .. } => {
                assert_eq!(ssa_ids[0].ssa_index(), SsaIndex::new(1).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }
    }

    /// `max_predeposit_packets = 0` must reach the gate as a zero budget.
    ///
    /// The budget is computed here, not in the gate, and nothing else exercises that line with a
    /// zero cap — every other config in the tree sets a four-figure allowance. A `max(1, ..)` or an
    /// off-by-one creeping into it would hand an unfunded Entry service that the Exit had
    /// explicitly declined to offer, and the gate's own tests would still pass.
    #[tokio::test]
    async fn zero_predeposit_config_reaches_the_gate_as_strict_prepay() {
        let cfg = SupervisorConfig {
            max_predeposit_packets: 0,
            // Fill the funded ceiling immediately before recovery. `Recovered` itself must reset
            // it even if the final progress snapshot was dropped.
            max_served_without_progress: 1,
            ..default_cfg()
        };
        let (handle, _action_rx) = spawn_supervisor_worker(cfg, dims(), HoprPseudonym::random(), Instant::now());

        assert!(
            !handle.gate.try_acquire_sync().expect("a fresh gate is not poisoned"),
            "a strict-prepay gate must refuse the first packet"
        );
        assert_eq!(handle.gate.served_total(), 0);

        // What the action driver does when it carries out `ReleaseService`.
        handle.gate.release_service();
        assert!(
            handle.gate.try_acquire_sync().expect("a funded gate is not poisoned"),
            "funding must open a strict-prepay gate"
        );
        assert_eq!(handle.gate.served_total(), 1);
    }

    /// The configured cap can only lower the predeposit budget, never raise it.
    ///
    /// The other half of the same `min`: `dims()` is 10 × 5, so the dimensions bound the budget at 49
    /// however generous the configuration is. Pinned alongside the zero case so the line is covered
    /// from both directions rather than only where the cap happens to win.
    #[tokio::test]
    async fn predeposit_budget_is_bounded_by_the_ssa_dimensions() {
        let cfg = SupervisorConfig {
            max_predeposit_packets: 10_000,
            ..default_cfg()
        };
        let (handle, _action_rx) = spawn_supervisor_worker(cfg, dims(), HoprPseudonym::random(), Instant::now());

        for i in 0..49 {
            assert!(
                handle.gate.try_acquire_sync().expect("a fresh gate is not poisoned"),
                "packet {i} is within `target_useful_shares - 1` and must be served"
            );
        }
        assert!(
            !handle.gate.try_acquire_sync().expect("a fresh gate is not poisoned"),
            "the budget must stop at `target_useful_shares - 1`, not at the configured cap"
        );
    }

    #[tokio::test]
    async fn dropped_action_driver_fails_closed_and_poisons_gate() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Drop the action receiver — worker should detect and close.
        drop(action_rx);

        poll_until("the gate is poisoned after the action driver is dropped", || {
            handle.gate.try_acquire_sync().is_err()
        })
        .await;

        assert!(
            handle.gate.acquire().await.is_err(),
            "a poisoned gate must refuse the awaiting path too, not only the synchronous one"
        );
    }

    #[tokio::test]
    async fn worker_rearms_strict_prepay_without_waiting_for_the_action_driver() -> anyhow::Result<()> {
        let cfg = SupervisorConfig {
            max_predeposit_packets: 0,
            ..default_cfg()
        };
        let p = HoprPseudonym::random();
        let (handle, _stalled_action_rx) = spawn_supervisor_worker(cfg, dims(), p, Instant::now());
        let first = SsaId::new(
            p,
            SsaIndex::new(1).ok_or_else(|| anyhow::anyhow!("SSA index 1 must be valid"))?,
        );

        handle
            .send_event(SessionPixEvent::SsaRequestSent(first))
            .await
            .map_err(|()| anyhow::anyhow!("worker stopped"))?;
        handle
            .send_event(SessionPixEvent::CommitmentVerified(first))
            .await
            .map_err(|()| anyhow::anyhow!("worker stopped"))?;
        handle
            .send_event(SessionPixEvent::DepositConfirmed {
                ssa_id: first,
                amount: HoprBalance::new_base(1),
            })
            .await
            .map_err(|()| anyhow::anyhow!("worker stopped"))?;

        poll_until("the paid front opens the gate", || handle.gate.funded()).await;
        assert!(handle.gate.try_acquire_sync()?);

        // The receiver remains alive but deliberately unread, modeling an action driver blocked on
        // RequestSsa I/O. Cryptographic recovery alone must keep the gate funded for the bounded
        // predecessor tail.
        handle
            .send_event(SessionPixEvent::Recovered(first))
            .await
            .map_err(|()| anyhow::anyhow!("worker stopped"))?;
        poll_until("the recovered predecessor keeps its paid tail open", || {
            handle.gate.funded()
        })
        .await;
        assert!(
            handle.gate.try_acquire_sync()?,
            "cryptographic recovery must reopen a gate whose final progress notification was lost"
        );

        // The first successor-bound share is the actual FIFO boundary. Even though that successor
        // is unfunded, the worker must synchronously consume the predecessor receipt and restore
        // strict prepay; waiting for the stalled action driver would leak new service.
        let successor = SsaId::new(
            p,
            SsaIndex::new(2).ok_or_else(|| anyhow::anyhow!("SSA index 2 must be valid"))?,
        );
        handle
            .send_event(SessionPixEvent::RecoveryProgress(SsaRecoveryProgress {
                ssa_id: successor,
                useful_shares: 1,
                shares_seen: 1,
                target_useful_shares: dims().target_useful_shares(),
                recovered_polynomials: 0,
            }))
            .await
            .map_err(|()| anyhow::anyhow!("worker stopped"))?;
        poll_until("the unfunded successor closes the gate", || !handle.gate.funded()).await;
        assert!(!handle.gate.try_acquire_sync()?);
        Ok(())
    }

    #[tokio::test]
    async fn event_sent_via_handle_reaches_core() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Consume initial RequestSsa.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        // Send SsaRequestSent via the handle — the worker should process it
        // and produce no further actions (event is idempotent).
        let id = SsaId::new(p, SsaIndex::new(1).unwrap());
        handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.unwrap();

        // Give the worker time to process the event.
        tokio::time::sleep(Duration::from_millis(50)).await;

        // No extra actions should appear (idempotent event).
        let maybe_action = tokio::time::timeout(Duration::from_millis(50), action_rx.recv()).await;
        assert!(maybe_action.is_err(), "expected no extra actions");
    }

    #[tokio::test]
    async fn action_result_feedback_processed() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Consume initial RequestSsa.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        // Send action result for a failed RequestSsa — should trigger close.
        handle
            .send_action_result(
                SessionPixAction::RequestSsa {
                    ssa_ids: vec![SsaId::new(p, SsaIndex::new(1).unwrap())],
                    params: dims(),
                },
                false,
            )
            .await
            .unwrap();

        let close_action = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");
        assert!(matches!(close_action, SessionPixAction::Close(_)));
    }

    #[tokio::test]
    async fn deadline_via_worker_closes() {
        let mut cfg = default_cfg();
        cfg.max_ssa_delivery_time = Duration::from_millis(10);
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(cfg, dims(), p, Instant::now());

        // Consume initial RequestSsa.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        // Tell the worker the request was sent so the commitment deadline starts.
        let id = SsaId::new(p, SsaIndex::new(1).unwrap());
        handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.unwrap();

        // Wait for the commitment deadline to expire.
        let close_action = tokio::time::timeout(Duration::from_secs(2), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");
        assert!(matches!(
            close_action,
            SessionPixAction::Close(SessionPixCloseReason::CommitmentTimeout)
        ));
    }

    // -----------------------------------------------------------------------
    // H-04 / M-06: Worker termination and channel saturation
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn worker_terminates_when_all_command_senders_dropped() {
        let (handle, _action_rx) =
            spawn_supervisor_worker(default_cfg(), dims(), HoprPseudonym::random(), Instant::now());

        // Clone the gate before dropping the handle so we can assert the worker
        // poisoned it on exit.
        let gate = handle.gate.clone();

        // Drop the handle — last cmd_tx sender is dropped, cmd_rx yields None.
        drop(handle);

        poll_until("the gate is poisoned after all command senders are dropped", || {
            gate.try_acquire_sync().is_err()
        })
        .await;
    }

    #[tokio::test]
    async fn worker_terminates_when_supervisor_closes_after_event() {
        let mut cfg = default_cfg();
        cfg.max_ssa_delivery_time = Duration::from_millis(10);
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(cfg, dims(), p, Instant::now());

        // Consume initial RequestSsa.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        // Register the SSA so the commitment deadline starts.
        let id = SsaId::new(p, SsaIndex::new(1).unwrap());
        handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.unwrap();

        // Deadline expires → worker closes.  The Close action proves the worker
        // processed the event and ran the termination path.
        let close_action = tokio::time::timeout(Duration::from_secs(2), action_rx.recv())
            .await
            .expect("timeout waiting for close action")
            .expect("action stream ended before close");
        assert!(
            matches!(
                close_action,
                SessionPixAction::Close(SessionPixCloseReason::CommitmentTimeout)
            ),
            "expected Close due to commitment timeout, got {close_action:?}"
        );
    }

    /// A `Close` the supervisor did not flag itself with must still fail closed.
    ///
    /// Several handlers return [`SessionPixAction::Close`] without setting `supervisor.closed` —
    /// among them the two anti-abuse verdicts. A `dispatch` that reads only the flag sends the close
    /// and then keeps running with the gate open, which is what this pins against. The foreign
    /// pseudonym is the cheapest of those paths to reach: `on_ssa_request_sent` rejects it before it
    /// looks anything up, and returns `InvalidTransition` without touching the flag.
    #[tokio::test]
    async fn a_close_the_supervisor_did_not_flag_still_poisons_the_gate() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Consume the initial RequestSsa so the close below is unambiguous.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        let foreign = SsaId::new(HoprPseudonym::random(), SsaIndex::new(1).unwrap());
        handle
            .send_event(SessionPixEvent::SsaRequestSent(foreign))
            .await
            .expect("worker must accept the event");

        let close_action = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout waiting for close action")
            .expect("action stream ended before close");
        assert!(
            matches!(
                close_action,
                SessionPixAction::Close(SessionPixCloseReason::InvalidTransition)
            ),
            "expected Close on a foreign pseudonym, got {close_action:?}"
        );

        poll_until("the gate is poisoned by a close the supervisor did not flag", || {
            handle.gate.try_acquire_sync().is_err()
        })
        .await;
    }

    #[tokio::test]
    async fn worker_terminates_on_action_send_failure() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Drop the action receiver — worker's next send_actions will fail.
        drop(action_rx);

        // Polled on the assertion itself rather than on a proxy, because the worker drops its
        // command receiver only after poisoning the gate. The channel holds 64 and at most one send
        // can land in the window between those two, so this cannot fill it.
        let id = SsaId::new(p, SsaIndex::new(1).unwrap());
        let terminated = tokio::time::timeout(Duration::from_secs(5), async {
            while handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.is_ok() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;

        assert!(
            terminated.is_ok(),
            "the worker must exit after a failed action send, which closes the command channel"
        );
    }

    #[tokio::test]
    async fn send_event_on_disconnected_channel_returns_error() {
        let (cmd_tx, cmd_rx) = crossfire::mpsc::bounded_async::<WorkerCommand>(2);
        let gate = ServiceGate::new(1, 256);
        let handle = SessionPixSupervisorHandle {
            cmd_tx,
            gate: gate.clone(),
        };

        // Drop the receiver so the channel is disconnected.
        drop(cmd_rx);

        let id = SsaId::new(HoprPseudonym::random(), SsaIndex::new(1).unwrap());
        assert!(handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.is_err());
        assert!(
            handle
                .send_action_result(
                    SessionPixAction::RequestSsa {
                        ssa_ids: vec![id],
                        params: dims(),
                    },
                    true,
                )
                .await
                .is_err()
        );
    }

    // -----------------------------------------------------------------------
    // PD-07: Command-channel backpressure
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn command_channel_backpressures_on_full() {
        // Use a tiny channel (capacity 1) with no worker so events queue up.
        let (cmd_tx, _cmd_rx) = crossfire::mpsc::bounded_async::<WorkerCommand>(1);
        let gate = ServiceGate::new(1, 256);
        let handle = SessionPixSupervisorHandle {
            cmd_tx,
            gate: gate.clone(),
        };

        let id = SsaId::new(HoprPseudonym::random(), SsaIndex::new(1).unwrap());

        // First send should succeed immediately.
        handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.unwrap();

        // Second send should fail (channel full, no worker draining).
        let result = tokio::time::timeout(
            Duration::from_millis(50),
            handle.send_event(SessionPixEvent::SsaRequestSent(id)),
        )
        .await;

        // The send should be pending (not complete) since there's no worker
        // to drain the channel. A timeout means it correctly backpressured.
        assert!(result.is_err(), "send_event should backpressure when channel is full");
    }

    #[tokio::test]
    async fn backpressure_releases_when_channel_drained() {
        let (cmd_tx, cmd_rx) = crossfire::mpsc::bounded_async::<WorkerCommand>(1);
        let gate = ServiceGate::new(1, 256);
        let handle = SessionPixSupervisorHandle {
            cmd_tx,
            gate: gate.clone(),
        };

        let id = SsaId::new(HoprPseudonym::random(), SsaIndex::new(1).unwrap());

        // Fill the channel.
        handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.unwrap();

        // Spawn a task that will send a second event and wait.
        let parked = {
            let handle = handle.clone();
            tokio::spawn(async move { handle.send_event(SessionPixEvent::SsaRequestSent(id)).await })
        };

        // Give the parked send a moment to register.
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Drain one item from the channel — the parked send should now complete.
        let drained = cmd_rx.recv().await;
        assert!(drained.is_ok(), "expected one command in the channel");

        // The parked send should complete within a reasonable timeout.
        let result = tokio::time::timeout(Duration::from_secs(1), parked).await;
        assert!(result.is_ok(), "parked send should complete after channel is drained");
        assert!(result.unwrap().is_ok(), "send should succeed");
    }

    // -----------------------------------------------------------------------
    // PD-02: Action-channel Full handling
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn coalescible_dropped_on_full_non_coalescible_and_disconnected_are_fatal() {
        let (action_tx, _action_rx) = crossfire::mpsc::bounded_async::<SessionPixAction>(1);

        // Fill the channel so the next send hits TrySendError::Full.
        action_tx.try_send(SessionPixAction::ReleaseService).unwrap();

        // Flood with coalescible actions — silently dropped, send_actions
        // returns true (session survives).
        let progress: Vec<_> = (0..100).map(|_| SessionPixAction::ProgressNotification).collect();
        assert!(send_actions(&progress, &action_tx));

        assert!(!send_actions(&[SessionPixAction::WithholdService], &action_tx));

        // Non-coalescible action on the same full channel → returns false
        // (session must be terminated).
        assert!(!send_actions(
            &[SessionPixAction::Close(SessionPixCloseReason::CommitmentTimeout)],
            &action_tx
        ));

        // Disconnected channel — also fatal. Use a separate channel so the
        // sender is still alive but the receiver is dropped.
        let (action_tx2, action_rx2) = crossfire::mpsc::bounded_async::<SessionPixAction>(1);
        drop(action_rx2);
        assert!(!send_actions(&progress, &action_tx2));
    }
}
