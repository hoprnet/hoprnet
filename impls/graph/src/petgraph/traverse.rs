use std::{collections::HashSet, hash::RandomState};

use hopr_api::{
    OffchainPublicKey,
    ct::PathId,
    graph::traits::{CostFn, EdgeNetworkObservableRead, EdgeObservableRead},
};
use petgraph::graph::NodeIndex;

use crate::{ChannelGraph, algorithm::all_simple_paths_multi, costs::LoopbackPathCostFn, graph::InnerGraph};

/// Core path-finding routine that runs `all_simple_paths_multi` on the
/// inner petgraph.
#[allow(clippy::too_many_arguments)]
pub(crate) fn find_paths<C, F>(
    inner: &InnerGraph,
    source: NodeIndex,
    destinations: &HashSet<NodeIndex>,
    length: usize,
    take_count: Option<usize>,
    initial_cost: C,
    min_cost: Option<C>,
    cost_fn: F,
) -> Vec<(Vec<OffchainPublicKey>, PathId, C)>
where
    C: Clone + PartialOrd,
    F: Fn(C, &crate::Observations, usize) -> C,
{
    let intermediates = length - 1;

    let paths = all_simple_paths_multi::<Vec<_>, _, RandomState, _, _>(
        &inner.graph,
        source,
        destinations,
        intermediates,
        Some(intermediates),
        initial_cost,
        min_cost,
        cost_fn,
    )
    .filter_map(|(node_indices, final_cost)| {
        // Build PathId from node indices along the path
        let mut path_id: PathId = [0u64; 5];
        for (i, &node_idx) in node_indices.iter().enumerate() {
            if i >= path_id.len() {
                return None;
            }
            path_id[i] = node_idx.index() as u64;
        }

        // Convert node indices to public keys
        let nodes = node_indices
            .into_iter()
            .filter_map(|v| inner.indices.get_by_right(&v).copied())
            .collect::<Vec<_>>();
        // Path includes source + intermediates + destination = length + 1 nodes
        (nodes.len() == length + 1).then_some((nodes, path_id, final_cost))
    });

    if let Some(take_count) = take_count {
        paths.take(take_count).collect::<Vec<_>>()
    } else {
        paths.collect::<Vec<_>>()
    }
}

impl hopr_api::graph::NetworkGraphTraverse for ChannelGraph {
    type NodeId = OffchainPublicKey;
    type Observed = crate::Observations;

    fn simple_paths<C: CostFn<Weight = Self::Observed>>(
        &self,
        source: &Self::NodeId,
        destination: &Self::NodeId,
        length: usize,
        take_count: Option<usize>,
        cost_fn: C,
    ) -> Vec<(Vec<Self::NodeId>, PathId, C::Cost)> {
        if length == 0 {
            return Default::default();
        }

        let inner = self.inner.read();
        let Some(start) = inner.indices.get_by_left(source) else {
            return Default::default();
        };
        let Some(end) = inner.indices.get_by_left(destination) else {
            return Default::default();
        };
        let end = HashSet::from_iter([*end]);

        find_paths(
            &inner,
            *start,
            &end,
            length,
            take_count,
            cost_fn.initial_cost(),
            cost_fn.min_cost(),
            cost_fn.into_cost_fn(),
        )
    }

