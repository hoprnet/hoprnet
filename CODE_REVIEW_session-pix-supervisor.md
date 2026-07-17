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

Findings are sorted by severity. Line numbers in the original findings refer to `13af142125`.

---

## Re-review (2026-07-17, branch HEAD `21c876cde4`)

Two commits landed after the original review: `b3154a3af7` (per-session PIX supervisor
telemetry) and `21c876cde4` (mitigations for the findings below, including committed
regression tests for HIGH-1 and HIGH-2). Each fix was re-verified against the actual code, the
full unit suites were re-run (`hopr-transport-session`: 192 passed, `hopr-protocol-pix`:
34 passed), `cargo check -p hopr-protocol-session` / `-p hopr-transport-session` /
`--features telemetry` all pass, and clippy is warning-only. Additionally, three scratch tests
probing HIGH-1 event orderings _not_ covered by the committed regression test
(`Recovered` before `CommitmentVerified`; `AlmostRecovered`+`Recovered` both before the
deposit; `AlmostRecovered` before `CommitmentVerified`) all pass — the deferral logic holds
the "exactly one successor request" invariant on every path.

| Finding  | Status                   | Notes                                                                                                                                           |
| -------- | ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| HIGH-1   | ✅ Fixed                 | `recovered_pending` deferral + replay in `on_deposit_confirmed`; regression test committed; extra orderings verified                            |
| HIGH-2   | ✅ Fixed                 | Params stored in `new_session`, bounds check + quota check restored; regression test committed. Residual: dims not compared exactly (see below) |
| MEDIUM-1 | ❌ **Still open**        | Only comments changed; a user-held writer still hangs forever — **new failing test embedded below**                                             |
| MEDIUM-2 | ✅ Fixed                 | `deposit_forwarded` flag → silent observer exit after first forwarded deposit                                                                   |
| MEDIUM-3 | ✅ Fixed                 | `parking_lot::Mutex` swap; unused return of `record_invalid_share` dropped                                                                      |
| MEDIUM-4 | ✅ Fixed (with residual) | Standalone checks pass; but the `default` feature addition is inert for workspace consumers (see below)                                         |
| LOW-1    | ✅ Fixed                 | `first_failure_reason.unwrap_or(NoSsaRemaining)` in the drain path                                                                              |
| LOW-2    | ✅ Fixed                 | `self.closed = true` latched in both branches                                                                                                   |
| LOW-3    | ✅ Fixed                 | Pre-CAS poison re-check added to the predeposit path                                                                                            |
| LOW-4    | ❌ Open                  | `on_ssa_request_sent` unchanged — still re-creates state for any untracked SSA id                                                               |
| LOW-5    | ❌ Open                  | All four caches still capacity-bounded by `MAX_POLYS_PER_SSA + 1`                                                                               |
| LOW-6    | 🆕 New                   | Per-session SSA-phase gauge conflates concurrent SSAs (telemetry commit)                                                                        |
| LOW-7    | 🆕 New                   | Metrics lifecycle: PIX series created for non-PIX sessions; unbounded per-session close-reason counter labels                                   |

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

> **Re-review status: ✅ Fixed in `21c876cde4`.** Implemented exactly as suggested:
> `PerSsaState.recovered_pending` is set when `Recovered` arrives in `AwaitingCommitment`/
> `AwaitingDeposit` (the flag survives `on_commitment_verified` untouched) and
> `on_deposit_confirmed` replays it via the new `perform_recovered_transition`.
> `on_almost_recovered`'s `AwaitingCommitment` arm now also defers the successor request.
> Verified invariants: every site that sets `next_request_pending_deposit` also sets
> `next_requested`, so the replay path and the `pending` path can never both emit
> `RequestSsa` (no double successor); `on_deposit_confirmed` / `on_almost_recovered` are
> guarded by phase / `is_terminal()`, so tombstones cannot be resurrected or trigger stray
> requests. The regression test `recovered_before_deposit_then_funded_session_survives` is
> committed and passes. During re-review three additional scratch tests confirmed the
> orderings the committed test doesn't cover (`Recovered` before `CommitmentVerified`;
> `AlmostRecovered`+`Recovered` before deposit; `AlmostRecovered` before
> `CommitmentVerified`) all yield exactly one successor request and no `RecoveryIdle` close.
> _Optional hardening:_ commit the `Recovered`-before-`CommitmentVerified` variant as a test,
> since that deferral hop (flag surviving two phase transitions) is otherwise untested.

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

