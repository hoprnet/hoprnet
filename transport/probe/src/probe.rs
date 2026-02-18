use std::sync::Arc;

use futures::{FutureExt, SinkExt, StreamExt, pin_mut};
use futures_concurrency::stream::StreamExt as _;
use hopr_api::{
    ct::{ProbeRouting, ProbingTrafficGeneration},
    graph::{EdgeTransportTelemetry, NetworkGraphError, NetworkGraphUpdate, NetworkGraphView},
};
use hopr_async_runtime::AbortableList;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use hopr_platform::time::native::current_time;
use hopr_primitive_types::traits::AsUnixTimestamp;
use hopr_protocol_app::{
    prelude::{ApplicationDataIn, ApplicationDataOut, ReservedTag},
    v1::Tag,
};

use crate::{
    HoprProbeProcess,
    config::ProbeConfig,
    content::Message,
    ping::PingQueryReplier,
    types::{NeighborProbe, NeighborTelemetry, PathTelemetry},
};

type CacheNeighborKey = (HoprPseudonym, NeighborProbe);
type CacheNeighborValue = (Box<NodeId>, std::time::Duration, Option<PingQueryReplier>);

/// Probe functionality builder.
///
/// The builder holds information about this node's own addresses and the configuration for the probing process. It is
/// then used to construct the probing process itself.
pub struct Probe {
    /// Probe configuration.
    cfg: ProbeConfig,
}

impl Probe {
    pub fn new(cfg: ProbeConfig) -> Self {
        Self { cfg }
    }

    /// The main function that assembles and starts the probing process.
    pub async fn continuously_scan<T, U, V, Up, Tr, G>(
        self,
        api: (T, U),      // lower (tx, rx) channels for sending and receiving messages
        manual_events: V, // explicit requests from the API
        move_up: Up,      // forward up non-probing messages from the network
        probing_traffic_generator: Tr,
        network_graph: G,
    ) -> AbortableList<HoprProbeProcess>
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + 'static,
        T::Error: Send,
        U: futures::Stream<Item = (HoprPseudonym, ApplicationDataIn)> + Send + Sync + 'static,
        V: futures::Stream<Item = (OffchainPublicKey, PingQueryReplier)> + Send + Sync + 'static,
        Up: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Clone + Send + Sync + 'static,
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

        let active_path_probes: moka::future::Cache<Tag, PathTelemetry> = moka::future::Cache::builder()
            .time_to_live(timeout)
            .max_capacity(100_000)
            .async_eviction_listener(
                move |tag: Arc<Tag>, path: PathTelemetry, cause| -> moka::notification::ListenerFuture {
                    if matches!(cause, moka::notification::RemovalCause::Expired) {
                        // If the eviction cause is expiration => record as a failed probe
                        let store = network_graph_internal_path.clone();

                        tracing::debug!(%tag, reason = "timeout", "loopback probe failed");

                        store.record_edge::<NeighborTelemetry, PathTelemetry>(hopr_api::graph::MeasurableEdge::Probe(
                            Err(NetworkGraphError::ProbeLoopbackTimeout(path)),
                        ));
                        futures::FutureExt::boxed(futures::future::ready(()))
                    } else {
                        // If the eviction cause is not expiration, nothing needs to be done
                        futures::FutureExt::boxed(futures::future::ready(()))
                    }
                },
            )
            .build();

        let active_probes_rx = active_neighbor_probes.clone();
        let push_to_network = api.0.clone();

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

