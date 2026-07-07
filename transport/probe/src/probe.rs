use std::sync::Arc;

use futures::{FutureExt, SinkExt, StreamExt};
use futures_concurrency::stream::StreamExt as _;
use hopr_api::{
    ct::{ProbeRouting, ProbingTrafficGeneration},
    graph::{EdgeTransportTelemetry, NetworkGraphError, NetworkGraphUpdate, NetworkGraphView},
    types::{
        crypto::types::OffchainPublicKey, crypto_random::Randomizable, internal::prelude::*,
        primitive::traits::AsUnixTimestamp,
    },
};
use hopr_protocol_app::{
    prelude::{ApplicationDataIn, ApplicationDataOut, OutgoingPacketInfo, ReservedTag},
    v1::Tag,
};
use hopr_transport_tag_allocator::{AllocatedTag, TagAllocator};
use hopr_utils::{platform::time::native::current_time, runtime::AbortableList};

use crate::{
    HoprProbeProcess,
    config::ProbeConfig,
    content::Message,
    ping::PingQueryReplier,
    types::{NeighborProbe, NeighborTelemetry, PathTelemetry},
};

type CacheNeighborKey = (HoprPseudonym, NeighborProbe);
type CacheNeighborValue = (Box<NodeId>, std::time::Duration, Option<PingQueryReplier>);

/// Result of classifying one incoming message through the probe layer.
pub enum ProbeDispatch {
    /// The message was a probe message and has been consumed internally.
    Consumed,
    /// The message was not related to probing; caller should route it further.
    Passthrough(HoprPseudonym, ApplicationDataIn),
}

/// Shared state used to classify incoming messages as probe or non-probe.
///
/// Obtained from [`Probe::continuously_scan`]. Use [`filter_stream`](ProbeClassifierState::filter_stream)
/// to wrap an incoming stream so that probe messages are consumed internally and non-probe messages
/// are yielded to the caller.
#[derive(Clone)]
pub struct ProbeClassifierState<G> {
    active_neighbor_probes: moka::future::Cache<CacheNeighborKey, CacheNeighborValue>,
    active_path_probes: moka::future::Cache<Tag, (PathTelemetry, Arc<AllocatedTag>)>,
    network_graph: G,
}

impl<G> ProbeClassifierState<G>
where
    G: NetworkGraphUpdate + Clone + Send + Sync + 'static,
{
    /// Classify one incoming `(pseudonym, data)` pair.
    ///
    /// The `push_to_network` sink is used to send pong replies when the message is a Ping.
    /// Returns `Consumed` if the message was a probe, or `Passthrough` for all other messages.
    pub async fn classify<T>(
        &self,
        mut push_to_network: T,
        pseudonym: HoprPseudonym,
        in_data: ApplicationDataIn,
    ) -> ProbeDispatch
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Unpin + Send + 'static,
        T::Error: Send,
    {
        let tag: Tag = in_data.data.application_tag;

        if let Some((path_telemetry, _allocated_tag)) = self.active_path_probes.remove(&tag).await {
            tracing::debug!(%tag, "loopback probe successfully received");
            self.network_graph
                .record_edge::<NeighborTelemetry, PathTelemetry>(hopr_api::graph::MeasurableEdge::Probe(Ok(
                    EdgeTransportTelemetry::Loopback(path_telemetry),
                )));
        } else if tag == ReservedTag::Ping.into() {
            let message: anyhow::Result<Message> = in_data
                .data
                .try_into()
                .map_err(|e| anyhow::anyhow!("failed to convert data into message: {e}"));

            match message {
                Ok(message) => match message {
                    Message::Telemetry(_) => {
                        tracing::warn!(%pseudonym, "received telemetry on reserved ping tag, ignoring");
                    }
                    Message::Probe(NeighborProbe::Ping(ping)) => {
                        tracing::debug!(%pseudonym, nonce = const_hex::encode(ping), "received ping");
                        tracing::trace!(%pseudonym, nonce = const_hex::encode(ping), "wrapping a pong in the found SURB");

                        let message = Message::Probe(NeighborProbe::Pong(ping));
                        if let Ok(data) = message.try_into() {
                            let routing = DestinationRouting::Return(pseudonym.into());
                            let data = ApplicationDataOut::with_no_packet_info(data);
                            if let Err(_error) = push_to_network.send((routing, data)).await {
                                tracing::debug!(%pseudonym, "failed to send back a pong");
                            }
                        } else {
                            tracing::debug!(%pseudonym, "failed to convert pong message into data");
                        }
                    }
                    Message::Probe(NeighborProbe::Pong(pong)) => {
                        tracing::debug!(%pseudonym, nonce = const_hex::encode(pong), "received pong");
                        if let Some((peer, start, replier)) = self
                            .active_neighbor_probes
                            .remove(&(pseudonym, NeighborProbe::Ping(pong)))
                            .await
                        {
                            let latency = current_time().as_unix_timestamp().saturating_sub(start);

                            if let NodeId::Offchain(opk) = peer.as_ref() {
                                tracing::debug!(%pseudonym, nonce = const_hex::encode(pong), latency_ms = latency.as_millis(), "probe successful");
                                self.network_graph.record_edge::<NeighborTelemetry, PathTelemetry>(
                                    hopr_api::graph::MeasurableEdge::Probe(Ok(EdgeTransportTelemetry::Neighbor(
                                        NeighborTelemetry {
                                            peer: *opk,
                                            rtt: latency,
                                        },
                                    ))),
                                )
                            } else {
                                tracing::warn!(%pseudonym, nonce = const_hex::encode(pong), latency_ms = latency.as_millis(), "probe successful to non-offchain peer");
                            }

                            if let Some(replier) = replier {
                                replier.notify(Ok(latency));
                            }
                        } else {
                            tracing::warn!(%pseudonym, nonce = const_hex::encode(pong), possible_reasons = "[timeout, adversary]", "received pong for unknown probe");
                        }
                    }
                },
                Err(error) => tracing::error!(%pseudonym, %error, "cannot deserialize message"),
            }
        } else {
            return ProbeDispatch::Passthrough(pseudonym, in_data);
        }

        ProbeDispatch::Consumed
    }

    /// Wraps `stream` as an in-place filter: probe messages are handled internally (telemetry,
    /// pong replies via `push_to_network`), non-probe messages are yielded.
    pub fn filter_stream<T, S>(
        self,
        push_to_network: T,
        stream: S,
    ) -> impl futures::Stream<Item = (HoprPseudonym, ApplicationDataIn)>
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Unpin + Send + Sync + 'static,
        T::Error: Send,
        S: futures::Stream<Item = (HoprPseudonym, ApplicationDataIn)>,
    {
        use futures::StreamExt;
        stream.filter_map(move |(pseudonym, data)| {
            let state = self.clone();
            let push = push_to_network.clone();
            async move {
                match state.classify(push, pseudonym, data).await {
                    ProbeDispatch::Consumed => None,
                    ProbeDispatch::Passthrough(ps, d) => Some((ps, d)),
                }
            }
        })
    }
}

