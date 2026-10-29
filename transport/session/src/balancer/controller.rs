use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicU64},
    },
    time::Duration,
};

use futures::{StreamExt, pin_mut};
use hopr_crypto_packet::prelude::{PacketSignal, PacketSignals};
use hopr_utils::runtime::AbortHandle;
use tracing::{Instrument, instrument};

use super::{
    BalancerControllerBounds, MIN_BALANCER_SAMPLING_INTERVAL, SimpleSurbFlowEstimator, SurbBalancerController,
    SurbFlowController, SurbFlowEstimator,
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
    /// The default is 5000 (which is 2500 packets/second currently)
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
    /// SURBs counted in [`buffer_level`](Self::buffer_level), held in the counterparty's store, whose
    /// return path runs through a relayer judged dead on any hop, and their share of what is outstanding:
    /// count in the high and share (of `u32::MAX`) in the low 32 bits, so both are read together.
    invalid_surbs: AtomicU64,
    /// How many SURBs the counterparty can physically hold, or 0 when unknown.
    ///
    /// The level estimate is `produced - consumed - evicted`. The Exit observes its own store's
    /// evictions first-hand (`num_evicted_surbs` on each incoming packet) and counts them, so the
    /// level it reports tracks what the store actually holds instead of growing without bound when a
    /// return path drops replies -- consumption alone is observed only once a reply arrives, and
    /// before evictions were counted the belief ran away (measured during an outage: 51 917 believed
    /// against a 15 000-entry store).
    ///
    /// This capacity still bounds the level, as defense-in-depth, via the `max(capacity, target)`
    /// clamp below: it covers the paths that cannot see evictions -- the Entry's own
    /// `produced - consumed` belief between the Exit's reports, and any session whose capacity is
    /// unknown. The bound is `max(capacity, target)` rather than `capacity` so a target above the
    /// counterparty's real store does not pin the error permanently negative and hold production at
    /// maximum forever.
    ///
    /// ## When the store is smaller than the target
    ///
    /// With an accurate level, a counterparty whose store is *smaller* than the target reports a level
    /// that plateaus at its real capacity and never reaches target, which on its own would leave
    /// organic production running and the full buffer evicting. That corner is closed by
    /// [`observed_counterparty_capacity`](Self::observed_counterparty_capacity), which infers the real
    /// capacity from the plateau and caps the effective target there. It is unreachable with
    /// homogeneous configuration (`target <= 2/3` of the store) in any case; it needs the Entry's store
    /// to exceed ~1.5x the Exit's.
    pub counterparty_buffer_capacity: AtomicU64,
    /// Whether the counterparty's last packet said it was running low on SURBs of its own.
    ///
    /// A plain flag with no deadline: a distress flag that is never withdrawn merely keeps organic
    /// production at one SURB per packet — the behaviour that predates the gate — so it does not
    /// need to expire.
    ///
    /// It clears on its own in the normal case: the counterparty recomputes both SURB signals on
    /// every return packet and strips them once its pool recovers, so the next healthy packet
    /// resets this.
    pub counterparty_in_surb_distress: AtomicBool,
    /// The counterparty's real store capacity, inferred from where its reported level plateaus, or
    /// `0` when not yet inferred.
    ///
    /// Closes the `capacity < target` corner noted on
    /// [`counterparty_buffer_capacity`](Self::counterparty_buffer_capacity): when the reported level
    /// stops rising while the Entry is still minting toward the target and the counterparty is not in
    /// distress, its store is full at a level below the target, and that level is its real capacity.
    /// [`organic_surbs_per_packet`](Self::organic_surbs_per_packet) then stops production at
    /// `min(target, this)` instead of a target the store can never reach.
    ///
    /// Biased hard against firing, since a false positive under-supplies the counterparty (see the
    /// destructive-recovery note in the SURB balancer gotchas): it takes several consecutive
    /// non-rising reports (see [`observe_reported_level`](Self::observe_reported_level)), and a
    /// [`SurbDistress`](hopr_crypto_packet::prelude::PacketSignal::SurbDistress) signal discards it
    /// outright -- a counterparty short of SURBs is not full. Set back to `0` (re-probe) if the level
    /// later rises above it.
    pub observed_counterparty_capacity: AtomicU64,
    /// Consecutive reported levels that did not rise while below target -- the plateau hysteresis
    /// behind [`observed_counterparty_capacity`](Self::observed_counterparty_capacity).
    pub capacity_plateau_count: AtomicU64,
    /// Set by [`on_counterparty_wiped`](Self::on_counterparty_wiped), consumed by the next balancer
    /// update: the counterparty dropped its whole stock, so that update restarts the level estimate
    /// from empty.
    ///
    /// A flag rather than a plain store of `0` into [`buffer_level`](Self::buffer_level) because the
    /// level has another writer: the update in flight, which would write back the level it loaded
    /// before the reset. (The counterparty's keep-alive is the other, which may still carry its
    /// pre-wipe level; [`observe_reported_level`](Self::observe_reported_level) drops such a report
    /// while the refill is paced.)
    counterparty_wiped: AtomicBool,
    /// Paces the refill after a wipe: the rate in SURBs per second in the high 32 bits and the
    /// milliseconds it still holds for in the low 32 bits, so both are read together. Zero is
    /// inactive. See [`REFILL_PACING_SECS`].
    refill_pacing: AtomicU64,
    /// The instant [`last_heard`](Self::last_heard) and [`last_sent`](Self::last_sent) count from.
    epoch: ActivityEpoch,
    /// When the counterparty was last heard from, in milliseconds since `epoch` plus one, so that `0`
    /// is "never". Any packet from it counts, and so does a reply the estimator counted as consumed.
    /// Entry-side only; see [`silence_sustain`](Self::silence_sustain).
    last_heard: AtomicU64,
    /// When a Session data packet last went out, in the same unit as [`last_heard`](Self::last_heard).
    last_sent: AtomicU64,
}

/// The instant the activity stamps of a [`BalancerStateValues`] count from: when it was made.
#[derive(Debug, Clone, Copy)]
struct ActivityEpoch(std::time::Instant);

impl Default for ActivityEpoch {
    fn default() -> Self {
        Self(std::time::Instant::now())
    }
}

/// How long data can go out with nothing at all coming back before the Entry sustains the
/// counterparty's SURBs on its own: three seconds.
///
/// The Exit reports its SURB level at least every second and replies at the rate of the data it
/// receives, so three seconds of data going out and nothing coming back, neither a report, nor any
/// other packet, nor a reply home, means that the Exit's consumption is invisible: its stock may be
/// draining into a return path that delivers nothing, and the balancer, which learns the level from
/// those reports and replies, cannot see it and would mint nothing. See
/// [`silence_sustain`](BalancerStateValues::silence_sustain).
pub const SILENCE_SUSTAIN_AFTER: Duration = Duration::from_secs(3);

/// How long the refill that follows a wipe of the counterparty's stock is paced for, at a rate of
/// one fifth of the target per second, so the refill takes about as many seconds.
///
/// After a wipe the level is zero against the target, and left alone the PID commands its output
/// limit (`max_surbs_per_sec`; the prototype hit 4975 per second against a ceiling of 5000 per
/// second in the integration cluster) until the level is
/// back. On a path through two or three relayers on one machine that burst congests the relays
/// themselves, the replies it is meant to restore arrive late, and late replies are what the
/// verdicts are made of. The organic gate is open for the whole refill anyway, so spreading it over
/// a few seconds costs little: the SURBs of the next data packets arrive in the meantime.
///
/// The start of a session at the Entry is the same state: the counterparty holds nothing, the level
/// estimate says so, and the PID would command its ceiling from the first tick. That flood lost
/// replies at session start (one flush tick about three seconds in: 1037 expected, 201 observed),
/// the corroborated verdict judged the two slowest relayers dead on those, and wiped the fresh
/// stock; hoprnet#8484's cold burst was the same flood seen from the data path. The Entry arms the
/// same pacing when its balancer starts ([`BalancerStateValues::arm_refill_pacing`]), and the fill
/// reaches the target in about five seconds instead of one and a half, sparing the relays.
///
/// While the window lasts the PID does not run: the balancer commands the cap while the level is
/// below the target and nothing from the moment it is reached. A cap applied to the PID's output
/// instead would let it integrate the full error for the whole window and answer a level at or past
/// the target with its ceiling when the window ends. The controller is rebuilt with its own gains and
/// bounds when the window ends, so that it starts from the current error. Neither `max_surbs_per_sec`
/// nor the bounds [`SurbBalancer::start_control_loop`] rebuilds the PID from change.
pub const REFILL_PACING_SECS: u64 = 5;

