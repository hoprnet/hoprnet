use futures::{StreamExt, stream::BoxStream};
use futures_concurrency::stream::StreamExt as _;
use hopr_api::{
    ct::{CoverTrafficGeneration, ProbeRouting, ProbingTrafficGeneration},
    graph::{NetworkGraphTraverse, NetworkGraphView, costs::HoprForwardCostFn, traits::EdgeObservableRead},
    types::{
        crypto::types::OffchainPublicKey,
        crypto_random::Randomizable,
        internal::{
            NodeId,
            protocol::HoprPseudonym,
            routing::{DestinationRouting, RoutingOptions},
        },
    },
};
use rand::RngExt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Configuration for the probing mechanism
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
pub struct ProberConfig {
    /// The delay between individual probing rounds for neighbor discovery
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_probing_interval", with = "humantime_serde")
    )]
    #[default(default_probing_interval())]
    pub interval: std::time::Duration,

    /// Weight for staleness factor in probe priority (0.0–1.0).
    ///
    /// Higher values prioritize probing edges that haven't been measured recently.
    #[cfg_attr(feature = "serde", serde(default = "default_staleness_weight"))]
    #[default(default_staleness_weight())]
    pub staleness_weight: f64,

    /// Weight for inverse quality factor in probe priority (0.0–1.0).
    ///
    /// Higher values prioritize probing edges with poor quality scores.
    #[cfg_attr(feature = "serde", serde(default = "default_quality_weight"))]
    #[default(default_quality_weight())]
    pub quality_weight: f64,

    /// Base priority ensuring all peers have a nonzero chance of being probed (0.0–1.0).
    #[cfg_attr(feature = "serde", serde(default = "default_base_priority"))]
    #[default(default_base_priority())]
    pub base_priority: f64,

    /// TTL for the cached weighted shuffle order.
    ///
    /// When expired, the graph is re-traversed and a new priority-ordered shuffle is computed.
    #[cfg_attr(feature = "serde", serde(default = "default_shuffle_ttl", with = "humantime_serde"))]
    #[default(default_shuffle_ttl())]
    pub shuffle_ttl: std::time::Duration,
}

/// Delay before repeating immediate probing rounds, should include enough time to traverse NATs
const DEFAULT_REPEATED_PROBING_DELAY: std::time::Duration = std::time::Duration::from_secs(30);

#[inline]
const fn default_staleness_weight() -> f64 {
    0.4
}

#[inline]
const fn default_quality_weight() -> f64 {
    0.3
}

#[inline]
const fn default_base_priority() -> f64 {
    0.3
}

#[inline]
const fn default_shuffle_ttl() -> std::time::Duration {
    std::time::Duration::from_secs(600)
}

#[inline]
const fn default_probing_interval() -> std::time::Duration {
    DEFAULT_REPEATED_PROBING_DELAY
}

/// Maximum staleness (in seconds) used to cap the staleness factor.
const MAX_STALENESS_SECS: f64 = 3600.0;

/// Computes the probing priority for an immediate neighbor edge.
///
/// Higher values mean the peer should be probed sooner. Combines:
/// - **Staleness**: time since the edge was last measured (capped at [`MAX_STALENESS_SECS`])
/// - **Inverse quality**: `1.0 - score`, so worse edges get higher priority
/// - **Base**: ensures even well-measured, recently-probed peers get some chance
///
/// Peers with no edge observations receive maximum priority.
fn immediate_probe_priority(
    score: f64,
    last_update: std::time::Duration,
    now: std::time::Duration,
    cfg: &ProberConfig,
) -> f64 {
    let staleness_secs = if last_update.is_zero() {
        MAX_STALENESS_SECS
    } else {
        now.saturating_sub(last_update).as_secs_f64().min(MAX_STALENESS_SECS)
    };
    let normalized_staleness = staleness_secs / MAX_STALENESS_SECS;
    let inverse_quality = 1.0 - score.clamp(0.0, 1.0);

    cfg.staleness_weight * normalized_staleness + cfg.quality_weight * inverse_quality + cfg.base_priority
}

