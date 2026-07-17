//! Per-session actor for the [`SessionPixSupervisor`].
//!
//! The worker serializes lifecycle events through the deterministic core,
//! manages the deadline timer, and drives actions through an external
//! action driver.
//!
//! Runtime-agnostic: uses crossfire channels and the runtime prelude from
//! `hopr_utils` so no direct tokio dependency (tests use tokio freely).

use std::{sync::Arc, time::Instant};

use crossfire::{AsyncRx, MAsyncTx, SendError, TrySendError, mpsc::Array};
use futures_time::future::FutureExt as TimeExt;
use hopr_utils::runtime::prelude::spawn;

use super::{
    SessionPixAction, SessionPixCloseReason, SessionPixEvent, SsaDimensions, SupervisorConfig, gate::ServiceGate,
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

/// Cloneable handle to a running [`SessionPixWorker`].
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
    dims: SsaDimensions,
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
    // Emit initial actions.
    if !send_actions(&initial_actions, &action_tx) {
        gate.poison();
        return;
    }

    loop {
        let deadline = supervisor.next_deadline();

        if let Some(dl) = deadline {
            let now = Instant::now();
            if now >= dl {
                let actions = supervisor.handle_deadline(now, gate.served_total());
                if supervisor.closed {
                    send_actions(&actions, &action_tx);
                    gate.poison();
                    return;
                }
                if !send_actions(&actions, &action_tx) {
                    gate.poison();
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
                    if supervisor.closed {
                        send_actions(&actions, &action_tx);
                        gate.poison();
                        return;
                    }
                    if !send_actions(&actions, &action_tx) {
                        gate.poison();
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
            // All senders dropped — close.
            let actions = vec![SessionPixAction::Close(SessionPixCloseReason::SupervisorUnavailable)];
            send_actions(&actions, action_tx);
            gate.poison();
            return false;
        }
    };

    match cmd {
        WorkerCommand::Event(ev) => {
            let now = Instant::now();
            let actions = supervisor.handle_event(&ev, now, gate.served_total());
            if supervisor.closed {
                send_actions(&actions, action_tx);
                gate.poison();
                return false;
            }
            if !send_actions(&actions, action_tx) {
                gate.poison();
                return false;
            }
        }
        WorkerCommand::ActionResult { action, ok } => {
            let now = Instant::now();
            let actions = supervisor.action_result(&action, ok, now);
            if supervisor.closed {
                send_actions(&actions, action_tx);
                gate.poison();
                return false;
            }
            if !send_actions(&actions, action_tx) {
                gate.poison();
                return false;
            }
        }
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
/// - **Non-coalescible** actions (`Close`, `RequestSsa`, `RetireSsa`) are treated as fatal.  If these cannot be
///   delivered the channel is genuinely wedged.
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

    use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
    use hopr_api::HoprBalance;
    use hopr_protocol_pix::{SsaId, SsaIndex};

    use super::*;

    fn default_cfg() -> SupervisorConfig {
        SupervisorConfig {
            max_ssa_delivery_time: Duration::from_secs(20),
            max_deposit_wait: Duration::from_secs(60),
            max_recovery_idle: Duration::from_secs(10),
            max_recovery_time: Duration::from_secs(3600),
            max_unverifiable_shares_per_ssa: 3,
            max_unverifiable_shares_per_session: 10,
            max_predeposit_packets: 1024,
            max_served_without_progress: 256,
            tombstone_retention_window: Duration::from_secs(30),
            min_deposit: HoprBalance::new_base(0),
        }
    }

    fn dims() -> SsaDimensions {
        SsaDimensions {
            polys: 10,
            threshold: 5,
        }
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
            SessionPixAction::RequestSsa { ssa_id, .. } => {
                assert_eq!(ssa_id.ssa_index(), SsaIndex::new(1).unwrap());
            }
            other => panic!("expected RequestSsa, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dropped_action_driver_fails_closed_and_poisons_gate() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Drop the action receiver — worker should detect and close.
        drop(action_rx);

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = handle.gate.acquire().await;
        assert!(result.is_err(), "gate should be poisoned after driver dropped");
    }

    #[tokio::test]
    async fn release_service_action_flips_gate() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Consume initial RequestSsa.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        assert!(!handle.gate.funded());

        handle.gate.acquire().await.unwrap();
        handle.gate.release_service();
        assert!(handle.gate.funded());
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
                    ssa_id: SsaId::new(p, SsaIndex::new(1).unwrap()),
                    polys: 10,
                    threshold: 5,
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

        // Give the worker time to process the disconnect and poison the gate.
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(
            gate.try_acquire_sync().is_err(),
            "gate should be poisoned after all senders dropped"
        );
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

    #[tokio::test]
    async fn worker_terminates_on_action_send_failure() {
        let p = HoprPseudonym::random();
        let (handle, action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Drop the action receiver — worker's next send_actions will fail.
        drop(action_rx);

        // Give the worker time to detect the failure and exit.
        tokio::time::sleep(Duration::from_millis(100)).await;

        // send_event should fail because the worker already exited.
        let id = SsaId::new(p, SsaIndex::new(1).unwrap());
        assert!(handle.send_event(SessionPixEvent::SsaRequestSent(id)).await.is_err());
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
                        ssa_id: id,
                        polys: 10,
                        threshold: 5,
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
}
