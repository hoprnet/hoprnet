use std::fmt::Display;

use pid::Pid;

use crate::{
    balancer::{SimpleSurbFlowEstimator, SurbFlowController, SurbFlowEstimator},
    errors,
    errors::SessionManagerError,
};

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

/// Carries finite Proportional, Integral and Derivative controller gains for a PID controller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControllerGains(f64, f64, f64);

impl ControllerGains {
    /// Creates PID controller gains, returns an error if the gains are not finite.
    pub fn new(p: f64, i: f64, d: f64) -> errors::Result<Self> {
        if p.is_finite() && i.is_finite() && d.is_finite() {
            Ok(Self(p, i, d))
        } else {
            Err(SessionManagerError::Other("gains must be finite".into()).into())
        }
    }

    /// P gain.
    #[inline]
    pub fn p(&self) -> f64 {
        self.0
    }

    /// I gain.
    #[inline]
    pub fn i(&self) -> f64 {
        self.1
    }

    /// D gain.
    #[inline]
    pub fn d(&self) -> f64 {
        self.2
    }
}

// Safe to implement Eq, because the floats are finite
impl Eq for ControllerGains {}

// Default coefficients for the PID controller
// This might be tweaked in the future.
const DEFAULT_P_GAIN: f64 = 0.6;
const DEFAULT_I_GAIN: f64 = 0.7;
const DEFAULT_D_GAIN: f64 = 0.2;

impl Default for ControllerGains {
    fn default() -> Self {
        Self(DEFAULT_P_GAIN, DEFAULT_I_GAIN, DEFAULT_D_GAIN)
    }
}

impl TryFrom<(f64, f64, f64)> for ControllerGains {
    type Error = errors::TransportSessionError;

    fn try_from(value: (f64, f64, f64)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1, value.2)
    }
}

/// Configuration for the [`SurbBalancer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault)]
pub struct SurbBalancerConfig {
    /// The desired number of SURBs to be always kept as a buffer locally or at the Session counterparty.
    ///
    /// The [`SurbBalancer`] will try to maintain approximately this number of SURBs
    /// locally or remotely (at the counterparty) at all times.
    ///
    /// The local buffer is maintained by [regulating](SurbFlowController) the egress from the Session.
    /// The remote buffer (at session counterparty) is maintained by regulating the flow of non-organic SURBs via
    /// [keep-alive messages](crate::initiation::StartProtocol::KeepAlive).
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
    /// PID controller gains.
    ///
    /// Default is (0.6, 0.7, 0.2), suitable for active SURB balancing.
    pub gains: ControllerGains,
    /// If set to `true`, the control output will be inverted with respect to `max_surbs_per_sec`
    /// (i.e.: `output = max_surbs_per_sec - output`).
    ///
    /// Default is false
    #[default(false)]
    pub invert_output: bool,
}

/// Runs a continuous process that attempts to [evaluate](SurbFlowEstimator) and
/// [regulate](SurbFlowController) the flow of SURBs to the Session counterparty,
/// to keep the number of SURBs locally or at the counterparty at a certain level.
///
/// Internally, the Balancer uses a PID (Proportional Integral Derivative) controller to
/// control the rate of SURBs consumed or sent to the counterparty
/// each time the [`update`](SurbBalancer::update) method is called:
///
/// 1. The size of the SURB buffer at locally or at the counterparty is estimated using [`SurbFlowEstimator`].
/// 2. Error against a set-point given in [`SurbBalancerConfig`] is evaluated in the PID controller.
/// 3. The PID controller applies a new SURB flow rate value using the [`SurbFlowController`].
///
/// In the local context, the `SurbFlowController` might simply regulate the egress traffic from the
/// Session, slowing it down to avoid fast SURB drainage.
///
/// In the remote context, the `SurbFlowController` might regulate the flow of non-organic SURBs via
/// Start protocol's [`KeepAlive`](crate::initiation::StartProtocol::KeepAlive) messages to deliver additional
/// SURBs to the counterparty.
pub struct SurbBalancer<E, F, S> {
    session_id: S,
    pid: Pid<f64>,
    surb_estimator: E,
    flow_control: F,
    cfg: SurbBalancerConfig,
    current_target_buffer: u64,
    last_estimator_state: SimpleSurbFlowEstimator,
    last_update: std::time::Instant,
    was_below_target: bool,
}

