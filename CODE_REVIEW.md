# Re-review: `lukas/session-pix-supervisor`

**Review date:** 2026-07-17
**Compared:** `origin/lukas/pix` (`0fc767f904fe8587fa8d4513b78ff88147eba88a`) to `HEAD` (`ca42ab145d556b45b11afac3b1761999cb2e4b2c`)
**Diff size:** 23 files, 5,364 insertions, 517 deletions
**Verdict:** **CHANGES REQUESTED**

## Executive summary

The branch addresses the main implementation defects behind H-01, H-02, H-03, M-01, L-01, and the unbounded-channel part of L-03. M-02, M-03, and L-02 are only partially addressed, while M-04 remains and was directly reproduced.

This re-review also found one new high-severity regression and two new medium-severity issues:

1. **H-04:** a closed supervisor command channel makes the worker spin forever.
2. **M-05:** funding can fail to release service because predeposit traffic is charged against the post-funding no-progress ceiling.
3. **M-06:** bounded supervisor queues now drop lifecycle/security events without a fail-closed fallback.

The affected unit and session integration suites pass, but the downstream multi-hop PIX suite is not green or deterministic: the one-hop case passed, the two-hop case timed out once and passed on retry, and the three-hop case timed out without obtaining a usable route. The branch should not be approved until the new worker regression and remaining lifecycle/concurrency issues are fixed and covered by deterministic tests.

## Status of the previous findings

| ID   | Severity | Status                    | Re-review result                                                                                                                         |
| ---- | -------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| H-01 | High     | Addressed                 | Both PIX egress adapters now acquire the gate before forwarding, and a poisoned gate parks instead of forwarding.                        |
| H-02 | High     | Addressed in code         | The reconstructor emits a mutex-protected cross-peer aggregate. The requested two-peer interleaving regression test is still missing.    |
| H-03 | High     | Addressed with caveat     | A finite `max_served_without_progress` ceiling now exists. See M-02 and M-05 for epoch/reset weaknesses.                                 |
| M-01 | Medium   | Addressed                 | The supervisor and gate are installed before session construction and application publication.                                           |
| M-02 | Medium   | Partial                   | Lower snapshots are ignored, but terminal/tombstone ordering remains unsafe.                                                             |
| M-03 | Medium   | Partial                   | The initial SSA is tracked and supervisor-driven close retires it, but cancellation and normal tombstone expiry still bypass retirement. |
| M-04 | Medium   | Not addressed             | Both replacement notification futures can still miss a wake; they can also complete on a second poll without any notification.           |
| L-01 | Low      | Addressed                 | Canceled waiters unregister on `Drop`; notification correctness remains tracked under M-04.                                              |
| L-02 | Low      | Partial                   | More invariants and a duration cap were added, but validation remains bypassable and uses unsafe/lossy horizon arithmetic.               |
| L-03 | Low      | Addressed with regression | The queues are bounded, but overflow handling introduced M-06.                                                                           |

## New findings

### H-04 — Closed supervisor workers spin forever

**Affected code**

- `transport/session/src/pix/worker.rs:146-190`
- `transport/session/src/pix/worker.rs:195-248`
- `transport/session/src/manager.rs:2320-2337`

The runtime-conversion refactor moved command processing into `process_cmd()`, but that helper returns `()` and cannot terminate `worker_loop()`.

When all command senders are dropped, `cmd_rx.recv()` immediately returns `Err`. `worker_loop()` converts that to `None`, `process_cmd()` sends a close action, poisons the gate, and returns only to the outer loop. The next receive is immediately ready with the same disconnection, so the task loops continuously. Because each receive is immediately ready, the loop need not yield to cooperative cancellation; the review reproducer could not stop it with `JoinHandle::abort()`. The same control-flow defect occurs when `process_cmd()` observes a closed supervisor or an action-send failure.

The implementation immediately before commit `f559162dbf44bd272ff94071ee94a0d0ab14278a` returned from the outer worker on these paths. The regression is therefore introduced by this branch's runtime conversion.

**Impact**

- Every normally closed or rolled-back PIX session can leave a permanently runnable task.
- A remote client can trigger closure through ordinary supervisor deadlines and accumulate CPU-consuming workers.
- Setup failure after spawning the worker can leak the worker even before the session is published.
- The non-yielding loop can prevent ordinary async task cancellation and runtime shutdown.

