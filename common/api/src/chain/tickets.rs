use futures::{StreamExt, future::BoxFuture};
pub use hopr_internal_types::prelude::AcknowledgedTicket;
use hopr_internal_types::prelude::AcknowledgedTicketStatus;

use crate::{
    chain::ChainReceipt,
    db::{HoprDbTicketOperations, TicketSelector},
};

// TODO: change these APIs to accept RedeemableTicket (see #7616)
/// On-chain write operations with tickets.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainWriteTicketOperations {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Redeems a single ticket on-chain.
    async fn redeem_ticket(
        &self,
        ticket: AcknowledgedTicket,
    ) -> Result<BoxFuture<'life0, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Allows to batch-redeem multiple tickets on-chain.
    async fn redeem_tickets(
        &self,
        tickets: Vec<AcknowledgedTicket>,
    ) -> Result<BoxFuture<'life0, Vec<Result<ChainReceipt, Self::Error>>>, Self::Error>;
}

/// Possible errors for [`redeem_tickets_via_selector`].
#[derive(Debug, thiserror::Error)]
pub enum BatchRedeemTicketError<EDb, ERedeem> {
    #[error("no tickets to redeem")]
    NoTickets,
    #[error("db error: {0}")]
    DbError(EDb),
    #[error("ticket redemption error: {0}")]
    RedeemError(ERedeem),
    #[error(transparent)]
    CoreTypes(#[from] hopr_internal_types::errors::CoreTypesError),
}

/// Convenience function combining the [`HoprDbTicketOperations`] and [`ChainWriteTicketOperations`]
/// to allow batch redemption given the [`selector`](TicketSelector).
pub async fn redeem_tickets_via_selector<'a, Db, R>(
    selector: TicketSelector,
    db: &'a Db,
    resolver: &'a R,
) -> Result<
    (
        usize,
        BoxFuture<'a, Vec<Result<ChainReceipt, <R as ChainWriteTicketOperations>::Error>>>,
    ),
    BatchRedeemTicketError<Db::Error, <R as ChainWriteTicketOperations>::Error>,
>
where
    Db: HoprDbTicketOperations,
    R: ChainWriteTicketOperations,
{
    let tickets = db
        .update_ticket_states_and_fetch(selector.clone(), AcknowledgedTicketStatus::BeingRedeemed)
        .await
        .map_err(BatchRedeemTicketError::DbError)?
        .collect::<Vec<AcknowledgedTicket>>()
        .await;

    if tickets.is_empty() {
        return Err(BatchRedeemTicketError::NoTickets);
    }

    let len = tickets.len();
    match resolver.redeem_tickets(tickets).await {
        Ok(res) => Ok((len, res)),
        Err(error) => {
            // Restore the ticket states to untouched on error
            db.update_ticket_states(selector, AcknowledgedTicketStatus::Untouched)
                .await
                .map_err(BatchRedeemTicketError::DbError)?;

            Err(BatchRedeemTicketError::RedeemError(error))
        }
    }
}