impl<E, F, S> SurbBalancer<E, F, S>
where
    E: SurbFlowEstimator,
    F: SurbFlowController,
    S: Display,
{
    pub fn new(session_id: S, surb_estimator: E, flow_control: F, cfg: SurbBalancerConfig) -> Self {
        // Initialize the PID controller with default tuned gains
        let max_surbs_per_sec = cfg.max_surbs_per_sec as f64;
        let pid = Pid::new(cfg.target_surb_buffer_size as f64, max_surbs_per_sec);

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], 0.0);
            METRIC_CONTROL_OUTPUT.set(&[&sid], 0.0);
        }

        let mut ret = Self {
            surb_estimator,
            flow_control,
            pid,
            cfg,
            session_id,
            current_target_buffer: 0,
            last_estimator_state: Default::default(),
            last_update: std::time::Instant::now(),
            was_below_target: true,
        };

        ret.reconfigure(cfg);
        ret
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

        // Take a snapshot of the active SURB estimator and calculate the balance change
        let snapshot = SimpleSurbFlowEstimator::from(&self.surb_estimator);
        let Some(target_buffer_change) = snapshot.estimated_surb_buffer_change(&self.last_estimator_state) else {
            tracing::error!("non-monotonic change in SURB estimators");
            return self.current_target_buffer;
        };

        self.last_estimator_state = snapshot;
        self.current_target_buffer = self.current_target_buffer.saturating_add_signed(target_buffer_change);

        // Error from the desired target SURB buffer size at counterparty
        let error = self.current_target_buffer as i64 - self.cfg.target_surb_buffer_size as i64;

        if self.was_below_target && error >= 0 {
            tracing::trace!(current = self.current_target_buffer, "reached target SURB buffer size");
            self.was_below_target = false;
        } else if !self.was_below_target && error < 0 {
            tracing::trace!(current = self.current_target_buffer, "SURB buffer size is below target");
            self.was_below_target = true;
        }

        tracing::trace!(
            ?dt,
            delta = target_buffer_change,
            rate = target_buffer_change as f64 / dt.as_secs_f64(),
            current = self.current_target_buffer,
            error,
            "estimated SURB buffer change"
        );

        let output = self.pid.next_control_output(self.current_target_buffer as f64);

        // Clamp the control output between [0, max_surbs_per_sec] and invert if needed
        let mut corrected_output = output.output.clamp(0.0, self.cfg.max_surbs_per_sec as f64);
        if self.cfg.invert_output {
            corrected_output = self.cfg.max_surbs_per_sec as f64 - corrected_output;
        }

        tracing::debug!(
            control_output = corrected_output,
            "next balancer control output for session"
        );

        self.flow_control.adjust_surb_flow(corrected_output as usize);

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            let sid = self.session_id.to_string();
            METRIC_TARGET_ERROR_ESTIMATE.set(&[&sid], error as f64);
            METRIC_CONTROL_OUTPUT.set(&[&sid], corrected_output);
            METRIC_SURB_RATE.set(&[&sid], target_buffer_change as f64 / dt.as_secs_f64());
        }

        self.current_target_buffer
    }

    /// Allows setting the target buffer size when its value is known exactly.
    #[allow(unused)]
    pub fn set_exact_target_buffer_size(&mut self, target_buffer_size: u64) {
        self.current_target_buffer = target_buffer_size;
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &SurbBalancerConfig {
        &self.cfg
    }

    /// Allows reconfiguring the instance.
    pub fn reconfigure(&mut self, cfg: SurbBalancerConfig) {
        let max_surbs_per_sec = cfg.max_surbs_per_sec as f64;
        self.pid.setpoint = cfg.target_surb_buffer_size as f64;
        self.pid.output_limit = cfg.max_surbs_per_sec as f64;
        self.pid.p(
            std::env::var("HOPR_BALANCER_PID_P_GAIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(cfg.gains.p()),
            max_surbs_per_sec,
        );
        self.pid.i(
            std::env::var("HOPR_BALANCER_PID_I_GAIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(cfg.gains.i()),
            max_surbs_per_sec,
        );
        self.pid.d(
            std::env::var("HOPR_BALANCER_PID_D_GAIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(cfg.gains.d()),
            max_surbs_per_sec,
        );
        tracing::debug!(
            session_id = %self.session_id,
            p = self.pid.kp,
            i = self.pid.ki,
            d = self.pid.kd,
            target = self.pid.setpoint,
            "reconfigured balancer"
        );
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::AtomicU64};

    use super::*;
    use crate::balancer::{AtomicSurbFlowEstimator, MockSurbFlowController};

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
            "test",
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
            "test",
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

            let next_update = balancer.update();
            assert!(
                i == 0 || next_update < last_update,
                "{next_update} should be greater than {last_update}"
            );
            last_update = next_update;
        }
    }
}
