use std::{collections::HashMap, sync::Arc};

use futures::{FutureExt, SinkExt, StreamExt, pin_mut};
use futures_concurrency::stream::StreamExt as _;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::protocol::HoprPseudonym;
use hopr_network_types::types::{DestinationRouting, RoutingOptions};
use hopr_platform::time::native::current_time;
use hopr_primitive_types::{prelude::Address, traits::AsUnixTimestamp};
use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut, ReservedTag};
use libp2p_identity::PeerId;

use crate::{
    HoprProbeProcess,
    config::ProbeConfig,
    content::{Message, NeighborProbe},
    errors::ProbeError,
    neighbors::neighbors_to_probe,
    ping::PingQueryReplier,
    traits::{PeerDiscoveryFetch, ProbeStatusUpdate},
};

#[inline(always)]
fn to_nonce(message: &Message) -> String {
    match message {
        Message::Probe(NeighborProbe::Ping(ping)) => hex::encode(ping),
        Message::Probe(NeighborProbe::Pong(pong)) => hex::encode(pong),
        _ => "<telemetry>".to_string(),
    }
}

#[inline(always)]
fn to_pseudonym(path: &DestinationRouting) -> Option<HoprPseudonym> {
    match path {
        DestinationRouting::Forward { pseudonym, .. } => *pseudonym,
        DestinationRouting::Return(matcher) => match matcher {
            hopr_network_types::types::SurbMatcher::Exact(sender_id) => Some(sender_id.pseudonym()),
            hopr_network_types::types::SurbMatcher::Pseudonym(pseudonym) => Some(*pseudonym),
        },
    }
}

#[derive(Clone, Debug)]
struct Sender<T> {
    downstream: T,
}

impl<T> Sender<T>
where
    T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + 'static,
{
    #[tracing::instrument(level = "debug", skip(self, path, message), fields(message=%message, nonce=%to_nonce(&message), pseudonym=?to_pseudonym(&path)), ret(level = tracing::Level::TRACE), err(Display))]
    async fn send_message(self, path: DestinationRouting, message: Message) -> crate::errors::Result<()> {
        let push_to_network = self.downstream;
        pin_mut!(push_to_network);
        if push_to_network
            .as_mut()
            .send((path, ApplicationDataOut::with_no_packet_info(message.try_into()?)))
            .await
            .is_ok()
        {
            Ok(())
        } else {
            Err(ProbeError::SendError("transport error".to_string()))
        }
    }
}

/// Probe functionality builder.
///
/// The builder holds information about this node's own addresses and the configuration for the probing process. It is
/// then used to construct the probing process itself.
pub struct Probe {
    /// Own addresses for self reference and surb creation.
    me: (OffchainPublicKey, Address),
    /// Probe configuration.
    cfg: ProbeConfig,
}

impl Probe {
    pub fn new(me: (OffchainPublicKey, Address), cfg: ProbeConfig) -> Self {
        Self { me, cfg }
    }

