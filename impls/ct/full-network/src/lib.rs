use futures::{StreamExt, stream::BoxStream};
use futures_concurrency::stream::StreamExt as _;
use hopr_api::{
    ct::{CoverTrafficGeneration, DestinationRouting, ProbeRouting, ProbingTrafficGeneration},
    graph::{NetworkGraphTraverse, NetworkGraphView, costs::HoprCostFn},
};
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::{NodeId, protocol::HoprPseudonym};
use hopr_network_types::types::RoutingOptions;
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
}

/// Delay before repeating immediate probing rounds, should include enough time to traverse NATs
const DEFAULT_REPEATED_PROBING_DELAY: std::time::Duration = std::time::Duration::from_secs(30);

#[inline]
const fn default_probing_interval() -> std::time::Duration {
    DEFAULT_REPEATED_PROBING_DELAY
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
                let junk = futures::stream::repeat(futures::stream::iter([2, 3, 4]))
                    .flatten()
                    .filter_map(move |edge_count| async move {
                        hopr_async_runtime::prelude::sleep(cfg.interval).await;
                        Some(edge_count)
                    })
                    .flat_map(move |edge_count| {
                        let simple_paths = graph_intermediates.simple_paths(
                            &me,
                            &me,
                            edge_count,
                            Some(100),
                            HoprCostFn::new(edge_count),
                        );
                        futures::stream::iter(simple_paths)
                    })
                    .filter_map(move |(path, path_id, _cost)| {
                        let me_node: NodeId = me.into();
                        let path: Vec<NodeId> = path.into_iter().map(NodeId::from).collect();

                        let routing = hopr_api::network::BoundedVec::try_from(path).ok().map(|path| {
                            DestinationRouting::Forward {
                                destination: Box::new(me_node),
                                pseudonym: Some(HoprPseudonym::random()),
                                forward_options: RoutingOptions::IntermediatePath(path),
                                return_options: None,
                            }
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
        // For each probe target a cached version of transport routing is stored
        let cache_immediate_neigbor_routing: moka::future::Cache<OffchainPublicKey, ProbeRouting> =
            moka::future::Cache::builder()
                .time_to_live(std::time::Duration::from_secs(600))
                .max_capacity(100_000)
                .build();

        let cfg = self.cfg;
        let graph = self.graph.clone();
        let graph_intermediates = self.graph.clone();

        let immediates = futures::stream::repeat(())
            .filter_map(move |_| {
                let nodes = graph.nodes();

                async move {
                    hopr_async_runtime::prelude::sleep(cfg.interval).await;
                    Some(nodes)
                }
            })
            .flatten()
            .filter_map(move |peer| {
                let cache_immediate_neigbor_routing = cache_immediate_neigbor_routing.clone();

                async move {
                    cache_immediate_neigbor_routing
                        .try_get_with(peer, async move {
                            Ok::<ProbeRouting, anyhow::Error>(ProbeRouting::Neighbor(DestinationRouting::Forward {
                                destination: Box::new(peer.into()),
                                pseudonym: Some(HoprPseudonym::random()),
                                forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                                return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                            }))
                        })
                        .await
                        .ok()
                }
            });

        let me = self.me;

        // 2, 3 and 4 edges => 1-, 2- and 3-hops in the HOPR protocol
        let intermediates = futures::stream::repeat(futures::stream::iter([2, 3, 4]))
            .flatten()
            .filter_map(move |edge_count| async move {
                hopr_async_runtime::prelude::sleep(cfg.interval).await;
                Some(edge_count)
            })
            .flat_map(move |edge_count| {
                let simple_paths =
                    graph_intermediates.simple_paths(&me, &me, edge_count, Some(100), HoprCostFn::new(edge_count));
                futures::stream::iter(simple_paths)
            })
            .filter_map(move |(path, path_id, _cost)| {
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
    };
    use hopr_crypto_types::keypairs::Keypair;
    use hopr_internal_types::NodeId;
    use hopr_network_graph::ChannelGraph;
    use tokio::time::timeout;

    use super::*;

    const TINY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Node {
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
                id: OffchainPublicKey::from_privkey(&hopr_crypto_random::random_bytes::<32>()).unwrap(),
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

    #[tokio::test]
    async fn cover_traffic_should_produce_empty_stream() -> anyhow::Result<()> {
        let cfg = ProberConfig {
            interval: std::time::Duration::from_millis(1),
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

        // graph.nodes() returns me + peer_a + peer_b = 3 nodes per round.
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
                .take(6)
                .collect::<Vec<_>>(),
        )
        .await?;

        let unique: HashSet<NodeId> = destinations.iter().cloned().collect();
        let expected_nodes: HashSet<NodeId> = [me, peer_a, peer_b].into_iter().map(NodeId::from).collect();

        assert_eq!(
            unique, expected_nodes,
            "probes should cover all known graph nodes (me + peers)"
        );
        // With 3 unique nodes and 6 items, each node appeared in multiple rounds
        assert_eq!(
            destinations.len(),
            6,
            "should have collected probes across multiple rounds"
        );

        Ok(())
    }
}
