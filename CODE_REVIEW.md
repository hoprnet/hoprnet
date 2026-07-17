# Re-review: `lukas/session-pix-supervisor`

**Review date:** 2026-07-17
**Compared:** `origin/lukas/pix` (`0fc767f904fe8587fa8d4513b78ff88147eba88a`) to `HEAD` (`4b598c20cd7734ed655b4314601f8616a6d61920`)
**Verdict:** **CHANGES REQUESTED**

## Executive summary

The latest changes genuinely fix the previous command-queue event loss: command producers now await bounded capacity. Recovered tombstones also emit `RetireSsa`, completed observer handles are removed, and invalid PIX configuration is rejected rather than logged and accepted.

No critical or high-severity regression was found. Approval is still blocked by five medium implementation/design findings:

1. Default SURB keep-alive egress bypasses the PIX service gate.
2. Transient action-channel saturation kills an otherwise healthy session.
3. Counter-lifetime validation omits the deposit-wait portion of the supervision horizon.
4. SSA creation is not transactionally owned during send and partial session setup, so stale commitments can block retries.
5. Counter-decrease behavior contradicts the public close-reason contract and two test names; the intended protocol semantics must be made explicit.

The affected crate tests and one full downstream run pass on the reviewed head. Other full-suite runs during review were not consistently green, and several requested regression tests remain absent.

## Status of all previous findings

| ID   | Severity | Status            | Re-review result |
| ---- | -------- | ----------------- | ---------------- |
| H-01 | High     | Partial           | Both data-egress branches acquire permits, but the default SURB keep-alive path does not; see PD-01. |
| H-02 | High     | Addressed in code | Fault totals are mutex-protected and cross-peer. The requested producer-level two-peer test is still missing; see PD-06. |
| H-03 | High     | Addressed         | `max_served_without_progress` supplies a finite post-deposit backstop and funding snapshots the correct watermark. |
| H-04 | High     | Addressed         | All terminal worker paths return and poison the gate; disconnection and action-send failure have observable assertions. |
| M-01 | Medium   | Addressed         | The supervisor and gate are installed before session construction and publication. |
| M-02 | Medium   | Addressed         | Recovery is phase-gated and tombstones absorb late events. Counter-decrease semantics remain a contract issue under PD-04. |
| M-03 | Medium   | Partial           | Normal tombstone and whole-session retirement work; failed request/setup and nonterminal SSA-removal paths remain incomplete (PD-14, PD-19). |
| M-04 | Medium   | Addressed         | `SlotNotify` uses generation-based signaling with cancellation and waker-replacement coverage. |
| M-05 | Medium   | Addressed         | Funding snapshots the current service watermark, including predeposit usage above the post-funding ceiling. |
| M-06 | Medium   | Addressed in code | Command events backpressure rather than drop. The action side remains fail-closed on transient `Full`, creating the availability issue in PD-02. |
| M-07 | Medium   | Addressed         | Next-SSA index calculation is checked and same-index replacement is rejected. |
| L-01 | Low      | Addressed         | Canceled notification futures unregister their wakers. |
| L-02 | Low      | Partial           | Validation returns an error, but it runs after `msg_sender.set()` and leaves a failed manager permanently unstartable (PD-09). |
| L-03 | Low      | Addressed         | Worker channels are bounded; command overflow backpressures and action overflow fails closed. |

## New and remaining findings

| ID    | Severity | Title |
| ----- | -------- | ----- |
| PD-01 | Medium | Default keep-alive egress bypasses the PIX service gate |
| PD-02 | Medium | Action-channel `Full` spuriously closes healthy sessions |
| PD-03 | Medium | Counter-lifetime validation omits `max_deposit_wait` |
| PD-04 | Medium | Counter-decrease contract and tests contradict implementation |
| PD-05 | Medium (test) | Full downstream PIX suite is timing-sensitive/flaky |
| PD-06 | Medium (test) | H-02 cross-peer aggregation lacks a producer-level test |
| PD-07 | Medium (test) | Command-channel backpressure is untested |
| PD-08 | Medium (test) | Enforced-PIX rejection accepts an unrelated timeout as success |
| PD-09 | Low | Invalid startup commits `msg_sender` before validation |
| PD-10 | Low | Deposit timeout can be shorter than the observer's fixed delay |
| PD-11 | Low | Repeated commitment completion can reset live counters |
| PD-12 | Low | Empty SSA state closes with misleading `InvalidTransition` |
| PD-13 | Low | A deadline batch can emit `RetireSsa` after `Close` |
| PD-14 | Low | A nonterminal failed SSA is removed without retirement |
| PD-15 | Low | Initial action-send failure contains ineffective recovery code |
| PD-16 | Low (test) | Counter persistence across eviction/rotation is not meaningfully tested |
| PD-17 | Info | Gate poison permits already-in-flight acquires to complete |
| PD-18 | Info | Exit accepts any nonzero deposit; confirm this policy |
| PD-19 | Medium | Failed SSA request/setup state escapes retirement and blocks retry |

