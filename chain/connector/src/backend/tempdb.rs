use hopr_api::chain::HoprKeyIdent;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::account::AccountEntry;
use hopr_internal_types::channels::{ChannelEntry, ChannelId};
use hopr_primitive_types::prelude::Address;

#[derive(Clone)]
pub struct TempDbBackend {
    db: std::sync::Arc<redb::Database>
}

impl TempDbBackend {
    pub fn new() -> Result<Self, std::io::Error> {
        let file = tempfile::NamedTempFile::new().map_err(std::io::Error::other)?;

        Ok(Self {
            db: std::sync::Arc::new(redb::Database::create(file.path()).map_err(std::io::Error::other)?),
        })

    }
}

impl super::Backend for TempDbBackend {
    type Error = redb::Error;

    fn insert_account(&self, entry: AccountEntry) -> Result<Option<AccountEntry>, Self::Error> {
        todo!()
    }

    fn insert_channel(&self, channel: ChannelEntry) -> Result<Option<ChannelEntry>, Self::Error> {
        todo!()
    }

    fn get_account_by_id(&self, id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error> {
        todo!()
    }

    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error> {
        todo!()
    }

    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error> {
        todo!()
    }

    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        todo!()
    }
}