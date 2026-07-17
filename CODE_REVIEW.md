# Re-review: `lukas/session-pix-supervisor`

**Review date:** 2026-07-17
**Compared:** `origin/lukas/pix` (`0fc767f904fe8587fa8d4513b78ff88147eba88a`) to `HEAD` (`63e3930afa`)
**Verdict:** **CHANGES REQUESTED**

## Executive summary

The latest fixes address the prior worker hot loop, funding watermark, notification race, and event-ordering findings. The affected crate tests and the complete downstream PIX integration suite are green on the reviewed head, including all one-, two-, and three-hop cases.

Three previous findings remain open or partial:

1. **M-06:** a full supervisor command queue still drops lifecycle/security events without closing the session or coalescing state.
2. **M-03:** cancellation cleanup is now guarded, but recovered SSA state and completed deposit-observer handles remain retained until the entire session closes.
3. **L-02:** cross-configuration validation is comprehensive, but `SessionManager::start()` only logs validation errors and continues with invalid settings.

No new high-severity implementation regression was found. Approval should wait for the remaining queue-overflow behavior and lifecycle cleanup to be made deterministic and covered by tests.

## Status of all previous findings

| ID   | Severity | Status                     | Re-review result                                                                                                                                                 |
| ---- | -------- | -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| H-01 | High     | Addressed                  | PIX egress acquires a permit before forwarding; predeposit service is capped below full recovery.                                                                |
| H-02 | High     | Addressed in code          | Invalid-share totals are cross-peer and mutex-protected. The requested two-peer interleaving regression test is still missing.                                   |
| H-03 | High     | Addressed                  | `max_served_without_progress` provides a finite post-deposit backstop, and M-05's funding watermark issue is fixed.                                              |
| H-04 | High     | Addressed in code          | `process_cmd()` now returns a termination signal and every terminal path exits `worker_loop()`. The drop-sender test does not actually observe task termination. |
| M-01 | Medium   | Addressed                  | The gate and supervisor are installed before session construction and publication.                                                                               |
| M-02 | Medium   | Addressed                  | Recovery is phase-gated, stale snapshots are ignored, and tombstones absorb late progress/fault events.                                                          |
| M-03 | Medium   | Partial                    | A `Drop` guard makes whole-session retirement cancellation-safe, but normal tombstone expiry does not retire individual SSA state.                               |
| M-04 | Medium   | Addressed                  | `SlotNotify` now uses a generation counter and covers latent notification, spurious polls, cancellation, and waker replacement.                                  |
| M-05 | Medium   | Addressed                  | Funding snapshots the served watermark and has a regression test with predeposit usage above the post-funding ceiling.                                           |
| M-06 | Medium   | Not addressed              | Capacity-64 queues still use `try_send`; `Full` drops the event and several callers only log the error.                                                          |
| L-01 | Low      | Addressed                  | Canceled notification futures unregister their wakers on `Drop`.                                                                                                 |
| L-02 | Low      | Partial                    | Checked `Duration` arithmetic and boundary tests were added, but invalid configuration is only warned about.                                                     |
| L-03 | Low      | Addressed with M-06 caveat | The queues are bounded; overflow semantics remain tracked under M-06.                                                                                            |

## Remaining findings

### M-06 — Queue overflow still drops supervisor events without failing closed

**Affected code**

- `transport/session/src/pix/worker.rs:44-75`
- `transport/session/src/manager.rs:1720-1723`
- `transport/session/src/manager.rs:2333-2353`
- `transport/session/src/manager.rs:2607-2675`

`SessionPixSupervisorHandle::send_event()` and `send_action_result()` return `Err(())` when the capacity-64 command queue is full, but they do not poison the gate or initiate centralized session closure. Several manager producers merely log that error and continue.

This can discard `SsaRequestSent`, action results, `CommitmentVerified`, `DepositConfirmed`, deposit-observer closure, recovery progress, fault totals, and terminal recovery events. A lost fault can delay enforcement; a lost valid lifecycle event can leave the supervisor in an earlier phase until it closes a healthy session on a timeout. The comment calling this behavior “fail-closed” is therefore inaccurate: returning an error is only fail-closed if every caller synchronously closes the session.

**Required fix**