> **Re-review status: ✅ Fixed in `21c876cde4`.** All four points implemented:
> `new_session` pre-populates the slot's `ssa_params` OnceLock with the offered params (shared
> across both slot-construction sites), `handle_ssa_request` errors out when the lock is empty
> ("PIX was not negotiated"), the `MAX_POLYS_PER_SSA` / `MAX_POLY_THRESHOLD` bounds check is in
> place, and the dead bindings are gone. The regression test
> `entry_rejects_exit_dictated_ssa_params` is committed and passes.
>
> **Residual (LOW): dims are still compared via the quota product, which is not injective.**
> `pix_params_to_quota(polys, shares) = polys × shares × PAYLOAD_SIZE`
> (`transport/session/src/types.rs:94`), so an Exit answering with _transposed_ dimensions
> (e.g. offered 4×2, answered 2×4 — same product, within bounds) still passes. The quota the
> Entry reports upward now comes from `our_params`, so the price can no longer be inflated —
> the residual effect is only a dims mismatch between the Entry's generator and the Exit's
> claimed reconstructor dims (share verification then fails Exit-side → recovery stalls →
> session closes after the Entry deposited; griefing power a malicious Exit already has).
> Since the offered dims are now stored in the slot, compare them exactly:
> `our_params.polys_per_ssa != msg.polys_per_ssa() || our_params.shares_per_poly != msg.shares_per_poly()`
> — strictly stronger and makes the quota comparison redundant.

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

