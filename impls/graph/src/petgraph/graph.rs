use std::sync::Arc;

use bimap::BiHashMap;
use hopr_api::OffchainPublicKey;
use parking_lot::RwLock;
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{Observations, errors::ChannelGraphError};

/// Internal mutable state of a [`ChannelGraph`], protected by a lock.
#[derive(Debug, Clone, Default)]
pub(crate) struct InnerGraph {
    pub(crate) graph: DiGraph<OffchainPublicKey, Observations>,
    pub(crate) indices: BiHashMap<OffchainPublicKey, NodeIndex>,
}

/// A directed graph representing logical channels between nodes.
///
/// The graph is directed, with nodes representing the physical nodes in the network using
/// their [`OffchainPublicKey`] as identifier and edges representing the logical channels
/// between them. Each logical channel aggregates different weighted properties, like
/// channel capacity (calculated from the on-chain channel balance, ticket price and ticket probability)
/// and evaluated transport network properties between the nodes.
///
/// Interior mutability is provided via an internal [`RwLock`] so that all trait
/// methods (which take `&self`) can safely read and write the graph. In production
/// code, the graph is expected to be shared behind an `Arc<ChannelGraph>`.
#[derive(Debug, Clone)]
pub struct ChannelGraph {
    pub(crate) me: OffchainPublicKey,
    pub(crate) inner: Arc<RwLock<InnerGraph>>,
}

impl ChannelGraph {
    /// Creates a new channel graph with the given self identity.
    ///
    /// The `me` key represents the local node which is automatically added
    /// to the graph as the first node.
    pub fn new(me: OffchainPublicKey) -> Self {
        let mut graph = DiGraph::new();
        let mut indices = BiHashMap::new();

        let idx = graph.add_node(me);
        indices.insert(me, idx);

        Self {
            me,
            inner: Arc::new(RwLock::new(InnerGraph { graph, indices })),
        }
    }

    /// Returns the self-identity key of this graph.
    pub fn me(&self) -> &OffchainPublicKey {
        &self.me
    }
}

#[async_trait::async_trait]
impl hopr_api::graph::NetworkGraphView for ChannelGraph {
    type NodeId = OffchainPublicKey;
    type Observed = Observations;

    fn node_count(&self) -> usize {
        self.inner.read().graph.node_count()
    }

    fn contains_node(&self, key: &OffchainPublicKey) -> bool {
        self.inner.read().indices.contains_left(key)
    }

    fn nodes(&self) -> futures::stream::BoxStream<'static, Self::NodeId> {
        let keys: Vec<OffchainPublicKey> = {
            let inner = self.inner.read();
            inner.indices.left_values().copied().collect()
        };

        Box::pin(futures::stream::iter(keys))
    }

    fn has_edge(&self, src: &OffchainPublicKey, dest: &OffchainPublicKey) -> bool {
        let inner = self.inner.read();
        let (Some(src_idx), Some(dest_idx)) = (inner.indices.get_by_left(src), inner.indices.get_by_left(dest)) else {
            return false;
        };
        inner.graph.contains_edge(*src_idx, *dest_idx)
    }

    fn edge(&self, src: &Self::NodeId, dest: &Self::NodeId) -> Option<Self::Observed> {
        let inner = self.inner.read();
        let src_idx = inner.indices.get_by_left(src)?;
        let dest_idx = inner.indices.get_by_left(dest)?;
        let edge_idx = inner.graph.find_edge(*src_idx, *dest_idx)?;
        inner.graph.edge_weight(edge_idx).copied()
    }
}

impl hopr_api::graph::NetworkGraphWrite for ChannelGraph {
    type Error = ChannelGraphError;
    type NodeId = OffchainPublicKey;
    type Observed = Observations;

    fn add_node(&self, key: OffchainPublicKey) {
        let mut inner = self.inner.write();
        if !inner.indices.contains_left(&key) {
            let idx = inner.graph.add_node(key);
            inner.indices.insert(key, idx);
        }
    }

