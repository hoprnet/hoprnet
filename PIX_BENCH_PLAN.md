# Benchmark PIX at the answered production envelope

## Context

The PIX benchmark suite was recalibrated in `46beab643e`, `b44953f904` and `42f7edf9c6` against a
production profile of 4096–8192 polynomials, threshold 16–64, 16–20 Mbps per Session, 10–30 clients
per Exit, 2–3 SSAs in flight. Two facts have since been added: the **on-chain deposit settles in
4–6 s** (settlement only — the `SsaRequest` round trip, commitment generation and `SsaCommit` burst
are separate and additional), and **cycles are not run shorter than ~120 s**, so the deposit stays
under 5 % of a cycle.

Those two close the envelope, and closing it exposes that the suite still measures a shape no Exit
runs: **every acknowledgement group builds one `OffchainKeypair::random()` and one `SsaId`**
(`ssa_reconstructor_bench.rs:420, 472, 609, 683, 790, 879, 954`). `bench_acknowledge_shares_concurrent`
runs N threads, but all N hit the same peer and the same cycle — the concurrency dimension is
pipeline width, not tenancy. Nothing in the suite has had two peers or two cycles live at once, and
nothing rotates a cycle. Production is 30 peers and 90 resident cycles at ~72 000 shares/s aggregate.

### What the answered envelope changes

**The viable dimension box is much smaller than the configurable one.** A 120 s floor at 20 Mbps
(2.5 MB/s) means quota ≥ 300 MB; at 16 Mbps, ≥ 240 MB. Since
`quota = polys × threshold × 1.25 × 1038`, that is `polys × threshold ≥ 231 000` (20 Mbps) or
`≥ 185 000` (16 Mbps). Within the stated 4096–8192 × 16–64 box only three points qualify:

| polys × threshold | product | cycle @ 20 Mbps | viable |
| ----------------- | ------- | --------------- | ------ |
| 8192 × 64         | 524 288 | 272 s           | ✓      |
| 8192 × 32         | 262 144 | 136 s           | ✓      |
| 4096 × 64         | 262 144 | 136 s           | ✓      |
| 8192 × 16         | 131 072 | 68 s            | ✗      |
| 4096 × 32         | 131 072 | 68 s            | ✗      |
| 4096 × 16         | 65 536  | 34 s            | ✗      |

**Threshold 16 is unreachable in the stated envelope** — it needs ≥ 16192 polynomials or ≤ 11 Mbps.

**Two corrections to what I said before the deposit was known.** Both changed what is worth building:

1. _The interpolation result overstates the threshold's value._ The measured sweep gives 76.1 K
   shares/s at t=16 and 93.6 K at t=64, and `42f7edf9c6`'s message reports "dropping to 16 costs 23 %
   of Exit throughput". Within the operable range (t=32–64) the spread is 89.9 K → 93.6 K, i.e.
   **~4 %**. The verdict "8192 × 64 stands" is unchanged and in fact strengthened; the lever is just
   much smaller than recorded.
2. _The deferred-ack cap is not the correctness risk I ranked first._ I sized it against line rate
   and got a 13× shortfall. The binding constraint is not line rate but the gap between
   `early_recovery_threshold` (0.85) firing `request_next_ssa` and the emission window straddling
   the SSA boundary. `SHARE_EMISSION_WINDOW` is 256 (`generator.rs:70`), so the straddle begins only
   when fewer than 256 polynomials of the current cycle remain — 3.1–6.25 % of it. The gap is
   therefore 8.75–11.9 % of a cycle, i.e. **10.5–32 s at a 120 s floor**, against a commitment
   exchange of roughly 1–2 s (RTT + `new_ssa_commitment` + 147–293 `SsaCommit` packets). 5–30×
   margin. It is a **documentation defect only** — see Part 5.

**Derived rates used throughout below.** Aggregate 30 × 20 Mbps = 72 254 shares/s. Cycle boundary
every 4–9 s node-wide; at `ssas_per_request` = 3, a burst of 3 installs (12 288–24 576 commitments,
441–879 `insert_coefficient_commitments` calls) every 12–27 s. Under 20 % loss the awaiting-ack
buffer holds `loss × rate × TTL` = 0.2 × 72 254 × 30 s ≈ **433 000 entries ≈ 173 MB**, versus
~36 000 for the lossless `rate × RTT` case the suite implicitly measures.

## Part 1 — Multi-tenant acknowledgement rate

New group in `protocols/pix/benches/ssa_reconstructor_bench.rs`:
`SsaReconstructor::acknowledge_shares/multi_tenant_quota_rate`.

This is the item that revises the headroom number the capacity story rests on — today's figure is a
single-tenant measurement.

Shape, taken from the code rather than invented: a Session draws from **one** cycle except within
the last 256 polynomials, where the window straddles two (`generator.rs:229-244`). So of the 90
resident cycles, ~30 are actively emitting at any instant and ~60 are resident-but-idle.

Points `(peers, resident cycles, actively emitting)`:

- `(1, 1, 1)` — control, identical shape to `sustained_quota_rate` so the two are comparable
- `(30, 30, 30)` — one cycle per Session
- `(30, 90, 30)` — the batch of 3 resident, one emitting per Session
- `(30, 90, 60)` — every Session mid-straddle; the worst realistic instant

Reuse `generate_commitment_matrix` (`:222`), `install_commitment` (`:241`) and `stage_shares`
(`:254`); `stage_shares` needs a variant taking a peer slice and round-robining across it, since it
currently closes over one `&OffchainKeypair`.