**Required fix**

- Make `process_cmd()` return `ControlFlow`, `bool`, or a result that causes `worker_loop()` to break/return on every terminal path.
- Retain an abort/join handle in the session owner as a second cancellation boundary, but do not rely on cancellation to fix the non-yielding loop.
- Add a deterministic test that drops all command handles and asserts the worker task terminates; also cover closed/full action queues and supervisor-generated close.

### M-05 — Funding may not release a predeposit-blocked writer

**Affected code**

- `transport/session/src/pix/gate.rs:45-105`
- `transport/session/src/pix/gate.rs:116-123`
- `transport/session/src/pix/mod.rs:89-96`

`release_service()` sets `funded = true` and wakes waiters, but it does not snapshot `served_total` into `served_at_last_progress`. The funded path therefore counts every predeposit packet against `max_served_without_progress`.

With the defaults, `max_predeposit_packets` is 1,024 while `max_served_without_progress` is 256. If 256 or more packets were served before deposit confirmation, the waiter wakes on `ReleaseService`, enters the funded path, and parks again immediately. A valid deposit therefore does not actually release service until an independent recovery-progress event arrives.

**Impact**

- Validly funded sessions can remain stalled.
- The behavior depends on the interaction of two independently configurable limits and is not validated or documented.
- Existing tests cover release with no prior service and release after an explicit progress reset, but not release after consuming more than the funded ceiling predeposit.

**Required fix**

- If the ceiling is intended to be post-funding, set the progress watermark when funding is confirmed.
- If predeposit traffic is intentionally included, validate/document the relationship between both limits and make `ReleaseService` semantics explicit.
- Add a test that consumes more than `max_served_without_progress` under predeposit credit, confirms funding, and asserts the intended behavior.

### M-06 — Queue overflow drops supervisor events without failing closed

**Affected code**

- `transport/session/src/pix/worker.rs:30-64`
- `transport/session/src/pix/worker.rs:242-248`
- `transport/session/src/manager.rs:2297-2304`
- `transport/session/src/manager.rs:2545-2605`
- `transport/hopr/src/lib.rs:1281-1323`

The old unbounded channels were replaced with capacity-64 `crossfire` queues, but all producers use `try_send()`. Queue-full and disconnected errors are either ignored or only logged.

This can discard `CommitmentVerified`, `DepositConfirmed`, `RecoveryProgress`, unverifiable-share totals, terminal recovery events, and action results. A dropped fault/action event can delay enforcement; a dropped valid lifecycle event can produce a false timeout. Poisoning only inside the worker does not help when the command never reaches the worker.

**Required fix**

- Define explicit behavior per event class: coalesce monotonic snapshots, await bounded capacity where safe, or synchronously poison/close the session when a non-coalescible security event cannot be delivered.
- Do not ignore action-result or deposit-observer send failures.
- Add queue-saturation tests proving that overflow is bounded and fail-closed without losing the latest absolute counters.

## Remaining findings

### M-02 — Event ordering remains only partially hardened

**What improved**

- Lower absolute progress and fault snapshots are treated as stale instead of closing the session.
- `AlmostRecovered` no longer mutates state while awaiting commitment.

**What remains**

- Relay acknowledgement batches are still processed with `for_each_concurrent()` and cloned sinks, so inter-batch order is not guaranteed.
- `on_recovered()` accepts completion from any non-tombstone phase. If recovery wins the race against commitment/deposit notification, the supervisor can skip the normal release transition and tombstone the SSA.
- Recovered tombstones still process late `RecoveryProgress` and unverifiable-share events. A late old-SSA progress event can reset the session-wide gate watermark during a later epoch.

Require phase-valid terminal events and absorb late tombstone progress/faults. Add reversed/interleaved event tests covering commitment, deposit, progress, almost-recovered, recovered, and tombstone delivery.

### M-03 — Reconstructor cleanup is still cancellation-unsafe

**What improved**

- `tracked_ssas` is seeded with the initial SSA.
- Supervisor-driven `Close` and a clean action-channel shutdown retire tracked SSAs.

**What remains**

- `close_session()` only poisons the gate and aborts session tasks. Explicit close, idle eviction, empty read, and setup rollback can abort the action driver before its post-loop cleanup executes.
- Normal tombstone expiry removes supervisor state without calling `retire_ssa()`.
- Deposit-observer abort handles remain indexed by completed SSA IDs.

