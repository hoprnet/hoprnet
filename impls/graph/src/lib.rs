//! The library code containing the graph data structure for transport and incentivization layer (through the "channel
//! graph").
//!
//! [`NetworkGraph`] is the main data structure representing the network of nodes and channels. It combines 2 layers:
//! 1. The **channel graph** layer, which represents the network topology with nodes and channels as loaded from the
//!    chain.
//! 2. The **network layer**, which represents the nodes based on their physical connectability and QoS attributes.
//!
//! What does the graph look like:
//! - Nodes are represented as vertices in the graph.
//! - Possible connections, a combination of channel availability and or network usability are represented as edges
//!   between nodes.
//!
//! ## Weights
//! The weights accumulate different properties of the edges to represent the cost of using that edge for routing or
//! whether the edge can be used at all. Weights are represented as a struct containing different fields, each
//! representing a different property of the edge. The used properties are:
//! - presence of incentivization channel with remaining balance (Option<Balance>)
//! - presence of peer for immediate direct network connection and its quality (Option<ImmediateQoS>)
//! - presence of intermediate connection through other nodes (Option<IntermediateQoS>)

#[cfg(feature = "petgraph")]
pub mod petgraph;

pub mod costs;
pub mod errors;
pub mod weight;

use hopr_api::OffchainPublicKey;
#[cfg(feature = "petgraph")]
pub use petgraph::*;
pub use weight::Observations;

/// A thread-safe, shareable handle to a [`ChannelGraph`].
///
/// This is a convenience alias. Since [`ChannelGraph`] uses interior mutability,
/// wrapping it in `Arc` is sufficient for concurrent sharing without an
/// external `RwLock`.
#[cfg(feature = "petgraph")]
pub type SharedChannelGraph = std::sync::Arc<ChannelGraph>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphNode(OffchainPublicKey);

impl From<GraphNode> for OffchainPublicKey {
    fn from(val: GraphNode) -> Self {
        val.0
    }
}
