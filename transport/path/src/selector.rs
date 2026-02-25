use std::{sync::Arc, time::Duration};

#[cfg(feature = "runtime-tokio")]
use futures::StreamExt;
use hopr_api::graph::{NetworkGraphTraverse, NetworkGraphView, costs::HoprCostFn, traits::EdgeObservableRead};
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::errors::PathError;
#[cfg(feature = "runtime-tokio")]
use hopr_network_types::types::RoutingOptions;
#[cfg(feature = "runtime-tokio")]
use tracing::debug;
use tracing::trace;

#[cfg(feature = "runtime-tokio")]
use crate::traits::BackgroundRefreshable;
use crate::{
    errors::{PathPlannerError, Result},
    traits::PathSelector,
};

/// Configuration for [`GraphPathSelector`].
#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct PathSelectorConfig {
    /// Maximum number of (destination, hops) entries in the path cache.
    #[default = 10_000]
    pub max_cache_capacity: u64,
    /// Time-to-live for a cached path list. When an entry expires the next
    /// [`GraphPathSelector::select_path`] call transparently recomputes it via
    /// the graph (lazy refresh, always available).
    #[default(Duration::from_secs(60))]
    pub cache_ttl: Duration,
    /// Period between proactive background cache-refresh sweeps.
    ///
    /// Only used when the `runtime-tokio` feature is enabled and
    /// [`GraphPathSelector::run_background_refresh`] is spawned.
    #[cfg(feature = "runtime-tokio")]
    #[default(Duration::from_secs(30))]
    pub refresh_period: Duration,
    /// Maximum number of candidate paths cached per (destination, hops) slot.
    #[default = 8]
    pub max_cached_paths: usize,
}

type PathToDestination = Vec<OffchainPublicKey>;

type CacheKey = (OffchainPublicKey, OffchainPublicKey, usize); // (src, dest, hops)
type CacheValue = Arc<Vec<PathToDestination>>;

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
    length: usize,
    take: usize,
) -> Vec<PathToDestination>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey> + NetworkGraphView<NodeId = OffchainPublicKey>,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
{
    let raw = graph.simple_paths(src, dest, length, Some(take), HoprCostFn::new(length));

    let mut paths: Vec<(PathToDestination, f64)> = raw
        .into_iter()
        .filter(|(_, _, cost)| *cost > 0.0)
        .map(|(path, _, cost)| {
            // Drop the first element (src) — callers expect [intermediates..., dest].
            let trimmed = path.into_iter().skip(1).collect();
            (trimmed, cost)
        })
        .collect();

    // Sort by cost descending so the best path is first.
    paths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    paths.into_iter().map(|(p, _)| p).collect()
}

struct SelectorInner<G> {
    config: PathSelectorConfig,
    cache: moka::future::Cache<CacheKey, CacheValue>,
    graph: G,
}

#[cfg(feature = "runtime-tokio")]
impl<G> SelectorInner<G>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey> + NetworkGraphView<NodeId = OffchainPublicKey> + Send + Sync,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
{
    async fn refresh_for(&self, src: &OffchainPublicKey, dest: OffchainPublicKey, hops: usize) {
        let paths = compute_paths(&self.graph, src, &dest, hops + 1, self.config.max_cached_paths);
        if !paths.is_empty() {
            trace!(%dest, hops, count = paths.len(), "refreshing path cache");
            self.cache.insert((*src, dest, hops), Arc::new(paths)).await;
        }
    }

    async fn refresh_all(&self) {
        let keys: Vec<CacheKey> = self.cache.iter().map(|(k, _)| k).collect();
        debug!(count = keys.len(), "running path cache refresh sweep");
        for (src, dest, hops) in keys {
            self.refresh_for(&src, dest, hops).await;
        }
    }
}

/// A graph-backed path selector with a `moka` LRU/TTL cache.
///
/// **Default behaviour (no `runtime-tokio` feature)**: cache entries are populated
/// lazily on the first [`select_path`][GraphPathSelector::select_path] call for a given
/// `(destination, hops)` pair via moka's `optionally_get_with`, which is atomic and
/// prevents thundering-herd re-computation. When a TTL expires the next call
/// transparently recomputes the paths from the graph.
///
/// **With `runtime-tokio` feature**: additionally exposes
/// [`run_background_refresh`][GraphPathSelector::run_background_refresh], a future that
/// should be spawned as a long-lived background task. It periodically pre-warms the cache
/// for all reachable `(destination, hops)` pairs so that steady-state traffic is always
/// served from cache.
pub struct GraphPathSelector<G>(Arc<SelectorInner<G>>, OffchainPublicKey);

