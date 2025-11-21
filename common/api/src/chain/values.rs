use std::{error::Error, time::Duration};

pub use hopr_chain_types::ContractAddresses;
use hopr_crypto_types::prelude::Hash;
pub use hopr_internal_types::prelude::WinningProbability;
pub use hopr_primitive_types::balance::HoprBalance;

/// Contains domain separator information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainSeparators {
    /// HOPR Ledger smart contract domain separator.
    pub ledger: Hash,
    /// HOPR Node Safe Registry smart contract domain separator.
    pub safe_registry: Hash,
    /// HOPR Channels smart contract domain separator.
    pub channel: Hash,
}

/// Contains information about the HOPR on-chain network deployment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainInfo {
    /// ID of the blockchain network (e.g.: `0x64` for Gnosis Chain)
    pub chain_id: u64,
    /// Name of the HOPR network (e.g.: `dufour`)
    pub hopr_network_name: String,
    /// Addresses of the deployed HOPR smart contracts.
    pub contract_addresses: ContractAddresses,
}

/// Retrieves various on-chain information.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainValues {
    type Error: Error + Send + Sync + 'static;
    /// Retrieves the domain separators of HOPR smart contracts.
    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error>;
    /// Retrieves the network-set minimum incoming ticket winning probability.
    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error>;
    /// Retrieves the network-set minimum ticket price.
    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error>;
    /// Gets the grace period for channel closure finalization.
    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error>;
    /// Gets the information about the HOPR network on-chain deployment.
    async fn chain_info(&self) -> Result<ChainInfo, Self::Error>;
}
