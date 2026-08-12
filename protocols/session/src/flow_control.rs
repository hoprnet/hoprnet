//! Client/ENTRY-side send-window flow control for Session sockets.
//!
//! A `Segmentation`-only Session has no end-to-end flow control: the ENTRY writer runs to
//! completion and floods the return path (data + acknowledgements + SURB replenishments), capped
//! at the EXIT SURB-reply rate. Unpaced, this deterministically stalls.
//!
//! This module provides a self-adapting send window that replaces manual pacing. It speeds up
//! **only** on *proven* delivery and slows down on congestion/loss, so throughput auto-tracks the
//! drain rate and unpaced send becomes safe.
//!
//! # Trust model (why the window is asymmetric)
//!
//! The counterparty is **not** assumed cooperative. A SURB carries `PoRValues`, and reply packets
//! sent with it are paid from the SURB creator's (our) channels along the return path *regardless
//! of payload* (RFC-0005 §3.2). SURBs are therefore prepaid value a greedy EXIT can harvest
//! without serving us; the SURB balancer is the anti-grief throttle. Consequently:
//!
//! 1. **The window only opens on HONEST delivery** — data proven to have reached us: reliable-mode frame
//!    acknowledgements (the `ack_state` machinery) or application-verified return bytes. Nothing else authorizes
//!    speeding up. See `WindowController::on_delivered`.
//! 2. **SURB state may only slow us down, never speed us up.** `SupplyConstraint` can shrink or cap the window; it can
//!    never grow it. The SURB `buffer_level` is partly counterparty-reported and dead-reckoned, so a hostile or lossy
//!    peer must not be able to accelerate us or make us overspend.
//! 3. **A malicious counterparty can only ever make us slower** — never faster, never draining our SURBs beyond the
//!    client-configured ceiling.
//! 4. **1–20 % path loss is expected and recovered** (reliable mode retransmits; the window multiplicatively decreases
//!    and re-grows on delivery).
//! 5. **The anti-grief ⇄ throughput trade is the client's dial** (`FlowControlConfig`), set explicitly — SURB supply is
//!    never silently widened for speed.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

/// How the send window learns that data was delivered (its "honest clock").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlowControlMode {
    /// Reliable-mode frame acknowledgements drive the window. This is the universal honest clock:
    /// it is independent of the application payload, so it works for arbitrary carried protocols
    /// (e.g. a VPN tunnel), and it additionally provides loss recovery via retransmission.
    #[default]
    Reliable,
    /// No end-to-end acknowledgements. The window cannot open on delivery (there is no honest
    /// signal), so it is governed purely by the [`SupplyConstraint`] ceiling and degrades
    /// gracefully. A `Segmentation`-only client accepts loss knowingly. For a loopback echo the
    /// application can still supply a verified-return-byte clock (see the `DeliverySignal` impls).
    Segmentation,
}

/// Client-tunable flow-control parameters. Defaults are deliberately conservative
/// (anti-grief-preserving): the window starts at the floor and only grows on proven delivery, and
/// the opt-in robustness knobs are off. This is the **clean** profile.
///
/// Flow control is enabled per-session by *providing* this config (the transport's
/// `SessionClientConfig::flow_control` is an `Option` — `None` leaves the session unpaced with
/// today's behaviour), so there is no separate `enabled` flag.
#[derive(Clone, Copy, Debug, PartialEq, smart_default::SmartDefault)]
pub struct FlowControlConfig {
    /// Honest-clock mode. Default [`FlowControlMode::Reliable`].
    pub mode: FlowControlMode,

    /// Minimum in-flight bytes that are always admitted, regardless of delivery feedback or SURB
    /// supply. Guarantees the duplex socket never deadlocks (acks/keep-alives can always flow) and
    /// is the hard floor a malicious peer can never push the window below — but also never above
    /// without honest delivery. Keep small (a few frames). Default 4 KiB.
    #[default(4 * 1024)]
    pub min_window_size: usize,

    /// Hard ceiling on the send window, seeded from the bandwidth-delay product
    /// (`drain_rate_hint × rtt`). The window never opens past this even under sustained delivery.
    /// Default 2 MiB.
    #[default(2 * 1024 * 1024)]
    pub max_window_size: usize,

    /// Additive-increase step (bytes added to the window per fully-delivered window) while in
    /// congestion avoidance. Default 16 KiB.
    #[default(16 * 1024)]
    pub ai_step: usize,

    /// Multiplicative-decrease factor applied to the window on loss or a soft backoff hint. Must be
    /// in `(0.0, 1.0)`. Default 0.5 (classic AIMD).
    #[default(0.5)]
    pub md_factor: f64,

    /// In [`FlowControlMode::Segmentation`] there is no honest delivery signal, so admission may
    /// park indefinitely behind a shrinking SURB ceiling. This deadline bounds how long the writer
    /// parks before it re-checks the ceiling / makes keep-progress at the floor. Default 250 ms.
    #[default(Duration::from_millis(250))]
    pub no_honest_deadline: Duration,

