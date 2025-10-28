use hopr_api::chain::HoprKeyIdent;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::Address;

pub trait Backend {
    type Error: std::error::Error + Send + Sync + 'static;

    fn insert_account(&self, entry: AccountEntry) -> Result<(), Self::Error>;

    fn insert_channel(&self, channel: ChannelEntry) -> Result<(), Self::Error>;

    fn get_account_by_id(&self, id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error>;

    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error>;

    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error>;

    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;
}