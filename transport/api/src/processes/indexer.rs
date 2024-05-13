use core_network::{network::Network, PeerId};
use core_p2p::libp2p::swarm::derive_prelude::Multiaddr;
use futures::{channel::mpsc::Sender, future::poll_fn, StreamExt};
use hopr_crypto_types::types::OffchainPublicKey;
use std::{pin::Pin, sync::Arc};
use tracing::{error, warn};

use hopr_db_api::{
    peers::HoprDbPeersOperations, registry::HoprDbRegistryOperations, resolver::HoprDbResolverOperations,
};

use chain_types::chain_events::NetworkRegistryStatus;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PeerEligibility {
    Eligible,
    Ineligible,
}

impl From<NetworkRegistryStatus> for PeerEligibility {
    fn from(value: NetworkRegistryStatus) -> Self {
        match value {
            NetworkRegistryStatus::Allowed => Self::Eligible,
            NetworkRegistryStatus::Denied => Self::Ineligible,
        }
    }
}

/// Indexer events triggered externally from the [crate::HoprTransport] object.
pub enum IndexerToProcess {
    EligibilityUpdate(PeerId, PeerEligibility),
    RegisterStatusUpdate,
    Announce(PeerId, Vec<Multiaddr>),
}

#[derive(Debug)]
/// Processed indexer generated events.
pub enum IndexerProcessed {
    Allow(PeerId),
    Ban(PeerId),
    Announce(PeerId, Vec<Multiaddr>),
}

/// Implementor interface for indexer actions
#[derive(Debug, Clone)]
pub struct IndexerActions {
    internal_emitter: Sender<IndexerToProcess>,
}

impl IndexerActions {
    pub fn new<T>(db: T, network: Arc<Network<T>>, emitter: Sender<IndexerProcessed>) -> Self
    where
        T: HoprDbPeersOperations
            + HoprDbResolverOperations
            + HoprDbRegistryOperations
            + Send
            + Sync
            + 'static
            + std::fmt::Debug
            + Clone,
    {
        let (to_process_tx, mut to_process_rx) =
            futures::channel::mpsc::channel::<IndexerToProcess>(crate::constants::INDEXER_UPDATE_QUEUE_SIZE);

        // NOTE: This spawned task does not need to be explicitly canceled, since it will
        // be automatically dropped when the event sender object is dropped.
        async_std::task::spawn(async move {
            let mut emitter = emitter;
            let db_local = db.clone();

            while let Some(value) = to_process_rx.next().await {
                let event = match value {
                    IndexerToProcess::EligibilityUpdate(peer, eligibility) => match eligibility {
                        PeerEligibility::Eligible => IndexerProcessed::Allow(peer),
                        PeerEligibility::Ineligible => {
                            if let Err(e) = network.remove(&peer).await {
                                error!("failed to remove '{peer}' from the local registry: {e}")
                            }
                            IndexerProcessed::Ban(peer)
                        }
                    },
                    IndexerToProcess::Announce(peer, multiaddress) => IndexerProcessed::Announce(peer, multiaddress),
                    // TODO: when is this even triggered? network registry missing?
                    IndexerToProcess::RegisterStatusUpdate => {
                        let peers = network
                            .peer_filter(|peer| async move { Some(peer.id.1) })
                            .await
                            .unwrap_or(vec![]);

                        for peer in peers.into_iter() {
                            let is_allowed = {
                                let address = {
                                    if let Ok(key) = OffchainPublicKey::try_from(peer) {
                                        match db_local.resolve_chain_key(&key).await.and_then(|maybe_address| {
                                            maybe_address.ok_or(hopr_db_api::errors::DbError::LogicalError(format!(
                                                "No address available for peer '{peer}'",
                                            )))
                                        }) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                error!("{e}");
                                                continue;
                                            }
                                        }
                                    } else {
                                        warn!("Could not convert the peer id '{peer}' to an offchain public key");
                                        continue;
                                    }
                                };

                                match db_local.is_allowed_in_network_registry(None, address).await {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                }
                            };

                            let event = if is_allowed {
                                IndexerProcessed::Allow(peer)
                            } else {
                                if let Err(e) = network.remove(&peer).await {
                                    error!("failed to remove '{peer}' from the local registry: {e}");
                                }
                                IndexerProcessed::Ban(peer)
                            };

                            match poll_fn(|cx| Pin::new(&mut emitter).poll_ready(cx)).await {
                                Ok(_) => match emitter.start_send(event) {
                                    Ok(_) => {}
                                    Err(e) => error!("Failed to emit an indexer event: {}", e),
                                },
                                Err(e) => {
                                    warn!("The receiver for processed indexer events no longer exists: {}", e);
                                }
                            };
                        }
                        continue;
                    }
                };

                match poll_fn(|cx| Pin::new(&mut emitter).poll_ready(cx)).await {
                    Ok(_) => match emitter.start_send(event) {
                        Ok(_) => {}
                        Err(e) => error!("Failed to emit an indexer event: {}", e),
                    },
                    Err(e) => {
                        warn!("The receiver for processed indexer events no longer exists: {}", e);
                    }
                };
            }
        });

        Self {
            internal_emitter: to_process_tx,
        }
    }
}

impl IndexerActions {
    pub async fn emit_indexer_update(&self, event: IndexerToProcess) {
        let mut internal_emitter = self.internal_emitter.clone();

        match poll_fn(|cx| Pin::new(&mut internal_emitter).poll_ready(cx)).await {
            Ok(_) => match internal_emitter.start_send(event) {
                Ok(_) => {}
                Err(e) => error!("Failed to send register update 'eligibility' to the receiver: {}", e),
            },
            Err(e) => error!("The receiver for indexer updates was dropped: {}", e),
        }
    }
}