/// Performs a weighted random shuffle using the Efraimidis-Spirakis algorithm.
///
/// Each item is assigned a key `random()^(1/weight)` and the items are sorted
/// by descending key. Items with higher weights appear earlier with higher
/// probability. All items retain a nonzero chance of appearing in any position.
///
/// Items with weight ≤ 0 are placed at the end.
fn weighted_shuffle<T>(items: Vec<(T, f64)>) -> Vec<T> {
    let mut rng = rand::rng();

    let mut keyed: Vec<(T, f64)> = items
        .into_iter()
        .map(|(item, weight)| {
            let key = if weight > 0.0 {
                let u: f64 = rng.random_range(f64::EPSILON..1.0);
                u.powf(1.0 / weight)
            } else {
                0.0
            };
            (item, key)
        })
        .collect();

    keyed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    keyed.into_iter().map(|(item, _)| item).collect()
}

pub struct FullNetworkDiscovery<U> {
    me: OffchainPublicKey,
    cfg: ProberConfig,
    graph: U,
}

impl<U> FullNetworkDiscovery<U> {
    pub fn new(me: OffchainPublicKey, cfg: ProberConfig, graph: U) -> Self {
        Self { me, cfg, graph }
    }
}

impl<U> CoverTrafficGeneration for FullNetworkDiscovery<U>
where
    U: NetworkGraphTraverse<NodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
{
    fn build(&self) -> BoxStream<'static, DestinationRouting> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "noise")] {
                let cfg = self.cfg;
                let me = self.me;
                let graph_intermediates = self.graph.clone();

                let junk = futures::stream::repeat(futures::stream::iter([2usize, 3, 4]))
                    .flatten()
                    .filter_map(move |edge_count| async move {
                        hopr_async_runtime::prelude::sleep(cfg.interval).await;
                        std::num::NonZeroUsize::new(edge_count)
                    })
                    .flat_map(move |edge_count| {
                        let paths = graph_intermediates.simple_paths(
                            &me,
                            &me,
                            edge_count.get(),
                            Some(100),
                            HoprForwardCostFn::new(edge_count),
                        );

                        // Weighted shuffle: lower cost → higher probe priority for cover traffic
                        let weighted: Vec<_> = paths
                            .into_iter()
                            .map(|(path, _path_id, cost)| {
                                let priority = (1.0 - cost).max(0.0) + cfg.base_priority;
                                (path, priority)
                            })
                            .collect();

                        futures::stream::iter(weighted_shuffle(weighted))
                    })
                    .filter_map(move |path| {
                        let me_node: NodeId = me.into();
                        let path: Vec<NodeId> = path.into_iter().map(NodeId::from).collect();

                        let routing =
                            hopr_api::network::BoundedVec::try_from(path)
                                .ok()
                                .map(|path| DestinationRouting::Forward {
                                    destination: Box::new(me_node),
                                    pseudonym: Some(HoprPseudonym::random()),
                                    forward_options: RoutingOptions::IntermediatePath(path),
                                    return_options: None,
                                });

                        futures::future::ready(routing)
                    });

                junk.boxed()
            } else {
                // Cover traffic not generating any data
                Box::pin(futures::stream::empty())
            }
        }
    }
}

