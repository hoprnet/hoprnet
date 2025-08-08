use std::str::FromStr;

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
            Err(SessionManagerError::Other("gains must be finite".into()).into())
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
