use async_trait::async_trait;
use futures::stream::BoxStream;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
pub use sea_query::SimpleExpr;

use hopr_db_api::peers::{PeerOrigin, PeerStatus, Stats};

use crate::errors::Result;

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
    /// The `sort_last_seen_asc` indicates whether the results should be sorted in ascending
    /// or descending order of the `last_seen` field.
    async fn get_multiple<'a>(
        &'a self,
        filter: Option<SimpleExpr>,
        sort_last_seen_asc: bool,
    ) -> Result<BoxStream<'a, PeerStatus>>;

    /// Returns the [statistics](Stats) on the stored peers.
    async fn stats(&self) -> Result<Stats>;
}
