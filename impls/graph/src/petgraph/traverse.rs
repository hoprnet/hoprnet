use std::{collections::HashSet, hash::RandomState};

use async_stream::stream;
use futures::stream::BoxStream;
use hopr_api::OffchainPublicKey;

use crate::{ChannelGraph, algorithm::all_simple_paths_multi, build_hopr_cost_fn};

impl hopr_api::graph::NetworkGraphTraverse for ChannelGraph {
    type Cost = f64;
    type NodeId = OffchainPublicKey;

    fn simple_paths(
        &self,
        source: &Self::NodeId,
        destination: &Self::NodeId,
        length: usize,
        take_count: Option<usize>,
    ) -> Vec<(Vec<Self::NodeId>, Self::Cost)> {
        if length == 0 {
            Default::default()
        }

        let inner = self.inner.read();
        let Some(start) = inner.indices.get_by_left(source) else {
            return Default::default();
        };
        let Some(end) = inner.indices.get_by_left(destination) else {
            return Default::default();
        };
        let end = HashSet::from_iter([*end]);
        let intermediates = length - 1;

        let paths = all_simple_paths_multi::<Vec<_>, _, RandomState, _, _>(
            &inner.graph,
            *start,
            &end,
            intermediates,
            Some(intermediates),
            1.0,
            Some(0.0),
            build_hopr_cost_fn(length),
        )
        .filter_map(|(nodes, final_cost)| {
            // 20260217: replacable by `try_collect` that is still unstable
            let nodes = nodes
                .into_iter()
                .filter_map(|v| inner.indices.get_by_right(&v).copied())
                .collect::<Vec<_>>();
            // Path includes source + intermediates + destination = length + 1 nodes
            (nodes.len() == length + 1).then_some((nodes, final_cost))
        });

        if let Some(take_count) = take_count {
            paths.take(take_count).collect::<Vec<_>>()
        } else {
            paths.collect::<Vec<_>>()
        }
    }

    fn simple_paths_stream(
        &self,
        source: &Self::NodeId,
        destination: &Self::NodeId,
        length: usize,
    ) -> BoxStream<'static, (Vec<Self::NodeId>, Self::Cost)> {
        let this = self.clone();
        let source = *source;
        let destination = *destination;
        Box::pin(stream! {
            loop {
                let paths = this.simple_paths(&source, &destination, length, Some(20));
                if paths.is_empty() {
                    break
                } else {
                    for path in paths {
                        yield path
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use futures::StreamExt;
    use hex_literal::hex;
    use hopr_api::graph::{
        NetworkGraphTraverse, NetworkGraphWrite,
        traits::{EdgeObservableWrite, EdgeWeightType},
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

        let routes = graph.simple_paths(&me, &dest, 1, None);

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

        let routes = graph.simple_paths(&me, &dest, 2, None);

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

        let routes = graph.simple_paths(&me, &dest, 1, None);

        assert!(routes.is_empty(), "should return no routes when unreachable");

        Ok(())
    }

    #[test]
    fn unknown_destination_should_return_empty() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let unknown = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        let routes = graph.simple_paths(&me, &unknown, 1, None);

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

        let routes = graph.simple_paths(&me, &dest, 2, None);
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

        let routes = graph.simple_paths(&me, &dest, 3, None);
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

        let routes = graph.simple_paths(&me, &dest, 3, None);
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

        let routes = graph.simple_paths(&me, &dest, 2, None);
        assert!(routes.is_empty(), "no 2-hop route should exist for a direct edge");
        Ok(())
    }

    #[test]
    fn zero_hop_should_always_return_empty() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let other = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);

        let routes = graph.simple_paths(&me, &other, 0, None);
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

        let routes = graph.simple_paths(&me, &dest, 2, None);
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
        let routes_3 = graph.simple_paths(&me, &f, 3, None);
        assert_eq!(routes_3.len(), 5, "should find exactly 5 three-hop paths");

        // Verify all returned paths have positive cost
        for (path, cost) in &routes_3 {
            assert!(*cost > 0.0, "path {path:?} should have positive cost, got {cost}");
            assert_eq!(path.len(), 4, "3-hop path should contain 4 nodes");
            assert_eq!(path.first(), Some(&me), "path should start at me");
            assert_eq!(path.last(), Some(&f), "path should end at f");
        }

        // --- 2-hop paths: a→f and b→f are NOT connected, so cost is pruned ---
        let routes_2 = graph.simple_paths(&me, &f, 2, None);
        assert!(
            routes_2.is_empty(),
            "2-hop paths should be pruned (last edge not connected)"
        );

        // --- 1-hop path: no direct me→f edge ---
        let routes_1 = graph.simple_paths(&me, &f, 1, None);
        assert!(routes_1.is_empty(), "no direct edge from me to f");

        Ok(())
    }

    #[tokio::test]
    async fn stream_should_generate_an_endless_stream_of_paths() -> anyhow::Result<()> {
        // Simple 1-hop graph: me → dest
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;
        mark_edge_connected(&graph, &me, &dest);

        let stream = graph.simple_paths_stream(&me, &dest, 1);

        // Take 5 items — this spans at least 2 full batches,
        // proving the stream loops endlessly rather than stopping after one batch.
        let paths: Vec<_> = tokio::time::timeout(std::time::Duration::from_secs(5), stream.take(5).collect::<Vec<_>>())
            .await
            .context("stream should not stall — timed out after 5s")?;
        assert_eq!(paths.len(), 5, "stream should endlessly generate paths");

        for (path, cost) in &paths {
            assert_eq!(path, &vec![me, dest], "each path should be the single direct route");
            assert!(*cost > 0.0, "cost should be positive");
        }

        Ok(())
    }

    #[tokio::test]
    async fn stream_should_terminate_after_edge_removal() -> anyhow::Result<()> {
        // Simple 1-hop graph: me → dest
        let me = pubkey_from(&SECRET_0);
        let dest = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest)?;
        mark_edge_connected(&graph, &me, &dest);

        let mut stream = graph.simple_paths_stream(&me, &dest, 1);

        // Consume at least 2 full rounds (2*1) to prove the stream was producing results before the edge removal.
        let before: Vec<_> = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            stream.by_ref().take(2).collect::<Vec<_>>(),
        )
        .await
        .context("stream should not stall — timed out after 5s")?;
        assert_eq!(before.len(), 2, "stream should yield paths before edge removal");

        // Remove the edge — the shared Arc<RwLock<InnerGraph>> makes this
        // visible to the stream on the next iteration.
        graph.remove_edge(&me, &dest);

        // The stream should now terminate: collect all remaining items.
        let remaining: Vec<_> = tokio::time::timeout(std::time::Duration::from_secs(5), stream.collect::<Vec<_>>())
            .await
            .context("stream should have terminated — timed out after 5s")?;
        assert_eq!(
            remaining.len(),
            0,
            "stream should terminate after edge removal (got {} remaining items)",
            remaining.len()
        );

        Ok(())
    }
}