    fn simple_loopback_to_self(&self, length: usize, take_count: Option<usize>) -> Vec<(Vec<Self::NodeId>, PathId)> {
        if length > 1 {
            let inner = self.inner.read();

            if let Some(me_idx) = inner.indices.get_by_left(&self.me) {
                let connected_neighbors = inner
                    .graph
                    .neighbors(*me_idx)
                    .filter(|neighbor| {
                        inner
                            .graph
                            .edges_connecting(*me_idx, *neighbor)
                            .next()
                            .and_then(|e| e.weight().immediate_qos().map(|e| e.is_connected()))
                            .unwrap_or(false)
                    })
                    .collect::<HashSet<_>>();

                let cost_fn = LoopbackPathCostFn::new();

                return find_paths(
                    &inner,
                    *me_idx,
                    &connected_neighbors,
                    length,
                    take_count,
                    cost_fn.initial_cost(),
                    cost_fn.min_cost(),
                    cost_fn.into_cost_fn(),
                )
                .into_iter()
                .map(|(mut a, mut b, _c)| {
                    // Append me's node index to close the loopback in the PathId
                    let path_node_count = a.len();
                    if path_node_count < b.len() {
                        b[path_node_count] = me_idx.index() as u64;
                    }
                    a.push(self.me);
                    (a, b)
                })
                .collect();
            };
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_api::graph::{
        NetworkGraphTraverse, NetworkGraphWrite,
        traits::{EdgeObservableWrite, EdgeWeightType},
    };
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};

    use super::*;
    use crate::costs::SimpleHoprCostFn;

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

    /// Marks an edge as connected with an immediate probe measurement, satisfying the
    /// cost function's requirement for the last edge in a path.
    fn mark_edge_connected(graph: &ChannelGraph, src: &OffchainPublicKey, dest: &OffchainPublicKey) {
        graph.upsert_edge(src, dest, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
        });
    }

    #[test]
    fn one_hop_path_should_return_direct_route() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;
        mark_edge_connected(&graph, &me, &dest);

        let routes = graph.simple_paths(&me, &dest, 1, None, SimpleHoprCostFn::new(1));

        assert_eq!(routes.len(), 1, "should find exactly one 1-hop route");

