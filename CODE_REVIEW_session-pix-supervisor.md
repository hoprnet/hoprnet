# Code Review: `lukas/session-pix-supervisor` vs `origin/lukas/pix`

**Scope:** `git diff origin/lukas/pix...HEAD` — 34 files, ~8,200 insertions. The bulk is the new
Exit-side PIX supervision module (`transport/session/src/supervision/`: pure state machine,
lock-free `ServiceGate`, per-session worker actor), its integration into `SessionManager`
(action driver, deposit observer, egress gating), and the reconstructor extensions in
`protocols/pix` (progress/fault counters, RAII SSA retirement).

**Review history (consolidated).** This document merges three review rounds: the original
review (HIGH-1..2, MEDIUM-1..4, LOW-1..5, at `13af142125`), the 2026-07-17 re-review at
`21c876cde4` (added LOW-6..7), and the 2026-07-18 re-review plus fresh full-diff sweep at
`dbc351e304` (added HIGH-3, MEDIUM-5, LOW-8..12). Finding IDs are stable across rounds. Earlier
pre-deploy rounds (PD-02..PD-19, H-01..H-04, M-01..M-06) predate this document and were fixed
before its scope — see the commit history. Line numbers refer to `dbc351e304` unless noted.

**Current status: 2 open issues that should block a merge (HIGH-3, MEDIUM-5), 6 further open
low-severity issues plus residuals and nits, and 13 findings fixed and verified.**

---

# Open issues

## HIGH-3 — `hopr-protocol-hopr` test build is broken: 16 compile errors in `surb_store.rs` (merge blocker)

**Files:** `protocols/hopr/src/surb_store.rs:409-462` (tests
`surb_ring_buffer_len_reflects_push_and_pop`, `memory_surb_store_surb_count`)

