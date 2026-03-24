use std::str::FromStr;

use anyhow::anyhow;
use pid::Pid;

use crate::{
    balancer::{BalancerControllerBounds, SurbBalancerController},
    errors,
    errors::SessionManagerError,
};

/// Carries finite Proportional, Integral and Derivative controller gains for a PID controller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PidControllerGains(f64, f64, f64);

impl PidControllerGains {
    /// Creates PID controller gains, returns an error if the gains are not finite.
    pub fn new(p: f64, i: f64, d: f64) -> errors::Result<Self> {
        if p.is_finite() && i.is_finite() && d.is_finite() {
            Ok(Self(p, i, d))
        } else {
            Err(SessionManagerError::other(anyhow!("gains must be finite")).into())
        }
    }

    /// Uses PID controller gains from the env variables or uses the defaults if not set.
    pub fn from_env_or_default() -> Self {
        let default = Self::default();
        Self(
            std::env::var("HOPR_BALANCER_PID_P_GAIN")
                .ok()
                .and_then(|v| f64::from_str(&v).ok())
                .unwrap_or(default.0),
            std::env::var("HOPR_BALANCER_PID_I_GAIN")
                .ok()
                .and_then(|v| f64::from_str(&v).ok())
                .unwrap_or(default.1),
            std::env::var("HOPR_BALANCER_PID_D_GAIN")
                .ok()
                .and_then(|v| f64::from_str(&v).ok())
                .unwrap_or(default.2),
        )
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
impl Eq for PidControllerGains {}

// Default coefficients for the PID controller
// This might be tweaked in the future.
const DEFAULT_P_GAIN: f64 = 0.6;
const DEFAULT_I_GAIN: f64 = 0.7;
const DEFAULT_D_GAIN: f64 = 0.2;

impl Default for PidControllerGains {
    fn default() -> Self {
        Self(DEFAULT_P_GAIN, DEFAULT_I_GAIN, DEFAULT_D_GAIN)
    }
}

impl TryFrom<(f64, f64, f64)> for PidControllerGains {
    type Error = errors::TransportSessionError;

    fn try_from(value: (f64, f64, f64)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1, value.2)
    }
}

/// Implementation of [`SurbBalancerController`] using a PID controller.
#[derive(Clone, Copy, Debug)]
pub struct PidBalancerController(Pid<f64>);

impl PidBalancerController {
    /// Creates new instance given the `setpoint`, `output_limit` and PID gains (P,I and D).
    pub fn new(setpoint: u64, output_limit: u64, gains: PidControllerGains) -> Self {
        let mut pid = Pid::new(setpoint as f64, output_limit as f64);
        pid.p(gains.p(), output_limit as f64);
        pid.i(gains.i(), output_limit as f64);
        pid.d(gains.d(), output_limit as f64);
        Self(pid)
    }

    /// Creates new instance with setpoint and output limit set to 0.
    ///
    /// Needs to be [reconfigured](SurbBalancerController::set_target_and_limit) in order to function
    /// correctly.
    pub fn from_gains(gains: PidControllerGains) -> Self {
        Self::new(0, 0, gains)
    }
}

impl Default for PidBalancerController {
    /// The default instance does nothing unless [reconfigured](SurbBalancerController::set_target_and_limit).
    fn default() -> Self {
        Self::new(0, 0, PidControllerGains::default())
    }
}

impl SurbBalancerController for PidBalancerController {
    fn bounds(&self) -> BalancerControllerBounds {
        BalancerControllerBounds::new(self.0.setpoint as u64, self.0.output_limit as u64)
    }

    fn set_target_and_limit(&mut self, bounds: BalancerControllerBounds) {
        let mut pid = Pid::new(bounds.target() as f64, bounds.output_limit() as f64);
        pid.p(self.0.kp, bounds.output_limit() as f64);
        pid.i(self.0.ki, bounds.output_limit() as f64);
        pid.d(self.0.kd, bounds.output_limit() as f64);
        self.0 = pid;
    }

