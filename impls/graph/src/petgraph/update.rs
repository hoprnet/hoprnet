use hopr_api::graph::{MeasurableEdge, MeasurableNode, NetworkGraphWrite, traits::EdgeObservableWrite};
use petgraph::graph::{EdgeIndex, NodeIndex};

use crate::{ChannelGraph, Observations, graph::InnerGraph};

/// Resolves a loopback path from serialized node-index bytes into a validated chain of edge indices.
///
/// The `path_bytes` encode a [`PathId`] where each `u64` is a [`NodeIndex`].
/// The path is expected to start and end at `me_idx` (a closed loop).
///
/// Walks consecutive node pairs, finding the connecting edge for each.
/// Stops when the loop closes back to `me_idx` or when no edge exists
/// between a pair. Returns `None` if the path bytes have wrong length,
/// the first node is not `me_idx`, or fewer than 2 edges can be resolved.
fn resolve_loopback_edges(inner: &InnerGraph, me_idx: NodeIndex, path_bytes: &[u8]) -> Option<Vec<EdgeIndex>> {
    if path_bytes.len() != size_of::<hopr_api::ct::PathId>() {
        tracing::warn!(
            path_len = path_bytes.len(),
            expected = size_of::<hopr_api::ct::PathId>(),
            "invalid loopback path byte length"
        );
        return None;
    }

    let mut path_id = [0u64; 5];
    for (i, chunk) in path_bytes.chunks_exact(8).enumerate() {
        path_id[i] = u64::from_le_bytes(chunk.try_into().expect("chunk is 8 bytes"));
    }

    let me_val = me_idx.index() as u64;

    // First node must be self
    if path_id[0] != me_val {
        tracing::warn!("loopback path does not start at self");
        return None;
    }

    // Find the closing node: the first reoccurrence of me after position 0
    let Some(end_pos) = path_id[1..].iter().position(|&v| v == me_val).map(|p| p + 1) else {
        tracing::warn!("loopback path does not close back to self");
        return None;
    };

    // Walk consecutive node pairs up to (and including) the closing node
    let mut edges = Vec::new();

    for pair in path_id[..=end_pos].windows(2) {
        let from = NodeIndex::new(pair[0] as usize);
        let to = NodeIndex::new(pair[1] as usize);
        let Some(edge) = inner.graph.find_edge(from, to) else {
            break;
        };
        edges.push(edge);
    }

    if edges.len() < 2 {
        tracing::warn!(
            edge_count = edges.len(),
            "loopback path too short to attribute intermediate measurement"
        );
        return None;
    }

    Some(edges)
}

