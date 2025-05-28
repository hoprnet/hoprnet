use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use futures::{FutureExt, SinkExt, StreamExt, channel::oneshot::channel, pin_mut};
use futures_concurrency::stream::StreamExt as _;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::protocol::HoprPseudonym;
use hopr_network_types::types::{ResolvedTransportRouting, SurbMatcher, ValidatedPath};
use hopr_platform::time::native::current_time;
use hopr_primitive_types::{prelude::Address, traits::AsUnixTimestamp};
use hopr_transport_packet::prelude::{ApplicationData, ReservedTag, Tag};
use hopr_transport_protocol::processor::{PacketError, PacketSendFinalizer};
use libp2p_identity::PeerId;

use crate::{
    HoprProbeProcess,
    config::ProbeConfig,
    content::{Message, NeighborProbe},
    errors::ProbeError,
    neighbors::neighbors_to_probe,
    ping::PingQueryReplier,
    traits::{CacheOperations, PeerDiscoveryFetch, ProbeStatusUpdate},
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
struct Sender<T>
where
    T: futures::Sink<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)> + Clone + Send + Sync + 'static,
{
    downstream: T,
}

impl<T> Sender<T>
where
    T: futures::Sink<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)> + Clone + Send + Sync + 'static,
{
    #[tracing::instrument(level = "debug", skip(self, path, message), fields(message=%message, nonce=%to_nonce(&message), pseudonym = %to_pseudonym(&path)), err(level = tracing::Level::DEBUG, Display))]
    async fn send_message(self, path: ResolvedTransportRouting, message: Message) -> crate::errors::Result<()> {
        let message: ApplicationData = message
            .try_into()
            .map_err(|e: anyhow::Error| ProbeError::SendError(e.to_string()))?;
        let (packet_sent_tx, packet_sent_rx) = channel::<std::result::Result<(), PacketError>>();

        let push_to_network = self.downstream;
        futures::pin_mut!(push_to_network);
        if push_to_network
            .as_mut()
            .send((message, path, packet_sent_tx.into()))
            .await
            .is_ok()
        {
            packet_sent_rx
                .await
                .map_err(|error| ProbeError::SendError(error.to_string()))?
                .map_err(|error| ProbeError::SendError(error.to_string()))
        } else {
            Err(ProbeError::SendError("transport error".to_string()))
        }
    }
}

