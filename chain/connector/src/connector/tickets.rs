use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, future::BoxFuture, stream::FuturesUnordered};
use hopr_api::chain::{ChainReadChannelOperations, ChainReceipt, ChainValues};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;

use crate::{
    backend::Backend,
    connector::{HoprBlockchainConnector, utils::track_transaction},
    errors::ConnectorError,
};

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainWriteTicketOperations for HoprBlockchainConnector<C, B, P>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn redeem_ticket(
        &self,
        ticket: AcknowledgedTicket,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let channel = self
            .channel_by_id(&ticket.ticket.verified_ticket().channel_id)
            .await?
            .ok_or(ConnectorError::ChannelDoesNotExist(
                ticket.ticket.verified_ticket().channel_id,
            ))?;

        if &channel.destination != self.chain_key.public().as_ref() {
            return Err(ConnectorError::InvalidArguments("ticket is not for this node"));
        }

        // `into_redeemable` is a CPU-intensive operation. See #7616 for a future resolution.
        let channel_dst = self.domain_separators().await?.channel;
        let chain_key = self.chain_key.clone();
        let ticket =
            hopr_async_runtime::prelude::spawn_blocking(move || ticket.into_redeemable(&chain_key, &channel_dst))
                .await
                .map_err(|e| ConnectorError::OtherError(e.into()))??;

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

    async fn redeem_tickets(
        &self,
        mut tickets: Vec<AcknowledgedTicket>,
    ) -> Result<BoxFuture<'_, Vec<Result<ChainReceipt, Self::Error>>>, Self::Error> {
        self.check_connection_state()?;

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
