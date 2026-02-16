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

pub mod errors;
pub mod weight;

use hopr_api::{
    OffchainPublicKey,
    graph::traits::{EdgeNetworkObservableRead, EdgeObservableRead, EdgeProtocolObservable},
};
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

/// Build a HOPR cost function for network graph traversals
// TODO: implement precise cost function for path traversal
pub fn build_hopr_cost_fn(length: usize) -> impl Fn(f64, &crate::Observations, usize) -> f64 {
    move |initial_cost: f64, observation: &crate::Observations, path_index: usize| {
        match path_index {
            0 => {
                // the first edge should always go to an already connected and measured peer,
                // otherwise use a negative cost that should remove the edge from consideration
                if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                    // TODO(20260217: extend once 1-hop probing verifiably works)
                    // && o.average_latency().is_some_and(|latency| latency)
                    if observation.intermediate_qos().is_some_and(|o| o.capacity().is_some()) {
                        return initial_cost;
                    }
                }

                -initial_cost
            }
            v if v == length => {
                // the last edge should always go from an already connected and measured peer,
                // otherwise use a negative cost that should remove the edge from consideration
                if observation.immediate_qos().is_some_and(|o| o.is_connected()) {
                    // TODO(20260217: extend once 1-hop probing verifiably works)
                    // if observation.intermediate_qos().is_some_and(|o| o.capacity().o.score()()) {
                    return initial_cost;
                    // }
                }

                -initial_cost
            }
            _ => initial_cost,
        }
    }
}
