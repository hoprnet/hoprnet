# Code Review: `lukas/session-pix-supervisor` vs `origin/lukas/pix`

**Scope:** `git diff origin/lukas/pix...HEAD` — 31 files, ~7,000 insertions. The bulk is the new
Exit-side PIX supervision module (`transport/session/src/supervision/`: pure state machine,
lock-free `ServiceGate`, per-session worker actor), its integration into `SessionManager`
(action driver, deposit observer, egress gating), and the reconstructor extensions in
`protocols/pix` (progress/fault counters, RAII SSA retirement).

**Overall assessment:** the design is strong — the pure state machine, RAII
`SsaCommitmentGuard`/`SsaRetirementGuard`, the generation-counter `SlotNotify`, and the
drop-safe worker/driver teardown paths all hold up under tracing. Prior review rounds
(PD-02..PD-19, H-01..H-04, M-01..M-06) visibly hardened the code. Two high-severity issues
remain, both **confirmed with failing unit tests** (reproduced on this branch, then removed
from the tree; the tests are embedded below).

Findings are sorted by severity. Line numbers refer to the branch HEAD (`13af142125`).

---

## HIGH-1 — Terminal recovery events dropped outside `Recovering` are never re-synthesized: a fully recovered **and fully funded** session gets force-closed

**Files:**

- `transport/session/src/supervision/supervisor.rs:491-511` (`on_recovered`)
- `transport/session/src/supervision/supervisor.rs:454-489` (`on_almost_recovered`)
- `transport/session/src/supervision/supervisor.rs:328-385` (`on_deposit_confirmed` — no replay)

**Defect.** `on_recovered` ignores `Recovered` unless the SSA is in `Recovering`
(the M-02 ordering guard), with the comment _"the normal lifecycle transition will handle it
when those events arrive"_. But nothing handles it: the reconstructor emits `Recovered`
**exactly once** (subsequent shares for completed polynomials return `Surplus` and emit no
`Progress` either — see `SsaPartBuilder::add_share`, `protocols/pix/src/reconstructor/utils.rs:113-143`).
`on_deposit_confirmed` transitions the SSA to `Recovering` without checking whether recovery
already completed. The SSA is then permanently stuck in `Recovering` with no event source left.

**Failure scenario.** Share processing in the reconstructor starts as soon as the client
commitment is installed and is completely independent of the on-chain deposit. With realistic
dimensions (the integration tests use 2×2 → target of **4 useful shares**), the Entry's shares
complete recovery well within the deposit-confirmation latency (`max_deposit_wait` default 60 s):

