//! This crate contains various on-chain related modules and types.
use hopr_primitive_types::primitives::Address;
use serde::{Deserialize, Serialize};

pub mod chain_events;
pub mod errors;
#[cfg(feature = "use-bindings")]
mod parser;
pub mod payload;

#[cfg(feature = "use-bindings")]
pub use {hopr_bindings::exports, parser::ParsedHoprChainAction};

pub mod prelude {
    #[cfg(feature = "use-bindings")]
    pub use super::payload::{BasicPayloadGenerator, SafePayloadGenerator, TransactionRequest};
    pub use super::{
        ContractAddresses,
        chain_events::ChainEvent,
        payload::{GasEstimation, PayloadGenerator, SignableTransaction},
    };
}

/// Holds addresses of all smart contracts.
#[serde_with::serde_as]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractAddresses {
    /// Token contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub token: Address,
    /// Channels contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub channels: Address,
    /// Announcement contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub announcements: Address,
    /// Safe registry contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_safe_registry: Address,
    /// Price oracle contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub ticket_price_oracle: Address,
    /// Minimum ticket winning probability contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub winning_probability_oracle: Address,
    /// Migration helper for node safes and modules
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_safe_migration: Address,
    /// Stake factory contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_stake_factory: Address,
    /// Node management module contract (can be zero if safe is not used)
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub module_implementation: Address,
}

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

#[cfg(feature = "use-bindings")]
impl From<hopr_bindings::ContractAddresses> for ContractAddresses {
    fn from(value: hopr_bindings::ContractAddresses) -> Self {
        Self {
            token: Address::new(&value.token.0.0),
            channels: Address::new(&value.channels.0.0),
            announcements: Address::new(&value.announcements.0.0),
            node_safe_registry: Address::new(&value.node_safe_registry.0.0),
            ticket_price_oracle: Address::new(&value.ticket_price_oracle.0.0),
            winning_probability_oracle: Address::new(&value.winning_probability_oracle.0.0),
            node_safe_migration: Address::new(&value.node_safe_migration.0.0),
            node_stake_factory: Address::new(&value.node_stake_factory.0.0),
            module_implementation: Address::new(&value.module_implementation.0.0),
        }
    }
}

#[cfg(feature = "use-bindings")]
impl ContractAddresses {
    /// Returns contract addresses for the given HOPR network `name`
    /// or `None` if the network does not exist.
    pub fn for_network(name: &str) -> Option<Self> {
        hopr_bindings::config::NetworksWithContractAddresses::default()
            .networks
            .get(name)
            .cloned()
            .map(|n| n.addresses.into())
    }
}
