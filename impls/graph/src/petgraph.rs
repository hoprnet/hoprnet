use bimap::BiHashMap;
use hopr_api::{
    OffchainPublicKey,
    graph::{MeasurableEdge, MeasurableNode, NetworkGraphWrite, traits::EdgeObservableWrite},
};
use parking_lot::RwLock;
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{errors::ChannelGraphError, weight::Observations};

/// Internal mutable state of a [`ChannelGraph`], protected by a lock.
#[derive(Debug, Clone, Default)]
struct InnerGraph {
    graph: DiGraph<OffchainPublicKey, Observations>,
    indices: BiHashMap<OffchainPublicKey, NodeIndex>,
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
#[derive(Debug)]
pub struct ChannelGraph {
    me: OffchainPublicKey,
    inner: RwLock<InnerGraph>,
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
            inner: RwLock::new(InnerGraph { graph, indices }),
        }
    }

    /// Returns the self-identity key of this graph.
    pub fn me(&self) -> &OffchainPublicKey {
        &self.me
    }

    /// Finds all simple paths of exactly `length` hops from `src` to `dest`. (naive implementation)
    ///
    /// Returns a list of paths, where each path is a vector of intermediate
    /// [`OffchainPublicKey`]s (excluding `src` and `dest`).
    fn find_paths(
        &self,
        src: &OffchainPublicKey,
        dest: &OffchainPublicKey,
        length: usize,
    ) -> Vec<Vec<OffchainPublicKey>> {
        // TOOD: use simple_path from `petgraph`
        let inner = self.inner.read();
        let Some(src_idx) = inner.indices.get_by_left(src) else {
            return vec![];
        };
        let Some(dest_idx) = inner.indices.get_by_left(dest) else {
            return vec![];
        };

        let mut results = Vec::new();
        let mut visited = vec![false; inner.graph.node_count()];
        let mut current_path = Vec::new();

        visited[src_idx.index()] = true;
        Self::dfs_paths(
            &inner,
            *src_idx,
            *dest_idx,
            length,
            &mut visited,
            &mut current_path,
            &mut results,
        );

        results
    }

    /// DFS helper to enumerate simple paths of a specific length.
    fn dfs_paths(
        inner: &InnerGraph,
        current: NodeIndex,
        target: NodeIndex,
        remaining_hops: usize,
        visited: &mut Vec<bool>,
        current_path: &mut Vec<OffchainPublicKey>,
        results: &mut Vec<Vec<OffchainPublicKey>>,
    ) {
        if remaining_hops == 0 {
            if current == target {
                results.push(current_path.clone());
            }
            return;
        }

        for neighbor in inner.graph.neighbors(current) {
            if neighbor == target && remaining_hops == 1 {
                // Reached target at exactly the right depth
                results.push(current_path.clone());
                continue;
            }

            if !visited[neighbor.index()] && remaining_hops > 1 {
                visited[neighbor.index()] = true;
                if let Some(key) = inner.indices.get_by_right(&neighbor) {
                    current_path.push(*key);
                    Self::dfs_paths(
                        inner,
                        neighbor,
                        target,
                        remaining_hops - 1,
                        visited,
                        current_path,
                        results,
                    );
                    current_path.pop();
                }
                visited[neighbor.index()] = false;
            }
        }
    }
}

