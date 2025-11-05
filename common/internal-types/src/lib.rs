//! This crate contains all types that are specific to the HOPR protocol.
//! As opposed to `hopr-primitive-types` which contains more generic types (not necessarily specific only to HOPR).

use std::fmt::Formatter;

use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_primitive_types::{
    errors::GeneralError,
    prelude::{Address, BytesRepresentable},
};

/// Contains all types related to node identities.
pub mod account;
/// Implements types for on-chain announcement of nodes.
pub mod announcement;
/// Implements types related to HOPR payment channels.
pub mod channels;
/// Lists all errors in this crate.
pub mod errors;
/// Types related to internal HOPR protocol logic.
pub mod protocol;
/// Implements types for tickets.
pub mod tickets;

/// Uniquely identifies a HOPR node either by its [`Address`] or [`OffchainPublicKey`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::EnumIs, strum::EnumTryAs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeId {
    /// Node represented by its on-chain [`Address`].
    Chain(Address),
    /// Node represented by its off-chain [`OffchainPublicKey`].
    Offchain(OffchainPublicKey),
}

impl From<Address> for NodeId {
    fn from(value: Address) -> Self {
        Self::Chain(value)
    }
}

impl From<OffchainPublicKey> for NodeId {
    fn from(value: OffchainPublicKey) -> Self {
        Self::Offchain(value)
    }
}

impl TryFrom<NodeId> for Address {
    type Error = GeneralError;

    fn try_from(value: NodeId) -> Result<Self, Self::Error> {
        value.try_as_chain().ok_or(GeneralError::InvalidInput)
    }
}

impl TryFrom<NodeId> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: NodeId) -> Result<Self, Self::Error> {
        value.try_as_offchain().ok_or(GeneralError::InvalidInput)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeId::Chain(a) => a.fmt(f),
            NodeId::Offchain(a) => a.fmt(f),
        }
    }
}

impl std::str::FromStr for NodeId {
    type Err = GeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < OffchainPublicKey::SIZE * 2 {
            Ok(Self::Chain(s.parse()?))
        } else {
            Ok(Self::Offchain(s.parse()?))
        }
    }
}

pub use multiaddr::Multiaddr;

#[doc(hidden)]
pub mod prelude {
    pub use super::{
        Multiaddr, NodeId, account::*, announcement::*, channels::*, errors::CoreTypesError, protocol::*, tickets::*,
    };
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hopr_crypto_types::prelude::*;

    use super::*;

    #[test]
    fn test_node_id_display_from_str() -> anyhow::Result<()> {
        let offchain = OffchainKeypair::random().public().clone();
        let onchain = ChainKeypair::random().public().to_address();

        let n1 = NodeId::from_str(&offchain.to_string())?;
        let n2 = NodeId::from_str(&onchain.to_string())?;

        assert_eq!(n1, NodeId::Offchain(offchain));
        assert_eq!(n2, NodeId::Chain(onchain));

        assert_eq!(n1.to_string(), offchain.to_string());
        assert_eq!(n2.to_string(), onchain.to_string());

        Ok(())
    }
}