> **Re-review status: ❌ Still open** (`21c876cde4` changed only the comments). The behavior is
> unchanged: a writer holding the `HoprSession` object parks forever on `pending()` after the
> supervisor closes the session. Worse, the updated comments now claim _"they will hang on
> `pending()` until the session teardown aborts their task"_ — which is only true for
> manager-spawned pump tasks tracked in `abort_handles`; the user's own task holding the
> session write half is never aborted by teardown. `CrossfireSinkError::GateClosed` was kept
> as `#[allow(dead_code)]` instead of removed or used.
>
> **Failing test** (drop into `manager.rs`'s `mod tests`; fails on `21c876cde4` with
> `write into a PIX-closed session must error out, got Err(Elapsed(()))` — the write+flush
> never resolves and the 2 s timeout elapses):
>
> ```rust
> /// A writer holding the session object must observe an error (not hang
> /// forever) when the PIX supervisor closes the session and poisons the
> /// egress gate. Currently the egress adapter parks on `pending()`, so a
> /// one-way writer that never reads hangs indefinitely.
> #[test_log::test(tokio::test)]
> async fn writer_on_poisoned_gate_errors_out() -> anyhow::Result<()> {
>     use futures::AsyncWriteExt;
>     use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
>     use hopr_protocol_start::StartInitiation;
>
>     let ssa_gen_config = SsaGeneratorConfig {
>         polynomials_per_ssa: 2,
>         threshold: 2,
>         surplus_shares: 1,
>     };
>
>     let (pix_toolbox, _) = PixToolbox::new(
>         SsaShareGenerator::new(ssa_gen_config).into(),
>         SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
>     );
>
>     let mgr = SessionManager::new(SessionManagerConfig {
>         pix_config: IncomingSessionPixConfig {
>             quota_range: 0..=1024 * 1024 * 1024,
>             supervisor_cfg: SupervisorConfig {
>                 max_unverifiable_shares_per_session: 0,
>                 ..Default::default()
>             },
>             ..Default::default()
>         },
>         ..Default::default()
>     });
>
>     let mut bob_transport = MockMsgSender::new();
>     bob_transport
>         .expect_send_message()
>         .returning(|_, _| Box::pin(async { Ok(()) }));
>
>     let (bob_sender, _bob_handle) = mock_packet_planning(bob_transport);
>     let (new_session_tx, mut new_session_rx) = futures::channel::mpsc::channel(1);
>     mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;
>
>     let alice_pseudonym = HoprPseudonym::random();
>     mgr.handle_incoming_session_initiation(
>         alice_pseudonym,
>         StartInitiation {
>             challenge: MIN_CHALLENGE,
>             target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
>             capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
>             additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
>         },
>     )
>     .await?;
>
>     // This is the session object the target/user task owns and writes into.
>     let mut session = new_session_rx.next().await.expect("incoming session").session;
>
>     // Dispatch one unverifiable share → triggers close (max=0).
>     let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(1).expect("non-zero"));
>     mgr.dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShares {
>         ssa_id,
>         observed_total: 1,
>     })
>     .await?;
>
>     // Yield to let the action driver process the close.
>     tokio::time::sleep(std::time::Duration::from_millis(50)).await;
>     assert!(mgr.active_sessions().is_empty(), "session must be closed");
>
>     // A write + flush into the closed session must resolve with an error
>     // within a bounded time; it must not park forever.
>     let payload = vec![0xAB_u8; 4096];
>     let res = tokio::time::timeout(std::time::Duration::from_secs(2), async {
>         session.write_all(&payload).await?;
>         session.flush().await
>     })
>     .await;
>
>     assert!(
>         matches!(res, Ok(Err(_))),
>         "write into a PIX-closed session must error out, got {res:?}"
>     );
>
>     Ok(())
> }
> ```
>
> Observed failure:
>
> ```
> ERROR hopr_transport_session::manager: pix supervisor closed session ... reason=TooManyUnverifiableShares
> thread '...' panicked at transport/session/src/manager.rs:
> write into a PIX-closed session must error out, got Err(Elapsed(()))
> ```

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

> **Re-review status: ✅ Fixed in `21c876cde4`** via the `deposit_forwarded` flag (first
> mitigation option): after any forwarded deposit, the timeout branch exits with `debug!` and
> no `DepositObserverClosed`. Two accepted consequences, both benign: (1) a top-up installment
> arriving more than `max_deposit_wait` after the first is still unobserved — inherent to this
> mitigation option; (2) when the _first_ deposit is below `min_deposit`, the observer now also
> exits silently, but the supervisor's deposit deadline stays armed (`on_deposit_confirmed`
> returns early on insufficiency without clearing it — verified), so the session still closes,
> with `DepositTimeout` instead of `DepositObserverClosed`.

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

> **Re-review status: ✅ Fixed in `21c876cde4`.** All lock sites converted to
> `parking_lot::Mutex` (no `std::sync::Mutex` remains in the file) and the unused return value
> of `record_invalid_share` was removed.

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

> **Re-review status: ✅ Fixed in `21c876cde4`** (third mitigation option) — verified:
> `cargo check -p hopr-protocol-session` and `cargo check -p hopr-transport-session` both pass.
>
> **Residual worth knowing:** the workspace root pins
> `hopr-protocol-session = { ..., default-features = false }` (`Cargo.toml:209`), so the
> `default = [..., "runtime-tokio"]` addition is **inert for every workspace consumer** — what
> actually fixed `-p hopr-transport-session` is the explicit `"runtime-tokio"` added to its
> dependency features in `transport/session/Cargo.toml`. The default addition only helps the
> standalone check and external consumers using default features. The four unconditional
> `spawn` call sites in `socket/` remain, so the crate is still not runtime-agnostic: any new
> workspace consumer that doesn't (transitively) enable `runtime-tokio` will hit the same
> compile break. Fine as a stopgap; the `#[cfg]`-gating of the spawn sites is still the real fix.

