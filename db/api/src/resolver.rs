use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_primitive_types::primitives::Address;

use crate::errors::Result;

/// Trait for linking and resolving the corresponding `OffchainPublicKey` and on-chain `Address`.
#[async_trait]
pub trait HoprDbResolverOperations {
    /// Tries to resolve off-chain public key given the on-chain address
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Result<Option<OffchainPublicKey>>;

    /// Tries to resolve on-chain public key given the off-chain public key
    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Result<Option<Address>>;
}
