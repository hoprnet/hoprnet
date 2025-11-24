use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, TryFutureExt, future::BoxFuture};
use hopr_api::chain::{ChainReceipt, TicketRedeemError};
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
    async fn prepare_ticket_redeem_payload(&self, ticket: RedeemableTicket) -> Result<P::TxRequest, ConnectorError> {
        self.check_connection_state()?;

        let channel_id = ticket.verified_ticket().channel_id;
        if generate_channel_id(ticket.ticket.verified_issuer(), self.chain_key.public().as_ref()) != channel_id {
            tracing::error!(%channel_id, "redeemed ticket is not ours");
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
            .ok_or_else(|| {
                tracing::error!(%channel_id, "trying to redeem a ticket on a channel that does not exist");
                ConnectorError::ChannelDoesNotExist(channel_id)
            })?;

        if channel.status == blokli_client::api::types::ChannelStatus::Closed {
            tracing::error!(%channel_id, "trying to redeem a ticket on a closed channel");
            return Err(ConnectorError::ChannelClosed(channel_id));
        }

        if channel.epoch as u32 != ticket.verified_ticket().channel_epoch {
            tracing::error!(
                channel_epoch = channel.epoch,
                ticket_epoch = ticket.verified_ticket().channel_epoch,
                "invalid redeemed ticket epoch"
            );
            return Err(ConnectorError::InvalidTicket);
        }

        let channel_index: u64 = channel
            .ticket_index
            .0
            .parse()
            .map_err(|e| ConnectorError::TypeConversion(format!("unparseable channel index at redemption: {e}")))?;

        if channel_index > ticket.verified_ticket().index {
            tracing::error!(
                channel_index,
                ticket_index = ticket.verified_ticket().index,
                "invalid redeemed ticket index"
            );
            return Err(ConnectorError::InvalidTicket);
        }

        Ok(self.payload_generator.redeem_ticket(ticket)?)
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
        ticket: RedeemableTicket,
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
                        if let Some(reject_error) = tx_tracking_error.as_transaction_rejection_error() {
                            TicketRedeemError::Rejected(ticket.ticket.clone(), format!("on-chain rejection: {reject_error:?}"))
                        } else {
                            TicketRedeemError::ProcessingError(ticket.ticket.clone(), tx_tracking_error)
                        })
                    .and_then(move |receipt| futures::future::ok((ticket_clone.ticket, receipt)))
                    .boxed())
            }
            Err(e @ ConnectorError::InvalidTicket)
            | Err(e @ ConnectorError::ChannelDoesNotExist(_))
            | Err(e @ ConnectorError::ChannelClosed(_)) => {
                Err(TicketRedeemError::Rejected(ticket.ticket, e.to_string()))
            }
            Err(e) => Err(TicketRedeemError::ProcessingError(ticket.ticket, e)),
        }
    }
}