impl hopr_api::graph::NetworkGraphUpdate for ChannelGraph {
    #[tracing::instrument(level = "debug", skip(self, update))]
    fn record_edge<N, P>(&self, update: MeasurableEdge<N, P>)
    where
        N: hopr_api::graph::MeasurablePeer + Send + Clone,
        P: hopr_api::graph::MeasurablePath + Send + Clone,
    {
        use hopr_api::graph::{
            EdgeLinkObservable,
            traits::{EdgeObservableRead, EdgeWeightType},
        };

        match update {
            MeasurableEdge::Probe(Ok(hopr_api::graph::EdgeTransportTelemetry::Neighbor(ref telemetry))) => {
                tracing::trace!(
                    peer = %telemetry.peer(),
                    latency_ms = telemetry.rtt().as_millis(),
                    "neighbor probe successful"
                );

                // Both directions are set for immediate connections, because the graph is directional
                // and must be directionally complete for looping traffic.
                self.upsert_edge(&self.me, telemetry.peer(), |obs| {
                    obs.record(EdgeWeightType::Immediate(Ok(telemetry.rtt() / 2)));
                });
                self.upsert_edge(telemetry.peer(), &self.me, |obs| {
                    obs.record(EdgeWeightType::Immediate(Ok(telemetry.rtt() / 2)));
                });
            }
            MeasurableEdge::Probe(Ok(hopr_api::graph::EdgeTransportTelemetry::Loopback(telemetry))) => {
                let mut inner = self.inner.write();
                let Some(me_idx) = inner.indices.get_by_left(&self.me).copied() else {
                    return;
                };
                let Some(edges) = resolve_loopback_edges(&inner, me_idx, telemetry.path()) else {
                    return;
                };

                let target_idx = edges.len() - 2;

                // Attributed duration = total RTT - sum of all known edge latencies.
                // For each edge (including the target), use intermediate QoS if available,
                // otherwise fall back to immediate QoS. The residual is attributed to the
                // target edge as its new intermediate measurement.
                let total_rtt = std::time::Duration::from_millis(telemetry.timestamp() as u64);
                let mut known_latency = std::time::Duration::ZERO;

                for &edge in &edges {
                    if let Some(weight) = inner.graph.edge_weight(edge) {
                        let lat = weight
                            .intermediate_qos()
                            .and_then(|q| q.average_latency())
                            .or_else(|| weight.immediate_qos().and_then(|q| q.average_latency()));
                        if let Some(lat) = lat {
                            known_latency += lat;
                        }
                    }
                }

                let attributed_duration = total_rtt.saturating_sub(known_latency);

                tracing::trace!(
                    target_edge = edges[target_idx].index(),
                    attributed_ms = attributed_duration.as_millis(),
                    total_rtt_ms = total_rtt.as_millis(),
                    path_edges = edges.len(),
                    "loopback probe attributed to intermediate edge"
                );

                if let Some(weight) = inner.graph.edge_weight_mut(edges[target_idx]) {
                    weight.record(EdgeWeightType::Intermediate(Ok(attributed_duration)));
                }
            }
            MeasurableEdge::Probe(Err(hopr_api::graph::NetworkGraphError::ProbeNeighborTimeout(ref peer))) => {
                tracing::trace!(
                    peer = %peer,
                    reason = "probe timeout",
                    "neighbor probe failed"
                );

                // Both directions are set for immediate connections, because the graph is directional
                // and must be directionally complete for looping traffic.
                self.upsert_edge(&self.me, peer, |obs| {
                    obs.record(EdgeWeightType::Immediate(Err(())));
                });
                self.upsert_edge(peer, &self.me, |obs| {
                    obs.record(EdgeWeightType::Immediate(Err(())));
                });
            }
            MeasurableEdge::Probe(Err(hopr_api::graph::NetworkGraphError::ProbeLoopbackTimeout(telemetry))) => {
                let mut inner = self.inner.write();
                let Some(me_idx) = inner.indices.get_by_left(&self.me).copied() else {
                    return;
                };
                let Some(edges) = resolve_loopback_edges(&inner, me_idx, telemetry.path()) else {
                    return;
                };

                let target_idx = edges.len() - 2;

                tracing::trace!(
                    target_edge = edges[target_idx].index(),
                    path_edges = edges.len(),
                    "loopback probe timeout attributed to intermediate edge"
                );

                if let Some(weight) = inner.graph.edge_weight_mut(edges[target_idx]) {
                    weight.record(EdgeWeightType::Intermediate(Err(())));
                }
            }
            MeasurableEdge::Capacity(update) => {
                self.upsert_edge(&update.src, &update.dest, |obs: &mut Observations| {
                    obs.record(EdgeWeightType::Capacity(update.capacity));
                });
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self, update))]
    fn record_node<N>(&self, update: N)
    where
        N: MeasurableNode + Clone + Send + Sync + 'static,
    {
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
    const SECRET_2: [u8; 32] = hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299");
    const SECRET_3: [u8; 32] = hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e");

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
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));

        let obs = graph.edge(&me, &peer_key).context("edge observation should exist")?;
        let immediate = obs
            .immediate_qos()
            .context("immediate QoS should be present after probe")?;
        assert_eq!(immediate.average_latency().context("latency should be set")?, rtt / 2,);
        Ok(())
    }

    #[tokio::test]
    async fn neighbor_probe_should_create_symmetric_edges() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        // No edges pre-created — upsert should create both directions

        let rtt = std::time::Duration::from_millis(100);
        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Ok(EdgeTransportTelemetry::Neighbor(TestNeighbor { peer, rtt }));
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));

        // me → peer
        let obs_fwd = graph.edge(&me, &peer).context("edge me→peer should exist")?;
        let imm_fwd = obs_fwd.immediate_qos().context("me→peer should have immediate QoS")?;
        assert_eq!(
            imm_fwd.average_latency().context("me→peer latency should be set")?,
            rtt / 2
        );

        // peer → me
        let obs_rev = graph.edge(&peer, &me).context("edge peer→me should exist")?;
        let imm_rev = obs_rev.immediate_qos().context("peer→me should have immediate QoS")?;
        assert_eq!(
            imm_rev.average_latency().context("peer→me latency should be set")?,
            rtt / 2
        );

        Ok(())
    }

    #[tokio::test]
    async fn neighbor_probe_timeout_should_create_symmetric_edges() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let peer = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        // No edges pre-created

        let telemetry: Result<EdgeTransportTelemetry<TestNeighbor, TestPath>, NetworkGraphError<TestPath>> =
            Err(NetworkGraphError::ProbeNeighborTimeout(Box::new(peer)));
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));

        // me → peer
        let obs_fwd = graph
            .edge(&me, &peer)
            .context("edge me→peer should exist after timeout")?;
        let imm_fwd = obs_fwd.immediate_qos().context("me→peer should have immediate QoS")?;
        assert!(
            imm_fwd.average_latency().is_none(),
            "failed probe should not set latency"
        );
        assert!(
            imm_fwd.average_probe_rate() < 1.0,
            "failed probe should lower success rate"
        );

        // peer → me
        let obs_rev = graph
            .edge(&peer, &me)
            .context("edge peer→me should exist after timeout")?;
        let imm_rev = obs_rev.immediate_qos().context("peer→me should have immediate QoS")?;
        assert!(
            imm_rev.average_latency().is_none(),
            "failed probe should not set latency on reverse"
        );
        assert!(
            imm_rev.average_probe_rate() < 1.0,
            "failed probe should lower success rate on reverse"
        );

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
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));

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
        graph.record_edge::<TestNeighbor, TestPath>(hopr_api::graph::MeasurableEdge::Capacity(Box::new(
            capacity_update,
        )));

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
        graph.record_edge::<TestNeighbor, TestPath>(hopr_api::graph::MeasurableEdge::Capacity(Box::new(
            capacity_update,
        )));

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
        graph.record_node(peer);
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
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));

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
            graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));
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

    /// A `MeasurablePath` carrying a serialized `PathId` and a timestamp for
    /// loopback probe telemetry tests.
    #[derive(Debug, Clone)]
    struct LoopbackTestPath {
        path_bytes: Vec<u8>,
        timestamp_ms: u128,
    }

    impl LoopbackTestPath {
        fn new(path_id: [u64; 5], timestamp_ms: u128) -> Self {
            let path_bytes = path_id.iter().flat_map(|v| v.to_le_bytes()).collect();
            Self {
                path_bytes,
                timestamp_ms,
            }
        }
    }

    impl MeasurablePath for LoopbackTestPath {
        fn id(&self) -> &[u8] {
            &[]
        }

        fn path(&self) -> &[u8] {
            &self.path_bytes
        }

        fn timestamp(&self) -> u128 {
            self.timestamp_ms
        }
    }

    /// Helper to send a loopback probe with the given path and timestamp.
    fn send_loopback(graph: &ChannelGraph, path_id: [u64; 5], timestamp_ms: u128) {
        let telemetry: Result<
            EdgeTransportTelemetry<TestNeighbor, LoopbackTestPath>,
            NetworkGraphError<LoopbackTestPath>,
        > = Ok(EdgeTransportTelemetry::Loopback(LoopbackTestPath::new(
            path_id,
            timestamp_ms,
        )));
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));
    }

    /// Helper to send a loopback timeout with the given path.
    fn send_loopback_timeout(graph: &ChannelGraph, path_id: [u64; 5]) {
        let telemetry: Result<
            EdgeTransportTelemetry<TestNeighbor, LoopbackTestPath>,
            NetworkGraphError<LoopbackTestPath>,
        > = Err(NetworkGraphError::ProbeLoopbackTimeout(LoopbackTestPath::new(
            path_id, 0,
        )));
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));
    }

    #[tokio::test]
    async fn loopback_three_hop_should_attribute_to_penultimate_edge() -> anyhow::Result<()> {
        // Loopback: me(0) → a(1) → b(2) → me(0)
        // PathId nodes: [me=0, a=1, b=2, me=0, 0]
        // Resolved edges: me→a, a→b, b→me (3 edges)
        // Target = edges[len-2] = edges[1] = a→b
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &me)?; // return edge

        send_loopback(&graph, [0, 1, 2, 0, 0], 200);

        let obs = graph.edge(&a, &b).context("edge a→b should exist")?;
        let qos = obs
            .intermediate_qos()
            .context("intermediate QoS should be present on a→b")?;
        assert_eq!(
            qos.average_latency().context("latency should be set")?,
            std::time::Duration::from_millis(200),
        );

        // me→a should NOT have intermediate QoS from this probe
        let obs_me_a = graph.edge(&me, &a).context("edge me→a should exist")?;
        assert!(obs_me_a.intermediate_qos().is_none());

        Ok(())
    }

    #[tokio::test]
    async fn loopback_four_hop_should_attribute_to_penultimate_edge() -> anyhow::Result<()> {
        // Loopback: me(0) → a(1) → b(2) → c(3) → me(0)
        // PathId nodes: [me=0, a=1, b=2, c=3, me=0]
        // Resolved edges: me→a, a→b, b→c, c→me (4 edges)
        // Target = edges[len-2] = edges[2] = b→c
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&c, &me)?; // return edge

        send_loopback(&graph, [0, 1, 2, 3, 0], 300);

        // Edge b→c (target) should have the intermediate QoS
        let obs = graph.edge(&b, &c).context("edge b→c should exist")?;
        let qos = obs
            .intermediate_qos()
            .context("intermediate QoS should be present on b→c")?;
        assert_eq!(
            qos.average_latency().context("latency should be set")?,
            std::time::Duration::from_millis(300),
            "no preceding intermediate latencies, so full RTT is attributed"
        );

        // Earlier edges should NOT have intermediate QoS from this probe
        let obs_me_a = graph.edge(&me, &a).context("edge me→a should exist")?;
        assert!(obs_me_a.intermediate_qos().is_none());
        let obs_a_b = graph.edge(&a, &b).context("edge a→b should exist")?;
        assert!(obs_a_b.intermediate_qos().is_none());

        Ok(())
    }

    #[tokio::test]
    async fn loopback_should_subtract_known_preceding_latencies() -> anyhow::Result<()> {
        // Loopback: me(0) → a(1) → b(2) → c(3) → me(0)
        // Resolved edges: me→a, a→b, b→c, c→me (4 edges). Target = b→c (idx 2).
        // Preceding edges = [me→a, a→b]
        // Pre-set me→a = 80ms, a→b = 40ms
        // Attributed for b→c = 300 - 80 - 40 = 180ms
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&c, &me)?; // return edge

        // Pre-set intermediate latency on me→a and a→b
        graph.upsert_edge(&me, &a, |obs| {
            use hopr_api::graph::traits::EdgeObservableWrite;
            obs.record(hopr_api::graph::traits::EdgeWeightType::Intermediate(Ok(
                std::time::Duration::from_millis(80),
            )));
        });
        graph.upsert_edge(&a, &b, |obs| {
            use hopr_api::graph::traits::EdgeObservableWrite;
            obs.record(hopr_api::graph::traits::EdgeWeightType::Intermediate(Ok(
                std::time::Duration::from_millis(40),
            )));
        });

        send_loopback(&graph, [0, 1, 2, 3, 0], 300);

        let obs = graph.edge(&b, &c).context("edge b→c should exist")?;
        let qos = obs
            .intermediate_qos()
            .context("intermediate QoS should be present on b→c")?;
        assert_eq!(
            qos.average_latency().context("latency should be set")?,
            std::time::Duration::from_millis(180),
            "300ms total - 80ms (me→a) - 40ms (a→b) = 180ms attributed to b→c"
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_should_subtract_immediate_latency_on_first_edge() -> anyhow::Result<()> {
        // Loopback: me(0) → a(1) → b(2) → me(0)
        // Resolved edges: me→a, a→b, b→me (3 edges). Target = a→b (idx 1).
        // me→a has immediate QoS = 60ms (from my neighbor probing of a), no intermediate yet.
        // Attributed for a→b = 200 - 60 = 140ms
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &me)?;

        // Pre-set immediate QoS on me→a (my direct measurement to neighbor a)
        graph.upsert_edge(&me, &a, |obs| {
            use hopr_api::graph::traits::EdgeObservableWrite;
            obs.record(hopr_api::graph::traits::EdgeWeightType::Immediate(Ok(
                std::time::Duration::from_millis(60),
            )));
        });

        send_loopback(&graph, [0, 1, 2, 0, 0], 200);

        let obs = graph.edge(&a, &b).context("edge a→b should exist")?;
        let qos = obs
            .intermediate_qos()
            .context("intermediate QoS should be present on a→b")?;
        assert_eq!(
            qos.average_latency().context("latency should be set")?,
            std::time::Duration::from_millis(140),
            "200ms total - 60ms (me→a immediate) = 140ms attributed to a→b"
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_invalid_path_length_should_be_ignored() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a)?;

        // Send loopback with wrong-length path bytes (not 40 bytes)
        let telemetry: Result<
            EdgeTransportTelemetry<TestNeighbor, LoopbackTestPath>,
            NetworkGraphError<LoopbackTestPath>,
        > = Ok(EdgeTransportTelemetry::Loopback(LoopbackTestPath {
            path_bytes: vec![0u8; 16], // wrong: 16 bytes instead of 40
            timestamp_ms: 100,
        }));
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));

        // Edge should have no intermediate observations
        let obs = graph.edge(&me, &a).context("edge should exist")?;
        assert!(
            obs.intermediate_qos().is_none(),
            "invalid path bytes should not produce any intermediate measurement"
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_single_edge_path_should_be_ignored_if_no_immediate_or_intermediate_result_exists_for_the_edge()
    -> anyhow::Result<()> {
        // A path with only 1 edge has no "edge before the last"
        // me(0) → a(1)
        // PathId nodes: [me=0, a=1, 0, 0, 0]
        // Trailing 0 = me which is already visited → stops at 1 edge
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a)?;

        send_loopback(&graph, [0, 1, 0, 0, 0], 100);

        let obs = graph.edge(&me, &a).context("edge should exist")?;
        assert!(
            obs.intermediate_qos().is_none(),
            "single-edge path should not produce intermediate measurement"
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_two_edge_path_should_attribute_when_return_edge_exists() -> anyhow::Result<()> {
        // Loopback: me(0) → a(1) → me(0)
        // PathId nodes: [me=0, a=1, me=0, 0, 0]
        // Resolved edges: me→a, a→me (2 edges). Target = me→a (idx 0).
        // me→a already has immediate QoS = 50ms (from my neighbor probing).
        // No known latency on the non-target edge a→me, so attributed = full RTT.
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &me)?; // return edge

        // Pre-set immediate QoS on me→a (my direct neighbor measurement)
        graph.upsert_edge(&me, &a, |obs| {
            use hopr_api::graph::traits::EdgeObservableWrite;
            obs.record(hopr_api::graph::traits::EdgeWeightType::Immediate(Ok(
                std::time::Duration::from_millis(50),
            )));
        });

        send_loopback(&graph, [0, 1, 0, 0, 0], 100);

        let obs = graph.edge(&me, &a).context("edge me→a should exist")?;
        let qos = obs
            .intermediate_qos()
            .context("intermediate QoS should be present on me→a")?;
        assert_eq!(
            qos.average_latency().context("latency should be set")?,
            std::time::Duration::from_millis(50),
            "100ms total - 50ms (me→a immediate) = 50ms attributed to me→a"
        );

        // Immediate QoS should still be intact
        let imm = obs
            .immediate_qos()
            .context("immediate QoS should still be present on me→a")?;
        assert_eq!(
            imm.average_latency().context("immediate latency should be set")?,
            std::time::Duration::from_millis(50),
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_broken_chain_should_be_ignored() -> anyhow::Result<()> {
        // Nodes exist but no edge connects a to c (only b→c exists)
        // me(0) → a(1), b(2) → c(3)
        // PathId nodes: [me=0, a=1, c=3, 0, 0]
        // Edge me→a exists, but edge a→c does NOT → chain breaks, 1 edge < 2
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&b, &c)?; // b→c, NOT a→c

        send_loopback(&graph, [0, 1, 3, 0, 0], 200);

        let obs_me_a = graph.edge(&me, &a).context("edge me→a should exist")?;
        assert!(
            obs_me_a.intermediate_qos().is_none(),
            "broken chain should not attribute any intermediate measurement"
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_wrong_start_node_should_be_ignored() -> anyhow::Result<()> {
        // PathId starts with node 99 which is not me → early reject
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a)?;

        send_loopback(&graph, [99, 1, 0, 0, 0], 200);

        let obs = graph.edge(&me, &a).context("edge should exist")?;
        assert!(
            obs.intermediate_qos().is_none(),
            "wrong start node should not produce any measurement"
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_probes_should_accumulate_on_target_edge() -> anyhow::Result<()> {
        // Send multiple loopback probes for the same target edge
        // Loopback: me(0) → a(1) → b(2) → me(0). Target = a→b.
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &me)?; // return edge

        // Send 5 probes all with 100ms RTT.
        // After each probe the target's intermediate QoS is subtracted from subsequent
        // attributions, so the attributed value converges rather than staying at 100ms.
        for _ in 0..5 {
            send_loopback(&graph, [0, 1, 2, 0, 0], 100);
        }

        let obs = graph.edge(&a, &b).context("edge a→b should exist")?;
        let qos = obs.intermediate_qos().context("intermediate QoS should be present")?;
        assert!(
            qos.average_latency().is_some(),
            "latency should be set after multiple probes"
        );
        assert!(
            qos.average_probe_rate() > 0.9,
            "all probes succeeded, rate should be high"
        );

        Ok(())
    }

    // This is handled by the moving average object, but the expectation test can stay here.
    #[tokio::test]
    async fn loopback_saturating_sub_should_not_underflow() -> anyhow::Result<()> {
        // If preceding edge latencies exceed total RTT, duration should saturate at 0
        // Loopback: me(0) → a(1) → b(2) → c(3) → me(0). Target = b→c.
        // Preceding = [me→a, a→b] with me→a = 500ms
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&c, &me)?; // return edge

        // Pre-set me→a intermediate latency to 500ms
        graph.upsert_edge(&me, &a, |obs| {
            use hopr_api::graph::traits::EdgeObservableWrite;
            obs.record(hopr_api::graph::traits::EdgeWeightType::Intermediate(Ok(
                std::time::Duration::from_millis(500),
            )));
        });

        // Total RTT = 100ms, but preceding latency is 500ms → 100 - 500 saturates to 0
        send_loopback(&graph, [0, 1, 2, 3, 0], 100);

        let obs = graph.edge(&b, &c).context("edge b→c should exist")?;
        let qos = obs.intermediate_qos().context("intermediate QoS should be present")?;
        // Duration::ZERO means latency_average gets updated with 0ms
        // which the EMA may not report as Some(0) but rather None if <= 0
        // Let's check the probe rate instead — it should be recorded
        assert!(
            qos.average_probe_rate() > 0.0,
            "probe should still be recorded even with saturated duration"
        );

        Ok(())
    }

    #[tokio::test]
    async fn loopback_timeout_should_record_failed_intermediate_on_target_edge() -> anyhow::Result<()> {
        // Loopback: me(0) → a(1) → b(2) → me(0)
        // PathId nodes: [me=0, a=1, b=2, me=0, 0]
        // Resolved edges: me→a, a→b, b→me. Target = edges[1] = a→b
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &me)?; // return edge

        send_loopback_timeout(&graph, [0, 1, 2, 0, 0]);

        let obs = graph.edge(&a, &b).context("edge a→b should exist")?;
        let qos = obs
            .intermediate_qos()
            .context("intermediate QoS should be present on a→b after timeout")?;
        assert!(qos.average_latency().is_none(), "failed probe should not set latency");
        assert!(qos.average_probe_rate() < 1.0, "failed probe should lower success rate");

        // me→a should NOT have intermediate QoS
        let obs_me_a = graph.edge(&me, &a).context("edge me→a should exist")?;
        assert!(obs_me_a.intermediate_qos().is_none());

        Ok(())
    }

    #[tokio::test]
    async fn loopback_timeout_four_hop_should_attribute_to_penultimate_edge() -> anyhow::Result<()> {
        // Loopback: me(0) → a(1) → b(2) → c(3) → me(0)
        // Target = last resolved edge = b→c
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);
        let b = pubkey_from(&SECRET_2);
        let c = pubkey_from(&SECRET_3);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;
        graph.add_edge(&b, &c)?;
        graph.add_edge(&c, &me)?;

        send_loopback_timeout(&graph, [0, 1, 2, 3, 0]);

        // Edge b→c (target) should have a failed intermediate record
        let obs = graph.edge(&b, &c).context("edge b→c should exist")?;
        let qos = obs
            .intermediate_qos()
            .context("intermediate QoS should be present on b→c")?;
        assert!(qos.average_latency().is_none());
        assert!(qos.average_probe_rate() < 1.0);

        // Earlier edges should NOT have intermediate QoS
        let obs_me_a = graph.edge(&me, &a).context("edge me→a should exist")?;
        assert!(obs_me_a.intermediate_qos().is_none());
        let obs_a_b = graph.edge(&a, &b).context("edge a→b should exist")?;
        assert!(obs_a_b.intermediate_qos().is_none());

        Ok(())
    }

    #[tokio::test]
    async fn loopback_timeout_invalid_path_should_be_ignored() -> anyhow::Result<()> {
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a)?;

        // Wrong-length path
        let telemetry: Result<
            EdgeTransportTelemetry<TestNeighbor, LoopbackTestPath>,
            NetworkGraphError<LoopbackTestPath>,
        > = Err(NetworkGraphError::ProbeLoopbackTimeout(LoopbackTestPath {
            path_bytes: vec![0u8; 8],
            timestamp_ms: 0,
        }));
        graph.record_edge(hopr_api::graph::MeasurableEdge::Probe(telemetry));

        let obs = graph.edge(&me, &a).context("edge should exist")?;
        assert!(obs.intermediate_qos().is_none());

        Ok(())
    }

    #[tokio::test]
    async fn loopback_timeout_single_edge_should_be_ignored() -> anyhow::Result<()> {
        // me(0) → a(1), PathId: [0, 1, 0, 0, 0] → 1 edge < 2
        let me = pubkey_from(&SECRET_0);
        let a = pubkey_from(&SECRET_1);

        let graph = ChannelGraph::new(me);
        graph.add_node(a);
        graph.add_edge(&me, &a)?;

        send_loopback_timeout(&graph, [0, 1, 0, 0, 0]);

        let obs = graph.edge(&me, &a).context("edge should exist")?;
        assert!(
            obs.intermediate_qos().is_none(),
            "single-edge timeout should not produce intermediate measurement"
        );

        Ok(())
    }
}
