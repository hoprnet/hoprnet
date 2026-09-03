use std::{sync::Arc, time::Duration};

use futures::{StreamExt as _, TryStreamExt, stream::FuturesUnordered};
#[cfg(all(feature = "telemetry", not(test)))]
use hopr_api::types::internal::path::Path;
use hopr_api::{
    OffchainPublicKey,
    chain::{ChainKeyOperations, ChainPathResolver, ChainReadChannelOperations},
    types::{
        crypto::crypto_traits::Randomizable,
        internal::{errors::PathError, prelude::*},
        primitive::traits::ToHex,
    },
};
use hopr_crypto_packet::prelude::*;
use hopr_protocol_hopr::{FoundSurb, SurbStore};
use tracing::trace;
use validator::{Validate, ValidationError};

use super::{
    errors::{PathPlannerError, Result},
    traits::{BackgroundPathCacheRefreshable, PathSelector, PathWithMetrics},
};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PATH_LENGTH: hopr_api::types::telemetry::SimpleHistogram = hopr_api::types::telemetry::SimpleHistogram::new(
        "hopr_path_length",
        "Distribution of number of hops of sent messages",
        vec![0.0, 1.0, 2.0, 3.0, 4.0]
    ).unwrap();
}

/// Rejects a temper exponent outside `(0, 1]`.
///
/// `w^γ` is only a flattening of the weights for `γ ∈ (0, 1]`.
///
/// `γ > 1` would sharpen instead — concentrating harder than raw path value, the opposite of the
/// knob's purpose — and `γ <= 0` inverts or annihilates the ordering, sending traffic *preferentially*
/// to the worst relays. Both are almost certainly typos, so refuse them rather than silently
/// degrade every return path the node builds.
fn validate_weight_temper(temper: f64) -> std::result::Result<(), ValidationError> {
    if temper > 0.0 && temper <= 1.0 {
        return Ok(());
    }
    let mut err = ValidationError::new("weight_temper_out_of_range");
    err.message = Some(
        format!(
            "return_path_weight_temper ({temper}) must be in (0, 1]: 1.0 samples by raw path value, smaller values \
             flatten towards uniform"
        )
        .into(),
    );
    Err(err)
}

/// Configuration for [`PathPlanner`]'s internal path cache.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate)]
pub struct PathPlannerConfig {
    /// Maximum number of `(source, destination, options)` entries in the path cache.
    #[default = 10_000]
    pub max_cache_capacity: u64,
    /// Time-to-live for a cached path list.  When an entry expires the next
    /// [`PathPlanner::resolve_routing`] call transparently recomputes it (lazy refresh).
    ///
    /// Candidate weights are computed once, when the entry is filled, and frozen in the
    /// [`hopr_utils::statistics::WeightedCollection`] -- so this bounds how stale the *weights* a
    /// live session draws from can be, not merely how stale the candidate set is. At the previous
    /// 60 s a relay that stopped delivering kept its full share of return-path draws for a minute
    /// after the graph had already scored it down.
    #[default(Duration::from_secs(10))]
    pub cache_ttl: Duration,
    /// Period between proactive background cache-refresh sweeps.
    ///
    /// Held at half the TTL so a steady-state session is normally served from an entry that was
    /// re-weighted rather than one that expired under it.
    #[default(Duration::from_secs(5))]
    pub refresh_period: Duration,
    /// Maximum number of candidate paths the selector may return per query.
    /// All returned candidates are validated and cached.
    #[default = 50]
    pub max_cached_paths: usize,
    /// Penalty multiplier for edges lacking probe-based quality observations.
    /// Applied during path cost evaluation to down-weight unprobed edges.
    /// Must be finite and in `0.0..=1.0`.
    #[default = 0.5]
    #[validate(custom(function = "validate_unit_interval"))]
    pub edge_penalty: f64,
    /// Minimum acceptable message acknowledgment rate for path selection.
    /// Edges with an ack rate below this threshold are excluded from candidate paths.
    /// Must be finite and in `0.0..=1.0`.
    #[default = 0.1]
    #[validate(custom(function = "validate_unit_interval"))]
    pub min_ack_rate: f64,
    /// Candidate count below which no latency-based pruning occurs.
    ///
    /// When fewer paths than this value are found, the selector returns all of them
    /// unchanged (`min(found_count, floor)` semantics — the floor is never a minimum
    /// to fabricate).  Set to 0 to disable pruning entirely.
    #[default = 8]
    pub min_paths_anonymity_floor: usize,
    /// Total path latency at which the latency factor in the composite weight equals 0.5.
    /// Higher values make the weight less sensitive to latency differences.
    #[default(Duration::from_millis(100))]
    pub latency_halflife: Duration,
    /// Reference channel balance used to scale the capacity factor in the composite weight.
    /// `capacity_factor` saturates at 1.0 near this value.
    /// Defaults to 10_000_000 (~10 MiB in wxHOPR tokens).
    #[default = 10_000_000]
    pub capacity_reference: u128,
    /// Exponent applied to return-path weights before sampling, flattening the distribution.
    ///
    /// Return paths are drawn weighted-random by path value, which concentrates a session's SURBs
    /// on the few highest-valued relays — losing one then costs far more than the reliable-mode
    /// loss tolerance. Raising each weight to `γ ∈ (0, 1]` compresses the spread between good and
    /// bad candidates without changing their order: `w' = w^γ`.
    ///
    /// `1.0` samples by raw path value (most traffic on the best relays, largest blast radius when
    /// one dies). Values approaching `0.0` tend to a uniform draw (smallest blast radius, most
    /// traffic on poor relays). With weights `(0.4, 0.3, 0.2, 0.1)`, `γ = 0.5` moves the busiest
    /// relay's share from 40% to 33% and the ratio between busiest and least-busy from 4.0 to 2.0.
    ///
    /// Defaults to 0.5.
    #[validate(custom(function = "validate_weight_temper"))]
    #[default = 0.5]
    pub return_path_weight_temper: f64,
    /// Fraction of return-path draws made uniformly at random instead of by weight.
    ///
    /// Weights come from observations, and observations only exist for paths that get selected — a
    /// closed loop in which a path that falls out of favour stops being measured, so its score can
    /// never recover and it is never chosen again. Spending a small share of draws uniformly keeps
    /// every candidate under observation, which is what lets a recovered one climb back.
    ///
    /// Costs throughput in proportion: this share of return paths deliberately ignores which
    /// candidate looks best. Candidates have already passed the selector's gates (open channels,
    /// `min_ack_rate`, and so on) before reaching here, so an exploratory draw is random only with
    /// respect to *quality*, never a route the cost function rejected. `0.0` disables it.
    /// Defaults to 0.1.
    #[validate(custom(function = "validate_unit_interval"))]
    #[default = 0.1]
    pub return_path_exploration: f64,
    /// Upper bound on a loopback probe's round-trip time considered plausible.
    /// Measurements above this cap (clock skew, stale telemetry) are discarded
    /// instead of poisoning the latency EMA with an absurd value.
    #[default(Duration::from_secs(30))]
    pub max_plausible_loopback_rtt: Duration,
}

fn validate_unit_interval(value: f64) -> std::result::Result<(), ValidationError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::new("value must be finite and in 0.0..=1.0"))
    }
}

/// Parameters that shape how per-path aggregates modulate the `WeightedCollection` weight.
#[derive(Debug, Clone, Copy)]
struct WeightingParams {
    latency_halflife: Duration,
    capacity_reference: u128,
}

/// Continuous, monotonically decreasing latency factor, bounded in (0, 1].
///
/// Returns 1.0 for zero latency and 0.5 when `latency == halflife`.
fn latency_factor(latency: Duration, halflife: Duration) -> f64 {
    let ms = latency.as_millis() as f64;
    let h = halflife.as_millis().max(1) as f64;
    1.0 / (1.0 + ms / h)
}

/// Continuous, monotonically increasing capacity factor, bounded in (0.05, 1.0].
///
/// Uses a log scale because channel balances span many orders of magnitude.
/// Saturates at 1.0 near `reference`.
fn capacity_factor(c: u128, reference: u128) -> f64 {
    let log = (c as f64).max(1.0).log10();
    let ref_log = (reference as f64).max(10.0).log10();
    (log / ref_log).clamp(0.05, 1.0)
}

/// Composite selection weight for a candidate path.
///
/// Refines `pwc.cost` (the `EdgeValueFn` output) with latency and capacity factors
/// derived from the per-path aggregates.  Factors are neutral (1.0) when the
/// corresponding aggregate is unavailable to avoid penalising unprobed paths.
/// For 0-hop routes (`hops == 0`) the capacity factor is always 1.0 — direct
/// `me -> dest` packets use no payment channel, so `fundable_tickets_floor = None` is expected.
fn composite_weight(pwc: &PathWithMetrics, hops: usize, params: WeightingParams) -> f64 {
    let lat = pwc
        .total_latency_ms
        .map(|ms| latency_factor(Duration::from_millis(ms as u64), params.latency_halflife))
        .unwrap_or(1.0);
    let cap = if hops == 0 {
        1.0
    } else {
        pwc.fundable_tickets_floor
            .map(|c| capacity_factor(c, params.capacity_reference))
            .unwrap_or(1.0)
    };
    pwc.cost * lat * cap
}

/// Picks an index into `weights` with probability proportional to the weight; `None` if all are
/// non-positive.
///
/// Mirrors `WeightedCollection::pick_index`, which cannot be reused because the callers below
/// select over *subsets* of a single collection.
fn pick_weighted_index(weights: &[f64]) -> Option<usize> {
    let total: f64 = weights.iter().map(|w| w.max(0.0)).sum();
    if total <= 0.0 {
        return None;
    }

    // Path selection is privacy-relevant, so draw from the CSPRNG rather than a thread RNG.
    let r = hopr_api::types::crypto_random::random_float_in_range(0.0..total);
    let mut cumulative = 0.0;
    for (i, w) in weights.iter().enumerate() {
        cumulative += w.max(0.0);
        if r < cumulative {
            return Some(i);
        }
    }

    // Floating-point edge case: fall back to the last positive-weight entry.
    weights.iter().rposition(|w| *w > 0.0)
}

