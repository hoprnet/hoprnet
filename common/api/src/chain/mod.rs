mod accounts;
mod channels;
mod events;
mod keys;
mod tickets;
mod values;

pub use accounts::*;
pub use channels::*;
pub use events::*;
pub use keys::*;
pub use tickets::*;
pub use values::*;

/// Receipt of an on-chain operation.
pub type ChainReceipt = hopr_crypto_types::prelude::Hash;

/// Complete set of HOPR on-chain operation APIs.
///
/// This trait is automatically implemented for types
/// that implement all the individual chain API traits to be implemented with the same error.
pub trait HoprChainApi:
    ChainReadAccountOperations<Error = Self::ChainError>
    + ChainWriteAccountOperations<Error = Self::ChainError>
    + ChainReadChannelOperations<Error = Self::ChainError>
    + ChainWriteChannelOperations<Error = Self::ChainError>
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
        + ChainEvents<Error = E>
        + ChainKeyOperations<Error = E>
        + ChainValues<Error = E>
        + ChainWriteTicketOperations<Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type ChainError = E;
}
