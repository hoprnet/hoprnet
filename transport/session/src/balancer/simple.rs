use crate::balancer::SurbBalancerController;

/// Controller that uses the simple linear formula `limit * min(current / setpoint, 1.0)` to
/// compute the control output.
#[derive(Clone, Debug, Default)]
pub struct SimpleBalancerController {
    setpoint: u64,
    limit: u64,
}

impl SurbBalancerController for SimpleBalancerController {
    fn set_target_and_limit(&mut self, target: u64, output_limit: u64) {
        self.limit = output_limit;
        self.setpoint = target;
    }

    fn next_control_output(&mut self, current_buffer_level: u64) -> u64 {
        let ratio = (current_buffer_level as f64 / self.setpoint as f64).clamp(0.0, 1.0);

        (self.limit as f64 * ratio).floor() as u64
    }
}
