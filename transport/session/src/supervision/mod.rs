//! PIX supervision for incoming PIX-enabled sessions.
//!
//! This module implements the **Exit-side** supervision logic for sessions
//! using the Packet Information eXtension (PIX) protocol.  The Exit node
//! runs a deterministic supervisor that tracks each *Secret Sharing Aggregate*
//! (SSA) through a well-defined lifecycle, enforcing timeouts, deposit
//! sufficiency, recovery progress, and fault tolerances.  Egress data
//! packets are gated behind a concurrent `ServiceGate` that allows
//! bounded predeposit service before funding and a ceiling-limited
//! post-funding path.
//!
//! # Why supervision is needed
//!
//! PIX sessions use SSAs: cryptographic aggregates that distribute the
//! cost of a deposit across many packets sent from the Entry to the Exit.
//! Each SSA requires:
//!
//! 1. A **commitment** from the Entry (polynomial coefficients) that the Exit can verify.
//! 2. A **deposit** to an SSA-specific on-chain address before the Exit fully trusts the SSA.
//! 3. **Recovery** of the SSA shares from data packets as they arrive.
//!
//! Without supervision, a misbehaving or stalled Entry can hold a session
//! slot indefinitely without ever funding or completing recovery — a
//! resource-exhaustion vector.  The supervisor enforces hard per-SSA and
//! per-session bounds so the Exit can reclaim resources deterministically.
//!
//! # Architecture
//!
//! The module is split into three components:
//!
//! | Component | File | Role |
//! |---|---|---|
//! | `SessionPixSupervisor` | `supervisor` | Pure state machine — no I/O, no async, no spawning.  Driven by explicit `Instant` timestamps and service-gate snapshots. |
//! | `ServiceGate` | `gate` | Concurrent, lock-free egress gate.  Before funding: bounded predeposit budget.  After funding: ceiling on packets served without SSA recovery progress.  Callers park on a generation-counter waker. |
//! | Worker loop | `worker` | Per-session actor that bridges the pure supervisor to async reality.  Receives commands via a backpressured channel, manages the deadline timer, and forwards supervisor actions to the caller. |
//!
//! The [`SlotNotify`](crate::utils::SlotNotify) multi-waker primitive is shared from [`utils`] and used
//! by `ServiceGate` to park and wake callers without a tokio dependency.
//!
//! ## The `SessionPixSupervisor` state machine
//!
//! The supervisor tracks each SSA through these phases:
//!
//! ```text
//! RequestSsa  ──►  SsaRequestSent  ──►  AwaitingCommitment
//!                                          │
//!                                     CommitmentVerified
//!                                          │
//!                                      AwaitingDeposit
//!                                          │
//!                                     DepositConfirmed (≥ expected)
//!                                          │
//!                                        Recovering
//!                                          ├── (idle re-arms when no service)
//!                                          ├── hard deadline is immutable
//!                                          └── progress resets idle timer
//!                                          │
//!                                     Recovered (tombstone phase)
//!                                          │
//!                                     tombstone expiry → RetireSsa
//! ```
//!
//! **Key deadlines** (all configurable via [`SupervisorConfig`]):
//!
//! * **Commitment timeout** — time from `SsaRequestSent` to `CommitmentVerified`.
//! * **Deposit timeout** — time from `CommitmentVerified` to a sufficient deposit.
//! * **Recovery idle** — time without *any share arriving* while service is being consumed. **Service-gated**: if no
//!   packets were served since the last progress snapshot, the timer re-arms instead of closing (prevents a
//!   slow-but-honest Entry from being disconnected). Deliberately not "useful progress": a conforming Entry emits a
//!   whole window's surplus in one run, none of it useful, and treating that as silence closes honest Sessions.
//! * **Recovery hard deadline** — absolute per-SSA backstop, never extended. This is a resource guard (session slot +
//!   reconstructor memory), not a liveliness mechanism.
//!
//! Both recovery clocks start when the cycle reaches the **front of its batch**, not when its deposit
//! confirms. A batch is served in index order — the Entry's emission window is clamped to one cycle —
//! so a cycle behind the front is queued, not stalled, and arming at funding would measure the queue
//! wait instead of the recovery. The front cycle is still on the clock from funding, so a
//! funded-but-never-served cycle is caught as before.
//!
//! **Fault tracking** — the supervisor tracks unverifiable shares via the
//! `UnverifiableShares` event (observed as absolute per-SSA totals that may
//! arrive from multiple concurrent ack processing batches).  It charges only
//! the delta from the maximum seen so far, preventing stale or out-of-order
//! snapshots from double-counting.  Limits exist per-SSA and per-session.
//!
//! **Rolling SSAs** — to maintain continuity, the supervisor requests the *next* batch when the
//! current one is "almost recovered" (early threshold reached) or fully recovered, so the commitment
//! and deposit work for the next cycle overlaps the tail of the current one. With a batch, only its
//! **last** cycle may ask: the request flags are per-cycle, so without that gate every member would
//! answer its own early signal with a batch of its own, compounding into `ssas_per_request` batches
//! per batch. The last rather than the first, because a batch is served in index order and gating on
//! the first would ask a whole batch too early. Retiring the cycle holding the gate hands it to the
//! newest survivor, or a batch whose last cycle lost its deposit would strand with no successor.
//!
//! Live cycles at any moment are therefore `ssas_per_request` plus whatever the overlap has already
//! requested, each with its own reconstructor state, plus tombstones inside their retention window.
//!
//! ## The `ServiceGate` — egress gating
//!
//! Every egress data packet from the Exit back to the Entry must pass
//! through the `ServiceGate` via `acquire`:
//!
//! ### Pre-funding (predeposit)
//!
//! Before the first deposit is confirmed, a provisional budget lets the Exit answer the Entry for a
//! bounded number of packets, so that the application's opening exchange is not held up for as long
//! as an on-chain deposit takes to confirm.  The budget is
//! `min(target_useful_shares - 1, max_predeposit_packets)`; at production dimensions the first term
//! is in the hundreds of thousands, so the configured cap is what binds and the `min` only matters
//! for the small dimensions used in tests.
//!
//! When the budget is exhausted, `acquire` parks the caller on a
//! `SlotNotify` future.  A concurrent `release_service`,
//! `notify_progress`, or `poison`
//! wakes all parkers.
//!
//! ### Strict prepay (`max_predeposit_packets = 0`)
//!
//! Zero is a supported setting, and it means the Exit serves nothing at all until a sufficient
//! deposit is confirmed: the first egress data packet parks, and the Entry has to commit and fund
//! before a single payload byte flows back to it.
//!
//! This does not deadlock, because nothing on the path to funding passes through the gate:
//!
//! * The `SsaRequest` goes out on the [`SessionManager`]'s own message sender, not the Session's gated sink.
//! * The Entry's commitment travels Entry→Exit, and the deposit is on-chain.
//! * The SURB-level keep-alive stream is deliberately left ungated, precisely so an exhausted budget cannot silence the
//!   signal that keeps the Session fundable. Under strict prepay this carries more weight than it does at a non-zero
//!   budget: with no egress at all, the keep-alives are the *only* traffic reaching the Entry, and therefore the only
//!   thing resetting the idle timer on the Entry's own session slot. An operator who both sets this budget to zero and
//!   disables `surb_balance_notify_period` gives the Entry nothing to stay alive on, and it will evict the Session
//!   before a deposit can land.
//!
//! So the Entry can always be asked for a deposit and can always answer, whatever the budget is.
//! What a zero budget costs is latency, not liveness: the application stalls until the deposit
//! confirms, instead of proceeding optimistically. If the deposit never arrives, the deposit
//! deadline closes the Session and `poison` fails the parked writer rather
//! than leaving it pending.
//!
//! ### Post-funding (ceiling)
//!
//! Once the first deposit is confirmed and the supervisor emits
//! `ReleaseService`, the gate flips
//! to funded mode.  It then enforces `max_served_without_progress`: a
//! ceiling on how many packets may be served between SSA recovery progress
//! events, as a defense-in-depth backstop even when the supervisor's
//! service-gated idle timer is alive.  Each `ProgressNotification`
//! resets the ceiling by snapshotting the served counter as the new watermark.
//!
//! The gate is implemented with lock-free atomics and CAS loops.  It uses
//! the generation-counter `SlotNotify` to avoid the two classic
//! race conditions of waker-vector approaches:
//!
//! 1. **Latent wake** — notification between future creation and first `poll()` is caught because the generation was
//!    captured at creation time and compared on `poll()`.
//! 2. **Spurious `Ready`** — a second `poll()` of an already-registered future re-checks the generation; if unchanged
//!    it stays `Pending`.
//!
//! ## The Worker — bridging pure logic to async
//!
//! `spawn_supervisor_worker` creates the `SessionPixSupervisor`,
//! the `ServiceGate`, and a bounded async command channel.  It returns
//! a `SessionPixSupervisorHandle` (cloneable, for sending events) and an
//! `ActionRx` receiver (for driving actions).
//!
//! The worker loop:
//!
//! 1. Reads the next deadline from the supervisor.
//! 2. If the deadline has already expired, calls `handle_deadline` immediately.
//! 3. Otherwise, waits on the command channel with a timeout set to the remaining deadline duration.
//! 4. On command received → calls `handle_event` or `action_result`.
//! 5. On timeout → calls `handle_deadline`.
//! 6. Forwards resulting actions to the action channel (non-blocking `try_send`).
//!
//! **Coalescing** — `ProgressNotification` actions are coalescible: when
//! the action channel is transiently full, they are dropped rather than
//! blocking or failing the worker.  They are idempotent and the next
//! notification will replace the missed one.
//!
//! All other actions (`RequestSsa`, `ReleaseService`, `RetireSsa`, `Close`)
//! are non-coalescible — if they cannot be delivered, the channel is
//! genuinely wedged and the worker fails the session.
//!
//! ## Integration with [`SessionManager`]
//!
//! ### Exit side (incoming sessions)
//!
//! When `handle_incoming_session_initiation` processes a session request
//! with `Capability::UsePIX`:
//!
//! 1. Validates the offered PIX parameters (polys, threshold, quota range).
//! 2. Spawns the supervisor worker via `spawn_supervisor_worker` (this emits the initial `RequestSsa` action and
//!    creates the gate).
//! 3. Reads the initial action, calls `send_ssa_request` on the wire, and notifies the supervisor of `SsaRequestSent`.
//! 4. Stores the supervisor handle and gate in the session slot (via `OnceLock`).
//! 5. Constructs the [`HoprSession`] — the egress adapter acquires the gate on every outgoing data packet.
//! 6. After session publication, spawns the **action driver task** that receives actions from `ActionRx` and executes
//!    them:
//!
//!    | Action | Driver behaviour |
//!    |---|---|
//!    | `RequestSsa` | Calls `send_ssa_request`, feeds back result to supervisor.  Tracks SSA in `SsaRetirementGuard` for Drop-safe cleanup. |
//!    | `ReleaseService` | Calls `gate.release_service()` — flips to funded mode. |
//!    | `ProgressNotification` | Calls `gate.notify_progress()` — resets ceiling watermark. |
//!    | `RetireSsa` | Calls `share_processor.retire_ssa`, aborts the deposit observer task. |
//!    | `Close` | Poisons gate, retires all SSAs, publishes close metric, removes session slot. |
//!
//! 7. PIX protocol events from the packet pipeline arrive via `dispatch_pix_event` and are forwarded to the supervisor
//!    as `SessionPixEvent::RecoveryProgress`, `UnverifiableShares`, `AlmostRecovered`, or `Recovered`.
//! 8. When a commitment becomes verifiable, a `PixDepositObserver` task loops on deposit confirmations, forwarding each
//!    as `DepositConfirmed` to the supervisor.
//!
//! ### Entry side (outgoing sessions)
//!
//! The Entry does **not** run a supervisor — the Exit is authoritative for
//! lifecycle decisions.  The Entry creates a session slot when
//! `new_session()` succeeds, but the slot's `pix_supervisor` and
//! `pix_egress_gate` remain unpopulated.  On receiving an `SsaRequest`
//! from the Exit, the Entry generates its client commitment via the
//! share generator, sends the commitment messages, and emits
//! `ReadyToDeposit` so the caller can fund the deposit address.
//!
//! ## Lifecycle sketch (Exit side)
//!
//! ```text
//! Session Initiation (Entry→Exit, with UsePIX flag)
//!     │
//!     ▼
//! handle_incoming_session_initiation
//!     │  validate PIX params
//!     │  spawn supervisor (emits initial RequestSsa)
//!     │  send SsaRequest on the wire
//!     │  install gate & handle in slot
//!     │  construct HoprSession (egress adaptor acquires gate)
//!     │  spawn action driver
//!     ▼
//! ┌────────────────────────────────────────────────────┐
//! │  Ongoing lifecycle (concurrent)                    │
//! │                                                    │
//! │  Entry → CommitmentVerified  → supervisor          │
//! │  Entry → DepositConfirmed    → supervisor (via     │
//! │                                PixDepositObserver) │
//! │  Packets → share_processor   → RecoveryProgress    │
//! │  Action: ReleaseService      → gate.release_service│
//! │  Action: ProgressNotification → gate.notify_progress│
//! │  Action: RequestSsa (next)   → send on wire        │
//! │  Action: RetireSsa           → reconstructor.retire│
//! │  Action: Close               → poison + teardown   │
//! └────────────────────────────────────────────────────┘
//! ```
//!
//! # Configuration
//!
//! ## What each parameter prevents
//!
//! Every field of [`SupervisorConfig`], the failure it exists to stop, and what shipping default it
//! carries. Nothing here is a preference: each one is the only thing standing between the Exit and a
//! specific way of taking service without paying for it.
//!
//! | Parameter | Default | What it prevents |
//! |---|---|---|
//! | `ssas_per_request` | 1 | Nothing by itself — an exposure dial. Raising it amortises the request round trip over several cycles, at the price of multiplying the *unfunded* exposure to that many quotas, and it scales the two deadlines below. |
//! | `max_failed_cycles` | 1 | An Entry losing one cycle per batch indefinitely while a single funded sibling holds the Session open. One loss is survivable, the second closes the Session. Only reachable above a batch of one, where the failing cycle is not always the last one standing. |
//! | `max_ssa_delivery_time` | 20 s | An Entry that accepts a request and never delivers the commitment set, holding a session slot and a reconstructor cycle that can never be funded. |
//! | `max_deposit_wait` | 60 s | An Entry that commits but never deposits — typically after it has already drawn the predeposit budget. |
//! | `max_recovery_idle` | 60 s | An Entry, or a colluding first return relayer, consuming service while returning no shares. Service-gated, so a Session that is merely quiet is never punished. |
//! | `max_recovery_time` | 2 h | A cycle that dribbles just enough progress to refresh the idle timer forever. A resource backstop for the slot and the reconstructor state, *not* the anti-drip rule. It must clear a whole cycle at the widest dimensions the node accepts — 778 240 packets, ~72 min, at the defaults — or it closes honest Sessions instead. |
//! | `max_unverifiable_shares_per_ssa` | 0 | Serving on past a polynomial whose share set failed to open its commitment. That already dooms the cycle, so tolerating it only buys the Entry more unpaid packets. |
//! | `max_unverifiable_shares_per_session` | 0 | The same failure recurring once per cycle, which a per-cycle limit alone would reset each time. |
//! | `max_off_front_share_fraction` | 0.25 | An Entry spreading a batch's shares across all of its cycles, taking `ssas_per_request` quotas of service while completing none of them — and a cycle short of completion pays nothing at all. |
//! | `min_share_order_sample` | 16384 | Convicting on a thin sample: the shares that legitimately cross a cycle boundary out of order while in flight. |
//! | `max_predeposit_packets` | 10000 | Bounds what an Entry that never funds can extract. `0` is supported and means strict prepay. |
//! | `max_served_without_progress` | 2048 | Packets served with no share of *any* kind coming back — in *packets*, so unlike the idle timer the bound does not move with the Session's rate. Counts `shares_seen`, so a conforming Entry's surplus resets it; see below. |
//! | `tombstone_retention_window` | 30 s | A late acknowledgement arriving after its cycle's record is gone, with nothing left to attribute it to. |
//! | `min_deposit` | 0 | A dust deposit counting as funding and releasing service. Top-ups accumulate, so this is a floor on the total, not on any one transfer. |
//!
//! ## Constraints between parameters
//!
//! [`validate_pix_supervision`] enforces, at config-load time and against the reconstructor config
//! actually in use: `max_recovery_idle >= max_ack_await_time`; `tombstone_retention_window >=
//! max_ack_await_time`; `max_recovery_idle < unused_verifier_lifetime`; `ssas_per_request` in
//! `1..=MAX_SSA_BATCH_SIZE`; both scaled deadlines under 24 h; non-zero durations; a share fraction in
//! `0.0..=1.0`; and non-zero `max_served_without_progress`, `min_share_order_sample` and
//! `max_failed_cycles`.
//!
//! ### The surplus run, and why it no longer constrains anything
//!
//! Emission is round-robin over a window of up to `hopr_protocol_pix::SHARE_EMISSION_WINDOW` (256)
//! polynomials advancing in lockstep. They reach `threshold` on the same pass and then take their
//! surplus shares together, so every block ends with `surplus_shares × window` consecutive packets
//! carrying no *useful* share — **8192 at the shipped dimensions**, against a
//! `max_served_without_progress` of 2048.
//!
//! That used to be a live constraint on two parameters, and an unmeetable one: the ceiling and the
//! idle timer both counted useful shares, so the run looked exactly like an Entry gone silent. The
//! gate would park the writer partway through, a parked writer spends no SURBs, and the idle rule
//! could not rescue it — its re-arm branch needs *no service consumed* since the last progress, and
//! 2048 packets were. An honest Session closed with `RecoveryIdle` one emission block into a cycle.
//!
//! Both now read [`SsaRecoveryProgress::shares_seen`](hopr_protocol_pix::SsaRecoveryProgress::shares_seen),
//! which counts every share the cycle accepts — surplus included, up to the negotiated budget per
//! polynomial. The run resets the ceiling and refreshes the idle deadline exactly like the useful
//! shares around it, so **neither parameter has to be sized against the dimensions any more.** What
//! `max_served_without_progress` still bounds is genuine silence: packets served with no share of any
//! kind coming back.
//!
//! The one dimension-dependent property that remains is not a supervisor parameter at all: a
//! polynomial's whole emission, `threshold + surplus`, should fit one peer deferral bucket
//! (`hopr_protocol_pix::MAX_DEFERRED_ACKS_PER_POLYNOMIAL`, 128), or acknowledgements arriving before
//! the cycle's commitments install can be dropped. The shipped 96 fits; both halves are a byte wide
//! on the wire, so the sum can reach 510. It is a sufficient condition rather than a required one —
//! only the pre-install prefix is deferred — which is why it is asserted on the defaults rather than
//! enforced.
//!
//! ## Worked example
//!
//! A profile of **5 Mbps sustained per direction** and **5–6 s on-chain settlement** for an SSA
//! deposit. `HoprPacket::PAYLOAD_SIZE` is 1038 B, so 5 Mbps is ~602 packets/s of return traffic —
//! one SURB and one share each.
//!
//! **Dimensions first**, because every supervisor value derives from them. `2048 × 64` with the
//! derived surplus of 16 gives 131 072 useful shares out of 163 840 emitted, and a cycle of 163 840
//! packets — about **4.5 min** at this rate. The quota is priced on all of them, surplus included, so
//! it is `2048 × 80 × 1038 B` = **162.2 MiB**, which is exactly the bottom of the default
//! `quota_range` (a quarter of the 648.8 MiB the default dimensions imply), so no range change is
//! needed. The default `8192 × 64` would make that 18.1 min per cycle, which at 6 s settlement is a
//! long time to leave one cycle's traffic unsettled for no benefit. Keep it a multiple of 256 so the
//! emission window never narrows.
//!
//! | Parameter | Value | Why, at this profile |
//! |---|---|---|
//! | `PixGlobalConfig::num_ssa_parts` (Entry) | 2048 | 162.2 MiB quota, 4.5 min cycle; multiple of the emission window |
//! | `PixGlobalConfig::ssa_part_size` (Entry) | 64 | Shipped threshold; with the surplus below, it is what fixes the quota |
//! | `PixGlobalConfig::additional_shares` (Entry) | unset | Derives to 16 — a quarter of the threshold, i.e. a 1.25× surplus factor and a fifth of the quota. Setting it explicitly buys loss tolerance and charges the Entry for it. Keep `ssa_part_size + additional_shares` inside one deferral bucket (128) |
//! | `ssas_per_request` | **1** | The 85 % early signal leaves a 24 627-packet runway — **41 s** — before the cycle drains, against 6 s of settlement plus a commitment round trip. There is nothing to amortise, and a batch of `n` would multiply the unfunded exposure to `n × 162.2 MiB` |
//! | `max_ssa_delivery_time` | 20 s | 2048 commitments ship in ~71 forward packets, well under a second; the margin covers commitment generation |
//! | `max_deposit_wait` | 30 s | 5× the 6 s settlement, leaving room for the Entry to notice `ReadyToDeposit`, submit, and the observer to see it |
//! | `max_predeposit_packets` | 4096 | ~6.8 s of service, matching expected settlement rather than the deadline. This is exactly what is lost to an Entry that never funds — 4.2 MB. Use `0` for strict prepay and accept a ~6 s stall at session start |
//! | `max_served_without_progress` | 2048 | Shipped value, and no longer dimension-dependent: the surplus run resets it like any other share, so this bounds genuine silence only |
//! | `max_recovery_idle` | 60 s | Shipped value. Satisfies `>= max_ack_await_time` and `< unused_verifier_lifetime`. It no longer has to cover the surplus run — that resets it — so what it now implies is only that a Session returning *nothing at all* for a minute is closed |
//! | `max_recovery_time` | 2 h | Resource backstop only. A cycle needs 272 s at full rate, so 2 h implies a floor of ~23 packets/s (~0.19 Mbps) — deliberately far below the idle rule, which is the instrument that should bind |
//! | `max_off_front_share_fraction` | 0.25 | Shipped value. A conforming Entry sits near 0; two-way spreading is 0.5 |
//! | `min_share_order_sample` | 16384 | Shipped value, and safe here: with emission clamped to one cycle the front cycle is essentially complete before any off-front progress is possible, so even a loss-doomed cycle peaks near 15 % against the 25 % ceiling |
//! | `max_unverifiable_shares_per_ssa` / `_per_session` | 0 / 0 | Shipped values; a failed polynomial has already doomed the cycle |
//! | `tombstone_retention_window` | 60 s | 2× the reconstructor's 30 s ack window |
//! | `max_failed_cycles` | 1 | Shipped value, and inert at this batch size of one — the failing cycle is always the last one standing, which closes the Session first |
//! | `min_deposit` | ≥ one quota's value | 162.2 MiB at the operator's `price_per_byte`; anything less releases service for a fraction of a cycle |
//!
//! Related settings outside [`SupervisorConfig`] that this profile also pins:
//!
//! | Parameter | Value | Why |
//! |---|---|---|
//! | `SurbStoreConfig::rb_capacity` | 100 000 | An overwritten SURB is a permanently lost share, so the buffer must clear the balancer's target with room for overshoot — 14× here |
//! | `SurbBalancerConfig::target_surb_buffer_size` | 7 000 | ~11.6 s of return traffic at 5 Mbps; must cover the forward round trip or the Exit starves mid-cycle |
//! | `SessionManagerConfig::maximum_surb_buffer_size` | 10 000 | Ceiling the balancer may be steered to |
//! | `SsaReconstructorConfig::max_ack_await_time` | 30 s | Bounds how long an unacknowledged share is held; both `max_recovery_idle` and `tombstone_retention_window` must clear it |
//! | `SsaReconstructorConfig::unused_verifier_lifetime` | 1800 s | Must exceed `max_recovery_idle`, so the supervisor gives up on a stalled cycle before the reconstructor reclaims what it would need to finish |
//! | `SsaReconstructorConfig::early_recovery_threshold` | 0.85 | Sets the 54 s pipelining runway quoted above. Bounded below by `MIN_EARLY_RECOVERY_THRESHOLD` and equal to it today: the Entry's successor gate is computed at that floor, so a lower value asks for the next batch before any conforming Entry admits the request. Raising it shortens the runway |
//! | `PixGlobalConfig::max_ssas_per_request` (Entry) | 2 | Must be ≥ every peer Exit's `ssas_per_request`; not negotiated, and an over-cap batch is refused in full |
//! | `IncomingSessionPixConfig::max_live_cycle_bytes` | 3 GiB | Shipped value. At `2048 × 64 + 16` a Session reserves ≈20.5 MiB, so this profile admits ≈150 concurrent PIX Sessions rather than the ≈37 the default dimensions imply. It is the ceiling on live reconstructor state, and it — not `maximum_managed_sessions` — is what decides how many PIX Sessions this Exit accepts |

