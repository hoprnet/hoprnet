use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64},
    },
    time::Duration,
};

use futures::{StreamExt, pin_mut};
use hopr_async_runtime::AbortHandle;
use tracing::{Instrument, instrument};

use super::{
    BalancerControllerBounds, MIN_BALANCER_SAMPLING_INTERVAL, SimpleSurbFlowEstimator, SurbBalancerController,
    SurbFlowController, SurbFlowEstimator,
};
use crate::SessionId;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TARGET_ERROR_ESTIMATE: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new(
            "hopr_surb_balancer_target_error_estimate",
            "Target error estimation by the SURB balancer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CONTROL_OUTPUT: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new(
            "hopr_surb_balancer_control_output",
            "Control output of the SURB balancer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CURRENT_BUFFER: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new(
            "hopr_surb_balancer_current_buffer_estimate",
            "Estimated number of SURBs in the buffer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CURRENT_TARGET: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new(
            "hopr_surb_balancer_current_buffer_target",
            "Current target (setpoint) number of SURBs in the buffer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_SURB_RATE: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new(
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
    /// The local buffer is maintained by [regulating](SurbFlowController) the egress from the Session.
    /// The remote buffer (at session counterparty) is maintained by regulating the flow of non-organic SURBs via
    /// [keep-alive messages](crate::initiation::StartProtocol::KeepAlive).
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
    /// The [`SurbBalancer`] will discard the given percentage of `target_surb_buffer_size` at each
    /// window with the given `Duration`.
    ///
    /// The default is `(60, 0.05)` (5% of the target buffer size is discarded every 60 seconds).
    #[default(_code = "Some((Duration::from_secs(60), 0.05))")]
    pub surb_decay: Option<(Duration, f64)>,
}

impl SurbBalancerConfig {
    /// Convenience function to convert the [`SurbBalancerConfig`] into [`BalancerControllerBounds`].
    #[inline]
    pub fn as_controller_bounds(&self) -> BalancerControllerBounds {
        BalancerControllerBounds::new(self.target_surb_buffer_size, self.max_surbs_per_sec)
    }
}

/// Runtime state of the [`SurbBalancer`].
#[derive(Debug, Default)]
pub struct BalancerStateData {
    pub target_surb_buffer_size: AtomicU64,
    pub max_surbs_per_sec: AtomicU64,
    pub decay_duration_msec: AtomicU64,
    pub decay_volume_pct: AtomicU8,
    pub buffer_level: AtomicU64,
}

impl BalancerStateData {
    pub fn new(cfg: SurbBalancerConfig) -> Self {
        let state = Self::default();
        state.update(&cfg);
        state
    }

    /// Performs update of the [`BalancerStateData`] from the [`SurbBalancerConfig`] and
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

    /// Extracts the [`SurbBalancerConfig`] from the [`BalancerStateData`].
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

    /// Extracts the SURB decay configuration from the [`BalancerStateData`].
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

    /// Returns the current [`BalancerControllerBounds`] from the [`BalancerStateData`].
    #[inline]
    pub fn controller_bounds(&self) -> BalancerControllerBounds {
        BalancerControllerBounds::new(
            self.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
            self.max_surbs_per_sec.load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}

impl From<SurbBalancerConfig> for BalancerStateData {
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
/// Start protocol's [`KeepAlive`](crate::initiation::StartProtocol::KeepAlive) messages to deliver additional
/// SURBs to the counterparty.
pub struct SurbBalancer<C, E, F> {
    session_id: SessionId,
    controller: C,
    surb_estimator: E,
    flow_control: F,
    state: Arc<BalancerStateData>,
    last_estimator_state: SimpleSurbFlowEstimator,
    last_update: std::time::Instant,
    last_decay: std::time::Instant,
    was_below_target: bool,
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
        state: Arc<BalancerStateData>,
    ) -> Self {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], 0.0);
            METRIC_CONTROL_OUTPUT.set(&[&sid], 0.0);
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

        self.flow_control.adjust_surb_flow(output as usize);

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = self.session_id.to_string();
            METRIC_CURRENT_BUFFER.set(&[&sid], current as f64);
            METRIC_CURRENT_TARGET.set(&[&sid], self.controller.bounds().target() as f64);
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], error as f64);
            METRIC_CONTROL_OUTPUT.set(&[&sid], output as f64);
            METRIC_SURB_RATE.set(&[&sid], target_buffer_change as f64 / dt.as_secs_f64());
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
        hopr_async_runtime::prelude::spawn(
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

    use hopr_crypto_random::Randomizable;
    use hopr_internal_types::prelude::HoprPseudonym;

    use super::*;
    use crate::balancer::{AtomicSurbFlowEstimator, MockSurbFlowController, pid::PidBalancerController};

    #[test]
    fn surb_balancer_config_should_be_convertible_to_atomics() {
        let cfg = SurbBalancerConfig::default();
        let state_data = BalancerStateData::new(cfg);
        assert_eq!(cfg, state_data.as_config());
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
            SessionId::new(1234_u64, HoprPseudonym::random()),
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
            SessionId::new(1234_u64, HoprPseudonym::random()),
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

    #[test_log::test(tokio::test)]
    async fn surb_balancer_should_start_decrease_level_when_above_target_and_decay_enabled() {
        const NUM_STEPS: usize = 5;
        let session_id = SessionId::new(1234_u64, HoprPseudonym::random());
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

        assert_eq!(levels, vec![5000, 4750, 4750, 4500, 4500]);
    }
}
