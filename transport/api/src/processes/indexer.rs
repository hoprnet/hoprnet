use tracing::error;

use chain_types::chain_events::NetworkRegistryStatus;
use core_network::PeerId;

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
pub enum IndexerTransportEvent {
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

/// Implementor interface for indexer actions
#[derive(Debug, Clone)]
pub struct IndexerActions {
    pub(crate) internal_emitter: async_channel::Sender<IndexerTransportEvent>,
}

impl IndexerActions {
    pub const INDEXER_UPDATE_QUEUE_SIZE: usize = 4096;

    pub fn new(emitter: async_channel::Sender<IndexerTransportEvent>) -> Self {
        Self {
            internal_emitter: emitter,
        }
    }
}

impl IndexerActions {
    pub async fn emit_indexer_update(&self, event: IndexerTransportEvent) {
        match self.internal_emitter.clone().send(event).await {
            Ok(_) => {}
            Err(e) => error!("Failed to send index update event to transport: {e}"),
        }
    }
}