---

## LOW-1 — `handle_deadline`'s `NoSsaRemaining` close ignores `first_failure_reason`

**File:** `transport/session/src/supervision/supervisor.rs:227-231`

`close_ssa_and_collect` carefully preserves the _first_ failure reason for the final `Close`,
but the drain path pushes `Close(SessionPixCloseReason::NoSsaRemaining)` directly. Sequence:
SSA 2's commitment times out (`first_failure_reason = CommitmentTimeout`, SSA removed), SSA 1
later tombstones and expires → session closes reporting `NoSsaRemaining`, hiding the actual
cause (and the `hopr_session_pix_closures_total` metric is binned under the wrong reason).

**Mitigation:** `actions.push(SessionPixAction::Close(self.first_failure_reason.unwrap_or(SessionPixCloseReason::NoSsaRemaining)))`.

> **Re-review status: ✅ Fixed in `21c876cde4`** exactly as suggested.

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

> **Re-review status: ✅ Fixed in `21c876cde4`** — both the `SsaIndex::try_from` error branch
> in `emit_request_next_ssa` and the `RequestSsa { ok: false }` branch in `action_result` now
> latch `self.closed = true` before emitting `Close`.

---

## LOW-3 — `ServiceGate::acquire` predeposit path lacks the pre-CAS poison re-check the funded path has

**File:** `transport/session/src/supervision/gate.rs:118-132` (compare with `:101-105`)

