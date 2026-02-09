use hopr_network_types::types::DestinationRouting;
use hopr_primitive_types::prelude::{Balance, WxHOPR};

use super::{MeasurableNeighbor, MeasurablePath, NetworkGraphError, Telemetry};

pub type EdgeTransportMeasurement = std::result::Result<std::time::Duration, ()>;

pub enum EdgeWeightType {
    Immediate(EdgeTransportMeasurement),
    Intermediate(EdgeTransportMeasurement),
}

pub trait EdgeObservable {
    type ImmediateMeasurement: EdgeTransportObservable + Send;
    type IntermediateMeasurement: EdgeTransportObservable + Send;

    /// Record a new result of the probe over this path segment.
    fn record(&mut self, measurement: EdgeWeightType);

    /// The timestamp of the last update.
    fn last_update(&self) -> std::time::Duration;

    /// Balance present in the virtual channel
    fn balance(&self) -> Option<&Balance<WxHOPR>>;

    /// Transport level measurements performed over the path segment exposed.
    fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement>;

    /// Transport level measurements performed over the path segment hidden through an intermediate.
    fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement>;

    /// A value scoring the observed peer.
    ///
    /// It is from the range [0.0, 1.0]. The higher the value, the better the score.
    fn score(&self) -> f64;
}

pub trait EdgeTransportObservable {
    /// Record a new result of the probe over this path segment.
    fn record(&mut self, measurement: EdgeTransportMeasurement);

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

pub enum NodeObservation<T> {
    Discovered(T),
    Connected(T),
    Disconnected(T),
}

pub trait NodeObservable {
    type Node: MeasurableNeighbor + Send;

    /// Record a new observation for the given node.
    fn record(&mut self, observation: NodeObservation<Self::Node>);
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
    type Observed: EdgeObservable + Send;
    type NodeId: Send;

    /// Returns a stream of all known nodes in the network graph.
    fn nodes(&self) -> futures::stream::BoxStream<'static, Self::NodeId>;

    /// Returns the weight represented by the observations for the edge between the
    /// given source and destination, if available.
    fn edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Option<Self::Observed>;
}

/// A trait specifying the graph update functionality
#[async_trait::async_trait]
pub trait NetworkGraphUpdate {
    /// Update the observation for the telemetry.
    async fn record_edge<N, P>(&self, telemetry: std::result::Result<Telemetry<N, P>, NetworkGraphError<P>>)
    where
        N: MeasurableNeighbor + Clone + Send + Sync + 'static,
        P: MeasurablePath + Clone + Send + Sync + 'static;
}

/// A trait specifying the graph traversal functionality
#[async_trait::async_trait]
pub trait NetworkGraphTraverse {
    type NodeId: Send;

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
