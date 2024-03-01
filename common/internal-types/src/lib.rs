//! This crate contains all types that are specific to the HOPR protocol.
//! As opposed to `hopr-primitive-types` which contains more generic types (not necessarily specific only to HOPR).

use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_primitive_types::errors::GeneralError;
use hopr_primitive_types::prelude::Address;

/// Contains all types related to node identities.
pub mod account;
/// Implements types for ticket acknowledgement.
pub mod acknowledgement;
/// Implements types for on-chain announcement of nodes.
pub mod announcement;

/// Implements types related to HOPR payment channels.
pub mod channels;
/// Enumerates all errors in this crate.
pub mod errors;
/// Types related to internal HOPR protocol logic.
pub mod protocol;

#[doc(hidden)]
pub mod prelude {
    pub use super::account::*;
    pub use super::acknowledgement::*;
    pub use super::announcement::*;
    pub use super::channels::*;
    pub use super::errors::CoreTypesError;
    pub use super::protocol::*;
}

/// A type that can represent both [chain public key](Address) and [packet public key](OffchainPublicKey).
pub enum ChainOrPacketKey {
    /// Represents [chain public key](Address).
    ChainKey(Address),
    /// Represents [packet public key](OffchainPublicKey).
    PacketKey(OffchainPublicKey),
}

impl From<Address> for ChainOrPacketKey {
    fn from(value: Address) -> Self {
        Self::ChainKey(value)
    }
}

impl From<OffchainPublicKey> for ChainOrPacketKey {
    fn from(value: OffchainPublicKey) -> Self {
        Self::PacketKey(value)
    }
}

impl TryFrom<ChainOrPacketKey> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: ChainOrPacketKey) -> std::result::Result<Self, Self::Error> {
        match value {
            ChainOrPacketKey::ChainKey(_) => Err(GeneralError::InvalidInput),
            ChainOrPacketKey::PacketKey(k) => Ok(k),
        }
    }
}

impl TryFrom<ChainOrPacketKey> for Address {
    type Error = GeneralError;

    fn try_from(value: ChainOrPacketKey) -> std::result::Result<Self, Self::Error> {
        match value {
            ChainOrPacketKey::ChainKey(k) => Ok(k),
            ChainOrPacketKey::PacketKey(_) => Err(GeneralError::InvalidInput),
        }
    }
}