use hopr_network_types::types::DestinationRouting;

use super::{MeasurablePath, MeasurablePeer};
use crate::graph::{MeasurableEdge, MeasurableNode};

pub type EdgeTransportMeasurement = std::result::Result<std::time::Duration, ()>;

pub type Capacity = u128;

pub enum EdgeObservation {
    Update(EdgeWeightType),
    Remove,
    Add(Capacity),
}

pub enum EdgeWeightType {
    Immediate(EdgeTransportMeasurement),
    Intermediate(EdgeTransportMeasurement),
    Capacity(Option<Capacity>),
}

pub trait EdgeObservableWrite {
    fn record(&mut self, measurement: EdgeWeightType);
}
pub trait EdgeObservableRead {
    type ImmediateMeasurement: EdgeTransportObservable + Send;
    type IntermediateMeasurement: EdgeTransportObservable + Send;

    /// The timestamp of the last update.
    fn last_update(&self) -> std::time::Duration;

    /// Capacity present in the channel to send through this path segment using PoR of HOPR protocol.
    fn capacity(&self) -> Option<u128>;

    /// Transport level measurements performed over the path segment exposed.
    fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement>;

    /// Transport level measurements performed over the path segment hidden through an intermediate.
    fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement>;

    /// A value scoring the observed peer.
    ///
    /// It is from the range [0.0, 1.0]. The higher the value, the better the score.
    fn score(&self) -> f64;
}

pub trait EdgeObservable: EdgeObservableRead + EdgeObservableWrite {}

impl<T: EdgeObservableWrite + EdgeObservableRead> EdgeObservable for T {}

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

#[derive(Debug, Clone)]
pub enum NodeObservation<T> {
    Discovered(T),
    Connected(T),
    Disconnected(T),
}

pub trait NodeObservable {
    type Node: MeasurablePeer + Send;

    /// Record a new observation for the given node.
    fn record_node(&mut self, observation: NodeObservation<Self::Node>);
}

// TODO: with final implementation the return objects should be back-linked
// pieces of code that can interact with the graph directly and therefore be
// updated when the graph changes.
//
// The final implementation should also return nicely wrapped BoxStream objects
// instead of Vecs, i.e. it should return a generator that could be efficiently
// and fast polled.

/// A trait specifying read-only graph view functionality.
///
/// Provides methods to inspect the graph topology: node membership, node count,
/// edge existence, and edge observation retrieval.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait NetworkGraphView {
    /// The concrete type of observations for peers.
    type Observed: EdgeObservable + Send;
    type NodeId: Send;

    /// Returns the number of nodes in the graph.
    fn node_count(&self) -> usize;

    /// Checks whether the graph contains the given node.
    fn contains_node(&self, key: &Self::NodeId) -> bool;

    /// Returns a stream of all known nodes in the network graph.
    fn nodes(&self) -> futures::stream::BoxStream<'static, Self::NodeId>;

    /// Checks whether a directed edge exists between two nodes.
    ///
    /// The default implementation delegates to [`edge`](Self::edge).
    fn has_edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> bool {
        self.edge(src, dest).is_some()
    }

    /// Returns the weight represented by the observations for the edge between the
    /// given source and destination, if available.
    fn edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Option<Self::Observed>;
}

/// A trait for mutating the graph topology.
///
/// Provides methods to add/remove nodes and add edges.
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait NetworkGraphWrite {
    /// The error type returned by fallible write operations.
    type Error;
    /// The concrete type of observations for peers.
    type Observed: EdgeObservable + Send;
    type NodeId: Send;

    /// Adds a node to the graph if it does not already exist.
    fn add_node(&self, key: Self::NodeId);

    /// Removes a node and all its associated edges from the graph.
    fn remove_node(&self, key: &Self::NodeId);

    /// Adds a directed edge between two existing nodes with default observations.
    ///
    /// Returns an error if either node is not present in the graph.
    fn add_edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Result<(), Self::Error>;
}

/// A trait specifying the graph update functionality
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait NetworkGraphUpdate {
    /// Update the observation for the telemetry.
    async fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: MeasurablePeer + std::fmt::Debug + Clone + Send + Sync + 'static,
        P: MeasurablePath + std::fmt::Debug + Clone + Send + Sync + 'static;

    /// Update the observation for the telemetry.
    async fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + std::fmt::Debug + Clone + Send + Sync + 'static;
}

/// A trait specifying the graph traversal functionality
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait NetworkGraphTraverse {
    type NodeId: Send + Sync;

    /// Returns a list of all routes to the given destination of the specified length.
    async fn simple_route(&self, destination: &Self::NodeId, length: usize) -> Vec<DestinationRouting>;
}
