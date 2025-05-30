use async_stream::stream;
use futures::Stream;
use libp2p_identity::PeerId;
use rand::seq::SliceRandom;

use crate::{config::ProbeConfig, traits::PeerDiscoveryFetch};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_PROBE: hopr_metrics::metrics::SimpleHistogram =
        hopr_metrics::metrics::SimpleHistogram::new(
            "hopr_probe_round_time_sec",
            "Measures total time in seconds it takes to probe all nodes",
            vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0],
        ).unwrap();
}

pub fn neighbors_to_probe<T>(fetcher: T, cfg: ProbeConfig) -> impl Stream<Item = PeerId>
where
    T: PeerDiscoveryFetch + Send + Sync + 'static,
{
    stream! {
        let mut rng = hopr_crypto_random::rng();
        loop {
            let now = std::time::SystemTime::now();

            let mut peers = fetcher.get_peers(now.checked_sub(cfg.recheck_threshold).unwrap_or(now)).await;
            peers.shuffle(&mut rng);    // shuffle peers to randomize order between rounds

            #[cfg(all(feature = "prometheus", not(test)))]
            let probe_round_timer = if peers.is_empty() { None } else { Some(hopr_metrics::histogram_start_measure!(METRIC_TIME_TO_PROBE)) };

            for peer in peers {
                yield peer;
            }

            #[cfg(all(feature = "prometheus", not(test)))]
            if let Some(probe_round_timer) = probe_round_timer {
                METRIC_TIME_TO_PROBE.record_measure(probe_round_timer);
            }

            hopr_async_runtime::prelude::sleep(cfg.interval).await;
        }
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
