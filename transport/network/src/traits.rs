use async_trait::async_trait;
use futures::stream::BoxStream;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;

use crate::errors::Result;
use crate::network::{PeerOrigin, PeerStatus};

/// Object containing statistics on all peer entries stored
/// in the [crate::network::Network] object.
/// See [crate::network::NetworkConfig] on information about the quality thresholds
/// (i.e. when is a peer considered of good/bad quality).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    /// Number of good quality public nodes.
    pub good_quality_public: u32,
    /// Number of bad quality public nodes.
    pub bad_quality_public: u32,
    /// Number of good quality nodes non-public nodes.
    pub good_quality_non_public: u32,
    /// Number of bad quality nodes non-public nodes.
    pub bad_quality_non_public: u32,
}

#[cfg(all(feature = "prometheus", not(test)))]
impl Stats {
    /// Returns count of all peers.
    pub fn all_count(&self) -> usize {
        self.good_quality_public as usize
            + self.bad_quality_public as usize
            + self.good_quality_non_public as usize
            + self.bad_quality_non_public as usize
    }
}

pub use sea_query::SimpleExpr;

/// An abstraction over a backend that stores the peer information.
#[async_trait]
pub trait NetworkBackend {
    /// Adds a peer to the backend.
    /// Should fail if the given peer id already exists in the store.
    async fn add(&self, peer: &PeerId, origin: PeerOrigin, mas: Vec<Multiaddr>) -> Result<()>;

    /// Removes the peer from the backend.
    /// Should fail if the given peer id does not exist.
    async fn remove(&self, peer: &PeerId) -> Result<()>;

    /// Updates stored information about the peer.
    /// Should fail if the peer does not exist in the store.
    async fn update(&self, new_status: &PeerStatus) -> Result<()>;

    /// Gets stored information about the peer.
    /// Should return `None` if such peer does not exist in the store.
    async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>>;

    /// Returns a stream of all stored peers, optionally matching the given [SimpleExpr] filter.
    async fn get_multiple<'a>(&'a self, filter: Option<SimpleExpr>) -> Result<BoxStream<'a, PeerStatus>>;

    /// Returns the [statistics](Stats) on the stored peers.
    async fn stats(&self) -> Result<Stats>;
}
