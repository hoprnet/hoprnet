use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, future::BoxFuture, stream::BoxStream};
use hopr_api::chain::{ChainReceipt, ChannelSelector};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::Keypair;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{backend::Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

#[async_trait::async_trait]
impl<B, C, P, R> hopr_api::chain::ChainReadChannelOperations for HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
    C: Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    type Error = ConnectorError;

    fn me(&self) -> &Address {
        self.chain_key.public().as_ref()
    }

    async fn channel_by_parties(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, Self::Error> {
        self.check_connection_state()?;

        let backend = self.backend.clone();
        let src = *src;
        let dst = *dst;
        Ok(self
            .channel_by_parties
            .try_get_with(ChannelParties::new(src, dst), async move {
                tracing::warn!(%src, %dst, "cache miss on channel_by_parties");
                match hopr_async_runtime::prelude::spawn_blocking(move || {
                    let channel_id = generate_channel_id(&src, &dst);
                    backend.get_channel_by_id(&channel_id)
                })
                .await
                {
                    Ok(Ok(value)) => Ok(value),
                    Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                    Err(e) => Err(ConnectorError::BackendError(e.into())),
                }
            })
            .await?)
    }

    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        self.check_connection_state()?;

        let channel_id = *channel_id;
        let backend = self.backend.clone();
        Ok(self
            .channel_by_id
            .try_get_with_by_ref(&channel_id, async move {
                tracing::warn!(%channel_id, "cache miss on channel_by_id");
                match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_channel_by_id(&channel_id)).await
                {
                    Ok(Ok(value)) => Ok(value),
                    Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                    Err(e) => Err(ConnectorError::BackendError(e.into())),
                }
            })
            .await?)
    }

    async fn stream_channels<'a>(
        &'a self,
        selector: ChannelSelector,
    ) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        self.check_connection_state()?;

        // Note: Since the graph does not contain Closed channels, they cannot
        // be selected if requested via the ChannelSelector.
        if selector.allowed_states.contains(&ChannelStatusDiscriminants::Closed) {
            return Err(ConnectorError::InvalidArguments("cannot stream closed channels"));
        }

        let channels = self
            .graph
            .read()
            .all_edges()
            .map(|(_, _, e)| e)
            .copied()
            .collect::<Vec<_>>();
        let backend = self.backend.clone();
        Ok(futures::stream::iter(channels)
            .filter_map(move |channel_id| {
                let backend = backend.clone();
                let selector = selector.clone();
                // This avoids the cache on purpose so it does not get spammed
                async move {
                    match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_channel_by_id(&channel_id))
                        .await
                    {
                        Ok(Ok(value)) => value.filter(|c| selector.satisfies(c)),
                        Ok(Err(error)) => {
                            tracing::error!(%error, %channel_id, "backend error when looking up channel");
                            None
                        }
                        Err(error) => {
                            tracing::error!(%error, %channel_id, "join error when looking up channel");
                            None
                        }
                    }
                }
            })
            .boxed())
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainWriteChannelOperations for HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn open_channel<'a>(
        &'a self,
        dst: &'a Address,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'a, Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let id = generate_channel_id(self.chain_key.public().as_ref(), dst);
        let tx_req = self.payload_generator.fund_channel(*dst, amount)?;
        tracing::debug!(channel_id = %id, %dst, %amount, "opening channel");

        Ok(self
            .send_tx(tx_req)
            .await?
            .and_then(move |tx_hash| futures::future::ok((id, tx_hash)))
            .boxed())
    }

    async fn fund_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        use hopr_api::chain::ChainReadChannelOperations;

        let channel = self
            .channel_by_id(channel_id)
            .await?
            .ok_or_else(|| ConnectorError::ChannelDoesNotExist(*channel_id))?;
        let tx_req = self.payload_generator.fund_channel(channel.destination, amount)?;
        tracing::debug!(%channel_id, %amount, "funding channel");

        Ok(self.send_tx(tx_req).await?.boxed())
    }

    async fn close_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
    ) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        use hopr_api::chain::ChainReadChannelOperations;

        let channel = self
            .channel_by_id(channel_id)
            .await?
            .ok_or_else(|| ConnectorError::ChannelDoesNotExist(*channel_id))?;

        let direction = channel.direction(self.me()).ok_or(ConnectorError::InvalidArguments(
            "cannot close channels that is not own",
        ))?;

        let tx_req = match channel.status {
            ChannelStatus::Closed => return Err(ConnectorError::ChannelClosed(*channel_id)),
            ChannelStatus::Open => {
                if direction == ChannelDirection::Outgoing {
                    tracing::debug!(%channel_id, "initiating outgoing channel closure");
                    self.payload_generator
                        .initiate_outgoing_channel_closure(channel.destination)?
                } else {
                    tracing::debug!(%channel_id, "closing incoming channel");
                    self.payload_generator.close_incoming_channel(channel.source)?
                }
            }
            c if c.closure_time_elapsed(&std::time::SystemTime::now()) => {
                if direction == ChannelDirection::Outgoing {
                    tracing::debug!(%channel_id, "finalizing outgoing channel closure");
                    self.payload_generator
                        .finalize_outgoing_channel_closure(channel.destination)?
                } else {
                    tracing::debug!(%channel_id, "closing incoming channel");
                    self.payload_generator.close_incoming_channel(channel.source)?
                }
            }
            _ => return Err(ConnectorError::InvalidState("channel closure time has not elapsed")),
        };

        Ok(self.send_tx(tx_req).await?.boxed())
    }
}
