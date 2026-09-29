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
    /// [`net_held`](Self::net_held) equal to what the buffer actually holds. Defaults to `0` for
    /// estimators that never observe eviction (the Entry, whose store does not overflow, and test
    /// mocks). Value returned on each call must be equal or greater to a previous call.
    fn estimate_surbs_evicted(&self) -> u64 {
        0
    }

    /// The buffer's net held count -- `produced - consumed - evicted` -- as one coherent value.
    ///
    /// This is the figure to read wherever correctness depends on consistency. The three counters are
    /// separate atomics, so a snapshot built by loading them one at a time can tear: read an eviction
    /// ahead of its matching production and the change comes out too negative, saturates at zero in the
    /// accumulator, and settles the level *above* true occupancy (the loss never telescopes back).
    /// [`AtomicSurbFlowEstimator`] backs this with a single atomic that cannot tear; the default here
    /// is the three-counter difference, for mocks and snapshots that stay consistent by construction.
    fn net_held(&self) -> i64 {
        self.estimate_surbs_produced() as i64
            - self.estimate_surbs_consumed() as i64
            - self.estimate_surbs_evicted() as i64
    }

    /// What the buffer actually holds: [`net_held`](Self::net_held) floored at zero.
    ///
    /// `allow(dead_code)`: only test callers on this branch, since the report reads `buffer_level`; kept
    /// as the tested held-count accessor.
    #[allow(dead_code)]
    fn saturating_diff(&self) -> u64 {
        self.net_held().max(0) as u64
    }

    /// The change in held SURBs since the `earlier` state: positive is a surplus added to the buffer,
    /// negative a loss (consumption or eviction outrunning production).
    ///
    /// One coherent subtraction over [`net_held`](Self::net_held). No per-counter overflow guard is
    /// needed -- a single held counter cannot read non-monotonically -- and a negative result is a
    /// legitimate drain, not an error to discard.
    fn estimated_surb_buffer_change<E: SurbFlowEstimator>(&self, earlier: &E) -> i64 {
        self.net_held() - earlier.net_held()
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
    /// Number of produced SURBs (observability).
    pub produced: u64,
    /// Number of consumed SURBs (observability).
    pub consumed: u64,
    /// Number of SURBs evicted from the buffer on overflow (observability).
    pub evicted: u64,
    /// Coherent held count captured at snapshot time; see [`SurbFlowEstimator::net_held`]. Carried
    /// separately from the three counters above so it stays consistent even when they were read torn.
    net: i64,
}

impl SimpleSurbFlowEstimator {
    /// Builds a snapshot whose net is consistent with its counters.
    #[cfg(test)]
    fn from_counts(produced: u64, consumed: u64, evicted: u64) -> Self {
        Self {
            produced,
            consumed,
            evicted,
            net: produced as i64 - consumed as i64 - evicted as i64,
        }
    }
}