    /// **Persist probe (opt-in robustness).** Consecutive no-progress parks before the writer admits
    /// a bounded `min_window_size` beyond `cwnd` (still SURB-capped) to break an end-of-stream tail deadlock
    /// on a slow/throttled return path. `0` **disables** it — the default, i.e. the verified clean
    /// behaviour. A robust profile (e.g. for deliberately SURB-throttled paths) sets ~8 (≈2 s at the
    /// default keep-progress deadline). See the `PacedWriter` admission logic.
    #[default(0)]
    pub persist_stall_parks: u32,

    /// **Frame retransmission budget under flow control (opt-in robustness).** Applied to the
    /// reliable socket's `max_outgoing_frame_retries` when flow control is enabled. `2` (the
    /// original) is the default; a robust profile raises it (~8) so a merely-*delayed* ack on a
    /// temporarily-starved return path recovers the frame instead of abandoning it (an abandoned
    /// frame leaves a gap → stream corruption).
    #[default(2)]
    pub frame_retries: u32,

    /// **Anti-bufferbloat bound (opt-in).** Maximum age of a data-path frame — from its first send
    /// on the sending side, from entering the ordering buffer on the receiving side.
    ///
    /// Older frames are dropped rather than delivered late, so a stall surfaces as clean loss
    /// instead of a multi-second latency tail (the burst-drain "sawtooth"). A packet arriving
    /// seconds late is worthless to a real-time consumer, but the latency it adds is not.
    ///
    /// Distinct from `frame_timeout`, which bounds how long a *missing* frame is waited for; this
    /// bounds how stale a *present* frame may be. `None` (default) keeps the previous behaviour.
    #[default(None)]
    pub max_frame_age: Option<Duration>,

    /// **Reactive return-path re-planning (opt-in).** When set, sustained loss in the return
    /// direction triggers a re-plan of the return path instead of waiting for probing to notice.
    ///
    /// `None` (the default) leaves the return path to be corrected by probing alone, which is
    /// EMA-smoothed behind a path cache and therefore takes tens of seconds — measured at 54–55 %
    /// arrival for a session opened right after a return relayer dies, against 100 % for one opened
    /// 60 s later.
    #[default(None)]
    pub return_path_replan: Option<ReturnPathReplanConfig>,
}

/// Governs when sustained return-direction loss is taken as "this return path is dead".
///
/// The signal is the honest delivery clock's `lost_bytes`: a frame retires as lost when its
/// acknowledgement never came back, which — for a session whose forward path is healthy — means
/// the *return* path dropped the ack. That is visible within an RTT, whereas probing needs tens of
/// seconds to move an EMA behind a 60 s path cache.
#[derive(Clone, Copy, Debug, PartialEq, smart_default::SmartDefault)]
pub struct ReturnPathReplanConfig {
    /// Loss ratio (`lost / (acked + lost)`) above which an observation window counts as bad.
    ///
    /// Default 0.2, matching the reliable-mode tolerance: below it retransmission already recovers
    /// the stream, and re-planning would only add churn.
    #[default(0.2)]
    pub loss_threshold: f64,

    /// Length of one observation window. Default 1 s — long enough to span a few RTTs, so a single
    /// unlucky frame cannot make a window look bad.
    #[default(Duration::from_secs(1))]
    pub window: Duration,

    /// Consecutive bad windows required before re-planning. Default 3, so a blip has to persist for
    /// ~3 s to count as sustained.
    #[default(3)]
    pub sustained_windows: u32,

    /// Minimum interval between re-plans. Default 5 s, so a return path that stays lossy cannot
    /// drive continuous re-planning churn.
    #[default(Duration::from_secs(5))]
    pub cooldown: Duration,
}

impl ReturnPathReplanConfig {
    /// Clamps the parameters so a misconfiguration cannot make the detector fire on every sample.
    fn normalized(self) -> Self {
        Self {
            loss_threshold: if self.loss_threshold.is_finite() {
                self.loss_threshold.clamp(0.0, 1.0)
            } else {
                0.2
            },
            window: self.window.max(Duration::from_millis(1)),
            sustained_windows: self.sustained_windows.max(1),
            cooldown: self.cooldown,
        }
    }
}

/// Notified when the return direction has been losing for long enough that the current return path
/// should be abandoned and re-planned.
///
/// Implemented on the transport side, where the path planner lives; kept as a one-method trait so
/// a test can satisfy it without standing up a planner.
#[cfg_attr(test, mockall::automock)]
pub trait ReturnPathFeedback: Send + Sync {
    /// The return path currently in use is degraded; select a different one.
    fn return_path_degraded(&self);
}

/// Watches the honest delivery clock for *sustained* return-direction loss.
///
/// Deliberately not a simple threshold on a single observation: reliable mode tolerates ~20 % loss
/// because retransmission recovers it, so only loss that persists across several windows means the
/// return path itself is gone rather than merely congested.
pub struct ReturnPathMonitor {
    feedback: std::sync::Arc<dyn ReturnPathFeedback>,
    cfg: ReturnPathReplanConfig,
    window_started: std::time::Instant,
    acked: u64,
    lost: u64,
    /// Consecutive windows whose loss ratio exceeded the threshold.
    bad_windows: u32,
    last_replan: Option<std::time::Instant>,
}

