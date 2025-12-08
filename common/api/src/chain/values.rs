use std::{error::Error, time::Duration};

pub use hopr_chain_types::ContractAddresses;
use hopr_crypto_types::prelude::Hash;
pub use hopr_internal_types::prelude::WinningProbability;
pub use hopr_primitive_types::balance::HoprBalance;
use hopr_primitive_types::{
    balance::{Balance, Currency},
    prelude::Address,
};

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

    /// Returns the native or token currency balance of the given on-chain account.
    async fn balance<C: Currency, A: Into<Address> + Send>(&self, address: A) -> Result<Balance<C>, Self::Error>;
    /// Retrieves the domain separators of HOPR smart contracts.
    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error>;
    /// Retrieves the network-set minimum incoming ticket winning probability.
    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error>;
    /// Retrieves the network-set minimum ticket price.
    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error>;
    /// Retrieves the current key binding fee
    /// used for new key-binding [announcements](crate::chain::ChainWriteAccountOperations::announce).
    async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error>;
    /// Gets the grace period for channel closure finalization.
    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error>;
    /// Gets the information about the HOPR network on-chain deployment.
    async fn chain_info(&self) -> Result<ChainInfo, Self::Error>;
    /// Convenience function to determine the winning probability and ticket price for outgoing
    /// tickets.
    async fn outgoing_ticket_values(
        &self,
        cfg_out_wp: Option<WinningProbability>,
        cfg_out_price: Option<HoprBalance>,
    ) -> Result<(WinningProbability, HoprBalance), Self::Error> {
        // This operation hits the cache unless the new value is fetched for the first time
        // NOTE: as opposed to the winning probability, the ticket price does not have
        // a reasonable default, and therefore the operation fails
        let network_ticket_price = self.minimum_ticket_price().await?;

        let outgoing_ticket_price = cfg_out_price.unwrap_or(network_ticket_price);

        // This operation hits the cache unless the new value is fetched for the first time
        let network_win_prob = self
            .minimum_incoming_ticket_win_prob()
            .await
            .inspect_err(|error| tracing::error!(%error, "failed to determine current network winning probability"))
            .ok();

        // If no explicit winning probability is configured, use the network value
        // or 1 if the network value was not determined.
        // This code does not take the max from those, as it is the upper layer's responsibility
        // to ensure the configured value is not smaller than the network value.
        let outgoing_ticket_win_prob = cfg_out_wp.or(network_win_prob).unwrap_or_default(); // Absolute default WinningProbability is 1.0

        Ok((outgoing_ticket_win_prob, outgoing_ticket_price))
    }
}
