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
use futures::{channel::mpsc::{channel, UnboundedReceiver}, SinkExt, StreamExt};
use futures_concurrency::stream::StreamExt as _;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_api::{protocol::HoprDbProtocolOperations, resolver::HoprDbResolverOperations};
use hopr_internal_types::protocol::HoprPseudonym;
use hopr_network_types::types::{ResolvedTransportRouting, ValidatedPath};
use hopr_primitive_types::prelude::Address;
use hopr_transport_packet::prelude::{ApplicationData, ReservedTag};
use hopr_transport_protocol::processor::{PacketError, PacketSendFinalizer};
use libp2p_identity::PeerId;
use messaging::{Message, NeighborProbe};
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

pub struct Probe {
    /// Own addresses for self reference and surb creation
    me: (OffchainPublicKey, Address),
    cfg: config::ProbeConfig,
}

impl Probe {
    pub fn new(me: (OffchainPublicKey, Address), cfg: config::ProbeConfig) -> Self {
        Self { me, cfg }
    }

    /// The main function that assembles and starts the probing process.
    pub async fn continuous_network_probe<T, U, V, W, Db>(self,
        api: (T, U),          // lower (tx, rx) channels for sending and receiving messages
        manual_evenets_tx: V, // ping requests from the API
        store: W,             // peer store
        db: Db,               // database for SURB & peer resolution
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
        Db: HoprDbResolverOperations + HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        let max_parallel_probes = self.cfg.max_parallel_probes;

        // For each probe target a cached version of transport routing is stored
        let cache_peer_routing: moka::future::Cache<PeerId, ResolvedTransportRouting> = moka::future::Cache::builder()
            .time_to_live(std::time::Duration::from_secs(600))
            .max_capacity(100_000)
            .build();

        let active_probes: moka::future::Cache<(HoprPseudonym, messaging::NeighborProbe), PingQueryReplier> =
            moka::future::Cache::builder()
                .time_to_live(self.cfg.timeout)
                .max_capacity(100_000)
                .build();

        
        // Link between the sending side and receiving side's network updater
        let (store_updater_tx, store_updater_rx) = channel::<(
            PeerId,
            UnboundedReceiver<PingQueryResult>,
        )>(1000);

        let active_probes_rx = active_probes.clone();
        let db_rx = db.clone();
        let push_to_network = api.0.clone();

        let mut processes = HashMap::<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>::new();

        // for each peer from the `immediate_peers`
        // - check if the mapping of peer_id to ResolvedTransportRouting::Forward exists, if not, create it
        // - map peer_id to stored ResolvedTransportRouting::Forward
        // - generate a PingQueryReplier
        // - map (HoprPseudonym, nonce) to a PingQueryReplier
        // - store the rx side of the PingQueryReplier if needed
        // - send the nonce and wait for the packet sent response
        let direct_neighbors = neighbors::neighbors_to_probe(store.clone(), self.cfg)
            .map(|peer| (peer, None))
            .merge(manual_evenets_tx.map(|(peer, notifier)| (peer, Some(notifier))));

        processes.insert(HoprProbeProcess::Tx,
    hopr_async_runtime::prelude::spawn(direct_neighbors
        .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
            let db = db.clone();
            let cache_peer_routing = cache_peer_routing.clone();
            let active_probes = active_probes.clone();
            let push_to_network = push_to_network.clone();
            let mut tx_network_updater = store_updater_tx.clone();
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
                            forward_path: ValidatedPath::direct(cp_ofk.into(), cp_address),
                            return_paths: vec![ValidatedPath::direct(me.0.into(), me.1)],
                        })
                    })
                    .await;

                match result {
                    Ok(resolved_transport_routing) => {
                        if let ResolvedTransportRouting::Forward { pseudonym, .. } = &resolved_transport_routing {
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
                                    futures::pin_mut!(push_to_network);
                                    if let Err(_error) = push_to_network.send((data, resolved_transport_routing.clone(), packet_sent_tx.into())).await {
                                        tracing::error!(%peer, "failed to send message");
                                    } else {
                                        if let Err(error) = packet_sent_rx.await {
                                            tracing::error!(%peer, %error, "failed to receive packet sent response")
                                        } else {
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
        hopr_async_runtime::prelude::spawn(store_updater_rx.for_each(move |(peer, rx)| {
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
        processes.insert(
        HoprProbeProcess::Tx,
        hopr_async_runtime::prelude::spawn(api.1.for_each_concurrent(max_parallel_probes, move |(pseudonym, data)| {
            let active_probes = active_probes_rx.clone();
            let msg_out = api.0.clone();
            let db = db_rx.clone();

            async move {
            if data.application_tag != ReservedTag::Ping as u32 {
                todo!("receive as data");
            }

            let pong: anyhow::Result<Message> = data.try_into().context("failed to convert data into message");
            match pong {
                Ok(pong) => {
                    match pong {
                        Message::Telemetry(_path_telemetry) => {
                            tracing::debug!(%pseudonym, reason = "feature not implemented", "received telemetry this node originate");
                        },
                        Message::Probe(NeighborProbe::Ping(ping)) => {
                            match db.find_surb(hopr_network_types::types::SurbMatcher::Pseudonym(pseudonym)).await {
                                Ok((sender_id, surb)) => {
                                    let (packet_sent_tx, packet_sent_rx) = futures::channel::oneshot::channel::<std::result::Result<(), PacketError>>();
                                    futures::pin_mut!(msg_out);
                                    if let Err(_error) = msg_out.send((Message::Probe(NeighborProbe::Pong(ping)).try_into().expect("Pong to message conversion cannot fail"), ResolvedTransportRouting::Return(sender_id, surb), packet_sent_tx.into())).await {
                                        tracing::error!(%pseudonym, "failed to send message");
                                    } else {
                                        if let Err(error) = packet_sent_rx.await {
                                            tracing::error!(%pseudonym, %error, "failed to receive packet sent response")
                                        } 
                                    }
                                },
                                Err(error) => tracing::error!(%pseudonym, %error, "failed to get a SURB"),
                            }
                        },
                        Message::Probe(NeighborProbe::Pong(ping)) => {
                            if let Some(replier) = active_probes.get(&(pseudonym, NeighborProbe::Ping(ping))).await {
                                replier.notify(NeighborProbe::Pong(ping));
                            } else {
                                tracing::warn!(%pseudonym, %ping, "received pong for unknown ping, adversarial injection?");
                            };
                        },
                    }
                },
                Err(error) => {
                    tracing::error!(%pseudonym, %error, "cannot deserialize message");
                },
            }
        }})),
    );

        HashMap::<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: test that with NeighporProbe and Telemetry at least 1 SURB can be sent as well.
}
