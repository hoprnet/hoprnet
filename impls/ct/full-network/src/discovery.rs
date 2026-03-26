use futures::{StreamExt, stream::BoxStream};
use hopr_api::{
    ct::{CoverTrafficGeneration, ProbeRouting, ProbingTrafficGeneration},
    graph::{
        NetworkGraphTraverse, NetworkGraphView,
        traits::{EdgeNetworkObservableRead, EdgeObservableRead},
    },
    types::{
        crypto::types::OffchainPublicKey,
        crypto_random::Randomizable,
        internal::{
            NodeId,
            protocol::HoprPseudonym,
            routing::{DestinationRouting, PathId, RoutingOptions},
        },
    },
};
use hopr_statistics::WeightedCollection;

use crate::{ProberConfig, priority::immediate_probe_priority};

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

/// Strip the leading source and trailing destination from a loopback path.
///
/// `simple_loopback_to_self` returns `[me, intermediates..., me]`.  The caller
/// adds `me` as the destination and the planner prepends `me` as source, so
/// only the interior intermediate nodes are needed.
fn strip_loopback_endpoints(mut path: Vec<OffchainPublicKey>, me: &OffchainPublicKey) -> Vec<OffchainPublicKey> {
    if path.last() == Some(me) {
        path.pop();
    }
    if path.first() == Some(me) {
        path.remove(0);
    }
    path
}

/// Build a `DestinationRouting::Forward` with a random pseudonym and an explicit intermediate path.
fn loopback_routing(me: NodeId, path: Vec<OffchainPublicKey>) -> Option<DestinationRouting> {
    let path: Vec<NodeId> = path.into_iter().map(NodeId::from).collect();
    hopr_api::network::BoundedVec::try_from(path)
        .ok()
        .map(|path| DestinationRouting::Forward {
            destination: Box::new(me),
            pseudonym: Some(HoprPseudonym::random()),
            forward_options: RoutingOptions::IntermediatePath(path),
            return_options: None,
        })
}

/// Stream that cycles through 1-, 2-, and 3-hop loopback paths with weighted shuffle.
///
/// Shared by both cover traffic and intermediate probing — the caller wraps each
/// emitted item into the appropriate outer type.
fn loopback_path_stream<U>(cfg: ProberConfig, graph: U) -> impl futures::Stream<Item = (Vec<OffchainPublicKey>, PathId)>
where
    U: NetworkGraphTraverse<NodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
{
    // 2, 3, 4 edges → 1-, 2-, 3-hops in the HOPR protocol
    futures_time::stream::interval(futures_time::time::Duration::from(cfg.interval))
        .flat_map(|_| futures::stream::iter([2usize, 3, 4]))
        .filter_map(move |edge_count| futures::future::ready(std::num::NonZeroUsize::new(edge_count)))
        .flat_map(move |edge_count| {
            let paths = graph.simple_loopback_to_self(edge_count.get(), Some(100));

            let count = paths.len();
            tracing::debug!(edge_count = edge_count.get(), count, "loopback path candidates");
            let weighted: Vec<_> = paths
                .into_iter()
                .map(|(path, path_id)| ((path, path_id), cfg.base_priority))
                .collect();

            futures::stream::iter(WeightedCollection::new(weighted).into_shuffled())
        })
}

impl<U> CoverTrafficGeneration for FullNetworkDiscovery<U>
where
    U: NetworkGraphTraverse<NodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
{
    fn build(&self) -> BoxStream<'static, DestinationRouting> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "noise")] {
                let me = self.me;
                let me_node: NodeId = me.into();

                loopback_path_stream(self.cfg, self.graph.clone())
                    .filter_map(move |(path, _)| {
                        let intermediates = strip_loopback_endpoints(path, &me);
                        futures::future::ready(loopback_routing(me_node, intermediates))
                    })
                    .boxed()
            } else {
                Box::pin(futures::stream::empty())
            }
        }
    }
}

