use hopr_primitive_types::{prelude::SMA, sma::NoSumSMA};

use crate::balancer::SurbBalancerController;

/// Controller that uses the simple linear formula `limit * min(current / setpoint, 1.0)` to
/// compute the control output.
///
/// The controller also allows optionally increasing the setpoint if its simple moving average
/// over a given number of samples is above a certain threshold.
/// See [`SimpleBalancerController::with_increasing_setpoint`] for details.
///
/// If the controller is constructed via the `Default` constructor, no setpoint increase takes place.
#[derive(Clone, Debug, Default)]
pub struct SimpleBalancerController {
    setpoint: u64,
    limit: u64,
    increasing: Option<(f64, NoSumSMA<f64>)>,
}

impl SimpleBalancerController {
    /// Constructs the controller with increasing setpoint.
    ///
    /// If the simple moving average of the `current / setpoint` ratio over `window_size` of samples
    /// is greater than the `ratio_threshold + 1`, the setpoint adjusted by multiplying
    /// by the value of the moving average.
    ///
    /// The given `ratio_threshold` must be between 0 and 1, otherwise it is clamped to this range.
    /// The `window_size` must be greater or equal to 1, otherwise it is set to 1.
    pub fn with_increasing_setpoint(ratio_threshold: f64, window_size: usize) -> Self {
        Self {
            setpoint: 0,
            limit: 0,
            increasing: Some((ratio_threshold.clamp(0.0, 1.0), NoSumSMA::new(window_size.max(1)))),
        }
    }
}

impl SurbBalancerController for SimpleBalancerController {
    fn target(&self) -> u64 {
        self.setpoint
    }

    fn output_limit(&self) -> u64 {
        self.limit
    }

    fn set_target_and_limit(&mut self, target: u64, output_limit: u64) {
        self.limit = output_limit;
        self.setpoint = target;
    }

    fn next_control_output(&mut self, current_buffer_level: u64) -> u64 {
        let ratio = current_buffer_level as f64 / self.setpoint as f64;

        if let Some((threshold, sma)) = self.increasing.as_mut() {
            sma.push(ratio);
            if let Some(avg) = sma.average().filter(|avg| *avg >= *threshold + 1.0) {
                let new_setpoint = (avg * self.setpoint as f64).round() as u64;
                tracing::debug!(
                    old_setpoint = self.setpoint,
                    new_setpoint,
                    avg,
                    ?sma,
                    "setpoint increased"
                );
                self.setpoint = new_setpoint;
            }
        }

        (self.limit as f64 * ratio.clamp(0.0, 1.0)).floor() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_balancer() {
        let mut controller = SimpleBalancerController::default();
        controller.set_target_and_limit(100, 100);
        assert_eq!(100, controller.target());
        assert_eq!(controller.next_control_output(10), 10);
        assert_eq!(controller.next_control_output(100), 100);
        assert_eq!(controller.next_control_output(101), 100);
        assert_eq!(100, controller.target());
    }

    #[test]
    fn test_simple_balance_with_increasing_setpoint() {
        let mut controller = SimpleBalancerController::with_increasing_setpoint(0.2, 3);
        controller.set_target_and_limit(100, 100);
        assert_eq!(100, controller.target());

        assert_eq!(100, controller.next_control_output(101));
        assert_eq!(100, controller.target());
        assert_eq!(100, controller.next_control_output(120));
        assert_eq!(100, controller.target());
        assert_eq!(100, controller.next_control_output(200));
        assert_eq!(140, controller.target());
    }
}
