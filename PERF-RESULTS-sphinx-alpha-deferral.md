# Perf results: Sphinx alpha deferral & single point encoding

- **Branch:** `perf/sphinx-defer-alpha-and-single-compress`
- **Commit:** `a075a107e2`
- **Files:** `crypto/packet/src/sphinx/shared_keys.rs`, `crypto/packet/src/sphinx/packet.rs` (+117 / −19)
- **Date:** 2026-11-18
- **Commit is unsigned** (no ssh agent in the session; `git commit -S` failed with `Couldn't get agent socket?`, re-run without `-S` per environment rule).

## Motivation

Profiling (`perf record -F 999 -g --call-graph dwarf`, DWARF unwinding; `linux-perf` installed for this task)
of the packet hot paths showed elliptic-curve scalar multiplication dominates, in every direction:

| path | dominant cost |
|---|---|
| relay decode | ~30% `curve25519_dalek` variable-base mul; secp256k1 PoR (`fe_sqrt`, `gej_add_ge`); ChaCha/Blake3 |
| final-hop decode | ~36% variable-base mul; ~13% field ops — **two** scalar mults per packet |
| sender (3 hops + 2 SURBs) | ~25% variable-base mul per hop (forward + each return path) |

Replay filter (~68 ns op), session dispatch (sub-µs) and mixer enqueue were already cheap and were left alone.

## Changes (all mechanical, hardware-independent)

1. **Defer the alpha blinding on the final hop.** `SharedKeys::forward_transform` eagerly computed the
   blinded alpha — a second variable-base scalar multiplication — before the header was inspected.
   Only a *relayed* packet consumes it; a final-hop decode discards it. Split into
   `forward_transform_deferred` (returns `DeferredAlpha` + secret) and `DeferredAlpha::finalize()`.
   Only the `ForwardedHeader::Relayed` branch in `MetaPacket::into_forwarded_with_secret` finalizes it.
2. **Encode the shared EC point once.** Both `SharedKeys::generate` (sender) and `forward_transform`
   (relayer) derive two KDF keys — the secret and the blinding factor — from the same point and called
   `to_alpha()` for each. On Edwards `to_alpha` is a field inversion. Added
   `GroupElement::extract_key_from_alpha`; both derivations now share one encoding.
3. **Fixed-base alpha in `SharedKeys::generate`.** `alpha_prev` is `coeff_prev · G` by construction, so
   the update is the *fixed-base* multiply `G::generate(&coeff_prev)`, not the previous variable-base
   `alpha_prev.mul(&b_k)`. Same point (associativity: `(coeff·G)·b == (coeff·b)·G`), precomputed table.

`forward_transform` is retained as `#[cfg(test)]`-only; the Sphinx suite test now asserts the deferred
form yields the same secret and the same finalized alpha as the eager form.

## Benchmark method

Isolated A/B: baseline built from `HEAD` (`ff0076efaa`) in a separate git worktree with its own
`CARGO_TARGET_DIR`; the fixed build from the branch. Both binaries run back-to-back on a quiet box, one
process at a time (an earlier attempt ran baseline and fixed concurrently and produced contaminated,
noisy numbers — those were discarded). Criterion defaults otherwise; `--sample-size 30–60`.
`change:` intervals reported by criterion, all `p < 0.001`.

## Results (isolated A/B, significant)

| Benchmark | Before | After | Δ |
|---|---|---|---|
| `hopr_decoder/recipient` (final hop) | 155.45 µs | 135.47 µs | **−13.3%** |
| `hopr_decoder/ack_recipient` | 158.64 µs | 132.77 µs | **−16.2%** |
| `hopr_decoder/relay_replay` (rejected replay) | 45.69 µs | 24.25 µs | **−47.0%** |
| `hopr_decoder/relay` | 219.66 µs | 216.35 µs | −1.8% |
| `hopr_encoder/3hop_2surbs` | 391.40 µs | 348.73 µs | **−10.9%** |
| `packet_receiving/any_hop` | 58.640 µs | 36.918 µs | **−37.0%** |
| `packet_sending_no_precomputation/3_hop_2_surbs/pix_off` | 400.69 µs | 365.48 µs | **−8.8%** |
| `packet_forwarding/any_hop` | 76.856 µs | 74.869 µs | −2.6% |
| `pipeline_e2e_forward/3hop_5000pkt` | 211.46 ms (73.2 MiB/s) | 184.15 ms (84.1 MiB/s) | **−12.9% (+14.8% thrpt)** |

`relay` moves least because a relay still must compute the blinded alpha (it forwards); the win there is
items 2 and 3 only. `recipient` / `packet_receiving` move most because the deferred multiply is one of
the two per-packet scalar mults.

## Measurements that are NOT part of the isolated sweep

- `packet_precompute/3_hop_2_surbs/pix_off`: 353.4 → 331.5 µs (≈ −6%) measured across runs of the fixed
  build at different times, **not** in the final isolated A/B. Directionally consistent with item 1/3
  (precompute builds paths through `SharedKeys::generate`); re-measure before quoting.
- `protocol_throughput_pipeline` (emulated): intermediate run showed 405.8 → 382.7 ms (≈ −5.5%), but the
  final run was taken while the baseline bench was running and regressed to 440 ms — contaminated.
  **Re-measure in isolation before relying on it.**

## Test evidence

- `cargo nextest run -p hopr-crypto-packet -p hopr-protocol-hopr --lib` → **266 passed, 0 failed**.
- `cargo nextest run -p hopr-transport --features testing --test '*' -j 1` → **61 passed, 0 failed**
  (includes `protocol_stress_throughput`, `protocol_msg_ack_workflows`, `pipeline_stage_isolation`).
- `cargo clippy -p hopr-crypto-packet --all-targets` and `cargo fmt --check`: clean.
- New coverage: the suite test compares deferred vs eager secret and alpha for every hop.

## Caveats / things a reviewer should probe

- Associativity argument for item 3 assumes `G::generate` is exactly `scalar · G` for all suites
  (Ed25519 used by `HoprSphinxSuite`; secp256k1 and x25519 suites also implement `SphinxSuite`).
- `DeferredAlpha` carries the raw `b_k` blended into the secret's derivation; confirm the pairing
  (secret ↔ tag ↔ alpha) cannot be crossed by a caller. `ForwardSecret`'s fields stay private and the
  only external consumer is `.tag()`.
- Constant-time: `to_alpha` is now called once instead of twice — same operation, fewer times; no new
  branching introduced. `finalize()` is on the relayed branch, as the multiply was before.
- Pre-existing and unrelated: `cargo test --all-features` for `hopr-crypto-packet` fails to compile
  (ambiguous `X25519Suite` / `Secp256k1Suite` imports). Reproduced on clean `HEAD`.
- An `expand_path` lock-batching attempt was measured at no significant gain (p≈0.27) and reverted.
  Its first draft self-deadlocked (`return self.f(...)` evaluating while holding a read guard); caught
  by a bench hang, fixed, then reverted for lack of gain.