/// Whether this draw should explore — ignore the weights and pick uniformly.
///
/// Weights are derived from observations, and observations only exist for paths that were selected.
/// Left alone that is a closed loop: a path that falls out of favour stops being measured, so its
/// score can never recover and it is never chosen again. Spending a small share of draws uniformly
/// keeps every candidate under observation, which is what lets one that has recovered climb back.
fn should_explore(exploration: f64) -> bool {
    exploration > 0.0 && hopr_api::types::crypto_random::random_float_in_range(0.0..1.0) < exploration
}

/// Picks a uniformly random index over `len` entries.
fn pick_uniform_index(len: usize) -> Option<usize> {
    (len > 0).then(|| (hopr_api::types::crypto_random::random_float_in_range(0.0..len as f64) as usize).min(len - 1))
}

/// Flattens `weights` by raising each to `temper`, compressing the spread between good and bad
/// candidates without reordering them.
///
/// `x^γ` is monotone for `γ > 0`, so the best candidate stays the best — only the *ratio* between
/// them shrinks, which is what bounds how much of a session rides on any single relayer. Weights
/// are clamped at zero first: a negative weight would make `powf` return NaN and poison the draw.
fn temper_weights(weights: &[f64], temper: f64) -> Vec<f64> {
    weights.iter().map(|w| w.max(0.0).powf(temper)).collect()
}

/// Rebuilds the weighted candidate collection for one `(source, destination, hops)` triple from
/// whatever the graph currently says.
///
/// Shared by the lazy cache fill, the background sweep and the on-demand recompute, so all three
/// necessarily agree: a divergence here would make a session's weights depend on which of the three
/// happened to run last.
///
/// `Ok(None)` means the selector offered nothing, or nothing survived validation. Callers decide
/// what that means — a fill turns it into `PathNotFound`, a refresh leaves the existing entry
/// alone. `Err` is reserved for a selector that actually failed.
#[allow(clippy::too_many_arguments)]
async fn rebuild_candidates<R, S>(
    resolver: &R,
    selector: &S,
    weighting: WeightingParams,
    me: OffchainPublicKey,
    src_key: OffchainPublicKey,
    dest_key: OffchainPublicKey,
    hops: usize,
    kind: &'static str,
) -> Result<Option<hopr_utils::statistics::WeightedCollection<ValidatedPath>>>
where
    R: ChainKeyOperations + ChainReadChannelOperations + Send + Sync,
    S: PathSelector,
{
    let candidates = selector.select_path(src_key, dest_key, hops)?;

    let chain_resolver = ChainPathResolver::from(resolver);
    let mut valid_paths: Vec<(ValidatedPath, f64)> = Vec::with_capacity(candidates.len());
    let mut path_metrics: Vec<PathWithMetrics> = Vec::with_capacity(candidates.len());
    for mut pwc in candidates {
        let path_nodes = std::mem::take(&mut pwc.path);
        let node_ids: Vec<NodeId> = path_nodes.into_iter().map(NodeId::Offchain).collect::<Vec<_>>();
        match ValidatedPath::new(NodeId::Offchain(src_key), node_ids, &chain_resolver).await {
            Ok(vp) => {
                valid_paths.push((vp, composite_weight(&pwc, hops, weighting)));
                path_metrics.push(pwc);
            }
            Err(e) => tracing::debug!(kind, error = %e, "path candidate failed validation"),
        }
    }

    // A return path that resolves to a single relayer is the blind spot the degradation detector
    // cannot escape: with no sibling relayer to the same destination, sustained silence is
    // indistinguishable from a quiet peer, so a dead return relayer is never attributed and never
    // re-planned. Checked here, after validation, rather than in the selector: validation can reject
    // every candidate through one of several relayers the selector saw, so only the survivors here
    // reflect what a session can actually draw from.
    if src_key != me && hops > 0 && !path_metrics.is_empty() {
        let relayers = super::selector::distinct_first_relayers(&path_metrics);
        if relayers <= 1 {
            tracing::warn!(
                src = %src_key,
                kind,
                distinct_relayers = relayers,
                candidates = path_metrics.len(),
                "return-path relayer diversity collapsed to a single relayer; degradation detection \
                 cannot corroborate a dead relayer for this destination",
            );
        }
    }

    if valid_paths.is_empty() {
        return Ok(None);
    }

    let weighted = hopr_utils::statistics::WeightedCollection::new(valid_paths);
    let total_wt = weighted.total_weight();
    for ((vp, w), pwm) in weighted.iter().zip(path_metrics.iter()) {
        tracing::debug!(
            kind,
            destination = %dest_key,
            hops,
            path = %vp,
            cost = pwm.cost,
            composite_weight = w,
            sampling_probability = if total_wt > 0.0 && *w > 0.0 { *w / total_wt } else { 0.0 },
            total_latency_ms = ?pwm.total_latency_ms,
            min_probe_success_rate = ?pwm.min_probe_success_rate,
            min_ack_rate = ?pwm.min_ack_rate,
            fundable_tickets_floor = ?pwm.fundable_tickets_floor,
            "weighted candidate path",
        );
    }
    Ok(Some(weighted))
}

/// Smallest change in a candidate's share of the draws that counts as traffic having moved.
///
/// Weights are recomputed from live observations, so they jitter constantly without meaning
/// anything. One percentage point of a candidate's share is well below the shift a relayer going
/// silent produces (measured: 33% to near zero) and well above the noise of an idle graph.
const MIN_SHARE_SHIFT: f64 = 0.01;

/// Each candidate's share of the total weight, keyed by the route it takes.
fn shares_by_route(paths: &hopr_utils::statistics::WeightedCollection<ValidatedPath>) -> Vec<(String, f64)> {
    let total: f64 = paths.iter().map(|(_, w)| w.max(0.0)).sum();
    paths
        .iter()
        .map(|(vp, w)| {
            let share = if total > 0.0 { w.max(0.0) / total } else { 0.0 };
            (vp.to_string(), share)
        })
        .collect()
}

/// Whether re-weighting would send a materially different share of the draws somewhere else.
///
/// Shares rather than raw weights, because the draw normalises over the collection: every weight
/// halving changes nothing about where traffic goes. Callers use this to decide whether a re-plan
/// achieved anything, and a re-plan that moved nothing is a reason *not* to act on it -- so
/// answering "yes" by default would defeat the check it exists for.
fn weights_moved(
    before: &hopr_utils::statistics::WeightedCollection<ValidatedPath>,
    after: &hopr_utils::statistics::WeightedCollection<ValidatedPath>,
) -> bool {
    let (before, after) = (shares_by_route(before), shares_by_route(after));
    if before.len() != after.len() {
        return true;
    }
    before.iter().any(|(route, was)| {
        // A route that vanished has lost its whole share, which is the largest move there is.
        after
            .iter()
            .find(|(other, _)| other == route)
            .is_none_or(|(_, now)| (now - was).abs() >= MIN_SHARE_SHIFT)
    })
}

/// Cache key for the path planner: `(source, destination, hops)`.
///
/// Only the `Hops` variant of [`RoutingOptions`] is cached (explicit intermediate
/// paths bypass the cache), so the key stores the hop count as a plain `u32`.
///
/// Keyed on resolved offchain keys rather than [`NodeId`], because a `NodeId` naming a node by its
/// chain address is never equal to one naming the same node by its packet key. Callers hold
/// whichever form their layer happens to use -- Sessions carry chain addresses, the SURB telemetry
/// reports packet keys -- so a raw-`NodeId` key silently stores the same route twice. Resolving
/// first makes lookup and insertion agree by construction.
type PlannerCacheKey = (OffchainPublicKey, OffchainPublicKey, u32);
type PlannerCacheValue = Arc<hopr_utils::statistics::WeightedCollection<ValidatedPath>>;

/// Path planner that resolves [`DestinationRouting`] to [`ResolvedTransportRouting`].
///
/// The planner delegates path *discovery* to any [`PathSelector`] implementation and
/// owns the `moka` cache of fully-validated [`ValidatedPath`] objects paired with
/// their traversal cost, keyed by `(source: NodeId, destination: NodeId, hops: u32)`.
///
/// On a cache miss the planner calls the selector, validates every candidate against
/// the chain resolver, and stores an `Arc<WeightedCollection<ValidatedPath>>` in the
/// cache. On a cache hit a candidate is picked via weighted random selection (higher
/// cost = higher quality = higher probability).
///
/// A background sweep (`background_refresh`) can be spawned to
/// proactively re-warm the cache for all previously-seen keys.
#[derive(Clone)]
pub struct PathPlanner<Surb, R, S> {
    me: OffchainPublicKey,
    pub surb_store: Surb,
    resolver: Arc<R>,
    selector: Arc<S>,
    cache: moka::future::Cache<PlannerCacheKey, PlannerCacheValue>,
    refresh_period: Duration,
    weighting: WeightingParams,
    return_path_weight_temper: f64,
    return_path_exploration: f64,
}

