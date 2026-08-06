//! PIX supervision for incoming PIX-enabled sessions.
//!
//! This module implements the **Exit-side** supervision logic for sessions
//! using the Packet Information eXtension (PIX) protocol.  The Exit node
//! runs a deterministic supervisor that tracks each *Secret Sharing Aggregate*
//! (SSA) through a well-defined lifecycle, enforcing timeouts, deposit
//! sufficiency, recovery progress, and fault tolerances.  Egress data
//! packets are gated behind a concurrent [`ServiceGate`] that allows
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
//! | [`SessionPixSupervisor`] | [`supervisor`] | Pure state machine — no I/O, no async, no spawning.  Driven by explicit [`Instant`] timestamps and service-gate snapshots. |
//! | [`ServiceGate`] | [`gate`] | Concurrent, lock-free egress gate.  Before funding: bounded predeposit budget.  After funding: ceiling on packets served without SSA recovery progress.  Callers park on a generation-counter waker. |
//! | Worker loop | [`worker`] | Per-session actor that bridges the pure supervisor to async reality.  Receives commands via a backpressured channel, manages the deadline timer, and forwards supervisor actions to the caller. |
//!
//! The [`SlotNotify`](crate::utils::SlotNotify) multi-waker primitive is shared from [`utils`](crate::utils) and used
//! by [`ServiceGate`] to park and wake callers without a tokio dependency.
//!
//! ## The [`SessionPixSupervisor`] state machine
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
//! * **Recovery idle** — time without *useful progress* while service is being consumed. **Service-gated**: if no
//!   packets were served since the last progress snapshot, the timer re-arms instead of closing (prevents a
//!   slow-but-honest Entry from being disconnected).
//! * **Recovery hard deadline** — absolute per-SSA backstop, never extended. This is a resource guard (session slot +
//!   reconstructor memory), not a liveliness mechanism.
//!
//! **Fault tracking** — the supervisor tracks unverifiable shares via the
//! `UnverifiableShares` event (observed as absolute per-SSA totals that may
//! arrive from multiple concurrent ack processing batches).  It charges only
//! the delta from the maximum seen so far, preventing stale or out-of-order
//! snapshots from double-counting.  Limits exist per-SSA and per-session.
//!
//! **Rolling SSAs** — to maintain continuity, the supervisor requests a
//! *next* SSA when the current one is "almost recovered" (early threshold
//! reached) or fully recovered.  It keeps at most two live SSAs in flight
//! plus one in tombstone phase.
//!
//! ## The [`ServiceGate`] — egress gating
//!
//! Every egress data packet from the Exit back to the Entry must pass
//! through the [`ServiceGate`] via [`acquire`](ServiceGate::acquire):
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
//! [`SlotNotify`] future.  A concurrent [`release_service`](ServiceGate::release_service),
//! [`notify_progress`](ServiceGate::notify_progress), or [`poison`](ServiceGate::poison)
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
//! * The `SsaRequest` goes out on the [`SessionManager`](crate::SessionManager)'s own message sender, not the Session's
//!   gated sink.
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
//! deadline closes the Session and [`poison`](ServiceGate::poison) fails the parked writer rather
//! than leaving it pending.
//!
//! ### Post-funding (ceiling)
//!
//! Once the first deposit is confirmed and the supervisor emits
//! [`ReleaseService`](SessionPixAction::ReleaseService), the gate flips
//! to funded mode.  It then enforces `max_served_without_progress`: a
//! ceiling on how many packets may be served between SSA recovery progress
//! events, as a defense-in-depth backstop even when the supervisor's
//! service-gated idle timer is alive.  Each [`ProgressNotification`](SessionPixAction::ProgressNotification)
//! resets the ceiling by snapshotting the served counter as the new watermark.
//!
//! The gate is implemented with lock-free atomics and CAS loops.  It uses
//! the generation-counter [`SlotNotify`] to avoid the two classic
//! race conditions of waker-vector approaches:
//!
//! 1. **Latent wake** — notification between future creation and first `poll()` is caught because the generation was
//!    captured at creation time and compared on `poll()`.
//! 2. **Spurious `Ready`** — a second `poll()` of an already-registered future re-checks the generation; if unchanged
//!    it stays `Pending`.
//!
//! ## The Worker — bridging pure logic to async
//!
//! [`spawn_supervisor_worker`] creates the [`SessionPixSupervisor`],
//! the [`ServiceGate`], and a bounded async command channel.  It returns
//! a [`SessionPixSupervisorHandle`] (cloneable, for sending events) and an
//! [`ActionRx`] receiver (for driving actions).
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
//! ## Integration with [`SessionManager`](crate::SessionManager)
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
//!    | `RequestSsa` | Calls `send_ssa_request`, feeds back result to supervisor.  Tracks SSA in [`SsaRetirementGuard`] for Drop-safe cleanup. |
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
    /// * The Exit holds that many live reconstructor cycles at once (≈49 MiB of peak state each at the profiled
    ///   dimensions).
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
    /// Default: 1 hour.
    #[default(Duration::from_secs(3600))]
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

    /// Useful shares tolerated on a cycle while an *earlier* cycle of the batch is still unrecovered,
    /// across the whole session, before it is closed.
    ///
    /// A batch only de-risks the Exit if its cycles are served one at a time. An Entry that instead
    /// spread shares across all `ssas_per_request` cycles could take `ssas_per_request × quota` of
    /// service while leaving every one of them short of recovery — and a cycle short of recovery pays
    /// nothing at all, since the SSA is the sum of *every* polynomial's constant term. That turns the
    /// batch knob from cost-neutral into an `n`-fold amplifier of unpaid traffic, and none of the other
    /// guards see it: the service gate and the idle deadline both measure the *absence* of progress,
    /// and a spreading Entry makes progress on every cycle continuously — it simply never finishes one.
    ///
    /// A conforming Entry contributes nothing here. Its emission window is clamped to one cycle (see
    /// [`hopr_protocol_pix::SHARE_EMISSION_WINDOW`]), and it keeps feeding a cycle's tail for
    /// `surplus_shares` per polynomial *after* that cycle became reconstructable, so the earlier cycle
    /// is terminal well before the next one's first share is emitted. What the allowance covers is
    /// in-flight reordering: acknowledgements unlock shares, and a batch of them can be processed
    /// after the following cycle's have already landed.
    ///
    /// Session-cumulative rather than per-cycle, because an Entry that misorders is misbehaving and
    /// will not improve. What separates it from noise is rate, not total: a spreading Entry books
    /// out-of-order shares at nearly its whole service rate and crosses any ceiling at once, while
    /// reordering is a trickle.
    ///
    /// ## Why the ceiling is this large
    ///
    /// It is derived, not chosen, and the binding case is not reordering — it is a cycle that has
    /// become *unrecoverable* through loss. A polynomial that loses more than `surplus_shares` of its
    /// shares can never reach its threshold, so its cycle never completes and never goes terminal,
    /// which makes every later cycle's progress count here. That cycle is not the Exit's problem to
    /// close a Session over: it stops progressing, its
    /// [`max_recovery_idle`](Self::max_recovery_idle) deadline expires, and it is retired — after
    /// which the later cycles are the earliest live ones again and counting stops on its own. The
    /// ceiling only has to outlast that window, or a single unlucky polynomial would close a Session
    /// that the retirement path was about to fix by itself.
    ///
    /// At the deployed 1.5 Mbps per-Session cap that window admits ~181 shares/s × 60 s ≈ 10 900
    /// shares, so 16 384 leaves ~1.5× headroom. It is still only ~3 % of one cycle's
    /// `target_useful_shares` at deployed dimensions, which is the bound it puts on how much
    /// misdirected service a spreading Entry can extract before the Session closes.
    ///
    /// Default: 16384 shares.
    #[default(16384)]
    pub max_out_of_order_shares_per_session: u64,

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

    /// Maximum packets served without SSA recovery progress before the gate
    /// blocks further service as a defense-in-depth backstop.
    ///
    /// This is a ceiling enforced by `ServiceGate::acquire` after the gate is
    /// funded. Each [`crate::HoprSessionInPixEvent::RecoveryProgress`] event resets the ceiling
    /// counter.
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
// SsaDimensions
// ---------------------------------------------------------------------------