1. `CommitmentVerified` → SSA in `AwaitingDeposit`.
2. All shares arrive; reconstructor emits `AlmostRecovered` + `Recovered` → **both dropped**
   (`Recovered` always; `AlmostRecovered` too if it races `CommitmentVerified` — the two travel
   via independent tasks into the worker's command channel, so ordering is not guaranteed).
3. Deposit confirms → SSA enters `Recovering`; idle deadline armed; **no further events will ever arrive**.
4. Exit keeps serving egress packets → at `now + max_recovery_idle` the service-gated idle check
   sees service-without-progress → `close_ssa_and_collect`.
   - If this was the only SSA (step 2's `AlmostRecovered` was missed in `AwaitingCommitment`, so
     no successor was ever requested): **the entire session closes with `RecoveryIdle`**, despite
     the Entry having fully paid and fully delivered.
   - If a successor exists: the healthy, paid SSA is closed/retired with a spurious warn-level
     `RecoveryIdle`, and `first_failure_reason` is permanently polluted, misreporting any later
     genuine close.
   - If no service is consumed, the idle timer re-arms forever and the SSA occupies reconstructor
     memory until the hard deadline (default **1 hour**), then the same close plays out via
     `RecoveryDeadline`.

**Failing test** (drop into `supervisor.rs`'s `mod tests`; fails on this branch with
`session must not be closed after full recovery + sufficient deposit, got actions: [Close(RecoveryIdle)]`):

```rust
/// A fully recovered and subsequently funded SSA must not lead to the
/// session being closed. Currently the `Recovered` event arriving during
/// `AwaitingDeposit` is dropped and never re-synthesized, so the SSA is
/// stuck in `Recovering` after the deposit confirms; with service being
/// consumed the idle deadline then closes the whole session.
#[test]
fn recovered_before_deposit_then_funded_session_survives() {
    let p = pseudonym();
    let start = Instant::now();
    let (mut sup, _) = SessionPixSupervisor::new(default_cfg(), dims(2, 2), p, start);
    let id = ssa_id(p, 1);

    sup.handle_event(&SessionPixEvent::SsaRequestSent(id), start, 0);
    sup.handle_event(
        &SessionPixEvent::CommitmentVerified { ssa_id: id, expected_deposit: None },
        start,
        0,
    );

    // Entry delivered all shares before the on-chain deposit confirmed
    // (target is only 4 useful shares here). Recovered arrives while
    // still AwaitingDeposit — dropped by the M-02 ordering guard.
    sup.handle_event(&SessionPixEvent::Recovered(id), start, 0);

    // The deposit confirms shortly after.
    sup.handle_event(
        &SessionPixEvent::DepositConfirmed { ssa_id: id, amount: sufficient_balance() },
        start,
        10,
    );
    assert!(!sup.closed, "session must be alive after funding");

    // Service continues to be consumed; the recovery-idle deadline fires.
    let actions = sup.handle_deadline(start + Duration::from_secs(61), 100);

    assert!(
        !sup.closed,
        "session must not be closed after full recovery + sufficient deposit, got actions: {actions:?}"
    );
}
```

Observed failure:

```
thread '...' panicked at transport/session/src/supervision/supervisor.rs:
session must not be closed after full recovery + sufficient deposit, got actions: [Close(RecoveryIdle)]
```

**Suggested mitigation.** Record the dropped terminal events on `PerSsaState` instead of
discarding them, and replay them when the phase catches up:

- Add `recovered_pending: bool` and extend the existing `next_request_pending_deposit`
  mechanism to also cover `AwaitingCommitment` (`on_almost_recovered` currently sets no flag in
  that phase).
- In `on_recovered` for `AwaitingCommitment`/`AwaitingDeposit`: set `recovered_pending = true`
  (do not transition yet — funding must still be enforced).
- In `on_deposit_confirmed` (and `on_commitment_verified`, for the commitment→deposit hop):
  after the normal transition, if `recovered_pending`, immediately perform the
  `on_recovered` transition (tombstone + request-next-if-needed).

The existing tests `recovered_before_commitment_is_ignored` / `recovered_before_deposit_is_ignored`
only pin "no immediate action", so the replay behavior can be added without breaking them.

---

## HIGH-2 — Entry-side PIX parameter verification is a tautology: the Exit dictates the quota (regression vs base branch)

**Files:**

- `transport/session/src/manager.rs:1135-1140` (`new_session` computes `ssa_params` but never stores them in the slot)
- `transport/session/src/manager.rs:2718-2737` (`handle_ssa_request` initializes `ssa_params` **from the Exit's own message**, then compares the message against itself)
- `transport/session/src/manager.rs:2742-2743` (dead `_negotiated_polys`/`_negotiated_shares` — remnants of the lost check)

**Defect.** On the base branch, `new_session()` stored the Entry's offered PIX params in the
slot (`current_ssa_state.set(SessionSsaState::new(...))`) and `handle_session_ssa_request`
rejected an Exit response whose quota didn't match. On this branch, `new_session()` computes
`ssa_params` only for encoding `additional_data` and leaves the slot's `ssa_params` OnceLock
**empty**; the first `SsaRequest` then does:

```rust
let _ = session_slot.ssa_params.get_or_init(|| SessionSsaParameters {
    polys_per_ssa: msg.polys_per_ssa(),
    shares_per_poly: msg.shares_per_poly(),
});
let quota_per_ssa = session_slot.ssa_params.get().unwrap().quota_per_ssa();
let server_quota = pix_params_to_quota(msg.polys_per_ssa(), msg.shares_per_poly());
if quota_per_ssa != server_quota { /* unreachable on first SsaRequest */ }
```

`quota_per_ssa == server_quota` by construction, so the "Exit sent unacceptable quota" error can
never fire. There is also no bounds check against `MAX_POLYS_PER_SSA` / `MAX_POLY_THRESHOLD`
(the Exit side has one in `check_pix_params`; the Entry side has none).

**Failure scenario.** A malicious or buggy Exit answers the session initiation with arbitrary
dimensions. The Entry silently accepts them and emits `HoprSessionOutPixEvent::ReadyToDeposit`
with the **Exit-chosen** `quota_per_ssa` — the value the upper layer uses to decide the deposit.
The Exit can thus unilaterally change the price-per-byte the Entry believes it negotiated
(e.g. shrink the quota so each deposit buys far less traffic than offered).

**Failing test** (drop into `manager.rs`'s `mod tests`; fails on this branch with
`got Ok(())` — the Entry accepted `u16::MAX × u16::MAX` and emitted `ReadyToDeposit` with
`quota_per_ssa=4458040001550`, ≈ 4.4 TB, never offered):

```rust
#[test_log::test(tokio::test)]
async fn entry_rejects_exit_dictated_ssa_params() -> anyhow::Result<()> {
    use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};

    let ssa_gen_config = SsaGeneratorConfig {
        polynomials_per_ssa: 2,
        threshold: 2,
        surplus_shares: 1,
    };

    let reconstructor: Arc<SsaReconstructor<HoprPixSpec>> =
        Arc::new(SsaReconstructor::new(SsaReconstructorConfig::default()));
    let (pix_toolbox, _pix_events) =
        PixToolbox::new(SsaShareGenerator::new(ssa_gen_config).into(), reconstructor.clone());

    let mgr = SessionManager::new(SessionManagerConfig::default());

    let mut transport = MockMsgSender::new();
    transport
        .expect_send_message()
        .returning(|_, _| Box::pin(async { Ok(()) }));
    let (sender, _handle) = mock_packet_planning(transport);
    let (new_session_tx, _new_session_rx) = futures::channel::mpsc::channel(1);
    mgr.start(sender, new_session_tx, Some(pix_toolbox))?;

    // Simulate the slot left behind by `new_session()` on the Entry:
    // the Entry offered (2, 2), but the slot's `ssa_params` are EMPTY.
    let pseudonym = HoprPseudonym::random();
    let (dummy_tx, _dummy_rx) =
        crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(SESSION_FORWARD_CAPACITY);
    mgr.sessions.insert(
        pseudonym,
        SessionSlot {
            session_tx: dummy_tx,
            routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(pseudonym)),
            abort_handles: Default::default(),
            surb_mgmt: Default::default(),
            surb_estimator: Default::default(),
            ssa_params: Default::default(),
            pix_supervisor: Default::default(),
            pix_egress_gate: Default::default(),
        },
    );

    // A well-formed exit commitment so processing reaches the quota check and beyond.
    let ssa_index = SsaIndex::new(1).expect("non-zero");
    let exit_commitment = reconstructor.new_exit_commitment(SsaId::new(pseudonym, ssa_index), 2, 2)?;
    let exit_commitment = HoprPixGroupElement(exit_commitment.to_bytes());

    // The Exit dictates dimensions the Entry never offered — far beyond the
    // protocol maxima (MAX_POLYS_PER_SSA = 16192, MAX_POLY_THRESHOLD = 4096).
    let msg = SsaServerCommitmentMessage::new(pseudonym, u16::MAX, u16::MAX, [(ssa_index, exit_commitment)]);

    let result = mgr.handle_ssa_request(pseudonym, msg).await;
    assert!(
        result.is_err(),
        "Entry must reject Exit-dictated PIX params it never offered / beyond protocol bounds, got {result:?}"
    );

    Ok(())
}
```

Observed failure:

```
thread '...' panicked at transport/session/src/manager.rs:
Entry must reject Exit-dictated PIX params it never offered / beyond protocol bounds, got Ok(())
-- preceded by --
INFO ... generated client SSA commitment and deposit address ssa_index=1
     deposit_address=0xc7bf... quota_per_ssa=4458040001550
```

**Suggested mitigation.**

1. In `new_session()`, store the offered params in the slot again:
   `let _ = slot.ssa_params.set(params)` for PIX-capable outgoing sessions (restores the base-branch
   behavior with the new type).
2. In `handle_ssa_request`, remove the `get_or_init`-from-message; require `ssa_params` to be
   already populated (error out otherwise) and keep the quota comparison.
3. Additionally bounds-check `msg.polys_per_ssa() <= MAX_POLYS_PER_SSA` and
   `(2..=MAX_POLY_THRESHOLD).contains(&msg.shares_per_poly())` as defense in depth, mirroring
   `check_pix_params`.
4. Delete the dead `_negotiated_polys`/`_negotiated_shares` bindings.

---

## MEDIUM-1 — Egress writes against a poisoned gate hang forever instead of erroring; `CrossfireSinkError::GateClosed` is dead code

**Files:**

- `transport/session/src/manager.rs:2064-2077` and `:2181-2196` (egress adapters:
  `if g.acquire().await.is_err() { futures::future::pending::<()>().await }`)
- `utils/src/network_types/crossfire_sink.rs:16-19` (`GateClosed` variant — never constructed anywhere)
- `transport/session/src/manager.rs:130`, `:2331`, `:2348` (comments claiming _"parked writers get `GateClosed`"_)

**Defect.** When the supervisor closes a PIX session, the gate is poisoned and any egress write
enters `futures::future::pending()` — an await that never resolves and never errors. The write
future lives in the session user's task (whoever `.await`s a write into `HoprSession`), which
the manager's `abort_handles` do **not** cover; the inline comment ("the task will be aborted by
session teardown") only holds for manager-spawned pumps. A caller that only writes (one-way
egress streaming, no read loop observing EOF) hangs indefinitely with no error and no timeout.
The `CrossfireSinkError::GateClosed` variant added in this branch suggests the original intent
was to surface an error; it is never used, and three comments describing the behavior are wrong.

**Suggested mitigation.** Surface the poison as a sink error instead of parking: e.g. make the
egress adapter's closure return `Err` by mapping through a concrete sink-error type (the sink is
generic over `S::Error`, so either constrain `S::Error: From<GateClosed>`-style, or wrap the
egress sink in a fallible adapter closer to where the concrete sink type is known). At minimum,
fix the comments and remove the dead enum variant so the actual behavior (hang until the caller
drops the session in reaction to the closure notification) is documented truthfully.

