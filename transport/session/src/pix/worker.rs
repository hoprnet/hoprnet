//! Per-session actor for the [`SessionPixSupervisor`].
//!
//! The worker serializes lifecycle events through the deterministic core,
//! manages the deadline timer, and drives actions through an external
//! action driver.

use std::{sync::Arc, time::Instant};

use tokio::{select, sync::mpsc, time::sleep_until};

use super::{
    SessionPixAction, SessionPixCloseReason, SessionPixEvent, SsaDimensions, SupervisorConfig, gate::ServiceGate,
    supervisor::SessionPixSupervisor,
};

// ---------------------------------------------------------------------------
// SessionPixSupervisorHandle
// ---------------------------------------------------------------------------

/// Cloneable handle to a running [`SessionPixWorker`].
#[derive(Clone)]
pub(crate) struct SessionPixSupervisorHandle {
    cmd_tx: mpsc::UnboundedSender<WorkerCommand>,
    pub(crate) gate: Arc<ServiceGate>,
}

impl SessionPixSupervisorHandle {
    /// Send a PIX event to the supervisor.
    ///
    /// Returns `Err` if the worker is no longer running (fail-closed).
    pub fn send_event(&self, ev: SessionPixEvent) -> Result<(), ()> {
        self.cmd_tx.send(WorkerCommand::Event(ev)).map_err(|_| ())
    }

    /// Send an action result feedback to the supervisor.
    pub fn send_action_result(&self, action: SessionPixAction, ok: bool) -> Result<(), ()> {
        self.cmd_tx
            .send(WorkerCommand::ActionResult { action, ok })
            .map_err(|_| ())
    }
}

// ---------------------------------------------------------------------------
// WorkerCommand
// ---------------------------------------------------------------------------

pub(crate) enum WorkerCommand {
    Event(SessionPixEvent),
    ActionResult { action: SessionPixAction, ok: bool },
}

// ---------------------------------------------------------------------------
// spawn_supervisor_worker
// ---------------------------------------------------------------------------

/// Spawn a supervisor worker and return its handle and action driver receiver.
pub(crate) fn spawn_supervisor_worker(
    cfg: SupervisorConfig,
    dims: SsaDimensions,
    pseudonym: hopr_api::types::internal::prelude::HoprPseudonym,
    now: Instant,
) -> (SessionPixSupervisorHandle, mpsc::UnboundedReceiver<SessionPixAction>) {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<WorkerCommand>();
    let (action_tx, action_rx) = mpsc::unbounded_channel::<SessionPixAction>();

    let predeposit_budget = std::cmp::min(
        dims.target_useful_shares().saturating_sub(1),
        cfg.max_predeposit_packets,
    );
    let gate = ServiceGate::new(predeposit_budget);

    let handle = SessionPixSupervisorHandle {
        cmd_tx,
        gate: gate.clone(),
    };

    let (supervisor, initial_actions) = SessionPixSupervisor::new(cfg, dims, pseudonym, now);

    tokio::spawn(worker_loop(supervisor, cmd_rx, action_tx, gate, initial_actions));

    (handle, action_rx)
}

// ---------------------------------------------------------------------------
// Worker loop
// ---------------------------------------------------------------------------

