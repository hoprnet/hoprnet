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
pub trait HoprChainApi:
 ChainReadAccountOperations + ChainWriteAccountOperations +
 ChainReadChannelOperations + ChainWriteChannelOperations +
 ChainEvents + ChainKeyOperations + ChainValues +
 ChainWriteTicketOperations { }

impl<T> HoprChainApi for T where
 T: ChainReadAccountOperations + ChainWriteAccountOperations +
 ChainReadChannelOperations + ChainWriteChannelOperations +
 ChainEvents + ChainKeyOperations + ChainValues +
 ChainWriteTicketOperations
{ }
