use std::sync::Arc;

use futures::{Stream, StreamExt};
use tracing::{error, warn};

use chain_types::chain_events::NetworkRegistryStatus;
use core_network::{network::Network, PeerId};
use core_p2p::libp2p::swarm::derive_prelude::Multiaddr;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_api::{
    peers::HoprDbPeersOperations, registry::HoprDbRegistryOperations, resolver::HoprDbResolverOperations,
};

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
pub enum PeerTransportEvent {
    Allow(PeerId),
    Ban(PeerId),
    Announce(PeerId, Vec<Multiaddr>),
}

pub async fn add_peer_update_processing<S, T>(
    src: S,
    db: T,
    network: Arc<Network<T>>,
) -> impl Stream<Item = PeerTransportEvent> + Send + 'static
where
    T: HoprDbPeersOperations
        + HoprDbResolverOperations
        + HoprDbRegistryOperations
        + Send
        + Sync
        + 'static
        + std::fmt::Debug
        + Clone,
    S: Stream<Item = IndexerToProcess> + Send + 'static,
{
    let out_stream = src
        .filter_map(move |event| {
            let network = network.clone();
            let db = db.clone();

            async move {
                match event {
                    IndexerToProcess::EligibilityUpdate(peer, eligibility) => match eligibility {
                        PeerEligibility::Eligible => Some(vec![PeerTransportEvent::Allow(peer)]),
                        PeerEligibility::Ineligible => {
                            if let Err(e) = network.remove(&peer).await {
                                error!("failed to remove '{peer}' from the local registry: {e}")
                            }
                            Some(vec![PeerTransportEvent::Ban(peer)])
                        }
                    },
                    IndexerToProcess::Announce(peer, multiaddress) => {
                        Some(vec![PeerTransportEvent::Announce(peer, multiaddress)])
                    }
                    // TODO: when is this even triggered? network registry missing?
                    IndexerToProcess::RegisterStatusUpdate => Some(
                        futures::stream::iter(
                            network
                                .peer_filter(|peer| async move { Some(peer.id.1) })
                                .await
                                .unwrap_or(vec![]),
                        )
                        .filter_map(move |peer| {
                            let network = network.clone();
                            let db = db.clone();

                            async move {
                            if let Ok(key) = OffchainPublicKey::try_from(peer) {
                                match db.resolve_chain_key(&key).await.and_then(|maybe_address| {
                                    maybe_address.ok_or(hopr_db_api::errors::DbError::LogicalError(format!(
                                        "No address available for peer '{peer}'",
                                    )))
                                }) {
                                    Ok(address) => match db.is_allowed_in_network_registry(None, address).await {
                                        Ok(is_allowed) => {
                                            if is_allowed {
                                                Some(PeerTransportEvent::Allow(peer))
                                            } else {
                                                if let Err(e) = network.remove(&peer).await {
                                                    error!("failed to remove '{peer}' from the local registry: {e}");
                                                }
                                                Some(PeerTransportEvent::Ban(peer))
                                            }
                                        }
                                        Err(_) => None,
                                    },
                                    Err(e) => {
                                        error!("{e}");
                                        None
                                    }
                                }
                            } else {
                                warn!("Could not convert the peer id '{peer}' to an offchain public key");
                                None
                            }
                        }
                })
                        .collect::<Vec<_>>()
                        .await,
                    ),
                }
            }
        })
        .flat_map(futures::stream::iter);

    Box::pin(out_stream)
}

/// Implementor interface for indexer actions
#[derive(Debug, Clone)]
pub struct IndexerActions {
    pub(crate) internal_emitter: async_channel::Sender<IndexerToProcess>,
}

impl IndexerActions {
    pub const INDEXER_UPDATE_QUEUE_SIZE: usize = 4096;

    pub fn new(emitter: async_channel::Sender<IndexerToProcess>) -> Self {
        Self {
            internal_emitter: emitter,
        }
    }
}

impl IndexerActions {
    pub async fn emit_indexer_update(&self, event: IndexerToProcess) {
        match self.internal_emitter.clone().send(event).await {
            Ok(_) => {}
            Err(e) => error!("Failed to send index update event to transport: {e}"),
        }
    }
}