### PD-01 — Default keep-alive egress bypasses the PIX service gate

**Affected code:** `transport/session/src/manager.rs:2183-2217`

Both application-data sinks call `pix_egress_gate.acquire()` before forwarding. `spawn_keep_alive_stream()` instead wraps `msg_sender` with SURB accounting only. Transport configuration enables these level-notifying keep-alives every 60 seconds by default, so an unfunded PIX session can continue emitting return-path packets outside `max_predeposit_packets` and `max_served_without_progress`.

Gate keep-alives like the other egress paths, suppress them until funding, or explicitly document and test a deliberate control-plane exemption and its separate abuse bound.

### PD-02 — Action-channel saturation kills healthy sessions

**Affected code:** `transport/session/src/pix/worker.rs:245-253`, `transport/session/src/manager.rs:2317-2394`

`send_actions()` uses `try_send()` and treats `Full` exactly like `Disconnected`: it poisons the gate and exits, after which the driver closes with `PixFailure`. The driver can stop draining while it awaits an SSA network send and command feedback, while the worker can emit a `ProgressNotification` for each progress event. More than 64 queued actions therefore converts transient load into session loss and can replace a specific pending close reason with a generic failure.

Keep the action drain continuously serviced, process/coalesce replaceable progress notifications without filling the queue, and distinguish `Full` from `Disconnected`. Do not simply await `action_tx`, which would recreate a command/action channel deadlock cycle.

### PD-03 — Supervision horizon omits the deposit wait

**Affected code:** `transport/session/src/pix/mod.rs:263-270`, `protocols/pix/src/reconstructor/mod.rs:387-397`

The counter TTL starts when the client commitment becomes verifiable, before deposit confirmation. Validation requires only:

`ssa_counter_lifetime > max_recovery_time + tombstone_retention_window`

The real required horizon is:

`max_deposit_wait + max_recovery_time + tombstone_retention_window`

For example, `3600s + 3600s + 30s` passes the current check against the default 7200-second counter TTL even though the counter can expire mid-recovery. Once absent, progress and invalid-share updates become no-ops, defeating fault enforcement and causing false idle closure. Use checked arithmetic for the full horizon and add an old-pass/new-fail boundary test.

### PD-04 — Counter-decrease semantics are contradictory

**Affected code:** `transport/session/src/pix/mod.rs:195-196`, `transport/session/src/pix/supervisor.rs:419-427,552-557,1574-1587,1692-1718`

`CounterRegression` is documented as a decreasing observation, and tests are named `lower_snapshot_closes_as_counter_regression` and `decreasing_invalid_count_closes_as_counter_regression`. The implementation and assertions instead silently ignore decreases as stale because concurrent acknowledgement batches can reorder snapshots.

Ignoring stale snapshots is the safer behavior unless event ordering is first made deterministic. Record that decision in the protocol contract, update the close-reason documentation, and rename the tests. If close-on-decrease is actually required, sequence snapshots at their producer before enforcing it.

### PD-19 — SSA request/setup cleanup is not transactional

**Affected code:** `transport/session/src/manager.rs:1541-1599,2038-2077,2307-2351`, `protocols/pix/src/reconstructor/mod.rs:299-327`

`send_ssa_request()` inserts an exit commitment before the fallible wire send. A send failure does not retire it. During initial setup, the retirement guard is not created until after session construction, external notification, establishment response, and other fallible work. Later-cycle IDs are added to the guard only after `send_ssa_request()` succeeds.

An initial failure therefore leaves `(pseudonym, SSA index 1)` in the commitment cache until its default 120-second idle expiry; an immediate same-pseudonym retry gets `DuplicateCommitment` (and repeated access can prolong retention). Later-cycle send failures similarly leave untracked state.

Roll back commitment creation on send failure, establish retirement ownership before the initial request, and transfer that ownership to the driver only after setup succeeds. Cover initial-send failure, post-send setup failure, immediate retry, and later-cycle failure.

## Lower-severity implementation observations