Move authoritative SSA ownership and retirement to a cancellation-safe object reached from centralized session closure, and retire completed SSAs when their tombstones expire.

### M-04 — Notification race remains

**Affected code**

- `transport/session/src/manager.rs:152-225`
- `transport/session/src/pix/notify.rs:29-92`
- `transport/session/src/pix/gate.rs:75-105`

Creating `notified()` does not register a waiter; registration happens only on the future's first poll. A state change between the final condition check and first poll can therefore drain no waker, after which the future parks indefinitely.

The replacement future also returns `Ready` whenever `registered == true`; it does not verify that `notify_waiters()` removed/woke its registration. Futures may be polled spuriously, so a second poll can complete without any notification.

Use a generation counter/event-listener style primitive that atomically observes a notification generation while registering. Add deterministic tests for notification before first poll, cancellation, waker replacement, spurious repoll, and release/poison racing with registration.

### L-02 — Configuration validation is still incomplete

**What improved**

- The tombstone/ack-window relationship is now checked.
- Zero values and a 24-hour upper bound are checked.
- `HoprTransport::new()` invokes cross-validation.

**What remains**

- Direct `SessionManager::new()` users bypass validation entirely.
- `max_recovery_time.as_secs() + tombstone_retention_window.as_secs()` truncates subsecond precision and performs unchecked `u64` addition before the 24-hour cap is applied.
- Validation uses a default reconstructor at the top-level call site rather than being tied to the component that owns both effective configurations.

Use checked `Duration` addition/comparison before integer conversion and validate at the session-manager/supervisor construction boundary. Add subsecond-boundary and extreme-duration tests.

## Verification performed

### Passed

- `git diff --check origin/lukas/pix...HEAD`
- `cargo nextest run --lib -p hopr-protocol-pix -p hopr-transport-session -p hopr-transport`
  - 308 passed, 0 failed.
- `cargo nextest run -p hopr-transport-session --test pix -j 1`
  - 3 passed, 0 failed.
- Isolated retry: `cargo nextest run -p hopr-lib --features testing --test transport_session_pix -j 1 -E 'test(=capture_n_hop_pix_session::case_2)'`
  - 1 passed in 63.462 seconds.

### Not green / inconclusive

- `cargo nextest run -p hopr-lib --features testing --test transport_session_pix -j 1`
  - `case_1` passed in 52.449 seconds.
  - `case_2` timed out after 177.869 seconds.
- Isolated `case_3` timed out after 178.082 seconds while repeatedly reporting that no three-hop path could be found in the channel graph.

The two-hop pass-on-retry and three-hop route-availability failure make this suite timing-sensitive. These runs do not prove that the branch caused the timeout, but they also do not provide a clean downstream verification result. The test uses a fixed propagation sleep instead of waiting for the required route to become observable.

The previous report's downstream command omitted `--features testing`; without it, the test target does not compile because `hopr_lib::testing` is feature-gated.

### Deterministic finding reproducers

Three review-only tests were temporarily added, executed, and removed:

- A waiter created before `notify_waiters()` returned `Pending` when first polled afterward.
- A registered waiter returned `Ready` on a second poll without any notification.
- Dropping every supervisor command sender did not terminate the worker within 100 ms. The hot loop also prevented task abortion/runtime shutdown, so the test process had to be killed by the 300-second command timeout.

These probes directly reproduce H-04 and M-04; they are not retained as failing tests in the branch.

### Warnings

The library run emits five `unused_mut` warnings in `transport/session/src/pix/worker.rs` tests and one `dead_code` warning for `ServiceGate::remaining_budget`. The session integration test also uses the deprecated `UnverifiableShare` variant. New branch code should be warning-clean.

## Required before approval

1. Fix H-04 and prove worker termination on every terminal/disconnection path.
2. Replace the notification primitive with one that cannot miss or invent wakes.
3. Make queue overflow explicitly fail-closed/coalescing instead of dropping lifecycle events.
4. Centralize cancellation-safe SSA retirement and cover every closure path plus tombstone expiry.
5. Resolve/document the funding watermark semantics and finish component-boundary duration validation.
6. Add the missing deterministic multi-peer, ordering, queue-saturation, and retained-writer tests.
7. Make the downstream multi-hop PIX verification deterministic and green, and remove branch-introduced warnings.