/// PIX dimensions agreed upon during session negotiation.
///
/// Re-exported rather than defined here: this supervisor once carried its own `polys`/`threshold`
/// pair, which meant the same thing as the one a Session offers but named it differently and could
/// not be handed across the boundary without a field-by-field copy. `target_useful_shares` moved
/// onto the shared type with it.
pub use crate::types::SsaDimensions;

// ---------------------------------------------------------------------------
// SessionPixEvent
// ---------------------------------------------------------------------------

/// Events consumed by the [`SessionPixSupervisor`].
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

/// Actions emitted by the [`SessionPixSupervisor`] for the caller to execute.
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
        polys: u16,
        threshold: u16,
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
/// These are mapped to public [`ClosureReason`] by the caller.
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
    /// The Entry served too many shares for a cycle while an earlier cycle of the batch was still
    /// unrecovered — see [`SupervisorConfig::max_out_of_order_shares_per_session`].
    TooManyOutOfOrderShares,
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
    if cfg.max_recovery_idle < reconstructor_cfg.max_ack_await_time {
        return Err(TransportSessionError::InvalidConfig(
            "max_recovery_idle must be >= max_ack_await_time".into(),
        ));
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
    // deadlines. 24 h is a safe upper bound — no supervisor duration should
    // ever be this large, and the cap prevents silent deadline loss via
    // Instant::checked_add returning None.
    //
    // The first two are checked *as armed*, i.e. multiplied by the batch size, because that product
    // is the duration a deadline is actually set to. Checking the unscaled value would let a config
    // pass here and then be silently clamped at arming time, which is the failure mode this cap
    // exists to make loud.
    const MAX_SUPERVISOR_DURATION: Duration = Duration::from_secs(86400);
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
    Ok(())
}

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

#[cfg(test)]
#[cfg(feature = "runtime-tokio")]
mod tests {
    use std::time::Duration;

    use hopr_protocol_pix::SsaReconstructorConfig;

    use super::*;

    fn valid_cfg() -> SupervisorConfig {
        SupervisorConfig {
            ssas_per_request: 1,
            max_ssa_delivery_time: Duration::from_secs(20),
            max_deposit_wait: Duration::from_secs(60),
            max_recovery_idle: Duration::from_secs(60),
            max_recovery_time: Duration::from_secs(3600),
            max_unverifiable_shares_per_ssa: 0,
            max_unverifiable_shares_per_session: 0,
            max_predeposit_packets: 1024,
            max_served_without_progress: 256,
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

    /// `max_recovery_idle` is bounded from both directions, so its cap check must be reachable
    /// without first tripping the verifier-lifetime rule above.
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
    fn the_default_configuration_closes_on_the_first_unverifiable_share() {
        use hopr_api::types::crypto_random::Randomizable;

        use crate::supervision::supervisor::SessionPixSupervisor;

        let pseudonym = HoprPseudonym::random();
        let ssa_id = SsaId::new(pseudonym, hopr_protocol_pix::SsaIndex::MIN);
        let dims = SsaDimensions::new(10, 5);
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
    }
}