/// Probe functionality builder.
///
/// The builder holds information about this node's own addresses and the configuration for the probing process. It is
/// then used to construct the probing process itself.
pub struct Probe {
    /// Probe configuration.
    cfg: ProbeConfig,
    /// Tag allocator for probing telemetry tags.
    tag_allocator: Arc<dyn TagAllocator + Send + Sync>,
}

impl Probe {
    pub fn new(cfg: ProbeConfig, tag_allocator: Arc<dyn TagAllocator + Send + Sync>) -> Self {
        Self { cfg, tag_allocator }
    }

    /// The main function that assembles and starts the probing process.
    ///
    /// Returns the abortable list of background tasks (probe emission) and a
    /// [`ProbeClassifierState`] for inline classification of incoming messages.
    /// Use [`ProbeClassifierState::filter_stream`] to wrap the incoming stream.
    pub async fn continuously_scan<T, V, Tr, G>(
        self,
        api_out: T,       // lower tx channel for sending outgoing probes and pong replies
        manual_events: V, // explicit requests from the API
        probing_traffic_generator: Tr,
        network_graph: G,
    ) -> (AbortableList<HoprProbeProcess>, ProbeClassifierState<G>)
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
        T::Error: Send,
        V: futures::Stream<Item = (OffchainPublicKey, PingQueryReplier)> + Send + 'static,
        Tr: ProbingTrafficGeneration + Send + Sync + 'static,
        G: NetworkGraphView + NetworkGraphUpdate + Clone + Send + Sync + 'static,
    {
        let max_parallel_probes = self.cfg.max_parallel_probes;

        let probing_routes = probing_traffic_generator.build();

        // Currently active probes
        let network_graph_internal_neighbor = network_graph.clone();
        let network_graph_internal_path = network_graph.clone();
        let timeout = self.cfg.timeout;
        let active_neighbor_probes: moka::future::Cache<CacheNeighborKey, CacheNeighborValue> =
            moka::future::Cache::builder()
                .time_to_live(timeout)
                .max_capacity(100_000)
                .async_eviction_listener(
                    move |k: Arc<CacheNeighborKey>,
                          v: CacheNeighborValue,
                          cause|
                          -> moka::notification::ListenerFuture {
                        if matches!(cause, moka::notification::RemovalCause::Expired) {
                            // If the eviction cause is expiration => record as a failed probe
                            let store = network_graph_internal_neighbor.clone();
                            let (peer, _start, notifier) = v;

                            tracing::debug!(%peer, pseudonym = %k.0, probe = %k.1, reason = "timeout", "neighbor probe failed");
                            if let Some(replier) = notifier {
                                if matches!(peer.as_ref(), NodeId::Offchain(_)) {
                                    replier.notify(Err(()));
                                } else {
                                    tracing::warn!(
                                        reason = "non-offchain peer",
                                        "cannot notify timeout for non-offchain peer"
                                    );
                                }
                            };

                            if let NodeId::Offchain(opk) = peer.as_ref() {
                                let opk: OffchainPublicKey = *opk;
                                store
                                    .record_edge::<NeighborTelemetry, PathTelemetry>(
                                        hopr_api::graph::MeasurableEdge::Probe(Err(
                                            NetworkGraphError::ProbeNeighborTimeout(Box::new(opk)),
                                        )),
                                    );
                                futures::FutureExt::boxed(futures::future::ready(()))

                            } else {
                                futures::FutureExt::boxed(futures::future::ready(()))
                            }
                        } else {
                            // If the eviction cause is not expiration, nothing needs to be done
                            futures::FutureExt::boxed(futures::future::ready(()))
                        }
                    },
                )
                .build();

        let active_path_probes: moka::future::Cache<Tag, (PathTelemetry, Arc<AllocatedTag>)> =
            moka::future::Cache::builder()
                .time_to_live(timeout)
                .max_capacity(100_000)
                .async_eviction_listener(
                    move |tag: Arc<Tag>,
                          (path, _allocated_tag): (PathTelemetry, Arc<AllocatedTag>),
                          cause|
                          -> moka::notification::ListenerFuture {
                        if matches!(cause, moka::notification::RemovalCause::Expired) {
                            // If the eviction cause is expiration => record as a failed probe
                            let store = network_graph_internal_path.clone();

                            tracing::debug!(%tag, reason = "timeout", "loopback probe failed");

                            store.record_edge::<NeighborTelemetry, PathTelemetry>(
                                hopr_api::graph::MeasurableEdge::Probe(Err(NetworkGraphError::ProbeLoopbackTimeout(
                                    path,
                                ))),
                            );
                            futures::FutureExt::boxed(futures::future::ready(()))
                        } else {
                            // If the eviction cause is not expiration, nothing needs to be done
                            futures::FutureExt::boxed(futures::future::ready(()))
                        }
                    },
                )
                .build();

        let push_to_network = api_out.clone();

        let mut processes = AbortableList::default();

        // -- Emit probes --
        let direct_neighbors =
            probing_routes
                .map(|peer| (peer, None))
                .merge(manual_events.filter_map(|(peer, notifier)| async move {
                    let routing = DestinationRouting::Forward {
                        destination: Box::new(peer.into()),
                        pseudonym: Some(HoprPseudonym::random()),
                        forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                        return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                    };
                    Some((ProbeRouting::Neighbor(routing), Some(notifier)))
                }));

        let tag_allocator = self.tag_allocator.clone();
        let classifier_neighbor_probes = active_neighbor_probes.clone();
        let classifier_path_probes = active_path_probes.clone();
        let emit_diag = hopr_utils::runtime::diagnostics::ConcurrentDiagnostics::new(
            "probe_emit_for_each_concurrent",
            module_path!(),
            file!(),
            line!(),
        );
        processes.insert(
            HoprProbeProcess::Emit,
            hopr_utils::spawn_as_abortable_named!("probe_emit", async move {
                direct_neighbors
                    .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
                        let active_neighbor_probes = active_neighbor_probes.clone();
                        let active_path_probes = active_path_probes.clone();
                        let push_to_network = push_to_network.clone();
                        let tag_allocator = tag_allocator.clone();
                        let emit_diag = emit_diag.clone();

                        emit_diag.wrap(|| async move {
                            match peer {
                                ProbeRouting::Neighbor(DestinationRouting::Forward {
                                    destination,
                                    pseudonym,
                                    forward_options,
                                    return_options,
                                }) => {
                                    let nonce = NeighborProbe::random_nonce();

                                    let message = Message::Probe(nonce);

                                    if let Ok(data) = message.try_into() {
                                        let routing = DestinationRouting::Forward {
                                            destination: destination.clone(),
                                            pseudonym,
                                            forward_options,
                                            return_options,
                                        };
                                        // Neighbor probes are sent to a direct neighbor and returned via a zero-hop
                                        // return path: only 1 SURB is ever consumed. See hoprnet/hoprnet#7972.
                                        let data = ApplicationDataOut {
                                            data,
                                            packet_info: Some(OutgoingPacketInfo {
                                                max_surbs_in_packet: 1,
                                                ..Default::default()
                                            }),
                                        };
                                        let mut push_to_network = push_to_network.clone();

                                        if let Err(_error) = push_to_network.send((routing, data)).await {
                                            tracing::debug!("failed to send out a ping");
                                        } else {
                                            active_neighbor_probes
                                                .insert(
                                                    (
                                                        pseudonym
                                                            .expect("the pseudonym must be present in Forward routing"),
                                                        nonce,
                                                    ),
                                                    (destination, current_time().as_unix_timestamp(), notifier),
                                                )
                                                .await;
                                        }
                                    } else {
                                        tracing::debug!("failed to convert ping message into data");
                                    }
                                }
                                ProbeRouting::Neighbor(DestinationRouting::Return(_surb_matcher)) => tracing::error!(
                                    error = "logical error",
                                    "resolved transport routing is not forward"
                                ),
                                ProbeRouting::Looping((routing, path_id)) => {
                                    let message = Message::Telemetry(PathTelemetry {
                                        id: hopr_api::types::crypto_random::random_bytes(),
                                        path: std::array::from_fn(|i| path_id[i / 8].to_le_bytes()[i % 8]),
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis(),
                                    });

                                    if let Some(allocated_tag) = tag_allocator.allocate() {
                                        let tag_value = allocated_tag.value();

                                        if let Ok(packet) = hopr_protocol_app::prelude::ApplicationData::new(
                                            tag_value,
                                            message.to_bytes().as_ref(),
                                        ) {
                                            let mut push_to_network = push_to_network.clone();

                                            // Loopback telemetry probes are self-routed and never replied to via
                                            // SURB, so no SURBs should be bundled. See hoprnet/hoprnet#7972.
                                            if let Err(_error) = push_to_network
                                                .send((
                                                    routing,
                                                    ApplicationDataOut {
                                                        data: packet,
                                                        packet_info: Some(OutgoingPacketInfo {
                                                            max_surbs_in_packet: 0,
                                                            ..Default::default()
                                                        }),
                                                    },
                                                ))
                                                .await
                                            {
                                                tracing::debug!("failed to send out a ping");
                                            } else {
                                                // the object is constructed above, so will always match
                                                if let Message::Telemetry(telemetry) = message {
                                                    active_path_probes
                                                        .insert(tag_value.into(), (telemetry, Arc::new(allocated_tag)))
                                                        .await;
                                                }
                                            }
                                        } else {
                                            tracing::debug!("failed to construct data for path telemetry")
                                        }
                                    } else {
                                        tracing::warn!("probing telemetry tag pool exhausted, skipping loopback probe");
                                    }
                                }
                            }
                        })
                    })
                    .inspect(|_| {
                        tracing::warn!(
                            task = "transport (probe - generate outgoing)",
                            "long-running background task finished"
                        )
                    })
                    .await;
            }),
        );

        let classifier = ProbeClassifierState {
            active_neighbor_probes: classifier_neighbor_probes,
            active_path_probes: classifier_path_probes,
            network_graph,
        };

        (processes, classifier)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::RwLock, time::Duration};

    use async_trait::async_trait;
    use futures::future::BoxFuture;
    use hopr_api::{
        graph::{
            EdgeLinkObservable, MeasurableEdge, NetworkGraphError,
            traits::{EdgeNetworkObservableRead, EdgeObservableRead, EdgeObservableWrite, EdgeProtocolObservable},
        },
        types::crypto::keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    };
    use hopr_protocol_app::prelude::{ApplicationData, ReservedTag, Tag};

    use super::*;
    use crate::errors::ProbeError;

    lazy_static::lazy_static!(
        static ref OFFCHAIN_KEYPAIR: OffchainKeypair = OffchainKeypair::random();
        static ref ONCHAIN_KEYPAIR: ChainKeypair = ChainKeypair::random();
        static ref NEIGHBOURS: Vec<OffchainPublicKey> = vec![
            *OffchainKeypair::random().public(),
            *OffchainKeypair::random().public(),
            *OffchainKeypair::random().public(),
            *OffchainKeypair::random().public(),
        ];
    );

    /// Test stub implementation of Observable.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct TestEdgeTransportObservations;

    impl EdgeLinkObservable for TestEdgeTransportObservations {
        fn record(&mut self, _latency: std::result::Result<Duration, ()>) {}

        fn average_latency(&self) -> Option<Duration> {
            None
        }

        fn average_probe_rate(&self) -> f64 {
            1.0
        }

        fn score(&self) -> f64 {
            1.0
        }
    }

    impl EdgeNetworkObservableRead for TestEdgeTransportObservations {
        fn is_connected(&self) -> bool {
            true
        }
    }

    impl EdgeProtocolObservable for TestEdgeTransportObservations {
        fn capacity(&self) -> Option<u128> {
            None
        }
    }

    impl hopr_api::graph::EdgeImmediateProtocolObservable for TestEdgeTransportObservations {
        fn ack_rate(&self) -> Option<f64> {
            None
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub struct TestEdgeObservations;

    impl EdgeObservableWrite for TestEdgeObservations {
        fn record(&mut self, _measurement: hopr_api::graph::traits::EdgeWeightType) {}
    }

    impl EdgeObservableRead for TestEdgeObservations {
        type ImmediateMeasurement = TestEdgeTransportObservations;
        type IntermediateMeasurement = TestEdgeTransportObservations;

        fn last_update(&self) -> std::time::Duration {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
        }

        fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement> {
            None
        }

        fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement> {
            None
        }

        fn score(&self) -> f64 {
            1.0
        }
    }

    #[derive(Debug, Clone)]
    pub struct PeerStore {
        me: OffchainPublicKey,
        get_peers: Arc<RwLock<VecDeque<Vec<OffchainPublicKey>>>>,
        #[allow(clippy::type_complexity)]
        on_finished: Arc<RwLock<Vec<(OffchainPublicKey, crate::errors::Result<Duration>)>>>,
    }

    impl NetworkGraphUpdate for PeerStore {
        fn record_edge<N, P>(&self, telemetry: MeasurableEdge<N, P>)
        where
            N: hopr_api::graph::MeasurablePeer + Send + Clone,
            P: hopr_api::graph::MeasurablePath + Send + Clone,
        {
            let mut on_finished = self.on_finished.write().unwrap();

            match telemetry {
                hopr_api::graph::MeasurableEdge::Probe(Ok(EdgeTransportTelemetry::Neighbor(neighbor_telemetry))) => {
                    let peer: OffchainPublicKey = *neighbor_telemetry.peer();
                    let duration = neighbor_telemetry.rtt();
                    on_finished.push((peer, Ok(duration)));
                }
                hopr_api::graph::MeasurableEdge::Probe(Err(NetworkGraphError::ProbeNeighborTimeout(peer))) => {
                    on_finished.push((
                        *peer.as_ref(),
                        Err(ProbeError::TrafficError(NetworkGraphError::ProbeNeighborTimeout(peer))),
                    ));
                }
                _ => panic!("unexpected telemetry type, unimplemented"),
            }
        }

        fn record_node<N>(&self, _node: N)
        where
            N: hopr_api::graph::MeasurableNode + Send + Clone,
        {
            unimplemented!()
        }
    }

    #[async_trait]
    impl NetworkGraphView for PeerStore {
        type NodeId = OffchainPublicKey;
        type Observed = TestEdgeObservations;

        fn identity(&self) -> &OffchainPublicKey {
            &self.me
        }

        fn node_count(&self) -> usize {
            self.get_peers.read().unwrap().front().map_or(0, |v| v.len())
        }

        fn contains_node(&self, _key: &OffchainPublicKey) -> bool {
            false
        }

        /// Returns a stream of all known nodes in the network graph.
        fn nodes(&self) -> futures::stream::BoxStream<'static, OffchainPublicKey> {
            let mut get_peers = self.get_peers.write().unwrap();
            Box::pin(futures::stream::iter(get_peers.pop_front().unwrap_or_default()))
        }

        fn edge(&self, _src: &OffchainPublicKey, _dest: &OffchainPublicKey) -> Option<TestEdgeObservations> {
            Some(TestEdgeObservations)
        }
    }

    type TestClassifier = ProbeClassifierState<PeerStore>;

    struct TestInterface {
        probe_classifier: TestClassifier,
        from_probing_to_network_rx: futures::channel::mpsc::Receiver<(DestinationRouting, ApplicationDataOut)>,
        from_probing_to_network_tx: futures::channel::mpsc::Sender<(DestinationRouting, ApplicationDataOut)>,
        manual_probe_tx: futures::channel::mpsc::Sender<(OffchainPublicKey, PingQueryReplier)>,
    }

    async fn test_with_probing<F, Fut>(cfg: ProbeConfig, store: PeerStore, test: F) -> anyhow::Result<()>
    where
        Fut: std::future::Future<Output = anyhow::Result<()>>,
        F: Fn(TestInterface) -> Fut + Send + Sync + 'static,
    {
        let tag_allocators = hopr_transport_tag_allocator::create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (hopr_transport_tag_allocator::Usage::Session, 2048),
                (hopr_transport_tag_allocator::Usage::SessionTerminalTelemetry, 4000),
                (hopr_transport_tag_allocator::Usage::ProvingTelemetry, 10000),
            ],
        )
        .expect("tag allocators should be created");
        let probing_allocator = tag_allocators
            .into_iter()
            .find_map(|(u, alloc)| matches!(u, hopr_transport_tag_allocator::Usage::ProvingTelemetry).then_some(alloc))
            .expect("probing allocator should exist");

        let probe = Probe::new(cfg, probing_allocator);

        let (from_probing_to_network_tx, from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(DestinationRouting, ApplicationDataOut)>(100);

        let (manual_probe_tx, manual_probe_rx) =
            futures::channel::mpsc::channel::<(OffchainPublicKey, PingQueryReplier)>(100);

        let (jhs, probe_classifier) = probe
            .continuously_scan(
                from_probing_to_network_tx.clone(),
                manual_probe_rx,
                TestProbeStrategy::ImmediateNeighbor { store: store.clone() },
                store,
            )
            .await;

        let interface = TestInterface {
            probe_classifier,
            from_probing_to_network_rx,
            from_probing_to_network_tx,
            manual_probe_tx,
        };

        let result = test(interface).await;

        jhs.abort_all();

        result
    }

    const NO_PROBE_PASSES: f64 = 0.0;
    const ALL_PROBES_PASS: f64 = 1.0;

    /// Simulates the network: receives outgoing probe packets and feeds pong responses back
    /// through the classifier (mirroring what the remote peer + packet pipeline would do).
    fn concurrent_classify(
        delay: Option<std::time::Duration>,
        pass_rate: f64,
        classifier: TestClassifier,
        push_to_network: futures::channel::mpsc::Sender<(DestinationRouting, ApplicationDataOut)>,
    ) -> impl Fn((DestinationRouting, ApplicationDataOut)) -> BoxFuture<'static, ()> {
        debug_assert!(
            (NO_PROBE_PASSES..=ALL_PROBES_PASS).contains(&pass_rate),
            "Pass rate must be between {NO_PROBE_PASSES} and {ALL_PROBES_PASS}"
        );

        move |(path, data_out): (DestinationRouting, ApplicationDataOut)| -> BoxFuture<'static, ()> {
            let classifier = classifier.clone();
            let push_to_network = push_to_network.clone();

            Box::pin(async move {
                if let DestinationRouting::Forward { pseudonym, .. } = path {
                    let message: Message = data_out.data.try_into().expect("failed to convert data into message");
                    if let Message::Probe(NeighborProbe::Ping(ping)) = message {
                        let pong_message = Message::Probe(NeighborProbe::Pong(ping));

                        if let Some(delay) = delay {
                            tokio::time::sleep(delay).await;
                        }

                        if rand::random_range(NO_PROBE_PASSES..=ALL_PROBES_PASS) < pass_rate {
                            let pseudonym = pseudonym.expect("the pseudonym is always known from cache");
                            classifier
                                .classify(
                                    push_to_network,
                                    pseudonym,
                                    ApplicationDataIn {
                                        data: pong_message
                                            .try_into()
                                            .expect("failed to convert pong message into data"),
                                        packet_info: Default::default(),
                                    },
                                )
                                .await;
                        }
                    }
                };
            })
        }
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_value_for_manual_neighbor_probe() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(5),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore {
            me: *OFFCHAIN_KEYPAIR.public(),
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let mut manual_probe_tx = iface.manual_probe_tx;
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(None, ALL_PROBES_PASS, probe_classifier, from_probing_to_network_tx),
                    )
                    .await;
            });

            let _duration = tokio::time::timeout(std::time::Duration::from_secs(1), rx.next())
                .await?
                .ok_or_else(|| anyhow::anyhow!("Probe did not return a result in time"))?
                .map_err(|_| anyhow::anyhow!("Probe failed"))?;

            Ok(())
        })
        .await
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_failure_on_manual_fail() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(5),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore {
            me: *OFFCHAIN_KEYPAIR.public(),
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let mut manual_probe_tx = iface.manual_probe_tx;
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(None, NO_PROBE_PASSES, probe_classifier, from_probing_to_network_tx),
                    )
                    .await;
            });

            assert!(tokio::time::timeout(cfg.timeout * 2, rx.next()).await.is_err());

            Ok(())
        })
        .await
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_results_of_successful_automatically_generated_probes() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(20),
            max_parallel_probes: NEIGHBOURS.len(),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore {
            me: *OFFCHAIN_KEYPAIR.public(),
            get_peers: Arc::new(RwLock::new({
                let mut neighbors = VecDeque::new();
                neighbors.push_back(NEIGHBOURS.clone());
                neighbors
            })),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(None, ALL_PROBES_PASS, probe_classifier, from_probing_to_network_tx),
                    )
                    .await;
            });

            // Wait for the full probe lifecycle: emit → network round trip → process.
            // Must exceed cfg.timeout (the cache TTL) to avoid probes being evicted
            // as timeouts before the pong response is processed.
            tokio::time::sleep(cfg.timeout * 3).await;

            Ok(())
        })
        .await?;

        assert_eq!(
            store
                .on_finished
                .read()
                .expect("should be lockable")
                .iter()
                .filter(|(_peer, result)| result.is_ok())
                .count(),
            NEIGHBOURS.len()
        );

        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_results_of_timed_out_automatically_generated_probes() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(10),
            max_parallel_probes: NEIGHBOURS.len(),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore {
            me: *OFFCHAIN_KEYPAIR.public(),
            get_peers: Arc::new(RwLock::new({
                let mut neighbors = VecDeque::new();
                neighbors.push_back(NEIGHBOURS.clone());
                neighbors
            })),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        let timeout = cfg.timeout * 2;

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(
                            Some(timeout),
                            ALL_PROBES_PASS,
                            probe_classifier,
                            from_probing_to_network_tx,
                        ),
                    )
                    .await;
            });

            // wait for the probes to start and finish
            tokio::time::sleep(timeout * 2).await;

            Ok(())
        })
        .await?;

        assert_eq!(
            store
                .on_finished
                .read()
                .expect("should be lockable")
                .iter()
                .filter(|(_peer, result)| result.is_err())
                .count(),
            NEIGHBOURS.len()
        );

        Ok(())
    }

    #[tokio::test]
    async fn probe_should_reply_with_pong_when_receiving_ping() -> anyhow::Result<()> {
        use anyhow::Context;

        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(10),
            ..Default::default()
        };

        let store = PeerStore {
            me: *OFFCHAIN_KEYPAIR.public(),
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let probe_classifier = iface.probe_classifier;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let mut from_probing_to_network_rx = iface.from_probing_to_network_rx;

            // Build a Ping message with the reserved Ping tag
            let ping = NeighborProbe::random_nonce();
            let ping_nonce = match ping {
                NeighborProbe::Ping(n) => n,
                _ => unreachable!(),
            };
            let ping_msg = Message::Probe(ping);
            let app_data: ApplicationData = ping_msg.try_into().context("converting ping to ApplicationData")?;

            // Classify the ping directly — the classifier sends the pong reply to push_to_network
            let result = probe_classifier
                .classify(
                    from_probing_to_network_tx,
                    HoprPseudonym::random(),
                    ApplicationDataIn {
                        data: app_data,
                        packet_info: Default::default(),
                    },
                )
                .await;
            anyhow::ensure!(matches!(result, ProbeDispatch::Consumed), "ping should be consumed");

            // The probe should reply with a Pong on the network channel
            let (routing, data_out) = tokio::time::timeout(Duration::from_secs(2), from_probing_to_network_rx.next())
                .await
                .context("timeout waiting for pong")?
                .context("probe should send pong reply")?;

            // Verify it's a Return routing (SURB-based reply)
            anyhow::ensure!(
                matches!(routing, DestinationRouting::Return(_)),
                "pong should use Return routing, got: {routing:?}"
            );

            // Verify the payload is a Pong with the same nonce
            let response_msg: Message = data_out.data.try_into().context("converting response to Message")?;
            anyhow::ensure!(
                matches!(response_msg, Message::Probe(NeighborProbe::Pong(n)) if n == ping_nonce),
                "response should be Pong with matching nonce"
            );

            Ok(())
        })
        .await
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_pass_through_non_associated_tags() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(20),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore {
            me: *OFFCHAIN_KEYPAIR.public(),
            get_peers: Arc::new(RwLock::new({
                let mut neighbors = VecDeque::new();
                neighbors.push_back(NEIGHBOURS.clone());
                neighbors
            })),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let probe_classifier = iface.probe_classifier;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;

            let expected_data = ApplicationData::new(Tag::MAX, b"Hello, this is a test message!")?;

            let result = probe_classifier
                .classify(
                    from_probing_to_network_tx,
                    HoprPseudonym::random(),
                    ApplicationDataIn {
                        data: expected_data.clone(),
                        packet_info: Default::default(),
                    },
                )
                .await;

            match result {
                ProbeDispatch::Passthrough(_, actual) => assert_eq!(actual.data, expected_data),
                ProbeDispatch::Consumed => anyhow::bail!("expected Passthrough, got Consumed"),
            }

            Ok(())
        })
        .await
    }

    /// How a probe gets triggered inside the test: either via the manual ping channel
    /// (for neighbor probes) or by injecting a single [`ProbeRouting::Looping`] into the
    /// probing traffic generator (for loopback probes).
    #[derive(Clone)]
    enum TestProbeStrategy {
        ManualNeighbor,
        ImmediateNeighbor {
            store: PeerStore,
        },
        OneShotLoopback {
            routing: DestinationRouting,
            path_id: hopr_api::types::internal::routing::PathId,
        },
    }

    impl hopr_api::ct::ProbingTrafficGeneration for TestProbeStrategy {
        fn build(&self) -> futures::stream::BoxStream<'static, hopr_api::ct::ProbeRouting> {
            match self {
                Self::ManualNeighbor => Box::pin(futures::stream::pending()),
                Self::ImmediateNeighbor { store } => {
                    let peers: Vec<OffchainPublicKey> =
                        store.get_peers.write().unwrap().pop_front().unwrap_or_default();
                    Box::pin(futures::StreamExt::chain(
                        futures::stream::iter(peers.into_iter().map(|peer| {
                            ProbeRouting::Neighbor(DestinationRouting::Forward {
                                destination: Box::new(peer.into()),
                                pseudonym: Some(HoprPseudonym::random()),
                                forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                                return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                            })
                        })),
                        futures::stream::pending(),
                    ))
                }
                Self::OneShotLoopback { routing, path_id } => {
                    let probe = hopr_api::ct::ProbeRouting::Looping((routing.clone(), *path_id));
                    Box::pin(futures::StreamExt::chain(
                        futures::stream::iter(std::iter::once(probe)),
                        futures::stream::pending(),
                    ))
                }
            }
        }
    }

    /// Regression test for hoprnet/hoprnet#7972: each probe type must be emitted with the
    /// exact SURB count it actually consumes — no more, no less — so the path planner does
    /// not resolve (and the SURB store does not accumulate) unused return paths.
    ///
    /// - Neighbor probe: 1 SURB (zero-hop pong return).
    /// - Loopback probe: 0 SURBs (self-routed, never replied to).
    #[rstest::rstest]
    #[case::neighbor_probe_requests_one_surb(TestProbeStrategy::ManualNeighbor, 1)]
    #[case::loopback_probe_requests_zero_surbs(
        TestProbeStrategy::OneShotLoopback {
            routing: DestinationRouting::Forward {
                destination: Box::new((*OFFCHAIN_KEYPAIR.public()).into()),
                pseudonym: Some(HoprPseudonym::random()),
                forward_options: RoutingOptions::Hops(1.try_into().expect("1 is a valid u8")),
                return_options: None,
            },
            path_id: [1, 2, 3, 4, 5],
        },
        0,
    )]
    #[tokio::test]
    async fn probe_should_emit_with_expected_surb_count(
        #[case] strategy: TestProbeStrategy,
        #[case] expected_max_surbs: usize,
    ) -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_secs(1),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        // Wire up a full probing harness with the parameterized strategy injected as the
        // traffic generator. Mirrors `test_with_probing` but allows arbitrary `ProbingTrafficGeneration`.
        let tag_allocators = hopr_transport_tag_allocator::create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (hopr_transport_tag_allocator::Usage::Session, 2048),
                (hopr_transport_tag_allocator::Usage::SessionTerminalTelemetry, 4000),
                (hopr_transport_tag_allocator::Usage::ProvingTelemetry, 10000),
            ],
        )
        .expect("tag allocators should be created");
        let probing_allocator = tag_allocators
            .into_iter()
            .find_map(|(u, alloc)| matches!(u, hopr_transport_tag_allocator::Usage::ProvingTelemetry).then_some(alloc))
            .expect("probing allocator should exist");

        let probe = Probe::new(cfg, probing_allocator);

        let (from_probing_to_network_tx, mut from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(DestinationRouting, ApplicationDataOut)>(100);
        let (mut manual_probe_tx, manual_probe_rx) =
            futures::channel::mpsc::channel::<(OffchainPublicKey, PingQueryReplier)>(100);

        let store = PeerStore {
            me: *OFFCHAIN_KEYPAIR.public(),
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        // Kick off the probing process before triggering — the strategy's stream drives
        // the loopback case, and we push to `manual_probe_tx` below for the neighbor case.
        let is_manual = matches!(strategy, TestProbeStrategy::ManualNeighbor);
        let (jhs, _probe_classifier) = probe
            .continuously_scan(from_probing_to_network_tx, manual_probe_rx, strategy, store)
            .await;

        if is_manual {
            let (tx, _rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;
        }

        let (_routing, data_out) =
            tokio::time::timeout(std::time::Duration::from_secs(1), from_probing_to_network_rx.next())
                .await?
                .ok_or_else(|| anyhow::anyhow!("no probe emitted"))?;

        jhs.abort_all();

        let packet_info = data_out
            .packet_info
            .ok_or_else(|| anyhow::anyhow!("probe must carry explicit OutgoingPacketInfo"))?;
        assert_eq!(
            packet_info.max_surbs_in_packet, expected_max_surbs,
            "probe must request exactly {expected_max_surbs} SURB(s)"
        );

        Ok(())
    }
}