- Define delivery behavior by event class: coalesce replaceable absolute snapshots, await bounded capacity where safe, or synchronously poison/close for non-coalescible events.
- Ensure every `Full`/`Disconnected` path reaches centralized session closure rather than only logging.
- Add a real capacity-exhaustion test. The new disconnected-channel test does not exercise `TrySendError::Full`.

### M-03 — Recovered SSA state is retained until session close

**Affected code**

- `transport/session/src/pix/supervisor.rs:206-208`
- `transport/session/src/manager.rs:2305-2414`
- `transport/session/src/manager.rs:2623-2681`

The new `SsaRetirementGuard` fixes the cancellation hole: aborting the action driver drops the guard and retires every tracked SSA. Normal recovery remains incomplete, however. Once a tombstone expires, `handle_deadline()` removes it from supervisor state without emitting an action that calls `retire_ssa()`. The retirement guard keeps every requested ID until the whole session closes. Completed `PixDepositObserver` abort handles are likewise left in the session map.

Long-lived, high-volume PIX sessions therefore accumulate reconstructor entries until their independent cache TTLs expire and retain completed observer handles for the session lifetime.

**Required fix**

- Emit an explicit per-SSA retirement action when the tombstone window expires and remove that ID from the guard after successful retirement.
- Remove completed deposit-observer handles, or use ownership that does not retain completed handles.
- Add a multi-cycle test that verifies old reconstructor and observer state is released while the session remains active.

### L-02 — Invalid supervisor configuration is accepted

**Affected code**

- `transport/session/src/manager.rs:933-955`
- `transport/session/src/pix/mod.rs:207-286`

`validate_pix_supervision()` now uses checked `Duration` arithmetic and validates the required timing relationships. `SessionManager::start()` invokes it with the effective reconstructor configuration, but handles failure with `warn!` and continues initialization.

Direct `SessionManager` users can therefore start PIX supervision with zero durations, inconsistent acknowledgement/tombstone windows, or counter TTLs shorter than the supervision horizon. These are rejected by the validator but still become active runtime settings.

**Required fix**

- Return the validation error from `start()` before mutating its `OnceLock` state.
- Add a component-boundary test proving invalid PIX configuration fails startup and does not leave the manager partially started.

## Test-quality gaps

- `worker_terminates_when_all_command_senders_dropped` and `worker_terminates_when_supervisor_closes_after_event` only sleep; because `spawn_supervisor_worker()` exposes no task handle or completion signal, they do not assert that the worker exited.
- `send_event_on_disconnected_channel_returns_error` covers disconnection, not queue saturation.
- `invalid_shares_reports_absolute_totals_per_ssa` uses one peer; no retained test proves that interleaved invalid shares from two peers produce the cross-peer aggregate required by H-02.
- `enforce_pix_rejects_non_pix_session` accepts a generic 15-second connection timeout as success, so unrelated routing/start-protocol hangs can satisfy the policy test.
- The affected-crate test build emits one branch-local `unused_mut` warning at `transport/session/src/pix/worker.rs:450`.

## Verification performed

### Passed

- `cargo check --workspace`
- `cargo nextest run -p hopr-protocol-pix -p hopr-transport-session -p hopr-transport`
  - 391 passed, 0 failed across 16 binaries.
- `cargo nextest run -p hopr-lib --features testing --test transport_session_pix -j 1`
  - 6 passed, 0 failed in 387.393 seconds.
  - Includes one-, two-, and three-hop PIX recovery, deposit timeout, enforced-PIX rejection, and recovery hard deadline.
- Earlier focused runs also passed the protocol, supervisor, manager, session integration, and isolated downstream cases.

### Repository-wide feature note

`cargo check --workspace --all-targets --all-features` is not a supported aggregate configuration: it enables mutually exclusive `allocator-jemalloc`/`allocator-mimalloc` and crypto-suite features, causing pre-existing duplicate allocator/import errors outside this branch diff. The supported default workspace check is green.

## Required before approval

1. Make command-queue overflow explicitly fail-closed or safely coalescing, and prove it with a capacity-exhaustion test.
2. Retire recovered SSA state at tombstone expiry and release completed observer handles during long-lived sessions.
3. Reject invalid PIX supervision configuration at the `SessionManager` boundary without partially starting the manager.
4. Strengthen the termination, cross-peer aggregation, and policy-rejection tests, and remove the branch-local warning.
