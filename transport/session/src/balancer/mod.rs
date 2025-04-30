mod controller;
mod rate_limiting;

use hopr_crypto_packet::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;

pub use controller::{SurbBalancer, SurbBalancerConfig};
pub use rate_limiting::RateController;

use crate::balancer::rate_limiting::RateLimitExt;
use crate::errors::TransportSessionError;
use crate::initiation::StartProtocol;
use crate::traits::SendMsg;

/// Allows estimating the flow of SURBs in a Session (production or depletion).
pub trait SurbFlowEstimator {
    /// Estimates the number of SURBs produced or depleted, depending on the context.
    fn estimate_surb_turnout(&self) -> u64;
}

impl SurbFlowEstimator for std::sync::Arc<std::sync::atomic::AtomicU64> {
    fn estimate_surb_turnout(&self) -> u64 {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }
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
    pub fn set_rate(&self, rate: usize, unit: std::time::Duration) {
        self.0.set_rate_per_unit(rate, unit);
    }

    /// Gets the current rate (per unit) of keep-alive messages.
    pub fn rate(&self) -> (usize, std::time::Duration) {
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
    unit: std::time::Duration,
) -> (impl futures::Stream<Item = StartProtocol<S>>, KeepAliveController) {
    let elem = StartProtocol::KeepAlive(session_id);
    let (stream, abort_handle) = futures::stream::abortable(futures::stream::repeat(elem));
    let (stream, controller) = stream.rate_limit_per_unit(initial_msg_per_unit, unit);

    (stream, KeepAliveController(controller, abort_handle))
}

pub(crate) struct CountingSendMsg<T>(T, std::sync::Arc<std::sync::atomic::AtomicU64>);

impl<T: SendMsg> CountingSendMsg<T> {
    pub fn new(msg: T, counter: std::sync::Arc<std::sync::atomic::AtomicU64>) -> Self {
        Self(msg, counter)
    }
}

#[async_trait::async_trait]
impl<T: SendMsg + Send + Sync> SendMsg for CountingSendMsg<T> {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: DestinationRouting,
    ) -> Result<(), TransportSessionError> {
        let num_surbs = HoprPacket::max_surbs_with_message(data.len()) as u64;
        let res = self.0.send_message(data, destination).await;
        if res.is_err() {
            self.1.fetch_add(num_surbs, std::sync::atomic::Ordering::Relaxed);
        }
        res
    }
}