---

## MEDIUM-2 — Deposit observer logs an error and self-terminates 60 s after every _successful_ deposit; later top-ups are lost

**File:** `transport/session/src/manager.rs:2585-2648` (observer loop in `handle_ssa_commit` path)

**Defect.** The observer loops forever with a per-iteration timeout of `max_deposit_wait` "to
support top-up deposits". After the first (sufficient) deposit is forwarded, the SSA moves to
`Recovering`, no further deposits arrive, and 60 s later the timeout branch fires:

```rust
Err(_) => {
    error!(%session_id, "deposit confirmation timed out; check deposit address and funding");
    ... send_event(SessionPixEvent::DepositObserverClosed(...)) ...
    break;
}
```

The supervisor correctly ignores `DepositObserverClosed` outside `AwaitingDeposit`, so no close
happens — but **every healthy funded session emits a misleading error-level log** exactly
`max_deposit_wait` after funding, and the observer then exits, so with a configured
`min_deposit` any second top-up installment arriving more than `max_deposit_wait` after the
first is silently never forwarded (the supervisor's deposit deadline then closes a session the
user believes they topped up).

**Suggested mitigation.** Track whether at least one deposit was forwarded; on timeout after a
forwarded deposit, exit silently (`debug!`) without sending `DepositObserverClosed`. If split
top-up payments are meant to be supported, arm the per-iteration timeout only while the
accumulated amount is insufficient (or let the `RetireSsa` abort be the sole terminator and
drop the timeout entirely once a deposit was seen).

