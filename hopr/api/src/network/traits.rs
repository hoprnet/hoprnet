use std::collections::HashSet;

use futures::{AsyncRead, AsyncWrite, Stream, future::BoxFuture};
use hopr_crypto_types::keypairs::OffchainKeypair;

use super::Health;
use crate::{Multiaddr, PeerId};

/// Type alias for a boxed function returning a boxed future.
pub type BoxedProcessFn = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>;

/// Trait representing a read-only view of the network state.
pub trait NetworkView {
    /// Multiaddresses used for listening by the local node.
    fn listening_as(&self) -> HashSet<Multiaddr>;

    /// Translation of the peer into its known multiaddresses.
    fn multiaddress_of(&self, peer: &PeerId) -> Option<HashSet<Multiaddr>>;

    /// Peers collected by the network discovery mechanism.
    fn discovered_peers(&self) -> HashSet<PeerId>;

    /// Peers currently connected and tracked by the network.
    fn connected_peers(&self) -> HashSet<PeerId>;

    /// Peers currently connected and tracked by the network.
    fn is_connected(&self, peer: &PeerId) -> bool;

    /// Represents perceived health of the network.
    fn health(&self) -> Health;
}

/// Control object for the opening and receiving of network connections in the
/// form of network streams.
#[async_trait::async_trait]
pub trait NetworkStreamControl: std::fmt::Debug {
    fn accept(
        self,
    ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error>;

    async fn open(self, peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error>;
}

/// Builder for a network object that returns the network specified in
/// the `NetworkBuilder::Network` associated type.
#[async_trait::async_trait]
pub trait NetworkBuilder {
    type Network: NetworkView + NetworkStreamControl + Send + Sync + Clone + 'static;

    /// Build and return an instance of a network viewer along with a process to start
    /// and run the network component.
    async fn build(
        self,
        identity: &OffchainKeypair,
        my_multiaddresses: Vec<Multiaddr>,
        protocol: &'static str,
        allow_private_addresses: bool,
    ) -> Result<(Self::Network, BoxedProcessFn), impl std::error::Error>;
}