async fn worker_loop(
    mut supervisor: SessionPixSupervisor,
    mut cmd_rx: mpsc::UnboundedReceiver<WorkerCommand>,
    action_tx: mpsc::UnboundedSender<SessionPixAction>,
    gate: Arc<ServiceGate>,
    initial_actions: Vec<SessionPixAction>,
) {
    // Emit initial actions.
    if !send_actions(&initial_actions, &action_tx) {
        supervisor = SessionPixSupervisor::new(
            supervisor.cfg.clone(),
            supervisor.dims,
            supervisor.pseudonym,
            Instant::now(),
        )
        .0;
        let close_actions = supervisor.handle_deadline(Instant::now(), gate.served_total());
        send_actions(&close_actions, &action_tx);
        gate.poison();
        return;
    }

    loop {
        let deadline = supervisor.next_deadline();

        let cmd = if let Some(dl) = deadline {
            let now = Instant::now();
            if now >= dl {
                let actions = supervisor.handle_deadline(now, gate.served_total());
                if supervisor.closed {
                    send_actions(&actions, &action_tx);
                    gate.poison();
                    return;
                }
                send_actions(&actions, &action_tx);
                continue;
            }

            let sleep = sleep_until(dl.into());
            select! {
                biased;

                cmd = cmd_rx.recv() => { cmd }
                _ = sleep => {
                    let now = Instant::now();
                    let actions = supervisor.handle_deadline(now, gate.served_total());
                    if supervisor.closed {
                        send_actions(&actions, &action_tx);
                        gate.poison();
                        return;
                    }
                    send_actions(&actions, &action_tx);
                    continue;
                }
            }
        } else {
            cmd_rx.recv().await
        };

        let cmd = match cmd {
            Some(c) => c,
            None => {
                // All senders dropped — close.
                let actions = vec![SessionPixAction::Close(SessionPixCloseReason::SupervisorUnavailable)];
                send_actions(&actions, &action_tx);
                gate.poison();
                return;
            }
        };

        match cmd {
            WorkerCommand::Event(ev) => {
                let now = Instant::now();
                let actions = supervisor.handle_event(&ev, now, gate.served_total());
                if supervisor.closed {
                    send_actions(&actions, &action_tx);
                    gate.poison();
                    return;
                }
                send_actions(&actions, &action_tx);
            }
            WorkerCommand::ActionResult { action, ok } => {
                let now = Instant::now();
                let actions = supervisor.action_result(&action, ok, now);
                if supervisor.closed {
                    send_actions(&actions, &action_tx);
                    gate.poison();
                    return;
                }
                send_actions(&actions, &action_tx);
            }
        }
    }
}

/// Non-blocking forward of actions to the driver.
fn send_actions(actions: &[SessionPixAction], action_tx: &mpsc::UnboundedSender<SessionPixAction>) -> bool {
    for action in actions {
        if action_tx.send(action.clone()).is_err() {
            tracing::warn!(?action, "action driver disconnected");
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[cfg(feature = "runtime-tokio")]
mod tests {
    use std::time::{Duration, Instant};

    use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
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
            tombstone_retention_window: Duration::from_secs(30),
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
        let (_handle, mut action_rx) =
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
        let (handle, mut action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

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
        let (handle, mut action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

        // Consume initial RequestSsa.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        // Send SsaRequestSent via the handle — the worker should process it
        // and produce no further actions (event is idempotent).
        let id = SsaId::new(p, SsaIndex::new(1).unwrap());
        handle.send_event(SessionPixEvent::SsaRequestSent(id)).unwrap();

        // Give the worker time to process the event.
        tokio::time::sleep(Duration::from_millis(50)).await;

        // No extra actions should appear (idempotent event).
        let maybe_action = tokio::time::timeout(Duration::from_millis(50), action_rx.recv()).await;
        assert!(maybe_action.is_err(), "expected no extra actions");
    }

    #[tokio::test]
    async fn action_result_feedback_processed() {
        let p = HoprPseudonym::random();
        let (handle, mut action_rx) = spawn_supervisor_worker(default_cfg(), dims(), p, Instant::now());

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
        let (handle, mut action_rx) = spawn_supervisor_worker(cfg, dims(), p, Instant::now());

        // Consume initial RequestSsa.
        let _initial = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("timeout")
            .expect("action stream ended");

        // Tell the worker the request was sent so the commitment deadline starts.
        let id = SsaId::new(p, SsaIndex::new(1).unwrap());
        handle.send_event(SessionPixEvent::SsaRequestSent(id)).unwrap();

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
}