---

## MEDIUM-3 — `std::sync::Mutex` + `expect("lock poisoned")` in reconstructor counters (project-rule violation, poison-cascade panic)

**File:** `protocols/pix/src/reconstructor/mod.rs` — `SsaCounterEntry` storage
(`ssa_counters: moka::sync::Cache<_, Arc<std::sync::Mutex<SsaCounterEntry>>>`) and all accessors
(`record_useful_share`, `record_completed_part`, `record_invalid_share`, `snapshot_progress`,
the `invalid_ssas` aggregation in `acknowledge_shares`).

**Defect.** The project rules (`.claude/INSTRUCTIONS.md`, "Common Mistakes #1") mandate
`parking_lot::Mutex` over `std::sync::Mutex`; the rest of this file uses `parking_lot`. Beyond
style: every access does `.lock().expect("ssa counter lock poisoned")`. If any panic ever occurs
while a counter lock is held, the lock is poisoned and **every subsequent acknowledgement batch
for that SSA panics** inside the packet pipeline — a single fault escalates into a repeating
pipeline crash. `record_invalid_share` also returns the per-peer count that no caller uses.

**Suggested mitigation.** Use `parking_lot::Mutex` (no poisoning, matches surrounding code and
project rules); drop the unused return value of `record_invalid_share`.

