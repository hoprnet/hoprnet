use async_stream::stream;
use futures::Stream;
use libp2p_identity::PeerId;
use rand::seq::SliceRandom;

use hopr_crypto_random::Randomizable;
use hopr_internal_types::protocol::HoprPseudonym;
use hopr_network_types::types::{DestinationRouting, RoutingOptions};
use hopr_primitive_types::prelude::Address;

use crate::{
    config::ProbeConfig,
    traits::{PeerDiscoveryFetch, TrafficGeneration},
};

pub struct ImmediateNeighborProber<T> {
    cfg: ProbeConfig,
    me: (PeerId, Address),
    fetcher: T,
}

impl<T> ImmediateNeighborProber<T> {
    pub fn new(cfg: ProbeConfig, me: (PeerId, Address), fetcher: T) -> Self {
        Self { cfg, me, fetcher }
    }
}

impl<T> TrafficGeneration for ImmediateNeighborProber<T>
where
    T: PeerDiscoveryFetch + Send + Sync + 'static,
{
    fn build(
        self,
    ) -> (
        impl futures::Stream<Item = DestinationRouting>,
        impl futures::Sink<crate::errors::Result<crate::TrafficReturnedObservation>, Error = impl std::error::Error>,
    ) {
        // For each probe target a cached version of transport routing is stored
        let cache_peer_routing: moka::future::Cache<PeerId, DestinationRouting> = moka::future::Cache::builder()
            .time_to_live(std::time::Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

        let s = stream! {
            let mut rng = hopr_crypto_random::rng();
            loop {
                let now = std::time::SystemTime::now();

                let mut peers = self.fetcher.get_peers(now.checked_sub(self.cfg.recheck_threshold).unwrap_or(now)).await;
                peers.shuffle(&mut rng);    // shuffle peers to randomize order between rounds

                for peer in peers {
                    if let Ok(routing) = cache_peer_routing
                        .try_get_with(peer, async move {
                            Ok::<DestinationRouting, anyhow::Error>(DestinationRouting::Forward {
                                destination: self.me.1,     // TODO: convert peer_id to address
                                pseudonym: Some(HoprPseudonym::random()),
                                forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                                return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                            })
                        })
                        .await {
                            yield routing;
                        }
                }

                hopr_async_runtime::prelude::sleep(self.cfg.interval).await;
            }
        };

        (s, futures::sink::drain())
    }
}

#[cfg(test)]
mod tests {
    use futures::{StreamExt, pin_mut};
    use tokio::time::timeout;

    use super::*;
    use crate::traits::MockPeerDiscoveryFetch;

    const TINY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

    #[tokio::test]
    async fn peers_should_not_be_passed_if_none_are_present() -> anyhow::Result<()> {
        let mut fetcher = MockPeerDiscoveryFetch::new();
        fetcher.expect_get_peers().returning(|_| vec![]);

        let stream = neighbors_to_probe(fetcher, Default::default());
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await.is_err());

        Ok(())
    }

    lazy_static::lazy_static! {
        static ref RANDOM_PEERS: Vec<PeerId> = (1..10).map(|_| PeerId::random()).collect::<Vec<_>>();
    }

    #[tokio::test]
    async fn peers_should_have_randomized_order() -> anyhow::Result<()> {
        let mut fetcher = MockPeerDiscoveryFetch::new();
        fetcher.expect_get_peers().returning(|_| RANDOM_PEERS.clone());

        let stream = neighbors_to_probe(fetcher, Default::default());
        pin_mut!(stream);

        let actual = timeout(TINY_TIMEOUT * 20, stream.take(RANDOM_PEERS.len()).collect::<Vec<_>>()).await?;

        assert_eq!(actual.len(), RANDOM_PEERS.len());
        assert!(!actual.iter().zip(RANDOM_PEERS.iter()).all(|(a, b)| a == b));

        Ok(())
    }

    #[tokio::test]
    async fn peers_should_be_generated_in_multiple_rounds_as_long_as_they_are_available() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            interval: std::time::Duration::from_millis(1),
            recheck_threshold: std::time::Duration::from_millis(1000),
            ..Default::default()
        };

        let mut fetcher = MockPeerDiscoveryFetch::new();
        fetcher
            .expect_get_peers()
            .times(2)
            .returning(|_| vec![PeerId::random()]);
        fetcher.expect_get_peers().returning(|_| vec![]);

        let stream = neighbors_to_probe(fetcher, cfg);
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        assert!(timeout(TINY_TIMEOUT, stream.next()).await.is_err());

        Ok(())
    }
}