    fn remove_node(&self, key: &OffchainPublicKey) {
        let mut inner = self.inner.write();
        if let Some((_, idx)) = inner.indices.remove_by_left(key) {
            inner.graph.remove_node(idx);

            // petgraph swaps the last node into the removed slot,
            // so we need to update the index mapping for the swapped node.
            if let Some(swapped_key) = inner.graph.node_weight(idx) {
                let swapped_key = *swapped_key;
                inner.indices.insert(swapped_key, idx);
            }
        }
    }

    fn add_edge(&self, src: &OffchainPublicKey, dest: &OffchainPublicKey) -> Result<(), ChannelGraphError> {
        let mut inner = self.inner.write();
        let src_idx = inner
            .indices
            .get_by_left(src)
            .copied()
            .ok_or(ChannelGraphError::PublicKeyNodeNotFound(*src))?;
        let dest_idx = inner
            .indices
            .get_by_left(dest)
            .copied()
            .ok_or(ChannelGraphError::PublicKeyNodeNotFound(*dest))?;

        inner.graph.add_edge(src_idx, dest_idx, Observations::default());
        Ok(())
    }

    fn remove_edge(&self, src: &OffchainPublicKey, dest: &OffchainPublicKey) {
        let mut inner = self.inner.write();
        if let (Some(src_idx), Some(dest_idx)) = (
            inner.indices.get_by_left(src).copied(),
            inner.indices.get_by_left(dest).copied(),
        ) {
            if let Some(edge_idx) = inner.graph.find_edge(src_idx, dest_idx) {
                inner.graph.remove_edge(edge_idx);
            }
        }
    }

    /// Mutably updates the edge observations between two nodes.
    ///
    /// If the edge does not exist, it gets created first.
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn upsert_edge<F>(&self, src: &OffchainPublicKey, dest: &OffchainPublicKey, f: F)
    where
        F: FnOnce(&mut Observations),
    {
        let mut inner = self.inner.write();

        if let (Some(src_idx), Some(dest_idx)) = (
            inner.indices.get_by_left(src).copied(),
            inner.indices.get_by_left(dest).copied(),
        ) {
            let edge_idx = inner
                .graph
                .find_edge(src_idx, dest_idx)
                .unwrap_or_else(|| inner.graph.add_edge(src_idx, dest_idx, Observations::default()));

            if let Some(weight) = inner.graph.edge_weight_mut(edge_idx) {
                f(weight);
                tracing::debug!(%src, %dest, ?weight, "updated edge weight with an observation");
            }
        } else {
            tracing::warn!(%src, %dest, reason = "one or both of the nodes do not exist", "edge update failed" );
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_api::graph::{
        EdgeLinkObservable, NetworkGraphView, NetworkGraphWrite,
        traits::{EdgeObservableRead, EdgeObservableWrite, EdgeWeightType},
    };
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};

    use super::*;

    /// Fixed test secret keys (reused from the broader codebase).
    const SECRET_0: [u8; 32] = hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");
    const SECRET_1: [u8; 32] = hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a");
    const SECRET_2: [u8; 32] = hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299");
    const SECRET_3: [u8; 32] = hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e");
    const SECRET_4: [u8; 32] = hex!("cfc66f718ec66fb822391775d749d7a0d66b690927673634816b63339bc12a3c");
    const SECRET_5: [u8; 32] = hex!("203ca4d3c5f98dd2066bb204b5930c10b15c095585c224c826b4e11f08bfa85d");
    const SECRET_7: [u8; 32] = hex!("4ab03f6f75f845ca1bf8b7104804ea5bda18bda29d1ec5fc5d4267feca5fb8e1");

    /// Creates an OffchainPublicKey from a fixed secret.
    fn pubkey_from(secret: &[u8; 32]) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(secret).expect("valid secret key").public()
    }

