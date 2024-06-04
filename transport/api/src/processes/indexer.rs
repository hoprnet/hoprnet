use std::sync::Arc;

use futures::{Stream, StreamExt};
use tracing::error;

use chain_types::chain_events::NetworkRegistryStatus;
use core_network::{network::Network, PeerId};

use hopr_db_sql::api::peers::HoprDbPeersOperations;
use hopr_transport_p2p::libp2p::swarm::derive_prelude::Multiaddr;

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
    network: Arc<Network<T>>,
) -> impl Stream<Item = PeerTransportEvent> + Send + 'static
where
    T: HoprDbPeersOperations + Send + Sync + 'static + std::fmt::Debug + Clone,
    S: Stream<Item = IndexerToProcess> + Send + 'static,
{
    Box::pin(
        src.filter_map(move |event| {
            let network = network.clone();

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
                }
            }
        })
        .flat_map(futures::stream::iter),
    )
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
