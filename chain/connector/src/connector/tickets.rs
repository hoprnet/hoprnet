use blokli_client::{
    api::{BlokliQueryClient, BlokliTransactionClient},
    errors::{ErrorKind, TrackingErrorKind},
};
use futures::{FutureExt, TryFutureExt, future::BoxFuture};
use hopr_api::chain::{ChainReceipt, ChainValues, TicketRedeemError};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;

use crate::{backend::Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P> HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    async fn prepare_ticket_redeem_payload(&self, ticket: AcknowledgedTicket) -> Result<P::TxRequest, ConnectorError> {
        self.check_connection_state()?;

        let channel_id = ticket.verified_ticket().channel_id;
        if generate_channel_id(ticket.ticket.verified_issuer(), self.chain_key.public().as_ref()) != channel_id {
            return Err(ConnectorError::InvalidTicket);
        }

        // Do a fresh Blokli query to ensure the channel is not closed.
        let channel = self
            .client
            .query_channels(blokli_client::api::ChannelSelector {
                filter: blokli_client::api::ChannelFilter::ChannelId(ticket.verified_ticket().channel_id.into()),
                status: None,
            })
            .await?
            .first()
            .cloned()
            .ok_or_else(|| ConnectorError::ChannelDoesNotExist(channel_id))?;

        if channel.status == blokli_client::api::types::ChannelStatus::Closed {
            return Err(ConnectorError::ChannelClosed(channel_id));
        }

        // `into_redeemable` is a CPU-intensive operation. See #7616 for a future resolution.
        let channel_dst = self.domain_separators().await?.channel;

        let chain_key = self.chain_key.clone();
        let ticket =
            hopr_async_runtime::prelude::spawn_blocking(move || ticket.into_redeemable(&chain_key, &channel_dst))
                .await
                .map_err(|e| ConnectorError::OtherError(e.into()))??;

        Ok(self.payload_generator.redeem_ticket(ticket.clone())?)
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainWriteTicketOperations for HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn redeem_ticket<'a>(
        &'a self,
        ticket: AcknowledgedTicket,
    ) -> Result<
        BoxFuture<'a, Result<(VerifiedTicket, ChainReceipt), TicketRedeemError<Self::Error>>>,
        TicketRedeemError<Self::Error>,
    > {
        match self.prepare_ticket_redeem_payload(ticket.clone()).await {
            Ok(tx_req) => {
                let ticket_clone = ticket.clone();
                Ok(self
                    .send_tx(tx_req)
                    .await
                    .map_err(|e| TicketRedeemError::ProcessingError(ticket.ticket.clone(), e))?
                    .map_err(move |tx_tracking_error|
                        // For ticket redemption, certain errors are to be handled differently
                        match tx_tracking_error {
                            ConnectorError::ClientError(client_error)
                            if matches!(
                                client_error.kind(),
                                ErrorKind::TrackingError(TrackingErrorKind::Reverted) |
                                ErrorKind::TrackingError(TrackingErrorKind::ValidationFailed)
                            ) =>
                                {
                                    TicketRedeemError::Rejected(ticket.ticket.clone(), client_error.to_string())
                                }
                            _ => TicketRedeemError::ProcessingError(ticket.ticket.clone(), tx_tracking_error.into())
                        })
                    .and_then(move |receipt| futures::future::ok((ticket_clone.ticket, receipt)))
                    .boxed())
            }
            Err(e @ ConnectorError::InvalidTicket)
            | Err(e @ ConnectorError::ChannelDoesNotExist(_))
            | Err(e @ ConnectorError::ChannelClosed(_)) => {
                Err(TicketRedeemError::Rejected(ticket.ticket, e.to_string()))
            }
            Err(e) => Err(TicketRedeemError::ProcessingError(ticket.ticket, e.into())),
        }
    }
}
