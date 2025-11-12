//! This crate contains various on-chain related modules and types.

use hopr_primitive_types::primitives::Address;
use serde::{Deserialize, Serialize};

pub mod chain_events;
pub mod errors;
mod parser;
pub mod payload;

pub use parser::ParsedHoprChainAction;

pub mod prelude {
    pub use super::{
        ContractAddresses,
        chain_events::ChainEvent,
        payload::{BasicPayloadGenerator, PayloadGenerator, SafePayloadGenerator, SignableTransaction},
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
    /// Stake factory contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_stake_v2_factory: Address,
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
            self.node_stake_v2_factory,
            self.module_implementation,
        ]
        .into_iter()
    }
}
