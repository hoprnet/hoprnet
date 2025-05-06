mod controller;
mod rate_limiting;

use futures::stream::AbortHandle;
use hopr_crypto_packet::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use std::time::Duration;

pub use controller::{SurbBalancer, SurbBalancerConfig};
pub use rate_limiting::RateController;

use crate::balancer::rate_limiting::RateLimitExt;
use crate::errors::TransportSessionError;
use crate::initiation::StartProtocol;
use crate::traits::SendMsg;

/// Allows estimating the flow of SURBs in a Session (production or depletion).
#[cfg_attr(test, mockall::automock)]
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
#[cfg_attr(test, mockall::automock)]
pub trait SurbFlowController {
    /// Adjusts the amount of non-organic SURB flow.
    fn adjust_surb_flow(&self, surbs_per_sec: usize);
}

/// Allows dynamically controlling the rate at which the keep-alive messages are yielded from
/// an associated infinite stream.
///
/// This is done by wrapping the [`RateController`] to implement [`SurbFlowController`] for HOPR Keep-Alive messages
/// that bear SURBs.
pub(crate) struct KeepAliveController(RateController);

impl SurbFlowController for KeepAliveController {
    fn adjust_surb_flow(&self, surbs_per_sec: usize) {
        // Currently, a keep-alive message can bear `HoprPacket::MAX_SURBS_IN_PACKET` SURBs,
        // so the correction by this factor is applied.
        self.0.set_rate_per_unit(
            surbs_per_sec,
            HoprPacket::MAX_SURBS_IN_PACKET as u32 * Duration::from_secs(1),
        );
    }
}

/// Returns an infinite rate-limited stream of [`StartProtocol::KeepAlive`] messages and its [controller](KeepAliveController).
///
/// The stream is initialized with 0 rate (suspended), until a non-zero rate is set via the controller.
pub fn keep_alive_stream<S: Clone>(
    session_id: S,
) -> (
    impl futures::Stream<Item = StartProtocol<S>>,
    KeepAliveController,
    AbortHandle,
) {
    let elem = StartProtocol::KeepAlive(session_id);
    let (stream, abort_handle) = futures::stream::abortable(futures::stream::repeat(elem));
    let (stream, controller) = stream.rate_limit_per_unit(0, Duration::from_secs(1));

    (stream, KeepAliveController(controller), abort_handle)
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
        self.0
            .send_message(data, destination).await
            .inspect(|_| {
                self.1.fetch_add(num_surbs, std::sync::atomic::Ordering::Relaxed);
            })
    }
}