impl ReturnPathMonitor {
    /// Creates a monitor reporting to `feedback`.
    pub fn new(feedback: std::sync::Arc<dyn ReturnPathFeedback>, cfg: ReturnPathReplanConfig) -> Self {
        Self {
            feedback,
            cfg: cfg.normalized(),
            window_started: std::time::Instant::now(),
            acked: 0,
            lost: 0,
            bad_windows: 0,
            last_replan: None,
        }
    }

    /// Folds one delivery observation in, closing the current window if it has elapsed.
    pub fn observe(&mut self, delivered: Delivered) {
        self.acked = self.acked.saturating_add(delivered.acked_bytes as u64);
        self.lost = self.lost.saturating_add(delivered.lost_bytes as u64);

        if self.window_started.elapsed() < self.cfg.window {
            return;
        }
        self.close_window();
    }

    /// Scores the elapsed window and re-plans if loss has been sustained for long enough.
    fn close_window(&mut self) {
        let total = self.acked.saturating_add(self.lost);
        // An idle window carries no evidence either way, so it neither accuses nor exonerates.
        if total > 0 {
            let loss_ratio = self.lost as f64 / total as f64;
            if loss_ratio > self.cfg.loss_threshold {
                self.bad_windows = self.bad_windows.saturating_add(1);
                tracing::debug!(
                    loss_ratio,
                    bad_windows = self.bad_windows,
                    "return-direction loss above threshold"
                );
            } else {
                self.bad_windows = 0;
            }
        }

        self.acked = 0;
        self.lost = 0;
        self.window_started = std::time::Instant::now();

        if self.bad_windows < self.cfg.sustained_windows {
            return;
        }

        let now = std::time::Instant::now();
        if self
            .last_replan
            .is_some_and(|last| now.duration_since(last) < self.cfg.cooldown)
        {
            // Still cooling down from the previous re-plan; keep observing rather than churn.
            return;
        }

        tracing::info!(
            bad_windows = self.bad_windows,
            "sustained return-direction loss; re-planning the return path"
        );
        self.feedback.return_path_degraded();
        self.last_replan = Some(now);
        self.bad_windows = 0;
    }
}

impl FlowControlConfig {
    /// Clamps parameters into their valid ranges (`min_window_size ≤ max_window_size`, `md_factor ∈ (0,1)`,
    /// `ai_step ≥ 1`). Called by [`WindowController::new`] so out-of-range config cannot violate
    /// the invariants.
    fn normalized(self) -> FlowControlConfig {
        let min_window_size = self.min_window_size.max(1);
        FlowControlConfig {
            mode: self.mode,
            min_window_size,
            max_window_size: self.max_window_size.max(min_window_size),
            ai_step: self.ai_step.max(1),
            md_factor: if self.md_factor.is_finite() {
                self.md_factor.clamp(0.01, 0.99)
            } else {
                0.5
            },
            no_honest_deadline: self.no_honest_deadline.max(Duration::from_millis(1)),
            persist_stall_parks: self.persist_stall_parks,
            // At least one retry: under reliable-mode flow control an abandoned frame leaves a gap
            // (stream corruption), so `frame_retries` must never clamp the retry budget to 0.
            frame_retries: self.frame_retries.max(1),
            return_path_replan: self.return_path_replan.map(ReturnPathReplanConfig::normalized),
            // A zero bound would drop every frame on sight; treat it as "not set".
            max_frame_age: self.max_frame_age.filter(|age| !age.is_zero()),
        }
    }

    /// The **robust** profile: the clean defaults plus the opt-in tail-tolerance bundle (persist
    /// probe + larger retransmission budget) for deliberately SURB-throttled / high-latency return
    /// paths. See [`Self::persist_stall_parks`] and [`Self::frame_retries`].
    ///
    /// The larger retry budget alone would let a transport stall be absorbed as buffering and
    /// drained afterwards as a multi-second latency sawtooth, so the profile also bounds frame age
    /// at 2 seconds: past that a frame surfaces as recoverable loss instead of arriving late.
    pub fn robust() -> Self {
        Self {
            persist_stall_parks: 8,
            frame_retries: 8,
            max_frame_age: Some(Duration::from_secs(2)),
            ..Self::default()
        }
    }

    /// Convenience constructor seeding [`max_window_size`](Self::max_window_size) from a bandwidth-delay product.
    ///
    /// `drain_rate_bytes_per_sec` is the estimated rate at which the counterparty can drain the
    /// return path (for a SURB-capped echo: `max_surbs_per_sec / surbs_per_reply_packet ×
    /// bytes_per_packet`). `rtt` is the round-trip delivery latency.
    pub fn with_bdp(mut self, drain_rate_bytes_per_sec: u64, rtt: Duration) -> Self {
        let bdp = (drain_rate_bytes_per_sec as f64 * rtt.as_secs_f64()) as usize;
        self.max_window_size = bdp.max(self.min_window_size);
        self
    }
}

/// Bytes retired from the in-flight window this observation, split by outcome. Produced by a
/// [`DeliverySignal`] and fed to [`WindowController::apply_delivery`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Delivered {
    /// Bytes proven delivered (honest signal → authorizes additive increase).
    pub acked_bytes: usize,
    /// Bytes proven lost (retransmission needed → triggers multiplicative decrease).
    pub lost_bytes: usize,
}

