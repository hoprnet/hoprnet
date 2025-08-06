mod controller;
/// Contains implementation of the [`SurbBalancerController`] trait using a Proportional Integral Derivative (PID)
/// controller.
pub mod pid;
#[allow(dead_code)]
mod rate_limiting;
/// Contains simple proportional output implementation of the [`SurbBalancerController`] trait.
pub mod simple;

pub use controller::{SurbBalancer, SurbBalancerConfig};
pub use rate_limiting::{RateController, RateLimitSinkExt, RateLimitStreamExt};

/// Allows estimating the flow of SURBs in a Session (production or consumption).
pub trait SurbFlowEstimator {
    /// Estimates the number of SURBs consumed.
    ///
    /// Value returned on each call must be equal or greater to the value returned by a previous call.
    fn estimate_surbs_consumed(&self) -> u64;
    /// Estimates the number of SURBs produced or received.
    ///
    /// Value returned on each call must be equal or greater to the value returned by a previous call.
    fn estimate_surbs_produced(&self) -> u64;

    /// Computes the estimated change in SURB buffer.
    ///
    /// This is done by computing the change in produced and consumed SURBs since the `earlier`
    /// state and then taking their difference.
    ///
    /// A positive result is a surplus number of SURBs added to the buffer, a negative result is a loss of SURBs
    /// from the buffer.
    /// Returns `None` if `earlier` had more SURBs produced/consumed than this instance (overflow).
    fn estimated_surb_buffer_change<E: SurbFlowEstimator>(&self, earlier: &E) -> Option<i64> {
        match (
            self.estimate_surbs_produced()
                .checked_sub(earlier.estimate_surbs_produced()),
            self.estimate_surbs_consumed()
                .checked_sub(earlier.estimate_surbs_consumed()),
        ) {
            (Some(surbs_delivered_delta), Some(surbs_consumed_delta)) => {
                Some(surbs_delivered_delta as i64 - surbs_consumed_delta as i64)
            }
            _ => None,
        }
    }
}

/// Allows controlling the production or consumption of SURBs in a Session.
#[cfg_attr(test, mockall::automock)]
pub trait SurbFlowController {
    /// Adjusts the amount of SURB production or consumption.
    fn adjust_surb_flow(&self, surbs_per_sec: usize);
}

/// Trait abstracting a controller used in the [`SurbBalancer`].
pub trait SurbBalancerController {
    /// Gets the current target (setpoint).
    fn target(&self) -> u64;
    /// Gets the current output limit.
    fn output_limit(&self) -> u64;
    /// Updates the controller's target (setpoint) and output limit.
    fn set_target_and_limit(&mut self, target: u64, output_limit: u64);
    /// Queries the controller for the next control output based on the `current_buffer_level` of SURBs.
    fn next_control_output(&mut self, current_buffer_level: u64) -> u64;
}

/// Implementation of [`SurbFlowEstimator`] that tracks the number of produced
/// and consumed SURBs via two `u64`s.
///
/// This implementation can take "snapshots" of other `SurbFlowEstimators` (via `From` trait) by simply
/// calling their respective methods to fill in its values.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimpleSurbFlowEstimator {
    /// Number of produced SURBs.
    pub produced: u64,
    /// Number of consumed SURBs.
    pub consumed: u64,
}

impl<T: SurbFlowEstimator> From<&T> for SimpleSurbFlowEstimator {
    fn from(value: &T) -> Self {
        Self {
            produced: value.estimate_surbs_produced(),
            consumed: value.estimate_surbs_consumed(),
        }
    }
}

impl SurbFlowEstimator for SimpleSurbFlowEstimator {
    fn estimate_surbs_consumed(&self) -> u64 {
        self.consumed
    }

    fn estimate_surbs_produced(&self) -> u64 {
        self.produced
    }
}

/// An implementation of [`SurbFlowEstimator`] that tracks the number of produced
/// and consumed SURBs via two `AtomicU64`s.
#[derive(Clone, Debug, Default)]
pub struct AtomicSurbFlowEstimator {
    /// Number of consumed SURBs.
    pub consumed: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Number of produced or received SURBs.
    pub produced: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl SurbFlowEstimator for AtomicSurbFlowEstimator {
    fn estimate_surbs_consumed(&self) -> u64 {
        self.consumed.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn estimate_surbs_produced(&self) -> u64 {
        self.produced.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimated_surb_buffer_change() {
        let estimator_1 = SimpleSurbFlowEstimator {
            produced: 10,
            consumed: 5,
        };
        let estimator_2 = SimpleSurbFlowEstimator {
            produced: 15,
            consumed: 11,
        };
        let estimator_3 = SimpleSurbFlowEstimator {
            produced: 25,
            consumed: 16,
        };
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_1), Some(0));
        assert_eq!(estimator_2.estimated_surb_buffer_change(&estimator_1), Some(-1));
        assert_eq!(estimator_3.estimated_surb_buffer_change(&estimator_2), Some(5));
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_2), None);
    }
}
