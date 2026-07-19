# Plan: Prevent permanent share loss when verifier is not yet available

## Problem

In `SsaReconstructor::process_verified_ack` (reconstructor/mod.rs:135-199), the encrypted
share is **removed** from the `awaiting_ack_from_peer` cache **before** the verifier lookup:

```
awaiting_ack_from_peer.remove(&ack_challenge)   ← shares leaves cache (line 141)
    ↓
ssa_polynomial_id()                              ← read from removed share (line 146)
    ↓
ssa_verifiers.get(&spi)                          ← may return MissingVerifier (line 148)
```

When the verifier returns `MissingVerifier`, the function errors out but the share is already
permanently gone from the cache. Since acknowledgements are fire-and-forget (never
retransmitted), the share is **unrecoverable**.

This is plausible in practice: the Exit starts processing SURB acks immediately while
`SsaCommit` messages may still be arriving and being processed. The verifier simply hasn't
been inserted into `ssa_verifiers` yet.

## Two sub-problems

1. **Ordering bug** — share is removed before the verifier is confirmed present.
2. **No re-processing hook** — even if the share stays in the cache, nothing re-checks
   parked shares when a verifier is eventually inserted via `insert_coefficient_commitments`.

## Proposed fix

### 1. Fix the ordering: defer removal until after verifier is confirmed

In `process_verified_ack`, use `.get()` instead of `.remove()`, then explicitly remove
only after the verifier is confirmed present:

```rust
// Before:
let Some(share) = awaiting_ack_from_peer.remove(&ack_challenge) else { ... };

// After:
let Some(share) = awaiting_ack_from_peer.get(&ack_challenge) else { ... };
let spi = share.ssa_polynomial_id().ok_or(PixError::ShareIsEmpty)?;
let reconstructor = self.ssa_verifiers.get(&spi).ok_or(PixError::MissingVerifier)?;
// Verifier confirmed — safe to remove:
awaiting_ack_from_peer.remove(&ack_challenge);
```

On `MissingVerifier`, the share stays in `awaiting_ack_from_peer` and can be picked up on
the next call to `acknowledge_shares` (same ack arrives again) or when the verifier
insertion hooks into the awaiting-ack cache (see step 2).

### 2. Drain parked shares when verifiers are inserted

In `insert_coefficient_commitments`, when `CommitmentResult::Completed(...)` produces new
`ssa_reconstructors`, also scan `self.awaiting_acks` for shares whose
`ssa_polynomial_id()` matches the newly inserted verifiers, and **synchronously process
them**.

Pseudo-code (inserted at `reconstructor/mod.rs:284`, after inserting verifiers):

```rust
for ssa_reconstructor in &ssa_reconstructors {
    let spi = ssa_reconstructor.verifier.spi;
    self.ssa_verifiers.insert(spi, Arc::new(Mutex::new(ssa_reconstructor)));

    // Drain any shares that arrived before this verifier was ready.
    // Iterate all peers' inner caches to find shares matching this SPI.
    let mut to_process: Vec<(OffchainPublicKey, HalfKeyChallenge)> = Vec::new();
    for peer_list in self.awaiting_acks.iter() {
        let (peer, inner) = peer_list.pair();
        for entry in inner.iter() {
            let (challenge, share) = entry.pair();
            if share.ssa_polynomial_id() == Some(spi) {
                to_process.push((*peer, *challenge));
            }
        }
    }
    // Process each parked share inline (decrypt, add to verifier).
    for (peer, challenge) in to_process {
        // Remove from inner cache, decrypt, add to verifier...
        // Returns any ShareResolution produced.
    }
}
```

**Implementation notes:**

- The inner `EncryptedShareCache` supports iteration via `moka::sync::Cache::iter()`.
- The iteration happens _after_ the verifier is in `ssa_verifiers`, so `process_verified_ack`
  can be reused (delegated to for each parked share).
- The result of processing a parked share (e.g. `FullRecovery`, `AlmostRecoveredSsa`) can
  be appended to the `res: SsaCommitmentState` returned by `insert_coefficient_commitments`,
  or returned via a new field. The caller (`acknowledge_shares` or the `insert_coefficient_commitments`
  caller) needs to handle any newly discovered SSA resolutions.

### 3. Adjust error handling in `acknowledge_shares`

`MissingVerifier` is no longer a permanent error — it just means "try again later" (either
the same ack will be re-dispatched on the next `acknowledge_shares` call, or the verifier
insertion will process it). Adjust the error handling around `reconstructor/mod.rs:357`:

```rust
Err(PixError::MissingVerifier) => {
    tracing::trace!(%peer, "verifier not yet available, share retained in cache");
    // Not an error — share is still in awaiting_acks and will be
    // processed when the verifier arrives.
}
```

This downgrades from `tracing::error!()` (which makes it look like a bug) to `tracing::trace!()`.

### What share lifetime looks like after the fix

```
ack arrives via acknowledge_shares
  → process_verified_ack called
    → .get() keeps share in awaiting_acks
    → verifier lookup: MissingVerifier!
    → return error (share still in cache)
    ↓ (time passes)
SsaCommit arrives → insert_coefficient_commitments
  → CommitmentResult::Completed
    → verifiers inserted into ssa_verifiers
    → self.awaiting_acks scanned for matching SPI
    → matching share found → decrypted → added to verifier
    → FullRecovery or EarlyRecovery emitted
```

### Out of scope for this plan

- The nested cache architecture (`Cache<OffchainPublicKey, Cache<HalfKeyChallenge, ...>>`)
  adds complexity to the drain iteration. A flat `Cache<(OffchainPublicKey, HalfKeyChallenge), ...>`
  would simplify future designs but changing it now is a larger refactor than warranted.
- Per-`SsaIndex` error tracking in `SessionManager` (separate CodeRabbit finding).

## Acceptance criteria

- The regression test `reconstructor_missing_verifier_destroys_share` must pass (asserts share survives `MissingVerifier`).
- Pre-existing tests must continue to pass: `cargo test -p hopr-protocol-pix --lib`
- No changes to public API types unless the drain-on-insert step requires it.

## Files to modify

| File                                                   | Changes                                                                                                                                                                                    |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `protocols/pix/src/reconstructor/mod.rs`               | `process_verified_ack`: `.get()` → `.remove()` ordering; `insert_coefficient_commitments`: drain parked shares on `Completed`; `acknowledge_shares`: downgrade `MissingVerifier` log level |
| `protocols/pix/src/reconstructor/mod.rs` (return type) | May need to extend `SsaCommitmentState` or the return of `insert_coefficient_commitments` to surface extra `ShareResolution`s from drained shares                                          |
