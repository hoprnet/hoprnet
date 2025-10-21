use futures::future::BoxFuture;
use futures::stream::BoxStream;
use multiaddr::Multiaddr;
use hopr_api::chain::{AccountSelector, AnnouncementError, ChainReceipt};
use hopr_crypto_types::keypairs::OffchainKeypair;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::account::AccountEntry;
use hopr_primitive_types::balance::{Balance, Currency};
use hopr_primitive_types::prelude::Address;
use crate::connector::{HoprBlockchainConnector, OnchainDataStorage};
use crate::errors::ConnectorError;

#[async_trait::async_trait]
impl<T: OnchainDataStorage + Send + Sync> hopr_api::chain::ChainReadAccountOperations for HoprBlockchainConnector<T> {
    type Error = ConnectorError;

    async fn node_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error> {
        todo!()
    }

    async fn safe_balance<C: Currency>(&self) -> Result<Balance<C>, Self::Error> {
        todo!()
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
impl<T: OnchainDataStorage + Send + Sync> hopr_api::chain::ChainWriteAccountOperations for HoprBlockchainConnector<T> {
    type Error = ConnectorError;

    async fn announce(&self, multiaddrs: &[Multiaddr], key: &OffchainKeypair) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>> {
        todo!()
    }

    async fn withdraw<C: Currency + Send>(&self, balance: Balance<C>, recipient: &Address) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        todo!()
    }

    async fn register_safe(&self, safe_address: &Address) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        todo!()
    }
}