        let minimum_allowed_tag = ReservedTag::range().end;
        processes.insert(
            HoprProbeProcess::Emit,
            hopr_async_runtime::spawn_as_abortable!(async move {
                direct_neighbors
                    .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
                        let active_neighbor_probes = active_neighbor_probes.clone();
                        let active_path_probes = active_path_probes.clone();
                        let push_to_network = push_to_network.clone();

                        async move {
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
                                        let data = ApplicationDataOut::with_no_packet_info(data);
                                        pin_mut!(push_to_network);

                                        if let Err(_error) = push_to_network.send((routing, data)).await {
                                            tracing::error!("failed to send out a ping");
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
                                        tracing::error!("failed to convert ping message into data");
                                    }
                                }
                                ProbeRouting::Neighbor(DestinationRouting::Return(_surb_matcher)) => tracing::error!(
                                    error = "logical error",
                                    "resolved transport routing is not forward"
                                ),
                                ProbeRouting::Looping((routing, path_id)) => {
                                    let message = Message::Telemetry(PathTelemetry {
                                        id: hopr_crypto_random::random_bytes(),
                                        path: std::array::from_fn(|i| path_id[i / 8].to_le_bytes()[i % 8]),
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis(),
                                    });

                                    let random_tag: u64 = hopr_crypto_random::random_integer(minimum_allowed_tag, None);

                                    if let Ok(packet) = hopr_protocol_app::prelude::ApplicationData::new(
                                        random_tag,
                                        message.to_bytes().as_ref(),
                                    ) {
                                        pin_mut!(push_to_network);

                                        if let Err(_error) = push_to_network
                                            .send((routing, ApplicationDataOut::with_no_packet_info(packet)))
                                            .await
                                        {
                                            tracing::error!("failed to send out a ping");
                                        } else {
                                            // the object is constructed above, so will always match
                                            if let Message::Telemetry(telemetry) = message {
                                                active_path_probes.insert(random_tag.into(), telemetry).await;
                                            }
                                        }
                                    } else {
                                        tracing::error!("failed to construct data for path telemetry")
                                    }
                                }
                            }
                        }
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

        // -- Process probes --
        processes.insert(
            HoprProbeProcess::Process,
            hopr_async_runtime::spawn_as_abortable!(api.1.for_each_concurrent(max_parallel_probes, move |(pseudonym, in_data)| {
                let active_probes = active_probes_rx.clone();
                let push_to_network = api.0.clone();
                let move_up = move_up.clone();
                let store = network_graph.clone();

                async move {
                    if in_data.data.application_tag == ReservedTag::Ping.into() {
                        let message: anyhow::Result<Message> = in_data.data.try_into().map_err(|e| anyhow::anyhow!("failed to convert data into message: {e}"));

                        match message {
                            Ok(message) => {
                                match message {
                                    Message::Telemetry(path_telemetry) => {
                                        store.record_edge::<NeighborTelemetry, PathTelemetry>(hopr_api::graph::MeasurableEdge::Probe(Ok(EdgeTransportTelemetry::Loopback(path_telemetry))))
                                    },
                                    Message::Probe(NeighborProbe::Ping(ping)) => {
                                        tracing::debug!(%pseudonym, nonce = hex::encode(ping), "received ping");
                                        tracing::trace!(%pseudonym, nonce = hex::encode(ping), "wrapping a pong in the found SURB");

                                        let message = Message::Probe(NeighborProbe::Pong(ping));
                                        if let Ok(data) = message.try_into() {
                                            let routing = DestinationRouting::Return(pseudonym.into());
                                            let data = ApplicationDataOut::with_no_packet_info(data);
                                            pin_mut!(push_to_network);

                                            if let Err(_error) = push_to_network.send((routing, data)).await {
                                                tracing::error!(%pseudonym, "failed to send back a pong");
                                            }
                                        } else {
                                            tracing::error!(%pseudonym, "failed to convert pong message into data");
                                        }
                                    },
                                    Message::Probe(NeighborProbe::Pong(pong)) => {
                                        tracing::debug!(%pseudonym, nonce = hex::encode(pong), "received pong");
                                        if let Some((peer, start, replier)) = active_probes.remove(&(pseudonym, NeighborProbe::Ping(pong))).await {
                                            let latency = current_time()
                                                .as_unix_timestamp()
                                                .saturating_sub(start);

                                            if let NodeId::Offchain(opk) = peer.as_ref() {
                                                tracing::debug!(%pseudonym, nonce = hex::encode(pong), latency_ms = latency.as_millis(), "probe successful");
                                                store.record_edge::<NeighborTelemetry, PathTelemetry>(hopr_api::graph::MeasurableEdge::Probe(Ok(EdgeTransportTelemetry::Neighbor(NeighborTelemetry {
                                                    peer: *opk,
                                                    rtt: latency,
                                                }))))
                                            } else {
                                                tracing::warn!(%pseudonym, nonce = hex::encode(pong), latency_ms = latency.as_millis(), "probe successful to non-offchain peer");
                                            }

                                            if let Some(replier) = replier {
                                                replier.notify(Ok(latency))
                                            };
                                        } else {
                                            tracing::warn!(%pseudonym, nonce = hex::encode(pong), possible_reasons = "[timeout, adversary]", "received pong for unknown probe");
                                        };
                                    },
                                }
                            },
                            Err(error) => tracing::error!(%pseudonym, %error, "cannot deserialize message"),
                        }
                    } else {
                        // If the message is not a probing message, forward it up
                        pin_mut!(move_up);
                        if move_up.send((pseudonym, in_data)).await.is_err() {
                            tracing::error!(%pseudonym, error = "receiver error", "failed to send message up");
                        }
                    }
                }
            }).inspect(|_| tracing::warn!(task = "transport (probe - processing incoming)", "long-running background task finished")))
        );

        processes
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::RwLock, time::Duration};

