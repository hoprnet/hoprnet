use std::{
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, AtomicU8, AtomicU64},
    },
    time::{Duration, Instant},
};

/// Monotonic origin for the degraded-return-path deadline.
///
/// An `Instant` cannot live in an atomic, and the deadline is written by one layer and read by
/// another, so it travels as milliseconds elapsed from a fixed point. Monotonic rather than
/// wall-clock, so a clock adjustment cannot extend or cancel the window.
static EPOCH: LazyLock<Instant> = LazyLock::new(Instant::now);

use futures::{StreamExt, pin_mut};
use hopr_crypto_packet::prelude::{PacketSignal, PacketSignals};
use hopr_utils::runtime::AbortHandle;
use tracing::{Instrument, instrument};

use super::{
    BalancerControllerBounds, ControlInput, MIN_BALANCER_SAMPLING_INTERVAL, SimpleSurbFlowEstimator,
    SurbBalancerController, SurbFlowController, SurbFlowEstimator,
};
use crate::SessionId;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TARGET_ERROR_ESTIMATE: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_target_error_estimate",
            "Target error estimation by the SURB balancer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CONTROL_OUTPUT: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_control_output",
            "Control output of the SURB balancer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CURRENT_BUFFER: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_current_buffer_estimate",
            "Estimated number of SURBs in the buffer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CURRENT_TARGET: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_current_buffer_target",
            "Current target (setpoint) number of SURBs in the buffer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_SURB_RATE: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_surbs_rate",
            "Estimation of SURB rate per second (positive is buffer surplus, negative is buffer loss)",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CONSUMED_RATE: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_consumed_surbs_per_sec",
            "SURBs consumed per second, averaged over the last reporting window",
            &["session_id"]
    ).unwrap();
    static ref METRIC_ORGANIC_RATE: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_organic_surbs_per_sec",
            "SURBs per second piggybacked on Session data rather than delivered by keep-alives",
            &["session_id"]
    ).unwrap();
    static ref METRIC_KEEP_ALIVE_PACKET_RATE: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_keep_alive_packets_per_sec",
            "Keep-alive packets per second delivering SURBs (sent by the Entry, received by the Exit)",
            &["session_id"]
    ).unwrap();
    static ref METRIC_KEEP_ALIVE_WIRE_RATE: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_surb_balancer_keep_alive_wire_bytes_per_sec",
            "Wire bytes per second spent on SURB-bearing keep-alive packets",
            &["session_id"]
    ).unwrap();
}

/// Time constant over which the consumption fed forward to the controller is smoothed.
///
/// Long enough to ride out the jitter of packet arrivals between samples, short enough that a
/// download starting or stopping is followed within a couple of seconds.
const CONSUMPTION_SMOOTHING: Duration = Duration::from_secs(1);

/// Per-second SURB flow over one window between two estimator snapshots.
///
/// Splits production into what rode along with Session data and what keep-alives had to carry on
/// their own: the first is free on the wire, the second is the upstream budget the balancer spends.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SurbFlowRates {
    consumed: f64,
    organic: f64,
    keep_alive_surbs: f64,
    keep_alive_packets: f64,
}

impl SurbFlowRates {
    /// Rates between the `earlier` and `later` snapshots taken `window` apart.
    ///
    /// Counters are monotonic, but the keep-alive attribution is recorded a moment before the
    /// production it belongs to, so a snapshot taken in between can momentarily show more keep-alive
    /// SURBs than produced ones. Saturating keeps that from surfacing as a huge organic rate.
    fn between(earlier: &SimpleSurbFlowEstimator, later: &SimpleSurbFlowEstimator, window: Duration) -> Self {
        let secs = window.as_secs_f64();
        if secs <= 0.0 {
            return Self::default();
        }
        let delta = |later: u64, earlier: u64| later.saturating_sub(earlier) as f64;

        let produced = delta(later.produced, earlier.produced);
        let keep_alive_surbs = delta(later.keep_alive_surbs, earlier.keep_alive_surbs);
        Self {
            consumed: delta(later.consumed, earlier.consumed) / secs,
            organic: (produced - keep_alive_surbs).max(0.0) / secs,
            keep_alive_surbs: keep_alive_surbs / secs,
            keep_alive_packets: delta(later.keep_alive_packets, earlier.keep_alive_packets) / secs,
        }
    }

    /// Wire bytes per second the keep-alive packets occupied.
    fn keep_alive_wire_bytes(&self) -> f64 {
        self.keep_alive_packets * hopr_crypto_packet::prelude::HoprPacket::SIZE as f64
    }

    /// Average number of SURBs each keep-alive packet carried, or zero when none was sent.
    fn surbs_per_keep_alive(&self) -> f64 {
        if self.keep_alive_packets > 0.0 {
            self.keep_alive_surbs / self.keep_alive_packets
        } else {
            0.0
        }
    }
}

/// Configuration for the `SurbBalancer`.
#[derive(Clone, Copy, Debug, PartialEq, smart_default::SmartDefault)]
pub struct SurbBalancerConfig {
    /// The desired number of SURBs to be always kept as a buffer locally or at the Session counterparty.
    ///
    /// The `SurbBalancer` will try to maintain approximately this number of SURBs
    /// locally or remotely (at the counterparty) at all times.
    ///
    /// The local buffer is maintained by regulating (`SurbFlowController`) the egress from the Session.
    /// The remote buffer (at session counterparty) is maintained by regulating the flow of non-organic SURBs via
    /// keep-alive messages.
    ///
    /// It does not make sense to set this value higher than the [`max_surb_buffer_size`](crate::SessionManagerConfig)
    /// configuration at the counterparty.
    ///
    /// Default is 7000 SURBs.
    #[default(7_000)]
    pub target_surb_buffer_size: u64,
    /// Maximum outflow of SURBs.
    ///
    /// - In the context of the local SURB buffer (Entry), this is the maximum egress Session traffic (= SURB
    ///   consumption).
    /// - In the context of the remote SURB buffer (Exit), this is the maximum egress of keep-alive messages to the
    ///   counterparty (= artificial SURB production).
    ///
    /// On the Entry this is an upstream budget, and a larger one than `max_surbs_per_sec × SURB_SIZE`
    /// suggests: each keep-alive is a whole packet carrying two SURBs, so every SURB it delivers
    /// costs half a packet on the wire. Derive the value from an upstream bit rate with
    /// [`max_surbs_per_sec_for_wire_bps`](crate::max_surbs_per_sec_for_wire_bps), and report what a
    /// given value costs with [`keep_alive_wire_bps`](crate::keep_alive_wire_bps).
    ///
    /// The default is 5000 (2500 keep-alive packets/second, about 29 Mbit/s of upstream).
    #[default(5_000)]
    pub max_surbs_per_sec: u64,

    /// Sets what percentage of the target buffer size should be discarded at each window.
    ///
    /// The `SurbBalancer` will discard the given percentage of `target_surb_buffer_size` at each
    /// window with the given `Duration`.
    ///
    /// The default is `(60, 0.05)` (5% of the target buffer size is discarded every 60 seconds).
    #[default(_code = "Some((Duration::from_secs(60), 0.05))")]
    pub surb_decay: Option<(Duration, f64)>,

    /// Keeps producing SURBs while the return path is known to be failing, instead of reading the
    /// resulting silence as a full counterparty buffer.
    ///
    /// The remote buffer is estimated as *produced − consumed*, and consumption is only observed
    /// when a reply reaches us. A return path that drops every reply therefore looks exactly like a
    /// counterparty that is well stocked, so production is throttled at the very moment the
    /// counterparty is in fact draining towards empty and needs more.
    ///
    /// Distinguishing that from a peer which simply has nothing to say is impossible from here --
    /// both show no consumption -- so this only takes effect once an outside observer marks the
    /// return path degraded, and it expires on its own if no further evidence arrives.
    ///
    /// Off by default: sustaining production spends bandwidth on a path that may be genuinely idle,
    /// which is only worth it for sessions that value recovery latency over that bandwidth.
    #[default(false)]
    pub sustain_on_return_path_loss: bool,
}

