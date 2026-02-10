use futures::StreamExt;
use hopr_api::ct::{DestinationRouting, NetworkGraphView, TrafficGeneration};
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::protocol::HoprPseudonym;
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

    /// The time threshold after which it is reasonable to recheck the nearest neighbor
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_recheck_threshold", with = "humantime_serde")
    )]
    #[default(default_recheck_threshold())]
    pub recheck_threshold: std::time::Duration,
}

/// Delay before repeating probing rounds, must include enough time to traverse NATs
const DEFAULT_REPEATED_PROBING_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

/// Time after which the availability of a node gets rechecked
const DEFAULT_PROBE_RECHECK_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(60);

#[inline]
const fn default_probing_interval() -> std::time::Duration {
    DEFAULT_REPEATED_PROBING_DELAY
}

#[inline]
const fn default_recheck_threshold() -> std::time::Duration {
    DEFAULT_PROBE_RECHECK_THRESHOLD
}

pub struct ImmediateNeighborProber {
    cfg: ProberConfig,
}

impl ImmediateNeighborProber {
    pub fn new(cfg: ProberConfig) -> Self {
        Self { cfg }
    }
}

impl TrafficGeneration for ImmediateNeighborProber {
    type NodeId = OffchainPublicKey;

    fn build<U>(self, network_graph: U) -> impl futures::Stream<Item = DestinationRouting> + Send
    where
        U: NetworkGraphView<NodeId = OffchainPublicKey> + Send + Sync + 'static,
    {
        // For each probe target a cached version of transport routing is stored
        let cache_peer_routing: moka::future::Cache<OffchainPublicKey, DestinationRouting> =
            moka::future::Cache::builder()
                .time_to_live(std::time::Duration::from_secs(600))
                .max_capacity(100_000)
                .build();

        let cfg = self.cfg;

        futures::stream::repeat(())
            .filter_map(move |_| {
                let nodes = network_graph.nodes();

                async move {
                    hopr_async_runtime::prelude::sleep(cfg.interval).await;
                    Some(nodes)
                }
            })
            .flatten()
            .filter_map(move |peer| {
                let cache_peer_routing = cache_peer_routing.clone();

                async move {
                    cache_peer_routing
                        .try_get_with(peer, async move {
                            Ok::<DestinationRouting, anyhow::Error>(DestinationRouting::Forward {
                                destination: Box::new(peer.into()),
                                pseudonym: Some(HoprPseudonym::random()),
                                forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                                return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                            })
                        })
                        .await
                        .ok()
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use futures::{StreamExt, pin_mut};
    use hopr_api::{
        OffchainKeypair,
        graph::{MeasurableNode, NetworkGraphUpdate},
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
        pub is_connected: bool,
    }

    impl MeasurableNode for Node {
        fn id(&self) -> &OffchainPublicKey {
            &self.id
        }

        fn is_connected(&self) -> bool {
            self.is_connected
        }
    }

    lazy_static::lazy_static! {
        static ref RANDOM_PEERS: HashSet<Node> = (1..10).map(|_| {
            Node {
                id: OffchainPublicKey::from_privkey(&hopr_crypto_random::random_bytes::<32>()).unwrap(),
                is_connected: false
            }
        }).collect::<HashSet<_>>();
    }

    #[tokio::test]
    async fn peers_should_not_be_passed_if_none_are_present() -> anyhow::Result<()> {
        let channel_graph = ChannelGraph::new(OffchainKeypair::random().public().clone());

        let prober = ImmediateNeighborProber::new(Default::default());
        let stream = prober.build(channel_graph);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn peers_should_have_randomized_order() -> anyhow::Result<()> {
        let channel_graph = ChannelGraph::new(OffchainKeypair::random().public().clone());

        for node in RANDOM_PEERS.iter() {
            channel_graph.record_node(node.clone()).await;
        }

        let prober = ImmediateNeighborProber::new(ProberConfig {
            interval: std::time::Duration::from_millis(1),
            ..Default::default()
        });

        let stream = prober.build(channel_graph);
        pin_mut!(stream);

        let actual = timeout(
            TINY_TIMEOUT * 20,
            stream
                .take(RANDOM_PEERS.len())
                .map(|routing| match routing {
                    DestinationRouting::Forward { destination, .. } => {
                        if let NodeId::Offchain(peer_key) = destination.as_ref() {
                            *peer_key
                        } else {
                            panic!("expected offchain destination, got chain address");
                        }
                    }
                    _ => panic!("expected Forward routing"),
                })
                .collect::<Vec<_>>(),
        )
        .await?;

        assert_eq!(actual.len(), RANDOM_PEERS.len());
        assert!(!actual.iter().zip(RANDOM_PEERS.iter()).all(|(a, b)| a == b.id()));

        Ok(())
    }

    #[tokio::test]
    async fn peers_should_be_generated_in_multiple_rounds_as_long_as_they_are_available() -> anyhow::Result<()> {
        let cfg = ProberConfig {
            interval: std::time::Duration::from_millis(1),
            recheck_threshold: std::time::Duration::from_millis(1000),
            ..Default::default()
        };

        let channel_graph = ChannelGraph::new(OffchainKeypair::random().public().clone());
        channel_graph
            .record_node(RANDOM_PEERS.iter().next().unwrap().clone())
            .await;

        let prober = ImmediateNeighborProber::new(cfg);
        let stream = prober.build(channel_graph);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());

        Ok(())
    }
}