pub struct Probe {
    /// Own addresses for self reference and surb creation
    me: (OffchainPublicKey, Address),
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
    ) -> HashMap<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>
    where
        T: futures::Sink<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)>
            + Clone
            + Send
            + Sync
            + 'static,
        T::Error: Send,
        U: futures::Stream<Item = (HoprPseudonym, ApplicationData)> + Send + Sync + 'static,
        W: PeerDiscoveryFetch + ProbeStatusUpdate + Clone + Send + Sync + 'static,
        C: CacheOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
        V: futures::Stream<Item = (PeerId, PingQueryReplier)> + Send + Sync + 'static,
        Up: futures::Sink<(HoprPseudonym, ApplicationData)> + Clone + Send + Sync + 'static,
    {
        let max_parallel_probes = self.cfg.max_parallel_probes;

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
                        let (peer, _start, _notifier) = v;

                        tracing::debug!(%peer, pseudonym = %k.0, probe = %k.1, reason = "timeout", "probe failed");
                        async move {
                            store
                                .on_finished(&peer, &Err(ProbeError::Timeout(timeout.as_millis() as u64)))
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

        let mut processes = HashMap::<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>::new();

        // -- Emit probes --
        let direct_neighbors = neighbors_to_probe(store.clone(), self.cfg)
            .map(|peer| (peer, None))
            .merge(manual_events.map(|(peer, notifier)| (peer, Some(notifier))));

        processes.insert(
            HoprProbeProcess::Emit,
            hopr_async_runtime::prelude::spawn(direct_neighbors
                .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
                    let db = db.clone();
                    let cache_peer_routing = cache_peer_routing.clone();
                    let active_probes = active_probes.clone();
                    let push_to_network = Sender { downstream: push_to_network.clone() };
                    let me = self.me;

                    async move {
                        let result = cache_peer_routing
                            .try_get_with(peer, async move {
                                let cp_ofk = OffchainPublicKey::try_from(peer)
                                    .context(format!("failed to convert {peer} to offchain public key"))?;
                                let cp_address = db
                                    .resolve_chain_key(&cp_ofk)
                                    .await?
                                    .ok_or_else(|| anyhow::anyhow!("Failed to resolve chain key for peer: {peer}"))?;

                                Ok::<ResolvedTransportRouting, anyhow::Error>(ResolvedTransportRouting::Forward {
                                    pseudonym: HoprPseudonym::random(),
                                    forward_path: ValidatedPath::direct(cp_ofk, cp_address),
                                    return_paths: vec![ValidatedPath::direct(me.0, me.1)],
                                })
                            })
                            .await;

                        match result {
                            Ok(ResolvedTransportRouting::Forward { pseudonym, forward_path, return_paths }) => {
                                let nonce = if let Some(notifier) = &notifier {
                                    notifier.challenge()
                                } else {
                                    NeighborProbe::random_nonce()
                                };

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
            )
        );

        // -- Process probes --
        processes.insert(
            HoprProbeProcess::Process,
            hopr_async_runtime::prelude::spawn(api.1.for_each_concurrent(None, move |(pseudonym, data)| {
                let active_probes = active_probes_rx.clone();
                let push_to_network = Sender { downstream: api.0.clone() };
                let db = db_rx.clone();
                let store = store.clone();
                let move_up = move_up.clone();

                async move {
                    // TODO(v3.1): compare not only against ping tag, but also against telemetry that will be occuring on random tags
                    if data.application_tag == Tag::from(ReservedTag::Ping) {
                        let message: anyhow::Result<Message> = data.try_into().context("failed to convert data into message");

                        match message {
                            Ok(message) => {
                                match message {
                                    Message::Telemetry(_path_telemetry) => {
                                        tracing::warn!(%pseudonym, reason = "feature not implemented", "this node could not originate the telemetry");
                                    },
                                    Message::Probe(NeighborProbe::Ping(ping)) => {
                                        tracing::debug!(%pseudonym, nonce = hex::encode(ping), "received ping");
                                        match db.find_surb(SurbMatcher::Pseudonym(pseudonym)).await.map(|(sender_id, surb)| ResolvedTransportRouting::Return(sender_id, surb)) {
                                            Ok(path) => {
                                                let message = Message::Probe(NeighborProbe::Pong(ping));
                                                let _ = push_to_network.send_message(path, message).await;
                                            },
                                            Err(error) => tracing::error!(%pseudonym, %error, "failed to get a SURB, cannot send pong"),
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
                                                replier.notify(NeighborProbe::Pong(pong))
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
                        if move_up.send((pseudonym, data.clone())).await.is_err() {
                            tracing::error!(%pseudonym, error = "receiver error", "failed to send message up");
                        }
                    }
                }
            }))
        );

        processes
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::RwLock, time::Duration};

    use async_trait::async_trait;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};

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
        on_finished: Arc<RwLock<Vec<(PeerId, crate::errors::Result<Duration>)>>>,
    }

    #[async_trait]
    impl ProbeStatusUpdate for PeerStore {
        async fn on_finished(&self, peer: &PeerId, result: &crate::errors::Result<Duration>) {
            let mut on_finished = self.on_finished.write().unwrap();
            on_finished.push((
                peer.clone(),
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
    impl CacheOperations for Cache {
        async fn find_surb(
            &self,
            _matcher: SurbMatcher,
        ) -> hopr_db_api::errors::Result<(hopr_db_api::protocol::HoprSenderId, hopr_db_api::protocol::HoprSurb)>
        {
            // Mock implementation for testing purposes
            Ok((
                hopr_db_api::protocol::HoprSenderId::random(),
                random_memory_violating_surb(),
            ))
        }

        async fn resolve_chain_key(
            &self,
            _offchain_key: &OffchainPublicKey,
        ) -> hopr_db_api::errors::Result<Option<Address>> {
            Ok(Some(ONCHAIN_KEYPAIR.public().to_address()))
        }
    }

    /// !!! This generates a completely random data blob that pretends to be a SURB.
    ///
    /// !!! It must never by accessed or invoked, only passed around.
    fn random_memory_violating_surb() -> hopr_db_api::protocol::HoprSurb {
        const SURB_SIZE: usize = std::mem::size_of::<hopr_db_api::protocol::HoprSurb>();
        let surb = unsafe {
            std::mem::transmute::<[u8; SURB_SIZE], hopr_db_api::protocol::HoprSurb>(hopr_crypto_random::random_bytes())
        };

        surb
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_value_for_manual_neighbor_probe() -> anyhow::Result<()> {
        let mut cfg: ProbeConfig = Default::default();
        cfg.timeout = std::time::Duration::from_millis(5);

        let probe = Probe::new((*OFFCHAIN_KEYPAIR.public(), ONCHAIN_KEYPAIR.public().to_address()), cfg);

        let (from_probing_up_tx, _from_probing_up_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationData)>(100);

        let (from_probing_to_network_tx, mut from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)>(100);

        let (mut from_network_to_probing_tx, from_network_to_probing_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationData)>(100);

        let (mut manual_probe_tx, manual_probe_rx) = futures::channel::mpsc::channel::<(PeerId, PingQueryReplier)>(100);

        let db = Cache {};

        let stub_store = PeerStore {
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        let jhs = probe
            .continuously_scan(
                (from_probing_to_network_tx, from_network_to_probing_rx),
                manual_probe_rx,
                stub_store,
                db,
                from_probing_up_tx,
            )
            .await;

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<std::result::Result<Duration, ProbeError>>();
        manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

        let channel: hopr_async_runtime::prelude::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            while let Some((data, path, finalizer)) = from_probing_to_network_rx.next().await {
                if let ResolvedTransportRouting::Forward { pseudonym, .. } = path {
                    finalizer.finalize(Ok(()));

                    let message: Message = data.try_into().context("failed to convert data into message")?;
                    if let Message::Probe(NeighborProbe::Ping(ping)) = message {
                        let pong_message = Message::Probe(NeighborProbe::Pong(ping));
                        from_network_to_probing_tx
                            .send((pseudonym, pong_message.try_into()?))
                            .await?;
                    }
                }
            }

            Ok(())
        });

        let _duration = tokio::time::timeout(std::time::Duration::from_secs(1), rx.next())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Probe did not return a result in time"))??;

        jhs.into_iter().for_each(|(_name, handle)| handle.abort());
        let _ = channel.await?;

        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_failure_on_manual_fail() -> anyhow::Result<()> {
        let mut cfg: ProbeConfig = Default::default();
        cfg.timeout = std::time::Duration::from_millis(5);

        let probe = Probe::new((*OFFCHAIN_KEYPAIR.public(), ONCHAIN_KEYPAIR.public().to_address()), cfg);

        let (from_probing_up_tx, _from_probing_up_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationData)>(100);

        let (from_probing_to_network_tx, mut from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)>(100);

        let (_from_network_to_probing_tx, from_network_to_probing_rx) =
            futures::channel::mpsc::channel::<(HoprPseudonym, ApplicationData)>(100);

        let (mut manual_probe_tx, manual_probe_rx) = futures::channel::mpsc::channel::<(PeerId, PingQueryReplier)>(100);

        let db = Cache {};

        let stub_store = PeerStore {
            get_peers: Arc::new(RwLock::new(VecDeque::new())),
            on_finished: Arc::new(RwLock::new(Vec::new())),
        };

        let jhs = probe
            .continuously_scan(
                (from_probing_to_network_tx, from_network_to_probing_rx),
                manual_probe_rx,
                stub_store,
                db,
                from_probing_up_tx,
            )
            .await;

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<std::result::Result<Duration, ProbeError>>();
        manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

        let channel: hopr_async_runtime::prelude::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            while let Some((_data, path, finalizer)) = from_probing_to_network_rx.next().await {
                if let ResolvedTransportRouting::Forward { .. } = path {
                    finalizer.finalize(Err(PacketError::MissingDomainSeparator));
                    // Simulate a failure to sent a manual probe
                }
            }

            Ok(())
        });

        assert!(tokio::time::timeout(cfg.timeout * 2, rx.next()).await.is_err());

        jhs.into_iter().for_each(|(_name, handle)| handle.abort());
        let _ = channel.await?;

        Ok(())
    }
}
