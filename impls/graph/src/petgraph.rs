use std::collections::HashSet;

use bimap::BiHashMap;
use hopr_api::{
    OffchainPublicKey,
    graph::{MeasurableEdge, MeasurableNode},
};
use parking_lot::RwLock;
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{errors::ChannelGraphError, weight::Observations};

/// Internal mutable state of a [`ChannelGraph`], protected by a lock.
#[derive(Debug, Clone, Default)]
struct InnerGraph {
    graph: DiGraph<OffchainPublicKey, Observations>,
    indices: BiHashMap<OffchainPublicKey, NodeIndex>,
    connected: HashSet<OffchainPublicKey>,
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
            inner: RwLock::new(InnerGraph {
                graph,
                indices,
                ..Default::default()
            }),
        }
    }

    /// Returns the self-identity key of this graph.
    pub fn me(&self) -> &OffchainPublicKey {
        &self.me
    }

    /// Mutably updates the edge observations between two nodes.
    ///
    /// If the edge exists, applies the given function to its observations.
    fn update_edge<F>(&self, src: &OffchainPublicKey, dest: &OffchainPublicKey, f: F)
    where
        F: FnOnce(&mut Observations),
    {
        let mut inner = self.inner.write();
        if let (Some(src_idx), Some(dest_idx)) = (
            inner.indices.get_by_left(src).copied(),
            inner.indices.get_by_left(dest).copied(),
        ) {
            if let Some(edge_idx) = inner.graph.find_edge(src_idx, dest_idx) {
                if let Some(weight) = inner.graph.edge_weight_mut(edge_idx) {
                    f(weight);
                }
            }
        }
    }

    /// Finds all simple paths of exactly `length` hops from `src` to `dest`.
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

    /// Finds all simple cycles that start and end at `me` with length between 2 and
    /// `MAX_INTERMEDIATE_HOPS + 1`.
    fn find_loopback_paths(&self) -> Vec<Vec<OffchainPublicKey>> {
        let inner = self.inner.read();
        let Some(me_idx) = inner.indices.get_by_left(&self.me) else {
            return vec![];
        };

        let mut results = Vec::new();

        // Try cycle lengths from 2 (me -> A -> me) to MAX_INTERMEDIATE_HOPS + 1
        for cycle_len in 2..=4 {
            let mut visited = vec![false; inner.graph.node_count()];
            let mut current_path = Vec::new();
            visited[me_idx.index()] = true;

            Self::dfs_loopback(
                &inner,
                *me_idx,
                *me_idx,
                cycle_len,
                &mut visited,
                &mut current_path,
                &mut results,
            );
        }

        results
    }

    /// DFS helper for finding loopback (cycle) paths.
    fn dfs_loopback(
        inner: &InnerGraph,
        current: NodeIndex,
        home: NodeIndex,
        remaining_hops: usize,
        visited: &mut Vec<bool>,
        current_path: &mut Vec<OffchainPublicKey>,
        results: &mut Vec<Vec<OffchainPublicKey>>,
    ) {
        if remaining_hops == 0 {
            return;
        }

        for neighbor in inner.graph.neighbors(current) {
            if neighbor == home && remaining_hops == 1 {
                // Completed a cycle back to home
                results.push(current_path.clone());
                continue;
            }

            if !visited[neighbor.index()] && remaining_hops > 1 {
                visited[neighbor.index()] = true;
                if let Some(key) = inner.indices.get_by_right(&neighbor) {
                    current_path.push(*key);
                    Self::dfs_loopback(
                        inner,
                        neighbor,
                        home,
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
    async fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: hopr_api::graph::MeasurablePeer + Send + Clone,
        P: hopr_api::graph::MeasurablePath + Send + Clone,
    {
        use hopr_api::graph::traits::{EdgeObservable, EdgeWeightType};

        match update {
            MeasurableEdge::Probe(Ok(hopr_api::graph::EdgeTransportTelemetry::Neighbor(ref telemetry))) => {
                tracing::trace!(
                    peer = %telemetry.peer(),
                    latency_ms = telemetry.rtt().as_millis(),
                    "neighbor probe successful"
                );

                self.update_edge(&self.me, telemetry.peer(), |obs| {
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

                self.update_edge(&self.me, peer, |obs| {
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
                self.update_edge(&update.src, &update.dest, |obs: &mut Observations| {
                    obs.record(EdgeWeightType::Capacity(update.capacity));
                });
            }
        }
    }

    async fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + Clone + Send + Sync + 'static,
    {
        hopr_api::graph::NetworkGraphWrite::add_node(self, update.id().clone());
        if update.is_connected() {
            self.inner.write().connected.insert(*update.id());
        } else {
            self.inner.write().connected.remove(update.id());
        }
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
}

#[async_trait::async_trait]
impl hopr_api::graph::NetworkGraphTraverse for ChannelGraph {
    type NodeId = OffchainPublicKey;

    async fn routes(&self, destination: &Self::NodeId, length: usize) -> Vec<hopr_api::ct::DestinationRouting> {
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

    async fn loopback_routes(&self) -> Vec<Vec<hopr_api::ct::DestinationRouting>> {
        use hopr_internal_types::prelude::NodeId;
        use hopr_network_types::types::{DestinationRouting, RoutingOptions};

        let paths = self.find_loopback_paths();

        // Group all loopback routes into a single batch
        let routes: Vec<DestinationRouting> = paths
            .into_iter()
            .filter_map(|intermediates| {
                let node_ids: Vec<NodeId> = intermediates.into_iter().map(NodeId::from).collect();
                let bounded: hopr_primitive_types::bounded::BoundedVec<
                    NodeId,
                    { RoutingOptions::MAX_INTERMEDIATE_HOPS },
                > = node_ids.try_into().ok()?;

                Some(DestinationRouting::forward_only(
                    self.me,
                    RoutingOptions::IntermediatePath(bounded),
                ))
            })
            .collect();

        if routes.is_empty() { vec![] } else { vec![routes] }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hex_literal::hex;
    use hopr_api::graph::{
        EdgeTransportObservable, EdgeTransportTelemetry, MeasurablePath, MeasurablePeer, NetworkGraphError,
        NetworkGraphUpdate, NetworkGraphView, NetworkGraphWrite, traits::EdgeObservable,
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

        fn seq_id(&self) -> u128 {
            0
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
            Err(NetworkGraphError::ProbeNeighborTimeout(peer_key));
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
        use hopr_api::graph::NetworkGraphTraverse;

        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;

        let routes = graph.routes(&dest, 1).await;

        assert_eq!(routes.len(), 1, "should find exactly one 1-hop route");
        assert!(routes[0].is_forward(), "route should be a forward route");

        Ok(())
    }

    #[tokio::test]
    async fn routes_returns_two_hop_route_through_intermediate() -> anyhow::Result<()> {
        use hopr_api::graph::NetworkGraphTraverse;

        let me = pubkey_from(&SECRET_0);
        let hop = pubkey_from(&SECRET_1);
        let dest = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);
        graph.add_edge(&me, &hop)?;
        graph.add_edge(&hop, &dest)?;

        let routes = graph.routes(&dest, 2).await;

        assert!(!routes.is_empty(), "should find at least one 2-hop route");

        Ok(())
    }

    #[tokio::test]
    async fn routes_returns_empty_when_no_path_exists() -> anyhow::Result<()> {
        use hopr_api::graph::NetworkGraphTraverse;

        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        // No edge between me and dest

        let routes = graph.routes(&dest, 1).await;

        assert!(routes.is_empty(), "should return no routes when unreachable");

        Ok(())
    }

    #[tokio::test]
    async fn routes_returns_empty_for_unknown_destination() -> anyhow::Result<()> {
        use hopr_api::graph::NetworkGraphTraverse;

        let me = pubkey_from(&SECRET_0);
        let unknown = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        let routes = graph.routes(&unknown, 1).await;

        assert!(routes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn loopback_routes_finds_cycle_through_peers() -> anyhow::Result<()> {
        use hopr_api::graph::NetworkGraphTraverse;

        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        // Create a cycle: me -> a -> b -> me
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &me)?;

        let routes = graph.loopback_routes().await;

        assert!(!routes.is_empty(), "should find at least one loopback route batch");

        Ok(())
    }

    #[tokio::test]
    async fn loopback_routes_returns_empty_when_no_cycle() -> anyhow::Result<()> {
        use hopr_api::graph::NetworkGraphTraverse;

        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a)?;
        // a has no outgoing edge back, so no cycle

        let routes = graph.loopback_routes().await;

        assert!(routes.is_empty(), "should return empty when no cycle exists");

        Ok(())
    }
}
