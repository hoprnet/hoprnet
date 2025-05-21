//! The crate provides the probing functionality used by the transport layer
//! to identify different attributes of possible transport paths in the network.
//!
//! The goal of probing is to establish a map with weighted properties that will
//! allow the caller to optimize transport, verify transport path properties and
//! lay groundworks for mitigating potential adversarial behavior.
//!
//! There are 2 fundamental types of probing:
//! 1. Immediate hop probing - collects telemetry for direct 0-hop neighbors. Such telemetry
//! can be identified and potentially gamed by an adversary, but it is still useful to identify
//! the basic properties of the most immediate connection to the neighbor, since in the worst
//! case scenario the mitigation strategy can discard unsuitable peers.
//!
//! 2. Multi-hop probing - collects telemetry using a probing mechanism based on looping. A loop
//! is a message sent by this peer to itself through different pre-selected peers. This probing
//! mechanism can be combined together with the cover traffic into a single mechanism improving
//! the network view.

pub mod config;
pub mod errors;
pub mod messaging;
pub mod ping;

pub mod neighbors;
pub mod store;

use std::collections::HashMap;

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use futures_concurrency::stream::StreamExt as _;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_api::resolver::HoprDbResolverOperations;
use hopr_internal_types::protocol::HoprPseudonym;
use hopr_network_types::types::{ResolvedTransportRouting, ValidatedPath};
use hopr_primitive_types::prelude::Address;
use hopr_transport_packet::prelude::ApplicationData;
use hopr_transport_protocol::processor::{PacketError, PacketSendFinalizer};
use libp2p_identity::PeerId;
use messaging::Message;
use ping::{PingQueryReplier, PingQueryResult};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, strum::Display)]
pub enum HoprProbeProcess {
    #[strum(to_string = "probing queue")]
    ProbeQueue,
    #[strum(to_string = "probe rx side")]
    Rx,
    #[strum(to_string = "probe tx side")]
    Tx,
}

