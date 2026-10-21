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
    /// SURBs, counted and as a share (`f64` bits) of what is outstanding, returning through a relayer
    /// judged dead: held and reported by the counterparty, but unable to bring a reply home.
    counterparty_unusable: AtomicU64,
    counterparty_unusable_share: AtomicU64,
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
    /// ## Tradeoff left to a follow-up
    ///
    /// With an accurate level, a counterparty whose store is *smaller* than the target reports a level
    /// that plateaus at its real capacity and never reaches target, so organic production does not stop
    /// and the full buffer keeps evicting. This is unreachable with homogeneous configuration
    /// (`target <= 2/3` of the store); it needs the Entry's store to exceed ~1.5x the Exit's. The
    /// intended close is a distress-based capacity guesstimate on the Entry -- inferring the real
    /// capacity from where the reported level plateaus, with `SurbDistress` as the corrective -- not a
    /// return to an inflated estimate.
    pub counterparty_buffer_capacity: AtomicU64,
    /// Whether the counterparty's last packet said it was running low on SURBs of its own.
    ///
    /// A plain flag with no deadline: a distress flag that is never withdrawn merely keeps organic
    /// production at one SURB per packet — the behaviour that predates the gate. A flag whose stuck
    /// state is the old behaviour does not need to expire.
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
                || self.usable_level(self.buffer_level())
                    < self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
        )
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

    pub fn set_counterparty_unusable(&self, count: u64, share: f64) {
        self.counterparty_unusable
            .store(count, std::sync::atomic::Ordering::Relaxed);
        self.counterparty_unusable_share
            .store(share.clamp(0.0, 1.0).to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    /// The part of `level` that can still bring a reply home.
    fn usable_level(&self, level: u64) -> u64 {
        let share = f64::from_bits(
            self.counterparty_unusable_share
                .load(std::sync::atomic::Ordering::Relaxed),
        );
        // The count misses SURBs older than we track; the share overstates when spent ones linger.
        let count = self.counterparty_unusable.load(std::sync::atomic::Ordering::Relaxed);
        level.saturating_sub(count.min((level as f64 * share).round() as u64))
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

        // Take a snapshot of the active SURB estimator and calculate the balance change. The change is
        // one coherent subtraction over the net held count (a single atomic), so it never reads
        // non-monotonically and a negative result is a legitimate drain.
        let snapshot = SimpleSurbFlowEstimator::from(&self.surb_estimator);
        let target_buffer_change = snapshot.estimated_surb_buffer_change(&self.last_estimator_state);

        self.last_estimator_state = snapshot;
        current = current.saturating_add_signed(target_buffer_change);

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

        // Control on what can still bring a reply home; the stored level stays what the counterparty
        // holds, so its own reports keep matching it.
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

        let output = self.controller.next_control_output(usable);
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
                unusable = current - usable,
                target = self.controller.bounds().target(),
                output,
                produced = self.surb_estimator.estimate_surbs_produced(),
                consumed = self.surb_estimator.estimate_surbs_consumed(),
                distress = self
                    .state
                    .counterparty_in_surb_distress
                    .load(std::sync::atomic::Ordering::Relaxed),
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

    /// SURBs that return through a dead relayer are held, and reported, but cannot bring a reply home,
    /// so a counterparty above target on paper is below it in fact.
    #[test]
    fn organic_surbs_should_be_produced_while_unusable_surbs_hide_a_shortfall() {
        let state = gate_state(100, 150);
        assert_eq!(0, state.organic_surbs_per_packet(), "precondition: above target");
        state.set_counterparty_unusable(80, 0.9);
        assert_eq!(1, state.organic_surbs_per_packet(), "80 of 150 unusable");
        state.set_counterparty_unusable(80, 0.2);
        assert_eq!(0, state.organic_surbs_per_packet(), "the share caps it at 30");
        state.set_counterparty_unusable(10_000, 0.5);
        assert_eq!(
            1,
            state.organic_surbs_per_packet(),
            "the count no more than the share of 75"
        );
        assert_eq!(150, state.buffer_level(), "the held level itself is unchanged");
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
            surb_estimator
                .record_produced(production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs());
            surb_estimator.record_consumed(consumption_rate * step_duration.as_secs());

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
            surb_estimator
                .record_produced(production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs());
            surb_estimator.record_consumed(consumption_rate * step_duration.as_secs());

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
        surb_estimator.record_produced(minted);
        surb_estimator.record_consumed(consumed);
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
            surb_estimator.record_produced(500);
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

    /// Feed `iters` receives of `batch` SURBs into a modelled store of capacity `cap`, mirroring the
    /// Exit rx (batch into `produced`, overflow into `evicted`) and ticking the balancer after each.
    /// Advances `occupancy` to the resulting level. One source of truth for the overflow model the
    /// eviction-aware tests share.
    fn overflow_fill(
        balancer: &mut SurbBalancer<PidBalancerController, AtomicSurbFlowEstimator, MockSurbFlowController>,
        est: &AtomicSurbFlowEstimator,
        occupancy: &mut u64,
        cap: u64,
        batch: u64,
        iters: usize,
    ) {
        for _ in 0..iters {
            let evicted = (*occupancy + batch).saturating_sub(cap);
            *occupancy = (*occupancy + batch).min(cap);
            est.record_incoming(batch, evicted);
            std::thread::sleep(Duration::from_millis(12));
            balancer.update();
        }
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

        let mut occupancy = 0u64;
        overflow_fill(&mut balancer, &est, &mut occupancy, CAPACITY, 500, 10);
        assert_eq!(occupancy, CAPACITY, "the model store is full");
        assert_eq!(
            state.buffer_level.load(Ordering::Relaxed),
            CAPACITY,
            "the level equals the full store, not the {} SURBs received",
            10 * 500
        );

        // Drain the store: every spent SURB leaves the buffer.
        for _ in 0..(CAPACITY / 200) {
            est.record_consumed(200);
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

    /// Exit side of the documented capacity<target tradeoff: when the store is smaller than the
    /// target, an accurate level plateaus at the real capacity and never reaches the target. This is
    /// what leaves the Entry minting forever (pinned from the Entry side in `manager.rs`), and is the
    /// behavior the follow-up distress-based capacity guesstimate is meant to close.
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
        overflow_fill(&mut balancer, &est, &mut occupancy, CAPACITY, 200, 20);

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

    /// CR-1 resolved: a delivery that evicts moves the held count by one coherent step, so a small
    /// believed level cannot be over-counted by a torn read. Previously the eviction and its matching
    /// production landed in separate atomics; a snapshot taken between them read the change too negative,
    /// `saturating_add_signed` clamped the surplus away at zero, and the next tick's positive delta
    /// telescoped only the production back -- settling the level above true occupancy. With the held
    /// count behind a single atomic there is no such window: a packet delivering 10 into a store that
    /// evicts 8, on a level of 3, lands at exactly 5.
    #[test_log::test]
    fn a_delivery_that_evicts_is_accounted_coherently_on_a_small_level() {
        use std::sync::atomic::Ordering::Relaxed;

        // No decay and an unbounded capacity clamp (default 0), so only the change under test moves
        // the level.
        let cfg = SurbBalancerConfig {
            target_surb_buffer_size: 1_000,
            max_surbs_per_sec: 2_500,
            surb_decay: None,
        };
        let (mut balancer, est, state, _output) = balancer_with_feedback(cfg);

        // Baseline: last_estimator_state becomes net=1000.
        est.record_produced(1000);
        std::thread::sleep(Duration::from_millis(12));
        balancer.update();

        // The level has since drained to a small value.
        state.buffer_level.store(3, Relaxed);

        // A packet delivers 10 SURBs into the full store, 8 of which evict older ones: one coherent
        // net change of +2, with no window in which the eviction is visible without the production.
        est.record_incoming(10, 8);
        std::thread::sleep(Duration::from_millis(12));
        balancer.update();

        // Net change is +2 (10 delivered - 8 evicted); on a level of 3 that is 5.
        assert_eq!(
            5,
            state.buffer_level.load(Relaxed),
            "the delivery net of eviction must be accounted exactly; the level must not over-count"
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
}
