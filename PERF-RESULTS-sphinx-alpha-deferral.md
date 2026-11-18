# Perf results: Sphinx alpha deferral & single point encoding

- **Branch:** `perf/sphinx-defer-alpha-and-single-compress` (PR #8700)
- **Benchmarked fixed revision:** `6b196b5dde` (the final head: guard, small-order `is_valid`, review follow-up)
- **Baseline:** `ff0076efaa`
- **Files:** `crypto/packet/src/sphinx/shared_keys.rs`, `crypto/packet/src/sphinx/packet.rs`,
  `crypto/packet/src/sphinx/ec_groups.rs`
- **Date:** 2026-11-18
- **All commits are SSH-signed** with `git-signer`; GitHub reports `verified: true`.

## Motivation

Profiling (`perf record -F 999 -g --call-graph dwarf`, DWARF unwinding; `linux-perf` installed for this task)
of the packet hot paths showed elliptic-curve scalar multiplication dominates, in every direction:

| path                      | dominant cost                                                                                     |
| ------------------------- | ------------------------------------------------------------------------------------------------- |
| relay decode              | ~30% `curve25519_dalek` variable-base mul; secp256k1 PoR (`fe_sqrt`, `gej_add_ge`); ChaCha/Blake3 |
| final-hop decode          | ~36% variable-base mul; ~13% field ops — **two** scalar mults per packet                          |
| sender (3 hops + 2 SURBs) | ~25% variable-base mul per hop (forward + each return path)                                       |

Replay filter (~68 ns op), session dispatch (sub-µs) and mixer enqueue were already cheap and were left alone.

## Changes (all mechanical, hardware-independent)

1. **Defer the alpha blinding on the final hop.** `SharedKeys::forward_transform` eagerly computed the
   blinded alpha — a second variable-base scalar multiplication — before the header was inspected.
   Only a _relayed_ packet consumes it; a final-hop decode discards it. Split into
   `forward_transform_deferred` (returns `DeferredAlpha` + secret) and `DeferredAlpha::finalize()`.
   Only the `ForwardedHeader::Relayed` branch in `MetaPacket::into_forwarded_with_secret` finalizes it.
2. **Encode the shared EC point once.** Both `SharedKeys::generate` (sender) and `forward_transform`
   (relayer) derive two KDF keys — the secret and the blinding factor — from the same point and called
   `to_alpha()` for each. On Edwards `to_alpha` is a field inversion. Added
   `GroupElement::extract_key_from_alpha`; both derivations now share one encoding.
3. **Fixed-base alpha in `SharedKeys::generate`.** `alpha_prev` is `coeff_prev · G` by construction, so
   the update is the _fixed-base_ multiply `G::generate(&coeff_prev)`, not the previous variable-base
   `alpha_prev.mul(&b_k)`. Same point (associativity: `(coeff·G)·b == (coeff·b)·G`), precomputed table.

`forward_transform` is retained as `#[cfg(test)]`-only; the Sphinx suite test now asserts the deferred
form yields the same secret and the same finalized alpha as the eager form.

## Benchmark method

Isolated A/B, **re-measured on the final head `6b196b5dde`** against `ff0076efaa`: the baseline is built in
a separate git worktree with its own `CARGO_TARGET_DIR`, the fixed build from the branch, and the two
binaries run back-to-back on a quiet box, one process at a time. Each benchmark below was confirmed twice; the
values are single runs at `--sample-size 20–25` unless a row says otherwise. Where an early reading disagreed
with a later one, the pair was re-run **interleaved** (base, fixed, base, fixed, base, fixed) and the
interleaved result is the one quoted — that is how the relay row's first 235 µs reading was shown to be drift.

Machine: AMD EPYC 9575F, 16 vCPU, Linux 7.0.0. Raw Criterion samples live under the untracked
`target/criterion/` of each build and were not archived; re-running the two binaries reproduces them.

## Results (isolated A/B, final head `6b196b5dde`)

The MiB/s figures are **normalized to the maximum payload capacity** (`HoprPacket::PAYLOAD_SIZE` per
packet), which is what the benchmark's `Throughput` declares; the fixtures carry 300 application bytes per
packet, so the application-level rate is lower — for the 5,000-packet e2e run, 1.5 MB in 184.7 ms ≈ 7.8 MiB/s
of application data. The relative speedups are unaffected by the normalization.

| Benchmark                                                | Before                  | After                   | Δ                         |
| -------------------------------------------------------- | ----------------------- | ----------------------- | ------------------------- |
| `packet_receiving` (final-hop decode)                    | 58.41 µs                | 37.30 µs                | **−36.1%**                |
| `hopr_decoder/relay_replay` (rejected replay)            | 45.71 µs                | 24.40 µs                | **−46.6%**                |
| `pipeline_e2e_forward/3hop_5000pkt`                      | 211.43 ms (73.2 MiB/s)  | 184.74 ms (83.8 MiB/s)  | **−12.6% (+14.5% thrpt)** |
| `packet_precompute/3_hop_2_surbs/pix_off`                | 369.21 µs               | 327.51 µs               | **−11.3%**                |
| `packet_sending_no_precomputation/3_hop_2_surbs/pix_off` | 402.49 µs               | 367.51 µs               | **−8.7%**                 |
| `hopr_encoder/3hop_2surbs`                               | 387.92 µs               | 357.90 µs               | **−7.7%**                 |
| `protocol_throughput_pipeline/63.4 MiB/pix_off`          | 440.86 ms (143.8 MiB/s) | 415.23 ms (152.7 MiB/s) | **−5.8% (+6.2% thrpt)**   |
| `hopr_decoder/ack_recipient`                             | 155.54 µs               | 150.03 µs               | −3.5%                     |
| `hopr_decoder/recipient`                                 | 155.88 µs               | 152.11 µs               | −2.4%                     |
| `packet_forwarding` (relay decode)                       | 76.96 µs                | 75.57 µs                | −1.8%                     |
| `hopr_decoder/relay`                                     | 217.60 µs               | 217.49 µs               | −0.1% (flat)              |

`hopr_decoder/recipient` and `ack_recipient` are smaller than in the first measurement round (155.5 → 152.1
rather than 155.5 → 135.5). Both figures are real; the gap is the small-order guard now on the hot path. The
guard is ~0.5% when it is `is_small_order` (three doublings) and was ~40% when it was `is_torsion_free` (a
full scalar multiplication) — see "Guard cost" below. The relay row is flat because it pays the guard and the
`to_alpha` deduplication but not the deferred multiply, which only the final hop skips.

`relay` moves least because a relay still must compute the blinded alpha (it forwards), so only items 1
and 2 help it — **not** item 3, whose fixed-base update is in `SharedKeys::generate`, which a relay never
calls. `recipient` / `packet_receiving` move most because the deferred multiply is one of their two
per-packet scalar mults. `packet_precompute` pays `SharedKeys::generate` once per path, which is why
item 3 shows up there.

`protocol_throughput_pipeline/pix_on` showing no change is expected rather than disappointing: with PIX
on, each SURB also carries a share, so share generation dilutes the fixed Sphinx saving. Its `pix_off`
twin moves with the rest.

## Earlier measurements superseded by the isolated sweep

`packet_precompute` and both `protocol_throughput_pipeline` ids were re-measured in the isolated A/B above
and are now quoted from it. The earlier ad-hoc readings — `packet_precompute` ≈ −6% from runs of the
fixed build at different times, and a throughput reading taken while the baseline bench was running that
regressed to 440 ms — are **superseded**; the table rows above are the numbers to cite.

The re-measured `packet_precompute` (−9.8%) is consistent with item 1/3 through `SharedKeys::generate`,
which precompute pays once per path.

## Independent verification (two reviewers, parallel, same brief, neither saw the other)

Both reviewers re-derived the analysis from scratch and ran their own A/B harnesses against the parent
commit's verbatim code:

| claim                                                                   | Sonnet                                           | Opus                                                              |
| ----------------------------------------------------------------------- | ------------------------------------------------ | ----------------------------------------------------------------- |
| 1. `alpha_prev = coeff_prev·G`; fixed-base `generate` is the same point | CONFIRMED (16 trials × 3 suites, byte-identical) | CONFIRMED (100 trials × 5 hops Ed25519; 20 each secp256k1/x25519) |
| 2. single `to_alpha()` leaves secret + blinding factor byte-identical   | CONFIRMED                                        | CONFIRMED                                                         |
| 3. deferral is output-preserving; no caller can cross-pair              | CONFIRMED                                        | CONFIRMED                                                         |

Both confirmed `G::generate(s) == s·G` for all three suites (Ed25519 `mul_base`, secp256k1
`mul_by_generator`, X25519 `mul_base(..).to_montgomery()`, where the Montgomery `u`-coordinate is invariant
under negation). Both observed **267** tests after the fix below (266 before it) and clean clippy/fmt.

### Defect they independently found, and the fix

The deferral moved more than the multiply: the old eager `forward_transform` also ran `alpha_new.is_valid()`
on **every** hop, and the new code left that check only in `finalize()` — i.e. on the relaying branch. On the
final hop an identity-point alpha was therefore no longer rejected.

This is not cosmetic, though it is narrower than "an attacker can forge any packet". A Sphinx sender can
already build a MAC-valid `Final` packet for any node by using that node's public key with its own scalar, so
the ordinary case is not a forgery. What the identity alpha costs is **secrecy of that hop's shared secret**:
`s_k = identity · sk = identity` regardless of the node's private key, so the secret — and the header MAC
derived from it — is computable by a third party from public information alone, where normally only the
recipient can compute it. The recipient's public key is still the KDF salt, but the salt is public, so it does
not restore secrecy. Opus reproduced the gap end-to-end and showed the secret is key-independent; Sonnet
independently located the same gap and its replay-tag commit-timing side effect on the relay hop.

**Fix:** reject the degenerate alpha on the **public input** (`alpha_point.is_valid()`) in the first half of the
transform, so every hop rejects it before the private-scalar multiplication. Because `is_valid` for the Edwards
and Montgomery suites now excludes all small-order points — the identity _and_ the order-2/4/8 classes, which
the old identity-only check let through — this is slightly stricter than the pre-deferral check, which is the
intent: a small-order alpha makes `alpha · sk` independent of `sk` up to sign for any clamped scalar, so it has
the same secrecy loss as the identity. The secp256k1 case, where a zero blinding scalar was previously rejected
only through the identity check, is preserved by parsing the blinding scalar in the first half.

Regression tests: `a_degenerate_alpha_is_rejected_on_the_deferred_path` (identity; fails without the guard) and
`a_nonidentity_small_order_alpha_is_rejected` (all eight torsion classes). Both verified to fail without the
guard and pass with it.

**Guard cost, with a measured regression.** The guard's first implementation used `is_torsion_free`, which
multiplies by the group order. Measured on the then-head against `ff0076efaa`, that pushed `packet_forwarding`
from 76.9 to **108.9 µs (+41%)**; the relay path pays the check on every packet, so a full scalar
multiplication there swamps the deferral. Switching to `is_small_order` (three doublings; it rejects exactly
the eight torsion points, identity included) brought forwarding back to 75.57 µs and receiving to 37.30 µs —
the numbers in the table. That is why the final head, not `2d718ac599`, is what this document reports.

### Also corrected on their reports

- `forward_secret`'s doc comment still said it returns "the blinded alpha"; it now returns it deferred.
- The in-tree "eager vs deferred" assertion was **weaker than its comment claimed**: `forward_transform` now
  delegates to `forward_transform_deferred`, so the test compares the deferral against itself. The comment now
  says so, and names the parent-commit A/B as the actual equivalence evidence.

## Test evidence

- `cargo nextest run -p hopr-crypto-packet -p hopr-protocol-hopr --lib` → **267 passed, 0 failed** (266 before
  the degenerate-alpha regression test).
- `cargo nextest run -p hopr-transport --features testing --test '*' -j 1` → **61 passed, 0 failed**
  (includes `protocol_stress_throughput`, `protocol_msg_ack_workflows`, `pipeline_stage_isolation`).
- `cargo clippy -p hopr-crypto-packet --all-targets` and `cargo fmt --check`: clean.
- New coverage: the suite test compares deferred vs eager secret and alpha for every hop; and
  `a_degenerate_alpha_is_rejected_on_the_deferred_path` pins the identity-alpha rejection.

## Caveats / things a reviewer should probe

- Associativity argument for item 3 assumes `G::generate` is exactly `scalar · G` for all suites
  (Ed25519 used by `HoprSphinxSuite`; secp256k1 and x25519 suites also implement `SphinxSuite`).
- `DeferredAlpha` carries the raw `b_k` blended into the secret's derivation; confirm the pairing
  (secret ↔ tag ↔ alpha) cannot be crossed by a caller. `ForwardSecret`'s fields stay private and the
  only external consumer is `.tag()`.
- Constant-time: `to_alpha` is now called once instead of twice — same operation, fewer times; no new
  branching on secret data. `finalize()` is on the relayed branch, as the multiply was before. The
  `is_valid()` guard restored in the first half is the same identity test the eager path ran.
- Pre-existing and unrelated: `cargo test --all-features` for `hopr-crypto-packet` fails to compile
  (ambiguous `X25519Suite` / `Secp256k1Suite` imports). Reproduced on clean `HEAD`.
- An `expand_path` lock-batching attempt was measured at no significant gain (p≈0.27) and reverted.
  Its first draft self-deadlocked (`return self.f(...)` evaluating while holding a read guard); caught
  by a bench hang, fixed, then reverted for lack of gain.