**Defect.** The two tests added in `dbc351e304` (commit message: _"Fix missing closing brace in
`surb_store.rs` test module"_) are written against APIs that do not exist:
`SurbRingBuffer::{len, is_empty}`, `MemorySurbStore::surb_count`, `HoprSurbId::new`,
`HoprSurb::random`, and `SimplePseudonym::random` in that scope — 16 E0599 errors. They were
committed without ever being compiled (the 2026-07-17 re-review only ran the
`hopr-transport-session` and `hopr-protocol-pix` suites; `hopr-protocol-hopr` was never
test-built). They are also scope creep: they exercise pre-existing SURB-store behavior
unrelated to this branch.

**Failure scenario.** `cargo nextest run --lib` (workspace-wide or `-p hopr-protocol-hopr`) and
`cargo nextest run --no-run` — the project's own prescribed verification steps — fail to build:

```
error[E0599]: no method named `surb_count` found for struct `surb_store::MemorySurbStore`
error[E0599]: no method named `len` found for struct `surb_store::SurbRingBuffer<S>`
... (16 total)
```

**Suggested mitigation.** Delete both tests, or rewrite them against the real API. If deleted,
`protocols/hopr` reverts to test-only parity with the base branch (its only other change in
this diff is the codec-test adjustment for the new `Progress` resolutions).

---

## MEDIUM-5 — reconstructor cache capacity regressed ~42×, and `max_concurrent_sessions` is never plumbed from real config

**Files:**

- `protocols/pix/src/reconstructor/mod.rs:163-183` (caches sized `max_concurrent_sessions × 3` = 384 at the default of 128)
- `transport/hopr/src/lib.rs:333` (`SsaReconstructorConfig::default()` hardcoded — the only production construction site)
- `transport/session/src/manager.rs:432` (`SessionManagerConfig::maximum_sessions` default: 10 000)

**Defect.** The LOW-5 fix replaced the `MAX_POLYS_PER_SSA + 1` (≈16 193) capacity of
`commitment_builder` / `ssa_builders` / `ssa_counters` / `ssa_to_verifier_ids` with
`max_concurrent_sessions × 3`. The intent (derive from session count) was right, but (a) the
new default bound is **384** — roughly 42× smaller than before — and (b) nothing plumbs the
session manager's actual limit into `max_concurrent_sessions`: `HoprTransport::new` constructs
the reconstructor from `SsaReconstructorConfig::default()` unconditionally, while
`SessionManagerConfig::maximum_sessions` defaults to 10 000. The very mismatch LOW-5 warned
about ("the bound should be derived from `maximum_sessions × 3` … or passed in explicitly") is
now reachable at roughly 1/42 of the previous load.

**Failure scenario.** Above ~384 concurrent live SSAs (≈128 PIX sessions, vs 10 000 allowed),
moka begins evicting entries for _live_ SSAs (TinyLFU victims are not necessarily idle ones):

- an evicted `ssa_counters` entry makes `snapshot_progress` return `None` and
  `record_useful_share` silently no-op → the supervisor sees a permanent stall → healthy,
  funded sessions close with `RecoveryIdle` (while service flows) or `RecoveryDeadline`;
- an evicted `commitment_builder` mid-handshake makes subsequent client commitment parts fail
  with `MissingSsaCommitment` → `CommitmentTimeout` close;
- an evicted `ssa_builders` entry mid-recovery makes completed polynomial parts error out →
  recovery stalls.

**Suggested mitigation.** Plumb `maximum_sessions` into `max_concurrent_sessions` at the
reconstructor construction site (10 000 fits the `u16`), or size the caches from the session
manager's limit directly; add a cross-check to `validate_pix_supervision` so the pair cannot
drift again. (`ssa_verifiers` keeps its separate `MAX_POLYS_PER_SSA × 4` bound — fine at
default dims, worth revisiting in the same pass.)

---

## MEDIUM-4 residual (accepted stopgap; effectively LOW) — `hopr-protocol-session` is still not runtime-agnostic

The standalone compile break is fixed (see Fixed list), but via feature plumbing:
`default = [..., "runtime-tokio"]` in `protocols/session/Cargo.toml` plus an explicit
`"runtime-tokio"` in `transport/session/Cargo.toml`. The workspace root pins the crate with
`default-features = false`, so the `default` addition is inert for every workspace consumer —
and the four unconditional `spawn` call sites in `protocols/session/src/socket/`
(`ack_state.rs:275,314,334`, `mod.rs:332`) remain. Any new workspace consumer that doesn't
(transitively) enable `runtime-tokio` hits the same compile break. Fine as a stopgap; the real
fix is `#[cfg]`-gating the spawn sites (or making `spawn` runtime-agnostic, the stated goal of
the crossfire refactor).

---

## LOW-4 residual — tombstone-retired SSA indices are not recorded; a stale `SsaRequestSent` can still resurrect them

**File:** `transport/session/src/supervision/supervisor.rs:224-234` (tombstone retirement),
`:303-306` (guard), `:701` (failure-path recording)

**Defect.** `dbc351e304` added `retired_ssa_indices`: it now guards `on_ssa_request_sent` and
is recorded on the failure-path removal in `close_ssa_and_collect` — but the tombstone-expiry
retirement in `handle_deadline` removes SSAs _without_ recording them. A stale duplicate
`SsaRequestSent` for a recovered-and-retired index therefore still re-creates
`AwaitingCommitment` state with a fresh commitment deadline, which can pollute
`first_failure_reason` with a spurious `CommitmentTimeout`. Latent, as originally filed: the
driver emits the event exactly once per request today.

**Suggested mitigation.** Push the retired indices in the tombstone `retain` path too. Note:
`retired_ssa_indices` is an unbounded `Vec`, but it grows one entry per failed SSA over a
session's lifetime — negligible in practice.

---

## LOW-8 — action driver reports `SsaRequestSent` to the supervisor even when the wire send failed

**File:** `transport/session/src/manager.rs:2313-2321` (driver `RequestSsa` arm)

The driver calls `send_ssa_request` and then sends `SessionPixEvent::SsaRequestSent`
**unconditionally**, before reporting `send_action_result(…, result.is_ok())`. On a failed
send, the supervisor first registers a phantom SSA in `AwaitingCommitment` (arming a
commitment deadline for a request that never left the node) and only then closes via the
`ok: false` feedback (`RequestSsa` failure → `Close(SupervisorUnavailable)`). The end state is
fail-closed, so this is not exploitable — but the event stream lies to the state machine and
the ordering only works by accident of the feedback loop.

**Suggested mitigation.** Gate the `SsaRequestSent` event on `result.is_ok()`.

---

## LOW-9 — telemetry residuals of the LOW-6/LOW-7 rework

**Files:**

- `transport/session/src/telemetry.rs:300-307` (`remove_session_metrics_state`), `:402` (`PixSsaPhase::Recovering`), `:406` (`set_pix_current_ssa_phase`)
- `transport/session/src/manager.rs:1680`, `:1695` (telemetry call sites), `:2340` (`ReleaseService` arm)
- `METRICS.md:52`

Four residuals:

1. **Phase-gauge series are never cleared.** `hopr_session_pix_current_ssa_phase` is now keyed
   by `(session_id, ssa_index)`, but `remove_session_metrics_state` no longer touches it (it
   cannot — it doesn't know which indices exist). Closed sessions leave stale per-SSA series
   (typically frozen at `Recovered = 3`) in `/metrics` for the node's lifetime — the
   cardinality/lifecycle problem LOW-7 flagged, in a new shape.
2. **`PixSsaPhase::Recovering` is dead code.** Its only write was removed from the
   `ReleaseService` arm and never re-added, so the gauge jumps 1 → 3 and the most
   operationally interesting phase is never reported. Compiler-confirmed under
   `--features telemetry`: `warning: variant 'Recovering' is never constructed`.
3. **METRICS.md drift.** The row still documents `keys: session_id` (and a `2=Recovering`
   value that never occurs); `dbc351e304` touched the adjacent row but not this one.
4. **`hopr_session_pix_recovery_progress` conflates concurrent SSAs.** Still keyed by
   `session_id` only, so during SSA overlap the successor's early progress (e.g. 0.05)
   overwrites the predecessor's 0.95 — the same defect LOW-6 fixed for the phase gauge.

The LOW-7 nit (clippy `needless_borrow` at `manager.rs:1680`/`:1695` under
`--features telemetry`) is also still present.

**Suggested mitigation.** Report the phase from a single place (e.g. the action driver, from a
supervisor snapshot); key the progress gauge by `ssa_index` as well; track the live indices per
session so `remove_session_metrics_state` can clear (or remove) the series; update METRICS.md;
drop the needless borrows.

---

## LOW-10 — missing version bumps on crates with public API changes

`hopr-protocol-pix` stays at 0.1.0 despite breaking public API changes (the `ShareResolution`
variants were replaced — `InvalidShare` removed, `Progress`/`InvalidShares` added; `retire_ssa`
was added to the `ExitAcknowledgementShareProcessor` trait; `SsaCommitmentGuard` and
`SsaRecoveryProgress` are newly exported). `hopr-utils` is likewise unbumped despite the new
public `CrossfireSinkError::GateClosed` variant. Every sibling crate in this diff was bumped
(`hopr-crypto-packet` 1.4→1.5, `hopr-protocol-hopr` 4.7→4.8, `hopr-protocol-session`
1.2.2→1.2.3, `hopr-transport` 0.32→0.33, `hopr-transport-session` 0.23.1→0.24.0), so these two
look like omissions rather than policy.

---

## LOW-11 — stale comments still describe the removed `pending()` behavior

**Files:** `transport/session/src/manager.rs:2358-2359` and `:2377-2378` (driver `Close` arm
and channel-closed fallback: _"they hang on `pending()` until the session task is aborted"_),
`:5866` (regression-test doc comment: _"Currently the egress adapter parks on `pending()`"_).

All three describe the pre-fix behavior; the code now surfaces `io::Error` (see MEDIUM-1 in
the Fixed list). MEDIUM-1's original complaint was partly about comments contradicting the
behavior — the fix introduced new ones. Update or delete them.

---

## LOW-12 — tombstone expiry can close a healthy session while the successor request is in flight

**File:** `transport/session/src/supervision/supervisor.rs:224-243`

When a recovered SSA's tombstone expires, `handle_deadline` removes it and — if `ssas` is then
empty — closes the session with `NoSsaRemaining`. The fact that a successor `RequestSsa` is
outstanding lives only on the removed tombstone's `next_requested` flag, so if the driver's
`send_message` for the successor stalls longer than `tombstone_retention_window` (30 s), a
funded, healthy session is closed despite a request being in flight. Requires a >30 s transport
stall while the session is otherwise alive — theoretical today. A pending-request counter on
the supervisor (set on `RequestSsa` emission, cleared on `SsaRequestSent` or a failed
`action_result`) would close the hole cheaply.

---

## Nits (open)

- New clippy warnings in branch code (project rule: no warnings): doc-list indentation at
  `manager.rs:654-655` and `supervision/mod.rs:68`; `u64`→`u64` casts at
  `supervisor.rs:1511,1513`; `.clone()` on a `Copy` type at `supervisor.rs:1677`; two
  `type_complexity` warnings in `hopr-protocol-pix`; plus the LOW-9 items under
  `--features telemetry`.
- Late PIX events for a just-closed session produce two `error!` lines each
  (`dispatch_pix_event` slot-miss at `manager.rs:1667` plus the transport wrapper in
  `share_resolution_to_pix_event`). Bounded by the 30 s ack window, but `NonExistingSession`
  shortly after a close is expected traffic — `debug!` would be kinder to operators.
- `CrossfireSinkError::GateClosed` (hopr-utils) duplicates the concept of
  `supervision::gate::GateClosed` in an unrelated crate purely so the manager can wrap it in
  `io::Error::other`; using the gate's own error type would avoid the cross-crate coupling.
- A one-off rustc ICE (LLVM stack overflow in ThinLTO codegen) was observed while compiling
  `hopr-transport-session`'s test binary; it did not reproduce on retry. The deeply nested
  egress sink-combinator types are a plausible trigger; `RUST_MIN_STACK=16777216` is the
  workaround if CI ever hits it.
- This document itself ships in the branch. Previous rounds removed the review docs before
  merge (`798d513fd5`, `4d902d93e7`) — do the same here before this branch is merged.

---

# Fixed issues

All fixes re-verified against `dbc351e304` by code inspection and test runs (see the
verification log below). Regression tests named here are committed and passing.

- **HIGH-1 — terminal recovery events dropped outside `Recovering` were never re-synthesized**:
  a fully recovered **and fully funded** session got force-closed with `RecoveryIdle` (or
  `RecoveryDeadline`) because `Recovered` arriving during `AwaitingCommitment`/`AwaitingDeposit`
  was discarded and the reconstructor emits it exactly once. Fixed in `21c876cde4`:
  `PerSsaState.recovered_pending` defers the event and `on_deposit_confirmed` replays it via
  `perform_recovered_transition`; `on_almost_recovered` also defers in `AwaitingCommitment`.
  All orderings re-traced on `dbc351e304` — each yields exactly one successor request and no
  spurious close. Regression test: `recovered_before_deposit_then_funded_session_survives`.

- **HIGH-2 — Entry-side PIX parameter verification was a tautology** (regression vs base): the
  slot's `ssa_params` were initialized _from the Exit's own message_ and compared against
  themselves, with no protocol-bounds check, letting a malicious Exit dictate the quota the
  Entry reported upward (e.g. `u16::MAX × u16::MAX` ≈ 4.4 TB accepted). Fixed in `21c876cde4`
  (slot pre-populated with the offered params in `new_session`; empty-lock rejection; bounds
  check against `MAX_POLYS_PER_SSA` / `MAX_POLY_THRESHOLD`) and completed in `dbc351e304`
  (dimensions compared **exactly** at `manager.rs:2779`, closing the transposed-dims residual;
  reported quota derived from our params). Regression test:
  `entry_rejects_exit_dictated_ssa_params`.

- **MEDIUM-1 — egress writes against a poisoned gate hung forever**: the egress adapters parked
  on `futures::future::pending()`, and a user-held writer (one-way egress, no read loop) is not
  covered by the manager's `abort_handles`, so it hung indefinitely;
  `CrossfireSinkError::GateClosed` was dead code. Fixed in `dbc351e304`: both adapters go
  through `sink_map_err(std::io::Error::other)` and return
  `Err(io::Error::other(CrossfireSinkError::GateClosed))` on poison (`manager.rs:2081-2107`,
  `:2204-2222`); all removal paths poison the gate (`close_session`, driver `Close` arm, worker
  death, cache eviction), so writers observe a bounded `io::Error`. Regression test:
  `writer_on_poisoned_gate_errors_out`. Leftover stale comments → LOW-11 (open).

- **MEDIUM-2 — deposit observer logged an error and self-terminated 60 s after every
  _successful_ deposit**, losing later top-ups. Fixed in `21c876cde4` via the
  `deposit_forwarded` flag: after any forwarded deposit the timeout branch exits with `debug!`
  and no `DepositObserverClosed`. Verified on `dbc351e304`: serial top-ups are forwarded as
  long as each arrives within `max_deposit_wait` of the previous one; an insufficient first
  deposit still closes via the supervisor's deposit deadline.

- **MEDIUM-3 — `std::sync::Mutex` + `expect("lock poisoned")` in the reconstructor counters**
  (project-rule violation; one poisoned lock would panic every subsequent ack batch for that
  SSA). Fixed in `21c876cde4`: all lock sites converted to `parking_lot::Mutex`; the unused
  return value of `record_invalid_share` was dropped.

- **MEDIUM-4 — `hopr-protocol-session` no longer compiled standalone** (`runtime-tokio` removed
  from its `hopr-utils` dependency while `socket/` still calls `spawn` unconditionally).
  Fixed in `21c876cde4`: `runtime-tokio` added to the crate's `default` features and explicitly
  to `transport/session`'s dependency features; `cargo check -p hopr-protocol-session` passes.
  Residual (crate still not runtime-agnostic) → "MEDIUM-4 residual" in the open list.

- **LOW-1 — the drain-path close ignored `first_failure_reason`**, reporting `NoSsaRemaining`
  instead of the actual first cause. Fixed in `21c876cde4`:
  `first_failure_reason.unwrap_or(NoSsaRemaining)`.

- **LOW-2 — `Close` emitted without latching `self.closed`** on the `SsaIndex::try_from` error
  branch of `emit_request_next_ssa` and the `RequestSsa { ok: false }` branch of
  `action_result`. Fixed in `21c876cde4`: both branches latch `closed = true` before emitting.

- **LOW-3 — `ServiceGate::acquire`'s predeposit path lacked the pre-CAS poison re-check** the
  funded path has. Fixed in `21c876cde4`: same re-check added (`gate.rs:122-126`).

- **LOW-4 — stale `SsaRequestSent` could resurrect a closed SSA** (any untracked SSA id with a
  matching pseudonym re-created fresh state). Partially fixed in `dbc351e304`:
  `retired_ssa_indices` guard + failure-path recording. The tombstone-path gap remains →
  "LOW-4 residual" in the open list.

- **LOW-5 — reconstructor counter caches were globally capacity-bounded by the unrelated
  `MAX_POLYS_PER_SSA + 1` constant.** Fixed as filed in `dbc351e304` (caches sized
  `max_concurrent_sessions × 3`, validated config field) — but the chosen default plus the
  missing plumbing from `maximum_sessions` made the practical bound far smaller than before,
  escalated as **MEDIUM-5** in the open list.

- **LOW-6 — `hopr_session_pix_current_ssa_phase` conflated concurrent SSAs** (keyed by
  `session_id` only, written from four unrelated call sites). Fixed in `dbc351e304`: keyed by
  `(session_id, ssa_index)`. Fallout (series lifecycle, dead `Recovering` value, METRICS.md,
  progress-gauge conflation) → LOW-9 in the open list.

- **LOW-7 — PIX gauge series created for non-PIX sessions; unbounded per-session close-reason
  counter labels.** Fixed in `dbc351e304`: `remove_session_metrics_state` takes a `has_pix`
  flag derived from the slot's gate, and `hopr_session_pix_close_reason_total` was removed in
  favor of the aggregate `hopr_session_pix_closures_total`. The clippy nit and the new
  series-lifetime issue → LOW-9 in the open list.

---

# Verified sound (no findings)

Re-verified on `dbc351e304` during the 2026-07-18 sweep:

- **Supervisor state machine** — the `recovered_pending` deferral under all event orderings
  (`Recovered` before/after commitment and deposit, with and without `AlmostRecovered`; each
  yields exactly one successor request and no spurious close); `closed` latching on every
  `Close` emission; `first_failure_reason` preservation including the drain path; deadline
  arithmetic (`checked_add` everywhere plus the 24 h config cap), with
  `validate_pix_supervision` called in both production paths (`SessionManager::start` and
  `HoprTransport::new`) — including the counter-TTL vs supervision-horizon cross-check.
- **`SlotNotify`** (utils.rs) — the generation-counter design is correct against both
  latent-wake and spurious-ready races; drop-based waker removal verified.
- **`ServiceGate`** — CAS on `served` prevents ceiling overshoot under concurrency;
  `release_service` watermark snapshot prevents predeposit traffic from counting against the
  post-funding ceiling (the ±1 window is harmless); register-then-recheck on every park path;
  pre-CAS poison re-checks on both branches; at-most-one-straggler poison semantics documented
  and acceptable.
- **Worker/driver teardown** — drop-safe in all traced orders: dropped `ActionRx` → worker
  poisons gate and exits; worker death → driver fallback poisons, retires SSAs, removes the
  slot with `ClosureReason::PixFailure`; setup failure before driver spawn →
  `SessionSlotGuard` rollback + `SsaCommitmentGuard` drop retires the initial SSA; cache
  eviction aborts the driver, whose RAII guards retire all tracked SSAs.
- **Deposit observer** — serial top-up semantics verified (each installment within
  `max_deposit_wait` of the last is forwarded; supervisor accumulates and decides sufficiency).
- **Reconstructor counting** — duplicate-before-count, surplus exclusion, one `Progress` per
  SSA per batch, cross-peer aggregate invalid totals matching the supervisor's monotonic-max
  delta-charging contract; `retire_ssa` covers all five internal structures;
  `DuplicateCommitment` prevents duplicate deposit observers.
- The keep-alive egress bypassing the gate is a documented, bounded exemption (≤1
  win-prob-scaled ticket/s for at most ~80 s on unfunded sessions) — reasonable.

---

# Reproduction commands

```bash
# HIGH-3 — broken test build (16 E0599 errors in surb_store.rs tests):
cargo check --tests -p hopr-protocol-hopr

# passing suites on dbc351e304:
cargo nextest run --lib -p hopr-transport-session -p hopr-protocol-pix   # 228/228
cargo nextest run --no-run -p hopr-lib --test transport_session_pix
cargo check -p hopr-protocol-session
cargo check -p hopr-transport-session --features telemetry
```

# Verification log (2026-07-18, `dbc351e304`)

- `cargo check -p hopr-protocol-session` / `-p hopr-transport-session` /
  `-p hopr-transport-session --features telemetry` ✅
- `cargo nextest run --lib -p hopr-transport-session -p hopr-protocol-pix` → **228/228** ✅
  (includes the committed regression tests `writer_on_poisoned_gate_errors_out`,
  `recovered_before_deposit_then_funded_session_survives`, `entry_rejects_exit_dictated_ssa_params`)
- `cargo nextest run --lib -p hopr-protocol-hopr` → ❌ **does not compile** (HIGH-3, 16 E0599
  errors in `surb_store.rs` tests)
- `cargo nextest run --no-run -p hopr-lib --test transport_session_pix` ✅ (new integration
  tests compile)
- `cargo clippy -p hopr-transport-session -p hopr-protocol-pix --all-targets` (also with
  `--features telemetry`) → warnings only; locations listed in the open nits and LOW-9
- Fix verification: MEDIUM-1 (adapter chain traced end-to-end, all four poison paths verified),
  HIGH-2 residual (exact dims comparison), LOW-4 (partial — tombstone gap remains), LOW-5
  (superseded by MEDIUM-5), LOW-6/LOW-7 (fixed with residuals → LOW-9)
- Fresh sweep: `recovered_pending` interleavings re-traced; `ServiceGate`/`SlotNotify`
  protocols re-verified; worker/driver/eviction teardown traced drop-safe; deposit-observer
  top-up semantics verified; reconstructor counter monotonicity, duplicate/surplus exclusion,
  and `retire_ssa` coverage verified; `validate_pix_supervision` confirmed called in both
  production paths
- One-off rustc ICE (LLVM ThinLTO stack overflow) on the first `hopr-transport-session` test
  build; not reproducible on retry — see open nits
