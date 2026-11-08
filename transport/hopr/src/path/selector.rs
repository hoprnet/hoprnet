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
    fundable_tickets_floor: Option<u128>,
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
            fundable_tickets_floor: metrics.fundable_tickets_floor,
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
    /// Face value used to express each edge's balance as a count of single-hop tickets.
    ///
    /// Read once per query rather than stored on any edge: deriving a count at query time is
    /// safe, whereas deriving it at edge-update time is what made a price change stale the
    /// whole graph.
    ticket_face_value: Option<hopr_api::graph::traits::Balance>,
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
            fundable_tickets_floor: None,
        }
    }

    fn min_value(&self) -> Option<Self::Value> {
        self.inner.min_value().map(|c| PathCostWithMetrics {
            cost: c,
            total_latency_ms: None,
            min_probe_success_rate: None,
            min_ack_rate: None,
            fundable_tickets_floor: None,
        })
    }

    fn into_value_fn(self) -> BasicValueFn<Self::Value, Self::Weight> {
        let inner = self.inner.into_value_fn();
        let ticket_face_value = self.ticket_face_value;
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
            // `and_then`, not `map`: a stream with no observations contributes nothing to the min
            // rather than contributing a zero it never measured.
            let edge_probe = observed
                .immediate_qos()
                .and_then(|m| m.average_probe_rate())
                .into_iter()
                .chain(observed.intermediate_qos().and_then(|m| m.average_probe_rate()))
                .reduce(f64::min);
            let min_probe_success_rate = opt_min(prev.min_probe_success_rate, edge_probe);

            let edge_ack = observed.immediate_qos().and_then(|m| m.ack_rate());
            let min_ack_rate = opt_min(prev.min_ack_rate, edge_ack);

            // Expressed in single-hop tickets, not base units: the weighting factor below is a
            // log ratio, and a base-unit balance would compress every realistic channel into a
            // sliver of its output range.
            //
            // `zip`, not a fallback: an absent face value means the price is unknown, not that it
            // is one. Dividing a base-unit balance by `1` yields a ticket count that saturates to
            // `u128::MAX` and reads downstream as unlimited capacity — the opposite of what not
            // knowing should imply. Contribute nothing to the floor until pricing arrives.
            let edge_tickets = observed
                .intermediate_qos()
                .and_then(|m| m.balance())
                .zip(ticket_face_value)
                .map(|(balance, face_value)| {
                    let tickets = if face_value.is_zero() {
                        hopr_api::graph::traits::Balance::zero()
                    } else {
                        balance / face_value
                    };
                    if tickets > hopr_api::graph::traits::Balance::from(u128::MAX) {
                        u128::MAX
                    } else {
                        tickets.low_u128()
                    }
                });
            let fundable_tickets_floor = opt_min(prev.fundable_tickets_floor, edge_tickets);

            PathCostWithMetrics {
                cost,
                total_latency_ms,
                min_probe_success_rate,
                min_ack_rate,
                fundable_tickets_floor,
            }
        })
    }
}

/// Trim the candidate set to lower median latency and minimise variance while
/// preserving an anonymity floor and relayer diversity in every position.
///
/// Behaviour:
/// - If `candidates.len() <= floor`, returns all candidates unchanged (`min(found_count, floor)` semantics — the floor
///   is never a minimum to fabricate).
/// - Sorts candidates with a known `total_latency_ms` ascending.
/// - Keeps `floor` of them preferring distinct first relayers and bounding how often a relayer recurs in each later
///   position (see `take_relayer_diverse` and `MIDDLE_HOP_SHARE`), lowest latency first within those rules.
/// - If still over the floor with no populated candidates left, drops unpopulated candidates from the input-order tail.
///
/// A path is "fully measured" — and therefore preferred over unmeasured alternatives —
/// when `total_latency_ms` is known AND either `hops == 0` (direct path, no channel
/// expected) OR `fundable_tickets_floor` is also known.  This prevents 0-hop direct paths from
/// being demoted simply because they carry no channel-capacity data.
pub fn prune_for_consistency(candidates: Vec<PathWithMetrics>, floor: usize, hops: usize) -> Vec<PathWithMetrics> {
    // floor == 0 means "no pruning" — caller opts out entirely.
    if floor == 0 || candidates.len() <= floor {
        return candidates;
    }

    let fully_measured =
        |p: &PathWithMetrics| p.total_latency_ms.is_some() && (hops == 0 || p.fundable_tickets_floor.is_some());

    let (mut populated, unpopulated): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|p| fully_measured(p));

    // Sort populated ascending by latency (lowest first → drop from the end).
    populated.sort_by_key(|p| p.total_latency_ms.unwrap_or(u32::MAX));

    // Prefer measured paths: keep as many populated paths as fit within the floor,
    // then fill the remaining slots with unpopulated paths.  This ensures that
    // latency-measured candidates are never discarded when unprobed paths alone
    // would satisfy the floor.
    let target_populated = populated.len().min(floor);
    let populated = take_relayer_diverse(populated, target_populated);
    let remaining = floor - target_populated;

    let mut result = populated;
    result.extend(unpopulated.into_iter().take(remaining));

    result
}

/// A relayer may sit past the first position in at most `n.div_ceil(MIDDLE_HOP_SHARE)` of the `n`
/// candidates `take_relayer_diverse` keeps, while the pool allows it, counted over every position past
/// the first together: a relayer in the second position of some paths and the third of others costs
/// as many paths when it dies as one in either alone.
///
/// Distinct first relayers alone leave every other position free to collapse: the best path from
/// each first relayer often runs through the same second hop, so one relayer sat in the middle of
/// every forward candidate, and when it died the whole forward direction went dark at once, with
/// nothing left to corroborate a verdict against (measured: the busiest relay of a 2-hop session
/// carried 29 % of all relayed packets; killed, the session never recovered). A third bounds what
/// one relayer's death costs to about a third of the draws before the weights are even looked at.
const MIDDLE_HOP_SHARE: usize = 3;

