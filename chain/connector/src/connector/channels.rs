use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::{FutureExt, StreamExt};
use hopr_api::chain::{ChainReceipt, ChannelSelector};
use hopr_internal_types::channels::{ChannelEntry, ChannelId, ChannelStatus};
use hopr_primitive_types::balance::HoprBalance;
use hopr_primitive_types::prelude::Address;
use crate::connector::{HoprBlockchainConnector, OnchainDataStorage};
use crate::errors::ConnectorError;
use crate::payload::{sign_payload, PayloadGenerator};

#[async_trait::async_trait]
impl<T: OnchainDataStorage + Send + Sync> hopr_api::chain::ChainReadChannelOperations for  HoprBlockchainConnector<T> {
    type Error = ConnectorError;

    fn me(&self) -> &Address {
        todo!()
    }

    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        self.backend.lock()
    }

    async fn stream_channels<'a>(&'a self, selector: ChannelSelector) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        todo!()
    }
}

#[async_trait::async_trait]
impl<T: OnchainDataStorage + Send + Sync> hopr_api::chain::ChainWriteChannelOperations for HoprBlockchainConnector<T> {
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