/// TODO
pub async fn continuous_network_probe<T, U, V, W, Db>(
    me: (OffchainPublicKey, Address),
    api: (T, U),
    manual_evenets_tx: V,
    store: W,
    cfg: config::ProbeConfig,
    db: Db,
) -> HashMap<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>
where
    T: futures::Sink<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)> + Clone + Send + Sync + 'static,
    T::Error: Send,
    U: futures::Stream<Item = (HoprPseudonym, ApplicationData)> + Send + Sync + 'static,
    W: store::PeerDiscoveryFetch + store::ProbeStatusUpdate + Clone + Send + Sync + 'static,
    V: futures::Stream<Item = (PeerId, PingQueryReplier)> + Send + Sync + 'static,
    Db: HoprDbResolverOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    let max_parallel_probes = cfg.max_parallel_probes;

    let immediate_peers = neighbors::neighbors_to_probe(store.clone(), cfg)
        .map(|peer| (peer, None))
        .merge(manual_evenets_tx.map(|(peer, notifier)| (peer, Some(notifier))));

    let msg_out = api.0.clone();

    let cache_hopr_pseudonym_to_peer_id: moka::future::Cache<HoprPseudonym, PeerId> = moka::future::Cache::builder()
        .time_to_live(std::time::Duration::from_secs(600))
        .max_capacity(100_000)
        .build();

    let cache_peer_routing: moka::future::Cache<PeerId, ResolvedTransportRouting> = moka::future::Cache::builder()
        .time_to_live(std::time::Duration::from_secs(600))
        .max_capacity(100_000)
        .build();

    let active_probes: moka::future::Cache<(HoprPseudonym, messaging::NeighborProbe), PingQueryReplier> =
        moka::future::Cache::builder()
            .time_to_live(cfg.timeout)
            .max_capacity(100_000)
            .build();

    let (tx_network_updater, rx_network_updater) = futures::channel::mpsc::channel::<(PeerId, futures::channel::mpsc::UnboundedReceiver<PingQueryResult>)>(1000);

    let mut processes = HashMap::<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>::new();

    // for each peer from the `immediate_peers`
    // - check if the mapping of peer_id to ResolvedTransportRouting::Forward exists, if not, create it
    // - map peer_id to stored ResolvedTransportRouting::Forward
    // - generate a PingQueryReplier
    // - map (HoprPseudonym, nonce) to a PingQueryReplier
    // - store the rx side of the PingQueryReplier if needed
    // - send the nonce and wait for the packet sent response

    processes.insert(HoprProbeProcess::Tx,
    hopr_async_runtime::prelude::spawn(immediate_peers
        .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
            let db = db.clone();
            let cache_hopr_pseudonym_to_peer_id = cache_hopr_pseudonym_to_peer_id.clone();
            let cache_peer_routing = cache_peer_routing.clone();
            let active_probes = active_probes.clone();
            let msg_out = msg_out.clone();
            let mut tx_network_updater = tx_network_updater.clone();

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
                            forward_path: ValidatedPath::direct(cp_ofk.into(), cp_address),
                            return_paths: vec![ValidatedPath::direct(me.0.into(), me.1)],
                        })
                    })
                    .await;

                match result {
                    Ok(resolved_transport_routing) => {
                        if let ResolvedTransportRouting::Forward { pseudonym, .. } = &resolved_transport_routing {
                            cache_hopr_pseudonym_to_peer_id
                                .insert(*pseudonym, peer)
                                .await;

                            let replier = if let Some(notifier) = notifier {
                                notifier
                            } else {
                                let (tx, rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();
                                
                                if let Err(error) = tx_network_updater
                                    .send((peer, rx))
                                    .await {
                                        tracing::error!(%peer, %error, "failed to send ping query result");
                                    }

                                PingQueryReplier::new(tx)
                            };

                            let maybe_data: anyhow::Result<ApplicationData> = Message::Probe(replier.challenge()).try_into();
                            match maybe_data {
                                Ok(data) => {
                                    let (packet_sent_tx, packet_sent_rx) = futures::channel::oneshot::channel::<std::result::Result<(), PacketError>>();
                                    futures::pin_mut!(msg_out);
                                    if let Err(_error) = msg_out.send((data, resolved_transport_routing.clone(), packet_sent_tx.into())).await {
                                        tracing::error!(%peer, "failed to send message");
                                    } else {
                                        if let Err(error) = packet_sent_rx.await {
                                            tracing::error!(%peer, %error, "failed to receive packet sent response")
                                        } else {
                                            // TODO: insert rx into a queue that reads
                                            active_probes
                                                .insert((*pseudonym, replier.challenge()), replier)
                                                .await;
                                        }
                                    }
                                },
                                Err(error) => tracing::error!(%peer, %error, "failed to convert message into data"),
                            }
                        } else {
                            tracing::error!(%peer, error = "logical error", "resolved transport routing is not forward");
                        }
                    },
                    Err(error) => tracing::error!(%peer, %error, "failed to resolve transport routing"),
                };
            }
        })));

    // read the rx sides of the PingQueryReplier and perform update actions on the network object
    processes.insert(
        HoprProbeProcess::Tx,
        hopr_async_runtime::prelude::spawn(rx_network_updater.for_each(move |(peer, rx)| {
            let store = store.clone();

            async move {
                let mut rx = rx;

                if let Some(result) = rx.next().await {
                    store.on_finished(&peer, &result).await;
                } else {
                    tracing::error!(%peer, "failed to receive ping query result, result channel closed before receiving");
                }
            }
        }))
    );

    // for each (HoprPseudonym, ApplicationData) tuple
    // - for each ApplicationData check Tag = 0, if not, forward further
    // - check if the pseudonym is inside my mappings to outgoing peers
    //   - if yes process response
    //   - if no, reply to the sender with a Pong

    // TODO: add

    HashMap::<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>::new()
}
