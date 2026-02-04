use hopr_network_types::types::DestinationRouting;
use multiaddr::PeerId;

use super::{MeasurableNeighbor, MeasurablePath, NetworkGraphError, Telemetry};

/// A trait specifying the graph traversal functionality
#[async_trait::async_trait]
pub trait NetworkGraphView {
    /// Returns a stream of all known nodes in the network graph.
    fn nodes(&self) -> futures::stream::BoxStream<'static, PeerId>;

    /// Returns a list of all routes to the given destination of the specified length.
    async fn routes(&self, destination: &PeerId, length: usize) -> Vec<DestinationRouting>;

    /// Returns a list of batches of loopback routes. Each batch contains a set of routes
    /// that start and end at the same node, while also belonging to the same path discovery
    /// batch.
    async fn loopback_routes(&self) -> Vec<Vec<DestinationRouting>>;
}

/// A trait specifying the graph update functionality
#[async_trait::async_trait]
pub trait NetworkGraphUpdate {
    /// Update the observation for the telemetry.
    async fn record<N, P>(&self, telemetry: std::result::Result<Telemetry<N, P>, NetworkGraphError<P>>)
    where
        N: MeasurableNeighbor + Clone + Send + Sync + 'static,
        P: MeasurablePath + Clone + Send + Sync + 'static;
}
