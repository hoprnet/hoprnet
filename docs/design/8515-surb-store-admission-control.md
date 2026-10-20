# Implementation Plan: Admission-gated two-class SURB store with a Session-lifecycle admit/retire seam (issue #8515)

> **Restructure note (supersedes v1's two-independent-caches shape).** Four critics converged on two structural defects in v1 that cannot be patched by policy: (a) a *separate* admission-marker cache that `insert_surbs` routes on while the ring lives in a *different* cache produces a marker/ring desync and an admit-vs-insert race (Critic 2/4), and (b) a global budget that *refuses admitted inserts* drops live PIX shares, contradicting the headline guarantee (Critic 2 blocker, Critic 3 major, Critic 4 blocker). The plan below **removes the marker cache** (the admitted ring cache is the single source of truth), **moves admission to the established-Session point** (`SessionSlotGuard::commit`, not `allocate_session_slot`'s `Inserted` arm), and **replaces the "refuse admitted inserts" budget with a structural admitted bound plus a quarantine-only counter**. Every other critic item is folded in or resolved in the appendix.

---

## Problem restatement (mechanism-first)

Every `HoprPacket::Final` carrying SURBs reaches `HoprDecoder::decode` → `SurbStore::insert_surbs(incoming.sender, incoming.surbs)` (`protocols/hopr/src/codec/decoder.rs:252`) with **no admission check**: the `Final` arm never verifies a ticket, and `insert_surbs` mints a `SurbRingBuffer` for whatever `HoprPseudonym` the sender chose (`surb_store.rs:696-701`). That ring lives in a single LRU cache `surbs_per_pseudonym` bounded by `max_pseudonyms.max(MINIMUM_SURBS_PER_PSEUDONYM)` entries (`surb_store.rs:545-553`, floor 1000 at `surb_store.rs:24`/`:552`), so ~10 000 fresh pseudonyms can LRU-evict a live Session's ring before its 600 s `pseudonyms_lifetime` (`surb_store.rs:68-70`), permanently losing every PIX share those SURBs carried (a share reaches the reconstructor only when its SURB is *used*; documented `protocols/hopr/src/types.rs:36-68`). The same single-tier design has no global bound: `max_pseudonyms` × `rb_capacity` × ~400 B is ~400 GB with defaults (`surb_store.rs:52-66`). The only counter-like state is `EvictionStats` (`surb_store.rs:388`).

`decoder.rs:252` is the **only** production `insert_surbs` call site (all other hits are tests/benches), so gating there covers all receive-side ingress. The sending-side `insert_reply_opener` (`protocols/hopr/src/codec/encoder.rs:122`) is out of scope — see Residual risk.

---

## Design summary

Split the receive-side store into **two physically separate moka caches**: a small **quarantine** cache that any `Final` packet may write into, holding un-admitted pseudonyms at a tiny per-ring cap and short TTI; and an **admitted** cache entered **only** through `SurbStore::admit(pseudonym, expected_surbs)`, called by `SessionManager` at the point a Session is **established** (`SessionSlotGuard::commit`, `transport/session/src/manager.rs:1182`), and left through `SurbStore::retire(pseudonym)` on teardown. Because the caches are distinct objects, quarantine churn can never evict an admitted ring — a structural guarantee, not a policy.

**The admitted cache is the single source of truth for admission** (v1's separate marker cache is deleted): `admit` inserts an *empty* ring so standing is known before any SURB arrives, and `insert_surbs` routes by `admitted_surbs_per_pseudonym.get(&pseudonym)` and pushes into the ring it finds. This removes the marker/ring desync and the marker-Size-eviction failure mode entirely.

**Memory bounds are split by trust class, enforced at the correct point:**
- **Quarantine** (attacker-controlled, unauthenticated) is bounded three ways: per-ring cap (`quarantine_rb_capacity`, default 16 ≥ one packet, `MAX_SURBS_IN_PACKET = 8` at `crypto/packet/src/packet.rs:503`), entry count (`quarantine_max_pseudonyms`), and a `quarantine_surb_budget` counter drained at insert time (evict quarantine, then refuse). Default ceiling ≈ 1024 × 16 × ~400 B ≈ 6.4 MB.
- **Admitted** (requires an established Session) is bounded **structurally**, not by a per-insert counter: `admitted_max_pseudonyms` (default 10 000 = `maximum_sessions` default, `manager.rs:1701`) entries × per-ring cap `min(rb_capacity, expected × (1 + headroom))`. Total ≤ `admitted_max_pseudonyms × rb_capacity`, the operator's own configured ceiling, which no unauthenticated peer can inflate. **An admitted insert is never refused and an admitted ring is never evicted to satisfy a counter** — so no live share is ever dropped by the budget (resolves the Critic 2/3/4 blockers).

`find_surb` consults admitted then quarantine (probe replies at `transport/probe/src/probe.rs:94` still resolve) and **does not promote** (promotion on `find_surb` would be attacker-reachable via `DestinationRouting::Return(pseudonym)`).

```
   Final packet ──► decoder.rs:252 insert_surbs(pseudonym, surbs)
                          │
            admitted_surbs_per_pseudonym.get(&pseudonym)?         ◄── admit(p, expected) [at Session commit]
            │                                              ▲                retire(p) [at Session teardown]
     yes ──► admitted ring, cap = min(rb_capacity,        │
             expected*(1+headroom)) floored at quarantine │  promotion: with_capacity deep-copy
             │                                             │  of any quarantine ring (budget net-zero)
     no  ──► quarantine_surbs: cap 16, TTI 60s, LRU ──────┘
             │
             └─► quarantine_surb_budget (AtomicUsize): drain quarantine (arbitrary order), then refuse
```

---

## The admission token (why `admit`/`retire` carry a number)

A pseudonym is client-chosen and freely reusable, so two Sessions can be established on the same one, and the admitted cache holds at most one ring per pseudonym. A teardown does not retire its ring immediately: it defers the retire past the packet pipeline's SURB-resolution window, because the closing Session's own return-routed notification — the PIX-failure notice is the one such packet — still needs its ring while the resolution stage comes to look it up. Retiring inline destroyed that ring mid-lookup, the packet was dropped, and the Entry never learned the Session was gone (`deposit_timeout_closes_session` fails exactly there).

The deferral opens a window in which a *successor* Session can be admitted on the same pseudonym before the first teardown's retire fires. An unconditional retire landing then would invalidate the successor's live ring, and it would go on inserting its SURBs into the 16-cap quarantine with nothing logged anywhere — worse than the failure being fixed. The token closes that window:

- **`AdmissionToken`** (`transport/session/src/types.rs`) is the opaque number naming one admission. `admit` mints a fresh one on every call — a process-wide counter (`ADMISSION_EPOCHS` in `surb_store.rs`), so an `admit` need not look up the pseudonym's previous number — and the token rides on the admitted cache entry next to the ring. The session side threads it from `commit` (which stores it in the slot's cell) to the teardown, and hands it to `retire`.
- **The epoch-0 sentinel.** `0` is never minted: it is the session side's `AdmissionToken::NONE`, what a teardown of a never-committed Session passes through the seam. `MemorySurbStore` starts `ADMISSION_EPOCHS` at `1` and `retire` returns early on `0`, so such a retire matches nothing. `SurbStore::admit` documents the rule for any second implementation.
- **The deferred retire.** `release_session_slot` schedules the retire on a spawned task after `surb_retire_grace(surb_resolution_wait)` (the pipeline window plus a one-second margin), falling back to an inline retire when no runtime is running — a teardown with no runtime has no pipeline resolving anything on its behalf either, so there is nothing to wait out. No teardown path needs its retire suppressed: a stale retire is a no-op at the store, never a lost one.
- **The token-matched removal.** `retire` is not an `invalidate`; it is an atomic compare-and-remove (`entry_by_ref(...).and_compute_with`), which removes the entry only when its stored epoch still equals the token presented. The key-level lock is held across the compare and the removal, so a successor `admit` cannot interleave between them; a retire whose token has moved on is a `Nop`, leaving the successor's ring and its number untouched.