use std::time::Duration;

use hopr_api::{HoprBalance, types::internal::prelude::HoprPseudonym};
use hopr_protocol_pix::{SsaId, SsaReconstructorConfig, SsaRecoveryProgress};

use crate::errors::TransportSessionError;

mod gate;
mod supervisor;
mod worker;

// ---------------------------------------------------------------------------
// SupervisorConfig
// ---------------------------------------------------------------------------

/// Configuration for the `SessionPixSupervisor`.
///
/// Reachable from a node's configuration file as
/// `incoming_session_pix_config.supervision`. The fields interact — with each other and with the
/// reconstructor's own lifetimes — so a set of them is only meaningful as a whole; that is what
/// [`validate_pix_supervision`] checks, and the node's config validator runs it at load time.
///
/// A worked set of values, and what each field defends against, is written out under
/// "Configuration" in the `supervision` module documentation — the module is crate-private, so it is
/// named rather than linked here.
#[serde_with::serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SupervisorConfig {
    /// Number of SSAs the Exit asks the Entry to commit to in a single `SsaRequest`.
    ///
    /// Batching amortizes the request round trip over several deposit cycles. It lives here rather
    /// than beside the other Exit settings because this is what acts on it: the supervisor allocates
    /// the indices, and it scales both deadlines below by this factor.
    ///
    /// Two things it costs, both linear in the value:
    ///
    /// * The Exit holds that many live reconstructor cycles at once — worst case ≈41 MiB each at the profiled
    ///   dimensions, per `hopr_protocol_pix::peak_cycle_bytes`. That cost is reserved up front against
    ///   `IncomingSessionPixConfig::max_live_cycle_bytes`, so this value divides the number of PIX Sessions the node
    ///   will admit.
    /// * The unfunded exposure. The supervisor never lets a *second* batch go out while the first is still unfunded,
    ///   but within one batch every cycle is unfunded at once — so the ceiling is this many SSA quotas rather than
    ///   one.
    ///
    /// **Must not exceed the peer Entry's `max_ssas_per_request`.** The batch size is not negotiated
    /// — `StartSession.additional_data` is fully allocated (PIX dimensions in the upper 32 bits, SURB
    /// balancer target in the lower 32), so the Entry cannot advertise its cap and the Exit cannot
    /// learn it. An Entry that finds a batch too large refuses it in full and replies with an
    /// `UnacceptablePixParams` `SessionError`, which closes the Session on both sides in about a round
    /// trip. Every such Session is still lost; the reply only makes the failure immediate and
    /// attributable instead of surfacing as a commitment timeout.
    ///
    /// Range-checked by [`validate_pix_supervision`] and clamped where it is read, so it can never
    /// exceed `MAX_SSA_BATCH_SIZE`.
    ///
    /// Default: 1, which reproduces the unbatched exchange byte-for-byte.
    #[default(1)]
    pub ssas_per_request: usize,

    /// Cycles this Session may lose without recovering them before it is closed. The next one closes
    /// it.
    ///
    /// Counted over the whole Session, not per batch, because a retired cycle leaves no trace on its
    /// siblings: without a cumulative count an Entry can lose one cycle per batch forever while a
    /// single funded sibling keeps the Session alive, paying for a fraction of what it is served.
    ///
    /// Only reachable with [`ssas_per_request`](Self::ssas_per_request)` > 1`. At the shipping batch
    /// of one the failing cycle is always the last one standing, and that closes the Session before
    /// this is ever consulted — so nothing changes at the defaults.
    ///
    /// A recovered cycle never counts: it leaves through tombstone expiry, not through the failure
    /// path.
    ///
    /// Default: 1 — one lost cycle is survivable, which is what the retire-and-hand-on-the-successor
    /// -gate path exists for, and a second is not.
    #[default(1)]
    pub max_failed_cycles: usize,

    /// Maximum time to wait for the SSA to be fully committed.
    ///
    /// Together with [`max_deposit_wait`](Self::max_deposit_wait) this bounds how long a Session may
    /// be served unincentivized: the Entry has to be able to deliver its SSA commitment before it
    /// can be asked to pay for it.
    ///
    /// Multiplied by [`ssas_per_request`](Self::ssas_per_request) when the deadline is armed: a batch
    /// asks the Entry for that many commitment sets in answer to one request, and all of their clocks
    /// start together, so holding each to a single cycle's budget would fail an Entry that is
    /// answering in order.
    ///
    /// Default: 20 s.
    #[default(Duration::from_secs(20))]
    #[serde(with = "humantime_serde")]
    pub max_ssa_delivery_time: Duration,

    /// Maximum time to wait for a deposit after the commitment is verifiable.
    ///
    /// Also multiplied by [`ssas_per_request`](Self::ssas_per_request). Each cycle's clock starts at
    /// its own commitment, which staggers them somewhat — but a batch's commitments arrive
    /// back-to-back while an Entry funding them serially finishes the last one that many deposits
    /// later, so the stagger does not cover it.
    ///
    /// Default: 60 s.
    #[default(Duration::from_secs(60))]
    #[serde(with = "humantime_serde")]
    pub max_deposit_wait: Duration,

    /// Maximum idle time during recovery when service is being consumed.
    ///
    /// Gated on service consumption — if no packets were served, the timer
    /// re-arms instead of closing.
    ///
    /// Default: 60 s.
    #[default(Duration::from_secs(60))]
    #[serde(with = "humantime_serde")]
    pub max_recovery_idle: Duration,

    /// Absolute per-SSA recovery deadline (immutable once set).
    ///
    /// This is a **resource backstop** (session slot + reconstructor memory),
    /// not the anti-drip mechanism. The service-gated idle rule is.
    ///
    /// ## Why two hours, and not one
    ///
    /// It has to cover the *whole* of a cycle at the dimensions the node will accept, and at the
    /// default dimensions one cycle does not fit in an hour. Emission runs in lockstep over windows
    /// of [`SHARE_EMISSION_WINDOW`](hopr_protocol_pix::SHARE_EMISSION_WINDOW) polynomials, so with
    /// 8192 polynomials the last useful share of the cycle lands at
    ///
    /// ```text
    /// (31 full windows x 96 emitted + 1 window x 64 useful) x 256 = 778 240 packets
    /// ```
    ///
    /// which is about **72 minutes** at the 1.5 Mbps per-Session cap this crate documents, before
    /// any mixing latency or loss. A one-hour ceiling closes an honest, fully saturated Session at
    /// the default configuration — with the cycle roughly five sixths recovered and therefore worth
    /// nothing, since the SSA is the sum of every polynomial's constant term.
    ///
    /// Two hours is the value the worked profile in the module documentation already used, and it
    /// keeps this instrument where it belongs: far enough out that
    /// [`max_recovery_idle`](Self::max_recovery_idle) is what actually binds.
    ///
    /// The clock starts when the cycle reaches the front of its batch, which is up to one
    /// predecessor's surplus tail — ~8192 packets, ~45 s at that rate — before it can make any
    /// progress of its own. Negligible against two hours. Arming on first progress instead is
    /// rejected because a funded cycle that is never served must still be caught.
    ///
    /// `HoprProtocolConfig::validate` checks this against the dimensions the node will actually
    /// accept, so a raised `quota_range` that outgrows it is refused at load rather than discovered
    /// one closed Session at a time.
    ///
    /// Default: 2 hours.
    #[default(Duration::from_secs(7200))]
    #[serde(with = "humantime_serde")]
    pub max_recovery_time: Duration,

    /// Unverifiable shares tolerated for one SSA before the session is closed.
    ///
    /// Zero means the first one closes, and that is deliberate rather than austere. A share is no
    /// longer checked on arrival — the non-constant coefficient commitments that made per-share
    /// verification possible were dropped, so a failure now means a whole polynomial's share set did
    /// not open its commitment. Two things follow:
    ///
    /// * A failed polynomial already dooms the cycle, since the SSA is the sum of *every* polynomial's constant term.
    ///   There is no partial recovery to preserve by tolerating it.
    /// * The failure surfaces on the `threshold`-th share of that polynomial, so the Exit has already served that many
    ///   packets by the time it learns. Closing on the first failure is what keeps the exposure at `threshold` packets
    ///   rather than a multiple of it.
    ///
    /// Kept as a limit rather than hard-coded so the tolerance stays a one-value decision.
    ///
    /// Default: 0 (the first closes).
    #[default(0)]
    pub max_unverifiable_shares_per_ssa: u64,

    /// Unverifiable shares tolerated across the whole session before it is closed.
    ///
    /// Distinct from the per-SSA limit so a steady trickle of one failure per cycle still escalates,
    /// rather than resetting with each new SSA. At the default per-SSA limit of zero this never gets
    /// the chance to fire; it earns its keep only if that limit is raised.
    ///
    /// Default: 0.
    #[default(0)]
    pub max_unverifiable_shares_per_session: u64,

    /// Largest share of recovery progress that may land on a cycle *other* than the one at the front
    /// of the batch, as a fraction of all progress since that cycle took the front.
    ///
    /// ## What it defends
    ///
    /// A batch only de-risks the Exit if its cycles are served one at a time, so that recovering the
    /// first pays for the traffic that earned it. An Entry that instead spread a batch's shares across
    /// all `ssas_per_request` cycles could take `ssas_per_request × quota` of service while leaving
    /// every cycle short of recovery — and a cycle short of recovery pays nothing at all, since the SSA
    /// is the sum of *every* polynomial's constant term. That turns the batch knob from cost-neutral
    /// into an n-fold amplifier of unpaid traffic.
    ///
    /// Nothing else sees it. The service gate and [`max_recovery_idle`](Self::max_recovery_idle) both
    /// measure the *absence* of progress, and a spreading Entry makes progress on every cycle
    /// continuously — including the front one, whose idle timer it therefore keeps refreshing. It
    /// simply never finishes one.
    ///
    /// ## Why a fraction and not a count
    ///
    /// Out-of-order *arrival* is not an Entry property. SURBs cross the mixnet outbound and their
    /// acknowledgements cross it again inbound, so arrival order is a permutation of emission order
    /// whose width is roughly `delay_range × packet rate`. An absolute tolerance would therefore have
    /// to be re-derived every time the mixer is reconfigured, and — being cumulative — would drift
    /// into closing honest Sessions given enough cycle boundaries.
    ///
    /// A ratio has neither problem: a permutation of bounded width is a vanishing *fraction* of a
    /// growing window, so widening `delay_range` cannot move it, and the denominator grows with the
    /// numerator so nothing accumulates. It is also independent of packet **rate** (both terms scale
    /// together, so unlike a wall-clock bound it implies no throughput floor), of **loss** (which hits
    /// every cycle alike), and of **`surplus_shares`** — which matters even now that the surplus is
    /// negotiated rather than Entry-private. The Exit knows the budget, not the spend: how many of a
    /// cycle's emitted shares turn out to be useful depends on the loss the Entry actually met, so no
    /// expected shares-per-packet ratio follows from the parameter. Comparing the batch's cycles
    /// against each other cancels it.
    ///
    /// ## The value
    ///
    /// A conforming Entry sits at ~0: with emission clamped to one cycle
    /// ([`hopr_protocol_pix::SHARE_EMISSION_WINDOW`]) the only off-front progress is boundary
    /// reordering. Spreading across `n` cycles gives `(n − 1) / n` — 0.5 at two cycles, 0.67 at three.
    /// 0.25 separates those with room on both sides.
    ///
    /// It bounds rather than eliminates: an Entry may divert up to this fraction and stay legal, so
    /// worst-case unpaid exposure becomes ~`1 / (1 − f)` cycles instead of `n`. Alternating — serving
    /// the front cycle honestly, then dumping on later ones — buys nothing, because the accounting only
    /// resets when the front cycle *completes*, which means it was paid for.
    ///
    /// Default: 0.25.
    #[default(0.25)]
    pub max_off_front_share_fraction: f64,

    /// Progress that must accumulate before
    /// [`max_off_front_share_fraction`](Self::max_off_front_share_fraction) is judged at all, in useful
    /// shares.
    ///
    /// A sample-size floor, not a tolerance: it does not bound how much a cheat can extract — the
    /// fraction does that — it only refuses to convict on thin evidence.
    ///
    /// Sized by the one case that would otherwise produce a false positive: a cycle made
    /// *unrecoverable* by loss. A polynomial that loses more than `surplus_shares` of its shares can
    /// never reach its threshold, so its cycle stops progressing while later ones continue, and the
    /// off-front fraction climbs to 1.0. That cycle is not something to close a Session over — it stops
    /// progressing, its service-gated [`max_recovery_idle`](Self::max_recovery_idle) deadline expires,
    /// it is retired, and the accounting resets on the new front. The sample floor only has to outlast
    /// that window: ~181 shares/s at the deployed 1.5 Mbps cap over 60 s is ~10 900, so 16 384 leaves
    /// ~1.5x headroom.
    ///
    /// Unlike an absolute tolerance this does not have to grow with Session length or track the mixer
    /// configuration — it is a floor on evidence, evaluated once and then continuously.
    ///
    /// Default: 16384 shares.
    #[default(16384)]
    pub min_share_order_sample: u64,

    /// Cap on the provisional predeposit service budget.
    ///
    /// This buys the application its opening exchange while the deposit confirms on chain; it is not
    /// needed for the Session to become fundable, so it is a latency-versus-exposure dial rather than
    /// a correctness requirement.
    ///
    /// **Zero is supported and means strict prepay**: the Exit answers nothing until a sufficient
    /// deposit is confirmed. Everything on the path to funding — the `SsaRequest`, the Entry's
    /// commitment, the deposit itself, and the SURB keep-alive stream — bypasses the egress gate, so
    /// a zero budget stalls the application without ever stalling the funding handshake. Deliberately
    /// *not* rejected by [`validate_pix_supervision`] for that reason, unlike
    /// [`max_served_without_progress`](Self::max_served_without_progress), where zero would wedge a
    /// Session that has already paid.
    ///
    /// The effective budget is `min(target_useful_shares - 1, max_predeposit_packets)`, so this value
    /// is the binding one at any realistic set of PIX dimensions.
    ///
    /// Default: 10000 packets.
    #[default(10000)]
    pub max_predeposit_packets: u64,

    /// Maximum packets served without a single share coming back, before the gate blocks further
    /// service as a defence-in-depth backstop.
    ///
    /// A ceiling enforced by `ServiceGate::acquire` after the gate is funded. Each
    /// [`crate::HoprSessionInPixEvent::RecoveryProgress`] event resets it.
    ///
    /// Those events now follow `shares_seen` rather than `useful_shares`, which is what makes a flat
    /// 2048 safe at any dimensions. Keyed on useful shares, this had to exceed
    /// `surplus x min(polys, SHARE_EMISSION_WINDOW)` — 8192 at the shipped dimensions, four times
    /// this value — because a conforming Entry emits a whole window's surplus in one contiguous run
    /// during which no share is useful. The gate parked the writer partway through, and a parked
    /// writer spends no SURBs, so nothing could arrive to release it. See
    /// [`SsaRecoveryProgress::shares_seen`](hopr_protocol_pix::SsaRecoveryProgress::shares_seen).
    ///
    /// What it bounds now is genuine silence, and that is rate-independent: it is a count of
    /// *packets*, so unlike [`max_recovery_idle`](Self::max_recovery_idle) it implies no throughput
    /// floor.
    ///
    /// Default: 2048 packets.
    #[default(2048)]
    pub max_served_without_progress: u64,

    /// How long to retain recovered-SSA tombstones for late events.
    ///
    /// Must be >= the reconstructor's `max_ack_await_time`.
    ///
    /// Default: 30 s.
    #[default(Duration::from_secs(30))]
    #[serde(with = "humantime_serde")]
    pub tombstone_retention_window: Duration,

    /// Minimum deposit amount required before the gate is released.
    ///
    /// A deposit confirmation below this amount is a no-op (the deposit
    /// deadline keeps running and further top-ups accumulate).  Set to zero
    /// (default) to accept any non-zero deposit.
    ///
    /// Default: zero (accepts any deposit).
    #[default(HoprBalance::zero())]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub min_deposit: HoprBalance,
}

