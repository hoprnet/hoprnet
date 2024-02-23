use hopr_internal_types::protocol::PeerAddressResolver;
use std::time::Duration;
use moka::future::{Cache, CacheBuilder};
use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_primitive_types::primitives::Address;

/// Implementation of [PeerAddressResolver] that wraps another resolver with added caching.
#[derive(Debug, Clone)]
pub struct CachedPeerAddressResolver<R: PeerAddressResolver + Clone + Send + Sync> {
    inner: R,
    // Cloning of caches is cheap, as it is represented internally as Arc
    address_to_key: Cache<Address, OffchainPublicKey>,
    key_to_address: Cache<OffchainPublicKey, Address>,
}

impl<R: PeerAddressResolver + Clone + Send + Sync> CachedPeerAddressResolver<R> {
    /// Instantiates a new peer address resolver with the given cache size.
    pub fn new(inner: R, cache_size: usize, ttl: Duration) -> Self {
        Self {
            inner,
            // Expiration policies can be optionally set here by constructing via CacheBuilder
            address_to_key: CacheBuilder::new(cache_size as u64).time_to_live(ttl).build(),
            key_to_address: CacheBuilder::new(cache_size as u64).time_to_live(ttl).build(),
        }
    }
}

#[async_trait]
impl<R: PeerAddressResolver + Clone + Send + Sync> PeerAddressResolver for CachedPeerAddressResolver<R> {
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
        self.address_to_key
            .optionally_get_with_by_ref(onchain_key, self.inner.resolve_packet_key(onchain_key))
            .await
    }

    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
        self.key_to_address
            .optionally_get_with_by_ref(offchain_key, self.inner.resolve_chain_key(offchain_key))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use hex_literal::hex;
    use mockall::mock;
    use hopr_crypto_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    mock! {
        AddrResolver{}

        #[async_trait]
        impl PeerAddressResolver for AddrResolver {
            async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey>;
            async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address>;
        }

        impl Clone for AddrResolver {
            fn clone(&self) -> Self;
        }
    }

    #[async_std::test]
    async fn test_caching_resolver() {
        let pk_1 = OffchainKeypair::random().public().clone();
        let pk_2: Address = hex!("60f8492b6fbaf86ac2b064c90283d8978a491a01").into();

        let mut resolver = MockAddrResolver::new();
        resolver
            .expect_resolve_chain_key()
            .once()
            .withf(move |pk| pk_1.eq(pk))
            .return_once(move |_| Some(pk_2));

        resolver
            .expect_resolve_packet_key()
            .once()
            .withf(move |pk| pk_2.eq(pk))
            .return_once(move |_| Some(pk_1));

        let caching = CachedPeerAddressResolver::new(resolver, 10, Duration::from_secs(30));

        assert_eq!(Some(pk_2), caching.resolve_chain_key(&pk_1).await);
        assert_eq!(Some(pk_2), caching.resolve_chain_key(&pk_1).await);

        assert_eq!(Some(pk_1), caching.resolve_packet_key(&pk_2).await);
        assert_eq!(Some(pk_1), caching.resolve_packet_key(&pk_2).await);
    }
}