mod controller;
#[allow(dead_code)]
mod rate_limiting;

pub use controller::{SurbBalancer, SurbBalancerConfig};
pub use rate_limiting::{RateController, RateLimitSinkExt, RateLimitStreamExt};

/// Allows estimating the flow of SURBs in a Session (production or depletion).
#[cfg_attr(test, mockall::automock)]
pub trait SurbFlowEstimator {
    /// Estimates the number of SURBs produced or depleted, depending on the context.
    fn estimate_surb_turnout(&self) -> u64;
}

impl SurbFlowEstimator for std::sync::Arc<std::sync::atomic::AtomicU64> {
    fn estimate_surb_turnout(&self) -> u64 {
        self.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Allows controlling the flow of non-organic SURBs in a Session.
#[cfg_attr(test, mockall::automock)]
pub trait SurbFlowController {
    /// Adjusts the amount of non-organic SURB flow.
    fn adjust_surb_flow(&self, surbs_per_sec: usize);
}