// ---------------------------------------------------------------------------
// PixParams
// ---------------------------------------------------------------------------

/// PIX dimensions agreed upon during session negotiation.
///
/// Re-exported rather than defined here: this supervisor once carried its own `polys`/`threshold`
/// pair, which meant the same thing as the one a Session offers but named it differently and could
/// not be handed across the boundary without a field-by-field copy. `target_useful_shares` moved
/// onto the shared type with it.
///
/// That shared type is now the one the two nodes actually negotiate, down to the byte layout it is
/// packed into — so it carries a third field, the surplus, which the supervisor does not read. The
/// supervisor's own arithmetic is unchanged: the surplus is by definition the shares that arrive
/// after a polynomial is already complete, so it never enters
/// [`target_useful_shares`](PixParams::target_useful_shares).
pub use hopr_protocol_pix::PixParams;

// ---------------------------------------------------------------------------
// SessionPixEvent
// ---------------------------------------------------------------------------

/// Events consumed by the [`SessionPixSupervisor`](supervisor::SessionPixSupervisor).
#[derive(Debug, Clone)]
pub enum SessionPixEvent {
    /// The initial or next SSA request was successfully sent on the wire.
    SsaRequestSent(SsaId<HoprPseudonym>),
    /// A verifiable commitment was installed in the reconstructor.
    CommitmentVerified {
        ssa_id: SsaId<HoprPseudonym>,
        expected_deposit: Option<HoprBalance>,
    },
    /// Deposit for a specific SSA was confirmed with the given amount.
    DepositConfirmed {
        ssa_id: SsaId<HoprPseudonym>,
        amount: HoprBalance,
    },
    /// The deposit observer channel closed without delivering a confirmation.
    DepositObserverClosed(SsaId<HoprPseudonym>),
    /// Recovery progress snapshot from the reconstructor.
    RecoveryProgress(SsaRecoveryProgress<HoprPseudonym>),
    /// Early-recovery threshold reached.
    AlmostRecovered(SsaId<HoprPseudonym>),
    /// Full SSA recovery completed.
    Recovered(SsaId<HoprPseudonym>),
    /// Absolute per-SSA unverifiable-share count observation.
    UnverifiableShares {
        ssa_id: SsaId<HoprPseudonym>,
        observed_total: u64,
    },
}

