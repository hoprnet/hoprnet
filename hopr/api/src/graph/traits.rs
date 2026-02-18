use futures::stream::BoxStream;

use super::{MeasurablePath, MeasurablePeer};
use crate::graph::{MeasurableEdge, MeasurableNode};

/// The result of a transport-level probe over a transport path segment.
///
/// Contains the measured latency on success, or a unit error on failure.
pub type EdgeTransportMeasurement = std::result::Result<std::time::Duration, ()>;

/// The capacity of a payment channel representing an average amount of messages remaining in the channel.
pub type Capacity = u128;

/// Represents the different kinds of observations that can be recorded for a graph edge.
pub enum EdgeWeightType {
    /// A direct transport measurement between this and another adjacent peer.
    Immediate(EdgeTransportMeasurement),
    /// A transport measurement relayed through an intermediate peer.
    Intermediate(EdgeTransportMeasurement),
    /// An update to the payment channel capacity along this edge.
    Capacity(Option<Capacity>),
    /// An update to the physical connectivity status of this edge.
    Connected(bool),
}

/// Trait for recording new observations onto a graph edge.
pub trait EdgeObservableWrite {
    /// Records a new measurement or status update for this edge.
    fn record(&mut self, measurement: EdgeWeightType);
}

/// Trait for reading network-level properties of an edge.
pub trait EdgeNetworkObservableRead {
    /// Whether this edge represents also an existing physical connection between the peers.
    ///
    /// This is obviously settable only between the emitter of the measurement (this node) and
    /// arbitrary other node in the graph, but could be used for optimizations and path planning.
    fn is_connected(&self) -> bool;
}

/// Trait for reading HOPR protocol-level properties of an edge.
pub trait EdgeProtocolObservable {
    /// Capacity present in the channel to send through this path segment using PoR of HOPR protocol.
    fn capacity(&self) -> Option<u128>;
}

/// Trait for reading aggregated quality-of-service observations from a graph edge.
pub trait EdgeObservableRead {
    /// Measurement type for direct (1-hop) probes, including network connectivity info.
    type ImmediateMeasurement: EdgeLinkObservable + EdgeNetworkObservableRead + Send;
    /// Measurement type for relayed probes through an intermediate, including channel capacity.
    type IntermediateMeasurement: EdgeLinkObservable + EdgeProtocolObservable + Send;

    /// The timestamp of the last update.
    fn last_update(&self) -> std::time::Duration;

    /// Transport level measurements between this node and any other node in the network.
    fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement>;

    /// Transport level measurements performed in a transparent mode using looping measurements.
    fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement>;

    /// A value scoring the observed peer.
    ///
    /// It is from the [0.0, 1.0] range. The higher the value, the better the score.
    fn score(&self) -> f64;
}

/// Combined trait for full read/write access to edge observations.
///
/// Automatically implemented for any type that implements both [`EdgeObservableRead`]
/// and [`EdgeObservableWrite`].
pub trait EdgeObservable: EdgeObservableRead + EdgeObservableWrite {}

impl<T: EdgeObservableWrite + EdgeObservableRead> EdgeObservable for T {}

/// Trait for recording and querying transport-level link quality metrics for a transport link.
pub trait EdgeLinkObservable {
    /// Records a new result of the probe over this path segment.
    fn record(&mut self, measurement: EdgeTransportMeasurement);

    /// Returns average latency observed for the measured peer.
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

/// Lifecycle events observed for a node in the network.
#[derive(Debug, Clone)]
pub enum NodeObservation<T> {
    /// The node was discovered in the network.
    Discovered(T),
    /// A direct connection to the node was established.
    Connected(T),
    /// The direct connection to the node was lost.
    Disconnected(T),
}

/// Trait for recording node lifecycle observations into the graph.
pub trait NodeObservable {
    /// The node identifier type that can be measured as a peer.
    type Node: MeasurablePeer + Send;

    /// Record a new observation for the given node.
    fn record_node(&mut self, observation: NodeObservation<Self::Node>);
}

/// A trait specifying read-only graph view functionality.
///
/// Provides methods to inspect the graph topology: node membership, node count,
/// edge existence, and edge observation retrieval.
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait NetworkGraphView {
    /// The concrete type of observations for peers.
    type Observed: EdgeObservable + Send;
    /// The identifier type used to reference nodes in the graph.
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
    /// The identifier type used to reference nodes in the graph.
    type NodeId: Send;

    /// Adds a node to the graph if it does not already exist.
    fn add_node(&self, key: Self::NodeId);

    /// Removes a node and all its associated edges from the graph.
    fn remove_node(&self, key: &Self::NodeId);

    /// Adds a directed edge between two existing nodes with default observations.
    ///
    /// Returns an error if either node is not present in the graph.
    fn add_edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Result<(), Self::Error>;

    /// Removes a directed edge between two nodes.
    ///
    /// If the edge does not exist, this operation has no effect.
    fn remove_edge(&self, src: &Self::NodeId, dest: &Self::NodeId);

    /// Updates an existing edge or inserts a new edge between two nodes.
    ///
    /// If the nodes do not exist, they are inserted into the graph.
    ///
    /// The provided closure `f` is applied to modify the edge's observations.
    /// If the edge already exists, its observations are updated.
    /// If the edge does not exist, it is created and the closure is applied.
    fn upsert_edge<F>(&self, src: &Self::NodeId, dest: &Self::NodeId, f: F)
    where
        F: FnOnce(&mut Self::Observed);
}

/// A trait for recording observed measurment updates to graph edges and nodes.
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait NetworkGraphUpdate {
    /// Records an edge measurement derived from network telemetry.
    fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: MeasurablePeer + Clone + Send + Sync + 'static,
        P: MeasurablePath + Clone + Send + Sync + 'static;

    /// Records a node observation derived from network telemetry.
    fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + Clone + Send + Sync + 'static;
}

/// A trait specifying the graph traversal functionality.
///
/// Provides methods for finding simple paths between nodes in the network graph.
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait NetworkGraphTraverse {
    /// The identifier type used to reference nodes in the graph.
    type NodeId: Send + Sync;
    /// The cost metric associated with a path, used for ordering routes.
    type Cost: PartialOrd + Send + Sync;

    /// Returns a stream of routes from the source to the destination with the specified length.
    ///
    /// The length argument specifies the number of edges in the graph, over which the path should
    /// be formed, i.e. source -> intermediate -> destination is 2 edges.
    ///
    /// The stream should only terminate, if there's no more routes available.
    fn simple_paths_stream(
        &self,
        source: &Self::NodeId,
        destination: &Self::NodeId,
        length: usize,
    ) -> BoxStream<'static, (Vec<Self::NodeId>, Self::Cost)>;

    /// Returns a list of routes from the source to the destination with the specified length
    /// at the time of calling.
    ///
    /// The length argument specifies the number of edges in the graph, over which the path should
    /// be formed, i.e. source -> intermediate -> destination is 2 edges.
    ///
    /// The take count argument should be set in case the graph is expected to be large enough
    /// to be traversed slowly.
    fn simple_paths(
        &self,
        source: &Self::NodeId,
        destination: &Self::NodeId,
        length: usize,
        take_count: Option<usize>,
    ) -> Vec<(Vec<Self::NodeId>, Self::Cost)>;
}
