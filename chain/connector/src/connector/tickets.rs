use std::collections::BTreeSet;
use std::collections::hash_map::Entry;
use blokli_client::api::BlokliTransactionClient;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
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
        let event_tx = self.events.0.clone();
        let client = self.client.clone();
        let jh = hopr_async_runtime::prelude::spawn(async move {
            match track_transaction(client.as_ref(), tx_id)?.await {
                Ok(hash) => {
                    let _ = event_tx.broadcast_direct(ChainEvent::TicketRedeemed(channel, Some(ticket.ticket.into()))).await;
                    Ok(hash)
                }
                Err(e) => {
                    let _ = event_tx.broadcast_direct(ChainEvent::RedeemFailed(channel, ticket.ticket.into())).await;
                    Err(e)
                },
            }
        });

        Ok(jh.map_err(|e| ConnectorError::OtherError(e.into()))
            .and_then(|res| futures::future::ready(res))
            .boxed())
    }

    async fn redeem_tickets(&self, tickets: Vec<RedeemableTicket>) -> Result<BoxFuture<'_, Vec<Result<ChainReceipt, Self::Error>>>, Self::Error> {
        let mut tickets_by_channels = std::collections::HashMap::<ChannelId, BTreeSet<RedeemableTicket>>::new();
        for ticket in tickets {
            if let Some(channel) = self.channel_by_id(&ticket.ticket.verified_ticket().channel_id).await? {
                if &channel.destination != self.chain_key.public().as_ref() {
                    return Err(ConnectorError::InvalidArguments("ticket is not for this node"));
                }

                match tickets_by_channels.entry(ticket.ticket.verified_ticket().channel_id) {
                    Entry::Occupied(mut set) => {
                        set.get_mut().insert(ticket);
                    }
                    Entry::Vacant(v) => {
                        v.insert(BTreeSet::from([ticket]));
                    }
                }
            } else {
                return Err(ConnectorError::ChannelDoesNotExist(ticket.ticket.verified_ticket().channel_id));
            }
        }

        todo!()
    }
}