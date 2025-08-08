use std::time::Duration;

use futures::{StreamExt, future::AbortRegistration, pin_mut};
use hopr_async_runtime::AbortHandle;
use tracing::{Instrument, debug, error, instrument, trace};

use super::{
    BalancerControllerBounds, MIN_BALANCER_SAMPLING_INTERVAL, SimpleSurbFlowEstimator, SurbBalancerController,
    SurbFlowController, SurbFlowEstimator,
};
use crate::SessionId;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TARGET_ERROR_ESTIMATE: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_surb_balancer_target_error_estimate",
            "Target error estimation by the SURB balancer",
            &["session_id"]
    ).unwrap();
    static ref METRIC_CONTROL_OUTPUT: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_surb_balancer_control_output",
            "hopr_surb_balancer_control_output",
            &["session_id"]
    ).unwrap();
    static ref METRIC_SURB_RATE: hopr_metrics::metrics::MultiGauge =
        hopr_metrics::metrics::MultiGauge::new(
            "hopr_surb_balancer_surbs_rate",
            "Estimation of SURB rate per second (positive is buffer surplus, negative is buffer loss)",
            &["session_id"]
    ).unwrap();
}

/// Allows updating [`SurbBalancerConfig`] from the [control loop](spawn_surb_balancer_control_loop).
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait BalancerConfigFeedback {
    /// Gets current [`SurbBalancerConfig`] for a Session with `id`.
    async fn get_config(&self, id: &SessionId) -> crate::errors::Result<SurbBalancerConfig>;
    /// Updates current [`SurbBalancerConfig`] for a Session with `id`.
    async fn on_config_update(&self, id: &SessionId, cfg: SurbBalancerConfig) -> crate::errors::Result<()>;
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
    /// - In the context of the local SURB buffer, this is the maximum egress Session traffic.
    /// - In the context of the remote SURB buffer, this is the maximum egress of keep-alive messages to the
    ///   counterparty.
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
    current_buffer: u64,
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
        cfg: SurbBalancerConfig,
    ) -> Self {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], 0.0);
            METRIC_CONTROL_OUTPUT.set(&[&sid], 0.0);
        }

        controller.set_target_and_limit(BalancerControllerBounds::new(
            cfg.target_surb_buffer_size,
            cfg.max_surbs_per_sec,
        ));

        Self {
            surb_estimator,
            flow_control,
            controller,
            session_id,
            current_buffer: 0,
            last_estimator_state: Default::default(),
            last_update: std::time::Instant::now(),
            last_decay: std::time::Instant::now(),
            was_below_target: true,
        }
    }

    /// Computes the next control update and adjusts the [`SurbFlowController`] rate accordingly.
    #[tracing::instrument(level = "trace", skip_all)]
    fn update(&mut self, surb_decay: Option<&(Duration, f64)>) -> u64 {
        let dt = self.last_update.elapsed();
        if dt < Duration::from_millis(10) {
            debug!("time elapsed since last update is too short, skipping update");
            return self.current_buffer;
        }

        self.last_update = std::time::Instant::now();

        // Take a snapshot of the active SURB estimator and calculate the balance change
        let snapshot = SimpleSurbFlowEstimator::from(&self.surb_estimator);
        let Some(target_buffer_change) = snapshot.estimated_surb_buffer_change(&self.last_estimator_state) else {
            error!("non-monotonic change in SURB estimators");
            return self.current_buffer;
        };

        self.last_estimator_state = snapshot;
        self.current_buffer = self.current_buffer.saturating_add_signed(target_buffer_change);

        // If SURB decaying is enabled, check if the decay window has elapsed
        // and calculate the number of SURBs that will be discarded
        if let Some(num_decayed_surbs) = surb_decay
            .as_ref()
            .filter(|(decay_window, _)| &self.last_decay.elapsed() >= decay_window)
            .map(|(_, decay_coeff)| (self.controller.bounds().target() as f64 * *decay_coeff).round() as u64)
        {
            self.current_buffer = self.current_buffer.saturating_sub(num_decayed_surbs);
            self.last_decay = std::time::Instant::now();
            trace!(num_decayed_surbs, "SURBs were discarded due to automatic decay");
        }

        // Error from the desired target SURB buffer size at counterparty
        let error = self.current_buffer as i64 - self.controller.bounds().target() as i64;

        if self.was_below_target && error >= 0 {
            trace!(current = self.current_buffer, "reached target SURB buffer size");
            self.was_below_target = false;
        } else if !self.was_below_target && error < 0 {
            trace!(current = self.current_buffer, "SURB buffer size is below target");
            self.was_below_target = true;
        }

        debug!(
            ?dt,
            delta = target_buffer_change,
            rate = target_buffer_change as f64 / dt.as_secs_f64(),
            current = self.current_buffer,
            error,
            "estimated SURB buffer change"
        );

        let output = self.controller.next_control_output(self.current_buffer);
        debug!(output, "next balancer control output for session");

        self.flow_control.adjust_surb_flow(output as usize);

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = self.session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], error as f64);
            METRIC_CONTROL_OUTPUT.set(&[&sid], output as f64);
            METRIC_SURB_RATE.set(&[&sid], target_buffer_change as f64 / dt.as_secs_f64());
        }

        self.current_buffer
    }

    /// Spawns a new task that performs updates of the given [`SurbBalancer`] at the given `sampling_interval`.
    ///
    /// If `surb_decay` is given, SURBs are removed at each window as the given percentage of the target buffer.
    /// If `cfg_feedback` is given, [`SurbBalancerConfig`] can be queried for updates and also updated
    /// if the underlying [`SurbBalancerController`] also does target updates.
    ///
    /// Returns a stream of current estimated buffer levels, and also an `AbortHandle`
    /// to terminate the loop. If `abort_reg` was given, the returned `AbortHandle` corresponds
    /// to it.
    #[instrument(level = "debug", skip_all, fields(session_id = %self.session_id))]
    pub fn start_control_loop<B>(
        mut self,
        sampling_interval: Duration,
        cfg_feedback: B,
        abort_reg: Option<AbortRegistration>,
    ) -> (impl futures::Stream<Item = u64>, AbortHandle)
    where
        B: BalancerConfigFeedback + Send + Sync + 'static,
    {
        // Get abort handle and registration (or create new ones)
        let (abort_handle, abort_reg) = abort_reg
            .map(|reg| (reg.handle(), reg))
            .unwrap_or_else(AbortHandle::new_pair);

        // Start an interval stream at which the balancer will sample and perform updates
        let sampling_stream = futures::stream::Abortable::new(
            futures_time::stream::interval(sampling_interval.max(MIN_BALANCER_SAMPLING_INTERVAL).into()),
            abort_reg,
        );

        let (level_tx, level_rx) = futures::channel::mpsc::unbounded();
        hopr_async_runtime::prelude::spawn(
            async move {
                pin_mut!(sampling_stream);
                while sampling_stream.next().await.is_some() {
                    let Ok(mut current_cfg) = cfg_feedback.get_config(&self.session_id).await else {
                        error!("cannot get config for session");
                        break;
                    };

                    let current_bounds = BalancerControllerBounds::new(
                        current_cfg.target_surb_buffer_size,
                        current_cfg.max_surbs_per_sec,
                    );

                    // Check if the balancer controller needs to be reconfigured
                    if current_bounds != self.controller.bounds() {
                        self.controller.set_target_and_limit(current_bounds);
                        debug!(?current_cfg, "surb balancer has been reconfigured");
                    }

                    let bounds_before_update = self.controller.bounds();

                    // Perform controller update (this internally samples the SurbFlowEstimator)
                    // and send an update about the current level to the outgoing stream
                    let level = self.update(current_cfg.surb_decay.as_ref());
                    let _ = level_tx.unbounded_send(level);

                    // See if the setpoint has been updated at the controller as a result
                    // of the update step, because some controllers (such as the SimpleBalancerController)
                    // permit that.
                    let bounds_after_update = self.controller.bounds();
                    if bounds_before_update != bounds_after_update {
                        current_cfg.target_surb_buffer_size = bounds_after_update.target();
                        current_cfg.max_surbs_per_sec = bounds_after_update.output_limit();
                        match cfg_feedback.on_config_update(&self.session_id, current_cfg).await {
                            Ok(_) => debug!(
                                ?bounds_before_update,
                                ?bounds_after_update,
                                "controller bounds has changed after update"
                            ),
                            Err(error) => error!(%error, "failed to update controller bounds after it changed"),
                        }
                    }
                }

                debug!("balancer done");
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
    use crate::balancer::{
        AtomicSurbFlowEstimator, MockSurbFlowController, pid::PidBalancerController, simple::SimpleBalancerController,
    };

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
            SurbBalancerConfig {
                target_surb_buffer_size: 5_000,
                max_surbs_per_sec: 2500,
                ..Default::default()
            },
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

            let next_update = balancer.update(None);
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
            SurbBalancerConfig::default(),
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

            let next_update = balancer.update(None);
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
            ..Default::default()
        };

        let mut mock_balancer_feedback = MockBalancerConfigFeedback::new();
        mock_balancer_feedback
            .expect_get_config()
            .with(mockall::predicate::eq(session_id))
            .times(NUM_STEPS)
            .returning(move |_| Ok(cfg));

        mock_balancer_feedback.expect_on_config_update().never();

        let mut mock_flow_ctl = MockSurbFlowController::new();
        mock_flow_ctl
            .expect_adjust_surb_flow()
            .times(NUM_STEPS)
            .returning(|_| ());

        let mut balancer = SurbBalancer::new(
            session_id,
            PidBalancerController::default(),
            SimpleSurbFlowEstimator::default(),
            mock_flow_ctl,
            cfg,
        );

        balancer.current_buffer = 5000;

        let (stream, handle) = balancer.start_control_loop(Duration::from_millis(100), mock_balancer_feedback, None);
        let levels = stream.take(NUM_STEPS).collect::<Vec<_>>().await;
        handle.abort();

        assert_eq!(levels, vec![5000, 4750, 4750, 4500, 4500]);
    }

    struct IterSurbFlowEstimator<P, C>(std::sync::Mutex<P>, std::sync::Mutex<C>);

    impl<P, C> IterSurbFlowEstimator<P, C> {
        fn new<I1, I2>(production: I1, consumption: I2) -> Self
        where
            I1: IntoIterator<Item = u64, IntoIter = P>,
            I2: IntoIterator<Item = u64, IntoIter = C>,
        {
            Self(
                std::sync::Mutex::new(production.into_iter()),
                std::sync::Mutex::new(consumption.into_iter()),
            )
        }
    }

    impl<P, C> SurbFlowEstimator for IterSurbFlowEstimator<P, C>
    where
        P: Iterator<Item = u64>,
        C: Iterator<Item = u64>,
    {
        fn estimate_surbs_consumed(&self) -> u64 {
            self.1.lock().ok().and_then(|mut it| it.next()).unwrap_or(0)
        }

        fn estimate_surbs_produced(&self) -> u64 {
            self.0.lock().ok().and_then(|mut it| it.next()).unwrap_or(0)
        }
    }

    #[test_log::test(tokio::test)]
    async fn surb_balancer_should_increase_target_when_using_simple_controller() {
        let session_id = SessionId::new(1234_u64, HoprPseudonym::random());
        let cfg_1 = SurbBalancerConfig {
            target_surb_buffer_size: 4500,
            max_surbs_per_sec: 2500,
            surb_decay: None,
            ..Default::default()
        };

        let cfg_2 = SurbBalancerConfig {
            target_surb_buffer_size: 5500,
            max_surbs_per_sec: 3055,
            surb_decay: None,
            ..Default::default()
        };

        let mut seq = mockall::Sequence::new();

        let mut mock_balancer_feedback = MockBalancerConfigFeedback::new();
        mock_balancer_feedback
            .expect_get_config()
            .times(3)
            .in_sequence(&mut seq)
            .with(mockall::predicate::eq(session_id))
            .returning(move |_| {
                tracing::trace!("get config 1");
                Ok(cfg_1)
            });

        mock_balancer_feedback
            .expect_on_config_update()
            .once()
            .in_sequence(&mut seq)
            .with(mockall::predicate::eq(session_id), mockall::predicate::eq(cfg_2))
            .returning(|_, _| {
                tracing::trace!("on config update");
                Ok(())
            });

        mock_balancer_feedback
            .expect_get_config()
            .times(2)
            .in_sequence(&mut seq)
            .with(mockall::predicate::eq(session_id))
            .returning(move |_| {
                tracing::trace!("get config 2");
                Ok(cfg_2)
            });

        let mut mock_flow_ctl = MockSurbFlowController::new();
        mock_flow_ctl.expect_adjust_surb_flow().times(5).returning(|_| ());

        let mut balancer = SurbBalancer::new(
            session_id,
            SimpleBalancerController::with_increasing_setpoint(0.2, 5),
            IterSurbFlowEstimator::new([1000, 2000, 3000, 3000, 3000], vec![500, 1000, 1500, 1500, 1500]),
            mock_flow_ctl,
            cfg_1,
        );

        balancer.current_buffer = 4500;

        let (stream, handle) = balancer.start_control_loop(Duration::from_millis(100), mock_balancer_feedback, None);

        let levels = stream.take(5).collect::<Vec<_>>().await;
        handle.abort();

        assert_eq!(levels, vec![5000, 5500, 6000, 6000, 6000]);
    }
}
