use hopr_api::{Multiaddr, PeerId};

use crate::errors::{NetworkError, Result};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_NETWORK_HEALTH:  hopr_metrics::SimpleGauge =
         hopr_metrics::SimpleGauge::new("hopr_network_health", "Connectivity health indicator").unwrap();
    static ref METRIC_PEER_COUNT:  hopr_metrics::SimpleGauge =
         hopr_metrics::SimpleGauge::new("hopr_peer_count", "Number of all peers").unwrap();
}

pub struct NetworkPeerStore {
    me: PeerId,
    my_addresses: Vec<Multiaddr>,
    addresses: dashmap::DashMap<PeerId, Vec<Multiaddr>>,
}

impl NetworkPeerStore {
    pub fn new(me: PeerId, my_addresses: Vec<Multiaddr>) -> Self {
        Self {
            me,
            my_addresses,
            addresses: dashmap::DashMap::new(),
        }
    }

    /// Check whether the PeerId is present in the network.
    #[inline]
    #[tracing::instrument(level = "trace", skip(self), ret(Display))]
    pub fn has(&self, peer: &PeerId) -> bool {
        peer == &self.me || self.addresses.contains_key(peer)
    }

    /// Add a new peer into the network.
    #[tracing::instrument(level = "debug", skip(self), ret(level = "trace"), err)]
    pub fn add(&self, peer: PeerId, mut addresses: Vec<Multiaddr>) -> Result<()> {
        if peer == self.me {
            return Err(NetworkError::DisallowedOperationOnOwnPeerIdError);
        }

        if let Some(mut unit) = self.addresses.get_mut(&peer) {
            unit.value_mut().append(&mut addresses);
        } else {
            self.addresses.insert(peer, addresses);
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_PEER_COUNT.increment(1.0);

        Ok(())
    }

    /// Get peer multiaddress.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get(&self, peer: &PeerId) -> Option<Vec<Multiaddr>> {
        if peer == &self.me {
            Some(self.my_addresses.clone())
        } else {
            self.addresses.get(peer).map(|addrs| addrs.value().clone())
        }
    }

    /// Remove peer from the network
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove(&self, peer: &PeerId) -> Result<()> {
        if peer == &self.me {
            return Err(NetworkError::DisallowedOperationOnOwnPeerIdError);
        }

        self.addresses.remove(peer);

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_PEER_COUNT.decrement(1.0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkPeerStore;
    use anyhow::Context;
    use hopr_api::PeerId;

    #[test]
    fn network_peer_store_should_recognize_self() {
        let me = PeerId::random();
        let store = NetworkPeerStore::new(me.clone(), vec![]);

        assert!(store.has(&me));
    }

    #[test]
    fn network_peer_store_should_add_and_recognize_peer() {
        let peer = PeerId::random();
        let store = NetworkPeerStore::new(PeerId::random(), vec![]);

        assert!(!store.has(&peer));

        assert!(store.add(peer, vec![]).is_ok());

        assert!(store.has(&peer));
    }

    #[test]
    fn network_peer_store_own_peer_should_fail_on_adding() {
        let me = PeerId::random();
        let store = NetworkPeerStore::new(me.clone(), vec![]);

        assert!(store.add(me.clone(), vec![]).is_err());
    }

    #[test]
    fn network_peer_store_adding_the_same_peer_should_extend_multiaddresses() -> anyhow::Result<()> {
        let store = NetworkPeerStore::new(PeerId::random(), vec![]);

        let peer = PeerId::random();
        assert!(store.add(peer.clone(), vec![]).is_ok());

        assert_eq!(store.get(&peer).context("should contain a value")?, vec![]);

        let multiaddresses = vec!["/ip4/127.0.0.1/tcp/12345".try_into()?];
        assert!(store.add(peer.clone(), multiaddresses.clone()).is_ok());

        assert_eq!(store.get(&peer).context("should contain a value")?, multiaddresses);

        Ok(())
    }

    #[test]
    fn network_peer_store_should_return_own_multiaddresses() -> anyhow::Result<()> {
        let me = PeerId::random();
        let multiaddresses = vec!["/ip4/127.0.0.1/tcp/12345".try_into()?];
        let store = NetworkPeerStore::new(me.clone(), multiaddresses.clone());

        assert_eq!(store.get(&me), Some(multiaddresses));

        Ok(())
    }

    #[test]
    fn network_peer_store_should_return_stored_multiaddress() -> anyhow::Result<()> {
        let me = PeerId::random();
        let store = NetworkPeerStore::new(me.clone(), vec![]);

        let peer = PeerId::random();
        let multiaddresses = vec!["/ip4/127.0.0.1/tcp/12345".try_into()?];
        assert!(store.add(peer.clone(), multiaddresses.clone()).is_ok());

        assert_eq!(store.get(&peer), Some(multiaddresses));

        Ok(())
    }

    #[test]
    fn network_peer_store_should_fail_on_removing_self() -> anyhow::Result<()> {
        let me = PeerId::random();
        let multiaddresses = vec!["/ip4/127.0.0.1/tcp/12345".try_into()?];
        let store = NetworkPeerStore::new(me.clone(), multiaddresses.clone());

        assert!(store.remove(&me).is_err());

        Ok(())
    }

    #[test]
    fn network_peer_store_should_succeed_on_removing_a_known_peer() -> anyhow::Result<()> {
        let me = PeerId::random();
        let store = NetworkPeerStore::new(me.clone(), vec![]);

        let peer = PeerId::random();
        let multiaddresses = vec!["/ip4/127.0.0.1/tcp/12345".try_into()?];
        assert!(store.add(peer.clone(), multiaddresses.clone()).is_ok());

        store.remove(&peer)?;

        Ok(())
    }

    #[test]
    fn network_peer_store_should_succeed_on_removing_an_unknown_peer() -> anyhow::Result<()> {
        let me = PeerId::random();
        let store = NetworkPeerStore::new(me.clone(), vec![]);

        let peer = PeerId::random();

        store.remove(&peer)?;

        Ok(())
    }
}
