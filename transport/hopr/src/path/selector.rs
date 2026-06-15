use std::{cmp::Ordering, sync::Arc};

use hopr_api::{
    OffchainPublicKey,
    graph::{
        NetworkGraphTraverse, NetworkGraphView,
        function::{BasicValueFn, EdgeValueFn},
        traits::{
            EdgeImmediateProtocolObservable, EdgeLinkObservable, EdgeObservableRead, EdgeProtocolObservable, ValueFn,
        },
    },
    types::internal::errors::PathError,
};

use super::{
    errors::{PathPlannerError, Result},
    traits::{PathSelector, PathWithMetrics},
};

/// Accumulated path cost and quality aggregates, folded edge-by-edge during DFS.
///
/// `PartialOrd` / `PartialEq` compare only the `cost` field so the DFS
/// pruning threshold (`min_cost`) operates on the same scalar as before.
#[derive(Clone, Debug)]
struct PathCostWithMetrics {
    cost: f64,
    total_latency_ms: Option<u32>,
    min_probe_success_rate: Option<f64>,
    min_ack_rate: Option<f64>,
    capacity_floor: Option<u128>,
}

impl PartialOrd for PathCostWithMetrics {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

impl PartialEq for PathCostWithMetrics {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl From<(PathCostWithMetrics, Vec<OffchainPublicKey>)> for PathWithMetrics {
    fn from((metrics, path): (PathCostWithMetrics, Vec<OffchainPublicKey>)) -> Self {
        PathWithMetrics {
            path,
            cost: metrics.cost,
            total_latency_ms: metrics.total_latency_ms,
            min_probe_success_rate: metrics.min_probe_success_rate,
            min_ack_rate: metrics.min_ack_rate,
            capacity_floor: metrics.capacity_floor,
        }
    }
}

/// Returns the minimum of two `Option<T>` values, preferring `Some` over `None`.
fn opt_min<T: PartialOrd>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(if x <= y { x } else { y }),
        (x, y) => x.or(y),
    }
}

/// Wraps an `EdgeValueFn<f64, W>` as a `ValueFn` whose `Value` type carries
/// both the cost and per-path quality aggregates.
///
/// All cost semantics are delegated to the inner `EdgeValueFn`; the wrapper
/// only adds the aggregate fold on the same `&Weight` reference already
/// available during DFS, so no extra graph lookups are needed.
struct MetricsValueFn<W: EdgeObservableRead> {
    inner: EdgeValueFn<f64, W>,
}

