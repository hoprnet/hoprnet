use std::{error::Error, future::Future};

pub use hopr_internal_types::prelude::{RedeemableTicket, WinningProbability};
use hopr_primitive_types::balance::HoprBalance;

use crate::chain::ChainReceipt;

/// On-chain write operations with tickets.
#[async_trait::async_trait]
pub trait ChainWriteTicketOperations {
    type Error: Error + Send + Sync + 'static;
    /// Redeems a single ticket on-chain.
    async fn redeem_ticket(
        &self,
        ticket: RedeemableTicket,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>> + Send + '_, Self::Error>;
}

/// On-chain read operations with tickets.
#[async_trait::async_trait]
pub trait ChainReadTicketOperations {
    type Error: Error + Send + Sync + 'static;
    /// Retrieves the network-set minimum incoming ticket winning probability.
    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error>;
    /// Retrieves the network-set minimum ticket price.
    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error>;
}
