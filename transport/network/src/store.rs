use std::{collections::HashSet, sync::Arc};

use hopr_api::{Multiaddr, PeerId};

use crate::errors::{NetworkError, Result};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PEER_COUNT:  hopr_metrics::SimpleGauge =
         hopr_metrics::SimpleGauge::new("hopr_peer_count", "Number of all peers").unwrap();
}

#[derive(Clone, Debug)]
pub struct NetworkPeerStore {
    me: PeerId,
    my_addresses: HashSet<Multiaddr>,
    addresses: Arc<dashmap::DashMap<PeerId, HashSet<Multiaddr>>>,
}

impl NetworkPeerStore {
    pub fn new(me: PeerId, my_addresses: HashSet<Multiaddr>) -> Self {
        Self {
            me,
            my_addresses,
            addresses: Arc::new(dashmap::DashMap::new()),
        }
    }

    #[inline]
    pub fn me(&self) -> &PeerId {
        &self.me
    }

    /// Check whether the PeerId is present in the network.
    #[inline]
    #[tracing::instrument(level = "trace", skip(self), ret(Display))]
    pub fn has(&self, peer: &PeerId) -> bool {
        peer == &self.me || self.addresses.contains_key(peer)
    }

    /// Add a new peer into the network.
    #[tracing::instrument(level = "debug", skip(self), ret(level = "trace"), err)]
    pub fn add(&self, peer: PeerId, addresses: HashSet<Multiaddr>) -> Result<()> {
        if peer == self.me {
            return Err(NetworkError::DisallowedOperationOnOwnPeerIdError);
        }

        if let Some(mut unit) = self.addresses.get_mut(&peer) {
            unit.value_mut().extend(addresses.into_iter());
        } else {
            self.addresses.insert(peer, addresses);

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_PEER_COUNT.increment(1.0);
        }

        Ok(())
    }

    /// Get peer multiaddress.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get(&self, peer: &PeerId) -> Option<HashSet<Multiaddr>> {
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

        if self.addresses.remove(peer).is_some() {
            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_PEER_COUNT.decrement(1.0);
        }

        Ok(())
    }

    #[inline]
    pub fn iter_keys(&self) -> impl Iterator<Item = PeerId> + '_ {
        self.addresses.iter().map(|entry| *entry.key())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use anyhow::Context;
    use hopr_api::PeerId;

    use super::NetworkPeerStore;

    #[test]
    fn network_peer_store_should_recognize_self() {
        let me = PeerId::random();
        let store = NetworkPeerStore::new(me.clone(), HashSet::new());

        assert!(store.has(&me));
    }

    #[test]
    fn network_peer_store_should_add_and_recognize_peer() {
        let store = NetworkPeerStore::new(PeerId::random(), HashSet::new());

        let peer = PeerId::random();

        assert!(!store.has(&peer));

        assert!(store.add(peer, HashSet::new()).is_ok());

        assert!(store.has(&peer));
    }

    #[test]
    fn network_peer_store_own_peer_should_fail_on_adding() {
        let me = PeerId::random();
        let store = NetworkPeerStore::new(me.clone(), HashSet::new());

        assert!(store.add(me, HashSet::new()).is_err());
    }

    #[test]
    fn network_peer_store_adding_the_same_peer_should_extend_multiaddresses() -> anyhow::Result<()> {
        let store = NetworkPeerStore::new(PeerId::random(), HashSet::new());

        let peer = PeerId::random();
        assert!(store.add(peer.clone(), HashSet::new()).is_ok());

        assert_eq!(store.get(&peer).context("should contain a value")?, HashSet::new());

        let multiaddresses = HashSet::from(["/ip4/127.0.0.1/tcp/12345".try_into()?]);
        assert!(store.add(peer.clone(), multiaddresses.clone()).is_ok());

        assert_eq!(store.get(&peer).context("should contain a value")?, multiaddresses);

        Ok(())
    }

    #[test]
    fn network_peer_store_should_return_own_multiaddresses() -> anyhow::Result<()> {
        let me = PeerId::random();
        let multiaddresses = HashSet::from(["/ip4/127.0.0.1/tcp/12345".try_into()?]);
        let store = NetworkPeerStore::new(me.clone(), multiaddresses.clone());

        assert_eq!(store.get(&me), Some(multiaddresses));

        Ok(())
    }

    #[test]
    fn network_peer_store_should_return_stored_multiaddress() -> anyhow::Result<()> {
        let store = NetworkPeerStore::new(PeerId::random(), HashSet::new());

        let peer = PeerId::random();
        let multiaddresses = HashSet::from(["/ip4/127.0.0.1/tcp/12345".try_into()?]);
        assert!(store.add(peer.clone(), multiaddresses.clone()).is_ok());

        assert_eq!(store.get(&peer), Some(multiaddresses));

        Ok(())
    }

    #[test]
    fn network_peer_store_should_fail_on_removing_self() -> anyhow::Result<()> {
        let me = PeerId::random();
        let multiaddresses = HashSet::from(["/ip4/127.0.0.1/tcp/12345".try_into()?]);
        let store = NetworkPeerStore::new(me.clone(), multiaddresses.clone());

        assert!(store.remove(&me).is_err());

        Ok(())
    }

    #[test]
    fn network_peer_store_should_succeed_on_removing_a_known_peer() -> anyhow::Result<()> {
        let store = NetworkPeerStore::new(PeerId::random(), HashSet::new());

        let peer = PeerId::random();
        let multiaddresses = HashSet::from(["/ip4/127.0.0.1/tcp/12345".try_into()?]);
        assert!(store.add(peer.clone(), multiaddresses.clone()).is_ok());

        store.remove(&peer)?;

        Ok(())
    }

    #[test]
    fn network_peer_store_should_succeed_on_removing_an_unknown_peer() -> anyhow::Result<()> {
        let store = NetworkPeerStore::new(PeerId::random(), HashSet::new());

        let peer = PeerId::random();

        store.remove(&peer)?;

        Ok(())
    }

    #[test]
    fn network_peer_store_should_be_referencing_the_same_underlying_data() -> anyhow::Result<()> {
        let store = NetworkPeerStore::new(PeerId::random(), HashSet::new());
        let store2 = store.clone();

        let peer = PeerId::random();

        store2.add(peer, HashSet::new())?;

        assert_eq!(store.get(&peer).context("should contain a value")?, HashSet::new());

        Ok(())
    }
}