impl<W> ValueFn for MetricsValueFn<W>
where
    W: EdgeObservableRead + Send + 'static,
{
    type Value = PathCostWithMetrics;
    type Weight = W;

    fn initial_value(&self) -> Self::Value {
        PathCostWithMetrics {
            cost: self.inner.initial_value(),
            total_latency_ms: Some(0),
            min_probe_success_rate: None,
            min_ack_rate: None,
            capacity_floor: None,
        }
    }

    fn min_value(&self) -> Option<Self::Value> {
        self.inner.min_value().map(|c| PathCostWithMetrics {
            cost: c,
            total_latency_ms: None,
            min_probe_success_rate: None,
            min_ack_rate: None,
            capacity_floor: None,
        })
    }

    fn into_value_fn(self) -> BasicValueFn<Self::Value, Self::Weight> {
        let inner = self.inner.into_value_fn();
        Arc::new(move |prev: PathCostWithMetrics, observed: &W, idx: usize| {
            let cost = inner(prev.cost, observed, idx);

            let edge_lat = observed
                .immediate_qos()
                .and_then(|m| m.average_latency())
                .or_else(|| observed.intermediate_qos().and_then(|m| m.average_latency()));
            let total_latency_ms = match (prev.total_latency_ms, edge_lat) {
                (Some(acc), Some(lat)) => Some(((acc as u128 + lat.as_millis()).min(u32::MAX as u128)) as u32),
                _ => None,
            };

            // Probe rate: taking the min of immediate and intermediate guards against nodes that
            // look good on direct probes but degrade under multi-hop load.
            let edge_probe = observed
                .immediate_qos()
                .map(|m| m.average_probe_rate())
                .into_iter()
                .chain(observed.intermediate_qos().map(|m| m.average_probe_rate()))
                .reduce(f64::min);
            let min_probe_success_rate = opt_min(prev.min_probe_success_rate, edge_probe);

            let edge_ack = observed.immediate_qos().and_then(|m| m.ack_rate());
            let min_ack_rate = opt_min(prev.min_ack_rate, edge_ack);

            let edge_cap = observed.intermediate_qos().and_then(|m| m.capacity());
            let capacity_floor = opt_min(prev.capacity_floor, edge_cap);

            PathCostWithMetrics {
                cost,
                total_latency_ms,
                min_probe_success_rate,
                min_ack_rate,
                capacity_floor,
            }
        })
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
///
/// A path is "fully measured" — and therefore preferred over unmeasured alternatives —
/// when `total_latency_ms` is known AND either `hops == 0` (direct path, no channel
/// expected) OR `capacity_floor` is also known.  This prevents 0-hop direct paths from
/// being demoted simply because they carry no channel-capacity data.
pub fn prune_for_consistency(candidates: Vec<PathWithMetrics>, floor: usize, hops: usize) -> Vec<PathWithMetrics> {
    // floor == 0 means "no pruning" — caller opts out entirely.
    if floor == 0 || candidates.len() <= floor {
        return candidates;
    }

    let fully_measured =
        |p: &PathWithMetrics| p.total_latency_ms.is_some() && (hops == 0 || p.capacity_floor.is_some());

    let (mut populated, unpopulated): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|p| fully_measured(p));

    // Sort populated ascending by latency (lowest first → drop from the end).
    populated.sort_by_key(|p| p.total_latency_ms.unwrap_or(u32::MAX));

    // Prefer measured paths: keep as many populated paths as fit within the floor,
    // then fill the remaining slots with unpopulated paths.  This ensures that
    // latency-measured candidates are never discarded when unprobed paths alone
    // would satisfy the floor.
    let target_populated = populated.len().min(floor);
    populated.truncate(target_populated);
    let remaining = floor - target_populated;

    let mut result = populated;
    result.extend(unpopulated.into_iter().take(remaining));

    result
}

/// Compute candidate paths from `src` to `dest` through `graph`.
///
/// `length` is the number of edges to traverse (= intermediate hops + 1).
/// `take` caps the number of candidate paths returned.
/// The graph crate returns only the intermediate nodes (both `src` and `dest` stripped);
/// this function re-appends `dest` so callers receive `([intermediates..., dest], cost)`.
fn compute_paths<G, W>(
    graph: &G,
    src: &OffchainPublicKey,
    dest: &OffchainPublicKey,
    length: std::num::NonZeroUsize,
    take: usize,
    value_fn: MetricsValueFn<W>,
) -> Vec<PathWithMetrics>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey, Observed = W>,
    W: EdgeObservableRead + Send + 'static,
{
    let raw = graph.simple_paths(src, dest, length.get(), Some(take), value_fn);

    raw.into_iter()
        .filter_map(|(path, _, metrics)| {
            tracing::trace!(?path, cost = metrics.cost, "evaluating candidate path");
            if metrics.cost > 0.0 {
                let mut path = path;
                path.push(*dest);
                Some(PathWithMetrics::from((metrics, path)))
            } else {
                None
            }
        })
        .collect()
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
        let value_fn = MetricsValueFn {
            inner: EdgeValueFn::forward_without_self_loopback(self.edge_penalty, self.min_ack_rate),
        };
        let raw = self
            .graph
            .simple_paths_from(src, shorter_length.get(), Some(take), value_fn);

        raw.into_iter()
            .filter_map(|(path, _, metrics)| {
                if metrics.cost <= 0.0 {
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

                if existing.iter().any(|pwm| pwm.path == candidate) {
                    return None;
                }

                tracing::trace!(?candidate, cost = metrics.cost, "extended forward path candidate");
                Some(PathWithMetrics::from((metrics, candidate)))
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
                MetricsValueFn {
                    inner: EdgeValueFn::forward(length, self.edge_penalty, self.min_ack_rate),
                },
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
                MetricsValueFn {
                    inner: EdgeValueFn::returning(length, self.edge_penalty, self.min_ack_rate),
                },
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
            Ok(prune_for_consistency(paths, self.anonymity_floor, hops))
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
        mark_edge_full(&graph, &hop, &dest);
        // Reverse: dest → hop → me
        graph.add_edge(&dest, &hop).unwrap();
        graph.add_edge(&hop, &me).unwrap();
        mark_edge_full(&graph, &dest, &hop);
        mark_edge_full(&graph, &hop, &me);
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
        mark_edge_full(&graph, &b, &dest);
        // Reverse: dest → B → A → me
        graph.add_edge(&dest, &b).unwrap();
        graph.add_edge(&b, &a).unwrap();
        graph.add_edge(&a, &me).unwrap();
        mark_edge_full(&graph, &dest, &b);
        mark_edge_full(&graph, &b, &a);
        mark_edge_full(&graph, &a, &me);

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
        mark_edge_full(&graph, &a, &dest);
        mark_edge_full(&graph, &b, &dest);
        // Reverse: dest → a → me,  dest → b → me
        graph.add_edge(&dest, &a).unwrap();
        graph.add_edge(&dest, &b).unwrap();
        graph.add_edge(&a, &me).unwrap();
        graph.add_edge(&b, &me).unwrap();
        mark_edge_full(&graph, &dest, &a);
        mark_edge_full(&graph, &dest, &b);
        mark_edge_full(&graph, &a, &me);
        mark_edge_full(&graph, &b, &me);

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
        mark_edge_full(&graph, &relay, &me);

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
        mark_edge_full(&graph, &c, &dest);
        // Reverse: dest → c → b → a → me
        graph.add_edge(&dest, &c).unwrap();
        graph.add_edge(&c, &b).unwrap();
        graph.add_edge(&b, &a).unwrap();
        graph.add_edge(&a, &me).unwrap();
        mark_edge_full(&graph, &dest, &c);
        mark_edge_full(&graph, &c, &b);
        mark_edge_full(&graph, &b, &a);
        mark_edge_full(&graph, &a, &me);

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
        mark_edge_full(&graph, &relay, &me);

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
        mark_edge_full(&graph, &relay, &me);

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

    fn make_path_with_capacity(latency_ms: Option<u32>, capacity_floor: Option<u128>) -> PathWithMetrics {
        PathWithMetrics {
            path: vec![],
            cost: 1.0,
            total_latency_ms: latency_ms,
            min_probe_success_rate: None,
            min_ack_rate: None,
            capacity_floor,
        }
    }

    #[test]
    fn prune_keeps_all_when_below_floor() {
        let candidates: Vec<_> = (0..5).map(|i| make_path_with_latency(Some(i * 10))).collect();
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 5, "below floor: nothing should be dropped");
    }

    #[test]
    fn prune_drops_high_latency_first() {
        // 30 paths with strictly increasing latency, floor=8 → keep lowest 8
        let candidates: Vec<_> = (0..30u32)
            .map(|i| make_path_with_capacity(Some(i * 10), Some(1_000_000)))
            .collect();
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
        for p in &result {
            assert!(p.total_latency_ms.unwrap() < 80, "only the 8 lowest should survive");
        }
    }

    #[test]
    fn prune_preserves_populated_paths_over_unpopulated() {
        // 3 populated + 6 unpopulated, floor=8
        // total=9 > floor=8: all 3 populated are kept (populated always preferred),
        // then 5 unpopulated fill the remaining slots.
        let mut candidates: Vec<_> = vec![
            make_path_with_capacity(Some(10), Some(1_000)),
            make_path_with_capacity(Some(30), Some(1_000)),
            make_path_with_capacity(Some(20), Some(1_000)),
        ];
        candidates.extend((0..6).map(|_| make_path_with_latency(None)));
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
        // All 3 populated paths survive; 1 unpopulated is trimmed.
        let populated: Vec<_> = result.iter().filter(|p| p.total_latency_ms.is_some()).collect();
        assert_eq!(populated.len(), 3);
        assert!(populated.iter().any(|p| p.total_latency_ms == Some(10)));
        assert!(populated.iter().any(|p| p.total_latency_ms == Some(20)));
        assert!(populated.iter().any(|p| p.total_latency_ms == Some(30)));
    }

    #[test]
    fn prune_drops_unpopulated_when_all_populated_exhausted() {
        // 0 populated, 20 unpopulated, floor=8 → keep first 8
        let candidates: Vec<_> = (0..20).map(|_| make_path_with_latency(None)).collect();
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn prune_keeps_populated_when_unpopulated_exceeds_floor() {
        // Regression: 2 populated + 10 unpopulated, floor=8.
        // Old formula: target_populated = 8.saturating_sub(10) = 0 → both populated dropped.
        // Correct: keep up to 8 populated (only 2 exist), fill 6 remaining with unpopulated.
        let mut candidates: Vec<_> = vec![
            make_path_with_capacity(Some(10), Some(1_000)),
            make_path_with_capacity(Some(20), Some(1_000)),
        ];
        candidates.extend((0..10).map(|_| make_path_with_latency(None)));
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
        let populated: Vec<_> = result.iter().filter(|p| p.total_latency_ms.is_some()).collect();
        assert_eq!(populated.len(), 2, "both measured paths must survive");
        assert!(populated.iter().any(|p| p.total_latency_ms == Some(10)));
        assert!(populated.iter().any(|p| p.total_latency_ms == Some(20)));
    }

    #[test]
    fn prune_exact_floor_is_unchanged() {
        let candidates: Vec<_> = (0..8)
            .map(|i| make_path_with_capacity(Some(i * 10), Some(1_000)))
            .collect();
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn prune_0_hop_with_measured_latency_and_no_capacity_is_populated() {
        // 0-hop: capacity_floor = None is expected; path should be treated as "fully measured"
        // if latency is known.
        let mut candidates: Vec<_> = vec![
            make_path_with_capacity(Some(50), None), // 0-hop: no capacity, latency known
        ];
        candidates.extend((0..10).map(|_| make_path_with_latency(None)));
        let result = prune_for_consistency(candidates, 8, 0);
        assert_eq!(result.len(), 8);
        // The 0-hop path must be in the populated bucket and survive.
        let has_0_hop = result.iter().any(|p| p.total_latency_ms == Some(50));
        assert!(has_0_hop, "0-hop path with measured latency must survive pruning");
    }

    #[test]
    fn prune_multi_hop_without_capacity_floor_is_unpopulated() {
        // A 1-hop path with measured latency but NO capacity is unmeasured (unpopulated).
        // It should be demoted below paths that have both latency and capacity.
        let mut candidates: Vec<_> = vec![
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(50), Some(1_000)), // fully measured
            make_path_with_capacity(Some(40), None),        // missing capacity → unpopulated
        ];
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
        // The path with missing capacity should not survive when all 8 slots are filled
        // by fully measured paths.
        let has_missing_cap = result.iter().any(|p| p.capacity_floor.is_none());
        assert!(
            !has_missing_cap,
            "path without capacity floor must be pruned when fully-measured paths fill the floor"
        );
    }

    #[test]
    fn prune_for_consistency_floor_zero_returns_all() {
        // floor == 0 must be treated as "no pruning" — all candidates survive unchanged.
        let candidates = vec![
            make_path_with_capacity(Some(10), Some(1_000)),
            make_path_with_capacity(Some(20), None),
            make_path_with_capacity(None, None),
        ];
        let result = prune_for_consistency(candidates, 0, 1);
        assert_eq!(result.len(), 3, "floor=0 must return all candidates");
    }

    // ── path metrics aggregation tests ────────────────────────────────────────

    #[tokio::test]
    async fn path_metrics_aggregate_latency_correctly() -> anyhow::Result<()> {
        // 3-hop path: me → A (30ms) → B (40ms) → dest (50ms)
        // Total latency accumulated during DFS should be ~120ms.
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

        // Edges only in the forward direction (me → a → b → dest)
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &b).unwrap();
        graph.add_edge(&b, &dest).unwrap();

        let selector = test_selector(me, graph, MAX_PATHS);
        let paths = selector.select_path(me, dest, 2).context("forward 2-hop path")?;
        assert!(!paths.is_empty());

        let total = paths[0].total_latency_ms.expect("latency must be Some");
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
        graph.add_edge(&me, &hop).unwrap();
        graph.add_edge(&hop, &dest).unwrap();

        let selector = test_selector(me, graph, MAX_PATHS);
        let paths = selector.select_path(me, dest, 1).context("1-hop path")?;
        assert!(!paths.is_empty());
        assert_eq!(
            paths[0].capacity_floor,
            Some(200),
            "floor must be the smaller of 500 and 200"
        );
        Ok(())
    }

    // NOTE: a test for capacity_floor=None is omitted intentionally.
    // The forward cost function requires intermediate capacity on all non-last edges, so
    // every path the selector returns has capacity data on at least one edge, making
    // capacity_floor always Some for reachable paths.
}