impl<G: Clone> Clone for GraphPathSelector<G> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), self.1)
    }
}

impl<G> GraphPathSelector<G>
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
    /// `me` is the node's own [`OffchainPublicKey`] — it is used as the path source
    /// when querying the graph and is excluded from returned path vectors.
    pub fn new(me: OffchainPublicKey, graph: G, config: PathSelectorConfig) -> Self {
        let cache = moka::future::Cache::builder()
            .max_capacity(config.max_cache_capacity)
            .time_to_live(config.cache_ttl)
            .build();

        Self(Arc::new(SelectorInner { graph, cache, config }), me)
    }

    /// Returns the node's own key that this selector uses as the path source.
    pub fn me(&self) -> &OffchainPublicKey {
        &self.1
    }
}

impl<G> PathSelector for GraphPathSelector<G>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + NetworkGraphView<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
{
    /// Select a path from `src` to `dest` using `hops` intermediate relays.
    ///
    /// Returns a `Vec<OffchainPublicKey>` containing the intermediate nodes
    /// **and** `dest` (`src` is excluded).
    ///
    /// On a cache miss (or after TTL expiry) the graph is queried via moka's
    /// `optionally_get_with`, which atomically computes and caches the result —
    /// preventing thundering-herd re-computation. Paths with no candidates are
    /// never cached, so the next call will retry immediately.
    ///
    async fn select_path(
        &self,
        src: OffchainPublicKey,
        dest: OffchainPublicKey,
        hops: usize,
    ) -> Result<Vec<OffchainPublicKey>> {
        let key = (src, dest, hops);
        let inner = Arc::clone(&self.0);
        let compute_inner = Arc::clone(&inner);
        let length = hops + 1;

        let paths = inner
            .cache
            .optionally_get_with(key, async move {
                trace!(%dest, hops, "path cache miss, computing from graph");
                let paths = compute_paths(
                    &compute_inner.graph,
                    &src,
                    &dest,
                    length,
                    compute_inner.config.max_cached_paths,
                );
                if paths.is_empty() { None } else { Some(Arc::new(paths)) }
            })
            .await
            .ok_or_else(|| {
                PathPlannerError::Path(PathError::PathNotFound(hops, src.to_peerid_str(), dest.to_peerid_str()))
            })?;

        // Randomly select among cached candidates for routing diversity.
        let idx = if paths.len() > 1 {
            hopr_crypto_random::random_integer(0, Some((paths.len() - 1) as u64)) as usize
        } else {
            0
        };
        trace!(%dest, hops, idx, "path selected");
        Ok(paths[idx].clone())
    }
}

