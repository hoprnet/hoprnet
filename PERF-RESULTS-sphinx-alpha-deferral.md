# Perf results: Sphinx alpha deferral & single point encoding

- **Branch:** `perf/sphinx-defer-alpha-and-single-compress` (PR #8700)
- **Benchmarked fixed revision:** `2d718ac599` (includes the degenerate-alpha guard)
- **Later commits (review follow-up, not separately benchmarked):** `4c302b614d`…`d88904384c` — the guard,
  the small-order `is_valid` fix, and doc/test corrections. None of them touches a timed hot path except by
  adding the `is_identity`/`is_torsion_free` check, whose cost is estimated below.
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

Isolated A/B: baseline built from `ff0076efaa` in a separate git worktree with its own
`CARGO_TARGET_DIR`; the **fixed build is `2d718ac599`**, which already contains the degenerate-alpha guard.
Both binaries run back-to-back on a quiet box, one process at a time (an earlier attempt ran baseline and
fixed concurrently and produced contaminated, noisy numbers — those were discarded). Criterion defaults
otherwise; `--sample-size 30–60`. The `change:` intervals are criterion's, with the `p` value quoted per
row; every row except the `pix_on` one is `p < 0.001`. Δ is criterion's `change` estimate (median of the
two sample distributions) against the saved baseline, not a recomputation from the two means in the row.

The exact commands, the baseline/fixed revisions, and the machine (AMD EPYC 9575F, 16 vCPU, Linux 7.0.0)
are recorded here; the raw Criterion samples live under the untracked `target/criterion/` of each build and
were not archived — re-running the two binaries reproduces them.

## Results (isolated A/B)

The MiB/s figures are **normalized to the maximum payload capacity** (`HoprPacket::PAYLOAD_SIZE` per
packet), which is what the benchmark's `Throughput` declares; the fixtures carry 300 application bytes
per packet, so the application-level rate is lower — for the 5,000-packet e2e run, 1.5 MB in 184.15 ms
≈ 7.8 MiB/s of application data. The relative speedups are unaffected by the normalization.

| Benchmark                                                | Before                  | After                   | Δ                         |
| -------------------------------------------------------- | ----------------------- | ----------------------- | ------------------------- |
| `hopr_decoder/recipient` (final hop)                     | 155.45 µs               | 135.47 µs               | **−13.3%**                |
| `hopr_decoder/ack_recipient`                             | 158.64 µs               | 132.77 µs               | **−16.2%**                |
| `hopr_decoder/relay_replay` (rejected replay)            | 45.69 µs                | 24.25 µs                | **−47.0%**                |
| `hopr_decoder/relay`                                     | 219.66 µs               | 216.35 µs               | −1.8%                     |
| `hopr_encoder/3hop_2surbs`                               | 391.40 µs               | 348.73 µs               | **−10.9%**                |
| `packet_receiving/any_hop`                               | 58.640 µs               | 36.918 µs               | **−37.0%**                |
| `packet_sending_no_precomputation/3_hop_2_surbs/pix_off` | 400.69 µs               | 365.48 µs               | **−8.8%**                 |
| `packet_forwarding/any_hop`                              | 76.856 µs               | 74.869 µs               | −2.6%                     |
| `pipeline_e2e_forward/3hop_5000pkt`                      | 211.46 ms (73.2 MiB/s)  | 184.15 ms (84.1 MiB/s)  | **−12.9% (+14.8% thrpt)** |
| `packet_precompute/3_hop_2_surbs/pix_off`                | 353.45 µs               | 319.01 µs               | **−9.8%**                 |
| `protocol_throughput_pipeline/63.4 MiB/pix_off`          | 402.35 ms (157.6 MiB/s) | 379.84 ms (166.9 MiB/s) | **−5.6% (+5.9% thrpt)**   |
| `protocol_throughput_pipeline/63.4 MiB/pix_on`           | 416.39 ms (152.3 MiB/s) | 415.63 ms (152.5 MiB/s) | −0.2% (n.s., p=0.88)      |

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

**Cost of the guard.** The table above is measured on a build that already contains the guard
(`2d718ac599`), so it is the net figure, not the pre-guard one. A separate run of the _same_ build with the
guard then added back in (an intermediate revision) read `packet_forwarding` 74.87 → 75.23 µs (+0.5%) and
`packet_receiving` 36.92 → 36.98 µs (+0.5%); that is a run-to-run comparison on one revision boundary, not
a third isolated A/B, and the +0.5% is at the edge of the noise it was taken over.

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
