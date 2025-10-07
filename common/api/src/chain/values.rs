use std::{error::Error, time::Duration};

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

/// Retrieves various on-chain information.
#[async_trait::async_trait]
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
}
