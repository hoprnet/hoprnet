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
