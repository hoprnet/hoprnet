use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt};
use futures::stream::BoxStream;
use hopr_api::chain::{AccountSelector, AnnouncementError, ChainReceipt, Multiaddr};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::account::AccountEntry;
use hopr_primitive_types::prelude::*;

use crate::connector::{Backend, HoprBlockchainConnector};
use crate::errors::ConnectorError;
use crate::payload::{sign_payload, PayloadGenerator};

#[async_trait::async_trait]
impl<B, C> hopr_api::chain::ChainReadAccountOperations for HoprBlockchainConnector<B, C>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + Send + Sync + 'static {
    type Error = ConnectorError;

    async fn node_balance<Cy: Currency>(&self) -> Result<Balance<Cy>, Self::Error> {
        if Cy::is::<WxHOPR>() {
            Ok(self.client.query_native_balance(&self.chain_key.public().to_address().into()).await?.balance.0.parse()?)
        } else if Cy::is::<XDai>() {
            Ok(self.client.query_token_balance(&self.chain_key.public().to_address().into()).await?.balance.0.parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_balance<Cy: Currency>(&self) -> Result<Balance<Cy>, Self::Error> {
        if Cy::is::<WxHOPR>() {
            Ok(self.client.query_native_balance(&self.safe_address.into()).await?.balance.0.parse()?)
        } else if Cy::is::<XDai>() {
            Ok(self.client.query_token_balance(&self.safe_address.into()).await?.balance.0.parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_allowance<Cy: Currency>(&self) -> Result<Balance<Cy>, Self::Error> {
        todo!()
    }

    async fn check_node_safe_module_status(&self) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn can_register_with_safe(&self, safe_address: &Address) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn stream_accounts<'a>(&'a self, selector: AccountSelector) -> Result<BoxStream<'a, AccountEntry>, Self::Error> {
        let accounts = self.graph.read().nodes().collect::<Vec<_>>();
        let backend = self.backend.clone();
        Ok(futures::stream::iter(accounts)
            .filter_map(move |account_id| {
                let backend = backend.clone();
                let selector = selector.clone();
                // This avoids the cache on purpose so it does not get spammed
                async move {
                    match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_account_by_id(&account_id)).await {
                        Ok(Ok(value)) => value.filter(|c| selector.satisfies(c)),
                        Ok(Err(error)) => {
                            tracing::error!(%error, %account_id, "backend error when looking up account");
                            None
                        },
                        Err(error) => {
                            tracing::error!(%error, %account_id, "join error when looking up account");
                            None
                        },
                    }
                }
            })
            .boxed())
    }

    async fn count_accounts(&self, selector: AccountSelector) -> Result<usize, Self::Error> {
        Ok(self.stream_accounts(selector).await?.count().await)
    }
}

#[async_trait::async_trait]
impl<B,C> hopr_api::chain::ChainWriteAccountOperations for HoprBlockchainConnector<B, C>
where
    B: Send + Sync,
    C: BlokliTransactionClient + Send + Sync + 'static {
    type Error = ConnectorError;

    async fn announce(&self, multiaddrs: &[Multiaddr], key: &OffchainKeypair) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>> {
        todo!()
    }

    async fn withdraw<Cy: Currency + Send>(&self, balance: Balance<Cy>, recipient: &Address) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        let payload = self.payload_generator.transfer(*recipient, balance)?;
        let signed_payload = sign_payload(payload, &self.chain_key).await?;
        let receipt = self.client.submit_transaction(&signed_payload).await.map_err(ConnectorError::from)?;
        Ok(futures::future::always_ready(move || Ok(ChainReceipt::from(receipt))).boxed())
    }

    async fn register_safe(&self, safe_address: &Address) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        todo!()
    }
}