use hopr_internal_types::account::AccountEntry;
use hopr_internal_types::channels::{ChannelEntry, ChannelId};
use crate::connector::{AccountId, OnchainDataStorage};

pub struct CachedDataStorage<B> {
    backend: std::sync::Arc<async_lock::Mutex<B>>,
    channels: moka::future::Cache<ChannelId, ChannelEntry>,
    accounts: moka::future::Cache<AccountId, AccountEntry>,
}

#[async_trait::async_trait]
impl<B: OnchainDataStorage + Send + Sync>  OnchainDataStorage for CachedDataStorage<B> {
    type Error = B::Error;

    async fn store_account(&mut self, id: &AccountId, entry: AccountEntry) -> Result<(), Self::Error> {
        todo!()
    }

    async fn get_account(&self, id: &AccountId) -> Result<Option<AccountEntry>, Self::Error> {
        todo!()
    }

    async fn delete_account(&mut self, id: &AccountId) -> Result<(), Self::Error> {
        todo!()
    }

    async fn store_channel(&mut self, id: &ChannelId, entry: ChannelEntry) -> Result<(), Self::Error> {
        todo!()
    }

    async fn get_channel(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        todo!()
    }

    async fn delete_channel(&mut self, id: &ChannelId) -> Result<(), Self::Error> {
        todo!()
    }
}