// ---------------------------------------------------------------------------
// SessionPixAction
// ---------------------------------------------------------------------------

/// Actions emitted by the [`SessionPixSupervisor`](supervisor::SessionPixSupervisor) for the caller to execute.
#[derive(Debug, Clone)]
pub enum SessionPixAction {
    /// Request one or more new SSAs with the given dimensions.
    ///
    /// Always non-empty, and always contiguous ascending indices. The whole batch travels in a single
    /// `SsaRequest` — one message, one `params` field — because every SSA in a Session shares the
    /// negotiated dimensions. Carrying the set rather than one id is what keeps the batch atomic:
    /// the carrier either puts all of them on the wire or none, and a partial failure releases every
    /// registration it made.
    RequestSsa {
        ssa_ids: Vec<SsaId<HoprPseudonym>>,
        params: PixParams,
    },
    /// Release the service gate (from predeposit to funded mode).
    ReleaseService,
    /// Notifies the gate that SSA recovery made progress, resetting the
    /// served-without-progress ceiling.
    ProgressNotification,
    /// Close the session with the given reason.
    Close(SessionPixCloseReason),
    /// Retire a previously-used SSA from the reconstructor (idempotent).
    /// Emitted when an SSA's tombstone period expires so mid-session state
    /// does not accumulate.
    RetireSsa(SsaId<HoprPseudonym>),
}

