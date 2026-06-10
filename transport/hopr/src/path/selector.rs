use hopr_api::{
    OffchainPublicKey,
    graph::{
        NetworkGraphTraverse, NetworkGraphView, ValueFn,
        function::EdgeValueFn,
        traits::{EdgeImmediateProtocolObservable, EdgeLinkObservable, EdgeObservableRead, EdgeProtocolObservable},
    },
    types::internal::errors::PathError,
};

use super::{
    errors::{PathPlannerError, Result},
    traits::{PathSelector, PathWithMetrics},
};

/// Walk every edge in `[src → path[0] → path[1] → … → path[last]]` and aggregate
/// per-edge quality data into path-level optional metrics.
fn aggregate_metrics<G>(graph: &G, src: &OffchainPublicKey, path: &[OffchainPublicKey], cost: f64) -> PathWithMetrics
where
    G: NetworkGraphView<NodeId = OffchainPublicKey>,
{
    let mut total_latency_ms: Option<u32> = Some(0);
    let mut min_probe_rate: Option<f64> = None;
    let mut min_ack_rate: Option<f64> = None;
    let mut capacity_floor: Option<u128> = None;

    let nodes: Vec<OffchainPublicKey> = std::iter::once(*src).chain(path.iter().copied()).collect();
    for edge_pair in nodes.windows(2) {
        let a = &edge_pair[0];
        let b = &edge_pair[1];

        let obs_opt = graph.edge(a, b);

        // Latency: propagate None if any edge is unprobed.
        let edge_lat = obs_opt.as_ref().and_then(|obs| {
            obs.immediate_qos()
                .and_then(|m| m.average_latency())
                .or_else(|| obs.intermediate_qos().and_then(|m| m.average_latency()))
        });
        total_latency_ms = match (total_latency_ms, edge_lat) {
            (Some(acc), Some(lat)) => Some(acc.saturating_add(lat.as_millis() as u32)),
            _ => None,
        };

        if let Some(obs) = obs_opt.as_ref() {
            // Probe success rate: minimum over edges that have data.
            if let Some(imm) = obs.immediate_qos() {
                let r = imm.average_probe_rate();
                min_probe_rate = Some(min_probe_rate.map_or(r, |prev: f64| prev.min(r)));
            }
            // Ack rate: minimum over edges that have sent messages.
            if let Some(imm) = obs.immediate_qos()
                && let Some(rate) = imm.ack_rate()
            {
                min_ack_rate = Some(min_ack_rate.map_or(rate, |prev: f64| prev.min(rate)));
            }
            // Capacity: minimum over edges that carry channel-capacity data.
            if let Some(inter) = obs.intermediate_qos()
                && let Some(cap) = inter.capacity()
            {
                capacity_floor = Some(capacity_floor.map_or(cap, |prev: u128| prev.min(cap)));
            }
        }
    }

    PathWithMetrics {
        path: path.to_vec(),
        cost,
        total_latency_ms,
        min_probe_success_rate: min_probe_rate,
        min_ack_rate,
        capacity_floor,
    }
}

/// Trim the candidate set to lower median latency and minimise variance while
/// preserving an anonymity floor.
///
/// Behaviour:
/// - If `candidates.len() <= floor`, returns all candidates unchanged (`min(found_count, floor)` semantics — the floor
///   is never a minimum to fabricate).
/// - Sorts candidates with a known `total_latency_ms` ascending.
/// - Drops from the high-latency tail until the total count equals `floor`, or until no populated candidates remain.
/// - If still over the floor with no populated candidates left, drops unpopulated candidates from the input-order tail.
pub fn prune_for_consistency(candidates: Vec<PathWithMetrics>, floor: usize) -> Vec<PathWithMetrics> {
    if candidates.len() <= floor {
        return candidates;
    }

    let (mut populated, unpopulated): (Vec<_>, Vec<_>) =
        candidates.into_iter().partition(|p| p.total_latency_ms.is_some());

    // Sort populated ascending by latency (lowest first → drop from the end).
    populated.sort_by_key(|p| p.total_latency_ms.unwrap_or(u32::MAX));

    let total_unpopulated = unpopulated.len();
    let target_populated = floor.saturating_sub(total_unpopulated);
    populated.truncate(populated.len().max(target_populated).min(populated.len()));
    // Clamp: keep at most `target_populated` populated paths.
    if populated.len() > target_populated {
        populated.truncate(target_populated);
    }

    let mut result = populated;
    result.extend(unpopulated);

    // If we're still over the floor (all populated were dropped and unpopulated > floor),
    // trim the unpopulated tail.
    if result.len() > floor {
        result.truncate(floor);
    }

    result
}