impl<Surb, R, S> PathPlanner<Surb, R, S>
where
    Surb: SurbStore + Send + Sync + 'static,
    R: ChainKeyOperations + ChainReadChannelOperations + Send + Sync + 'static,
    S: PathSelector + Send + Sync + 'static,
{
    /// Create a new path planner.
    ///
    /// `me` is this node's [`OffchainPublicKey`]; it is used as the source in path queries.
    pub fn new(me: OffchainPublicKey, surb_store: Surb, resolver: R, selector: S, config: PathPlannerConfig) -> Self {
        let cache = moka::future::Cache::builder()
            .max_capacity(config.max_cache_capacity)
            .time_to_live(config.cache_ttl)
            .build();

        Self {
            me,
            surb_store,
            resolver: Arc::new(resolver),
            selector: Arc::new(selector),
            cache,
            refresh_period: config.refresh_period,
            weighting: WeightingParams {
                latency_halflife: config.latency_halflife,
                capacity_reference: config.capacity_reference,
            },
            return_path_weight_temper: config.return_path_weight_temper,
            return_path_exploration: config.return_path_exploration,
        }
    }

    /// Resolve a [`NodeId`] to an [`OffchainPublicKey`].
    async fn resolve_node_id_to_offchain_key(&self, node_id: &NodeId) -> Result<OffchainPublicKey> {
        match node_id {
            NodeId::Offchain(key) => Ok(*key),
            NodeId::Chain(addr) => {
                let resolver = ChainPathResolver::from(&*self.resolver);
                resolver
                    .resolve_transport_address(addr)
                    .await
                    .map_err(|e| PathPlannerError::Other(anyhow::anyhow!("{e}")))?
                    .ok_or_else(|| {
                        PathPlannerError::Other(anyhow::anyhow!("no offchain key found for chain address {addr}"))
                    })
            }
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn resolve_path(
        &self,
        source: NodeId,
        destination: NodeId,
        options: RoutingOptions,
    ) -> Result<ValidatedPath> {
        let path = match options {
            RoutingOptions::IntermediatePath(explicit_path) => {
                tracing::debug!(
                    direction = "loopback",
                    ?source,
                    ?destination,
                    ?explicit_path,
                    "resolving intermediate path"
                );
                let resolver = ChainPathResolver::from(&*self.resolver);
                ValidatedPath::new(
                    source,
                    explicit_path
                        .into_iter()
                        .chain(std::iter::once(destination))
                        .collect::<Vec<_>>(),
                    &resolver,
                )
                .await?
            }

            RoutingOptions::Hops(hops) if u32::from(hops) == 0 => {
                trace!(hops = 0, "resolving zero-hop direct path");
                let resolver = ChainPathResolver::from(&*self.resolver);
                ValidatedPath::new(source, vec![destination], &resolver).await?
            }

            RoutingOptions::Hops(hops) => {
                let hops_usize: usize = hops.into();
                let paths = self.cached_paths(source, destination, hops).await?;

                // Format from the `NodeId`s rather than re-resolving them: `cached_paths` has
                // already done that, and for `NodeId::Chain` each resolution is a resolver lookup.
                paths.pick_one().ok_or_else(|| {
                    PathPlannerError::Path(PathError::PathNotFound(
                        hops_usize,
                        source.to_string(),
                        destination.to_string(),
                    ))
                })?
            }
        };

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            hopr_api::types::telemetry::SimpleHistogram::observe(&METRIC_PATH_LENGTH, (path.num_hops() - 1) as f64);
        }

        trace!(%path, "validated resolved path");
        Ok(path)
    }

    /// Cached weighted collection of validated `hops`-hop paths from `source` to `destination`,
    /// computed on a miss.
    ///
    /// Single-path callers draw with `pick_one`; batch callers (see
    /// [`PathPlanner::resolve_diverse_return_paths`]) work over the collection directly, paying one
    /// cache lookup instead of one per path.
    #[tracing::instrument(level = "trace", skip(self))]
    async fn cached_paths(
        &self,
        source: NodeId,
        destination: NodeId,
        hops: hopr_api::types::primitive::bounded::BoundedSize<{ RoutingOptions::MAX_INTERMEDIATE_HOPS }>,
    ) -> Result<PlannerCacheValue> {
        let hops_usize: usize = hops.into();
        trace!(hops = hops_usize, "resolving path via planner cache");

        let src_key = self.resolve_node_id_to_offchain_key(&source).await?;
        let dest_key = self.resolve_node_id_to_offchain_key(&destination).await?;

        let cache_key: PlannerCacheKey = (src_key, dest_key, u32::from(hops));

        let resolver = self.resolver.clone();
        let selector = self.selector.clone();
        let weighting = self.weighting;
        let me = self.me;

        self.cache
            .try_get_with(cache_key, async move {
                trace!(hops = hops_usize, "path cache miss, querying selector");
                rebuild_candidates(
                    &*resolver, &*selector, weighting, me, src_key, dest_key, hops_usize, "fill",
                )
                .await?
                .map(Arc::new)
                .ok_or_else(|| {
                    PathPlannerError::Path(PathError::PathNotFound(hops_usize, src_key.to_hex(), dest_key.to_hex()))
                })
            })
            .await
            .map_err(PathPlannerError::CacheError)
    }

    /// Rebuilds every cached entry whose paths originate at `source`, replacing each in place.
    ///
    /// Return paths are cached under `(counterparty, me, hops)`, so this is how a caller that has
    /// just learned a counterparty's return traffic went silent forces those weights to be rebuilt
    /// from the current graph, instead of waiting up to
    /// [`PathPlannerConfig::refresh_period`] for the background sweep to reach them.
    ///
    /// Entries are replaced, never dropped: one that rebuilds to nothing keeps serving what it
    /// already holds. Dropping them was measured to collapse a healthy session from 100 % to
    /// 0.14 %, because a live session draws its next return path from this very entry.
    ///
    /// Returns how many entries came back with a materially different **share** of the draws --
    /// not how many were rebuilt. Callers use this to decide whether the re-plan achieved anything
    /// worth acting on, and on a healthy graph a rebuild always succeeds, so counting rebuilds
    /// would answer "something moved" every time.
    pub async fn recompute_paths_from(&self, source: &OffchainPublicKey) -> usize {
        // 0-hop entries name a direct route with nothing to re-weight, exactly as in the sweep.
        let keys = self
            .cache
            .iter()
            .map(|(key, _)| *key.as_ref())
            .filter(|(src, _, hops)| src == source && *hops > 0)
            .collect::<Vec<PlannerCacheKey>>();

        let mut moved = 0usize;
        for (src_key, dest_key, hops) in keys {
            if let Ok(Some(weighted)) = rebuild_candidates(
                &*self.resolver,
                &*self.selector,
                self.weighting,
                self.me,
                src_key,
                dest_key,
                hops as usize,
                "recompute",
            )
            .await
            {
                let key = (src_key, dest_key, hops);
                // A fresh entry counts as moved -- there was no previous distribution to compare
                // against, so nothing here can say the traffic stayed put.
                let shifted = match self.cache.get(&key).await {
                    Some(previous) => weights_moved(&previous, &weighted),
                    None => true,
                };
                self.cache.insert(key, Arc::new(weighted)).await;
                if shifted {
                    moved += 1;
                }
            }
        }

        tracing::debug!(%source, moved, "recomputed cached paths originating at peer");
        moved
    }

    /// Resolves `count` return paths from `destination` back to this node, drawn weighted-random
    /// over [`PathPlannerConfig::return_path_weight_temper`]-flattened path values.
    ///
    /// Tempering is what bounds the blast radius of losing one relay: raw path values concentrate a
    /// session's SURBs on the few best candidates, so the flatter the effective distribution, the
    /// smaller any single relay's share of the return stream.
    ///
    /// Draws are independent — deliberately. Spreading a *batch* over K distinct relayers was tried
    /// and cannot work: a batch is the SURBs that fit in one packet, and `HoprPacket::PAYLOAD_SIZE /
    /// HoprSurb::SIZE` is 2, so K was capped at 2 whatever the configuration said, while each packet
    /// re-drew independently regardless. Tempering has no such ceiling.
    ///
    /// Only `Hops` routing has alternatives to draw over — an explicit path resolves to itself, so
    /// those fall back to plain repeated resolution.
    #[tracing::instrument(level = "trace", skip(self))]
    async fn resolve_diverse_return_paths(
        &self,
        destination: NodeId,
        options: RoutingOptions,
        count: usize,
    ) -> Result<Vec<ValidatedPath>> {
        // No return paths requested (e.g. the message fills the payload, leaving no room for
        // SURBs). Resolve nothing — querying the planner here would turn "none wanted" into a
        // `PathNotFound` error.
        if count == 0 {
            return Ok(Vec::new());
        }

        let me = NodeId::Offchain(self.me);

        let hops = match options {
            RoutingOptions::Hops(hops) if u32::from(hops) > 0 => hops,
            // A fixed path or a direct return has no alternatives to weight over.
            other => {
                return (0..count)
                    .map(|_| self.resolve_path(destination, me, other.clone()))
                    .collect::<FuturesUnordered<_>>()
                    .try_collect::<Vec<_>>()
                    .await;
            }
        };

        let candidates = self.cached_paths(destination, me, hops).await?;
        let items: Vec<&(ValidatedPath, f64)> = candidates.iter().collect();
        let weights = temper_weights(
            &items.iter().map(|(_, w)| *w).collect::<Vec<_>>(),
            self.return_path_weight_temper,
        );

        if weights.iter().all(|w| *w <= 0.0) {
            return Err(PathPlannerError::Path(PathError::PathNotFound(
                hops.into(),
                destination.to_string(),
                me.to_string(),
            )));
        }

        // Distinct first relayers in the candidate pool: the return-relayer diversity actually
        // available for this destination. A value of 1 is the corroboration blind spot (a dead
        // relayer cannot be told from a quiet peer) — the same condition `rebuild_candidates` WARNs
        // on. This draw runs per-packet, so both sets below are built only when DEBUG is actually
        // recorded -- not on every call.
        let log_relayer_diversity = tracing::enabled!(tracing::Level::DEBUG);
        let candidate_relayers = log_relayer_diversity.then(|| {
            items
                .iter()
                .filter_map(|item| item.0.first())
                .collect::<std::collections::HashSet<_>>()
        });

        tracing::debug!(
            %destination,
            count,
            candidates = items.len(),
            distinct_relayers = candidate_relayers.as_ref().map_or(0, |r| r.len()),
            temper = self.return_path_weight_temper,
            exploration = self.return_path_exploration,
            "drawing return paths from tempered weights"
        );

        let drawn = (0..count)
            .filter_map(|_| {
                if should_explore(self.return_path_exploration) {
                    pick_uniform_index(items.len())
                } else {
                    pick_weighted_index(&weights)
                }
                .map(|i| items[i].0.clone())
            })
            .collect::<Vec<_>>();

        // How many distinct relayers the actual SURBs went to. Fewer than the candidate pool means
        // the weighting concentrated the stream — expected — but a persistent 1 here while
        // `candidates` > 1 says the draw itself is starving the siblings the detector relies on.
        let drawn_relayers = log_relayer_diversity.then(|| {
            drawn
                .iter()
                .filter_map(|vp| vp.first())
                .collect::<std::collections::HashSet<_>>()
        });
        tracing::debug!(
            %destination,
            drawn = drawn.len(),
            distinct_relayers = drawn_relayers.as_ref().map_or(0, |r| r.len()),
            of_candidates = candidate_relayers.as_ref().map_or(0, |r| r.len()),
            "drew return paths"
        );

        Ok(drawn)
    }

    /// Resolve a [`DestinationRouting`] to a [`ResolvedTransportRouting`].
    ///
    /// Returns the resolved routing and, for `Return` variants, the number of remaining SURBs.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn resolve_routing(
        &self,
        size_hint: usize,
        max_surbs: usize,
        routing: DestinationRouting,
    ) -> Result<(ResolvedTransportRouting<HoprSurb>, Option<usize>)> {
        match routing {
            DestinationRouting::Forward {
                destination,
                pseudonym,
                forward_options,
                return_options,
            } => {
                tracing::debug!(direction = "forward", %destination, "resolving forward path");

                let forward_path = self
                    .resolve_path(NodeId::Offchain(self.me), *destination, forward_options)
                    .await?;
                tracing::debug!(direction = "forward", %destination, path = %forward_path, "resolved path");

                let return_paths = if let Some(return_options) = return_options {
                    let num_possible_surbs = HoprPacket::max_surbs_with_message(size_hint).min(max_surbs);
                    trace!(
                        %destination,
                        %num_possible_surbs,
                        data_len = size_hint,
                        max_surbs,
                        "resolving packet return paths"
                    );

                    self.resolve_diverse_return_paths(*destination, return_options, num_possible_surbs)
                        .await?
                        .into_iter()
                        .enumerate()
                        .inspect(|(i, rp)| {
                            tracing::debug!(direction = "return", %destination, index = i, path = %rp, "resolved return path");
                        })
                        .map(|(_, rp)| rp)
                        .collect()
                } else {
                    vec![]
                };

                trace!(%destination, num_surbs = return_paths.len(), data_len = size_hint, "resolved packet");

                Ok((
                    ResolvedTransportRouting::Forward {
                        pseudonym: pseudonym.unwrap_or_else(HoprPseudonym::random),
                        forward_path,
                        return_paths,
                    },
                    None,
                ))
            }

            DestinationRouting::Return(matcher) => {
                let FoundSurb {
                    sender_id,
                    surb,
                    remaining,
                } = self
                    .surb_store
                    .find_surb(matcher)
                    .ok_or_else(|| PathPlannerError::Surb(format!("no surb for pseudonym {}", matcher.pseudonym())))?;
                Ok((ResolvedTransportRouting::Return(sender_id, surb), Some(remaining)))
            }
        }
    }
}