// ---------------------------------------------------------------------------
// SessionPixCloseReason
// ---------------------------------------------------------------------------

/// Internal close reasons emitted by the supervisor.
///
/// These are mapped to public [`ClosureReason`](crate::types::ClosureReason) by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum SessionPixCloseReason {
    /// The commitment delivery deadline expired.
    CommitmentTimeout,
    /// The deposit deadline expired without a sufficient deposit.
    DepositTimeout,
    /// The deposit observer channel closed without delivering a confirmation.
    DepositObserverClosed,
    /// Service was consumed but no useful progress was made — service-gated idle.
    RecoveryIdle,
    /// The per-SSA hard recovery deadline expired.
    RecoveryDeadline,
    /// Too many unverifiable shares (per-SSA or session-limit exceeded).
    TooManyUnverifiableShares,
    /// Too much of the batch's recovery progress landed on cycles behind the front one — see
    /// [`SupervisorConfig::max_off_front_share_fraction`].
    BatchServedOutOfOrder,
    /// A counter observation decreased (protocol violation).
    CounterRegression,
    /// Internal inconsistency (e.g., mismatched target, event on unknown SSA).
    InvalidTransition,
    /// The SSA set drained (all SSAs expired/recovered without a successor).
    NoSsaRemaining,
    /// The supervisor action driver failed or was dropped.
    SupervisorUnavailable,
}

