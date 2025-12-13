use std::collections::HashSet;

use hopr_api::{Multiaddr, PeerId};

use crate::{Health, track::Observations};

/// Trait representing a read-only view of the network state.
pub trait NetworkView {
    /// Multiaddresses used for listening by the local node.
    fn listening_as(&self) -> HashSet<Multiaddr>; //local_multiaddresses

    /// Translation of the peer into its known multiaddresses.
    fn multiaddress_of(&self, peer: &PeerId) -> Option<HashSet<Multiaddr>>;

    /// Peers collected by the network discovery mechanism.
    fn discovered_peers(&self) -> HashSet<PeerId>;

    /// Peers currently connected and tracked by the network.
    fn connected_peers(&self) -> HashSet<PeerId>;

    /// Observations related to a specific peer in the network.
    ///
    /// The absence of observations means that the peer is currently not connected
    /// to the network and therefore has no observations.
    fn observations_for(&self, peer: &PeerId) -> Option<Observations>;

    /// Represents perceived health of the network.
    fn health(&self) -> Health;
}

/// Trait representing a reporter of network immediate peer testing observations.
pub trait NetworkObservations {
    fn update(&self, peer: &PeerId, result: std::result::Result<std::time::Duration, ()>);
}
