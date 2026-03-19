use crate::balancer::{BalancerControllerBounds, SurbBalancerController};

/// Controller that uses the simple linear formula `limit * min(current / setpoint, 1.0)` to
/// compute the control output.
#[derive(Clone, Debug, Default)]
pub struct SimpleBalancerController {
    bounds: BalancerControllerBounds,
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
        (self.bounds.output_limit() as f64 * ratio.clamp(0.0, 1.0)).floor() as u64
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
}