    fn next_control_output(&mut self, current_buffer_level: u64) -> u64 {
        self.0.next_control_output(current_buffer_level as f64).output.max(0.0) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- PidControllerGains tests ---

    #[test]
    fn gains_default_values_are_stable() {
        let gains = PidControllerGains::default();
        insta::assert_yaml_snapshot!((gains.p(), gains.i(), gains.d()));
    }

    #[test]
    fn gains_finite_values_are_accepted() -> anyhow::Result<()> {
        let gains = PidControllerGains::new(1.0, 2.0, 3.0)?;
        insta::assert_yaml_snapshot!((gains.p(), gains.i(), gains.d()));
        Ok(())
    }

    #[test]
    fn gains_infinity_is_rejected() {
        assert!(PidControllerGains::new(f64::INFINITY, 0.0, 0.0).is_err());
        assert!(PidControllerGains::new(0.0, f64::NEG_INFINITY, 0.0).is_err());
        assert!(PidControllerGains::new(0.0, 0.0, f64::NAN).is_err());
    }

    #[test]
    fn gains_try_from_tuple() -> anyhow::Result<()> {
        let gains = PidControllerGains::try_from((0.5, 0.3, 0.1))?;
        insta::assert_yaml_snapshot!((gains.p(), gains.i(), gains.d()));
        Ok(())
    }

    #[test]
    fn gains_try_from_tuple_with_nan_fails() {
        assert!(PidControllerGains::try_from((f64::NAN, 0.0, 0.0)).is_err());
    }

    #[test]
    fn gains_eq_works() -> anyhow::Result<()> {
        let a = PidControllerGains::new(1.0, 2.0, 3.0)?;
        let b = PidControllerGains::new(1.0, 2.0, 3.0)?;
        assert_eq!(a, b);
        Ok(())
    }

    // --- PidBalancerController tests ---

    #[test]
    fn controller_default_has_zero_bounds() {
        let ctrl = PidBalancerController::default();
        assert_eq!(ctrl.bounds().unzip(), (0, 0));
    }

    #[test]
    fn controller_new_stores_bounds() {
        let gains = PidControllerGains::default();
        let ctrl = PidBalancerController::new(100, 50, gains);
        assert_eq!(ctrl.bounds().unzip(), (100, 50));
    }

    #[test]
    fn controller_set_target_and_limit_updates_bounds() {
        let mut ctrl = PidBalancerController::default();
        ctrl.set_target_and_limit(BalancerControllerBounds::new(200, 100));
        assert_eq!(ctrl.bounds().unzip(), (200, 100));
    }

    #[test]
    fn controller_step_response_snapshot() {
        // Apply a step input (setpoint=100, buffer=0) and observe outputs over N steps
        let gains = PidControllerGains::default();
        let mut ctrl = PidBalancerController::new(100, 200, gains);

        let outputs: Vec<u64> = (0..10).map(|_| ctrl.next_control_output(0)).collect();
        insta::assert_yaml_snapshot!(outputs);
    }

    #[test]
    fn controller_at_setpoint_outputs_zero_or_near_zero() {
        let gains = PidControllerGains::default();
        let mut ctrl = PidBalancerController::new(100, 200, gains);

        // When buffer level equals setpoint, output should converge toward 0
        let output = ctrl.next_control_output(100);
        // First call at setpoint: P=0, I=0, D=0 → output=0
        assert_eq!(output, 0);
    }

    #[test]
    fn controller_above_setpoint_clamps_to_zero() {
        let gains = PidControllerGains::default();
        let mut ctrl = PidBalancerController::new(100, 200, gains);

        // Buffer well above setpoint — PID error is negative, output clamped to 0
        let output = ctrl.next_control_output(200);
        assert_eq!(output, 0);
    }

    #[test]
    fn controller_convergence_from_empty_buffer() {
        // Simulate filling a buffer from 0 toward setpoint=100
        let gains = PidControllerGains::default();
        let mut ctrl = PidBalancerController::new(100, 200, gains);

        let mut buffer: f64 = 0.0;
        let mut history = Vec::new();

        for _ in 0..20 {
            let output = ctrl.next_control_output(buffer as u64);
            buffer += output as f64;
            buffer = buffer.min(200.0); // clamp to limit
            history.push(buffer as u64);
        }

        insta::assert_yaml_snapshot!(history);
    }

    #[test]
    fn controller_from_gains_uses_defaults_for_bounds() {
        let gains = PidControllerGains::default();
        let ctrl = PidBalancerController::from_gains(gains);
        assert_eq!(ctrl.bounds().unzip(), (0, 0));
    }
}