/// Consecutive non-rising reported levels (below target, no distress) before the counterparty's
/// store capacity is inferred. Small, but greater than one: a single flat sample is ordinary churn,
/// and the cost of a false inference is under-supplying the counterparty, so the bar errs high.
pub(crate) const CAPACITY_PLATEAU_THRESHOLD: u64 = 3;

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
        // Any packet from the counterparty carries them, so this is the Entry hearing the Exit.
        self.note_heard();
        let distress = signals.contains(PacketSignal::SurbDistress);
        self.counterparty_in_surb_distress
            .store(distress, std::sync::atomic::Ordering::Relaxed);

        if distress {
            // A counterparty short of SURBs is not full: discard any inferred capacity so the Entry
            // resumes producing toward the full target and re-probes once distress clears. This is the
            // corrective that keeps a too-low capacity guess from starving the counterparty.
            self.observed_counterparty_capacity
                .store(0, std::sync::atomic::Ordering::Relaxed);
            self.capacity_plateau_count
                .store(0, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Records the SURB level the counterparty reported, and infers its real store capacity from a
    /// plateau in that level.
    ///
    /// Called on every `BalancerState` keep-alive, including one that repeats the last value -- a
    /// level that stops rising while the Entry is still minting toward the target is the plateau
    /// signal. See [`observed_counterparty_capacity`](Self::observed_counterparty_capacity).
    ///
    /// While the refill after a wipe is paced, a report above the level believed here is dropped
    /// altogether: stored by neither the level nor the plateau bookkeeping. It is the counterparty
    /// reporting the stock it had before it saw the new SURB generation, and storing it would put the
    /// level back at the target, shut the organic gate and silence the PID while the counterparty holds
    /// nothing. A legitimate report is never above this side's own belief, which counts a SURB as
    /// produced when it is sent and as consumed only when the reply is home.
    pub fn observe_reported_level(&self, new_level: u64) {
        use std::sync::atomic::Ordering::Relaxed;

        // Heard even when the report is dropped below: it is the Exit speaking.
        self.note_heard();
        let prev = self.buffer_level.load(Relaxed);
        if new_level > prev && self.refill_pacing_active() {
            tracing::debug!(
                reported = new_level,
                believed = prev,
                "dropping a counterparty SURB level above the believed one while the refill after a wipe is paced"
            );
            return;
        }
        let target = self.target_surb_buffer_size.load(Relaxed);
        let distress = self.counterparty_in_surb_distress.load(Relaxed);
        let inferred = self.observed_counterparty_capacity.load(Relaxed);

        if inferred != 0 && new_level > inferred {
            // The counterparty is holding more than we inferred, so the store is larger than the
            // plateau suggested (or has grown). Forget the guess and probe again.
            self.observed_counterparty_capacity.store(0, Relaxed);
            self.capacity_plateau_count.store(0, Relaxed);
        } else if inferred == 0 && !distress && new_level > 0 && new_level < target && new_level <= prev {
            // Below target with no distress, yet the level is not rising: the Entry is minting toward
            // the target but the counterparty is not accepting more, so its store is full at a level
            // below the target. Require several consecutive such reports before acting.
            let plateau = self.capacity_plateau_count.fetch_add(1, Relaxed) + 1;
            if plateau >= CAPACITY_PLATEAU_THRESHOLD {
                self.observed_counterparty_capacity.store(new_level, Relaxed);
            }
        } else {
            // Rising, at/above target, in distress, or refilling below an inferred cap: not a plateau.
            self.capacity_plateau_count.store(0, Relaxed);
        }

        self.buffer_level.store(new_level, Relaxed);
    }

    /// How many SURBs an outgoing Session data packet should carry along with its payload.
    ///
    /// Entry-side only. [`BalancerStateValues`] is shared by both ends of a Session, but only the
    /// initiator mints SURBs for its counterparty, so on the Exit this answer is meaningless.
    ///
    /// Returning `0` is the point: a SURB delivered to a counterparty that is already at its target
    /// evicts the oldest one it holds, and under PIX that destroys an SSA share rather than merely
    /// wasting a SURB. Production resumes at one per packet as soon as the estimate falls back below
    /// target, or immediately if the counterparty signals it is running low.
    ///
    /// This uses the usable level: SURBs the counterparty holds through a relayer judged dead do not
    /// hold the gate shut.
    pub fn organic_surbs_per_packet(&self) -> usize {
        // Not defensive boilerplate: nothing rejects a `SurbBalancerConfig` with a zero target, and
        // without this branch `level >= 0` would hold forever and shut organic production off
        // permanently — while a PID with a zero output limit produces nothing either.
        if self.is_disabled() {
            return 1;
        }

        if self
            .counterparty_in_surb_distress
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return 1;
        }

        // Stop at the counterparty's real capacity when it is known to be smaller than the target:
        // filling past a store that cannot grow only evicts, which under PIX destroys SSA shares.
        let target = self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed);
        let effective_target = match self
            .observed_counterparty_capacity
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            0 => target,
            capacity => target.min(capacity),
        };

        if self.usable_level(self.buffer_level()) >= effective_target {
            0
        } else {
            1
        }
    }

    /// Records that the counterparty was heard from now.
    pub(crate) fn note_heard(&self) {
        self.stamp(&self.last_heard, std::time::Instant::now());
    }

    /// Records that a Session data packet is going out now.
    pub(crate) fn note_sent(&self) {
        self.stamp(&self.last_sent, std::time::Instant::now());
    }

    /// Test hook: as if the counterparty had last been heard from `ago` ago.
    ///
    /// The stamps count from the instant the state was made, so for one to lie in the past the epoch
    /// has to be moved back before it.
    #[cfg(test)]
    pub(crate) fn last_heard_ago(mut self, ago: Duration) -> Self {
        let now = std::time::Instant::now();
        self.epoch = ActivityEpoch(
            now.checked_sub(ago + Duration::from_secs(60))
                .expect("monotonic clock younger than the heard-ago offset"),
        );
        self.stamp(
            &self.last_heard,
            now.checked_sub(ago).expect("monotonic clock younger than `ago`"),
        );
        self
    }

    /// Stamps `slot` with `at`, never moving it back.
    fn stamp(&self, slot: &AtomicU64, at: std::time::Instant) {
        let ms = u64::try_from(at.saturating_duration_since(self.epoch.0).as_millis()).unwrap_or(u64::MAX - 1);
        slot.fetch_max(ms.saturating_add(1), std::sync::atomic::Ordering::Relaxed);
    }

    /// Whether data is going out into silence: the balancer is enabled, the counterparty was heard from
    /// at least once, and the latest data packet went out more than [`SILENCE_SUSTAIN_AFTER`] after the
    /// last time it was.
    ///
    /// The Entry then keeps one SURB in each data packet it sends even though its level estimate is at
    /// or above the target (see `cap_organic_surbs` in the session manager). The estimate is fed by the
    /// Exit's reports and replies; with none arriving it stays where it was, while the Exit goes on
    /// answering the data it gets, one SURB for each reply, into a return path that may be delivering
    /// nothing. The estimate then never falls below the target and the Entry mints nothing, until the
    /// Exit's stock is empty. One SURB per data packet is what the Exit can be expected to spend and no
    /// more: the sustain is bounded by the rate of the data, makes no burst, and wipes nothing. It ends
    /// at the first packet or reply from the Exit.
    ///
    /// Judged between the two stamps, not against the clock, so that the answer is about the packet being
    /// sent: it takes [`note_sent`](Self::note_sent) for that packet first.
    pub(crate) fn silence_sustain(&self) -> bool {
        use std::sync::atomic::Ordering::Relaxed;

        if self.is_disabled() {
            return false;
        }
        match self.last_heard.load(Relaxed) {
            0 => false,
            heard => {
                let after = u64::try_from(SILENCE_SUSTAIN_AFTER.as_millis()).unwrap_or(u64::MAX);
                self.last_sent.load(Relaxed) > heard.saturating_add(after)
            }
        }
    }

    /// Tells the state that the counterparty dropped every SURB it held, because the SURB generation
    /// of the session advanced and the counterparty clears its store on the first batch of the new one.
    ///
    /// The level estimate restarts from empty at the next balancer update, and for
    /// [`REFILL_PACING_SECS`] the refill is held to `target / REFILL_PACING_SECS` SURBs per second.
    /// The counterparty's own reports follow suit once the first SURBs of the new generation reach
    /// it. Until then it may still report the level it had before; for the paced window such a
    /// report, being above what is believed here, is dropped (see
    /// [`observe_reported_level`](Self::observe_reported_level)).
    pub fn on_counterparty_wiped(&self) {
        use std::sync::atomic::Ordering::Release;

        self.buffer_level.store(0, Release);
        self.counterparty_wiped.store(true, Release);
        self.arm_refill_pacing();
    }

    /// Holds the refill of an empty counterparty to `target / REFILL_PACING_SECS` SURBs per second for
    /// [`REFILL_PACING_SECS`] (at least one per second), whatever the PID would command. A disabled
    /// balancer arms nothing.
    ///
    /// The second half of [`on_counterparty_wiped`](Self::on_counterparty_wiped), and what the Entry
    /// calls when its balancer starts: a session that has just been opened is in the same state as one
    /// that has just been wiped, the counterparty holds nothing and the level estimate says so, and
    /// the PID would command its ceiling just the same. Unlike a wipe it resets nothing: the estimate
    /// is already empty, and what the first tick produced is the new session's.
    pub fn arm_refill_pacing(&self) {
        use std::sync::atomic::Ordering::Relaxed;

        if self.is_disabled() {
            return;
        }
        let cap = (self.target_surb_buffer_size.load(Relaxed) / REFILL_PACING_SECS).clamp(1, u64::from(u32::MAX));
        self.refill_pacing
            .store((cap << 32) | (REFILL_PACING_SECS * 1000), Relaxed);
    }

    /// Takes the one-shot request to restart the level estimate from empty, if there is one.
    fn take_counterparty_wiped(&self) -> bool {
        self.counterparty_wiped.swap(false, std::sync::atomic::Ordering::AcqRel)
    }

    /// Whether the refill after a wipe is still paced. Read-only: unlike [`pace_refill`](Self::pace_refill)
    /// it does not use up any of the window.
    pub(crate) fn refill_pacing_active(&self) -> bool {
        self.refill_pacing.load(std::sync::atomic::Ordering::Relaxed) & u64::from(u32::MAX) > 0
    }

    /// The cap on the control output while the refill after a wipe is paced, in SURBs per second, and
    /// uses up `elapsed` of the pacing window. `None` when no pacing is active.
    fn pace_refill(&self, elapsed: Duration) -> Option<u64> {
        const REMAINING: u64 = u32::MAX as u64;

        let elapsed_ms = elapsed.as_millis().min(u128::from(u32::MAX)) as u64;
        self.refill_pacing
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |packed| {
                    let remaining = packed & REMAINING;
                    (remaining > 0).then(|| match remaining.saturating_sub(elapsed_ms) {
                        0 => 0,
                        left => (packed & !REMAINING) | left,
                    })
                },
            )
            .ok()
            .map(|packed| packed >> 32)
    }

    /// Sets how many stored SURBs are invalid and their share of what is outstanding.
    pub fn set_invalid_surbs(&self, count: u64, share: f64) {
        let share = (share.clamp(0.0, 1.0) * f64::from(u32::MAX)).round() as u64;
        self.invalid_surbs.store(
            count.min(u64::from(u32::MAX)) << 32 | share,
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    /// The part of `level` that can still bring a reply home.
    fn usable_level(&self, level: u64) -> u64 {
        let packed = self.invalid_surbs.load(std::sync::atomic::Ordering::Relaxed);
        let (count, share) = (
            packed >> 32,
            (packed & u64::from(u32::MAX)) as f64 / f64::from(u32::MAX),
        );
        // The count can overstate what the counterparty still holds; the share caps it by its level.
        level.saturating_sub(count.min((level as f64 * share).round() as u64))
    }

    /// Extracts the [`SurbBalancerConfig`] from the [`BalancerStateValues`].
    pub fn as_config(&self) -> SurbBalancerConfig {
        SurbBalancerConfig {
            target_surb_buffer_size: self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
            max_surbs_per_sec: self.max_surbs_per_sec.load(std::sync::atomic::Ordering::Relaxed),
            surb_decay: self.surb_decay(),
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
    /// Whether the previous update ran inside the refill pacing window of a wipe, to notice the one
    /// in which the window ends.
    was_pacing: bool,
    /// DIAGNOSTIC: when the last balancer-state line was emitted, to rate-limit it.
    last_report: std::time::Instant,
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
            was_pacing: false,
            last_report: std::time::Instant::now(),
        }
    }

    /// Computes the next control update and adjusts the [`SurbFlowController`] rate accordingly.
    #[tracing::instrument(level = "trace", skip_all)]
    fn update(&mut self) -> u64 {
        let dt = self.last_update.elapsed();

        // Load the updated current buffer level
        let mut current = self.state.buffer_level.load(std::sync::atomic::Ordering::Acquire);

        if dt < Duration::from_millis(10) {
            tracing::debug!("time elapsed since last update is too short, skipping update");
            return current;
        }

        self.last_update = std::time::Instant::now();

        // Take a snapshot of the active SURB estimator and calculate the balance change
        let snapshot = SimpleSurbFlowEstimator::from(&self.surb_estimator);
        let Some(target_buffer_change) = snapshot.estimated_surb_buffer_change(&self.last_estimator_state) else {
            tracing::error!("non-monotonic change in SURB estimators");
            return current;
        };

        // A reply came home since the last tick: the counterparty answered, whether or not any of its
        // packets reached the hooks that record it.
        if snapshot.consumed > self.last_estimator_state.consumed {
            self.state.note_heard();
        }

        self.last_estimator_state = snapshot;
        if self.state.take_counterparty_wiped() {
            // The counterparty dropped everything it held, so what this tick's production and
            // consumption add to the level belongs to the stock that is gone: restart from empty.
            tracing::debug!(
                believed = current,
                delta = target_buffer_change,
                "counterparty SURBs were wiped, restarting the level estimate from empty"
            );
            current = 0;
        } else {
            current = current.saturating_add_signed(target_buffer_change);
        }

        // If SURB decaying is enabled, check if the decay window has elapsed
        // and calculate the number of SURBs that will be discarded
        if let Some(num_decayed_surbs) = self
            .state
            .surb_decay()
            .filter(|(decay_window, _)| &self.last_decay.elapsed() >= decay_window)
            .map(|(_, decay_coeff)| (self.controller.bounds().target() as f64 * decay_coeff).round() as u64)
        {
            current = current.saturating_sub(num_decayed_surbs);
            self.last_decay = std::time::Instant::now();
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

        self.state
            .buffer_level
            .store(current, std::sync::atomic::Ordering::Release);

        // Invalid SURBs are held but cannot bring a reply home, so the balancer replaces them.
        let usable = self.state.usable_level(current);

        // Error from the desired target SURB buffer size at counterparty
        let error = usable as i64 - self.controller.bounds().target() as i64;

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

        // After a wipe the level is far below the target and the PID would command its output limit
        // until it is back. Hold the refill to a pace the return paths can carry instead. The PID does
        // not run at all while the window lasts: clamping its output would still let it integrate the
        // full error for the whole window, and when the window ends it would answer a level at or past
        // the target with the ceiling. The refill stops by itself the moment the target is reached. The
        // controller's bounds are not touched.
        let refill_cap = self.state.pace_refill(dt);
        let output = match refill_cap {
            Some(cap) => {
                if usable >= self.controller.bounds().target() {
                    0
                } else {
                    cap
                }
            }
            None => {
                if self.was_pacing {
                    // The window just ended. Start the controller from the current error rather than
                    // from whatever it had accumulated before the wipe, which no longer describes
                    // anything: setting the bounds it already has rebuilds it with its own gains.
                    self.controller.set_target_and_limit(self.controller.bounds());
                    tracing::debug!(
                        current,
                        "refill pacing is over, the controller restarts from the current level"
                    );
                }
                self.controller.next_control_output(usable)
            }
        };
        self.was_pacing = refill_cap.is_some();
        tracing::trace!(output, "next balancer control output for session");

        // Both ends run this same loop -- the Entry with the PID driving production, the Exit with
        // the proportional controller gating egress -- so one line covers both and the session id
        // tells them apart. Rate-limited to one per second so it can run under a full-rate session.
        //
        // At `debug` rather than `info`: one line per session per second is fine for a handful of
        // sessions and is a lot of formatting work for a node carrying many, none of which an
        // operator needs to see during healthy operation.
        if self.last_report.elapsed() >= Duration::from_secs(1) {
            self.last_report = std::time::Instant::now();
            tracing::debug!(
                session = %self.session_id,
                level = current,
                invalid = current - usable,
                target = self.controller.bounds().target(),
                output,
                refill_cap = refill_cap.unwrap_or(0),
                produced = self.surb_estimator.estimate_surbs_produced(),
                consumed = self.surb_estimator.estimate_surbs_consumed(),
                distress = self
                    .state
                    .counterparty_in_surb_distress
                    .load(std::sync::atomic::Ordering::Relaxed),
                silent = self.state.silence_sustain(),
                "surb balancer state"
            );
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
    use crate::balancer::{AtomicSurbFlowEstimator, MockSurbFlowController, pid::PidBalancerController};

    #[test]
    fn surb_balancer_config_should_be_convertible_to_atomics() {
        let cfg = SurbBalancerConfig::default();
        let state_data = BalancerStateValues::new(cfg);
        assert_eq!(cfg, state_data.as_config());
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

    #[test]
    fn balancer_state_values_from_config() {
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 5000,
            max_surbs_per_sec: 2500,
            surb_decay: Some((Duration::from_secs(60), 0.05)),
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
                }
                .into(),
            ),
        );

        let mut last_update = 0;
        for i in 0..steps {
            std::thread::sleep(step_duration);
            surb_estimator.produced.fetch_add(
                production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );
            surb_estimator.consumed.fetch_add(
                consumption_rate * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );

            let next_update = balancer.update();
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

        let mut last_update = 0;
        for i in 0..steps {
            std::thread::sleep(step_duration);
            surb_estimator.produced.fetch_add(
                production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );
            surb_estimator.consumed.fetch_add(
                consumption_rate * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );

            let next_update = balancer.update();
            assert!(
                i == 0 || next_update < last_update,
                "{next_update} should be greater than {last_update}"
            );
            last_update = next_update;
        }
    }

    /// A balancer whose production follows its own control output, as it does in a live Session.
    ///
    /// Returns the balancer, the shared estimator and the latest control output. Production must be
    /// fed back rather than held constant: with production pinned to consumption the buffer never
    /// fills, maximum output is the correct answer, and every phase of the test reads the same.
    #[allow(clippy::type_complexity)]
    fn balancer_with_feedback(
        cfg: SurbBalancerConfig,
    ) -> (
        SurbBalancer<PidBalancerController, AtomicSurbFlowEstimator, MockSurbFlowController>,
        AtomicSurbFlowEstimator,
        Arc<BalancerStateValues>,
        Arc<AtomicU64>,
    ) {
        let output = Arc::new(AtomicU64::new(0));
        let output_clone = output.clone();
        let mut controller = MockSurbFlowController::new();
        controller.expect_adjust_surb_flow().returning(move |r| {
            output_clone.store(r as u64, std::sync::atomic::Ordering::Relaxed);
        });

        let surb_estimator = AtomicSurbFlowEstimator::default();
        let state: Arc<BalancerStateValues> = Arc::new(cfg.into());
        let balancer = SurbBalancer::new(
            HoprPseudonym::random(),
            PidBalancerController::default(),
            surb_estimator.clone(),
            controller,
            state.clone(),
        );

        (balancer, surb_estimator, state, output)
    }

    /// One sampling interval: mint at the rate last commanded, and consume `consumed` of them.
    fn tick(
        balancer: &mut SurbBalancer<PidBalancerController, AtomicSurbFlowEstimator, MockSurbFlowController>,
        surb_estimator: &AtomicSurbFlowEstimator,
        output: &AtomicU64,
        consumed: u64,
    ) {
        let step = Duration::from_millis(50);
        std::thread::sleep(step);

        let minted = output.load(std::sync::atomic::Ordering::Relaxed) * step.as_millis() as u64 / 1000;
        surb_estimator
            .produced
            .fetch_add(minted, std::sync::atomic::Ordering::Relaxed);
        surb_estimator
            .consumed
            .fetch_add(consumed, std::sync::atomic::Ordering::Relaxed);
        balancer.update();
    }

    /// SURBs the counterparty spends per interval while it is answering normally.
    const REPLIES_PER_TICK: u64 = 40;

    /// Drives a balancer through a healthy stretch, then through one where no reply comes back.
    ///
    /// Returns the control output at the end of each stretch.
    fn drive_until_replies_stop(cfg: SurbBalancerConfig) -> (u64, u64) {
        let (mut balancer, surb_estimator, _state, output) = balancer_with_feedback(cfg);

        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }
        let healthy = output.load(std::sync::atomic::Ordering::Relaxed);

        // Replies stop while production continues.
        for _ in 0..20 {
            tick(&mut balancer, &surb_estimator, &output, 0);
        }

        (healthy, output.load(std::sync::atomic::Ordering::Relaxed))
    }

    fn small_target_config() -> SurbBalancerConfig {
        SurbBalancerConfig {
            // Small enough that the healthy phase actually reaches the setpoint and backs off
            // within the test's tick budget; saturated-at-maximum makes every phase read alike.
            target_surb_buffer_size: 1_000,
            max_surbs_per_sec: 2_500,
            surb_decay: None,
        }
    }

    /// A peer with nothing to say really is filling up, so throttling it is correct.
    ///
    /// This is the case that makes the estimate impossible to fix locally: it is byte-for-byte the
    /// same observation as a dead return path.
    #[test_log::test]
    fn surb_balancer_should_throttle_when_a_quiet_counterparty_stops_consuming() {
        let (healthy, quiet) = drive_until_replies_stop(small_target_config());

        assert!(healthy > 0, "a balanced session must keep minting");
        assert!(
            quiet < healthy,
            "an idle counterparty accumulates SURBs, so production must back off: healthy={healthy}/s, idle={quiet}/s"
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

        let cfg = small_target_config();
        let (mut balancer, surb_estimator, state, output) = balancer_with_feedback(cfg);
        state.set_counterparty_buffer_capacity(CAPACITY);

        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }

        // Replies stop, while production continues from a source the controller does not drive --
        // keep-alives mint on their own schedule, which is how the live estimate ran away.
        for _ in 0..20 {
            surb_estimator
                .produced
                .fetch_add(500, std::sync::atomic::Ordering::Relaxed);
            tick(&mut balancer, &surb_estimator, &output, 0);
        }

        let believed = state.buffer_level.load(std::sync::atomic::Ordering::Relaxed);
        assert!(
            believed <= CAPACITY,
            "the estimate must be bounded by the counterparty's store: believed {believed} against a {CAPACITY}-entry \
             buffer"
        );
    }

    /// The bound must not become the setpoint: a store larger than the target changes nothing.
    #[test_log::test]
    fn surb_balancer_should_leave_a_healthy_session_untouched_by_the_capacity_bound() {
        let cfg = small_target_config();
        let (mut balancer, surb_estimator, state, output) = balancer_with_feedback(cfg);
        state.set_counterparty_buffer_capacity(cfg.target_surb_buffer_size * 10);

        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }

        let level = state.buffer_level.load(std::sync::atomic::Ordering::Relaxed);
        assert!(
            level >= cfg.target_surb_buffer_size / 2,
            "a capacity well above the target must not hold the session below its setpoint: level {level}, target {}",
            cfg.target_surb_buffer_size
        );
    }

    /// Exit side, common case: with evictions counted, the reported level equals what the store
    /// actually holds -- it rises only to the real capacity during an overflow, and falls back to
    /// empty as the overflow drains, rather than staying inflated by the discarded surplus.
    ///
    /// The sibling test above bounds an eviction-blind estimate with the clamp; this one asserts the
    /// stronger property the eviction term buys: the level *tracks* occupancy instead of merely being
    /// capped by it.
    #[test_log::test]
    fn surb_balancer_level_tracks_true_occupancy_once_evictions_are_counted() {
        use std::sync::atomic::Ordering;

        const CAPACITY: u64 = 2_000;
        let cfg = small_target_config();
        let (mut balancer, est, state, _output) = balancer_with_feedback(cfg);
        state.set_counterparty_buffer_capacity(CAPACITY);

        // Overflow the store: each receive is mirrored into the estimator the way the Exit rx does --
        // the batch into `produced`, the overflow into `evicted`.
        let mut occupancy = 0u64;
        for _ in 0..10 {
            let batch = 500u64;
            let evicted = (occupancy + batch).saturating_sub(CAPACITY);
            occupancy = (occupancy + batch).min(CAPACITY);
            est.produced.fetch_add(batch, Ordering::Relaxed);
            est.evicted.fetch_add(evicted, Ordering::Relaxed);
            std::thread::sleep(Duration::from_millis(12));
            balancer.update();
        }
        assert_eq!(occupancy, CAPACITY, "the model store is full");
        assert_eq!(
            state.buffer_level.load(Ordering::Relaxed),
            CAPACITY,
            "the level equals the full store, not the {} SURBs received",
            10 * 500
        );

        // Drain the store: every spent SURB leaves the buffer.
        for _ in 0..(CAPACITY / 200) {
            est.consumed.fetch_add(200, Ordering::Relaxed);
            occupancy -= 200;
            std::thread::sleep(Duration::from_millis(12));
            balancer.update();
        }
        assert_eq!(occupancy, 0, "the model store is empty");
        assert_eq!(
            state.buffer_level.load(Ordering::Relaxed),
            0,
            "the level falls back to empty as the store drains; the evicted surplus is not held forever"
        );
    }

    /// Exit side of the capacity<target case: when the store is smaller than the target, an accurate
    /// level plateaus at the real capacity and never reaches the target. That plateau is the input the
    /// Entry infers its counterparty's capacity from (see
    /// `a_plateau_below_target_infers_the_counterparty_capacity`), which is what stops the Entry
    /// minting into a full store below its target.
    #[test_log::test]
    fn surb_balancer_level_plateaus_at_capacity_when_store_is_smaller_than_target() {
        use std::sync::atomic::Ordering;

        const CAPACITY: u64 = 500;
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 2_000,
            max_surbs_per_sec: 2_500,
            surb_decay: None,
        };
        let (mut balancer, est, state, _output) = balancer_with_feedback(cfg);
        state.set_counterparty_buffer_capacity(CAPACITY);

        let mut occupancy = 0u64;
        for _ in 0..20 {
            let batch = 200u64;
            let evicted = (occupancy + batch).saturating_sub(CAPACITY);
            occupancy = (occupancy + batch).min(CAPACITY);
            est.produced.fetch_add(batch, Ordering::Relaxed);
            est.evicted.fetch_add(evicted, Ordering::Relaxed);
            std::thread::sleep(Duration::from_millis(12));
            balancer.update();
        }

        let believed = state.buffer_level.load(Ordering::Relaxed);
        assert_eq!(occupancy, CAPACITY);
        assert_eq!(
            believed, CAPACITY,
            "the level reflects the real store, capped at its true capacity"
        );
        assert!(
            believed < cfg.target_surb_buffer_size,
            "an accurate level cannot reach a target above the store's capacity: {believed} < {}",
            cfg.target_surb_buffer_size
        );
    }

    /// A state at the given target with no capacity inferred yet, ready to receive reported levels.
    fn capacity_probe_state(target: u64) -> BalancerStateValues {
        BalancerStateValues::new(SurbBalancerConfig {
            target_surb_buffer_size: target,
            ..Default::default()
        })
    }

    /// Feed a level that repeats (a plateau) below target with no distress: after the threshold, the
    /// real capacity is inferred and organic production stops there instead of chasing an unreachable
    /// target.
    #[test]
    fn a_plateau_below_target_infers_the_counterparty_capacity() {
        use std::sync::atomic::Ordering;

        const TARGET: u64 = 2_000;
        const CAPACITY: u64 = 500;
        let state = capacity_probe_state(TARGET);

        // Fill up to the store's ceiling.
        for level in [100, 300, CAPACITY] {
            state.observe_reported_level(level);
            assert_eq!(
                1,
                state.organic_surbs_per_packet(),
                "still below capacity, keep minting"
            );
        }

        // The level now plateaus at the ceiling: the store is full below the target.
        for _ in 0..CAPACITY_PLATEAU_THRESHOLD {
            state.observe_reported_level(CAPACITY);
        }

        assert_eq!(
            CAPACITY,
            state.observed_counterparty_capacity.load(Ordering::Relaxed),
            "the plateau level is inferred as the real capacity"
        );
        assert_eq!(
            0,
            state.organic_surbs_per_packet(),
            "production stops at the real capacity, not the unreachable target"
        );
    }

    /// One flat report is churn, not a plateau: the capacity must not be inferred from it.
    #[test]
    fn a_single_flat_report_does_not_infer_capacity() {
        use std::sync::atomic::Ordering;

        let state = capacity_probe_state(2_000);
        state.observe_reported_level(500);
        state.observe_reported_level(500);

        assert_eq!(0, state.observed_counterparty_capacity.load(Ordering::Relaxed));
        assert_eq!(
            1,
            state.organic_surbs_per_packet(),
            "one flat sample must not stop production"
        );
    }

    /// A level that rises again resets the plateau: a counterparty still accepting SURBs is not full.
    #[test]
    fn a_rising_level_resets_the_plateau() {
        use std::sync::atomic::Ordering;

        let state = capacity_probe_state(2_000);
        state.observe_reported_level(500);
        state.observe_reported_level(500);
        // A rise before the threshold clears the count.
        state.observe_reported_level(600);
        state.observe_reported_level(600);

        assert_eq!(
            0,
            state.observed_counterparty_capacity.load(Ordering::Relaxed),
            "the plateau restarted at the higher level, so nothing is inferred yet"
        );
    }

    /// Distress discards an inferred capacity: a counterparty short of SURBs is not full, so the Entry
    /// must resume producing toward the full target.
    #[test]
    fn distress_discards_the_inferred_capacity() {
        use std::sync::atomic::Ordering;

        const CAPACITY: u64 = 500;
        let state = capacity_probe_state(2_000);
        for _ in 0..CAPACITY_PLATEAU_THRESHOLD + 1 {
            state.observe_reported_level(CAPACITY);
        }
        assert_eq!(CAPACITY, state.observed_counterparty_capacity.load(Ordering::Relaxed));

        state.observe_counterparty_signals(PacketSignal::SurbDistress.into());

        assert_eq!(
            0,
            state.observed_counterparty_capacity.load(Ordering::Relaxed),
            "distress clears the inferred capacity"
        );
        assert_eq!(1, state.organic_surbs_per_packet(), "and production resumes");
    }

    /// After inference, the counterparty draining and refilling toward the inferred capacity must not
    /// clear it -- only a level rising *above* the inferred capacity means the store can hold more.
    #[test]
    fn refilling_below_the_inferred_capacity_keeps_it_but_a_higher_level_clears_it() {
        use std::sync::atomic::Ordering;

        const CAPACITY: u64 = 500;
        let state = capacity_probe_state(2_000);
        for _ in 0..CAPACITY_PLATEAU_THRESHOLD + 1 {
            state.observe_reported_level(CAPACITY);
        }
        assert_eq!(CAPACITY, state.observed_counterparty_capacity.load(Ordering::Relaxed));

        // Drain and refill toward the cap: the inference stands.
        state.observe_reported_level(300);
        state.observe_reported_level(450);
        assert_eq!(
            CAPACITY,
            state.observed_counterparty_capacity.load(Ordering::Relaxed),
            "a refill up to the inferred capacity does not forget it"
        );

        // A level above the inferred capacity means the store grew: re-probe.
        state.observe_reported_level(CAPACITY + 200);
        assert_eq!(
            0,
            state.observed_counterparty_capacity.load(Ordering::Relaxed),
            "a level above the inferred capacity forgets it"
        );
    }

    /// Reproduces the balancer state seen in the `gnosis_vpn-20260824-101952.log` outage (India
    /// exit `0x1062..8106`): under sustained return-path loss the entry's counterparty-buffer
    /// estimate (`produced - consumed`, clamped to the store) pins at/above target because
    /// `consumed` stops advancing -- the replies that would retire it never arrive -- and the PID
    /// winds production down to zero. The live `surb balancer state` lines read
    /// `level=15000 target=9766 output=0 degraded=false` for the whole 15-minute session.
    ///
    /// Scope: this guards only the entry-side balancer behaviour the logs show directly -- the
    /// estimate pinning and the output collapse. It deliberately makes no claim about the exit's
    /// real SURB store or whether the counterparty starves. The estimate is not a proxy for that
    /// store: `update` documents that `produced - consumed` "reads as a filling buffer precisely
    /// when it is emptying" during return-path loss, so a pinned `level` says nothing about how
    /// many SURBs the exit actually holds. Whether the throttle-to-zero contributes to the outage
    /// is a separate question this test does not address; it only nails down the observed state so
    /// a change in it is noticed.
    #[test_log::test]
    fn surb_balancer_pins_estimate_and_collapses_output_under_return_path_loss() {
        use std::sync::atomic::Ordering;

        // The live 15 000-entry counterparty store the estimate is clamped to.
        const COUNTERPARTY_CAPACITY: u64 = 15_000;

        // The configuration a gnosis_vpn tunnel session actually ran with: SURB balancing on.
        let cfg = small_target_config();
        let (mut balancer, surb_estimator, state, output) = balancer_with_feedback(cfg);
        state.set_counterparty_buffer_capacity(COUNTERPARTY_CAPACITY);

        // Healthy stretch: replies arrive, and the buffer holds at the setpoint.
        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }

        // Replies stop arriving (`consumed` no longer advances). The entry keeps minting what its
        // controller last commanded; with `consumed` frozen the estimate climbs to the store cap
        // and the PID drives the commanded rate down to zero.
        let step = Duration::from_millis(50);
        for _ in 0..60 {
            std::thread::sleep(step);
            let minted = output.load(Ordering::Relaxed) * step.as_millis() as u64 / 1000;
            surb_estimator.produced.fetch_add(minted, Ordering::Relaxed);
            // `consumed` stays flat: nothing comes back over the dead return path.
            balancer.update();
        }

        // The field `level >= target`: the estimate is pinned at/above target.
        assert!(
            state.buffer_level.load(Ordering::Relaxed) >= cfg.target_surb_buffer_size,
            "the counterparty-buffer estimate must pin at/above target while replies are lost, reproducing the field \
             `level >= target` state: level={}",
            state.buffer_level.load(Ordering::Relaxed)
        );

        // The field `output=0`: production has collapsed to zero.
        assert_eq!(
            output.load(Ordering::Relaxed),
            0,
            "production must collapse to zero once the estimate pins above target, reproducing the field `output=0` \
             state"
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

    /// SURBs through a relayer judged dead are held, and reported, but cannot bring a reply home,
    /// so a counterparty above target on paper is below it in fact.
    #[test]
    fn organic_surbs_should_be_produced_while_invalid_surbs_hide_a_shortfall() {
        let state = gate_state(100, 150);
        assert_eq!(0, state.organic_surbs_per_packet(), "precondition: above target");
        state.set_invalid_surbs(80, 0.9);
        assert_eq!(1, state.organic_surbs_per_packet(), "80 of 150 invalid");
        state.set_invalid_surbs(80, 0.2);
        assert_eq!(0, state.organic_surbs_per_packet(), "the share caps it at 30");
        state.set_invalid_surbs(10_000, 0.5);
        assert_eq!(
            1,
            state.organic_surbs_per_packet(),
            "the count no more than the share of 75"
        );
        assert_eq!(150, state.buffer_level(), "the held level itself is unchanged");
        state.set_invalid_surbs(u64::MAX, 1.0);
        assert_eq!(1, state.organic_surbs_per_packet(), "a saturated count writes off all");
        state.set_invalid_surbs(0, 0.0);
        assert_eq!(0, state.organic_surbs_per_packet(), "clearing restores the held level");
    }

    type TestBalancer = SurbBalancer<PidBalancerController, AtomicSurbFlowEstimator, MockSurbFlowController>;

    /// One sampling interval of `dt`, without waiting for it: the balancer's clock is wound back instead.
    fn tick_for(balancer: &mut TestBalancer, dt: Duration) {
        balancer.last_update = std::time::Instant::now()
            .checked_sub(dt)
            .expect("monotonic clock too young");
        balancer.update();
    }

    /// The configuration of a session at the client's usual target, against the usual mint ceiling.
    fn paced_refill_config() -> SurbBalancerConfig {
        SurbBalancerConfig {
            target_surb_buffer_size: 7_000,
            max_surbs_per_sec: 5_000,
            surb_decay: None,
        }
    }

    /// A wipe means the counterparty holds nothing, whatever the estimate says, and the estimate must
    /// say so at the next update: not the update that was already running, nor the production that was
    /// in flight when the SURB generation advanced, and not a report from the counterparty that has
    /// not seen the new generation yet may bring it back.
    #[test_log::test]
    fn a_wipe_should_restart_the_estimate_from_empty_and_survive_the_next_tick() {
        use std::sync::atomic::Ordering::Relaxed;

        let (mut balancer, estimator, state, _output) = balancer_with_feedback(paced_refill_config());
        let step = Duration::from_millis(50);

        state.observe_reported_level(7_000);
        tick_for(&mut balancer, step);
        assert_eq!(
            7_000,
            state.buffer_level(),
            "precondition: the counterparty is at target"
        );

        // 300 SURBs were minted for the old generation just before it advanced.
        estimator.produced.fetch_add(300, Relaxed);
        state.on_counterparty_wiped();
        assert_eq!(0, state.buffer_level(), "the level is reset at once");

        // The counterparty reports what it held until its first SURB of the new generation lands.
        state.observe_reported_level(7_000);
        assert_eq!(0, state.buffer_level(), "a report above the believed level is dropped");

        // An update that was already running writes back the level it loaded before the reset.
        state.buffer_level.store(7_000, Relaxed);
        tick_for(&mut balancer, step);
        assert_eq!(
            0,
            state.buffer_level(),
            "the stale level and the old generation's production must not survive the first update"
        );

        // From then on the estimate counts what is minted for the new generation, and is not reset again.
        estimator.produced.fetch_add(200, Relaxed);
        tick_for(&mut balancer, step);
        assert_eq!(200, state.buffer_level());
        estimator.consumed.fetch_add(50, Relaxed);
        tick_for(&mut balancer, step);
        assert_eq!(150, state.buffer_level(), "the reset is one-shot");
    }

    /// The counterparty keeps reporting the stock it had until the first SURB of the new generation
    /// reaches it. A report that lands after the first update, when the level is already rebuilding,
    /// must not put the level back at the target, which would shut the organic gate and silence the PID
    /// while the counterparty holds nothing. Reports at or below what is believed are legitimate, and
    /// everything is taken again once the paced window is over.
    #[test_log::test]
    fn a_stale_report_after_the_first_tick_should_not_restore_the_wiped_level() {
        use std::sync::atomic::Ordering::Relaxed;

        let (mut balancer, estimator, state, _output) = balancer_with_feedback(paced_refill_config());
        let step = Duration::from_millis(50);

        state.observe_reported_level(7_000);
        tick_for(&mut balancer, step);
        state.on_counterparty_wiped();
        tick_for(&mut balancer, step);
        assert_eq!(0, state.buffer_level(), "precondition: restarted from empty");

        estimator.produced.fetch_add(200, Relaxed);
        tick_for(&mut balancer, step);
        assert_eq!(
            200,
            state.buffer_level(),
            "precondition: 200 SURBs of the new generation sent"
        );

        state.observe_reported_level(7_000);
        assert_eq!(
            200,
            state.buffer_level(),
            "the pre-wipe level is dropped for the window"
        );
        assert_eq!(
            1,
            state.organic_surbs_per_packet(),
            "the organic gate stays open while the counterparty holds almost nothing"
        );

        state.observe_reported_level(150);
        assert_eq!(
            150,
            state.buffer_level(),
            "a report at or below the belief is legitimate"
        );

        // Past the window (5 s of 50 ms ticks, and then some) a report is taken as it comes.
        for _ in 0..110 {
            tick_for(&mut balancer, step);
        }
        state.observe_reported_level(7_000);
        assert_eq!(
            7_000,
            state.buffer_level(),
            "after the window the counterparty is believed again"
        );
    }

    /// What a freshly built controller commands for `level`: the answer the first tick after the pacing
    /// window has to give, and the one a wound-up integral would corrupt.
    fn fresh_controller_output(cfg: SurbBalancerConfig, level: u64) -> u64 {
        let mut controller = PidBalancerController::default();
        controller.set_target_and_limit(cfg.as_controller_bounds());
        controller.next_control_output(level)
    }

    /// After a wipe the PID would command its output limit until the level is back. The refill is held
    /// to a fifth of the target per second for five seconds, and left alone after that.
    #[test_log::test]
    fn a_wipe_should_cap_output_for_the_pacing_window() {
        let cfg = paced_refill_config();
        let (mut balancer, _estimator, state, output) = balancer_with_feedback(cfg);
        let step = Duration::from_millis(50);
        let cap = cfg.target_surb_buffer_size / REFILL_PACING_SECS;
        assert_eq!(1_400, cap);

        state.on_counterparty_wiped();

        // Nothing is minted, so the level stays empty and the PID asks for everything it may.
        let mut outputs = Vec::new();
        for _ in 0..120 {
            tick_for(&mut balancer, step);
            outputs.push(output.load(std::sync::atomic::Ordering::Relaxed));
        }

        // Margins of half a second either side of the five-second window, so that a slow tick cannot
        // move the edge into the assertion.
        let window = (REFILL_PACING_SECS * 1000 / step.as_millis() as u64) as usize;
        assert!(
            outputs[..window - 10].iter().all(|output| *output == cap),
            "the output must be the cap, {cap}, for the window: {outputs:?}"
        );
        assert!(
            outputs[window + 10..].iter().all(|output| *output > cap),
            "after the window the controller runs free: {outputs:?}"
        );

        // The first free tick is a fresh controller's answer to the level, not an accumulated one. At
        // an empty level that is as much as the controller may command: one tick of its proportional
        // and integral terms already exceeds the limit.
        let first_free = outputs
            .iter()
            .position(|output| *output != cap)
            .expect("the window ends");
        assert_eq!(fresh_controller_output(cfg, 0), outputs[first_free]);
    }

    /// The PID does not run while the refill is paced, so it integrates nothing. Whatever it had
    /// accumulated before the wipe (here, an integral wound up to its limit by a stretch at an empty
    /// level) is dropped when the window ends, and the first free tick answers the level the refill
    /// reached with a fresh controller's output. An integral carried through would answer the same
    /// level with the ceiling, and a level at or past the target with a burst.
    #[test_log::test]
    fn the_first_tick_after_the_pacing_window_should_start_from_the_current_level() {
        use std::sync::atomic::Ordering::Relaxed;

        let cfg = paced_refill_config();
        let (mut balancer, estimator, state, output) = balancer_with_feedback(cfg);
        let step = Duration::from_millis(50);

        // The PID winds its integral up against an empty counterparty.
        for _ in 0..20 {
            tick_for(&mut balancer, step);
        }
        assert_eq!(
            cfg.max_surbs_per_sec,
            output.load(Relaxed),
            "precondition: at the ceiling"
        );

        // A wipe, and a refill that reaches 80 % of the target inside the window.
        state.on_counterparty_wiped();
        for _ in 0..20 {
            tick_for(&mut balancer, step);
        }
        let level = cfg.target_surb_buffer_size * 4 / 5;
        estimator.produced.fetch_add(level, Relaxed);
        while state.refill_pacing_active() {
            tick_for(&mut balancer, step);
            assert!(
                state.refill_pacing_active() || state.buffer_level() == level,
                "the level is the refill's"
            );
        }

        tick_for(&mut balancer, step);
        let first_free = output.load(Relaxed);
        assert_eq!(fresh_controller_output(cfg, level), first_free);
        assert!(
            first_free < cfg.max_surbs_per_sec,
            "a level at 80 % of the target is not met with the ceiling: {first_free}"
        );
    }

    /// A session that has just been opened is in the state a wipe leaves: the counterparty holds nothing
    /// and the PID would command its ceiling at once. The Entry arms the same pacing when its balancer
    /// starts, and the output is the cap, a fifth of the target, for the whole window, whatever the PID
    /// would say; then a fresh controller answers the level the fill reached. Nothing is reset on the
    /// way in: what was minted before the first tick is the new session's.
    #[test_log::test]
    fn a_fresh_entry_balancer_should_pace_its_initial_fill() {
        use std::sync::atomic::Ordering::Relaxed;

        let cfg = paced_refill_config();
        let (mut balancer, estimator, state, output) = balancer_with_feedback(cfg);
        let step = Duration::from_millis(50);
        let cap = cfg.target_surb_buffer_size / REFILL_PACING_SECS;
        assert_eq!(1_400, cap);
        assert!(!state.refill_pacing_active(), "precondition: a new state paces nothing");

        // What the Entry does as its balancer starts; some SURBs are already on their way.
        state.arm_refill_pacing();
        estimator.produced.fetch_add(100, Relaxed);

        let mut outputs = Vec::new();
        tick_for(&mut balancer, step);
        outputs.push(output.load(Relaxed));
        assert_eq!(100, state.buffer_level(), "starting paced does not reset the estimate");

        // The fill reaches 80 % of the target inside the window; the PID, if it ran, would still ask for
        // everything it may.
        let level = cfg.target_surb_buffer_size * 4 / 5;
        estimator.produced.fetch_add(level - 100, Relaxed);
        while state.refill_pacing_active() {
            tick_for(&mut balancer, step);
            outputs.push(output.load(Relaxed));
            assert!(outputs.len() < 200, "the window must end");
        }

        // The window is five seconds of 50 ms ticks; margins of half a second either side, so that a slow
        // tick cannot move the edge into the assertion.
        let window = (REFILL_PACING_SECS * 1000 / step.as_millis() as u64) as usize;
        assert!(
            outputs.len().abs_diff(window) <= 10,
            "the window lasts about {window} ticks: {}",
            outputs.len()
        );
        assert!(
            outputs.iter().all(|output| *output == cap),
            "the output is the cap, {cap}, on every tick of the window: {outputs:?}"
        );

        tick_for(&mut balancer, step);
        let first_free = output.load(Relaxed);
        assert_eq!(fresh_controller_output(cfg, level), first_free);
        assert!(
            first_free < cfg.max_surbs_per_sec,
            "a level at 80 % of the target is not met with the ceiling: {first_free}"
        );
    }

    /// Arming does nothing for a balancer that is switched off.
    #[test]
    fn arming_the_pacing_of_a_disabled_balancer_should_do_nothing() {
        let state = BalancerStateValues::default();
        state.arm_refill_pacing();
        assert!(!state.refill_pacing_active());
    }

    /// The refill stops by itself the moment the target is reached, and stays stopped until the window
    /// ends, when the controller, fresh, answers a level at the target with nothing.
    #[test_log::test]
    fn pacing_should_stop_the_refill_once_the_target_is_reached_inside_the_window() {
        use std::sync::atomic::Ordering::Relaxed;

        let cfg = paced_refill_config();
        let (mut balancer, estimator, state, output) = balancer_with_feedback(cfg);
        let step = Duration::from_millis(50);
        let cap = cfg.target_surb_buffer_size / REFILL_PACING_SECS;

        state.on_counterparty_wiped();
        for _ in 0..60 {
            tick_for(&mut balancer, step);
            assert_eq!(cap, output.load(Relaxed), "refilling at the cap while below the target");
        }

        // Three seconds in, the refill has delivered the target.
        estimator.produced.fetch_add(cfg.target_surb_buffer_size, Relaxed);
        let mut after_target = Vec::new();
        for _ in 0..30 {
            tick_for(&mut balancer, step);
            after_target.push(output.load(Relaxed));
        }
        assert_eq!(cfg.target_surb_buffer_size, state.buffer_level());
        assert!(
            after_target.iter().all(|output| *output == 0),
            "nothing more is commanded once the target is reached, inside the window or at its end: {after_target:?}"
        );

        // And on past the window's end, which the level holds at the target through.
        while state.refill_pacing_active() {
            tick_for(&mut balancer, step);
            assert_eq!(0, output.load(Relaxed));
        }
        tick_for(&mut balancer, step);
        assert_eq!(
            0,
            output.load(Relaxed),
            "the first free tick at the target commands nothing"
        );
    }

    /// The pacing is a rate the balancer commands, not a bound. The PID is rebuilt whenever the state's
    /// bounds differ from its own, which would discard its integral, so the pacing must not be
    /// implemented as a bound.
    #[test_log::test]
    fn pacing_must_not_reconfigure_the_controller_bounds() {
        let cfg = paced_refill_config();
        let (mut balancer, _estimator, state, _output) = balancer_with_feedback(cfg);
        let bounds = state.controller_bounds();
        assert_eq!(bounds, balancer.controller.bounds(), "precondition: in step");

        state.on_counterparty_wiped();
        for _ in 0..120 {
            tick_for(&mut balancer, Duration::from_millis(50));
            assert_eq!(
                bounds,
                state.controller_bounds(),
                "the state's bounds are what the control loop compares the controller's against"
            );
            assert_eq!(bounds, balancer.controller.bounds());
        }
        assert_eq!(cfg, state.as_config());
    }

    /// With no target there is no balancing, and nothing to hold back.
    #[test]
    fn a_wipe_of_a_disabled_balancer_should_arm_no_pacing() {
        let state = BalancerStateValues::default();
        state.on_counterparty_wiped();
        assert_eq!(None, state.pace_refill(Duration::from_millis(50)));
    }

    // -----------------------------------------------------------------------
    // Sustaining the counterparty's SURBs through a silence
    // -----------------------------------------------------------------------

    /// The sustain is about data going out into a silence. Without a packet sent after the last time the
    /// counterparty was heard, more than the grace later, there is nothing to sustain, and neither is
    /// there before it was heard at all, or while there is no balancer.
    #[test]
    fn silence_without_outgoing_data_should_not_sustain() {
        let state = gate_state(100, 200).last_heard_ago(Duration::from_secs(10));
        assert!(
            !state.silence_sustain(),
            "heard 10 s ago, and nothing has been sent since"
        );

        // A packet that goes out inside the grace after the last time it was heard is not a silence yet.
        state.stamp(&state.last_sent, std::time::Instant::now() - Duration::from_secs(8));
        assert!(!state.silence_sustain(), "sent 2 s after it was heard");

        let never_heard = gate_state(100, 200);
        never_heard.note_sent();
        assert!(
            !never_heard.silence_sustain(),
            "nothing was ever heard, so nothing has gone quiet"
        );

        let disabled = gate_state(0, 0).last_heard_ago(Duration::from_secs(10));
        disabled.note_sent();
        assert!(!disabled.silence_sustain(), "no balancer, no sustain");

        let silent = gate_state(100, 200).last_heard_ago(Duration::from_secs(10));
        silent.note_sent();
        assert!(
            silent.silence_sustain(),
            "a packet 10 s after the last word is a silence"
        );
    }

    /// Whatever the Exit says is heard: a report of its level, and any packet at all, whose signals are
    /// all the Entry reads of it.
    #[test]
    fn a_report_or_any_packet_from_the_counterparty_should_count_as_heard() {
        let state = gate_state(100, 200);
        assert_eq!(
            0,
            state.last_heard.load(std::sync::atomic::Ordering::Relaxed),
            "nothing heard yet"
        );

        state.observe_counterparty_signals(PacketSignals::default());
        let after_a_packet = state.last_heard.load(std::sync::atomic::Ordering::Relaxed);
        assert!(after_a_packet > 0, "a packet from the Exit");

        let state = gate_state(100, 200);
        state.observe_reported_level(50);
        assert!(
            state.last_heard.load(std::sync::atomic::Ordering::Relaxed) > 0,
            "a level report"
        );
    }

    /// A reply home is the Exit answering, counted by the estimator as a SURB consumed, even where none of
    /// its packets reached the hooks that record it. Production minted and nothing consumed is not.
    #[test_log::test]
    fn a_reply_counted_as_consumed_should_count_as_heard() {
        let (mut balancer, estimator, state, output) = balancer_with_feedback(small_target_config());
        let heard = || state.last_heard.load(std::sync::atomic::Ordering::Relaxed);

        for _ in 0..5 {
            tick(&mut balancer, &estimator, &output, 0);
        }
        assert_eq!(0, heard(), "SURBs minted and none consumed: nobody has answered");

        tick(&mut balancer, &estimator, &output, 3);
        assert!(heard() > 0, "three replies came home in that tick");
    }
}
