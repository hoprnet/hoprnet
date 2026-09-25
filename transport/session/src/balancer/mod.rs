mod congestion;
mod controller;
/// Contains the paced-refill implementation of the [`SurbBalancerController`] trait, which drives
/// the Entry's keep-alive production.
pub mod paced;
/// Contains implementation of the [`SurbBalancerController`] trait using a Proportional Integral Derivative (PID)
/// controller.
pub mod pid;
#[allow(dead_code)]
mod rate_limiting;
/// Contains a simple proportional output implementation of the [`SurbBalancerController`] trait.
pub mod simple;

pub use controller::{BalancerStateValues, SurbBalancer, SurbBalancerConfig};
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

    /// Of the [produced](SurbFlowEstimator::estimate_surbs_produced) SURBs, how many were delivered
    /// by keep-alive messages rather than piggybacked on Session data.
    ///
    /// A subset of the produced count, never an addition to it. The distinction matters for the
    /// upstream budget: piggybacked SURBs ride in packets that are sent anyway, whereas every
    /// keep-alive is a whole packet sent for nothing but its SURBs.
    ///
    /// Defaults to zero for estimators that do not tell the two apart.
    fn estimate_keep_alive_surbs_produced(&self) -> u64 {
        0
    }

    /// How many keep-alive messages delivered the
    /// [keep-alive SURBs](SurbFlowEstimator::estimate_keep_alive_surbs_produced).
    ///
    /// Defaults to zero for estimators that do not tell the two apart.
    fn estimate_keep_alive_packets(&self) -> u64 {
        0
    }

    /// Subtracts SURBs consumed from SURBs produced, saturating at zero.
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

/// What a [`SurbBalancerController`] is told about the SURB buffer on each sample.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ControlInput {
    /// Estimated number of SURBs in the buffer.
    pub level: u64,
    /// SURBs per second the buffer loses beyond what reaches it without the controller's help,
    /// smoothed over recent samples.
    ///
    /// Consumption minus organic supply. Negative when SURBs piggybacked on Session data alone
    /// arrive faster than they are spent.
    pub net_consumption_per_sec: f64,
    /// Time elapsed since the previous sample.
    pub dt: std::time::Duration,
    /// The most the controller may output on this sample.
    ///
    /// Controllers also respect their own output limit, so this can only lower it.
    pub ceiling: u64,
}

#[cfg(test)]
impl ControlInput {
    /// An input carrying only a buffer level, for testing controllers that need nothing else.
    pub fn at_level(level: u64) -> Self {
        Self {
            level,
            ceiling: u64::MAX,
            ..Default::default()
        }
    }
}

/// Trait abstracting a controller used in the [`SurbBalancer`].
pub trait SurbBalancerController {
    /// Gets the current bounds of the controller.
    fn bounds(&self) -> BalancerControllerBounds;
    /// Updates the controller's target (setpoint) and output limit.
    fn set_target_and_limit(&mut self, bounds: BalancerControllerBounds);
    /// Queries the controller for the next control output given what is known about the buffer on
    /// this sample.
    fn next_control_output(&mut self, input: ControlInput) -> u64;
    /// Discards accumulated history, leaving the bounds intact.
    ///
    /// Used when the buffer estimate the controller has been acting on stops meaning what it meant
    /// -- crossing into or out of a period where the counterparty could not be observed at all.
    /// Carrying the error accumulated under the old regime into the new one makes the controller
    /// answer a question nobody asked.
    fn reset(&mut self);
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
    /// Of `produced`, the SURBs delivered by keep-alive messages.
    pub keep_alive_surbs: u64,
    /// Number of keep-alive messages that delivered `keep_alive_surbs`.
    pub keep_alive_packets: u64,
}

