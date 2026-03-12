use std::collections::VecDeque;

use futures::{StreamExt, stream::BoxStream};
use futures_concurrency::stream::StreamExt as _;
use hopr_api::{
    ct::{CoverTrafficGeneration, ProbeRouting, ProbingTrafficGeneration},
    graph::{NetworkGraphTraverse, NetworkGraphView, costs::EdgeCostFn, traits::EdgeObservableRead},
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

/// Default penalty factor applied to edge cost functions.
const DEFAULT_EDGE_PENALTY: f64 = 0.5;

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
fn loopback_path_stream<U>(
    me: OffchainPublicKey,
    cfg: ProberConfig,
    graph: U,
) -> impl futures::Stream<Item = (Vec<OffchainPublicKey>, PathId)>
where
    U: NetworkGraphTraverse<NodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
{
    // 2, 3, 4 edges → 1-, 2-, 3-hops in the HOPR protocol
    futures::stream::repeat(futures::stream::iter([2usize, 3, 4]))
        .flatten()
        .filter_map(move |edge_count| async move {
            hopr_async_runtime::prelude::sleep(cfg.interval).await;
            std::num::NonZeroUsize::new(edge_count)
        })
        .flat_map(move |edge_count| {
            let paths = graph.simple_paths(
                &me,
                &me,
                edge_count.get(),
                Some(100),
                EdgeCostFn::forward(edge_count, DEFAULT_EDGE_PENALTY),
            );

            let weighted: Vec<_> = paths
                .into_iter()
                .map(|(path, path_id, cost)| {
                    let priority = (1.0 - cost).max(0.0) + cfg.base_priority;
                    ((path, path_id), priority)
                })
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

                loopback_path_stream(me, self.cfg, self.graph.clone())
                    .filter_map(move |(path, _)| futures::future::ready(loopback_routing(me_node, path)))
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
        let intermediates = loopback_path_stream(me, cfg, self.graph.clone()).filter_map(move |(path, path_id)| {
            let routing = loopback_routing(me_node, path).map(|r| ProbeRouting::Looping((r, path_id)));
            futures::future::ready(routing)
        });

        immediates.merge(intermediates).boxed()
    }
}

/// Owned state for the immediate-probe stream (no lock needed — consumed sequentially).
struct ImmediateProbeCache {
    items: VecDeque<ProbeRouting>,
    created_at: std::time::Instant,
}

/// Stream of neighbor probes with a TTL-guarded weighted shuffle cache.
///
/// The current batch is always fully drained before a new shuffle is computed,
/// ensuring no queued probes are discarded when the TTL expires.
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
    let initial = ImmediateProbeCache {
        items: VecDeque::new(),
        created_at: std::time::Instant::now(),
    };

    futures::stream::unfold(initial, move |mut state| {
        let graph = graph.clone();

        async move {
            hopr_async_runtime::prelude::sleep(cfg.interval).await;

            // Drain the current batch before rebuilding — never discard queued probes.
            if let Some(item) = state.items.pop_front() {
                return Some((item, state));
            }

            // Batch exhausted — re-traverse the graph and compute a new weighted shuffle.
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();

            let nodes: Vec<OffchainPublicKey> = graph
                .nodes()
                .filter(|peer| futures::future::ready(peer != &me))
                .collect()
                .await;

            let weighted: Vec<_> = nodes
                .into_iter()
                .map(|peer| {
                    let priority = match graph.edge(&me, &peer) {
                        Some(obs) => immediate_probe_priority(obs.score(), obs.last_update(), now, &cfg),
                        None => immediate_probe_priority(0.0, std::time::Duration::ZERO, now, &cfg),
                    };
                    (peer, priority)
                })
                .collect();

            let zero_hop = RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"));
            state.items = WeightedCollection::new(weighted)
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
            state.created_at = std::time::Instant::now();

            state.items.pop_front().map(|item| (item, state))
        }
    })
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
        let expected: HashSet<NodeId> = [peer_a, peer_b].into_iter().map(NodeId::from).collect();

        assert_eq!(unique, expected, "probes should cover all known graph peers");
        assert_eq!(destinations.len(), 4, "should have probes across multiple rounds");
        Ok(())
    }
}
