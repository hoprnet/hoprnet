mod tempdb;

use hopr_api::chain::HoprKeyIdent;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::account::AccountEntry;
use hopr_internal_types::channels::{ChannelEntry, ChannelId};
use hopr_primitive_types::prelude::Address;

pub trait Backend {
    type Error: std::error::Error + Send + Sync + 'static;
    fn insert_account(&self, entry: AccountEntry) -> Result<Option<AccountEntry>, Self::Error>;
    fn insert_channel(&self, channel: ChannelEntry) -> Result<Option<ChannelEntry>, Self::Error>;
    fn get_account_by_id(&self, id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error>;
    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error>;
    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error>;
    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;
}

pub use tempdb::TempDbBackend;

#[derive(Clone)]
pub struct InMemoryBackend {
    accounts: std::sync::Arc<dashmap::DashMap<HoprKeyIdent, AccountEntry, ahash::RandomState>>,
    channels: std::sync::Arc<dashmap::DashMap<ChannelId, ChannelEntry, ahash::RandomState>>,
}

const DEFAULT_INMEMORY_BACKEND_CAPACITY: usize = 1024;

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self {
            accounts: dashmap::DashMap::with_capacity_and_hasher(DEFAULT_INMEMORY_BACKEND_CAPACITY, ahash::RandomState::default()).into(),
            channels: dashmap::DashMap::with_capacity_and_hasher(DEFAULT_INMEMORY_BACKEND_CAPACITY, ahash::RandomState::default()).into(),
        }
    }
}

impl Backend for InMemoryBackend {
    type Error = std::convert::Infallible;

    fn insert_account(&self, entry: AccountEntry) -> Result<Option<AccountEntry>, Self::Error> {
        Ok(self.accounts.insert(entry.key_id, entry))
    }

    fn insert_channel(&self, channel: ChannelEntry) -> Result<Option<ChannelEntry>, Self::Error> {
        Ok(self.channels.insert(channel.get_id(), channel))
    }

    fn get_account_by_id(&self, id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error> {
        Ok(self.accounts.get(id).map(|e| e.value().clone()))
    }

    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error> {
        Ok(self.accounts.iter().find(|account| &account.public_key == key).map(|account| account.value().clone()))
    }

    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error> {
        Ok(self.accounts.iter().find(|account| &account.chain_addr == chain_key).map(|account| account.value().clone()))
    }

    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        Ok(self.channels.get(id).map(|e| e.value().clone()))
    }
}