impl SurbBalancerConfig {
    /// Convenience function to convert the [`SurbBalancerConfig`] into `BalancerControllerBounds`.
    #[inline]
    pub fn as_controller_bounds(&self) -> BalancerControllerBounds {
        BalancerControllerBounds::new(self.target_surb_buffer_size, self.max_surbs_per_sec)
    }
}

/// Runtime state of the `SurbBalancer`.
#[derive(Debug, Default)]
pub struct BalancerStateValues {
    pub target_surb_buffer_size: AtomicU64,
    pub max_surbs_per_sec: AtomicU64,
    pub decay_duration_msec: AtomicU64,
    pub decay_volume_pct: AtomicU8,
    pub buffer_level: AtomicU64,
    /// Whether this session opted into sustaining production through return-path loss.
    pub sustain_on_return_path_loss: AtomicBool,
    /// How many SURBs the counterparty can physically hold, or 0 when unknown.
    ///
    /// The estimate is `produced - consumed`, and consumption is only observed once a reply
    /// arrives -- so a return path that drops replies lets the believed level grow without bound.
    /// The counterparty's store is a ring buffer that evicts the oldest entry on overflow, so
    /// everything above its capacity was discarded on arrival and was never a real level. Measured
    /// during an outage: 51 917 believed against a 15 000-entry store.
    ///
    /// ## Why evictions are not subtracted from the level
    ///
    /// The counterparty reports its evictions (`num_evicted_surbs` on the incoming packet), so the
    /// level could be corrected to the exact truth instead of merely bounded here. It deliberately
    /// is not, because the clamp below is `max(capacity, target)` rather than `capacity`: a level
    /// inflated past a full buffer still climbs to the target and shuts organic production off,
    /// whereas an accurate level pins at the counterparty's real capacity. If that capacity is below
    /// the target -- which nothing prevents, since this figure is the *local* store size and the
    /// counterparty may be smaller -- the accurate level never reaches the target, production never
    /// stops, and the buffer evicts forever. The imprecise estimate fails safe and the precise one
    /// does not, so the eviction count stays an observability signal.
    pub counterparty_buffer_capacity: AtomicU64,
    /// Milliseconds from the crate-internal `EPOCH` monotonic origin until which the return path
    /// counts as degraded.
    ///
    /// A deadline rather than a flag: it is set by a layer that observes the return path and read
    /// here, and nothing is guaranteed to come back and clear it. Expiring on its own bounds the
    /// damage of a marker that is never withdrawn to a short over-production instead of a session
    /// that mints forever.
    pub return_path_degraded_until_ms: AtomicU64,
    /// Whether the counterparty's last packet said it was running low on SURBs of its own.
    ///
    /// A plain flag with no deadline, unlike `return_path_degraded_until_ms` above, because the two
    /// fail in opposite directions: a degraded-path marker that is never withdrawn makes this side
    /// mint forever, whereas a distress flag that is never withdrawn merely keeps organic production
    /// at one SURB per packet — the behaviour that predates the gate. A flag whose stuck state is
    /// the old behaviour does not need to expire.
    ///
    /// It clears on its own in the normal case: the counterparty recomputes both SURB signals on
    /// every return packet and strips them once its pool recovers, so the next healthy packet
    /// resets this.
    ///
    /// Last write wins, and the signal is recorded at dispatch -- ahead of Session sequencing -- so
    /// a reordered clean packet can clear a distress signal the counterparty sent after it. That
    /// costs the safety valve rather than the recovery: `surb_decay` subtracts from the level
    /// estimate on a timer regardless of what any packet says, so once the estimate falls back under
    /// target both this gate and the keep-alives reopen on their own.
    ///
    /// How long that takes is a property of the configuration, not a guarantee of this type. It
    /// scales with the decay rate and with how far above target the estimate sits, and the
    /// `max(capacity, target)` clamp bounds the latter only while `counterparty_buffer_capacity` is
    /// known -- its `0` ("unknown") arm leaves the estimate unclamped. With decay switched off the
    /// estimate does not drain on its own at all, and only a fresh distress signal or observed
    /// consumption reopens the gate.
    ///
    /// Sequencing the flag would buy a faster reopen, at the cost of making a hot-path signal
    /// depend on the Session's reassembly.
    pub counterparty_in_surb_distress: AtomicBool,
}

impl BalancerStateValues {
    /// Constructor from a [`SurbBalancerConfig`].
    pub fn new(cfg: SurbBalancerConfig) -> Self {
        let state = Self::default();
        state.update(&cfg);
        state
    }

    /// Performs update of the [`BalancerStateValues`] from the [`SurbBalancerConfig`] and
    /// enables it.
    pub fn update(&self, cfg: &SurbBalancerConfig) {
        self.target_surb_buffer_size
            .store(cfg.target_surb_buffer_size, std::sync::atomic::Ordering::Relaxed);
        self.max_surbs_per_sec
            .store(cfg.max_surbs_per_sec, std::sync::atomic::Ordering::Relaxed);
        self.decay_duration_msec.store(
            cfg.surb_decay
                .map(|(d, _)| d.as_millis().min(u64::MAX as u128) as u64)
                .unwrap_or_default(),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.decay_volume_pct.store(
            cfg.surb_decay
                .map(|(_, p)| (p.clamp(0.0, 1.0) * 100.0).round() as u8)
                .unwrap_or_default(),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.sustain_on_return_path_loss
            .store(cfg.sustain_on_return_path_loss, std::sync::atomic::Ordering::Relaxed);
    }

    /// Declares how many SURBs the counterparty's store can hold, bounding the level estimate.
    ///
    /// Taken from the session manager's `maximum_surb_buffer_size`, which is the same capacity
    /// already used to clamp a counterparty's requested target. Zero leaves the estimate unbounded.
    pub fn set_counterparty_buffer_capacity(&self, capacity: u64) {
        self.counterparty_buffer_capacity
            .store(capacity, std::sync::atomic::Ordering::Relaxed);
    }

    /// Caps `level` at what the counterparty can actually hold.
    ///
    /// Never below the configured target: a target above the counterparty's capacity is
    /// unreachable by construction, and clamping to capacity there would hold the error permanently
    /// negative and pin production at maximum forever -- a worse failure than the unbounded
    /// estimate this exists to prevent. In that configuration the capacity figure is simply not
    /// usable for this session.
    fn clamp_to_counterparty_capacity(&self, level: u64) -> u64 {
        match self
            .counterparty_buffer_capacity
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            0 => level,
            capacity => {
                level.min(capacity.max(self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed)))
            }
        }
    }