The admitted cache carries **no `time_to_idle`**. Its entry count is already bounded by `admitted_max_pseudonyms`, and every teardown retires its entry, so a ring's lifetime is exactly its admission's lifetime — the invariant the token exists to preserve, which a TTI could only violate. A Session's idle timeout is unbounded above and independent of `pseudonyms_lifetime`, so a ring reclaimed by time while its Session lives would silently fall back to the quarantine and drop that Session's later SURBs and PIX shares, and `ping_session` does not re-admit. Only `admit` may create an admitted entry for the same reason: an `insert_surbs` that recreated one after a racing retire would leave a ring no teardown removes, at a floor capacity a successor's re-stamp would not raise.

---

## Crate-boundary decision

`hopr-transport-session` **cannot** name `hopr_protocol_hopr::SurbStore` — `transport/session/Cargo.toml:84-96` depends on `hopr-protocol-app`/`hopr-protocol-pix`/`hopr-protocol-session`/`hopr-protocol-start`, not `hopr-protocol-hopr`; and `hopr-protocol-hopr` cannot name the session crate (`protocols/hopr/Cargo.toml:50-60`, no `hopr-transport*`). `hopr-transport` is the **only** crate that depends on both (`transport/hopr/Cargo.toml:141` `hopr-protocol-hopr`, `:145` `hopr-transport-session`), so it is the composition root.

- Declare a narrow trait **`SurbAdmissionControl`** in `transport/session/src/types.rs` (near `SessionAdmissionSink` at `types.rs:475`), naming only `SessionId`. `admit` returns an `AdmissionToken` naming the admission, and `retire` takes one back — see *The admission token* above for why the pair carries a token at all:
  ```rust
  pub trait SurbAdmissionControl: Send + Sync + 'static {
      fn admit(&self, pseudonym: &SessionId, expected_surbs: usize) -> AdmissionToken;
      fn retire(&self, pseudonym: &SessionId, token: AdmissionToken);
  }
  ```
  `SessionId` is a bare alias for `HoprPseudonym` (`transport/session/src/types.rs:402-406` re-exports it from `hopr_utils::network_types`; the doc block at `:400-406` states it), proven in-repo by `manager.rs:4662` `let session_id: SessionId = pseudonym;`. **`SessionId` is already re-exported** at `transport/session/src/lib.rs:54-58`, so add only `SurbAdmissionControl` to that existing `pub use types::{...}` list (do not re-add `SessionId`).
- `SessionManager<S>` holds `surb_admission: Arc<OnceLock<Arc<dyn SurbAdmissionControl>>>` next to `admission_tx` (`manager.rs:2161`), cloned in `Clone` (`manager.rs:2164-2180`), with `pub fn set_surb_admission(&self, ctl: Arc<dyn SurbAdmissionControl>) -> bool` (a `OnceLock::set`, mirroring `self.admission_tx.set(...)` at `manager.rs:2521`). **Do not** change `SessionManager::start`'s signature (`manager.rs:2474`) — 86 `.start(` call sites exist, ~15 of them tests.
- `hopr-transport` implements the adapter: `struct StoreSurbAdmission { store: MemorySurbStore }` delegating to `SurbStore::admit`/`retire`, built in `HoprTransport::new` where both the store (`transport/hopr/src/lib.rs:407`) and the `smgr` (`lib.rs:421-455`) exist. `hopr_transport::SurbStore` is already re-exported at `lib.rs:65`.
- `SurbStore` (`protocols/hopr/src/traits.rs:16`) gains `admit`/`retire` as **default no-op** methods, exactly like `invalidate_relayer` (`traits.rs:73`) / `revalidate_relayer` (`traits.rs:78`). This keeps the decoder's generic `Arc<S>` (`decoder.rs:19,46`) and every mock/bench compiling; the only in-tree impl is `MemorySurbStore` (`surb_store.rs:261`).

This mirrors the unmerged prior art `e90b7228ac` ("purge stored SURBs and reply openers on session close", on `origin/em/memory-leak-fable-fixes`), which threaded a `SessionCloseHook` through `close_session`/`SessionSlotGuard`; this plan reuses that seam shape.

---

## Work breakdown

### Phase 1 — Config surface (`protocols/hopr/src/surb_store.rs`)

1. Add floor consts beside `surb_store.rs:18-26`: `MINIMUM_QUARANTINE_PSEUDONYMS: usize = 64`, `MINIMUM_QUARANTINE_RB_CAPACITY: usize = 1`. Do **not** reuse `MINIMUM_SURBS_PER_PSEUDONYM` (1000, `:24`) — it floors the *admitted* cache at `:552`.
2. Add default fns beside `surb_store.rs:52-82` and `validate_quarantine_lifetime` beside `:28-34` (mirroring `validate_pseudonyms_lifetime`, floored at `MINIMUM_SURB_LIFETIME = 30 s`, `:19`).
3. Add six fields to `SurbStoreConfig` (`surb_store.rs:127`), each with the established `#[default(...)]` + `#[validate(...)]` + `#[cfg_attr(feature = "serde", serde(default = "..."))]` triple (style at `:151-154`). All are `usize`/`Duration`/`u32`, so the struct's `#[derive(Clone, Copy, ...)]` (`surb_store.rs:121`) still holds, and `deny_unknown_fields` (`:125`) still loads operator YAML that omits them:
   - `quarantine_rb_capacity: usize` — default 16, `range(min = 1)`.
   - `quarantine_max_pseudonyms: usize` — default 1024, `range(min = 64)`.
   - `quarantine_lifetime: Duration` — default 60 s, `validate(custom(validate_quarantine_lifetime))`, serde `with = "humantime_serde"`. Must exceed the worst-case Session initiation window `2 × initiation_timeout_base × RoutingOptions::MAX_INTERMEDIATE_HOPS` (`transport/hopr/src/lib.rs:131`; `constants.rs:17` base = 5 s; `manager.rs:2417`). At base 500 ms (`manager.rs:1627` default) × 3 hops that is 3 s; at the transport's 5 s base × 3 hops it is 30 s. **60 s clears both**; the [ASSUMPTION] is that `MAX_INTERMEDIATE_HOPS ≤ 3`.
   - `quarantine_surb_budget: usize` — default 65_536, `range(min = 4096)`. Bounds total *unadmitted* SURBs; a backstop above the cache's own 1024 × 16 = 16 384 bound. (~26 MB.)
   - `admitted_max_pseudonyms: usize` — default 10_000, `range(min = 1)`. Capacity of the admitted ring cache; **must be ≥ `maximum_managed_sessions`** (default 100, `DEFAULT_MAXIMUM_MANAGED_SESSIONS` in `transport/hopr/src/config.rs:818`, validates to 100 000 at `:919`). The 10 000 default is not this session count but the session crate's separate `maximum_sessions` default (`transport/session/src/manager.rs:1994`), which is the ceiling the admitted cache must cover. Document the invariant; `hopr-transport` logs an error at startup if violated (Phase 5).
   - `admitted_headroom_percent: u32` — default 50, `range(min = 0)`. Inverse of the 2/3 `surb_buffer_target_ceiling` (`transport/hopr/src/lib.rs:148-159`), so an Entry announcing `maximum_surb_buffer_size` (= `rb_capacity × 2/3`, `lib.rs:442`) yields a full-`rb_capacity` admitted ring.