impl<T: SurbFlowEstimator> From<&T> for SimpleSurbFlowEstimator {
    fn from(value: &T) -> Self {
        Self {
            produced: value.estimate_surbs_produced(),
            consumed: value.estimate_surbs_consumed(),
            evicted: value.estimate_surbs_evicted(),
            net: value.net_held(),
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

    fn net_held(&self) -> i64 {
        self.net
    }
}

/// An implementation of `SurbFlowEstimator` backed by atomics.
///
/// `net` is the authoritative held count and the only field correctness-critical readers should
/// consult (via the `net_held` accessor): being a single atomic, a snapshot of it
/// cannot tear. The three counters beside it are cumulative totals kept for observability (telemetry
/// gauges, the balancer trace, cross-node conservation checks) and may read slightly inconsistent
/// with each other and with `net` under concurrency -- which is why nothing that must be correct reads
/// them. Every mutation goes through the `record_*` methods so `net` stays in step; do not write the
/// counters directly.
#[derive(Clone, Debug, Default)]
pub struct AtomicSurbFlowEstimator {
    /// Number of consumed SURBs (observability). Private: read via
    /// [`estimate_surbs_consumed`](SurbFlowEstimator::estimate_surbs_consumed), written only through the
    /// `record_*` methods, so callers cannot mutate it out of step with `net`.
    consumed: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Number of produced or received SURBs (observability). Private; see `consumed`.
    produced: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Number of received SURBs the buffer evicted on overflow (observability). Private; see `consumed`.
    evicted: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Coherent held count: `produced - consumed - evicted`, maintained as a single atomic.
    net: std::sync::Arc<std::sync::atomic::AtomicI64>,
}

impl AtomicSurbFlowEstimator {
    /// Books an incoming packet's SURBs: `saved` entered the store, `evicted` older ones were dropped
    /// to make room. The net change to what the store holds is `saved - evicted`.
    ///
    /// The `evicted` add is guarded because eviction only happens once the store is already full, so
    /// the per-packet common case is `evicted == 0`: skipping the write there keeps the `evicted`
    /// cache line clean for the balancer's periodic read instead of dirtying it on every packet.
    pub fn record_incoming(&self, saved: u64, evicted: u64) {
        self.produced.fetch_add(saved, std::sync::atomic::Ordering::Relaxed);
        if evicted != 0 {
            self.evicted.fetch_add(evicted, std::sync::atomic::Ordering::Relaxed);
        }
        self.net
            .fetch_add(saved as i64 - evicted as i64, std::sync::atomic::Ordering::Relaxed);
    }

    /// Books `n` SURBs produced/sent into the counterparty's store.
    pub fn record_produced(&self, n: u64) {
        self.produced.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        self.net.fetch_add(n as i64, std::sync::atomic::Ordering::Relaxed);
    }

    /// Books `n` SURBs consumed (spent from the store).
    pub fn record_consumed(&self, n: u64) {
        self.consumed.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        self.net.fetch_sub(n as i64, std::sync::atomic::Ordering::Relaxed);
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

    fn net_held(&self) -> i64 {
        self.net.load(std::sync::atomic::Ordering::Relaxed)
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
        let estimator_1 = SimpleSurbFlowEstimator::from_counts(10, 5, 0);
        let estimator_2 = SimpleSurbFlowEstimator::from_counts(15, 11, 0);
        let estimator_3 = SimpleSurbFlowEstimator::from_counts(25, 16, 0);
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_1), 0);
        assert_eq!(estimator_2.estimated_surb_buffer_change(&estimator_1), -1);
        assert_eq!(estimator_3.estimated_surb_buffer_change(&estimator_2), 5);
        // A held count that fell since `earlier` is a valid drain, not the discarded overflow it was.
        assert_eq!(estimator_1.estimated_surb_buffer_change(&estimator_2), 1);
    }

    /// An evicted SURB has left the buffer, so the held count -- what the buffer actually holds --
    /// must subtract evictions as well as consumption. Received 100, spent 20, and the full store
    /// dropped 50 on arrival leaves 30 held, not 80.
    #[test]
    fn saturating_diff_subtracts_evicted_surbs() {
        assert_eq!(SimpleSurbFlowEstimator::from_counts(100, 20, 50).saturating_diff(), 30);

        let atomic = AtomicSurbFlowEstimator::default();
        atomic.record_incoming(100, 50);
        atomic.record_consumed(20);
        assert_eq!(atomic.saturating_diff(), 30);
        // The gross totals stay available for observability.
        assert_eq!(atomic.estimate_surbs_produced(), 100);
        assert_eq!(atomic.estimate_surbs_consumed(), 20);
        assert_eq!(atomic.estimate_surbs_evicted(), 50);
    }

    /// Eviction can drive the held count to zero: after an overflow whose surplus is later drained,
    /// consumed + evicted can meet produced, and the buffer is empty rather than "still full".
    #[test]
    fn saturating_diff_saturates_when_evicted_and_consumed_exceed_produced() {
        assert_eq!(SimpleSurbFlowEstimator::from_counts(40, 25, 25).saturating_diff(), 0);
    }

    /// The per-tick buffer-change delta the balancer accumulates must treat evictions as SURBs leaving
    /// the buffer, or the accumulated level drifts above the real occupancy by every SURB the store
    /// ever evicted. A tick that receives 10 while the full store evicts 8 nets +2, not +10.
    #[test]
    fn estimated_surb_buffer_change_subtracts_evicted_delta() {
        let earlier = SimpleSurbFlowEstimator::from_counts(100, 50, 30);
        let later = SimpleSurbFlowEstimator::from_counts(110, 50, 38);
        assert_eq!(later.estimated_surb_buffer_change(&earlier), 2);

        // A full store receiving 10 more evicts all 10: the level does not move.
        let full_store = SimpleSurbFlowEstimator::from_counts(120, 50, 48);
        assert_eq!(full_store.estimated_surb_buffer_change(&later), 0);
    }

    /// CR-1 resolved: the held count is a single atomic, so a delivery that evicts is booked as one
    /// coherent change. There is no window in which the eviction is visible without its matching
    /// production, which is what previously let a torn snapshot read the change too negative, saturate
    /// at zero, and settle the level above true occupancy. A packet delivering 10 into a full store
    /// that evicts 8 moves the net by exactly +2 -- never transiently by -8.
    #[test]
    fn record_incoming_books_delivery_and_eviction_as_one_coherent_change() {
        let est = AtomicSurbFlowEstimator::default();
        let base = SimpleSurbFlowEstimator::from(&est);

        est.record_incoming(10, 8);

        let after = SimpleSurbFlowEstimator::from(&est);
        assert_eq!(after.estimated_surb_buffer_change(&base), 2);
        assert_eq!(est.net_held(), 2);
        assert_eq!(est.estimate_surbs_produced(), 10, "gross delivery is still counted");
        assert_eq!(est.estimate_surbs_evicted(), 8, "gross eviction is still counted");
    }
}
