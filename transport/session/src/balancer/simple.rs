use hopr_statistics::moving::simple::{NoSumSMA, SMA};

use crate::balancer::{BalancerControllerBounds, SurbBalancerController};

/// Minimum control output (SURBs/sec) to prevent complete starvation and deadlock.
/// This ensures that even when the buffer is empty, some traffic can flow to allow recovery.
const MIN_CONTROL_OUTPUT: u64 = 10;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_SIMPLE_CONTROL_OUTPUT: hopr_metrics::SimpleHistogram =
        hopr_metrics::SimpleHistogram::new(
            "hopr_surb_balancer_simple_control_output",
            "Control output of the simple SURB balancer controller",
            vec![0.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 2500.0, 5000.0]
        ).unwrap();
    static ref METRIC_SIMPLE_STARVATION_FLOOR_APPLIED: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_surb_balancer_simple_starvation_floor_applied_total",
            "Number of times the minimum floor was applied to prevent starvation"
        ).unwrap();
}

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
    bounds: BalancerControllerBounds,
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
            bounds: BalancerControllerBounds::default(),
            increasing: Some((ratio_threshold.clamp(0.0, 1.0), NoSumSMA::new(window_size.max(1)))),
        }
    }
}

impl SurbBalancerController for SimpleBalancerController {
    fn bounds(&self) -> BalancerControllerBounds {
        self.bounds
    }

    fn set_target_and_limit(&mut self, bounds: BalancerControllerBounds) {
        self.bounds = bounds;
    }

    fn next_control_output(&mut self, current_buffer_level: u64) -> u64 {
        let ratio = current_buffer_level as f64 / self.bounds.target() as f64;

        if let Some((threshold, sma)) = self.increasing.as_mut() {
            sma.push(ratio);
            if let Some(avg) = sma.average().filter(|avg| *avg >= *threshold + 1.0) {
                let new_setpoint = (avg * self.bounds.target() as f64).round() as u64;
                let new_limit = (avg * self.bounds.output_limit() as f64).floor() as u64;
                tracing::debug!(
                    old_setpoint = self.bounds.target(),
                    new_setpoint,
                    old_limit = self.bounds.output_limit(),
                    new_limit,
                    avg,
                    "setpoint increased"
                );
                self.bounds = BalancerControllerBounds::new(new_setpoint, new_limit);
            }
        }

        let raw_output = (self.bounds.output_limit() as f64 * ratio.clamp(0.0, 1.0)).floor() as u64;
        let output = raw_output.max(MIN_CONTROL_OUTPUT);

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_SIMPLE_CONTROL_OUTPUT.observe(output as f64);
            if raw_output < MIN_CONTROL_OUTPUT {
                METRIC_SIMPLE_STARVATION_FLOOR_APPLIED.increment();
                tracing::warn!(
                    raw_output,
                    applied_output = output,
                    "SURB starvation detected, applying minimum rate floor"
                );
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_balancer() {
        let mut controller = SimpleBalancerController::default();
        controller.set_target_and_limit(BalancerControllerBounds::new(100, 100));
        assert_eq!(100, controller.bounds.target());
        assert_eq!(controller.next_control_output(10), 10);
        assert_eq!(controller.next_control_output(100), 100);
        assert_eq!(controller.next_control_output(101), 100);
        assert_eq!(100, controller.bounds.target());
    }

    #[test_log::test]
    fn test_simple_balance_with_increasing_setpoint() {
        let mut controller = SimpleBalancerController::with_increasing_setpoint(0.2, 3);
        controller.set_target_and_limit(BalancerControllerBounds::new(100, 100));
        assert_eq!(100, controller.bounds.target());

        assert_eq!(100, controller.next_control_output(101));
        assert_eq!(100, controller.bounds.target());
        assert_eq!(100, controller.next_control_output(120));
        assert_eq!(100, controller.bounds.target());
        assert_eq!(140, controller.next_control_output(200));
        assert_eq!(140, controller.bounds.target());
    }

    #[test]
    fn simple_controller_should_never_output_zero_for_empty_buffer() {
        let mut controller = SimpleBalancerController::default();
        controller.set_target_and_limit(BalancerControllerBounds::new(1000, 500));

        // With buffer at 0, output should still be non-zero to allow recovery
        let output = controller.next_control_output(0);
        assert!(
            output > 0,
            "Controller output {} should never be zero to prevent deadlock",
            output
        );
    }

    #[test]
    fn simple_controller_should_never_output_zero_for_very_low_buffer() {
        let mut controller = SimpleBalancerController::default();
        controller.set_target_and_limit(BalancerControllerBounds::new(10000, 5000));

        // Even with a tiny buffer, output should be at least MIN_CONTROL_OUTPUT
        let output = controller.next_control_output(1);
        assert!(
            output > 0,
            "Controller output {} should never be zero to prevent deadlock",
            output
        );
    }
}