    #[test]
    fn new_graph_contains_self_node() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        assert!(graph.contains_node(&me));
        assert_eq!(graph.node_count(), 1);
        Ok(())
    }

    #[test]
    fn adding_a_node_increases_count() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peer = pubkey_from(&SECRET_1);
        graph.add_node(peer);
        assert!(graph.contains_node(&peer));
        assert_eq!(graph.node_count(), 2);
        Ok(())
    }

    #[test]
    fn adding_duplicate_node_is_idempotent() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peer = pubkey_from(&SECRET_1);
        graph.add_node(peer);
        graph.add_node(peer);
        assert_eq!(graph.node_count(), 2);
        Ok(())
    }

    #[test]
    fn removing_a_node_decreases_count() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peer = pubkey_from(&SECRET_1);
        graph.add_node(peer);
        assert_eq!(graph.node_count(), 2);
        graph.remove_node(&peer);
        assert!(!graph.contains_node(&peer));
        assert_eq!(graph.node_count(), 1);
        Ok(())
    }

    #[test]
    fn removing_nonexistent_node_is_noop() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        graph.remove_node(&pubkey_from(&SECRET_7));
        assert_eq!(graph.node_count(), 1);
        Ok(())
    }

    #[test]
    fn adding_an_edge_between_nodes() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peer = pubkey_from(&SECRET_1);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;
        assert!(graph.has_edge(&me, &peer));
        assert!(!graph.has_edge(&peer, &me));
        Ok(())
    }

    #[test]
    fn adding_edge_to_missing_node_errors() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        assert!(graph.add_edge(&me, &pubkey_from(&SECRET_7)).is_err());
        Ok(())
    }

    #[test]
    fn removing_a_node_also_removes_its_edges() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peer = pubkey_from(&SECRET_1);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;
        assert!(graph.has_edge(&me, &peer));
        graph.remove_node(&peer);
        assert!(!graph.has_edge(&me, &peer));
        Ok(())
    }

    #[tokio::test]
    async fn view_nodes_returns_all_graph_nodes() -> anyhow::Result<()> {
        use futures::StreamExt;

        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peers: Vec<_> = [SECRET_1, SECRET_2, SECRET_3, SECRET_4, SECRET_5]
            .iter()
            .map(|s| pubkey_from(s))
            .collect();
        for &peer in &peers {
            graph.add_node(peer);
        }
        let nodes: Vec<_> = graph.nodes().collect().await;
        assert_eq!(nodes.len(), 6);
        assert!(nodes.contains(&me));
        for peer in &peers {
            assert!(nodes.contains(peer));
        }
        Ok(())
    }

    #[tokio::test]
    async fn view_edge_returns_observations_for_existing_edge() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peer = pubkey_from(&SECRET_1);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;
        assert!(graph.edge(&me, &peer).is_some());
        Ok(())
    }

    #[tokio::test]
    async fn view_edge_returns_none_for_missing_edge() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let peer = pubkey_from(&SECRET_1);
        assert!(graph.edge(&me, &peer).is_none());
        Ok(())
    }

    #[test]
    fn me_returns_self_identity() {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        assert_eq!(*graph.me(), me);
    }

    #[test]
    fn removing_an_edge_disconnects_nodes() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;
        assert!(graph.has_edge(&me, &peer));

        graph.remove_edge(&me, &peer);
        assert!(!graph.has_edge(&me, &peer));
        // Nodes should still exist
        assert!(graph.contains_node(&me));
        assert!(graph.contains_node(&peer));
        Ok(())
    }

    #[test]
    fn removing_nonexistent_edge_is_noop() {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        // No edge exists — should not panic
        graph.remove_edge(&me, &peer);
        assert!(!graph.has_edge(&me, &peer));
    }

    #[test]
    fn removing_edge_for_unknown_nodes_is_noop() {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let unknown = pubkey_from(&SECRET_7);
        // Neither node known — should not panic
        graph.remove_edge(&me, &unknown);
    }

    #[test]
    fn edge_should_not_be_present_when_nodes_not_in_graph() {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let unknown = pubkey_from(&SECRET_7);
        assert!(!graph.has_edge(&me, &unknown));
        assert!(!graph.has_edge(&unknown, &me));
    }

    #[test]
    fn edge_returns_none_when_nodes_not_in_graph() {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let unknown = pubkey_from(&SECRET_7);
        assert!(graph.edge(&me, &unknown).is_none());
        assert!(graph.edge(&unknown, &me).is_none());
    }

    #[test]
    fn upsert_edge_creates_edge_when_absent() {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);

        assert!(!graph.has_edge(&me, &peer));
        graph.upsert_edge(&me, &peer, |obs| {
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
        });
        assert!(graph.has_edge(&me, &peer));

        let obs = graph.edge(&me, &peer).expect("edge should exist after upsert");
        assert!(obs.immediate_qos().is_some());
    }

    #[test]
    fn upsert_edge_updates_existing_edge() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        graph.upsert_edge(&me, &peer, |obs| {
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(100))));
        });
        graph.upsert_edge(&me, &peer, |obs| {
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(200))));
        });

        let obs = graph.edge(&me, &peer).expect("edge should exist");
        let latency = obs
            .immediate_qos()
            .expect("should have immediate QoS")
            .average_latency()
            .expect("should have latency");
        // After two updates (100ms and 200ms), average should be between 100 and 200
        assert!(latency > std::time::Duration::from_millis(100));
        assert!(latency < std::time::Duration::from_millis(200));
        Ok(())
    }

    #[test]
    fn upsert_edge_with_unknown_nodes_is_noop() {
        let me = pubkey_from(&SECRET_0);
        let unknown = pubkey_from(&SECRET_7);
        let graph = ChannelGraph::new(me);

        // One or both nodes unknown — should not panic, no edge created
        graph.upsert_edge(&me, &unknown, |_obs| {
            panic!("closure should not be called when nodes are missing");
        });
        assert!(!graph.has_edge(&me, &unknown));
    }

    #[test]
    fn removing_non_last_node_preserves_other_nodes() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        assert_eq!(graph.node_count(), 4);

        // Remove a node that is not the last one (triggers index swap in petgraph)
        graph.remove_node(&a);
        assert_eq!(graph.node_count(), 3);
        assert!(!graph.contains_node(&a));
        assert!(graph.contains_node(&me));
        assert!(graph.contains_node(&b));
        assert!(graph.contains_node(&c));

        // Verify edges can still be added to remaining nodes
        graph.add_edge(&me, &b)?;
        graph.add_edge(&me, &c)?;
        assert!(graph.has_edge(&me, &b));
        assert!(graph.has_edge(&me, &c));
        Ok(())
    }

    #[test]
    fn removing_multiple_nodes_preserves_consistency() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);
        let d = pubkey_from(&SECRET_4);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_node(d);

        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&c, &d)?;

        // Remove middle nodes
        graph.remove_node(&b);
        graph.remove_node(&c);

        assert_eq!(graph.node_count(), 3);
        assert!(graph.contains_node(&me));
        assert!(graph.contains_node(&a));
        assert!(graph.contains_node(&d));

        // Edges through removed nodes should be gone
        assert!(!graph.has_edge(&a, &b));
        assert!(!graph.has_edge(&b, &c));
        assert!(!graph.has_edge(&c, &d));

        // Edge not involving removed nodes should survive
        assert!(graph.has_edge(&me, &a));
        Ok(())
    }

    #[test]
    fn edges_are_directed() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        assert!(graph.has_edge(&me, &peer));
        assert!(!graph.has_edge(&peer, &me));

        assert!(graph.edge(&me, &peer).is_some());
        assert!(graph.edge(&peer, &me).is_none());
        Ok(())
    }
}
