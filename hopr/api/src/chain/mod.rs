mod accounts;
mod channels;
mod events;
mod keys;
mod safe;
mod tickets;
mod values;

pub use accounts::*;
pub use channels::*;
pub use events::*;
pub use keys::*;
pub use safe::*;
pub use tickets::*;
pub use values::*;

/// Receipt of an on-chain operation.
pub type ChainReceipt = hopr_crypto_types::prelude::Hash;

/// Complete set of HOPR on-chain operation APIs.
///
/// This trait is automatically implemented for types
/// that implement all the individual chain API traits with the same error.
pub trait HoprChainApi:
    ChainReadAccountOperations<Error = Self::ChainError>
    + ChainWriteAccountOperations<Error = Self::ChainError>
    + ChainReadChannelOperations<Error = Self::ChainError>
    + ChainWriteChannelOperations<Error = Self::ChainError>
    + ChainReadSafeOperations<Error = Self::ChainError>
    + ChainWriteSafeOperations<Error = Self::ChainError>
    + ChainEvents<Error = Self::ChainError>
    + ChainKeyOperations<Error = Self::ChainError>
    + ChainValues<Error = Self::ChainError>
    + ChainWriteTicketOperations<Error = Self::ChainError>
{
    type ChainError: std::error::Error + Send + Sync + 'static;
}

impl<T, E> HoprChainApi for T
where
    T: ChainReadAccountOperations<Error = E>
        + ChainWriteAccountOperations<Error = E>
        + ChainReadChannelOperations<Error = E>
        + ChainWriteChannelOperations<Error = E>
        + ChainReadSafeOperations<Error = E>
        + ChainWriteSafeOperations<Error = E>
        + ChainEvents<Error = E>
        + ChainKeyOperations<Error = E>
        + ChainValues<Error = E>
        + ChainWriteTicketOperations<Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type ChainError = E;
}

/// Reduced set of read-only HOPR on-chain operation APIs.
///
/// Useful when only basic on-chain operations are required for querying.
///
/// This trait is automatically implemented for types
/// that implement all the individual chain API traits with the same error.
pub trait HoprStaticChainApi:
    ChainReadSafeOperations<Error = Self::ChainError> + ChainValues<Error = Self::ChainError>
{
    type ChainError: std::error::Error + Send + Sync + 'static;
}

impl<T, E> HoprStaticChainApi for T
where
    T: ChainReadSafeOperations<Error = E> + ChainValues<Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type ChainError = E;
}

/// [`PathAddressResolver`] which uses the [HOPR chain API](self) to resolve addresses and channels.
///
/// This type implements a `From` trait for all types that implement both
/// [`ChainKeyOperations`] and [`ChainReadChannelOperations`].
pub struct ChainPathResolver<'a, R>(&'a R);

impl<R> Clone for ChainPathResolver<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for ChainPathResolver<'_, R> {}

impl<'a, R: ChainKeyOperations + ChainReadChannelOperations> From<&'a R> for ChainPathResolver<'a, R> {
    fn from(value: &'a R) -> Self {
        Self(value)
    }
}

#[async_trait::async_trait]
impl<'c, R: ChainKeyOperations + ChainReadChannelOperations + Sync> hopr_internal_types::path::PathAddressResolver
    for ChainPathResolver<'c, R>
{
    async fn resolve_transport_address(
        &self,
        address: &hopr_primitive_types::prelude::Address,
    ) -> Result<Option<hopr_crypto_types::prelude::OffchainPublicKey>, hopr_internal_types::errors::PathError> {
        self.0
            .chain_key_to_packet_key(address)
            .await
            .map_err(|e| hopr_internal_types::errors::PathError::UnknownPeer(format!("{address}: {e}")))
    }

    async fn resolve_chain_address(
        &self,
        key: &hopr_crypto_types::prelude::OffchainPublicKey,
    ) -> Result<Option<hopr_primitive_types::prelude::Address>, hopr_internal_types::errors::PathError> {
        self.0
            .packet_key_to_chain_key(key)
            .await
            .map_err(|e| hopr_internal_types::errors::PathError::UnknownPeer(format!("{key}: {e}")))
    }

    async fn get_channel(
        &self,
        src: &hopr_primitive_types::prelude::Address,
        dst: &hopr_primitive_types::prelude::Address,
    ) -> Result<Option<ChannelEntry>, hopr_internal_types::errors::PathError> {
        self.0
            .channel_by_parties(src, dst)
            .await
            .map_err(|e| hopr_internal_types::errors::PathError::MissingChannel(src.to_string(), format!("{dst}: {e}")))
    }
}