/// Implements periodic background cache refresh for [`GraphPathSelector`].
///
/// The returned future pre-warms the path cache for all reachable
/// `(destination, hops)` pairs on a configurable schedule, so that
/// steady-state traffic is always served without a blocking graph query.
#[cfg(feature = "runtime-tokio")]
impl<G> BackgroundRefreshable for GraphPathSelector<G>
where
    G: NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + NetworkGraphView<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
    <G as NetworkGraphTraverse>::Observed: EdgeObservableRead + Send + 'static,
{
    fn run_background_refresh(&self) -> impl std::future::Future<Output = ()> + Send + 'static {
        let inner = Arc::clone(&self.0);
        let me = self.1;
        async move {
            let mut interval = tokio::time::interval(inner.config.refresh_period);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                inner.refresh_all(&me).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_api::graph::{
        NetworkGraphWrite,
        traits::{EdgeObservableWrite, EdgeWeightType},
    };
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
    use hopr_network_graph::ChannelGraph;
    use hopr_network_types::types::RoutingOptions;

    use crate::traits::PathSelector;

    use super::*;

    const SECRET_0: [u8; 32] = hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");
    const SECRET_1: [u8; 32] = hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a");
    const SECRET_2: [u8; 32] = hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299");
    const SECRET_3: [u8; 32] = hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e");
    const SECRET_4: [u8; 32] = hex!("cfc66f718ec66fb822391775d749d7a0d66b690927673634816b63339bc12a3c");

    fn pubkey(secret: &[u8; 32]) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(secret).expect("valid secret").public()
    }

    fn default_config() -> PathSelectorConfig {
        PathSelectorConfig {
            max_cache_capacity: 1000,
            cache_ttl: Duration::from_secs(60),
            #[cfg(feature = "runtime-tokio")]
            refresh_period: Duration::from_secs(30),
            max_cached_paths: 4,
        }
    }

    /// Mark an edge as fully ready for intermediate routing:
    /// connected, immediate probe success, intermediate probe + capacity.
    fn mark_edge_full(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });
    }

    /// Mark an edge as ready only as the last hop: connected + immediate probe.
    fn mark_edge_last(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
        });
    }

    // Helper: build a simple 2-hop graph me → hop → dest.
    fn two_hop_graph() -> (OffchainPublicKey, OffchainPublicKey, OffchainPublicKey, ChannelGraph) {
        let me = pubkey(&SECRET_0);
        let hop = pubkey(&SECRET_1);
        let dest = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        graph.add_node(hop);
        graph.add_node(dest);
        graph.add_edge(&me, &hop).unwrap();
        graph.add_edge(&hop, &dest).unwrap();
        mark_edge_full(&graph, &me, &hop);
        mark_edge_last(&graph, &hop, &dest);
        (me, hop, dest, graph)
    }

    #[tokio::test]
    async fn cache_miss_computes_and_caches_path() {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, default_config());

        // Cache should be empty initially.
        assert!(selector.0.cache.get(&(dest, 1)).await.is_none());

        let path = selector.select_path(me, dest, 1).await.expect("should find 1-hop path");
        assert!(!path.is_empty());
        assert_eq!(path.last(), Some(&dest));

        // Cache should now be populated.
        assert!(selector.0.cache.get(&(dest, 1)).await.is_some());
    }

    #[tokio::test]
    async fn cache_hit_returns_cached_path() {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, default_config());

        let path1 = selector.select_path(me, dest, 1).await.expect("first call");
        let path2 = selector.select_path(me, dest, 1).await.expect("second call");

        // Both calls must succeed and return the destination.
        assert_eq!(path1.last(), Some(&dest));
        assert_eq!(path2.last(), Some(&dest));
    }

    #[tokio::test]
    async fn no_path_returns_error() {
        let me = pubkey(&SECRET_0);
        let unreachable = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        // No edges at all.
        let selector = GraphPathSelector::new(me, graph, default_config());

        let err = selector.select_path(me, unreachable, 1).await;
        assert!(err.is_err(), "should error when destination is unreachable");
        assert!(
            matches!(err.unwrap_err(), PathPlannerError::Path(PathError::PathNotFound(..))),
            "error must be PathNotFound"
        );
    }

    #[tokio::test]
    async fn path_excludes_source() {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, default_config());

        let path = selector.select_path(me, dest, 1).await.expect("path");
        assert!(!path.contains(&me), "returned path must not contain the source node");
    }

    #[tokio::test]
    async fn multi_hop_path_has_correct_length() {
        // me → A → B → dest  (2 intermediate hops)
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let dest = pubkey(&SECRET_3);
        let graph = ChannelGraph::new(me);
        for n in [a, b, dest] {
            graph.add_node(n);
        }
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &b).unwrap();
        graph.add_edge(&b, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        // middle edge must also have capacity so the cost function can score it
        mark_edge_full(&graph, &a, &b);
        // b→dest is the last hop
        mark_edge_last(&graph, &b, &dest);

        let selector = GraphPathSelector::new(me, graph, default_config());
        let path = selector.select_path(me, dest, 2).await.expect("2-hop path");

        // Path should be [A, B, dest] — length 2 intermediates + dest = 3 elements.
        assert_eq!(
            path.len(),
            3,
            "2-hop path should have 3 elements (2 intermediates + destination)"
        );
        assert_eq!(path.last(), Some(&dest));
    }

    #[tokio::test]
    async fn one_hop_path_has_relay_and_destination() {
        // me → relay → dest  (1 intermediate hop)
        // The returned path must be [relay, dest] (source excluded).
        let (me, relay, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, default_config());
        let path = selector.select_path(me, dest, 1).await.expect("1-hop path");
        // Path = [relay, dest] — 2 elements
        assert_eq!(path.len(), 2, "1-hop path must contain relay + destination");
        assert_eq!(path.last(), Some(&dest));
        assert!(!path.contains(&me));
        // Suppress unused-variable warnings for the relay name — we only check endpoints.
        let _ = relay;
    }

    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn refresh_all_populates_cache_for_reachable_destinations() {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, default_config());

        // Pre-condition: cache is empty.
        assert!(selector.0.cache.get(&(dest, 1)).await.is_none());

        // Manually trigger a refresh sweep.
        selector.0.refresh_all(&me).await;

        // Post-condition: cache should now contain paths to `dest` at 1 hop.
        assert!(
            selector.0.cache.get(&(dest, 1)).await.is_some(),
            "cache should be populated after refresh_all"
        );
    }

    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn multiple_paths_cached_for_diamond_topology() {
        // me → a → dest,  me → b → dest
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let dest = pubkey(&SECRET_3);
        let graph = ChannelGraph::new(me);
        for n in [a, b, dest] {
            graph.add_node(n);
        }
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&me, &b).unwrap();
        graph.add_edge(&a, &dest).unwrap();
        graph.add_edge(&b, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &me, &b);
        mark_edge_last(&graph, &a, &dest);
        mark_edge_last(&graph, &b, &dest);

        let selector = GraphPathSelector::new(me, graph, default_config());
        // Trigger cache population.
        selector.0.refresh_all(&me).await;

        let cached = selector.0.cache.get(&(dest, 1)).await.expect("cache populated");
        assert_eq!(cached.len(), 2, "both paths through a and b should be cached");

        // Each cached path should end at dest.
        for p in cached.iter() {
            assert_eq!(p.last(), Some(&dest));
        }
    }

    #[tokio::test]
    async fn select_path_ignores_zero_cost_paths() {
        // Build a graph where destination is in graph but has no valid edges.
        let me = pubkey(&SECRET_0);
        let dest = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest).unwrap();
        // No observations → cost function will return negative cost → pruned.

        let selector = GraphPathSelector::new(me, graph, default_config());
        let result = selector.select_path(me, dest, 1).await;
        assert!(
            result.is_err(),
            "edge with no observations should produce no valid path"
        );
    }

    #[tokio::test]
    async fn unreachable_at_requested_hop_count_returns_error() {
        // Graph only has a 1-hop route; requesting 2 hops should fail.
        let me = pubkey(&SECRET_0);
        let dest = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest).unwrap();
        mark_edge_full(&graph, &me, &dest);

        let selector = GraphPathSelector::new(me, graph, default_config());
        let result = selector.select_path(me, dest, 2).await;
        assert!(result.is_err(), "no 2-hop path should exist for a direct edge");
    }

    #[tokio::test]
    async fn selector_clone_shares_cache() {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, default_config());
        let clone = selector.clone();

        // Populate cache via original.
        let _ = selector.select_path(me, dest, 1).await.expect("path");

        // Clone should see the same cache entry.
        assert!(
            clone.0.cache.get(&(dest, 1)).await.is_some(),
            "cloned selector must share the same cache"
        );
    }

    #[tokio::test]
    async fn path_with_five_nodes_supports_up_to_max_hops() {
        // me → a → b → c → dest  (3 intermediate hops = MAX_INTERMEDIATE_HOPS)
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let c = pubkey(&SECRET_3);
        let dest = pubkey(&SECRET_4);
        let graph = ChannelGraph::new(me);
        for n in [a, b, c, dest] {
            graph.add_node(n);
        }
        graph.add_edge(&me, &a).unwrap();
        graph.add_edge(&a, &b).unwrap();
        graph.add_edge(&b, &c).unwrap();
        graph.add_edge(&c, &dest).unwrap();
        mark_edge_full(&graph, &me, &a);
        mark_edge_full(&graph, &a, &b);
        mark_edge_full(&graph, &b, &c);
        mark_edge_last(&graph, &c, &dest);

        let selector = GraphPathSelector::new(me, graph, default_config());
        let path = selector
            .select_path(me, dest, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .await
            .expect("3-hop path");

        assert_eq!(path.len(), 4, "3-hop path has 3 intermediates + destination = 4 nodes");
        assert_eq!(path.last(), Some(&dest));
        assert!(!path.contains(&me));
    }
}