#[async_trait::async_trait]
impl hopr_api::graph::NetworkGraphUpdate for ChannelGraph {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: hopr_api::graph::MeasurablePeer + std::fmt::Debug + Send + Clone,
        P: hopr_api::graph::MeasurablePath + std::fmt::Debug + Send + Clone,
    {
        use hopr_api::graph::traits::EdgeWeightType;

        match update {
            MeasurableEdge::Probe(Ok(hopr_api::graph::EdgeTransportTelemetry::Neighbor(ref telemetry))) => {
                tracing::trace!(
                    peer = %telemetry.peer(),
                    latency_ms = telemetry.rtt().as_millis(),
                    "neighbor probe successful"
                );

                self.upsert_edge(&self.me, telemetry.peer(), |obs| {
                    obs.record(EdgeWeightType::Immediate(Ok(telemetry.rtt() / 2)));
                });
            }
            MeasurableEdge::Probe(Ok(hopr_api::graph::EdgeTransportTelemetry::Loopback(_))) => {
                tracing::warn!(
                    reason = "feature not implemented",
                    "loopback path telemetry not supported"
                );
            }
            MeasurableEdge::Probe(Err(hopr_api::graph::NetworkGraphError::ProbeNeighborTimeout(ref peer))) => {
                tracing::trace!(
                    peer = %peer,
                    reason = "probe timeout",
                    "neighbor probe failed"
                );

                self.upsert_edge(&self.me, peer, |obs| {
                    obs.record(EdgeWeightType::Immediate(Err(())));
                });
            }
            MeasurableEdge::Probe(Err(hopr_api::graph::NetworkGraphError::ProbeLoopbackTimeout(_))) => {
                tracing::warn!(
                    reason = "feature not implemented",
                    "loopback path telemetry not supported"
                );
            }
            MeasurableEdge::Capacity(update) => {
                self.upsert_edge(&update.src, &update.dest, |obs: &mut Observations| {
                    obs.record(EdgeWeightType::Capacity(update.capacity));
                });
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        tracing::trace!(?update, "recording node update");
        hopr_api::graph::NetworkGraphWrite::add_node(self, update.into());
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

#[async_trait::async_trait]
impl hopr_api::graph::NetworkGraphTraverse for ChannelGraph {
    type NodeId = OffchainPublicKey;

    async fn simple_route(&self, destination: &Self::NodeId, length: usize) -> Vec<hopr_api::ct::DestinationRouting> {
        use hopr_internal_types::prelude::NodeId;
        use hopr_network_types::types::{DestinationRouting, RoutingOptions};

        let paths = self.find_paths(&self.me, destination, length);

        paths
            .into_iter()
            .filter_map(|intermediates| {
                let node_ids: Vec<NodeId> = intermediates.into_iter().map(NodeId::from).collect();
                let bounded: hopr_primitive_types::bounded::BoundedVec<
                    NodeId,
                    { RoutingOptions::MAX_INTERMEDIATE_HOPS },
                > = node_ids.try_into().ok()?;

                Some(DestinationRouting::forward_only(
                    *destination,
                    RoutingOptions::IntermediatePath(bounded),
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hex_literal::hex;
    use hopr_api::graph::{
        EdgeLinkObservable, EdgeTransportTelemetry, MeasurablePath, MeasurablePeer, NetworkGraphError,
        NetworkGraphTraverse, NetworkGraphUpdate, NetworkGraphView, NetworkGraphWrite,
        traits::{EdgeObservableRead, EdgeObservableWrite, EdgeProtocolObservable, EdgeWeightType},
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

    #[derive(Debug, Clone)]
    struct TestNeighbor {
        peer: OffchainPublicKey,
        rtt: std::time::Duration,
    }

    impl MeasurablePeer for TestNeighbor {
        fn peer(&self) -> &OffchainPublicKey {
            &self.peer
        }

        fn rtt(&self) -> std::time::Duration {
            self.rtt
        }
    }

    #[derive(Debug, Clone)]
    struct TestPath;

    impl MeasurablePath for TestPath {
        fn id(&self) -> &[u8] {
            &[]
        }

        fn path(&self) -> &[u8] {
            &[]
        }

        fn timestamp(&self) -> u128 {
            0
        }
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
    async fn record_edge_updates_observation_for_neighbor_probe() -> anyhow::Result<()> {
        let me_kp = OffchainKeypair::from_secret(&SECRET_0)?;
        let me = *me_kp.public();
        let peer_kp = OffchainKeypair::from_secret(&SECRET_1)?;
        let peer_key = *peer_kp.public();

        let graph = ChannelGraph::new(me);
        graph.add_node(peer_key);
        graph.add_edge(&me, &peer_key)?;

        let rtt = std::time::Duration::from_millis(100);
        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Ok(EdgeTransportTelemetry::Neighbor(TestNeighbor { peer: peer_key, rtt }));
        graph
            .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
            .await;

        let obs = graph.edge(&me, &peer_key).context("edge observation should exist")?;
        let immediate = obs
            .immediate_qos()
            .context("immediate QoS should be present after probe")?;
        assert_eq!(immediate.average_latency().context("latency should be set")?, rtt / 2,);
        Ok(())
    }

    #[tokio::test]
    async fn record_edge_records_timeout_as_failed_probe() -> anyhow::Result<()> {
        let me_kp = OffchainKeypair::from_secret(&SECRET_0)?;
        let me = *me_kp.public();
        let peer_kp = OffchainKeypair::from_secret(&SECRET_1)?;
        let peer_key = *peer_kp.public();

        let graph = ChannelGraph::new(me);
        graph.add_node(peer_key);
        graph.add_edge(&me, &peer_key)?;

        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Err(NetworkGraphError::ProbeNeighborTimeout(Box::new(peer_key)));
        graph
            .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
            .await;

        let obs = graph.edge(&me, &peer_key).context("edge observation should exist")?;
        let immediate = obs
            .immediate_qos()
            .context("immediate QoS should be present after failed probe")?;
        assert!(immediate.average_latency().is_none());
        assert!(immediate.average_probe_rate() < 1.0);
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

    // --- NetworkGraphTraverse tests ---

    #[tokio::test]
    async fn routes_returns_direct_route_for_length_one() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;

        let routes = graph.simple_route(&dest, 1).await;

        assert_eq!(routes.len(), 1, "should find exactly one 1-hop route");
        assert!(routes[0].is_forward(), "route should be a forward route");

        Ok(())
    }

    #[tokio::test]
    async fn routes_returns_two_hop_route_through_intermediate() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let hop = pubkey_from(&SECRET_1);
        let dest = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);
        graph.add_edge(&me, &hop)?;
        graph.add_edge(&hop, &dest)?;

        let routes = graph.simple_route(&dest, 2).await;

        assert!(!routes.is_empty(), "should find at least one 2-hop route");

        Ok(())
    }

    #[tokio::test]
    async fn routes_returns_empty_when_no_path_exists() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        // No edge between me and dest

        let routes = graph.simple_route(&dest, 1).await;

        assert!(routes.is_empty(), "should return no routes when unreachable");

        Ok(())
    }

    #[tokio::test]
    async fn routes_returns_empty_for_unknown_destination() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let unknown = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        let routes = graph.simple_route(&unknown, 1).await;

        assert!(routes.is_empty());

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
    fn has_edge_returns_false_when_nodes_not_in_graph() {
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

    #[tokio::test]
    async fn record_edge_capacity_updates_observations() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        let capacity_update = hopr_api::graph::EdgeCapacityUpdate {
            src: me,
            dest: peer,
            capacity: Some(1000),
        };
        graph
            .record_edge::<TestNeighbor, TestPath>(hopr_api::graph::MeasurableEdge::Capacity(Box::new(capacity_update)))
            .await;

        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        let intermediate = obs
            .intermediate_qos()
            .context("intermediate QoS should be present after capacity update")?;
        assert_eq!(intermediate.capacity(), Some(1000));
        Ok(())
    }

    #[tokio::test]
    async fn record_edge_capacity_with_none_value() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        let capacity_update = hopr_api::graph::EdgeCapacityUpdate {
            src: me,
            dest: peer,
            capacity: None,
        };
        graph
            .record_edge::<TestNeighbor, TestPath>(hopr_api::graph::MeasurableEdge::Capacity(Box::new(capacity_update)))
            .await;

        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        let intermediate = obs.intermediate_qos().context("intermediate QoS should be present")?;
        assert_eq!(intermediate.capacity(), None);
        Ok(())
    }

    // --- record_node ---

    #[tokio::test]
    async fn record_node_adds_node_to_graph() {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);

        assert!(!graph.contains_node(&peer));
        graph.record_node(peer).await;
        assert!(graph.contains_node(&peer));
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

    #[tokio::test]
    async fn routes_finds_multiple_paths_in_diamond_topology() -> anyhow::Result<()> {
        //   me -> a -> dest
        //   me -> b -> dest
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let dest = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(dest);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&me, &b)?;
        graph.add_edge(&a, &dest)?;
        graph.add_edge(&b, &dest)?;

        let routes = graph.simple_route(&dest, 2).await;
        assert_eq!(routes.len(), 2, "diamond topology should yield two 2-hop routes");
        Ok(())
    }

    #[tokio::test]
    async fn routes_finds_three_hop_path() -> anyhow::Result<()> {
        // me -> a -> b -> dest
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let dest = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(dest);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &dest)?;

        let routes = graph.simple_route(&dest, 3).await;
        assert_eq!(routes.len(), 1, "should find exactly one 3-hop route");
        Ok(())
    }

    #[tokio::test]
    async fn routes_avoids_cycles() -> anyhow::Result<()> {
        // me -> a -> b -> dest, plus a -> me (back-edge creating cycle)
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let dest = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(dest);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &dest)?;
        graph.add_edge(&a, &me)?; // back-edge

        let routes = graph.simple_route(&dest, 3).await;
        assert_eq!(routes.len(), 1, "cycle should not produce extra paths");
        Ok(())
    }

    #[tokio::test]
    async fn routes_wrong_length_returns_empty() -> anyhow::Result<()> {
        // me -> dest (1 hop), but ask for 2 hops
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;

        let routes = graph.simple_route(&dest, 2).await;
        assert!(routes.is_empty(), "no 2-hop route should exist for a direct edge");
        Ok(())
    }

    #[tokio::test]
    async fn routes_length_zero_to_self_returns_one_route() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);

        let routes = graph.simple_route(&me, 0).await;
        assert_eq!(routes.len(), 1, "zero-hop route to self should find exactly one route");
        Ok(())
    }

    #[tokio::test]
    async fn routes_length_zero_to_other_returns_empty() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);

        let routes = graph.simple_route(&dest, 0).await;
        assert!(routes.is_empty(), "zero-hop route to different node should be empty");
        Ok(())
    }

