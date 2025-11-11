use std::fmt::Formatter;

use futures::{FutureExt, StreamExt, future::BoxFuture, stream::FuturesUnordered};
use hopr_internal_types::prelude::AcknowledgedTicketStatus;
pub use hopr_internal_types::prelude::{AcknowledgedTicket, VerifiedTicket};

use crate::{
    chain::ChainReceipt,
    db::{HoprDbTicketOperations, TicketMarker, TicketSelector},
};

/// Result of [`redeem_tickets_via_selector`].
///
/// Contains tickets that were successfully redeemed, rejected or left untouched.
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

impl<E> std::fmt::Display for BatchRedemptionResult<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "redemption results - successful: {}, rejected: {}, retriable: {}",
            self.successful.len(),
            self.rejected.len(),
            self.will_retry.len()
        )
    }
}

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

    /// Fetches a batch of tickets via [`selector`](TicketSelector) to [`HoprDbTicketOperations`]
    /// and performs batched ticket redemption.
    ///
    /// The function takes care of properly marking the tickets in the DB as being redeemed and
    /// also properly unmarking or removing them on redemption success or failure.
    ///
    /// The method waits until all matched tickets are either redeemed or fail to redeem,
    /// reporting the results in the [`BatchRedemptionResult`] object.
    async fn redeem_tickets_via_selector<Db>(
        &self,
        db: &Db,
        selector: TicketSelector,
    ) -> Result<BatchRedemptionResult<Self::Error>, Db::Error>
    where
        Db: HoprDbTicketOperations + Sync,
    {
        // Collect the tickets first so we don't hold up the DB connection
        let mut tickets = db
            .update_ticket_states_and_fetch(selector.clone(), AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect::<Vec<_>>()
            .await;

        if tickets.is_empty() {
            return Ok(BatchRedemptionResult::default());
        }

        // Make sure that the tickets are sorted
        tickets.sort();

        let futures = FuturesUnordered::new();
        for ticket in tickets {
            match self.redeem_ticket(ticket).await {
                Ok(redeem_tracker) => futures.push(redeem_tracker),
                Err(error) => futures.push(futures::future::err(error).boxed()),
            }
        }

        Ok(futures
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
}
