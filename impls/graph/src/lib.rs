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

pub mod immediate;
pub mod observation;

use std::collections::HashMap;

use hopr_internal_types::channels::{ChannelEntry, ChannelStatus};
use hopr_primitive_types::{
    // prelude::{Balance, WxHOPR},
    primitives::Address,
};
use petgraph::graph::{DiGraph, NodeIndex};
use thiserror::Error;

// pub struct Weight {
//     channel_balance: Option<Balance<WxHOPR>>,
//     immediate_qos: Option<f32>,
//     intermediate_qos: Option<f32>,
// }

/// Errors that can occur when manipulating the channel graph.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChannelGraphError {
    /// Source and destination are the same node.
    #[error("channel source and destination must be different")]
    LoopChannel,
    /// Channel already exists in the graph.
    #[error("channel already exists between {0} and {1}")]
    ChannelAlreadyExists(Address, Address),
    /// Channel not found in the graph.
    #[error("channel not found between {0} and {1}")]
    ChannelNotFound(Address, Address),
    /// Node not found in the graph.
    #[error("node not found: {0}")]
    NodeNotFound(Address),
}

/// A directed graph representing payment channels between nodes.
///
/// The graph uses `Address` as node identifiers and `ChannelEntry` as edge weights.
/// Only open channels are stored in the graph.
#[derive(Debug, Default, Clone)]
pub struct ChannelGraph {
    graph: DiGraph<Address, ChannelEntry>,
    node_indices: HashMap<Address, NodeIndex>,
}

impl ChannelGraph {
    /// Creates a new empty channel graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of channels (edges) in the graph.
    pub fn channel_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Checks if a node exists in the graph.
    pub fn contains_node(&self, address: &Address) -> bool {
        self.node_indices.contains_key(address)
    }

    /// Adds a node to the graph if it doesn't exist.
    ///
    /// Returns the `NodeIndex` for the node.
    pub fn add_node(&mut self, address: Address) -> NodeIndex {
        *self
            .node_indices
            .entry(address)
            .or_insert_with(|| self.graph.add_node(address))
    }

    /// Removes a node and all its associated channels.
    ///
    /// Returns `true` if the node was removed, `false` if it didn't exist.
    pub fn remove_node(&mut self, address: &Address) -> bool {
        if let Some(idx) = self.node_indices.remove(address) {
            self.graph.remove_node(idx);
            // Rebuild indices after removal since petgraph reuses indices
            self.rebuild_node_indices();
            true
        } else {
            false
        }
    }

    /// Rebuilds the node indices map after a removal operation.
    fn rebuild_node_indices(&mut self) {
        self.node_indices.clear();
        for idx in self.graph.node_indices() {
            if let Some(addr) = self.graph.node_weight(idx) {
                self.node_indices.insert(*addr, idx);
            }
        }
    }

    /// Adds a channel to the graph.
    ///
    /// Only open channels are added. Closed or pending-to-close channels are ignored.
    /// Both source and destination nodes are added automatically if they don't exist.
    pub fn add_channel(&mut self, channel: ChannelEntry) -> Result<(), ChannelGraphError> {
        if channel.source == channel.destination {
            return Err(ChannelGraphError::LoopChannel);
        }

        // Only add open channels
        if !matches!(channel.status, ChannelStatus::Open) {
            tracing::debug!(
                "ignoring non-open channel from {} to {}",
                channel.source,
                channel.destination
            );
            return Ok(());
        }

        let src_idx = self.add_node(channel.source);
        let dst_idx = self.add_node(channel.destination);

        // Check if channel already exists
        if self.graph.find_edge(src_idx, dst_idx).is_some() {
            return Err(ChannelGraphError::ChannelAlreadyExists(
                channel.source,
                channel.destination,
            ));
        }

        self.graph.add_edge(src_idx, dst_idx, channel);
        Ok(())
    }

    /// Updates an existing channel in the graph.
    ///
    /// If the channel status changes to non-open, the channel is removed.
    pub fn update_channel(&mut self, channel: ChannelEntry) -> Result<(), ChannelGraphError> {
        let src_idx = self
            .node_indices
            .get(&channel.source)
            .copied()
            .ok_or(ChannelGraphError::NodeNotFound(channel.source))?;
        let dst_idx = self
            .node_indices
            .get(&channel.destination)
            .copied()
            .ok_or(ChannelGraphError::NodeNotFound(channel.destination))?;

        let edge_idx = self
            .graph
            .find_edge(src_idx, dst_idx)
            .ok_or(ChannelGraphError::ChannelNotFound(channel.source, channel.destination))?;

        if matches!(channel.status, ChannelStatus::Open) {
            // Update the channel
            if let Some(edge) = self.graph.edge_weight_mut(edge_idx) {
                *edge = channel;
            }
        } else {
            // Remove non-open channels
            self.graph.remove_edge(edge_idx);
        }

        Ok(())
    }

