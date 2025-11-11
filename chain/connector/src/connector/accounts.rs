use std::str::FromStr;

use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, future::BoxFuture, stream::BoxStream};
use hopr_api::chain::{AccountSelector, AnnouncementError, ChainReceipt, Multiaddr};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::{
    account::AccountEntry,
    prelude::{AnnouncementData, KeyBinding},
};
use hopr_primitive_types::prelude::*;

use crate::{backend::Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

#[async_trait::async_trait]
impl<B, C, P, R> hopr_api::chain::ChainReadAccountOperations for HoprBlockchainConnector<C, R, B, P>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + Send + Sync + 'static,
    P: Send + Sync + 'static,
    R: Send + Sync,
{
    type Error = ConnectorError;

    async fn get_balance<Cy: Currency, A: Into<Address> + Send>(&self, address: A) -> Result<Balance<Cy>, Self::Error> {
        self.check_connection_state()?;

        let address = address.into();
        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_token_balance(&address.into())
                .await?
                .balance
                .0
                .parse()?)
        } else if Cy::is::<XDai>() {
            Ok(self
                .client
                .query_native_balance(&address.into())
                .await?
                .balance
                .0
                .parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_allowance<Cy: Currency, A: Into<Address> + Send>(
        &self,
        address: A,
    ) -> Result<Balance<Cy>, Self::Error> {
        self.check_connection_state()?;

        let address = address.into();
        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_safe_allowance(&address.into())
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
impl<B, C, P> hopr_api::chain::ChainWriteAccountOperations for HoprBlockchainConnector<C, P::TxRequest, B, P>
where
    B: Send + Sync,
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainKeypair,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>> {
        self.check_connection_state()
            .map_err(|e| AnnouncementError::ProcessingError(e))?;

        let new_announced_addrs = ahash::HashSet::from_iter(multiaddrs.iter().map(|a| a.to_string()));

        let existing_account = self
            .client
            .query_accounts(blokli_client::api::v1::AccountSelector::Address(
                self.chain_key.public().to_address().into(),
            ))
            .await
            .map_err(|e| AnnouncementError::ProcessingError(ConnectorError::from(e)))?
            .into_iter()
            .find(|account| OffchainPublicKey::from_str(&account.packet_key).is_ok_and(|k| &k == key.public()));

        if let Some(account) = &existing_account {
            let old_announced_addrs = ahash::HashSet::from_iter(account.multi_addresses.iter().cloned());
            if old_announced_addrs == new_announced_addrs || old_announced_addrs.is_superset(&new_announced_addrs) {
                return Err(AnnouncementError::AlreadyAnnounced);
            }
        }

        let key_binding = KeyBinding::new(self.chain_key.public().to_address(), key);

        let tx_req = self
            .payload_generator
            .announce(
                AnnouncementData::new(key_binding, multiaddrs.first().cloned())
                    .map_err(|e| AnnouncementError::ProcessingError(ConnectorError::OtherError(e.into())))?,
                existing_account
                    .map(|_| HoprBalance::zero())
                    .unwrap_or(HoprBalance::from_str("0.01 wxHOPR").unwrap()),
            )
            .map_err(|e| AnnouncementError::ProcessingError(ConnectorError::from(e)))?;

        Ok(self
            .send_tx(tx_req)
            .map_err(AnnouncementError::ProcessingError)
            .await?
            .boxed())
    }

    async fn withdraw<Cy: Currency + Send>(
        &self,
        balance: Balance<Cy>,
        recipient: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let tx_req = self.payload_generator.transfer(*recipient, balance)?;

        Ok(self.send_tx(tx_req).await?.boxed())
    }

    async fn register_safe(
        &self,
        safe_address: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let tx_req = self.payload_generator.register_safe_by_node(*safe_address)?;

        Ok(self.send_tx(tx_req).await?.boxed())
    }
}
