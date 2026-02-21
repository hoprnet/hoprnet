mod controller;
/// Contains implementation of the [`SurbBalancerController`] trait using a Proportional Integral Derivative (PID)
/// controller.
pub mod pid;
#[allow(dead_code)]
mod rate_limiting;
/// Contains a simple proportional output implementation of the [`SurbBalancerController`] trait.
pub mod simple;

pub use controller::{SurbBalancer, SurbBalancerConfig, UpdatableSurbBalancerConfig};
pub use rate_limiting::{RateController, RateLimitSinkExt, RateLimitStreamExt};

/// Smallest possible interval for balancer sampling.
pub const MIN_BALANCER_SAMPLING_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

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

    /// Subtracts SURBs produced from consumed, saturating at zero.
    fn saturating_diff(&self) -> u64 {
        self.estimate_surbs_produced()
            .saturating_sub(self.estimate_surbs_consumed())
    }

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

/// Represents the setpoint (target) and output limit of a controller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BalancerControllerBounds(u64, u64);

impl BalancerControllerBounds {
    /// Creates a new instance.
    pub fn new(target: u64, output_limit: u64) -> Self {
        Self(target, output_limit)
    }

    /// Gets the target (setpoint) of a controller.
    #[inline]
    pub fn target(&self) -> u64 {
        self.0
    }

    /// Gets the output limit of a controller.
    #[inline]
    pub fn output_limit(&self) -> u64 {
        self.1
    }

    /// Unpacks the controller bounds into two `u64`s (target and output limit).
    #[inline]
    pub fn unzip(&self) -> (u64, u64) {
        (self.0, self.1)
    }
}

/// Trait abstracting a controller used in the [`SurbBalancer`].
pub trait SurbBalancerController {
    /// Gets the current bounds of the controller.
    fn bounds(&self) -> BalancerControllerBounds;
    /// Updates the controller's target (setpoint) and output limit.
    fn set_target_and_limit(&mut self, bounds: BalancerControllerBounds);
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

/// Wraps a [`RateController`] as [`SurbFlowController`] with the given correction
/// factor on time unit.
///
/// For example, when this is used to control the flow of keep-alive messages (carrying SURBs),
/// the correction factor is `HoprPacket::MAX_SURBS_IN_PACKET` - which is the number of SURBs
/// a single keep-alive message can bear.
///
/// In another case, when this is used to control the egress of a Session, each outgoing packet
/// consumes only a single SURB and therefore the correction factor is `1`.
pub struct SurbControllerWithCorrection(pub RateController, pub u32);

impl SurbFlowController for SurbControllerWithCorrection {
    fn adjust_surb_flow(&self, surbs_per_sec: usize) {
        self.0
            .set_rate_per_unit(surbs_per_sec, self.1 * std::time::Duration::from_secs(1));
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