    /// Removes a channel from the graph.
    pub fn remove_channel(
        &mut self,
        source: &Address,
        destination: &Address,
    ) -> Result<ChannelEntry, ChannelGraphError> {
        let src_idx = self
            .node_indices
            .get(source)
            .copied()
            .ok_or(ChannelGraphError::NodeNotFound(*source))?;
        let dst_idx = self
            .node_indices
            .get(destination)
            .copied()
            .ok_or(ChannelGraphError::NodeNotFound(*destination))?;

        let edge_idx = self
            .graph
            .find_edge(src_idx, dst_idx)
            .ok_or(ChannelGraphError::ChannelNotFound(*source, *destination))?;

        self.graph
            .remove_edge(edge_idx)
            .ok_or(ChannelGraphError::ChannelNotFound(*source, *destination))
    }

    /// Gets a channel between two nodes.
    pub fn get_channel(&self, source: &Address, destination: &Address) -> Option<&ChannelEntry> {
        let src_idx = self.node_indices.get(source)?;
        let dst_idx = self.node_indices.get(destination)?;
        let edge_idx = self.graph.find_edge(*src_idx, *dst_idx)?;
        self.graph.edge_weight(edge_idx)
    }

    /// Returns all outgoing channels from a node.
    pub fn outgoing_channels(&self, address: &Address) -> Vec<&ChannelEntry> {
        let Some(&idx) = self.node_indices.get(address) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(idx, petgraph::Direction::Outgoing)
            .map(|e| e.weight())
            .collect()
    }

    /// Returns all incoming channels to a node.
    pub fn incoming_channels(&self, address: &Address) -> Vec<&ChannelEntry> {
        let Some(&idx) = self.node_indices.get(address) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(idx, petgraph::Direction::Incoming)
            .map(|e| e.weight())
            .collect()
    }

    /// Returns an iterator over all channels in the graph.
    pub fn all_channels(&self) -> impl Iterator<Item = &ChannelEntry> {
        self.graph.edge_weights()
    }

    /// Returns an iterator over all nodes in the graph.
    pub fn all_nodes(&self) -> impl Iterator<Item = &Address> {
        self.graph.node_weights()
    }

    /// Checks if there's a direct channel from source to destination.
    pub fn has_channel(&self, source: &Address, destination: &Address) -> bool {
        self.get_channel(source, destination).is_some()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hex_literal::hex;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::prelude::HoprBalance;

    use super::*;

    lazy_static::lazy_static! {
        static ref ALICE_KP: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("keypair should be valid");
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("keypair should be valid");
        static ref CHARLIE_KP: ChainKeypair = ChainKeypair::from_secret(&hex!("d39a926980d6fa96a7c3a630b3a04a7665f89531e2e5e8c489afbb23c797a701")).expect("keypair should be valid");

        static ref ALICE: Address = ALICE_KP.public().to_address();
        static ref BOB: Address = BOB_KP.public().to_address();
        static ref CHARLIE: Address = CHARLIE_KP.public().to_address();
    }

    fn make_open_channel(source: Address, destination: Address, balance: u64) -> ChannelEntry {
        ChannelEntry::new(
            source,
            destination,
            HoprBalance::from(balance),
            0,
            ChannelStatus::Open,
            1,
        )
    }

    fn make_closed_channel(source: Address, destination: Address) -> ChannelEntry {
        ChannelEntry::new(source, destination, HoprBalance::zero(), 0, ChannelStatus::Closed, 1)
    }

    #[test]
    fn new_graph_is_empty() {
        let graph = ChannelGraph::new();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.channel_count(), 0);
    }

