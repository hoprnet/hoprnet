use async_trait::async_trait;
use hopr_api::db::FoundSurb;
use hopr_transport_protocol::FoundSurb;
use libp2p_identity::PeerId;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PeerDiscoveryFetch {
    /// Get untested peers not observed since a specific timestamp.
    async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<PeerId>;
}

#[async_trait]
pub trait ProbeStatusUpdate {
    /// Update the peer status after probing
    async fn on_finished(&self, peer: &PeerId, result: &crate::errors::Result<std::time::Duration>);
}

/// A common interface for wrapping caching operations needed by the probing mechanism.
///
/// This trait should eventually disappear as parts of this functionality move closer
/// to the network layer.
#[async_trait]
pub trait DbOperations {
    type ChainError: std::error::Error + Send + Sync + 'static;

    /// Attempts to find SURB and its ID given the [`SurbMatcher`](hopr_network_types::types::SurbMatcher).
    async fn find_surb(&self, matcher: hopr_network_types::types::SurbMatcher) -> Option<FoundSurb>;

    /// Tries to resolve on-chain public key given the off-chain public key
    async fn resolve_chain_key(
        &self,
        offchain_key: &hopr_crypto_types::types::OffchainPublicKey,
    ) -> Result<Option<hopr_primitive_types::prelude::Address>, Self::ChainError>;
}
