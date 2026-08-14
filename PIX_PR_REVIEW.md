# Code Review — PIX branch (`lukas/pix` → `master`)

**Scope:** 304 commits, 20 338 insertions / 4 878 deletions across 98 files
(`c4f22c55b7`, 2026-08-02).
Main additions: new `protocols/pix` crate (generator, reconstructor, ack-verify),
PIX handshake in `transport/session/src/manager.rs` and
`protocols/start/src/lib.rs`, share embedding/extraction in `crypto/packet` +
`transport/hopr` pipelines, plus the `crypto/sphinx` → `crypto/packet` merge.

> **The strategy crate is intentionally outside this branch.** `7ab2e5d7f8`
> ("fix: remove the strategies") deleted `impls/strategy/**`
> outright — 11 452 lines, including `non_anonymous_pix.rs` and
> `pix_recovery_store.rs`. Every finding below whose location is in that crate —
> **M4, M5, M6, M7, L8–L12, L17**, and the deposit/sweep mechanics quoted in **C2**
> and **C3** — now describes code that is not here. Per the author's clarification,
> the implementation lives in the standalone `hopr-strategy` repository and consumes
> the `PixEvent`s raised by `hopr-lib`. Those entries are retained as historical review
> context, but they are **not** actionable against this branch or this PR.
>
> **Re-checked at `4f30a70629`.** `10f6d80c3c`
> ("consume published impls crates from crates.io; remove impls/") moved the rest of
> `impls/` out to published crates — but `hopr-strategy` is **not among them**. There
> is no `hopr-strategy` dependency in any manifest, and `NonAnonymousPix` /
> `PixRecoveryStore` appear nowhere in this workspace or its cargo dependencies. That
> is expected: the integration boundary is the `PixEvent` stream exposed by `hopr-lib`,
> not an in-workspace crate dependency. The absence of the strategy implementation here
> is therefore not evidence of a missing PIX incentive loop in this PR.

> **Line numbers below are as of the review's original writing** and have mostly
> drifted. Where a finding is still open, its location has been re-verified and
> corrected in place; for fixed findings the stale references are left alone.

**Prioritisation basis (per request):** production profile of **~1 SSA cycle per
500 MB of return-path data** and **~100 concurrent Sessions on an Exit node**.
Findings that only manifest outside that envelope are pushed down.

**Verification:** originally `cargo check -p hopr-protocol-pix -p hopr-strategy -p
hopr-transport-session --all-targets`; `hopr-strategy` no longer exists, so the
current equivalent is `-p hopr-protocol-pix -p hopr-transport-session`. Findings
below are from static reading unless stated otherwise; performance/memory figures are
_derived estimates_ from the code and are labelled as such. The C1 arithmetic was
additionally confirmed by running the built constants.

**Re-verified after the L4 / L14 / L19 fixes (2026-08-09).** `cargo nextest run --lib` →
**786/786 pass** (781 plus the five new pins). `cargo nextest run -p hopr-transport-session
--test '*' -j 1` → 10/10 and `-p hopr-protocol-pix --test '*' -j 1` → 2/2; `cargo nextest run
-p hopr-lib --features testing --test transport_session -j 1` → 6/6, which exercises the PIX
handshake end to end through all three changes. `cargo nextest run --no-run` builds every test
target.

**Re-verified again after the M2 / M3 changes (2026-08-09).** `cargo nextest run --lib` →
**790/790 pass** (786 plus four new config pins). `cargo nextest run --lib -p hopr-transport
--features serde` → 136/136 — the `serde` feature is _not_ default on that crate, and
`pix_configs_are_reachable_from_serialized_config` is `#[cfg(feature = "serde")]`, so a plain
`--lib` run silently skips the one test that proves the new config is reachable at all.
`-p hopr-transport-session --test pix -j 1` → 5/5 and `-p hopr-protocol-pix --test '*' -j 1` →
2/2. `cargo nextest run -p hopr-lib --features testing --test transport_session_pix -j 1` →
**4/4 in 248 s**, running the real PIX handshake at 1, 2 and 3 hops plus the batched-request
cycle through the threaded config. `cargo shear` changed no manifest; `cargo nextest run
--no-run` builds every target.

**Re-verified after M2's runtime budget replaced the modelled one (2026-08-10).**
`cargo nextest run --lib` → **792/792** (790 plus the five new budget pins, less the three
modelled-validation tests deleted with the check they covered). `transport_session_pix` again
**4/4 in 248 s** with the budget armed. `acknowledge_shares/sustained_quota_rate` benchmarked
against a `73efa5c8a1` baseline reports **no change** (p = 0.65, p = 0.61).

The `awaiting_ack_entry_cost` profile now doubles as the end-to-end proof that an entry ceiling is
a byte ceiling: filled to its configured `max_ack_buffer_bytes`, live heap lands at **37.2 MiB
against a 38.1 MiB budget — 98 % of it** — and the next share is refused. 390 B/entry measured
against the 400 B constant.

**Latest-tip / combined-branch audit (2026-08-10).** The worktree is at
`e345470ead` (the six local L4/L14/L19/M2/M3 fixes on top of `4f30a70629`), while
the fetched `origin/lukas/pix` has one divergent commit, `ddadbc86ac` (the PIX
curve-feature override). The follow-up remains at `6d671409ea`. The status below
treats all three sets of changes as the intended merge sequence, per the author's
instruction: an item fixed by the follow-up is counted as fixed here even where the
base branch still contains the code it deletes.

The curve commit is no longer merely fetched: the six base-branch commits were **rebased onto
`ddadbc86ac`**, which is now this branch's base, so the L22 cleanup could be written against the
`pix-bjj`/`pix-secp256k1` names rather than the ones they replace. `cargo check -p
hopr-crypto-packet --all-targets` passes with the default BabyJubJub selection, and both supported
secp256k1 forms pass as well (`--no-default-features --features ed25519,rayon,pix-secp256k1` and
defaults plus `--features pix-secp256k1`). All three are now warning-free; the two unused-`Group`
imports the override builds used to emit are fixed under L22. No open review finding is closed by
the curve commit itself.

_A caution for anyone re-running the microbenchmarks here:_ `insert_encrypted_share/single_share`
measured 610 ns, 880 ns and 1.05 µs on **identical code** across three runs on this machine, and
criterion's `change` line compares against whatever run happened last — one reading of "+17 %" was
against a stale baseline from an unrelated commit. Use `--save-baseline`/`--baseline` explicitly,
and treat that particular benchmark id as unreadable without it. (It is also a poor proxy for
production regardless: it replaces one `ack_challenge` 5.5 M times, a path no real Exit takes.)

The per-entry cost behind `AWAITING_ACK_ENTRY_BYTES` was **measured, not estimated** —
`cargo test --release -p hopr-protocol-pix --test memory_profile -- --ignored --nocapture
awaiting_ack_entry_cost` reports 383 B/entry at 20 000 entries and 389 B/entry at 100 000, of
which `size_of` accounts for 145 B. This is one of the few figures in this document that is not
a derived estimate.

`cargo clippy --workspace --all-targets` reports five pre-existing warnings, none in the files
touched here (still five after the M2/M3 work): `assertions_on_constants` in
`transport/hopr/src/constants.rs`, three `vsss_rs`
`deprecated` uses in `protocols/pix/src/lib.rs`, and a `useless_vec` at
`protocols/pix/src/types.rs:1092`. **Correction:** the previous entry here said the latter four
"are gone". They are not — that reading came from a clippy run in which those crates were not
recompiled and so re-emitted nothing. Confirmed by touching both files and re-running:
`cargo clippy -p hopr-protocol-pix --all-targets` emits all four. Local clippy is 1.97; CI's
pinned toolchain may still differ, but that cannot be inferred from a cached local run.

`cargo nextest run --lib -p hopr-lib --no-run` **compiles** — M11 is closed, so the caveat that
used to sit here about `cargo check --workspace --all-targets` no longer applies. Note that
`cargo nextest run --workspace --test '*'` does _not_ build: `hopr-lib`'s integration targets
require `--features testing`.

**Latest base-tip audit (2026-08-12).** Re-checked at `01466b416e`, including the four new
calibration/surplus commits (`46beab643e` through `01466b416e`). L21 and L22 remain fixed, and the
new Exit interpolation benchmark closes the previously missing Exit measurement. Two residuals
were found in the new work: the 20 % surplus calculation rounds down for non-multiples of four
(L23), and the polys/threshold conclusion measured only Entry commitment construction while
omitting Entry share generation, whose polynomial evaluation is threshold-dependent (M17).

**Re-review after the three fixes (2026-08-13).** Checked `3fea835f3a`. M16, M17 and L23 are
functionally closed, and the implementation also fixes L25 (two stale/failing session tests and five
old quota descriptions exposed by the surplus change). The affected PIX/session tests pass, as do
the `hopr-crypto-packet` unit tests under the default BabyJubJub build, secp256k1-only build, and the
default-plus-secp override build. One new low cleanup was raised and is now **fixed at `14bb5aedbe`**:
L26, stale “triple”/“all three” prose left where `PixParams` has four components.

**Combined-branch post-merge audit (2026-08-14).** Checked the current checkout,
`lukas/session-pix-supervisor` at `cd4370f233` (the branch referred to as the Session PIX manager
branch), after all of `lukas/pix` through `d48244448c` was merged into it and H6's returned-data gate
landed. The merge preserves the base fixes: M2/M3's configured reconstructor is the instance
installed and validated at `SessionManager::start`, the suite travels in `PixParams`, surplus is
priced and rounded up, and the supervisor owns commitment/deposit/recovery deadlines and retirement
guards. No functional merge or H6 regression was found.

Verification at the combined tip:

- `cargo nextest run -p hopr-protocol-pix -p hopr-transport-session --features runtime-tokio` →
  **399/399 pass** (two explicitly skipped tests), including the six new H6 tests;
- `cargo test -p hopr-crypto-packet --lib` under default BabyJubJub, secp256k1-only, and the
  default-plus-secp override → **93/93, 92/92 and 92/92 pass**;
- `cargo nextest run -p hopr-transport --features serde --lib config::tests` → **46/46 pass**;
- `cargo nextest run -p hopr-lib --features testing --test transport_session_pix -j 1` →
  **8/8 pass**, including batching, 1–3 hops, deposit timeout, hard-recovery timeout and strict
  prepay. The sandboxed attempt could not bind its local UDP ports and timed out before connectivity;
  the unrestricted rerun passed, so that was an environment limitation rather than a regression.

Two review residuals now belong directly to this combined branch. **H6 is fixed**: the Entry gates a
successor on `SessionSlot::returned_packets`, the Exit→Entry packets it actually received, discounted
by the surplus ratio and with a bounded wait for reordering.

**H3 Tier 3 is also fixed**: the per-cycle memory model was re-derived post-M9 (the ≈49 MiB figure
three sites carried was the deleted Feldman matrix), exported as `peak_cycle_bytes`, and is now
reserved node-wide at Session admission; `retired_ssas` has a capacity derived from that budget; and
a cumulative unpaid-cycle failure limit closes a batched Session.

- **M15 remains open:** serialized config rejects a recovery deadline too short for the accepted
  quota, while a direct `SessionManagerConfig` can still select (for example) one second.

H1 and H2 remain separately tracked, and the strategy-owned findings remain outside this repository.
No additional finding was introduced by the merge.

### Historical follow-up audit before the branches were merged

**Reviewed at `6d671409ea` (2026-08-09), which contains `4f30a70629`.** The
targeted suite was run from an archive of that commit so this worktree did not have
to change branches:

- `cargo nextest run -p hopr-protocol-pix -p hopr-transport-session --lib` →
  **348/348 pass**;
- `cargo nextest run -p hopr-transport-session --test pix` → **5/5 pass**;
- `cargo nextest run -p hopr-transport --lib config::tests` → **40/40 pass**
  (94 non-matching tests skipped by the filter).

That branch materially improves the lifecycle story, but it does not make every
open item below disappear. The important delta is:

