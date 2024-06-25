use core_path::path::TransportPath;
use libp2p_identity::PeerId;

use crate::SendOptions;

#[async_trait::async_trait]
pub trait PathResolve {
    async fn resolve(&mut self, destination: PeerId, options: SendOptions) -> crate::errors::Result<TransportPath>;
}
