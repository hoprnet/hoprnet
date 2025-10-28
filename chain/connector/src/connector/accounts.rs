use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::stream::BoxStream;
use hopr_api::chain::{AccountSelector, AnnouncementError, ChainReceipt, Multiaddr};
use hopr_crypto_types::keypairs::OffchainKeypair;
use hopr_crypto_types::prelude::{Keypair, OffchainPublicKey};
use hopr_internal_types::account::AccountEntry;
use hopr_primitive_types::prelude::*;
use crate::connector::HoprBlockchainConnector;
use crate::errors::ConnectorError;
use crate::payload::{sign_payload, PayloadGenerator};

#[async_trait::async_trait]
impl<B> hopr_api::chain::ChainReadAccountOperations for HoprBlockchainConnector<B> {
    type Error = ConnectorError;

    async fn node_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error> {
        if C::is::<WxHOPR>() {
            Ok(self.client.query_native_balance(&self.chain_key.public().to_address().into()).await?.balance.0.parse()?)
        } else if C::is::<XDai>() {
            Ok(self.client.query_token_balance(&self.chain_key.public().to_address().into()).await?.balance.0.parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error> {
        if C::is::<WxHOPR>() {
            Ok(self.client.query_native_balance(&self.safe_address.into()).await?.balance.0.parse()?)
        } else if C::is::<XDai>() {
            Ok(self.client.query_token_balance(&self.safe_address.into()).await?.balance.0.parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_allowance<C: Currency>(&self) -> Result<Balance<C>, Self::Error> {
        todo!()
    }

    async fn find_account_by_address(&self, address: &Address) -> Result<Option<AccountEntry>, Self::Error> {
        todo!()
    }

    async fn find_account_by_packet_key(&self, packet_key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error> {
        todo!()
    }

    async fn check_node_safe_module_status(&self) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn can_register_with_safe(&self, safe_address: &Address) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn stream_accounts<'a>(&'a self, selector: AccountSelector) -> Result<BoxStream<'a, AccountEntry>, Self::Error> {
        todo!()
    }

    async fn count_accounts(&self, selector: AccountSelector) -> Result<usize, Self::Error> {
        todo!()
    }
}

#[async_trait::async_trait]
impl<B> hopr_api::chain::ChainWriteAccountOperations for HoprBlockchainConnector<B> {
    type Error = ConnectorError;

    async fn announce(&self, multiaddrs: &[Multiaddr], key: &OffchainKeypair) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>> {
        todo!()
    }

    async fn withdraw<C: Currency + Send>(&self, balance: Balance<C>, recipient: &Address) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        let payload = self.payload_generator.transfer(*recipient, balance)?;
        let signed_payload = sign_payload(payload, &self.chain_key).await?;
        let receipt = self.client.submit_transaction(&signed_payload).await.map_err(ConnectorError::from)?;
        Ok(futures::future::always_ready(move || Ok(ChainReceipt::from(receipt))).boxed())
    }

    async fn register_safe(&self, safe_address: &Address) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        todo!()
    }
}