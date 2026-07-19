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

### 2. Persistent retry cache for acks that hit `MissingVerifier`

Add a `pending_ack_keys` cache on `SsaReconstructor` that stores `HalfKeyChallenge → HalfKey`
pairs for acks that encountered `MissingVerifier`. This cache has a TTL matching
`max_ack_await_time` so stale entries eventually expire.

At the beginning of each `acknowledge_shares` call, drain this cache: for each entry, look up
the share in `awaiting_acks` (which still holds it thanks to the ordering fix), check if the
verifier now exists in `ssa_verifiers`, and if so, process it via `process_verified_ack`. On
success the share is consumed from `awaiting_acks` and the entry removed from
`pending_ack_keys`; on continued `MissingVerifier` the entry stays in the cache for the next
call.

```rust
// On SsaReconstructor:
pending_ack_keys: moka::sync::Cache<HalfKeyChallenge, HalfKey>,

// In acknowledge_shares, before the main loop:
let mut pending_resolutions: Vec<ShareResolution<...>> = Vec::new();
self.pending_ack_keys.iter().for_each(|entry| {
    let (challenge, ack) = entry.pair();
    if let Some(peer_cache) = self.awaiting_acks.get(&peer_key) {
        if peer_cache.contains_key(challenge) {
            if let Ok(spi_entry) = peer_cache.get(challenge) {
                if let Ok(spi) = spi_entry.ssa_polynomial_id() {
                    if self.ssa_verifiers.contains_key(&spi) {
                        match self.process_verified_ack(*ack, *challenge, &peer_cache) {
                            Ok(result) => {
                                self.pending_ack_keys.invalidate(challenge);
                                // collect into pending_resolutions
                            }
                            Err(PixError::MissingVerifier) => { /* leave in cache */ }
                            Err(other) => {
                                self.pending_ack_keys.invalidate(challenge);
                                tracing::error!(...);
                            }
                        }
                    }
                }
            }
        }
    }
});
```

**Why this is better than a Vec:**

- The cache lives across `acknowledge_shares` calls — if the verifier arrives in between,
  the next invocation picks up the pending ack.
- TTL-based expiry prevents unbounded growth if the verifier never arrives.
- No changes needed to `insert_coefficient_commitments` — it only needs to insert verifiers
  into `ssa_verifiers` as it already does.
- Combined with the ordering fix, a share can survive multiple `MissingVerifier` hits across
  separate batches of acks before eventually being processed or expiring.

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
acknowledge_shares called (call #1)
  → drain pending_ack_keys cache — any leftovers from previous call
  → main loop over incoming acks
    → process_verified_ack
      → .get() keeps share in awaiting_acks
      → verifier lookup: MissingVerifier!
      → insert (challenge, ack) into pending_ack_keys, share stays in awaiting_acks
  ↓ (call returns, pending_ack_keys persists on SsaReconstructor)
  ↓ (time passes — insert_coefficient_commitments inserts the verifier)
acknowledge_shares called (call #2)
  → drain pending_ack_keys cache
    → (challenge, ack) found in pending_ack_keys
    → awaiting_acks still has the share (.get(), never removed)
    → ssa_verifiers has the verifier now
    → process_verified_ack → .remove() from awaiting_acks → decrypt → add share
    → invalidate from pending_ack_keys
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

| File                                     | Changes                                                                                                                                                                                                                                |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `protocols/pix/src/reconstructor/mod.rs` | Add `pending_ack_keys` cache field to `SsaReconstructor`; `process_verified_ack`: `.get()` → `.remove()` ordering; `acknowledge_shares`: drain `pending_ack_keys` before main loop, insert on `MissingVerifier`, downgrade to `trace!` |
