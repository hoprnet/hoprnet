# Pre-deployment code review: `lukas/session-pix-supervisor`

**Review date:** 2026-07-17
**Compared:** `origin/lukas/pix` (`0fc767f904fe8587fa8d4513b78ff88147eba88a`) → `HEAD` (`4b598c20cd`)
**Scope:** ~6,600 added lines — `transport/session/src/pix/{supervisor,gate,worker,notify,mod}.rs`, `transport/session/src/manager.rs`, `protocols/pix/src/reconstructor/*`, `transport/hopr/src/lib.rs`, `protocols/hopr/src/codec/mod.rs`, integration tests.
**Verdict:** **CHANGES REQUESTED** — no critical or high-severity defect found, but one availability regression introduced by the M-06 fix (PD-02), a config-validation gap that can silently disable fault enforcement (PD-03), a spec-vs-code contradiction with misleading tests (PD-04), and a flaky integration suite (PD-05) should be resolved before deployment.

---

## Executive summary

The three findings left open by the previous review round (M-06, M-03, L-02) are now **genuinely fixed in code**:

- **M-06** — the supervisor command channel switched from `try_send` to awaited `send()`. Event overflow-drop is impossible by construction, and the topology is deadlock-free: the worker never blocks emitting actions (it uses non-blocking `try_send` toward the action channel), so awaited producer sends always drain. Disconnected sends reach centralized closure via the action driver's `recv() == Err` fallback.
- **M-03** — `RetireSsa` is emitted on tombstone expiry, `next_deadline()` includes `tombstone_until` so the worker actually wakes to emit it, and the action driver retires the reconstructor state, shrinks the retirement guard, and removes the deposit-observer abort handle. Per-SSA state is bounded mid-session (≤ 2 live + 1 tombstone).
- **L-02** — `SessionManager::start()` now propagates `validate_pix_supervision` errors instead of warn-and-continue.

However, the fixes introduced or exposed new issues: the M-06 rework makes transient action-channel fullness **fatal** to a healthy session (PD-02), the L-02 validation misses the deposit-wait leg of the supervision horizon (PD-03) and leaves partial `OnceLock` state on error (PD-09), and the promised M-06 capacity test was made obsolete by the design change but no backpressure test replaced it (PD-07).