        Ok(())
    }

    #[test]
    fn two_hop_path_should_route_through_intermediate() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let hop = pubkey_from(&SECRET_1);
        let dest = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);
        graph.add_edge(&me, &hop)?;
        graph.add_edge(&hop, &dest)?;
        mark_edge_connected(&graph, &hop, &dest);

        let routes = graph.simple_paths(&me, &dest, 2, None, SimpleHoprCostFn::new(2));

        assert!(!routes.is_empty(), "should find at least one 2-hop route");

        Ok(())
    }

    #[test]
    fn unreachable_destination_should_return_empty() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        // No edge between me and dest

        let routes = graph.simple_paths(&me, &dest, 1, None, SimpleHoprCostFn::new(1));

        assert!(routes.is_empty(), "should return no routes when unreachable");

        Ok(())
    }

    #[test]
    fn unknown_destination_should_return_empty() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let unknown = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        let routes = graph.simple_paths(&me, &unknown, 1, None, SimpleHoprCostFn::new(1));

        assert!(routes.is_empty());

        Ok(())
    }

    #[test]
    fn diamond_topology_should_yield_multiple_paths() -> anyhow::Result<()> {
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
        mark_edge_connected(&graph, &a, &dest);
        mark_edge_connected(&graph, &b, &dest);

        let routes = graph.simple_paths(&me, &dest, 2, None, SimpleHoprCostFn::new(2));
        assert_eq!(routes.len(), 2, "diamond topology should yield two 2-hop routes");
        Ok(())
    }

    #[test]
    fn three_hop_chain_should_find_single_path() -> anyhow::Result<()> {
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
        mark_edge_connected(&graph, &b, &dest);

        let routes = graph.simple_paths(&me, &dest, 3, None, SimpleHoprCostFn::new(3));
        assert_eq!(routes.len(), 1, "should find exactly one 3-hop route");
        Ok(())
    }

    #[test]
    fn back_edge_should_not_produce_cyclic_paths() -> anyhow::Result<()> {
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
        mark_edge_connected(&graph, &b, &dest);

        let routes = graph.simple_paths(&me, &dest, 3, None, SimpleHoprCostFn::new(3));
        assert_eq!(routes.len(), 1, "cycle should not produce extra paths");
        Ok(())
    }

    #[test]
    fn mismatched_hop_count_should_return_empty() -> anyhow::Result<()> {
        // me -> dest (1 hop), but ask for 2 hops
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;

        let routes = graph.simple_paths(&me, &dest, 2, None, SimpleHoprCostFn::new(2));
        assert!(routes.is_empty(), "no 2-hop route should exist for a direct edge");
        Ok(())
    }

    #[test]
    fn zero_hop_should_always_return_empty() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let other = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);

        let routes = graph.simple_paths(&me, &other, 0, None, SimpleHoprCostFn::new(0));
        assert!(routes.is_empty(), "zero-hop path should find no routes");
        Ok(())
    }

    #[test]
    fn reverse_edge_should_not_be_traversable() -> anyhow::Result<()> {
        // me -> a, but no a -> dest, only dest -> a
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let dest = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(dest);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&dest, &a)?; // wrong direction

        let routes = graph.simple_paths(&me, &dest, 2, None, SimpleHoprCostFn::new(2));
        assert!(routes.is_empty(), "should not traverse edge in wrong direction");
        Ok(())
    }

    #[test]
    fn non_trivial_graph_should_find_all_simple_paths() -> anyhow::Result<()> {
        // Topology (7 nodes):
        //
        //         ┌─→ c ──→ f
        //    me ──┤         ↑↑↑
        //         └─→ a ──→ d ──→ f
        //              │         ↑
        //              └──→ e ──┘
        //
        //   me(0) ──→ a(1)
        //   me(0) ──→ b(2)
        //   a(1)  ──→ c(3)
        //   a(1)  ──→ d(4)
        //   a(1)  ──→ f(7)   [NOT connected — pruned by cost]
        //   b(2)  ──→ c(3)
        //   b(2)  ──→ d(4)
        //   b(2)  ──→ e(5)
        //   b(2)  ──→ f(7)   [NOT connected — pruned by cost]
        //   c(3)  ──→ f(7)   [connected]
        //   d(4)  ──→ f(7)   [connected]
        //   e(5)  ──→ f(7)   [connected]
        //
        // Valid 3-hop paths (me → ? → ? → f):
        //   1. me → a → c → f
        //   2. me → a → d → f
        //   3. me → b → c → f
        //   4. me → b → d → f
        //   5. me → b → e → f
        //
        // Blocked paths:
        //   - me → a → e → f : edge a→e missing
        //   - me → e → … → f : edge me→e missing
        //
        // Cost-pruned 2-hop paths:
        //   - me → a → f : a→f not connected → cost goes negative
        //   - me → b → f : b→f not connected → cost goes negative

        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);
        let d = pubkey_from(&SECRET_4);
        let e = pubkey_from(&SECRET_5);
        let f = pubkey_from(&SECRET_7);

        let graph = ChannelGraph::new(me);
        for node in [a, b, c, d, e, f] {
            graph.add_node(node);
        }

        // Edges from me
        graph.add_edge(&me, &a)?;
        graph.add_edge(&me, &b)?;

        // Edges from a
        graph.add_edge(&a, &c)?;
        graph.add_edge(&a, &d)?;
        graph.add_edge(&a, &f)?; // exists but NOT connected

        // Edges from b
        graph.add_edge(&b, &c)?;
        graph.add_edge(&b, &d)?;
        graph.add_edge(&b, &e)?;
        graph.add_edge(&b, &f)?; // exists but NOT connected

        // Edges to f (last hop)
        graph.add_edge(&c, &f)?;
        graph.add_edge(&d, &f)?;
        graph.add_edge(&e, &f)?;

        // Only mark c→f, d→f, e→f as connected (satisfies cost function for last edge)
        mark_edge_connected(&graph, &c, &f);
        mark_edge_connected(&graph, &d, &f);
        mark_edge_connected(&graph, &e, &f);

        // --- 3-hop paths: should find exactly 5 ---
        let routes_3 = graph.simple_paths(&me, &f, 3, None, SimpleHoprCostFn::new(3));
        assert_eq!(routes_3.len(), 5, "should find exactly 5 three-hop paths");

        // Verify all returned paths have positive cost
        for (path, _path_id, cost) in &routes_3 {
            assert!(*cost > 0.0, "path {path:?} should have positive cost, got {cost}");
            assert_eq!(path.len(), 4, "3-hop path should contain 4 nodes");
            assert_eq!(path.first(), Some(&me), "path should start at me");
            assert_eq!(path.last(), Some(&f), "path should end at f");
        }

        // --- 2-hop paths: a→f and b→f are NOT connected, so cost is pruned ---
        let routes_2 = graph.simple_paths(&me, &f, 2, None, SimpleHoprCostFn::new(2));
        assert!(
            routes_2.is_empty(),
            "2-hop paths should be pruned (last edge not connected)"
        );

        // --- 1-hop path: no direct me→f edge ---
        let routes_1 = graph.simple_paths(&me, &f, 1, None, SimpleHoprCostFn::new(1));
        assert!(routes_1.is_empty(), "no direct edge from me to f");

        Ok(())
    }

    #[test]
    fn three_edge_loop_should_return_empty_because_source_is_visited() -> anyhow::Result<()> {
        // Ring topology: me → a → b → me (3 edges forming a cycle)
        //
        // The underlying all_simple_paths_multi algorithm marks the source node
        // as visited before traversal begins. Because the destination equals the
        // source, the algorithm can never "reach" it — the visited-set check
        // (`visited.contains(&child)`) rejects the back-edge to source, and the
        // expansion guard (`to.iter().any(|n| !visited.contains(n))`) is always
        // false since the only target (source) is always visited.
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &me)?;
        mark_edge_connected(&graph, &b, &me);

        let routes = graph.simple_paths(&me, &me, 3, None, SimpleHoprCostFn::new(3));
        assert!(
            routes.is_empty(),
            "simple_paths cannot discover cycles (source == destination) due to visited-set semantics"
        );

        Ok(())
    }

    #[test]
    fn path_id_should_contain_node_indices_for_one_hop() -> anyhow::Result<()> {
        // me = node 0, dest = node 1
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;
        mark_edge_connected(&graph, &me, &dest);

        let routes = graph.simple_paths(&me, &dest, 1, None, SimpleHoprCostFn::new(1));
        assert_eq!(routes.len(), 1);

        let (_path, path_id, _cost) = &routes[0];
        assert_eq!(path_id[0], 0, "first node should be me (node index 0)");
        assert_eq!(path_id[1], 1, "second node should be dest (node index 1)");
        assert_eq!(path_id[2..], [0, 0, 0], "unused positions should be 0");

        Ok(())
    }

    #[test]
    fn path_id_should_contain_node_indices_for_three_hops() -> anyhow::Result<()> {
        // me = node 0, a = node 1, b = node 2, dest = node 3
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
        mark_edge_connected(&graph, &b, &dest);

        let routes = graph.simple_paths(&me, &dest, 3, None, SimpleHoprCostFn::new(3));
        assert_eq!(routes.len(), 1);

        let (_path, path_id, _cost) = &routes[0];
        assert_eq!(path_id[0], 0, "me should be node index 0");
        assert_eq!(path_id[1], 1, "a should be node index 1");
        assert_eq!(path_id[2], 2, "b should be node index 2");
        assert_eq!(path_id[3], 3, "dest should be node index 3");
        assert_eq!(path_id[4], 0, "unused position should be 0");

        Ok(())
    }

    #[test]
    fn path_id_should_differ_for_distinct_paths_in_diamond() -> anyhow::Result<()> {
        //   me → a → dest
        //   me → b → dest
        // me = node 0, a = node 1, b = node 2, dest = node 3
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
        mark_edge_connected(&graph, &a, &dest);
        mark_edge_connected(&graph, &b, &dest);

        let routes = graph.simple_paths(&me, &dest, 2, None, SimpleHoprCostFn::new(2));
        assert_eq!(routes.len(), 2, "diamond should yield two 2-hop routes");

        let path_ids: Vec<PathId> = routes.iter().map(|(_, pid, _)| *pid).collect();
        assert_ne!(path_ids[0], path_ids[1], "distinct paths should have different PathIds");

        // Each path: [me(0), intermediate(1 or 2), dest(3), 0, 0]
        for pid in &path_ids {
            assert_eq!(pid[0], 0, "first node should be me (node index 0)");
            assert!(pid[1] == 1 || pid[1] == 2, "second node should be a (1) or b (2)");
            assert_eq!(pid[2], 3, "third node should be dest (node index 3)");
            assert_eq!(pid[3..], [0, 0], "unused positions should be 0");
        }

        Ok(())
    }

    // ── simple_loopback_to_self tests ──────────────────────────────────

    /// Marks an edge as connected AND with intermediate capacity so that it
    /// satisfies the `LoopbackPathCostFn` at edge index 0 (connected + capacity)
    /// and at any other index (capacity).
    fn mark_edge_loopback_ready(graph: &ChannelGraph, src: &OffchainPublicKey, dest: &OffchainPublicKey) {
        graph.upsert_edge(src, dest, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });
    }

    /// Marks an edge with intermediate capacity and probe data (no connected flag).
    /// Satisfies `LoopbackPathCostFn` at index > 0 but NOT at index 0.
    fn mark_edge_with_capacity(graph: &ChannelGraph, src: &OffchainPublicKey, dest: &OffchainPublicKey) {
        graph.upsert_edge(src, dest, |obs| {
            obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });
    }

    #[test]
    fn loopback_returns_empty_for_length_zero() {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        assert!(graph.simple_loopback_to_self(0, None).is_empty());
    }

    #[test]
    fn loopback_returns_empty_for_length_one() {
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a).unwrap();
        mark_edge_loopback_ready(&graph, &me, &a);

        assert!(
            graph.simple_loopback_to_self(1, None).is_empty(),
            "length=1 is below the minimum threshold"
        );
    }

    #[test]
    fn loopback_returns_empty_without_any_peers() {
        let me = pubkey_from(&SECRET_0);
        let graph = ChannelGraph::new(me);
        assert!(
            graph.simple_loopback_to_self(2, None).is_empty(),
            "no peers means no connected neighbors"
        );
    }

    #[test]
    fn loopback_returns_empty_without_connected_neighbors() -> anyhow::Result<()> {
        // me → a → b, me → b exists but is NOT connected
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&me, &b)?;
        // me→b is NOT marked connected, so b is not in connected_neighbors
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_with_capacity(&graph, &a, &b);

        assert!(
            graph.simple_loopback_to_self(2, None).is_empty(),
            "b is not a connected neighbor, so no loopback destinations exist"
        );

        Ok(())
    }

    #[test]
    fn loopback_returns_empty_when_first_hop_lacks_capacity() -> anyhow::Result<()> {
        // me → a → b, me → b (connected)
        // me→a is connected but has NO intermediate capacity → edge-0 cost goes negative
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&me, &b)?;
        // me→a: connected but no capacity (only Connected + Immediate)
        mark_edge_connected(&graph, &me, &a);
        // a→b: has capacity
        mark_edge_with_capacity(&graph, &a, &b);
        // me→b: connected (makes b a connected neighbor)
        mark_edge_connected(&graph, &me, &b);

        assert!(
            graph.simple_loopback_to_self(2, None).is_empty(),
            "edge me→a lacks intermediate capacity, so LoopbackPathCostFn prunes it"
        );

        Ok(())
    }

    #[test]
    fn loopback_returns_empty_when_intermediate_edge_lacks_capacity() -> anyhow::Result<()> {
        // me → a → b, me → b (connected)
        // me→a passes cost-0, but a→b has NO capacity → cost goes negative at edge-1
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&me, &b)?;
        // me→a: connected + capacity (passes edge-0)
        mark_edge_loopback_ready(&graph, &me, &a);
        // a→b: NO capacity — default edge weight
        // me→b: connected
        mark_edge_connected(&graph, &me, &b);

        assert!(
            graph.simple_loopback_to_self(2, None).is_empty(),
            "edge a→b lacks capacity, so LoopbackPathCostFn prunes the path"
        );

        Ok(())
    }

    #[test]
    fn loopback_two_edge_triangle() -> anyhow::Result<()> {
        // Topology: me → a → b, me → b (connected)
        // Loopback path: me → a → b → me
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&me, &b)?;
        // me→a: connected + capacity (edge-0 cost passes)
        mark_edge_loopback_ready(&graph, &me, &a);
        // a→b: capacity (edge-1 cost passes)
        mark_edge_with_capacity(&graph, &a, &b);
        // me→b: connected (makes b a connected neighbor destination)
        mark_edge_connected(&graph, &me, &b);

        let routes = graph.simple_loopback_to_self(2, None);
        assert_eq!(routes.len(), 1, "should find exactly one 2-edge loopback");

        let (path, _path_id) = &routes[0];
        assert_eq!(
            path.len(),
            4,
            "loopback path should have 4 nodes (2 internal edges + appended me)"
        );
        assert_eq!(path.first(), Some(&me), "path should start with me");
        assert_eq!(path.last(), Some(&me), "path should end with me (appended)");
        assert_eq!(path[1], a, "first intermediate should be a");
        assert_eq!(path[2], b, "destination (connected neighbor) should be b");

        Ok(())
    }

    #[test]
    fn loopback_three_edge_chain() -> anyhow::Result<()> {
        // Topology: me → a → b → c, me → c (connected)
        // Loopback path: me → a → b → c → me
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&me, &c)?;
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_with_capacity(&graph, &a, &b);
        mark_edge_with_capacity(&graph, &b, &c);
        mark_edge_connected(&graph, &me, &c);

        let routes = graph.simple_loopback_to_self(3, None);
        assert_eq!(routes.len(), 1, "should find exactly one 3-edge loopback");

        let (path, _path_id) = &routes[0];
        assert_eq!(path.len(), 5, "3-edge internal path + appended me = 5 nodes");
        assert_eq!(path.first(), Some(&me), "starts with me");
        assert_eq!(path.last(), Some(&me), "ends with me");
        assert_eq!(&path[1..4], &[a, b, c], "interior nodes");

        Ok(())
    }

    #[test]
    fn loopback_multiple_paths_through_diamond() -> anyhow::Result<()> {
        // Topology:
        //   me → a → c, me → b → c, me → c (connected)
        // Two 2-edge loopback paths: me → a → c → me, me → b → c → me
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&me, &b)?;
        graph.add_edge(&a, &c)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&me, &c)?;
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_loopback_ready(&graph, &me, &b);
        mark_edge_with_capacity(&graph, &a, &c);
        mark_edge_with_capacity(&graph, &b, &c);
        mark_edge_connected(&graph, &me, &c);

        let routes = graph.simple_loopback_to_self(2, None);
        assert_eq!(routes.len(), 2, "diamond should yield two 2-edge loopback paths");

        for (path, _path_id) in &routes {
            assert_eq!(path.first(), Some(&me), "every path starts with me");
            assert_eq!(path.last(), Some(&me), "every path ends with me");
            assert_eq!(path[path.len() - 2], c, "penultimate node is c (connected neighbor)");
        }

        // Verify distinct intermediates (a and b)
        let intermediates: HashSet<_> = routes.iter().map(|(p, _)| p[1]).collect();
        assert!(intermediates.contains(&a), "should include path through a");
        assert!(intermediates.contains(&b), "should include path through b");

        Ok(())
    }

    #[test]
    fn loopback_to_multiple_connected_neighbors() -> anyhow::Result<()> {
        // Topology: me → a, me → b (both connected)
        // a and b are both connected neighbors of me.
        // With length=2: me → a → b → me and me → b → a → me
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&me, &b)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &a)?;
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_loopback_ready(&graph, &me, &b);
        mark_edge_with_capacity(&graph, &a, &b);
        mark_edge_with_capacity(&graph, &b, &a);

        let routes = graph.simple_loopback_to_self(2, None);
        assert_eq!(
            routes.len(),
            2,
            "should find loopback paths to both connected neighbors"
        );

        // Both paths start and end with me
        for (path, _) in &routes {
            assert_eq!(path.first(), Some(&me));
            assert_eq!(path.last(), Some(&me));
        }

        // Collect the connected-neighbor destinations (penultimate node)
        let destinations: HashSet<_> = routes.iter().map(|(p, _)| p[p.len() - 2]).collect();
        assert_eq!(destinations.len(), 2, "should reach both connected neighbors");
        assert!(destinations.contains(&a));
        assert!(destinations.contains(&b));

        Ok(())
    }

    #[test]
    fn loopback_disconnected_neighbor_is_excluded() -> anyhow::Result<()> {
        // me → a → b, me → a → c
        // me → b (connected), me → c (NOT connected)
        // length=2: only me → a → b → me should be found, not me → a → c → me
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&a, &c)?;
        graph.add_edge(&me, &b)?;
        graph.add_edge(&me, &c)?;
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_with_capacity(&graph, &a, &b);
        mark_edge_with_capacity(&graph, &a, &c);
        // me→b: connected (b IS a connected neighbor)
        mark_edge_connected(&graph, &me, &b);
        // me→c: NOT connected (c is NOT a connected neighbor)

        let routes = graph.simple_loopback_to_self(2, None);
        assert_eq!(routes.len(), 1, "only the path to connected neighbor b should be found");

        let (path, _) = &routes[0];
        assert_eq!(path[path.len() - 2], b, "destination should be b, not c");

        Ok(())
    }

    #[test]
    fn loopback_take_count_limits_results() -> anyhow::Result<()> {
        // Create 3 possible loopback paths, but take_count=1
        //   me → a → d, me → b → d, me → c → d, me → d (connected)
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);
        let d = pubkey_from(&SECRET_4);

        let graph = ChannelGraph::new(me);
        for node in [a, b, c, d] {
            graph.add_node(node);
        }
        graph.add_edge(&me, &a)?;
        graph.add_edge(&me, &b)?;
        graph.add_edge(&me, &c)?;
        graph.add_edge(&me, &d)?;
        graph.add_edge(&a, &d)?;
        graph.add_edge(&b, &d)?;
        graph.add_edge(&c, &d)?;
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_loopback_ready(&graph, &me, &b);
        mark_edge_loopback_ready(&graph, &me, &c);
        mark_edge_with_capacity(&graph, &a, &d);
        mark_edge_with_capacity(&graph, &b, &d);
        mark_edge_with_capacity(&graph, &c, &d);
        mark_edge_connected(&graph, &me, &d);

        // Without limit: should find 3 paths
        let all_routes = graph.simple_loopback_to_self(2, None);
        assert_eq!(all_routes.len(), 3, "should find 3 loopback paths without limit");

        // With take_count=1: should return exactly 1
        let limited = graph.simple_loopback_to_self(2, Some(1));
        assert_eq!(limited.len(), 1, "take_count=1 should limit to 1 result");

        Ok(())
    }

    #[test]
    fn loopback_path_ids_differ_for_distinct_routes() -> anyhow::Result<()> {
        // me → a → c, me → b → c, me → c (connected)
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&me, &b)?;
        graph.add_edge(&a, &c)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&me, &c)?;
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_loopback_ready(&graph, &me, &b);
        mark_edge_with_capacity(&graph, &a, &c);
        mark_edge_with_capacity(&graph, &b, &c);
        mark_edge_connected(&graph, &me, &c);

        let routes = graph.simple_loopback_to_self(2, None);
        assert_eq!(routes.len(), 2);

        let path_ids: Vec<PathId> = routes.iter().map(|(_, pid)| *pid).collect();
        assert_ne!(
            path_ids[0], path_ids[1],
            "distinct loopback paths should have different PathIds"
        );

        Ok(())
    }

    #[test]
    fn loopback_mismatched_length_returns_empty() -> anyhow::Result<()> {
        // Topology only supports 2-edge internal path, but we request 3
        // me → a → b, me → b (connected)
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&me, &b)?;
        mark_edge_loopback_ready(&graph, &me, &a);
        mark_edge_with_capacity(&graph, &a, &b);
        mark_edge_connected(&graph, &me, &b);

        // length=2 works
        assert_eq!(graph.simple_loopback_to_self(2, None).len(), 1);
        // length=3 has no 3-edge path to any connected neighbor
        assert!(
            graph.simple_loopback_to_self(3, None).is_empty(),
            "no 3-edge internal path exists"
        );

        Ok(())
    }
}
