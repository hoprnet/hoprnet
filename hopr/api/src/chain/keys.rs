use std::error::Error;

pub use hopr_types::primitive::prelude::{KeyIdMapping, KeyIdent as HoprKeyIdent};
use hopr_types::{crypto::prelude::OffchainPublicKey, primitive::prelude::Address};

/// Operations for offchain keys.
///
/// This typically translates to optimized (and cached) versions of [`ChainReadChannelOperations`].
///
/// The `Id` is a unique identifier for offchain public keys.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainKeyOperations {
    type Error: Error + Send + Sync + 'static;
    /// [Mapping](KeyIdMapping) between [`KeyIdent`] and [`OffchainPublicKey`]
    type Mapper: KeyIdMapping<HoprKeyIdent, OffchainPublicKey> + Clone + Send + Sync + 'static;
    /// Translates [`Address`] into [`OffchainPublicKey`].
    async fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error>;
    /// Translates [`OffchainPublicKey`] into [`Address`].
    async fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error>;
    /// Returns [mapper](KeyIdMapper) for offchain key IDs as a reference.
    fn key_id_mapper_ref(&self) -> &Self::Mapper;
}
