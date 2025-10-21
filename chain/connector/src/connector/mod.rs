use petgraph::prelude::StableDiGraph;
use hopr_crypto_types::prelude::ChainKeypair;
use hopr_internal_types::account::AccountEntry;
use hopr_internal_types::prelude::{ChannelEntry, ChannelId};
use crate::payload::SafePayloadGenerator;

mod channels;
mod accounts;
mod keys;
mod cache;

pub type AccountId = u32;

#[async_trait::async_trait]
pub trait OnchainDataStorage {
    type Error: std::error::Error + Send + Sync + 'static;
    async fn store_account(&mut self, id: &AccountId, entry: AccountEntry) -> Result<(), Self::Error>;
    async fn get_account(&self, id: &AccountId) -> Result<Option<AccountEntry>, Self::Error>;
    async fn delete_account(&mut self, id: &AccountId) -> Result<(), Self::Error>;
    async fn store_channel(&mut self, id: &ChannelId, entry: ChannelEntry) -> Result<(), Self::Error>;
    async fn get_channel(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;
    async fn delete_channel(&mut self, id: &ChannelId) -> Result<(), Self::Error>;
}

pub struct HoprBlockchainConnector<S> {
    payload_generator: SafePayloadGenerator,
    chain_key: ChainKeypair,
    graph: std::sync::Arc<parking_lot::Mutex<StableDiGraph<AccountId, ChannelId>>>,
    backend: std::sync::Arc<async_lock::Mutex<S>>
}

