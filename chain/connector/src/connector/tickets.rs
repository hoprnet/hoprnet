use blokli_client::api::BlokliTransactionClient;
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt};
use futures::stream::FuturesUnordered;
use hopr_api::chain::{ChainReadChannelOperations, ChainReceipt};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;

use crate::backend::Backend;
use crate::connector::{HoprBlockchainConnector};
use crate::connector::utils::track_transaction;
use crate::errors::ConnectorError;


#[async_trait::async_trait]
impl<B,C,P> hopr_api::chain::ChainWriteTicketOperations for HoprBlockchainConnector<B,C,P>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static
{
    type Error = ConnectorError;

    async fn redeem_ticket(&self, ticket: RedeemableTicket) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        let channel = self.channel_by_id(&ticket.ticket.verified_ticket().channel_id)
            .await?
            .ok_or(ConnectorError::ChannelDoesNotExist(ticket.ticket.verified_ticket().channel_id))?;

        if &channel.destination != self.chain_key.public().as_ref() {
            return Err(ConnectorError::InvalidArguments("ticket is not for this node"));
        }

        let signed_payload = self
            .payload_generator
            .redeem_ticket(ticket.clone())?
            .sign_and_encode_to_eip2718(&self.chain_key)
            .await?;

        let tx_id = self.client.submit_and_track_transaction(&signed_payload).await?;
        let tracker = track_transaction(self.client.as_ref(), tx_id)?.boxed();

        let ticket_id = TicketId::from(ticket.verified_ticket());
        self.redeeming_tickets.insert(ticket_id, ticket.ticket.into()).await;
        tracing::debug!(%ticket_id, "ticket sent for redemption");

        Ok(tracker)
    }

    async fn redeem_tickets(&self, mut tickets: Vec<RedeemableTicket>) -> Result<BoxFuture<'_, Vec<Result<ChainReceipt, Self::Error>>>, Self::Error> {
        // Make sure we redeem the tickets in the correct order and per channel
        tickets.sort();
        tickets.dedup();

        // Send the tickets for redemption
        let futures = FuturesUnordered::new();
        for ticket in tickets {
            let ticket_id = TicketId::from(ticket.verified_ticket());
            match self.redeem_ticket(ticket.clone()).await {
                Ok(tracker) => {
                    self.redeeming_tickets.insert(ticket_id, ticket.ticket.into()).await;
                    tracing::debug!(%ticket_id, "ticket sent for redemption");
                    futures.push(tracker);
                }
                Err(error) => {
                    tracing::error!(%error, %ticket_id, "error sending ticket for redeeming");
                }
            }
        }

        if futures.is_empty() {
            Err(ConnectorError::BatchRedemptionFailed)
        } else {
            Ok(futures.collect().boxed())
        }
    }
}