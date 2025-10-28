use blokli_client::api::BlokliQueryClient;
use blokli_client::api::v1::ChannelFilter;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::{FutureExt, StreamExt};
use hopr_api::chain::{ChainReceipt, ChannelSelector};
use hopr_crypto_types::prelude::Keypair;
use hopr_internal_types::channels::{ChannelEntry, ChannelId, ChannelStatus, ChannelStatusDiscriminants};
use hopr_internal_types::prelude::{generate_channel_id, ChannelParties};
use hopr_primitive_types::balance::HoprBalance;
use hopr_primitive_types::prelude::Address;
use crate::connector::{ HoprBlockchainConnector};
use crate::connector::backend::Backend;
use crate::errors::ConnectorError;
use crate::payload::{sign_payload, PayloadGenerator};


#[async_trait::async_trait]
impl<B> hopr_api::chain::ChainReadChannelOperations for  HoprBlockchainConnector<B>
where
    B: Backend + Send + Sync + 'static
{
    type Error = ConnectorError;

    fn me(&self) -> &Address {
        self.chain_key.public().as_ref()
    }

    async fn channel_by_parties(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, Self::Error> {
        let backend = self.backend.clone();
        let src = *src;
        let dst = *dst;
        Ok(self.channel_by_parties.try_get_with(ChannelParties::new(src, dst), async move {
            match hopr_async_runtime::prelude::spawn_blocking(move || {
                let channel_id = generate_channel_id(&src, &dst);
                backend.get_channel_by_id(&channel_id)
            }).await {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                Err(e) => Err(ConnectorError::BackendError(e.into())),
            }
        }).await?)
    }

    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        let channel_id = *channel_id;
        let backend = self.backend.clone();
        Ok(self.channel_by_id.try_get_with_by_ref(&channel_id, async move {
            match hopr_async_runtime::prelude::spawn_blocking(move || {
                backend.get_channel_by_id(&channel_id)
            }).await {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                Err(e) => Err(ConnectorError::BackendError(e.into())),
            }
        }).await?)
    }

    async fn stream_channels<'a>(&'a self, selector: ChannelSelector) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        // Note: Since the graph does not contain Closed channels, they cannot
        // be selected if requested via the ChannelSelector.
        if selector.allowed_states.contains(&ChannelStatusDiscriminants::Closed) {
            return Err(ConnectorError::InvalidArguments("cannot stream closed channels"));
        }

        let channels = self.graph.lock().edge_weights().copied().collect::<Vec<_>>();
        let backend = self.backend.clone();
        Ok(futures::stream::iter(channels)
            .filter_map(move |channel_id| {
                let backend = backend.clone();
                let selector = selector.clone();
                async move {
                    match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_channel_by_id(&channel_id)).await {
                        Ok(Ok(value)) => value.filter(|c| selector.satisfies(c)),
                        Ok(Err(error)) => {
                            tracing::error!(%error, %channel_id, "backend error when looking up channel");
                            None
                        },
                        Err(error) => {
                            tracing::error!(%error, %channel_id, "join error when looking up channel");
                            None
                        },
                    }
                }
            })
            .boxed())
    }
}

#[async_trait::async_trait]
impl<B: Send + Sync> hopr_api::chain::ChainWriteChannelOperations for HoprBlockchainConnector<B> {
    type Error = ConnectorError;

    async fn open_channel<'a>(&'a self, dst: &'a Address, amount: HoprBalance) -> Result<BoxFuture<'a, Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error> {
        let payload = self.payload_generator.fund_channel(*dst, amount)?;
        let signed_payload = sign_payload(payload, &self.chain_key).await?;
        todo!()
    }

    async fn fund_channel<'a>(&'a self, channel_id: &'a ChannelId, amount: HoprBalance) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error> {
        todo!()
    }

    async fn close_channel<'a>(&'a self, channel_id: &'a ChannelId) -> Result<BoxFuture<'a, Result<(ChannelStatus, ChainReceipt), Self::Error>>, Self::Error> {
        todo!()
    }
}