use std::{sync::Arc, time::Duration};

use hopr_api::graph::{NetworkGraphTraverse, NetworkGraphView, costs::HoprCostFn, traits::EdgeObservableRead};
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::errors::PathError;
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
///
/// The complexity of this function call is at least the complexity of the
/// underlying graph traversal, which may be expensive for large graphs and
/// high `length` values.
///
/// For real networks, a simulation/benchmark should be performed to identify
/// whether this function can be blocking, e.g. for an async executor.
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

struct SelectorInner<G> {
    config: PathSelectorConfig,
    cache: moka::future::Cache<CacheKey, CacheValue>,
    graph: G,
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
    /// Select a path from `src` to `dest` using `hops` HOPR relays.
    ///
    /// The HOPR hop count represents the number of intermediate relays,
    /// so the total path length is `hops + 1`.
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

        let paths = inner
            .cache
            .optionally_get_with(key, async move {
                trace!(%dest, hops, "path cache miss, computing from graph");
                let paths = compute_paths(
                    &compute_inner.graph,
                    &src,
                    &dest,
                    hops + 1,
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
        async move {
            let mut interval = tokio::time::interval(inner.config.refresh_period);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                for (key, _) in inner.cache.iter() {
                    let src = key.0;
                    let dest = key.1;
                    let hops = key.2;

                    let paths = compute_paths(&inner.graph, &src, &dest, hops + 1, inner.config.max_cached_paths);
                    if !paths.is_empty() {
                        trace!(%dest, hops, count = paths.len(), "refreshing path cache");
                        inner.cache.insert((src, dest, hops), Arc::new(paths)).await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
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

    fn test_config() -> PathSelectorConfig {
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
    async fn cache_miss_should_compute_and_cache_path() -> anyhow::Result<()> {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, test_config());

        assert!(selector.0.cache.get(&(me, dest, 1)).await.is_none());
        assert!(selector.0.cache.get(&(dest, me, 1)).await.is_none());

        let path = selector.select_path(me, dest, 1).await.context("forward path")?;
        assert!(!path.is_empty());
        assert_eq!(path.last(), Some(&dest));
        assert!(selector.0.cache.get(&(me, dest, 1)).await.is_some());

        let rev = selector.select_path(dest, me, 1).await.context("reverse path")?;
        assert!(!rev.is_empty());
        assert_eq!(rev.last(), Some(&me));
        assert!(selector.0.cache.get(&(dest, me, 1)).await.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn cache_hit_should_return_cached_path() -> anyhow::Result<()> {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, test_config());

        let p1 = selector.select_path(me, dest, 1).await.context("forward call 1")?;
        let p2 = selector.select_path(me, dest, 1).await.context("forward call 2")?;
        assert_eq!(p1.last(), Some(&dest));
        assert_eq!(p2.last(), Some(&dest));

        let r1 = selector.select_path(dest, me, 1).await.context("reverse call 1")?;
        let r2 = selector.select_path(dest, me, 1).await.context("reverse call 2")?;
        assert_eq!(r1.last(), Some(&me));
        assert_eq!(r2.last(), Some(&me));

        Ok(())
    }

    #[tokio::test]
    async fn unreachable_dest_should_return_error() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let unreachable = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        // No edges at all — neither direction has a path.
        let selector = GraphPathSelector::new(me, graph, test_config());

        let fwd = selector.select_path(me, unreachable, 1).await;
        assert!(fwd.is_err(), "forward: should error when destination is unreachable");
        assert!(matches!(
            fwd.unwrap_err(),
            PathPlannerError::Path(PathError::PathNotFound(..))
        ));

        let rev = selector.select_path(unreachable, me, 1).await;
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
        let selector = GraphPathSelector::new(me, graph, test_config());

        let fwd = selector.select_path(me, dest, 1).await.context("forward path")?;
        assert!(!fwd.contains(&me), "forward path must not contain the source");

        let rev = selector.select_path(dest, me, 1).await.context("reverse path")?;
        assert!(!rev.contains(&dest), "reverse path must not contain the source");

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

        let selector = GraphPathSelector::new(me, graph, test_config());

        let fwd = selector.select_path(me, dest, 2).await.context("forward 2-hop path")?;
        assert_eq!(fwd.len(), 3, "forward 2-hop path: [A, B, dest]");
        assert_eq!(fwd.last(), Some(&dest));

        let rev = selector.select_path(dest, me, 2).await.context("reverse 2-hop path")?;
        assert_eq!(rev.len(), 3, "reverse 2-hop path: [B, A, me]");
        assert_eq!(rev.last(), Some(&me));

        Ok(())
    }

    #[tokio::test]
    async fn one_hop_path_should_include_relay_and_destination() -> anyhow::Result<()> {
        // Bidirectional: me ↔ relay ↔ dest
        let (me, relay, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, test_config());

        let fwd = selector.select_path(me, dest, 1).await.context("forward 1-hop path")?;
        assert_eq!(fwd.len(), 2, "forward: [relay, dest]");
        assert_eq!(fwd.last(), Some(&dest));
        assert!(!fwd.contains(&me));

        let rev = selector.select_path(dest, me, 1).await.context("reverse 1-hop path")?;
        assert_eq!(rev.len(), 2, "reverse: [relay, me]");
        assert_eq!(rev.last(), Some(&me));
        assert!(!rev.contains(&dest));

        let _ = relay;
        Ok(())
    }

    #[tokio::test]
    async fn diamond_topology_should_cache_multiple_paths() -> anyhow::Result<()> {
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

        let selector = GraphPathSelector::new(me, graph, test_config());

        selector.select_path(me, dest, 1).await.context("forward path")?;
        let fwd_cached = selector.0.cache.get(&(me, dest, 1)).await.context("forward cache")?;
        assert_eq!(fwd_cached.len(), 2, "forward: both paths via a and b should be cached");
        for p in fwd_cached.iter() {
            assert_eq!(p.last(), Some(&dest));
        }

        selector.select_path(dest, me, 1).await.context("reverse path")?;
        let rev_cached = selector.0.cache.get(&(dest, me, 1)).await.context("reverse cache")?;
        assert_eq!(rev_cached.len(), 2, "reverse: both paths via a and b should be cached");
        for p in rev_cached.iter() {
            assert_eq!(p.last(), Some(&me));
        }

        Ok(())
    }

    #[tokio::test]
    async fn zero_cost_paths_should_be_ignored() -> anyhow::Result<()> {
        // Graph has edges in both directions but no observations → all costs zero → pruned.
        let me = pubkey(&SECRET_0);
        let dest = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(dest);
        graph.add_edge(&me, &dest).unwrap();
        graph.add_edge(&dest, &me).unwrap();
        // No observations → cost function will return non-positive cost → pruned.

        let selector = GraphPathSelector::new(me, graph, test_config());
        assert!(
            selector.select_path(me, dest, 1).await.is_err(),
            "forward: edge with no observations should produce no valid path"
        );
        assert!(
            selector.select_path(dest, me, 1).await.is_err(),
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

        let selector = GraphPathSelector::new(me, graph, test_config());
        assert!(
            selector.select_path(me, dest, 2).await.is_err(),
            "forward: no 2-hop path should exist for a direct edge"
        );
        assert!(
            selector.select_path(dest, me, 2).await.is_err(),
            "reverse: no 2-hop path should exist for a direct edge"
        );
        Ok(())
    }

    #[tokio::test]
    async fn selector_clone_should_share_cache() -> anyhow::Result<()> {
        let (me, _hop, dest, graph) = two_hop_graph();
        let selector = GraphPathSelector::new(me, graph, test_config());
        let clone = selector.clone();

        selector.select_path(me, dest, 1).await.context("forward path")?;
        selector.select_path(dest, me, 1).await.context("reverse path")?;

        assert!(
            clone.0.cache.get(&(me, dest, 1)).await.is_some(),
            "clone must share the forward cache entry"
        );
        assert!(
            clone.0.cache.get(&(dest, me, 1)).await.is_some(),
            "clone must share the reverse cache entry"
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

        let selector = GraphPathSelector::new(me, graph, test_config());

        let fwd = selector
            .select_path(me, dest, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .await
            .context("forward 3-hop path")?;
        assert_eq!(fwd.len(), 4, "forward: [a, b, c, dest]");
        assert_eq!(fwd.last(), Some(&dest));
        assert!(!fwd.contains(&me));

        let rev = selector
            .select_path(dest, me, RoutingOptions::MAX_INTERMEDIATE_HOPS)
            .await
            .context("reverse 3-hop path")?;
        assert_eq!(rev.len(), 4, "reverse: [c, b, a, me]");
        assert_eq!(rev.last(), Some(&me));
        assert!(!rev.contains(&dest));

        Ok(())
    }
}