    use async_trait::async_trait;
    use futures::future::BoxFuture;
    use hopr_api::graph::{
        EdgeLinkObservable, MeasurableEdge, NetworkGraphError,
        traits::{EdgeNetworkObservableRead, EdgeObservableRead, EdgeObservableWrite, EdgeProtocolObservable},
    };
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_ct_telemetry::{ImmediateNeighborProber, ProberConfig};
    use hopr_protocol_app::prelude::{ApplicationData, Tag};

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
                    let peer: OffchainPublicKey = neighbor_telemetry.peer().clone();
                    let duration = neighbor_telemetry.rtt();
                    on_finished.push((peer, Ok(duration)));
                }
                hopr_api::graph::MeasurableEdge::Probe(Err(NetworkGraphError::ProbeNeighborTimeout(peer))) => {
                    on_finished.push((
                        peer.as_ref().clone(),
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

    struct TestInterface {
        from_probing_up_rx: futures::channel::mpsc::Receiver<(HoprPseudonym, ApplicationDataIn)>,
        from_probing_to_network_rx: futures::channel::mpsc::Receiver<(DestinationRouting, ApplicationDataOut)>,
        from_network_to_probing_tx: futures::channel::mpsc::Sender<(HoprPseudonym, ApplicationDataIn)>,
        manual_probe_tx: futures::channel::mpsc::Sender<(OffchainPublicKey, PingQueryReplier)>,
    }

    async fn test_with_probing<F, St, Fut>(cfg: ProbeConfig, store: St, test: F) -> anyhow::Result<()>
    where
        Fut: std::future::Future<Output = anyhow::Result<()>>,
        F: Fn(TestInterface) -> Fut + Send + Sync + 'static,
        St: NetworkGraphUpdate + NetworkGraphView<NodeId = OffchainPublicKey> + Clone + Send + Sync + 'static,
    {
        let probe = Probe::new(cfg);

        let (from_probing_up_tx, from_probing_up_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationDataIn)>(100);

        let (from_probing_to_network_tx, from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(DestinationRouting, ApplicationDataOut)>(100);

        let (from_network_to_probing_tx, from_network_to_probing_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationDataIn)>(100);

        let (manual_probe_tx, manual_probe_rx) =
            futures::channel::mpsc::channel::<(OffchainPublicKey, PingQueryReplier)>(100);

        let interface = TestInterface {
            from_probing_up_rx,
            from_probing_to_network_rx,
            from_network_to_probing_tx,
            manual_probe_tx,
        };

        let jhs = probe
            .continuously_scan(
                (from_probing_to_network_tx, from_network_to_probing_rx),
                manual_probe_rx,
                from_probing_up_tx,
                ImmediateNeighborProber::new(
                    ProberConfig {
                        interval: cfg.interval,
                        recheck_threshold: cfg.recheck_threshold,
                    },
                    store.clone(),
                ),
                store,
            )
            .await;

        let result = test(interface).await;

        jhs.abort_all();

        result
    }

    const NO_PROBE_PASSES: f64 = 0.0;
    const ALL_PROBES_PASS: f64 = 1.0;

    /// Channel that can drop any probes and concurrently replies to a probe correctly
    fn concurrent_channel(
        delay: Option<std::time::Duration>,
        pass_rate: f64,
        from_network_to_probing_tx: futures::channel::mpsc::Sender<(HoprPseudonym, ApplicationDataIn)>,
    ) -> impl Fn((DestinationRouting, ApplicationDataOut)) -> BoxFuture<'static, ()> {
        debug_assert!(
            (NO_PROBE_PASSES..=ALL_PROBES_PASS).contains(&pass_rate),
            "Pass rate must be between {NO_PROBE_PASSES} and {ALL_PROBES_PASS}"
        );

        move |(path, data_out): (DestinationRouting, ApplicationDataOut)| -> BoxFuture<'static, ()> {
            let mut from_network_to_probing_tx = from_network_to_probing_tx.clone();

            Box::pin(async move {
                if let DestinationRouting::Forward { pseudonym, .. } = path {
                    let message: Message = data_out.data.try_into().expect("failed to convert data into message");
                    if let Message::Probe(NeighborProbe::Ping(ping)) = message {
                        let pong_message = Message::Probe(NeighborProbe::Pong(ping));

                        if let Some(delay) = delay {
                            // Simulate a delay if specified
                            tokio::time::sleep(delay).await;
                        }

                        if rand::random_range(NO_PROBE_PASSES..=ALL_PROBES_PASS) < pass_rate {
                            from_network_to_probing_tx
                                .send((
                                    pseudonym.expect("the pseudonym is always known from cache"),
                                    ApplicationDataIn {
                                        data: pong_message
                                            .try_into()
                                            .expect("failed to convert pong message into data"),
                                        packet_info: Default::default(),
                                    },
                                ))
                                .await
                                .expect("failed to send pong message");
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
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let mut manual_probe_tx = iface.manual_probe_tx;
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_network_to_probing_tx = iface.from_network_to_probing_tx;

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

            let _jh: hopr_async_runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_channel(None, ALL_PROBES_PASS, from_network_to_probing_tx),
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
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let mut manual_probe_tx = iface.manual_probe_tx;
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_network_to_probing_tx = iface.from_network_to_probing_tx;

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

            let _jh: hopr_async_runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_channel(None, NO_PROBE_PASSES, from_network_to_probing_tx),
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
            get_peers: Arc::new(RwLock::new({
                let mut neighbors = VecDeque::new();
                neighbors.push_back(NEIGHBOURS.clone());
                neighbors
            })),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_network_to_probing_tx = iface.from_network_to_probing_tx;

            let _jh: hopr_async_runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_channel(None, ALL_PROBES_PASS, from_network_to_probing_tx),
                    )
                    .await;
            });

            // wait for the probes to start and finish
            tokio::time::sleep(cfg.timeout).await;

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
            let from_network_to_probing_tx = iface.from_network_to_probing_tx;

            let _jh: hopr_async_runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_channel(Some(timeout), ALL_PROBES_PASS, from_network_to_probing_tx),
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
    // #[tracing_test::traced_test]
    async fn probe_should_pass_through_non_associated_tags() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(20),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore {
            get_peers: Arc::new(RwLock::new({
                let mut neighbors = VecDeque::new();
                neighbors.push_back(NEIGHBOURS.clone());
                neighbors
            })),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let mut from_network_to_probing_tx = iface.from_network_to_probing_tx;
            let mut from_probing_up_rx = iface.from_probing_up_rx;

            let expected_data = ApplicationData::new(Tag::MAX, b"Hello, this is a test message!")?;

            from_network_to_probing_tx
                .send((
                    HoprPseudonym::random(),
                    ApplicationDataIn {
                        data: expected_data.clone(),
                        packet_info: Default::default(),
                    },
                ))
                .await?;

            let actual = tokio::time::timeout(cfg.timeout, from_probing_up_rx.next())
                .await?
                .ok_or_else(|| anyhow::anyhow!("Did not return any data in time"))?
                .1;

            assert_eq!(actual.data, expected_data);

            Ok(())
        })
        .await
    }
}
