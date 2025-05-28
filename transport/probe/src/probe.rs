use std::{collections::HashMap, ops::Div, sync::Arc};

use anyhow::Context;
use futures::{FutureExt, SinkExt, StreamExt, channel::oneshot::channel, pin_mut};
use futures_concurrency::stream::StreamExt as _;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_api::{protocol::HoprDbProtocolOperations, resolver::HoprDbResolverOperations};
use hopr_internal_types::protocol::HoprPseudonym;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleHistogram};
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
    neighbors::neighbors_to_probe,
    ping::PingQueryReplier,
    store,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_PING: SimpleHistogram =
        SimpleHistogram::new(
            "hopr_ping_time_sec",
            "Measures total time it takes to ping a single node (seconds)",
            vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0],
        ).unwrap();
    static ref METRIC_PROBE_COUNT: MultiCounter = MultiCounter::new(
            "hopr_probe_count",
            "Total number of pings by result",
            &["success"]
        ).unwrap();
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
    pub async fn continuously_scan<T, U, V, W, Db, Up>(
        self,
        api: (T, U),          // lower (tx, rx) channels for sending and receiving messages
        manual_evenets_tx: V, // ping requests from the API
        store: W,             // peer store
        db: Db,               // database for SURB & peer resolution
        move_up: Up,          // send actual messages from the network
    ) -> HashMap<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>
    where
        T: futures::Sink<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)>
            + Clone
            + Send
            + Sync
            + 'static,
        T::Error: Send,
        U: futures::Stream<Item = (HoprPseudonym, ApplicationData)> + Send + Sync + 'static,
        W: store::PeerDiscoveryFetch + store::ProbeStatusUpdate + Clone + Send + Sync + 'static,
        V: futures::Stream<Item = (PeerId, PingQueryReplier)> + Send + Sync + 'static,
        Up: futures::Sink<(HoprPseudonym, ApplicationData)> + Clone + Send + Sync + 'static,
        Db: HoprDbResolverOperations + HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
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
                        let store_eviction = store_eviction.clone();
                        let (peer, _start, _notifier) = v;

                        tracing::debug!(%peer, pseudonym = %k.0, probe = %k.1, reason = "timeout", "evicting probe");
                        async move {
                            store_eviction
                                .on_finished(
                                    &peer,
                                    &Err(crate::errors::ProbeError::Timeout(timeout.as_millis() as u64)),
                                )
                                .await;

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_PROBE_COUNT.increment(&["false"]);
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
            .merge(manual_evenets_tx.map(|(peer, notifier)| (peer, Some(notifier))));

        processes.insert(
            HoprProbeProcess::Emit,
            hopr_async_runtime::prelude::spawn(direct_neighbors
                .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
                    let db = db.clone();
                    let cache_peer_routing = cache_peer_routing.clone();
                    let active_probes = active_probes.clone();
                    let push_to_network = push_to_network.clone();
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

                                let maybe_data: anyhow::Result<ApplicationData> = Message::Probe(nonce).try_into();
                                match maybe_data {
                                    Ok(data) => {
                                        let (packet_sent_tx, packet_sent_rx) = channel::<std::result::Result<(), PacketError>>();

                                        futures::pin_mut!(push_to_network);
                                        if push_to_network.send((data, ResolvedTransportRouting::Forward { pseudonym, forward_path, return_paths }, packet_sent_tx.into())).await.is_ok() {
                                            if let Err(error) = packet_sent_rx.await {
                                                tracing::error!(%peer, %error, "failed to receive packet sent confirmation")
                                            } else {
                                                tracing::debug!(%peer, %pseudonym, %nonce, "waiting for a sent probe");
                                                active_probes
                                                    .insert((pseudonym, nonce), (peer, current_time().as_unix_timestamp(), notifier))
                                                    .await;
                                            }
                                        } else {
                                            tracing::error!(%peer, error = "transport error", "failed to send message");
                                        }
                                    },
                                    Err(error) => tracing::error!(%peer, %error, "failed to convert message into data"),
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
                let push_to_network = api.0.clone();
                let db = db_rx.clone();
                let store = store.clone();
                let move_up = move_up.clone();

                async move {
                    // TODO(v3.1): compare not only against ping tag, but also against telemetry that will be occuring on random tags
                    if data.application_tag != Tag::from(ReservedTag::Ping) {
                        pin_mut!(move_up);
                        if move_up.send((pseudonym, data.clone())).await.is_err() {
                            tracing::error!(%pseudonym, error = "receiver error", "failed to send message up");
                        }
                    }

                    let message: anyhow::Result<Message> = data.try_into().context("failed to convert data into message");
                    match message {
                        Ok(message) => {
                            match message {
                                Message::Telemetry(_path_telemetry) => {
                                    tracing::warn!(%pseudonym, reason = "feature not implemented", "this node could not originate the telemetry");
                                },
                                Message::Probe(NeighborProbe::Ping(ping)) => {
                                    tracing::debug!(%pseudonym, nonce = hex::encode(ping), "received ping");
                                    match db.find_surb(SurbMatcher::Pseudonym(pseudonym)).await {
                                        Ok((sender_id, surb)) => {
                                            let (packet_sent_tx, packet_sent_rx) = channel::<std::result::Result<(), PacketError>>();

                                            futures::pin_mut!(push_to_network);
                                            tracing::debug!(%pseudonym, nonce = hex::encode(ping), "sending pong");
                                            if push_to_network.send((Message::Probe(NeighborProbe::Pong(ping)).try_into().expect("Pong to message conversion cannot fail"), ResolvedTransportRouting::Return(sender_id, surb), packet_sent_tx.into())).await.is_ok() {
                                                if let Err(error) = packet_sent_rx.await {
                                                    tracing::error!(%pseudonym, %error, "failed to receive pong sent confirmation")
                                                }
                                            } else {
                                                tracing::error!(%pseudonym, error = "transport error", "failed to send pong");
                                            }
                                        },
                                        Err(error) => tracing::error!(%pseudonym, %error, "failed to get a SURB, cannot send pong back"),
                                    }
                                },
                                Message::Probe(NeighborProbe::Pong(ping)) => {
                                    tracing::debug!(%pseudonym, nonce = hex::encode(ping), "received pong");
                                    if let Some((peer, start, replier)) = active_probes.remove(&(pseudonym, NeighborProbe::Ping(ping))).await {
                                        let latency = current_time()
                                            .as_unix_timestamp()
                                            .saturating_sub(start)
                                            .div(2u32); // RTT -> unidirectional latency
                                        store.on_finished(&peer, &Ok(latency)).await;

                                        #[cfg(all(feature = "prometheus", not(test)))]
                                        {
                                            METRIC_TIME_TO_PING.observe((latency.as_millis() as f64) / 1000.0); // precision for seconds
                                            METRIC_PROBE_COUNT.increment(&["true"]);
                                        }

                                        if let Some(replier) = replier {
                                            replier.notify(NeighborProbe::Pong(ping))
                                        };
                                    } else {
                                        tracing::warn!(%pseudonym, nonce = hex::encode(ping), possible_reasons = "[timeout, adversary]", "received pong for unknown probe");
                                    };
                                },
                            }
                        },
                        Err(error) => tracing::error!(%pseudonym, %error, "cannot deserialize message"),
                    }
                }
            }))
        );

        processes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};

    lazy_static::lazy_static!(
        static ref OFFCHAIN_KEYPAIR: OffchainKeypair = OffchainKeypair::random();
        static ref ONCHAIN_KEYPAIR: ChainKeypair = ChainKeypair::random();
    );

    #[tokio::test]
    async fn probe_should_timeout_if_no_response_arrives_back() -> anyhow::Result<()> {
        let mut cfg: ProbeConfig = Default::default();
        cfg.timeout = std::time::Duration::from_millis(2);

        let probe = Probe::new((*OFFCHAIN_KEYPAIR.public(), ONCHAIN_KEYPAIR.public().to_address()), cfg);
        // let jhs = probe.continuously_scan().await;

        Ok(())
    }
}
