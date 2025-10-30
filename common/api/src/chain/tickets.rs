use std::error::Error;
use std::todo;
use chrono::format::Item;
use futures::future::BoxFuture;
pub use hopr_internal_types::prelude::RedeemableTicket;
use crate::{chain::ChainReceipt, db::TicketSelector};

/// On-chain write operations with tickets.
#[async_trait::async_trait]
pub trait ChainWriteTicketOperations {
    type Error: Error + Send + Sync + 'static;
    /// Redeems a single ticket on-chain.
    async fn redeem_ticket(
        &self,
        ticket: RedeemableTicket,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Allows to batch-redeem multiple tickets on-chain.
    async fn redeem_tickets(
        &self,
        tickets: Vec<RedeemableTicket>,
    ) -> Result<BoxFuture<'_, Vec<Result<ChainReceipt, Self::Error>>>, Self::Error>;
}
