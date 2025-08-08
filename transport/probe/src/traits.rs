use async_trait::async_trait;
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
    /// Attempts to find SURB and its ID given the [`SurbMatcher`](hopr_network_types::types::SurbMatcher).
    async fn find_surb(
        &self,
        matcher: hopr_network_types::types::SurbMatcher,
    ) -> hopr_db_api::errors::Result<(hopr_db_api::protocol::HoprSenderId, hopr_db_api::protocol::HoprSurb)>;

    /// Tries to resolve on-chain public key given the off-chain public key
    async fn resolve_chain_key(
        &self,
        offchain_key: &hopr_crypto_types::types::OffchainPublicKey,
    ) -> hopr_db_api::errors::Result<Option<hopr_primitive_types::prelude::Address>>;
}