// ---------------------------------------------------------------------------
// Re-exports from submodules
// ---------------------------------------------------------------------------

// `GateClosed` is re-exported despite nothing in the crate naming it today: it is the error type of
// both [`ServiceGate::acquire`] and [`ServiceGate::try_acquire_sync`], and an error a caller cannot
// name is one it cannot match on or downcast to. `manager` currently only forwards it into an
// `io::Error`, which is why the import reads as unused.
#[allow(unused_imports)]
pub use gate::{GateClosed, ServiceGate};
pub use worker::{ActionRx, SessionPixSupervisorHandle, spawn_supervisor_worker};

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validates that [`SupervisorConfig`] and [`SsaReconstructorConfig`] are
/// mutually consistent.
///
/// Returns an error if constraints are violated.
pub fn validate_pix_supervision(
    cfg: &SupervisorConfig,
    reconstructor_cfg: &SsaReconstructorConfig,
) -> Result<(), TransportSessionError> {
    if !(1..=crate::MAX_SSA_BATCH_SIZE).contains(&cfg.ssas_per_request) {
        return Err(TransportSessionError::InvalidConfig(format!(
            "ssas_per_request ({}) must be between 1 and {}",
            cfg.ssas_per_request,
            crate::MAX_SSA_BATCH_SIZE
        )));
    }
    // Zero would close the Session on the first lost cycle *and* on the last one standing, which is
    // the same thing at the shipping batch size but silently disables batching above it. The
    // supervisor clamps to one where it reads this; rejecting here is what an operator sees.
    if cfg.max_failed_cycles == 0 {
        return Err(TransportSessionError::InvalidConfig(
            "max_failed_cycles must be non-zero".into(),
        ));
    }
    if cfg.max_ssa_delivery_time.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_ssa_delivery_time must be non-zero".into(),
        ));
    }
    if cfg.max_deposit_wait.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_deposit_wait must be non-zero".into(),
        ));
    }
    if cfg.max_recovery_idle.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be non-zero".into(),
        ));
    }
    if cfg.max_recovery_time.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_time must be non-zero".into(),
        ));
    }
    if cfg.tombstone_retention_window.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "tombstone_retention_window must be non-zero".into(),
        ));
    }
    if cfg.max_served_without_progress == 0 {
        return Err(TransportSessionError::InvalidConfig(
            "max_served_without_progress must be non-zero".into(),
        ));
    }
    // A fraction outside 0..=1 is unreachable rather than strict — the ratio can never exceed 1 — so a
    // typo would silently disable the check instead of tightening it.
    if !cfg.max_off_front_share_fraction.is_finite() || !(0.0..=1.0).contains(&cfg.max_off_front_share_fraction) {
        return Err(TransportSessionError::InvalidConfig(format!(
            "max_off_front_share_fraction ({}) must be a fraction between 0.0 and 1.0",
            cfg.max_off_front_share_fraction
        )));
    }
    // Zero would judge the very first off-front share, which mixnet reordering alone can produce.
    if cfg.min_share_order_sample == 0 {
        return Err(TransportSessionError::InvalidConfig(
            "min_share_order_sample must be non-zero".into(),
        ));
    }
    if cfg.max_recovery_idle < reconstructor_cfg.max_ack_await_time {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be >= max_ack_await_time".into(),
        ));
    }
    // An Exit below the floor asks for its next batch before any conforming Entry will admit the
    // request. The Entry's gate is computed at the floor precisely because this value does not travel
    // on the wire (see `MIN_EARLY_RECOVERY_THRESHOLD`), so the mismatch cannot be detected on the
    // link: the one-shot request is dropped, nothing acknowledges the refusal, and the Session dies
    // on `max_ssa_delivery_time` waiting for a commitment that was deliberately withheld. Rejecting
    // it here turns a silent remote failure into a local startup error, which is the only place the
    // operator who set the value can see it.
    //
    // Written as a range rather than `< floor`, and with the finiteness test in front, for the same
    // reason `max_off_front_share_fraction` above is: every IEEE comparison against `NaN` is false, so
    // `NaN < floor` admits it, and a `NaN` threshold is not inert — `ceil` leaves it `NaN` and casting
    // that to an integer saturates to zero, so the Exit signals early recovery on its first
    // reconstructed polynomial. That is the *earliest* possible request against a gate whose entire
    // job is to refuse early ones.
    if !reconstructor_cfg.early_recovery_threshold.is_finite()
        || !(hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD..=1.0)
            .contains(&reconstructor_cfg.early_recovery_threshold)
    {
        return Err(TransportSessionError::InvalidConfig(format!(
            "early_recovery_threshold ({}) must be a finite fraction between MIN_EARLY_RECOVERY_THRESHOLD ({}) and \
             1.0; a lower value asks for the next SSA batch before a conforming Entry admits the request",
            reconstructor_cfg.early_recovery_threshold,
            hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD
        )));
    }
    // Documented invariant: tombstone must outlive the ack window.
    if cfg.tombstone_retention_window < reconstructor_cfg.max_ack_await_time {
        return Err(TransportSessionError::InvalidConfig(
            "tombstone_retention_window must be >= max_ack_await_time (otherwise late acks arrive after the tombstone \
             expires)"
                .into(),
        ));
    }
    // The supervisor must give up on a stalled SSA before the reconstructor reclaims the state it
    // would need to finish one. Written against `unused_verifier_lifetime` because that is now the
    // only duration governing a live cycle: it is measured from the last acknowledged share
    // *anywhere* in the cycle, so a cycle that is still being served never expires and the two
    // deadlines only ever race on a cycle that has genuinely stopped.
    //
    // This replaces a comparison against `incomplete_ssa_lifetime`, which no longer exists. That
    // field was inert in any case — it was clamped to the larger of the two lifetimes, so the
    // default configuration always resolved it to 1800 s rather than its own 600 s — which is
    // exactly the kind of hand-held lifetime pairing that scoping reclamation to the cycle removed.
    if cfg.max_recovery_idle >= reconstructor_cfg.unused_verifier_lifetime {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be < unused_verifier_lifetime".into(),
        ));
    }
    // Reject durations that would overflow the monotonic clock when used as
    // deadlines. See `MAX_SUPERVISOR_DURATION`.
    //
    // The first two are checked *as armed*, i.e. multiplied by the batch size, because that product
    // is the duration a deadline is actually set to. Checking the unscaled value would let a config
    // pass here and then be silently clamped at arming time, which is the failure mode this cap
    // exists to make loud.
    for (name, dur) in [
        (
            "max_ssa_delivery_time",
            scaled_deadline(cfg.max_ssa_delivery_time, cfg.ssas_per_request),
        ),
        (
            "max_deposit_wait",
            scaled_deadline(cfg.max_deposit_wait, cfg.ssas_per_request),
        ),
        ("max_recovery_idle", cfg.max_recovery_idle),
        ("max_recovery_time", cfg.max_recovery_time),
        ("tombstone_retention_window", cfg.tombstone_retention_window),
    ] {
        if dur > MAX_SUPERVISOR_DURATION {
            return Err(TransportSessionError::InvalidConfig(format!(
                "{name} ({dur:?}, as armed for a batch of {}) must not exceed {MAX_SUPERVISOR_DURATION:?}",
                cfg.ssas_per_request
            )));
        }
    }
    // The two recovery clocks measure different things and only one of them is an anti-drip rule.
    // `max_recovery_idle` is service-gated, so it spares a Session that is merely quiet and closes
    // one that is consuming service while returning nothing. `max_recovery_time` is neither gated nor
    // refreshable — it is a resource backstop for the slot and the reconstructor state, and is
    // documented as such throughout this module.
    //
    // Set at or below the idle deadline, the backstop fires first on *every* cycle. The idle rule —
    // the only instrument that can tell a slow honest Session from a cheating one — becomes
    // unreachable, and honest Sessions are closed on `RecoveryDeadline` instead. Both values pass
    // every check above individually, so this inversion is invisible unless the two are compared.
    //
    // Checked after the cap loop deliberately. An over-cap `max_recovery_idle` cannot satisfy this
    // rule either — no legal `max_recovery_time` exceeds it — so running this first would make the
    // cap's own `max_recovery_idle` branch unreachable and swap a precise diagnostic for a vaguer
    // one. Representability first, then policy.
    if cfg.max_recovery_time <= cfg.max_recovery_idle {
        return Err(TransportSessionError::InvalidConfig(format!(
            "max_recovery_time ({:?}) must be > max_recovery_idle ({:?}); the hard deadline is a resource backstop, \
             and at or below the idle deadline it pre-empts the service-gated idle rule on every cycle",
            cfg.max_recovery_time, cfg.max_recovery_idle
        )));
    }
    Ok(())
}

