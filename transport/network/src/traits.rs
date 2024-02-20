use async_trait::async_trait;
use futures::stream::BoxStream;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;

use crate::errors::Result;
use crate::network::{PeerOrigin, PeerStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    pub good_quality_public: u32,
    pub bad_quality_public: u32,
    pub good_quality_non_public: u32,
    pub bad_quality_non_public: u32,
}

#[cfg(all(feature = "prometheus", not(test)))]
impl Stats {
    pub fn all_count(&self) -> usize {
        self.good_quality_public as usize
            + self.bad_quality_public as usize
            + self.good_quality_non_public as usize
            + self.bad_quality_non_public as usize
    }
}

pub use sea_query::SimpleExpr;

#[async_trait]
pub trait NetworkBackend {
    async fn add(&self, peer: &PeerId, origin: PeerOrigin, mas: Vec<Multiaddr>) -> Result<()>;

    async fn remove(&self, peer: &PeerId) -> Result<()>;

    async fn update(&self, new_status: &PeerStatus) -> Result<()>;

    async fn get(&self, peer: &PeerId) -> Result<Option<PeerStatus>>;

    // ? Can it be without the filter? Or what should the filter format be?
    async fn get_multiple<'a>(&'a self, filter: Option<SimpleExpr>) -> Result<BoxStream<'a, PeerStatus>>;

    async fn stats(&self) -> Result<Stats>;
}