impl<Surb, R, S> BackgroundPathCacheRefreshable for PathPlanner<Surb, R, S>
where
    Surb: SurbStore + Send + Sync + 'static,
    R: ChainKeyOperations + ChainReadChannelOperations + Send + Sync + 'static,
    S: PathSelector + Send + Sync + 'static,
{
    /// Returns a future that runs the background path-cache refresh loop.
    ///
    /// The returned future iterates over all keys currently in the planner's cache
    /// and recomputes their paths on a configurable schedule, so that steady-state
    /// traffic is always served from cache.
    fn run_background_refresh(&self) -> impl std::future::Future<Output = ()> + Send + 'static {
        // Clone only the fields we need — avoids requiring R: Clone + S: Clone.
        let cache = self.cache.clone();
        let resolver = self.resolver.clone();
        let selector = self.selector.clone();
        let refresh_period = self.refresh_period;
        let weighting = self.weighting;
        let me = self.me;

        // run at a non-zero interval
        futures_time::stream::interval(futures_time::time::Duration::from_millis(
            refresh_period.as_millis() as u64 + 1u64,
        ))
        .for_each(move |_| {
            let cache = cache.clone();
            let resolver = resolver.clone();
            let selector = selector.clone();
            let weighting = weighting;

            async move {
                for (key, _) in cache.iter() {
                    let (src_key, dest_key, hops_u32) = {
                        let k = key.as_ref();
                        (k.0, k.1, k.2)
                    };

                    if hops_u32 == 0 {
                        continue;
                    }

                    // The key already holds resolved offchain keys, which is what the selector
                    // wants -- so nothing has to be resolved again here.
                    if let Ok(Some(weighted)) = rebuild_candidates(
                        &*resolver,
                        &*selector,
                        weighting,
                        me,
                        src_key,
                        dest_key,
                        hops_u32 as usize,
                        "background-refresh",
                    )
                    .await
                    {
                        cache.insert((src_key, dest_key, hops_u32), Arc::new(weighted)).await;
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bimap::BiMap;
    use futures::stream::{self, BoxStream};
    use hex_literal::hex;
    use hopr_api::{
        chain::{ChainKeyOperations, ChainReadChannelOperations, ChannelSelector, HoprKeyIdent},
        graph::{NetworkGraphWrite, traits::EdgeObservableWrite},
        types::{
            crypto::prelude::{Keypair, OffchainKeypair},
            internal::channels::{ChannelEntry, ChannelStatus, generate_channel_id},
            primitive::prelude::*,
        },
    };
    use hopr_network_graph::ChannelGraph;

    use super::*;
    use crate::path::selector::HoprGraphPathSelector;

    #[derive(Debug)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl std::error::Error for TestError {}

    const SECRET_ME: [u8; 32] = hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");
    const SECRET_A: [u8; 32] = hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a");
    const SECRET_DEST: [u8; 32] = hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299");
    /// A second relayer, so a return fixture can offer a real alternative to the one that dies.
    const SECRET_B: [u8; 32] = hex!("3d5f2c8a91b4e07d6a3c1f8e2b95d40c7e6a1938f2c5b8d04e7a1c396b2f8d05");

    fn pubkey(secret: &[u8; 32]) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(secret).expect("valid secret").public()
    }

    #[derive(Clone)]
    struct Mapper {
        map: Arc<BiMap<OffchainPublicKey, HoprKeyIdent>>,
    }

    impl KeyIdMapping<HoprKeyIdent, OffchainPublicKey> for Mapper {
        fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
            self.map.get_by_left(key).copied()
        }

        fn map_id_to_public(&self, id: &HoprKeyIdent) -> Option<OffchainPublicKey> {
            self.map.get_by_right(id).copied()
        }

        fn map_keys_to_ids(&self, keys: &[OffchainPublicKey]) -> Vec<Option<HoprKeyIdent>> {
            keys.iter().map(|key| self.map_key_to_id(key)).collect()
        }

        fn map_ids_to_keys(&self, ids: &[HoprKeyIdent]) -> Vec<Option<OffchainPublicKey>> {
            ids.iter().map(|id| self.map_id_to_public(id)).collect()
        }
    }

    struct TestChainApi {
        me: Address,
        key_addr_map: BiMap<OffchainPublicKey, Address>,
        channels: Vec<ChannelEntry>,
        id_mapper: Mapper,
    }

    impl TestChainApi {
        fn new(me_key: OffchainPublicKey, me_addr: Address, peers: Vec<(OffchainPublicKey, Address)>) -> Self {
            let mut key_addr_map = BiMap::new();
            let mut key_id_map: BiMap<OffchainPublicKey, HoprKeyIdent> = BiMap::new();
            key_addr_map.insert(me_key, me_addr);
            key_id_map.insert(me_key, 0u32.into());
            for (i, (k, a)) in peers.iter().enumerate() {
                key_addr_map.insert(*k, *a);
                key_id_map.insert(*k, ((i + 1) as u32).into());
            }
            Self {
                me: me_addr,
                key_addr_map,
                channels: vec![],
                id_mapper: Mapper {
                    map: Arc::new(key_id_map),
                },
            }
        }

        fn with_open_channel(mut self, src: Address, dst: Address) -> Self {
            self.channels.push(
                ChannelEntry::builder()
                    .between(src, dst)
                    .amount(100)
                    .ticket_index(1)
                    .status(ChannelStatus::Open)
                    .epoch(1)
                    .build()
                    .unwrap(),
            );
            self
        }
    }

    impl ChainKeyOperations for TestChainApi {
        type Error = TestError;
        type Mapper = Mapper;

        fn chain_key_to_packet_key(
            &self,
            chain: &Address,
        ) -> std::result::Result<Option<OffchainPublicKey>, TestError> {
            Ok(self.key_addr_map.get_by_right(chain).copied())
        }

        fn packet_key_to_chain_key(
            &self,
            packet: &OffchainPublicKey,
        ) -> std::result::Result<Option<Address>, TestError> {
            Ok(self.key_addr_map.get_by_left(packet).copied())
        }

        fn key_id_mapper_ref(&self) -> &Self::Mapper {
            &self.id_mapper
        }
    }

    impl ChainReadChannelOperations for TestChainApi {
        type Error = TestError;

        fn me(&self) -> &Address {
            &self.me
        }

        fn channel_by_id(&self, channel_id: &ChannelId) -> std::result::Result<Option<ChannelEntry>, TestError> {
            Ok(self
                .channels
                .iter()
                .find(|c| generate_channel_id(&c.source, &c.destination) == *channel_id)
                .cloned())
        }

        fn stream_channels<'a>(
            &'a self,
            _selector: ChannelSelector,
        ) -> std::result::Result<BoxStream<'a, ChannelEntry>, TestError> {
            Ok(Box::pin(stream::iter(self.channels.clone())))
        }
    }

    #[test]
    fn exploration_should_be_off_at_zero_and_certain_at_one() {
        // The two endpoints are what callers actually configure, and a mistake at either end is
        // silent: 0.0 that still explores wastes throughput, 1.0 that never does starves the
        // observations the weights are built from.
        assert!((0..200).all(|_| !should_explore(0.0)));
        assert!((0..200).all(|_| should_explore(1.0)));
    }

    #[test]
    fn exploration_rate_should_be_near_the_configured_fraction() {
        let explored = (0..10_000).filter(|_| should_explore(0.1)).count();
        // Loose bounds: this is a CSPRNG draw, so the test is guarding the wiring, not the
        // generator's uniformity.
        assert!(
            (700..1_300).contains(&explored),
            "expected roughly 1000 of 10000 draws to explore, got {explored}"
        );
    }

    #[test]
    fn uniform_index_should_stay_in_range_and_reach_every_candidate() {
        assert_eq!(None, pick_uniform_index(0), "nothing to pick from");

        let mut seen = std::collections::HashSet::new();
        for _ in 0..1_000 {
            let i = pick_uniform_index(4).expect("non-empty");
            assert!(i < 4, "index {i} out of range");
            seen.insert(i);
        }
        assert_eq!(4, seen.len(), "every candidate must be reachable");
    }

    #[test]
    fn exploration_should_reach_a_candidate_that_weighting_would_never_pick() {
        // The point of the knob: a path whose weight has collapsed still gets traffic occasionally,
        // which is the only way it can ever be re-measured and recover.
        let weights = [1.0, 0.0];
        assert!(
            (0..200).all(|_| pick_weighted_index(&weights) == Some(0)),
            "a zero weight is never drawn by weight"
        );
        assert!(
            (0..1_000).any(|_| pick_uniform_index(weights.len()) == Some(1)),
            "an exploratory draw must be able to reach it"
        );
    }

    /// Cached candidate weights are frozen at fill time, so the TTL bounds how stale the numbers a
    /// live session draws from can be -- not just how stale the candidate set is.
    #[test]
    fn the_path_cache_should_expire_faster_than_a_session_can_be_lost() {
        let cfg = PathPlannerConfig::default();

        // The SURB round-trip window slices at 5s. A TTL far above that lets a relay keep its full
        // share of return-path draws long after the graph has scored it down -- measured as a
        // session that stayed degraded for minutes after the evidence was in.
        assert!(
            cfg.cache_ttl <= Duration::from_secs(15),
            "path cache TTL {:?} outlives the evidence that should displace it",
            cfg.cache_ttl
        );
        assert!(
            cfg.refresh_period < cfg.cache_ttl,
            "the background sweep ({:?}) must re-weight entries before they expire ({:?})",
            cfg.refresh_period,
            cfg.cache_ttl
        );
    }

    #[test]
    fn config_should_reject_a_weight_temper_outside_the_unit_range() {
        assert!(PathPlannerConfig::default().validate().is_ok());

        // 1.0 is the raw path value — the sharpest permitted, and the pre-tempering behaviour.
        assert!(
            PathPlannerConfig {
                return_path_weight_temper: 1.0,
                ..PathPlannerConfig::default()
            }
            .validate()
            .is_ok()
        );

        // Above 1.0 sharpens instead of flattening, concentrating harder than raw path value.
        let sharpening = PathPlannerConfig {
            return_path_weight_temper: 1.5,
            ..PathPlannerConfig::default()
        };
        // Assert on the rendered message, which is what an operator actually sees.
        let err = sharpening
            .validate()
            .expect_err("a sharpening exponent must be rejected");
        assert!(err.to_string().contains("must be in (0, 1]"), "{err}");

        // Zero annihilates the ordering: every candidate would weigh exactly 1.
        assert!(
            PathPlannerConfig {
                return_path_weight_temper: 0.0,
                ..PathPlannerConfig::default()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn temper_should_compress_the_weight_spread_without_reordering() {
        let raw = [0.4, 0.3, 0.2, 0.1];

        let untouched = temper_weights(&raw, 1.0);
        assert_eq!(raw.to_vec(), untouched, "temper 1.0 must be the identity");

        let flattened = temper_weights(&raw, 0.5);
        // Order preserved…
        assert!(flattened.windows(2).all(|w| w[0] > w[1]), "{flattened:?}");
        // …but the best-to-worst ratio shrinks from 4.0 towards 1.0, which is what bounds how much
        // of a session rides on the single best relayer.
        let raw_ratio = raw[0] / raw[3];
        let tempered_ratio = flattened[0] / flattened[3];
        assert!(
            tempered_ratio < raw_ratio,
            "{tempered_ratio} should be below {raw_ratio}"
        );
        assert!((tempered_ratio - 2.0).abs() < 1e-9, "{tempered_ratio}");
    }

    #[test]
    fn temper_should_clamp_negative_weights_instead_of_producing_nan() {
        // `powf` on a negative base with a fractional exponent is NaN, which would poison the
        // cumulative sum in `pick_weighted_index` and make selection return nothing.
        let out = temper_weights(&[-1.0, 0.0, 4.0], 0.5);
        assert!(out.iter().all(|w| w.is_finite()), "{out:?}");
        assert_eq!(vec![0.0, 0.0, 2.0], out);
    }

    #[test]
    fn pick_weighted_index_should_reject_a_non_positive_total() {
        assert_eq!(None, pick_weighted_index(&[]));
        assert_eq!(None, pick_weighted_index(&[0.0, 0.0]));
        assert_eq!(Some(1), pick_weighted_index(&[0.0, 1.0]));
    }

    // ── address fixtures ──────────────────────────────────────────────────────
    fn me_addr() -> Address {
        Address::from_str("0x1000d5786d9e6799b3297da1ad55605b91e2c882").expect("valid addr")
    }
    fn a_addr() -> Address {
        Address::from_str("0x200060ddced1e33c9647a71f4fc2cf4ed33e4a9d").expect("valid addr")
    }
    fn dest_addr() -> Address {
        Address::from_str("0x30004105095c8c10f804109b4d1199a9ac40ed46").expect("valid addr")
    }
    fn b_addr() -> Address {
        Address::from_str("0x40001a7ec3d5b28f9047c6b1e83d5a2f9c71b0e4").expect("valid addr")
    }

    // ── graph helpers ──────────────────────────────────────────────────────────
    fn mark_edge_full(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        use hopr_api::graph::traits::EdgeWeightType;
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
                1000u64,
            ))));
        });
    }

    fn small_config() -> PathPlannerConfig {
        PathPlannerConfig {
            max_cache_capacity: 100,
            cache_ttl: std::time::Duration::from_secs(60),
            refresh_period: std::time::Duration::from_secs(60),
            max_cached_paths: 2,
            ..PathPlannerConfig::default()
        }
    }

    // ── test: zero-hop path ───────────────────────────────────────────────────

    /// A Session names its destination by chain address; the SURB telemetry names it by packet key.
    ///
    /// Regression: the cache used to be keyed on the `NodeId` as handed in, and
    /// `NodeId::Chain(addr) != NodeId::Offchain(key)` even for the same node -- so the two layers
    /// stored and looked up the same route under different keys without either noticing.
    #[tokio::test]
    async fn a_return_path_should_cache_under_one_key_whichever_form_names_the_node() {
        let me = pubkey(&SECRET_ME);
        let a = pubkey(&SECRET_A);
        let dest = pubkey(&SECRET_DEST);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(dest);
        // A return path runs from the destination back to us.
        graph.add_edge(&dest, &a).unwrap();
        graph.add_edge(&a, &me).unwrap();
        mark_edge_full(&graph, &dest, &a);
        mark_edge_full(&graph, &a, &me);

        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(dest_addr(), a_addr())
            .with_open_channel(a_addr(), me_addr());
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();
        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        // Populated the way a Session does it: by chain address.
        let _ = planner
            .resolve_diverse_return_paths(
                NodeId::Chain(dest_addr()),
                RoutingOptions::Hops(1.try_into().expect("valid 1")),
                1,
            )
            .await
            .expect("return path resolution should succeed");

        let cache_key: PlannerCacheKey = (dest, me, 1);
        assert!(
            planner.cache.get(&cache_key).await.is_some(),
            "the return path should be cached after resolution"
        );

        // Asking again the way the SURB telemetry names the node -- by packet key -- must land on
        // that same entry rather than resolving and caching a second copy.
        planner.cache.run_pending_tasks().await;
        assert_eq!(planner.cache.entry_count(), 1, "one resolution, one entry");

        let _ = planner
            .resolve_diverse_return_paths(
                NodeId::Offchain(dest),
                RoutingOptions::Hops(1.try_into().expect("valid 1")),
                1,
            )
            .await
            .expect("return path resolution should succeed");

        planner.cache.run_pending_tasks().await;
        assert_eq!(
            planner.cache.entry_count(),
            1,
            "naming the destination by packet key must hit the entry cached from its chain address"
        );
    }

    #[tokio::test]
    async fn zero_hop_path_should_bypass_selector() {
        let me = pubkey(&SECRET_ME);
        let dest = pubkey(&SECRET_DEST);

        // Build empty graph (no edges) — selector would fail if called.
        let graph = ChannelGraph::new(me);
        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );

        let chain_api = TestChainApi::new(me, me_addr(), vec![(dest, dest_addr())]);
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();

        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        let routing = DestinationRouting::Forward {
            destination: Box::new(NodeId::Offchain(dest)),
            pseudonym: None,
            forward_options: RoutingOptions::Hops(0.try_into().expect("valid 0")),
            return_options: None,
        };

        let result = planner.resolve_routing(100, 0, routing).await;
        assert!(result.is_ok(), "zero-hop should succeed: {:?}", result.err());

        let (resolved, rem) = result.unwrap();
        assert!(rem.is_none());
        if let ResolvedTransportRouting::Forward { forward_path, .. } = resolved {
            assert_eq!(
                forward_path.num_hops(),
                1,
                "zero-hop = 1 node in path (just destination)"
            );
        } else {
            panic!("expected Forward routing");
        }
    }

    // ── test: one-hop path via graph selector ─────────────────────────────────

    #[tokio::test]
    async fn one_hop_path_should_use_selector() {
        let me = pubkey(&SECRET_ME);
        let a = pubkey(&SECRET_A);
        let dest = pubkey(&SECRET_DEST);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(dest);
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &a, &dest);

        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );

        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(me_addr(), a_addr())
            .with_open_channel(a_addr(), dest_addr());

        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();
        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        let routing = DestinationRouting::Forward {
            destination: Box::new(NodeId::Offchain(dest)),
            pseudonym: None,
            forward_options: RoutingOptions::Hops(1.try_into().expect("valid 1")),
            return_options: None,
        };

        let result = planner.resolve_routing(100, 0, routing).await;
        assert!(result.is_ok(), "1-hop routing should succeed: {:?}", result.err());

        let (resolved, _) = result.unwrap();
        if let ResolvedTransportRouting::Forward { forward_path, .. } = resolved {
            assert_eq!(
                forward_path.num_hops(),
                2,
                "1 intermediate hop means path has 2 nodes [a, dest]"
            );
        } else {
            panic!("expected Forward routing");
        }
    }

    #[tokio::test]
    async fn explicit_intermediate_path_should_bypass_selector() {
        let me = pubkey(&SECRET_ME);
        let a = pubkey(&SECRET_A);
        let dest = pubkey(&SECRET_DEST);

        // Empty graph — selector would fail; explicit path should not use it.
        let graph = ChannelGraph::new(me);
        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );

        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(me_addr(), a_addr())
            .with_open_channel(a_addr(), dest_addr());

        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();
        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        use hopr_api::types::primitive::prelude::BoundedVec;
        let intermediate_path = BoundedVec::try_from(vec![NodeId::Offchain(a)]).expect("valid");

        let routing = DestinationRouting::Forward {
            destination: Box::new(NodeId::Offchain(dest)),
            pseudonym: None,
            forward_options: RoutingOptions::IntermediatePath(intermediate_path),
            return_options: None,
        };

        let result = planner.resolve_routing(100, 0, routing).await;
        assert!(result.is_ok(), "explicit path should succeed: {:?}", result.err());

        let (resolved, _) = result.unwrap();
        if let ResolvedTransportRouting::Forward { forward_path, .. } = resolved {
            assert_eq!(forward_path.num_hops(), 2, "one intermediate + destination = 2 hops");
        } else {
            panic!("expected Forward routing");
        }
    }

    #[tokio::test]
    async fn return_routing_without_surb_should_return_error() {
        let me = pubkey(&SECRET_ME);
        let graph = ChannelGraph::new(me);
        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let chain_api = TestChainApi::new(me, me_addr(), vec![]);
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();

        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        use hopr_api::types::internal::routing::SurbMatcher;
        let matcher = SurbMatcher::Pseudonym(HoprPseudonym::random());
        let routing = DestinationRouting::Return(matcher);

        let result = planner.resolve_routing(0, 0, routing).await;
        assert!(result.is_err(), "should fail when no SURB exists");
        assert!(
            matches!(result.unwrap_err(), PathPlannerError::Surb(_)),
            "error should be Surb variant"
        );
    }

    // ── test: cache integration ───────────────────────────────────────────────

    /// Builds a planner over `me -> a -> dest` with both channels open.
    fn diversity_planner(
        cfg: PathPlannerConfig,
    ) -> PathPlanner<hopr_protocol_hopr::MemorySurbStore, TestChainApi, HoprGraphPathSelector<ChannelGraph>> {
        let (me, a, dest) = (pubkey(&SECRET_ME), pubkey(&SECRET_A), pubkey(&SECRET_DEST));

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(dest);
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &dest).unwrap();
        graph.add_edge(&dest, &a).unwrap();
        graph.add_edge(&a, &me).unwrap();
        for (from, to) in [(me, a), (a, dest), (dest, a), (a, me)] {
            mark_edge_full(&graph, &from, &to);
        }

        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(me_addr(), a_addr())
            .with_open_channel(a_addr(), dest_addr())
            .with_open_channel(dest_addr(), a_addr())
            .with_open_channel(a_addr(), me_addr());

        PathPlanner::new(
            me,
            hopr_protocol_hopr::MemorySurbStore::default(),
            chain_api,
            selector,
            cfg,
        )
    }

    #[tokio::test]
    async fn resolve_diverse_return_paths_should_return_empty_without_querying_when_none_requested() {
        let planner = diversity_planner(small_config());
        let dest = NodeId::Offchain(pubkey(&SECRET_DEST));
        let hops = RoutingOptions::Hops(1.try_into().expect("valid 1"));

        // `max_surbs_with_message` returns 0 when the message fills the payload. That must yield an
        // empty result, not a `PathNotFound` — and must not populate the cache.
        let paths = planner
            .resolve_diverse_return_paths(dest, hops, 0)
            .await
            .expect("zero return paths is not an error");
        assert!(paths.is_empty());

        // Keyed on resolved offchain keys, so the test names the nodes the same way.
        let dest_key = match dest {
            NodeId::Offchain(k) => k,
            NodeId::Chain(_) => unreachable!("the fixture names the destination by its packet key"),
        };
        let cache_key: PlannerCacheKey = (dest_key, pubkey(&SECRET_ME), 1);
        assert!(
            planner.cache.get(&cache_key).await.is_none(),
            "nothing requested must not query the planner"
        );
    }

    #[tokio::test]
    async fn resolve_diverse_return_paths_should_return_count_paths_at_any_temper() {
        // Tempering changes *which* candidates are favoured, never how many paths come back — the
        // caller has already sized the batch to the SURBs that fit in its packet.
        for temper in [1.0, 0.5, 0.05] {
            let planner = diversity_planner(PathPlannerConfig {
                return_path_weight_temper: temper,
                ..small_config()
            });
            let dest = NodeId::Offchain(pubkey(&SECRET_DEST));
            let hops = RoutingOptions::Hops(1.try_into().expect("valid 1"));

            let paths = planner
                .resolve_diverse_return_paths(dest, hops, 3)
                .await
                .expect("weighted resolution should succeed");
            assert_eq!(3, paths.len(), "temper={temper} must still return `count` paths");
        }
    }

    #[tokio::test]
    async fn resolve_diverse_return_paths_should_fall_back_for_routing_without_alternatives() {
        let planner = diversity_planner(small_config());
        let dest = NodeId::Offchain(pubkey(&SECRET_DEST));

        // A direct return has no relayer to spread over.
        let direct = planner
            .resolve_diverse_return_paths(dest, RoutingOptions::Hops(0.try_into().expect("valid 0")), 2)
            .await
            .expect("zero-hop return should resolve");
        assert_eq!(2, direct.len());

        // An explicit path resolves to itself.
        let explicit = planner
            .resolve_diverse_return_paths(
                dest,
                RoutingOptions::IntermediatePath(vec![NodeId::Offchain(pubkey(&SECRET_A))].try_into().expect("valid")),
                2,
            )
            .await
            .expect("explicit return path should resolve");
        assert_eq!(2, explicit.len());
    }

    #[tokio::test]
    async fn resolve_diverse_return_paths_should_return_the_requested_count() {
        let planner = diversity_planner(small_config());
        let dest = NodeId::Offchain(pubkey(&SECRET_DEST));
        let hops = RoutingOptions::Hops(1.try_into().expect("valid 1"));

        // Fewer paths than the configured diversity: capped to `count`, still exactly `count` paths.
        for count in [1usize, 2, 5] {
            let paths = planner
                .resolve_diverse_return_paths(dest, hops.clone(), count)
                .await
                .expect("should resolve");
            assert_eq!(count, paths.len(), "count={count}");
        }
    }

    #[tokio::test]
    async fn planner_cache_miss_should_populate_cache() {
        let me = pubkey(&SECRET_ME);
        let a = pubkey(&SECRET_A);
        let dest = pubkey(&SECRET_DEST);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(dest);
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &a, &dest);

        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(me_addr(), a_addr())
            .with_open_channel(a_addr(), dest_addr());
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();
        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        let cache_key: PlannerCacheKey = (me, dest, 1);

        assert!(
            planner.cache.get(&cache_key).await.is_none(),
            "cache should be empty before first call"
        );

        let routing = DestinationRouting::Forward {
            destination: Box::new(NodeId::Offchain(dest)),
            pseudonym: None,
            forward_options: RoutingOptions::Hops(1.try_into().expect("valid 1")),
            return_options: None,
        };
        planner.resolve_routing(100, 0, routing).await.expect("should succeed");

        let cached = planner.cache.get(&cache_key).await;
        assert!(cached.is_some(), "cache should be populated after first call");
        let paths = cached.unwrap();
        assert!(!paths.is_empty(), "cached paths must be non-empty");
        let (first_path, first_cost) = paths.iter().next().expect("at least one cached path");
        assert_eq!(first_path.num_hops(), 2, "path should have 2 hops [a, dest]");
        assert!(*first_cost > 0.0, "cost should be positive");
    }

    #[tokio::test]
    async fn planner_cache_hit_should_return_valid_path() {
        let me = pubkey(&SECRET_ME);
        let a = pubkey(&SECRET_A);
        let dest = pubkey(&SECRET_DEST);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(dest);
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &a, &dest);

        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(me_addr(), a_addr())
            .with_open_channel(a_addr(), dest_addr());
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();
        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        let make_routing = || DestinationRouting::Forward {
            destination: Box::new(NodeId::Offchain(dest)),
            pseudonym: None,
            forward_options: RoutingOptions::Hops(1.try_into().expect("valid 1")),
            return_options: None,
        };

        let (r1, _) = planner.resolve_routing(100, 0, make_routing()).await.expect("call 1");
        let (r2, _) = planner.resolve_routing(100, 0, make_routing()).await.expect("call 2");

        let hops1 = if let ResolvedTransportRouting::Forward { forward_path, .. } = r1 {
            forward_path.num_hops()
        } else {
            panic!("expected Forward");
        };
        let hops2 = if let ResolvedTransportRouting::Forward { forward_path, .. } = r2 {
            forward_path.num_hops()
        } else {
            panic!("expected Forward");
        };
        assert_eq!(hops1, 2);
        assert_eq!(hops2, 2);
    }

    #[tokio::test]
    async fn background_refresh_should_produce_a_future() {
        let me = pubkey(&SECRET_ME);
        let graph = ChannelGraph::new(me);
        let cfg = small_config();
        let selector = HoprGraphPathSelector::new(
            me,
            graph,
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let chain_api = TestChainApi::new(me, me_addr(), vec![]);
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();

        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());
        // Just ensure it compiles and produces a future.
        let _future = planner.run_background_refresh();
    }

    // ── recomputing one cache entry in place ──────────────────────────────────

    type TestPlanner =
        PathPlanner<hopr_protocol_hopr::MemorySurbStore, TestChainApi, HoprGraphPathSelector<ChannelGraph>>;

    /// Return-path fixture with two interchangeable relayers: `dest -> {A, B} -> me`.
    ///
    /// Both start fully observed and identically good, so any later divergence in their share of
    /// the return draws is attributable to what the test recorded and not to the fixture.
    ///
    /// The graph is returned alongside the planner because it is the handle a test needs to move
    /// the evidence *under* an already-populated cache -- which is the whole point of the primitive
    /// under test.
    fn two_relayer_return_planner() -> (TestPlanner, ChannelGraph) {
        two_relayer_return_planner_with_floor(small_config().min_paths_anonymity_floor)
    }

    /// A `me <- {a,b} <- dest` return topology with the given anonymity floor, so a test can watch
    /// the cap collapse the two relayers to one (or, at floor 0, keep both).
    fn two_relayer_return_planner_with_floor(floor: usize) -> (TestPlanner, ChannelGraph) {
        let me = pubkey(&SECRET_ME);
        let a = pubkey(&SECRET_A);
        let b = pubkey(&SECRET_B);
        let dest = pubkey(&SECRET_DEST);

        let graph = ChannelGraph::new(me);
        for node in [a, b, dest] {
            graph.add_node(node);
        }
        for (src, dst) in [(dest, a), (a, me), (dest, b), (b, me)] {
            graph.add_edge(&src, &dst).expect("edge should be addable");
            mark_edge_full(&graph, &src, &dst);
        }

        let cfg = PathPlannerConfig {
            min_paths_anonymity_floor: floor,
            ..small_config()
        };
        let selector = HoprGraphPathSelector::new(
            me,
            graph.clone(),
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (b, b_addr()), (dest, dest_addr())])
            .with_open_channel(dest_addr(), a_addr())
            .with_open_channel(a_addr(), me_addr())
            .with_open_channel(dest_addr(), b_addr())
            .with_open_channel(b_addr(), me_addr());

        let planner = PathPlanner::new(
            me,
            hopr_protocol_hopr::MemorySurbStore::default(),
            chain_api,
            selector,
            cfg,
        );
        (planner, graph)
    }

    /// Same `me <- {a,b} <- dest` return topology as [`two_relayer_return_planner_with_floor`], but
    /// `unresolvable` is left out of the chain resolver's key-to-address map -- present in the graph,
    /// so the selector still offers it as a candidate relayer, but `ValidatedPath::new` cannot map it
    /// to a chain address and drops every path through it. Floor 0, so pruning does not also remove
    /// it before validation gets the chance to.
    fn two_relayer_return_planner_with_unresolvable_relayer(
        unresolvable: OffchainPublicKey,
    ) -> (TestPlanner, ChannelGraph) {
        let me = pubkey(&SECRET_ME);
        let a = pubkey(&SECRET_A);
        let b = pubkey(&SECRET_B);
        let dest = pubkey(&SECRET_DEST);

        let graph = ChannelGraph::new(me);
        for node in [a, b, dest] {
            graph.add_node(node);
        }
        for (src, dst) in [(dest, a), (a, me), (dest, b), (b, me)] {
            graph.add_edge(&src, &dst).expect("edge should be addable");
            mark_edge_full(&graph, &src, &dst);
        }

        let cfg = PathPlannerConfig {
            min_paths_anonymity_floor: 0,
            ..small_config()
        };
        let selector = HoprGraphPathSelector::new(
            me,
            graph.clone(),
            cfg.max_cached_paths,
            cfg.edge_penalty,
            cfg.min_ack_rate,
            cfg.min_paths_anonymity_floor,
        );
        let resolvable_peers = [(a, a_addr()), (b, b_addr()), (dest, dest_addr())]
            .into_iter()
            .filter(|(key, _)| *key != unresolvable)
            .collect::<Vec<_>>();
        let chain_api = TestChainApi::new(me, me_addr(), resolvable_peers)
            .with_open_channel(dest_addr(), a_addr())
            .with_open_channel(a_addr(), me_addr())
            .with_open_channel(dest_addr(), b_addr())
            .with_open_channel(b_addr(), me_addr());

        let planner = PathPlanner::new(
            me,
            hopr_protocol_hopr::MemorySurbStore::default(),
            chain_api,
            selector,
            cfg,
        );
        (planner, graph)
    }

    /// Populates the return-path cache the way a Session does and hands back the cached entry.
    async fn fill_return_cache(planner: &TestPlanner) -> PlannerCacheValue {
        planner
            .resolve_diverse_return_paths(
                NodeId::Offchain(pubkey(&SECRET_DEST)),
                RoutingOptions::Hops(1.try_into().expect("valid 1")),
                1,
            )
            .await
            .expect("return path resolution should succeed");
        planner
            .cache
            .get(&(pubkey(&SECRET_DEST), pubkey(&SECRET_ME), 1))
            .await
            .expect("the return path should be cached after resolution")
    }

    /// Share of the collection's total weight held by candidates whose first hop is `relayer`.
    ///
    /// This -- not the raw weight -- is what decides how much of a session's return stream rides on
    /// one relay, because the draw normalises over the collection.
    fn share_of(paths: &PlannerCacheValue, relayer: &OffchainPublicKey) -> f64 {
        let total: f64 = paths.iter().map(|(_, w)| *w).sum();
        let held: f64 = paths
            .iter()
            .filter(|(vp, _)| vp.first() == Some(relayer))
            .map(|(_, w)| *w)
            .sum();
        if total > 0.0 { held / total } else { 0.0 }
    }

    /// The candidate paths as stable strings, weights discarded.
    fn candidate_set(paths: &PlannerCacheValue) -> std::collections::BTreeSet<String> {
        paths.iter().map(|(vp, _)| vp.to_string()).collect()
    }

    /// Records `rounds` SURB round-trips on the edge, of which `observed` per round came back.
    fn record_surbs(
        graph: &ChannelGraph,
        src: &OffchainPublicKey,
        dst: &OffchainPublicKey,
        expected: u64,
        observed: u64,
    ) {
        use hopr_api::graph::traits::EdgeWeightType;
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::SurbRoundTrips { expected, observed });
        });
    }

    /// A relay that stops delivering must lose its share of the return draws *without* the
    /// candidate set changing underneath the live session.
    ///
    /// Both halves matter and they pull against each other: dropping the candidates is what
    /// collapsed a healthy session from 100 % to 0.14 % in the cluster, while leaving the weights
    /// alone is what kept minting SURBs onto a dead relay for a full refresh period.
    #[tokio::test]
    async fn recomputing_an_entry_should_reweight_the_candidates_without_changing_the_set() {
        let (planner, graph) = two_relayer_return_planner();
        let (me, b, dest) = (pubkey(&SECRET_ME), pubkey(&SECRET_B), pubkey(&SECRET_DEST));

        let before = fill_return_cache(&planner).await;
        let set_before = candidate_set(&before);
        let share_before = share_of(&before, &b);

        assert_eq!(
            set_before.len(),
            2,
            "the fixture must offer both relayers as candidates"
        );
        assert!(
            share_before > 0.2,
            "vacuity guard: B must hold a material share before the collapse, held {share_before}"
        );

        // A delivers, B stops. Same interval, so the contrast is in the evidence and not in time.
        record_surbs(&graph, &dest, &pubkey(&SECRET_A), 1_000, 1_000);
        record_surbs(&graph, &dest, &b, 1_000, 1_000);
        record_surbs(&graph, &dest, &b, 4_000, 0);

        let replaced = planner.recompute_paths_from(&dest).await;
        assert_eq!(
            replaced, 1,
            "exactly the one cached return entry should have been rebuilt"
        );

        let after = planner
            .cache
            .get(&(dest, me, 1))
            .await
            .expect("the entry must still exist after a recompute");

        assert_eq!(
            candidate_set(&after),
            set_before,
            "a recompute must re-weight the candidates, never replace the set"
        );

        let share_after = share_of(&after, &b);
        assert!(
            share_after < share_before,
            "the relay that stopped delivering must lose share: {share_before} -> {share_after}"
        );
    }

    /// A recompute that finds nothing must leave the previous candidates in place.
    ///
    /// This is the property that separates recomputation from invalidation. A live session draws
    /// its next return path from this entry, so an empty result has to mean "no better information"
    /// rather than "no route".
    #[tokio::test]
    async fn recomputing_an_entry_should_keep_the_old_candidates_when_it_finds_none() {
        let (planner, graph) = two_relayer_return_planner();
        let (me, dest) = (pubkey(&SECRET_ME), pubkey(&SECRET_DEST));

        let before = fill_return_cache(&planner).await;
        let set_before = candidate_set(&before);

        // Total blackout: every route out of the destination disappears from the graph, so the
        // selector can no longer offer any candidate at all.
        for relay in [pubkey(&SECRET_A), pubkey(&SECRET_B)] {
            graph.remove_edge(&dest, &relay);
        }

        let replaced = planner.recompute_paths_from(&dest).await;
        assert_eq!(replaced, 0, "a recompute that finds nothing must replace nothing");

        let after = planner
            .cache
            .get(&(dest, me, 1))
            .await
            .expect("an entry must never be dropped by a recompute that found nothing");
        assert_eq!(
            candidate_set(&after),
            set_before,
            "the previous candidates must survive a fruitless recompute"
        );
    }

    /// A recompute is addressed at one counterparty, so it must not sweep the whole cache.
    ///
    /// Rebuilding every entry on every detection would put the cost of one dead relay onto every
    /// other session the node is carrying.
    /// A recompute that lands on the same weights has moved no traffic, and the caller uses that
    /// answer to decide whether refilling is worth anything.
    ///
    /// Reporting the number of entries *rebuilt* instead would say "moved" every time, since a
    /// rebuild always succeeds on a healthy graph -- and refilling behind a re-plan that changed
    /// nothing just mints more SURBs onto the same route.
    #[tokio::test]
    async fn a_recompute_that_lands_on_the_same_weights_should_report_nothing_moved() {
        let (planner, graph) = two_relayer_return_planner();
        let dest = pubkey(&SECRET_DEST);

        let _ = fill_return_cache(&planner).await;

        assert_eq!(
            planner.recompute_paths_from(&dest).await,
            0,
            "nothing about the graph changed, so no traffic can have moved"
        );

        // Vacuity guard: the same call must report movement once the evidence actually shifts.
        // B has to deliver first -- the rate is read against a peak, so a relayer that never had
        // one has no rate to fall from and the collapse would be invisible.
        record_surbs(&graph, &dest, &pubkey(&SECRET_A), 1_000, 1_000);
        record_surbs(&graph, &dest, &pubkey(&SECRET_B), 1_000, 1_000);
        record_surbs(&graph, &dest, &pubkey(&SECRET_B), 4_000, 0);
        assert_eq!(
            planner.recompute_paths_from(&dest).await,
            1,
            "a collapse on one relayer must register as moved"
        );
    }

    /// End-to-end proof of the fix: a disabled cap keeps every return relayer, a cap of one
    /// collapses the stream onto a single relayer. The latter is the blind spot the degradation
    /// detector cannot escape, and the shape of the 2026-08-28 outage; the former is what the
    /// edge-client `latency_path_planner_config` now requests (`min_paths_anonymity_floor = 0`).
    #[tokio::test]
    async fn floor_zero_keeps_every_return_relayer_but_a_cap_of_one_collapses_it() {
        let (a, b) = (pubkey(&SECRET_A), pubkey(&SECRET_B));

        let (uncapped, _g) = two_relayer_return_planner_with_floor(0);
        let paths = fill_return_cache(&uncapped).await;
        assert_eq!(2, candidate_set(&paths).len(), "floor 0 keeps both return relayers");
        assert!(
            share_of(&paths, &a) > 0.0,
            "relayer a carries part of the return stream"
        );
        assert!(
            share_of(&paths, &b) > 0.0,
            "relayer b carries part of the return stream"
        );

        let (capped, _g) = two_relayer_return_planner_with_floor(1);
        let paths = fill_return_cache(&capped).await;
        assert_eq!(1, candidate_set(&paths).len(), "floor 1 collapses to a single relayer");
    }

    /// Regression for the diversity check firing on the wrong set: the selector's raw candidates
    /// see both relayers, but chain validation rejects every path through `b` (its key never
    /// resolves to a chain address), so what a session can actually draw from has collapsed to `a`
    /// alone. A diversity check computed on the selector's pre-validation output would have missed
    /// this entirely -- it must be computed on the validated survivors, which is what
    /// `rebuild_candidates` now does.
    #[tokio::test]
    async fn validation_rejecting_one_relayer_should_collapse_diversity_even_though_the_selector_saw_two() {
        let (a, b) = (pubkey(&SECRET_A), pubkey(&SECRET_B));

        let (planner, _g) = two_relayer_return_planner_with_unresolvable_relayer(b);
        let paths = fill_return_cache(&planner).await;

        assert_eq!(
            1,
            candidate_set(&paths).len(),
            "only the relayer that resolves on-chain should survive validation"
        );
        assert!(
            share_of(&paths, &a) > 0.0,
            "the resolvable relayer must carry the whole return stream"
        );
        assert_eq!(
            0.0,
            share_of(&paths, &b),
            "the unresolvable relayer must not appear among the validated candidates"
        );
    }

    #[tokio::test]
    async fn recomputing_should_rebuild_only_the_entries_originating_at_that_peer() {
        let (planner, graph) = two_relayer_return_planner();
        let (me, a, dest) = (pubkey(&SECRET_ME), pubkey(&SECRET_A), pubkey(&SECRET_DEST));

        let _ = fill_return_cache(&planner).await;
        // Something has to actually move, or the count below would be zero for the wrong reason.
        record_surbs(&graph, &dest, &a, 1_000, 1_000);
        record_surbs(&graph, &dest, &pubkey(&SECRET_B), 1_000, 1_000);
        record_surbs(&graph, &dest, &pubkey(&SECRET_B), 4_000, 0);
        // A second entry in the other direction, which no `dest` recompute may touch.
        let _ = planner
            .cached_paths(
                NodeId::Offchain(me),
                NodeId::Offchain(a),
                1.try_into().expect("valid 1"),
            )
            .await;

        assert_eq!(
            planner.recompute_paths_from(&dest).await,
            1,
            "only the entry whose paths start at the named peer should be re-weighted"
        );
        assert_eq!(
            planner.recompute_paths_from(&pubkey(&SECRET_B)).await,
            0,
            "a peer with no cached entries of its own must rebuild nothing"
        );
    }

    // ── composite weight helpers ──────────────────────────────────────────────

    fn default_weighting() -> WeightingParams {
        WeightingParams {
            latency_halflife: Duration::from_millis(100),
            capacity_reference: 10_000_000,
        }
    }

    fn make_pwm(cost: f64, latency_ms: Option<u32>, fundable_tickets_floor: Option<u128>) -> PathWithMetrics {
        PathWithMetrics {
            path: vec![],
            cost,
            total_latency_ms: latency_ms,
            min_probe_success_rate: None,
            min_ack_rate: None,
            fundable_tickets_floor,
        }
    }

    #[test]
    fn latency_factor_is_monotonic_decreasing() {
        let halflife = Duration::from_millis(100);
        let f0 = latency_factor(Duration::ZERO, halflife);
        let f100 = latency_factor(Duration::from_millis(100), halflife);
        let f200 = latency_factor(Duration::from_millis(200), halflife);
        assert!(
            f0 > f100 && f100 > f200,
            "must be strictly decreasing: {f0} > {f100} > {f200}"
        );
        assert!(
            (f100 - 0.5).abs() < 1e-9,
            "at halflife factor should be 0.5, got {f100}"
        );
        assert!(f0 <= 1.0, "factor must never exceed 1.0, got {f0}");
    }

    #[test]
    fn capacity_factor_is_monotonic_increasing() {
        let reference = 10_000_000u128;
        let f_low = capacity_factor(100, reference);
        let f_mid = capacity_factor(1_000_000, reference);
        let f_ref = capacity_factor(reference, reference);
        assert!(
            f_low < f_mid && f_mid <= f_ref,
            "must be non-decreasing: {f_low} < {f_mid} <= {f_ref}"
        );
        assert!(f_ref <= 1.0, "factor must not exceed 1.0 at reference, got {f_ref}");
        assert!(f_low >= 0.05, "minimum clamp is 0.05, got {f_low}");
    }

    #[test]
    fn composite_weight_for_0_hop_skips_capacity_factor() {
        let params = default_weighting();
        let pwm = make_pwm(0.6, Some(100), None);
        let w = composite_weight(&pwm, 0, params);
        let expected = 0.6 * latency_factor(Duration::from_millis(100), params.latency_halflife);
        assert!(
            (w - expected).abs() < 1e-9,
            "0-hop weight should ignore capacity: {w} != {expected}"
        );
        assert!(w > 0.0, "0-hop weight must be positive");
    }

    #[test]
    fn composite_weight_with_all_aggregates_is_below_cost() {
        let params = default_weighting();
        let pwm = make_pwm(0.8, Some(150), Some(5_000_000));
        let w = composite_weight(&pwm, 1, params);
        assert!(
            w < pwm.cost,
            "composite must be below raw cost when factors < 1.0: {w} >= {}",
            pwm.cost
        );
        assert!(w > 0.0, "composite weight must be positive");
    }

    #[test]
    fn composite_weight_missing_capacity_on_multi_hop_neutral() {
        let params = default_weighting();
        let pwm_with = make_pwm(0.7, Some(80), Some(8_000_000));
        let pwm_without = make_pwm(0.7, Some(80), None);
        let w_with = composite_weight(&pwm_with, 2, params);
        let w_without = composite_weight(&pwm_without, 2, params);
        // Missing capacity is neutral (1.0), so without-capacity weight equals cost * lat only.
        let expected_without = 0.7 * latency_factor(Duration::from_millis(80), params.latency_halflife);
        assert!((w_without - expected_without).abs() < 1e-9);
        // With capacity the factor adds further reduction (capacity_factor < 1.0 here).
        assert!(w_with <= w_without, "known capacity should not increase the weight");
    }
}
