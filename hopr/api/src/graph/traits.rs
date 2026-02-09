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

pub trait EdgeObservable {
    type ImmediateMeasurement: EdgeTransportObservable + Send;
    type IntermediateMeasurement: EdgeTransportObservable + Send;

    /// Record a new result of the probe over this path segment.
    fn record(&mut self, measurement: EdgeWeightType);

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
pub trait NetworkGraphWrite: NetworkGraphView {
    /// The error type returned by fallible write operations.
    type Error;

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
pub trait NetworkGraphUpdate {
    /// Update the observation for the telemetry.
    async fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: MeasurablePeer + Clone + Send + Sync + 'static,
        P: MeasurablePath + Clone + Send + Sync + 'static;

    /// Update the observation for the telemetry.
    async fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + Clone + Send + Sync + 'static;
}

/// A trait specifying the graph traversal functionality
#[async_trait::async_trait]
pub trait NetworkGraphTraverse {
    type NodeId: Send + Sync;

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

// --- Blanket `Arc<T>` implementations ---
//
// These allow any graph implementation wrapped in `Arc` to satisfy the trait
// bounds required by `Hopr` and `HoprTransport` (which need `Clone + Send + Sync`).

#[async_trait::async_trait]
impl<T> NetworkGraphView for std::sync::Arc<T>
where
    T: NetworkGraphView + Send + Sync,
{
    type NodeId = T::NodeId;
    type Observed = T::Observed;

    fn node_count(&self) -> usize {
        (**self).node_count()
    }

    fn contains_node(&self, key: &Self::NodeId) -> bool {
        (**self).contains_node(key)
    }

    fn nodes(&self) -> futures::stream::BoxStream<'static, Self::NodeId> {
        (**self).nodes()
    }

    fn has_edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> bool {
        (**self).has_edge(src, dest)
    }

    fn edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Option<Self::Observed> {
        (**self).edge(src, dest)
    }
}

impl<T> NetworkGraphWrite for std::sync::Arc<T>
where
    T: NetworkGraphWrite + Send + Sync,
{
    type Error = T::Error;

    fn add_node(&self, key: Self::NodeId) {
        (**self).add_node(key)
    }

    fn remove_node(&self, key: &Self::NodeId) {
        (**self).remove_node(key)
    }

    fn add_edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Result<(), Self::Error> {
        (**self).add_edge(src, dest)
    }
}

#[async_trait::async_trait]
impl<T> NetworkGraphUpdate for std::sync::Arc<T>
where
    T: NetworkGraphUpdate + Send + Sync,
{
    async fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: MeasurablePeer + Clone + Send + Sync + 'static,
        P: MeasurablePath + Clone + Send + Sync + 'static,
    {
        (**self).record_edge(update).await
    }

    async fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + Clone + Send + Sync + 'static,
    {
        (**self).record_node(update).await
    }
}

#[async_trait::async_trait]
impl<T> NetworkGraphTraverse for std::sync::Arc<T>
where
    T: NetworkGraphTraverse + Send + Sync,
{
    type NodeId = T::NodeId;

    async fn routes(&self, destination: &Self::NodeId, length: usize) -> Vec<DestinationRouting> {
        (**self).routes(destination, length).await
    }

    async fn loopback_routes(&self) -> Vec<Vec<DestinationRouting>> {
        (**self).loopback_routes().await
    }
}