impl<U> ProbingTrafficGeneration for FullNetworkDiscovery<U>
where
    U: NetworkGraphView<NodeId = OffchainPublicKey, Observed = hopr_network_graph::Observations>
        + NetworkGraphTraverse<NodeId = OffchainPublicKey>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn build(&self) -> BoxStream<'static, ProbeRouting> {
        let cfg = self.cfg;
        let me = self.me;

        let immediates = immediate_probe_stream(me, cfg, self.graph.clone());

        let me_node: NodeId = me.into();
        let intermediates = loopback_path_stream(cfg, self.graph.clone()).filter_map(move |(path, path_id)| {
            let intermediates = strip_loopback_endpoints(path, &me);
            let routing = loopback_routing(me_node, intermediates).map(|r| ProbeRouting::Looping((r, path_id)));
            futures::future::ready(routing)
        });

        futures::stream::select(immediates, intermediates).boxed()
    }
}

/// Cached weighted-shuffle batch with a creation timestamp for TTL checks.
struct ShuffleCache {
    probes: Vec<ProbeRouting>,
    created_at: std::time::Instant,
}

/// Stream of neighbor probes emitted in bursts per tick.
///
/// Each tick emits the entire cached batch. The cache is recomputed when it is
/// empty (all probes drained) or when `cfg.shuffle_ttl` has expired, whichever
/// comes first. This avoids re-traversing the graph every tick while still
/// emitting all peers in each burst.
///
/// When `cfg.probe_connected_only` is `true`, only peers with a
/// `Connected(true)` edge from `me` are included. This assumes that a
/// background discovery mechanism (e.g. libp2p identify/kademlia) has
/// already established connections and recorded them in the graph.
fn immediate_probe_stream<U>(
    me: OffchainPublicKey,
    cfg: ProberConfig,
    graph: U,
) -> impl futures::Stream<Item = ProbeRouting>
where
    U: NetworkGraphView<NodeId = OffchainPublicKey, Observed = hopr_network_graph::Observations>
        + Clone
        + Send
        + Sync
        + 'static,
{
    let cache: Option<ShuffleCache> = None;

    futures::stream::unfold(
        (
            cache,
            futures_time::stream::interval(futures_time::time::Duration::from(cfg.interval)),
        ),
        move |(mut cache, mut ticker)| {
            let graph = graph.clone();

            async move {
                use futures::StreamExt as _;
                ticker.next().await?;

                // Reuse cached shuffle if still within TTL.
                let needs_refresh = cache
                    .as_ref()
                    .is_none_or(|c| c.probes.is_empty() || c.created_at.elapsed() >= cfg.shuffle_ttl);

                if needs_refresh {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();

                    let weighted: Vec<_> = graph
                        .nodes()
                        .filter(|peer| futures::future::ready(peer != &me))
                        .filter_map(|peer| {
                            let obs = graph.edge(&me, &peer);
                            if cfg.probe_connected_only {
                                let connected = obs
                                    .as_ref()
                                    .and_then(|o| o.immediate_qos())
                                    .map(|imm| imm.is_connected())
                                    .unwrap_or(false);
                                if !connected {
                                    return futures::future::ready(None);
                                }
                            }
                            let priority = match obs {
                                Some(obs) => immediate_probe_priority(obs.score(), obs.last_update(), now, &cfg),
                                None => immediate_probe_priority(0.0, std::time::Duration::ZERO, now, &cfg),
                            };
                            futures::future::ready(Some((peer, priority)))
                        })
                        .collect()
                        .await;

                    let peer_count = weighted.len();
                    let zero_hop = RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"));
                    let probes: Vec<_> = WeightedCollection::new(weighted)
                        .into_shuffled()
                        .into_iter()
                        .map(|peer| {
                            ProbeRouting::Neighbor(DestinationRouting::Forward {
                                destination: Box::new(peer.into()),
                                pseudonym: Some(HoprPseudonym::random()),
                                forward_options: zero_hop.clone(),
                                return_options: Some(zero_hop.clone()),
                            })
                        })
                        .collect();

                    tracing::debug!(peer_count, probes = probes.len(), "computed new neighbor probe shuffle");
                    cache = Some(ShuffleCache {
                        probes,
                        created_at: std::time::Instant::now(),
                    });
                }

                let batch = cache.as_ref().map(|c| c.probes.clone()).unwrap_or_default();
                tracing::debug!(probes = batch.len(), "emitting neighbor probe batch");

                Some((futures::stream::iter(batch), (cache, ticker)))
            }
        },
    )
    .flatten()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use futures::{StreamExt, pin_mut};
    use hopr_api::{
        OffchainKeypair,
        ct::{ProbeRouting, ProbingTrafficGeneration},
        graph::{NetworkGraphUpdate, NetworkGraphWrite},
        types::{crypto::keypairs::Keypair, internal::NodeId},
    };
    use hopr_network_graph::ChannelGraph;
    use tokio::time::timeout;

    use super::*;

    const TINY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

    fn fast_cfg() -> ProberConfig {
        ProberConfig {
            interval: std::time::Duration::from_millis(1),
            shuffle_ttl: std::time::Duration::ZERO,
            probe_connected_only: false,
            ..Default::default()
        }
    }

    fn random_key() -> OffchainPublicKey {
        *OffchainKeypair::random().public()
    }

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
        let me = random_key();
        let prober = FullNetworkDiscovery::new(me, Default::default(), Arc::new(ChannelGraph::new(me)));
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn peers_should_have_randomized_order() -> anyhow::Result<()> {
        let me = random_key();
        let graph = Arc::new(ChannelGraph::new(me));
        for node in RANDOM_PEERS.iter() {
            graph.record_node(node.clone());
        }

        let peer_count = RANDOM_PEERS.len();
        let prober = FullNetworkDiscovery::new(me, fast_cfg(), graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        let extract_peer = |routing: ProbeRouting| -> OffchainPublicKey {
            match routing {
                ProbeRouting::Neighbor(DestinationRouting::Forward { destination, .. }) => {
                    if let NodeId::Offchain(peer_key) = destination.as_ref() {
                        *peer_key
                    } else {
                        panic!("expected offchain destination");
                    }
                }
                _ => panic!("expected Neighbor Forward routing"),
            }
        };

        // Collect two full rounds from the stream.
        let both_rounds: Vec<OffchainPublicKey> = timeout(
            TINY_TIMEOUT * 40,
            stream.take(peer_count * 2).map(extract_peer).collect::<Vec<_>>(),
        )
        .await?;

        let round_1 = &both_rounds[..peer_count];
        let round_2 = &both_rounds[peer_count..];

        // Both rounds should cover the same set of peers.
        let set_1: HashSet<_> = round_1.iter().collect();
        let set_2: HashSet<_> = round_2.iter().collect();
        assert_eq!(set_1, set_2, "both rounds should cover the same peers");

        // The two rounds should (almost certainly) differ in order.
        assert_ne!(round_1, round_2, "two rounds should differ in order (probabilistic)");
        Ok(())
    }

    #[tokio::test]
    async fn peers_should_be_generated_in_multiple_rounds() -> anyhow::Result<()> {
        let me = random_key();
        let graph = Arc::new(ChannelGraph::new(me));
        graph.record_node(RANDOM_PEERS.iter().next().unwrap().clone());

        let prober = FullNetworkDiscovery::new(me, fast_cfg(), graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        Ok(())
    }

    #[cfg(not(feature = "noise"))]
    #[tokio::test]
    async fn cover_traffic_should_produce_empty_stream() -> anyhow::Result<()> {
        let me = random_key();
        let prober = FullNetworkDiscovery::new(me, fast_cfg(), Arc::new(ChannelGraph::new(me)));
        let stream = CoverTrafficGeneration::build(&prober);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn only_neighbor_probes_emitted_when_no_looping_paths_exist() -> anyhow::Result<()> {
        let me = random_key();
        let graph = Arc::new(ChannelGraph::new(me));

        let a = random_key();
        let b = random_key();
        graph.record_node(a);
        graph.record_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;

        let prober = FullNetworkDiscovery::new(me, fast_cfg(), graph);
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
        let me = random_key();
        let graph = Arc::new(ChannelGraph::new(me));

        let peer_a = random_key();
        let peer_b = random_key();
        graph.record_node(peer_a);
        graph.record_node(peer_b);

        let prober = FullNetworkDiscovery::new(me, fast_cfg(), graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        let destinations: Vec<NodeId> = timeout(
            TINY_TIMEOUT * 50,
            neighbor_destinations(stream).take(4).collect::<Vec<_>>(),
        )
        .await?;

        let unique: HashSet<NodeId> = destinations.iter().cloned().collect();
        let expected: HashSet<NodeId> = [peer_a, peer_b].into_iter().map(NodeId::from).collect();

        assert_eq!(unique, expected, "probes should cover all known graph peers");
        assert_eq!(destinations.len(), 4, "should have probes across multiple rounds");
        Ok(())
    }

    #[tokio::test]
    async fn single_tick_should_emit_all_peers_in_burst() -> anyhow::Result<()> {
        let me = random_key();
        let graph = Arc::new(ChannelGraph::new(me));

        let peer_count = RANDOM_PEERS.len();
        for node in RANDOM_PEERS.iter() {
            graph.record_node(node.clone());
        }

        let prober = FullNetworkDiscovery::new(me, fast_cfg(), graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        // Collect exactly peer_count items — should all arrive from a single tick burst.
        let burst: Vec<ProbeRouting> = timeout(TINY_TIMEOUT * 20, stream.take(peer_count).collect::<Vec<_>>()).await?;

        assert_eq!(
            burst.len(),
            peer_count,
            "a single tick should emit all {peer_count} peers"
        );
        assert!(
            burst.iter().all(|r| matches!(r, ProbeRouting::Neighbor(_))),
            "all burst items should be Neighbor probes"
        );

        Ok(())
    }

    /// Extract `NodeId` destinations from a `ProbeRouting::Neighbor` stream.
    fn neighbor_destinations(stream: impl futures::Stream<Item = ProbeRouting>) -> impl futures::Stream<Item = NodeId> {
        stream.filter_map(|r| {
            futures::future::ready(match r {
                ProbeRouting::Neighbor(DestinationRouting::Forward { destination, .. }) => Some(*destination),
                _ => None,
            })
        })
    }

    /// Helper: mark an edge as fully connected with capacity, so it passes all cost checks.
    fn mark_edge_ready(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
        use hopr_api::graph::traits::{EdgeObservableWrite, EdgeWeightType};
        graph.upsert_edge(src, dst, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });
    }

    #[tokio::test]
    async fn loopback_probes_should_be_emitted_for_two_edge_path() -> anyhow::Result<()> {
        // Topology: me → a → b, b → me (connected neighbor)
        // Loopback: me → a → b → me
        let me = random_key();
        let a = random_key();
        let b = random_key();
        let graph = Arc::new(ChannelGraph::new(me));
        graph.add_node(a);
        graph.add_node(b);

        // Forward edges
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        mark_edge_ready(&graph, &me, &a);
        mark_edge_ready(&graph, &a, &b);

        // Return edge: b is a connected neighbor of me
        graph.add_edge(&b, &me)?;
        mark_edge_ready(&graph, &b, &me);

        // Also need me → b edge so simple_loopback_to_self finds b as a connected neighbor
        graph.add_edge(&me, &b)?;
        mark_edge_ready(&graph, &me, &b);

        let prober = FullNetworkDiscovery::new(me, fast_cfg(), graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        // Collect enough items to get both neighbor and loopback probes
        let items: Vec<ProbeRouting> = timeout(TINY_TIMEOUT * 100, stream.take(20).collect::<Vec<_>>()).await?;

        let looping_count = items.iter().filter(|r| matches!(r, ProbeRouting::Looping(_))).count();
        let neighbor_count = items.iter().filter(|r| matches!(r, ProbeRouting::Neighbor(_))).count();

        assert!(neighbor_count > 0, "should have neighbor probes");
        assert!(
            looping_count > 0,
            "should have loopback probes (was {looping_count} out of {} total)",
            items.len()
        );

        // Verify loopback routing has IntermediatePath
        for item in &items {
            if let ProbeRouting::Looping((
                DestinationRouting::Forward {
                    destination,
                    forward_options,
                    ..
                },
                _,
            )) = item
            {
                assert_eq!(
                    destination.as_ref(),
                    &NodeId::Offchain(me),
                    "loopback destination should be me"
                );
                assert!(
                    matches!(forward_options, RoutingOptions::IntermediatePath(_)),
                    "loopback should use IntermediatePath routing"
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn probe_connected_only_should_skip_unconnected_peers() -> anyhow::Result<()> {
        let me = random_key();
        let graph = Arc::new(ChannelGraph::new(me));

        let connected_peer = random_key();
        let unconnected_peer = random_key();
        graph.record_node(connected_peer);
        graph.record_node(unconnected_peer);

        // Only mark one peer as connected.
        mark_edge_ready(&graph, &me, &connected_peer);

        let cfg = ProberConfig {
            probe_connected_only: true,
            ..fast_cfg()
        };
        let prober = FullNetworkDiscovery::new(me, cfg, graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        let destinations: Vec<NodeId> = timeout(
            TINY_TIMEOUT * 50,
            neighbor_destinations(stream).take(3).collect::<Vec<_>>(),
        )
        .await?;

        let unique: HashSet<NodeId> = destinations.iter().cloned().collect();
        assert_eq!(unique.len(), 1, "only one peer should be probed");
        assert!(
            unique.contains(&NodeId::from(connected_peer)),
            "only the connected peer should be probed"
        );
        Ok(())
    }

    #[tokio::test]
    async fn probe_connected_only_disabled_should_probe_all_peers() -> anyhow::Result<()> {
        let me = random_key();
        let graph = Arc::new(ChannelGraph::new(me));

        let connected_peer = random_key();
        let unconnected_peer = random_key();
        graph.record_node(connected_peer);
        graph.record_node(unconnected_peer);

        mark_edge_ready(&graph, &me, &connected_peer);

        // Default: probe_connected_only = false
        let prober = FullNetworkDiscovery::new(me, fast_cfg(), graph);
        let stream = ProbingTrafficGeneration::build(&prober);
        pin_mut!(stream);

        let destinations: Vec<NodeId> = timeout(
            TINY_TIMEOUT * 50,
            neighbor_destinations(stream).take(4).collect::<Vec<_>>(),
        )
        .await?;

        let unique: HashSet<NodeId> = destinations.iter().cloned().collect();
        let expected: HashSet<NodeId> = [connected_peer, unconnected_peer]
            .into_iter()
            .map(NodeId::from)
            .collect();
        assert_eq!(
            unique, expected,
            "both peers should be probed when probe_connected_only is false"
        );
        Ok(())
    }

    #[tokio::test]
    async fn loopback_routing_should_reject_full_path_with_me() -> anyhow::Result<()> {
        // Verify that passing the full path [me, a, b, me] to loopback_routing
        // exceeds BoundedVec capacity and returns None.
        let me = random_key();
        let a = random_key();
        let b = random_key();
        let me_node = NodeId::Offchain(me);

        // Full path as returned by simple_loopback_to_self: [me, a, b, me]
        let full_path = vec![me, a, b, me];
        assert!(
            loopback_routing(me_node, full_path).is_none(),
            "full path [me, a, b, me] should exceed BoundedVec<3> and return None"
        );

        // Stripped path (intermediates only): [a, b]
        let stripped_path = vec![a, b];
        assert!(
            loopback_routing(me_node, stripped_path).is_some(),
            "stripped path [a, b] should fit BoundedVec<3> and return Some"
        );

        Ok(())
    }
}