impl Delivered {
    /// Total bytes retired from the in-flight accounting (`acked + lost`).
    #[inline]
    pub fn retired(&self) -> usize {
        self.acked_bytes.saturating_add(self.lost_bytes)
    }
}

/// The honest clock. Reports how many in-flight bytes were delivered or lost since last polled.
///
/// Impl A (preferred): reliable-mode frame acknowledgements — an acked frame retires its bytes as
/// `acked_bytes`; a frame whose retransmissions are exhausted retires as `lost_bytes`.
///
/// Impl B: application-verified return bytes (e.g. a loopback echo the caller can checksum).
pub trait DeliverySignal {
    /// Non-blocking: returns bytes delivered/lost since the previous call (zeroes if none).
    fn poll_delivered(&mut self) -> Delivered;

    /// Best-effort round-trip delivery latency, used to (re)seed the BDP ceiling. `None` if unknown.
    fn rtt_hint(&self) -> Option<Duration>;
}

/// Severity of a SURB-supply backoff request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backoff {
    /// Buffer running low (below watermark): multiplicatively decrease.
    Soft,
    /// SURB distress / out-of-SURBs: collapse to the floor.
    Hard,
}

/// The SURB-supply ceiling. **Down-only**: it may cap or shrink the window, never open it.
///
/// This is the anti-grief governor. A healthy buffer returns [`backoff_hint`](Self::backoff_hint)
/// `None` — it does *not* mean "go faster"; only [`DeliverySignal`] can authorize that.
pub trait SupplyConstraint {
    /// Maximum in-flight bytes the current SURB stock can support
    /// (`buffer_level × bytes_per_reply_packet`). The window is capped to this.
    fn max_admissible_inflight(&self) -> usize;

    /// Backoff request derived from SURB distress signals. `None` when the buffer is healthy —
    /// never a signal to open the window.
    fn backoff_hint(&self) -> Option<Backoff>;
}

/// Pure AIMD send-window controller. Holds no I/O; every state transition is a plain method so the
/// invariants are exhaustively unit-testable. Byte-based (not frame-based) so it is agnostic to
/// frame sizing.
#[derive(Clone, Debug)]
pub struct WindowController {
    cfg: FlowControlConfig,
    /// Current congestion window in bytes, always within `[min_window_size, max_window_size]`.
    cwnd: usize,
    /// Bytes sent but not yet delivered or lost.
    inflight: usize,
    /// Bytes delivered toward the next additive-increase step (Reno-style congestion avoidance).
    ai_accumulated_size: usize,
}

impl WindowController {
    /// Creates a controller with the window at the floor (`min_window_size`). The window can only grow from
    /// here via proven delivery — so before any honest signal, the peer cannot make it exceed
    /// `min_window_size` (invariant 1 & 3).
    pub fn new(cfg: FlowControlConfig) -> Self {
        let cfg = cfg.normalized();
        Self {
            cwnd: cfg.min_window_size,
            inflight: 0,
            ai_accumulated_size: 0,
            cfg,
        }
    }

    /// Current window size in bytes (for diagnostics/tests).
    #[inline]
    pub fn window(&self) -> usize {
        self.cwnd
    }

    /// Current in-flight bytes (sent, not yet retired).
    #[inline]
    pub fn inflight(&self) -> usize {
        self.inflight
    }

    /// The active mode.
    #[inline]
    pub fn mode(&self) -> FlowControlMode {
        self.cfg.mode
    }

    /// The configured minimum window (duplex floor / persist-probe size).
    #[inline]
    pub fn min_window_size(&self) -> usize {
        self.cfg.min_window_size
    }

    /// Records `bytes` admitted onto the wire. Increases the in-flight accounting only; never grows
    /// the window.
    #[inline]
    pub fn on_sent(&mut self, bytes: usize) {
        self.inflight = self.inflight.saturating_add(bytes);
    }

    /// **The only path that grows the window.** Retires `bytes` of proven-delivered data and
    /// performs Reno congestion avoidance: roughly `+ai_step` per fully-delivered window, capped at
    /// `max_window_size`.
    pub fn on_delivered(&mut self, bytes: usize) {
        self.inflight = self.inflight.saturating_sub(bytes);
        if self.cwnd >= self.cfg.max_window_size {
            return;
        }
        self.ai_accumulated_size = self.ai_accumulated_size.saturating_add(bytes);
        // Emit one additive step per window's worth of delivered bytes.
        while self.ai_accumulated_size >= self.cwnd {
            self.ai_accumulated_size -= self.cwnd;
            self.cwnd = self.cwnd.saturating_add(self.cfg.ai_step).min(self.cfg.max_window_size);
            if self.cwnd >= self.cfg.max_window_size {
                self.ai_accumulated_size = 0;
                break;
            }
        }
    }

    /// Retires `bytes` of proven-lost data and multiplicatively decreases the window (never below
    /// `min_window_size`). Loss recovery itself (retransmission) is the reliable socket's job.
    pub fn on_lost(&mut self, bytes: usize) {
        self.inflight = self.inflight.saturating_sub(bytes);
        self.decrease(self.cfg.md_factor);
    }

