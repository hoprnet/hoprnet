# PR #8095 — CodeRabbit Review Comments Status

> **Branch:** `lukas/pix` — **PR:** [#8095](https://github.com/hoprnet/hoprnet/pull/8095)

## Status Summary

- **50 total review threads** on GitHub
- **46 resolved** on GitHub (resolved via code changes, documentation, or issue tracking)
- **4 still unresolved** on GitHub (see below)

---

## Unresolved on GitHub (4 threads)

| # | File | Line | Severity | Description | Status |
|---|------|------|----------|-------------|--------|
| 1 | `hopr/hopr-lib/src/testing/fixtures.rs` | 722 | 🟠 Major | **Move env var override out of runtime setup.** `set_var` after tokio runtime started violates safety precondition. | **Fixed.** Moved to module-level `std::sync::Once` that fires before any async context. Both `cluster_fixture` and `build_role_cluster` use it. |
| 2 | `impls/strategy/src/strategy.rs` | 73 | 🟠 Major | **Discard strategies on error.** Without `runtime-tokio`, the drain drops all sub-strategies then reports success. | **Outdated.** The silent-drain `#[cfg(not(feature = "runtime-tokio"))]` path no longer exists in the code. Current `run()` always spawns sub-strategies — removing `runtime-tokio` would be a compile error, not a silent success. |
| 3 | `protocols/pix/src/reconstructor/mod.rs` | 124 | 🟠 Major | **Bound caches by global protocol state, not per-SSA constants.** At max polynomial count, `ssa_verifiers` retains only four complete SSA sets. | Still open |
| 4 | `transport/session/src/test_helpers.rs` | 341 | 🟠 Major | **Return dispatch failures, don't hide them.** `mock_packet_planning` returns `JoinHandle<()>`, swallowing send failures. CodeRabbit rejected the "documented" justification. | **Fixed.** `mock_packet_planning` now returns `JoinHandle<anyhow::Result<()>>` and propagates errors via `?`. All 20+ callers updated to `handle.await??`. |

---

## Resolved on GitHub (46 threads)

All other CodeRabbit threads on the PR have been marked as resolved. This includes the items we fixed in this session's commits:

- Password serialization → env var reference (`non_anonymous_pix.rs`)
- Builder returns `Result` instead of `assert!` (`non_anonymous_pix.rs`)
- Deposit dedup — `insert` after withdrawal (`non_anonymous_pix.rs`)
- Stream poll — no double poll on immediate balance (`non_anonymous_pix.rs`)
- Recovery key cleanup — `store.remove()` after withdrawal (`non_anonymous_pix.rs`)
- Gap assertion false positive (`codec/mod.rs`)
- Share consumption documentation (`packet.rs`)
- Error tracking documented as intentional (`manager.rs:195`)
- PIX dimensions — product-only comparison documented (`manager.rs:2466`)
- Named constant `PIX_COEFF_COMMITMENT_REPR_SIZE` (`start/src/lib.rs`)
- Unused `A` generic removed from `run_exit` (`transport/hopr/src/lib.rs`)

Resolved through issues or documentation:
- PIX events silently dropped — tracked in #8236 (`transport/hopr/src/lib.rs:848`)
- Lossy broadcast — acknowledged (`builder.rs:435`)
- `JoinHandle::join` — documented (`fixtures.rs:890`)

---

## Additional Issues Not Tracked on GitHub

These were identified during review but don't have a corresponding CodeRabbit thread (or the thread was about something else):

| # | File | Line(s) | Description |
|---|------|---------|-------------|
| A | `protocols/pix/src/reconstructor/mod.rs` | 135-149 | Share removed from cache before verifier confirmed. `awaiting_ack_from_peer.remove()` consumes ciphertext before `ssa_verifiers.get()` confirms verifier exists. On `MissingVerifier` the share is permanently lost. Fix plan exists in `plans/fix-share-removed-before-verifier.md`. |
| B | `transport/hopr/src/protocol/pipeline/mod.rs` | 909-916 | Dual-role Relay+Exit nodes don't register outgoing PIX shares. `(node_type == NodeType::Exit).then(\|\| exit_ack_proc.clone())` at line 915 misses the Relay+Exit case. |
| C | `transport/session/src/manager.rs` | 1500-1575 | Kill-switch armed after SSA request is sent. SSA request at line ~1547 before deposit-timeout kill switch at line ~1557 — window with no enforcement if setup fails. |