4. No change to the config chain: `SurbStoreConfig` is already `#[validate(nested)]` `surb_store` on `HoprPacketPipelineConfig` (`transport/hopr/src/config.rs:614`) → `HoprProtocolConfig.packet` (`config.rs:217`) → `HoprLibConfig.protocol` (`hopr/hopr-lib/src/config.rs:78`). The `HOPR_INTERNAL_SURB_PSEUDONYM_LIFETIME_MS` override block (`transport/hopr/src/lib.rs:393-401`) uses `..cfg.packet.surb_store` and forwards the new fields unchanged.
5. Verify: `cargo check -p hopr-protocol-hopr`; add the config-validation test (Phase 7).

### Phase 2 — Two-class store core (`protocols/hopr/src/surb_store.rs`)

6. `MemorySurbStore` fields (`surb_store.rs:262-286`), all `Clone`/Arc-shared so the three independent holders (pipeline encoder+decoder via `transport/hopr/src/lib.rs:881`/`pipeline.rs:471-488`, chain wiring `hopr/hopr-lib/src/builder/chain_wiring.rs:45`, SURB-flush task `lib.rs:1056`) share one budget:
   - rename `surbs_per_pseudonym` (`surb_store.rs:264`) → `admitted_surbs_per_pseudonym` (naming discipline: quarantine/admitted, never "tier" — "tier" already means the shares/plain split at `surb_store.rs:919-959`).
   - `quarantine_surbs: moka::sync::Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>`.
   - `quarantine_budget: Arc<std::sync::atomic::AtomicUsize>` (count of unadmitted SURBs).
   - **No separate admission marker cache** — an entry in `admitted_surbs_per_pseudonym` (taken with `.get`, so it can be pushed into) is the admission predicate.
   - Update the two intra-doc links broken by the rename: `surb_store.rs:273` (`[`surbs_per_pseudonym`](Self::surbs_per_pseudonym)`) and `:319` (the `EvictedCache::SurbRing` doc `(`surbs_per_pseudonym`)`). CI builds `private_intra_doc_links` as an error (`surb_store.rs:157`).