| Finding on `lukas/pix`                | State on `lukas/session-pix-supervisor`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C3 residual / H2 stalled funded cycle | **Bounded.** A per-Session supervisor owns separate commitment, deposit, recovery-idle and hard-recovery deadlines. A deposit can no longer cancel the commitment clock. Commitment NACK/retransmission is tracked separately as [#8318](https://github.com/hoprnet/hoprnet/issues/8318).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| H3 Tier 3 / cycle admission           | **Substantially implemented; supervisor-only residual.** The Exit-side supervisor owns requests, permits only the last funded/recovering cycle of a batch to request one successor batch, and bounds failed cycles with commitment, deposit and recovery deadlines. `RetireSsa` drops the matching `SsaCommitmentGuard`; Session teardown drops every remaining guard. The residual — the capacity ceiling, the tombstone audit and the batched unpaid-cycle policy — **is now fixed on the combined branch**: `peak_cycle_bytes` replaces the stale pre-M9 model, a Session reserves against `max_live_cycle_bytes` at admission, `retired_ssas` is capped from that same budget, and `max_failed_cycles` closes a Session losing cycles repeatedly. None of it belonged on `lukas/pix`.                                                                                                                                                                                                      |
| H6 repeated SSA requests              | **Fixed.** The protocol floor rejects `early_recovery_threshold < 0.85`, and the Entry now admits a successor only once it has _received_ the corresponding Exit→Entry packets — `SessionSlot::returned_packets`, discounted by the surplus ratio since a share unlocks on first-relayer acknowledgement rather than on delivery here. A shortfall within one emission window is waited out rather than refused, because `RequestSsa` is never retried.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| M2 / M3 — reconstructor configuration | **Both fixed on the base branch; the stacked branch's version is now the older one.** `PixReconstructorConfig` mirrors all eight `SsaReconstructorConfig` fields under `pix.reconstructor`, and one `ssa_reconstructor()` helper feeds every construction site including the Entry "dummy". The acknowledgement buffer is bounded at runtime by `max_ack_buffer_bytes` rather than by validating a workload model — a model has to assume a Session count and a packet rate, and this node enforces neither (`maximum_managed_sessions` validates to 100 000 and `NoRateControl` removes egress shaping). The insertion check deliberately permits an overshoot of at most the concurrently in-flight insertions; that is a small bounded concurrency allowance, not M2's former product-scale growth. The stacked branch should drop `ssa_reconstructor_config()` for the base helper on rebase, which closes L20 there too, and retarget `validate_pix_supervision` at the configured value. |
| L5 / L6 / L18                         | **Fixed on the stacked branch.** `PixKillSwitch` and `DepositAwaiter` are gone. One supervisor owns the timers; `RetireSsa` aborts the per-cycle `PixDepositObserver`, and Session teardown aborts the action driver and retires its live guards.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| L4 / L14 / L19                        | **Fixed.** The deposit channel is allocated inside the first-encounter guard; `try_new` is now the validating constructor on both `SsaShareGenerator` and `SsaReconstructor`, with `new` a documented-panicking delegate, and `new_ssa_commitment` derives its `SsaId` from its parameters instead of indexing `commitments[0]`; and the integration test asks `StartProtocol::ssa_commit_chunking` for the bound rather than restating it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |

Two corrections matter when reading the older #8237 notes retained below:

- supervision now resets liveness from `SsaRecoveryProgress::shares_seen`, not
  `useful_shares`. Surplus shares are priced after H5 and count as liveness, while
  `useful_shares` remains the payment/order-progress counter. Thus
  `max_served_without_progress` now bounds genuine silence, not surplus abuse;
- the branch adds a two-hour default `max_recovery_time` and validates the
  operator-configured value against the top of `quota_range`. The old one-hour
  value could not cover the default cycle at the documented 1.5 Mbps cap.

The follow-up found two branch-only items: **M15** (the quota/deadline consistency
check is skipped for programmatically constructed configs) and **L20** (the stale
`max_awaiting_acks` value was reintroduced in a new safety comment). They are
recorded in P2 and P3 respectively. **L20 is now moot on the base branch** — the safety
comment it describes was deleted along with the `default()` construction it justified.

Under the combined-branch interpretation, **L5/L6/L18 are fixed**, and **L20 is
resolved by the base branch's single configured constructor when the follow-up is
rebased**. M15 is not fixed by `6469b3cbec`: that commit clamps durations that would
overflow `Instant`, but still accepts a representable `max_recovery_time` (for example
one second) that cannot serve the configured quota.

**This document is now the only one.** The separate `CODERABBIT_TRIAGE.md`, which triaged
45 inline comments across two CodeRabbit passes on PR #8095, has been folded in and deleted:
its still-open items are **L18** under P3 and **H1**'s parked-resolution delivery
residual, plus the supervisor-side
`retired_ssas` capacity audit under **H3 Tier 3** (both now fixed, along with **L19**, its other);
its corrections are recorded against the findings they correct (**H1**,
**M13**, **L15**), and its provenance and rejected findings are under "The CodeRabbit pass,
folded in".

**IMPORTANT CORRECTION (found while implementing the fixes):** `bjj` is a
**default** feature of `hopr-crypto-packet` (`crypto/packet/Cargo.toml:27`), so the
production curve is **BabyJubJub, not secp256k1**. Consequences for this review:
`size_of::<HoprPixGroupElement>()` is **32** bytes, not 33; BabyJubJub has
**cofactor 8**, which makes M13 a live vulnerability rather than a hypothetical one;
and the absolute EC-cost figures in H4/M9 are pessimistic, since twisted Edwards
arithmetic is cheaper than secp256k1 — the _relative_ claims (redundant work,
per-share cost scaling with `threshold`) are unaffected.

**Reference constants used throughout**

| Constant                                        | Value                                                                       | Source                                                        |
| ----------------------------------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `HoprPacket::PAYLOAD_SIZE`                      | 1038                                                                        | `DefaultSphinxPacketSize=U1040` − 1 padding − `HEADER_LEN=1`  |
| `ApplicationData::PAYLOAD_SIZE`                 | 1030                                                                        | 1038 − `Tag::SIZE`(8)                                         |
| `size_of::<HoprPixGroupElement>()`              | 32                                                                          | BabyJubJub compressed point (`bjj` is a default feature)      |
| `PixGlobalConfig` defaults                      | `num_ssa_parts=8192`, `ssa_part_size=64`, `additional_shares=None` → **16** | `transport/hopr/src/config.rs`; all three alias the pix crate |
| `IncomingSessionPixConfig::quota_range` default | `170 065 920 ..= 680 263 680` (162.2–648.8 MiB), derived — see C1           | `transport/session/src/types.rs`, span 4× (C1)                |
| Nominal quota per SSA                           | 8192 × **80** × 1038 = **680 263 680 B** (648.8 MiB)                        | `pix_params_to_quota`, surplus included (H5)                  |
| Commitments per SSA cycle                       | **8 192** EC points (~262 KB, ~320 Start packets)                           | constant terms only, post-M9                                  |
| Shares per SSA cycle                            | 8192 × (64 + 16) = **655 360** (~649 MiB return data)                       | `generator.rs`, `max_shares_per_poly`                         |

All packet-size and quota figures above were verified empirically against the
built code, not only derived from the constants.

> **Dimensions moved after the review was written.** The `threshold` re-tune
> (H3) took `4096 × 128 + 64 surplus` to `8192 × 64 + 32`, and M9 collapsed the
> commitment count from `polys × threshold` to `polys`. Neither changed the
> quota product; only the factorisation and the commitment row moved. Where the
> text below says "4096 × 128" or "524 288 commitments", read the row above.

> **The quota itself then moved, when H5 was closed (`20845807ae`).** It was
> `polys × threshold × PAYLOAD` = 544 210 944 B (519.0 MiB) — the shares needed to
> _reconstruct_. It is now `polys × (threshold + surplus) × PAYLOAD` =
> **816 316 416 B (778.5 MiB)**, the shares a cycle actually _emits_. That is the
> whole point of the fix: the "Nominal quota per SSA" and "Shares per SSA cycle"
> rows above are now the same quantity, where they used to differ by 1.5×.
>
> Consequences for the text below: wherever it says "519 MiB of quota" or
> "524 288 packets per cycle", read **778.5 MiB** and **786 432 packets**. C1's
> derived `quota_range` moved with it and is still correct by construction (it is
> computed from `pix_params_to_quota`, and `default_pix_dimensions_must_be_inside_
default_incoming_quota_range` pins the containment). **Commitment figures are
> untouched** — still 8 192 per cycle, one per polynomial, because M9 decoupled the
> commitment count from the share count. H2's channel sizing is likewise unaffected
> in substance: it derives from `quota_range.end()` and is clamped to
> `MAX_POLYS_PER_SSA`, and the clamp is what binds at these numbers.

> **And once more, when the surplus became a ratio (`42f7edf9c6`).** The default surplus went from
> `threshold/2` = 32 to `threshold/4` = 16, so the deployed emission factor is **1.25×**, not 1.5×,
> and the quota is **680 263 680 B (648.8 MiB)**. The reference table above is corrected; where the
> text below still says 778.5 MiB or `8192 × 96`, read 648.8 MiB and `8192 × 80`. Commitment
> figures are again untouched, for the same reason as before.

---

## P0 — Critical

### C3. The Entry can make the SSA deposit key one it alone knows, sweep its own deposit, and still be served (rogue-key)

> **STATUS: FIXED.** Reproduced first, then closed by requiring a proof of knowledge. The
> reproduction test is inverted and kept as the regression
> (`exit_refuses_a_client_commitment_whose_deposit_key_the_entry_knows`,
> `protocols/pix/src/reconstructor/mod.rs`).

The deposit key is `s + e`, where `s` is the sum of the Entry's polynomial constant terms and `e`
is the Exit's commitment secret. The construction is safe only because **neither party knows the
sum**. Three facts together broke that:

1. **The Exit publishes `e·G` first.** `SsaServerCommitmentMessage.commitments`
   (`protocols/start/src/lib.rs`) carries it to the Entry as `SsaRequest`, ahead of any `SsaCommit`,
   so the Entry can derive the address it must fund.
2. **The Entry derives the deposit address itself** as `client + exit`
   (`transport/session/src/manager.rs`, `handle_ssa_request`), in the same loop iteration that
   received `e·G`.
3. **Nothing proved the Entry knew what it published.** `decode_commitment` checks encoding and
   prime-order-subgroup membership only.

So a modified Entry could pick a random `w`, publish constant terms summing to `w·G − e·G`, and end
up with a combined commitment of `w·G`. Worked example with 3 polynomials, `e = 11`, `w = 100` and
honest constant terms `2, 5`:

```
C₀¹, C₀²  = 2·G, 5·G                 honest, h = 7
C₀³       = W − 7·G − E              computable as a point from public values;
                                     its dlog is 100 − 7 − 11 = 82, which needs e
Cs        = (2+5+82)·G = 89·G  =  W − E
full_ssa  = Cs + E     = 100·G =  W       deposit key 100 = w, known to the attacker
poly 3    needs a₀³ = 82             unknown ⇒ no valid shares ⇒ emitted last
```

The crux is line 2: the _point_ is computable by group subtraction without ever knowing its scalar.
The attacker knows the sum but not the last summand — precisely what lets it spend while being
unable to let the Exit reconstruct.

Consequences, all traced:

- **It can sweep its own deposit.** The Exit's own sweep is `ChainKeypair::from_secret(secret)` then
  `sweep_recovered` (`impls/strategy/src/non_anonymous_pix.rs`), so knowing the scalar is sufficient
  to spend.
- **The Exit keeps serving.** `DepositAwaiter` waits for a single `deposit_done` and then
  _permanently_ aborts `PixKillSwitch(idx)` (`transport/session/src/manager.rs`); the gap is
  acknowledged in-tree by its own TODO ("how to kill the Session if we do not observe progress
  towards the current SSA deposit recovery?").
- **Detection is deferred to the end of the cycle.** The Entry controls polynomial emission order
  (`generator.rs` builds `poly_queue`, `next_share` drains `front_mut()`), so it places the
  unservable polynomial last and is served ~8191/8192 of a 519 MiB cycle first.

Net: up to ~519 MiB of service per Session, money retained, Exit never paid, repeatable with a fresh
Session for the cost of two transactions. **This is not C2** — there _neither_ party can recover,
whereas here the party that owes can.

**Per-share Feldman verification does not defend this.** It moves detection from share 1 to share
`threshold` _of the rogue polynomial_, which is emitted last: 63 packets out of 524 288. This is the
concrete reason not to treat the non-constant coefficient commitments as load-bearing for PIX's
economics (relevant to M9 and to the `C₁..C_{t-1}` question) — and it is the reason M9 could
subsequently drop them outright without weakening this fix, which stands on the proof of knowledge
alone.

**Fix applied.** One Schnorr proof of knowledge of `s = dlog(Cs)` per SSA — 64 bytes, two scalar
multiplications on the Exit:

```
prover:    r random,  R = r·G,  c = H(pseudonym ‖ ssa_index ‖ Cs ‖ R),  z = r + c·s
verifier:  z·G == R + c·Cs
```

`Cs` and `full_ssa` differ by exactly `e`, so knowing the discrete log of one implies not knowing
the other's unless you know `e`. Pinning the Entry to `Cs` closes the whole space:

|                              | can produce the proof? | knows `deposit_key = s + e`?        |
| ---------------------------- | ---------------------- | ----------------------------------- |
| honest Entry (knows `s`)     | yes                    | no — `e` missing                    |
| attacker (knows `w = s + e`) | no — would need `e`    | yes, but rejected before depositing |

There is no third branch, which is why the proof is over the _sum_ rather than per polynomial: a
rogue individual `C₀⁽ᵏ⁾` is harmless as long as the sum's discrete log is known, because then the
deposit key is not.

What was done:

- `SsaCommitmentProof<S>` in `protocols/pix/src/types.rs`, with `prove` / `verify` / `to_bytes` /
  `try_from_bytes`, plus `PixSpec::commitment_proof_challenge` beside `msg_to_scalar` using the same
  domain-separated `hash_to_scalar` machinery and a new `HASH_COMMITMENT_PROOF_CONTEXT`.
- Produced in `new_ssa_commitment` where `our_commitment_secret = sub_secrets.iter().sum()` is
  already in scope — the witness existed and was being discarded, so no new state.
- Verified in `SsaCommitmentBuilder::add_transposed` at the constant-term milestone, **before**
  `full_ssa_commitment` is recorded and before the `SsaBuilder` is handed out. So an unproven cycle
  never yields a deposit address, `DepositNeeded` never fires, and the Exit never serves against it.
  New `PixError::UnprovenSsaCommitment`, distinct from `InvalidInput` so the Exit can log an
  attempted exploit rather than a malformed message.
- Carried on the wire by `SsaClientCommitmentMessage`, present exactly when
  `coefficient_index == 0`. Presence is implied by the coefficient index — no flag byte — and the
  encoder rejects any disagreement between the two. Every constant-term message carries it (~293
  per cycle, 18.7 KB) so no single lost packet strands an otherwise recoverable cycle; the Exit
  keeps the first it sees, since any one valid proof suffices and Schnorr is randomised.
- `StartProtocol` gained a fifth generic for the proof's wire form, instantiated as
  `HoprPixCommitmentProof` (a fixed-size byte newtype in `crypto/packet`, mirroring
  `HoprPixGroupElement`).
- `MIN_COMMITMENTS_PER_SSA_COMMIT_MSG` now subtracts the proof, so H2's channel sizing stays a safe
  over-estimate. Constant-term messages carry ~2 fewer entries; phase 2 keeps the full budget, so
  only the constant-term pass grows (~273 → ~293 messages per cycle).

**The Exit needs no matching proof** as long as it keeps committing _first_ — it cannot adapt `e·G`
to the Entry's commitment, so the symmetric attack is unavailable to it. Reversing the message order
would move the exploit to the Exit and oblige it to prove instead. Worth stating in any future
protocol revision that touches the handshake order.

**Historical base-only residual, now fixed in the combined branch:** the old
`DepositAwaiter` had no post-deposit progress deadline, so the Exit could serve a whole cycle before
noticing no progress. The merged supervisor's service gate, `max_recovery_idle` and
`max_recovery_time` now bound this independently of the deposit observer.

### C1. Default PIX parameters are rejected by the default Exit quota range — PIX cannot establish a session out of the box, and the config is unreachable

> **STATUS: FIXED** — see "Fix applied" below. The failure was confirmed
> empirically before the fix, not just derived.

`check_pix_params` (`transport/session/src/manager.rs:2007-2036`) computes

```
quota = polys_per_ssa × shares_per_ssa × HoprPacket::PAYLOAD_SIZE
```

With the default `PixGlobalConfig` the Entry can only announce `(4096, 128)` —
`new_session` (`manager.rs:1247-1260`) hard-rejects anything that does not match
the installed generator exactly. That yields

```
4096 × 128 × 1038 = 544 210 944 bytes  (519.0 MiB)
```

The default `quota_range` upper bound was `536 870 912` (512 MiB). **544 210 944 >
536 870 912**, so `in_quota_range` was `false`, the Exit replied
`StartErrorReason::UnacceptablePixParams`, and every `UsePIX` session was refused.
Over by ~7.0 MiB (1.37 %) — exactly the kind of off-by-a-constant that survives
review.

This is compounded by:

- Both PIX config structs are marked `#[cfg_attr(feature = "serde", serde(skip))]`
  in `HoprProtocolConfig` (`transport/hopr/src/config.rs:136-142`), so
  `PixGlobalConfig` **and** `IncomingSessionPixConfig` are pinned to their
  `Default` values. An operator cannot widen `quota_range`, cannot change
  `num_ssa_parts`/`ssa_part_size`, and cannot set `enforce_pix`,
  `max_deposit_wait` or `max_ssa_delivery_time`. `hoprd` contains no PIX config
  surface at all (grep for `pix` under `hoprd/` returns nothing).
- No test covers the default combination. Every PIX test overrides the range:
  `hopr/hopr-lib/tests/transport_session_pix.rs:82` uses `0..=100_000`,
  `transport/session/tests/pix.rs:61,414` use `0..=2048*1024*1024`, with tiny
  dimensions (`num_ssa_parts: 8, ssa_part_size: 2`).

**Fix applied.**

- `transport/session/src/types.rs` now owns the canonical dimensions —
  `DEFAULT_PIX_POLYS_PER_SSA` (4096), `DEFAULT_PIX_SHARES_PER_POLY` (128) and the
  derived `DEFAULT_PIX_SSA_QUOTA` — and both sides are computed from them:
  - `PixGlobalConfig::{num_ssa_parts, ssa_part_size}` default to those constants
    instead of bare literals;
  - `IncomingSessionPixConfig::quota_range` defaults to
    `DEFAULT_PIX_SSA_QUOTA / 4 ..= DEFAULT_PIX_SSA_QUOTA`
    (`136 052 736 ..= 544 210 944`), preserving the original 4× span and the
    original "our nominal dimensions are the ceiling" intent, while making it
    impossible for the two to drift apart again.
- `pix` and `incoming_session_pix_config` are now `serde(default)` rather than
  `serde(skip)`. `IncomingSessionPixConfig` gained `Serialize`/`Deserialize`
  (`serde` is already a non-optional dependency of `hopr-transport-session`);
  `RangeInclusive<u64>` uses serde's built-in impl (`{start, end}`) and the two
  `Duration` fields use `humantime_serde`, matching `StreamProtocolConfig`.
  Both structs also got container-level `serde(default)` so partial
  specification works — otherwise naming the section would require every field.
- `quota_range` is now operator-settable, so an empty/inverted range is newly
  reachable; `validate_incoming_session_pix_config` rejects it via the existing
  `Validate` derive on `HoprProtocolConfig`.
- Four regression tests in `transport/hopr/src/config.rs`:
  `default_pix_dimensions_must_be_inside_default_incoming_quota_range` (asserts
  both containment _and_ that the range end equals the nominal quota, so the
  derivation cannot be quietly broken), `default_pix_configs_must_validate`,
  `empty_pix_quota_range_is_rejected`, and
  `pix_configs_are_reachable_from_serialized_config`.

**Residual trade-off:** the default range ends exactly at the nominal quota, so an
Entry configured with even slightly larger dimensions is still rejected until the
Exit widens `quota_range` — which is now possible. Headroom was deliberately not
added: raising the ceiling raises both the unincentivized exposure per SSA cycle
and the reconstructor memory held per Session (see H3).

---

### C2. Deposited funds are permanently burned whenever an SSA cycle does not complete

> **STATUS: BY DESIGN** — confirmed intended by the author. Kept here as
> documentation of the economic model and its operational consequences, not as a
> defect. The operational notes below (Exit restarts, partial cycles) still apply
> and are worth surfacing in operator-facing docs.
>
> Note the distinction from **C3**: the burn is safe only because _neither_ party can recover an
> incomplete cycle. C3 was a case where the Entry alone could, which is what made it a defect rather
> than the accepted model. With C3 fixed, griefing the Exit costs the griefer its full prepaid
> deposit — which is the intended incentive.

The stealth-address private key is `client_secret + exit_secret`
(`reconstructor/utils.rs:292`, `generator.rs:290`). The Entry holds only
`our_commitment_secret`; the Exit holds only `exit_commitment_secret`, in memory,
inside `SsaCommitmentBuilder` / `SsaBuilder`. **Neither party can recover the
deposit alone, and there is no on-chain refund path.**

The Entry pre-pays the _full_ SSA quota the moment it derives the deposit address
(`manager.rs:2826-2833` → `HoprSessionOutPixEvent::ReadyToDeposit` →
`PixEvent::NewDepositAddress` → `withdraw(price_per_byte × quota, …)` in
`non_anonymous_pix.rs:176-186`), i.e. _before_ any of the cycle's data has flowed.

Every one of these destroys the Exit's half and burns the deposit:

- Kill-switch fire (`manager.rs:1692-1710`) → `retire_all_live_ssa_cycles`.
- `close_session` / idle eviction (`manager.rs:1757-1770`, `manager.rs:913-916`).
- `SsaReconstructor` TTL eviction (`incomplete_ssa_lifetime` 10 min,
  `unused_verifier_lifetime` 30 min) on a stalled cycle.
- **Any Exit process restart** — `SsaCommitmentBuilder` state is in-memory only.
  `PixRecoveryStore` persists _already-recovered_ keys, never the pending
  `exit_commitment_secret`.

At the stated profile (100 concurrent sessions, one cycle per ~500 MB) this is a
continuous drain, not a corner case: a single Exit restart burns up to 100 ×
`price_per_byte × 519 MiB`. Partial cycles are also fully charged — a session that
dies at 50 % has paid 100 % and delivered 50 %.

**Strategy-side mitigation (not part of this PR):** persist `exit_commitment_secret`
alongside the SSA id in the standalone `hopr-strategy` recovery store so a restart can
resume. Changing the burn semantics themselves would require incremental/streaming deposits
or a refund/escape hatch (for example, a client-recoverable timelock branch). This PR's only
remaining C2 question is whether the intended semantics need operator-facing documentation.

---

## P1 — High (will bite at the stated scale)

### H1. `pending_ack_keys` retry drain is O(stashed) per `acknowledge_shares` call → quadratic during every commitment window

> **STATUS: FIXED.** The stash is now keyed by `SsaPolynomialId` — the very thing whose
> arrival unblocks the entries inside it — so a bucket is drained exactly once, by the
> installation of its own verifier, and `acknowledge_shares` never scans anything. It only
> appends (`defer_ack`, O(1)).
>
> Two changes made this possible and are worth noting:
>
> - Deferral stopped travelling through the error channel. `process_verified_ack` now returns
>   `ProcessedAckResult::VerifierNotReady(spi)` instead of `Err(MissingVerifier)`, which both
>   models it honestly (a missing verifier is not a failure) and hands the caller the bucket key.
>   `PixError::MissingVerifier` is gone — nothing could return it any more.
> - The drain runs on the **commitment** path, not the acknowledgement path, since that is where
>   the unblocking verifier is installed. Resolutions it produces are parked in
>   `ready_resolutions` and picked up by the next `acknowledge_shares` call, because
>   `insert_coefficient_commitments` has no route to the upper layer. Pickup costs one relaxed
>   atomic load in the common case. The share verification burst therefore lands on the
>   `spawn_blocking` commitment path rather than in an ack batch.
>
> **Correction (2026-08-02).** This entry used to claim a second thing was closed: "a
> microsecond-wide race where an ack deferred _concurrently_ with its verifier's installation would
> land in a bucket whose only drain had already run — `defer_ack` re-probes after appending and
> drains itself if so." The re-probe narrowed that window; it did not close it. `drain_deferred_acks`
> opens with `pending_acks.get(..) else { return }`, and a miss is exactly what the interleaving
> produces: the concurrent drain has already invalidated the key, so the re-probe walks straight
> past the ack it just appended. Found by CodeRabbit, confirmed, and fixed in
> `fix(pix): redeem an acknowledgement deferred into an already-drained bucket` — the bucket now
> carries a `drained` flag set in the same critical section as the take, and an append that sees it
> redeems inline instead of parking. Pinned by
> `an_acknowledgement_deferred_into_a_drained_bucket_is_still_redeemed`, which forces the
> interleaving through a stale bucket handle rather than by timing.
>
> **Second correction, same area.** The hand-off above is a _pull_: `take_ready_resolutions` has one
> caller, `acknowledge_shares`, and both ack pipelines gate that on `has_pending_shares`. A
> `RecoveredSsa` parked by the drain — the recovered deposit key — was therefore delivered only if
> the producing peer sent another batch before its own `awaiting_acks` entry idled out. The guard now
> admits any batch whenever something is parked, and `retire_ssa` reports whatever is left at error
> level. Delivery still depends on _some_ peer acking again before teardown; closing that needs the
> reconstructor to push rather than be pulled — a sink on its constructor — which is bundled with
> **M3**, since that reworks the same construction sites (two of them now, not three).
>
> **Third correction, same rewrite.** The per-cycle cap check reintroduced an O(n) scan under a
> mutex on the very path this finding exists to keep O(1): `defer_ack_into` computed
> `bucket.by_poly.values().map(Vec::len).sum()` inside the bucket lock on **every** deferral,
> against a cap of `MAX_DEFERRED_ACKS_PER_CYCLE` (8192) entries that can each occupy their own
> sub-bucket — ~33 M map-entry visits to fill one bucket, on the path every acknowledgement takes
> during the commitment window. It also contradicted the doc three lines above it ("O(1) — this is
> the entire cost the acknowledgement path pays for a deferral"). Found by CodeRabbit and fixed in
> `9d86161773` with a `total: usize` on `DeferredAcks`, one increment site and one reset site, both
> under the same mutex. `deferred_ack_buckets_are_capped_per_cycle` already pinned it, because
> `deferred_ack_count` deliberately still recomputes from `by_poly` rather than reading the counter
> — an accessor that read the counter would agree with it whatever it said.
>
> Note the severity re-assessment behind the original text: the Entry's generator consumes
> polynomials from a FIFO queue, so cycle _n+1_'s shares only start flowing once cycle _n_'s are
> exhausted, giving pipelined cycles a ~78 MB head start on their commitments. The quadratic
> blow-up was therefore concentrated at **SSA index 1** and under Start-protocol congestion, not
> in steady state. Two things cut the other way, though, and were what justified a structural
> fix over "the stash stays small": `for_each_concurrent` runs up to
> `DEFAULT_ACK_INPUT_CONCURRENCY` (10) batches per peer simultaneously, each performing its own
> full scan; and the stash was keyed by `packet.next_hop`, the **first relayer of the return
> path**, which aggregates across every Session sharing that relayer.

`protocols/pix/src/reconstructor/mod.rs:514-549`:

```rust
if let Some(per_peer) = self.pending_ack_keys.get(&peer) {
    let stashed: Vec<(HalfKeyChallenge, HalfKey)> =
        per_peer.iter().map(|entry| (*entry.0, entry.1)).collect();   // full scan + copy
    for (challenge, ack) in &stashed { … }
}
```

The entire per-peer stash is iterated and cloned on **every** invocation.
`acknowledge_shares` runs once per inbound ack batch — hundreds per second on a
busy Exit.

Acks land in this stash whenever `process_verified_ack` returns `MissingVerifier`
(`mod.rs:568-577`), which is the normal state for the whole window between "Entry
starts emitting shares for SSA _n_" and "all 524 288 coefficient commitments for
SSA _n_ have arrived" (~19 000 Start packets). During that window every return-path
share stashes. At even 2 000 shares/s over a 15 s window that is ~30 000 stashed
entries, rescanned hundreds of times per second → **~10⁷ cache iterations/s per
peer**. This recurs at the start of every SSA cycle, and is worst for SSA index 1
(no pipelining to hide it).

**Fix:** index the stash by `SsaPolynomialId` (or keep a per-`SsaId` bucket) and
drain only the buckets whose verifier has just become available; or drain at most
N per call.

---

### H2. Start-protocol channel is sized for session setup, not for ~18 700 commitment messages per cycle — and drops are unrecoverable

> **STATUS: FIXED.** The capacity is now derived from the largest accepted quota
> rather than from the session count. `start_protocol_channel_capacity` computes
> `quota_range.end() / PAYLOAD_SIZE` commitments, divides by a conservative lower
> bound on commitments-per-message (`MIN_COMMITMENTS_PER_SSA_COMMIT_MSG`, mirroring
> `new_multiple`'s layout with a generous CBOR allowance so the message count comes
> out as an over-estimate), and adds `maximum_sessions +
START_PROTOCOL_CHANNEL_RESERVE` for ordinary Start traffic. At default config that
> is 18 725 + 10 010 = 28 735 slots (~4 MB ring at 144 B/slot). Pinned by
> `start_protocol_channel_is_sized_for_the_worst_case_commitment_burst`.
>
> **Still open, and worse than first stated.** There is no NACK or retransmission for
> an `SsaCommit` lost to genuine network loss, and on this branch a loss after the
> constant terms have arrived leaves the Exit serving with **no deadline at all**:
> `deposit_address_first_encountered` fires at `SsaCommitmentDone` (constant terms
> only), the deposit lands, the `DepositAwaiter` aborts `PixKillSwitch(idx)` — and if a
> _later_ coefficient's message was dropped, `Completed` never fires, no verifiers are
> installed, every share returns `MissingVerifier`, and nothing closes the session
> until idle eviction.
>
> The stacked `lukas/session-pix-supervisor` branch covers this failure mode: its commitment timeout is a distinct phase bound
> (`max_ssa_delivery_time`, 20 s, from `SsaRequestSent` to **`CommitmentVerified`**)
> rather than something the deposit can cancel, so a dropped `SsaCommit` closes the
> session on a bounded deadline. It still does not _recover_ the cycle — that needs the
> Exit to request the missing `(coefficient_index, poly_index)` ranges, which the
> `SsaCommitmentBuilder` already knows.
>
> **Re-assessed after M9, in both directions.**
>
> The "worse than first stated" case is **gone**. It depended on the deposit being
> triggered by the constant-term pass while _later_ coefficients were still in flight.
> There are no later coefficients now — only constant terms are sent, and
> `add_transposed` discards anything else without decoding it. A lost `SsaCommit` today
> means the commitment never completes, so `DepositNeeded` never fires and **no deposit
> is made**: the cycle dies unfunded rather than funded-and-stalled. That also removes
> it as a C2 burn path.
>
> Per-message severity went **up**, exposure per cycle went **down**. 8192 constant
> terms across ~320 messages is ~26 commitments per message, and every one is
> load-bearing (the SSA is the sum of all constant terms), so one lost message strands
> ~26 polynomials rather than one Feldman cell. Against that, there are 58× fewer
> messages to lose. Net per-cycle loss probability is far lower, which is why
> retransmission dropped down the priority list — but the retransmission request itself
> is now much simpler to express: a set of missing `polynomial_index` values at
> `coefficient_index == 0`, not `(coefficient, polynomial)` ranges.
>
> **Re-checked at `4f30a70629`: the retransmission half is still open.** Grep across
> `protocols/start`, `protocols/pix` and `transport/session` finds no NACK, resend or
> retransmission path for `SsaCommit` — the only hits are the Session layer's own frame
> retransmission, which is a different mechanism at a different layer. The sizing half has
> since been re-derived twice more and is in good shape: `start_protocol_channel_capacity`
> (`manager.rs:224`) now takes `min(quota_range.end() / PAYLOAD_SIZE, MAX_POLYS_PER_SSA)`,
> scales by the clamped `ssas_per_request` batch factor, and clamps the per-session term at
> `MAX_CONCURRENT_START_EXCHANGES` — the per-session addend used to be unclamped while the
> PIX term was bounded, which CodeRabbit caught.
> Note the capacity is **reserved**, not merely enforced: `crossfire`'s array flavour
> pre-allocates, which is why every term is clamped.
>
> One thing the H5 quota change did _not_ break here, worth recording so it is not
> re-derived: the quota grew by 1.5×, so `quota_range.end() / PAYLOAD_SIZE` grew with it —
> but that term is over-estimated by the whole `(threshold + surplus)` factor anyway and is
> clamped to `MAX_POLYS_PER_SSA`, which is what actually binds. The doc at
> `manager.rs:202-206` says so explicitly.
>
> **Tracking update:** the remaining NACK/retransmission work is a standalone item,
> [GitHub issue #8318](https://github.com/hoprnet/hoprnet/issues/8318), and is intentionally
> out of scope for this PR review. Keep the analysis below as the rationale for that issue, but do
> not count it in this document's implementation queue.

`manager.rs:976-977`:

```rust
let (start_protocol_tx, start_protocol_rx) =
    crossfire::mpsc::bounded_blocking_async(self.cfg.maximum_sessions + 10);
```

With `maximum_sessions` now defaulting to **100**, that is a 110-slot queue.
`dispatch_message` pushes with `try_send` and merely logs on failure
(`manager.rs:1902-1908`). PIX changed this channel's load from ~1 message per
session to **~19 000 `SsaCommit` messages per SSA cycle per session** — at 100
sessions, ~1.9 M messages per cycle generation.

Consequences of a single dropped `SsaCommit`:

- That `(poly_index, coeff_index)` cell is never filled. There is **no NACK, no
  retransmission and no timeout-driven resend** anywhere in the Start protocol.
- `all_entries_present` never becomes true → `CommitmentResult::Completed` never
  fires → no verifiers → every share for that cycle returns `MissingVerifier`
  forever (feeding H1) → the SSA is never recovered → the kill switch closes the
  session and burns the deposit (C2).

So one lost packet out of ~19 000 kills the cycle. This applies equally to
genuine network loss on the forward path, not just queue overflow.

**Fix:** size the Start channel independently of `maximum_sessions`; and add a
commitment-completion timeout on the Exit that requests retransmission of the
missing `(coefficient_index, poly_index)` ranges (the `SsaCommitmentBuilder`
already knows exactly which cells are empty).

---

### H3. Reconstructor memory is ~100–150 MB per active session; caches are deliberately unbounded

> **STATUS: all four tiers implemented.** Tier 3's residual — the node-wide live-cycle and tombstone
> budget, and the repeated-unpaid-cycle policy — is now closed on the combined branch; see the
> `STATUS: FIXED` block under Tier 3. Note that the per-cycle figures in the measurements below
> predate M9 and are corrected there.
>
> Also applied: the `threshold` 128 → 64 / `num_ssa_parts` 4096 → 8192 re-tune described
> at the end of this section, with `additional_shares` 64 → 32. The quota product is
> bit-identical (524 288 commitments), so C1's derived `quota_range` and H2's channel
> sizing are unchanged.
>
> The original figures were estimates against secp256k1. Re-measured on the real
> default build (BabyJubJub) by driving a full commitment set through
> `SsaReconstructor<HoprPixSpec>` and sampling `/proc/self/statm`:
>
> | Type                                                  | Size                |
> | ----------------------------------------------------- | ------------------- |
> | `PixGroup<HoprPixSpec>` (projective)                  | **96 B** (3 × 32 B) |
> | `PixGroupRepr` (compressed)                           | 32 B                |
> | `PixScalar`                                           | 32 B                |
> | `CompletedShare`                                      | **64 B**            |
> | `PartialSsaShareVerifier` / `SsaPartBuilder` (inline) | 72 B / 136 B        |
> | `TaggedEncryptedPartialSsaShare`                      | 112 B               |
>
> Measured **109.7 bytes of RSS per commitment** (13.7 MiB for 1024 × 128 =
> 131 072 commitments), which extrapolates to **~55 MiB per cycle** at the production
> 4096 × 128. That accounts almost exactly for the verifier set:
> `4096 × 129 × 96 B = 50.7 MiB` plus moka/`Arc`/`Mutex` overhead over 4096 entries.
>
> The probe fed no shares, so it **excludes** `SsaPartBuilder::shares` —
> `128 × 64 B = 8 KiB` per polynomial, `4096 × 8 KiB = 33.5 MiB` per cycle, which is
> **never freed**. Realistic end-of-cycle live set is therefore **~86 MiB per cycle**,
> ~110–170 MiB per session with pipelining, **~11–17 GiB at 100 sessions**. The
> original 100–150 MB/session estimate was right in magnitude.

Per live SSA cycle on the Exit (estimates for secp256k1, `k256::ProjectivePoint`
≈ 120 B):

| State                                                                                       | Size   |
| ------------------------------------------------------------------------------------------- | ------ |
| `SsaCommitmentBuilder::committed_polynomials` (nested `HashMap`, 524 288 × 33 B + overhead) | ~45 MB |
| `ssa_verifiers` — 4096 × `SsaPartBuilder` each holding 129 projective points                | ~63 MB |
| `SsaPartBuilder::shares` at completion — 4096 × 128 × ~64 B                                 | ~34 MB |

≈ **140 MB per session per live cycle**. Pipelining (`SsaAlmostRecovered` at 85 %)
keeps up to **two** cycles live. `ssa_builders`, `ssa_verifiers` and
`ssa_num_polys` are all built with **no `max_capacity`** by design
(`reconstructor/mod.rs:178-203`), and only TTL (10/30 min) reclaims them.

At 100 concurrent sessions this is **10–25 GB** of reconstructor state, with no
global cap and no backpressure. Session eviction is the only bound, and it is
driven by `maximum_sessions`, not by memory.

#### Fix plan

The central observation: **~84 MiB of the ~86 MiB per-cycle live set is provably dead
weight.** `SsaPartBuilder::add_share` early-returns on `self.reconstructed.is_some()`
_before_ touching either `self.verifier` or `self.shares`
(`reconstructor/utils.rs:101-104`), so once a polynomial part is reconstructed both its
129-point commitment vector (12.4 KiB) and its 128-share buffer (8 KiB) are unreachable
— yet they stay allocated until `remove_cycle` tears down the whole cycle.

**Tier 1 — release verification state on reconstruction. IMPLEMENTED.**

Measured by summing the live verification state across all builders (RSS is useless here —
glibc retains freed pages, so a _freeing_ win is invisible to it). Run at 512 × 64 and
rescaled from `TestSpec`'s secp256k1 point (120 B) to the production BabyJubJub point
(96 B):

| Point in the cycle              | Live verification state, 8192 × 64     |
| ------------------------------- | -------------------------------------- |
| Commitment just completed       | **48.7 MiB**                           |
| Half the shares delivered       | **24.4 MiB** (linear decay, confirmed) |
| End of cycle                    | **~0.2 MiB**                           |
| End of cycle, _before_ this fix | **80.7 MiB**                           |

So the peak drops 80.7 → 48.7 MiB (**−40 %**) and the tail collapses to nothing. With
pipelining, cycle _N_'s tail no longer overlaps cycle _N+1_'s install, so the per-session
figure goes from ~161 MiB to ~49 MiB (**−70 %**): **~11–17 GiB → ~5 GiB at 100 sessions.**

Note the re-tune left commitment memory essentially unchanged (48.7 MiB at 8192 × 64 vs
50.7 MiB at 4096 × 128), which is the "memory is invariant in the split" claim holding
empirically while CPU halved.

What was done:

- `SsaPartBuilder` gained a cached `min_shares` (it was derived from `poly_commitment.len()`,
  which stops being meaningful once that vector is released) and a
  `release_verification_state()` called the moment `reconstructed` is set. It assigns fresh
  empty `Vec`s rather than `clear()`, so the allocations are actually returned.
- `verifier` became **private**, with `spi()` and `constant_term()` accessors, so no caller
  outside the module can reach through it to `min_shares()` / `constant_term()` / `verify*()`
  on a released builder. That is what makes the release safe by construction rather than by
  convention.
- The cache entry is deliberately **kept** (stripped, not evicted) — see below.
- `reconstructed_polynomial_releases_verification_state_but_keeps_its_entry` pins both halves:
  released state _and_ retained entry, plus that an untouched polynomial keeps its
  commitments.

Why the entry is kept rather than evicted: invalidating `ssa_verifiers[spi]` on
reconstruction would make every late or surplus share for that polynomial return
`MissingVerifier`, which stashes the ack in `pending_ack_keys` for retry and so directly
worsens **H1**. A stripped-but-present builder keeps the cheap `Ok(Some(reconstructed))`
path instead. The residual cost of a stripped entry is ~200 B — 8192 of them is under
2 MiB per cycle.

**Tier 2 — write commitments straight into their final layout. EFFECTIVELY DELIVERED, by
M9 rather than as planned.** The plan below was written against a `polys × threshold`
Feldman matrix. M9 deleted the second dimension, and the rewrite that followed landed the
substance of this tier anyway:

- `SsaCommitmentBuilder::committed_polynomials` is now a **flat** `HashMap<PolynomialIndex,
PixGroup<S>>` (`reconstructor/utils.rs:446`) — the nested inner map, and its ~13 B/commitment of
  overhead over 8 192 entries, are gone;
- it stores **decoded** points, so the transposition copy and the second decompression at
  completion are gone too (that was H4);
- it is **drained** when the part builders are handed out, so the double-hold at `Completed` is
  gone;
- **L1 is fixed** as predicted — the `entry().or_default()` pre-check went with the rewrite.

What was _not_ built is the pre-allocated slot array with an occupancy bitset. It is no longer
worth building: the bitset existed to make duplicate detection and completeness O(1) over 524 288
cells, and completeness is now a counter comparison against 8 192 (`total_committed`,
`constant_terms_committed`, `installed_polynomials`, added with Tier 4). The remaining `HashMap` is
a fifth the size the flat array was meant to replace. **Treat this tier as closed** unless a
measurement says otherwise; the original text follows for the record.

> _Original plan:_ commitments land in `HashMap<PolynomialIndex, HashMap<CoefficientIndex,
PixGroup>>` and are copied into per-polynomial vectors at `Completed`. Instead pre-allocate the
> final layout in `new_exit_commitment` (dimensions are known there) and write each arriving
> commitment directly into its slot, with a `polys × threshold` occupancy bitset (524 288 bits =
> 64 KiB) for duplicate detection and completeness.

**Tier 3 — Exit-driven lifecycle and admission. IMPLEMENTED; its capacity residual is now
closed too — see the `STATUS: FIXED` block below.** The earlier version of
this finding treated the lack of a reconstructor-global counter as if it were the only
possible bound. That missed the component which actually owns admission: the Exit-side
Session supervisor.

At `6d671409ea` it provides the relevant structural bounds:

- only the last cycle in a batch can request the successor batch, and it does so once;
- that request is deferred until the triggering cycle's deposit has confirmed, so
  unfunded batches cannot recursively compound;
- commitment, deposit, recovery-idle and hard-recovery deadlines retire failed cycles
  or close the Session;
- `SessionPixAction::RetireSsa` drops the matching `SsaCommitmentGuard`, while Session
  closure clears all remaining guards, releasing live reconstructor state; and
- `SessionManager::maximum_sessions` supplies the node-wide Session admission bound.

The resulting memory ceiling is finite and derived from
`maximum_sessions × maximum overlapping batches × ssas_per_request × peak cycle memory`.
It is therefore wrong to leave this as implementation work for `lukas/pix` merely because
`ssa_cycles` has no `max_capacity`: size-evicting indispensable cycle state underneath
the supervisor would be incorrect.

**Supervisor-only residual:** validate that the configured product above is safe for the
node's memory budget (the defaults were not derived from that calculation). Also decide
the policy for batching above one: today an unfunded member can time out and be retired
while funded siblings keep the Session alive; there is no cumulative “unpaid cycle
failures” counter. If repeated failures must terminate the Session, that counter and
limit belong in `SessionPixSupervisor`. At the default `ssas_per_request = 1`, an unpaid
cycle is the only cycle and its failure already closes the Session.

> **STATUS: FIXED.** The residual above is closed. Three parts, and the first mattered most.
>
> **The per-cycle figure this whole tier was reasoned about was stale.** Three sites in code —
> `manager.rs:303`, `supervision/mod.rs:396`, `transport/hopr/src/config.rs:341` — stated ≈49 MiB of
> peak reconstructor state per cycle. That is `polys × threshold × 96 B`, the Feldman commitment
> matrix, and **M9 deleted that dimension**: `SsaPartCommitment` is now one `PixGroup` per
> polynomial, and the memory profile already measures the whole commitment pass at ~0.3 MiB. A budget
> derived from those numbers would have been sized against a structure that no longer exists. All
> three now state the re-derived model.
>
> What dominates today is `SsaPartBuilder::shares`, and how much of it is live depends on **share
> order, which the Entry chooses**. The shipped generator emits polynomial-major over a 256-wide
> window, so a conforming cycle peaks at a measured 7.8 MiB; nothing constrains the order, so a peer
> running anything else can hold _every_ polynomial one share short of its threshold and never
> release a buffer — a measured 39.1 MiB at the deployed dimensions, modelled at 41.1 MiB.
> `peak_cycle_bytes` models the second, because a bound a conforming peer can exceed fivefold is not
> a bound. It is a capacity bound rather than a
> security one: a share only reaches the Exit on a return SURB the Exit itself spends, so reaching
> that peak costs the peer a full quota deposit per cycle.
>
> **1. The budget is reserved at Session admission.** `IncomingSessionPixConfig::max_live_cycle_bytes`
> (3 GiB); a PIX Session charges `MAX_OVERLAPPING_BATCHES × ssas_per_request ×
peak_cycle_bytes(offered params)` when it is accepted, and is refused with the existing
> `StartErrorReason::NoSlotsAvailable` if it does not fit. The pipelining factor of two is structural:
> only a batch's last cycle may ask for a successor and it asks once, while a recovered cycle's state
> is already gone via `remove_cycle`. The default is derived, not picked — ≈82 MiB per Session at the
> deployed dimensions, so 3 GiB admits ≈37, comfortably covering the 10–30 clients per Exit the
> calibration profile models. **The same defaults with `maximum_sessions = 100` and no budget imply
> ≈8 GiB**, which is what makes this bind rather than decorate. It is a ceiling on _reservations_,
> not an allocation: nothing is claimed up front, and a node serving conforming peers holds five
> times less than it has reserved.
>
> Admission rather than the successor-request point, which was the other candidate: by then the Entry
> has funded a cycle and refusing costs it that deposit, whereas a Session refused at admission is one
> the peer retries elsewhere at no charge. And a runtime byte budget rather than validating
> `maximum_sessions × …` at config load, for exactly the reason M2's workload model was rejected — the
> product is a number no node holds, so the check would only ever reject the shipping defaults. What
> _is_ checked at load is that the budget admits one Session at the top of the operator's own
> `quota_range`, since otherwise the Exit advertises dimensions it will always refuse.
>
> Released explicitly by `close_session`, with `Drop` as the backstop, idempotently. Both, because the
> slot lives in a `moka` cache: `remove` hands the value back but drops the cache's own clone in a
> later maintenance pass, so a purely refcount-driven release returns the budget at an unpredictable
> time — a node whose Sessions had all closed could still refuse the next one. Caught by the release
> test, not by reasoning.
>
> **2. `retired_ssas` has a capacity, derived from that budget.** `MAX_RETIRED_SSAS = 262 144`, with
> the reachable count worked out in the declaration: the budget admits ≈37 concurrent Sessions, each
> retiring at most `ssas_per_request` cycles per generation and unable to replace one faster than
> `max_ssa_delivery_time`, so ≈67 000 within `unused_verifier_lifetime` at any batch size — a larger
> batch buys proportionally fewer Sessions, so the product is flat. ~3.9× headroom, ~72 MiB at the
> measured 256 B/entry. The "belongs with the global admission
> control the memory work still owes" note is replaced by that arithmetic — Part 1 is that admission
> control.
>
> **3. Repeated unpaid-cycle failures close the Session.** `SupervisorConfig::max_failed_cycles`,
> default 1, counted cumulatively across the Session rather than per batch — which is the point, since
> retiring a member leaves no trace on its siblings and the two losses that matter are typically in
> _different_ batches. One loss stays survivable, so the retire-and-hand-on-the-successor-gate path is
> unchanged; the second closes the Session, carrying the _first_ failure's reason. Nothing changes at
> the shipping `ssas_per_request = 1`, where the failing cycle is always the last one standing and
> that closes the Session first.
>
> Measurement: the profile gained a worst-case-share-order phase — the existing one feeds shares
> polynomial-major, so it only ever produced the conforming peak — which asserts the measured peak
> stays inside `peak_cycle_bytes`, plus per-entry costs for the two new constants. The same
> measured-constant-with-an-assertion shape as `AWAITING_ACK_ENTRY_BYTES`.

> **Base-branch history (`4f30a70629`).** The three caches this was written against are gone —
> H8's `SsaCycle` collapsed `ssa_builders` + `ssa_verifiers` + `ssa_num_polys` into one
> entry per cycle. On the standalone branch, one `ssa_cycles` entry was therefore the
> natural counting unit, and the budget was measured (6.4 MiB peak per Session,
> 0.63 GiB at 100 in-phase). The code says so itself at `reconstructor/mod.rs:345-348`,
> where `retired_ssas` is left unbounded with the note that a capacity "belongs with the
> global admission control the memory work still owes".
>
> The TTL-only `retired_ssas` tombstone cache remains part of the supervisor-side capacity
> audit, not a base-branch task. It is not live cycle memory: paid recovery removes the
> cycle state immediately, and the later `RetireSsa` guard drop installs the tombstone that
> prevents a racing commitment from resurrecting it. Its creation rate is now structurally
> bounded by Session admission and supervisor cycle turnover, but its worst-case retained
> count should still be calculated against that rate and `unused_verifier_lifetime`.
>
> `retired_ssas` is a tombstone set: a size eviction permits exactly the resurrection the
> tombstone prevents, so its capacity cannot be picked in isolation — it has to be chosen
> against the same concurrent-Session budget. If that calculation is unacceptable, account
> tombstones in the same supervisor-owned budget rather than adding an arbitrary cache cap.
>
> (CodeRabbit filed the cache under the same heading as a second complaint — that its
> comment under-described why the TTL must outlive the publish/retire race. **That half is
> fixed**: the declaration now records that retirement is also _permanent_, and that
> `abandoning_a_live_cycle_retires_it_rather_than_just_releasing_it` depends on the TTL
> outliving the race, so a reader who trusted the old comment and shortened it would have
> broken that test.)

The original recommendation was to add a commitment or byte budget directly in
`new_exit_commitment`. The supervisor branch supersedes that placement: these caches are
correctly built without `max_capacity` because size eviction would strand a live cycle, while
the Exit's request owner now provides the admission and retirement boundary. Any additional
budget or unpaid-failure limit should therefore be enforced before that owner requests another
batch, not independently inside the reconstructor.

**Tier 4 — sliding-window verifiers. IMPLEMENTED, and with no wire change after all.**

The Exit needed _all_ commitments before _any_ share could verify purely because they arrived
coefficient-major. It now installs a polynomial's verifier the moment that polynomial's own row
of `threshold` coefficients is complete, so — together with Tier 1's release on reconstruction —
the live commitment set is a sliding window over the polynomials in flight rather than the whole
`polys × threshold` matrix.

The original plan assumed a new message layout. It is not needed: the existing
`SsaCommit` message (fixed `coefficient_index`, list of `(polynomial_index, commitment)`) already
expresses everything, and only the **emission order** had to change. `new_multiple` now sends the
coefficient-0 pass first (as before), then walks _blocks of polynomials_, emitting each block's
remaining coefficients together. The block size is exactly what fits in one message, so:

- every message stays full — total message count is unchanged (~18 700 at production dimensions,
  versus ~24 600 if rows were sent one polynomial at a time, or ~20 200 with an explicit
  `(poly, coeff)` pair encoding at 36 B/entry);
- a whole block of polynomials becomes verifiable every `threshold − 1` messages instead of one
  batch at the very end.

Two decisions were load-bearing:

- **The Exit is order-agnostic.** It reacts to "this row is now complete", never to the sender's
  schedule, so a peer emitting pure coefficient-major still works — it just gets all verifiers at
  the end, exactly as before. Correctness does not depend on the peer cooperating.
- **A verifier is withheld until the SSA commitment is known**, even if its row is already
  complete. A reconstructed part needs an `SsaBuilder` to go into, and that requires all constant
  terms. Rows completing early queue up and are released in one batch when the last constant term
  lands. This is what makes the "rows before constants" ordering safe rather than share-losing,
  and it also means the part accumulator is now published _earlier_ than before — at
  SSA-commitment time rather than at full-commitment time. Both publication points are guarded by
  the retirement tombstone.

Fell out of the same work: progress tracking is now O(1) per commitment. Completion used to be
detected by walking every polynomial's map on every inserted batch — `O(polys)` per message, so
~8192 × ~18 700 ≈ 153 M map iterations per cycle — and the progress `trace!` recomputed the same
sum unconditionally, whether or not the level was enabled. Three counters (`total_committed`,
`constant_terms_committed`, `installed_polynomials`) replace all of it with comparisons.

Still open from the original note: the interaction with #8237's `CommitmentVerified` phase.
`is_verifiable` deliberately kept its old meaning ("every polynomial installed") so downstream
semantics did not shift, but "partially verifiable" is now a real state that #8237 may want to
model.

#### Considered and rejected

- **Store compressed (32 B) instead of projective (96 B) in the verifier** — 3× smaller,
  but reintroduces a decompression per point per share verification (128 per share,
  524 288 shares per cycle). Catastrophically undoes H4. No.
- **Store affine (64 B) instead of projective (96 B)** — a real 33 % cut of the verifier
  set (50.7 → 33.8 MiB) _if_ the MSM path accepts affine inputs. Worth checking, but it
  touches `ShareVerifierGroup`, a `vsss_rs` type.
- **Re-tune the dimensions for memory** — no win: commitment count is `quota / PAYLOAD`,
  so it is pinned by the negotiated quota. `polys × threshold` is invariant.

#### Free CPU win found while analysing this (relevant to M9, not H3)

> **Superseded by the M9 fix.** The cost model this argues from no longer holds: per-share
> verification is gone, so total EC work is `polys`, not `Q × threshold`, and commitment memory is
> `polys × 96 B`, not `Q × 96 B`. Both now favour _fewer_ polynomials and a _higher_ threshold —
> the opposite of the conclusion below. The 8192 × 64 split it produced is retained for now, but
> re-tuning against the new curve is an open item. Kept for the record.

For a fixed quota `Q = polys × threshold`, memory is `Q × 96 B` — **invariant in the
split** — but share verification is O(threshold) per share, so total EC work is
`polys × threshold² = Q × threshold`, i.e. **linear in `threshold`**. Halving
`ssa_part_size` from 128 to 64 and doubling `num_ssa_parts` from 4096 to 8192 keeps the
quota bit-identical (524 288 commitments, so the C1-derived `quota_range` is unchanged),
keeps memory identical, and **halves total verification work** — with no code change.
This also explains why the generator bench matrix tops out at `threshold = 64`
(`benches/ssa_generator_bench.rs:10-11`): the shipped default of 128 sits above the
entire benchmarked range, at the worst corner for CPU.

Caveat: `additional_shares` would need halving from 64 to 32 alongside it, otherwise the
H5 surplus ratio worsens from `(128+64)/128 = 1.5×` to `(64+64)/64 = 2.0×`.

---

### H4. Every coefficient commitment is EC-decoded two to three times

> **STATUS: FIXED.** `CommittedPolynomial` now stores decoded `PixGroup<S>` elements
> instead of the compressed representation, so each commitment is decompressed
> exactly once — on arrival, naturally spread across the ~18 700 separate
> `spawn_blocking` calls that deliver them. The completion path builds verifiers via
> the new `PartialSsaShareVerifier::from_decoded_commitments`, performing **no**
> decompression at all, and the constant-term path sums the stored points directly.
> That removes the single-threaded whole-set decode that previously ran while holding
> the per-SSA mutex. Verifiers are built by draining `committed_polynomials`
> per-polynomial so the peak does not hold both representations simultaneously.
> Equivalence with the byte path is pinned by
> `from_decoded_commitments_matches_from_serializable_commitments`.

- `add_transposed` validation loop decodes each incoming commitment and
  **throws the point away**, storing only the bytes
  (`reconstructor/utils.rs:209-215`).
- At `Completed`, `from_serializable_commitments` decodes all 524 288 again
  (`types.rs:220-229`).
- Constant terms are decoded a **third** time on the `SsaCommitmentDone` path
  (`utils.rs:315-320`).

Compressed-point decompression on secp256k1 needs a modular square root
(~20–50 µs). ~1 M decompressions ≈ **20–50 CPU-seconds per SSA cycle**, and the
`Completed` half of it runs **single-threaded inside one `spawn_blocking`** while
holding the per-SSA `parking_lot::Mutex`. With 100 sessions each hitting this once
per cycle, the blocking pool starves.

**Fix:** keep the decoded `PixGroup` from the validation step (store
`HashMap<CoefficientIndex, PixGroup<S>>` instead of `PixGroupRepr<S>`), which also
removes the third decode; and parallelise the `Completed` build with rayon as the
generator already does.

---

### H5. Quota under-charges by `(threshold + surplus)/threshold`, and `surplus_shares` is Entry-controlled and never validated

> **STATUS: FIXED — both halves, in `15458c1d69` and `20845807ae`.** The pricing
> decision the rest of this entry was waiting on has been taken, and taken the way the
> "Fix" line at the bottom proposed: the quota now counts `threshold + surplus`, and
> `surplus` is on the wire so the Exit can bound it.
>
> - **`PixParams`** (`protocols/pix/src/types.rs`) is now the single negotiated triple — `polys`,
>   `threshold`, `surplus` — encoded into the Start protocol's `additional_data` word. `surplus` was
>   previously Entry-local and invisible; it is now announced, range-checked by `PixParams::try_new`
>   at both ends, and `additional_shares` is capped at **255** rather than 4096 because it is one
>   byte of that word. The 65× abuse ratio is gone by construction, not by supervision.
> - **`pix_params_to_quota`** (`transport/session/src/types.rs`) multiplies
>   `emitted_shares_per_poly()` = `threshold + surplus`. `check_pix_params` compares _that_ against
>   `quota_range`, so the range now bounds what the Exit actually serves rather than two thirds of
>   it. Default quota 519.0 → **778.5 MiB**; see the note under the constants table.
> - Pinned by `quota_must_price_every_share_a_cycle_emits` (all four dimension corners, plus an
>   explicit assertion that charging the surplus exceeds charging the threshold alone) and
>   `default_quota_must_follow_the_default_dimensions`. The generator's own
>   `drain_shares_by_polynomial` pins the other side of the equality — that a cycle emits exactly
>   `polys × (threshold + surplus)` — and each doc comment names the other as the thing that must
>   move with it.
> - `d5b98d241d` swept the docs that still described the surplus as outside the quota, and
>   `transport/hopr/src/config.rs`'s `additional_shares` doc — which CodeRabbit
>   caught claiming the exact opposite of the truth — now states plainly that raising the dial costs
>   money rather than earning free service.
>
> **This closes the residual the block below called structural.** That paragraph argued a per-run
> ceiling could not see the honest 50 % gap "by construction", and it was right — but the answer was
> not a better ceiling, it was to stop treating the surplus as unbilled. **#8237 is no longer needed
> for H5 at all**; its `useful_shares` gate remains valuable for liveness (H2's stall, H6's repeat
> vector), which is where it should now be argued for.
>
> The original analysis follows, including the round-robin note, which remains correct as a
> statement about emission order and is now moot as a statement about pricing.
>
> ---
>
> _Superseded status:_ abuse vector addressed by PR #8237 (`lukas/session-pix-supervisor`, not on
> this branch); the honest-default gap remains a pricing decision.
>
> **Correction for the current stacked branch.** The description immediately below is the
> original #8237 design, not `6d671409ea`: supervision now treats every accepted share as
> liveness (`shares_seen`) and reserves `useful_shares` for payment/order progress. Consequently
> `max_served_without_progress` bounds genuine silence; it no longer detects a surplus-only run.
> That is correct now that H5 prices and negotiates the surplus, and per-polynomial surplus credit
> is capped at the negotiated value.
>
> #8237 adds `transport/session/src/supervision/` (~4 400 lines: supervisor, worker,
> `ServiceGate`) and measures progress as `SsaRecoveryProgress::useful_shares` —
> "non-duplicate, **non-surplus**, verified" shares. That is exactly the counter that
> stops advancing when an Entry inflates `surplus_shares`, and two independent
> mechanisms act on it:
>
> - `max_served_without_progress` (default **2048 packets**) — `ServiceGate::acquire`
>   parks callers once `served − served_at_last_progress` hits the ceiling; each
>   `RecoveryProgress` resets the watermark and wakes them.
> - `max_recovery_idle` (default **60 s**, service-gated so it re-arms when nothing was
>   served) — closes the session outright.
>
> So the unbounded 33× vector is bounded to ~2048 packets per no-progress run, and it
> costs the attacker its own already-paid deposit (C2). Honest operation is unaffected:
> the default `additional_shares = 64` produces no-progress runs at most 64 packets
> long, 32× below the ceiling.
>
> **Residual — and it is structural, not an oversight in #8237.** The honest 50 % gap
> is invisible to a per-run ceiling _by construction_: the gate must tolerate up to
> `surplus` consecutive non-useful shares for the loss-tolerance design to work at all.
> With defaults that is 4096 separate runs of 64, aggregating to 262 080 unbilled
> packets (~259 MiB per cycle), none of which comes near 2048. The supervisor solves
> liveness and abuse; it does not and cannot solve the accounting.
>
> What remains is therefore a decision to write down explicitly: either fold the 1.5×
> into `price_per_byte`, or make the quota count `threshold + surplus`. Worth resolving
> either way, because `SsaQuota`'s own doc comment advertises the quota as "the maximum
> amount of data that can be sent from Exit → Entry before the SSA deposit can be
> recovered", and that figure is 33 % below what actually flows. If the quota formula
> is changed, the `quota_range` derivation added for C1 must be recalibrated with it.
>
> **The round-robin emission change (`53a0d88be1`, see H9) does not close this**, and
> its commit message reads as though it might. It claims the SSA "now recovers at
> exactly `polys × threshold`", which holds only when `SHARE_EMISSION_WINDOW ≥ polys` —
> the generator's own test says as much (`ssa_generator_should_round_robin_within_the_
emission_window`: "with fewer polynomials than `SHARE_EMISSION_WINDOW` the whole SSA
> is one window"). At production the window is **256** against **8192** polynomials, so
> the last window's polynomials only start once the preceding 31 windows have each been
> drained to `threshold + surplus`. Recovery therefore completes at
>
> ```
> (polys − window)(t + s) + window · t  =  7936 × 96 + 256 × 64  =  778 240 shares
> ```
>
> against 524 288 priced — **1.484×**, versus 1.4999× for the old drain-to-exhaustion
> order. A 1 % improvement in the overage, not its removal.
>
> And the data actually delivered per cycle is **unchanged at `polys × (t + s)` =
> 786 432**. The Entry's `poly_queue` only drops a polynomial at
> `shares_generated >= threshold + surplus`, and the next cycle's polynomials are
> appended _behind_ it, so after the Exit retires the recovered cycle the Entry still
> emits the last window's leftover surplus before any share of the next cycle. Those
> shares are billed to nobody.
>
> H5's arithmetic is therefore intact as written. Round-robin is a loss-robustness fix;
> the pricing gap is untouched and still a decision to take.

```rust
// transport/session/src/types.rs:94
polys_per_ssa as SsaQuota * shares_per_poly as SsaQuota * HoprPacket::PAYLOAD_SIZE as SsaQuota
```

But the generator emits `threshold + surplus_shares` shares per polynomial before
moving to the next (`generator.rs:150-171`), and full recovery requires **all**
`num_polys` polynomials (`utils.rs:65-68`). So the return-path data actually
delivered before the next SSA is due is

```
polys × (threshold + surplus) × PAYLOAD  =  8192 × 96 × 1038 = 816 MB  (778 MiB)
```

against a charged quota of 519 MiB — **a 50 % free ride with stock defaults**.

Worse: `surplus_shares` comes from `PixGlobalConfig::additional_shares` on the
**Entry**, and is never announced on the wire nor checked by the Exit. Only
`polys_per_ssa` and `shares_per_ssa` (= threshold) are transmitted
(`manager.rs:1264`, validated at `manager.rs:1247-1260` against the _local_
generator). A modified Entry setting `additional_shares` to its validated maximum
of 4096 receives `(64+4096)/64 = 65×` the data it pays for — the ratio worsened
with the `threshold` 128 → 64 re-tune, since the cap on `additional_shares` did
not move with it.

**Fix:** make the quota `polys × (threshold + surplus) × PAYLOAD`, and put
`surplus` on the wire so the Exit can bound it (or fix it protocol-side).

---

### H6. `handle_ssa_request` will fund up to 27 SSAs per message, with no per-session cap on unrecovered SSAs

> **STATUS: PARTIALLY FIXED.** `handle_ssa_request` (now `manager.rs:3167`) rejects any
> request carrying more than `max_ssas_per_ssa_request` commitments, before generating
> anything or emitting any `ReadyToDeposit` — closing the one-packet → 27-deposit
> amplification. Pinned by `entry_rejects_ssa_request_exceeding_ssa_cap`.
>
> **Updated at `4f30a70629`.** `eeebc22702` turned the cap from a constant into a
> negotiated pair of config knobs, which is a real improvement and one new sharp edge:
>
> - the Entry's limit is `max_ssas_per_ssa_request` (default `DEFAULT_MAX_SSAS_PER_SSA_REQUEST` = 2);
>   the Exit's ask is `IncomingSessionPixConfig::ssas_per_request` (default 1, i.e. the unbatched
>   exchange). Both are clamped to `1..=MAX_SSA_BATCH_SIZE` (20) in `SessionManager::new`;
> - **the two are not negotiated.** An Exit whose `ssas_per_request` exceeds the peer Entry's
>   `max_ssas_per_ssa_request` loses every Session, and the field doc says so. `45fc808de4` at least
>   makes the refusal explicit to the Exit rather than silent;
> - the kill switch and the deposit awaiter are both scaled by the batch size, deliberately and for
>   the reason documented at `manager.rs:2949` — an unscaled awaiter would abandon a legitimately
>   late N-th deposit.
>
> **Still open, unchanged:** the repeat vector. An Exit can still issue many _separate_
> requests with strictly increasing indices (gaps are allowed by design —
> `manager.rs:3230`), so there is still no cap on _outstanding unrecovered_ SSA cycles
> per session. Closing that needs the Entry to track how many of its funded cycles
> remain unrecovered, which is state it does not currently keep. The blast radius is
> now bounded on the _Exit_ side by `SSA_TEARDOWN_SWEEP_WINDOW`, which caps the retirement
> sweep at teardown — but that bounds teardown cost, not the Entry's exposure to repeated
> funding.
>
> **Branch ownership after the merge:** this was correctly deferred from standalone `lukas/pix` to
> `lukas/session-pix-supervisor`. At combined HEAD `128e44c3d1`, those branches are now together, so
> the received-data-vs-emission residual described below is branch-local and immediately actionable.
>
> **Combined-branch state (`128e44c3d1`): strongly mitigated, with one narrower residual.** The
> successor gate now serialises requests per pseudonym and requires emission to have reached the
> last committed cycle and the exact boundary at which the earliest conforming Exit can signal
> early recovery. This prevents a malicious Exit from firing an unbounded sequence of increasing
> requests immediately; every accepted successor costs it almost a full cycle of Entry emission.
> `f488596d1c` also makes attacker-controlled batch validation all-or-nothing before any deposit
> event is published.
>
> The intended invariant does **not** require proof of Exit-side reconstruction. It is enough for
> the Entry to refuse another deposit until the Exit has delivered the configured portion of paid
> data back to it, with the protocol requiring `early_recovery_threshold >= 0.85`. The branch does
> enforce that floor in `validate_pix_supervision`, but the Entry-side gate measures
> `SsaShareGenerator::emission_progress`. That counter increments in `create_surb_for_path` when a
> share is put into a SURB during forward-packet construction; `crypto/packet/src/packet.rs` even
> notes that this consumption is not rolled back when later packet construction fails. It therefore
> proves shares were generated, not that their Exit→Entry packets were delivered.
>
> **Residual fix:** drive the successor/deposit gate from the Entry Session's received packet/byte
> counter (using the same conservative boundary implied by the 0.85 floor), rather than from
> generator emission. Then the Entry locally enforces "at least this much service arrived before I
> prepay again," without trusting the Exit to report recovery.
>
> ---
>
> **STATUS: FIXED.** `SessionSlot::returned_packets` counts Exit→Entry Session packets on both Entry
> receive paths — including the `surb_management: None` branch, which previously passed `session_rx`
> through untouched. It is a counter of its own rather than `surb_estimator.consumed`, which
> increments on the same event but is documented as a balancer _estimate_ and is only wired when the
> balancer runs; a deposit gate must be neither, and duplicating the increment fails safe.
>
> The measure is exact rather than approximate, which is what makes it usable without trusting the
> peer: a share is encrypted with the first relayer's challenge solution and rides a return SURB, so
> the Exit can only decrypt it by _using_ that SURB. One returned packet is one SURB consumed is one
> share unlocked. An Exit cannot inflate the count without unlocking exactly as many shares, which
> advances the recovery it is claiming.
>
> Two things the naive form of this gets wrong, and both had to be handled:
>
> - **A refused request is fatal on a delay.** `RequestSsa` is emitted once per index and never
>   retried — `supervisor.rs` gives `AwaitingCommitment` one exit, `CommitmentTimeout`, which closes
>   the Session. The old comment calling an early refusal "not fatal" held only because the gate read
>   emission, which the Entry advances itself, so a conforming Exit could not trip it. A gate reading
>   returned traffic can trip on reordering alone: the `SsaRequest` travels the same mixed path as the
>   packets that earned it and can overtake them. So a shortfall within one `SHARE_EMISSION_WINDOW`
>   is now _waited_ for (`SSA_SUCCESSOR_SERVICE_WAIT`, 2 s) rather than refused. Only a near miss
>   waits: an Exit that has returned nothing is refused immediately, which is what stops the wait
>   being a way to park one of this node's bounded Start-protocol slots per Session.
> - **The Entry's count structurally lags the Exit's progress.** The share unlocks when the _first
>   relayer_ acknowledges, upstream of the Entry, so everything lost after that point is progress the
>   Exit legitimately has and this node cannot see. The boundary is therefore discounted by
>   `threshold/(threshold + surplus)` — exactly the loss the surplus already insures. At the deployed
>   dimensions that is **455 312 returned packets, 69.5 % of a 655 360-share cycle**, against an
>   undiscounted 569 140. An Exit losing more than the surplus covers could not have reconstructed
>   the cycle anyway.
>
> Service rendered before the first commitment is excluded by a baseline taken on the 0 → n watermark
> transition: until a cycle is committed the generator holds no polynomials, so those SURBs carry no
> shares and the Exit may be served up to `max_predeposit_packets` of them unpaid. Crediting that
> prefix would let it bank unpaid service against the first cycle it _is_ paid for.
>
> Six tests, of which `entry_refuses_a_successor_the_exit_has_not_paid_for_with_returned_data` is the
> regression — it satisfies the emission half outright and the returned half not at all, and was
> admitted before. `returned_packets_are_counted_on_the_entry_receive_path` guards the failure that
> would otherwise be silent and permanent: a gate wired to a counter nothing feeds.

`manager.rs:2770-2835` iterates over **all** `msg.commitments`. The decoder caps
this at `MAX_SSAS_PER_REQUEST = 27` (`start/lib.rs:349`), but the Entry applies no
further limit. For each entry it:

1. runs a full `new_ssa_commitment` (4096 polynomials, 524 288 EC commitments —
   seconds of CPU, tens of MB), then
2. emits `ReadyToDeposit` → the strategy immediately withdraws
   `price_per_byte × quota`.

The only guard is `new_ssa_commitment`'s strict monotonicity on `ssa_index`
(`generator.rs:250-252`), and the code explicitly documents that **index gaps are
allowed by design** (`manager.rs:2768`). A malicious or buggy Exit can therefore:

- Amplify: one 1 kB packet → 27 × ~19 000 = ~500 000 outbound packets and minutes
  of Entry CPU.
- Force 27 simultaneous prepaid deposits, then simply never recover them —
  combined with **C2** every one of those deposits is permanently burned. The only
  brake is the per-deposit `max_ssa_allocation`; there is no aggregate cap and the
  attack can be repeated with fresh indices indefinitely.

**Fix:** cap the number of _outstanding unrecovered_ SSA cycles per session
(1–2 is all pipelining needs), reject `SsaRequest` batches larger than that, and
reject index jumps beyond the pipelining depth.

---

### H7. All PIX recovery events for all sessions are serialised through one stream

> **STATUS: FIXED.** The share-resolution branch now uses
> `then_concurrent(..., PIX_EVENT_DISPATCH_CONCURRENCY)` instead of a sequential
> `filter_map`, so a slow or lock-contended session no longer blocks PIX progress for
> every other session on the node. The ~80-line block that was duplicated across the
> `(Exit, Some)` and `(Relay, Some)` arms is extracted into `pix_event_stream` /
> `dispatch_share_resolution` / `session_pix_event_to_pix_event` (this also resolves
> L15).
>
> Concurrency needs no ordering guarantee here: `request_next_ssa` serializes on a
> per-session lock and re-checks the SSA index under it, so of the events belonging to
> one cycle exactly one advances the index and the rest are recognised as stale — the
> same mechanism the code already relied on for `SsaAlmostRecovered` vs
> `SsaRecovered`.

`transport/hopr/src/lib.rs:801-846` (and the duplicated relay branch at
`:905-…`): `ssa_share_resolution_events_rx.filter_map(…)` **awaits**
`smgr.dispatch_pix_event(…)` inline. That call reaches `request_next_ssa`, which
takes a per-session lock with a **30 s timeout** (`manager.rs:1638-1643`), runs a
`spawn_blocking` commitment generation, and performs a network send.

Because the stream is a single `filter_map` over a merged channel shared by every
session, one slow or stalled session blocks `SsaAlmostRecovered` /`SsaRecovered`
/`UnverifiableShare` delivery for **all 100**. A 30 s lock timeout on one session
stalls global PIX progress for 30 s; downstream cycles miss their pipelining
window and hit their kill switches.

The same file also duplicates this ~80-line block verbatim for the
`(Exit, Some)` and `(Relay, Some)` arms — worth extracting.

**Fix:** `for_each_concurrent` (or spawn per event) keyed so that per-session
ordering is preserved but sessions do not block each other.

---

### H8. At the deployed line rate a cycle outlives `unused_verifier_lifetime`, so it can never complete

> **STATUS: FIXED** in `3f9c049233` — see "Fix applied: `SsaCycle`" below. (The header
> said "not yet fixed" while the body documented the fix; corrected here.) Reproduced
> first by `a_verifier_idles_out_before_its_polynomials_shares_arrive`
> (`protocols/pix/src/reconstructor/mod.rs`), scaled to two polynomials and a one-second
> `unused_verifier_lifetime`. The test drives one cycle twice: uninterrupted it recovers, and
> with a single pause longer than the lifetime inserted at the polynomial-0 → polynomial-1
> boundary it does not. It asserts the mechanism, not just the outcome — the SSA builder is
> still live (its TTL is clamped to `max(incomplete_ssa_lifetime, unused_verifier_lifetime)`),
> the late polynomial's verifier is gone, and its acks are stranded in a `pending_acks` bucket
> whose only drain — installation of that verifier — has already run and will not run again.
>
> The test is written to fail loudly if the defect is fixed, so it converts into the regression
> test by inverting its second assertion.
>
> M9 changed the balance here in both directions. The secondary margin it notes below —
> `incomplete_commitment_lifetime` (2 min) against the commitment window — is now comfortable,
> because the window collapsed 64× with the Feldman matrix. The primary defect is untouched and
> matters _more_: with the CPU ceiling lifted, cycle wall-clock is set by the line rate alone.
>
> ### Fix applied: `SsaCycle`
>
> One cache entry per cycle, holding the part accumulator and all `num_polys` part builders,
> replacing `ssa_builders` + `ssa_verifiers` + `ssa_num_polys`. The idle timer is now refreshed by a
> share for _any_ polynomial of the cycle — the property that actually matters, and correct at any
> line rate, with no constant left to keep calibrated against quota and rate.
>
> This was only possible because of M9: `insert_coefficient_commitments` publishes the accumulator
> and every part builder on **one call**, so the entry is immutable and fully populated at
> construction. Under the old Feldman matrix, rows completed progressively and the same design would
> have needed per-slot interior mutability and a partially-installed state.
>
> **What it deleted.** Three caches held in lifetime lockstep by hand became one whose lifetime is
> coherent by construction:
>
> - `ssa_num_polys` and its ~22-line "verifier-widow" doc — the widow existed only because the
>   builder and the verifiers could expire out of step.
> - The TTL clamp `incomplete_ssa_lifetime.max(unused_verifier_lifetime)` and its ~20-line comment.
>   `incomplete_ssa_lifetime` was already inert at default config (the clamp always picked 1800 s
>   over 600 s), and the field is now removed from `SsaReconstructorConfig`.
> - The "publish the accumulator BEFORE any verifier" hazard: a share can no more find a verifier
>   without a builder than it can find half an entry.
> - `remove_cycle`'s per-polynomial invalidate loop, `retire_ssa`'s three-way `num_polys` fallback
>   chain, and the tombstone undo loop — all of which existed to enumerate keys spread across caches.
>
> **The line count is roughly neutral** (+24 production lines: −47 in `reconstructor/mod.rs`, +71
> for `SsaCycle` and its documentation in `utils.rs`). The win is not volume — it is that four caches
> become two, and the hand-maintained invariants that kept three of them in step stop existing. There
> is also one fewer cache lookup on the acknowledgement hot path.
>
> **Memory improved too**, measured with `tests/memory_profile.rs` at production dimensions before
> and after the change:
>
> | per Session, one cycle | before   | after               |
> | ---------------------- | -------- | ------------------- |
> | at commitment install  | +6.0 MiB | **+3.7 MiB**        |
> | peak live state        | 9.2 MiB  | **6.4 MiB** (−30 %) |
> | ×100 Sessions in phase | 0.90 GiB | **0.63 GiB**        |
>
> That is the moka per-entry overhead (8192 entries × ~200 B) and 8192 `Arc` allocations
> disappearing. It also settles the one concern the change raised: a stalled cycle no longer sheds
> state polynomial by polynomial, so it pins its whole live set for the lifetime — but that live set
> is now smaller than what the gradual version peaked at.
>
> **Two related items closed by the same change:**
>
> - **A latent share-dropping bug in `pending_acks`.** It was `CacheBuilder::new(2 *
MAX_POLYS_PER_SSA)` = 32 384 entries keyed _per polynomial_, but one cycle can create `num_polys`
>   = 8192 of them — so past roughly four concurrent cycles node-wide, moka began LRU-evicting buckets
>   and silently dropping real shares. The capacity comment argued from the per-session pipelining
>   factor, but the cache is node-wide. Re-keyed by cycle (with per-polynomial sub-buckets, so the
>   well-argued 128 sub-cap is unchanged), the capacity unit is cycles and the mismatch is gone.
> - **The cycle total was unbounded.** `num_polys × 128` is a million entries at production
>   dimensions. `MAX_DEFERRED_ACKS_PER_CYCLE = 8192` is derived rather than chosen: the drain already
>   discards any ack whose share has left `awaiting_acks`, which expires at `max_ack_await_time`
>   (30 s), so the ceiling need only cover the shares one cycle can receive inside that window —
>   ~5 400 at the 1.5 Mbps cap, so 8192 is ~1.5× headroom and ~786 KB per cycle.
>
> **Regression tests.** `a_cycle_stays_live_while_a_single_polynomial_goes_untouched` is the
> inverted reproduction: shares spaced at half the lifetime so the cycle is never idle, but
> polynomial 0's run takes twice the lifetime, and it asserts its own premise (that the elapsed time
> really does exceed the lifetime before polynomial 1's first share).
> `a_cycle_with_no_shares_at_all_still_expires` is the other half — widening the scope is only
> correct if reclamation still happens. `parts_of_one_cycle_lock_independently` guards the one way
> this change could have been a severe regression: collapsing the cycle to a single mutex would
> serialise every share of a Session.

Exits are currently capped at **1.5 Mbps per Session**. One SSA cycle is
`polys × threshold × PAYLOAD_SIZE` = 519 MiB of quota, so a cycle takes

```
544 210 944 B / 187 500 B/s = 2902 s = 48.4 min
```

`unused_verifier_lifetime` defaults to **30 minutes** and is a `time_to_idle` on
`ssa_verifiers` (`reconstructor/mod.rs:238-240`). A verifier's timer is only refreshed when a
share for _its own_ polynomial arrives — `defer_ack` deliberately probes with `contains_key`
rather than `get` to avoid touching it.

Meanwhile the commitment matrix is only 19 MiB against 519 MiB of payload, so at equal
forward/return rates **every verifier is installed inside the first ~100 s of the cycle** and
then waits. Shares arrive in polynomial order (`next_share` drains `poly_queue.front_mut()`),
so polynomial _k_'s shares land around `t ≈ 2902 · k/8192`. Anything past ≈32 min exceeds the
idle window, is evicted, and its shares then take `VerifierNotReady` → `defer_ack` → a bucket
with a 30 s TTL → dropped. Those polynomials never reconstruct, the cycle dies on the deposit
kill switch, and the deposit is burned (C2).

The general condition is `quota / line_rate < unused_verifier_lifetime`, i.e. the cycle only
completes above **≈2.4 Mbps**. The cap is 1.5 Mbps, and slower Exits are the common case.

A second, thinner margin: `commitment_builder` is `time_to_idle(incomplete_commitment_lifetime)`
= **2 min** (`:209-211`) against a ~100 s commitment window. Each insertion refreshes it, so a
steady stream is fine, but any forward-path stall over 2 min (M8) drops the builder mid-matrix.

This also means **Tier 4's sliding window does not slide at this line rate**. Commitments
arrive ~28× faster than shares drain them, so the live set is the whole matrix from ~100 s
onward. Tier 1's release-on-reconstruction still gives the linear decay (measured below); Tier
4's windowing does not engage. The claim in H3 that Tier 4 makes the live set "a window over
the polynomials in flight" holds only when the two streams flow at comparable rates.

Fixes, roughly in order of preference:

1. **Make verifier liveness per-cycle rather than per-polynomial.** The TTI exists to reclaim
   abandoned cycles, but applying it per polynomial evicts verifiers a live cycle has not
   reached yet. A per-cycle liveness entry already exists (`ssa_num_polys`, same TTI); driving
   reclamation from it is correct at any line rate and removes the calibration coupling.
2. **Couple the constants.** The Exit already validates the offered quota at establishment
   (`pix_config.quota_range`, from C1). That is the natural place to enforce
   `quota / min_supported_rate < unused_verifier_lifetime` so the two cannot drift apart.
3. **Shrink the quota.** `quota < rate × TTL` = 337 MB at 1.5 Mbps and 30 min; `threshold = 32`
   gives 272 MB and a 24-minute cycle. Also cuts per-share verification (405 µs at t=32 vs
   487 µs at t=64) and halves the live commitment set, at the cost of one on-chain deposit per
   24 min instead of per 48.

A targeted test is cheap: install a commitment, advance a mocked clock past
`unused_verifier_lifetime`, acknowledge a share for a high-index polynomial, assert it resolves.

---

### H9. SURB ring-buffer eviction silently destroys an SSA cycle

> **Found and fixed after this review was written** (`53a0d88be1`). Recorded here because
> it is the most severe defect the review missed, and because it is a class the review's
> method could not have caught: it lives in the interaction between the PIX crate's
> emission order and the transport's SURB store, neither of which is wrong on its own.

A PIX share only reaches the reconstructor when the Exit **uses** the SURB carrying it —
the pipeline registers the share on the outgoing packet's ack challenge, and the ack
half-key is what decrypts it. A SURB overwritten in the Exit's per-pseudonym ring buffer
is therefore a permanently lost share, and the loss is **silent**: a starved polynomial
never fails a check, it simply never reaches `threshold`, so the cycle stalls at
`polys − 1` recovered parts with nothing reported and no event fired. Since the SSA is
the sum of _every_ polynomial's constant term, one starved polynomial kills the cycle and
burns the deposit (**C2**).

Three separate things made that reachable, and each had to be addressed:

- **The emission order concentrated the damage.** `next_share` drained
  `poly_queue.front_mut()` to `threshold + surplus` before moving on, so the SURB stream
  was runs of **96 consecutive same-polynomial shares** at production dimensions. A ring
  buffer evicts its _oldest_ entries — a contiguous run — so a burst landed on one or two
  polynomials, and losing more than `surplus_shares` of any single one is fatal. Emission
  is now **round-robin across `SHARE_EMISSION_WINDOW = 256` polynomials**, spreading a
  contiguous loss of up to `surplus_shares × 256` over enough polynomials for the surplus
  to absorb it — which is what the surplus exists for. The window is bounded rather than
  spanning the SSA because the Exit holds a part builder's shares until that part
  reconstructs; 256 keeps the peak near a megabyte instead of the ~30 MB full
  interleaving would cost. Nothing on the wire changes: every share carries its own
  `SsaPolynomialId`, mixed into both `msg_to_scalar` and the encryption KDF, so the Exit
  files by polynomial regardless of arrival order and interoperates with either ordering.
- **The target had no headroom to overshoot into.** `maximum_surb_buffer_size` was wired
  to `rb_capacity` exactly, so a Session sitting at its balancer target was sitting at
  buffer capacity — the normal steady state when Entry → Exit traffic far exceeds the
  reverse, because the Exit then drains almost nothing. Every overshot SURB was an
  immediate eviction. The ceiling is now two thirds of the ring buffer
  (`surb_buffer_target_ceiling`, `transport/hopr/src/lib.rs:388`).
- **The buffer was small relative to a cycle.** `rb_capacity` goes from 15 000 to
  100 000, which is only affordable because of **H10**.

Regression coverage pins the property that actually protects the cycle rather than the
symptom: any contiguous run of `n` emitted shares must touch `min(n, window)` distinct
polynomials. Tests that assumed polynomial-major arrival were rewritten to group by
polynomial rather than to assume contiguity.

**One claim in that commit message is over-general** and should not be relied on: it says
the SSA "now recovers at exactly `polys × threshold`", which holds only for
`polys ≤ SHARE_EMISSION_WINDOW`. At production it is 778 240 shares, not 524 288, and the
data delivered per cycle is unchanged. See the note under **H5** — this did not close the
pricing gap.

---

### H10. `SurbRingBuffer` reserved its whole capacity per pseudonym, and any peer could mint one

> **Found and fixed after this review was written** (`c4f22c55b7`). Surfaced by H9's
> capacity increase, but it predates it.

`SurbRingBuffer` wrapped `ringbuffer::AllocRingBuffer`, which takes its entire capacity at
construction and never grows. That made `rb_capacity` a **reservation** rather than a
ceiling — and the reservation was reachable by anyone. The pseudonym a buffer is filed
under is chosen by whoever sent the packet: `decoder.rs` hands any `HoprPacket::Final`
carrying a SURB to `insert_surbs`, which mints a buffer for a pseudonym it has never
seen, with **no Session and no handshake behind it**. One valid packet with one SURB
therefore reserved a full buffer — 16.8 MB of address space at the 100 000 default — and
`max_pseudonyms` is 10 000, putting the ceiling around **205 GB of VSZ**.

Resident memory was never the exposure: untouched pages cost nothing, and 10 000 idle
buffers measured ~85 MB RSS. The reservation is real to anything that accounts address
space, though — strict overcommit, `ulimit -v`, and one large mapping per pseudonym
against `vm.max_map_count`. At the old `rb_capacity` of 15 000 the same path reserved
2.52 MB per pseudonym and ~25 GB at the cap, which is why it went unnoticed.

Backing the buffer with a `VecDeque` and an explicit bound keeps the semantics identical —
FIFO, oldest evicted once full — while allocation tracks what is actually held. Measured
over 100 buffers at capacity 100 000:

```
before   after new(): dVSZ = 2 150 800 KB, then 0 KB at every fill step
after    after new(): dVSZ =         0 KB, then growing with occupancy
                      (172 KB at 1k, 2 520 KB at 10k, 10 752 KB at 100k)
```

A fresh pseudonym now costs ~4 KB instead of 16.8 MB; a buffer that genuinely fills
reaches the same footprint as before, having earned it. This is what makes H9's larger
`rb_capacity` defensible, so the two belong together.

## P2 — Medium

### M1. `pending_ack_keys` inner cache has no TTL and a 1 M capacity, holding raw ack half-keys

`reconstructor/mod.rs:573-575` builds the per-peer stash with
`CacheBuilder::new(max_awaiting_acks)` and **no `time_to_live`** — unlike
`awaiting_acks`'s inner cache which does set one (`mod.rs:486-488`). The outer
`time_to_idle(30 s)` only expires the peer entry if the peer goes _completely_
silent, which never happens on an active relay. Worst case ~200 MB of
`(HalfKeyChallenge, HalfKey)` per peer, and those `HalfKey`s are live decryption
secrets held longer than necessary.

### M2. `max_awaiting_acks` (1 M) × `max_tracked_peers` (2 000) has no global bound

> **FIXED — by a runtime bound, after the modelled one was correctly rejected.** Both residuals
> below were confirmed against the tree and are closed; the modelled validation that carried them
> has been deleted rather than repaired.
>
> **Why the alternative remedy was not taken.** The suggestion was to close this either with a
> runtime budget or by deriving validation "from an actually enforced maximum packet rate and the
> configured Session ceiling". The second is not available, for two reasons found on inspection:
>
> - **There is no enforced maximum packet rate to derive from.** The only rate machinery is the SURB
>   balancer's _dynamic_ `RateController` (`transport/session/src/balancer/rate_limiting.rs`), and
>   `SessionCapability::NoRateControl` exists precisely to switch it off.
> - **Deriving from `maximum_managed_sessions` reproduces M2's own error.** It validates to 100 000
>   (`transport/hopr/src/config.rs:939`), which yields 18.1 M shares/s and an admissible window of
>   **148 ms** — the default configuration would stop validating. Multiplying that unrealistic
>   maximum by a modelled rate is the same "product of two independently-unrealistic maxima" this
>   finding is about, one level up.
>
> **What was done instead.** `SsaReconstructorConfig::max_ack_buffer_bytes` (default 1 GiB) is
> enforced by the reconstructor as shares arrive, so the bound is on occupancy and is indifferent to
> what any maximum claims — Session count, packet rate and acknowledgement window all drop out of
> it. `insert_encrypted_share` refuses past the ceiling with the new `PixError::AckBufferFull`,
> which the pipeline already logs and counts (`METRIC_SHARE_INSERT_FAILURES`).
>
> The accounting is the substance of the change. Entries leave the buffer four ways; an inner-cache
> eviction listener catches three of them exactly, but the fourth — a peer's whole entry leaving
> `awaiting_acks`, dropping its inner cache — cannot be listened for, because dropping a moka handle
> does not run its eviction listener. Left alone that residue accumulates _upward_, and an
> over-count is the dangerous direction: it would eventually refuse every share while the buffer sat
> empty. So the counter is a hint, and `resync_ack_buffer` recomputes ground truth at the one moment
> being wrong would cost something — when the counter says the buffer is full. Throttled by
> `try_lock` plus `max_ack_await_time / 16`, so the overload path cannot amplify into an
> `O(max_tracked_peers)` scan per insertion.
>
> **Adjacent, out of scope:** `HoprUnacknowledgedTicketProcessor`
> (`protocols/hopr/src/ticket_processing.rs:163-181`) has the identical nesting and the identical
> residue, with a comment saying so and a TODO ([#8014](https://github.com/hoprnet/hoprnet/issues/8014))
> noting it cannot afford `run_pending_tasks()` in the listener. There it only skews metrics, so it
> is not urgent — but the fix now exists next door.
>
> **The two residuals, closed.**
>
> - The hard-coded ingest ceiling is gone with the rest of the model. Nothing in the enforcement
>   path assumes a Session count or a packet rate.
> - The `as_secs()` flooring is gone with the function that contained it. (It was fixed in place
>   first — exact to the millisecond, boundary at 148 306 ms — before the runtime design made the
>   whole check unnecessary.)
>
> **Cost.** On the production-shaped path — `acknowledge_shares/sustained_quota_rate`, distinct keys,
> insert plus redeem — criterion reports **no change** against a `73efa5c8a1` baseline (p = 0.65 and
> p = 0.61). The `insert_encrypted_share/single_share` microbenchmark does show a regression, but it
> replaces _one_ `ack_challenge` 5.5 M times, so every iteration fires a `Replaced` eviction
> notification; production never takes that path, since every share carries a unique challenge. That
> benchmark also proved too noisy to read on this machine — identical code measured 610 ns, 880 ns
> and 1.05 µs across runs, and one apparent "+17%" was against a stale stored baseline.
>
> **What the earlier, modelled commit did establish**, and what remains true. The ~500 GB product is
> arithmetically right but not the expected occupancy at the stated operating point.
>
> An entry exists only for a share this node has already **sent**, so reaching 2×10⁹ entries
> inside the 30 s window needs ~66 M packets/s of egress. The reachable ceiling is
> `share-emission rate × max_ack_await_time`. At the operating point
> `protocols/pix/tests/memory_profile.rs` already models — 1.5 Mbit/s per Session ÷ 1038 B per
> share × 100 concurrent Sessions — that is ~18 100 shares/s, so **543 000 entries** node-wide.
> 0.03 % of the product.
>
> The two caps are also independent backstops against **mutually exclusive** concentrations.
> `max_awaiting_acks` sizes one cache _per peer_ (`reconstructor/mod.rs:1027`) and has to cover
> every Session returning through a single first-relayer — ~543 000 entries — which makes 1 M the
> right order of magnitude rather than an oversight. `max_tracked_peers` (`:323`) covers the
> opposite case. A product ceiling tight enough to bite would force one of them below what its own
> case needs, and a `max_awaiting_acks` set too low does not save memory: it size-evicts shares
> before their acknowledgements arrive, silently losing them. That would have traded a real
> regression for a theoretical bound.
>
> The superseded `validate_ack_buffer_budget` (`transport/hopr/src/config.rs`) bounded
> `min(caps product, ingest rate × window) × AWAITING_ACK_ENTRY_BYTES` against a 1 GiB budget. The
> `min` is load-bearing: a small node (`max_tracked_peers: 10`, `max_awaiting_acks: 10_000`, 40 MB
> whatever the window) keeps a long window it cannot abuse, which is what separates this from a
> plain `range(max = …)` on the window. `AWAITING_ACK_ENTRY_BYTES` is **measured**, not derived —
> `size_of` accounts for 145 B of it and moka's per-entry bookkeeping for the other 244 B; the new
> `awaiting_ack_entry_cost` profile reports 389 B/entry at 100 000 entries and the constant rounds
> to 400. The resulting operator-visible ceiling on `max_ack_await_time` is 148 s, asserted as a
> literal so a change to any of the three constants has to restate it.
> That function and its 148 s ceiling are now deleted; the runtime occupancy budget above is the
> current implementation.
>
> Defaults are unchanged and sit at ~207 MiB under the model, a fifth of the nominal budget. The
> earlier instruction against adding admission on `lukas/pix` concerned indispensable live-cycle
> eviction in H3; it does not make a workload estimate into a runtime acknowledgement-buffer cap.
>
> **The comment half was FIXED earlier.** The `SAFETY` comment at `transport/hopr/src/lib.rs:884`
> no longer quotes values at all — it states the invariant it is actually asserting (the
> struct's defaults sit inside the ranges its own `Validate` derive enforces, so
> `validate().expect()` cannot panic) and records _why_ it is phrased that way. Quoting
> `max_awaiting_acks = 10_000_000` against an actual `1_000_000` is precisely the failure
> mode: a comment naming a constant it does not reference cannot be checked by anything.
> The two sibling sites at `:920` and `:946` already used the value-free wording, so all
> three now agree.
>
> **At `4f30a70629`, the bound itself was wholly open** — see the historical note below.
>
> **Re-checked at `4f30a70629`:** unchanged. `max_tracked_peers` still defaults to 2 000
> (`#[validate(range(min = 10))]`, no max) and `max_awaiting_acks` to 1 000 000
> (`#[validate(range(min = 10000))]`, no max), at `reconstructor/mod.rs:36-51`. Neither is
> reachable by an operator, which is why this belongs with **M3** rather than standing
> alone: threading a real `SsaReconstructorConfig` through is the prerequisite for anyone
> being able to set a bound. It should be calibrated alongside the supervisor branch's
> **H3 Tier 3** capacity audit, but it is a separate acknowledgement-buffer bound.

`reconstructor/mod.rs:36-51`. Only the 30 s `max_ack_await_time` TTL keeps this
finite in practice. The configured product is ~500 GB.

### M3. Reconstructor is not configurable at all

> **FIXED.** `PixReconstructorConfig` (`transport/hopr/src/config.rs`) mirrors all eight
> `SsaReconstructorConfig` fields and is nested under `pix.reconstructor`, so the Exit side is now
> as configurable as the Entry side already was. All eight are exposed: a curated subset would have
> been arbitrary, and `use_batch_verification`'s own doc comment already argues for keeping it
> tunable pending a concurrent-pipeline measurement.
>
> The mirror was chosen over embedding the protocol type directly, so duplication is the cost and
> two guards pay it. `From<PixReconstructorConfig> for SsaReconstructorConfig` is written
> exhaustively in both directions, so a field added to either struct fails to compile until it is
> mirrored — the guard that actually matters, and free. Defaults are not duplicated at all: both
> `#[default(…)]` sites read the same new `SsaReconstructorConfig::DEFAULT_*` associated constants,
> so they cannot disagree by construction. Ranges are not duplicated either — one schema function
> converts and delegates to `SsaReconstructorConfig::validate()`, leaving `hopr-protocol-pix` the
> single source of truth for every per-field bound.
>
> One `ssa_reconstructor()` helper in `transport/hopr/src/lib.rs` is now the only place a config
> becomes a reconstructor, and it uses `try_new` (added for L14) so an out-of-range operator value
> is a startup error rather than a panic. It feeds **three** sites, not two: the Entry-side "dummy"
> at what was `:939` gets the same config, because it is not provably unreachable — the comment
> beside it already records that an inbound `UsePIX` reaches `handle_ssa_commit` there — and a knob
> that binds on some node roles and not others is the same defect shape as L20's comment.
> `wire_exit_pix` now takes the built reconstructor, like it already took the generator, so it stays
> infallible.
>
> Regression guard: `pix_configs_are_reachable_from_serialized_config` was extended rather than
> joined by a new test. It exists for exactly this defect one struct up — fields that were
> `serde(skip)` and so pinned to their defaults — and it now also asserts that a deserialized
> `max_ack_await_time` survives the conversion the constructors consume.
>
> **Previously:** partially improved by the C1 fix: `PixGlobalConfig` and
> `IncomingSessionPixConfig` are now operator-settable. `SsaReconstructorConfig`
> is still hard-coded — the point below stands.
>
> **Re-checked at `4f30a70629`: two production sites now, not three.**
> `transport/hopr/src/lib.rs:939` and `:1475`. The third disappeared into
> `4f30a70629`'s extraction of the shared Exit/Relay PIX wiring (the remainder of **L15**), which
> is a smaller surface to thread a config through — the work got cheaper, not done.
>
> This has also become the **hinge for two other open items**, which is the argument for
> doing it next: **M2**'s bound is unsettable until a real config reaches the constructor,
> and #8237's `validate_pix_supervision` validates against
> the default rather than the value in use. **H1's remaining residual rides here too** — a
> parked `RecoveredSsa` still needs the reconstructor to push rather than be pulled, which
> means a sink on the same constructor. One change, five entries.
>
> Of those, **M2 is now closed** — by a runtime budget in the reconstructor rather than by the
> configuration validation M3 unblocked, since a validated model cannot bound what the node does
> not enforce (see M2)
> and #8237's validation is **unblocked but not done** — retargeting
> `validate_pix_supervision` at the configured value is a stacked-branch edit. **H1's residual is
> likewise unblocked and not done**: `ssa_reconstructor()` is now the single constructor a sink
> would be attached to. Unlike the supervisor validation, that delivery path belongs on the base
> branch.
>
> **Stacked-branch update (`6d671409ea`): partially improved, still open.** The two
> production sites now call one `ssa_reconstructor_config()` helper, and `SessionManager::start`
> validates supervisor deadlines against `pix.share_processor.config()`, i.e. the reconstructor
> actually installed. This closes the "validator and constructor can silently use different
> defaults" part. The helper still returns `SsaReconstructorConfig::default()` and no field is
> exposed through operator configuration, so the configurability finding itself and M2's bound
> remain open. H3 Tier 3's remaining admission policy is independently owned by the supervisor.

The original seven fields are now settable under `pix.reconstructor`, as is M2's subsequently
added eighth field, `max_ack_buffer_bytes`:

`early_recovery_threshold`, `max_tracked_peers`, `max_awaiting_acks`,
`unused_verifier_lifetime`, `incomplete_commitment_lifetime`, `max_ack_await_time` and
`use_batch_verification` could not be tuned by an operator — which matters given H3/M2, and
matters more now that `use_batch_verification` has been flipped to `false` on the strength
of one benchmark shape that the Exit's ack pipeline does not use. (`incomplete_ssa_lifetime`
is no longer in the struct; H8's `SsaCycle` removed it.)

> **M4–M7 describe `impls/strategy`, which `7ab2e5d7f8` deleted from this branch.**
> The implementation now lives in the standalone `hopr-strategy` repository and handles
> the `PixEvent`s emitted by `hopr-lib`. The findings are kept below as historical context;
> none is verifiable against, or actionable on, `lukas/pix`. The same applies to L8–L12
> and L17.

### M4. In-flight sweep guard leaks on keypair-construction failure

> **STATUS: FIXED.** The guard is invalidated on the error path before returning, and
> `in_flight_sweeps` now carries a 10-minute TTL as a second line of defence against
> any other leak (see M7). Regression test:
> `test_unusable_secret_releases_the_in_flight_sweep_guard`.

`impls/strategy/src/non_anonymous_pix.rs:291-294`:

```rust
self.in_flight_sweeps.insert(private_key_recovered.id, ());
let chain_key = ChainKeypair::from_secret(private_key_recovered.secret.0.as_ref())
    .map_err(StrategyError::other)?;          // early return, guard never released
```

`in_flight_sweeps` is a capacity-only cache with no TTL, so that `PixAddressId` is
permanently blocked from ever being swept again in this process — including by the
startup replay path (`:355-359`), which checks the same guard. The store entry
survives, so the funds sit unrecoverable until manual intervention. Low
probability (requires an out-of-range scalar) but the failure is silent and
permanent.

### M5. `NewDepositAddress` withdrawal failure has no retry

> **STATUS: FIXED.** The withdrawal is now wrapped in `backon` exponential backoff with
> `MAX_DEPOSIT_WITHDRAW_RETRIES = 3` (≈7 s of total delay). The budget is deliberately
> smaller than the sweep path's: the Exit abandons the deposit after `max_deposit_wait`
> (60 s), so a longer backoff would outlive the session it is trying to save. The
> `in_flight_destinations` guard is held across the whole retry chain.

`non_anonymous_pix.rs:176-186` logs and returns `Err`; `run()` only logs
(`:577-580`). The SSA is then never funded, the Exit's kill switch fires, and the
session dies. Contrast the sweep path, which has `spawn_sweep_retry` with
exponential backoff. A transient RPC failure at the wrong moment kills a session
that would otherwise be worth 500 MB of traffic.

### M6. Deposit tracking spawns an unbounded number of polling tasks

> **STATUS: FIXED.** Three changes: a shared `AtomicUsize` caps live trackers at
> `MAX_CONCURRENT_DEPOSIT_TRACKERS = 256` (well above the 100-Session profile), released
> by an RAII `DepositTrackerSlot` that also fires when the timeout drops the future; the
> polling phase is jittered by a random `0..poll_interval` offset so trackers started in
> the same instant do not align their RPC calls; and `max_deposit_tracking_time` gains a
> `#[serde(default)]` of 60 s, matching the Exit's `max_deposit_wait`. The cross-crate
> bound on `max_deposit_wait` cannot be validated here and is documented on the field
> instead.

`non_anonymous_pix.rs:207-272`: every `DepositAddressReceived` spawns a detached
task polling `chain_api().balance()` at `max(max_tracking_time/10, 1 s)`. At 100
sessions that is ≥100 RPC calls/s with no shared cap and no jitter. There is also
no validation that the strategy's `max_deposit_tracking_time` is ≤ the Exit's
`max_deposit_wait` (60 s default) — if it is larger, the awaiter times out first
and the session is killed anyway; if much smaller, tracking gives up early.
`max_deposit_tracking_time` has no `#[serde(default)]`, unlike its neighbours.

### M7. Dedup caches are capacity-only, so LRU eviction can re-enable a duplicate withdrawal

> **STATUS: FIXED.** All three caches gained a TTL, and the misleading comment was
> replaced: `processed_deposits` is 8192 entries / 24 h (≈4 TB of traffic at the default
> quota), the two in-flight guards 1024 entries / 10 min. A duplicate sweep let through
> by guard expiry is harmless — `sweep_recovered` re-reads the on-chain balance and
> no-ops at zero.

`processed_deposits`, `in_flight_sweeps`, `in_flight_destinations` are all
`Cache::builder().max_capacity(1024).build()` with no TTL
(`non_anonymous_pix.rs:110-112`). The comment on `processed_deposits` argues that
omitting a TTL avoids "an expiration window that could allow duplicate
withdrawals" — but LRU eviction at 1024 entries reintroduces exactly that window.
On a long-lived Entry (>1024 SSA cycles ≈ 500 GB) a redelivered
`NewDepositAddress` would trigger a second withdrawal.

### M8. Commitment traffic competes with SURB delivery on the forward path

~19 000 Start packets per SSA cycle per session must share the forward path with
the SURB-bearing keep-alives that fund the return path. During each pipelined
window (the last 15 % of a cycle, ~118 000 return packets) commitments need a
material fraction of forward capacity, directly reducing achievable return
throughput. Worth measuring; there is no prioritisation between `SsaCommit` and
SURB-carrying keep-alives.

### M9. Share verification uses a naive `Σ Cᵢ·xⁱ` with per-share rayon

> **STATUS: FIXED — by removing Feldman entirely rather than batching it.** The Entry now commits
> only to each polynomial's constant term; the Exit interpolates `threshold` shares and checks the
> recovered `a₀` against that one commitment. Per-share verification is gone, and with it M14's
> ingest cost, since `polys × threshold` commitments become `polys`.
>
> **The wire format is unchanged.** `SsaClientCommitmentMessage` keeps `coefficient_index`,
> `commitment_proof` and `coefficient_commitments`; encode/decode are untouched. Only the
> constant-term pass is ever emitted, and an Exit that receives a higher coefficient index ignores
> it (without decoding it), so an Entry that still sends the full Feldman matrix merely wastes its
> own bandwidth.
>
> **Measured** (same harness and machine as M10; production `8192 × 64`):
>
> | per 519 MiB cycle              | before                            | after                       |         |
> | ------------------------------ | --------------------------------- | --------------------------- | ------- |
> | commitments sent               | 524 288                           | 8 192                       | 64×     |
> | `SsaCommit` messages           | ~18 700                           | ~320                        | 58×     |
> | commitment bytes               | ~16.8 MB                          | ~262 KB                     | 64×     |
> | Exit ingest, whole cycle (M14) | ≈81 s                             | **1.249 s**                 | 64×     |
> | Exit share verification        | ≈210 s, 3.4 × 10⁷ EC mults        | **≈0.51 s**, 8 192 EC mults | 411×    |
> | **Sustained Exit rate**        | **2.50 MiB/s**                    | **92.8 MiB/s**              | **37×** |
> | Entry `new_ssa_commitment`     | 3.43 s                            | **62.7 ms**                 | 54.7×   |
> | Exit commitment memory         | 48.0 MiB + 8 192 inner `HashMap`s | ~0.75 MiB, flat map         | 64×     |
>
> `decode_commitment` still measures **151.5 µs** and `next_share` is unchanged (7.40 → 7.48 ms,
> inside run-to-run variance) — nothing got faster per unit of work, there is simply far less of
> it. `SsaPartCommitment::verify_reconstructed` is 62.2 µs, paid once per polynomial instead of
> 487 µs per share. `acknowledge_shares/full_ssa/p4_t64` is −97.7 %.
>
> **The 100-Session target is no longer CPU-bound by PIX.** At 1.5 Mbps each, 100 Sessions demand
> 18.75 MiB/s in aggregate against a measured 92.8 MiB/s per Session; the binding constraint moves
> off share verification entirely.
>
> **New finding, surfaced by the fix: `use_batch_verification` now defaults to `false`.**
> It measured 2.46 vs 2.50 MiB/s before — indistinguishable, because the MSM dominated. With the
> MSM gone it costs **46 %**: 50.2 MiB/s batched against 92.8 MiB/s unbatched. **Flipped.** The
> flag stays configurable rather than being removed, because the figures come from the sequential
> `sustained_quota_rate` group and the Exit runs acks through a concurrent pipeline, where
> amortising the batch setup across more callers may yet pay — `concurrent_quota_rate` both ways
> is what would settle that, and would justify flipping it back.
>
> **Why per-share verification was safe to drop.** PIX has exactly one shareholder. The Exit holds
> every share, reconstructs locally, is the whole quorum, and consumes only the recovered `a₀`, so
> `a₀·G == C₀` is deterministic and exact for the property actually relied on. What the
> per-coefficient commitments bought was _fault isolation_ — rejecting one bad share so a surplus
> share could refill its slot — and that is what is given up.
>
> **What it costs.** Detection moves from the 1st share of a polynomial to its `threshold`-th
> (64th), and one corrupt share now kills the cycle instead of being absorbed. Both are acceptable
> because a corrupt share implies a dishonest or broken Entry: the share travels inside a
> Sphinx-authenticated SURB (`crypto/packet/src/packet.rs:466-481`) and is decrypted with the
> acknowledgement key its own challenge fixes (`reconstructor/mod.rs:306-333`), so there is no
> benign path to one — and such an Entry has already funded the deposit it thereby forfeits.
> `MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES` is therefore now **0**: a failed polynomial already dooms
> the cycle (`SsaBuilder` needs every part), so closing on the first failure caps the exposure at
> `threshold` packets — 64 out of a 524 288-packet cycle, 0.012 %. `surplus_shares` still absorbs
> _lost_ shares, since reconstruction starts at the first `threshold` distinct ones to arrive.
>
> **Knock-on: the polys/threshold split should be re-tuned.** `transport/session/src/types.rs`
> used to argue for keeping `threshold` small because verification was `O(threshold)` per share.
> That is gone. Commitment wire, ingest and memory are now linear in `polys` alone; the only cost
> still growing with `threshold` is interpolation (`Q × threshold` field operations), plus
> detection latency. The doc has been corrected; the 8192 × 64 split is retained pending
> measurement of where the new optimum sits. The product is what fixes the quota, so the split can
> move without touching session negotiation.
>
> The original analysis follows.

> **Now measured (M10).** This is the single largest Exit cost and the main reason a PIX
> Session is CPU-bound at **≈2.5 MiB/s**.

`lib.rs:274-317`: `verify_completed_share` performs ~`threshold` variable-base scalar
multiplications per share. At the deployed `8192 × 64` that is
`polys × threshold × threshold` = 8192 × 64 × 64 ≈ **3.4 × 10⁷ EC scalar mults** per 519 MiB
cycle.

Measured: **400 µs per share** end-to-end through `acknowledge_shares`, **487 µs** for
`verify` alone, giving ≈**210 s per cycle** and a sustained rate of **2.50 MiB/s**. One SURB
carries one share, so this is one 400 µs verification per return packet.

Two things the numbers say that the original note only guessed at:

- **`into_par_iter()` is at the wrong granularity.** `verify` costs 205 µs at `t = 8` and
  487 µs at `t = 64` — a ≈150 µs floor (`msg_to_scalar` plus rayon dispatch) that does not
  shrink with the work. Parallelising _across_ shares instead of within one share's MSM is
  the change to make.
- **`use_batch_verification` is not a lever here.** At batch 256 it measures 2.46 vs
  2.50 MiB/s — indistinguishable. It only batches the acknowledgement _signature_ check,
  which is noise next to the MSM.

A random-linear-combination batch check over the `threshold` shares of one polynomial remains
the real fix, and should cut this by ≈`threshold`×.

**What per-share verification is and is not protecting** (established while fixing C3, and it bears
on how aggressively this can be changed):

- PIX has exactly **one shareholder**. Classic VSS needs per-share verification because shares sit
  with mutually distrusting parties who reconstruct later; here the Exit holds every share,
  reconstructs locally, is the whole quorum, and consumes only the recovered `a₀`. So checking
  `G·a₀ == C₀` once per polynomial — one scalar multiplication against `threshold²` — is
  deterministic and exact for the property actually relied on. The threshold/incentive property is
  enforced by Shamir secrecy and the _arity_ of the interpolation, not by the commitments.
- What per-share verification does buy is **fault isolation**: rejecting an individual bad share so
  `surplus_shares` can absorb it. That is real — a failing share is rejected _before_
  `self.shares.push`, its slot stays open, a surplus share refills it, and the Session survives to
  the 4th `UnverifiableShare` event (`MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES` = 3). So corrupt shares
  are tolerated today, and any change here must reproduce that rather than write it off. _(The fix
  did not reproduce it: it was traced instead to only occur with a dishonest or broken Entry, and
  deliberately given up. See the status block above.)_
- What it does **not** buy is protection of the deposit: C3 was not defended by it at all. So the
  non-constant coefficient commitments (16.8 MB of wire, 81 s of `decode_commitment`, 48.0 MiB per
  cycle) are paying for fault isolation only, which is worth weighing once M9 lands. _(Weighed, and
  they were dropped.)_

Note also that `add_share` — verification included — runs inside the per-polynomial
`Arc<Mutex<SsaPartBuilder>>`, and the Entry emits polynomial-major, so one Session's shares
**serialise** at ~487 µs each: a per-Session ceiling of ~2050 shares/s (~17 Mbps). Above the
1.5 Mbps cap, but a latency floor no amount of parallelism removes.

### M14. Commitment ingest costs 152 µs per commitment — 81 s per cycle on the Exit

> **STATUS: CLOSED / NO SEPARATE BATCHING FIX.** M9 removed 63/64 of the work, taking
> the cycle from ≈81 s to ≈1.25 s. The proposed random-linear-combination remainder was flawed;
> see the correction below. The per-commitment cost is unchanged — this is
> still `decode_commitment`, still dominated by decompression plus the cofactor-8 subgroup check —
> but there are now `polys` commitments per cycle instead of `polys × threshold`, so the per-cycle
> figure drops from ≈81 s to ≈1.25 s. Item 2 below (the Entry's untabulated fixed-base
> multiplications) is fixed outright: 524 288 of them become 8 192.
>
> **Correction: there is no sound one-shot batch subgroup check here.** Write each point as
> `Pᵢ = Qᵢ + Tᵢ`, with `Tᵢ` in BabyJubJub's cofactor-8 torsion subgroup. A random linear
> combination accepts exactly when `Σ aᵢTᵢ = 0`; because the torsion group is tiny, cancellation is
> not a negligible `1/q` event. In the worst case of an order-2 component it succeeds with
> probability **1/2 per check**. Repeating enough independent combinations for cryptographic
> soundness removes the intended batching win.
>
> More importantly, `babyjubjub_ec::ProjectivePoint::from_bytes` already calls the backend decoder
> with `Validate::Yes`, whose documented contract includes prime-order-subgroup validation. The
> following explicit `is_torsion_free` is therefore defence in depth, not the acting check. The
> honest options are to keep that redundancy and its small post-M9 cost, or deliberately rely on
> the validated decoder after benchmarking/removing the second check. Replacing it with probabilistic
> batching is not recommended and is no longer an open review action.
>
> **Re-checked at `4f30a70629`: unchanged.** `SsaPartCommitment::decode_commitment`
> (`protocols/pix/src/lib.rs:321-325`) is still one `from_bytes` plus one
> `is_torsion_free` per point, with no batching:
>
> ```rust
> Option::<PixGroup<S>>::from(PixGroup::<S>::from_bytes(commitment))
>     .filter(|pt| bool::from(pt.is_torsion_free()))
>     .ok_or(errors::PixError::InvalidInput)
> ```
>
> Note the interaction with **M13**: the backend's own `from_bytes` already rejects a
> non-prime-order point, so `is_torsion_free` is defence in depth rather than the acting
> filter. The earlier suggestion to preserve that defence with a random-linear-combination
> batch is withdrawn for the small-torsion cancellation reason above. At ≈1.25 s per cycle,
> retaining the redundant check is a defensible conservative choice rather than an open defect.

Found while building the M10 benchmarks. `insert_coefficient_commitments` costs **152 µs per
commitment** (measured at `p = 8192`; 156 µs at `p = 512`, so it is width-independent). At
`polys × threshold = 524 288` commitments per cycle that is **≈81 s of Exit CPU per cycle**,
on top of M9's 210 s — together the ≈1.8 MiB/s per-Session ceiling.

The cost is **entirely** `decode_commitment` (`lib.rs:218-222`), measured in isolation at
**151.7 µs** against the 152 µs of the enclosing insert — the map insertion and accumulation
are noise. On the current backend it does a validated decompression from `PixGroupRepr`
(including subgroup validation) and then an explicit `is_torsion_free()` defence-in-depth check.
Subgroup validation is mandatory because BabyJubJub has cofactor 8 (see M13); performing it twice
is not.
Verifier construction is not implicated: the constant-term-only pass, which builds no
verifiers, shows the same per-commitment figure. And it is already called exactly once per
commitment (that was H4), so this is not redundant work.

151.7 µs is well above what one modular square root plus one scalar multiplication should
cost on this curve, so the first step is to split the two halves and find out which dominates
— the micro-bench added here is the harness for that.

Originally considered, in rough order of expected payoff:

1. ~~**Batch the subgroup check.**~~ **Rejected:** torsion components can cancel with probability
   as high as 1/2 per random combination; this is not a prime-field polynomial identity test.
   The decoder already performs the subgroup check itself.
2. **Entry side, same shape:** `new_polynomial_with_verifier` (`generator.rs:62-69`) computes
   `g * c` for every coefficient — 524 288 **fixed-base** multiplications against the group
   generator, with no precomputation table. Measured at 3.43 s per `new_ssa_commitment`
   (6.5 µs/commitment). A windowed/comb table is the textbook fix and is typically 3–6×.

The Entry figure matters less per packet (3.43 s amortises to ≈5 µs per share) but it is a
3.4 s stall on one `spawn_blocking` slot at every cycle boundary, which is the latency the
Exit's recovery gate is racing.

### M15. The supervisor's quota/recovery-deadline consistency check is skipped for programmatic configs

> **NEW; only on `lukas/session-pix-supervisor` at `6d671409ea`.** The serialized
> `HoprProtocolConfig` path is safe. Its `validate_incoming_session_pix_config`
> computes the packet count at `quota_range.end()`, divides by the documented
> per-Session rate, and rejects a `max_recovery_time` that cannot cover one cycle.

The public programmatic path does not run that check:

- `SessionManager::new` clamps zero/oversized batch values and oversized durations, and clamps
  `max_recovery_idle` below the default reconstructor lifetime, but it does not enforce a minimum
  `max_recovery_time` against `pix_config.quota_range`;
- `SessionManager::start` calls `validate_pix_supervision`, which checks non-zero durations and
  cross-lifetime invariants against the reconstructor actually installed, but has no quota or rate
  input and therefore cannot perform the missing comparison.

Consequently a programmatic `SessionManagerConfig` with the default quota and, for example,
`max_recovery_time = 1 s` survives both steps and deterministically closes an honest first cycle
with `RecoveryDeadline`. This is the same invalid combination the config-file path explicitly
rejects.

**Fix:** put the minimum-duration check on the path every `SessionManagerConfig` takes (preferably a
fallible constructor/start validation), with the assumed minimum supported packet rate made an
explicit input rather than known only to `transport/hopr/src/config.rs`. Add a regression that
constructs the manager directly, since the existing config tests exercise only the serialized
outer type.

### M16. The PIX curve override is wire-visible but is neither negotiated nor versioned

> **STATUS: FIXED.** The curve is now announced, and the Exit is decisive: `PixSuite`
> (`BabyJubJub = 0`, `Secp256k1 = 1`) is a fourth component of `PixParams`, carried in the two bits
> of the packed word that `MAX_POLYS_PER_SSA = 16192` has always left free, and `check_pix_params`
> refuses an Entry whose suite is not this build's with `UnacceptablePixParams`. Nothing is
> negotiated — there is nothing to negotiate, since the suite is fixed at build time on both sides —
> so the outcomes are "same curve" and "refused".
>
> The placement is what makes it work: `StartInitiation` is fixed-size, and the first curve-sized
> bytes in the exchange are the Exit's own commitments in `SsaRequest`, which it only sends after
> accepting. The check therefore runs before either side reads a field whose width it might have
> wrong.
>
> `START_PROTOCOL_VERSION` stays `0x03`. Compatibility is clean for pre-suite BabyJubJub builds: a
> word packed before the field existed sets neither bit and still reads as BabyJubJub. In the other
> direction, a new `Secp256k1` word presents a polynomial count of ≥ 16 384 to a peer that predates
> the field — above the maximum it already enforced, so it refuses rather than mis-parses. A `const`
> assertion fails the build if `MAX_POLYS_PER_SSA` is ever raised into the suite bits.
>
> **One residual, and it is unavoidable:** a node built _before_ this field but _for_ secp256k1
> announces zeros and is therefore indistinguishable from BabyJubJub, so it still fails later on the
> curve-sized fields. New against new, and new against old-BabyJubJub, are both clean. Recorded in
> the feature documentation rather than worked around.
>
> Also done: the network-wide invariant is stated on all three feature definitions
> (`crypto/packet`, `transport/hopr`, `hopr-lib`); `PixParams::try_from_config::<S>` and
> `try_new_for::<S>` take the suite from the spec that will generate the shares, so it cannot be
> stated wrongly; `LOCAL_PIX_SUITE` is re-exported up to `hopr-lib` for callers restating params by
> hand. Pinned by `check_pix_params_must_refuse_a_foreign_curve_suite`, the
> `protocols/pix/tests/curve_suite.rs` fixture (the only target where both curves exist at once), and
> `pix_wire_element_sizes_are_fixed_by_the_curve_feature`, which pins 32/64 B on BabyJubJub and
> 33/65 B on secp256k1 so a width change is a failing test rather than a silent wire break.

> **NEW at fetched `ddadbc86ac`.** The positive `pix-secp256k1` override fixes the Cargo-feature
> unification problem it targets: it wins even when another dependency enables the default
> `pix-bjj`. All three feature combinations compile and select exactly one implementation.
>
> The choice is not local to deposit settlement, however. It changes
> `HoprPixGroupRepr` from 32 bytes (BabyJubJub) to 33 bytes (secp256k1), and consequently changes
> `SsaCommitmentProof` from 64 to 65 bytes. `StartProtocol` derives its `SsaCommit` and
> `SsaRequest` layouts and chunking from those local type sizes, but still advertises protocol
> version `0x03`; neither `StartInitiation` nor `PixParams` carries a curve/suite identifier.
>
> Therefore a default node and a node built with the advertised override interpret the same
> versioned messages with different element boundaries and cannot establish a PIX cycle. The
> failure surfaces as malformed Start traffic rather than a clear "unsupported PIX suite"
> rejection. This was possible with the old opt-out too, but making the override reliable and
> explicitly recommending it to downstream settlement consumers makes the compatibility boundary
> actionable rather than theoretical.
>
> **Fix/policy decision:** if curve choice is a network-wide build invariant, state that explicitly
> on all three feature definitions and in operator/release documentation; a downstream consumer
> cannot select it independently. If mixed suites are meant to interoperate, add a suite ID to PIX
> negotiation (or use distinct Start protocol versions) and reject a mismatch before exchanging
> curve-sized fields. Add a cross-configuration wire fixture either way.

### M17. The polys/threshold calibration omits threshold-dependent Entry share generation

> **STATUS: FIXED, by stating the objective.** Per the author's decision the split optimises **Exit
> reconstruction capacity**, not total network CPU: an Exit is a shared resource serving 10–30
> concurrent clients while each Entry generates only its own shares, so the Exit is what saturates
> first and what sets network capacity. On that measure the deployed `8192 × 64` stands — the
> Exit-only optimum is `t = √(A/C) ≈ 54`, flat within 0.5 % from 48 to 64, with 64 inside 0.4 %.
>
> The three comments that claimed the Entry was threshold-free are corrected —
> `DEFAULT_POLY_THRESHOLD`, `bench_acknowledge_shares_interpolation`, and
> `DEFAULT_PIX_POLYS_PER_SSA`'s cost model, whose "the only cost that grows with the threshold"
> bullet was the same error one layer up. `next_share` is now listed as an Entry cost that grows
> with the threshold, both measured tables are recorded on `DEFAULT_POLY_THRESHOLD`, and the
> total-CPU reading that favours 48 by ~3 % is recorded as measured and deliberately not acted on
> rather than left unmentioned. `bench_next_share` gained the figures and the note that
> `--features all-benchmarks` is what makes it a sweep at all — without it `THRESHOLDS` is a single
> point, which is how these numbers came to be missing.

> **NEW at `01466b416e`.** The calibration calls the Entry side effectively threshold-free from
> `new_ssa_commitment` alone. That is only commitment construction. Every served packet also calls
> `SsaShareGenerator::next_share`, whose `IndexedPolynomial::next_share` evaluates a polynomial with
> `self.raw.evaluate(&x.into(), self.t)`. The existing benchmark already sweeps this operation, but
> its `all-benchmarks` results were not run or included in the conclusion.
>
> Running that group at the current tip on the same machine gives:
>
> | threshold | `next_share` µs/share |
> | --------- | --------------------: |
> | 16        |                  0.90 |
> | 32        |                  1.20 |
> | 48        |                  1.51 |
> | 64        |                  1.82 |
>
> Thus the Entry is not threshold-free. As a simple total-CPU model, adding these figures to the
> published Exit figures gives 14.05, 12.32, 12.13 and 12.50 µs/share respectively: threshold 48,
> not 64, is the best sampled point, and 64 is about 3 % above it rather than 0.4 %. This does not
> by itself require changing the default: if the objective is strictly Exit bottleneck capacity,
> the Exit-only result still supports 64, and Entry work is much smaller. But the review must state
> that objective; it cannot simultaneously claim that both sides are measured or threshold-free.
>
> **Fix:** decide whether the split optimises Exit bottleneck capacity or total network CPU. For
> the former, retain 64 but qualify the conclusion and correct the false "Entry threshold-free"
> comments in `DEFAULT_POLY_THRESHOLD` and `bench_acknowledge_shares_interpolation`. For the latter,
> include `next_share` and the per-cycle commitment cost in one cost model, then re-evaluate 48
> versus 64.

### M10. PIX was unmeasured: benchmarks ran below production dimensions, and no benchmark exercised PIX at all

> **STATUS: FIXED (benchmarks).** The dimension gap was real but turned out to be the
> smaller half of the problem. Numbers below; the two optimisation leads they expose are
> tracked as M9 and M14 and are deliberately _not_ implemented in the same pass.

Two separate defects.

**(a) No benchmark measured PIX-active behaviour.** Five bench files construct an
`SsaShareGenerator`; three never called `new_ssa_commitment`, so `next_share` took the
early return at `generator.rs:148` (`polynomials.get(pseudonym)` → `None` → `Ok(None)`) on
every iteration and no share was ever embedded into a SURB:

| file                                                           | sites                                         |
| -------------------------------------------------------------- | --------------------------------------------- |
| `crypto/packet/benches/packet_bench.rs`                        | `:90`, `:137`, `:192`, `:235` (all 4 benches) |
| `transport/hopr/benches/protocol_throughput_emulated_bench.rs` | `:115`                                        |
| `transport/hopr/benches/pipeline_e2e_bench.rs`                 | `:135`, `:246`                                |

Every published packet-path and protocol-throughput figure was therefore PIX-off, and the
"PIX Session vs non-PIX Session" comparison had never been measured. The transport bench
additionally passed `return_paths: vec![]`, so it built no SURBs at all — enabling PIX there
required giving the packets a return path first.

**(b) Reconstructor dimensions.** `ssa_reconstructor_bench.rs` used
`thresholds = [10, 50]`, `polynomials_per_ssa = [128, 512]`, and
`SINGLE_POLY_BENCH_POLYS = 4` for every `acknowledge_shares` case, against a production
`8192 × 64` (`transport/session/src/types.rs:119,125`). At 4 polynomials no cache-occupancy
effect is observable, which is why H1 and H3 were invisible to the suite.

#### Measured (release, rayon on, BabyJubJub, `t = 64`)

Entry side:

| operation                                    | measured    | per unit            |
| -------------------------------------------- | ----------- | ------------------- |
| `new_ssa_commitment` (`p = 8192`)            | 3.43 s      | 6.5 µs / commitment |
| `next_share` (per SURB, steady state)        | **1.47 µs** | —                   |
| `next_share` with no committed SSA (PIX off) | 49 ns       | —                   |

Exit side:

| operation                                                     | measured         | per unit                |
| ------------------------------------------------------------- | ---------------- | ----------------------- |
| `new_exit_commitment`                                         | 74.4 µs          | O(1)                    |
| `decode_commitment`                                           | **151.7 µs**     | — (see M14)             |
| `insert_coefficient_commitments` (`p = 8192`, constant terms) | 1.246 s / 8192   | **152 µs** / commitment |
| `insert_coefficient_commitments` (`p = 512`, whole matrix)    | 5.105 s / 32 768 | **156 µs** / commitment |
| `insert_encrypted_share`                                      | 807 ns           | —                       |
| `acknowledge_shares` (batch 256)                              | 102 ms / 256     | **400 µs** / share      |
| `acknowledge_shares` when the ack is deferred (H1 path)       | 241 µs / 10      | **24 µs** / ack         |
| deferred-bucket drain at verifier installation                | +0.42 s / 1024   | 409 µs / ack            |
| `SsaShareVerifier::verify`                                    | 487–519 µs       | —                       |

Per-commitment cost is flat across matrix width (152 µs at 8192 vs 156 µs at 512), so these
scale linearly. For one production cycle (524 288 commitments, ≈519 MiB of quota):

- commitment insertion ≈ **81 s**
- share acknowledgement ≈ **210 s**
- total ≈ **291 s** → **≈1.8 MiB/s per Session**, CPU-bound

The `sustained_quota_rate` group reports the acknowledgement half directly: **2.50 MiB/s**.
For comparison the SURB balancer's own default is `max_surbs_per_sec = 5000` → 2500 pkt/s
≈ 2.5 MiB/s, so PIX share processing is at or below the rate the balancer is already tuned
to deliver. One SURB carries exactly one share
(`crypto/packet/src/packet.rs:149-152`), so this is one 400 µs verification **per return
packet**.

Two secondary observations:

- `use_batch_verification` is indistinguishable at batch 256 (2.50 vs 2.46 MiB/s). It only
  batches the acknowledgement _signature_ check, which is noise next to the MSM — so it is
  not a lever on this number.
- The H1 deferral path is cheap (24 µs vs 400 µs for a verified ack, 17×), and the drain that
  redeems a deferred ack at verifier-installation time costs exactly what a normal
  acknowledgement costs (409 µs vs 400 µs) — i.e. the work is moved, not duplicated. Both
  behave as the H1 fix intended.

#### The PIX overhead of a Session, measured (`packet_bench`, 3 hops, 2 SURBs)

| path                               | PIX off   | PIX on    | delta                 |
| ---------------------------------- | --------- | --------- | --------------------- |
| `packet_sending_no_precomputation` | 304.77 µs | 315.90 µs | **+11.1 µs (+3.6 %)** |
| `packet_precompute`                | 289.03 µs | 295.22 µs | **+6.2 µs (+2.1 %)**  |
| `packet_forwarding` (relay)        | 66.91 µs  | n/a       | PIX not involved      |
| `packet_receiving` (destination)   | 49.55 µs  | n/a       | PIX not involved      |

So the answer to "does PIX cost a Session too much" is **asymmetric, and the Entry is fine**:

- **Entry: +2–4 % per packet.** Two SURBs cost two `next_share` calls plus two share
  encryptions. The once-per-cycle `new_ssa_commitment` amortises to ≈5 µs per share, though it
  is a 3.4 s stall on one `spawn_blocking` slot at each cycle boundary.
- **Exit: ≈8× the cost of the packet itself.** 400 µs of share verification against 49.55 µs
  to receive and decode the packet it arrived on. This is where PIX changes the performance
  class of a Session, and it is entirely M9 plus M14.

The same A/B through the full sending pipeline
(`protocol_throughput_emulated_bench`, 20.3 MiB, 3 peers, 1 SURB per packet) agrees:
**97.5 MiB/s** with PIX off against **87.8 MiB/s** with PIX on, ≈10 % lower. Treat that as
approximate — the `pix_on` confidence interval spans 78.7–94.1 MiB/s. The pipeline is
concurrent, so `packet_bench`'s +11 µs of single-threaded CPU shows up as ≈1 µs of wall clock
per packet here; the single-threaded figure is the one to reason about for capacity.

Caveat on that bench: it reuses the forward path as the return path, because
`MockPathResolver` resolves against a one-directional `CHANNELS` chain and a topologically
correct multi-hop return path is not constructible. Path _length_ is what determines whether a
share is embedded, and no SURB is ever redeemed there, so this does not affect the measured
delta — but it is not a realistic topology.

#### What was added

- `all-benchmarks` feature on `protocols/pix`, matching the six other crates that have it, so
  full-width points are opt-in.
- Reconstructor bench rebuilt at production `threshold`: commitment insertion driven in the
  **Tier 4 wire order** (constant-term pass, then polynomial blocks, replicating
  `SsaClientCommitmentMessage::new_multiple`), realistic acknowledgement batch sizes
  (`MAX_ACKNOWLEDGEMENTS_BATCH_SIZE`-shaped, not `threshold`-shaped), a sustained-rate group
  reported in MiB/s of Session quota, and first-ever coverage of the H1 deferral path
  (`defer_ack` / `drain_deferred_acks`).
- `pix_off` / `pix_on` A/B in `packet_bench.rs` and `protocol_throughput_emulated_bench.rs`,
  both with an assertion that the `pix_on` share budget was not exhausted mid-run — without
  it, exhaustion silently turns `pix_on` into `pix_off` and PIX looks free.
- `bench_next_share` no longer builds a fresh generator and commitment _inside_ the timed
  loop. That was not only slow (~110 commitments at ~1 s each per parameter point, ~20 min for
  the group) but **wrong**: every timed call was the first `next_share` against a cold moka
  entry, so the bench reported 24.1 µs where the steady-state cost is **1.47 µs** — a 16×
  overstatement of the Entry's per-SURB cost. The in-situ `packet_bench` delta only makes
  sense against the corrected figure.

Groups that time a whole matrix, and the acknowledgement groups, default to a narrower
`polys` than production and are documented as such: installing 524 288 commitments takes
81 s, so a production-width fixture is over a minute of untimed setup per benchmark id. The
per-unit figures are width-independent (measured, above), and production width is available
under `all-benchmarks`.

#### Capacity at the deployed operating point (100 Sessions × 1.5 Mbps, 48-core host)

Per Session: 181 shares/s and a 48.4-minute cycle, so 100 Sessions means a new SSA cycle every
29 s and a steady **18 060 shares/s and 18 060 commitments/s** (the same figure — the quota is
one packet per commitment).

Single-threaded cost, measured with `--no-default-features` (rayon off): `verify` at `t = 64` is
**4.62 ms**, against ~520 µs with rayon on and one caller — an 8.9× speedup on 48 cores, i.e.
**18 % parallel efficiency**.

Concurrency scaling of `acknowledge_shares` (`concurrent_quota_rate`, aggregate quota rate):

| concurrent callers | throughput |                                 |
| ------------------ | ---------- | ------------------------------- |
| 1                  | 2.46 MiB/s |                                 |
| 10                 | 5.96 MiB/s | `DEFAULT_ACK_INPUT_CONCURRENCY` |
| 48                 | 6.26 MiB/s | saturated                       |

So the machine ceiling is **6.26 MiB/s = 6321 shares/s → 35 Sessions**, against the 18 060
shares/s that 100 Sessions demand — **2.9× short**. Two consequences: raising
`ack_input_concurrency` is _not_ a lever (10 → 48 buys 5 %), and the ceiling is only 61 % of the
naive ideal (48 cores ÷ 4.62 ms = 10 390 shares/s), the gap being rayon contention as each
caller spawns 63 tasks onto the shared pool.

Even at perfect parallel efficiency the work does not fit:

|                     | per unit | at 18 060/s                                      |
| ------------------- | -------- | ------------------------------------------------ |
| share verification  | 4.62 ms  | **83.4 cores**                                   |
| `decode_commitment` | 151.7 µs | 2.74 cores                                       |
|                     |          | **86 cores needed vs 48 available — 1.8× short** |

Share verification is **97 %** of it. M14's commitment ingest is 5.7 % of the machine at 100
Sessions: real, but not what blocks the target.

Batched per-polynomial verification (M9) takes 4.62 ms to roughly 0.5 ms, i.e. **8.6 cores at
100 Sessions — 5.6× headroom**. It is both necessary and sufficient; fixing rayon granularity
alone moves 35 → ~57 Sessions.

#### Memory (`protocols/pix/tests/memory_profile.rs`, tracking allocator, production width)

`PixGroup` is 96 B decoded, so the matrix is 48.0 MiB of raw points.

| phase                                   | live state        |
| --------------------------------------- | ----------------- |
| after `new_exit_commitment`             | +0.2 MiB          |
| after constant-term pass                | +4.6 MiB          |
| **after full matrix (all verifiers)**   | **+53.4 MiB**     |
| after 25 % / 50 % of the cycle's shares | +36.4 / +15.6 MiB |
| peak, production-shaped ack batches     | **57.2 MiB**      |

Only ~11 % above the raw point size, and it decays to zero as polynomials reconstruct — Tier 1
works. At 100 Sessions: **2.61 GiB** uniformly staggered, **5.59 GiB** if cycles synchronise,
which they do after an Exit restart when every Session re-establishes at once.

**Memory is not the constraint.** CPU is, by 3×, and it is one algorithm.

Two measurement caveats recorded in the test: past the cycle midpoint the readings go negative
against the baseline, because the Entry-side generator frees its polynomial queue in the same
process — the install figure and the endpoint are the clean ones. And acknowledgements must be
fed in production-shaped batches; driving a quarter-cycle through one `acknowledge_shares` call
allocates ~250 MiB of transient intermediates and the high-water mark then measures the harness.

### M11. `cargo nextest run --lib -p hopr-lib` does not compile on this branch

> **STATUS: FIXED.** Verified at `4f30a70629`: `cargo nextest run --lib -p hopr-lib
--no-run` compiles cleanly. `hopr/hopr-lib/src/lib.rs:28` is back to
> `#[cfg(feature = "testing")]`, so the module is no longer compiled under bare
> `cfg(test)` without the dependencies the feature pulls in. The first command in
> `CLAUDE.md` works again, and so does `cargo check --workspace --all-targets`.
>
> Of the two options this entry offered — add the crates as dev-dependencies, or revert
> the `cfg` to feature-only — the second was taken. That is the right one given how the
> unresolved set had grown (`hopr-transport-p2p` no longer exists as a crate at all), and
> it matches what `hopr-transport` does at `transport/hopr/src/lib.rs:27`.
>
> **`hopr-transport-session` went the other way, deliberately, and that is also correct.**
> CodeRabbit asked for the same gating there and warned against copying hopr-lib's
> then-broken shape. `transport/session/src/lib.rs:29` now reads
> `#[cfg(any(test, feature = "testing"))]` with `testing = ["dep:mockall", "dep:tokio"]`
> **and a self dev-dependency** carrying the feature — which is precisely the piece
> hopr-lib was missing. A release build of the crate no longer links `mockall`, and its
> runtime-agnostic boundary is restored — and it was done together with M11, which is what
> CodeRabbit advised, so hopr-lib's mistake was not copied a second time.
>
> _Superseded status:_ still open at `c4f22c55b7` with 21 errors, up from 8.

Commit `c3d869d52b` changed `hopr/hopr-lib/src/lib.rs:27` from
`#[cfg(feature = "testing")]` to `#[cfg(any(feature = "testing", test))]`, but the
`testing` module's dependencies (`hex-literal`, `tokio-util`, `hopr-transport-p2p`,
`hopr-network-graph`, `hopr-ticket-manager`, …) are still only pulled in by the
`testing` **feature**. Under `cfg(test)` the module is compiled without them:

```
$ cargo nextest run --lib -p hopr-lib --no-run
error[E0432]: unresolved import `hopr_network_graph`
error[E0433]: cannot find module or crate `tokio_util`
error[E0432]: unresolved import `hopr_transport_p2p`
error[E0432]: unresolved import `hex_literal`
… (8 errors)
```

This is the first command in `CLAUDE.md`'s test instructions, and it also breaks
`cargo check --workspace --all-targets`. Passing `--features testing` works.
Fix: add those crates as `dev-dependencies` as well, or revert the `cfg` to
feature-only. _(Found while verifying the C1 fix; unrelated to PIX itself.)_

### M12. Pure Exit nodes process PIX acks strictly sequentially

> **STATUS: FIXED.** `start_exit_incoming_ack_pipeline` now takes a `concurrency`
> parameter and uses `for_each_concurrent`, wired to the same `ack_input_concurrency`
> config value the relay pipeline already used.

`start_exit_incoming_ack_pipeline` uses `.for_each(…)`
(`transport/hopr/src/protocol/pipeline/mod.rs:597-635`) while the relay variant
uses `.for_each_concurrent(concurrency, …)` (`:686-687`). On a dedicated Exit,
every ack batch — each of which can trigger up to `threshold` share verifications
— is awaited one at a time.

### M13. Non-torsion-free commitment permanently poisons a cell

> **STATUS: FIXED — and this was _not_ "bjj feature only": `bjj` is a default
> feature, so BabyJubJub (cofactor 8) is the production curve and this was live.**
> The arrival-time validation in `add_transposed` now calls the same
> `PartialSsaShareVerifier::decode_commitment` helper that the verifier-build path
> uses, checking decodability _and_ prime-order-subgroup membership. A decodable
> small-order point can therefore no longer occupy a cell and then make completion
> fail unconditionally with no way to retransmit a correction.
>
> **Correction (2026-08-02) — "pinned by `decode_commitment_is_the_single_validation_point`" was
> wrong**, and CodeRabbit was right to say so. That test feeds the generator and an all-`0xFF`
> buffer; the second fails at `from_bytes` and never reaches `is_torsion_free`. It could not do
> better: its `TestSpec` is secp256k1, cofactor 1, so no small-order point exists to feed it.
>
> Chasing that produced a further finding. The subgroup case now lives in
> `pix_group_element_rejects_a_small_order_point` (`hopr-crypto-packet`, `bjj`-gated), which builds
> the order-2 point (0, −1) — and it turns out the backend's own `GroupEncoding::from_bytes` already
> refuses a non-prime-order point, so the encoding never reaches `is_torsion_free` there either. The
> filter is therefore **unpinnable by any test**: it is defence in depth against a backend or curve
> change that stops checking, not the acting rejection. It stays, and both tests now say why. By the
> same token the original M13 hole was narrower than described — the "decodable small-order point"
> it feared is not decodable on this backend — though the asymmetry between the two validation paths
> was still worth removing.

The insert-time validation checks `from_bytes` but **not** `is_torsion_free`
(`utils.rs:212`), while `from_serializable_commitments` checks both
(`types.rs:220-229`). A decodable small-order point therefore occupies the cell,
and only fails at `Completed` — after which re-insertion returns
`DuplicateCommitment`, so the corrected retransmission the M2 fix was designed to
allow is impossible. Harmless on secp256k1 (cofactor 1); a real hole if the `bjj`
feature is enabled.

---

## P3 — Low / cleanup

> **Re-verified at `4f30a70629`.** The table below is the original record; locations in it
> have drifted. Current status:
>
> | #      | Status                                                                                                                               |
> | ------ | ------------------------------------------------------------------------------------------------------------------------------------ |
> | L1     | **fixed** — `or_default()` is gone with the Tier 2 / M9 rewrite                                                                      |
> | L2     | **FIXED** (`606e51ea8e`) — `reconstructor/utils.rs:569` builds a `seen` set over `validated` and rejects the repeat                  |
> | L3     | **moot** — `reconstruct_verifiers` was removed by M9                                                                                 |
> | L4     | **fixed** — the channel is built inside the first-encounter guard, so only the ~1 call per cycle that needs it pays                  |
> | L5     | **open here; fixed on supervisor** — the stacked branch replaces the awaiter/kill-switch pair with one supervisor and observer       |
> | L6     | **open here; fixed on supervisor** — `RetireSsa` aborts and removes the indexed `PixDepositObserver`                                 |
> | L7     | **fixed** — `weighted_size` is gone with the H1 rewrite                                                                              |
> | L8–L12 | **out of scope here** — all belong to the standalone `hopr-strategy` repository                                                      |
> | L13    | **fixed** — `DEFAULT_POLY_THRESHOLD` is now 64 and the session constants are aliases of it; see detail below                         |
> | L14    | **fixed** — `try_new` validates and returns; `new` delegates to it and stays panicking by contract; the array index is gone          |
> | L15    | **fixed** — event wiring with H7; the two match arms themselves with `4f30a70629`                                                    |
> | L16    | **fixed** — `plans/fix-share-removed-before-verifier.md` deleted from the branch                                                     |
> | L17    | fixed, then **moot** — crate deleted                                                                                                 |
> | L18    | **open here; fixed on supervisor** — the replacement-prone per-index kill switch no longer exists                                    |
> | L19    | **fixed** — the test asks `StartProtocol::ssa_commit_chunking`; a new start-crate test pins the arithmetic itself                    |
> | L20    | **fixed in the combined merge sequence** — the follow-up should take the base branch's configured helper, deleting the stale comment |
> | L21    | **fixed** — the operator doc states the enforced 25 600 B floor, pinned by a test so prose and validator cannot drift apart again    |
> | L22    | **fixed** — both comments now describe the override model; the two imports carry the same cfg arms the curve selection itself uses   |
> | L23    | **fixed** — ceiling division makes the documented 20 % a floor; both tests now sweep all accepted thresholds                         |
> | L26    | **fixed** — the comments describe a quadruple; where "three" is still right, the suite's exclusion from the quota is said outright   |
>
> **L18 and L19 were new** in the CodeRabbit pass when its triage document was
> folded into this one — see "The CodeRabbit pass, folded in" below.
> **L20** is from the `6d671409ea` stacked-branch follow-up. **L21** is from
> `e345470ead`; **L22** is from the fetched `ddadbc86ac`, which is no longer merely fetched — the
> six base-branch commits were rebased onto it, so `ddadbc86ac` is now this branch's base and the
> two findings are fixed against the post-rename feature names rather than against the ones they
> replace.

> **L23 detail — FIXED at `3fea835f3a`.** `default_surplus_for(threshold)` now uses
> `threshold.div_ceil(4)`, so the documented tolerance is a floor and thresholds 2 and 3 no longer
> derive zero surplus. Both protocol- and operator-config tests sweep the complete accepted range;
> the deployed default remains 16 surplus at threshold 64.
>
> **L26 detail — FIXED at `14bb5aedbe`.** Raised at `3fea835f3a`: the suite addition correctly made
> `PixParams` a quadruple, but five nearby comments still described “the same triple”, ranges on
> “all three”, comparing “the whole triple”, or said “all three travel together”. Runtime behavior
> was correct—the equality and decoding include the suite—but the documentation understated exactly
> the invariant M16 added.
>
> All five now say quadruple/all four. Two sites the finding did not list were corrected with them:
> `SsaServerCommitmentMessage::dimensions`' doc, which now says the suite rides along with the
> dimensions its callers want (the method keeps its name), and a test comment reading “all three pass
> protocol bounds”.
>
> One place keeps “three”, deliberately, and now says why: the suite is **not** a dimension and does
> not enter the quota, so the passage about what a deposit buys names the other three on purpose. A
> blanket three→four rewrite would have made that one wrong in the opposite direction.
>
> **On the standalone base branch, L5, L6 and L18 were one sitting.** All three lived in the
> `DepositAwaiter` block at `manager.rs:2942-3005`; the combined view counts them fixed because the
> follow-up deletes that block:
>
> - **L5** — the awaiter is spawned and inserted into `abort_handles` at `:2959`, _then_
>   `pix_events.try_send(DepositNeeded)` at `:3000` propagates its failure with `?`. On that path the
>   task is orphaned: nothing will ever abort it, and the session dies on the kill switch with no
>   diagnostic linking the two.
> - **L6** — `DepositAwaiter(idx)` is never `abort_one`'d. Only `PixKillSwitch(idx)` is (`:2981`), and
>   only on the success branch.
> - **L18 — `AbortableList::insert` replaces a live task without aborting it.** Filed by CodeRabbit
>   as unverified; **confirmed** at `4f30a70629` against the published source. It is
>   `IndexMap::insert` (`hopr-utilities-0.4.0/src/runtime.rs:487-489`), and `abort_one` is the only
>   method that aborts. So an `SsaRequest` retry that re-inserts `PixKillSwitch(idx)` or
>   `DepositAwaiter(idx)` at the same index leaves the first timer armed, free to close the session
>   with `UnrealizedDeposit` after the retry has succeeded. L6's leak is what makes this reachable:
>   the stale entry is still there to be displaced.
>
> The in-tree TODO at `:2955` ("generalize the awaiter into a perpetual Session task") is the shape
> that closes all three at once. #8237 deletes the machinery outright, so do not gold-plate it here —
> but the orphan and the un-aborted replacement are cheap to fix in place if #8237 slips.
>
> **L19 detail — now FIXED.** `expected_ssa_commits` (`transport/session/tests/pix.rs:75-81`)
> recomputed the production chunking formula instead of asserting a number, so a change in that
> formula moved the expectation with it and the test could not fail. It also recomputed it
> **wrongly**, with `size_of::<SsaIndex>()` (`NonZero<u32>`, 4 B) where an `SsaCommit` entry prefix
> is a `PolynomialIndex` (`u16`, 2 B) — the same confusion the wire layer had at
> `protocols/start/src/lib.rs` until `1fd6727b5d`. The test passed, which proved only that the two
> formulas agree at _these_ dimensions: the wrong per-entry size and the different fixed overhead
> happen to floor to the same commitments-per-message, and `polynomials_per_ssa = 64` then rounds to
> the same message count. Both halves of that coincidence are dimension-dependent.
> (`MAX_SSAS_PER_REQUEST` at `protocols/start/src/lib.rs:417` uses `SsaIndex` **correctly** — that
> one really is an `SsaRequest`, whose entries do carry an `SsaIndex`.)
>
> The wrongness and the duplication are gone: the encoder's derivation is now
> `StartProtocol::ssa_commit_chunking`, returning a named `SsaCommitChunking` with both phase
> bounds, and `new_multiple` and the test both call it. The tautology is not — an expectation asked
> of the encoder still moves with it. That is a deliberate trade, and the pin it gives up is
> recovered where the sizes are fixed instead: `ssa_commit_chunking_should_match_the_encode_layout`
> (`protocols/start/src/lib.rs`) states 28 and 27 as literals for a fully determined instantiation,
> so a change to the arithmetic has to be acknowledged there. The count is unchanged at these
> dimensions, which is what the review predicted and what the integration test passing before and
> after confirms.
>
> **L20 detail.** M2's stale safety comment was fixed on `lukas/pix` by removing quoted values and
> stating the invariant. The supervisor branch introduces `ssa_reconstructor_config()` and
> reintroduces the same failure mode in its new `SAFETY` comment: it says
> `max_awaiting_acks = 10_000_000`, while `SsaReconstructorConfig::default()` is 1 000 000. No
> runtime behavior is wrong, but this is direct evidence for M2's original point that a comment
> naming a constant it does not reference cannot stay synchronized. Use the value-free invariant
> wording already present on the base branch.
>
> **Combined status:** do not repair this comment independently. Rebasing the follow-up over M3
> should remove `ssa_reconstructor_config()` in favour of the base branch's configured
> `ssa_reconstructor()` helper, deleting the comment and fixing L20 as part of the merge.
>
> **L21 detail — now FIXED.** The doc states the enforced floor with its derivation (25 600 B, 64
> entries at the measured per-entry cost) and defers the reasoning to the protocol type that
> enforces it. The number is now checkable:
> `the_documented_ack_budget_floor_is_the_enforced_one` asserts 25 600 validates and 25 599 does
> not, so moving the floor on one side fails the build on the other. That is the same lesson L20
> recorded — a comment quoting a constant it does not reference cannot be checked by anything — and
> a doc comment is exactly that, so the check has to live somewhere a compiler can reach.
>
> The original finding:
> (`transport/hopr/src/config.rs:499`) says "minimum 16 MiB". The only validation is delegated to
> `SsaReconstructorConfig`, whose actual minimum is 25 600 B (64 measured entries) at
> `protocols/pix/src/reconstructor/mod.rs:110-117`. Runtime behaviour follows 25 600 B, so this is
> a documentation error, not an ineffective bound. Pick one minimum and make the text and validator
> agree; the protocol-side comment explicitly argues for the 64-entry sanity floor.
>
> **L22 detail — now FIXED**, after rebasing the six base-branch commits onto `ddadbc86ac`. Neither
> half was fixable before that: the two comments were still _accurate_ pre-rename, and the correct
> gate for the imports differs on either side of it (`feature = "bjj"` versus
> `all(feature = "pix-bjj", not(feature = "pix-secp256k1"))`), so writing either against the old
> base would have been wrong the moment the rename landed.
>
> The root manifest comment now describes the override model and records why the old reasoning was
> not merely stale but never sound — withholding a feature only holds while every crate in the graph
> agrees to it, which is the hazard `pix-secp256k1` exists to remove. The M13 comment names
> `pix-bjj` and the override that can displace it.
>
> The two `Group` imports are gated with the _same_ arms the curve selection uses, rather than a
> hand-written approximation of them: the trait supplies `mul_by_generator` on Baby JubJub, while
> secp256k1's `ProjectivePoint` has it inherently, which is why the import was unused on exactly one
> side. Verified warning-free on all three supported combinations — default, `--no-default-features
--features ed25519,rayon,pix-secp256k1`, and defaults plus `--features pix-secp256k1`.
>
> The original finding:
>
> **L22 detail.** `ddadbc86ac` renames the PIX curve features to `pix-bjj` and
> `pix-secp256k1`, but the root `Cargo.toml` comment still describes opting out of `bjj` — the
> mechanism this commit deliberately replaces — and the M13 test comment still calls `bjj` the
> default feature. More concretely, both supported secp override builds emit unused-import warnings
> for `hopr_protocol_pix::Group` at `crypto/packet/src/types.rs:521` and `:623`; the default BJJ
> build is clean. Gate or remove those imports and update the two comments. All three feature
> combinations compile successfully, so this is cleanup rather than a curve-selection defect.
>
> **L2 detail — now FIXED** in `606e51ea8e`. The transactional rewrite had preserved the
> shape of the bug: the pre-check tested each validated entry against
> `self.committed_polynomials` but never against the rest of `validated`, so two entries in
> one batch sharing a `polynomial_index` both saw a vacant slot; the second `insert`
> overwrote the first while `total_committed` was incremented twice. A third pass now runs
> between validation and insertion (`reconstructor/utils.rs:569-575`), collecting the batch's
> own indices into a `HashSet` and rejecting a repeat as a duplicate.
>
> Rejecting rather than de-duplicating is the right call, and CodeRabbit's addition to this
> finding is why: a
> batch of `num_polys` entries containing a repeat can never complete the cycle, and every
> retransmission of it would then be rejected as a duplicate against the slots the first
> batch _did_ fill. Failing the whole batch keeps the retry path open, which is the same
> property `malformed_commitment_does_not_poison_corrected_retransmission` exists to
> protect.
>
> **L13 detail — now FIXED.** The polynomial counts had converged on 8192 but were still
> two independent literals; the thresholds had **not** converged —
> `protocols/pix/src/lib.rs:57` still said `DEFAULT_POLY_THRESHOLD = 128` against
> `DEFAULT_PIX_SHARES_PER_POLY = 64`, so `SsaGeneratorConfig::default()` implied a quota of
> `8192 × 128 × 1038` = 1.01 GiB, outside the default `quota_range` whose upper bound is
> 519 MiB. That is the original L13 failure mode, relocated from the polynomial count to
> the threshold. **Four** files (not three, as first counted) had grown a workaround comment
> of the form "deliberately not the pix crate's `DEFAULT_POLY_THRESHOLD`, which is still
> 128" — `protocols/pix/benches/{ssa_generator,ssa_reconstructor}_bench.rs`,
> `crypto/packet/benches/packet_bench.rs`,
> `transport/hopr/benches/protocol_throughput_emulated_bench.rs`.
>
> Resolved the way C1 resolved its own version of this — by removing the choice rather
> than re-synchronising the values. `DEFAULT_POLY_THRESHOLD` is now **64**, and the session
> layer's constants are **aliases**, not copies:
>
> ```rust
> pub const DEFAULT_PIX_POLYS_PER_SSA:  u16 = hopr_protocol_pix::DEFAULT_POLYS_PER_SSA;
> pub const DEFAULT_PIX_SHARES_PER_POLY: u16 = hopr_protocol_pix::DEFAULT_POLY_THRESHOLD;
> ```
>
> `hopr-transport-session` already depends on `hopr-protocol-pix`, so the dependency runs
> the right way and the two are now structurally incapable of drifting. All four benchmark
> constants derive from `DEFAULT_POLY_THRESHOLD`, and their workaround comments are gone.
> Production is unaffected: `transport/hopr/src/lib.rs:853-857` builds
> `SsaGeneratorConfig` explicitly from `PixGlobalConfig`, which was already deriving from
> the session-layer constants — only `::default()` consumers (tests, benches) saw 128.

| #       | Finding                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Location                                                                                        |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| L1      | `add_transposed`'s duplicate pre-check uses `entry(..).or_default()`, inserting empty polynomial maps that are left behind when it bails with `DuplicateCommitment`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | `reconstructor/utils.rs:219-224`                                                                |
| L2      | Duplicate polynomial indices inside one `add_transposed` batch silently overwrite (pre-check sees both slots vacant). Unreachable from the wire — the decoder rejects duplicates at `start/lib.rs:694` — but reachable via `process_into_reconstructor`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | `reconstructor/utils.rs:218-230`                                                                |
| L3      | `SsaCommitment::reconstruct_verifiers` compacts a `BTreeMap<CoefficientIndex, _>` via `into_values()`; non-contiguous coefficient indices silently shift positions instead of erroring. Currently only used by tests/benches.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | `types.rs:489-505`                                                                              |
| ~~L4~~  | **Fixed.** `handle_ssa_commit` allocated `mpsc::channel(10)` on every call even when the awaiter was not spawned. The binding now lives inside the first-encounter guard, so only the one call per cycle that uses it pays.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | `manager.rs:2942`                                                                               |
| L5      | If `pix_events.try_send(DepositNeeded)` fails, the already-spawned `DepositAwaiter` is orphaned and the session dies on the kill switch with no diagnostic linking the two.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | `manager.rs:2588-2637`                                                                          |
| L6      | `abort_handles` accumulates one `DepositAwaiter(idx)` entry per SSA cycle that is never removed — only `PixKillSwitch` is `abort_one`'d. ~2000 stale entries on a 1 TB session.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | `manager.rs:2588`, `utils/src/runtime.rs:504`                                                   |
| L7      | `per_peer.weighted_size() == 0` is used as an emptiness test; moka's weighted size is only accurate after `run_pending_tasks()`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | `reconstructor/mod.rs:546`                                                                      |
| L8      | `PixRecoveryStore::iter()` silently `continue`s past corrupt/undecryptable entries — stranded funds with no warning log.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | `pix_recovery_store.rs:289-305`                                                                 |
| L9      | `scrypt::Params::recommended()` (log₂N = 17) allocates ~128 MiB at store open; the `#[allow(deprecated)]` signals upstream API churn.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | `pix_recovery_store.rs:162-168`                                                                 |
| L10     | The derived encryption key is a plain `[u8; 32]` in a `Clone` struct, never zeroized.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | `pix_recovery_store.rs:88-91`                                                                   |
| L11     | If the salt table is missing from an existing DB (older format), a fresh salt is generated and **all existing entries become undecryptable** rather than erroring.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | `pix_recovery_store.rs:208-225`                                                                 |
| L12     | `FundingConfig` doc/default mismatches: `topup_capacity` says "1 GiB" (is 512 MiB), `lower_capacity_threshold` says "128 MiB" (is 512 MiB), `min_safe_capacity_required` says "1 GiB" (is 512 MiB).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | `impls/strategy/src/channel_lifecycle/config.rs`                                                |
| L13     | `DEFAULT_POLYS_PER_SSA = 8192` in the PIX crate is inconsistent with `PixGlobalConfig::num_ssa_parts = 4096`; its implied quota (1.01 GiB) is also outside the default `quota_range`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | `protocols/pix/src/lib.rs:50`, `transport/hopr/src/config.rs:174`                               |
| ~~L14~~ | **Fixed.** `try_new` is now the validating constructor on both types, returning `PixError::InvalidConfiguration`; `new` delegates to it and stays panicking by documented contract, which is what the ~40 bench and test call sites want. The array index is gone: `new_ssa_commitment` builds its `SsaId` from `pseudonym`/`ssa_index` rather than reading it back off `commitments[0]`, so nothing there depends on `polynomials_per_ssa ≥ 1` any more.                                                                                                                                                                                                                                                                            | `generator.rs:141`, `reconstructor/mod.rs:306`, `lib.rs:864`                                    |
| L15     | ~80 lines of PIX event-wiring are duplicated verbatim between the `(Exit, Some)` and `(Relay, Some)` match arms.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | `transport/hopr/src/lib.rs:762-855` vs `:865-…`                                                 |
| L16     | `plans/fix-share-removed-before-verifier.md` (162 lines) is a working document committed to the branch — likely should not ship.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | `plans/`                                                                                        |
| ~~L17~~ | **Fixed.** The recovery-store password lives in the process environment (`/proc/<pid>/environ`); the module threat model documented the config-file case but not this one. Now spelled out, including inheritance by child processes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | `pix_recovery_store.rs:42-52`, `non_anonymous_pix.rs:97-101`                                    |
| ~~L23~~ | **Fixed.** `default_surplus_for` ceiling-divides, so the documented 20 % is a floor rather than an approximation; thresholds 2 and 3 no longer derive a surplus of zero. `DEFAULT_SURPLUS_SHARES` is unchanged (64 is a multiple of four), so no negotiated value moved. Both tolerance tests now sweep the whole accepted threshold range instead of the four multiples of four on which the bug was invisible.                                                                                                                                                                                                                                                                                                                     | `protocols/pix/src/lib.rs:100`                                                                  |
| ~~L25~~ | **Fixed, found while implementing L23.** `42f7edf9c6` moved the default surplus from `threshold/2` to `threshold/4` and added `surplus_must_not_exceed_threshold` without updating the session layer, leaving **two failing unit tests** on the base tip: `default_quota_must_follow_the_default_dimensions` asserted `DEFAULT_PIX_SURPLUS_SHARES == DEFAULT_PIX_SHARES_PER_POLY / 2` (16 against 32), and `new_session_rejects_usepix_when_quota_mismatches_generator` built a generator with surplus 5 against threshold 3, which the new validator panics on. Both fixed, the first re-anchored on `default_surplus_for` so it cannot restate the ratio wrongly again. Five stale `1.5×`/778 MiB prose sites corrected with them. | `transport/session/src/{types,manager}.rs`                                                      |
| ~~L26~~ | **Fixed** (`14bb5aedbe`). The M16 fix made `PixParams` a four-component value while five comments still called it a “triple”, validated “all three”, or compared “the whole triple”. All five corrected, plus two sites the finding did not list (`dimensions`' doc and a test comment). One passage keeps “three” on purpose and now says why: the suite is not a dimension and does not enter the quota.                                                                                                                                                                                                                                                                                                                           | `protocols/pix/src/params.rs`, `transport/session/src/manager.rs`, `protocols/start/src/lib.rs` |

---

## The CodeRabbit pass, folded in

CodeRabbit reviewed PR #8095 in two passes — 45 inline comments, at `16a6acd008` (2026-08-02)
and `b7fa6a7766` (2026-08-03). They were triaged in a separate `CODERABBIT_TRIAGE.md`, which
has since been merged into this document and deleted. This section keeps the parts that do
not belong to an existing finding.

**It was unusually high signal**, and worth saying so because it is the argument for running
it again: every claim checked against the source held up bar one, and **three findings were
things this review had got wrong**:

- **H1's two "closed" races were not closed.** The first (the re-probe on an already-drained bucket) narrowed the window rather than closing it and was fixed by `850df2dfbc`. The second (a parked `RecoveredSsa` whose delivery depends on future unrelated ack traffic) was mitigated by `15af1931ba`, which lets any later acknowledgement collect it, but the no-later-ack case remains open and needs push delivery. Both corrections are recorded under **H1**.
- **M13 was fixed but not pinned.** The test this review cited as pinning it could not have: its `TestSpec` is secp256k1, cofactor 1, so no small-order point exists to feed it. Chasing that produced the sharper conclusion now recorded under **M13** — the filter is unpinnable by _any_ test, because the backend's own `from_bytes` rejects first.
- **L15 was marked fixed when only half of it was.** The event wiring had been extracted; the two match arms had not. Closed for real by `4f30a70629`.

It also independently confirmed **L2** and added the consequence that made the fix's shape
obvious (a batch containing a repeat can never complete, and every retransmission of it would
then be rejected against the slots the first batch did fill — which is why the whole batch is
rejected rather than de-duplicated), and attached a concrete reachability argument to **L14**.

**Everything from that pass is now closed** except **L18** and **H1**'s parked-resolution delivery
residual. The `retired_ssas` capacity audit folded into **H3 Tier 3**'s residual is fixed — the cache
has a capacity derived from the Session layer's live-cycle budget — as is **L19**, its other promoted
item.

### Considered and rejected — recorded so they are not re-filed

- **"The unused `n` binding fails a `-D warnings` build."** The binding is unused; the consequence is not real. `cargo rustc -p hopr-transport-session --lib -- -D warnings` is clean, verified after touching the file to force a rebuild. Cosmetic, not a build breaker.
- **"Use the curve backend's unchecked affine constructor"** (`crypto/packet/src/types.rs`). `babyjubjub_ec::arithmetic::AffinePoint` v0.4.0 exposes no unchecked constructor, and its `new` rejects precisely the order-2 point the test is about. CodeRabbit's own analysis script grepped for `new_unchecked|from_raw_coordinates|from_xy_unchecked`, found nothing, and filed the comment conditionally anyway. The struct literal is the only route, and the test documents why.
- **"Keep `bjj` out of the `hopr-lib` default set."** Rejected as filed, and this one is worth understanding before anyone acts on it. The three-tier arrangement is deliberate and documented at every level: the workspace root deliberately does **not** hard-enable `bjj`, because Cargo features are additive and a hard-enabled `hopr-crypto-packet/bjj` could never be turned off downstream; `hopr-transport` and `hopr-lib` each forward it in `default` so opt-out stays possible; and the root already pins `hopr-lib = { default-features = false }`, so in-workspace consumers do not get it. The proposed remedy — "downstream must pass `default-features = false`" — describes the escape hatch this design exists to provide. Flipping the default would change the production curve for every default build: a protocol decision, not a defect.

  The justification comment used to name `hopr-strategy`'s `NonAnonymousPix` as the consumer needing secp256k1. That consumer is now in the standalone strategy repository rather than in-tree; its requirements therefore cannot be inferred from this workspace alone. `1df62d72c7` reworded the in-tree comment generically.

---

## What is done well

- **`reconstructor/mod.rs` state-machine hardening is genuinely careful.** The
  tombstone (`retired_ssas`) + liveness-map (`ssa_num_polys`) design correctly
  closes the "verifier-widow" race, and the reasoning is documented at the
  declaration rather than in a commit message. The builder-TTL clamp
  (`incomplete_ssa_lifetime.max(unused_verifier_lifetime)`) and the explicit
  "no `max_capacity` here, and here is why" comments are exactly the right level
  of commentary for cache-lifetime invariants.
- **`process_verified_ack` consumes the share only after both the verifier and the
  builder lookups succeed**, so `MissingVerifier` is non-destructive — with a
  regression test that asserts the retention rather than the error.
- **The transactional rewrite of `add_transposed`** (validate-all → check-all →
  insert) genuinely fixes the poisoning bug, and
  `malformed_commitment_does_not_poison_corrected_retransmission` tests the
  actual failure mode.
- **Wire-format decoding in `protocols/start/src/lib.rs` is properly bounded** —
  every length is checked before slicing, `num_polys` is validated against both
  `MAX_POLYS_PER_SSA` _and_ the payload-derived limit, and duplicate indices are
  rejected at decode time.
- **Debug redaction is tested by exact string equality**, not `!contains`, so a
  future field addition that leaks a secret fails the test.
- **The incentive construction is sound**: pre-encrypting the share with the first
  return-path relayer's ack-key solution means the Exit provably cannot extract a
  share without actually forwarding the packet, and `InvalidShare` correctly
  attributes fault to the Entry.
- Subgroup checks (`is_torsion_free`) are applied on all externally supplied group
  elements, and the `HASH_TO_SCALAR_SUITE_ID` is a fixed constant with a comment
  explaining why it must not be derived from `Debug` output.

---

## Status summary

_Re-verified on the combined `lukas/session-pix-supervisor` tip `cd4370f233`, which contains
`lukas/pix` through `d48244448c`, 2026-08-14._

| Finding                                                   | Status                                                                                                                                                                                                                                                                                                                                                                              |
| --------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C1 — default quota range rejects default dimensions       | **fixed** (range re-derived again when H5 moved the quota; still pinned by containment tests)                                                                                                                                                                                                                                                                                       |
| C2 — deposit burned on incomplete cycle                   | by design (author); operator-facing documentation still owed                                                                                                                                                                                                                                                                                                                        |
| C3 — Entry can reclaim its own deposit (rogue-key)        | **fixed** (Schnorr PoK on the client SSA commitment)                                                                                                                                                                                                                                                                                                                                |
| H1 — O(n) `pending_ack_keys` drain per call               | **O(n) defect fixed; residual tracked separately.** The parked-resolution delivery concern is not part of this branch's immediate implementation queue                                                                                                                                                                                                                              |
| H2 — Start channel sizing                                 | **sizing fixed**; retransmission is tracked separately in [#8318](https://github.com/hoprnet/hoprnet/issues/8318)                                                                                                                                                                                                                                                                   |
| H3 — reconstructor memory per session                     | **fixed, all four tiers.** Tier 3's residual closed: the per-cycle model was re-derived post-M9 (the ≈49 MiB in three comments was the deleted Feldman matrix), exported as `peak_cycle_bytes`, and is reserved node-wide at Session admission against `max_live_cycle_bytes`; `retired_ssas` is capped from that budget; a cumulative `max_failed_cycles` closes a batched Session |
| H4 — commitments decoded 2–3×                             | **fixed**                                                                                                                                                                                                                                                                                                                                                                           |
| H5 — quota ignores `surplus_shares`                       | **FIXED** (`15458c1d69`, `20845807ae`) — surplus is on the wire in `PixParams` and priced into the quota; #8237 no longer needed for this                                                                                                                                                                                                                                           |
| H6 — SSAs funded per request                              | **fixed** — batch size, request serialization and the 0.85 floor were already enforced; successor admission now counts Exit→Entry packets received (`returned_packets`), discounted by the surplus ratio, with a near-miss wait so reordering does not kill a conforming Session                                                                                                    |
| H7 — serialised PIX event dispatch                        | **fixed**                                                                                                                                                                                                                                                                                                                                                                           |
| H8 — cycle outlives `unused_verifier_lifetime`            | **fixed** (`SsaCycle`: reclamation scoped to the cycle, not the polynomial)                                                                                                                                                                                                                                                                                                         |
| H9 — SURB eviction silently kills an SSA cycle            | **fixed** (round-robin emission, 2/3 target ceiling, larger ring buffer)                                                                                                                                                                                                                                                                                                            |
| H10 — SURB ring buffer reserves capacity per peer         | **fixed** (`VecDeque`; ~205 GB VSZ ceiling → allocation tracks occupancy)                                                                                                                                                                                                                                                                                                           |
| M1 — `pending_ack_keys` has no TTL                        | **fixed** by H1's rewrite (TTL, bounded capacity, per-bucket cap)                                                                                                                                                                                                                                                                                                                   |
| M2 — `max_awaiting_acks × max_tracked_peers`              | **fixed on base** — `max_ack_buffer_bytes` (1 GiB) enforced at insertion on a measured 400 B/entry, with a resync backstop. The check deliberately permits only a concurrency-sized overshoot; the unbounded caps product is gone. The rejected workload model assumed a Session count and packet rate the node does not enforce                                                    |
| M3 — reconstructor not configurable                       | **fixed** — all eight fields are settable under `pix.reconstructor`; one `ssa_reconstructor()` feeds all three sites via `try_new`, and the merged branch validates the actually installed reconstructor in `SessionManager::start`                                                                                                                                                 |
| M8 — commitment traffic competes with SURBs               | **largely moot** via M9 (~19 000 Start packets per cycle → ~320)                                                                                                                                                                                                                                                                                                                    |
| M9 — naive share verification                             | **fixed** (Feldman removed; one check per polynomial, wire format kept)                                                                                                                                                                                                                                                                                                             |
| M10 — PIX was unmeasured                                  | **fixed** (benchmarks; two optimisation leads handed to M9/M14)                                                                                                                                                                                                                                                                                                                     |
| M11 — `nextest --lib -p hopr-lib` does not compile        | **FIXED** — `cfg` reverted to feature-only; verified compiling. The session crate's test-module gating went with it                                                                                                                                                                                                                                                                 |
| M12 — sequential Exit ack pipeline                        | **fixed**                                                                                                                                                                                                                                                                                                                                                                           |
| M13 — small-order commitment poisons a cell               | **fixed**; not pinned, and unpinnable — the backend's `from_bytes` rejects first, so `is_torsion_free` is defence in depth                                                                                                                                                                                                                                                          |
| M14 — 152 µs per commitment ingest (81 s/cycle)           | **closed** via M9 (81 s → 1.25 s); proposed random-combination subgroup batching is unsound for small cofactor torsion                                                                                                                                                                                                                                                              |
| M15 — programmatic recovery deadline bypasses quota check | **open on this combined branch** — serialized config rejects an impossible deadline, direct `SessionManagerConfig` construction does not                                                                                                                                                                                                                                            |
| M16 — curve override is not negotiated/versioned          | **fixed** — `PixSuite` rides the two free high bits of the `PixParams` word; the Exit refuses a foreign suite with `UnacceptablePixParams` before any curve-sized field is exchanged. Pre-suite BabyJubJub remains compatible and old peers reject new secp words early. One residual: pre-suite secp builds announce zeros and are indistinguishable from BabyJubJub               |
| M17 — threshold calibration omits Entry share generation  | **fixed** — objective stated as Exit bottleneck capacity, `8192 × 64` retained, three false threshold-free comments corrected, both measured tables recorded in-tree, and the total-CPU reading that favours 48 recorded as rejected rather than missed                                                                                                                             |
| M4–M7 — `NonAnonymousPixStrategy` robustness              | **out of scope** — implementation lives in the standalone `hopr-strategy` repository                                                                                                                                                                                                                                                                                                |
| L1, L3, L7, L13, L15, L16                                 | **fixed** (L1/L7 by the H1 and M9 rewrites; L3 removed by M9; L15 completed by `4f30a70629`)                                                                                                                                                                                                                                                                                        |
| L2 — intra-batch duplicate polynomial indices             | **FIXED** (`606e51ea8e`) — batch-local `seen` set; the whole batch is rejected, keeping the retry path open                                                                                                                                                                                                                                                                         |
| L5 / L6 / L18 — awaiter / kill-switch lifecycle           | **fixed by the supervisor branch**, therefore counted fixed in this combined view                                                                                                                                                                                                                                                                                                   |
| L20 — stale `max_awaiting_acks` safety comment            | **fixed in the combined merge sequence** — rebasing onto M3 removes the follow-up's default-config helper and its stale comment                                                                                                                                                                                                                                                     |
| L21 — acknowledgement-budget minimum documentation        | **fixed** — the doc states the enforced 25 600 B floor with its derivation, and `the_documented_ack_budget_floor_is_the_enforced_one` pins the number so prose and validator cannot drift apart again                                                                                                                                                                               |
| L22 — PIX curve-feature override cleanup                  | **fixed** — root manifest and M13 comments describe the override model; the two `Group` imports carry the same cfg arms the curve selection uses, so all three supported combinations are warning-free                                                                                                                                                                              |
| L23 — surplus ratio rounds down                           | **fixed** — ceiling division makes 20 % a floor; both tolerance tests sweep the whole threshold range rather than the four values on which the bug was invisible. No negotiated value moved                                                                                                                                                                                         |
| L25 — `42f7edf9c6` left two failing tests behind          | **fixed** — the surplus ratio change and its new validator were not carried into the session layer; found while implementing L23, along with five stale `1.5×`/778 MiB prose sites                                                                                                                                                                                                  |
| L26 — `PixParams` still documented as a triple            | **fixed** (`14bb5aedbe`) — comments say quadruple/all four; the one passage that keeps “three” now states why (the suite is not a dimension and does not enter the quota)                                                                                                                                                                                                           |
| M4–M7, L8–L12, L17                                        | **out of scope on this branch** — owned by the standalone `hopr-strategy` repository, which consumes `hopr-lib`'s `PixEvent`s                                                                                                                                                                                                                                                       |

### Immediately actionable on the combined branch

1. **M15:** apply the quota-versus-minimum-supported-packet-rate recovery-deadline validation to
   directly constructed `SessionManagerConfig`s, with a direct-construction regression test.

~~**H3 Tier 3 residual**~~ — done: the node-wide live-cycle/tombstone budget is calibrated and
enforced at Session admission, and `max_failed_cycles` settles the batched unpaid-cycle policy.

### Completed `lukas/pix` queue (history)

**The four previously queued items are done.** Recorded for the audit trail:

1. ~~**M17**~~ — the objective is stated as Exit bottleneck capacity, `8192 × 64` retained on that
   basis, and the three comments claiming a threshold-free Entry corrected.
2. ~~**L21**~~ and ~~**L22**~~ — the acknowledgement-budget floor is documented as enforced and
   pinned by a test, and the curve-feature cleanup landed after rebasing onto `ddadbc86ac`.
3. ~~**L23**~~ — ceiling division, with both tolerance sweeps widened past the multiples of four.
   Along the way, **L25**: `42f7edf9c6` had left two failing tests and five stale prose sites in the
   session layer.
4. ~~**M16**~~ — taken further than the documented minimum, at the author's direction: the suite is
   announced on the wire and the Exit refuses a foreign one. The network-wide invariant is
   documented on all three feature definitions as well.

5. ~~**L26**~~ — the five-comment terminology cleanup this re-review raised, done at `14bb5aedbe`.
   It never reopened M16's runtime result.

**Explicitly excluded from that historical base-branch queue:** H1 is tracked separately; H2
retransmission is [#8318](https://github.com/hoprnet/hoprnet/issues/8318); H3 Tier 3, H6 and M15 were
deferred to the supervisor branch, where H6 and H3 Tier 3 are now fixed and M15 is the combined-branch
queue immediately above; L5/L6/L18 are fixed; funding/sweeping/recovery storage belong to
`hopr-strategy`; M14 needs no fix. C2 documentation remains a decision, not immediate implementation
work.

## Calibration results

Measured 2026-08-10 on 48 cores. The first pass was run against the benchmark suite's own model of
production; the figures below are re-measured against the **real operating box**, which the suite
did not describe:

|                            | benchmarks, before                | production        |
| -------------------------- | --------------------------------- | ----------------- |
| polynomials per SSA        | 8 192 (single point)              | 4 096 – 8 192     |
| threshold                  | 64 (never swept on the Exit side) | 16 – 64           |
| surplus                    | 32 (`threshold/2`)                | 20 (flat)         |
| SSAs in flight per Session | 1                                 | 2 – 3             |
| per-Session rate           | 1.5 Mbps                          | 16 – 20 Mbps      |
| clients per Exit           | 100                               | 10 – 30           |
| **aggregate Exit load**    | **18.75 MiB/s**                   | **19 – 72 MiB/s** |
| ack-group fixture width    | 512 polys                         | —                 |

### `use_batch_verification` — settled, `false` stands

`concurrent_quota_rate` **could not answer this as written**: it hard-coded `bench_recon_cfg(true)`,
so both arms of the comparison were the same arm. Parameterised by mode and re-run at production
width (4096 polys, surplus 20), in aggregate MiB/s of Session quota:

| callers | unbatched | batched   |                                    |
| ------- | --------- | --------- | ---------------------------------- |
| 1       | **90.9**  | 46.4      | unbatched 1.96×                    |
| 10      | 130.2     | 126.2     | tie — confidence intervals overlap |
| 48      | 139.8     | **152.6** | batched 1.09×                      |

The hypothesis was right in direction and wrong in consequence. Batching's sequential penalty does
dissolve under concurrency and does eventually invert — but only _above_ the concurrency the pipeline
is configured for. `DEFAULT_ACK_INPUT_CONCURRENCY`
(`transport/hopr/src/protocol/pipeline/mod.rs:33`) is **10**, exactly the row where the two are
indistinguishable. `false` stays, on a better argument than before: far superior at low concurrency,
indistinguishable at the configured one, therefore the safer default across the range an operator can
set. Kept configurable for whoever raises `ack_input_concurrency` well past its default.

**Correction — headroom was overstated by ~4×.** The first pass reported "7.3× the 18.75 MiB/s that
100 concurrent Sessions demand". That compared against the _benchmark's_ model, which understates
per-Session rate by 13×. A real Exit at 30 clients × 20 Mbps absorbs **71.5 MiB/s**, so the headroom
at the configured concurrency is **1.8×**, not 7.3×. PIX is far closer to binding than the first
reading suggested. Production width itself costs about 5 % of the rate — the rest of the gap is the
load model.

### polys/threshold — CLOSED under the stated Exit-capacity objective (M17)

The `all-benchmarks` sweep **aborted on its first point**: `POLYNOMIALS` read
`[65535, 32768, 16384, 8192]` against `polynomials_per_ssa`'s validated maximum of 16 192, so three
of four points panicked and only the deployed one was ever constructible. It was then briefly
repointed at an iso-quota diagonal, which compiled but modelled polynomial counts no node runs. Both
mistakes share a root — choosing the sweep from an idea about the parameter space rather than from
deployed configuration — and a `const` assertion now makes the first one a build failure.

Repointed at the production box, **Entry commitment construction** is unambiguous (ms per
`new_ssa_commitment`):

| polys | t=16  | t=32  | t=48  | t=64      |
| ----- | ----- | ----- | ----- | --------- |
| 4 096 | 29.59 | 30.28 | 31.10 | 31.68     |
| 6 144 | 43.77 | 44.92 | 45.94 | 46.99     |
| 8 192 | 57.92 | 59.35 | 60.98 | **62.32** |

A 4× threshold change moves this cost 7 %; a 2× polynomial change moves it 1.96×. M9's model —
_commitment_ work linear in `polys` and nearly independent of `threshold` — holds across the box.
It does not describe `next_share`, which evaluates a threshold-wide polynomial for every packet.

The **Exit** side is now measured too, by the new `acknowledge_shares/interpolation` group (128
polynomials, `surplus_shares: 0` so every share completes work, reported per share so the two
threshold-quadratic terms in `SsaPartBuilder::add_share` — the Lagrange `combine()` and the
duplicate-identifier scan — show up as a rising line if they matter):

| threshold | shares/s   | µs/share  | µs/polynomial |
| --------- | ---------- | --------- | ------------- |
| 16        | 76.1 K     | **13.15** | 210.4         |
| 32        | 89.9 K     | 11.12     | 355.9         |
| 48        | **94.2 K** | **10.62** | 509.5         |
| 64        | 93.6 K     | 10.68     | 683.5         |

**The line falls and then flattens — it does not rise.** Fitting µs/polynomial = `A + B·t + C·t²`
gives `A ≈ 81 µs`, `B ≈ 7.61 µs`, `C ≈ 0.028 µs`, reproducing all four points to within 0.1 %. Read
per share that is `A/t + B + C·t`, and the terms say the whole story:

- `A/t` — the **fixed per-polynomial cost**, dominated by the one fixed-base multiplication
  `verify_reconstructed` performs to open the commitment (M9 measured it at 62 µs). It is amortised
  over `threshold` shares, so it _punishes low thresholds_: 5.09 µs/share at t=16 against 1.27 at
  t=64.
- `C·t` — interpolation and the duplicate scan. Real, but 1.79 µs/share at t=64 and 0.45 at t=16.

So the two Exit effects pull in opposite directions and the fixed cost wins across the deployed
range. The Exit-only per-share optimum is at `t = √(A/C) ≈ **54**`, the curve is flat within 0.5 %
from 48 to 64, and **the deployed 64 sits within 0.4 % of the Exit-only optimum**. Dropping to 16
costs **23 % of Exit throughput**.

This inverts the hypothesis the item was queued on. M9 removed the per-share MSM and left
interpolation as the only threshold-growing cost, which suggested a lower threshold would be
cheaper; what it actually left is a per-_polynomial_ cost that a lower threshold has fewer shares to
amortise. Interpolation never gets big enough to overtake it below 64.

The missing Entry packet-generation term changes the combined reading. The current-tip
`all-benchmarks` sweep measures `next_share` at 0.90, 1.20, 1.51 and 1.82 µs/share for thresholds
16, 32, 48 and 64. Adding those to the Exit figures makes the sampled total 14.05, 12.32, 12.13 and
12.50 µs/share: 48 is about 3 % cheaper than 64 in that simple total-CPU model. Entry generation is
still much smaller than Exit reconstruction. The author has now explicitly chosen **Exit bottleneck
throughput** as the objective because one Exit serves 10–30 clients while an Entry generates its own
shares. Under that stated objective `8192 × 64` remains settled; the total-CPU result is retained so
the rejected alternative is visible rather than mistaken for an omitted measurement.

### The surplus is now a ratio of the threshold — FIXED (L23)

`DEFAULT_SURPLUS_SHARES` was `DEFAULT_POLY_THRESHOLD / 2`: a _ratio_, but evaluated once at the
**default** threshold and then applied whatever the configured one was, while
`PixGlobalConfig::additional_shares` was an absolute count. Across the deployed 16–64 range a flat 20
meant the emission factor swinging from 1.31× to **2.25×**, where the insurance exceeds the shares it
insures — and since H5 the surplus travels in the negotiated `PixParams` and is billed on purchase
rather than on claim, so that over-insurance is quota paid for in every deposit. The field's own
documentation named the hazard ("**an absolute share count, not a ratio** … Re-tune the two
together") and nothing enforced it.

A ratio is the physically correct shape: a polynomial reconstructs from the first `threshold`
distinct shares of `threshold + surplus` emitted, so surviving loss rate `p` needs
`surplus ≥ threshold · p/(1−p)`, and `surplus/(threshold + surplus)` **is** the loss rate covered.
`default_surplus_for(threshold) = threshold.div_ceil(4)` makes **20 % a floor**. At thresholds
divisible by four it is exact:

| threshold | surplus | emitted | covers |
| --------- | ------- | ------- | ------ |
| 16        | 4       | 20      | 20 %   |
| 32        | 8       | 40      | 20 %   |
| 48        | 12      | 60      | 20 %   |
| 64        | 16      | 80      | 20 %   |

For other accepted thresholds ceiling division over-covers by less than one share rather than
under-covering. Threshold 33 now receives 9 surplus shares and covers 21.43 %; thresholds 2 and 3
receive one instead of zero. Both tolerance tests sweep the full accepted threshold range, so the
rounding boundary is no longer hidden by testing only multiples of four.

Against the previous flat 20 that is −4.8 % billed quota at threshold 64 and −44 % at 16.

`PixGlobalConfig::additional_shares` becomes `Option<usize>` — serde cannot express "default to a
function of a sibling field", so the field alone cannot be the configuration — with a
`surplus_shares()` resolver every reader goes through. Leaving it unset is now the recommended
setting.

Two validators enforce "insurance must not exceed the payload it insures", one on
`SsaGeneratorConfig` in `hopr-protocol-pix` so it binds every constructor, one on `PixGlobalConfig`
so the error names the operator's own field. The bound is deliberately loose at 2.0× emission:
over-insuring a genuinely lossy path is a legitimate choice, paying for more redundancy than payload
is not.

**This makes some previously valid configurations invalid** — specifically a flat surplus at or below
threshold 20, which is exactly the case the finding is about. That is intended, and it fails at
startup with a message naming the field rather than degrading silently.

## Remaining work

**The standalone `lukas/pix` queue is empty, and H6 and H3 Tier 3 are now fixed, leaving this
combined branch one residual:**

- **M15:** enforce quota/recovery-deadline consistency for direct `SessionManagerConfig`
  construction.

The remaining work owned elsewhere is:

- **H1**'s parked-resolution delivery residual — tracked separately.
- **H2**'s `SsaCommit` retransmission — [#8318](https://github.com/hoprnet/hoprnet/issues/8318).
- **M4–M7, L8–L12, L17** — the standalone `hopr-strategy` repository.
- **C2**'s operator-facing documentation — a decision, not implementation work.
- **M14** needs no fix.

`PIX_BENCH_PLAN.md` is a separate queue: multi-tenant acknowledgement rate, ack-buffer occupancy
under loss, cycle rotation under load, and memory at true multi-tenancy. Its §Part 5 prose
corrections for `quota_range`'s stale "≈195 MiB to ≈778 MiB" are done here, as L23 fallout; its
`MAX_DEFERRED_ACKS_PER_CYCLE` re-derivation is not.

---

_The base-branch sweep's L19 is fixed. The merged supervisor fixes L18 by deleting the awaiter
machinery, and L20 disappeared when merged over the base branch's configured reconstructor helper.
H1/H2 are tracked separately; M15 is the last combined-branch residual._
