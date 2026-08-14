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
    pub counterparty_buffer_capacity: AtomicU64,
    /// Milliseconds from [`EPOCH`] until which the return path counts as degraded.
    ///
    /// A deadline rather than a flag: it is set by a layer that observes the return path and read
    /// here, and nothing is guaranteed to come back and clear it. Expiring on its own bounds the
    /// damage of a marker that is never withdrawn to a short over-production instead of a session
    /// that mints forever.
    pub return_path_degraded_until_ms: AtomicU64,
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

    /// Marks the return path as degraded for the next `grace` period.
    ///
    /// Called by whichever layer can actually tell a dead return path from a quiet peer -- from
    /// here the two are indistinguishable, since neither delivers replies. Re-marking simply
    /// extends the window.
    pub fn mark_return_path_degraded(&self, grace: Duration) {
        let until = EPOCH.elapsed().saturating_add(grace).as_millis().min(u64::MAX as u128) as u64;
        self.return_path_degraded_until_ms
            .fetch_max(until, std::sync::atomic::Ordering::Relaxed);
    }

    /// Whether production should currently ignore the counterparty buffer estimate.
    ///
    /// Both the opt-in and live evidence are required: without the opt-in this is not our
    /// behaviour to change, and without evidence there is nothing to distinguish a dead return path
    /// from an idle one.
    fn should_sustain_through_return_path_loss(&self) -> bool {
        let deadline = self
            .return_path_degraded_until_ms
            .load(std::sync::atomic::Ordering::Relaxed);

        // Zero is "never marked", not "marked at the epoch" -- otherwise every session that opted
        // in would start out believing its return path was already dead.
        deadline > 0
            && (EPOCH.elapsed().as_millis() as u64) < deadline
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
            was_degraded: false,
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
            tracing::debug!(
                believed,
                capacity = current,
                "counterparty SURB estimate exceeded its store; the surplus was never held"
            );
        }

        let degraded = self.state.should_sustain_through_return_path_loss();
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
                self.last_decay = std::time::Instant::now();
                tracing::debug!("return path recovered; restarting closed-loop SURB control");
            }
        }

        if degraded {
            // While replies are being lost there is no valid estimate to act on: every SURB the
            // counterparty spends is invisible from here, so the accumulated `produced - consumed`
            // reads as a filling buffer precisely when it is emptying. Drop to open loop and assume
            // the worst, which drives production to the maximum until replies resume.
            tracing::debug!(
                believed = current,
                "return path degraded; ignoring the counterparty buffer estimate"
            );
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

        let output = self.controller.next_control_output(current);
        tracing::trace!(output, "next balancer control output for session");

        // Both ends run this same loop -- the Entry with the PID driving production, the Exit with
        // the proportional controller gating egress -- so one line covers both and the session id
        // tells them apart. Rate-limited to one per second so it can run under a full-rate session.
        if self.last_report.elapsed() >= Duration::from_secs(1) {
            self.last_report = std::time::Instant::now();
            tracing::info!(
                session = %self.session_id,
                level = current,
                target = self.controller.bounds().target(),
                output,
                produced = self.surb_estimator.estimate_surbs_produced(),
                consumed = self.surb_estimator.estimate_surbs_consumed(),
                degraded,
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
    /// Returns the control output at the end of each stretch. The two phases are deliberately
    /// indistinguishable from inside the balancer -- consumption simply stops -- which is the whole
    /// point: only the caller's `sustain` choice separates a dead return path from an idle peer.
    fn drive_until_replies_stop(cfg: SurbBalancerConfig, mark_degraded: bool) -> (u64, u64) {
        let (mut balancer, surb_estimator, state, output) = balancer_with_feedback(cfg);

        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }
        let healthy = output.load(std::sync::atomic::Ordering::Relaxed);

        if mark_degraded {
            state.mark_return_path_degraded(Duration::from_secs(30));
        }

        // Replies stop while production continues.
        for _ in 0..20 {
            tick(&mut balancer, &surb_estimator, &output, 0);
        }

        (healthy, output.load(std::sync::atomic::Ordering::Relaxed))
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

    /// Both edges of a degraded window: open loop must engage at once, and let go afterwards.
    #[test_log::test]
    fn surb_balancer_should_return_to_closed_loop_when_the_return_path_recovers() {
        let (mut balancer, surb_estimator, state, output) = balancer_with_feedback(sustaining_config(true));

        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }
        let healthy = output.load(std::sync::atomic::Ordering::Relaxed);

        state.mark_return_path_degraded(Duration::from_millis(500));
        tick(&mut balancer, &surb_estimator, &output, 0);
        let first_degraded = output.load(std::sync::atomic::Ordering::Relaxed);

        for _ in 0..9 {
            tick(&mut balancer, &surb_estimator, &output, 0);
        }

        // The mark lapses and the counterparty starts answering again.
        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }
        let recovered = output.load(std::sync::atomic::Ordering::Relaxed);

        assert!(
            first_degraded > healthy,
            "open loop must engage on the first update after the mark, not ramp towards it: healthy={healthy}/s, \
             first degraded update={first_degraded}/s"
        );
        assert!(
            recovered < first_degraded,
            "once replies are arriving again the controller must return to closed loop rather than stay pinned at \
             maximum: degraded={first_degraded}/s, recovered={recovered}/s"
        );
    }

    /// After the outage the counterparty must be refilled, not merely un-throttled.
    ///
    /// Returning to closed loop is only half the claim: production has to actually climb the curve
    /// again and restore the buffer. Resetting the controller is what makes that prompt -- the error
    /// accumulated while the estimate was meaningless would otherwise have to be unwound first.
    #[test_log::test]
    fn surb_balancer_should_refill_the_counterparty_after_the_return_path_recovers() {
        let cfg = sustaining_config(true);
        let (mut balancer, surb_estimator, state, output) = balancer_with_feedback(cfg);

        for _ in 0..40 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
        }
        // A band, not the setpoint itself: the controller oscillates around its target, so a
        // sample taken at an arbitrary tick legitimately sits either side of it.
        let refilled = cfg.target_surb_buffer_size / 2;
        assert!(
            state.buffer_level.load(std::sync::atomic::Ordering::Relaxed) >= refilled,
            "the healthy phase must reach the setpoint band before an outage means anything"
        );

        // The return path dies: replies stop, and open loop takes over.
        state.mark_return_path_degraded(Duration::from_millis(400));
        for _ in 0..8 {
            tick(&mut balancer, &surb_estimator, &output, 0);
        }

        // The mark lapses, the counterparty answers again, and the belief restarts from empty.
        let mut ticks_to_refill = None;
        for n in 1..=60 {
            tick(&mut balancer, &surb_estimator, &output, REPLIES_PER_TICK);
            if state.buffer_level.load(std::sync::atomic::Ordering::Relaxed) >= refilled {
                ticks_to_refill = Some(n);
                break;
            }
        }

        let ticks = ticks_to_refill.expect("the counterparty must be refilled to the setpoint after recovery");
        tracing::info!(ticks, "refilled the counterparty after the outage");

        // Each tick is one sampling interval; the balancer samples far more often than this in a
        // live Session, so a bound in ticks is a bound in sampling intervals, not in wall clock.
        assert!(
            ticks <= 30,
            "refilling must ramp rather than crawl: took {ticks} sampling intervals"
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

        let cfg = sustaining_config(false);
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
        let cfg = sustaining_config(false);
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
}
