//! This crate contains various on-chain related modules and types.

pub mod chain_events;
pub mod errors;
#[cfg(feature = "use-bindings")]
mod parser;
pub mod payload;

use hopr_bindings::exports::alloy;
#[cfg(feature = "use-bindings")]
pub use {
    hopr_bindings::{ContractAddresses, exports},
    parser::ParsedHoprChainAction,
};

pub mod prelude {
    #[cfg(feature = "use-bindings")]
    pub use super::payload::{BasicPayloadGenerator, SafePayloadGenerator, TransactionRequest};
    pub use super::{
        ContractAddresses,
        chain_events::ChainEvent,
        payload::{GasEstimation, PayloadGenerator, SignableTransaction},
    };
}

#[cfg(not(feature = "use-bindings"))]
/// Holds addresses of all smart contracts.
#[serde_with::serde_as]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct ContractAddresses {
    /// Token contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub token: hopr_primitive_types::primitives::Address,
    /// Channels contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub channels: hopr_primitive_types::primitives::Address,
    /// Announcement contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub announcements: hopr_primitive_types::primitives::Address,
    /// Safe registry contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_safe_registry: hopr_primitive_types::primitives::Address,
    /// Price oracle contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub ticket_price_oracle: hopr_primitive_types::primitives::Address,
    /// Minimum ticket winning probability contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub winning_probability_oracle: hopr_primitive_types::primitives::Address,
    /// Migration helper for node safes and modules
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_safe_migration: hopr_primitive_types::primitives::Address,
    /// Stake factory contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_stake_factory: hopr_primitive_types::primitives::Address,
    /// Node management module contract (can be zero if safe is not used)
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub module_implementation: hopr_primitive_types::primitives::Address,
}

#[cfg(not(feature = "use-bindings"))]
impl IntoIterator for &ContractAddresses {
    type IntoIter = std::vec::IntoIter<Address>;
    type Item = Address;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            self.token,
            self.channels,
            self.announcements,
            self.node_safe_registry,
            self.ticket_price_oracle,
            self.winning_probability_oracle,
            self.node_stake_factory,
            self.module_implementation,
        ]
        .into_iter()
    }
}

/// Returns chain ID and contract addresses for a known HOPR network on-chain deployment with the given `name` (e.g.
/// `rotsee` or `dufour`)
///
/// Returns `None` if network deployment with the given `name` is not known.
#[cfg(feature = "use-bindings")]
pub fn contract_addresses_for_network(name: &str) -> Option<(u64, ContractAddresses)> {
    hopr_bindings::config::NetworksWithContractAddresses::default()
        .networks
        .get(name)
        .cloned()
        .map(|n| (n.chain_id, n.addresses))
}

// Used instead of From implementation to avoid alloy being a dependency of the primitive crates
#[inline]
pub(crate) fn a2h(a: hopr_primitive_types::prelude::Address) -> alloy::primitives::Address {
    alloy::primitives::Address::from_slice(a.as_ref())
}
