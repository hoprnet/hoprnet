use std::future::Future;

pub use hopr_internal_types::prelude::RedeemableTicket;

use crate::chain::ChainReceipt;

/// On-chain operations with tickets.
#[async_trait::async_trait]
pub trait ChainTicketOperations {
    type Error;
    /// Redeems a single ticket on-chain.
    async fn redeem_ticket(
        &self,
        ticket: RedeemableTicket,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>> + Send + '_, Self::Error>;
}