    /// Applies a batched [`Delivered`] observation from a [`DeliverySignal`].
    pub fn apply_delivery(&mut self, d: Delivered) {
        if d.acked_bytes > 0 {
            self.on_delivered(d.acked_bytes);
        }
        if d.lost_bytes > 0 {
            self.on_lost(d.lost_bytes);
        }
    }

    /// Applies a SURB-supply backoff. Down-only: `Soft` multiplicatively decreases, `Hard`
    /// collapses to the floor. Never grows the window.
    pub fn apply_backoff(&mut self, b: Backoff) {
        match b {
            Backoff::Soft => self.decrease(self.cfg.md_factor),
            Backoff::Hard => self.cwnd = self.cfg.min_window_size,
        }
    }

    /// Effective window against a raw ceiling value (bytes): `min(cwnd, ceiling)`, but never below
    /// `min_window_size` (duplex floor). The ceiling can only shrink the result — invariant 2 — and it
    /// does **not** mutate `cwnd`, so a transient dip is fully recovered the instant the ceiling lifts.
    pub fn effective_window_for(&self, ceiling: usize) -> usize {
        self.cwnd.min(ceiling).max(self.cfg.min_window_size)
    }

    /// Bytes admissible now against a raw ceiling value: `effective_window − inflight` (saturating).
    pub fn admissible_for(&self, ceiling: usize) -> usize {
        self.effective_window_for(ceiling).saturating_sub(self.inflight)
    }

    /// Effective window against a [`SupplyConstraint`] (convenience wrapper over
    /// [`effective_window_for`](Self::effective_window_for)).
    pub fn effective_window(&self, supply: &impl SupplyConstraint) -> usize {
        self.effective_window_for(supply.max_admissible_inflight())
    }

    /// Bytes that may be admitted right now against `supply`. Zero means park until delivery retires
    /// in-flight bytes (or the ceiling lifts).
    pub fn admissible(&self, supply: &impl SupplyConstraint) -> usize {
        self.admissible_for(supply.max_admissible_inflight())
    }

    /// Multiplicative decrease helper, flooring at `min_window_size`.
    fn decrease(&mut self, factor: f64) {
        let reduced = (self.cwnd as f64 * factor) as usize;
        self.cwnd = reduced.max(self.cfg.min_window_size);
        self.ai_accumulated_size = 0;
    }
}

/// Shared, lock-free honest-clock meter. Producers bump it **in place** with a single atomic add —
/// the reliable ack machinery on ack / retransmission-exhaustion (impl A), or an
/// application-return-byte reader (impl B). The window driver reads byte deltas via [`DeliveryClock`].
/// No channel, no per-frame allocation, no dedup bookkeeping (a duplicate ack merely over-credits by
/// one frame, which the SURB ceiling and `cwnd` still bound). Modelled on the existing atomic
/// `BalancerStateValues`.
#[derive(Clone, Default)]
pub struct DeliveryMeter(Arc<DeliveryAtomics>);

#[derive(Default)]
struct DeliveryAtomics {
    acked_bytes: AtomicU64,
    lost_bytes: AtomicU64,
}