impl<T: SurbFlowEstimator> From<&T> for SimpleSurbFlowEstimator {
    fn from(value: &T) -> Self {
        Self {
            produced: value.estimate_surbs_produced(),
            consumed: value.estimate_surbs_consumed(),
            keep_alive_surbs: value.estimate_keep_alive_surbs_produced(),
            keep_alive_packets: value.estimate_keep_alive_packets(),
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

    fn estimate_keep_alive_surbs_produced(&self) -> u64 {
        self.keep_alive_surbs
    }

    fn estimate_keep_alive_packets(&self) -> u64 {
        self.keep_alive_packets
    }
}

/// An implementation of `SurbFlowEstimator` that tracks the number of produced
/// and consumed SURBs via two `AtomicU64`s.
#[derive(Clone, Debug, Default)]
pub struct AtomicSurbFlowEstimator {
    /// Number of consumed SURBs.
    pub consumed: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Number of produced or received SURBs.
    pub produced: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Of `produced`, the SURBs delivered by keep-alive messages (sent by the Entry, received by
    /// the Exit). Counted in addition to `produced`, never instead of it.
    pub keep_alive_surbs: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Number of keep-alive messages that delivered `keep_alive_surbs`.
    pub keep_alive_packets: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl AtomicSurbFlowEstimator {
    /// Attributes one keep-alive message carrying `surbs` SURBs.
    ///
    /// Only the attribution: the same SURBs must also be added to `produced`, which is where the
    /// buffer estimate reads them from.
    pub fn record_keep_alive(&self, surbs: u64) {
        self.keep_alive_surbs
            .fetch_add(surbs, std::sync::atomic::Ordering::Relaxed);
        self.keep_alive_packets
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl SurbFlowEstimator for AtomicSurbFlowEstimator {
    fn estimate_surbs_consumed(&self) -> u64 {
        self.consumed.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn estimate_surbs_produced(&self) -> u64 {
        self.produced.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn estimate_keep_alive_surbs_produced(&self) -> u64 {
        self.keep_alive_surbs.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn estimate_keep_alive_packets(&self) -> u64 {
        self.keep_alive_packets.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// The controller that drives the Entry's keep-alive production.
///
/// [Paced refill](paced::PacedRefillController) unless `HOPR_BALANCER_CONTROLLER=pid` selects the
/// [PID controller](pid::PidBalancerController) it replaced, kept as a rollback.
#[derive(Clone, Debug)]
pub enum EntryBalancerController {
    /// Consumption fed forward, refills paced. The default.
    Paced(paced::PacedRefillController),
    /// The previous per-sample PID controller.
    Pid(pid::PidBalancerController),
}

impl EntryBalancerController {
    /// Selects the controller from `HOPR_BALANCER_CONTROLLER`, and its parameters from the
    /// controller's own environment variables.
    pub fn from_env_or_default() -> Self {
        match std::env::var("HOPR_BALANCER_CONTROLLER") {
            Ok(v) if v.trim().eq_ignore_ascii_case("pid") => Self::Pid(pid::PidBalancerController::from_gains(
                pid::PidControllerGains::from_env_or_default(),
            )),
            _ => Self::Paced(paced::PacedRefillController::new(
                paced::PacedRefillParams::from_env_or_default(),
            )),
        }
    }
}

impl SurbBalancerController for EntryBalancerController {
    fn bounds(&self) -> BalancerControllerBounds {
        match self {
            Self::Paced(c) => c.bounds(),
            Self::Pid(c) => c.bounds(),
        }
    }

    fn set_target_and_limit(&mut self, bounds: BalancerControllerBounds) {
        match self {
            Self::Paced(c) => c.set_target_and_limit(bounds),
            Self::Pid(c) => c.set_target_and_limit(bounds),
        }
    }

    fn next_control_output(&mut self, input: ControlInput) -> u64 {
        match self {
            Self::Paced(c) => c.next_control_output(input),
            Self::Pid(c) => c.next_control_output(input),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Paced(c) => c.reset(),
            Self::Pid(c) => c.reset(),
        }
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
            ..Default::default()
        };
        let estimator_2 = SimpleSurbFlowEstimator {
            produced: 15,
            consumed: 11,
            ..Default::default()
        };
        let estimator_3 = SimpleSurbFlowEstimator {
            produced: 25,
            consumed: 16,
            ..Default::default()
        };
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_1), Some(0));
        assert_eq!(estimator_2.estimated_surb_buffer_change(&estimator_1), Some(-1));
        assert_eq!(estimator_3.estimated_surb_buffer_change(&estimator_2), Some(5));
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_2), None);
    }

    /// The balancer only ever sees snapshots, so an attribution the snapshot drops is invisible.
    #[test]
    fn a_snapshot_should_carry_the_keep_alive_attribution() {
        let estimator = AtomicSurbFlowEstimator::default();
        estimator.produced.fetch_add(7, std::sync::atomic::Ordering::Relaxed);
        estimator.record_keep_alive(2);
        estimator.record_keep_alive(2);

        let snapshot = SimpleSurbFlowEstimator::from(&estimator);
        assert_eq!(
            (7, 4, 2),
            (
                snapshot.produced,
                snapshot.keep_alive_surbs,
                snapshot.keep_alive_packets
            )
        );
    }
}
