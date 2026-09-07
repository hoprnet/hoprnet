//! PIX supervision for incoming PIX-enabled sessions.
//!
//! This module implements the **Exit-side** supervision logic for sessions
//! using the Protocol for Incentivization of eXits (PIX).  The Exit node
//! runs a deterministic supervisor that tracks each *Secret Sharing Aggregate*
//! (SSA) through a well-defined lifecycle, enforcing timeouts, deposit
//! sufficiency, recovery progress, and fault tolerances.  Egress data
//! packets are gated behind a concurrent `ServiceGate` that follows the
//! front cycle: bounded predeposit service while it is unfunded and a
//! ceiling-limited path after it funds.
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
//! | `ServiceGate` | `gate` | Concurrent, lock-free egress gate.  Before the front funds: bounded predeposit budget.  After it funds: ceiling on packets served without that cycle's recovery progress.  Callers park on a generation-counter waker. |
//! | Worker loop | `worker` | Per-session actor that bridges the pure supervisor to async reality.  Receives commands via a backpressured channel, manages the deadline timer, applies gate actions synchronously, and forwards actions to the caller. |
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
//!                                          ├── (CommitmentProgress arms/re-arms the re-request timer)
//!                                          ├── (its expiry → RequestCommitmentRetransmission, bounded)
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
//!                                     Recovered
//!                                          ├── bounded paid FIFO drain
//!                                          ├── successor progress → handoff
//!                                          └── diagnostic tombstone expiry → RetireSsa
//! ```
//!
//! **Key deadlines** (all configurable via [`SupervisorConfig`]):
//!
//! * **Commitment timeout** — time from `SsaRequestSent` to `CommitmentVerified`.
//! * **Commitment re-request** — time a *partially* delivered commitment may go quiet before the missing parts are
//!   asked for again. The one deadline here that is not a deadline: it emits a repair and re-arms, never a close, and
//!   the commitment timeout above remains the bound. It is armed only by `CommitmentProgress`, so a commitment nothing
//!   was ever delivered for never asks — there is no scope for it to name. Bounded by count, at
//!   [`MAX_COMMITMENT_RETRANSMISSIONS`](hopr_protocol_pix::MAX_COMMITMENT_RETRANSMISSIONS), which is what the Entry
//!   will answer.
//! * **Deposit timeout** — time from `CommitmentVerified` to a sufficient deposit.
//! * **Recovery idle** — time without *any share arriving* while service is being consumed. **Service-gated**: if no
//!   packets were served since the last progress snapshot, the timer re-arms instead of closing (prevents a
//!   slow-but-honest Entry from being disconnected). Deliberately not "useful progress": a conforming Entry emits a
//!   whole window's surplus in one run, none of it useful, and treating that as silence closes honest Sessions.
//! * **Recovery hard deadline** — absolute per-SSA backstop, never extended. This is a resource guard (session slot +
//!   reconstructor memory), not a liveliness mechanism.
//!
//! Both recovery clocks start when the cycle reaches the **paid transport front**, not when its
//! deposit confirms. A batch is served in index order, and the final SURBs buffered for a recovered
//! predecessor still carry that predecessor's shares. The predecessor keeps its original immutable
//! hard clock while this bounded tail drains; the successor's clocks start only when the negotiated
//! remainder is exhausted or funded successor progress proves the FIFO boundary. A queued cycle is
//! therefore not charged for either its batch wait or its predecessor's pipeline delay.
//!
//! **Fault tracking** — an `UnverifiableShares` event closes the Session on arrival, with no
//! tolerance to configure and no running totals to keep: the event means a polynomial's share set
//! failed to open its commitment, which permanently dooms the cycle. Late fault reports against an
//! already terminal SSA are absorbed. Progress is the one narrow exception: the sole immediate
//! recovered predecessor may report its bounded, already-paid FIFO tail. See the configuration
//! section below for why this is not a dial.
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
//! Allocation independently enforces the reservation: at most
//! `MAX_OVERLAPPING_BATCHES` generations and
//! `MAX_OVERLAPPING_BATCHES × ssas_per_request` live cycles may own reconstructor state. If the
//! early signal arrives while both generations are still live, the supervisor records the owed
//! request and retries it only after one generation releases its heavyweight cycle state. Recovered
//! supervisor tombstones do not count because full recovery removes their heavyweight reconstructor
//! state; one constant-bounded per-Session tail receipt is tracked separately.
//!
//! ## The `ServiceGate` — egress gating
//!
//! Every egress data packet from the Exit back to the Entry must pass
//! through the `ServiceGate` via `acquire`:
//!
//! ### Pre-funding (predeposit)
//!
//! While the current front cycle is unfunded, a provisional budget lets the Exit answer the Entry
//! for a bounded number of packets, so that the application is not held up for as long as an
//! on-chain deposit takes to confirm. The initial front starts with this allowance; a successor gets
//! a fresh allowance after its predecessor was funded and leaves the front, whether by recovery or a
//! tolerated failure. A failed funded cycle has already cost the Entry its full deposit, and
//! `max_failed_cycles` bounds how often that can happen. Losing an unfunded front does not mint
//! another grant. The budget is
//! `min(target_useful_shares - 1, max_predeposit_packets)`; at production dimensions the first term
//! is in the hundreds of thousands, so the configured cap is what binds and the `min` only matters
//! for the small dimensions used in tests.
//!
//! When the budget is exhausted, `acquire` parks the caller on a
//! `SlotNotify` future. A concurrent `release_service`, `withhold_service`,
//! `notify_progress`, or `poison`
//! wakes all parkers.
//!
//! ### Strict prepay (`max_predeposit_packets = 0`)
//!
//! Zero is a supported setting, and it means the Exit serves nothing against any unfunded front:
//! the first egress data packet parks, and the Entry has to commit and fund that cycle before a
//! single payload byte flows back to it. The rule is restored on every paid rotation rather than
//! applying only to the Session's first SSA.
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
//! Once the current front's deposit is confirmed and the supervisor emits `ReleaseService`, the
//! gate flips to funded mode. It then enforces `max_served_without_progress`: a
//! ceiling on how many packets may be served between SSA recovery progress
//! events, as a defense-in-depth backstop even when the supervisor's
//! service-gated idle timer is alive. Only progress from that funded front—or from its sole bounded
//! recovered predecessor while the FIFO drains—emits `ProgressNotification` and resets the
//! watermark; unfunded, off-front, older, or over-budget progress cannot buy service. A rotation to
//! an unfunded successor emits `WithholdService` and restores the bounded allowance. A funded
//! successor stays open, but another `ReleaseService` rebaselines its ceiling.
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
//! 6. Applies gate-control actions immediately, then forwards all actions to the action channel (non-blocking
//!    `try_send`).
//!
//! **Coalescing** — `ProgressNotification` actions are coalescible: when
//! the action channel is transiently full, they are dropped rather than
//! blocking or failing the worker. The gate has already consumed the notification locally, and the
//! next forwarded notification replaces it for observers.
//!
//! All other actions (`RequestSsa`, `RequestCommitmentRetransmission`, `ReleaseService`,
//! `WithholdService`, `RetireSsa`, `Close`) are non-coalescible — if they cannot be delivered, the
//! channel is genuinely wedged and the worker fails the session.
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
//! 6. After session publication, spawns the **action driver task** that receives actions from `ActionRx`. Gate control
//!    has already happened in the worker; the driver performs I/O and records the gate-mode telemetry:
//!
//!    | Action | Behaviour |
//!    |---|---|
//!    | `RequestSsa` | Calls `send_ssa_request`, feeds back result to supervisor. Tracks each SSA in `SsaCommitmentGuard` for Drop-safe cleanup. |
//!    | `RequestCommitmentRetransmission` | Calls `send_commitment_retransmission_request`, which reads the missing scope from the reconstructor and asks the Entry for it. Registers nothing and reports nothing back: it repeats on its own interval and the commitment deadline is the backstop. |
//!    | `ReleaseService` | Worker calls `gate.release_service()`; driver records funded telemetry. |
//!    | `WithholdService` | Worker calls `gate.withhold_service()`; driver records unfunded telemetry. |
//!    | `ProgressNotification` | Worker calls `gate.notify_progress()`; no driver I/O. |
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
//! │  Action: ReleaseService       → gate.release_service│
//! │  Action: WithholdService      → gate.withhold_service│
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
//! | `ssas_per_request` | 1 | Nothing by itself — an exposure dial. With dynamic batching it is the largest batch the Exit may derive; otherwise it is the exact batch size. Raising it amortises the request round trip over several cycles, at the price of multiplying the *unfunded* exposure to that many quotas, and it scales the two deadlines below. |
//! | `allow_dynamic_ssa_batches` | true | Lets a sub-range per-SSA offer satisfy `quota_range` by committing to the smallest batch whose total quota enters the range. Disable it to retain the fixed-batch, per-SSA admission rule. |
//! | `max_failed_cycles` | 1 | An Entry losing one cycle per batch indefinitely while a single funded sibling holds the Session open. One loss is survivable, the second closes the Session. Only reachable above a batch of one, where the failing cycle is not always the last one standing. |
//! | `max_ssa_delivery_time` | 20 s | An Entry that accepts a request and never delivers the commitment set, holding a session slot and a reconstructor cycle that can never be funded. |
//! | `commitment_recommit_interval` | 3 s | A *single lost packet* costing the whole Session. A commitment set ships as hundreds of messages and is unusable until every one lands, so without this a drop stranded the cycle on the deadline above — after the Entry had already been told to fund it. The one parameter here that buys liveness rather than bounding an attack; it bounds nothing itself. |
//! | `max_deposit_wait` | 60 s | An Entry that commits but never deposits — typically after it has already drawn the predeposit budget. |
//! | `max_recovery_idle` | 60 s | An Entry, or a colluding first return relayer, consuming service while returning no shares. Service-gated, so a Session that is merely quiet is never punished. |
//! | `max_recovery_time` | 2 h | A cycle that dribbles just enough progress to refresh the idle timer forever. A resource backstop for the slot and the reconstructor state, *not* the anti-drip rule. It must clear a whole cycle at the widest dimensions the node accepts — 655 360 packets of *full emission*, ~61 min, at the defaults — or it closes honest Sessions instead. That is the quantity `quota_range` prices and `validate_incoming_session_pix_config` enforces; the *last useful share* lands earlier, at 651 264, which is the figure the "why two hours" argument on [`SupervisorConfig::max_recovery_time`] uses. |
//! | `max_off_front_share_fraction` | 0.25 | An Entry spreading a batch's shares across all of its cycles, taking `ssas_per_request` quotas of service while completing none of them — and a cycle short of completion pays nothing at all. |
//! | `min_share_order_sample` | 16384 | Convicting on a thin sample: the shares that legitimately cross a cycle boundary out of order while in flight. |
//! | `max_predeposit_packets` | 10000 | Bounds what an Entry can extract from an unfunded front. Restored only after a paid front handoff; `0` means strict prepay on every rotation. |
//! | `max_served_without_progress` | 2048 | Packets served with no share of *any* kind coming back — in *packets*, so unlike the idle timer the bound does not move with the Session's rate. Counts `shares_seen`, so a conforming Entry's surplus resets it; see below. |
//! | `tombstone_retention_window` | 30 s | Bounds how long recovered-cycle diagnostics and observer ownership remain; the separately bounded FIFO-tail receipt may outlive it. |
//!
//! ## What is *not* here: the price of a cycle
//!
//! No parameter above says what a quota costs, and none should. The Exit's deposit pool is handed the
//! [`DepositUpdated`](hopr_api::node::DepositUpdated) sender along with
//! [`AgreedSsaQuota`] when the deposit address arrives, and it is the component
//! that knows both the byte quota and the price it charges for one. It sends on that channel once the
//! deposit clears that price; the supervisor's `DepositConfirmed` handler acts on the verdict rather
//! than recomputing it. The Entry side mirrors this — its own strategy prices the `quota` carried by
//! `PixEvent::NewDepositAddress` and decides what to pay.
//!
//! There was briefly a `min_deposit` here, and a per-SSA `expected_deposit` on the commitment event to
//! go with it. Both are gone. The Session layer has no price for a byte to derive one from, and a
//! configured floor would not be a backstop but a second authority behind the first: set above the
//! pool's price it kills a cycle the pool has already been paid for, on `max_deposit_wait`, with
//! nothing on either side able to break the tie.
//!
//! ## The one fault with no dial: unverifiable shares
//!
//! An `UnverifiableShares` report closes the Session immediately, and there is no configuration for
//! it. This used to be a pair of tolerances, per-SSA and per-Session, both shipped at zero.
//!
//! Shares are not checked on arrival — the non-constant coefficient commitments that made per-share
//! verification possible were dropped — so a report means a whole *polynomial's* share set failed to
//! open its commitment. The reconstructor marks that part failed, releases its shares, and never
//! clears the flag; the SSA is the sum of every polynomial's constant term, so from that moment the
//! cycle cannot be reconstructed by any means and will never pay. A tolerance would therefore not buy
//! recovery, only unpaid service — including in the case it looks written for, a false positive from
//! a verification bug, where the part is just as permanently failed. Closing on the first report also
//! caps the exposure at the `threshold` packets already served when the failure surfaces, rather than
//! a multiple of it.
//!
//! What the tolerances bought instead was machinery: absolute cross-peer totals, per-SSA and
//! per-Session running sums, and delta accounting against the maximum seen so far to keep concurrent
//! ack batches from double-counting. None of it is observable at a limit of zero, since the first
//! report ends the Session.
//!
//! ## Constraints between parameters
//!
//! [`validate_pix_supervision`] enforces, at config-load time and against the reconstructor config
//! actually in use: `max_recovery_idle >= max_ack_await_time`; `tombstone_retention_window >=
//! max_ack_await_time`; `max_recovery_idle < unused_verifier_lifetime`;
//! `commitment_recommit_interval < max_ssa_delivery_time`; `ssas_per_request` in
//! `1..=MAX_SSA_BATCH_SIZE`; both scaled deadlines under 24 h; non-zero durations; a share fraction in
//! `0.0..=1.0`; and non-zero `max_served_without_progress`, `min_share_order_sample` and
//! `max_failed_cycles`.
//!
//! ### The surplus run, and why it no longer constrains anything
//!
//! Emission is round-robin over a window of up to `hopr_protocol_pix::SHARE_EMISSION_WINDOW` (256)
//! polynomials advancing in lockstep. They reach `threshold` on the same pass and then take their
//! surplus shares together, so every block ends with `surplus_shares × window` consecutive packets
//! carrying no *useful* share — **4096 at the shipped dimensions**, against a
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
//! polynomial. Full cryptographic recovery no longer cuts that signal off: the reconstructor drops
//! its heavy state but retains one shared per-polynomial remainder, and the supervisor keeps that
//! recovered predecessor as the paid front until the tail is exhausted or successor progress proves
//! the FIFO boundary. The run therefore resets the ceiling and refreshes the idle deadline exactly
//! like the useful shares around it, even across recovery, so **neither parameter has to be sized
//! against the dimensions any more.** What `max_served_without_progress` still bounds is genuine
//! silence: packets served with no share of any kind coming back.
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
//! | `max_predeposit_packets` | 4096 | ~6.8 s of service, matching expected settlement rather than the deadline. This is the most exposed against an unfunded front after the initial grant or a paid handoff — 4.2 MB. Use `0` for strict prepay at every rotation; normal overlap usually funds the successor before it reaches the front |
//! | `max_served_without_progress` | 2048 | Shipped value, and no longer dimension-dependent: the surplus run resets it like any other share, so this bounds genuine silence only |
//! | `max_recovery_idle` | 60 s | Shipped value. Satisfies `>= max_ack_await_time` and `< unused_verifier_lifetime`. It no longer has to cover the surplus run — that resets it — so what it now implies is only that a Session returning *nothing at all* for a minute is closed |
//! | `max_recovery_time` | 2 h | Resource backstop only. A cycle needs 272 s at full rate, so 2 h implies a floor of ~23 packets/s (~0.19 Mbps) — deliberately far below the idle rule, which is the instrument that should bind |
//! | `max_off_front_share_fraction` | 0.25 | Shipped value. A conforming Entry sits near 0; two-way spreading is 0.5 |
//! | `min_share_order_sample` | 16384 | Shipped value, and safe here: with emission clamped to one cycle the front cycle is essentially complete before any off-front progress is possible, so even a loss-doomed cycle peaks near 15 % against the 25 % ceiling |
//! | `tombstone_retention_window` | 60 s | 2× the reconstructor's 30 s ack window |
//! | `max_failed_cycles` | 1 | Shipped value, and inert at this batch size of one — the failing cycle is always the last one standing, which closes the Session first |
//!
//! What one cycle costs is not in this table, because it is not in this configuration: the 162.2 MiB
//! quota is priced by the deposit pool, which is also what decides that a deposit has cleared it.
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
//! | `SsaReconstructorConfig::early_recovery_threshold` | 0.85 | Sets the pipelining runway derived for `ssas_per_request` above — 24 627 packets, 41 s. Bounded below by `MIN_EARLY_RECOVERY_THRESHOLD` and equal to it today: the Entry's successor gate is computed at that floor, so a lower value asks for the next batch before any conforming Entry admits the request. Raising it shortens the runway |
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
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SupervisorConfig {
    /// Maximum number of SSAs the Exit may ask the Entry to commit to in one `SsaRequest`.
    ///
    /// With [`allow_dynamic_ssa_batches`](Self::allow_dynamic_ssa_batches), Session admission picks
    /// the smallest batch no larger than this value whose total quota is inside the Exit's accepted
    /// range. With dynamic batching disabled, this remains the exact batch size, preserving the
    /// original fixed-batch behavior. Batching amortizes the request round trip over several deposit
    /// cycles. It lives here rather than beside the other Exit settings because the supervisor acts
    /// on the selected value: it allocates the indices and scales both deadlines below by this factor.
    ///
    /// Two things it costs, both linear in the value:
    ///
    /// * Up to two batches can own live reconstructor state — worst case ≈41 MiB per cycle at the profiled dimensions,
    ///   per `hopr_protocol_pix::peak_cycle_bytes`. That cost is reserved up front against
    ///   `IncomingSessionPixConfig::max_live_cycle_bytes`; the supervisor enforces the two-generation/full-cycle bound
    ///   before allocating, so this value directly divides the number of PIX Sessions the node will admit.
    /// * The unfunded exposure within one batch: every cycle is initially requested unfunded, so the commitment and
    ///   deposit deadlines cover this many SSA quotas rather than one. The front-aware service gate still grants only
    ///   the current cycle's bounded predeposit allowance.
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

    /// Whether Session admission may derive a smaller SSA batch from the Entry's offered quota.
    ///
    /// When enabled, the Exit tries batch sizes from one through
    /// [`ssas_per_request`](Self::ssas_per_request) and selects the first whose total quota is inside
    /// `IncomingSessionPixConfig::quota_range`. If none fits, the Session is refused with
    /// `UnacceptablePixParams`. Choosing the smallest fit limits Entry work, deposits, Exit memory,
    /// and the chance of exceeding the Entry's unadvertised `max_ssas_per_request` cap.
    ///
    /// When disabled, the offered per-SSA quota itself must be inside `quota_range`, and the Exit
    /// requests exactly `ssas_per_request` SSAs, matching the behavior before dynamic batching.
    ///
    /// Default: `true`.
    #[default(true)]
    pub allow_dynamic_ssa_batches: bool,

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

    /// How long a *partially* delivered commitment may go quiet before the missing parts are
    /// re-requested.
    ///
    /// The Entry's answer to an `SsaRequest` is not one message but hundreds of them, and the Exit
    /// can use none of it until every one has landed — so a single lost packet used to cost the whole
    /// Session on [`max_ssa_delivery_time`](Self::max_ssa_delivery_time), typically after the Entry
    /// had already been told to fund the cycle's deposit address. This is the timer that repairs it:
    /// on expiry the Exit asks the Entry to re-send the polynomial commitments it never received.
    ///
    /// An *idle* timer, armed and re-armed by each arriving commitment message, so it measures
    /// silence rather than elapsed time and the first ask cannot land in the middle of the burst.
    /// It is only ever armed once something has arrived: a commitment nothing was delivered for has
    /// no scope to name, and there is nothing the Exit could ask for.
    ///
    /// It bounds nothing on its own — `max_ssa_delivery_time` remains the absolute deadline, so the
    /// unincentivized exposure of a cycle is unchanged however often this fires. How many asks a
    /// cycle makes is bounded by count rather than by this interval:
    /// [`MAX_COMMITMENT_RETRANSMISSIONS`](hopr_protocol_pix::MAX_COMMITMENT_RETRANSMISSIONS), which
    /// is also exactly how many the Entry will answer, so shortening this buys promptness and never
    /// a flood of wasted packets.
    ///
    /// Default: 3 s — well clear of the time a full burst takes to arrive, and short enough that a
    /// repair completes several times over inside the delivery deadline.
    #[default(Duration::from_secs(3))]
    #[serde(with = "humantime_serde")]
    pub commitment_recommit_interval: Duration,

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
    /// (31 full windows x 80 emitted + 1 window x 64 useful) x 256 = 651 264 packets
    /// ```
    ///
    /// which is just over **60 minutes** at the 1.5 Mbps per-Session cap this crate documents, before
    /// any mixing latency or loss. A one-hour ceiling closes an honest, fully saturated Session at
    /// the default configuration with its last useful share only about 18 seconds away — and the
    /// partial cycle is worth nothing, since the SSA is the sum of every polynomial's constant term.
    ///
    /// This is deliberately not the same count as the 655 360 quoted in the module documentation's
    /// parameter table, and the two must not be reconciled: 655 360 is `8192 × 80`, the *full*
    /// emission including the last window's surplus, which is what `quota_range` prices and what
    /// `validate_incoming_session_pix_config` checks this value against. 651 264 is where the last
    /// *useful* share lands, which is the point this argument is about — a deadline that clears the
    /// useful shares but not the surplus tail still collects a payable cycle.
    ///
    /// Two hours is the value the worked profile in the module documentation already used, and it
    /// keeps this instrument where it belongs: far enough out that
    /// [`max_recovery_idle`](Self::max_recovery_idle) is what actually binds.
    ///
    /// The clock starts when the cycle reaches the paid transport front. That is deliberately later
    /// than merely becoming the earliest unrecovered record: the predecessor's buffered surplus tail
    /// can still occupy ~4096 packets, ~23 s at that rate, and remains bounded by the predecessor's
    /// original hard clock. Starting the successor at the FIFO boundary prevents that pipeline delay
    /// from being charged twice. A funded cycle that reaches the front and is never served is still
    /// caught.
    ///
    /// `HoprProtocolConfig::validate` checks this against the dimensions the node will actually
    /// accept, so a raised `quota_range` that outgrows it is refused at load rather than discovered
    /// one closed Session at a time.
    ///
    /// Default: 2 hours.
    #[default(Duration::from_secs(7200))]
    #[serde(with = "humantime_serde")]
    pub max_recovery_time: Duration,

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
    /// Very large values can make this rule effectively inert for a Session. To disable share-order
    /// enforcement intentionally, prefer
    /// [`max_off_front_share_fraction = 1.0`](Self::max_off_front_share_fraction). No fixed upper bound
    /// is enforced because the accounting window can span multiple successor batches before the front
    /// advances.
    ///
    /// Default: 16384 shares.
    #[default(16384)]
    pub min_share_order_sample: u64,

    /// Cap on the provisional predeposit service budget.
    ///
    /// This buys the application an exchange while the current front cycle's deposit confirms on
    /// chain; it is not needed for the Session to become fundable, so it is a
    /// latency-versus-exposure dial rather than a correctness requirement. The initial front starts
    /// with this allowance, and it is restored for an unfunded successor after a paid front leaves
    /// the front, whether by recovery or a tolerated failure. Retiring an unfunded front does not
    /// grant it again.
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
    /// A ceiling enforced by `ServiceGate::acquire` while the current front is funded. A
    /// [`crate::HoprSessionInPixEvent::RecoveryProgress`] snapshot resets it only for that funded
    /// front. The sole recovered predecessor temporarily remains that paid front while its
    /// per-polynomial negotiated remainder drains; progress on an unfunded, queued, older, or
    /// over-budget cycle does not grant service.
    ///
    /// Those events now follow `shares_seen` rather than `useful_shares`, which is what makes a flat
    /// 2048 safe at any dimensions. Keyed on useful shares, this had to exceed
    /// `surplus x min(polys, SHARE_EMISSION_WINDOW)` — 4096 at the shipped dimensions, twice
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

    /// How long to retain recovered-SSA diagnostic records and observer ownership.
    ///
    /// Must be >= the reconstructor's `max_ack_await_time`. The compact paid-tail receipt is separate
    /// and may outlive this record; expiring a tombstone therefore does not truncate a FIFO drain.
    ///
    /// Default: 30 s.
    #[default(Duration::from_secs(30))]
    #[serde(with = "humantime_serde")]
    pub tombstone_retention_window: Duration,
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
/// packed into — so it carries a third field, the surplus. The supervisor never puts surplus into
/// [`target_useful_shares`](PixParams::target_useful_shares), because it advances no payment, but it
/// does use `threshold + surplus` to cap the recovered FIFO tail independently of the reconstructor.
pub use hopr_protocol_pix::PixParams;

// ---------------------------------------------------------------------------
// SessionPixEvent
// ---------------------------------------------------------------------------

/// Events consumed by the [`SessionPixSupervisor`](supervisor::SessionPixSupervisor).
#[derive(Debug, Clone)]
pub enum SessionPixEvent {
    /// The initial or next SSA request was successfully sent on the wire.
    SsaRequestSent(SsaId<HoprPseudonym>),
    /// Part of an SSA commitment arrived, but the set is still incomplete.
    ///
    /// Reported per message, which is what makes the re-request timer an *idle* timer: the burst
    /// keeps pushing it out, and it only expires once delivery has actually stalled. It is also the
    /// only evidence that the Entry answered at all, which is what distinguishes a cycle worth
    /// re-asking for from one where nothing was ever delivered and there is no scope to name.
    CommitmentProgress(SsaId<HoprPseudonym>),
    /// A verifiable commitment was installed in the reconstructor.
    CommitmentVerified(SsaId<HoprPseudonym>),
    /// The Exit's deposit pool reported a sufficient deposit for this SSA.
    ///
    /// The `amount` is the pool's, for the record and for the one check the supervisor still makes on
    /// it — that it is non-zero. Sufficiency itself is the pool's verdict; see
    /// `SessionPixSupervisor::on_deposit_confirmed`.
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
    /// Ask the Entry to re-send the parts of one SSA's commitment that never arrived.
    ///
    /// Emitted when a partially delivered commitment has been quiet for
    /// [`SupervisorConfig::commitment_recommit_interval`], and repeated on that interval until the
    /// commitment verifies or the absolute delivery deadline closes the Session. The scope is not
    /// carried here: what is missing is the reconstructor's to say, and it can change between this
    /// action being emitted and being carried out, so the carrier reads it at the point of sending.
    ///
    /// Unlike [`RequestSsa`](Self::RequestSsa) this asks for nothing new and its failure is not
    /// terminal — the delivery deadline remains the backstop, so a dropped ask costs one interval.
    RequestCommitmentRetransmission {
        ssa_id: SsaId<HoprPseudonym>,
        params: PixParams,
    },
    /// Put the service gate in funded mode for the front cycle, or rebaseline a funded successor.
    ReleaseService,
    /// Return the service gate to an unfunded successor's predeposit allowance after a paid handoff.
    WithholdService,
    /// Notifies the gate that the funded front made recovery progress, resetting its
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
    /// A polynomial's share set failed to open its commitment.
    ///
    /// There is no threshold behind this: the first report closes the Session, and the reported
    /// `observed_total` is diagnostic only. See the module documentation for why a tolerance would
    /// buy nothing — a failed part can never be reconstructed, so the cycle can never pay.
    UnverifiableShares,
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
/// The state machine itself, for benchmarks only.
///
/// `supervisor` is otherwise private on purpose: the worker owns the only instance that exists in
/// production, and driving one by hand means supplying the `Instant` and served-count arguments
/// that the worker derives from the gate. A benchmark is exactly the case where doing so is the
/// point — [`SessionPixSupervisor::handle_event`] runs once per acknowledgement batch, which is the
/// highest-rate input this module has, and it cannot be measured from outside the crate otherwise.
/// Same gate as [`SessionManager::pre_populate_session`](crate::SessionManager::pre_populate_session).
#[cfg(any(feature = "benchmark", test))]
pub use supervisor::SessionPixSupervisor;
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
    // Both halves are dead config rather than merely odd. A zero interval fires the re-request timer
    // on the same instant it is armed, so every arriving commitment message answers itself with an
    // ask; an interval at or above the delivery deadline never fires before the deadline closes the
    // Session, which is exactly the failure the timer exists to prevent — and neither is visible to
    // the operator anywhere but here.
    if cfg.commitment_recommit_interval.is_zero() {
        return Err(TransportSessionError::InvalidConfig(
            "commitment_recommit_interval must be non-zero".into(),
        ));
    }
    if cfg.commitment_recommit_interval >= cfg.max_ssa_delivery_time {
        return Err(TransportSessionError::InvalidConfig(format!(
            "commitment_recommit_interval ({:?}) must be shorter than max_ssa_delivery_time ({:?}), or a partially \
             delivered commitment is never re-requested before the Session closes",
            cfg.commitment_recommit_interval, cfg.max_ssa_delivery_time
        )));
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
    //
    // The other three are not scaled, so the scaling fact travels per entry rather than being
    // asserted for the whole loop. A blanket "as armed for a batch of N" invites the operator to
    // divide the reported figure by N and conclude they configured a value they did not — which is
    // the same class of misdirection the cap exists to prevent.
    for (name, dur, scaled) in [
        (
            "max_ssa_delivery_time",
            scaled_deadline(cfg.max_ssa_delivery_time, cfg.ssas_per_request),
            true,
        ),
        (
            "max_deposit_wait",
            scaled_deadline(cfg.max_deposit_wait, cfg.ssas_per_request),
            true,
        ),
        ("max_recovery_idle", cfg.max_recovery_idle, false),
        ("max_recovery_time", cfg.max_recovery_time, false),
        ("tombstone_retention_window", cfg.tombstone_retention_window, false),
    ] {
        if dur > MAX_SUPERVISOR_DURATION {
            let armed = if scaled {
                format!(", as armed for a batch of {}", cfg.ssas_per_request)
            } else {
                String::new()
            };
            return Err(TransportSessionError::InvalidConfig(format!(
                "{name} ({dur:?}{armed}) must not exceed {MAX_SUPERVISOR_DURATION:?}"
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
            allow_dynamic_ssa_batches: true,
            max_failed_cycles: 1,
            max_ssa_delivery_time: Duration::from_secs(20),
            commitment_recommit_interval: Duration::from_secs(3),
            max_deposit_wait: Duration::from_secs(60),
            max_recovery_idle: Duration::from_secs(60),
            max_recovery_time: Duration::from_secs(3600),
            max_predeposit_packets: 1024,
            max_served_without_progress: 256,
            max_off_front_share_fraction: 0.25,
            min_share_order_sample: 16384,
            tombstone_retention_window: Duration::from_secs(30),
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
    fn dynamic_ssa_batches_are_enabled_by_default() {
        assert!(SupervisorConfig::default().allow_dynamic_ssa_batches);
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

    /// Both ends of the re-request interval are dead config, and invisible anywhere but here.
    ///
    /// Asserts on the messages: a zero interval also trips nothing else, but an interval at or above
    /// the delivery deadline would stay green under `is_err()` if the branch naming it were deleted,
    /// since a lone `is_err()` cannot see which guard fired.
    #[test]
    fn validation_rejects_a_recommit_interval_that_cannot_fire() {
        let message_of = |cfg: &SupervisorConfig| match validate_pix_supervision(cfg, &valid_rcn_cfg()) {
            Err(TransportSessionError::InvalidConfig(message)) => message,
            other => panic!("expected an invalid-config error, got {other:?}"),
        };

        let mut zero = valid_cfg();
        zero.commitment_recommit_interval = Duration::ZERO;
        assert!(
            message_of(&zero).contains("commitment_recommit_interval must be non-zero"),
            "a zero interval fires on the instant it is armed, so every arriving part answers itself"
        );

        let mut too_long = valid_cfg();
        too_long.commitment_recommit_interval = too_long.max_ssa_delivery_time;
        assert!(
            message_of(&too_long).contains("must be shorter than max_ssa_delivery_time"),
            "an interval at the deadline never fires before the Session closes"
        );

        // And the shipping default satisfies it, which is what makes the rule safe to enforce.
        assert!(validate_pix_supervision(&SupervisorConfig::default(), &SsaReconstructorConfig::default()).is_ok());
    }

    /// Unlike its siblings, this one has to assert on the message.
    ///
    /// Zero is rejected twice over: by the non-zero check, and by
    /// `max_recovery_idle >= max_ack_await_time`, which `ZERO` also fails. `is_err()` cannot see
    /// which fired, so with it the test stays green after the branch it names is deleted. The other
    /// zero-value tests in this module target fields with no second guard and are sound as written.
    #[test]
    fn validation_rejects_zero_max_recovery_idle() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_idle = Duration::ZERO;
        let msg = validate_pix_supervision(&cfg, &valid_rcn_cfg())
            .expect_err("a zero idle deadline must be rejected")
            .to_string();
        assert!(msg.contains("max_recovery_idle must be non-zero"), "{msg}");
    }

    /// Shadowed the same way: `ZERO` also fails `max_recovery_time > max_recovery_idle`.
    #[test]
    fn validation_rejects_zero_max_recovery_time() {
        let mut cfg = valid_cfg();
        cfg.max_recovery_time = Duration::ZERO;
        let msg = validate_pix_supervision(&cfg, &valid_rcn_cfg())
            .expect_err("a zero hard deadline must be rejected")
            .to_string();
        assert!(msg.contains("max_recovery_time must be non-zero"), "{msg}");
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

    /// The tombstone has to outlive the reconstructor's acknowledgement window, or a late ack arrives
    /// with its cycle record already gone.
    ///
    /// Non-zero but too short is the case that needs stating: `valid_cfg()`'s 30 s window never
    /// approaches `valid_rcn_cfg()`'s 10 s, and `validation_rejects_zero_tombstone_retention_window`
    /// reaches the zero check earlier in the validator rather than this rule. Without this test the
    /// branch could be deleted and the suite would stay green.
    #[test]
    fn validation_rejects_a_tombstone_shorter_than_the_ack_window() {
        let mut cfg = valid_cfg();
        let rcn = valid_rcn_cfg();
        // max_ack_await_time is 10 s.
        cfg.tombstone_retention_window = Duration::from_secs(9);
        assert!(validate_pix_supervision(&cfg, &rcn).is_err(), "9 < 10 must reject");
        cfg.tombstone_retention_window = Duration::from_secs(10);
        assert!(
            validate_pix_supervision(&cfg, &rcn).is_ok(),
            "equal is admissible — the rule is >=, unlike the verifier-lifetime rule above"
        );
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

    /// The cap diagnostic must claim batch scaling only for the durations that are scaled.
    ///
    /// Only `max_ssa_delivery_time` and `max_deposit_wait` pass through `scaled_deadline`. Reporting
    /// "as armed for a batch of N" against the other three invites the operator to divide by N and
    /// conclude they configured a value they never set — at `ssas_per_request = 3`, an over-cap
    /// `max_recovery_time` would be read as a third of what it is. `is_err()` cannot see the
    /// difference, which is why the message itself is asserted here.
    #[test]
    fn the_deadline_cap_claims_batch_scaling_only_where_it_scales() {
        const BATCH: usize = 3;
        let rcn = valid_rcn_cfg();

        let mut cfg = valid_cfg();
        cfg.ssas_per_request = BATCH;
        cfg.max_recovery_time = Duration::MAX;
        let msg = validate_pix_supervision(&cfg, &rcn)
            .expect_err("Duration::MAX must be rejected")
            .to_string();
        assert!(
            msg.contains("max_recovery_time"),
            "the unscaled duration must be named: {msg}"
        );
        assert!(
            !msg.contains("as armed for a batch"),
            "max_recovery_time is not batch-scaled, so the message must not say it is: {msg}"
        );

        // The scaled side of the same loop, so the claim is shown to be carried where it is true
        // rather than merely dropped everywhere.
        let mut cfg = valid_cfg();
        cfg.ssas_per_request = BATCH;
        cfg.max_ssa_delivery_time = Duration::MAX;
        let msg = validate_pix_supervision(&cfg, &rcn)
            .expect_err("Duration::MAX must be rejected")
            .to_string();
        assert!(
            msg.contains("max_ssa_delivery_time") && msg.contains("as armed for a batch of 3"),
            "a batch-scaled duration must report the scaling: {msg}"
        );
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

    /// The `Display` value of every close reason is a metric label, so it is API.
    ///
    /// [`crate::telemetry::record_pix_closure`] derives the `reason` label of
    /// `hopr_session_pix_closures_total` from `to_string()`. A variant rename, or a `strum`
    /// attribute added to one, silently renames a metric series and breaks whatever dashboards and
    /// alerts were built on the old name.
    ///
    /// The `Display` string is snapshotted rather than the `Debug` form used by the sibling guard in
    /// [`crate::types`], because `Display` is what actually reaches the label: `#[strum(serialize =
    /// "…")]` makes the two diverge, and only one of them is observable to an operator.
    ///
    /// The array is hand-maintained, so the wildcard-free match below is what keeps it honest — a new
    /// variant fails to compile *here*, next to the array it has to be added to, rather than
    /// quietly falling outside a snapshot that still reads as a guarantee.
    #[test]
    fn pix_close_reason_display_values_are_stable() {
        let reasons = [
            SessionPixCloseReason::CommitmentTimeout,
            SessionPixCloseReason::DepositTimeout,
            SessionPixCloseReason::DepositObserverClosed,
            SessionPixCloseReason::RecoveryIdle,
            SessionPixCloseReason::RecoveryDeadline,
            SessionPixCloseReason::UnverifiableShares,
            SessionPixCloseReason::BatchServedOutOfOrder,
            SessionPixCloseReason::CounterRegression,
            SessionPixCloseReason::InvalidTransition,
            SessionPixCloseReason::NoSsaRemaining,
            SessionPixCloseReason::SupervisorUnavailable,
        ];

        for reason in reasons {
            match reason {
                SessionPixCloseReason::CommitmentTimeout
                | SessionPixCloseReason::DepositTimeout
                | SessionPixCloseReason::DepositObserverClosed
                | SessionPixCloseReason::RecoveryIdle
                | SessionPixCloseReason::RecoveryDeadline
                | SessionPixCloseReason::UnverifiableShares
                | SessionPixCloseReason::BatchServedOutOfOrder
                | SessionPixCloseReason::CounterRegression
                | SessionPixCloseReason::InvalidTransition
                | SessionPixCloseReason::NoSsaRemaining
                | SessionPixCloseReason::SupervisorUnavailable => {}
            }
        }

        let labels = reasons.map(|reason| reason.to_string());
        insta::assert_debug_snapshot!(labels);
    }

    /// One unverifiable-share report must close the Session, and no configuration may change that.
    ///
    /// Pinned end-to-end through the public [`SupervisorConfig`] rather than the state machine's own
    /// internals, because that is the surface a tolerance would come back on. The two limits this
    /// replaces were shipped at zero and read `total > limit`, so re-introducing either — or any other
    /// knob that makes the close conditional — fails here rather than silently restoring tolerance for
    /// a failure that has already doomed the cycle.
    #[test]
    fn one_unverifiable_share_report_closes_the_session() -> anyhow::Result<()> {
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
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::UnverifiableShares))),
            "one unverifiable share must close the session under the default config, got {actions:?}"
        );

        // And under a config whose every other tolerance is at its most permissive, so the close
        // cannot be attributed to something else being strict.
        let permissive = SupervisorConfig {
            max_failed_cycles: usize::MAX,
            max_off_front_share_fraction: 1.0,
            min_share_order_sample: u64::MAX,
            max_predeposit_packets: u64::MAX,
            max_served_without_progress: u64::MAX,
            ..SupervisorConfig::default()
        };
        let (mut supervisor, _) = SessionPixSupervisor::new(permissive, dims, pseudonym, now);
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
            actions
                .iter()
                .any(|a| matches!(a, SessionPixAction::Close(SessionPixCloseReason::UnverifiableShares))),
            "no configuration may tolerate an unverifiable share, got {actions:?}"
        );

        Ok(())
    }
}