/// Compute candidate paths from `src` to `dest` through `graph`.
///
/// `length` is the number of edges to traverse (= intermediate hops + 1).
/// `take` caps the number of candidate paths returned.
/// The graph crate returns only the intermediate nodes (both `src` and `dest` stripped);
/// this function re-appends `dest` so callers receive `([intermediates..., dest], cost)`.
fn compute_paths<G, C>(
    graph: &G,
    src: &OffchainPublicKey,
    dest: &OffchainPublicKey,
    length: std::num::NonZeroUsize,
    take: usize,
    cost_fn: C,
) -> Vec<PathWithMetrics>
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
                // The graph crate returns only intermediates (src and dest already stripped).
                // Re-append dest so callers receive [intermediates..., dest].
                let mut path = path;
                path.push(*dest);
                Some(aggregate_metrics(graph, src, &path, cost))
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
    anonymity_floor: usize,
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
    /// * `anonymity_floor` – minimum candidate count below which no latency-based pruning occurs.
    pub fn new(
        me: OffchainPublicKey,
        graph: G,
        max_paths: usize,
        edge_penalty: f64,
        min_ack_rate: f64,
        anonymity_floor: usize,
    ) -> Self {
        Self {
            me,
            graph,
            max_paths,
            edge_penalty,
            min_ack_rate,
            anonymity_floor,
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
        existing: &[PathWithMetrics],
    ) -> Vec<PathWithMetrics> {
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

                // The graph crate already strips `src`; `path` is [intermediates…, terminator].
                // Guard: if dest already appears as an intermediate or terminator, appending it
                // would produce a non-adjacent duplicate that ValidatedPath::new rejects — skip early.
                if path.contains(dest) {
                    return None;
                }
                let mut candidate = path;
                candidate.push(*dest);

                // Skip paths already found by Phase 1 (compare path component only).
                if existing.iter().any(|pwm| pwm.path == candidate) {
                    return None;
                }

                tracing::trace!(?candidate, ?cost, "extended forward path candidate");
                Some(aggregate_metrics(&self.graph, src, &candidate, cost))
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
    /// accumulated traversal cost and per-path quality aggregates.
    ///
    /// Returns `Err(PathNotFound)` when the graph yields no positive-cost paths.
    ///
    /// The function has a potential to run expensive operations, it should be benchmarked
    /// in a production environment and possibly guarded (e.g. by offloading the long execution
    /// in an async executor to avoid blocking the caller).
    #[tracing::instrument(level = "trace", skip(self), fields(src = %src, dest = %dest, hops), ret, err)]
    fn select_path(
        &self,
        src: OffchainPublicKey,
        dest: OffchainPublicKey,
        hops: usize,
    ) -> Result<Vec<PathWithMetrics>> {
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

        for (i, pwm) in paths.iter().enumerate() {
            tracing::debug!(
                direction,
                index = i,
                path = ?pwm.path,
                cost = pwm.cost,
                total_latency_ms = ?pwm.total_latency_ms,
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
            Ok(prune_for_consistency(paths, self.anonymity_floor))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Context;
    use hex_literal::hex;
    use hopr_api::{
        graph::{
            NetworkGraphWrite,
            traits::{EdgeObservableWrite, EdgeWeightType},
        },
        types::{
            crypto::prelude::{Keypair, OffchainKeypair},
            internal::routing::RoutingOptions,
        },
    };
    use hopr_network_graph::ChannelGraph;

    use super::*;
    use crate::path::{PathPlannerConfig, traits::PathSelector};

    fn test_selector(
        me: OffchainPublicKey,
        graph: ChannelGraph,
        max_paths: usize,
    ) -> HoprGraphPathSelector<ChannelGraph> {
        let cfg = PathPlannerConfig::default();
        HoprGraphPathSelector::new(
            me,
            graph,
            max_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        )
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

    fn mark_edge_last(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        mark_edge_full(graph, src, dst);
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
        for pwm in &fwd {
            assert!(!pwm.path.contains(&me), "forward path must not contain the source");
            assert!(pwm.cost > 0.0, "cost must be positive");
        }

        let rev = selector.select_path(dest, me, 1).context("reverse path")?;
        assert!(!rev.is_empty());
        for pwm in &rev {
            assert!(!pwm.path.contains(&dest), "reverse path must not contain the source");
            assert!(pwm.cost > 0.0, "cost must be positive");
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
        for pwm in &fwd {
            assert_eq!(pwm.path.len(), 3, "forward 2-hop path: [A, B, dest]");
            assert_eq!(pwm.path.last(), Some(&dest));
        }

        let rev = selector.select_path(dest, me, 2).context("reverse 2-hop path")?;
        assert!(!rev.is_empty());
        for pwm in &rev {
            assert_eq!(pwm.path.len(), 3, "reverse 2-hop path: [B, A, me]");
            assert_eq!(pwm.path.last(), Some(&me));
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
        for pwm in &fwd {
            assert_eq!(pwm.path.len(), 2, "forward: [relay, dest]");
            assert_eq!(pwm.path.last(), Some(&dest));
            assert!(!pwm.path.contains(&me));
        }

        let rev = selector.select_path(dest, me, 1).context("reverse 1-hop path")?;
        assert!(!rev.is_empty());
        for pwm in &rev {
            assert_eq!(pwm.path.len(), 2, "reverse: [relay, me]");
            assert_eq!(pwm.path.last(), Some(&me));
            assert!(!pwm.path.contains(&dest));
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
        for pwm in &fwd {
            assert_eq!(pwm.path.last(), Some(&dest));
        }

        let rev = selector.select_path(dest, me, 1).context("reverse path")?;
        assert_eq!(rev.len(), 2, "reverse: both paths via a and b should be returned");
        for pwm in &rev {
            assert_eq!(pwm.path.last(), Some(&me));
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
        for pwm in &fwd {
            assert_eq!(pwm.path.len(), 2, "forward: [relay, dest]");
            assert_eq!(pwm.path[0], relay);
            assert_eq!(pwm.path[1], dest);
        }

        // Return path should work normally (both edges exist)
        let rev = selector.select_path(dest, me, 1).context("return path")?;
        assert!(!rev.is_empty(), "return path should find at least one route");
        for pwm in &rev {
            assert_eq!(pwm.path.len(), 2, "return: [relay, me]");
            assert_eq!(pwm.path.last(), Some(&me));
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
        for pwm in &fwd {
            assert_eq!(pwm.path.len(), 4, "forward: [a, b, c, dest]");
            assert_eq!(pwm.path.last(), Some(&dest));
            assert!(!pwm.path.contains(&me));
        }

        let rev = selector
            .select_path(dest, me, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .context("reverse 3-hop path")?;
        assert!(!rev.is_empty());
        for pwm in &rev {
            assert_eq!(pwm.path.len(), 4, "reverse: [c, b, a, me]");
            assert_eq!(pwm.path.last(), Some(&me));
            assert!(!pwm.path.contains(&dest));
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
        for pwm in &fwd {
            assert_eq!(pwm.path.len(), 2, "path must be [relay, dest]");
            assert_eq!(pwm.path[0], relay, "first node must be relay, not dest");
            assert_eq!(pwm.path[1], dest);
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
        for pwm in &fwd {
            assert_eq!(pwm.path[0], relay, "intermediate must be relay, not dest");
            assert_ne!(pwm.path[0], dest, "dest must not appear as intermediate");
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

    // ── pruning tests ─────────────────────────────────────────────────────────

    fn make_path_with_latency(latency_ms: Option<u32>) -> PathWithMetrics {
        PathWithMetrics {
            path: vec![],
            cost: 1.0,
            total_latency_ms: latency_ms,
            min_probe_success_rate: None,
            min_ack_rate: None,
            capacity_floor: None,
        }
    }

    #[test]
    fn prune_keeps_all_when_below_floor() {
        let candidates: Vec<_> = (0..5).map(|i| make_path_with_latency(Some(i * 10))).collect();
        let result = prune_for_consistency(candidates, 8);
        assert_eq!(result.len(), 5, "below floor: nothing should be dropped");
    }

    #[test]
    fn prune_drops_high_latency_first() {
        // 30 paths with strictly increasing latency, floor=8 → keep lowest 8
        let candidates: Vec<_> = (0..30u32).map(|i| make_path_with_latency(Some(i * 10))).collect();
        let result = prune_for_consistency(candidates, 8);
        assert_eq!(result.len(), 8);
        for p in &result {
            assert!(p.total_latency_ms.unwrap() < 80, "only the 8 lowest should survive");
        }
    }

    #[test]
    fn prune_preserves_unpopulated_paths_correctly() {
        // 3 populated + 6 unpopulated, floor=8
        // total=9 > floor=8 → prune 1 populated (the highest-latency one)
        let mut candidates: Vec<_> = vec![
            make_path_with_latency(Some(10)),
            make_path_with_latency(Some(30)),
            make_path_with_latency(Some(20)),
        ];
        candidates.extend((0..6).map(|_| make_path_with_latency(None)));
        let result = prune_for_consistency(candidates, 8);
        assert_eq!(result.len(), 8);
        // The lowest 2 populated (10ms, 20ms) should survive; 30ms should be dropped.
        let populated: Vec<_> = result.iter().filter(|p| p.total_latency_ms.is_some()).collect();
        assert_eq!(populated.len(), 2);
        assert!(populated.iter().any(|p| p.total_latency_ms == Some(10)));
        assert!(populated.iter().any(|p| p.total_latency_ms == Some(20)));
    }

    #[test]
    fn prune_drops_unpopulated_when_all_populated_exhausted() {
        // 0 populated, 20 unpopulated, floor=8 → keep first 8
        let candidates: Vec<_> = (0..20).map(|_| make_path_with_latency(None)).collect();
        let result = prune_for_consistency(candidates, 8);
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn prune_exact_floor_is_unchanged() {
        let candidates: Vec<_> = (0..8).map(|i| make_path_with_latency(Some(i * 10))).collect();
        let result = prune_for_consistency(candidates, 8);
        assert_eq!(result.len(), 8);
    }

    // ── path metrics aggregation tests ────────────────────────────────────────

    #[tokio::test]
    async fn path_metrics_aggregate_latency_correctly() -> anyhow::Result<()> {
        // 3-hop path: me → A (30ms) → B (40ms) → dest (50ms)
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let dest = pubkey(&SECRET_3);
        let graph = ChannelGraph::new(me);
        for n in [a, b, dest] {
            graph.add_node(n);
        }

        let make_edge = |src: &OffchainPublicKey, dst: &OffchainPublicKey, lat_ms: u64| {
            graph.upsert_edge(src, dst, |obs| {
                obs.record(EdgeWeightType::Connected(true));
                obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(lat_ms))));
                obs.record(EdgeWeightType::Capacity(Some(1000)));
            });
        };

        // Drive EMA to convergence with many samples at the target value.
        for _ in 0..20 {
            make_edge(&me, &a, 30);
            make_edge(&a, &b, 40);
            make_edge(&b, &dest, 50);
        }

        // Build the path [a, b, dest] (src=me excluded per selector convention).
        let path = vec![a, b, dest];
        let metrics = aggregate_metrics(&graph, &me, &path, 1.0);

        // EMA with factor 3 converges slowly; just verify it's in the expected range.
        let total = metrics.total_latency_ms.expect("latency must be Some");
        assert!(
            total >= 100 && total <= 130,
            "expected ~120ms total latency, got {total}ms"
        );
        Ok(())
    }

    #[tokio::test]
    async fn path_metrics_capacity_floor_is_min() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let hop = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);

        graph.upsert_edge(&me, &hop, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(500)));
        });
        graph.upsert_edge(&hop, &dest, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(200)));
        });

        let path = vec![hop, dest];
        let metrics = aggregate_metrics(&graph, &me, &path, 1.0);
        assert_eq!(
            metrics.capacity_floor,
            Some(200),
            "floor must be the smaller of 500 and 200"
        );
        Ok(())
    }

    #[tokio::test]
    async fn path_metrics_capacity_floor_is_none_when_missing() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let hop = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);

        // Only immediate probes, no capacity data.
        graph.upsert_edge(&me, &hop, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
        });
        graph.upsert_edge(&hop, &dest, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
        });

        let path = vec![hop, dest];
        let metrics = aggregate_metrics(&graph, &me, &path, 1.0);
        assert_eq!(metrics.capacity_floor, None);
        Ok(())
    }
}
