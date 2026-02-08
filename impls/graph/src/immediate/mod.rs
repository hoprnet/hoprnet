use std::sync::Arc;

use hopr_api::{PeerId, ct::DestinationRouting, graph::traits::EdgeObservable, network::NetworkView};
use rand::seq::SliceRandom;

use crate::{immediate::tracker::NetworkPeerTracker, weight::Observations};

pub mod tracker;

#[derive(Clone)]
pub struct ImmediateNeighborChannelGraph<T> {
    network: Arc<T>,
    tracker: NetworkPeerTracker,
    recheck_threshold: std::time::Duration,
}

impl<T> ImmediateNeighborChannelGraph<T> {
    pub fn new(network: T, recheck_threshold: std::time::Duration) -> Self {
        Self {
            network: Arc::new(network),
            tracker: NetworkPeerTracker::new(),
            recheck_threshold,
        }
    }
}

#[async_trait::async_trait]
impl<T> hopr_api::graph::NetworkGraphUpdate for ImmediateNeighborChannelGraph<T>
where
    T: NetworkView + Send + Sync,
{
    async fn record_edge<N, P>(
        &self,
        telemetry: std::result::Result<hopr_api::graph::Telemetry<N, P>, hopr_api::graph::NetworkGraphError<P>>,
    ) where
        N: hopr_api::graph::MeasurableNeighbor + Send + Clone,
        P: hopr_api::graph::MeasurablePath + Send + Clone,
    {
        match telemetry {
            Ok(hopr_api::graph::Telemetry::Neighbor(telemetry)) => {
                tracing::trace!(
                    peer = %telemetry.peer(),
                    latency_ms = telemetry.rtt().as_millis(),
                    "neighbor probe successful"
                );
                self.tracker.alter(telemetry.peer(), |_peer, mut observation| {
                    observation.record(hopr_api::graph::traits::EdgeWeightType::Immediate(Ok(
                        telemetry.rtt() / 2
                    )));
                    observation
                });
            }
            Ok(hopr_api::graph::Telemetry::Loopback(_)) => {
                tracing::warn!(
                    reason = "feature not implemented",
                    "loopback path telemetry not supported"
                );
            }
            Err(hopr_api::graph::NetworkGraphError::ProbeNeighborTimeout(peer)) => {
                tracing::trace!(
                    peer = %peer,
                    reason = "probe timeout",
                    "neighbor probe failed"
                );
                self.tracker.alter(&peer, |_peer, mut observation| {
                    observation.record(hopr_api::graph::traits::EdgeWeightType::Immediate(Err(())));
                    observation
                });
            }
            Err(hopr_api::graph::NetworkGraphError::ProbeLoopbackTimeout(_)) => {
                tracing::warn!(
                    reason = "feature not implemented",
                    "loopback path telemetry not supported"
                );
            }
        }
    }
}

#[async_trait::async_trait]
impl<T> hopr_api::graph::NetworkGraphView for ImmediateNeighborChannelGraph<T>
where
    T: NetworkView + Send + Sync + Clone + 'static,
{
    type NodeId = PeerId;
    type Observed = Observations;

    fn nodes(&self) -> futures::stream::BoxStream<'static, Self::NodeId> {
        let fetcher = self.network.clone();
        let _recheck_threshold = self.recheck_threshold; // TODO: currently being ignored
        let mut rng = hopr_crypto_random::rng();

        Box::pin(async_stream::stream! {
            let mut peers: Vec<PeerId> = fetcher.discovered_peers().into_iter().collect();
            peers.shuffle(&mut rng);    // shuffle peers to randomize order between rounds

            for peer in peers {
                yield peer;
            }
        })
    }

    fn edge(&self, _src: &Self::NodeId, dest: &Self::NodeId) -> Option<Self::Observed> {
        if self.network.is_connected(dest) {
            self.tracker.get(dest)
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl<T> hopr_api::graph::NetworkGraphTraverse for ImmediateNeighborChannelGraph<T>
where
    T: NetworkView + Send + Sync + Clone + 'static,
{
    type NodeId = PeerId;

    async fn routes(&self, _destination: &Self::NodeId, _length: usize) -> Vec<DestinationRouting> {
        vec![]
    }

    async fn loopback_routes(&self) -> Vec<Vec<DestinationRouting>> {
        vec![]
    }
}
