mod controller;
mod rate_limiting;

pub use controller::{SurbBalancer, SurbBalancerConfig};
pub use rate_limiting::RateController;
use std::time::Duration;

use crate::balancer::rate_limiting::RateLimitExt;
use crate::initiation::StartProtocol;

/// Allows estimating the outflow of SURBs in a Session.
pub trait SurbOutflowEstimator {
    /// Estimates the number of SURBs sent.
    fn estimate_sent_surbs(&self) -> u64;
}

/// Allows estimating the inflow of SURBs in a Session.
pub trait SurbInflowEstimator {
    /// Estimates the number of SURBs used by the counterparty.
    fn estimate_used_surbs(&self) -> u64;
}

/// Allows controlling the flow of non-organic SURBs in a Session.
pub trait SurbFlowController {
    /// Adjusts the amount of non-organic SURB flow.
    fn adjust_surb_flow(&self, surbs_per_sec: usize);
}

impl SurbFlowController for RateController {
    fn adjust_surb_flow(&self, surbs_per_sec: usize) {
        self.set_rate_per_sec(surbs_per_sec)
    }
}

/// Allows dynamically controlling the rate at which the keep-alive messages are yielded from
/// an associated infinite stream, and to abort the stream.
///
/// See [`keep_alive_stream`].
pub struct KeepAliveController(RateController, futures::stream::AbortHandle);

impl KeepAliveController {
    /// Aborts the stream, so it will no longer yield any messages.
    pub fn abort(&self) {
        self.1.abort();
    }

    /// Sets the desired rate (per time unit) of keep-alive messages.
    ///
    /// No rate-limiting will apply if 0 is given.
    pub fn set_rate(&self, rate: usize, unit: Duration) {
        self.0.set_rate_per_unit(rate, unit);
    }

    /// Gets the current rate (per unit) of keep-alive messages.
    pub fn rate(&self) -> (usize, Duration) {
        self.0.get_rate_per_unit()
    }

    /// Consumes self and returns the rate controller and abort handle separately.
    pub fn split(self) -> (RateController, futures::stream::AbortHandle) {
        (self.0, self.1)
    }
}

/// Returns an infinite rate-limited stream of [`StartProtocol::KeepAlive`] messages and its [controller](KeepAliveController).
///
/// If `initial_msg_per_sec` is 0, the no rate limiting will apply.
pub fn keep_alive_stream<S: Clone>(
    session_id: S,
    initial_msg_per_unit: usize,
    unit: Duration,
) -> (impl futures::Stream<Item = StartProtocol<S>>, KeepAliveController) {
    let elem = StartProtocol::KeepAlive(session_id);
    let (stream, abort_handle) = futures::stream::abortable(futures::stream::repeat(elem));
    let (stream, controller) = stream.rate_limit_per_unit(initial_msg_per_unit, unit);

    (stream, KeepAliveController(controller, abort_handle))
}