    /// Records what the counterparty's latest packet said about *its own* SURB supply.
    ///
    /// Takes the whole signal set rather than a `bool` so the containment rule lives here: `OutOfSurbs`
    /// is a superset of `SurbDistress` on the wire, so `contains` catches both, whereas an equality
    /// match against `SurbDistress` would silently ignore the more severe of the two.
    pub fn observe_counterparty_signals(&self, signals: PacketSignals) {
        self.counterparty_in_surb_distress.store(
            signals.contains(PacketSignal::SurbDistress),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    /// Returns the number of organic SURBs to attach to an outgoing Session data packet.
    ///
    /// Only the Entry uses this value. Return `0` when the estimated counterparty buffer
    /// has reached its target. Return `1` when balancing is disabled, the estimate is
    /// below target, or the counterparty signals SURB distress.
    ///
    /// This method uses the raw buffer estimate. During return-path loss, a zero estimate
    /// correctly keeps organic SURB production enabled.
    pub fn organic_surbs_per_packet(&self) -> usize {
        // Not defensive boilerplate: nothing rejects a `SurbBalancerConfig` with a zero target, and
        // without this branch `level >= 0` would hold forever and shut organic production off
        // permanently — while a PID with a zero output limit produces nothing either.
        usize::from(
            self.is_disabled()
                || self
                    .counterparty_in_surb_distress
                    .load(std::sync::atomic::Ordering::Relaxed)
                || self.buffer_level() < self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
        )
    }

    /// Marks the return path as degraded for the next `grace` period.
    ///
    /// Called by whichever layer can actually tell a dead return path from a quiet peer -- from
    /// here the two are indistinguishable, since neither delivers replies. Re-marking simply
    /// extends the window.
    pub fn mark_return_path_degraded(&self, grace: Duration) {
        self.mark_return_path_degraded_at(Instant::now(), grace);
    }

    /// [`mark_return_path_degraded`](Self::mark_return_path_degraded) as of `now`, so that the
    /// balancer's own clock can drive the window in tests.
    pub(crate) fn mark_return_path_degraded_at(&self, now: Instant, grace: Duration) {
        let until = now
            .saturating_duration_since(*EPOCH)
            .saturating_add(grace)
            .as_millis()
            .min(u64::MAX as u128) as u64;
        self.return_path_degraded_until_ms
            .fetch_max(until, std::sync::atomic::Ordering::Relaxed);
    }

    /// Whether [`buffer_level`](Self::buffer_level) is currently an instruction rather than a
    /// measurement.
    ///
    /// While this holds, the controller deliberately writes `0` into the level to keep production
    /// up whatever the estimate says. That zero says "keep producing", not "the counterparty holds
    /// nothing", so anything reading the level as a *supply ceiling* must consult this first or it
    /// will read the instruction as an order to send nothing.
    ///
    /// True only when both the opt-in (`sustain_on_return_path_loss`) and live evidence
    /// ([`mark_return_path_degraded`](Self::mark_return_path_degraded), within its window) are
    /// present: without the opt-in this is not our behaviour to change, and without evidence there
    /// is nothing to tell a dead return path from an idle one.
    ///
    /// `pub` because it is not only the controller's business — hence the emphasis above on what
    /// the flag does *not* mean. It is not a general "the return path is degraded" signal.
    pub fn return_path_estimate_is_stale(&self) -> bool {
        self.should_sustain_through_return_path_loss_at(Instant::now())
    }

    fn should_sustain_through_return_path_loss_at(&self, now: Instant) -> bool {
        let deadline = self
            .return_path_degraded_until_ms
            .load(std::sync::atomic::Ordering::Relaxed);

        // Zero is "never marked", not "marked at the epoch" -- otherwise every session that opted
        // in would start out believing its return path was already dead.
        deadline > 0
            && (now.saturating_duration_since(*EPOCH).as_millis() as u64) < deadline
            && self
                .sustain_on_return_path_loss
                .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Extracts the [`SurbBalancerConfig`] from the [`BalancerStateValues`].
    pub fn as_config(&self) -> SurbBalancerConfig {
        SurbBalancerConfig {
            target_surb_buffer_size: self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
            max_surbs_per_sec: self.max_surbs_per_sec.load(std::sync::atomic::Ordering::Relaxed),
            surb_decay: self.surb_decay(),
            sustain_on_return_path_loss: self
                .sustain_on_return_path_loss
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Checks if SURB balancing is disabled (no target buffer size set).
    pub fn is_disabled(&self) -> bool {
        self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed) == 0
    }

    /// Extracts the SURB decay configuration from the [`BalancerStateValues`].
    pub fn surb_decay(&self) -> Option<(Duration, f64)> {
        Some((
            self.decay_duration_msec.load(std::sync::atomic::Ordering::Relaxed),
            self.decay_volume_pct.load(std::sync::atomic::Ordering::Relaxed),
        ))
        .filter(|&(d, p)| d > 0 && p > 0)
        .map(|(d, p)| (Duration::from_millis(d), p as f64 / 100.0))
    }

    /// Gets the current estimated SURB buffer level.
    #[inline]
    pub fn buffer_level(&self) -> u64 {
        self.buffer_level.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the current `BalancerControllerBounds` from the [`BalancerStateValues`].
    #[inline]
    pub fn controller_bounds(&self) -> BalancerControllerBounds {
        BalancerControllerBounds::new(
            self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
            self.max_surbs_per_sec.load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}

impl From<SurbBalancerConfig> for BalancerStateValues {
    fn from(cfg: SurbBalancerConfig) -> Self {
        Self::new(cfg)
    }
}

/// Runs a continuous process that attempts to [evaluate](SurbFlowEstimator) and
/// [regulate](SurbFlowController) the flow of SURBs to the Session counterparty,
/// to keep the number of SURBs locally or at the counterparty at a certain level.
///
/// Internally, the Balancer uses an implementation of [`SurbBalancerController`] to
/// control the rate of SURBs consumed or sent to the counterparty
/// each time the [`update`](SurbBalancer::update) method is called:
///
/// 1. The size of the SURB buffer at locally or at the counterparty is estimated using [`SurbFlowEstimator`].
/// 2. Error against a set-point given in [`SurbBalancerConfig`] is evaluated in the `SurbBalancerController`.
/// 3. The `SurbBalancerController` applies a new SURB flow rate value using the [`SurbFlowController`].
///
/// In the local context, the `SurbFlowController` might simply regulate the egress traffic from the
/// Session, slowing it down to avoid fast SURB drainage.
///
/// In the remote context, the `SurbFlowController` might regulate the flow of non-organic SURBs via
/// Start protocol's `KeepAlive` messages to deliver additional
/// SURBs to the counterparty.
pub struct SurbBalancer<C, E, F> {
    session_id: SessionId,
    controller: C,
    surb_estimator: E,
    flow_control: F,
    state: Arc<BalancerStateValues>,
    last_estimator_state: SimpleSurbFlowEstimator,
    last_update: std::time::Instant,
    last_decay: std::time::Instant,
    was_below_target: bool,
    /// Whether the previous update ran in open loop, so both edges can be acted on.
    was_degraded: bool,
    /// Smoothed consumption the buffer loses beyond organic supply, in SURBs per second.
    ///
    /// Held, not updated, while the return path is degraded: consumption is observed through the
    /// replies that reach us, so what it reads then is how many replies were lost, not how many
    /// SURBs the counterparty spent.
    net_consumption_per_sec: f64,
    /// DIAGNOSTIC: when the last balancer-state line was emitted, to rate-limit it.
    last_report: std::time::Instant,
    /// DIAGNOSTIC: estimator state at the last balancer-state line, to report rates over its window.
    last_report_snapshot: SimpleSurbFlowEstimator,
}

impl<C, E, F> SurbBalancer<C, E, F>
where
    C: SurbBalancerController + Send + Sync + 'static,
    E: SurbFlowEstimator + Send + Sync + 'static,
    F: SurbFlowController + Send + Sync + 'static,
{
    pub fn new(
        session_id: SessionId,
        mut controller: C,
        surb_estimator: E,
        flow_control: F,
        state: Arc<BalancerStateValues>,
    ) -> Self {
        #[cfg(all(feature = "telemetry", not(test)))]
        {
            let sid: &str = session_id.as_ref();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[sid], 0.0);
            METRIC_CONTROL_OUTPUT.set(&[sid], 0.0);
        }

        controller.set_target_and_limit(state.controller_bounds());
        // Rates are reported from now on: whatever was produced before the balancer started (e.g.
        // while pre-loading SURBs) is not part of the first window.
        let last_report_snapshot = SimpleSurbFlowEstimator::from(&surb_estimator);

        Self {
            surb_estimator,
            flow_control,
            controller,
            session_id,
            state,
            last_estimator_state: Default::default(),
            last_update: std::time::Instant::now(),
            last_decay: std::time::Instant::now(),
            was_below_target: true,
            was_degraded: false,
            net_consumption_per_sec: 0.0,
            last_report: std::time::Instant::now(),
            last_report_snapshot,
        }
    }

    /// Computes the next control update and adjusts the [`SurbFlowController`] rate accordingly.
    fn update(&mut self) -> u64 {
        self.update_at(Instant::now())
    }

    /// Folds one sample into the smoothed net consumption that the controller feeds forward.
    fn observe_net_consumption(&mut self, rates: SurbFlowRates, dt: Duration) {
        let sample = rates.consumed - rates.organic;
        let weight = 1.0 - (-dt.as_secs_f64() / CONSUMPTION_SMOOTHING.as_secs_f64()).exp();
        self.net_consumption_per_sec += weight * (sample - self.net_consumption_per_sec);
    }

    /// [`update`](Self::update) as of `now`, so that tests can drive the loop on a synthetic clock.
    #[tracing::instrument(level = "trace", skip_all)]
    fn update_at(&mut self, now: Instant) -> u64 {
        let dt = now.saturating_duration_since(self.last_update);

        // Load the updated current buffer level
        let mut current = self.state.buffer_level.load(std::sync::atomic::Ordering::Acquire);

        if dt < Duration::from_millis(10) {
            tracing::debug!("time elapsed since last update is too short, skipping update");
            return current;
        }

        self.last_update = now;

        // Take a snapshot of the active SURB estimator and calculate the balance change
        let snapshot = SimpleSurbFlowEstimator::from(&self.surb_estimator);
        let Some(target_buffer_change) = snapshot.estimated_surb_buffer_change(&self.last_estimator_state) else {
            tracing::error!("non-monotonic change in SURB estimators");
            return current;
        };

        let sample_rates = SurbFlowRates::between(&self.last_estimator_state, &snapshot, dt);
        self.last_estimator_state = snapshot;
        current = current.saturating_add_signed(target_buffer_change);

        // If SURB decaying is enabled, check if the decay window has elapsed
        // and calculate the number of SURBs that will be discarded
        if let Some(num_decayed_surbs) = self
            .state
            .surb_decay()
            .filter(|(decay_window, _)| &now.saturating_duration_since(self.last_decay) >= decay_window)
            .map(|(_, decay_coeff)| (self.controller.bounds().target() as f64 * decay_coeff).round() as u64)
        {
            current = current.saturating_sub(num_decayed_surbs);
            self.last_decay = now;
            tracing::trace!(num_decayed_surbs, "SURBs were discarded due to automatic decay");
        }

        // Believing a level the counterparty cannot hold keeps production throttled long after
        // the surplus was evicted on arrival, so the estimate is bounded by the store it describes.
        let believed = current;
        current = self.state.clamp_to_counterparty_capacity(current);
        if current != believed {
            // Not the configured capacity: the bound applied is `max(capacity, target)`, so name
            // the clamped level and the capacity separately rather than conflating them.
            tracing::debug!(
                believed,
                clamped_to = current,
                counterparty_capacity = self
                    .state
                    .counterparty_buffer_capacity
                    .load(std::sync::atomic::Ordering::Relaxed),
                "counterparty SURB estimate exceeded its store; the surplus was never held"
            );
        }

        let degraded = self.state.should_sustain_through_return_path_loss_at(now);
        if !degraded {
            self.observe_net_consumption(sample_rates, dt);
        }

        if degraded != self.was_degraded {
            // The estimate stops meaning what it meant on both edges: entering, it is inflated by
            // production nobody was seen to consume; leaving, it is a level that was never
            // observed. Either way the accumulated error belongs to a regime that has ended.
            self.controller.reset();
            self.was_degraded = degraded;

            if !degraded {
                // Coming back, treat the counterparty as freshly started rather than as whatever
                // the outage left behind. It really did drain while replies were lost, and this is
                // the estimate that self-corrects: consumption is observable again, so the buffer
                // level climbs on its own as production outruns it.
                current = 0;
                self.last_decay = now;
                tracing::debug!("return path recovered; restarting closed-loop SURB control");
            }
        }

        if degraded {
            // While replies are being lost there is no valid estimate to act on: every SURB the
            // counterparty spends is invisible from here, so the accumulated `produced - consumed`
            // reads as a filling buffer precisely when it is emptying. Drop to open loop and assume
            // the worst: an empty buffer, refilled on top of the last consumption seen while
            // replies still arrived.
            tracing::debug!(
                believed = current,
                "return path degraded; ignoring the counterparty buffer estimate"
            );
            // Reads as "keep producing" to the controller below. `SurbSupply` must not read it
            // as "the buffer is empty, admit nothing" -- see `return_path_estimate_is_stale`.
            current = 0;
        }

        self.state
            .buffer_level
            .store(current, std::sync::atomic::Ordering::Release);

        // Error from the desired target SURB buffer size at counterparty
        let error = current as i64 - self.controller.bounds().target() as i64;

        if self.was_below_target && error >= 0 {
            tracing::trace!(current, "reached target SURB buffer size");
            self.was_below_target = false;
        } else if !self.was_below_target && error < 0 {
            tracing::trace!(current, "SURB buffer size is below target");
            self.was_below_target = true;
        }

        tracing::trace!(
            ?dt,
            delta = target_buffer_change,
            rate = target_buffer_change as f64 / dt.as_secs_f64(),
            current,
            error,
            "estimated SURB buffer change"
        );

        let output = self.controller.next_control_output(ControlInput {
            level: current,
            net_consumption_per_sec: self.net_consumption_per_sec,
            dt,
            ceiling: self.controller.bounds().output_limit(),
        });
        tracing::trace!(output, "next balancer control output for session");

        // Both ends run this same loop -- the Entry with paced refill driving production, the Exit
        // with the proportional controller gating egress -- so one line covers both and the session
        // id tells them apart. Rate-limited to one per second so it can run under a full-rate session.
        //
        // At `debug` rather than `info`: one line per session per second is fine for a handful of
        // sessions and is a lot of formatting work for a node carrying many, none of which an
        // operator needs to see during healthy operation.
        //
        // The rates split production by what it costs: `organic_surbs_s` rode along with Session
        // data for free, `ka_*` is what keep-alives spent on the wire. Without that split the
        // upstream the balancer spends is invisible, since keep-alives carry no application bytes.
        let report_window = now.saturating_duration_since(self.last_report);
        if report_window >= Duration::from_secs(1) {
            self.last_report = now;
            let rates = SurbFlowRates::between(&self.last_report_snapshot, &snapshot, report_window);
            self.last_report_snapshot = snapshot;

            tracing::debug!(
                session = %self.session_id,
                level = current,
                target = self.controller.bounds().target(),
                output,
                produced = snapshot.produced,
                consumed = snapshot.consumed,
                consumed_s = rates.consumed.round() as u64,
                net_consumption_s = self.net_consumption_per_sec.round() as i64,
                organic_surbs_s = rates.organic.round() as u64,
                ka_surbs_s = rates.keep_alive_surbs.round() as u64,
                ka_pkts_s = rates.keep_alive_packets.round() as u64,
                ka_bytes_s = rates.keep_alive_wire_bytes().round() as u64,
                surbs_per_ka_pkt = (rates.surbs_per_keep_alive() * 100.0).round() / 100.0,
                degraded,
                distress = self
                    .state
                    .counterparty_in_surb_distress
                    .load(std::sync::atomic::Ordering::Relaxed),
                "surb balancer state"
            );

            #[cfg(all(feature = "telemetry", not(test)))]
            {
                let sid: &str = self.session_id.as_ref();
                METRIC_CONSUMED_RATE.set(&[sid], rates.consumed);
                METRIC_ORGANIC_RATE.set(&[sid], rates.organic);
                METRIC_KEEP_ALIVE_PACKET_RATE.set(&[sid], rates.keep_alive_packets);
                METRIC_KEEP_ALIVE_WIRE_RATE.set(&[sid], rates.keep_alive_wire_bytes());
            }
        }

        self.flow_control.adjust_surb_flow(output as usize);

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            let sid: &str = self.session_id.as_ref();
            METRIC_CURRENT_BUFFER.set(&[sid], current as f64);
            METRIC_CURRENT_TARGET.set(&[sid], self.controller.bounds().target() as f64);
            METRIC_TARGET_ERROR_ESTIMATE.set(&[sid], error as f64);
            METRIC_CONTROL_OUTPUT.set(&[sid], output as f64);
            METRIC_SURB_RATE.set(&[sid], target_buffer_change as f64 / dt.as_secs_f64());
        }

        current
    }

    /// Spawns a new task that performs updates of the given [`SurbBalancer`] at the given `sampling_interval`.
    ///
    /// If `cfg_feedback` is given, [`SurbBalancerConfig`] can be queried for updates and also updated
    /// if the underlying [`SurbBalancerController`] also does target updates.
    ///
    /// Returns a stream of current estimated buffer levels, and also an `AbortHandle`
    /// to terminate the loop. If `abort_reg` was given, the returned `AbortHandle` corresponds
    /// to it.
    #[instrument(level = "debug", skip(self), fields(session_id = %self.session_id))]
    pub fn start_control_loop(
        mut self,
        sampling_interval: Duration,
    ) -> (impl futures::Stream<Item = u64>, AbortHandle) {
        let (abort_handle, abort_reg) = AbortHandle::new_pair();

        // Start an interval stream at which the balancer will sample and perform updates

        // DropAbortable not needed because the stream only generates items when polled
        let sampling_stream = futures::stream::Abortable::new(
            futures_time::stream::interval(sampling_interval.max(MIN_BALANCER_SAMPLING_INTERVAL).into()),
            abort_reg,
        );

        let balancer_level_capacity = std::env::var("HOPR_INTERNAL_SESSION_BALANCER_LEVEL_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(32_768);

        tracing::debug!(
            capacity = balancer_level_capacity,
            "Creating session balancer level channel"
        );
        let (mut level_tx, level_rx) = futures::channel::mpsc::channel(balancer_level_capacity);
        hopr_utils::runtime::prelude::spawn(
            async move {
                pin_mut!(sampling_stream);
                while sampling_stream.next().await.is_some() {
                    // Check if the balancer controller needs to be reconfigured
                    let current_bounds = self.state.controller_bounds();
                    if current_bounds != self.controller.bounds() {
                        self.controller.set_target_and_limit(current_bounds);
                        tracing::debug!(new_cfg = ?self.state.as_config(), "surb balancer has been reconfigured");
                    }

                    // Perform controller update (this internally samples the SurbFlowEstimator)
                    // and send an update about the current level to the outgoing stream.
                    // If the other party has closed the stream, we don't care about the update.
                    let level = self.update();
                    if !level_tx.is_closed()
                        && let Err(error) = level_tx.try_send(level)
                    {
                        tracing::error!(%error, "cannot send balancer level update");
                    }
                }

                tracing::debug!("balancer done");
            }
            .in_current_span(),
        );

        (level_rx, abort_handle)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::AtomicU64};

    use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};

    use super::*;
    use crate::balancer::{
        AtomicSurbFlowEstimator, MockSurbFlowController,
        paced::{DEFAULT_RAMP_TIME, DEFAULT_REFILL_HORIZON, PacedRefillController, PacedRefillParams},
        pid::PidBalancerController,
    };

    #[test]
    fn surb_balancer_config_should_be_convertible_to_atomics() {
        let cfg = SurbBalancerConfig::default();
        let state_data = BalancerStateValues::new(cfg);
        assert_eq!(cfg, state_data.as_config());
    }

    #[test]
    fn flow_rates_should_split_production_into_organic_and_keep_alive() {
        let later = SimpleSurbFlowEstimator {
            produced: 3_000,
            consumed: 1_000,
            keep_alive_surbs: 2_000,
            keep_alive_packets: 1_000,
        };
        let rates = SurbFlowRates::between(&SimpleSurbFlowEstimator::default(), &later, Duration::from_secs(2));

        assert_eq!(
            SurbFlowRates {
                consumed: 500.0,
                organic: 500.0,
                keep_alive_surbs: 1_000.0,
                keep_alive_packets: 500.0,
            },
            rates
        );
        assert_eq!(
            500.0 * hopr_crypto_packet::prelude::HoprPacket::SIZE as f64,
            rates.keep_alive_wire_bytes()
        );
        assert_eq!(2.0, rates.surbs_per_keep_alive());
    }

    /// The keep-alive attribution is recorded just before the production it belongs to, so a
    /// snapshot can land in between. That must not read as negative -- or wrapped -- organic flow.
    #[test]
    fn flow_rates_should_not_invent_organic_surbs_from_an_attribution_seen_early() {
        let later = SimpleSurbFlowEstimator {
            keep_alive_surbs: 2,
            keep_alive_packets: 1,
            ..Default::default()
        };
        let rates = SurbFlowRates::between(&SimpleSurbFlowEstimator::default(), &later, Duration::from_secs(1));
        assert_eq!(0.0, rates.organic);
    }

    #[test]
    fn flow_rates_over_an_empty_window_should_be_zero() {
        let later = SimpleSurbFlowEstimator {
            produced: 10,
            ..Default::default()
        };
        let rates = SurbFlowRates::between(&SimpleSurbFlowEstimator::default(), &later, Duration::ZERO);
        assert_eq!(SurbFlowRates::default(), rates);
        assert_eq!(0.0, rates.surbs_per_keep_alive());
    }

    #[test]
    fn surb_balancer_config_default_snapshot() {
        let cfg = SurbBalancerConfig::default();
        insta::assert_debug_snapshot!(cfg);
    }

    #[test]
    fn surb_balancer_config_as_controller_bounds() {
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 1000,
            max_surbs_per_sec: 500,
            surb_decay: None,
            sustain_on_return_path_loss: false,
        };
        let bounds = cfg.as_controller_bounds();
        assert_eq!(bounds.target(), 1000);
        assert_eq!(bounds.output_limit(), 500);
    }

    #[test]
    fn balancer_state_values_disabled_when_target_is_zero() {
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 0,
            max_surbs_per_sec: 0,
            surb_decay: None,
            sustain_on_return_path_loss: false,
        };
        let state = BalancerStateValues::new(cfg);
        assert!(state.is_disabled());
    }

    #[test]
    fn balancer_state_values_enabled_when_target_is_nonzero() {
        let state = BalancerStateValues::new(SurbBalancerConfig::default());
        assert!(!state.is_disabled());
    }

    #[test]
    fn balancer_state_values_update_propagates_all_fields() {
        let state = BalancerStateValues::default();
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 3000,
            max_surbs_per_sec: 1500,
            surb_decay: Some((Duration::from_secs(30), 0.10)),
            sustain_on_return_path_loss: false,
        };
        state.update(&cfg);
        assert_eq!(state.as_config(), cfg);
        assert_eq!(state.controller_bounds(), cfg.as_controller_bounds());
    }

    #[test]
    fn balancer_state_values_surb_decay_none_maps_to_none() {
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 1000,
            max_surbs_per_sec: 500,
            surb_decay: None,
            sustain_on_return_path_loss: false,
        };
        let state = BalancerStateValues::new(cfg);
        assert!(state.surb_decay().is_none());
    }

    #[test]
    fn balancer_state_values_buffer_level_default_is_zero() {
        let state = BalancerStateValues::default();
        assert_eq!(state.buffer_level(), 0);
    }

    #[test]
    fn balancer_state_values_buffer_level_can_be_updated() {
        let state = BalancerStateValues::default();
        state.buffer_level.store(42, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(state.buffer_level(), 42);
    }

    /// State with a given target sitting at a given level, for the organic-gate tests below.
    fn gate_state(target: u64, buffer_level: u64) -> BalancerStateValues {
        let state = BalancerStateValues::new(SurbBalancerConfig {
            target_surb_buffer_size: target,
            max_surbs_per_sec: 5_000,
            ..Default::default()
        });
        state
            .buffer_level
            .store(buffer_level, std::sync::atomic::Ordering::Relaxed);
        state
    }

    #[test]
    fn organic_surbs_should_be_produced_while_the_counterparty_is_below_target() {
        assert_eq!(1, gate_state(100, 99).organic_surbs_per_packet());
        assert_eq!(1, gate_state(100, 0).organic_surbs_per_packet());
    }

    /// The `>=` boundary: at target is already too many, because the next SURB is the one that
    /// evicts. Pinned at exactly the target and well past it.
    #[test]
    fn organic_surbs_should_stop_once_the_counterparty_reaches_its_target() {
        assert_eq!(0, gate_state(100, 100).organic_surbs_per_packet());
        assert_eq!(0, gate_state(100, 200).organic_surbs_per_packet());
    }

    /// The safety valve. Our level estimate counts SURBs as delivered when they are *sent*, so
    /// forward-path loss inflates it — the counterparty's own word about its supply has to win over
    /// an estimate that can be wrong in exactly that direction.
    #[test]
    fn organic_surbs_should_resume_at_one_when_the_counterparty_signals_distress() {
        let state = gate_state(100, 200);
        assert_eq!(0, state.organic_surbs_per_packet(), "precondition: gate is closed");

        state.observe_counterparty_signals(PacketSignal::SurbDistress.into());
        assert_eq!(1, state.organic_surbs_per_packet());
    }

    /// `OutOfSurbs` is `0b11` and `SurbDistress` is `0b01`, so the former *contains* the latter.
    /// Matching on equality instead of containment would ignore the more severe of the two signals
    /// and leave production shut off for a counterparty that has nothing left to reply with.
    #[test]
    fn out_of_surbs_should_count_as_distress() {
        let state = gate_state(100, 200);
        state.observe_counterparty_signals(PacketSignal::OutOfSurbs.into());
        assert_eq!(1, state.organic_surbs_per_packet());
    }

    /// Distress is not sticky once the counterparty recovers: it recomputes both signals per return
    /// packet and strips them when its pool is healthy, so the next such packet re-arms the gate.
    #[test]
    fn a_recovered_counterparty_should_clear_distress() {
        let state = gate_state(100, 200);
        state.observe_counterparty_signals(PacketSignal::OutOfSurbs.into());
        assert_eq!(1, state.organic_surbs_per_packet(), "precondition: distress is set");

        state.observe_counterparty_signals(PacketSignals::default());
        assert_eq!(0, state.organic_surbs_per_packet());
    }

    /// A zero target means "no balancing", not "the target is already met". Nothing rejects such a
    /// config, so without the explicit branch `level >= 0` would hold forever and starve the session
    /// of organic SURBs permanently — while a PID with a zero output limit produces none either.
    #[test]
    fn a_disabled_balancer_should_keep_producing_organic_surbs() {
        let state = gate_state(0, 0);
        assert!(state.is_disabled(), "precondition: a zero target disables balancing");
        assert_eq!(1, state.organic_surbs_per_packet());
    }

    /// Sessions opened without SURB management hold a default state and route their outgoing packets
    /// through the same policy as balanced ones, relying on it to answer 1. A `Default` that ever
    /// gained a non-zero target would silently stop those sessions producing organic SURBs, with
    /// nothing at the call site to show why.
    #[test]
    fn a_default_state_should_keep_producing_organic_surbs() {
        assert_eq!(1, BalancerStateValues::default().organic_surbs_per_packet());
    }

    /// Mirror image of `a_degraded_return_path_should_not_zero_the_supply_ceiling` in `flow_control`:
    /// there, the degraded-path `0` must be suppressed because it would read as "admit no bytes";
    /// here it must be honoured, because it reads as "below target, keep producing" — which is
    /// exactly what opting into `sustain_on_return_path_loss` asks for. Same stored value, opposite
    /// polarity, so this must *not* grow the guard its counterpart needs.
    #[test]
    fn a_degraded_return_path_should_not_stop_organic_surb_production() {
        let state = BalancerStateValues::new(SurbBalancerConfig {
            target_surb_buffer_size: 100,
            sustain_on_return_path_loss: true,
            ..Default::default()
        });
        state.mark_return_path_degraded(Duration::from_secs(30));
        // What the control loop writes while it drives production flat out.
        state.buffer_level.store(0, std::sync::atomic::Ordering::Relaxed);

        assert!(state.return_path_estimate_is_stale(), "precondition: open loop");
        assert_eq!(1, state.organic_surbs_per_packet());
    }

    #[test]
    fn balancer_state_values_from_config() {
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 5000,
            max_surbs_per_sec: 2500,
            surb_decay: Some((Duration::from_secs(60), 0.05)),
            sustain_on_return_path_loss: false,
        };
        let state: BalancerStateValues = cfg.into();
        assert_eq!(state.as_config(), cfg);
    }

    #[test]
    fn balancer_state_values_decay_zero_duration_should_map_to_none() {
        let cfg = SurbBalancerConfig {
            surb_decay: Some((Duration::ZERO, 0.10)),
            ..Default::default()
        };
        let state = BalancerStateValues::new(cfg);
        assert!(
            state.surb_decay().is_none(),
            "zero duration decay should be filtered out"
        );
    }

    #[test]
    fn balancer_state_values_decay_zero_percent_should_map_to_none() {
        let cfg = SurbBalancerConfig {
            surb_decay: Some((Duration::from_secs(60), 0.0)),
            ..Default::default()
        };
        let state = BalancerStateValues::new(cfg);
        assert!(
            state.surb_decay().is_none(),
            "zero percent decay should be filtered out"
        );
    }

    #[test]
    fn balancer_state_values_decay_should_clamp_above_one() {
        let cfg = SurbBalancerConfig {
            surb_decay: Some((Duration::from_secs(1), 1.5)), // > 1.0 should be clamped
            ..Default::default()
        };
        let state = BalancerStateValues::new(cfg);
        let (_, pct) = state.surb_decay().expect("decay should be present");
        assert!((pct - 1.0).abs() < f64::EPSILON, "percentage should be clamped to 1.0");
    }

    #[test_log::test]
    fn surb_balancer_should_start_increase_level_when_below_target() {
        let production_rate = Arc::new(AtomicU64::new(0));
        let consumption_rate = 100;
        let steps = 3;
        let step_duration = std::time::Duration::from_millis(1000);

        let mut controller = MockSurbFlowController::new();
        let production_rate_clone = production_rate.clone();
        controller
            .expect_adjust_surb_flow()
            .times(steps)
            .with(mockall::predicate::ge(100))
            .returning(move |r| {
                production_rate_clone.store(r as u64, std::sync::atomic::Ordering::Relaxed);
            });

        let surb_estimator = AtomicSurbFlowEstimator::default();
        let mut balancer = SurbBalancer::new(
            HoprPseudonym::random(),
            PidBalancerController::default(),
            surb_estimator.clone(),
            controller,
            Arc::new(
                SurbBalancerConfig {
                    target_surb_buffer_size: 5_000,
                    max_surbs_per_sec: 2500,
                    surb_decay: None,
                    sustain_on_return_path_loss: false,
                }
                .into(),
            ),
        );

        let start = Instant::now();
        let mut last_update = 0;
        for i in 0..steps {
            surb_estimator.produced.fetch_add(
                production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );
            surb_estimator.consumed.fetch_add(
                consumption_rate * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );

            let next_update = balancer.update_at(start + step_duration * (i as u32 + 1));
            assert!(
                i == 0 || next_update > last_update,
                "{next_update} should be greater than {last_update}"
            );
            last_update = next_update;
        }
    }

    #[test_log::test]
    fn surb_balancer_should_start_decrease_level_when_above_target() {
        let production_rate = Arc::new(AtomicU64::new(11_000));
        let consumption_rate = 100;
        let steps = 3;
        let step_duration = std::time::Duration::from_millis(1000);

        let mut controller = MockSurbFlowController::new();
        let production_rate_clone = production_rate.clone();
        controller
            .expect_adjust_surb_flow()
            .times(steps)
            .with(mockall::predicate::ge(0))
            .returning(move |r| {
                production_rate_clone.store(r as u64, std::sync::atomic::Ordering::Relaxed);
            });

        let surb_estimator = AtomicSurbFlowEstimator::default();
        let mut balancer = SurbBalancer::new(
            HoprPseudonym::random(),
            PidBalancerController::default(),
            surb_estimator.clone(),
            controller,
            Arc::new(
                SurbBalancerConfig {
                    surb_decay: None,
                    ..Default::default()
                }
                .into(),
            ),
        );

        let start = Instant::now();
        let mut last_update = 0;
        for i in 0..steps {
            surb_estimator.produced.fetch_add(
                production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );
            surb_estimator.consumed.fetch_add(
                consumption_rate * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );

            let next_update = balancer.update_at(start + step_duration * (i as u32 + 1));
            assert!(
                i == 0 || next_update < last_update,
                "{next_update} should be greater than {last_update}"
            );
            last_update = next_update;
        }
    }

    /// One sampling interval of the closed-loop harness.
    const TICK: Duration = Duration::from_millis(50);

    /// Long enough for a paced refill to reach the setpoint band from empty: about
    /// `T_ramp + H·ln 10` ≈ 6.6 s at the defaults.
    const HEALTHY_TICKS: usize = 200;

    /// SURBs the counterparty spends per interval while it is answering normally (800 SURB/s).
    const REPLIES_PER_TICK: u64 = 40;

    /// A balancer whose production follows its own control output, as it does in a live Session,
    /// on a synthetic clock so that each tick is exactly one sampling interval.
    ///
    /// Production must be fed back rather than held constant: with production pinned to
    /// consumption the buffer never fills, maximum output is the correct answer, and every phase of
    /// the test reads the same. Minted SURBs are attributed to keep-alives, as the live Entry does,
    /// so that the balancer does not mistake its own production for organic supply.
    struct Harness {
        balancer: SurbBalancer<PacedRefillController, AtomicSurbFlowEstimator, MockSurbFlowController>,
        estimator: AtomicSurbFlowEstimator,
        state: Arc<BalancerStateValues>,
        output: Arc<AtomicU64>,
        now: Instant,
    }

    impl Harness {
        fn new(cfg: SurbBalancerConfig) -> Self {
            let output = Arc::new(AtomicU64::new(0));
            let output_clone = output.clone();
            let mut flow_control = MockSurbFlowController::new();
            flow_control.expect_adjust_surb_flow().returning(move |r| {
                output_clone.store(r as u64, std::sync::atomic::Ordering::Relaxed);
            });

            let estimator = AtomicSurbFlowEstimator::default();
            let state: Arc<BalancerStateValues> = Arc::new(cfg.into());
            let balancer = SurbBalancer::new(
                HoprPseudonym::random(),
                PacedRefillController::new(PacedRefillParams::default()),
                estimator.clone(),
                flow_control,
                state.clone(),
            );

            // The degraded-path deadline is kept relative to `EPOCH`, so it has to exist before the
            // clock this harness starts from.
            let _ = *EPOCH;
            Self {
                balancer,
                estimator,
                state,
                output,
                now: Instant::now(),
            }
        }

        /// One sampling interval: mint at the rate last commanded, and consume `consumed` of them.
        fn tick(&mut self, consumed: u64) {
            self.now += TICK;
            let minted = self.output() * TICK.as_millis() as u64 / 1000;
            self.estimator
                .produced
                .fetch_add(minted, std::sync::atomic::Ordering::Relaxed);
            self.estimator
                .keep_alive_surbs
                .fetch_add(minted, std::sync::atomic::Ordering::Relaxed);
            self.estimator
                .consumed
                .fetch_add(consumed, std::sync::atomic::Ordering::Relaxed);
            self.balancer.update_at(self.now);
        }

        fn run(&mut self, ticks: usize, consumed: u64) {
            for _ in 0..ticks {
                self.tick(consumed);
            }
        }

        fn output(&self) -> u64 {
            self.output.load(std::sync::atomic::Ordering::Relaxed)
        }

        fn level(&self) -> u64 {
            self.state.buffer_level.load(std::sync::atomic::Ordering::Relaxed)
        }

        fn mark_degraded(&self, grace: Duration) {
            self.state.mark_return_path_degraded_at(self.now, grace);
        }
    }

    /// Drives a balancer through a healthy stretch, then through one where no reply comes back.
    ///
    /// Returns the control output at the end of each stretch. The two phases are deliberately
    /// indistinguishable from inside the balancer -- consumption simply stops -- which is the whole
    /// point: only the caller's `sustain` choice separates a dead return path from an idle peer.
    fn drive_until_replies_stop(cfg: SurbBalancerConfig, mark_degraded: bool) -> (u64, u64) {
        let mut harness = Harness::new(cfg);

        harness.run(HEALTHY_TICKS, REPLIES_PER_TICK);
        let healthy = harness.output();

        if mark_degraded {
            harness.mark_degraded(Duration::from_secs(30));
        }

        // Replies stop while production continues.
        harness.run(20, 0);

        (healthy, harness.output())
    }

    fn sustaining_config(sustain: bool) -> SurbBalancerConfig {
        SurbBalancerConfig {
            // Small enough that the healthy phase actually reaches the setpoint and backs off
            // within the test's tick budget; saturated-at-maximum makes every phase read alike.
            target_surb_buffer_size: 1_000,
            max_surbs_per_sec: 2_500,
            surb_decay: None,
            sustain_on_return_path_loss: sustain,
        }
    }

    /// A peer with nothing to say really is filling up, so throttling it is correct.
    ///
    /// This is the case that makes the estimate impossible to fix locally: it is byte-for-byte the
    /// same observation as a dead return path.
    #[test_log::test]
    fn surb_balancer_should_throttle_when_a_quiet_counterparty_stops_consuming() {
        let (healthy, quiet) = drive_until_replies_stop(sustaining_config(false), false);

        assert!(healthy > 0, "a balanced session must keep minting");
        assert!(
            quiet < healthy,
            "an idle counterparty accumulates SURBs, so production must back off: healthy={healthy}/s, idle={quiet}/s"
        );
    }

    /// Once told the return path is dead, the same observation must not be read as a full buffer.
    ///
    /// `consumed` advances only when a reply reaches the entry (`manager.rs`, the `session_rx`
    /// inspect counting "received packets = SURB consumption estimate"). A return path that drops
    /// every reply therefore looks like a well-stocked counterparty, and production is cut at the
    /// exact moment the counterparty is draining towards empty -- the feedback signal travels on
    /// the very path whose failure it is meant to reveal.
    #[test_log::test]
    fn surb_balancer_should_sustain_production_through_a_degraded_return_path() {
        let (healthy, degraded) = drive_until_replies_stop(sustaining_config(true), true);

        assert!(healthy > 0, "a balanced session must keep minting");
        assert!(
            degraded >= healthy,
            "the counterparty is burning SURBs it cannot replace, so production must not be cut: healthy={healthy}/s, \
             degraded={degraded}/s"
        );
    }

    /// While replies are lost, the consumption observed is the loss, not the counterparty's
    /// spending. Feeding that forward would wind production down exactly as in the idle case, so
    /// the balancer must hold the last consumption it saw while replies still arrived.
    #[test_log::test]
    fn surb_balancer_should_hold_the_last_healthy_consumption_while_degraded() {
        let mut harness = Harness::new(sustaining_config(true));
        harness.run(HEALTHY_TICKS, REPLIES_PER_TICK);
        let consumption = (REPLIES_PER_TICK * 1000 / TICK.as_millis() as u64) as f64;
        assert!(
            harness.balancer.net_consumption_per_sec >= consumption * 0.95,
            "precondition: the healthy phase must have measured the consumption, got {}",
            harness.balancer.net_consumption_per_sec
        );

        harness.mark_degraded(Duration::from_secs(30));
        harness.run(100, 0);

        assert!(
            harness.output() as f64 >= consumption,
            "five seconds into an outage production must still cover the consumption last seen: {}/s against \
             {consumption}/s",
            harness.output()
        );
    }

    /// Both edges of a degraded window: open loop must engage at once, and let go afterwards.
    #[test_log::test]
    fn surb_balancer_should_return_to_closed_loop_when_the_return_path_recovers() {
        let mut harness = Harness::new(sustaining_config(true));

        harness.run(HEALTHY_TICKS, REPLIES_PER_TICK);
        let healthy = harness.output();

        harness.mark_degraded(Duration::from_millis(500));
        harness.tick(0);
        let first_degraded = harness.output();

        let mut degraded_peak = first_degraded;
        for _ in 0..9 {
            harness.tick(0);
            degraded_peak = degraded_peak.max(harness.output());
        }

        // The mark lapses and the counterparty starts answering again.
        harness.run(40, REPLIES_PER_TICK);
        let recovered = harness.output();

        assert!(
            first_degraded > healthy,
            "open loop must start raising production on the first update after the mark: healthy={healthy}/s, first \
             degraded update={first_degraded}/s"
        );
        assert!(
            recovered < degraded_peak,
            "once replies are arriving again the controller must return to closed loop rather than stay at the \
             open-loop rate: degraded peak={degraded_peak}/s, recovered={recovered}/s"
        );
    }

    /// After the outage the counterparty must be refilled, not merely un-throttled.
    ///
    /// Returning to closed loop is only half the claim: production has to actually climb the curve
    /// again and restore the buffer, within the refill bound the paced controller promises.
    #[test_log::test]
    fn surb_balancer_should_refill_the_counterparty_after_the_return_path_recovers() {
        let cfg = sustaining_config(true);
        let mut harness = Harness::new(cfg);

        harness.run(HEALTHY_TICKS, REPLIES_PER_TICK);
        // A band, not the setpoint itself: a sample taken at an arbitrary tick legitimately sits
        // either side of it.
        let refilled = cfg.target_surb_buffer_size / 2;
        assert!(
            harness.level() >= refilled,
            "the healthy phase must reach the setpoint band before an outage means anything"
        );

        // The return path dies: replies stop, and open loop takes over.
        harness.mark_degraded(Duration::from_millis(400));
        harness.run(8, 0);

        // The mark lapses, the counterparty answers again, and the belief restarts from empty.
        let mut ticks_to_refill = None;
        for n in 1..=200 {
            harness.tick(REPLIES_PER_TICK);
            if harness.level() >= refilled {
                ticks_to_refill = Some(n);
                break;
            }
        }

        let ticks = ticks_to_refill.expect("the counterparty must be refilled to the setpoint after recovery");
        tracing::info!(ticks, "refilled the counterparty after the outage");

        // Half the gap closes within `H·ln 2` once production has ramped, and ramping takes at
        // most `T_ramp`.
        let bound =
            (DEFAULT_RAMP_TIME.as_secs_f64() + DEFAULT_REFILL_HORIZON.as_secs_f64() * 2f64.ln()) / TICK.as_secs_f64();
        assert!(
            ticks as f64 <= bound.ceil(),
            "refilling must stay within the paced bound: took {ticks} sampling intervals, bound {bound:.0}"
        );
    }

    /// SURBs piggybacked on Session data refill the buffer for free, so keep-alives only have to
    /// make up what they do not cover.
    #[test_log::test]
    fn organic_supply_should_displace_keep_alive_production() {
        let mut harness = Harness::new(sustaining_config(false));

        // Every SURB spent is replaced by one riding on a data packet.
        for _ in 0..HEALTHY_TICKS {
            harness
                .estimator
                .produced
                .fetch_add(REPLIES_PER_TICK, std::sync::atomic::Ordering::Relaxed);
            harness.tick(REPLIES_PER_TICK);
        }

        assert!(
            harness.level() >= sustaining_config(false).target_surb_buffer_size * 9 / 10,
            "precondition: keep-alives must still have filled the buffer to the setpoint, got {}",
            harness.level()
        );
        assert!(
            harness.output() <= 50,
            "with consumption covered by organic supply, keep-alives only close what is left of the gap: {}/s",
            harness.output()
        );
    }

    /// The estimate must not claim a level the counterparty's store could never have held.
    ///
    /// `produced - consumed` only decreases when a reply arrives, so production that nobody is
    /// seen to consume accumulates without bound. The counterparty's store is a ring buffer that
    /// evicts the oldest entry on overflow, so everything above its capacity was discarded on
    /// arrival. Measured during a live outage: 51 917 believed against a 15 000-entry store, which
    /// keeps the controller throttling against a buffer that is in fact draining.
    #[test_log::test]
    fn surb_balancer_should_not_believe_a_level_the_counterparty_cannot_hold() {
        const CAPACITY: u64 = 2_000;

        let mut harness = Harness::new(sustaining_config(false));
        harness.state.set_counterparty_buffer_capacity(CAPACITY);

        harness.run(HEALTHY_TICKS, REPLIES_PER_TICK);

        // Replies stop, while production continues from a source the controller does not drive --
        // keep-alives mint on their own schedule, which is how the live estimate ran away.
        for _ in 0..20 {
            harness
                .estimator
                .produced
                .fetch_add(500, std::sync::atomic::Ordering::Relaxed);
            harness.tick(0);
        }

        let believed = harness.level();
        assert!(
            believed <= CAPACITY,
            "the estimate must be bounded by the counterparty's store: believed {believed} against a {CAPACITY}-entry \
             buffer"
        );
    }

    /// The bound must not become the setpoint: a store larger than the target changes nothing.
    #[test_log::test]
    fn surb_balancer_should_leave_a_healthy_session_untouched_by_the_capacity_bound() {
        let cfg = sustaining_config(false);
        let mut harness = Harness::new(cfg);
        harness
            .state
            .set_counterparty_buffer_capacity(cfg.target_surb_buffer_size * 10);

        harness.run(HEALTHY_TICKS, REPLIES_PER_TICK);

        let level = harness.level();
        assert!(
            level >= cfg.target_surb_buffer_size / 2,
            "a capacity well above the target must not hold the session below its setpoint: level {level}, target {}",
            cfg.target_surb_buffer_size
        );
    }

    /// The opt-in is what enables it; evidence alone must not change a session's behaviour.
    #[test_log::test]
    fn surb_balancer_should_ignore_a_degraded_return_path_unless_configured_to_sustain() {
        let (healthy, degraded) = drive_until_replies_stop(sustaining_config(false), true);

        assert!(
            degraded < healthy,
            "without the opt-in this is not our behaviour to change: healthy={healthy}/s, degraded={degraded}/s"
        );
    }

    #[test_log::test(tokio::test)]
    async fn surb_balancer_should_start_decrease_level_when_above_target_and_decay_enabled() {
        const NUM_STEPS: usize = 5;
        let session_id = HoprPseudonym::random();
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 5_000,
            max_surbs_per_sec: 2500,
            surb_decay: Some((Duration::from_millis(200), 0.05)),
            sustain_on_return_path_loss: false,
        };

        let mut mock_flow_ctl = MockSurbFlowController::new();
        mock_flow_ctl
            .expect_adjust_surb_flow()
            .times(NUM_STEPS)
            .returning(|_| ());

        let balancer = SurbBalancer::new(
            session_id,
            PidBalancerController::default(),
            SimpleSurbFlowEstimator::default(),
            mock_flow_ctl,
            Arc::new(cfg.into()),
        );

        balancer
            .state
            .buffer_level
            .store(5000, std::sync::atomic::Ordering::Relaxed);

        let (stream, handle) = balancer.start_control_loop(Duration::from_millis(100));
        let levels = stream.take(NUM_STEPS).collect::<Vec<_>>().await;
        handle.abort();

        assert_eq!(levels.len(), NUM_STEPS);
        assert!(
            levels.windows(2).all(|w| w[1] <= w[0]),
            "buffer levels should be monotonic non-increasing: {levels:?}"
        );
        assert!(
            levels.last().is_some_and(|last| *last < 5_000),
            "expected at least one decay step: {levels:?}"
        );
    }