/// Takes `n` candidates from `sorted`, preferring not-yet-represented first relayers and bounding
/// how often any relayer recurs past the first position.
///
/// Plain truncation keeps the `n` lowest-latency paths, which in a well-connected network are often
/// the *same* few relayers reached by different routes; every downstream draw then inherits that
/// concentration. Relative order is preserved, so each pass stays latency-ordered. Four passes:
/// paths with a new first relayer whose later relayers are all under the `MIDDLE_HOP_SHARE` cap;
/// then any path under the cap; then, over the cap, paths with a new first relayer, so a pool whose
/// every path shares a later relayer still keeps as many first relayers as it offers; then the best
/// of the rest, so a thin pool still fills the floor.
fn take_relayer_diverse(sorted: Vec<PathWithMetrics>, n: usize) -> Vec<PathWithMetrics> {
    if sorted.len() <= n {
        return sorted;
    }

    let cap = n.div_ceil(MIDDLE_HOP_SHARE).max(1);
    let mut seen: Vec<OffchainPublicKey> = Vec::with_capacity(n);
    let mut per_relayer: std::collections::HashMap<OffchainPublicKey, usize> = Default::default();
    let mut taken = vec![false; sorted.len()];
    let mut count = 0;

    // The relayers of `path` past the first: `path` ends with the destination, which is no relayer.
    let later_relayers = |p: &PathWithMetrics| {
        p.path
            .iter()
            .skip(1)
            .take(p.path.len().saturating_sub(2))
            .copied()
            .collect::<Vec<_>>()
    };

    let take = |i: usize,
                p: &PathWithMetrics,
                taken: &mut Vec<bool>,
                per_relayer: &mut std::collections::HashMap<OffchainPublicKey, usize>| {
        taken[i] = true;
        for relayer in later_relayers(p) {
            *per_relayer.entry(relayer).or_default() += 1;
        }
    };

    // First pass: one path per distinct first relayer, best (lowest-latency) first, with every
    // later relayer under the cap.
    for (i, p) in sorted.iter().enumerate() {
        if count == n {
            break;
        }
        let Some(first) = p.path.first() else { continue };
        let under_cap = later_relayers(p)
            .into_iter()
            .all(|relayer| per_relayer.get(&relayer).copied().unwrap_or(0) < cap);
        if !seen.contains(first) && under_cap {
            seen.push(*first);
            take(i, p, &mut taken, &mut per_relayer);
            count += 1;
        }
    }

    // Second pass: any path whose later relayers are still under the cap, best first.
    for (i, p) in sorted.iter().enumerate() {
        if count == n {
            break;
        }
        if taken[i] {
            continue;
        }
        let under_cap = later_relayers(p)
            .into_iter()
            .all(|relayer| per_relayer.get(&relayer).copied().unwrap_or(0) < cap);
        if under_cap {
            if let Some(first) = p.path.first()
                && !seen.contains(first)
            {
                seen.push(*first);
            }
            take(i, p, &mut taken, &mut per_relayer);
            count += 1;
        }
    }

    // Third pass: over the cap, a first relayer not yet represented still comes before a second path
    // from one that is. Losing a later relayer then costs what the cap could not bound, but losing a
    // first relayer costs no more than its own share.
    for (i, p) in sorted.iter().enumerate() {
        if count == n {
            break;
        }
        if taken[i] {
            continue;
        }
        let Some(first) = p.path.first() else { continue };
        if !seen.contains(first) {
            seen.push(*first);
            take(i, p, &mut taken, &mut per_relayer);
            count += 1;
        }
    }

    // Fourth pass: fill any remaining slots with the best of the rest.
    for slot in taken.iter_mut() {
        if count == n {
            break;
        }
        if !*slot {
            *slot = true;
            count += 1;
        }
    }

    sorted
        .into_iter()
        .zip(taken)
        .filter_map(|(p, keep)| keep.then_some(p))
        .collect()
}