impl<U> ProbingTrafficGeneration for FullNetworkDiscovery<U>
where
    U: NetworkGraphView<NodeId = OffchainPublicKey>
        + NetworkGraphTraverse<NodeId = OffchainPublicKey, Observed = hopr_network_graph::Observations>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn build(&self) -> BoxStream<'static, ProbeRouting> {
        let cfg = self.cfg;
        let graph = self.graph.clone();
        let graph_intermediates = self.graph.clone();
        let me = self.me;

        // Cached weighted shuffle with TTL for immediate neighbor probing.
        // When the TTL expires, the graph is re-traversed and a new
        // priority-ordered shuffle is computed.
        let cached_shuffle: std::sync::Arc<futures::lock::Mutex<(Vec<ProbeRouting>, std::time::Instant)>> =
            std::sync::Arc::new(futures::lock::Mutex::new((Vec::new(), std::time::Instant::now())));

        let immediates = futures::stream::repeat(()).filter_map(move |_| {
            let graph = graph.clone();
            let cached_shuffle = cached_shuffle.clone();

            async move {
                hopr_async_runtime::prelude::sleep(cfg.interval).await;

                let mut guard = cached_shuffle.lock().await;
                let (ref mut cached_items, ref mut created_at): (Vec<ProbeRouting>, std::time::Instant) = *guard;

                // Return the next item from the cache if not expired and not empty
                if !cached_items.is_empty() && created_at.elapsed() < cfg.shuffle_ttl {
                    return Some(cached_items.remove(0));
                }

                // Re-traverse the graph and compute a new weighted shuffle
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();

                let nodes: Vec<OffchainPublicKey> = graph
                    .nodes()
                    .filter(|peer| futures::future::ready(peer != &me))
                    .collect()
                    .await;

                let weighted: Vec<(OffchainPublicKey, f64)> = nodes
                    .into_iter()
                    .map(|peer| {
                        let priority = match graph.edge(&me, &peer) {
                            Some(obs) => immediate_probe_priority(obs.score(), obs.last_update(), now, &cfg),
                            None => immediate_probe_priority(0.0, std::time::Duration::ZERO, now, &cfg),
                        };
                        (peer, priority)
                    })
                    .collect();

                let shuffled: Vec<ProbeRouting> = weighted_shuffle(weighted)
                    .into_iter()
                    .map(|peer| {
                        ProbeRouting::Neighbor(DestinationRouting::Forward {
                            destination: Box::new(peer.into()),
                            pseudonym: Some(HoprPseudonym::random()),
                            forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                            return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                        })
                    })
                    .collect();

                *created_at = std::time::Instant::now();
                *cached_items = shuffled;

                if cached_items.is_empty() {
                    None
                } else {
                    Some(cached_items.remove(0))
                }
            }
        });

        let me = self.me;

        // 2, 3 and 4 edges => 1-, 2- and 3-hops in the HOPR protocol
        let intermediates = futures::stream::repeat(futures::stream::iter([2usize, 3, 4]))
            .flatten()
            .filter_map(move |edge_count| async move {
                hopr_async_runtime::prelude::sleep(cfg.interval).await;
                std::num::NonZeroUsize::new(edge_count)
            })
            .flat_map(move |edge_count| {
                let paths = graph_intermediates.simple_paths(
                    &me,
                    &me,
                    edge_count.get(),
                    Some(100),
                    HoprForwardCostFn::new(edge_count),
                );

                // Weighted shuffle: lower cost = worse quality = higher probe priority
                let weighted: Vec<_> = paths
                    .into_iter()
                    .map(|(path, path_id, cost)| {
                        let priority = (1.0 - cost).max(0.0) + cfg.base_priority;
                        ((path, path_id), priority)
                    })
                    .collect();

                futures::stream::iter(weighted_shuffle(weighted))
            })
            .filter_map(move |(path, path_id)| {
                let me_node: NodeId = me.into();
                let path: Vec<NodeId> = path.into_iter().map(NodeId::from).collect();

                let routing = hopr_api::network::BoundedVec::try_from(path).ok().map(|path| {
                    ProbeRouting::Looping((
                        DestinationRouting::Forward {
                            destination: Box::new(me_node),
                            pseudonym: Some(HoprPseudonym::random()),
                            forward_options: RoutingOptions::IntermediatePath(path),
                            return_options: None,
                        },
                        path_id,
                    ))
                });

                futures::future::ready(routing)
            });

        immediates.merge(intermediates).boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use futures::{StreamExt, pin_mut};
    use hopr_api::{
        OffchainKeypair,
        graph::{NetworkGraphUpdate, NetworkGraphWrite},
        types::{crypto::keypairs::Keypair, internal::NodeId},
    };
    use hopr_network_graph::ChannelGraph;
    use tokio::time::timeout;

    use super::*;

    const TINY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Node {
        pub id: OffchainPublicKey,
    }

    impl From<Node> for OffchainPublicKey {
        fn from(node: Node) -> Self {
            node.id
        }
    }

    lazy_static::lazy_static! {
        static ref RANDOM_PEERS: HashSet<Node> = (1..10).map(|_| {
            Node {
                id: OffchainPublicKey::from_privkey(&hopr_api::types::crypto_random::random_bytes::<32>()).unwrap(),
            }
        }).collect::<HashSet<_>>();
    }

    #[tokio::test]
    async fn peers_should_not_be_passed_if_none_are_present() -> anyhow::Result<()> {
        let me = OffchainKeypair::random().public().clone();
        let channel_graph = Arc::new(ChannelGraph::new(me));

        let prober = FullNetworkDiscovery::new(me, Default::default(), channel_graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn peers_should_have_randomized_order() -> anyhow::Result<()> {
        let me = OffchainKeypair::random().public().clone();
        let channel_graph = Arc::new(ChannelGraph::new(me));

        for node in RANDOM_PEERS.iter() {
            channel_graph.record_node(node.clone());
        }

        let prober = FullNetworkDiscovery::new(
            me,
            ProberConfig {
                interval: std::time::Duration::from_millis(1),
                ..Default::default()
            },
            channel_graph,
        );

        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        let actual = timeout(
            TINY_TIMEOUT * 20,
            stream
                .take(RANDOM_PEERS.len())
                .map(|routing| match routing {
                    ProbeRouting::Neighbor(DestinationRouting::Forward { destination, .. }) => {
                        if let NodeId::Offchain(peer_key) = destination.as_ref() {
                            *peer_key
                        } else {
                            panic!("expected offchain destination, got chain address");
                        }
                    }
                    _ => panic!("expected Neighbor Forward routing"),
                })
                .collect::<Vec<_>>(),
        )
        .await?;

        assert_eq!(actual.len(), RANDOM_PEERS.len());
        assert!(!actual.iter().zip(RANDOM_PEERS.iter()).all(|(a, b)| a == &b.id));

        Ok(())
    }

    #[tokio::test]
    async fn peers_should_be_generated_in_multiple_rounds_as_long_as_they_are_available() -> anyhow::Result<()> {
        let cfg = ProberConfig {
            interval: std::time::Duration::from_millis(1),
            ..Default::default()
        };

        let me = OffchainKeypair::random().public().clone();
        let channel_graph = Arc::new(ChannelGraph::new(me));
        channel_graph.record_node(RANDOM_PEERS.iter().next().unwrap().clone());

        let prober = FullNetworkDiscovery::new(me, cfg, channel_graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());

        Ok(())
    }

    #[cfg(not(feature = "noise"))]
    #[tokio::test]
    async fn cover_traffic_should_produce_empty_stream() -> anyhow::Result<()> {
        let cfg = ProberConfig {
            interval: std::time::Duration::from_millis(1),
            ..Default::default()
        };

        let me = OffchainKeypair::random().public().clone();
        let channel_graph = Arc::new(ChannelGraph::new(me));

        let prober = FullNetworkDiscovery::new(me, cfg, channel_graph);
        let stream = CoverTrafficGeneration::build(&prober);
        pin_mut!(stream);

        // The cover traffic stream should be empty (no items) and terminate immediately
        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn only_neighbor_probes_emitted_when_no_looping_paths_exist() -> anyhow::Result<()> {
        let cfg = ProberConfig {
            interval: std::time::Duration::from_millis(1),
            ..Default::default()
        };
        let me = OffchainKeypair::random().public().clone();
        let channel_graph = Arc::new(ChannelGraph::new(me));

        // Add peers with edges but no loopback paths discoverable by simple_paths
        let a = OffchainKeypair::random().public().clone();
        let b = OffchainKeypair::random().public().clone();
        channel_graph.record_node(a);
        channel_graph.record_node(b);
        channel_graph.add_edge(&me, &a)?;
        channel_graph.add_edge(&a, &b)?;

        let prober = FullNetworkDiscovery::new(me, cfg, channel_graph);

        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        let items: Vec<ProbeRouting> = timeout(TINY_TIMEOUT * 50, stream.take(10).collect::<Vec<_>>()).await?;

        assert!(!items.is_empty(), "should produce neighbor probes");
        assert!(
            items.iter().all(|r| matches!(r, ProbeRouting::Neighbor(_))),
            "all probes should be Neighbor when no looping paths exist"
        );

        Ok(())
    }

    #[tokio::test]
    async fn neighbor_probes_should_cover_all_known_nodes_across_rounds() -> anyhow::Result<()> {
        let cfg = ProberConfig {
            interval: std::time::Duration::from_millis(1),
            ..Default::default()
        };
        let me = OffchainKeypair::random().public().clone();
        let channel_graph = Arc::new(ChannelGraph::new(me));

        let peer_a = OffchainKeypair::random().public().clone();
        let peer_b = OffchainKeypair::random().public().clone();
        channel_graph.record_node(peer_a);
        channel_graph.record_node(peer_b);

        let prober = FullNetworkDiscovery::new(me, cfg, channel_graph);

        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        // graph.nodes() returns me + peer_a + peer_b, but the prober skips `me`,
        // so each round probes only peer_a and peer_b (2 nodes per round).
        // Collect enough items for at least 2 full rounds.
        let destinations: Vec<NodeId> = timeout(
            TINY_TIMEOUT * 50,
            stream
                .filter_map(|r| {
                    futures::future::ready(match r {
                        ProbeRouting::Neighbor(DestinationRouting::Forward { destination, .. }) => Some(*destination),
                        _ => None,
                    })
                })
                .take(4)
                .collect::<Vec<_>>(),
        )
        .await?;

        let unique: HashSet<NodeId> = destinations.iter().cloned().collect();
        let expected_nodes: HashSet<NodeId> = [peer_a, peer_b].into_iter().map(NodeId::from).collect();

        assert_eq!(
            unique, expected_nodes,
            "probes should cover all known graph peers (excluding me)"
        );
        // With 2 unique peers and 4 items, each peer appeared in multiple rounds
        assert_eq!(
            destinations.len(),
            4,
            "should have collected probes across multiple rounds"
        );

        Ok(())
    }

    #[test]
    fn weighted_shuffle_favors_higher_weight_items() -> anyhow::Result<()> {
        let items = vec![("low", 0.1), ("high", 10.0)];
        let mut high_first_count = 0;
        let trials = 1000;

        for _ in 0..trials {
            let shuffled = weighted_shuffle(items.clone());
            if shuffled[0] == "high" {
                high_first_count += 1;
            }
        }

        // With weights 0.1 vs 10.0, "high" should appear first the vast majority of the time
        assert!(
            high_first_count > trials * 8 / 10,
            "high-weight item should appear first in >{} of {} trials, but appeared first {} times",
            trials * 8 / 10,
            trials,
            high_first_count
        );

        Ok(())
    }

    #[test]
    fn weighted_shuffle_preserves_all_items() -> anyhow::Result<()> {
        let items: Vec<(u32, f64)> = (0..10).map(|i| (i, (i as f64 + 1.0) * 0.1)).collect();
        let shuffled = weighted_shuffle(items);
        assert_eq!(shuffled.len(), 10);

        let mut sorted = shuffled.clone();
        sorted.sort();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());

        Ok(())
    }

    #[test]
    fn immediate_probe_priority_is_maximal_for_unobserved_peers() {
        let cfg = ProberConfig::default();
        let now = std::time::Duration::from_secs(1000);

        // Unobserved peer: score=0, last_update=0
        let priority = immediate_probe_priority(0.0, std::time::Duration::ZERO, now, &cfg);

        // Should be close to the theoretical maximum: staleness_weight + quality_weight + base
        let max_priority = cfg.staleness_weight + cfg.quality_weight + cfg.base_priority;
        assert!(
            (priority - max_priority).abs() < 1e-9,
            "unobserved peer priority {priority} should equal max {max_priority}"
        );
    }

    #[test]
    fn immediate_probe_priority_increases_with_staleness() {
        let cfg = ProberConfig::default();
        let now = std::time::Duration::from_secs(10000);
        let score = 0.5;

        let recent = immediate_probe_priority(score, now - std::time::Duration::from_secs(10), now, &cfg);
        let stale = immediate_probe_priority(score, now - std::time::Duration::from_secs(3000), now, &cfg);

        assert!(
            stale > recent,
            "staler peer ({stale}) should have higher priority than recently probed ({recent})"
        );
    }

    #[test]
    fn immediate_probe_priority_increases_with_lower_score() {
        let cfg = ProberConfig::default();
        let now = std::time::Duration::from_secs(1000);
        let last_update = now - std::time::Duration::from_secs(100);

        let good = immediate_probe_priority(0.9, last_update, now, &cfg);
        let bad = immediate_probe_priority(0.1, last_update, now, &cfg);

        assert!(
            bad > good,
            "low-score peer ({bad}) should have higher priority than high-score ({good})"
        );
    }
}
