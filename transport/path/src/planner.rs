use std::{sync::Arc, time::Duration};

use futures::{StreamExt as _, TryStreamExt, stream::FuturesUnordered};
use hopr_api::chain::{ChainKeyOperations, ChainPathResolver, ChainReadChannelOperations};
use hopr_crypto_packet::prelude::*;
use hopr_protocol_hopr::{FoundSurb, SurbStore};
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_types::internal::path::Path;
use hopr_types::{
    crypto::{crypto_traits::Randomizable, types::OffchainPublicKey},
    internal::{errors::PathError, prelude::*},
    primitive::traits::ToHex,
};
use tracing::trace;

use crate::{
    errors::{PathPlannerError, Result},
    traits::{BackgroundPathCacheRefreshable, PathSelector},
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PATH_LENGTH: hopr_metrics::SimpleHistogram = hopr_metrics::SimpleHistogram::new(
        "hopr_path_length",
        "Distribution of number of hops of sent messages",
        vec![0.0, 1.0, 2.0, 3.0, 4.0]
    ).unwrap();
}

/// Configuration for [`PathPlanner`]'s internal path cache.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault)]
pub struct PathPlannerConfig {
    /// Maximum number of `(source, destination, options)` entries in the path cache.
    #[default = 10_000]
    pub max_cache_capacity: u64,
    /// Time-to-live for a cached path list.  When an entry expires the next
    /// [`PathPlanner::resolve_routing`] call transparently recomputes it (lazy refresh).
    #[default(Duration::from_secs(60))]
    pub cache_ttl: Duration,
    /// Period between proactive background cache-refresh sweeps.
    #[default(Duration::from_secs(30))]
    pub refresh_period: Duration,
    /// Maximum number of candidate paths the selector may return per query.
    /// All returned candidates are validated and cached.
    #[default = 8]
    pub max_cached_paths: usize,
}

/// Cache key for the path planner: `(source, destination, hops)`.
///
/// Only the `Hops` variant of [`RoutingOptions`] is cached (explicit intermediate
/// paths bypass the cache), so the key stores the hop count as a plain `u32`.
type PlannerCacheKey = (NodeId, NodeId, u32);
type PlannerCacheValue = Arc<Vec<(ValidatedPath, f64)>>;

/// Path planner that resolves [`DestinationRouting`] to [`ResolvedTransportRouting`].
///
/// The planner delegates path *discovery* to any [`PathSelector`] implementation and
/// owns the `moka` cache of fully-validated [`ValidatedPath`] objects, keyed by
/// `(source: NodeId, destination: NodeId, hops: u32)`.
///
/// On a cache miss the planner calls the selector, validates every candidate against
/// the chain resolver, and stores `Arc<Vec<ValidatedPath>>` in the cache.
/// On a cache hit a random candidate is picked for routing diversity.
///
/// A background sweep ([`PathPlanner::background_refresh`]) can be spawned to
/// proactively re-warm the cache for all previously-seen keys.
#[derive(Clone)]
pub struct PathPlanner<Surb, R, S> {
    me: OffchainPublicKey,
    pub surb_store: Surb,
    resolver: Arc<R>,
    selector: Arc<S>,
    cache: moka::future::Cache<PlannerCacheKey, PlannerCacheValue>,
    refresh_period: Duration,
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
                trace!(?explicit_path, "resolving an explicit intermediate path");
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
                trace!(hops = hops_usize, "resolving path via planner cache");

                let src_key = self.resolve_node_id_to_offchain_key(&source).await?;
                let dest_key = self.resolve_node_id_to_offchain_key(&destination).await?;

                let cache_key: PlannerCacheKey = (source, destination, u32::from(hops));

                let resolver = self.resolver.clone();
                let selector = self.selector.clone();

                let paths = self
                    .cache
                    .try_get_with(cache_key, async move {
                        trace!(hops = hops_usize, "path cache miss, querying selector");
                        let candidates = selector.select_path(src_key, dest_key, hops_usize)?;

                        let chain_resolver = ChainPathResolver::from(&*resolver);
                        let mut valid_paths: Vec<(ValidatedPath, f64)> = Vec::new();
                        for (candidate, cost) in candidates {
                            let node_ids: Vec<NodeId> = candidate.into_iter().map(NodeId::Offchain).collect::<Vec<_>>();
                            if let Ok(vp) = ValidatedPath::new(source, node_ids, &chain_resolver).await {
                                valid_paths.push((vp, cost));
                            }
                        }

                        if valid_paths.is_empty() {
                            return Err(PathPlannerError::Path(PathError::PathNotFound(
                                hops_usize,
                                src_key.to_hex(),
                                dest_key.to_hex(),
                            )));
                        }

                        Ok(Arc::new(valid_paths))
                    })
                    .await
                    .map_err(PathPlannerError::CacheError)?;