    #[test]
    fn default_creates_empty_graph() {
        let graph = ChannelGraph::default();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.channel_count(), 0);
    }

    #[test]
    fn adding_node_tracks_it_in_graph() {
        let mut graph = ChannelGraph::new();

        graph.add_node(*ALICE);

        assert!(graph.contains_node(&ALICE));
        assert!(!graph.contains_node(&BOB));
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn adding_same_node_twice_is_idempotent() {
        let mut graph = ChannelGraph::new();

        let idx1 = graph.add_node(*ALICE);
        let idx2 = graph.add_node(*ALICE);

        assert_eq!(idx1, idx2);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn adding_channel_creates_nodes_automatically() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *BOB, 100);

        graph.add_channel(channel).context("failed to add channel")?;

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.channel_count(), 1);
        assert!(graph.contains_node(&ALICE));
        assert!(graph.contains_node(&BOB));
        Ok(())
    }

    #[test]
    fn channels_are_directional() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *BOB, 100);

        graph.add_channel(channel).context("failed to add channel")?;

        assert!(graph.has_channel(&ALICE, &BOB));
        assert!(!graph.has_channel(&BOB, &ALICE));
        Ok(())
    }

    #[test]
    fn bidirectional_channels_are_supported() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let ab_channel = make_open_channel(*ALICE, *BOB, 100);
        let ba_channel = make_open_channel(*BOB, *ALICE, 50);

        graph
            .add_channel(ab_channel)
            .context("failed to add ALICE->BOB channel")?;
        graph
            .add_channel(ba_channel)
            .context("failed to add BOB->ALICE channel")?;

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.channel_count(), 2);
        assert!(graph.has_channel(&ALICE, &BOB));
        assert!(graph.has_channel(&BOB, &ALICE));
        Ok(())
    }

    #[test]
    fn loop_channels_are_rejected() {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *ALICE, 100);

        let result = graph.add_channel(channel);

        assert_eq!(result, Err(ChannelGraphError::LoopChannel));
    }

    #[test]
    fn duplicate_channels_are_rejected() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel1 = make_open_channel(*ALICE, *BOB, 100);
        let channel2 = make_open_channel(*ALICE, *BOB, 200);

        graph.add_channel(channel1).context("failed to add first channel")?;
        let result = graph.add_channel(channel2);

        assert_eq!(result, Err(ChannelGraphError::ChannelAlreadyExists(*ALICE, *BOB)));
        Ok(())
    }

    #[test]
    fn closed_channels_are_not_added() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_closed_channel(*ALICE, *BOB);

        graph.add_channel(channel).context("failed to process closed channel")?;

        assert_eq!(graph.channel_count(), 0);
        assert_eq!(graph.node_count(), 0);
        Ok(())
    }

    #[test]
    fn get_channel_returns_existing_channel() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *BOB, 100);

        graph.add_channel(channel).context("failed to add channel")?;

        let retrieved = graph.get_channel(&ALICE, &BOB).context("channel not found")?;
        assert_eq!(retrieved.balance, HoprBalance::from(100u64));
        Ok(())
    }

    #[test]
    fn get_channel_returns_none_for_missing() {
        let graph = ChannelGraph::new();

        let result = graph.get_channel(&ALICE, &BOB);

        assert!(result.is_none());
    }

    #[test]
    fn remove_channel_returns_removed_entry() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *BOB, 100);

        graph.add_channel(channel).context("failed to add channel")?;
        let removed = graph.remove_channel(&ALICE, &BOB).context("failed to remove channel")?;

        assert_eq!(removed.balance, HoprBalance::from(100u64));
        assert_eq!(graph.channel_count(), 0);
        // Nodes should still exist
        assert_eq!(graph.node_count(), 2);
        Ok(())
    }

    #[test]
    fn removing_nonexistent_channel_fails() {
        let mut graph = ChannelGraph::new();
        graph.add_node(*ALICE);
        graph.add_node(*BOB);

        let result = graph.remove_channel(&ALICE, &BOB);

        assert_eq!(result, Err(ChannelGraphError::ChannelNotFound(*ALICE, *BOB)));
    }

    #[test]
    fn removing_node_removes_associated_channels() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *BOB, 100);

        graph.add_channel(channel).context("failed to add channel")?;
        let removed = graph.remove_node(&ALICE);

        assert!(removed);
        assert!(!graph.contains_node(&ALICE));
        assert!(graph.contains_node(&BOB));
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.channel_count(), 0);
        Ok(())
    }

    #[test]
    fn removing_nonexistent_node_returns_false() {
        let mut graph = ChannelGraph::new();

        let removed = graph.remove_node(&ALICE);

        assert!(!removed);
    }

    #[test]
    fn updating_channel_changes_balance() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *BOB, 100);
        let updated = make_open_channel(*ALICE, *BOB, 50);

        graph.add_channel(channel).context("failed to add channel")?;
        graph.update_channel(updated).context("failed to update channel")?;

        let retrieved = graph
            .get_channel(&ALICE, &BOB)
            .context("channel not found after update")?;
        assert_eq!(retrieved.balance, HoprBalance::from(50u64));
        Ok(())
    }

    #[test]
    fn updating_channel_to_closed_removes_it() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let channel = make_open_channel(*ALICE, *BOB, 100);
        let closed = make_closed_channel(*ALICE, *BOB);

        graph.add_channel(channel).context("failed to add channel")?;
        graph
            .update_channel(closed)
            .context("failed to update channel to closed")?;

        assert_eq!(graph.channel_count(), 0);
        assert!(!graph.has_channel(&ALICE, &BOB));
        Ok(())
    }

    #[test]
    fn updating_nonexistent_channel_fails() {
        let mut graph = ChannelGraph::new();
        graph.add_node(*ALICE);
        graph.add_node(*BOB);
        let channel = make_open_channel(*ALICE, *BOB, 100);

        let result = graph.update_channel(channel);

        assert_eq!(result, Err(ChannelGraphError::ChannelNotFound(*ALICE, *BOB)));
    }

    #[test]
    fn outgoing_channels_returns_correct_edges() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let ab = make_open_channel(*ALICE, *BOB, 100);
        let ac = make_open_channel(*ALICE, *CHARLIE, 200);
        let ba = make_open_channel(*BOB, *ALICE, 50);

        graph.add_channel(ab).context("failed to add ALICE->BOB channel")?;
        graph.add_channel(ac).context("failed to add ALICE->CHARLIE channel")?;
        graph.add_channel(ba).context("failed to add BOB->ALICE channel")?;

        let alice_outgoing = graph.outgoing_channels(&ALICE);
        assert_eq!(alice_outgoing.len(), 2);

        let bob_outgoing = graph.outgoing_channels(&BOB);
        assert_eq!(bob_outgoing.len(), 1);
        Ok(())
    }

    #[test]
    fn incoming_channels_returns_correct_edges() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let ab = make_open_channel(*ALICE, *BOB, 100);
        let cb = make_open_channel(*CHARLIE, *BOB, 200);
        let ba = make_open_channel(*BOB, *ALICE, 50);

        graph.add_channel(ab).context("failed to add ALICE->BOB channel")?;
        graph.add_channel(cb).context("failed to add CHARLIE->BOB channel")?;
        graph.add_channel(ba).context("failed to add BOB->ALICE channel")?;

        let bob_incoming = graph.incoming_channels(&BOB);
        assert_eq!(bob_incoming.len(), 2);

        let alice_incoming = graph.incoming_channels(&ALICE);
        assert_eq!(alice_incoming.len(), 1);
        Ok(())
    }

    #[test]
    fn outgoing_channels_returns_empty_for_missing_node() {
        let graph = ChannelGraph::new();

        let channels = graph.outgoing_channels(&ALICE);

        assert!(channels.is_empty());
    }

    #[test]
    fn all_channels_iterates_over_edges() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let ab = make_open_channel(*ALICE, *BOB, 100);
        let bc = make_open_channel(*BOB, *CHARLIE, 200);

        graph.add_channel(ab).context("failed to add ALICE->BOB channel")?;
        graph.add_channel(bc).context("failed to add BOB->CHARLIE channel")?;

        let channels: Vec<_> = graph.all_channels().collect();
        assert_eq!(channels.len(), 2);
        Ok(())
    }

    #[test]
    fn all_nodes_iterates_over_vertices() -> anyhow::Result<()> {
        let mut graph = ChannelGraph::new();
        let ab = make_open_channel(*ALICE, *BOB, 100);

        graph.add_channel(ab).context("failed to add channel")?;
        graph.add_node(*CHARLIE);

        let nodes: Vec<_> = graph.all_nodes().collect();
        assert_eq!(nodes.len(), 3);
        Ok(())
    }

    #[test]
    fn triangle_topology_forms_cycle() -> anyhow::Result<()> {
        // Create a triangle: ALICE -> BOB -> CHARLIE -> ALICE
        let mut graph = ChannelGraph::new();

        graph
            .add_channel(make_open_channel(*ALICE, *BOB, 100))
            .context("failed to add ALICE->BOB channel")?;
        graph
            .add_channel(make_open_channel(*BOB, *CHARLIE, 100))
            .context("failed to add BOB->CHARLIE channel")?;
        graph
            .add_channel(make_open_channel(*CHARLIE, *ALICE, 100))
            .context("failed to add CHARLIE->ALICE channel")?;

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.channel_count(), 3);

        // Verify cycle
        assert!(graph.has_channel(&ALICE, &BOB));
        assert!(graph.has_channel(&BOB, &CHARLIE));
        assert!(graph.has_channel(&CHARLIE, &ALICE));
        Ok(())
    }
}
