use futures::{StreamExt, TryStreamExt, future::BoxFuture};
use hopr_internal_types::prelude::AcknowledgedTicketStatus;
pub use hopr_internal_types::prelude::{AcknowledgedTicket, VerifiedTicket};

use crate::{
    chain::ChainReceipt,
    db::{HoprDbTicketOperations, TicketMarker, TicketSelector},
};

/// Errors that can occur during ticket redemption.
#[derive(Debug, thiserror::Error)]
pub enum TicketRedeemError<E> {
    /// Non-retryable error, the ticket should be discarded
    #[error("redemption of ticket {0} was rejected due to: {1}")]
    Rejected(VerifiedTicket, String),
    /// Retryable error, the ticket redemption should be retried.
    #[error("processing error during redemption of ticket {0}: {1}")]
    ProcessingError(VerifiedTicket, E),
}

// TODO: change these APIs to accept RedeemableTicket (see #7616)
/// On-chain write operations with tickets.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainWriteTicketOperations {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Redeems a single ticket on-chain.
    ///
    /// The input `ticket` is always returned as [`VerifiedTicket`], either on success or failure.
    async fn redeem_ticket<'a>(
        &'a self,
        ticket: AcknowledgedTicket,
    ) -> Result<
        BoxFuture<'a, Result<(VerifiedTicket, ChainReceipt), TicketRedeemError<Self::Error>>>,
        TicketRedeemError<Self::Error>,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchRedemptionResult<E> {
    /// Tickets which were successfully redeemed and
    /// removed from the ticket database.
    pub successful: Vec<(VerifiedTicket, ChainReceipt)>,
    /// Tickets which were permanently rejected and removed from the ticket database.
    pub rejected: Vec<(VerifiedTicket, String)>,
    /// Tickets which could not be redeemed and will be retried later.
    pub will_retry: Vec<(VerifiedTicket, E)>,
}

impl<E> Default for BatchRedemptionResult<E> {
    fn default() -> Self {
        Self {
            successful: vec![],
            rejected: vec![],
            will_retry: vec![],
        }
    }
}

/// Convenience function combining the [`HoprDbTicketOperations`] and [`ChainWriteTicketOperations`]
/// to allow batch redemption given the [`selector`](TicketSelector).
///
/// The function returns only the successfully redeemed tickets. Those that were not successful
/// are either marked as rejected or untouched (if redemption should be retried later).
pub async fn redeem_tickets_via_selector<Db, R>(
    selector: TicketSelector,
    db: &Db,
    resolver: &R,
) -> Result<BatchRedemptionResult<R::Error>, Db::Error>
where
    Db: HoprDbTicketOperations,
    R: ChainWriteTicketOperations,
{
    // Collect the tickets first so we don't hold up the DB connection
    let tickets = db
        .update_ticket_states_and_fetch(selector.clone(), AcknowledgedTicketStatus::BeingRedeemed)
        .await?
        .collect::<Vec<_>>()
        .await;

    Ok(futures::stream::iter(tickets)
        .then(|ticket| resolver.redeem_ticket(ticket))
        .try_buffer_unordered(10)
        .fold(BatchRedemptionResult::default(), |mut res, item| async move {
            match item {
                Ok((ticket, receipt)) => {
                    if let Err(error) = db.mark_tickets_as((&ticket).into(), TicketMarker::Redeemed).await {
                        tracing::error!(%error, "failed to mark ticket as redeemed");
                    }
                    res.successful.push((ticket, receipt));
                }
                Err(TicketRedeemError::Rejected(ticket, reason)) => {
                    if let Err(error) = db.mark_tickets_as((&ticket).into(), TicketMarker::Rejected).await {
                        tracing::error!(%error, "failed to mark ticket as rejected");
                    }
                    res.rejected.push((ticket, reason));
                }
                Err(TicketRedeemError::ProcessingError(ticket, proc_error)) => {
                    if let Err(error) = db
                        .update_ticket_states((&ticket).into(), AcknowledgedTicketStatus::Untouched)
                        .await
                    {
                        tracing::error!(%error, "failed to update ticket state to untouched");
                    }
                    res.will_retry.push((ticket, proc_error));
                }
            }
            res
        })
        .await)
}