Use a **narrow polynomial count (512) for the resident set**, and say so in the group's doc: the
dimension under test is fan-out and contention, not per-cycle width, and width is already covered at
4096 by `sustained_quota_rate`. 90 × 4096 of commitment installation is minutes of untimed setup for
no extra signal.

Report `Throughput::Bytes` so the ids sit directly beside `sustained_quota_rate`. Gate the 90-cycle
points behind `all-benchmarks`.

What it answers: whether the measured headroom survives fan-out, and whether `ack_buffer_entries`
costs anything. That counter is a single `AtomicUsize` incremented by every insert
(`reconstructor/mod.rs:1284`) and decremented by every eviction (`:1296`) — ~144 000 RMWs/s on one
cache line, from every pipeline thread. It was added for M2 and has never been measured under
fan-out.

## Part 2 — Ack-buffer occupancy and resync cost under loss

Two pieces, both about the 433 000-entry figure that no measurement has ever produced.

**New group `SsaReconstructor::resync_ack_buffer`**, over occupancy ∈ {10 k, 100 k, 433 k} spread
across {1, 30} peers. Cheap to stage — `insert_encrypted_share` needs no cycle, so no commitment
matrix is required. It measures `count_ack_buffer_entries` (`reconstructor/mod.rs:911`), which calls
`run_pending_tasks()` on the outer cache plus once per peer and then iterates every entry, and which
runs **inline on the insertion path** (`:1271`). This is what justifies or revises the
`max_ack_buffer_bytes` floor (validated at 25 600 B; default 1 GiB).

**A loss dimension in `protocols/pix/tests/memory_profile.rs`**: acknowledge only `1 − p` of staged
shares and report the resulting steady-state occupancy and bytes, so the number is measured rather
than modelled. `p = 0` and `p = 0.2` (what the surplus is sized for) are the two points that matter.

## Part 3 — Cycle rotation under load

New group `SsaReconstructor::acknowledge_shares/during_cycle_install`, gated behind
`all-benchmarks`.

A steady acknowledgement stream at the `sustained_quota_rate` shape while a background thread
installs a batch of 3 cycles. Reports throughput during install against quiescent — the "does a
client joining stall the others" number. Nothing today overlaps the two paths, and they contend on
the same caches.

## Part 4 — Memory at true multi-tenancy

`memory_profile.rs:456-470` computes `SSAS_IN_FLIGHT × SESSIONS_PER_EXIT` = 90 cycles and prints
both an in-phase and a staggered extrapolation from a **single** measured cycle. At the ~49 MiB per
cycle quoted in `manager.rs`'s `ssas_per_request` doc that is 4.4 GiB in phase — the largest
production number in this whole analysis, resting entirely on an assumption of linearity.

Add a measured 90-cycle point at reduced width and compare it against 90 × the single-cycle figure,
so the assumption is checked rather than asserted.

## Part 5 — Documentation corrections (no benchmark)

- **`MAX_DEFERRED_ACKS_PER_CYCLE`** (`reconstructor/mod.rs:252-264`): the derivation cites
  "~181 shares/s at the deployed 1.5 Mbps per-Session cap". Re-derive against the straddle gap
  (10.5–32 s at a 120 s floor) rather than line rate, and reconcile with `generator.rs:240-244`,
  which tells the reader to size it for the straddle and is the more accurate of the two. The
  constant does not need to change.
- **`quota_range`'s prose** (`transport/session/src/manager.rs:557`) says "≈ 195 MiB to ≈ 778 MiB".
  Those figures are `8192 × (64 + 32) × 1038`, i.e. the **old** `DEFAULT_SURPLUS_SHARES =
threshold/2`. My own `42f7edf9c6` moved the surplus to `threshold/4`, so the real range is now
  **162–649 MiB**. The constant is derived and correct; only the prose is stale.
- **Bench module doc** (`ssa_reconstructor_bench.rs:3-17`) and `memory_profile.rs`: record the 120 s
  cycle floor, the deposit's 4–6 s settlement, and the viable-dimension table above.
- **The interpolation conclusion**: note that within the operable range the threshold is worth ~4 %,
  not the 23 % the sweep's endpoints suggest.

## Findings to record, not implement

`PIX_PR_REVIEW.md` only — a behaviour change that needs sign-off separately:

**The default `quota_range` floor admits sub-floor cycles.** `DEFAULT_PIX_QUOTA_RANGE_SPAN` is 4, so
the accepted minimum is 162 MiB = 170 MB, which at 16–20 Mbps rotates in **68–85 s** — well under the
120 s floor. A span of **2** would put the minimum at 324 MiB = 340 MB, i.e. 136 s at 20 Mbps. The
Exit cannot see the Session rate, so this cannot be validated, only defaulted; it is a one-constant
change with Session-acceptance consequences, which is why it is recorded rather than made here.

## Verification

```bash
cargo nextest run --lib -p hopr-protocol-pix
cargo nextest run -p hopr-protocol-pix --test memory_profile -j 1 -- --ignored
cargo bench -p hopr-protocol-pix --bench ssa_reconstructor_bench -- 'multi_tenant|resync'
cargo bench -p hopr-protocol-pix --no-run
cargo bench -p hopr-protocol-pix --features all-benchmarks --no-run
cargo clippy --workspace --all-targets
nix fmt
```

Criterion comparisons must use `--save-baseline` / `--baseline` rather than the reported `change`
column, which compares against whatever run was stored last — the trap that produced the spurious
"+17 %/+71 %" readings on `insert_encrypted_share` earlier in this work.

The multi-tenant group's control point `(1, 1, 1)` must land within noise of
`sustained_quota_rate`'s production point; if it does not, the fan-out harness itself is being
measured and the other points mean nothing. That is the group's own correctness check and should be
asserted, not eyeballed.
