use hopr_api::graph::{
    NetworkGraphTraverse, NetworkGraphView, ValueFn, function::EdgeValueFn, traits::EdgeObservableRead,
};
use hopr_types::{crypto::types::OffchainPublicKey, internal::errors::PathError};

use super::{
    errors::{PathPlannerError, Result},
    traits::{PathSelector, PathWithCost},
};

/// Compute candidate paths from `src` to `dest` through `graph`.
///
/// `length` is the number of edges to traverse (= intermediate hops + 1).
/// `take` caps the number of candidate paths returned.
/// The source node is stripped from every returned path; callers receive
/// `([intermediates..., dest], cost)`.
fn compute_paths<G, C>(
    graph: &G,
    src: &OffchainPublicKey,
    dest: &OffchainPublicKey,
    length: std::num::NonZeroUsize,
    take: usize,
    cost_fn: C,
) -> Vec<PathWithCost>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey> + NetworkGraphView<NodeId = OffchainPublicKey>,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
    C: ValueFn<Weight = <G as NetworkGraphTraverse>::Observed, Value = f64>,
{
    let raw = graph.simple_paths(src, dest, length.get(), Some(take), cost_fn);

    raw.into_iter()
        .filter_map(|(path, _, cost)| {
            tracing::trace!(?path, ?cost, "evaluating candidate path");
            if cost > 0.0 {
                // Drop the first element (src) — callers expect [intermediates..., dest].
                Some(PathWithCost {
                    path: path.into_iter().skip(1).collect::<Vec<_>>(),
                    cost,
                })
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
/// [`crate::path::planner::PathPlanner`]) is responsible for caching, TTL management,
/// background refresh, and final path selection.
///
/// Stores the planner's own identity (`me`) so that it can choose the
/// appropriate cost function:
/// - forward path (`src == me`): [`EdgeValueFn::forward`]
/// - return path (`dest == me`): [`EdgeValueFn::returning`]
#[derive(Clone)]
pub struct HoprGraphPathSelector<G> {
    me: OffchainPublicKey,
    graph: G,
    max_paths: usize,
    edge_penalty: f64,
    min_ack_rate: f64,
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
    /// * `me` – the planner's own offchain public key, used to determine path direction.
    /// * `graph` – the network graph to query.
    /// * `max_paths` – maximum number of candidate paths to return per query.
    /// * `edge_penalty` – penalty multiplier for edges lacking probe-based quality observations.
    /// * `min_ack_rate` – minimum acceptable message acknowledgment rate for path selection.
    pub fn new(me: OffchainPublicKey, graph: G, max_paths: usize, edge_penalty: f64, min_ack_rate: f64) -> Self {
        Self {
            me,
            graph,
            max_paths,
            edge_penalty,
            min_ack_rate,
        }
    }

    /// Extended forward path search: find shorter paths using
    /// [`EdgeValueFn::forward_without_self_loopback`] and append `dest` to each one.
    ///
    /// This handles the case where the last edge (relay -> dest) has no graph edge
    /// (e.g. no payment channel) but the path planner can still assume the last hop
    /// is reachable. Paths already found by Phase 1 are excluded via `existing`.
    ///
    /// The cost from the shorter traversal is preserved as-is — the missing last
    /// edge contributes a neutral `1.0` multiplier (no quality data available).
    fn compute_extended_forward_paths(
        &self,
        src: &OffchainPublicKey,
        dest: &OffchainPublicKey,
        shorter_length: std::num::NonZeroUsize,
        take: usize,
        existing: &[PathWithCost],
    ) -> Vec<PathWithCost> {
        let raw = self.graph.simple_paths_from(
            src,
            shorter_length.get(),
            Some(take),
            EdgeValueFn::forward_without_self_loopback(self.edge_penalty, self.min_ack_rate),
        );

        raw.into_iter()
            .filter_map(|(path, _, cost)| {
                if cost <= 0.0 {
                    return None;
                }

                // Strip source to get intermediate hops only.
                // Guard: if dest already appears as an intermediate, appending it
                // again creates a [dest, ..., dest] path that ValidatedPath::new
                // would reject as a non-adjacent duplicate — skip early.
                let candidate: Vec<_> = path.into_iter().skip(1).collect();
                if candidate.contains(dest) {
                    return None;
                }
                let mut candidate = candidate;
                candidate.push(*dest);

                // Skip paths already found by Phase 1 (compare path component only).
                if existing.iter().any(|pwc| pwc.path == candidate) {
                    return None;
                }

                tracing::trace!(?candidate, ?cost, "extended forward path candidate");
                Some(PathWithCost { path: candidate, cost })
            })
            .take(take)
            .collect()
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
    /// Each returned tuple contains a path `Vec<OffchainPublicKey>` of length
    /// `hops + 1` (`[intermediates..., dest]`; `src` excluded) paired with its
    /// accumulated traversal cost.
    ///
    /// Returns `Err(PathNotFound)` when the graph yields no positive-cost paths.
    ///
    /// The function has a potential to run expensive operations, it should be benchmarked
    /// in a production environment and possibly guarded (e.g. by offloading the long execution
    /// in an async executor to avoid blocking the caller).
    #[tracing::instrument(level = "trace", skip(self), fields(src = %src, dest = %dest, hops), ret, err)]
    fn select_path(&self, src: OffchainPublicKey, dest: OffchainPublicKey, hops: usize) -> Result<Vec<PathWithCost>> {
        let direction = if src == self.me { "forward" } else { "return" };
        tracing::debug!(%src, %dest, hops, direction, "computing paths from graph");

        let length = std::num::NonZeroUsize::new(hops + 1)
            .expect("can never fail, it is physically at least 1 after the addition");

        let paths = if src == self.me {
            // Phase 1: search for full-length forward paths to dest.
            let mut found = compute_paths(
                &self.graph,
                &src,
                &dest,
                length,
                self.max_paths,
                EdgeValueFn::forward(length, self.edge_penalty, self.min_ack_rate),
            );
            tracing::debug!(
                direction,
                phase = 1,
                count = found.len(),
                "[forward] phase 1 candidates"
            );

            // Phase 2: if not enough paths, do an extended search with EdgeValueFn::forward_without_self_loopback
            // for (length - 1) edges and assume the last hop can be done by anybody.
            if found.len() < self.max_paths
                && let Some(shorter) = std::num::NonZeroUsize::new(length.get() - 1)
            {
                let remaining = self.max_paths - found.len();
                let extended = self.compute_extended_forward_paths(&src, &dest, shorter, remaining, &found);
                tracing::debug!(
                    direction,
                    phase = 2,
                    count = extended.len(),
                    "[forward] phase 2 extended candidates"
                );
                found.extend(extended);
            }

            found
        } else {
            let found = compute_paths(
                &self.graph,
                &src,
                &dest,
                length,
                self.max_paths,
                EdgeValueFn::returning(length, self.edge_penalty, self.min_ack_rate),
            );
            tracing::debug!(direction, count = found.len(), "[return] candidates");
            found
        };

        for (i, pwc) in paths.iter().enumerate() {
            tracing::debug!(
                direction,
                index = i,
                path = ?pwc.path,
                cost = pwc.cost,
                "[{direction}] candidate path"
            );
        }

        if paths.is_empty() {
            Err(PathPlannerError::Path(PathError::PathNotFound(
                hops,
                src.to_string(),
                dest.to_string(),
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
    use hopr_network_graph::ChannelGraph;
    use hopr_types::{
        crypto::prelude::{Keypair, OffchainKeypair},
        internal::routing::RoutingOptions,
    };

    use super::*;
    use crate::path::{PathPlannerConfig, traits::PathSelector};

    fn test_selector(
        me: OffchainPublicKey,
        graph: ChannelGraph,
        max_paths: usize,
    ) -> HoprGraphPathSelector<ChannelGraph> {
        let cfg = PathPlannerConfig::default();
        HoprGraphPathSelector::new(me, graph, max_paths, cfg.edge_penalty, cfg.min_ack_rate)
    }

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

    /// Mark an edge as ready only as the last hop (forward or return).
    ///
    /// Forward last edge requires capacity; return last edge requires connectivity + score.
    /// This helper sets all of them so it works for either direction.
    fn mark_edge_last(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
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
        let selector = test_selector(me, graph, MAX_PATHS);

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
        let selector = test_selector(me, graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 1).context("forward path")?;
        assert!(!fwd.is_empty());
        for pwc in &fwd {
            assert!(!pwc.path.contains(&me), "forward path must not contain the source");
            assert!(pwc.cost > 0.0, "cost must be positive");
        }

        let rev = selector.select_path(dest, me, 1).context("reverse path")?;
        assert!(!rev.is_empty());
        for pwc in &rev {
            assert!(!pwc.path.contains(&dest), "reverse path must not contain the source");
            assert!(pwc.cost > 0.0, "cost must be positive");
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

        let selector = test_selector(me, graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 2).context("forward 2-hop path")?;
        assert!(!fwd.is_empty());
        for pwc in &fwd {
            assert_eq!(pwc.path.len(), 3, "forward 2-hop path: [A, B, dest]");
            assert_eq!(pwc.path.last(), Some(&dest));
        }

        let rev = selector.select_path(dest, me, 2).context("reverse 2-hop path")?;
        assert!(!rev.is_empty());
        for pwc in &rev {
            assert_eq!(pwc.path.len(), 3, "reverse 2-hop path: [B, A, me]");
            assert_eq!(pwc.path.last(), Some(&me));
        }

        Ok(())
    }

    #[tokio::test]
    async fn one_hop_path_should_include_relay_and_destination() -> anyhow::Result<()> {
        // Bidirectional: me ↔ relay ↔ dest
        let (me, relay, dest, graph) = two_hop_graph();
        let selector = test_selector(me, graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 1).context("forward 1-hop path")?;
        assert!(!fwd.is_empty());
        for pwc in &fwd {
            assert_eq!(pwc.path.len(), 2, "forward: [relay, dest]");
            assert_eq!(pwc.path.last(), Some(&dest));
            assert!(!pwc.path.contains(&me));
        }

        let rev = selector.select_path(dest, me, 1).context("reverse 1-hop path")?;
        assert!(!rev.is_empty());
        for pwc in &rev {
            assert_eq!(pwc.path.len(), 2, "reverse: [relay, me]");
            assert_eq!(pwc.path.last(), Some(&me));
            assert!(!pwc.path.contains(&dest));
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

        let selector = test_selector(me, graph, MAX_PATHS);

        let fwd = selector.select_path(me, dest, 1).context("forward path")?;
        assert_eq!(fwd.len(), 2, "forward: both paths via a and b should be returned");
        for pwc in &fwd {
            assert_eq!(pwc.path.last(), Some(&dest));
        }

        let rev = selector.select_path(dest, me, 1).context("reverse path")?;
        assert_eq!(rev.len(), 2, "reverse: both paths via a and b should be returned");
        for pwc in &rev {
            assert_eq!(pwc.path.last(), Some(&me));
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

        let selector = test_selector(me, graph, MAX_PATHS);
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

        let selector = test_selector(me, graph, MAX_PATHS);
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
    async fn forward_path_should_work_without_last_edge() -> anyhow::Result<()> {
        // In the real network, the last edge (relay → dest) may not have a
        // payment channel. The graph has me → relay but NO relay → dest edge.
        // The virtual last hop fallback should still find the forward path.
        let me = pubkey(&SECRET_0);
        let relay = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(relay);
        graph.add_node(dest);
        // Forward: me → relay (fully observed). NO relay → dest edge.
        graph.add_edge(&me, &relay).unwrap();
        mark_edge_full(&graph, &me, &relay);
        // Reverse: dest → relay → me (both edges exist for the return path).
        graph.add_edge(&dest, &relay).unwrap();
        graph.add_edge(&relay, &me).unwrap();
        mark_edge_full(&graph, &dest, &relay);
        mark_edge_last(&graph, &relay, &me);

        let selector = test_selector(me, graph, MAX_PATHS);

        // Forward path should work via virtual last hop
        let fwd = selector
            .select_path(me, dest, 1)
            .context("forward path with virtual last hop")?;
        assert!(!fwd.is_empty(), "forward path should find at least one route");
        for pwc in &fwd {
            assert_eq!(pwc.path.len(), 2, "forward: [relay, dest]");
            assert_eq!(pwc.path[0], relay);
            assert_eq!(pwc.path[1], dest);
        }

        // Return path should work normally (both edges exist)
        let rev = selector.select_path(dest, me, 1).context("return path")?;
        assert!(!rev.is_empty(), "return path should find at least one route");
        for pwc in &rev {
            assert_eq!(pwc.path.len(), 2, "return: [relay, me]");
            assert_eq!(pwc.path.last(), Some(&me));
        }

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

        let selector = test_selector(me, graph, MAX_PATHS);

        let fwd = selector
            .select_path(me, dest, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .context("forward 3-hop path")?;
        assert!(!fwd.is_empty());
        for pwc in &fwd {
            assert_eq!(pwc.path.len(), 4, "forward: [a, b, c, dest]");
            assert_eq!(pwc.path.last(), Some(&dest));
            assert!(!pwc.path.contains(&me));
        }

        let rev = selector
            .select_path(dest, me, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .context("reverse 3-hop path")?;
        assert!(!rev.is_empty());
        for pwc in &rev {
            assert_eq!(pwc.path.len(), 4, "reverse: [c, b, a, me]");
            assert_eq!(pwc.path.last(), Some(&me));
            assert!(!pwc.path.contains(&dest));
        }

        Ok(())
    }

    #[tokio::test]
    async fn selector_should_reject_extended_path_containing_destination() -> anyhow::Result<()> {
        // If me has a direct edge to dest (e.g. an edge node with a channel to its
        // exit server), Phase 2 must not emit [dest, dest] — appending dest to a
        // candidate that already ends in dest forms a loop that ValidatedPath::new
        // would catch anyway, but we want the guard to skip it cleanly.
        let me = pubkey(&SECRET_0);
        let relay = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(relay);
        graph.add_node(dest);
        // me → dest: direct channel (this is what the own_chain_addr fix exposes)
        graph.add_edge(&me, &dest).unwrap();
        mark_edge_full(&graph, &me, &dest);
        // me → relay: for 1-hop via relay
        graph.add_edge(&me, &relay).unwrap();
        mark_edge_full(&graph, &me, &relay);
        // Return path
        graph.add_edge(&dest, &relay).unwrap();
        graph.add_edge(&relay, &me).unwrap();
        mark_edge_full(&graph, &dest, &relay);
        mark_edge_last(&graph, &relay, &me);

        let selector = test_selector(me, graph, MAX_PATHS);

        let fwd = selector
            .select_path(me, dest, 1)
            .context("forward path with dest as direct neighbor")?;
        assert!(!fwd.is_empty(), "should find at least one path via relay");
        for pwc in &fwd {
            assert_eq!(pwc.path.len(), 2, "path must be [relay, dest]");
            assert_eq!(pwc.path[0], relay, "first node must be relay, not dest");
            assert_eq!(pwc.path[1], dest);
        }
        Ok(())
    }

    #[tokio::test]
    async fn selector_should_reject_one_hop_path_where_relay_equals_destination() -> anyhow::Result<()> {
        // Same as above but without relay→dest edge — only Phase 2 (virtual last hop)
        // is in play. The guard must skip the me→dest edge as a relay candidate.
        let me = pubkey(&SECRET_0);
        let relay = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(relay);
        graph.add_node(dest);
        // me → dest: direct channel (no relay→dest edge)
        graph.add_edge(&me, &dest).unwrap();
        mark_edge_full(&graph, &me, &dest);
        // me → relay (relay is a valid intermediate; no relay→dest edge needed for Phase 2)
        graph.add_edge(&me, &relay).unwrap();
        mark_edge_full(&graph, &me, &relay);
        // Return path
        graph.add_edge(&dest, &relay).unwrap();
        graph.add_edge(&relay, &me).unwrap();
        mark_edge_full(&graph, &dest, &relay);
        mark_edge_last(&graph, &relay, &me);

        let selector = test_selector(me, graph, MAX_PATHS);

        let fwd = selector
            .select_path(me, dest, 1)
            .context("forward path — dest is direct neighbor, relay is intermediate")?;
        assert!(!fwd.is_empty(), "should find path via relay (virtual last hop)");
        for pwc in &fwd {
            assert_eq!(pwc.path[0], relay, "intermediate must be relay, not dest");
            assert_ne!(pwc.path[0], dest, "dest must not appear as intermediate");
        }
        Ok(())
    }

    #[tokio::test]
    async fn selector_should_skip_zero_cost_paths() -> anyhow::Result<()> {
        // Build graph with edges but NO observations → cost function returns 0.
        let me = pubkey(&SECRET_0);
        let hop = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);
        graph.add_edge(&me, &hop).context("adding edge me -> hop")?;
        graph.add_edge(&hop, &dest).context("adding edge hop -> dest")?;
        // No mark_edge_full/mark_edge_last → observations are empty → cost = 0

        let selector = test_selector(me, graph, MAX_PATHS);

        let err = selector
            .select_path(me, dest, 1)
            .expect_err("zero-cost paths should be filtered out");
        anyhow::ensure!(
            matches!(err, PathPlannerError::Path(PathError::PathNotFound(..))),
            "expected PathNotFound, got: {err}"
        );
        Ok(())
    }
}
