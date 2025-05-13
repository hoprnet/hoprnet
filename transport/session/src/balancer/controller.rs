use pid::Pid;
use std::fmt::Display;

use crate::balancer::{SurbFlowController, SurbFlowEstimator};

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
    static ref METRIC_SURBS_CONSUMED: hopr_metrics::metrics::MultiCounter =
        hopr_metrics::metrics::MultiCounter::new(
            "hopr_surb_balancer_surbs_consumed",
            "Estimations of the number of SURBs consumed by the counterparty",
            &["session_id"]
    ).unwrap();
    static ref METRIC_SURBS_PRODUCED: hopr_metrics::metrics::MultiCounter =
        hopr_metrics::metrics::MultiCounter::new(
            "hopr_surb_balancer_surbs_produced",
            "Estimations of the number of SURBs produced for the counterparty",
            &["session_id"]
    ).unwrap();
}

/// Configuration for the [`SurbBalancer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurbBalancerConfig {
    /// The desired number of SURBs to be always kept as a buffer at the Session counterparty.
    ///
    /// The [`SurbBalancer`] will try to maintain approximately this number of SURBs
    /// at the counterparty at all times, by regulating the [flow of non-organic SURBs](SurbFlowController).
    #[default(5_000)]
    pub target_surb_buffer_size: u64,
    /// Maximum outflow of non-organic surbs.
    ///
    /// The default is 2500 (which is 1250 packets/second currently)
    #[default(2_500)]
    pub max_surbs_per_sec: u64,
}

/// Runs a continuous process that attempts to [evaluate](SurbFlowEstimator) and
/// [regulate](SurbFlowController) the flow of non-organic SURBs to the Session counterparty,
/// to keep the number of SURBs at the counterparty at a certain level.
///
/// Internally, the Balancer uses a PID (Proportional Integral Derivative) controller to
/// control the rate of SURBs sent to the counterparty
/// each time the [`update`](SurbBalancer::update) method is called:
///
/// 1. The size of the SURB buffer at the counterparty is estimated using [`SurbFlowEstimator`].
/// 2. Error against a set-point given in [`SurbBalancerConfig`] is evaluated in the PID controller.
/// 3. The PID controller applies a new SURB flow rate value using the [`SurbFlowController`].
pub struct SurbBalancer<I, O, F, S> {
    session_id: S,
    pid: Pid<f64>,
    surb_production_estimator: O,
    surb_consumption_estimator: I,
    controller: F,
    cfg: SurbBalancerConfig,
    current_target_buffer: u64,
    last_surbs_delivered: u64,
    last_surbs_used: u64,
    last_update: std::time::Instant,
    was_below_target: bool,
}

// Default coefficients for the PID controller
// This might be tweaked in the future.
const DEFAULT_P_GAIN: f64 = 0.6;
const DEFAULT_I_GAIN: f64 = 0.7;
const DEFAULT_D_GAIN: f64 = 0.2;

