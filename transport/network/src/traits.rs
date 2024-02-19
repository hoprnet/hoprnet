use async_trait::async_trait;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;

use crate::network::{PeerOrigin, PeerStatus};
use crate::errors::Result;

pub struct Stats {
    pub good_quality: usize,
    pub bad_quality: usize,
}

#[async_trait]
pub trait NetworkBackend {
    async fn add(&self, peer: &PeerId, origin: PeerOrigin, mas: Vec<Multiaddr>) -> Result<()>;

    async fn remove(&self, peer: &PeerId) -> Result<()>;

    async fn update(&self, peer: &PeerId, new_status: &PeerStatus) -> Result<()>;

    async fn get(&self, peer: &PeerId) -> Result<PeerStatus>;

    // ? Can it be without the filter? Or what should the filter format be?
    async fn get_multiple<F: FnOnce() -> T + Send + Sync, T: Send + Sync>(&self, filter: F) -> Result<Vec<T>>;

    async fn stats(&self) -> Result<Stats>;
}