The funded path re-checks `poisoned` immediately before the CAS ("so that a concurrent `poison()`
is not missed between the entry check and here"); the predeposit path does not, so a packet can
be admitted after `poison()` returns. The documented at-most-one-straggler semantics arguably
cover it, but the asymmetry is unintentional — add the same re-check before the predeposit CAS.

> **Re-review status: ✅ Fixed in `21c876cde4`** — the predeposit path now has the same
> pre-CAS `poisoned` re-check as the funded path.

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

> **Re-review status: ❌ Open** — `on_ssa_request_sent` is unchanged in `21c876cde4` (still
> latent: the driver emits the event exactly once per request today).

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

> **Re-review status: ❌ Open** — all four caches in `21c876cde4` are still built with
> `CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)`
> (`protocols/pix/src/reconstructor/mod.rs:152-173`).

---

## LOW-6 (new, `b3154a3af7`) — `hopr_session_pix_current_ssa_phase` conflates concurrent SSAs and is wired from four unrelated call sites

**Files:**

- `transport/session/src/telemetry.rs:408-420` (`PixSsaPhase`, `set_pix_current_ssa_phase`)
- `transport/session/src/manager.rs` — call sites: supervisor setup (`AwaitingCommitment`, first
  SSA only), `handle_ssa_commit` (`AwaitingDeposit`, any SSA), the action driver's
  `ReleaseService` arm (`Recovering`, fires **once per session**), `dispatch_pix_event`
  (`Recovered`, any SSA)

**Defect.** The gauge is keyed by `session_id` only, but a PIX session holds up to 2 live + 1
tombstone SSAs concurrently (supervisor invariant). The four write sites each describe a
_different_ SSA, so during normal SSA overlap the value flip-flops between phases of different
SSAs: after SSA 1 recovers the gauge reads `Recovered (3)` while the active successor is mid
`AwaitingDeposit`/`Recovering`; successor SSAs never produce `Recovering` at all (that value is
only written from `ReleaseService`, which is emitted once per session for the first funding).
The metric can't be used for the phase-monitoring purpose its name and help text suggest.

**Suggested mitigation.** Either key the gauge additionally by `ssa_index` (and clear the
series on `RetireSsa`), or derive the value in a single place — e.g. the action driver
reporting "phase of the newest live SSA" from a supervisor snapshot — instead of scattering
best-effort writes across four handlers.

**Failing unit test:** not applicable — every call site is compiled out under
`#[cfg(all(feature = "telemetry", not(test)))]`, so the behavior is untestable from unit tests
by construction.

---

## LOW-7 (new, `b3154a3af7`) — metrics lifecycle: PIX series created for non-PIX sessions; per-session close-reason counter labels grow unboundedly

**Files:**

- `transport/session/src/telemetry.rs:306-311` (`remove_session_metrics_state`)
- `transport/session/src/telemetry.rs:186-190` + `:398-401`
  (`hopr_session_pix_close_reason_total` / `record_pix_close_reason`)

**Defect.**

1. `remove_session_metrics_state` unconditionally does `.set(&[session_id], 0.0)` on the three
   new PIX gauges for **every** closing session. Setting a labeled gauge creates the series, so
   every plain non-PIX session now spawns three `hopr_session_pix_*` series (value 0) at
   teardown — cardinality pollution and misleading dashboards ("this session had a PIX gate in
   predeposit mode").
2. `hopr_session_pix_close_reason_total` is a `MultiCounter` keyed by `session_id × reason`.
   Counters cannot be unregistered, so one series accumulates per closed PIX session for the
   node's lifetime. Each session closes exactly once with exactly one reason, and the
   aggregate `hopr_session_pix_closures_total` (by reason) plus the existing error-level log
   line (with `session_id` and `reason`) already carry the same information.

**Suggested mitigation.** Guard the PIX-gauge zeroing on the slot actually having PIX state
(e.g. `pix_egress_gate.get().is_some()`); drop `hopr_session_pix_close_reason_total` in favor
of the aggregate + log, or make it a per-session gauge that `remove_session_metrics_state`
clears.

**Failing unit test:** not applicable — same `cfg(not(test))` gating as LOW-6.

**Nit (same commit):** `set_pix_recovery_progress(&session_id, …)` and
`set_pix_current_ssa_phase(&session_id, …)` in `dispatch_pix_event` (manager.rs:1680, 1695)
trigger clippy `needless_borrow`-family warnings under `--features telemetry`.

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

> **Re-review update:** the HIGH-1 and HIGH-2 tests are now **committed** (in `21c876cde4`) and
> pass — they serve as regression tests. The MEDIUM-4 check passes. The only failing repro left
> is MEDIUM-1:

```bash
# still-open MEDIUM-1 — after inserting the writer-hang test into manager.rs's test module:
cargo nextest run -p hopr-transport-session --lib writer_on_poisoned_gate
# fails with: write into a PIX-closed session must error out, got Err(Elapsed(()))

# committed regression tests (now passing):
cargo nextest run -p hopr-transport-session --lib recovered_before_deposit_then_funded
cargo nextest run -p hopr-transport-session --lib entry_rejects_exit_dictated

# MEDIUM-4 (now passing):
cargo check -p hopr-protocol-session
```

## Re-review verification log (2026-07-17)

- `cargo check -p hopr-protocol-session` ✅ (MEDIUM-4)
- `cargo check -p hopr-transport-session` / `-p hopr-protocol-pix` ✅
- `cargo check -p hopr-transport-session --features telemetry` ✅ (the new metric code is
  `cfg(not(test))`-gated, so plain check/test builds never compile it)
- `cargo nextest run --lib -p hopr-transport-session` → 192/192 ✅ (incl. both committed
  regression tests)
- `cargo nextest run --lib -p hopr-protocol-pix` → 34/34 ✅
- `cargo clippy` on the three touched crates → warnings only (two introduced by
  `b3154a3af7`, see LOW-7 nit)
- Scratch tests (run, then removed): 3 extra HIGH-1 orderings ✅ pass; MEDIUM-1 writer-hang
  test ❌ fails as documented above