---

## MEDIUM-4 — `hopr-protocol-session` no longer compiles standalone: `runtime-tokio` removed from its `hopr-utils` dependency while `socket/` still calls `spawn` unconditionally

**Files:**

- `protocols/session/Cargo.toml` (branch change: `hopr-utils = { workspace = true, features = ["network-types"] }`, previously also `"runtime-tokio"`)
- `protocols/session/src/socket/ack_state.rs:275,314,334` and `protocols/session/src/socket/mod.rs:332`
  (`hopr_utils::runtime::prelude::spawn` — configured out without a runtime feature)

**Defect.** The runtime-agnosticism refactor (commit `3cc0efd920`) removed the unconditional
`runtime-tokio` feature from `hopr-protocol-session`'s `hopr-utils` dependency, but four
`spawn` call sites in the socket module remain unconditional. The crate's own `runtime-tokio`
feature is not in `default`, so:

```
$ cargo check -p hopr-protocol-session      # also: cargo check -p hopr-transport-session
error[E0425]: cannot find function `spawn` in module `hopr_utils::runtime::prelude`
  --> protocols/session/src/socket/ack_state.rs:275:43
... (4 errors)
```

Workspace-level builds and test builds pass only because _other_ crates in the unified feature
graph happen to enable `hopr-utils/runtime-tokio` — so CI can stay green while every standalone
package check (the project's own prescribed verification step) and any downstream consumer using
default features is broken. This compiled on the base branch.

**Suggested mitigation.** Either gate the socket-module `spawn` call sites (and whatever they
spawn) behind `#[cfg(feature = "runtime-tokio")]` with a compile error or fallback for
runtime-less builds, or make `hopr_utils::runtime::prelude::spawn` available runtime-agnostically
(which was the stated goal of the crossfire refactor), or add `runtime-tokio` to the crate's
default features until the socket module is actually runtime-agnostic.

---

## LOW-1 — `handle_deadline`'s `NoSsaRemaining` close ignores `first_failure_reason`

**File:** `transport/session/src/supervision/supervisor.rs:227-231`

`close_ssa_and_collect` carefully preserves the _first_ failure reason for the final `Close`,
but the drain path pushes `Close(SessionPixCloseReason::NoSsaRemaining)` directly. Sequence:
SSA 2's commitment times out (`first_failure_reason = CommitmentTimeout`, SSA removed), SSA 1
later tombstones and expires → session closes reporting `NoSsaRemaining`, hiding the actual
cause (and the `hopr_session_pix_closures_total` metric is binned under the wrong reason).

**Mitigation:** `actions.push(SessionPixAction::Close(self.first_failure_reason.unwrap_or(SessionPixCloseReason::NoSsaRemaining)))`.

---

## LOW-2 — `emit_request_next_ssa` emits `Close` without setting `self.closed` on the `SsaIndex` conversion failure branch

**File:** `transport/session/src/supervision/supervisor.rs:646-654`

The `SsaIndex::try_from(index)` error branch returns `Close(InvalidTransition)` but does not set
`self.closed = true`, unlike the `checked_add` overflow branch directly below. Until the action
driver feeds back the `Close` result, the supervisor keeps accepting events and its deadlines
stay armed. Practically unreachable today (`next_ssa_index` starts at 1 and only increments),
but inconsistent fail-closed hygiene. Same pattern in `action_result` for
`RequestSsa { ok: false }` (`supervisor.rs:267-270`), where the emitted `Close` also relies on
the driver's feedback loop to latch `closed`.

**Mitigation:** set `self.closed = true` whenever a `Close` action is emitted.

---

## LOW-3 — `ServiceGate::acquire` predeposit path lacks the pre-CAS poison re-check the funded path has

**File:** `transport/session/src/supervision/gate.rs:118-132` (compare with `:101-105`)

The funded path re-checks `poisoned` immediately before the CAS ("so that a concurrent `poison()`
is not missed between the entry check and here"); the predeposit path does not, so a packet can
be admitted after `poison()` returns. The documented at-most-one-straggler semantics arguably
cover it, but the asymmetry is unintentional — add the same re-check before the predeposit CAS.

---

## LOW-4 — Stale `SsaRequestSent` can resurrect a closed SSA in the supervisor

**File:** `transport/session/src/supervision/supervisor.rs:283-301`

`on_ssa_request_sent` creates fresh `AwaitingCommitment` state for **any** SSA id with a matching
pseudonym that is not currently tracked. An SSA that was already closed and removed (e.g.
commitment timeout with a live sibling) would be re-created with a fresh deadline if a duplicate
`SsaRequestSent` ever arrived. The current driver sends the event exactly once per request, so
this is latent — but the guard comment claims idempotency while actually being
"idempotent only while the state still exists".

**Mitigation:** only accept `ssa_id.ssa_index() < next_ssa_index` _and_ not previously closed
(track a small set/watermark of retired indices), or restrict acceptance to the index of the
most recent `RequestSsa` action.

---

## LOW-5 — Reconstructor counter caches are globally capacity-bounded by an unrelated constant

**File:** `protocols/pix/src/reconstructor/mod.rs` (`ssa_counters` / `ssa_to_verifier_ids`
built with `CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)`)

The comment admits the capacity (`≈16k entries`) is a "generous per-SSA estimate" repurposed as
a global bound across _all_ sessions. If concurrent live SSAs ever exceed it, moka silently
evicts counters for live sessions: progress snapshots stop (supervisor sees a stall →
`RecoveryIdle`/ceiling park) and `retire_ssa`'s verifier cleanup loses its id list (verifiers
then only fall away via TTI). Not reachable at today's `maximum_sessions` defaults, but the
bound should be derived from `maximum_sessions × 3` (2 live + 1 tombstone SSA per session) or
passed in explicitly.

---

## Notes (no action required)

- **`SlotNotify`** (utils.rs): generation-counter design is correct against both latent-wake and
  spurious-ready races; drop-based waker removal verified. Nice primitive.
- **`ServiceGate`** budget/ceiling accounting: CAS on `served` prevents ceiling overshoot under
  concurrency; `release_service` watermark snapshot prevents predeposit traffic from counting
  against the post-funding ceiling. The tiny window where a predeposit acquirer has decremented
  `remaining` but not yet bumped `served` when `release_service` snapshots is harmless (±1).
- **Worker/driver teardown** is drop-safe in all traced orders: dropped `ActionRx` → worker
  poisons gate and exits; worker death → driver's `recv` errors → gate poisoned, SSAs retired,
  slot removed with `ClosureReason::PixFailure`; setup failure before driver spawn →
  `SessionSlotGuard` rollback + `SsaCommitmentGuard` drop retires the initial SSA.
- **Reconstructor counting** (duplicate-before-count, surplus exclusion, one `Progress` per SSA
  per batch, cross-peer aggregate invalid totals) matches the supervisor's monotonic-max
  delta-charging contract; verified against the new unit tests.
- The keep-alive egress bypassing the gate is a documented, bounded exemption (≤1 ticket/s for
  ≤ ~80 s on unfunded sessions) — reasonable.

## Reproduction commands

```bash
# after inserting the HIGH-1 test into supervisor.rs's test module:
cargo nextest run -p hopr-transport-session --lib recovered_before_deposit_then_funded

# after inserting the HIGH-2 test into manager.rs's test module:
cargo nextest run -p hopr-transport-session --lib entry_rejects_exit_dictated

# MEDIUM-4 reproduces directly:
cargo check -p hopr-protocol-session
```
