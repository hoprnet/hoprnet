use hopr_api::graph::{MeasurableEdge, MeasurableNode, NetworkGraphWrite, traits::EdgeObservableWrite};

use crate::{ChannelGraph, Observations};

#[async_trait::async_trait]
impl hopr_api::graph::NetworkGraphUpdate for ChannelGraph {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: hopr_api::graph::MeasurablePeer + std::fmt::Debug + Send + Clone,
        P: hopr_api::graph::MeasurablePath + std::fmt::Debug + Send + Clone,
    {
        use hopr_api::graph::traits::EdgeWeightType;

        match update {
            MeasurableEdge::Probe(Ok(hopr_api::graph::EdgeTransportTelemetry::Neighbor(ref telemetry))) => {
                tracing::trace!(
                    peer = %telemetry.peer(),
                    latency_ms = telemetry.rtt().as_millis(),
                    "neighbor probe successful"
                );

                self.upsert_edge(&self.me, telemetry.peer(), |obs| {
                    obs.record(EdgeWeightType::Immediate(Ok(telemetry.rtt() / 2)));
                });
            }
            MeasurableEdge::Probe(Ok(hopr_api::graph::EdgeTransportTelemetry::Loopback(_))) => {
                tracing::warn!(
                    reason = "feature not implemented",
                    "loopback path telemetry not supported"
                );
            }
            MeasurableEdge::Probe(Err(hopr_api::graph::NetworkGraphError::ProbeNeighborTimeout(ref peer))) => {
                tracing::trace!(
                    peer = %peer,
                    reason = "probe timeout",
                    "neighbor probe failed"
                );

                self.upsert_edge(&self.me, peer, |obs| {
                    obs.record(EdgeWeightType::Immediate(Err(())));
                });
            }
            MeasurableEdge::Probe(Err(hopr_api::graph::NetworkGraphError::ProbeLoopbackTimeout(_))) => {
                tracing::warn!(
                    reason = "feature not implemented",
                    "loopback path telemetry not supported"
                );
            }
            MeasurableEdge::Capacity(update) => {
                self.upsert_edge(&update.src, &update.dest, |obs: &mut Observations| {
                    obs.record(EdgeWeightType::Capacity(update.capacity));
                });
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        tracing::trace!(?update, "recording node update");
        hopr_api::graph::NetworkGraphWrite::add_node(self, update.into());
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hex_literal::hex;
    use hopr_api::{
        OffchainPublicKey,
        graph::{
            EdgeLinkObservable, EdgeTransportTelemetry, MeasurablePath, MeasurablePeer, NetworkGraphError,
            NetworkGraphUpdate, NetworkGraphView, NetworkGraphWrite,
            traits::{EdgeObservableRead, EdgeProtocolObservable},
        },
    };
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};

    use super::*;

    /// Fixed test secret keys (reused from the broader codebase).
    const SECRET_0: [u8; 32] = hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");
    const SECRET_1: [u8; 32] = hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a");

    /// Creates an OffchainPublicKey from a fixed secret.
    fn pubkey_from(secret: &[u8; 32]) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(secret).expect("valid secret key").public()
    }

    #[derive(Debug, Clone)]
    struct TestNeighbor {
        peer: OffchainPublicKey,
        rtt: std::time::Duration,
    }

    impl MeasurablePeer for TestNeighbor {
        fn peer(&self) -> &OffchainPublicKey {
            &self.peer
        }

        fn rtt(&self) -> std::time::Duration {
            self.rtt
        }
    }

    #[derive(Debug, Clone)]
    struct TestPath;

    impl MeasurablePath for TestPath {
        fn id(&self) -> &[u8] {
            &[]
        }

        fn path(&self) -> &[u8] {
            &[]
        }

        fn timestamp(&self) -> u128 {
            0
        }
    }

    #[tokio::test]
    async fn neighbor_probe_should_update_edge_observation() -> anyhow::Result<()> {
        let me_kp = OffchainKeypair::from_secret(&SECRET_0)?;
        let me = *me_kp.public();
        let peer_kp = OffchainKeypair::from_secret(&SECRET_1)?;
        let peer_key = *peer_kp.public();

        let graph = ChannelGraph::new(me);
        graph.add_node(peer_key);
        graph.add_edge(&me, &peer_key)?;

        let rtt = std::time::Duration::from_millis(100);
        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Ok(EdgeTransportTelemetry::Neighbor(TestNeighbor { peer: peer_key, rtt }));
        graph
            .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
            .await;

        let obs = graph.edge(&me, &peer_key).context("edge observation should exist")?;
        let immediate = obs
            .immediate_qos()
            .context("immediate QoS should be present after probe")?;
        assert_eq!(immediate.average_latency().context("latency should be set")?, rtt / 2,);
        Ok(())
    }

    #[tokio::test]
    async fn probe_timeout_should_record_as_failed_probe() -> anyhow::Result<()> {
        let me_kp = OffchainKeypair::from_secret(&SECRET_0)?;
        let me = *me_kp.public();
        let peer_kp = OffchainKeypair::from_secret(&SECRET_1)?;
        let peer_key = *peer_kp.public();

        let graph = ChannelGraph::new(me);
        graph.add_node(peer_key);
        graph.add_edge(&me, &peer_key)?;

        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Err(NetworkGraphError::ProbeNeighborTimeout(Box::new(peer_key)));
        graph
            .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
            .await;

        let obs = graph.edge(&me, &peer_key).context("edge observation should exist")?;
        let immediate = obs
            .immediate_qos()
            .context("immediate QoS should be present after failed probe")?;
        assert!(immediate.average_latency().is_none());
        assert!(immediate.average_probe_rate() < 1.0);
        Ok(())
    }

    #[tokio::test]
    async fn capacity_update_should_set_edge_capacity() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        let capacity_update = hopr_api::graph::EdgeCapacityUpdate {
            src: me,
            dest: peer,
            capacity: Some(1000),
        };
        graph
            .record_edge::<TestNeighbor, TestPath>(hopr_api::graph::MeasurableEdge::Capacity(Box::new(capacity_update)))
            .await;

        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        let intermediate = obs
            .intermediate_qos()
            .context("intermediate QoS should be present after capacity update")?;
        assert_eq!(intermediate.capacity(), Some(1000));
        Ok(())
    }

