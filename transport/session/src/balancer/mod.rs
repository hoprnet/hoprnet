mod rate_limited_stream;
mod controller;

pub use controller::{SurbBalancer, SurbBalancerConfig};
pub use rate_limited_stream::RateController;

use crate::balancer::rate_limited_stream::RateLimitExt;
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
        self.set_rate(surbs_per_sec)
    }
}

/// Returns a rate-limited stream of [`StartProtocol::KeepAlive`] messages and the rate controller at which
/// this stream yields messages.
///
/// If `initial_msg_per_sec` is 0, the no rate limiting will apply.
pub fn keep_alive_stream<S: Clone>(session_id: S, initial_msg_per_sec: usize) -> (impl futures::Stream<Item = StartProtocol<S>>, RateController) {
    let elem = StartProtocol::KeepAlive(session_id);
    futures::stream::repeat(elem).rate_limit(initial_msg_per_sec)
}

