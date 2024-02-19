use async_trait::async_trait;
use futures::stream::BoxStream;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;

use crate::errors::Result;
use crate::network::{PeerOrigin, PeerStatus};

pub struct Stats {
    pub good_quality: usize,
    pub bad_quality: usize,
}

pub use sea_query::SimpleExpr;

#[async_trait]
pub trait NetworkBackend {
    async fn add(&self, peer: &PeerId, origin: PeerOrigin, mas: Vec<Multiaddr>) -> Result<()>;

    async fn remove(&self, peer: &PeerId) -> Result<()>;

    async fn update(&self, peer: &PeerId, new_status: &PeerStatus) -> Result<()>;

    async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>>;

    // ? Can it be without the filter? Or what should the filter format be?
    async fn get_multiple<'a>(&'a self, filter: Option<SimpleExpr>) -> Result<BoxStream<'a, PeerStatus>>;

    async fn stats(&self) -> Result<Stats>;
}