Separately, the review found that **SURB keep-alive egress bypasses the PIX service gate** (PD-01) — the one egress path not covered by the H-01 fix. After analysis of the value flow this was assessed as an accepted, bounded exemption rather than a defect (keep-alives carry no client-usable data, can't advance recovery, and cost the Exit at most ~1 win-prob-scaled ticket/second per session for the pre-close window), and downgraded to Low with a documentation/test action. The review also found that the "decreasing absolute counter closes the session" spec invariant is silently not implemented, with two tests asserting the opposite of their names (PD-04).

All prior high findings (H-01 permit-before-forward, H-02 cross-peer fault aggregation, H-03 post-deposit backstop, H-04 worker termination) were re-verified as fixed. Unit tests: 325/325 green. The `transport_session_pix` integration suite is green per-test but flaky when run as a full suite (PD-05).

---

## Status of previous-round findings

| ID        | Previous status   | This review                                                                                                                                                                                                  |
| --------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| H-01      | Addressed         | **Confirmed** for both data-egress branches. Keep-alive egress is un-gated; assessed as an acceptable bounded exemption to be documented, see PD-01.                                                         |
| H-02      | Addressed in code | **Confirmed in code** (cross-peer `invalid_total`, mutex-protected, absolute totals forwarded). Cross-peer regression test still missing at the reconstructor layer, see PD-06.                              |
| H-03      | Addressed         | **Confirmed** (`max_served_without_progress` backstop; funding watermark snapshot correct).                                                                                                                  |
| H-04      | Addressed in code | **Confirmed**, incl. tests: every terminal path in `worker_loop`/`process_cmd` poisons the gate and returns; termination tests now carry real assertions.                                                    |
| M-01      | Addressed         | **Confirmed** — gate and supervisor installed in the slot before `HoprSession::new` and before publication.                                                                                                  |
| M-03      | Partial           | **Fixed** — `RetireSsa` on tombstone expiry, driver retires reconstructor + observer handle. Residual polish: PD-13, PD-14.                                                                                  |
| M-04/L-01 | Addressed         | **Confirmed** — no lost-wakeup/ABA found in `SlotNotify`; register-then-double-check plus mutex closes the missed-wakeup window; drop-unregister works.                                                      |
| M-06      | Not addressed     | **Fixed by design change** (awaited send, no drop, no deadlock). Follow-ups: PD-02 (action channel), PD-07 (missing backpressure test).                                                                      |
| L-02      | Partial           | **Mostly fixed** (`start()` propagates the error; checked arithmetic and boundary tests present). Follow-ups: PD-03 (horizon gap), PD-09 (partial state on error), PD-10 (missing floor vs. observer delay). |

Fault-close reason mapping (`TooManyUnverifiableShares` → `PixFailure`, never `Eviction`), drain-before-close of the deposit observer, double-close protection on `active_sessions`, and retirement-guard cancellation safety were all re-verified as correct.

---

## Findings

| ID    | Severity                     | Area                     | Title                                                                                                                     |
| ----- | ---------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| PD-01 | Low (downgraded from Medium) | manager.rs               | Keep-alive egress bypasses the PIX service gate — accepted bounded exemption; document and pin with a test                |
| PD-02 | Medium                       | worker.rs                | Action-channel `Full` is treated as fatal — spurious closure of healthy sessions under load                               |
| PD-03 | Medium                       | pix/mod.rs               | Supervision-horizon validation omits `max_deposit_wait` — counter can TTL-expire mid-recovery                             |
| PD-04 | Medium                       | supervisor.rs            | Decreasing-counter regressions are silently ignored, contradicting the spec; two tests assert the opposite of their names |
| PD-05 | Medium                       | tests                    | `transport_session_pix` integration suite is flaky under full-suite runs                                                  |
| PD-06 | Medium (test)                | protocols/pix            | H-02 cross-peer fault aggregation still untested at the layer that implements it                                          |
| PD-07 | Medium (test)                | worker.rs                | Command-channel backpressure (M-06 fix) unproven by any test                                                              |
| PD-08 | Medium (test)                | hopr-lib tests           | `enforce_pix_rejects_non_pix_session` still accepts a generic timeout as success                                          |
| PD-09 | Low                          | manager.rs               | `start()` populates `msg_sender` before validation — failed start becomes permanently `AlreadyStarted`                    |
| PD-10 | Low                          | manager.rs / pix/mod.rs  | `max_deposit_wait` below the observer's fixed 100 ms delay guarantees spurious closure                                    |
| PD-11 | Low                          | reconstructor            | Re-running the commitment flow after builder TTI eviction clobbers `ssa_counters` back to zero                            |
| PD-12 | Low                          | supervisor.rs            | Empty-SSA session close reports misleading `InvalidTransition`                                                            |
| PD-13 | Low                          | supervisor.rs            | `RetireSsa` can be emitted after `Close` in the same action batch                                                         |
| PD-14 | Low                          | supervisor.rs            | Non-recovered SSAs closed mid-session are never retired from the reconstructor                                            |
| PD-15 | Low                          | worker.rs                | Dead/ineffective recovery code on initial-action send failure                                                             |
| PD-16 | Low (test)                   | reconstructor/supervisor | Fault-counter persistence across builder eviction and SSA rotation is unverified                                          |
| PD-17 | Info                         | gate.rs                  | Poison is "no new service after poison observed", not "zero service after `poison()` returns"                             |
| PD-18 | Info                         | manager.rs               | Exit accepts any nonzero deposit (`expected_deposit: None`) — confirm against spec                                        |

---

### PD-01 — Low (downgraded from Medium) — Keep-alive egress bypasses the PIX gate: accepted bounded exemption, needs documentation + pinning test

**Location:** `transport/session/src/manager.rs:2189-2199` (keep-alive sink), vs. gated data paths at `manager.rs:2110-2137` and `manager.rs:2226-2240`.

**Observation.** Both data-egress branches wrap `msg_sender` with a closure that calls `pix_egress_gate.acquire().await` and parks forever on poison. The SURB-level keep-alive stream (`spawn_keep_alive_stream`) wraps `msg_sender.clone().with(...)` with only SURB accounting — no `acquire()`, no poison check. It is the one egress path outside the gate, so it formally violates the H-01 invariant "PIX egress acquires a permit before forwarding": an unfunded session keeps emitting keep-alives (≤ 1/s, `MIN_SURB_BUFFER_NOTIFICATION_PERIOD` floor) that are not counted against the predeposit budget or the `max_served_without_progress` ceiling, and `poison()` does not stop them (only the teardown abort handle does).

**Why this was downgraded.** Analysis of the value flow shows the bypass is not exploitable for service theft and its cost to the Exit is tightly bounded:

- Keep-alive payloads are SURB-level notifications — no client-usable data, so a client gains nothing from them.
- They cannot advance SSA recovery or deposit redemption: the client alone controls share release in exchange for useful service, and an Exit "spending" SURBs on keep-alives only starves itself of shares and reputation.
- The Exit-side cost per unfunded session is ≤ 1 win-prob-scaled ticket/second on the Exit's own return-path channel for at most `max_ssa_delivery_time + max_deposit_wait` (~80 s at defaults, ~80 packets), capped globally by `maximum_sessions`. This was assessed as economically negligible.

**Required action (documentation + test, not gating).**

1. Amend the H-01/spec invariant to "all _data_ egress acquires a permit; control-plane keep-alives are exempt", and add a comment at the spawn site (`manager.rs:2189`) stating the exemption and its bound.
2. Note where relevant (metrics/served-counter docs) that gate `served` under-counts actual egress by the keep-alive rate.
3. Document on `poison()` that it does not stop the keep-alive stream — teardown's `SessionHandles::KeepAlive` abort handle does.
4. Add a pinning test: unfunded PIX session, assert keep-alive emission stays within the documented bound (rate × pre-close window) and stops at teardown.

Gating the sink is deliberately **not** recommended: park-on-poison inside `try_for_each_concurrent(None, …)` would accumulate one parked future per tick, and deferring the stream until funded deprives the Entry's SURB balancer of level data during establishment for no real gain.

---

### PD-02 — Medium — Action-channel `Full` is treated as fatal, spuriously closing healthy sessions under load

**Location:** `transport/session/src/pix/worker.rs:246-254` (`send_actions`, `try_send`); action driver drain loop `transport/session/src/manager.rs:2317-2394`.

**Problem.** The worker forwards supervisor actions with non-blocking `try_send` and treats **any** error — including `TrySendError::Full`, which crossfire distinguishes from `Disconnected` — as permanent failure: it poisons the gate and exits, which the driver then converts into session closure with `PixFailure`. The 64-slot action channel _can_ transiently fill: the action driver stops draining `action_rx` whenever it awaits `send_ssa_request(...)` (a network round-trip) or the awaited `send_event`/`send_action_result` feedback, while the worker continues to emit actions (e.g. one `ProgressNotification` per progress-making share). This is fail-closed, so it is not unsafe — but it converts transient load into termination of an otherwise healthy, funded, paying session, and the true close reason (the dropped action's `Close(reason)`, if any) is lost, surfacing as generic `PixFailure`.

Note this was a deliberate trade-off in the M-06 fix (blocking on `action_tx` would reintroduce the `cmd_tx`↔`action_tx` deadlock cycle), but the availability cost was not addressed.

**Suggested fix (in preference order):**

1. Keep the driver's `action_rx.recv()` loop always draining: move blocking work (`send_ssa_request`, the awaited `send_event`/`send_action_result` feedback) off the drain path by spawning it (or pushing into a FuturesUnordered driven concurrently with `recv()`). With the drain loop non-blocking, 64 slots cannot realistically back up.
2. Additionally (cheap, independent): in `send_actions`, distinguish `TrySendError::Full` from `Disconnected`. On `Disconnected`, fail-close as today. On `Full`, do **not** kill the session for coalescible actions (`ProgressNotification` is idempotent and safe to drop); only treat `Full` as fatal for non-coalescible actions (`Close`, `RequestSsa`, `RetireSsa`), and log the distinct condition.

Do **not** switch `send_actions` to an awaited `send` — that reintroduces the deadlock cycle M-06 removed.

**Acceptance.** A test that stalls the action driver while the supervisor emits > 64 actions (progress notifications) and asserts the session survives; plus a test that `Disconnected` still fail-closes.

---

### PD-03 — Medium — Supervision-horizon validation omits `max_deposit_wait`; counter can TTL-expire mid-recovery

**Location:** `transport/session/src/pix/mod.rs:263-271`; counter insertion `protocols/pix/src/reconstructor/mod.rs:388-397`.

**Problem.** The per-SSA counter entry is inserted into the TTL-only `ssa_counters` cache when the commitment completes (during `AwaitingCommitment`), but the supervisor's `recovery_hard_deadline` clock starts only at deposit confirmation, which can be up to `max_deposit_wait` later. The true worst-case counter lifetime requirement is therefore `max_deposit_wait + max_recovery_time + tombstone_retention_window`, while `validate_pix_supervision` only checks `max_recovery_time + tombstone_retention_window`. (Independently confirmed by two reviewers.)

**Failure scenario.** `max_deposit_wait = 3600s`, `max_recovery_time = 3600s`, `tombstone = 30s` (true horizon 7230s) passes validation against the default `ssa_counter_lifetime = 7200s`. A slow-funding session then has its counter entry evicted mid-recovery: `record_useful_share` / `record_completed_part` / `record_invalid_share` / `snapshot_progress` are all `if let Some(entry)` no-ops, so **fault accounting and progress emission silently stop** — per-session fault-limit enforcement is defeated and the SSA can spuriously idle-close.

**Suggested fix.**

```rust
let supervision_horizon = cfg
    .max_deposit_wait
    .checked_add(cfg.max_recovery_time)
    .and_then(|d| d.checked_add(cfg.tombstone_retention_window))
    .unwrap_or(Duration::MAX);
```

Update the error string and doc comment to `"ssa_counter_lifetime must be > max_deposit_wait + max_recovery_time + tombstone_retention_window"`. Consider defensively including `max_ssa_delivery_time` as well (commitment→counter-insertion happens within the delivery window, so it is technically part of the span from session start). Add a boundary test with a config that passes the old check and fails the new one.

---

### PD-04 — Medium — Decreasing-counter regressions silently ignored, contradicting the spec; two tests assert the opposite of their names

**Location:** `transport/session/src/pix/supervisor.rs:425-427` (progress snapshot), `supervisor.rs:555-557` (unverifiable shares); tests `supervisor.rs:1574` (`lower_snapshot_closes_as_counter_regression`) and `supervisor.rs:1693` (`decreasing_invalid_count_closes_as_counter_regression`).

**Problem.** The recorded spec decision states: "a decreasing absolute counter closes as a protocol violation, never silently ignored." The implementation does the opposite — decreases are silently ignored as stale. Ignoring may actually be the _correct_ behavior (post-batch snapshots can arrive reordered via `for_each_concurrent`, so fail-closed would be a self-inflicted DoS), but right now:

- the code contradicts the written spec with no recorded rationale, and
- both tests are named `..._closes_as_counter_regression` while asserting `actions.is_empty()` — the exact opposite. A future maintainer "fixing" the tests to match their names would reintroduce a fail-closed DoS.

**Suggested fix.**

1. Decide the intended semantics with the spec owner. If ignore-as-stale is the decision (recommended, given the reordering reality), amend the spec/memory note and document the rationale in the supervisor module docs at both ignore sites.
2. Rename the tests to state the actual behavior, e.g. `lower_snapshot_is_ignored_as_stale` and `decreasing_invalid_count_is_ignored_as_stale`, with a comment explaining why close-on-regression was rejected.
3. If instead close-on-regression is required, the ordering problem must be fixed first (sequence-number the snapshots at the emission site) — do not enable it while reordering is possible.

---

### PD-05 — Medium — `transport_session_pix` integration suite is flaky in full-suite runs

**Location:** `hopr/hopr-lib/tests/transport_session_pix.rs`.

**Problem.** Observed during this review, on HEAD:

- Full run #1 (machine under review-agent load): `capture_n_hop_pix_session::case_1` FAILED at 240 s (the `TEST_GLOBAL_TIMEOUT` rstest timeout); passed on isolated rerun in 61 s.
- Full run #2 (idle machine): `deposit_timeout_closes_session::case_1` FAILED at 63.2 s; passed on isolated rerun in 63.2 s. All other 5 tests passed.

Two different tests failing across two full runs, each green in isolation, indicates real-time-deadline sensitivity (e.g. `max_ssa_delivery_time = 10s`, deposit windows, `chain_propagation_delay` multiples) and/or residual inter-test interference despite `#[serial]`/`-j 1`. For pre-deployment CI this will produce red builds unrelated to code changes and can mask genuine regressions.

**Suggested fix.**

- Capture and inspect the failing assertion of `deposit_timeout_closes_session` in a full-suite context (run the suite with `--no-capture` or nextest's failure output retained) to classify: timing margin vs. cross-test state.
- Widen the tightest real-time margins under test (the 10 s delivery window and the fixed sleeps derived from `chain_propagation_delay`) or gate progress on observed events rather than wall-clock sleeps, as was done for the transport_session suite previously.
- Until stabilized, configure nextest retries for this binary (`retries = 1`) with a tracking issue, so flakes are visible but not blocking.

---

### PD-06 — Medium (test) — H-02 cross-peer fault aggregation untested at the layer that implements it

**Location:** `protocols/pix/src/reconstructor/mod.rs:204-213` (`record_invalid_share`); reconstructor tests `mod.rs:818-1332` (all single-peer); supervisor test `transport/session/src/pix/supervisor.rs:1721`.

**Problem.** Cross-peer aggregation lives in the reconstructor (`invalid_total` bumps regardless of peer), yet every reconstructor test uses a single `OffchainKeypair`, and `invalid_shares_reports_absolute_totals_per_ssa` submits both bad shares from one peer. The new `cross_peer_invalid_shares_accumulates_separately` supervisor test never sees a peer at all — it feeds monotonically increasing `observed_total` values, exercising supervisor bookkeeping, not cross-peer behavior. The H-02 regression (per-peer counters allowing a multi-peer attacker to stay under the limit) would not be caught if reintroduced.

**Suggested fix.** Add a reconstructor test: two distinct peers each submit invalid shares for the same SSA across two `acknowledge_shares` calls; assert the second call's `InvalidShares.observed_total` equals the summed cross-peer total (e.g. peer A sends 2 invalid → total 2; peer B sends 1 invalid → total 3).

---

### PD-07 — Medium (test) — Command-channel backpressure (M-06 fix) unproven by any test

**Location:** `transport/session/src/pix/worker.rs:50-70` (awaited sends), channel created at `worker.rs:93` (capacity 64); only related test `send_event_on_disconnected_channel_returns_error` (`worker.rs:497`) covers disconnection.

**Problem.** The M-06 fix's core claim — "overflow cannot occur by construction; senders backpressure until the worker drains" — has no test. The previous review explicitly demanded a capacity-exhaustion test; the design changed, but the replacement guarantee (await-completes-once-drained, no deadlock) is equally untested.

**Suggested fix.** Construct a `SessionPixSupervisorHandle` over a small bounded channel with no worker attached, fill it to capacity, assert `send_event(...)` remains pending (poll once / `now_or_never`), then drain one slot from the rx side and assert the send completes. Wrap in a timeout so a deadlock fails the test rather than hanging it. Optionally add a worker-attached variant that floods 100+ events and asserts all are processed in order.

---

### PD-08 — Medium (test) — `enforce_pix_rejects_non_pix_session` still accepts a generic timeout as success

**Location:** `hopr/hopr-lib/tests/transport_session_pix.rs:684-686`.

**Problem.** Unchanged from the previous review's test-quality gap: the `Err(_)` arm of the 15 s timeout logs "timed out as expected" and passes. Any unrelated hang (routing, funding, start-protocol) satisfies the test, so it cannot detect a regression where `enforce_pix` stops rejecting non-PIX sessions.

**Suggested fix.** Require the explicit rejection path: treat `Ok(Err(rejection_error))` as the passing outcome (asserting on the error kind), and make a bare timeout a failure (or at minimum an explicit `#[ignore]`-worthy inconclusive panic). If the rejection currently only manifests client-side as a timeout by protocol design, assert the Exit-side effect instead (e.g. no session slot created / specific log-free assertion via the session manager's counters).

---

### PD-09 — Low — `start()` populates `msg_sender` before validation, leaving a permanently unstartable manager

**Location:** `transport/session/src/manager.rs:933-956`.

**Problem.** `self.msg_sender.set(...)` executes before `validate_pix_supervision(...)?`. On validation failure, `msg_sender` (a `OnceLock`) is already set while `pix_toolbox`/notifiers are not; a subsequent corrected `start()` call fails with `AlreadyStarted` instead of succeeding, and the original actionable `InvalidConfig` is replaced by a misleading error. No tasks are spawned before the check, so this is ordering-only.

**Suggested fix.** Move the `validate_pix_supervision` call (and the `pix_cfg` construction) above `self.msg_sender.set(...)` so no `OnceLock` is populated before all validation passes. Add a test: failed `start()` with invalid PIX config, then successful `start()` with a valid one on the same manager.

---

### PD-10 — Low — `max_deposit_wait` below the observer's fixed 100 ms delay guarantees spurious closure

**Location:** deposit-observer loop `transport/session/src/manager.rs` (~line 2650, `deposit_stream.next().delay(100ms).timeout(max_deposit_wait)`); validation `transport/session/src/pix/mod.rs:220-224`.

**Problem.** Validation only requires `max_deposit_wait` to be non-zero. Any value ≤ ~100 ms makes the `.timeout(max_deposit_wait)` always fire before the fixed `.delay(100ms)` completes, so even an already-buffered confirmation is never read — every PIX session closes with `DepositObserverClosed`, violating drain-before-close under that configuration.

**Suggested fix.** Either add a validation floor (`max_deposit_wait` must exceed the delay constant — extract the `100ms` into a named `const` and compare against it), or restructure the observer so the drain of an already-buffered confirmation is not subject to the delay (e.g. `try_next` first, then `delay`+`timeout` for the waiting path). The latter also removes a hidden coupling between an internal constant and a user-facing config bound.

---

### PD-11 — Low — Repeat commitment flow after builder TTI eviction clobbers `ssa_counters` to zero

**Location:** `protocols/pix/src/reconstructor/mod.rs:312-325` (`new_exit_commitment` duplicate guard), `mod.rs:388-397` (unconditional `ssa_counters.insert` on `Completed`).

**Problem.** `new_exit_commitment` rejects duplicates only while a **live** `commitment_builder` entry exists (TTI = `incomplete_commitment_lifetime`, default 120 s). If the builder evicts and the commitment flow re-runs for the same `SsaId`, completion unconditionally `insert`s a fresh counter entry (`useful_shares: 0, invalid_total: 0, ...`), clobbering the existing absolute counters. Downstream, the next snapshot looks like a counter regression (masked today by PD-04's silent-ignore, but it also **resets fault accounting to zero**, erasing accumulated `invalid_total` for the SSA).

**Suggested fix.** Make the counter insertion non-clobbering: only insert if absent (`ssa_counters.entry(ssa_id).or_insert_with(...)` equivalent for moka, or check `get` first under the entry API), or reject a repeated commitment completion for an `SsaId` that still has a live counter entry. Add a regression test: complete a commitment, record shares/invalids, force builder eviction, re-run the commitment flow, and assert counters retain their prior values.

---

### PD-12 — Low — Empty-SSA session close reports misleading `InvalidTransition`

**Location:** `transport/session/src/pix/supervisor.rs:221-224`.

**Problem.** If a recovered SSA's tombstone expires before the successor's `SsaRequestSent` feedback registers (slow/lost driver feedback), `ssas` becomes empty and the session closes with `InvalidTransition` — a confusing reason for "successor request never confirmed", complicating field diagnosis.

**Suggested fix.** Introduce a dedicated `SessionPixCloseReason` variant (e.g. `NoSuccessorRegistered` / `SsaSetDrained`) for this path, or at minimum document on `InvalidTransition` that it also covers tombstone-drained sessions. Remember the closure-reason snapshot test (`closure_reason_display_values_are_stable.snap`) must be updated with any new variant.

---

### PD-13 — Low — `RetireSsa` can be emitted after `Close` in the same action batch

**Location:** `transport/session/src/pix/supervisor.rs:206-218` (tombstone retirement block runs even when the preceding deadline loop already set `self.closed`).

**Problem.** When `handle_deadline` closes the session via `close_ssa_and_collect` and an expired tombstone also exists, the batch contains `Close(...)` followed by `RetireSsa(...)`. The driver handles this today (retire is idempotent, slot lookup tolerates removal), but the ordering is surprising and easy to break in the driver.

**Suggested fix.** Skip the retirement/empty-close block when `self.closed` is already set within the same `handle_deadline` invocation (whole-session teardown retires everything anyway via the retirement guard), or explicitly document "actions after `Close` are possible and must be tolerated" in the `SessionPixAction` contract.

---

### PD-14 — Low — Non-recovered SSAs closed mid-session are never retired from the reconstructor

**Location:** `transport/session/src/pix/supervisor.rs:634` (`close_ssa_and_collect`, non-terminal multi-SSA branch).

**Problem.** When an individual SSA is closed while the session continues (e.g. a pipelined successor requested at `AlmostRecovered` hits `CommitmentTimeout` while the predecessor still recovers), it is removed from `ssas` without a `RetireSsa`, so its reconstructor builder/verifier/counter state lingers until the independent cache TTLs expire. Bounded and self-healing, but inconsistent with the M-03 goal.

**Suggested fix.** Emit `RetireSsa(ssa_id)` from `close_ssa_and_collect` when removing a non-terminal SSA while the session stays open. The driver path is already idempotent.

---

### PD-15 — Low — Dead/ineffective recovery code on initial-action send failure

**Location:** `transport/session/src/pix/worker.rs:126-138`.

**Problem.** When the initial `send_actions` fails, the code rebuilds a fresh `SessionPixSupervisor` and calls `handle_deadline(Instant::now(), ...)` on it — a brand-new supervisor has no elapsed deadline, so `close_actions` is empty, and the subsequent `send_actions` goes to the already-broken channel anyway. Only `gate.poison(); return;` has any effect. The dead ceremony obscures the actual (correct) fail-closed behavior.

**Suggested fix.** Replace the block with `gate.poison(); return;` (optionally a `warn!`).

---

### PD-16 — Low (test) — Fault-counter persistence across eviction and SSA rotation is unverified

**Location:** `protocols/pix/src/reconstructor/mod.rs:1357` (`counters_survive_builder_eviction`); supervisor tests (no `session_invalid_total`-across-rotation test).

**Problem.** `counters_survive_builder_eviction` records nothing before invalidating the builder and only asserts the initial zeros — it does not demonstrate that non-zero `useful_shares`/`invalid_total` survive eviction. And no supervisor test verifies the documented "fault counters must not reset on SSA rotation" invariant end-to-end (accumulate faults on SSA #1, recover+retire it, fault SSA #2, assert the session-level total continues).

**Suggested fix.** Extend the eviction test to record useful and invalid shares first, then assert the values (including `invalid_total`) survive builder/verifier invalidation. Add the supervisor rotation test described above asserting `session_invalid_total` monotonicity across `RetireSsa`.

---

### PD-17 — Info — Poison semantics: "no new service after poison is observed"

**Location:** `transport/session/src/pix/gate.rs:65-109`.

An `acquire()` that has passed the `poisoned` check can still complete its `served` CAS if `poison()` runs concurrently — at most one in-flight packet per concurrent acquirer may be served after poisoning begins. Inherent to the lock-free design and acceptable for a fail-closed backstop. **Action:** document this guarantee on `poison()` so callers don't assume strict quiescence after it returns.

---

### PD-18 — Info — Exit accepts any nonzero deposit; confirm divergence from spec

**Location:** `transport/session/src/manager.rs` (~line 2620, `CommitmentVerified { expected_deposit: None }`).

The Exit-side supervisor is armed with `expected_deposit: None` (accept-any), per the inline comment "Exit does not know the expected deposit amount". The recorded spec decision mentions a deposit-amount check with `None` = accept-any as an explicit option, so this is likely intentional — but it means an attacker can fund with a dust amount and rely only on the `max_served_without_progress` backstop to bound over-service. **Action:** confirm with the spec owner that the post-funding volume backstop is considered sufficient compensation, and record the decision; otherwise plumb the negotiated quota into an expected amount.

---

## Verification performed

**Passed:**

- `cargo check --workspace` — clean.
- `cargo clippy -p hopr-transport-session -p hopr-protocol-pix -p hopr-transport` — no warnings from branch code (only the pre-existing `proc-macro-error2` future-incompat note).
- `cargo nextest run --lib -p hopr-transport-session -p hopr-protocol-pix -p hopr-transport` — **325/325 passed**.
- `cargo nextest run -p hopr-transport-session --test '*' -j 1` — **8/8 passed**.
- Every test in `transport_session_pix` passes in isolation, including both tests that failed in full-suite runs.

**Flaky (see PD-05):**

- `cargo nextest run -p hopr-lib --features testing --test transport_session_pix -j 1`:
  - Run #1 (loaded machine): `capture_n_hop_pix_session::case_1` failed at the 240 s global timeout; remaining tests cancelled.
  - Run #2 (idle machine, `--no-fail-fast`): 5/6 passed; `deposit_timeout_closes_session::case_1` failed at 63.2 s.
  - Both failures passed on immediate isolated reruns (61.3 s and 63.2 s respectively).

**Review method:** four parallel scoped reviews (supervisor core + config validation; gate/worker/notify concurrency; manager/transport integration; reconstructor + test quality), with all Medium findings independently re-verified against the code at HEAD by the coordinating reviewer. The supervision-horizon gap (PD-03) was found independently by two reviewers.

## Required before deployment

1. Stop treating transient action-channel fullness as fatal (PD-02).
2. Include `max_deposit_wait` in the supervision-horizon validation (PD-03).
3. Resolve the decreasing-counter spec contradiction and fix the two misleading test names (PD-04).
4. Stabilize (or retry-gate with a tracking issue) the `transport_session_pix` suite (PD-05).
5. Add the missing cross-peer, backpressure, and rejection-path tests (PD-06, PD-07, PD-08).

Low-severity items (PD-01, PD-09 – PD-16) are recommended for the same change set — most are a few lines or documentation each — but are not deployment blockers individually.
