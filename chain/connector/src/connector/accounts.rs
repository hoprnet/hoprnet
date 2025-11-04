use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, future::BoxFuture, stream::BoxStream};
use hopr_api::chain::{AccountSelector, AnnouncementError, ChainReceipt, Multiaddr};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::account::AccountEntry;
use hopr_primitive_types::prelude::*;

use crate::{
    backend::Backend,
    connector::{HoprBlockchainConnector, utils::track_transaction},
    errors::ConnectorError,
};

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainReadAccountOperations for HoprBlockchainConnector<B, C, P>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn node_balance<Cy: Currency>(&self) -> Result<Balance<Cy>, Self::Error> {
        self.check_connection_state()?;

        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_token_balance(&self.chain_key.public().to_address().into())
                .await?
                .balance
                .0
                .parse()?)
        } else if Cy::is::<XDai>() {
            Ok(self
                .client
                .query_native_balance(&self.chain_key.public().to_address().into())
                .await?
                .balance
                .0
                .parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_balance<Cy: Currency>(&self) -> Result<Balance<Cy>, Self::Error> {
        self.check_connection_state()?;

        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_token_balance(&self.safe_address.into())
                .await?
                .balance
                .0
                .parse()?)
        } else if Cy::is::<XDai>() {
            Ok(self
                .client
                .query_native_balance(&self.safe_address.into())
                .await?
                .balance
                .0
                .parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_allowance<Cy: Currency>(&self) -> Result<Balance<Cy>, Self::Error> {
        self.check_connection_state()?;

        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_safe_allowance(&self.safe_address.into())
                .await?
                .allowance
                .0
                .parse()?)
        } else if Cy::is::<XDai>() {
            Err(ConnectorError::InvalidState("cannot query allowance on xDai"))
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn stream_accounts<'a>(
        &'a self,
        selector: AccountSelector,
    ) -> Result<BoxStream<'a, AccountEntry>, Self::Error> {
        self.check_connection_state()?;

        let accounts = self.graph.read().nodes().collect::<Vec<_>>();
        let backend = self.backend.clone();
        Ok(futures::stream::iter(accounts)
            .filter_map(move |account_id| {
                let backend = backend.clone();
                let selector = selector.clone();
                // This avoids the cache on purpose so it does not get spammed
                async move {
                    match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_account_by_id(&account_id))
                        .await
                    {
                        Ok(Ok(value)) => value.filter(|c| selector.satisfies(c)),
                        Ok(Err(error)) => {
                            tracing::error!(%error, %account_id, "backend error when looking up account");
                            None
                        }
                        Err(error) => {
                            tracing::error!(%error, %account_id, "join error when looking up account");
                            None
                        }
                    }
                }
            })
            .boxed())
    }

    async fn count_accounts(&self, selector: AccountSelector) -> Result<usize, Self::Error> {
        self.check_connection_state()?;

        Ok(self.stream_accounts(selector).await?.count().await)
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainWriteAccountOperations for HoprBlockchainConnector<B, C, P>
where
    B: Send + Sync,
    C: BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn announce(
        &self,
        _multiaddrs: &[Multiaddr],
        _key: &OffchainKeypair,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>> {
        // self.check_connection_state()?;

        todo!()
    }

    async fn withdraw<Cy: Currency + Send>(
        &self,
        balance: Balance<Cy>,
        recipient: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let signed_payload = self
            .payload_generator
            .transfer(*recipient, balance)?
            .sign_and_encode_to_eip2718(&self.chain_key)
            .await?;

        let tx_id = self.client.submit_and_track_transaction(&signed_payload).await?;
        Ok(track_transaction(self.client.as_ref(), tx_id)?.boxed())
    }

    async fn register_safe(
        &self,
        safe_address: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let signed_payload = self
            .payload_generator
            .register_safe_by_node(*safe_address)?
            .sign_and_encode_to_eip2718(&self.chain_key)
            .await?;

        let tx_id = self.client.submit_and_track_transaction(&signed_payload).await?;
        Ok(track_transaction(self.client.as_ref(), tx_id)?.boxed())
    }
}
