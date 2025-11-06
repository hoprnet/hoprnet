use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use futures::{FutureExt, SinkExt, StreamExt, pin_mut};
use futures_concurrency::stream::StreamExt as _;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::prelude::*;
use hopr_network_types::types::{ResolvedTransportRouting};
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
    traits::{DbOperations, PeerDiscoveryFetch, ProbeStatusUpdate},
};

#[inline(always)]
fn to_nonce(message: &Message) -> String {
    match message {
        Message::Probe(NeighborProbe::Ping(ping)) => hex::encode(ping),
        Message::Probe(NeighborProbe::Pong(ping)) => hex::encode(ping),
        _ => "<telemetry>".to_string(),
    }
}

#[inline(always)]
fn to_pseudonym(path: &ResolvedTransportRouting) -> HoprPseudonym {
    match path {
        ResolvedTransportRouting::Forward { pseudonym, .. } => *pseudonym,
        ResolvedTransportRouting::Return(pseudonym, _) => pseudonym.pseudonym(),
    }
}

#[derive(Clone, Debug)]
struct Sender<T> {
    downstream: T,
}

impl<T> Sender<T>
where
    T: futures::Sink<(ResolvedTransportRouting, ApplicationDataOut)> + Clone + Send + Sync + 'static,
{
    #[tracing::instrument(level = "debug", skip(self, path, message), fields(message=%message, nonce=%to_nonce(&message), pseudonym=%to_pseudonym(&path)), ret(level = tracing::Level::TRACE), err(Display))]
    async fn send_message(self, path: ResolvedTransportRouting, message: Message) -> crate::errors::Result<()> {
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
    pub async fn continuously_scan<T, U, V, W, C, Up>(
        self,
        api: (T, U),      // lower (tx, rx) channels for sending and receiving messages
        manual_events: V, // explicit requests from the API
        store: W,         // peer store
        db: C,            // database for SURB & peer resolution
        move_up: Up,      // forward up non-probing messages from the network
    ) -> HashMap<HoprProbeProcess, hopr_async_runtime::AbortHandle>
    where
        T: futures::Sink<(ResolvedTransportRouting, ApplicationDataOut)> + Clone + Send + Sync + 'static,
        T::Error: Send,
        U: futures::Stream<Item = (HoprPseudonym, ApplicationDataIn)> + Send + Sync + 'static,
        W: PeerDiscoveryFetch + ProbeStatusUpdate + Clone + Send + Sync + 'static,
        C: DbOperations + Clone + Send + Sync + 'static,
        V: futures::Stream<Item = (PeerId, PingQueryReplier)> + Send + Sync + 'static,
        Up: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Clone + Send + Sync + 'static,
    {
        let max_parallel_probes = self.cfg.max_parallel_probes;
        let interval_between_rounds = self.cfg.interval;

        // For each probe target a cached version of transport routing is stored
        let cache_peer_routing: moka::future::Cache<PeerId, ResolvedTransportRouting> = moka::future::Cache::builder()
            .time_to_live(std::time::Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

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
                        async move {
                            store
                                .on_finished(&peer, &Err(ProbeError::Timeout(timeout.as_secs())))
                                .await;
                        }
                        .boxed()
                    } else {
                        // If the eviction cause is not expiration, nothing needs to be done
                        futures::future::ready(()).boxed()
                    }
                },
            )
            .build();

        let active_probes_rx = active_probes.clone();
        let db_rx = db.clone();
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
                        let db = db.clone();
                        let cache_peer_routing = cache_peer_routing.clone();
                        let active_probes = active_probes.clone();
                        let push_to_network = Sender { downstream: push_to_network.clone() };
                        let me = self.me;

                        async move {
                            let result = cache_peer_routing
                                .try_get_with(peer, async move {
                                    // TODO: This is a CPU intensive operation, convert the probing mechanism to use OffchainPublicKey instead of PeerIDs!
                                    // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
                                    let pubkey = hopr_parallelize::cpu::spawn_blocking(move || OffchainPublicKey::from_peerid(&peer))
                                        .await
                                        .context(format!("failed to convert {peer} to offchain public key"))?;
                                    let cp_address = db
                                        .resolve_chain_key(&pubkey)
                                        .await?
                                        .ok_or_else(|| anyhow::anyhow!("Failed to resolve chain key for peer: {peer}"))?;

                                    Ok::<ResolvedTransportRouting, anyhow::Error>(ResolvedTransportRouting::Forward {
                                        pseudonym: HoprPseudonym::random(),
                                        forward_path: ValidatedPath::direct(pubkey, cp_address),
                                        return_paths: vec![ValidatedPath::direct(me.0, me.1)],
                                    })
                                })
                                .await;

                            match result {
                                Ok(ResolvedTransportRouting::Forward { pseudonym, forward_path, return_paths }) => {
                                    let nonce = NeighborProbe::random_nonce();

                                    let message = Message::Probe(nonce);
                                    let path = ResolvedTransportRouting::Forward { pseudonym, forward_path, return_paths };
                                    if push_to_network.send_message(path, message).await.is_ok() {
                                        active_probes
                                            .insert((pseudonym, nonce), (peer, current_time().as_unix_timestamp(), notifier))
                                            .await;
                                    }
                                },
                                Ok(_) => tracing::error!(%peer, error = "logical error", "resolved transport routing is not forward"),
                                Err(error) => tracing::error!(%peer, %error, "failed to resolve transport routing"),
                            };
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
                let db = db_rx.clone();
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
                                        match db.find_surb(pseudonym.into()).await.map(|found_surb| ResolvedTransportRouting::Return(found_surb.sender_id, found_surb.surb)) {
                                            Ok(path) => {
                                                tracing::trace!(%pseudonym, nonce = hex::encode(ping), "wrapping a pong in the found SURB");
                                            let message = Message::Probe(NeighborProbe::Pong(ping));
                                                if let Err(error) = push_to_network.send_message(path, message).await {
                                                    tracing::error!(%pseudonym, %error, "failed to send back a pong");
                                                }
                                            },
                                            Err(error) => tracing::error!(%pseudonym, %error, "failed to get a SURB, cannot send back a pong"),
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
    use std::{collections::VecDeque, convert::Infallible, sync::RwLock, time::Duration};

    use async_trait::async_trait;
    use futures::future::BoxFuture;
    use hopr_api::db::FoundSurb;
    use hopr_crypto_packet::{HoprSurb, prelude::HoprSenderId};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_network_types::prelude::SurbMatcher;
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

    #[derive(Debug, Clone)]
    pub struct Cache {}

    #[async_trait]
    impl DbOperations for Cache {
        type ChainError = Infallible;
        type DbError = Infallible;

        async fn find_surb(&self, _matcher: SurbMatcher) -> Result<FoundSurb, Self::DbError> {
            // Mock implementation for testing purposes
            Ok(FoundSurb {
                sender_id: HoprSenderId::random(),
                surb: random_memory_violating_surb(),
                remaining: 0,
            })
        }

        async fn resolve_chain_key(
            &self,
            _offchain_key: &OffchainPublicKey,
        ) -> Result<Option<Address>, Self::ChainError> {
            Ok(Some(ONCHAIN_KEYPAIR.public().to_address()))
        }
    }

    /// !!! This generates a completely random data blob that pretends to be a SURB.
    ///
    /// !!! It must never be accessed or invoked, only passed around.
    fn random_memory_violating_surb() -> HoprSurb {
        const SURB_SIZE: usize = size_of::<HoprSurb>();

        unsafe { std::mem::transmute::<[u8; SURB_SIZE], HoprSurb>(hopr_crypto_random::random_bytes()) }
    }

    struct TestInterface {
        from_probing_up_rx: futures::channel::mpsc::Receiver<(HoprPseudonym, ApplicationDataIn)>,
        from_probing_to_network_rx: futures::channel::mpsc::Receiver<(ResolvedTransportRouting, ApplicationDataOut)>,
        from_network_to_probing_tx: futures::channel::mpsc::Sender<(HoprPseudonym, ApplicationDataIn)>,
        manual_probe_tx: futures::channel::mpsc::Sender<(PeerId, PingQueryReplier)>,
    }

    async fn test_with_probing<F, Db, St, Fut>(cfg: ProbeConfig, store: St, db: Db, test: F) -> anyhow::Result<()>
    where
        Fut: std::future::Future<Output = anyhow::Result<()>>,
        F: Fn(TestInterface) -> Fut + Send + Sync + 'static,
        Db: DbOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
        St: ProbeStatusUpdate + PeerDiscoveryFetch + Clone + Send + Sync + 'static,
    {
        let probe = Probe::new((*OFFCHAIN_KEYPAIR.public(), ONCHAIN_KEYPAIR.public().to_address()), cfg);

        let (from_probing_up_tx, from_probing_up_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationDataIn)>(100);

        let (from_probing_to_network_tx, from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(ResolvedTransportRouting, ApplicationDataOut)>(100);

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
                db,
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
    ) -> impl Fn((ResolvedTransportRouting, ApplicationDataOut)) -> BoxFuture<'static, ()> {
        debug_assert!(
            (NO_PROBE_PASSES..=ALL_PROBES_PASS).contains(&pass_rate),
            "Pass rate must be between {NO_PROBE_PASSES} and {ALL_PROBES_PASS}"
        );

        move |(path, data_out): (ResolvedTransportRouting, ApplicationDataOut)| -> BoxFuture<'static, ()> {
            let mut from_network_to_probing_tx = from_network_to_probing_tx.clone();

            Box::pin(async move {
                if let ResolvedTransportRouting::Forward { pseudonym, .. } = path {
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
                                    pseudonym,
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

        test_with_probing(cfg, store, Cache {}, move |iface: TestInterface| async move {
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

        test_with_probing(cfg, store, Cache {}, move |iface: TestInterface| async move {
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

        test_with_probing(cfg, store.clone(), Cache {}, move |iface: TestInterface| async move {
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

        test_with_probing(cfg, store.clone(), Cache {}, move |iface: TestInterface| async move {
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

        test_with_probing(cfg, store.clone(), Cache {}, move |iface: TestInterface| async move {
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
