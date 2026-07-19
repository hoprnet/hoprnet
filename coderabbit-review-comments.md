# PR #8095 — Truly Unresolved CodeRabbit Review Comments

> **Branch:** `lukas/pix` — **PR:** [#8095](https://github.com/hoprnet/hoprnet/pull/8095)
>
> Comments that have been verified as still applicable against current code.
> Resolved/withdrawn comments are omitted entirely.

---

## `crypto/packet/src/packet.rs`

| #   | Line(s)                   | Severity   | Comment                                                                                                                                                                                                                                                                    |
| --- | ------------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | 145-156, 467-478, 526-535 | ~~🟠 Major~~ | **Fixed.** Added TODO comment documenting the intentionality. The EntryShareGenerator emits surplus shares to absorb such packet-loss events, so the budget impact is bounded and expected. |
| 2   | 746-789                   | 🔵 Trivial | **No test exercising non-empty PIX share path.** `create_packet` helper creates a generator but never calls `new_ssa_commitment`, so `next_share` always returns `None`. The `Some(enc_share)` encryption/decryption path is untested.                                     |

## `protocols/pix/src/reconstructor/mod.rs`

| #   | Line(s) | Severity | Comment                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| --- | ------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 3   | 135-149 | 🟠 Major | **Share removed from cache before verifier confirmed.** `awaiting_ack_from_peer.remove()` (line 141) consumes the ciphertext before `ssa_verifiers.get()` (line 148) confirms the verifier exists. On `MissingVerifier` the share is permanently lost. This is plausible: acks from SURB packets can arrive before `SsaCommit` messages finish processing. Fix requires both deferred removal and draining parked shares when verifier is inserted. See `plans/fix-share-removed-before-verifier.md`. |

## `protocols/start/src/lib.rs`

| #   | Line(s) | Severity   | Comment                                                                                                                                                                                                                |
| --- | ------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 4   | 188-190 | 🔵 Trivial | **Commitments-per-message calculation uses `size_of::<G>()`.** Functionally equivalent to `PIX_COEFF_COMMITMENT_REPR_SIZE` in practice but should use the named constant for consistency with the encode/decode paths. |

## `protocols/hopr/src/codec/mod.rs`

| #   | Line(s) | Severity | Comment                                                                                                                                                                                                   |
| --- | ------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 5   | 598     | 🟡 Minor | **No assertion that gap packets contain no PIX share.** `no_share_indices` is populated from `pos_in_segment`, not from inspecting `out_packet.encrypted_pix_share`. Missing encoder-contract validation. |

## `transport/session/src/test_helpers.rs`

| #   | Line(s) | Severity | Comment                                                                                                                                                                                                                                                                        |
| --- | ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 6   | 210-249 | 🟠 Major | **`AtLeast` selection still gates on `call_count >= *n` instead of matcher.** Has `matched_calls` and drop-time validation, but `AtLeast(n)` still has no eligible expectation for its first n-1 calls.                                                                        |
| 7   | 326-341 | 🟠 Major | **Dispatch failures hidden inside spawned task.** `mock_packet_planning` returns `JoinHandle<()>`, not `JoinHandle<Result<()>>`. `send_message` failures are swallowed inside the task via `.expect()`. CodeRabbit explicitly rejected the "documentation is enough" argument. |

## `transport/session/src/manager.rs`

| #   | Line(s)   | Severity | Comment                                                                                                                                                                                                                                                                                                  |
| --- | --------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 8   | 179-195   | 🟠 Major | **Verification failures tracked globally, not per-`SsaIndex`.** `SessionSsaState::increment_errors()` uses a single `num_errors` atomic. `SsaAlmostRecovered` resets the counter while the current SSA still has shares in flight. Late failures become attributed to the next SSA cycle.                |
| 9   | 1500-1575 | 🟠 Major | **Kill-switch armed after SSA request is sent.** The SSA request (containing `ssa_index` and `exit_commitment`) is sent at line 1547 before the deposit-timeout kill switch is registered at line 1557. If the message is sent but the kill-switch setup fails, there's a window with no enforcement.    |
| 10  | 2466-2486 | 🟠 Major | **PIX dimensions compared by product only.** `quota_per_ssa()` comparison via `pix_params_to_quota` means different parameter pairs with the same product pass (e.g. `(2,6)` and `(3,4)`). The `_negotiated_polys` and `_negotiated_shares` are extracted but prefixed with `_` — not actually compared. |

## `transport/hopr/src/lib.rs`

| #   | Line(s)                   | Severity | Comment                                                                                                                                                                                                                                                                                                                                 |
| --- | ------------------------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 11  | 432-452                   | 🟠 Major | **Unused `A` generic in `run_exit`.** `ExitAcknowledgementShareProcessor<HoprPixSpec>` is declared and trait-bounded but unused in function body. Forces all callers to specify via turbofish.                                                                                                                                          |
| 12  | 833-848, 940-942, 978-980 | 🟠 Major | **PIX event `.forward()` silently fails on sink error.** All three role branches consume PIX events through `forward` whose future resolves to `Result`. `spawn_as_abortable!` doesn't handle the error, so any downstream sink failure silently terminates the task, permanently dropping all subsequent PIX events. Tracked in #8236. |

## `transport/hopr/src/protocol/pipeline/mod.rs`

| #   | Line(s) | Severity | Comment                                                                                                                                                                                                                                                                                                       |
| --- | ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 13  | 909-916 | 🟠 Major | **Dual-role Relay+Exit nodes don't register outgoing PIX shares.** `(node_type == NodeType::Exit).then(\|\| exit_ack_proc.clone())` at line 915 doesn't cover the Relay+Exit case. The Relay+Exit branch (line 866) installs a real reconstructor but the outgoing pipeline never receives the ack processor. |

## `impls/strategy/src/non_anonymous_pix.rs`

| #   | Line(s) | Severity     | Comment                                                                                                                                     |
| --- | ------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 14  | 44-47   | ~~🟠 Major~~ | **Fixed.** Field renamed to `pix_recovery_password_env` (env var name, not the password). Password resolved at build time from the environment — never in config output.                                                               |
| 15  | 71-83   | ~~🟠 Major~~ | **Fixed.** `build()` returns `Result`; `assert!` replaced with early `Err`; store open error propagated instead of `.expect()`.             |
| 16  | 112-146 | ~~🟠 Major~~ | **Fixed.** `processed_deposits.insert()` moved to after the withdrawal succeeds.                                                            |
| 17  | 175-194 | ~~🟡 Minor~~ | **Fixed.** `if let Some(balance) = immediate { ... } else { stream.try_next() ... }` — stream never polled when immediate balance suffices. |
| 18  | 211-244 | ~~🟠 Major~~ | **Fixed.** Added `PixRecoveryStore::remove()`, called after successful withdrawal.                                                          |

## `hopr/hopr-lib/src/builder.rs`

| #   | Line(s) | Severity | Comment                                                                                                                                                                                                                                                  |
| --- | ------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 19  | 433-435 | 🟠 Major | **`set_overflow(true)` on SSA broadcast channel can silently drop lifecycle events.** When subscriber lags, overflow evicts old messages. Losing `PrivateKeyRecovered` or `DepositAddressReceived` can strand deposit/withdrawal processing permanently. |

## `hopr/hopr-lib/src/testing/fixtures.rs`

| #   | Line(s) | Severity | Comment                                                                                                                                                                                                                         |
| --- | ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 20  | 743-890 | 🟠 Major | **`JoinHandle::join` blocks async executor; no timeout on readiness waits.** CodeRabbit explicitly rejected the "join is fine" justification in their follow-up. A stuck node can hang the test suite.                          |
| 21  | 924-962 | 🟠 Major | **`wait_for_connectivity` doesn't wait for routing metrics.** Only checks `connected_peers()` count via P2P connectivity, not whether probe/graph telemetry is ready. Tests that assume routing works immediately can be flaky. |

---

## Critical Unresolved Issues

1. **`reconstructor/mod.rs:135-149`** — Share permanently lost when verifier isn't ready yet (plan exists)
2. **`manager.rs:2466`** — PIX dimensions compared by product only; unrecoverable session on parameter collision
3. **`transport/hopr/src/lib.rs:833-848`** — PIX events silently dropped on sink error (tracked in #8236)
4. **`test_helpers.rs:326-341`** — Dispatch failures hidden in test mocks
5. **`hopr/hopr-lib/src/builder.rs:433-435`** — Lossy broadcast drops lifecycle events
6. **`pipeline/mod.rs:909-916`** — Dual-role Relay+Exit loses outgoing PIX shares
7. **`manager.rs:179-195`** — Per-SSAIndex error tracking missing