    /// The main function that assembles and starts the probing process.
    pub async fn continuously_scan<T, U, V, W, Up>(
        self,
        api: (T, U),      // lower (tx, rx) channels for sending and receiving messages
        manual_events: V, // explicit requests from the API
        store: W,         // peer store
        move_up: Up,      // forward up non-probing messages from the network
    ) -> HashMap<HoprProbeProcess, hopr_async_runtime::AbortHandle>
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + 'static,
        T::Error: Send,
        U: futures::Stream<Item = (HoprPseudonym, ApplicationDataIn)> + Send + Sync + 'static,
        W: PeerDiscoveryFetch + ProbeStatusUpdate + Clone + Send + Sync + 'static,
        V: futures::Stream<Item = (PeerId, PingQueryReplier)> + Send + Sync + 'static,
        Up: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Clone + Send + Sync + 'static,
    {
        let max_parallel_probes = self.cfg.max_parallel_probes;
        let interval_between_rounds = self.cfg.interval;

        // Currently active probes
        let store_eviction = store.clone();
        let timeout = self.cfg.timeout;
        let active_probes: moka::future::Cache<
            (HoprPseudonym, NeighborProbe),
            (PeerId, std::time::Duration, Option<PingQueryReplier>),
        > = moka::future::Cache::builder()
            .time_to_live(timeout)
            .max_capacity(100_000)
            .async_eviction_listener(
                move |k: Arc<(HoprPseudonym, NeighborProbe)>,
                      v: (PeerId, std::time::Duration, Option<PingQueryReplier>),
                      cause|
                      -> moka::notification::ListenerFuture {
                    if matches!(cause, moka::notification::RemovalCause::Expired) {
                        // If the eviction cause is expiration => record as a failed probe
                        let store = store_eviction.clone();
                        let (peer, _start, notifier) = v;

                        tracing::debug!(%peer, pseudonym = %k.0, probe = %k.1, reason = "timeout", "probe failed");
                        if let Some(replier) = notifier {
                            replier.notify(Err(ProbeError::Timeout(timeout.as_secs())));
                        };
                        
                        futures::FutureExt::boxed(async move {
                            store
                                .on_finished(&peer, &Err(ProbeError::Timeout(timeout.as_secs())))
                                .await;
                        })
                    } else {
                        // If the eviction cause is not expiration, nothing needs to be done
                        futures::FutureExt::boxed(futures::future::ready(()))
                    }
                },
            )
            .build();

        let active_probes_rx = active_probes.clone();
        let push_to_network = api.0.clone();

        let mut processes = HashMap::<HoprProbeProcess, hopr_async_runtime::AbortHandle>::new();

        // -- Emit probes --
        let direct_neighbors = neighbors_to_probe(store.clone(), self.cfg)
            .map(|peer| (peer, None))
            .merge(manual_events.map(|(peer, notifier)| (peer, Some(notifier))));

        processes.insert(
            HoprProbeProcess::Emit,
            hopr_async_runtime::spawn_as_abortable!(async move {
                hopr_async_runtime::prelude::sleep(2 * interval_between_rounds).await;   // delay to allow network to stabilize

                direct_neighbors
                    .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
                        let cache_peer_routing = cache_peer_routing.clone();
                        let active_probes = active_probes.clone();
                        let push_to_network = Sender { downstream: push_to_network.clone() };
                        let me = self.me;

                        async move {
                            let result = cache_peer_routing
                                .try_get_with(peer, async move {
                                    Ok::<DestinationRouting, anyhow::Error>(DestinationRouting::Forward {
                                        destination: me.1,
                                        pseudonym: Some(HoprPseudonym::random()),
                                        forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                                        return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                                    })
                                })
                                .await
                                .map(|path| async move {
                                    match path {
                                        DestinationRouting::Forward { destination, pseudonym, forward_options, return_options } => {
                                            let nonce = NeighborProbe::random_nonce();

                                            let message = Message::Probe(nonce);

                                            if let Err(error) = push_to_network.send_message(DestinationRouting::Forward { destination, pseudonym, forward_options, return_options }, message).await {
                                                tracing::error!(%peer, %error, "failed to send out a probe");
                                            } else {
                                                active_probes
                                                    .insert((pseudonym.expect("the pseudonym is always known from cache"), nonce), (peer, current_time().as_unix_timestamp(), notifier))
                                                    .await;
                                            }
                                        },
                                        DestinationRouting::Return(_surb_matcher) => tracing::error!(%peer, error = "logical error", "resolved transport routing is not forward"),
                                    }
                                })
                                .inspect_err(|error| tracing::error!(%peer, %error, "failed to resolve transport routing"));
                            
                            if let Err(error) = result {
                                tracing::error!(%peer, %error, "failed to get or build transport routing");
                            }
                        }

                    })
                    .inspect(|_| tracing::warn!(task = "transport (probe - generate outgoing)", "long-running background task finished"))
                    .await;
            })
        );

        // -- Process probes --
        processes.insert(
            HoprProbeProcess::Process,
            hopr_async_runtime::spawn_as_abortable!(api.1.for_each_concurrent(max_parallel_probes, move |(pseudonym, in_data)| {
                let active_probes = active_probes_rx.clone();
                let push_to_network = Sender { downstream: api.0.clone() };
                let store = store.clone();
                let move_up = move_up.clone();

                async move {
                    // TODO(v3.1): compare not only against ping tag, but also against telemetry that will be occurring on random tags
                    if in_data.data.application_tag == ReservedTag::Ping.into() {
                        let message: anyhow::Result<Message> = in_data.data.try_into().map_err(|e| anyhow::anyhow!("failed to convert data into message: {e}"));

                        match message {
                            Ok(message) => {
                                match message {
                                    Message::Telemetry(_path_telemetry) => {
                                        tracing::warn!(%pseudonym, reason = "feature not implemented", "this node could not originate the telemetry");
                                    },
                                    Message::Probe(NeighborProbe::Ping(ping)) => {
                                        tracing::debug!(%pseudonym, nonce = hex::encode(ping), "received ping");
                                        tracing::trace!(%pseudonym, nonce = hex::encode(ping), "wrapping a pong in the found SURB");

                                        let message = Message::Probe(NeighborProbe::Pong(ping));
                                        if let Err(error) = push_to_network.send_message(DestinationRouting::Return(pseudonym.into()), message).await {
                                            tracing::error!(%pseudonym, %error, "failed to send back a pong");
                                        }
                                    },
                                    Message::Probe(NeighborProbe::Pong(pong)) => {
                                        tracing::debug!(%pseudonym, nonce = hex::encode(pong), "received pong");
                                        if let Some((peer, start, replier)) = active_probes.remove(&(pseudonym, NeighborProbe::Ping(pong))).await {
                                            let latency = current_time()
                                                .as_unix_timestamp()
                                                .saturating_sub(start);
                                            store.on_finished(&peer, &Ok(latency)).await;

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
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_protocol_app::prelude::{ApplicationData, Tag};

    use super::*;

    lazy_static::lazy_static!(
        static ref OFFCHAIN_KEYPAIR: OffchainKeypair = OffchainKeypair::random();
        static ref ONCHAIN_KEYPAIR: ChainKeypair = ChainKeypair::random();
        static ref NEIGHBOURS: Vec<PeerId> = vec![
            OffchainKeypair::random().public().into(),
            OffchainKeypair::random().public().into(),
            OffchainKeypair::random().public().into(),
            OffchainKeypair::random().public().into(),
        ];
    );

    #[derive(Debug, Clone)]
    pub struct PeerStore {
        get_peers: Arc<RwLock<VecDeque<Vec<PeerId>>>>,
        #[allow(clippy::type_complexity)]
        on_finished: Arc<RwLock<Vec<(PeerId, crate::errors::Result<Duration>)>>>,
    }

    #[async_trait]
    impl ProbeStatusUpdate for PeerStore {
        async fn on_finished(&self, peer: &PeerId, result: &crate::errors::Result<Duration>) {
            let mut on_finished = self.on_finished.write().unwrap();
            on_finished.push((
                *peer,
                match result {
                    Ok(duration) => Ok(*duration),
                    Err(_e) => Err(ProbeError::Timeout(1000)),
                },
            ));
        }
    }

    #[async_trait]
    impl PeerDiscoveryFetch for PeerStore {
        async fn get_peers(&self, _from_timestamp: std::time::SystemTime) -> Vec<PeerId> {
            let mut get_peers = self.get_peers.write().unwrap();
            get_peers.pop_front().unwrap_or_default()
        }
    }

    struct TestInterface {
        from_probing_up_rx: futures::channel::mpsc::Receiver<(HoprPseudonym, ApplicationDataIn)>,
        from_probing_to_network_rx: futures::channel::mpsc::Receiver<(DestinationRouting, ApplicationDataOut)>,
        from_network_to_probing_tx: futures::channel::mpsc::Sender<(HoprPseudonym, ApplicationDataIn)>,
        manual_probe_tx: futures::channel::mpsc::Sender<(PeerId, PingQueryReplier)>,
    }

    async fn test_with_probing<F, St, Fut>(cfg: ProbeConfig, store: St, test: F) -> anyhow::Result<()>
    where
        Fut: std::future::Future<Output = anyhow::Result<()>>,
        F: Fn(TestInterface) -> Fut + Send + Sync + 'static,
        St: ProbeStatusUpdate + PeerDiscoveryFetch + Clone + Send + Sync + 'static,
    {
        let probe = Probe::new((*OFFCHAIN_KEYPAIR.public(), ONCHAIN_KEYPAIR.public().to_address()), cfg);

        let (from_probing_up_tx, from_probing_up_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationDataIn)>(100);

        let (from_probing_to_network_tx, from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(DestinationRouting, ApplicationDataOut)>(100);

        let (from_network_to_probing_tx, from_network_to_probing_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationDataIn)>(100);

        let (manual_probe_tx, manual_probe_rx) = futures::channel::mpsc::channel::<(PeerId, PingQueryReplier)>(100);

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
                store,
                from_probing_up_tx,
            )
            .await;

        let result = test(interface).await;

        jhs.into_iter().for_each(|(_name, handle)| handle.abort());

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

                        if rand::Rng::gen_range(&mut rand::thread_rng(), NO_PROBE_PASSES..=ALL_PROBES_PASS) < pass_rate
                        {
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

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ProbeError>>(128);
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
                .ok_or_else(|| anyhow::anyhow!("Probe did not return a result in time"))??;

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

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ProbeError>>(128);
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