                hopr_statistics::WeightedCollection::new(paths.to_vec())
                    .pick_one()
                    .ok_or_else(|| {
                        PathPlannerError::Path(PathError::PathNotFound(
                            hops_usize,
                            src_key.to_hex(),
                            dest_key.to_hex(),
                        ))
                    })?
            }
        };

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            hopr_metrics::SimpleHistogram::observe(&METRIC_PATH_LENGTH, (path.num_hops() - 1) as f64);
        }

        trace!(%path, "validated resolved path");
        Ok(path)
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
                let forward_path = self
                    .resolve_path(NodeId::Offchain(self.me), *destination, forward_options)
                    .await?;

                let return_paths = if let Some(return_options) = return_options {
                    let num_possible_surbs = HoprPacket::max_surbs_with_message(size_hint).min(max_surbs);
                    trace!(
                        %destination,
                        %num_possible_surbs,
                        data_len = size_hint,
                        max_surbs,
                        "resolving packet return paths"
                    );

                    (0..num_possible_surbs)
                        .map(|_| self.resolve_path(*destination, NodeId::Offchain(self.me), return_options.clone()))
                        .collect::<FuturesUnordered<_>>()
                        .try_collect::<Vec<ValidatedPath>>()
                        .await?
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
                } =
                    self.surb_store.find_surb(matcher).await.ok_or_else(|| {
                        PathPlannerError::Surb(format!("no surb for pseudonym {}", matcher.pseudonym()))
                    })?;
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

        // run at a non-zero interval
        futures_time::stream::interval(futures_time::time::Duration::from_millis(
            refresh_period.as_millis() as u64 + 1u64,
        ))
        .for_each(move |_| {
            let cache = cache.clone();
            let resolver = resolver.clone();
            let selector = selector.clone();

            async move {
                for (key, _) in cache.iter() {
                    let (src, dest, hops_u32) = {
                        let k = key.as_ref();
                        (k.0, k.1, k.2)
                    };

                    if hops_u32 == 0 {
                        continue;
                    }
                    let hops_usize = hops_u32 as usize;

                    let resolve_key = |node: NodeId| {
                        let resolver = resolver.clone();

                        async move {
                            match node {
                                NodeId::Offchain(k) => Some(k),
                                NodeId::Chain(addr) => ChainPathResolver::from(&*resolver)
                                    .resolve_transport_address(&addr)
                                    .await
                                    .ok()
                                    .flatten(),
                            }
                        }
                    };

                    if let (Some(src_key), Some(dest_key)) = (resolve_key(src).await, resolve_key(dest).await)
                        && let Ok(candidates) = selector.select_path(src_key, dest_key, hops_usize)
                    {
                        let chain_resolver = ChainPathResolver::from(&*resolver);
                        let mut valid_paths: Vec<(ValidatedPath, f64)> = Vec::new();
                        for (candidate, cost) in candidates {
                            let node_ids: Vec<NodeId> = candidate.into_iter().map(NodeId::Offchain).collect::<Vec<_>>();
                            if let Ok(vp) = ValidatedPath::new(src, node_ids, &chain_resolver).await {
                                valid_paths.push((vp, cost));
                            }
                        }

                        if !valid_paths.is_empty() {
                            cache.insert((src, dest, hops_u32), Arc::new(valid_paths)).await;
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use async_trait::async_trait;
    use bimap::BiMap;
    use futures::stream::{self, BoxStream};
    use hex_literal::hex;
    use hopr_api::{
        chain::{ChainKeyOperations, ChainReadChannelOperations, ChannelSelector, HoprKeyIdent},
        graph::{
            NetworkGraphWrite,
            traits::{EdgeObservableWrite, EdgeWeightType},
        },
    };
    use hopr_network_graph::ChannelGraph;
    use hopr_types::{
        crypto::prelude::{Keypair, OffchainKeypair},
        internal::channels::{ChannelEntry, ChannelStatus, generate_channel_id},
        primitive::prelude::*,
    };

    use super::*;
    use crate::selector::HoprGraphPathSelector;

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
            self.channels
                .push(ChannelEntry::new(src, dst, 100u32.into(), 1, ChannelStatus::Open, 1));
            self
        }
    }

    #[async_trait]
    impl ChainKeyOperations for TestChainApi {
        type Error = TestError;
        type Mapper = Mapper;

        async fn chain_key_to_packet_key(
            &self,
            chain: &Address,
        ) -> std::result::Result<Option<OffchainPublicKey>, TestError> {
            Ok(self.key_addr_map.get_by_right(chain).copied())
        }

        async fn packet_key_to_chain_key(
            &self,
            packet: &OffchainPublicKey,
        ) -> std::result::Result<Option<Address>, TestError> {
            Ok(self.key_addr_map.get_by_left(packet).copied())
        }

        fn key_id_mapper_ref(&self) -> &Self::Mapper {
            &self.id_mapper
        }
    }

    #[async_trait]
    impl ChainReadChannelOperations for TestChainApi {
        type Error = TestError;

        fn me(&self) -> &Address {
            &self.me
        }

        async fn channel_by_id(&self, channel_id: &ChannelId) -> std::result::Result<Option<ChannelEntry>, TestError> {
            Ok(self
                .channels
                .iter()
                .find(|c| generate_channel_id(&c.source, &c.destination) == *channel_id)
                .cloned())
        }

        async fn stream_channels<'a>(
            &'a self,
            _selector: ChannelSelector,
        ) -> std::result::Result<BoxStream<'a, ChannelEntry>, TestError> {
            Ok(Box::pin(stream::iter(self.channels.clone())))
        }
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

    // ── graph helpers ──────────────────────────────────────────────────────────
    fn mark_edge_full(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        use hopr_api::graph::traits::EdgeWeightType;
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });
    }

    fn mark_edge_last(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Intermediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });
    }

    fn small_config() -> PathPlannerConfig {
        PathPlannerConfig {
            max_cache_capacity: 100,
            cache_ttl: std::time::Duration::from_secs(60),
            refresh_period: std::time::Duration::from_secs(60),
            max_cached_paths: 2,
        }
    }

    // ── test: zero-hop path ───────────────────────────────────────────────────

    #[tokio::test]
    async fn zero_hop_path_should_bypass_selector() {
        let me = pubkey(&SECRET_ME);
        let dest = pubkey(&SECRET_DEST);

        // Build empty graph (no edges) — selector would fail if called.
        let graph = ChannelGraph::new(me);
        let selector = HoprGraphPathSelector::new(me, graph, small_config().max_cached_paths);

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
        mark_edge_last(&graph, &a, &dest);

        let selector = HoprGraphPathSelector::new(me, graph, small_config().max_cached_paths);

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
        let selector = HoprGraphPathSelector::new(me, graph, small_config().max_cached_paths);

        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(me_addr(), a_addr())
            .with_open_channel(a_addr(), dest_addr());

        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();
        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        use hopr_types::primitive::prelude::BoundedVec;
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
        let selector = HoprGraphPathSelector::new(me, graph, small_config().max_cached_paths);
        let chain_api = TestChainApi::new(me, me_addr(), vec![]);
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();

        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        use hopr_types::internal::routing::SurbMatcher;
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
        mark_edge_last(&graph, &a, &dest);

        let selector = HoprGraphPathSelector::new(me, graph, small_config().max_cached_paths);
        let chain_api = TestChainApi::new(me, me_addr(), vec![(a, a_addr()), (dest, dest_addr())])
            .with_open_channel(me_addr(), a_addr())
            .with_open_channel(a_addr(), dest_addr());
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();
        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());

        let cache_key: PlannerCacheKey = (NodeId::Offchain(me), NodeId::Offchain(dest), 1);

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
        assert_eq!(paths[0].0.num_hops(), 2, "path should have 2 hops [a, dest]");
        assert!(paths[0].1 > 0.0, "cost should be positive");
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
        mark_edge_last(&graph, &a, &dest);

        let selector = HoprGraphPathSelector::new(me, graph, small_config().max_cached_paths);
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
        let selector = HoprGraphPathSelector::new(me, graph, small_config().max_cached_paths);
        let chain_api = TestChainApi::new(me, me_addr(), vec![]);
        let surb_store = hopr_protocol_hopr::MemorySurbStore::default();

        let planner = PathPlanner::new(me, surb_store, chain_api, selector, small_config());
        // Just ensure it compiles and produces a future.
        let _future = planner.run_background_refresh();
    }
}