    #[tokio::test]
    async fn routes_respects_edge_direction() -> anyhow::Result<()> {
        // me -> a, but no a -> dest, only dest -> a
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let dest = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(dest);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&dest, &a)?; // wrong direction

        let routes = graph.simple_route(&dest, 2).await;
        assert!(routes.is_empty(), "should not traverse edge in wrong direction");
        Ok(())
    }

    #[tokio::test]
    async fn record_edge_probe_creates_edge_if_absent() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        // No explicit add_edge — record_edge should upsert

        let rtt = std::time::Duration::from_millis(80);
        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Ok(EdgeTransportTelemetry::Neighbor(TestNeighbor { peer: peer, rtt }));
        graph
            .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
            .await;

        assert!(graph.has_edge(&me, &peer), "probe should create edge via upsert");
        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        assert!(obs.immediate_qos().is_some());
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

    #[tokio::test]
    async fn multiple_probes_accumulate_in_observations() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        // Send several successful probes
        for _ in 0..5 {
            let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
                Ok(EdgeTransportTelemetry::Neighbor(TestNeighbor {
                    peer: peer,
                    rtt: std::time::Duration::from_millis(60),
                }));
            graph
                .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
                .await;
        }

        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        let qos = obs.immediate_qos().context("immediate QoS should exist")?;
        assert_eq!(
            qos.average_latency().context("latency should be set")?,
            std::time::Duration::from_millis(30), // rtt / 2 = 30ms
        );
        assert!(qos.average_probe_rate() > 0.9, "all probes succeeded");
        Ok(())
    }
}