7. Builders in `MemorySurbStore::new` (`surb_store.rs:523`):
   - `admitted_surbs_per_pseudonym`: keep the existing shape (`:545-553`) but `.max_capacity(cfg.admitted_max_pseudonyms.max(1) as u64)` (was `max_pseudonyms`, `:552`) and **no `.time_to_idle`** — see *The admission token* for why a ring's lifetime must be exactly its admission's. Its listener records `EvictedCache::SurbRing` and **logs at ERROR** on a Size removal (a Size eviction here means `admitted_max_pseudonyms < maximum_managed_sessions`, an operator misconfiguration, not routine churn; a teardown's own removal is `Explicit` and leaves through `retire`).
   - `quarantine_surbs`: `.time_to_idle(cfg.quarantine_lifetime.max(MINIMUM_SURB_LIFETIME))`, `.eviction_policy(lru())`, listener recording `EvictedCache::QuarantineRing` and decrementing `quarantine_budget` by `ring.len()` on `cause != RemovalCause::Explicit`, `.max_capacity(cfg.quarantine_max_pseudonyms.max(MINIMUM_QUARANTINE_PSEUDONYMS) as u64)`.
   - **Do not** size anything by `max_pseudonyms` any more; leave that field's validator (`:194-197`) intact for back-compat but unused by the SURB path (note it in the field doc).
8. `impl<S: Clone> SurbRingBuffer<S> { fn with_capacity(&self, capacity: usize) -> Self }` on `SurbRingBuffer<S>` (`surb_store.rs:905`, constructed via `Self::new` at `:1007`): lock `inner` and **deep-copy both deques and the generation** into a fresh buffer with the new `capacity`. A shallow Arc clone would not raise `capacity` (a plain field, `surb_store.rs:909`, read at `:1081`); a re-push would re-run `push` and corrupt generation supersede (`surb_store.rs:1053-1071`). Insertion order in both deques is preserved.
9. `insert_surbs` (`surb_store.rs:661`): preserve the empty-batch early return (`:665-674`) and the mixed-generation filter (`:682-694`) — but the empty-batch branch must report `retained: self.ring_len(&pseudonym)`, **not** a straight rename of `self.surbs_per_pseudonym` (the v1 contradiction). Then:
   - **Route first** by taking the admitted entry: `if let Some(admitted) = self.admitted_surbs_per_pseudonym.get(&pseudonym) { … }` — one moka lookup, and the same lookup yields the ring to push into. Do **not** compute `ring_len` on the hot path; take `before` from the ring the lookup returns.
   - **Admitted path**: no budget check. Take the entry `admit` installed (`get(&pseudonym)`) and `push` into it, after migrating any quarantine ring for the pseudonym under the ring lock via `with_capacity` (see step 12). **Never** `or_insert_with` here: only `admit` may create an admitted entry, because one recreated after a racing retire would carry epoch `0` — which no retire matches — and a successor `admit` finding it would only re-stamp the epoch, leaving it at the floor capacity, so later inserts would overflow it and lose SURBs and PIX shares. An entry absent after the lookup therefore routes to the quarantine path below like any other unadmitted batch.
   - **Quarantine path**: only here run the budget: if `quarantine_budget.load(Relaxed) + surbs.len() > cfg.quarantine_surb_budget`, **collect victim keys in a first pass** (`quarantine_surbs.iter().map(|(k, _)| k).take(n).collect::<Vec<_>>()`), drop the iterator, then `invalidate` each — never `invalidate` from inside a live `iter()` (moka 0.12.16 holds the cache lock during iteration; `Cargo.lock:6134-6136`). Quarantine entries are uniformly untrusted, so **eviction order is arbitrary** — the v1 "LRU-first" claim is dropped (moka `iter()` gives no order; the only in-repo `iter()` use, `protocols/pix/src/reconstructor/mod.rs:1450-1460`, sums counts, order-irrelevant). If still over after draining, **refuse the batch**: `SurbInsertOutcome { retained: <admitted-then-quarantine ring_len>, evicted: surbs.len(), evicted_shares: <count of surbs with surb.ssa_index().is_some()> }` — report the batch's real share count, not 0 (Critic 4).
   - after `push`, apply `quarantine_budget` delta = `outcome.retained - before` on the quarantine path only.
   - keep the `RingPlain`/`RingShare` recording (`:705-717`).
10. `find_surb` (`surb_store.rs:629`): consult `admitted_surbs_per_pseudonym` first, then `quarantine_surbs`. **Do not promote on hit** (Design 1's spoof). Apply `quarantine_budget` delta on the quarantine path only, = ring `len()` before minus after (covers the destructive `pop_next_valid` discards at `surb_store.rs:1114-1129`).
11. `tier_lens` (`surb_store.rs:722`) and the new `ring_len` helper: check admitted then quarantine so the decoder trace line (`decoder.rs:258`) and `retained` agree. `ring_len` returns the sum of both caches' ring lengths (so a transient dual-ring state cannot under-report).
12. `impl SurbStore for MemorySurbStore` (`surb_store.rs:627`): implement
    - `fn admit(&self, pseudonym: &HoprPseudonym, expected_surbs: usize)`: compute `capacity = min(cfg.rb_capacity.max(MIN_SURB_RB_CAPACITY), expected_surbs.max(cfg.quarantine_rb_capacity).saturating_mul(100 + cfg.admitted_headroom_percent) / 100).max(cfg.quarantine_rb_capacity)` — **floored at `quarantine_rb_capacity` (16), never 1**, and treating `expected_surbs == 0` as "unknown" → the floor (Critic 4). Insert an empty `SurbRingBuffer::new(capacity)` via `entry_by_ref().or_insert_with(...)`; **do not clobber an existing admitted ring** (a re-admitted fixed pseudonym, `manager.rs:2827`). Then, if `quarantine_surbs` holds a ring for it, promote: `let promoted = qring.with_capacity(capacity); admitted.insert(p, promoted); quarantine.invalidate(&p)` (quarantine listener skips `Explicit`, so `quarantine_budget` is net-zero). Record `admitted`/`promoted`.
    - `fn retire(&self, pseudonym: &HoprPseudonym, token: u64)`: an atomic compare-and-remove over the admitted entry — `entry_by_ref(pseudonym).and_compute_with(...)` removing only when the stored epoch equals `token`, and a `Nop` otherwise. **Not** an unconditional `invalidate`: the retire is deferred, so it can land after a successor Session was admitted on the same client-chosen pseudonym, and removing then would destroy the successor's live ring (see *The admission token*). `token == 0` returns early — it never matches. **Do not touch `quarantine_surbs`** — a rolled-back/unestablished Session's preload SURBs stay in quarantine and expire on their short TTI (repairs the rollback share-loss risk). Idempotent: retiring a non-admitted pseudonym is a no-op.
    - `pub fn surbs_held(&self) -> usize` (test/observability seam) = `quarantine_budget` + Σ `admitted_surbs_per_pseudonym.iter().map(|(_, rb)| rb.len())` (read-only; not on the hot path).
    - **No `resync_budget`.** v1's opportunistic resync was a CPU/lock amplifier on exactly the attack state (Critic 2/4). Accounting is exact: `admit`/`retire` change entry count (no counter), quarantine deltas are computed at the two mutation sites and in the listener.
13. `EvictedCache` (`surb_store.rs:314-330`): **append** `QuarantineRing` at the END of the enum and of `EvictedCache::ALL` (`:333-340`, grows to 7), so existing discriminants (indexed at `surb_store.rs:430` `state.counts[cache as usize]`) do not shift. The existing all-slots test (`:2630-2635`) already asserts `ALL` is contiguous from 0.

### Phase 3 — Trait surface (`protocols/hopr/src/traits.rs`)

14. After `bump_generation` (`traits.rs:99-101`) add, with default no-op bodies:
    ```rust
    fn admit(&self, _pseudonym: &HoprPseudonym, _expected_surbs: usize) -> u64 {
        0
    }
    fn retire(&self, _pseudonym: &HoprPseudonym, _token: u64) {}
    ```
    `admit` returns the raw number naming the admission — the session side's `AdmissionToken` — and defaults to `0`, which is reserved for "no admission": a store that does not separate a trusted class therefore honours the sentinel and a teardown presenting it matches nothing. `retire` takes that same number back. Document `expected_surbs` as the Session's announced buffer target; the store owns the headroom math. Add an optional defaulted `fn surbs_held(&self) -> usize { 0 }`.

### Phase 4 — Session seam (`transport/session/src/{types,lib,manager}.rs`)

15. `SurbAdmissionControl` trait + add it to the existing `pub use types::{...}` at `lib.rs:54-58` (Crate-boundary decision).
16. `SessionManager<S>` field + `Clone` + `set_surb_admission` (Crate-boundary decision).
17. Wire **admit at the established-Session point**, not at `allocate_session_slot`. `allocate_session_slot` (`manager.rs:2669`) returns a `SessionSlotGuard`; `commit()` (`manager.rs:1182`) is called at exactly three places, each the **last statement** of its branch, with no fallible step between admit and commit:
    - Entry balanced: `manager.rs:3078` — `admit(&session_id, self.cfg.maximum_surb_buffer_size)` (the Entry announced this; `manager.rs:1676`, derived at `transport/hopr/src/lib.rs:442`).
    - Entry unbalanced: `manager.rs:3153` — same value.
    - Exit: `manager.rs:5092` — `admit(&session_id, announced_surb_target)`, computed at `manager.rs:4865-4868`, which is **live at `:5092`** (it is computed at `:4865`, 227 lines before `commit()`; v1's bug was admitting inside `allocate_session_slot` at `:4800`, *before* `:4865` — fixed by admitting at `commit`).
    Change `SessionSlotGuard::commit` to `commit(&mut self, slot: Option<&SessionSlot>, admission: Option<&dyn SurbAdmissionControl>, capacity: usize)` so the guard performs the admit, keeping it paired with the commit that makes the Session live. Add `admission: Arc<OnceLock<Arc<dyn SurbAdmissionControl>>>` to the guard (`manager.rs:1156-1161`) and to `SessionSlotGuard::new` (`:1165-1180`), populated from `self.surb_admission.clone()` in `allocate_session_slot`.
    - `slot` is the cached copy when `commit_slot_guard` could still find it *and* it is this establishment's own, and `None` otherwise — a teardown removed it first, or the pseudonym was reallocated to a successor Session before this commit resumed. Both are reachable on both sides, since the Exit sends `SessionEstablished` before it commits (so an Entry can close or answer with a `SessionId`-scoped `SessionError` in that window). The guard carries its own slot's admission cell, and `commit_slot_guard` compares it by `Arc::ptr_eq` against the cell of the slot the cache hands back for the key: a slot reallocated to a successor has a different cell, so it is discarded like an absent one and the successor's live ring is never stamped with this commit's token. A `None` slot still commits: the Session *was* established, so the guard must be marked committed and `METRIC_ACTIVE_SESSIONS` incremented (the teardown already decremented it) and `close_session`'s "session closed" must be paired with "new session established". Only the admit is skipped — there is no cell to carry its token.
    - `commit` claims the token cell to arbitrate against a teardown racing it (see *The admission token*). The teardown claims the cell by storing `AdmissionToken::NONE` into it if empty (`SessionSlot::claim_admission_token`), so if `commit`'s own `set` then fails, a teardown already read the cell and saw nothing to retire: `commit` retires the admission it just made, immediately rather than deferred, since the Session is already gone and no packet of its was ever in flight for this admission.
18. Wire **retire at teardown**. Thread the handle into the two free fns: `release_session_slot(session_id, session_data, reason)` (`manager.rs:188`) and `close_session(session_id, session_data, reason)` (`manager.rs:178`) each gain `admission: Option<Arc<dyn SurbAdmissionControl>>`, `grace: Duration` and `token: AdmissionToken` parameters, and `release_session_slot` defers the retire by `grace` on a spawned task (`defer_admission_retire`), retiring inline when no runtime is running. **Every teardown funnels through `release_session_slot`**, so this single point covers: explicit close (`manager.rs:4062`), the moka sessions eviction listener (`:2438`), the supervisor `PixFailure` path (`:3524`), `handle_session_error` (`:5143`), `refuse_ssa_request` (`:5397`), and the `SessionSlotGuard::drop` rollback (`:1202`). Each teardown reads the token from the slot it removes via `claim_admission_token`, and the three `close_session` call sites (`:2438`, `:3524`, `:4062`) each pass `self.surb_admission.get().cloned()` (the listener closure captures an `Arc` clone at cache-build time, mirroring prior art `e90b7228ac`'s `close_hook_for_listener`); the guard passes its own field.
    - **Do not retire in `begin_surb_drain` (`manager.rs:3981`)** — it deliberately keeps the slot so the drain can spend the SURBs.
    - Retire on the guard rollback is now wired (`:1202` → `release_session_slot` → `retire`), closing Critic 2's "abandoned Session leaks an admitted marker" defect. Because admit happens at commit (not allocation) and retire is idempotent, a rolled-back Session was never admitted, so the rollback retire is a harmless no-op that also leaves the peer's preload SURBs in quarantine.

### Phase 5 — Composition root (`transport/hopr/src/lib.rs`)

19. In `HoprTransport::new`, the store is created at `lib.rs:407` and `smgr` inline at `lib.rs:421`. Restructure to `let smgr = Arc::new(SessionManager::new(SessionManagerConfig { ... }))` **before** the `Self { ... }` literal, then `let _ = smgr.set_surb_admission(Arc::new(StoreSurbAdmission { store: path_planner.surb_store.clone() }));`, then use `smgr` in the literal. This avoids binding `this` (v1's struct-literal problem) since `path_planner` is available before the literal. Add a startup check: `if cfg.packet.surb_store.admitted_max_pseudonyms < cfg.session.maximum_managed_sessions { tracing::error!(...) }`.
20. `hopr_transport::SurbStore` is already re-exported at `lib.rs:65`; `MemorySurbStore` likewise.

### Phase 6 — Metrics & docs

21. In `surb_store.rs`, under `#[cfg(all(feature = "telemetry", not(test)))]` beside `:291-305`, add `METRIC_SURB_STORE_ADMISSIONS: MultiCounter` = `hopr_surb_store_admissions_count`, labels `["event"]` with bounded values `{admitted, promoted, retired, quarantine_refused, quarantine_evicted, admitted_overflow}`, and `METRIC_SURB_STORE_SURBS_HELD: MultiGauge` = `hopr_surb_store_surbs_held`. Initialise both in `MemorySurbStore::new` (`surb_store.rs:524-528`). The existing `hopr_surb_store_evictions_count`/`_last_interval` (`METRICS.md:87-88`) pick up the new `quarantine_ring` label automatically via `EvictedCache::ALL`.
22. Run `.github/scripts/generate-metrics-docs.sh --fix` to regenerate `METRICS.md`; the pre-commit hook at `flake.nix:335` enforces sync. The cardinality check (`.github/scripts/generate-metrics-docs.sh:159-182`) only constrains `hopr_pix_*`, so keep the `event` label set bounded by construction.

### Phase 7 — Gates

23. `cargo nextest run --lib -p hopr-protocol-hopr`, `--lib -p hopr-transport-session`; `cargo clippy`; `cargo shear --fix -p hopr-protocol-hopr -p hopr-transport-session -p hopr-transport`; `nix fmt`; `cargo nextest run --no-run`.

---

## Invariants to preserve

- **Shares-first / oldest-first pop and eviction** inside `SurbRingBuffer`: `next_tier`/`make_room_for` (`surb_store.rs:944-988`), `push` ordering (`:1047-1101`). `with_capacity` preserves insertion order in both deques.
- **Generation supersede semantics**: newer batch clears both tiers, equal appends, older discarded (`surb_store.rs:1053-1071`); `with_capacity` carries `generation` unchanged.
- **`SurbInsertOutcome` contract** (`protocols/hopr/src/types.rs:208-222`): `retained`/`evicted`/`evicted_shares`; the decoder's `num_evicted_surbs` (`decoder.rs:253`, docs `types.rs:36-68`) keeps meaning "SURBs that left or never entered the store". A quarantine-budget refusal reports `evicted = batch len`, `evicted_shares` = the batch's true share count.
- **Empty-batch early return** (`surb_store.rs:665-674`, now via `ring_len`) and **mixed-generation filter** (`:682-694`) unchanged in meaning.
- **Reply-opener caches and `generations`** (sending-side, `surb_store.rs:263,283`) untouched by admit/retire.
- **`invalidate_relayer`/`revalidate_relayer`** (`surb_store.rs:752-766`) still gate `find_surb` via `is_surb_usable`.
- **Probe reply path** (`transport/probe/src/probe.rs:94` → `planner.rs:953-954`) still resolves, via `find_surb` consulting quarantine.
- **Eviction accounting**: `EvictionCounts::bump` skips `Explicit` (`surb_store.rs:365-372`); `record_many` on count 0 is free (`:422-425`).
- **`begin_surb_drain` keeps the slot** (`manager.rs:3976-3981`) — retire must not fire there.
- **`SessionManager` cheap-`Clone`** and the `OnceLock` late-injection pattern (`manager.rs:2161,2521`).
- **`MemorySurbStore` cheap-`Clone`** with all state Arc-shared (`surb_store.rs:261-286`) so all three holders share one budget.
- **Decoder's generic `Arc<S>`** (`decoder.rs:19`) — admit/retire defaulted so non-admitting `S` still compiles.

---

## Testing plan

All store unit tests live in the synchronous `#[cfg(test)] mod tests` (`surb_store.rs:1162-2923`), reuse the crypto-free fixtures `surb_via`/`plain_surb`/`share_surb` (`surb_store.rs:1246-1299`) and the insert-then-`run_pending_tasks()` idiom (`surb_store.rs:2793-2795`). Metrics are compiled out under `cfg(test)` (`surb_store.rs:291`), so assert the internal `EvictionStats` counters (`store.stats.state.lock().counts[..]`).

New tests in `protocols/hopr/src/surb_store.rs`:

1. `admitted_ring_survives_a_fresh_pseudonym_flood` — the issue's regression test, made non-tautological: set `quarantine_max_pseudonyms: 64` (the floor), `store.admit(&live, cap)`, insert one SURB for `live`, insert one SURB each for `65` fresh pseudonyms; `run_pending_tasks()` on **both** caches; **assert the quarantine cache actually shed a ring** (`store.stats.state.lock().counts[EvictedCache::QuarantineRing as usize].size > 0`) **before** asserting `find_surb(SurbPseudonym(live))` still returns a SURB. (v1's version could be configured so nothing was evicted, making it vacuous.)
2. `a_second_legitimate_session_can_still_insert_under_quarantine_pressure` — fills the quarantine budget with fresh-pseudonym churn, then admits a second pseudonym and asserts its `insert_surbs` is **accepted and retained** (proves the budget never refuses admitted inserts — the failure mode v1 had).
3. `admitted_pseudonyms_above_max_pseudonyms_do_not_degrade` — set `max_pseudonyms: 100` (→ floored 1000) but `admitted_max_pseudonyms: 2000`; admit 1500 pseudonyms; assert each admitted ring is still readable (proves admission no longer depends on `max_pseudonyms`).
4. `quarantine_surbs_should_be_promoted_on_admission` — insert SURBs for an unadmitted pseudonym, then `admit` it; assert the ring is now in `admitted_surbs_per_pseudonym` at full capacity, the quarantine entry is gone, and `surbs_held()` is unchanged.
5. `admit_should_not_clobber_an_existing_admitted_ring` — admit, insert SURBs, admit again; assert the SURBs survive.
6. `retire_should_release_only_the_admitted_class` — admit + insert, `retire`; assert `find_surb` is `None` and the admitted ring is gone, and that a pseudonym that was never admitted is unaffected.
7. `find_surb_should_consult_quarantine_for_probe_replies` — insert for an unadmitted pseudonym; assert `find_surb(SurbPseudonym(p))` returns it and does **not** promote it.
8. `surb_store_config_should_reject_out_of_range_quarantine_and_budget_fields` — mirror `config_should_reject_an_eviction_report_interval_below_the_minimum` (`surb_store.rs:2913`), one assertion per new field; plus a serde test asserting a `deny_unknown_fields` config with the new fields absent still loads.
9. Update `memory_surb_store_should_count_evictions_from_every_cache` (`surb_store.rs:2775-2809`): fresh-pseudonym inserts now land in quarantine, so assert `EvictedCache::QuarantineRing` size evictions > 0 (keep the `ReplyOpenerBatch`/`Generation` assertions).

Adjacent tests that **will break** and must be adjusted (fresh-pseudonym inserts now route to the 16-cap quarantine):
- `undecryptable_replies_after_upload_flood` (`surb_store.rs:1766-1800`) and its callers `replies_stay_decryptable_while_the_opener_cache_has_not_overflowed` (`:1798`) and `a_sustained_upload_keeps_the_return_path_alive_at_the_deployed_config` (`:1853`) pop 200 SURBs from an **unadmitted** pseudonym → panic once 16 are exhausted. Fix: call `exit.admit(&pseudonym, rb_capacity)` in the helper right after building `exit`.
- `memory_surb_store_should_count_ring_overflows_by_tier` (`surb_store.rs:2815-2835`) asserts `retained == MIN_SURB_RB_CAPACITY` on a default (unadmitted) store → now 16. Fix: `store.admit(&pseudonym, capacity)` before the fills.
- `protocols/hopr/src/codec/mod.rs:407` `decode_should_not_warn_per_packet_when_a_full_buffer_drops_share_less_surbs` fills to `rb_capacity` on an unadmitted pseudonym (`:446-451`). Fix: admit the pseudonym on the fixture's `receiver.surb_store` first.
- `memory_surb_store_should_report_the_size_of_each_tier_per_pseudonym` (`:2336`) and `memory_surb_store_should_skip_surbs_whose_first_relayer_was_invalidated` (`:1307`) insert ≤ 16 SURBs unadmitted → still pass (quarantine cap 16, `tier_lens` sums both caches), but run them to confirm.
- Benches (`protocols/hopr/benches/codec_bench.rs:46,58`, `transport/hopr/benches/protocol_throughput_emulated_bench.rs:184`) only construct the store → compile unchanged.

Session-seam tests: add a `hopr-transport-session` unit test with a mock `SurbAdmissionControl` (mockall, `transport/session/src/testing/mod.rs`) asserting `admit` fires at `commit()` and `retire` at close/rollback and **not** on `begin_surb_drain`. The manager test module already drives sessions with `mgr.start(...)` and `close_session` (e.g. `manager.rs:6860`).

Commands (from CLAUDE.md):

```
cargo nextest run --lib -p hopr-protocol-hopr
cargo nextest run --lib -p hopr-transport-session
cargo nextest run --lib -p hopr-lib
cargo nextest run -p hopr-lib --test transport_session_pix -j 1
cargo nextest run -p hopr-lib --test session_integration_tests -j 1
```

The last snapshots `hopr/hopr-lib/tests/snapshots/session_integration_tests__surb_metrics.snap` — update it. Then `cargo clippy`, `cargo shear --fix -p hopr-protocol-hopr -p hopr-transport-session -p hopr-transport`, `nix fmt`, `.github/scripts/generate-metrics-docs.sh --fix`.

---

## Migration & back-compat

- **Existing operator YAML keeps loading**: every new `SurbStoreConfig` field carries both `#[default(...)]` and `serde(default = "...")`, so `deny_unknown_fields` (`surb_store.rs:125`) and every enclosing level do not reject a file that omits them. Only additive fields are safe under `deny_unknown_fields`.
- **Defaults** (Phase 1 step 3): quarantine cap 16, quarantine pseudonyms 1024, quarantine TTI 60 s, quarantine budget 65 536 SURBs, admitted pseudonyms 10 000, admitted headroom 50 %.
- **Behaviour change operators will observe**: an established Session is admitted at commit (before its first inbound SURB), so it gets a full-capacity ring from the outset. A genuinely non-Session pseudonym (a bare `Final` packet, a probe) is capped at 16 SURBs with a 60 s TTI instead of 100 000 SURBs with 600 s. That is the intended tightening and the security fix.
- **No trait-signature break**: `admit`/`retire` are defaulted.
- **No `SessionManager::start` signature change**: the setter avoids churning 86 `.start(` call sites.

---

## Rollout & observability

- **New metrics**: `hopr_surb_store_admissions_count` (MultiCounter, label `event` ∈ {admitted, promoted, retired, quarantine_refused, quarantine_evicted, admitted_overflow}); `hopr_surb_store_surbs_held` (MultiGauge). Plus the new `quarantine_ring` value on the existing `hopr_surb_store_evictions_count` / `_last_interval` `cache` label (`METRICS.md:87-88`).
- **Operator before/after**: before, `hopr_surb_store_evictions_count{cache="surb_ring",cause="size"}` rising without bound under pseudonym churn, with no way to tell a Session's loss from noise. After, `quarantine_ring` size evictions absorb the churn, `hopr_surb_store_admissions_count{event="admitted"}` tracks live Sessions, and `hopr_surb_store_surbs_held` gives a ceiling reading.
- **Alerting**: alert on `event="admitted_overflow"` (any occurrence means `admitted_max_pseudonyms < maximum_managed_sessions`); alert on `event="quarantine_refused"` rising (unauthenticated ingress over budget); alert on any `cache="ring_share"` evictions (already warns at any count, `surb_store.rs:475-481`).
- **Feature gating**: all new metric series are behind `#[cfg(all(feature = "telemetry", not(test)))]`, enabled through `hopr-protocol-hopr/telemetry` from `hopr-transport/telemetry` (`transport/hopr/Cargo.toml:67`).

---

## Residual risk & open questions

- **The ~400 B/SURB figure is unmeasured.** `HoprSurb::SIZE = 402 B` is the wire size (`crypto/packet/src/lib.rs:219`); resident cost adds `HoprSurbId` + `VecDeque` + moka overhead. **[ASSUMPTION]** — validate empirically or reuse the reconstructor's measured `AWAITING_ACK_ENTRY_BYTES = 400` (`protocols/pix/src/reconstructor/mod.rs:307`).
- **Quarantine budget accounting is soft.** `ring.len()` read outside the ring lock plus concurrent inserts can drift the atomic by a few SURBs; moka's eviction listener runs on moka's internal thread. Bounded and self-correcting (the cache's own `max_capacity` is the hard bound; the counter is the backstop). No resync by design.
- **moka eviction is asynchronous and eventual.** The live-ring-survives assertion is robust (physically distinct caches); the quarantine-overflow assertion may need a small poll/retry if moka batches maintenance — tests follow the `run_pending_tasks()` idiom on **both** caches.
- **Quarantine TTI vs `MAX_INTERMEDIATE_HOPS`.** The 60 s default is derived from `2 × 5 s × hops` with hops assumed ≤ 3. **[ASSUMPTION]**; if the deployment allows more hops, raise `quarantine_lifetime`.
- **`admitted_max_pseudonyms` vs `maximum_managed_sessions`.** The store cannot see the session config, so the invariant is enforced only by a startup error log in `hopr-transport` (Phase 5) plus documentation, not a validator (the codebase refuses cross-config product validators: `transport/hopr/src/config.rs:319-320`, `manager.rs:1286-1290`).
- **Admitted-class memory is bounded by config, not by the budget.** `maximum_sessions` validates to 100 000 (`transport/hopr/src/config.rs:919`), so `admitted_max_pseudonyms × rb_capacity` can be large. This is the operator's own ceiling and requires an established Session per pseudonym; it is deliberately **not** counter-bounded (doing so would drop live shares — the blocker the critics raised).
- **Budget-refused newer-generation batches never reach `push`**, so return-path supersession can be delayed under quarantine pressure. Accepted: quarantine holds only untrusted traffic; documented in the refusal doc.
- **Entry-side admit.** Needed because the Exit can emit its first keep-alive SURBs before the Entry commits (Exit commits at `manager.rs:5092`, sends `SessionEstablished` at `:5064`; the Entry commits at `:3078`/`:3153` after receiving it). The one-RTT window is covered by quarantine.
- **Reply-opener caches are out of scope.** `pseudonym_openers` is capped by `max_openers_per_pseudonym` (100 000, `surb_store.rs:534-542`) and the inner per-pseudonym cache likewise (`:745`). These are reached only by *locally-originated* sends, not arbitrary peers, so they do not carry the #8515 attack; sizing them is #8382/#8419. Stated here so the "memory effectively unbounded" claim is honestly scoped.
- **Prior art not merged**: `e90b7228ac` (adds `remove_pseudonym` + close hook) and `5efe862167` (outer opener bound) on `origin/em/memory-leak-fable-fixes`. This plan builds on the same seam shape but does not depend on them; if they land, reconcile `remove_pseudonym` with `retire`.

---

## Effort estimate (engineer-days)

| Phase | Work | Days |
|---|---|---|
| 1 | Config surface: 6 fields, validators, consts, default fns, config-validation test | 0.75 |
| 2 | Store core: two caches + quarantine budget, `with_capacity`, `insert_surbs` routing/drain, `find_surb` dual-lookup, `admit`/`retire`/`surbs_held`, `EvictedCache::QuarantineRing`, doc-link fixes | 2.5 |
| 3 | Trait surface: defaulted `admit`/`retire`/`surbs_held` | 0.25 |
| 4 | Session seam: `SurbAdmissionControl` + re-export, field/Clone/setter, admit at `commit`, retire through `release_session_slot`, guard field, listener capture, seam mock test | 2.0 |
| 5 | Composition root: `StoreSurbAdmission` adapter, `smgr` restructure + setter, startup check | 0.5 |
| 6 | Metrics + `METRICS.md` regen | 0.5 |
| 7 | Tests (unit suite + adjusting 4 breaking tests + integration runs) and iteration | 2.0 |
| 8 | Gate runs (`clippy`/`shear`/`nix fmt`/`nextest`), moka-semantics review, residual-risk write-up | 1.0 |
| | **Total** | **~9.5 (range 8–11)** |

Matches the issue's `effort:medium` label with headroom for the cross-crate seam, the concurrency protocol, and the empirical budget-default validation.

---

## Critic resolutions (one line per blocker/major defect)

**Technical-soundness critic**
- *Budget cannot bound admitted memory / only drains quarantine* → Resolved by restructure: the admitted class is bounded **structurally** (`admitted_max_pseudonyms` × per-ring cap), not by the counter; the counter is renamed `quarantine_surb_budget` and explicitly scoped to quarantine. Arithmetic corrected (`admitted_max_pseudonyms` default 10 000, not `max_pseudonyms`).
- *`iter()` + `invalidate` deadlock in moka 0.12* → Fixed: two-pass victim collection (collect keys, drop iterator, then `invalidate`).
- *Exit `admit` can't use `announced_surb_target` (allocate at 4800 precedes it at 4865)* → Fixed: admit moved to `SessionSlotGuard::commit` (`manager.rs:5092`), where `announced_surb_target` (`:4865`) is live; the `[ASSUMPTION]` deleted.
- *Marker-TTL reasoning internally inconsistent* → Fixed: the marker cache is deleted; the admitted ring cache is the source of truth, and idle eviction runs `retire` (Session treated as dead, SURBs dropped) — stated explicitly.
- *Empty-batch early return reads the renamed field* → Fixed: it now uses `ring_len` (admitted-then-quarantine).
- *`SessionId` re-export redundant* → Fixed: only `SurbAdmissionControl` is added to the existing `pub use` at `lib.rs:54-58`.
- *Rename breaks intra-doc links at `:273`/`:319`* → Fixed: Phase 2 step 6 updates both.
- *Test 1 conflates the 64 and 1000 floors* → Fixed: test 1 pins `quarantine_max_pseudonyms: 64` and asserts a quarantine eviction occurred before asserting survival.
- *Hot-path extra lookups* → Fixed: route by one `get` (which also yields the ring to push into); admitted inserts skip the budget path entirely; `before` taken from the ring that lookup returns.
- *Marker/insert race* → Fixed: single-cache source of truth; admit promotes any existing quarantine ring via `with_capacity` and `insert_surbs` migrates a quarantine ring on the admit-window race (documented).
- *`close_session` free-fn churn / `HoprTransport::new` struct-literal* → Fixed: `release_session_slot`/`close_session` gain one `admission` param each; the guard gains one field; the listener captures an `Arc` at build; `HoprTransport::new` binds `smgr` before the literal.
- *No mechanism bounds admitted memory / no concurrency design / no doc-link step / no hot-path measurement / no close-session churn / no config-struct decision / no rollback case* → All addressed (structural admitted bound; the single-cache routing protocol; Phase 2 step 6; Phase 2 step 9 short-circuit; Phase 4 step 18; flat fields with all six fields tested together in test 8; admit-at-commit makes rollback a no-op).

**Red-team critic**
- *[blocker] Global budget starves/stops legitimate Sessions, ~10 Sessions exhaust it* → Resolved: budget is quarantine-only; admitted inserts are never refused; the admitted bound is structural.
- *[major] Admitted count bounded by `max_pseudonyms` not `maximum_sessions`* → Fixed: `admitted_max_pseudonyms` (default 10 000) + startup check.
- *[major] `admit` races `insert_surbs` and loses SURBs* → Fixed: single-cache routing + `with_capacity` promotion under the ring lock + quarantine re-check migration.
- *[major] `retire` not wired to the guard rollback* → Fixed: guard drop → `release_session_slot` → `retire` (idempotent).
- *[major] `resync_budget` is a CPU/lock amplifier* → Fixed: `resync_budget` removed entirely; accounting is exact at the two mutation sites.
- *[minor] Quarantine TTI 30 s vs initiation window* → Fixed: default 60 s, derived from `2 × base × hops`.
- *[major] Exit `admit` over-reserves* → Fixed: Exit admits at commit with the negotiated `announced_surb_target`.
- *[major] Regression test tautological* → Fixed: tests 1–3 assert the quarantine actually shed, that a second Session inserts under pressure, and that admitted rings survive above `max_pseudonyms`.
- *[minor] Entry admit unnecessary* → Rejected: Entry admit is required (the Exit can emit SURBs before the Entry commits; the window is one RTT) — evidence in Residual risk.
- *Missing: fair-share, admitted bound tied to `maximum_sessions`, rollback retire, pseudonym reuse, drain/retire verification, budget-exhausted metric, hot-path analysis, prior-art reconciliation* → All addressed (structural admitted bound; `admitted_max_pseudonyms`; Phase 4 step 18; test 5 no-clobber; `retire` only touches the admitted class so a drain's SURBs are untouched; `admitted_overflow` + `quarantine_refused` events; Phase 2 step 9; Residual risk).

**Missing-coverage critic**
- *[blocker] Admission boundary not authenticated (admit before establishment; no session server admits unconditionally)* → Fixed: admit moved to `SessionSlotGuard::commit`, the established point; rollback retires.
- *[major] moka `iter()` has no order guarantee → LRU-first unimplementable* → Fixed: the LRU-first claim is dropped; quarantine evicts in arbitrary order (uniformly untrusted).
- *[major] Empty-batch contradiction* → Fixed (as above).
- *[major] Test plan omits the tests its routing change breaks* → Fixed: the four breaking tests (`undecryptable_replies_after_upload_flood` + callers, `memory_surb_store_should_count_ring_overflows_by_tier`, `codec/mod.rs:407`) are enumerated with fixes; the plan runs the full `--lib -p hopr-protocol-hopr` suite.
- *[major] Budget refuses admitted batches* → Fixed (quarantine-only budget).
- *[major] Reply-opener caches unassessed* → Fixed: explicit scope section (out of scope, reached only by local sends, deferred to #8382/#8419).
- *[minor] Sole production ingress / sending side* → Fixed: stated in the Problem restatement.
- *Missing: module doc update, marker/ring divergence, Entry-vs-Exit admit, byte-vs-count, #8382, admitted-refusal metric, moka determinism fallback* → All addressed (Phase 2 step 6 updates the `SurbRingBuffer` doc; single-cache source of truth; Entry admit justified; count budget with the ~400 B estimate marked unmeasured; #8382 referenced in scope; `admitted_overflow` event; poll/retry note in the testing plan).

**PIX/Session-semantics critic**
- *[blocker] Budget refuses admitted → drops shares* → Fixed (quarantine-only budget).
- *[blocker] Marker/ring desync → second ring in quarantine* → Fixed: marker cache deleted; `admitted_surbs_per_pseudonym` is the single source of truth.
- *[major] Marker sized by `max_pseudonyms`* → Fixed: `admitted_max_pseudonyms`.
- *[major] `admit` capacity `.max(1)` yields 1* → Fixed: floored at `quarantine_rb_capacity`; `expected == 0` treated as unknown.
- *[major] Promotion not atomic; `with_capacity` cannot recover pre-admit evictions* → Fixed: routing+promotion on the single cache under the ring lock; pre-admit overflow documented as recoverable via `find_surb` consulting quarantine and the re-check migration.
- *[major] `resync_budget` CPU* → Fixed (removed).
- *[major] Budget refusal reports `evicted_shares: 0`* → Fixed: the refused batch's real share count is computed.
- *[minor] Budget-refused newer generation never reaches `push`* → Documented as accepted.
- *[major] Admitted capacity vs announced target reconciliation* → Fixed: Exit uses `announced_surb_target` at commit; Entry uses its own `maximum_surb_buffer_size`.
- *[minor] Entry full-capacity claim unproven* → Fixed: the Entry-side one-RTT window is stated and covered by quarantine.
- *[minor] `ring_len` ambiguity if rings in both caches* → Fixed: `ring_len` sums both.
- *[minor] `EvictedCache` variant insertion position* → Fixed: appended at the end of enum and `ALL`; the existing contiguity test guards it.
- *Missing: admitted-class bound, marker/ring desync decision, re-admit policy, pre-admit loss test, `pseudonyms_lifetime ≥ idle_timeout` validation, byte-vs-count, drain/retire policy, `set_surb_admission` never-called default* → All addressed (structural bound; single source of truth; test 5; test 4 + the re-check migration; `retire` fires on every teardown path so the validation is moot; count budget with the estimate marked; retire is idempotent and drain-safe; the silent-default risk is covered by `admitted_overflow`/the startup check — a never-called setter leaves every Session on the 16-cap quarantine, which the `admissions_count` metric and the ERROR-level `SurbRing` listener surface).
