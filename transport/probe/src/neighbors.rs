use std::sync::Arc;

use async_stream::stream;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::protocol::HoprPseudonym;
use hopr_network_types::types::{DestinationRouting, RoutingOptions};
use libp2p_identity::PeerId;
use rand::seq::SliceRandom;

use crate::{
    config::ProbeConfig,
    traits::{PeerDiscoveryFetch, ProbeStatusUpdate, TrafficGeneration},
};

struct TelemetrySink<T> {
    prober: Arc<T>,
}

impl<T> Clone for TelemetrySink<T> {
    fn clone(&self) -> Self {
        Self {
            prober: self.prober.clone(),
        }
    }
}

impl<T> futures::Sink<crate::errors::Result<crate::types::Telemetry>> for TelemetrySink<T>
where
    T: ProbeStatusUpdate + Send + Sync + 'static,
{
    type Error = std::convert::Infallible;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(
        self: std::pin::Pin<&mut Self>,
        item: crate::errors::Result<crate::types::Telemetry>,
    ) -> Result<(), Self::Error> {
        let prober = self.prober.clone();
        hopr_async_runtime::prelude::spawn(async move {
            match item {
                Ok(telemetry) => match telemetry {
                    crate::types::Telemetry::Loopback(_path_telemetry) => {
                        tracing::warn!(
                            reason = "feature not implemented",
                            "loopback path telemetry not supported yet"
                        );
                    }
                    crate::types::Telemetry::Neighbor(neighbor_telemetry) => {
                        tracing::trace!(
                            peer = %neighbor_telemetry.peer,
                            latency_ms = neighbor_telemetry.rtt.as_millis(),
                            "neighbor probe successful"
                        );
                        prober
                            .on_finished(&neighbor_telemetry.peer, &Ok(neighbor_telemetry.rtt))
                            .await;
                    }
                },
                Err(error) => match error {
                    crate::errors::ProbeError::ProbeNeighborTimeout(peer) => {
                        tracing::trace!(
                            %peer,
                            "neighbor probe timed out"
                        );
                        prober
                            .on_finished(&peer, &Err(crate::errors::ProbeError::ProbeNeighborTimeout(peer)))
                            .await;
                    }
                    crate::errors::ProbeError::ProbeLoopbackTimeout(_telemetry) => {
                        tracing::warn!(
                            reason = "feature not implemented",
                            "loopback path telemetry not supported yet"
                        );
                    }
                    _ => tracing::error!(%error, "unknown error on probe telemetry result evaluation"),
                },
            }
        });
        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

pub struct ImmediateNeighborProber<T> {
    cfg: ProbeConfig,
    prober: Arc<T>,
}

impl<T> ImmediateNeighborProber<T> {
    pub fn new(cfg: ProbeConfig, prober: T) -> Self {
        Self {
            cfg,
            prober: Arc::new(prober),
        }
    }
}

impl<T> TrafficGeneration for ImmediateNeighborProber<T>
where
    T: PeerDiscoveryFetch + ProbeStatusUpdate + Send + Sync + 'static,
{
    fn build(
        self,
    ) -> (
        impl futures::Stream<Item = DestinationRouting> + Send,
        impl futures::Sink<crate::errors::Result<crate::types::Telemetry>, Error = impl std::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ) {
        // For each probe target a cached version of transport routing is stored
        let cache_peer_routing: moka::future::Cache<PeerId, DestinationRouting> = moka::future::Cache::builder()
            .time_to_live(std::time::Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

        let prober = self.prober.clone();

        let route_stream = stream! {
            let mut rng = hopr_crypto_random::rng();
            loop {
                let now = std::time::SystemTime::now();

                let mut peers = prober.get_peers(now.checked_sub(self.cfg.recheck_threshold).unwrap_or(now)).await;
                peers.shuffle(&mut rng);    // shuffle peers to randomize order between rounds

                for peer in peers {
                    if let Ok(routing) = cache_peer_routing
                        .try_get_with(peer, async move {
                            Ok::<DestinationRouting, anyhow::Error>(DestinationRouting::Forward {
                                destination: Box::new(OffchainPublicKey::from_peerid(&peer)?.into()),
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

        let result_sink = TelemetrySink { prober: self.prober };

        (route_stream, result_sink)
    }
}

#[cfg(test)]
mod tests {
    use futures::{StreamExt, pin_mut};
    use hopr_internal_types::NodeId;
    use tokio::time::timeout;

    use super::*;

    const TINY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

    mockall::mock! {
        ScanInteraction {}

        #[async_trait::async_trait]
        impl ProbeStatusUpdate for ScanInteraction {
            async fn on_finished(&self, peer: &PeerId, result: &crate::errors::Result<std::time::Duration>);
        }

        #[async_trait::async_trait]
        impl PeerDiscoveryFetch for ScanInteraction {
            async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<PeerId>;
        }
    }

    #[tokio::test]
    async fn peers_should_not_be_passed_if_none_are_present() -> anyhow::Result<()> {
        let mut fetcher = MockScanInteraction::new();
        fetcher.expect_get_peers().returning(|_| vec![]);

        let prober = ImmediateNeighborProber::new(Default::default(), fetcher);
        let (stream, _sink) = prober.build();
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await.is_err());

        Ok(())
    }

    lazy_static::lazy_static! {
        static ref RANDOM_PEERS: Vec<PeerId> = (1..10).map(|_| {
            let peer: PeerId = OffchainPublicKey::from_privkey(&hopr_crypto_random::random_bytes::<32>()).unwrap().into();
            peer
        }).collect::<Vec<_>>();
    }

    #[tokio::test]
    async fn peers_should_have_randomized_order() -> anyhow::Result<()> {
        let mut fetcher = MockScanInteraction::new();
        fetcher.expect_get_peers().returning(|_| RANDOM_PEERS.clone());

        let prober = ImmediateNeighborProber::new(Default::default(), fetcher);
        let (stream, _sink) = prober.build();
        pin_mut!(stream);

        let actual = timeout(
            TINY_TIMEOUT * 20,
            stream
                .take(RANDOM_PEERS.len())
                .map(|routing| match routing {
                    DestinationRouting::Forward { destination, .. } => {
                        if let NodeId::Offchain(peer_key) = destination.as_ref() {
                            PeerId::from(peer_key)
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

        let mut fetcher = MockScanInteraction::new();
        fetcher.expect_get_peers().times(2).returning(|_| {
            let peer: PeerId = OffchainPublicKey::from_privkey(&hopr_crypto_random::random_bytes::<32>())
                .unwrap()
                .into();
            vec![peer]
        });
        fetcher.expect_get_peers().returning(|_| vec![]);

        let prober = ImmediateNeighborProber::new(cfg, fetcher);
        let (stream, _sink) = prober.build();
        pin_mut!(stream);

        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        assert!(timeout(TINY_TIMEOUT, stream.next()).await?.is_some());
        assert!(timeout(TINY_TIMEOUT, stream.next()).await.is_err());

        Ok(())
    }
}