impl<I, O, F, S> SurbBalancer<I, O, F, S>
where
    O: SurbFlowEstimator,
    I: SurbFlowEstimator,
    F: SurbFlowController,
    S: Display,
{
    pub fn new(
        session_id: S,
        surb_production_estimator: O,
        surb_consumption_estimator: I,
        controller: F,
        cfg: SurbBalancerConfig,
    ) -> Self {
        // Initialize the PID controller with default tuned gains
        let max_surbs_per_sec = cfg.max_surbs_per_sec as f64;
        let mut pid = Pid::new(cfg.target_surb_buffer_size as f64, max_surbs_per_sec);
        pid.p(
            std::env::var("HOPR_BALANCER_PID_P_GAIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_P_GAIN),
            max_surbs_per_sec,
        );
        pid.i(
            std::env::var("HOPR_BALANCER_PID_I_GAIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_I_GAIN),
            max_surbs_per_sec,
        );
        pid.d(
            std::env::var("HOPR_BALANCER_PID_D_GAIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_D_GAIN),
            max_surbs_per_sec,
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], 0.0);
            METRIC_CONTROL_OUTPUT.set(&[&sid], 0.0);
        }

        Self {
            surb_production_estimator,
            surb_consumption_estimator,
            controller,
            pid,
            cfg,
            session_id,
            current_target_buffer: 0,
            last_surbs_delivered: 0,
            last_surbs_used: 0,
            last_update: std::time::Instant::now(),
            was_below_target: true,
        }
    }

    /// Computes the next control update and adjusts the [`SurbFlowController`] rate accordingly.
    #[tracing::instrument(level = "trace", skip(self), fields(session_id = %self.session_id))]
    pub fn update(&mut self) -> u64 {
        let dt = self.last_update.elapsed();
        if dt < std::time::Duration::from_millis(10) {
            tracing::debug!("time elapsed since last update is too short, skipping update");
            return self.current_target_buffer;
        }

        self.last_update = std::time::Instant::now();

        // Number of SURBs sent to the counterparty since the last update
        let current_surbs_delivered = self.surb_production_estimator.estimate_surb_turnout();
        let surbs_delivered_delta = current_surbs_delivered - self.last_surbs_delivered;
        self.last_surbs_delivered = current_surbs_delivered;

        // Number of SURBs used by the counterparty since the last update
        let current_surbs_used = self.surb_consumption_estimator.estimate_surb_turnout();
        let surbs_used_delta = current_surbs_used - self.last_surbs_used;
        self.last_surbs_used = current_surbs_used;

        // Estimated change in counterparty's SURB buffer
        let target_buffer_change = surbs_delivered_delta as i64 - surbs_used_delta as i64;
        self.current_target_buffer = self.current_target_buffer.saturating_add_signed(target_buffer_change);

        // Error from the desired target SURB buffer size at counterparty
        let error = self.current_target_buffer as i64 - self.cfg.target_surb_buffer_size as i64;

        if self.was_below_target && error >= 0 {
            tracing::debug!(session_id = %self.session_id, current = self.current_target_buffer, "reached target SURB buffer size");
            self.was_below_target = false;
        } else if !self.was_below_target && error < 0 {
            tracing::debug!(session_id = %self.session_id, current = self.current_target_buffer, "SURB buffer size is below target");
            self.was_below_target = true;
        }

        tracing::trace!(
            session_id = %self.session_id,
            ?dt,
            delta = target_buffer_change,
            current = self.current_target_buffer,
            error,
            rate_up = surbs_delivered_delta as f64 / dt.as_secs_f64(),
            rate_down = surbs_used_delta as f64 / dt.as_secs_f64(),
            "estimated SURB buffer change"
        );

        let output = self.pid.next_control_output(self.current_target_buffer as f64);
        let corrected_output = output.output.clamp(0.0, self.cfg.max_surbs_per_sec as f64);
        self.controller.adjust_surb_flow(corrected_output as usize);

        tracing::trace!(control_output = corrected_output, "next control output");

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = self.session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], error as f64);
            METRIC_CONTROL_OUTPUT.set(&[&sid], corrected_output);
            METRIC_SURBS_CONSUMED.increment_by(&[&sid], surbs_used_delta);
            METRIC_SURBS_PRODUCED.increment_by(&[&sid], surbs_delivered_delta);
        }

        self.current_target_buffer
    }

    /// Allows setting the target buffer size when its value is known exactly.
    #[allow(unused)]
    pub fn set_exact_target_buffer_size(&mut self, target_buffer_size: u64) {
        self.current_target_buffer = target_buffer_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::balancer::MockSurbFlowController;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

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

        let surb_production_count = Arc::new(AtomicU64::new(0));
        let surb_consumption_count = Arc::new(AtomicU64::new(0));
        let mut balancer = SurbBalancer::new(
            "test",
            surb_production_count.clone(),
            surb_consumption_count.clone(),
            controller,
            SurbBalancerConfig::default(),
        );

        let mut last_update = 0;
        for i in 0..steps {
            std::thread::sleep(step_duration);
            surb_production_count.fetch_add(
                production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );
            surb_consumption_count.fetch_add(
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

        let surb_production_count = Arc::new(AtomicU64::new(0));
        let surb_consumption_count = Arc::new(AtomicU64::new(0));
        let mut balancer = SurbBalancer::new(
            "test",
            surb_production_count.clone(),
            surb_consumption_count.clone(),
            controller,
            SurbBalancerConfig::default(),
        );

        let mut last_update = 0;
        for i in 0..steps {
            std::thread::sleep(step_duration);
            surb_production_count.fetch_add(
                production_rate.load(std::sync::atomic::Ordering::Relaxed) * step_duration.as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );
            surb_consumption_count.fetch_add(
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
}
