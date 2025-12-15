use std::collections::HashSet;

use super::{Health, Observable};
use crate::{Multiaddr, PeerId};

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

    /// Observables related to a specific peer in the network.
    ///
    /// In the absence of Observables
    fn observations_for<'a>(&'a self, peer: &'a PeerId) -> Option<impl Observable + 'static>;

    /// Represents perceived health of the network.
    fn health(&self) -> Health;
}

/// Trait representing a reporter of network immediate peer testing observations.
pub trait NetworkObservations {
    fn update(&self, peer: &PeerId, result: std::result::Result<std::time::Duration, ()>);
}