impl DeliveryMeter {
    /// Records `bytes` proven delivered — a received frame ack, or application-verified return bytes.
    #[inline]
    pub fn record_acked(&self, bytes: usize) {
        self.0.acked_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Records `bytes` proven lost — sender-side retransmissions exhausted.
    #[inline]
    pub fn record_lost(&self, bytes: usize) {
        self.0.lost_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    #[inline]
    fn load(&self) -> (u64, u64) {
        (
            self.0.acked_bytes.load(Ordering::Relaxed),
            self.0.lost_bytes.load(Ordering::Relaxed),
        )
    }
}

/// Frame-granular tap installed into the reliable socket: it pairs a [`DeliveryMeter`] with a frame's
/// byte size, so the ack machinery can report deliveries in place without tracking byte counts.
/// Cheap to clone (`Arc` + `usize`).
#[derive(Clone)]
pub struct DeliveryTap {
    meter: DeliveryMeter,
    bytes_per_frame: usize,
}

impl DeliveryTap {
    /// Pairs `meter` with the socket's frame byte size.
    pub fn new(meter: DeliveryMeter, bytes_per_frame: usize) -> Self {
        Self {
            meter,
            bytes_per_frame: bytes_per_frame.max(1),
        }
    }

    /// The receiver acknowledged a frame — proof of delivery (impl A).
    #[inline]
    pub fn on_acked_frame(&self) {
        self.meter.record_acked(self.bytes_per_frame);
    }

    /// A frame's sender-side retransmissions were exhausted — given up as lost (impl A).
    #[inline]
    pub fn on_lost_frame(&self) {
        self.meter.record_lost(self.bytes_per_frame);
    }
}

/// Delta reader over a shared [`DeliveryMeter`], implementing [`DeliverySignal`]. Each poll returns
/// bytes delivered/lost since the previous poll. One reader serves either clock — impl A (reliable
/// acks via [`DeliveryTap`]) or impl B (return bytes via [`DeliveryMeter::record_acked`]) — because
/// both simply add to the same meter.
pub struct DeliveryClock {
    meter: DeliveryMeter,
    seen_acked: u64,
    seen_lost: u64,
    rtt_hint: Option<Duration>,
}

impl DeliveryClock {
    /// Creates a reader over `meter`. `rtt_hint` seeds the BDP ceiling when known.
    pub fn new(meter: DeliveryMeter, rtt_hint: Option<Duration>) -> Self {
        Self {
            meter,
            seen_acked: 0,
            seen_lost: 0,
            rtt_hint,
        }
    }
}

impl DeliverySignal for DeliveryClock {
    fn poll_delivered(&mut self) -> Delivered {
        let (acked, lost) = self.meter.load();
        let d = Delivered {
            acked_bytes: acked.saturating_sub(self.seen_acked) as usize,
            lost_bytes: lost.saturating_sub(self.seen_lost) as usize,
        };
        self.seen_acked = acked;
        self.seen_lost = lost;
        d
    }

    fn rtt_hint(&self) -> Option<Duration> {
        self.rtt_hint
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    /// Builds a monitor with a very short window so the tests do not sleep for seconds.
    fn monitor(feedback: Arc<dyn ReturnPathFeedback>, sustained_windows: u32, cooldown: Duration) -> ReturnPathMonitor {
        ReturnPathMonitor::new(
            feedback,
            ReturnPathReplanConfig {
                loss_threshold: 0.2,
                window: Duration::from_millis(10),
                sustained_windows,
                cooldown,
            },
        )
    }

    fn delivered(acked: usize, lost: usize) -> Delivered {
        Delivered {
            acked_bytes: acked,
            lost_bytes: lost,
        }
    }

    /// Feeds one observation and lets the window elapse, so the next observation closes it.
    fn close_window(m: &mut ReturnPathMonitor, d: Delivered) {
        m.observe(d);
        std::thread::sleep(Duration::from_millis(12));
        m.observe(delivered(0, 0));
    }

    #[test]
    fn return_path_monitor_should_replan_after_sustained_loss() {
        let mut feedback = MockReturnPathFeedback::new();
        feedback.expect_return_path_degraded().once().return_const(());

        let mut m = monitor(Arc::new(feedback), 3, Duration::from_secs(60));

        // Three consecutive windows at 50 % loss — well above the 20 % tolerance.
        for _ in 0..3 {
            close_window(&mut m, delivered(1_000, 1_000));
        }
    }

    #[test]
    fn return_path_monitor_should_ignore_loss_within_tolerance() {
        let mut feedback = MockReturnPathFeedback::new();
        feedback.expect_return_path_degraded().never();

        let mut m = monitor(Arc::new(feedback), 3, Duration::from_secs(60));

        // 10 % loss: retransmission recovers this, so it must not cause churn.
        for _ in 0..6 {
            close_window(&mut m, delivered(9_000, 1_000));
        }
    }

    #[test]
    fn return_path_monitor_should_require_loss_to_be_sustained() {
        let mut feedback = MockReturnPathFeedback::new();
        feedback.expect_return_path_degraded().never();

        let mut m = monitor(Arc::new(feedback), 3, Duration::from_secs(60));

        // A bad window followed by a good one resets the streak, so it never reaches 3.
        for _ in 0..4 {
            close_window(&mut m, delivered(1_000, 1_000));
            close_window(&mut m, delivered(10_000, 0));
        }
    }

    #[test]
    fn return_path_monitor_should_rate_limit_replans() {
        let mut feedback = MockReturnPathFeedback::new();
        // Six bad windows at `sustained_windows = 1` would be six re-plans without the cooldown.
        feedback.expect_return_path_degraded().once().return_const(());

        let mut m = monitor(Arc::new(feedback), 1, Duration::from_secs(60));

        for _ in 0..6 {
            close_window(&mut m, delivered(0, 1_000));
        }
    }

    #[test]
    fn return_path_monitor_should_ignore_idle_windows() {
        let mut feedback = MockReturnPathFeedback::new();
        feedback.expect_return_path_degraded().never();

        let mut m = monitor(Arc::new(feedback), 1, Duration::from_secs(60));

        // No traffic at all carries no evidence: it must neither accuse nor exonerate.
        for _ in 0..5 {
            close_window(&mut m, delivered(0, 0));
        }
    }

    use super::*;

    #[test]
    fn robust_profile_should_bound_frame_age() {
        assert_eq!(FlowControlConfig::robust().max_frame_age, Some(Duration::from_secs(2)));
        assert_eq!(FlowControlConfig::default().max_frame_age, None);
    }

    /// A `SupplyConstraint` with a fixed ceiling and no distress — the "honest, generous supply"
    /// baseline used to isolate the delivery clock.
    struct FixedSupply {
        ceiling: usize,
        backoff: Option<Backoff>,
    }
    impl FixedSupply {
        fn generous() -> Self {
            Self {
                ceiling: usize::MAX,
                backoff: None,
            }
        }
    }
    impl SupplyConstraint for FixedSupply {
        fn max_admissible_inflight(&self) -> usize {
            self.ceiling
        }

        fn backoff_hint(&self) -> Option<Backoff> {
            self.backoff
        }
    }

    fn cfg() -> FlowControlConfig {
        FlowControlConfig {
            min_window_size: 1_000,
            max_window_size: 100_000,
            ai_step: 1_000,
            md_factor: 0.5,
            ..Default::default()
        }
    }

    #[test]
    fn starts_at_floor() {
        let w = WindowController::new(cfg());
        assert_eq!(w.window(), 1_000, "window must start at min_window_size, not above");
    }

    #[test]
    fn normalization_clamps_out_of_range_config() {
        let w = WindowController::new(FlowControlConfig {
            min_window_size: 5_000,
            max_window_size: 1_000, // below min_window_size → must be raised to min_window_size
            ai_step: 0,             // → 1
            md_factor: 2.0,         // → clamped into (0,1)
            ..Default::default()
        });
        assert_eq!(w.window(), 5_000);
        assert!(w.cfg.max_window_size >= w.cfg.min_window_size);
        assert!(w.cfg.md_factor > 0.0 && w.cfg.md_factor < 1.0);
        assert!(w.cfg.ai_step >= 1);
    }

    #[test]
    fn additive_increase_one_step_per_window() {
        let mut w = WindowController::new(cfg());
        let start = w.window(); // 1_000
        w.on_sent(start);
        w.on_delivered(start); // exactly one window delivered → +ai_step
        assert_eq!(w.window(), start + 1_000);
    }

    #[test]
    fn additive_increase_capped_at_max_win() {
        let mut w = WindowController::new(cfg());
        // Deliver far more than needed to reach the ceiling.
        for _ in 0..1_000 {
            let win = w.window();
            w.on_sent(win);
            w.on_delivered(win);
        }
        assert_eq!(w.window(), 100_000, "must not exceed max_window_size");
    }

    #[test]
    fn multiplicative_decrease_on_loss() {
        let mut w = WindowController::new(cfg());
        // Grow first so the decrease is observable.
        for _ in 0..10 {
            let win = w.window();
            w.on_sent(win);
            w.on_delivered(win);
        }
        let before = w.window();
        w.on_sent(before);
        w.on_lost(before);
        assert_eq!(w.window(), before / 2, "loss must halve the window");
    }

    #[test]
    fn decrease_never_below_floor() {
        let mut w = WindowController::new(cfg());
        for _ in 0..100 {
            w.on_lost(0);
        }
        assert_eq!(w.window(), 1_000, "must never shrink below min_window_size");
    }

    // ---- Invariant 1 & 3: adversarial peer cannot open the window ----

    #[test]
    fn adversarial_healthy_supply_no_delivery_cannot_open_window() {
        // Peer reports an enormous healthy SURB ceiling but delivers nothing.
        let mut w = WindowController::new(cfg());
        let supply = FixedSupply::generous(); // ceiling = usize::MAX, no backoff
        // Simulate many admission cycles with zero delivery feedback.
        for _ in 0..1_000 {
            let can = w.admissible(&supply);
            w.on_sent(can); // send whatever is admissible into the void
            // no on_delivered — nothing comes back
        }
        assert_eq!(
            w.window(),
            1_000,
            "no honest delivery ⇒ window must stay pinned at min_window_size regardless of reported supply"
        );
        // And it must never admit more than one floor-window of unacked data.
        assert_eq!(
            w.admissible(&supply),
            0,
            "with a full in-flight floor and no delivery, nothing more may be admitted"
        );
    }

    #[test]
    fn generous_ceiling_cannot_raise_effective_window_above_cwnd() {
        let w = WindowController::new(cfg());
        let supply = FixedSupply::generous();
        assert_eq!(
            w.effective_window(&supply),
            w.window(),
            "an over-generous ceiling must not raise the window above cwnd"
        );
    }

    // ---- Invariant 2: SURB supply is down-only ----

    #[test]
    fn supply_ceiling_only_shrinks_window() {
        let mut w = WindowController::new(cfg());
        // Grow the honest window to near max.
        for _ in 0..50 {
            let win = w.window();
            w.on_sent(win);
            w.on_delivered(win);
        }
        let cwnd = w.window();
        let tight = FixedSupply {
            ceiling: cwnd / 4,
            backoff: None,
        };
        assert_eq!(w.effective_window(&tight), cwnd / 4, "tight ceiling clamps down");
        let loose = FixedSupply {
            ceiling: cwnd * 10,
            backoff: None,
        };
        assert_eq!(
            w.effective_window(&loose),
            cwnd,
            "loose ceiling cannot raise above cwnd"
        );
    }

    // ---- Hysteresis: a supply-ceiling dip clamps the effective window but PRESERVES cwnd ----

    #[test]
    fn ceiling_clamp_preserves_cwnd() {
        let mut w = WindowController::new(cfg());
        // Grow the honest window well above the floor.
        for _ in 0..30 {
            let win = w.window();
            w.on_sent(win);
            w.on_delivered(win);
        }
        let grown = w.window();
        assert!(grown > 10_000);

        // A tight ceiling clamps the *effective* window right down...
        assert_eq!(w.effective_window_for(1_000), 1_000);
        // ...but must NOT destroy the learned cwnd (this is the whole fix — no cliff collapse).
        assert_eq!(w.window(), grown, "ceiling clamp must not mutate cwnd");

        // The instant the ceiling lifts, the full learned window is available again — no re-growth.
        assert_eq!(w.effective_window_for(usize::MAX), grown);
        assert_eq!(w.admissible_for(usize::MAX), grown);
    }

    #[test]
    fn hard_backoff_collapses_to_floor_soft_halves() {
        let mut w = WindowController::new(cfg());
        for _ in 0..20 {
            let win = w.window();
            w.on_sent(win);
            w.on_delivered(win);
        }
        let grown = w.window();
        assert!(grown > 2_000);

        let mut soft = w.clone();
        soft.apply_backoff(Backoff::Soft);
        assert_eq!(soft.window(), grown / 2);

        w.apply_backoff(Backoff::Hard);
        assert_eq!(w.window(), 1_000, "hard backoff collapses to floor");
    }

    // ---- Invariant 4: loss is recovered (window re-grows after decrease) ----

    #[test]
    fn should_recover_after_loss_burst() {
        let mut w = WindowController::new(cfg());
        for _ in 0..30 {
            let win = w.window();
            w.on_sent(win);
            w.on_delivered(win);
        }
        let peak = w.window();
        // 20% loss burst.
        let chunk = w.window();
        w.on_sent(chunk);
        w.on_lost(chunk);
        assert!(w.window() < peak, "window backs off on loss");
        // Sustained delivery re-opens it.
        for _ in 0..30 {
            let win = w.window();
            w.on_sent(win);
            w.on_delivered(win);
        }
        assert!(w.window() > peak / 2, "window recovers with renewed delivery");
    }

    #[test]
    fn apply_delivery_batches_ack_and_loss() {
        let mut w = WindowController::new(cfg());
        w.on_sent(2_000);
        w.apply_delivery(Delivered {
            acked_bytes: 1_000,
            lost_bytes: 1_000,
        });
        assert_eq!(w.inflight(), 0, "both acked and lost retire in-flight bytes");
    }

    #[test]
    fn inflight_accounting_saturates() {
        let mut w = WindowController::new(cfg());
        w.on_sent(500);
        w.on_delivered(10_000); // more than in-flight → saturates at 0, no underflow
        assert_eq!(w.inflight(), 0);
    }

    // ---- DeliveryMeter + DeliveryClock: the in-place atomic honest clock ----

    #[test]
    fn delivery_clock_reports_byte_deltas() {
        let meter = DeliveryMeter::default();
        let mut clock = DeliveryClock::new(meter.clone(), Some(Duration::from_millis(50)));
        meter.record_acked(2_000);
        meter.record_lost(1_000);
        let d = clock.poll_delivered();
        assert_eq!(d.acked_bytes, 2_000);
        assert_eq!(d.lost_bytes, 1_000);
        assert_eq!(clock.rtt_hint(), Some(Duration::from_millis(50)));
        // Only the delta is reported next poll.
        assert_eq!(clock.poll_delivered(), Delivered::default());
        meter.record_acked(300);
        assert_eq!(clock.poll_delivered().acked_bytes, 300);
    }

    #[test]
    fn delivery_tap_reports_whole_frames() {
        // impl A: the reliable ack tap reports one frame's bytes per event.
        let meter = DeliveryMeter::default();
        let tap = DeliveryTap::new(meter.clone(), 1_000);
        let mut clock = DeliveryClock::new(meter, None);
        tap.on_acked_frame();
        tap.on_acked_frame();
        tap.on_lost_frame();
        let d = clock.poll_delivered();
        assert_eq!(d.acked_bytes, 2_000);
        assert_eq!(d.lost_bytes, 1_000);
    }

    #[test]
    fn delivery_clock_drives_window_growth() {
        let meter = DeliveryMeter::default();
        let tap = DeliveryTap::new(meter.clone(), 1_000);
        let mut clock = DeliveryClock::new(meter, None);
        let mut w = WindowController::new(cfg());
        let supply = FixedSupply::generous();
        let start = w.window();
        // Deliver a full window's worth of frames via the tap.
        let frames = start / 1_000 + 1;
        for _ in 0..frames {
            w.on_sent(1_000);
            tap.on_acked_frame();
        }
        w.apply_delivery(clock.poll_delivered());
        assert!(w.window() > start, "honest acks must open the window");
        assert!(w.admissible(&supply) > 0);
    }

    #[test]
    fn return_bytes_share_the_same_meter() {
        // impl B: the application-return-byte reader bumps the same meter directly.
        let meter = DeliveryMeter::default();
        let mut clock = DeliveryClock::new(meter.clone(), None);
        meter.record_acked(1_500);
        meter.record_acked(500);
        let d = clock.poll_delivered();
        assert_eq!(d.acked_bytes, 2_000, "sum of returned bytes since last poll");
        assert_eq!(d.lost_bytes, 0);
    }

    #[test]
    fn bdp_seed_sets_ceiling() {
        // 700 pkt/s × ~1 KB × 0.1 s ≈ 70 KB ceiling.
        let rate: u64 = 700 * 1024;
        let cfg = FlowControlConfig::default().with_bdp(rate, Duration::from_millis(100));
        assert_eq!(cfg.max_window_size, (rate as f64 * 0.1) as usize);
    }
}
