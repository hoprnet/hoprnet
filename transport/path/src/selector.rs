use hopr_api::graph::{NetworkGraphTraverse, NetworkGraphView, costs::HoprCostFn, traits::EdgeObservableRead};
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::errors::PathError;
use tracing::trace;

use crate::{
    errors::{PathPlannerError, Result},
    traits::PathSelector,
};

type PathToDestination = Vec<OffchainPublicKey>;

/// Compute candidate paths from `src` to `dest` through `graph`.
///
/// `length` is the number of edges to traverse (= intermediate hops + 1).
/// `take` caps the number of candidate paths returned.
/// The source node is stripped from every returned path; callers receive
/// `[intermediates..., dest]`.
fn compute_paths<G>(
    graph: &G,
    src: &OffchainPublicKey,
    dest: &OffchainPublicKey,
    length: std::num::NonZeroUsize,
    take: usize,
) -> Vec<PathToDestination>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey> + NetworkGraphView<NodeId = OffchainPublicKey>,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
{
    let raw = graph.simple_paths(src, dest, length.get(), Some(take), HoprCostFn::new(length));

    raw.into_iter()
        .filter_map(|(path, _, cost)| {
            if cost > 0.0 {
                // Drop the first element (src) — callers expect [intermediates..., dest].
                Some(path.into_iter().skip(1).collect::<Vec<_>>())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

/// A lightweight graph-backed path selector.
///
/// Returns all candidate paths for a `(src, dest, hops)` query directly from
/// the network graph — no caching is performed here.  The caller (typically
/// [`crate::planner::PathPlanner`]) is responsible for caching, TTL management,
/// background refresh, and final path selection.
#[derive(Clone)]
pub struct HoprGraphPathSelector<G> {
    graph: G,
    max_paths: usize,
}

impl<G> HoprGraphPathSelector<G>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + NetworkGraphView<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
{
    /// Create a new selector.
    ///
    /// * `graph` – the network graph to query.
    /// * `max_paths` – maximum number of candidate paths to return per query.
    pub fn new(graph: G, max_paths: usize) -> Self {
        Self { graph, max_paths }
    }
}

impl<G> PathSelector for HoprGraphPathSelector<G>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + NetworkGraphView<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
{
    /// Return all candidate paths from `src` to `dest` via `hops` relays.
    ///
    /// Each returned `Vec<OffchainPublicKey>` has length `hops + 1` and contains
    /// `[intermediates..., dest]`; `src` is excluded.
    ///
    /// Returns `Err(PathNotFound)` when the graph yields no positive-cost paths.
    ///
    /// The function has a potential to run expensive operations, it should be benchmarked
    /// in a production environment and possibly guarded (e.g. by offloading the long execution
    /// in an async executor to avoid blocking the caller).
    fn select_path(
        &self,
        src: OffchainPublicKey,
        dest: OffchainPublicKey,
        hops: usize,
    ) -> Result<Vec<Vec<OffchainPublicKey>>> {
        trace!(%src, %dest, hops, "computing paths from graph");
        let paths = compute_paths(
            &self.graph,
            &src,
            &dest,
            std::num::NonZeroUsize::new(hops + 1)
                .expect("can never fail, it is physically at least 1 after the addition"),
            self.max_paths,
        );

        if paths.is_empty() {
            Err(PathPlannerError::Path(PathError::PathNotFound(
                hops,
                src.to_peerid_str(),
                dest.to_peerid_str(),
            )))
        } else {
            Ok(paths)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Context;
    use hex_literal::hex;
    use hopr_api::graph::{
        NetworkGraphWrite,
        traits::{EdgeObservableWrite, EdgeWeightType},
    };
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
    use hopr_network_graph::ChannelGraph;
    use hopr_network_types::types::RoutingOptions;

    use super::*;
    use crate::traits::PathSelector;

    const SECRET_0: [u8; 32] = hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");
    const SECRET_1: [u8; 32] = hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a");
    const SECRET_2: [u8; 32] = hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299");
    const SECRET_3: [u8; 32] = hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e");
    const SECRET_4: [u8; 32] = hex!("cfc66f718ec66fb822391775d749d7a0d66b690927673634816b63339bc12a3c");

    const MAX_PATHS: usize = 4;

    fn pubkey(secret: &[u8; 32]) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(secret).expect("valid secret").public()
    }

    /// Mark an edge as fully ready for intermediate routing.
    fn mark_edge_full(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });
    }

    /// Mark an edge as ready only as the last hop.
    fn mark_edge_last(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
        });
    }

    // Helper: build a bidirectional 2-hop graph: me ↔ hop ↔ dest.
    fn two_hop_graph() -> (OffchainPublicKey, OffchainPublicKey, OffchainPublicKey, ChannelGraph) {
        let me = pubkey(&SECRET_0);
        let hop = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);
        // Forward: me → hop → dest
        graph.add_edge(&me, &hop).unwrap();
        graph.add_edge(&hop, &dest).unwrap();
        mark_edge_full(&graph, &me, &hop);
        mark_edge_last(&graph, &hop, &dest);
        // Reverse: dest → hop → me
        graph.add_edge(&dest, &hop).unwrap();
        graph.add_edge(&hop, &me).unwrap();
        mark_edge_full(&graph, &dest, &hop);
        mark_edge_last(&graph, &hop, &me);
        (me, hop, dest, graph)
    }

    #[tokio::test]
    async fn unreachable_dest_should_return_error() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let unreachable = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        // No edges at all — neither direction has a path.
        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);

        let fwd = selector.select_path(me, unreachable, 1);
        assert!(fwd.is_err(), "forward: should error when destination is unreachable");
        assert!(matches!(
            fwd.unwrap_err(),
            PathPlannerError::Path(PathError::PathNotFound(..))
        ));

        let rev = selector.select_path(unreachable, me, 1);
        assert!(rev.is_err(), "reverse: should error when destination is unreachable");
        assert!(matches!(
            rev.unwrap_err(),
            PathPlannerError::Path(PathError::PathNotFound(..))
        ));

        Ok(())
    }

    #[tokio::test]
    async fn path_should_exclude_source() -> anyhow::Result<()> {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 1).context("forward path")?;
        assert!(!fwd.is_empty());
        for path in &fwd {
            assert!(!path.contains(&me), "forward path must not contain the source");
        }

        let rev = selector.select_path(dest, me, 1).context("reverse path")?;
        assert!(!rev.is_empty());
        for path in &rev {
            assert!(!path.contains(&dest), "reverse path must not contain the source");
        }

        Ok(())
    }

    #[tokio::test]
    async fn multi_hop_path_should_have_correct_length() -> anyhow::Result<()> {
        // Bidirectional: me ↔ A ↔ B ↔ dest  (2 intermediate hops each way)
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let dest = pubkey(&SECRET_3);
        let graph = ChannelGraph::new(me);
        for n in [a, b, dest] {
            graph.add_node(n);
        }
        // Forward: me → A → B → dest
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &b).unwrap();
        graph.add_edge(&b, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &a, &b);
        mark_edge_last(&graph, &b, &dest);
        // Reverse: dest → B → A → me
        graph.add_edge(&dest, &b).unwrap();
        graph.add_edge(&b, &a).unwrap();
        graph.add_edge(&a, &me).unwrap();
        mark_edge_full(&graph, &dest, &b);
        mark_edge_full(&graph, &b, &a);
        mark_edge_last(&graph, &a, &me);

        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 2).context("forward 2-hop path")?;
        assert!(!fwd.is_empty());
        for path in &fwd {
            assert_eq!(path.len(), 3, "forward 2-hop path: [A, B, dest]");
            assert_eq!(path.last(), Some(&dest));
        }

        let rev = selector.select_path(dest, me, 2).context("reverse 2-hop path")?;
        assert!(!rev.is_empty());
        for path in &rev {
            assert_eq!(path.len(), 3, "reverse 2-hop path: [B, A, me]");
            assert_eq!(path.last(), Some(&me));
        }

        Ok(())
    }

    #[tokio::test]
    async fn one_hop_path_should_include_relay_and_destination() -> anyhow::Result<()> {
        // Bidirectional: me ↔ relay ↔ dest
        let (me, relay, dest, graph) = two_hop_graph();
        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 1).context("forward 1-hop path")?;
        assert!(!fwd.is_empty());
        for path in &fwd {
            assert_eq!(path.len(), 2, "forward: [relay, dest]");
            assert_eq!(path.last(), Some(&dest));
            assert!(!path.contains(&me));
        }

        let rev = selector.select_path(dest, me, 1).context("reverse 1-hop path")?;
        assert!(!rev.is_empty());
        for path in &rev {
            assert_eq!(path.len(), 2, "reverse: [relay, me]");
            assert_eq!(path.last(), Some(&me));
            assert!(!path.contains(&dest));
        }

        let _ = relay;
        Ok(())
    }

    #[tokio::test]
    async fn diamond_topology_should_return_multiple_paths() -> anyhow::Result<()> {
        // Bidirectional diamond: me ↔ a ↔ dest and me ↔ b ↔ dest
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let dest = pubkey(&SECRET_3);
        let graph = ChannelGraph::new(me);
        for n in [a, b, dest] {
            graph.add_node(n);
        }
        // Forward: me → a → dest,  me → b → dest
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&me, &b).unwrap();
        graph.add_edge(&a, &dest).unwrap();
        graph.add_edge(&b, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &me, &b);
        mark_edge_last(&graph, &a, &dest);
        mark_edge_last(&graph, &b, &dest);
        // Reverse: dest → a → me,  dest → b → me
        graph.add_edge(&dest, &a).unwrap();
        graph.add_edge(&dest, &b).unwrap();
        graph.add_edge(&a, &me).unwrap();
        graph.add_edge(&b, &me).unwrap();
        mark_edge_full(&graph, &dest, &a);
        mark_edge_full(&graph, &dest, &b);
        mark_edge_last(&graph, &a, &me);
        mark_edge_last(&graph, &b, &me);

        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 1).context("forward path")?;
        assert_eq!(fwd.len(), 2, "forward: both paths via a and b should be returned");
        for path in &fwd {
            assert_eq!(path.last(), Some(&dest));
        }

        let rev = selector.select_path(dest, me, 1).context("reverse path")?;
        assert_eq!(rev.len(), 2, "reverse: both paths via a and b should be returned");
        for path in &rev {
            assert_eq!(path.last(), Some(&me));
        }

        Ok(())
    }

    #[tokio::test]
    async fn zero_cost_paths_should_return_error() -> anyhow::Result<()> {
        // Graph has edges in both directions but no observations → all costs zero → pruned.
        let me = pubkey(&SECRET_0);
        let dest = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest).unwrap();
        graph.add_edge(&dest, &me).unwrap();
        // No observations → cost function will return non-positive cost → pruned.

        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);
        assert!(
            selector.select_path(me, dest, 1).is_err(),
            "forward: edge with no observations should produce no valid path"
        );
        assert!(
            selector.select_path(dest, me, 1).is_err(),
            "reverse: edge with no observations should produce no valid path"
        );
        Ok(())
    }

    #[tokio::test]
    async fn no_path_at_requested_hop_count_should_return_error() -> anyhow::Result<()> {
        // Graph has direct edges both ways; requesting 2 hops should fail in both directions.
        let me = pubkey(&SECRET_0);
        let dest = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest).unwrap();
        graph.add_edge(&dest, &me).unwrap();
        mark_edge_full(&graph, &me, &dest);
        mark_edge_full(&graph, &dest, &me);

        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);
        assert!(
            selector.select_path(me, dest, 2).is_err(),
            "forward: no 2-hop path should exist for a direct edge"
        );
        assert!(
            selector.select_path(dest, me, 2).is_err(),
            "reverse: no 2-hop path should exist for a direct edge"
        );
        Ok(())
    }

    #[tokio::test]
    async fn five_node_chain_should_support_max_hops() -> anyhow::Result<()> {
        // Bidirectional: me ↔ a ↔ b ↔ c ↔ dest  (3 intermediate hops each way)
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let c = pubkey(&SECRET_3);
        let dest = pubkey(&SECRET_4);
        let graph = ChannelGraph::new(me);
        for n in [a, b, c, dest] {
            graph.add_node(n);
        }
        // Forward: me → a → b → c → dest
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &b).unwrap();
        graph.add_edge(&b, &c).unwrap();
        graph.add_edge(&c, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &a, &b);
        mark_edge_full(&graph, &b, &c);
        mark_edge_last(&graph, &c, &dest);
        // Reverse: dest → c → b → a → me
        graph.add_edge(&dest, &c).unwrap();
        graph.add_edge(&c, &b).unwrap();
        graph.add_edge(&b, &a).unwrap();
        graph.add_edge(&a, &me).unwrap();
        mark_edge_full(&graph, &dest, &c);
        mark_edge_full(&graph, &c, &b);
        mark_edge_full(&graph, &b, &a);
        mark_edge_last(&graph, &a, &me);

        let selector = HoprGraphPathSelector::new(graph, MAX_PATHS);

        let fwd = selector
            .select_path(me, dest, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .context("forward 3-hop path")?;
        assert!(!fwd.is_empty());
        for path in &fwd {
            assert_eq!(path.len(), 4, "forward: [a, b, c, dest]");
            assert_eq!(path.last(), Some(&dest));
            assert!(!path.contains(&me));
        }

        let rev = selector
            .select_path(dest, me, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .context("reverse 3-hop path")?;
        assert!(!rev.is_empty());
        for path in &rev {
            assert_eq!(path.len(), 4, "reverse: [c, b, a, me]");
            assert_eq!(path.last(), Some(&me));
            assert!(!path.contains(&dest));
        }

        Ok(())
    }
}