- **PD-09:** `SessionManager::start()` sets `msg_sender` before validation. Move all validation before any startup `OnceLock` mutation and test invalid-then-valid startup on the same manager.
- **PD-10:** the deposit observer wraps every `next()` in a fixed 100 ms delay but accepts any nonzero `max_deposit_wait`; values at or below the delay guarantee timeout even for a buffered confirmation. Validate a floor or remove the delay from the already-ready path.
- **PD-11:** after commitment-builder idle eviction, repeating the same commitment flow unconditionally replaces `ssa_counters` with zeros, erasing progress and faults. Preserve an existing counter or reject reuse while it remains live.
- **PD-12:** when a tombstone expires before its successor is registered, an empty SSA set closes as `InvalidTransition`; use a diagnostic reason that identifies missing successor registration.
- **PD-13:** tombstone retirement still runs after the deadline loop has set `self.closed`, so a batch can contain `Close` followed by `RetireSsa`. Whole-session cleanup makes this safe today, but the action contract should avoid or document post-close actions.
- **PD-14:** `close_ssa_and_collect()` removes a failed SSA when another SSA remains live without emitting `RetireSsa`; its reconstructor state survives until TTL or whole-session cleanup.
- **PD-15:** rebuilding a fresh supervisor and immediately polling its deadline after initial action-send failure cannot produce a close action and sends it to the already failed channel anyway; poison and return directly.
- **PD-17:** lock-free poison semantics allow an acquire that already passed the poison check to complete; document the bounded in-flight behavior.
- **PD-18:** `CommitmentVerified` passes `expected_deposit: None`, so any nonzero deposit releases service. Confirm that the post-funding volume backstop is the intended compensation for accepting dust deposits.

## Test-quality and reliability gaps

- **PD-05:** one clean current-tip run passed all six downstream scenarios, but other full-suite runs during review hit a 240-second global timeout and a deposit-timeout failure; both failing cases passed alone. Replace wall-clock sleeps with event synchronization or add justified CI retry tracking after diagnosing cross-test timing.
- **PD-06:** `cross_peer_invalid_shares_accumulates_separately` creates no peers and feeds totals directly to a peer-agnostic supervisor event. Interleave invalid shares from two `OffchainPublicKey`s in a reconstructor test and assert aggregate emitted totals.
- **PD-07:** fill a small command channel, prove the next async send remains pending, drain one slot, and prove delivery completes without loss/deadlock.
- **PD-08:** `enforce_pix_rejects_non_pix_session` accepts a generic 15-second timeout as success. Require the explicit rejection or assert the Exit-side policy effect.
- **PD-16:** `counters_survive_builder_eviction` checks only initial zeros. Record nonzero progress/faults before eviction and test session fault totals across SSA rotation.
- No manager-level test verifies reconstructor and observer retirement over repeated cycles or any PD-19 rollback path.
- `cargo clippy --all-targets` reports three branch-local test warnings: two unnecessary `u64` casts and one `clone()` on a `Copy` progress value in `supervisor.rs`.

## Verification performed

### Passed on `4b598c20cd7734ed655b4314601f8616a6d61920`

- `cargo check --workspace`
- `cargo nextest run -p hopr-protocol-pix -p hopr-transport-session -p hopr-transport`
  - 392 passed, 0 failed, 0 skipped across 16 binaries.
- `RUST_LOG=off cargo nextest run -p hopr-lib --features testing --test transport_session_pix -j 1 --status-level fail --final-status-level fail`
  - 6 passed, 0 failed, 0 skipped in 387.229 seconds.
  - Covers one-, two-, and three-hop multi-cycle recovery, deposit timeout, enforced-PIX rejection, and recovery hard deadline.
- `cargo clippy -p hopr-transport-session -p hopr-protocol-pix -p hopr-transport --all-targets`
  - Exit 0 with the three test-code warnings listed above.
- `cargo fmt --all -- --check`

### Repository-wide feature note

`cargo check --workspace --all-targets --all-features` is not a supported aggregate configuration because it enables mutually exclusive allocator and crypto-suite features. The supported default workspace check is green.

## Required before approval

1. Gate or explicitly bound/document default keep-alive egress (PD-01).
2. Prevent transient action-channel fullness from terminating healthy sessions (PD-02).
3. Validate the complete deposit/recovery/tombstone counter horizon and the observer-delay floor (PD-03, PD-10).
4. Make SSA creation and retirement transactional across all request/setup/individual-close paths (PD-14, PD-19).
5. Resolve and document counter-decrease semantics, then fix the contradictory tests (PD-04).
6. Add the missing cross-peer, command-backpressure, rollback, lifecycle, and exact policy-rejection tests; stabilize the downstream suite.