/// Number of distinct first relayers across `paths`.
///
/// The first relayer is the only node the destination gets to pick when it uses a SURB, so this is
/// the count that decides whether the return-path degradation detector has a sibling to corroborate
/// against: a value of 1 is the blind spot where a dead relayer is indistinguishable from a quiet
/// peer. Paths with no hops (`path` empty) contribute nothing — a 0-hop return has no relayer.
pub(crate) fn distinct_first_relayers(paths: &[PathWithMetrics]) -> usize {
    paths
        .iter()
        .filter_map(|p| p.path.first())
        .collect::<std::collections::HashSet<_>>()
        .len()
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

/// Continues a fold that an earlier traversal started: it starts from `initial` and hands the
/// wrapped function each edge's index shifted by `offset`.
///
/// A traversal from a path's second node, continued from the value after its first edge with an
/// offset of one, costs every edge exactly as one traversal from the first node would: the value
/// functions read an edge's position in the whole path, and that is what they are given.
struct ContinuedValueFn<V, W> {
    value_fn: BasicValueFn<V, W>,
    initial: V,
    min: Option<V>,
    offset: usize,
}

impl<V, W> ValueFn for ContinuedValueFn<V, W>
where
    V: Clone + PartialOrd + Send + Sync + 'static,
    W: EdgeObservableRead + Send + 'static,
{
    type Value = V;
    type Weight = W;

    fn initial_value(&self) -> Self::Value {
        self.initial.clone()
    }

    fn min_value(&self) -> Option<Self::Value> {
        self.min.clone()
    }

    fn into_value_fn(self) -> BasicValueFn<Self::Value, Self::Weight> {
        let (value_fn, offset) = (self.value_fn, self.offset);
        Arc::new(move |value, observed, index| value_fn(value, observed, index + offset))
    }
}

/// Candidate paths from `src` to `dest`, spread evenly over `src`'s first relayers.
///
/// The traversal is depth-first and `take` keeps the paths it finds first, so a single traversal
/// returns the whole subtree of one first relayer before it reaches the next. Over seven relayers
/// a three-hop subtree holds 30 paths, so 50 candidates named one or two first relayers, and
/// every path drawn until the next refresh went through them. Pruning for distinct first relayers
/// cannot restore what the pool never held: a return pool of one first relayer put a Session's
/// whole SURB stock behind it, and when that relayer died nothing the Exit sent got home.
///
/// So the first edges are found once, and each first relayer's continuations are searched
/// separately within an even share of `take`, continued from the value after the first edge so
/// that each path costs exactly what the single traversal would have made it cost. The shares
/// are interleaved, so a pool cut short at `take` still keeps every first relayer.
fn compute_paths_spread<G, W>(
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
    // A direct path has no relayer to spread over.
    if length.get() < 2 {
        return compute_paths(graph, src, dest, length, take, value_fn);
    }

    let initial = value_fn.initial_value();
    let min = value_fn.min_value();
    let value_fn = value_fn.into_value_fn();

    // Every first edge the full traversal would take, with the value after it: the minimum
    // prunes here exactly as it does there.
    let firsts = graph
        .simple_paths_from(
            src,
            1,
            None,
            ContinuedValueFn {
                value_fn: value_fn.clone(),
                initial,
                min: min.clone(),
                offset: 0,
            },
        )
        .into_iter()
        .filter_map(|(nodes, _, after_first)| match nodes.as_slice() {
            [first] if first != dest && first != src => Some((*first, after_first)),
            _ => None,
        })
        .collect::<Vec<_>>();

    if firsts.is_empty() {
        return Vec::new();
    }

    let share = take.div_ceil(firsts.len()).max(1);
    let mut per_first = firsts
        .into_iter()
        .map(|(first, after_first)| {
            let continuations = |limit| {
                graph.simple_paths(
                    &first,
                    dest,
                    length.get() - 1,
                    Some(limit),
                    ContinuedValueFn {
                        value_fn: value_fn.clone(),
                        initial: after_first.clone(),
                        min: min.clone(),
                        offset: 1,
                    },
                )
            };
            // The single traversal never returns to `src`; one that starts past it can, and those
            // continuations are dropped below. They must not take the places of the ones that avoid
            // `src`, so the depth-first search, which returns the same paths in the same order
            // whatever its limit, is asked again for as many more as ran through `src`, until `share`
            // avoid it or the subtree runs out. Each round asks for more than the last.
            let mut limit = share;
            let found = loop {
                let found = continuations(limit);
                let through_src = found.iter().filter(|(rest, ..)| rest.contains(src)).count();
                if found.len() < limit || found.len() - through_src >= share {
                    break found;
                }
                limit = share + through_src;
            };
            found
                .into_iter()
                .filter_map(move |(rest, _, metrics)| {
                    if metrics.cost <= 0.0 || rest.contains(src) {
                        return None;
                    }
                    let mut path = Vec::with_capacity(rest.len() + 2);
                    path.push(first);
                    path.extend(rest);
                    path.push(*dest);
                    tracing::trace!(?path, cost = metrics.cost, "evaluating candidate path");
                    Some(PathWithMetrics::from((metrics, path)))
                })
                .take(share)
                .collect::<Vec<_>>()
                .into_iter()
        })
        .collect::<Vec<_>>();

    // Sized by what the searches found: `take` is a configured bound, not a size.
    let found = per_first.iter().map(|paths| paths.len()).sum::<usize>();
    let mut spread = Vec::with_capacity(take.min(found));
    while spread.len() < take {
        let before = spread.len();
        for paths in per_first.iter_mut() {
            if spread.len() == take {
                break;
            }
            spread.extend(paths.next());
        }
        if spread.len() == before {
            break;
        }
    }
    spread
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
        // The caller's snapshot, so both phases of one query cost against the same face value.
        ticket_face_value: Option<hopr_api::graph::traits::Balance>,
    ) -> Vec<PathWithMetrics> {
        // `forward_without_self_loopback` counts the *finished* path length — the one that includes
        // the `dest` edge appended below — not the shorter traversal depth. Passing the search depth
        // understates it by one: for a 1-hop path it collapses `length` to 1, so the first
        // (monetized) edge is funding-checked with zero remaining hops, waiving the check and
        // selecting a channel that cannot fund the first-hop ticket (GNO-805). Search `shorter_length`
        // edges, but cost them against the finished length.
        let finished_length = shorter_length.saturating_add(1);
        let value_fn = MetricsValueFn {
            inner: EdgeValueFn::forward_without_self_loopback(
                finished_length,
                self.edge_penalty,
                self.min_ack_rate,
                ticket_face_value,
            ),
            ticket_face_value,
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

                // The missing last edge is assumed reachable only while the graph has no word on it. An
                // edge it has measured dead (a score of exactly 0, which is how it reads a starved edge)
                // is not missing, and adding the relay back would fill the floor with a path that
                // delivers nothing, at a better cost than the healthy ones.
                if let [.., relay, _] = candidate.as_slice()
                    && self.graph.edge(relay, dest).and_then(|obs| obs.score()) == Some(0.0)
                {
                    return None;
                }

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

        // One read for the whole query. A concurrent `set_ticket_face_value` between two reads
        // would let the traversal cost and the fundable-ticket floor come from different snapshots
        // of the same graph, so `composite_weight` would rank a candidate against itself.
        let ticket_face_value = self.graph.ticket_face_value();

        let paths = if src == self.me {
            // Phase 1: search for full-length forward paths to dest.
            let mut found = compute_paths_spread(
                &self.graph,
                &src,
                &dest,
                length,
                self.max_paths,
                MetricsValueFn {
                    inner: EdgeValueFn::forward(length, self.edge_penalty, self.min_ack_rate, ticket_face_value),
                    ticket_face_value,
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
                let extended =
                    self.compute_extended_forward_paths(&src, &dest, shorter, remaining, &found, ticket_face_value);
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
            let found = compute_paths_spread(
                &self.graph,
                &src,
                &dest,
                length,
                self.max_paths,
                MetricsValueFn {
                    inner: EdgeValueFn::returning(length, self.edge_penalty, self.min_ack_rate, ticket_face_value),
                    ticket_face_value,
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
            return Err(PathPlannerError::Path(PathError::PathNotFound(
                hops,
                src.to_string(),
                dest.to_string(),
            )));
        }

        let pruned = prune_for_consistency(paths, self.anonymity_floor, hops);
        let relayer_set = pruned
            .iter()
            .filter_map(|p| p.path.first())
            .collect::<std::collections::HashSet<_>>();
        tracing::debug!(
            %src,
            %dest,
            hops,
            direction,
            survived = pruned.len(),
            distinct_relayers = relayer_set.len(),
            first_relayers = ?relayer_set,
            "pruned candidate paths",
        );

        Ok(pruned)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Context;
    use hex_literal::hex;
    use hopr_api::{
        graph::{
            NetworkGraphUpdate, NetworkGraphWrite,
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
            obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                1000u64,
            ))));
        });
    }

    /// An edge that pings and has a funded channel, with no loopback evidence yet.
    fn mark_edge_unlooped(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                1000u64,
            ))));
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

    /// The extended search that fills up the anonymity floor appends the destination to each relay it
    /// finds without looking at the edge into it. A relay whose edge into the destination the graph has
    /// judged dead must not come back through it: the floor is never a minimum to fabricate.
    #[tokio::test]
    async fn the_floor_should_not_bring_back_a_relay_whose_edge_into_the_destination_is_dead() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let good = pubkey(&SECRET_1);
        let bad1 = pubkey(&SECRET_2);
        let bad2 = pubkey(&SECRET_3);
        let dest = pubkey(&hex!(
            "b1c3ec2e9a3b1d5b7f0b5c3a8f3e0d6c4a2b1e0f9d8c7b6a59483726150f4e3d"
        ));

        let graph = ChannelGraph::new(me);
        for node in [good, bad1, bad2, dest] {
            graph.add_node(node);
        }
        graph.set_ticket_face_value(hopr_api::graph::traits::Balance::one());
        for relay in [good, bad1, bad2] {
            graph.add_edge(&me, &relay)?;
            mark_edge_full(&graph, &me, &relay);
            graph.add_edge(&relay, &dest)?;
        }
        mark_edge_full(&graph, &good, &dest);
        for bad in [bad1, bad2] {
            mark_edge_unlooped(&graph, &bad, &dest);
        }
        // Loops over the edge into the destination: they return for `good`, never for the others.
        for _ in 0..5 {
            graph.upsert_edge(&good, &dest, |obs| {
                obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(30))));
            });
            for bad in [bad1, bad2] {
                graph.upsert_edge(&bad, &dest, |obs| {
                    obs.record(EdgeWeightType::Intermediate(Err(())));
                });
            }
        }

        // Fewer routable candidates than the floor, so the extended search runs.
        let selector = test_selector(me, graph.clone(), 8);
        let paths = selector.select_path(me, dest, 1).context("forward path")?;

        assert!(
            paths.iter().any(|pwm| pwm.path.contains(&good)),
            "the healthy relay must stay a candidate"
        );
        assert!(
            !paths
                .iter()
                .any(|pwm| pwm.path.contains(&bad1) || pwm.path.contains(&bad2)),
            "a relay whose edge into the destination is dead must not be added to fill the floor, got {} candidates",
            paths.len()
        );
        Ok(())
    }

    /// A relay that rejects our packets still answers pings and acks, so the first edge to it reads
    /// healthy everywhere except in the loopbacks routed through it, none of which return.
    #[tokio::test]
    async fn a_first_relay_whose_loopbacks_never_return_should_be_excluded_for_one_to_three_hops() -> anyhow::Result<()>
    {
        let me = pubkey(&SECRET_0);
        let good = pubkey(&SECRET_1);
        let bad = pubkey(&SECRET_2);
        let m1 = pubkey(&SECRET_3);
        let m2 = pubkey(&SECRET_4);
        let dest = pubkey(&hex!(
            "b1c3ec2e9a3b1d5b7f0b5c3a8f3e0d6c4a2b1e0f9d8c7b6a59483726150f4e3d"
        ));

        let graph = ChannelGraph::new(me);
        for node in [good, bad, m1, m2, dest] {
            graph.add_node(node);
        }
        // me → good|bad → dest (1 hop), → m1 → dest (2 hops), → m1 → m2 → dest (3 hops)
        for (src, dst) in [
            (good, dest),
            (bad, dest),
            (good, m1),
            (bad, m1),
            (m1, dest),
            (m1, m2),
            (m2, dest),
            (me, good),
        ] {
            graph.add_edge(&src, &dst)?;
            mark_edge_full(&graph, &src, &dst);
        }
        graph.add_edge(&me, &bad)?;
        mark_edge_unlooped(&graph, &me, &bad);

        let selector = test_selector(me, graph.clone(), 50);
        let costs = |hops: usize| -> anyhow::Result<(Vec<f64>, Vec<f64>)> {
            let paths = selector.select_path(me, dest, hops).context("forward path")?;
            let (through, around): (Vec<_>, Vec<_>) = paths.into_iter().partition(|pwm| pwm.path.contains(&bad));
            Ok((
                through.into_iter().map(|pwm| pwm.cost).collect(),
                around.into_iter().map(|pwm| pwm.cost).collect(),
            ))
        };

        for hops in 1..=3 {
            let (through, around) = costs(hops)?;
            assert!(
                !through.is_empty() && !around.is_empty(),
                "{hops} hop(s): both relays must be candidates before any loop is seen"
            );
        }

        // 5 loops through the first relay, none returned.
        graph.upsert_edge(&me, &bad, |obs| {
            for _ in 0..5 {
                obs.record(EdgeWeightType::Intermediate(Err(())));
            }
        });

        // Excluded from routing (RFC-0010 4.2.3, the active tier); probing still reaches it.
        for hops in 1..=3 {
            let (through, around) = costs(hops)?;
            assert!(
                through.is_empty(),
                "{hops} hop(s): a relay whose loops never return must not be a candidate, got {through:?}"
            );
            assert!(
                !around.is_empty() && around.iter().all(|cost| *cost > 0.0),
                "{hops} hop(s): the honest relay must stay routable"
            );
        }
        Ok(())
    }

    /// On a return path `dest → relay → me`, the edge into the relay is judged and the edge into `me`
    /// is not: a relay that rejects packets shows it on the first, and the last leads to the receiver.
    #[tokio::test]
    async fn a_return_relay_should_be_excluded_when_loopbacks_fail_on_the_edge_into_it() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let good = pubkey(&SECRET_1);
        let bad = pubkey(&SECRET_2);
        let dest = pubkey(&hex!(
            "b1c3ec2e9a3b1d5b7f0b5c3a8f3e0d6c4a2b1e0f9d8c7b6a59483726150f4e3d"
        ));

        // Costs of the return paths through `bad` and around it, after loops failed over one edge of `bad`.
        let costs = |failed: (OffchainPublicKey, OffchainPublicKey)| -> anyhow::Result<(Vec<f64>, Vec<f64>)> {
            let graph = ChannelGraph::new(me);
            for node in [good, bad, dest] {
                graph.add_node(node);
            }
            for (src, dst) in [(dest, good), (good, me), (dest, bad), (bad, me)] {
                graph.add_edge(&src, &dst)?;
                if (src, dst) == failed {
                    // No loop returned over the edge, so the 5 below are all it has seen.
                    mark_edge_unlooped(&graph, &src, &dst);
                } else {
                    mark_edge_full(&graph, &src, &dst);
                }
            }
            graph.upsert_edge(&failed.0, &failed.1, |obs| {
                for _ in 0..5 {
                    obs.record(EdgeWeightType::Intermediate(Err(())));
                }
            });
            let paths = test_selector(me, graph, 50)
                .select_path(dest, me, 1)
                .context("return path")?;
            let (through, around): (Vec<_>, Vec<_>) = paths.into_iter().partition(|pwm| pwm.path.contains(&bad));
            Ok((
                through.into_iter().map(|pwm| pwm.cost).collect(),
                around.into_iter().map(|pwm| pwm.cost).collect(),
            ))
        };

        // A loop that fails blames the edge into the relay, and that is what excludes it.
        let (through, around) = costs((dest, bad))?;
        assert!(
            through.is_empty(),
            "a failed edge into the relay must exclude it, got {through:?}"
        );
        assert!(
            !around.is_empty() && around.iter().all(|cost| *cost > 0.0),
            "the honest relay must stay routable"
        );
        Ok(())
    }

    /// A relay that rejects our packets answers fast, so after the value function starves it, a latency-ordered
    /// pruning must not hand its slots to the starved paths. Layered graph of three relays per layer, every layer
    /// pair connected, and the dead relay in layer `position` being the one the failed loopbacks are on.
    /// Healthy paths outnumber the anonymity floor, so none of the kept paths may go through the dead relay.
    #[tokio::test]
    async fn a_fast_dead_relay_should_not_keep_a_slot_after_pruning_for_zero_to_three_hops_in_both_directions()
    -> anyhow::Result<()> {
        const FLOOR: usize = 2;
        let me = pubkey(&[1; 32]);
        let dest = pubkey(&[2; 32]);
        let relay = |layer: usize, index: usize| pubkey(&[10 + (layer * 3 + index) as u8; 32]);

        // 0 hops has no relay to judge: the direct route stays.
        for direction in ["forward", "return"] {
            let graph = ChannelGraph::new(me);
            graph.add_node(dest);
            let (src, dst) = if direction == "forward" { (me, dest) } else { (dest, me) };
            graph.add_edge(&src, &dst)?;
            mark_edge_full(&graph, &src, &dst);
            let paths = test_selector(me, graph, 50)
                .select_path(src, dst, 0)
                .context("direct path")?;
            assert_eq!(paths.len(), 1, "{direction}, 0 hops: the direct route must stay");
        }

        let mut failures = Vec::new();
        for direction in ["forward", "return"] {
            for hops in 1..=3usize {
                for position in 1..=hops {
                    let graph = ChannelGraph::new(me);
                    // A node always knows the ticket price, and without it no path counts as fully measured.
                    graph.set_ticket_face_value(hopr_api::graph::traits::Balance::one());
                    graph.add_node(dest);
                    for layer in 1..=hops {
                        for index in 0..3 {
                            graph.add_node(relay(layer, index));
                        }
                    }
                    // Layer 0 is the sender, layer `hops + 1` the receiver.
                    let (sender, receiver) = if direction == "forward" { (me, dest) } else { (dest, me) };
                    let dead = relay(position, 0);
                    let node = |layer: usize, index: usize| match layer {
                        0 => sender,
                        l if l == hops + 1 => receiver,
                        l => relay(l, index),
                    };
                    for layer in 0..=hops {
                        for from in 0..if layer == 0 { 1 } else { 3 } {
                            for to in 0..if layer == hops { 1 } else { 3 } {
                                let (src, dst) = (node(layer, from), node(layer + 1, to));
                                graph.add_edge(&src, &dst)?;
                                if dst == dead {
                                    mark_edge_unlooped(&graph, &src, &dst);
                                } else {
                                    mark_edge_full(&graph, &src, &dst);
                                }
                            }
                        }
                    }
                    // The dead relay answers faster than the healthy ones, and none of its loops return.
                    for from in 0..if position == 1 { 1 } else { 3 } {
                        graph.upsert_edge(&node(position - 1, from), &dead, |obs| {
                            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(5))));
                            for _ in 0..5 {
                                obs.record(EdgeWeightType::Intermediate(Err(())));
                            }
                        });
                    }

                    let selector = HoprGraphPathSelector::new(
                        me,
                        graph,
                        50,
                        PathPlannerConfig::default().edge_penalty,
                        PathPlannerConfig::default().min_ack_rate,
                        FLOOR,
                    );
                    let kept = selector.select_path(sender, receiver, hops).context("pruned paths")?;
                    let through_dead = kept.iter().filter(|pwm| pwm.path.contains(&dead)).count();
                    if kept.is_empty() || through_dead > 0 {
                        failures.push(format!(
                            "{direction}, {hops} hop(s), dead relay in position {position}: {through_dead} of {} kept \
                             paths run through the dead relay",
                            kept.len()
                        ));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        Ok(())
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
            fundable_tickets_floor: None,
        }
    }

    fn make_path_with_tickets(latency_ms: Option<u32>, fundable_tickets_floor: Option<u128>) -> PathWithMetrics {
        PathWithMetrics {
            path: vec![],
            cost: 1.0,
            total_latency_ms: latency_ms,
            min_probe_success_rate: None,
            min_ack_rate: None,
            fundable_tickets_floor,
        }
    }

    /// A candidate whose first hop is relayer `relayer_idx`, with the given latency.
    fn make_path_via(relayer_idx: u8, latency_ms: u32) -> PathWithMetrics {
        let mut secret = [1u8; 32];
        secret[0] = relayer_idx.max(1);
        let relayer = *OffchainKeypair::from_secret(&secret).expect("valid secret").public();
        PathWithMetrics {
            path: vec![relayer],
            cost: 1.0,
            total_latency_ms: Some(latency_ms),
            min_probe_success_rate: None,
            min_ack_rate: None,
            fundable_tickets_floor: Some(1000),
        }
    }

    #[test]
    fn prune_should_prefer_distinct_first_relayers_over_pure_latency_order() {
        // The four lowest-latency paths all leave via relayer 1; relayers 2..4 are slower. A pure
        // latency truncation to 3 would keep only relayer 1, collapsing all downstream diversity.
        let candidates = vec![
            make_path_via(1, 10),
            make_path_via(1, 11),
            make_path_via(1, 12),
            make_path_via(1, 13),
            make_path_via(2, 50),
            make_path_via(3, 60),
            make_path_via(4, 70),
        ];

        let result = prune_for_consistency(candidates, 3, 1);
        assert_eq!(3, result.len());

        let relayers: Vec<String> = result
            .iter()
            .map(|p| hopr_api::types::primitive::traits::ToHex::to_hex(p.path.first().expect("non-empty")))
            .collect();
        let mut distinct = relayers.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(3, distinct.len(), "the floor must be filled with distinct relayers");

        // The best path per relayer is the one kept, so latency still drives the choice within
        // each relayer.
        assert_eq!(Some(10), result[0].total_latency_ms, "still latency-ordered");
    }

    /// A 2-hop candidate: first relayer `first_idx`, middle relayer `middle_idx`, then the destination.
    fn make_path_through(first_idx: u8, middle_idx: u8, latency_ms: u32) -> PathWithMetrics {
        let key = |tag: u8, idx: u8| {
            let mut secret = [1u8; 32];
            secret[0] = idx.max(1);
            secret[2] = tag;
            *OffchainKeypair::from_secret(&secret).expect("valid secret").public()
        };
        PathWithMetrics {
            path: vec![key(1, first_idx), key(2, middle_idx), key(3, 1)],
            cost: 1.0,
            total_latency_ms: Some(latency_ms),
            min_probe_success_rate: None,
            min_ack_rate: None,
            fundable_tickets_floor: Some(1000),
        }
    }

    /// Eight first relayers, each with its fastest path through the same middle relayer and a
    /// slower one through its own. Distinct first relayers alone would keep the eight fastest paths
    /// and put the hub in the middle of every candidate; the cap lets it into at most 3 of the 8.
    #[test]
    fn prune_should_bound_how_often_a_relayer_recurs_in_a_middle_position() {
        let hub = 9u8;
        let mut candidates = Vec::new();
        for first in 1..=8u8 {
            candidates.push(make_path_through(first, hub, 10 + u32::from(first)));
            candidates.push(make_path_through(first, first, 50 + u32::from(first)));
        }

        let result = prune_for_consistency(candidates, 8, 2);
        assert_eq!(8, result.len());

        let hub_key = make_path_through(1, hub, 0).path[1];
        let through_hub = result.iter().filter(|p| p.path[1] == hub_key).count();
        assert_eq!(
            3, through_hub,
            "the hub sits in the middle of at most ceil(8 / 3) candidates"
        );

        let firsts: std::collections::HashSet<_> = result.iter().map(|p| p.path[0]).collect();
        assert_eq!(8, firsts.len(), "every first relayer is still represented");
        assert_eq!(
            Some(11),
            result[0].total_latency_ms,
            "the fastest path is still kept, and the order is still by latency"
        );
    }

    /// Over the cap, a new first relayer still comes before a second path from one already kept:
    /// every path runs through the same middle relayer, the first pass keeps two, and the floor of
    /// four is filled from the two first relayers not yet represented, not from the next-fastest
    /// paths of the two that are.
    #[test]
    fn prune_should_keep_distinct_first_relayers_when_the_cap_runs_out() {
        let hub = 9u8;
        let candidates = vec![
            make_path_through(1, hub, 10),
            make_path_through(1, hub, 11),
            make_path_through(2, hub, 12),
            make_path_through(2, hub, 13),
            make_path_through(3, hub, 14),
            make_path_through(4, hub, 15),
        ];
        let result = prune_for_consistency(candidates, 4, 2);
        assert_eq!(4, result.len());
        let firsts: std::collections::HashSet<_> = result.iter().map(|p| p.path[0]).collect();
        assert_eq!(4, firsts.len(), "four first relayers, not two kept twice each");
    }

    /// A 3-hop candidate over relayers named by index whatever their position, so that one relayer can
    /// sit second in some paths and third in others.
    fn make_three_hop_path(first: u8, second: u8, third: u8, latency_ms: u32) -> PathWithMetrics {
        let relayer = |idx: u8| {
            let mut secret = [1u8; 32];
            secret[0] = idx.max(1);
            secret[2] = 7;
            *OffchainKeypair::from_secret(&secret).expect("valid secret").public()
        };
        let mut destination = [1u8; 32];
        destination[2] = 3;
        PathWithMetrics {
            path: vec![
                relayer(first),
                relayer(second),
                relayer(third),
                *OffchainKeypair::from_secret(&destination)
                    .expect("valid secret")
                    .public(),
            ],
            cost: 1.0,
            total_latency_ms: Some(latency_ms),
            min_probe_success_rate: None,
            min_ack_rate: None,
            fundable_tickets_floor: Some(1000),
        }
    }

    /// The cap counts a relayer over every position past the first together. Six first relayers whose
    /// fastest paths put the same relayer second in three of them and third in the other three: counted
    /// per position, it could sit in four of the six candidates; counted together, it sits in two.
    #[test]
    fn prune_should_bound_a_relayer_over_all_later_positions_together() {
        let hub = 30u8;
        let mut candidates = Vec::new();
        for first in 1..=3u8 {
            candidates.push(make_three_hop_path(first, hub, 10 + first, u32::from(first)));
        }
        for first in 4..=6u8 {
            candidates.push(make_three_hop_path(first, 10 + first, hub, u32::from(first)));
        }
        for first in 1..=6u8 {
            candidates.push(make_three_hop_path(
                first,
                20 + first,
                40 + first,
                100 + u32::from(first),
            ));
        }

        let result = prune_for_consistency(candidates, 6, 3);
        assert_eq!(6, result.len());
        let hub_key = make_three_hop_path(1, hub, 2, 0).path[1];
        let through_hub = result.iter().filter(|p| p.path[1..3].contains(&hub_key)).count();
        assert_eq!(
            2, through_hub,
            "the hub sits past the first position in at most ceil(6 / 3) candidates"
        );
        let firsts: std::collections::HashSet<_> = result.iter().map(|p| p.path[0]).collect();
        assert_eq!(6, firsts.len(), "every first relayer is still represented");
    }

    /// With the pool too thin to respect the cap, the floor is still filled: the cap bounds, it
    /// never starves.
    #[test]
    fn prune_should_fill_the_floor_through_a_hub_when_nothing_else_is_left() {
        let candidates = (1..=6u8)
            .map(|first| make_path_through(first, 9, 10 + u32::from(first)))
            .collect();
        let result = prune_for_consistency(candidates, 4, 2);
        assert_eq!(4, result.len(), "four of six through the only middle relayer there is");
    }

    #[test]
    fn distinct_first_relayers_counts_unique_first_hops() {
        // Three paths, two of them via the same relayer → two distinct first relayers. Paths with
        // no hops contribute nothing.
        let paths = vec![
            make_path_via(1, 10),
            make_path_via(1, 20),
            make_path_via(2, 30),
            make_path_with_latency(Some(40)), // empty path
        ];
        assert_eq!(2, distinct_first_relayers(&paths));
    }

    #[test]
    fn floor_zero_keeps_all_distinct_relayers_unlike_a_small_cap() {
        // Ten qualifying paths, each via a distinct relayer. This is the "if 10 qualify, use 10"
        // case: with the cap disabled (floor = 0) every relayer survives, so the return-path
        // degradation detector always has siblings to corroborate against. A small cap (2) throws
        // eight good relayers away — the collapse that made the 2026-08-28 outage undetectable.
        let candidates: Vec<_> = (1..=10u8).map(|i| make_path_via(i, i as u32 * 10)).collect();

        let uncapped = prune_for_consistency(candidates.clone(), 0, 1);
        assert_eq!(10, uncapped.len(), "floor = 0 disables pruning: all candidates survive");
        assert_eq!(10, distinct_first_relayers(&uncapped), "all ten relayers kept");

        let capped = prune_for_consistency(candidates, 2, 1);
        assert_eq!(2, capped.len(), "floor = 2 caps survivors at two");
        assert_eq!(2, distinct_first_relayers(&capped));
    }

    #[test]
    fn prune_should_fall_back_to_latency_order_once_relayers_are_exhausted() {
        // Only two relayers available but a floor of 4: the remaining slots go to the next-best
        // paths regardless of relayer.
        let candidates = vec![
            make_path_via(1, 10),
            make_path_via(2, 20),
            make_path_via(1, 30),
            make_path_via(2, 40),
            make_path_via(1, 50),
        ];

        let result = prune_for_consistency(candidates, 4, 1);
        assert_eq!(4, result.len());
        assert_eq!(
            vec![Some(10), Some(20), Some(30), Some(40)],
            result.iter().map(|p| p.total_latency_ms).collect::<Vec<_>>()
        );
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
            .map(|i| make_path_with_tickets(Some(i * 10), Some(1_000_000)))
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
            make_path_with_tickets(Some(10), Some(1_000)),
            make_path_with_tickets(Some(30), Some(1_000)),
            make_path_with_tickets(Some(20), Some(1_000)),
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
            make_path_with_tickets(Some(10), Some(1_000)),
            make_path_with_tickets(Some(20), Some(1_000)),
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
            .map(|i| make_path_with_tickets(Some(i * 10), Some(1_000)))
            .collect();
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn prune_0_hop_with_measured_latency_and_no_capacity_is_populated() {
        // 0-hop: fundable_tickets_floor = None is expected; path should be treated as "fully measured"
        // if latency is known.
        let mut candidates: Vec<_> = vec![
            make_path_with_tickets(Some(50), None), // 0-hop: no capacity, latency known
        ];
        candidates.extend((0..10).map(|_| make_path_with_latency(None)));
        let result = prune_for_consistency(candidates, 8, 0);
        assert_eq!(result.len(), 8);
        // The 0-hop path must be in the populated bucket and survive.
        let has_0_hop = result.iter().any(|p| p.total_latency_ms == Some(50));
        assert!(has_0_hop, "0-hop path with measured latency must survive pruning");
    }

    #[test]
    fn prune_multi_hop_without_fundable_tickets_floor_is_unpopulated() {
        // A 1-hop path with measured latency but NO capacity is unmeasured (unpopulated).
        // It should be demoted below paths that have both latency and capacity.
        let candidates: Vec<_> = vec![
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(50), Some(1_000)), // fully measured
            make_path_with_tickets(Some(40), None),        // missing capacity → unpopulated
        ];
        let result = prune_for_consistency(candidates, 8, 1);
        assert_eq!(result.len(), 8);
        // The path with missing capacity should not survive when all 8 slots are filled
        // by fully measured paths.
        let has_missing_balance = result.iter().any(|p| p.fundable_tickets_floor.is_none());
        assert!(
            !has_missing_balance,
            "path without capacity floor must be pruned when fully-measured paths fill the floor"
        );
    }

    #[test]
    fn prune_for_consistency_floor_zero_returns_all() {
        // floor == 0 must be treated as "no pruning" — all candidates survive unchanged.
        let candidates = vec![
            make_path_with_tickets(Some(10), Some(1_000)),
            make_path_with_tickets(Some(20), None),
            make_path_with_tickets(None, None),
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
                obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                    1000u64,
                ))));
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
            (100..=130).contains(&total),
            "expected ~120ms total latency, got {total}ms"
        );
        Ok(())
    }

    #[tokio::test]
    async fn path_metrics_fundable_tickets_floor_is_min() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let hop = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);

        // Stated, not assumed. A face value of one makes the ticket count equal the balance, which
        // is what keeps the numbers below readable — but it has to be pushed, because without a
        // price the floor is now `None` rather than silently dividing by one.
        graph.set_ticket_face_value(hopr_api::graph::traits::Balance::one());

        graph.upsert_edge(&me, &hop, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                500u64,
            ))));
        });
        graph.upsert_edge(&hop, &dest, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                200u64,
            ))));
        });
        graph.add_edge(&me, &hop).unwrap();
        graph.add_edge(&hop, &dest).unwrap();

        let selector = test_selector(me, graph, MAX_PATHS);
        let paths = selector.select_path(me, dest, 1).context("1-hop path")?;
        assert!(!paths.is_empty());
        assert_eq!(
            paths[0].fundable_tickets_floor,
            Some(200),
            "floor must be the smaller of 500 and 200"
        );
        Ok(())
    }

    #[tokio::test]
    async fn starved_first_hop_channel_must_not_be_selected() -> anyhow::Result<()> {
        // GNO-805: the client's own outgoing channel — the first, monetized hop — drains below one
        // ticket face value. The first-hop ticket can then no longer be created (hopr-ticket-manager
        // rejects it with "balance of channel is too low"), so path selection must not hand out that
        // route at all. Phase 1 (`forward`) rejects it correctly; the regression is Phase 2
        // (`forward_without_self_loopback`), which for a 1-hop target searches a single edge with
        // `length = 1` and so checks `balance_suffices(balance, 0, face)` — zero remaining hops
        // waives the funding check and re-admits the drained hop.
        let me = pubkey(&SECRET_0);
        let relay = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);

        // Face value 100; a monetized first edge needs >= one full-path winning ticket = 100.
        let face = hopr_api::graph::traits::Balance::from(100u64);

        let build = |first_hop_balance: u64| {
            let graph = ChannelGraph::new(me);
            graph.add_node(relay);
            graph.add_node(dest);
            graph.set_ticket_face_value(face);
            // me -> relay: the monetized first hop, drained or funded per the argument.
            graph.upsert_edge(&me, &relay, |obs| {
                obs.record(EdgeWeightType::Connected(true));
                obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                    first_hop_balance,
                ))));
            });
            // relay -> dest: healthy; the final hop is not monetized regardless.
            mark_edge_full(&graph, &relay, &dest);
            graph.add_edge(&me, &relay).unwrap();
            graph.add_edge(&relay, &dest).unwrap();
            graph
        };

        // Control: a funded first hop yields a usable path, proving the topology is otherwise valid
        // and that a later PathNotFound is caused by the drain, not a missing route.
        let funded = test_selector(me, build(10_000), MAX_PATHS);
        let healthy = funded.select_path(me, dest, 1).context("funded first hop must route")?;
        assert!(!healthy.is_empty(), "a funded first hop must be selectable");
        assert_eq!(healthy[0].path.first(), Some(&relay), "route must go via the relay");

        // The bug: first hop drained to 50 — below one face value (100), so it cannot fund a ticket.
        let starved = test_selector(me, build(50), MAX_PATHS);
        let result = starved.select_path(me, dest, 1);
        assert!(
            matches!(result, Err(PathPlannerError::Path(PathError::PathNotFound(..)))),
            "a first hop that cannot fund a single ticket must not be selected, got: {result:?}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn first_hop_funded_for_one_ticket_is_selectable() -> anyhow::Result<()> {
        // Regression for the removed balance headroom (hopr-api#144): the selector used to require a
        // first edge to hold twice the path's ticket cost (`MIN_BALANCE_HEADROOM = 2`), so a channel
        // with enough for one winning ticket but less than two was pruned despite being fundable. The
        // planner now requires only one full-path winning ticket, so such a channel must route.
        let me = pubkey(&SECRET_0);
        let relay = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);

        // Face value 100; first hop at 150 — above one winning ticket (100), below the old 2x = 200
        // headroom. Selectable now, pruned before.
        let face = hopr_api::graph::traits::Balance::from(100u64);
        let graph = ChannelGraph::new(me);
        graph.add_node(relay);
        graph.add_node(dest);
        graph.set_ticket_face_value(face);
        graph.upsert_edge(&me, &relay, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                150u64,
            ))));
        });
        mark_edge_full(&graph, &relay, &dest);
        graph.add_edge(&me, &relay).unwrap();
        graph.add_edge(&relay, &dest).unwrap();

        let selector = test_selector(me, graph, MAX_PATHS);
        let paths = selector
            .select_path(me, dest, 1)
            .context("a first hop funded for one winning ticket must route")?;
        assert!(
            !paths.is_empty(),
            "a first hop funded for one winning ticket must be selectable"
        );
        assert_eq!(paths[0].path.first(), Some(&relay), "route must go via the relay");
        Ok(())
    }

    #[tokio::test]
    async fn starved_intermediate_hop_via_extended_search_must_not_be_selected() -> anyhow::Result<()> {
        // Same off-by-one, one hop deeper. With no relay -> dest channel, a 2-hop request falls to
        // the Phase 2 extended search (me -> a -> b, dest appended). `b` is the last *searched*
        // edge but still a monetized hop. The understated length waived its funding via the
        // intermediate `require_funding` branch collapsing to zero remaining hops; costing against
        // the finished length keeps the check at one ticket.
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let dest = pubkey(&SECRET_3);

        let face = hopr_api::graph::traits::Balance::from(100u64);

        let build = |second_hop_balance: u64| {
            let graph = ChannelGraph::new(me);
            for n in [a, b, dest] {
                graph.add_node(n);
            }
            graph.set_ticket_face_value(face);
            // me -> a: healthy and well funded.
            graph.upsert_edge(&me, &a, |obs| {
                obs.record(EdgeWeightType::Connected(true));
                obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                    10_000u64,
                ))));
            });
            // a -> b: the monetized hop under test.
            graph.upsert_edge(&a, &b, |obs| {
                obs.record(EdgeWeightType::Connected(true));
                obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                    second_hop_balance,
                ))));
            });
            // No b -> dest edge: forces the Phase 2 extended search (last hop assumed by anybody).
            graph.add_edge(&me, &a).unwrap();
            graph.add_edge(&a, &b).unwrap();
            graph
        };

        // Control: a funded second hop routes via the extended search.
        let funded = test_selector(me, build(10_000), MAX_PATHS);
        let healthy = funded
            .select_path(me, dest, 2)
            .context("funded second hop must route")?;
        assert!(!healthy.is_empty(), "a funded second hop must be selectable");
        assert_eq!(healthy[0].path.first(), Some(&a), "route must start at the first relay");

        // The bug: second hop drained to 50 — below one face value (100).
        let starved = test_selector(me, build(50), MAX_PATHS);
        let result = starved.select_path(me, dest, 2);
        assert!(
            matches!(result, Err(PathPlannerError::Path(PathError::PathNotFound(..)))),
            "a monetized intermediate hop that cannot fund a ticket must not be selected, got: {result:?}"
        );
        Ok(())
    }

    // NOTE: a test for fundable_tickets_floor=None is omitted intentionally.
    // The forward cost function requires a funding balance on all non-last edges, so
    // every path the selector returns has capacity data on at least one edge, making
    // fundable_tickets_floor always Some for reachable paths.

    /// `relayers` relayers in a full mesh between `me` and `dest`, every link in both directions.
    ///
    /// `tickets` gives each link's channel balance in tickets from a scrambled link number, so a
    /// test can make an edge's value depend on its position in the path (the tickets a channel
    /// must fund are the hops that follow it).
    fn mesh_graph(
        relayers: usize,
        tickets: impl Fn(u64) -> u64,
    ) -> (
        OffchainPublicKey,
        OffchainPublicKey,
        Vec<OffchainPublicKey>,
        ChannelGraph,
    ) {
        const FACE_VALUE: u64 = 100;
        let me = pubkey(&SECRET_0);
        let dest = pubkey(&SECRET_1);
        let relays = (0..relayers)
            .map(|_| *OffchainKeypair::random().public())
            .collect::<Vec<_>>();
        let graph = ChannelGraph::new(me);
        graph.set_ticket_face_value(hopr_api::graph::traits::Balance::from(FACE_VALUE));
        graph.add_node(dest);
        relays.iter().for_each(|r| graph.add_node(*r));

        let mut n = 0u64;
        let mut link = |a: &OffchainPublicKey, b: &OffchainPublicKey| {
            n += 1;
            graph.add_edge(a, b).unwrap();
            graph.upsert_edge(a, b, |obs| {
                obs.record(EdgeWeightType::Connected(true));
                obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(10 + n % 7 * 13))));
                obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(20 + n % 5 * 17))));
                obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                    tickets(n.wrapping_mul(2_654_435_761) >> 7) * FACE_VALUE,
                ))));
            });
        };
        for r in &relays {
            link(&me, r);
            link(r, &me);
            link(&dest, r);
            link(r, &dest);
            for s in relays.iter().filter(|s| *s != r) {
                link(r, s);
            }
        }
        (me, dest, relays, graph)
    }

    fn metrics_fn(
        graph: &ChannelGraph,
        length: std::num::NonZeroUsize,
        forward: bool,
    ) -> MetricsValueFn<<ChannelGraph as NetworkGraphTraverse>::Observed> {
        let cfg = PathPlannerConfig::default();
        let face = graph.ticket_face_value();
        MetricsValueFn {
            inner: if forward {
                EdgeValueFn::forward(length, cfg.edge_penalty, cfg.min_ack_rate, face)
            } else {
                EdgeValueFn::returning(length, cfg.edge_penalty, cfg.min_ack_rate, face)
            },
            ticket_face_value: face,
        }
    }

    fn first_relayers(paths: &[PathWithMetrics]) -> std::collections::HashSet<OffchainPublicKey> {
        paths.iter().filter_map(|p| p.path.first().copied()).collect()
    }

    /// The defect: a three-hop search over seven relayers kept 50 paths that named two first
    /// relayers, the depth-first order filling the pool with whole subtrees.
    #[rstest::rstest]
    #[case::forward(true)]
    #[case::returning(false)]
    fn a_spread_pool_should_name_every_first_relayer_where_one_traversal_named_two(
        #[case] forward: bool,
    ) -> anyhow::Result<()> {
        let (me, dest, relays, graph) = mesh_graph(7, |_| 1000);
        let (src, dst) = if forward { (me, dest) } else { (dest, me) };
        let length = std::num::NonZeroUsize::new(4).context("non-zero")?;

        let single = compute_paths(&graph, &src, &dst, length, 50, metrics_fn(&graph, length, forward));
        assert!(
            first_relayers(&single).len() <= 2,
            "30 paths per first relayer, cut at 50"
        );

        let spread = compute_paths_spread(&graph, &src, &dst, length, 50, metrics_fn(&graph, length, forward));
        assert!(spread.len() <= 50);
        assert_eq!(relays.len(), first_relayers(&spread).len());
        for p in &spread {
            assert_eq!(4, p.path.len());
            assert_eq!(Some(&dst), p.path.last());
            assert!(
                !p.path.contains(&src),
                "a path must not run back through its source: {:?}",
                p.path
            );
            let distinct = p.path.iter().collect::<std::collections::HashSet<_>>();
            assert_eq!(p.path.len(), distinct.len(), "a path must be simple: {:?}", p.path);
        }
        Ok(())
    }

    /// Every path the spread search returns is one the single traversal returns, valued the same:
    /// a channel must fund the hops after it, so an edge valued at the wrong position would pass
    /// or fail the funding check differently, and the two sets would differ.
    #[rstest::rstest]
    #[case::forward(true)]
    #[case::returning(false)]
    fn a_spread_pool_should_value_each_path_as_one_traversal_does(#[case] forward: bool) -> anyhow::Result<()> {
        let (me, dest, _, graph) = mesh_graph(5, |n| 1 + n % 4);
        let (src, dst) = if forward { (me, dest) } else { (dest, me) };

        for hops in 1..=3 {
            let length = std::num::NonZeroUsize::new(hops + 1).context("non-zero")?;
            let key = |p: &PathWithMetrics| {
                format!(
                    "{:?} {} {:?} {:?} {:?} {:?}",
                    p.path,
                    p.cost,
                    p.total_latency_ms,
                    p.min_probe_success_rate,
                    p.min_ack_rate,
                    p.fundable_tickets_floor
                )
            };
            let mut all = compute_paths(
                &graph,
                &src,
                &dst,
                length,
                usize::MAX,
                metrics_fn(&graph, length, forward),
            )
            .iter()
            .map(key)
            .collect::<Vec<_>>();
            let mut spread =
                compute_paths_spread(&graph, &src, &dst, length, 10_000, metrics_fn(&graph, length, forward))
                    .iter()
                    .map(key)
                    .collect::<Vec<_>>();
            all.sort();
            spread.sort();
            assert!(!all.is_empty(), "{hops} hops: the mesh must route");
            // From two hops on, the first channel must fund more tickets than some channels hold.
            let simple_paths = (5 - hops + 1..=5).product::<usize>();
            if hops > 1 {
                assert!(all.len() < simple_paths, "{hops} hops: funding must reject some paths");
            }
            assert_eq!(all, spread, "{hops} hops");
        }
        Ok(())
    }

    /// Fewer candidates than first relayers still take one path from each relayer they can.
    #[test]
    fn a_spread_pool_cut_short_should_keep_distinct_first_relayers() -> anyhow::Result<()> {
        let (me, dest, _, graph) = mesh_graph(7, |_| 1000);
        let length = std::num::NonZeroUsize::new(3).context("non-zero")?;

        let spread = compute_paths_spread(&graph, &dest, &me, length, 4, metrics_fn(&graph, length, false));
        assert_eq!(4, spread.len());
        assert_eq!(4, first_relayers(&spread).len());
        Ok(())
    }

    /// A first relayer's continuations can run back through the source, which the single traversal
    /// never returns, and they must not take the place of those that do not. Every relayer here links
    /// back to the source, so each first relayer's three-hop subtree holds 6 continuations through it
    /// beside its 30 good ones; each share is smaller than either.
    #[rstest::rstest]
    #[case::forward(true)]
    #[case::returning(false)]
    fn a_spread_pool_should_fill_each_share_with_paths_that_avoid_the_source(
        #[case] forward: bool,
    ) -> anyhow::Result<()> {
        let (me, dest, relays, graph) = mesh_graph(7, |_| 1000);
        let (src, dst) = if forward { (me, dest) } else { (dest, me) };
        let length = std::num::NonZeroUsize::new(4).context("non-zero")?;

        let per_relayer = 2;
        let take = per_relayer * relays.len();
        let spread = compute_paths_spread(&graph, &src, &dst, length, take, metrics_fn(&graph, length, forward));
        assert!(
            spread.iter().all(|p| !p.path.contains(&src)),
            "a path must not run back through its source"
        );
        assert_eq!(
            take,
            spread.len(),
            "every share filled with paths that avoid the source"
        );
        for relayer in &relays {
            assert_eq!(
                per_relayer,
                spread.iter().filter(|p| p.path.first() == Some(relayer)).count(),
                "each first relayer keeps its whole share"
            );
        }

        // Fewer candidates than first relayers: one from each relayer the pool has room for.
        let cut_short = compute_paths_spread(&graph, &src, &dst, length, 4, metrics_fn(&graph, length, forward));
        assert_eq!(4, cut_short.len());
        assert_eq!(4, first_relayers(&cut_short).len());
        Ok(())
    }

    /// The selector searches both directions spread over first relayers (pruning off, so the
    /// candidates it found are what it returns).
    #[tokio::test]
    async fn a_three_hop_pool_should_name_every_first_relayer_in_both_directions() -> anyhow::Result<()> {
        let (me, dest, relays, graph) = mesh_graph(7, |_| 1000);
        let cfg = PathPlannerConfig::default();
        let selector =
            HoprGraphPathSelector::new(me, graph, cfg.max_cached_paths, cfg.edge_penalty, cfg.min_ack_rate, 0);

        let forward = selector.select_path(me, dest, 3).context("forward pool")?;
        assert_eq!(relays.len(), distinct_first_relayers(&forward));
        let returning = selector.select_path(dest, me, 3).context("return pool")?;
        assert_eq!(relays.len(), distinct_first_relayers(&returning));
        Ok(())
    }
}
