mod controller;
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

    /// Estimates the number of received SURBs the buffer evicted on overflow.
    ///
    /// An evicted SURB has left the buffer exactly as a consumed one has; the difference is only that
    /// the full store dropped it on arrival instead of it being spent. Counting it keeps
    /// [`saturating_diff`](Self::saturating_diff) equal to what the buffer actually holds. Defaults to
    /// `0` for estimators that never observe eviction (the Entry, whose store does not overflow, and
    /// test mocks). Value returned on each call must be equal or greater to a previous call.
    fn estimate_surbs_evicted(&self) -> u64 {
        0
    }

    /// Subtracts SURBs consumed and evicted from SURBs produced, saturating at zero.
    ///
    /// Consumed and evicted SURBs have both left the buffer, so what remains is
    /// `produced - consumed - evicted` -- the count the buffer actually holds.
    ///
    /// `allow(dead_code)`: only test callers on this branch, since the report reads `buffer_level`; kept
    /// as the tested held-count accessor.
    #[allow(dead_code)]
    fn saturating_diff(&self) -> u64 {
        self.estimate_surbs_produced()
            .saturating_sub(self.estimate_surbs_consumed())
            .saturating_sub(self.estimate_surbs_evicted())
    }

    /// Computes the estimated change in SURB buffer.
    ///
    /// This is done by computing the change in produced and consumed SURBs since the `earlier`
    /// state and then taking their difference.
    ///
    /// A positive result is a surplus number of SURBs added to the buffer, a negative result is a loss of SURBs
    /// from the buffer.
    ///
    /// Evictions count as SURBs leaving the buffer alongside consumption: a tick that receives some
    /// SURBs while the full store drops others nets only what it actually retained. Without this the
    /// accumulated level drifts above the real occupancy by every SURB the store ever evicted.
    /// Returns `None` if `earlier` had more SURBs produced/consumed/evicted than this instance (overflow).
    fn estimated_surb_buffer_change<E: SurbFlowEstimator>(&self, earlier: &E) -> Option<i64> {
        match (
            self.estimate_surbs_produced()
                .checked_sub(earlier.estimate_surbs_produced()),
            self.estimate_surbs_consumed()
                .checked_sub(earlier.estimate_surbs_consumed()),
            self.estimate_surbs_evicted()
                .checked_sub(earlier.estimate_surbs_evicted()),
        ) {
            (Some(surbs_delivered_delta), Some(surbs_consumed_delta), Some(surbs_evicted_delta)) => {
                Some(surbs_delivered_delta as i64 - surbs_consumed_delta as i64 - surbs_evicted_delta as i64)
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
    /// Number of SURBs evicted from the buffer on overflow.
    pub evicted: u64,
}

impl<T: SurbFlowEstimator> From<&T> for SimpleSurbFlowEstimator {
    fn from(value: &T) -> Self {
        Self {
            produced: value.estimate_surbs_produced(),
            consumed: value.estimate_surbs_consumed(),
            evicted: value.estimate_surbs_evicted(),
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

    fn estimate_surbs_evicted(&self) -> u64 {
        self.evicted
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
    /// Number of received SURBs the buffer evicted on overflow.
    pub evicted: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl AtomicSurbFlowEstimator {
    /// Books an incoming packet's SURBs: `saved` entered the store, `evicted` older ones were dropped
    /// to make room. Both leave `produced - consumed - evicted` equal to what the store holds.
    ///
    /// The `evicted` add is guarded because eviction only happens once the store is already full, so
    /// the per-packet common case is `evicted == 0`: skipping the write there keeps the `evicted`
    /// cache line clean for the balancer's periodic read instead of dirtying it on every packet.
    pub fn record_incoming(&self, saved: u64, evicted: u64) {
        self.produced.fetch_add(saved, std::sync::atomic::Ordering::Relaxed);
        if evicted != 0 {
            self.evicted.fetch_add(evicted, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl SurbFlowEstimator for AtomicSurbFlowEstimator {
    fn estimate_surbs_consumed(&self) -> u64 {
        self.consumed.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn estimate_surbs_produced(&self) -> u64 {
        self.produced.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn estimate_surbs_evicted(&self) -> u64 {
        self.evicted.load(std::sync::atomic::Ordering::Relaxed)
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
            evicted: 0,
        };
        let estimator_2 = SimpleSurbFlowEstimator {
            produced: 15,
            consumed: 11,
            evicted: 0,
        };
        let estimator_3 = SimpleSurbFlowEstimator {
            produced: 25,
            consumed: 16,
            evicted: 0,
        };
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_1), Some(0));
        assert_eq!(estimator_2.estimated_surb_buffer_change(&estimator_1), Some(-1));
        assert_eq!(estimator_3.estimated_surb_buffer_change(&estimator_2), Some(5));
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_2), None);
    }

    /// An evicted SURB has left the buffer, so `saturating_diff` -- what the buffer actually holds --
    /// must subtract evictions as well as consumption. Received 100, spent 20, and the full store
    /// dropped 50 on arrival leaves 30 held, not 80.
    #[test]
    fn saturating_diff_subtracts_evicted_surbs() {
        let estimator = SimpleSurbFlowEstimator {
            produced: 100,
            consumed: 20,
            evicted: 50,
        };
        assert_eq!(estimator.saturating_diff(), 30);

        let atomic = AtomicSurbFlowEstimator::default();
        atomic.produced.store(100, std::sync::atomic::Ordering::Relaxed);
        atomic.consumed.store(20, std::sync::atomic::Ordering::Relaxed);
        atomic.evicted.store(50, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(atomic.saturating_diff(), 30);
    }

    /// Eviction can drive the held count to zero: after an overflow whose surplus is later drained,
    /// consumed + evicted can meet produced, and the buffer is empty rather than "still full".
    #[test]
    fn saturating_diff_saturates_when_evicted_and_consumed_exceed_produced() {
        let estimator = SimpleSurbFlowEstimator {
            produced: 40,
            consumed: 25,
            evicted: 25,
        };
        assert_eq!(estimator.saturating_diff(), 0);
    }

    /// The per-tick buffer-change delta the balancer accumulates must also treat evictions as SURBs
    /// leaving the buffer, or the accumulated level drifts above the real occupancy by every SURB the
    /// store ever evicted. A tick that receives 10 while the full store evicts 8 nets +2, not +10.
    #[test]
    fn estimated_surb_buffer_change_subtracts_evicted_delta() {
        let earlier = SimpleSurbFlowEstimator {
            produced: 100,
            consumed: 50,
            evicted: 30,
        };
        let later = SimpleSurbFlowEstimator {
            produced: 110,
            consumed: 50,
            evicted: 38,
        };
        assert_eq!(later.estimated_surb_buffer_change(&earlier), Some(2));

        // A full store receiving 10 more evicts all 10: the level does not move.
        let full_store = SimpleSurbFlowEstimator {
            produced: 120,
            consumed: 50,
            evicted: 48,
        };
        assert_eq!(full_store.estimated_surb_buffer_change(&later), Some(0));
    }
}