    // --- return-path-degraded deadline primitive (Test C, gap 1) ---------------------------------
    //
    // The behavioural loop tests above (sustain / ignore-unless-opted-in / return-to-closed-loop /
    // refill) already cover how the balancer *reacts* to the signal. What they do not isolate are
    // three edge cases of the `mark_return_path_degraded` / `return_path_estimate_is_stale`
    // deadline primitive itself, each guarding a specific correctness invariant.

    /// The zero-guard: never marked is never degraded. `return_path_degraded_until_ms == 0` means
    /// "never marked", not "marked at the epoch" — without the explicit `deadline > 0` check every
    /// opted-in session would believe its return path was dead from the first packet.
    #[test]
    fn return_path_should_not_be_stale_before_it_is_ever_marked() {
        let state = BalancerStateValues::new(SurbBalancerConfig {
            sustain_on_return_path_loss: true,
            ..Default::default()
        });
        assert!(!state.return_path_estimate_is_stale());
    }

    /// The window expires on its own, independently of any recovery signal. The behavioural tests
    /// clear the degraded state by resuming replies; this pins the other exit — a marker nobody
    /// ever withdraws must still lapse, bounding over-production to the grace window. Marking with a
    /// zero grace sets the deadline to "now"; the monotonic clock only moves forward, so the
    /// subsequent read is already past it — no sleep, no flake.
    #[test]
    fn an_expired_degraded_window_should_no_longer_be_stale() {
        let state = BalancerStateValues::new(SurbBalancerConfig {
            sustain_on_return_path_loss: true,
            ..Default::default()
        });
        state.mark_return_path_degraded(Duration::ZERO);
        assert!(!state.return_path_estimate_is_stale());
    }

    /// Re-marking extends the window: an already-expired deadline followed by a fresh mark is stale
    /// again. Guards the `fetch_max` in `mark_return_path_degraded`.
    #[test]
    fn re_marking_should_reopen_an_expired_window() {
        let state = BalancerStateValues::new(SurbBalancerConfig {
            sustain_on_return_path_loss: true,
            ..Default::default()
        });
        state.mark_return_path_degraded(Duration::ZERO);
        assert!(!state.return_path_estimate_is_stale());
        state.mark_return_path_degraded(Duration::from_secs(10));
        assert!(state.return_path_estimate_is_stale());
    }
}
