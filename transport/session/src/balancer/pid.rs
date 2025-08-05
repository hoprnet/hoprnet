use pid::Pid;

use crate::{balancer::SurbBalancerController, errors, errors::SessionManagerError};

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
    fn new(setpoint: u64, output_limit: u64, gains: PidControllerGains) -> Self {
        let mut pid = Pid::new(setpoint as f64, output_limit as f64);
        pid.p(gains.p(), output_limit as f64);
        pid.i(gains.i(), output_limit as f64);
        pid.d(gains.d(), output_limit as f64);
        Self(pid)
    }
}

impl Default for PidBalancerController {
    /// The default instance does nothing unless [reconfigured](SurbBalancerController::set_target_and_limit).
    fn default() -> Self {
        Self::new(0, 0, PidControllerGains::default())
    }
}

impl SurbBalancerController for PidBalancerController {
    fn set_target_and_limit(&mut self, target: u64, output_limit: u64) {
        self.0.setpoint = target as f64;
        self.0.output_limit = output_limit as f64;
        self.0.p_limit = output_limit as f64;
        self.0.i_limit = output_limit as f64;
        self.0.d_limit = output_limit as f64;
    }

    fn next_control_output(&mut self, current_buffer_level: u64) -> u64 {
        self.0.next_control_output(current_buffer_level as f64).output as u64
    }
}