    #[tokio::test]
    async fn capacity_update_should_accept_none_value() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        let capacity_update = hopr_api::graph::EdgeCapacityUpdate {
            src: me,
            dest: peer,
            capacity: None,
        };
        graph
            .record_edge::<TestNeighbor, TestPath>(hopr_api::graph::MeasurableEdge::Capacity(Box::new(capacity_update)))
            .await;

        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        let intermediate = obs.intermediate_qos().context("intermediate QoS should be present")?;
        assert_eq!(intermediate.capacity(), None);
        Ok(())
    }

    #[tokio::test]
    async fn record_node_should_add_node_to_graph() {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);

        assert!(!graph.contains_node(&peer));
        graph.record_node(peer).await;
        assert!(graph.contains_node(&peer));
    }

    #[tokio::test]
    async fn probe_should_create_edge_if_absent() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        // No explicit add_edge — record_edge should upsert

        let rtt = std::time::Duration::from_millis(80);
        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Ok(EdgeTransportTelemetry::Neighbor(TestNeighbor { peer: peer, rtt }));
        graph
            .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
            .await;

        assert!(graph.has_edge(&me, &peer), "probe should create edge via upsert");
        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        assert!(obs.immediate_qos().is_some());
        Ok(())
    }

    #[tokio::test]
    async fn multiple_probes_should_accumulate_in_observations() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        // Send several successful probes
        for _ in 0..5 {
            let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
                Ok(EdgeTransportTelemetry::Neighbor(TestNeighbor {
                    peer: peer,
                    rtt: std::time::Duration::from_millis(60),
                }));
            graph
                .record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry))
                .await;
        }

        let obs = graph.edge(&me, &peer).context("edge should exist")?;
        let qos = obs.immediate_qos().context("immediate QoS should exist")?;
        assert_eq!(
            qos.average_latency().context("latency should be set")?,
            std::time::Duration::from_millis(30), // rtt / 2 = 30ms
        );
        assert!(qos.average_probe_rate() > 0.9, "all probes succeeded");
        Ok(())
    }
}
