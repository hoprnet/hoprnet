use hopr_network_types::types::DestinationRouting;

use super::{MeasurableNeighbor, MeasurablePath, NetworkGraphError, Telemetry};

pub trait Observable {
    /// Record a new result of the probe towards the measured peer.
    fn record_probe(&mut self, latency: std::result::Result<std::time::Duration, ()>);

    /// The timestamp of the last update.
    fn last_update(&self) -> std::time::Duration;

    /// Return average latency observed for the measured peer.
    fn average_latency(&self) -> Option<std::time::Duration>;

    /// A value representing the average success rate of probes.
    ///
    /// It is from the range [0.0, 1.0]. The higher the value, the better the score.
    fn average_probe_rate(&self) -> f64;

    /// A value scoring the observed peer.
    ///
    /// It is from the range [0.0, 1.0]. The higher the value, the better the score.
    fn score(&self) -> f64;
}

// TODO: with final implementation the return objects should be back-linked
// pieces of code that can interact with the graph directly and therefore be
// updated when the graph changes.
//
// The final implementation should also return nicely wrapped BoxStream objects
// instead of Vecs, i.e. it should return a generator that could be efficiently
// and fast polled.

/// A trait specifying the graph traversal functionality
#[async_trait::async_trait]
pub trait NetworkGraphView {
    /// The concrete type of observations for peers.
    type Observed: Observable + Send;
    type NodeId: Send;

    /// Returns a stream of all known nodes in the network graph.
    fn nodes(&self) -> futures::stream::BoxStream<'static, Self::NodeId>;

    /// Returns the weight represented by the observations for the edge between the
    /// given source and destination, if available.
    fn edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Option<Self::Observed>;

    /// Returns a list of all routes to the given destination of the specified length.
    ///
    /// NOTE(20260204): for future usage in path planning this should contain a referencable
    /// object that can be updated whenever the graph changes instead of a static snapshot.
    async fn routes(&self, destination: &Self::NodeId, length: usize) -> Vec<DestinationRouting>;

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
