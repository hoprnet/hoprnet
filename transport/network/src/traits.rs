use futures::Stream;
use hopr_api::{
    db::PeerStatus,
    {Multiaddr, PeerId},
};

use crate::{Health, errors::Result};

#[async_trait::async_trait]
pub trait NetworkView {
    fn listening_as(&self) -> Vec<Multiaddr>; //local_multiaddresses

    fn health(&self) -> Health; // network_health

    async fn multiaddress_of(&self, peer: &PeerId) -> Vec<Multiaddr>; //network_observed_multiaddresses

    async fn peers(&self) -> Result<Vec<PeerId>>; //network_connected_peers

    async fn network_peer_info(&self, peer: &PeerId) -> Result<Option<PeerStatus>>; // TODO: replace with the filtered version of PeerStatus
}

/// Events that can occur in the network
pub enum NetworkEvent {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
}

/// Reader of events corresponding to network changes
#[async_trait::async_trait]
pub trait NetworkEventReader {
    fn subscribe(&self) -> impl Stream<Item = NetworkEvent> + Send;
}

/// Writer for network observations used to improve the internal observations by the Network
pub trait NetworkObservationWriter {
    fn update(&self, peer: &PeerId, result: std::result::Result<std::time::Duration, ()>);
}
