mod tempdb;

pub use hopr_api::chain::HoprKeyIdent;
pub use hopr_crypto_types::prelude::OffchainPublicKey;
pub use hopr_internal_types::{
    account::AccountEntry,
    channels::{ChannelEntry, ChannelId},
};
pub use hopr_primitive_types::prelude::Address;

/// Represents a cache backend for the connector.
pub trait Backend {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Inserts an [`AccountEntry`] into the backend.
    ///
    /// Returns the old value if one was present in the backend before.
    fn insert_account(&self, entry: AccountEntry) -> Result<Option<AccountEntry>, Self::Error>;
    /// Inserts a [`ChannelEntry`] into the backend.
    ///
    /// Returns the old value if one was present in the backend before.
    fn insert_channel(&self, channel: ChannelEntry) -> Result<Option<ChannelEntry>, Self::Error>;
    /// Retrieves an [`AccountEntry`] by [`HoprKeyIdent`].
    fn get_account_by_id(&self, id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error>;
    /// Retrieves an [`AccountEntry`] by [`OffchainPublicKey`].
    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error>;
    /// Retrieves an [`AccountEntry`] by an on-chain [`Address`].
    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error>;
    /// Retrieves a [`ChannelEntry`] by its [`ChannelId`].
    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;
}

pub use tempdb::TempDbBackend;

/// Represents a backend that stores all data in-memory.
///
/// This is useful mainly for testing.
#[cfg(any(test, feature = "testing"))]
#[derive(Clone)]
pub struct InMemoryBackend {
    accounts: std::sync::Arc<dashmap::DashMap<HoprKeyIdent, AccountEntry, ahash::RandomState>>,
    channels: std::sync::Arc<dashmap::DashMap<ChannelId, ChannelEntry, ahash::RandomState>>,
}

#[cfg(any(test, feature = "testing"))]
const DEFAULT_INMEMORY_BACKEND_CAPACITY: usize = 1024;

#[cfg(any(test, feature = "testing"))]
impl Default for InMemoryBackend {
    fn default() -> Self {
        Self {
            accounts: dashmap::DashMap::with_capacity_and_hasher(
                DEFAULT_INMEMORY_BACKEND_CAPACITY,
                ahash::RandomState::default(),
            )
            .into(),
            channels: dashmap::DashMap::with_capacity_and_hasher(
                DEFAULT_INMEMORY_BACKEND_CAPACITY,
                ahash::RandomState::default(),
            )
            .into(),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
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
        Ok(self
            .accounts
            .iter()
            .find(|account| &account.public_key == key)
            .map(|account| account.value().clone()))
    }

    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error> {
        Ok(self
            .accounts
            .iter()
            .find(|account| &account.chain_addr == chain_key)
            .map(|account| account.value().clone()))
    }

    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        Ok(self.channels.get(id).map(|e| e.value().clone()))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_internal_types::{
        account::{AccountEntry, AccountType},
        channels::{ChannelEntry, ChannelStatus, generate_channel_id},
    };
    use hopr_primitive_types::{balance::HoprBalance, prelude::Address};

    use crate::{Backend, InMemoryBackend};

    pub(crate) fn test_backend<B: Backend>(backend: B) -> anyhow::Result<()> {
        let kp = OffchainKeypair::random();
        let cp = ChainKeypair::random();

        let account = AccountEntry {
            public_key: (*kp.public()).into(),
            chain_addr: cp.public().to_address(),
            entry_type: AccountType::Announced("/ip4/1.2.3.4/tcp/1234".parse()?),
            safe_address: Some(Address::new(&[3u8; 32])),
            key_id: 3.into(),
        };

        let src = Address::new(&[1u8; 20]);
        let dst = Address::new(&[2u8; 20]);

        let channel = ChannelEntry::new(
            src,
            dst,
            HoprBalance::new_base(1000),
            10u32.into(),
            ChannelStatus::PendingToClose(std::time::SystemTime::now()),
            10u32.into(),
        );

        backend.insert_account(account.clone())?;
        backend.insert_channel(channel.clone())?;

        let a1 = backend
            .get_account_by_id(&account.key_id)?
            .ok_or(anyhow::anyhow!("account not found"))?;
        let a2 = backend
            .get_account_by_key(&kp.public())?
            .ok_or(anyhow::anyhow!("account not found"))?;
        let a3 = backend
            .get_account_by_address(cp.public().as_ref())?
            .ok_or(anyhow::anyhow!("account not found"))?;

        assert_eq!(a1, account);
        assert_eq!(a2, account);
        assert_eq!(a3, account);

        let id = generate_channel_id(&src, &dst);
        let c1 = backend
            .get_channel_by_id(&id)?
            .ok_or(anyhow::anyhow!("channel not found"))?;

        assert_eq!(c1, channel);

        Ok(())
    }

    #[test]
    fn test_inmemory() -> anyhow::Result<()> {
        let backend = InMemoryBackend::default();
        test_backend(backend)
    }
}
