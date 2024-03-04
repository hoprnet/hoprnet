use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_primitive_types::primitives::Address;

use crate::db::HoprDb;

/// Trait for linking and resolving the corresponding `OffchainPublicKey` and on-chain `Address`.
#[async_trait]
pub trait PeerAddressResolver {
    /// Tries to resolve off-chain public key given the on-chain address
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey>;

    /// Tries to resolve on-chain public key given the off-chain public key
    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address>;
}

#[async_trait]
impl PeerAddressResolver for HoprDb {
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
        None
    }

    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
        None
    }
}
