use crate::{Address, Multiaddr};

/// Origin of a peer announcement — how the node learned about this peer.
///
/// Currently only on-chain announcements exist. When DHT-based discovery
/// is added, a `DHT` variant will surface peers without requiring an
/// on-chain multiaddress announcement.
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn announcement_origin_should_be_usable_as_hash_key() {
        let mut set = HashSet::new();
        set.insert(AnnouncementOrigin::Chain);
        set.insert(AnnouncementOrigin::DHT);
        set.insert(AnnouncementOrigin::Chain);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn announcement_origin_copy_should_preserve_value() {
        let origin = AnnouncementOrigin::Chain;
        let copied = origin;
        assert_eq!(origin, copied);
    }

    #[test]
    fn announced_peer_should_support_equality() {
        let addr = Address::default();
        let peer_a = AnnouncedPeer {
            address: addr,
            multiaddresses: vec![],
            origin: AnnouncementOrigin::Chain,
        };
        let peer_b = AnnouncedPeer {
            address: addr,
            multiaddresses: vec![],
            origin: AnnouncementOrigin::Chain,
        };
        assert_eq!(peer_a, peer_b);
    }

    #[test]
    fn announced_peers_with_different_origins_should_not_be_equal() {
        let addr = Address::default();
        let chain_peer = AnnouncedPeer {
            address: addr,
            multiaddresses: vec![],
            origin: AnnouncementOrigin::Chain,
        };
        let dht_peer = AnnouncedPeer {
            address: addr,
            multiaddresses: vec![],
            origin: AnnouncementOrigin::DHT,
        };
        assert_ne!(chain_peer, dht_peer);
    }

    #[test]
    fn announced_peer_clone_should_be_independent() {
        let addr = Address::default();
        let peer = AnnouncedPeer {
            address: addr,
            multiaddresses: vec!["/ip4/1.2.3.4/tcp/9091".parse().unwrap()],
            origin: AnnouncementOrigin::Chain,
        };
        let mut cloned = peer.clone();
        cloned.multiaddresses.push("/ip4/5.6.7.8/tcp/9092".parse().unwrap());

        assert_eq!(peer.multiaddresses.len(), 1);
        assert_eq!(cloned.multiaddresses.len(), 2);
    }
}
