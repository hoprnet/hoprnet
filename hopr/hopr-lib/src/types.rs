use hopr_api::{Multiaddr, types::primitive::prelude::Address};

/// Origin of a peer announcement — how the node learned about this peer.
///
/// Currently only on-chain announcements exist. When DHT-based discovery
/// is added, a `DHT` variant will surface peers without requiring an
/// on-chain transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnnouncementOrigin {
    /// Announced via on-chain registration.
    Chain,
    /// Discovered via DHT (future).
    DHT,
}

/// A peer that has been announced and discovered by the node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnouncedPeer {
    /// On-chain address of the peer.
    pub address: Address,
    /// Multiaddresses associated with this peer.
    pub multiaddresses: Vec<Multiaddr>,
    /// How the announcement was discovered.
    pub origin: AnnouncementOrigin,
}