/// Upper bound on any supervisor duration, as it is actually armed.
///
/// 24 h is far above anything a supervisor deadline should be, and the point is not the number but
/// the failure it forecloses: every phase reads an absent deadline as *no deadline*, and
/// `Instant::checked_add` returns `None` for a duration the monotonic clock cannot represent. So a
/// large enough configured value does not produce a long deadline, it produces none at all — the
/// supervisor silently stops enforcing that phase.
///
/// Enforced twice, and deliberately so. [`validate_pix_supervision`] *rejects* a config above it,
/// which is what an operator should see. `SessionManager::new` *clamps* to it, because nothing in
/// this crate calls `validate` and a programmatically assembled config would otherwise reach the
/// supervisor unchecked.
pub(crate) const MAX_SUPERVISOR_DURATION: Duration = Duration::from_secs(86400);

/// A per-cycle deadline duration as it is actually armed for a batch of `ssas_per_request`.
///
/// One place computes this so the validator and the supervisor cannot disagree about what a
/// configuration means. `ssas_per_request` is clamped rather than trusted, because a supervisor can be
/// constructed from a config that never went through [`validate_pix_supervision`]; the multiplication
/// saturates for the same reason.
pub(crate) fn scaled_deadline(per_cycle: Duration, ssas_per_request: usize) -> Duration {
    per_cycle.saturating_mul(ssas_per_request.clamp(1, crate::MAX_SSA_BATCH_SIZE) as u32)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// Deliberately *not* gated on `runtime-tokio`: every test below is a synchronous `#[test]` over the
// pure validator, and `default = []`, so a feature gate here removed the whole suite from
// `cargo test -p hopr-transport-session`. The gate was invisible because the workspace build
// unifies the feature in from other crates — the coverage was only missing for the one command an
// operator of this crate would run.
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hopr_protocol_pix::SsaReconstructorConfig;

    use super::*;

    fn valid_cfg() -> SupervisorConfig {
        SupervisorConfig {
            ssas_per_request: 1,
            max_failed_cycles: 1,
            max_ssa_delivery_time: Duration::from_secs(20),
            max_deposit_wait: Duration::from_secs(60),
            max_recovery_idle: Duration::from_secs(60),
            max_recovery_time: Duration::from_secs(3600),
            max_unverifiable_shares_per_ssa: 0,
            max_unverifiable_shares_per_session: 0,
            max_predeposit_packets: 1024,
            max_served_without_progress: 256,
            max_off_front_share_fraction: 0.25,
            min_share_order_sample: 16384,
            tombstone_retention_window: Duration::from_secs(30),
            min_deposit: HoprBalance::new_base(0),
        }
    }

    fn valid_rcn_cfg() -> SsaReconstructorConfig {
        SsaReconstructorConfig {
            max_ack_await_time: Duration::from_secs(10),
            unused_verifier_lifetime: Duration::from_secs(600),
            ..Default::default()
        }
    }

    #[test]
    fn validation_accepts_valid_configs() {
        assert!(validate_pix_supervision(&valid_cfg(), &valid_rcn_cfg()).is_ok());
    }

    #[test]
    fn validation_rejects_zero_max_ssa_delivery_time() {
        let mut cfg = valid_cfg();
        cfg.max_ssa_delivery_time = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_deposit_wait() {
        let mut cfg = valid_cfg();
        cfg.max_deposit_wait = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_recovery_idle() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_idle = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_recovery_time() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_time = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_tombstone_retention_window() {
        let mut cfg = valid_cfg();
        cfg.tombstone_retention_window = Duration::ZERO;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    #[test]
    fn validation_rejects_zero_max_served_without_progress() {
        let mut cfg = valid_cfg();
        cfg.max_served_without_progress = 0;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    /// Both ends of the batch range, and the zero that the supervisor would otherwise clamp away.
    ///
    /// Zero is the one worth spelling out: the supervisor clamps `ssas_per_request` to at least one
    /// where it reads it, so a zero here does not fail loudly at runtime, it silently becomes a
    /// batch of one. This is the only place an operator is told the value was not honoured.
    #[test]
    fn validation_rejects_an_out_of_range_batch_size() {
        let rcn = valid_rcn_cfg();
        for batch in [0, crate::MAX_SSA_BATCH_SIZE + 1] {
            let mut cfg = valid_cfg();
            cfg.ssas_per_request = batch;
            assert!(
                validate_pix_supervision(&cfg, &rcn).is_err(),
                "ssas_per_request = {batch} is outside 1..={} and must be rejected",
                crate::MAX_SSA_BATCH_SIZE
            );
        }

        for batch in [1, crate::MAX_SSA_BATCH_SIZE] {
            let mut cfg = valid_cfg();
            cfg.ssas_per_request = batch;
            assert!(
                validate_pix_supervision(&cfg, &rcn).is_ok(),
                "ssas_per_request = {batch} is on the boundary and must be accepted"
            );
        }
    }

    /// Zero tolerated failures closes the Session on the first lost cycle.
    ///
    /// Reads as the consistent thing to allow next to the zero-value knobs that *are* legal, and is
    /// not: at the shipping batch of one it is indistinguishable from a limit of one, so it would
    /// pass unnoticed and then silently disable batching for anyone who raised `ssas_per_request`.
    #[test]
    fn validation_rejects_zero_max_failed_cycles() {
        let mut cfg = valid_cfg();
        cfg.max_failed_cycles = 0;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    /// The off-front fraction is a ratio, and a value outside `0.0..=1.0` is unreachable rather than
    /// strict — so a typo silently disables the check instead of tightening it.
    ///
    /// `NaN` is the case that needs the finiteness test in front of the range: every IEEE comparison
    /// against `NaN` is false, so a bare range test admits it, and an admitted `NaN` makes every
    /// later comparison against the fraction false too. That is the check disabled, not relaxed.
    #[test]
    fn validation_rejects_an_out_of_range_off_front_fraction() {
        let rcn = valid_rcn_cfg();
        for fraction in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
            let mut cfg = valid_cfg();
            cfg.max_off_front_share_fraction = fraction;
            assert!(
                validate_pix_supervision(&cfg, &rcn).is_err(),
                "max_off_front_share_fraction = {fraction} must be rejected"
            );
        }

        for fraction in [0.0, 1.0] {
            let mut cfg = valid_cfg();
            cfg.max_off_front_share_fraction = fraction;
            assert!(
                validate_pix_supervision(&cfg, &rcn).is_ok(),
                "max_off_front_share_fraction = {fraction} is on the boundary and must be accepted"
            );
        }
    }

    /// A zero sample would judge the very first off-front share, which mixnet reordering alone
    /// produces on an entirely conforming Entry.
    #[test]
    fn validation_rejects_zero_min_share_order_sample() {
        let mut cfg = valid_cfg();
        cfg.min_share_order_sample = 0;
        assert!(validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_err());
    }

    /// Zero predeposit packets is strict prepay, and it has to stay a legal configuration.
    ///
    /// Worth an explicit test because the neighbouring zero-checks make rejecting this one look like
    /// the consistent thing to do, and it is not: an Exit that declines to serve anything before the
    /// deposit is expressing a policy, not misconfiguring itself. Nothing on the path to funding goes
    /// through the egress gate, so the Session still becomes fundable on a zero budget — see
    /// [`SupervisorConfig::max_predeposit_packets`]. Without this test, adding a
    /// `max_predeposit_packets == 0` rejection alongside the others would silently remove the option.
    #[test]
    fn validation_accepts_zero_predeposit_packets_for_strict_prepay() {
        let mut cfg = valid_cfg();
        cfg.max_predeposit_packets = 0;
        assert!(
            validate_pix_supervision(&cfg, &valid_rcn_cfg()).is_ok(),
            "strict prepay (max_predeposit_packets = 0) must remain a supported configuration"
        );
    }

    #[test]
    fn validation_rejects_idle_shorter_than_ack_await() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_idle = Duration::from_secs(5);
        let rcn = valid_rcn_cfg();
        // max_ack_await_time is 10 s, so 5 < 10 should fail.
        assert!(validate_pix_supervision(&cfg, &rcn).is_err());
    }

    /// The supervisor must give up on a stalled SSA before the reconstructor reclaims the state
    /// needed to finish it, or the session would sit waiting on shares that can never be applied.
    #[test]
    fn validation_rejects_idle_reaching_the_verifier_lifetime() {
        let mut cfg = valid_cfg();
        let rcn = valid_rcn_cfg();
        // unused_verifier_lifetime is 600 s.
        cfg.max_recovery_idle = Duration::from_secs(700);
        assert!(validate_pix_supervision(&cfg, &rcn).is_err(), "700 > 600 must reject");
        cfg.max_recovery_idle = Duration::from_secs(600);
        assert!(
            validate_pix_supervision(&cfg, &rcn).is_err(),
            "equal must reject too — the supervisor has to act first, not at the same instant"
        );
        cfg.max_recovery_idle = Duration::from_secs(599);
        assert!(validate_pix_supervision(&cfg, &rcn).is_ok(), "599 < 600 must accept");
    }

    /// Durations are used as `Instant` offsets, so an absurd one would be silently lost to
    /// `checked_add` returning `None` rather than producing a deadline. The cap is what makes that
    /// unreachable, so it has to reject rather than saturate.
    #[test]
    fn validation_rejects_durations_beyond_the_deadline_cap() {
        let rcn = valid_rcn_cfg();
        for mutate in [
            (|c: &mut SupervisorConfig| c.max_ssa_delivery_time = Duration::MAX) as fn(&mut SupervisorConfig),
            |c: &mut SupervisorConfig| c.max_deposit_wait = Duration::MAX,
            |c: &mut SupervisorConfig| c.max_recovery_time = Duration::MAX,
            |c: &mut SupervisorConfig| c.tombstone_retention_window = Duration::MAX,
        ] {
            let mut cfg = valid_cfg();
            mutate(&mut cfg);
            assert!(
                validate_pix_supervision(&cfg, &rcn).is_err(),
                "a duration of Duration::MAX must be rejected, not saturated"
            );
        }
    }

    /// An Exit below the protocol floor cannot have its successor requests admitted by any conforming
    /// Entry, so the mismatch has to be caught locally.
    ///
    /// The Entry's gate is computed at [`hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD`] because
    /// this value never goes on the wire — see the constant. That makes a lower local setting a
    /// silent, remote failure: the one-shot request goes out early, the Entry drops it, nothing
    /// acknowledges the refusal, and the Session dies on its commitment deadline. This is the only
    /// place an operator can be told, so both individually valid halves have to be compared here.
    #[test]
    fn validation_rejects_an_early_recovery_threshold_below_the_protocol_floor() {
        let cfg = valid_cfg();
        let floor = hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD;

        let below = SsaReconstructorConfig {
            early_recovery_threshold: floor - 0.05,
            ..valid_rcn_cfg()
        };
        assert!(
            validate_pix_supervision(&cfg, &below).is_err(),
            "an Exit asking earlier than any Entry admits must not start"
        );

        for accepted in [floor, 1.0] {
            let rcn = SsaReconstructorConfig {
                early_recovery_threshold: accepted,
                ..valid_rcn_cfg()
            };
            assert!(
                validate_pix_supervision(&cfg, &rcn).is_ok(),
                "{accepted} is at or above the floor and must be accepted"
            );
        }

        assert!(
            SsaReconstructorConfig::default().early_recovery_threshold >= floor,
            "the shipped default must itself be admissible"
        );
    }

    /// `NaN` is unordered, so a bare `< floor` check does not reject it. The reconstructor's
    /// derived range validator has the same problem: both `NaN < 0.0` and `NaN > 1.0` are false.
    /// Once admitted, casting `ceil(NaN)` to `usize` produces zero and the early notification fires
    /// on the first recovered polynomial, well before the Entry's successor gate opens.
    #[test]
    fn validation_rejects_a_non_finite_early_recovery_threshold() {
        let cfg = valid_cfg();
        let rcn = SsaReconstructorConfig {
            early_recovery_threshold: f64::NAN,
            ..valid_rcn_cfg()
        };

        assert!(
            validate_pix_supervision(&cfg, &rcn).is_err(),
            "a non-finite threshold must not bypass the protocol floor"
        );
    }

    /// The hard recovery deadline is a backstop, so it has to sit outside the rule that should bind.
    ///
    /// `max_recovery_idle` is service-gated and `max_recovery_time` is not, so inverting them does not
    /// merely reorder two timeouts: the ungated deadline fires on every cycle and the gated one — the
    /// only rule that distinguishes a quiet Session from a cheating one — is never consulted. Both
    /// values are individually legal at every boundary tested elsewhere in this module, so nothing
    /// short of comparing them catches it.
    #[test]
    fn validation_rejects_a_recovery_deadline_that_pre_empts_the_idle_rule() {
        let mut cfg = valid_cfg();
        let rcn = valid_rcn_cfg();
        let idle = cfg.max_recovery_idle;

        cfg.max_recovery_time = idle;
        assert!(
            validate_pix_supervision(&cfg, &rcn).is_err(),
            "equal must reject: a backstop that expires with the rule it backs up has replaced it"
        );

        cfg.max_recovery_time = idle - Duration::from_secs(1);
        assert!(validate_pix_supervision(&cfg, &rcn).is_err(), "below must reject");

        cfg.max_recovery_time = idle + Duration::from_secs(1);
        assert!(
            validate_pix_supervision(&cfg, &rcn).is_ok(),
            "strictly above is the whole requirement; how far above is an operator's call"
        );

        let shipped = SupervisorConfig::default();
        assert!(
            shipped.max_recovery_time > shipped.max_recovery_idle,
            "the shipped defaults must themselves satisfy the rule"
        );
    }

    /// `max_recovery_idle` is bounded from both directions, so its cap check must be reachable
    /// without first tripping the verifier-lifetime rule above.
    ///
    /// The recovery-clock pairing rule is checked after the duration cap for this reason: an idle
    /// deadline above the cap admits no legal `max_recovery_time` greater than it, so running the
    /// pairing first would make this branch unreachable.
    #[test]
    fn validation_rejects_an_over_cap_idle_against_a_long_verifier_lifetime() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_idle = Duration::from_secs(86_401);
        let rcn = SsaReconstructorConfig {
            unused_verifier_lifetime: Duration::from_secs(200_000),
            ..valid_rcn_cfg()
        };
        assert!(validate_pix_supervision(&cfg, &rcn).is_err());
    }

    /// The shipped default must close on the *first* unverifiable share.
    ///
    /// Worth pinning separately from the state machine's own limit tests, which all configure a
    /// non-zero tolerance: the enforcement is `total > limit`, so a limit of zero is the one value
    /// where the comparison has to fire on the first observation rather than the second. Nothing
    /// else exercises it, and getting it wrong would silently restore tolerance for a failure that
    /// has already doomed the cycle.
    #[test]
    fn the_default_configuration_closes_on_the_first_unverifiable_share() -> anyhow::Result<()> {
        use hopr_api::types::crypto_random::Randomizable;

        use crate::supervision::supervisor::SessionPixSupervisor;

        let pseudonym = HoprPseudonym::random();
        let ssa_id = SsaId::new(pseudonym, hopr_protocol_pix::SsaIndex::MIN);
        let dims = PixParams::try_new(10, 5, 7, crate::types::LOCAL_PIX_SUITE)?;
        let now = std::time::Instant::now();

        let (mut supervisor, _) = SessionPixSupervisor::new(SupervisorConfig::default(), dims, pseudonym, now);
        supervisor.handle_event(&SessionPixEvent::SsaRequestSent(ssa_id), now, 0);

        let actions = supervisor.handle_event(
            &SessionPixEvent::UnverifiableShares {
                ssa_id,
                observed_total: 1,
            },
            now,
            0,
        );

        assert!(
            actions.iter().any(|a| matches!(
                a,
                SessionPixAction::Close(SessionPixCloseReason::TooManyUnverifiableShares)
            )),
            "one unverifiable share must close the session under the default config, got {actions:?}"
        );

        Ok(())
    }
}
