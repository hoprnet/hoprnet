use hopr_api::chain::HoprKeyIdent;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::{
    account::AccountEntry,
    channels::{ChannelEntry, ChannelId},
};
use hopr_primitive_types::prelude::{Address, BytesRepresentable};
use redb::{ReadableDatabase, TableDefinition};

#[derive(Clone)]
pub struct TempDbBackend {
    db: std::sync::Arc<redb::Database>,
}

impl TempDbBackend {
    pub fn new() -> Result<Self, std::io::Error> {
        let file = tempfile::NamedTempFile::new().map_err(std::io::Error::other)?;

        Ok(Self {
            db: std::sync::Arc::new(redb::Database::create(file.path()).map_err(std::io::Error::other)?),
        })
    }
}

const ACCOUNTS_TABLE_DEF: TableDefinition<u32, Vec<u8>> = TableDefinition::new("id_accounts");
const CHANNELS_TABLE_DEF: TableDefinition<[u8; ChannelId::SIZE], Vec<u8>> = TableDefinition::new("id_channels");
const ADDRESS_TO_ID: TableDefinition<[u8; Address::SIZE], u32> = TableDefinition::new("address_to_id");
const KEY_TO_ID: TableDefinition<[u8; OffchainPublicKey::SIZE], u32> = TableDefinition::new("key_to_id");

const BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
    .with_little_endian()
    .with_variable_int_encoding();

impl super::Backend for TempDbBackend {
    type Error = redb::Error;

    fn insert_account(&self, account: AccountEntry) -> Result<Option<AccountEntry>, Self::Error> {
        let write_tx = self.db.begin_write()?;
        let old_value = {
            let mut address_to_id = write_tx.open_table(ADDRESS_TO_ID)?;
            let chain_addr: [u8; Address::SIZE] = account.chain_addr.into();
            address_to_id.insert(chain_addr, u32::from(account.key_id))?;

            let mut key_to_id = write_tx.open_table(KEY_TO_ID)?;
            let packet_addr: [u8; OffchainPublicKey::SIZE] = account.public_key.into();
            key_to_id.insert(packet_addr, u32::from(account.key_id))?;

            let mut accounts = write_tx.open_table(ACCOUNTS_TABLE_DEF)?;
            accounts
                .insert(
                    u32::from(account.key_id),
                    bincode::serde::encode_to_vec(&account, BINCODE_CONFIGURATION)
                        .map_err(|e| redb::Error::Corrupted(format!("account encoding failed: {e}")))?,
                )?
                .map(|v| {
                    bincode::serde::decode_from_slice::<AccountEntry, _>(&v.value(), BINCODE_CONFIGURATION).map(|v| v.0)
                })
                .transpose()
                .map_err(|e| redb::Error::Corrupted(format!("account decoding failed: {e}")))?
        };
        write_tx.commit()?;

        tracing::debug!(new = %account, old = ?old_value, "upserted account");
        Ok(old_value)
    }

    fn insert_channel(&self, channel: ChannelEntry) -> Result<Option<ChannelEntry>, Self::Error> {
        let write_tx = self.db.begin_write()?;
        let old_value = {
            let mut channels = write_tx.open_table(CHANNELS_TABLE_DEF)?;
            let channel_id: [u8; ChannelId::SIZE] = channel.get_id().into();
            channels
                .insert(
                    channel_id,
                    bincode::serde::encode_to_vec(channel, BINCODE_CONFIGURATION)
                        .map_err(|e| redb::Error::Corrupted(format!("channel encoding failed: {e}")))?,
                )?
                .map(|v| {
                    bincode::serde::decode_from_slice::<ChannelEntry, _>(&v.value(), BINCODE_CONFIGURATION).map(|v| v.0)
                })
                .transpose()
                .map_err(|e| redb::Error::Corrupted(format!("account decoding failed: {e}")))?
        };
        write_tx.commit()?;

        tracing::debug!(new = %channel, old = ?old_value, "upserted channel");
        Ok(old_value)
    }

    fn get_account_by_id(&self, id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error> {
        let read_tx = self.db.begin_read()?;
        let accounts = read_tx.open_table(ACCOUNTS_TABLE_DEF)?;
        accounts
            .get(u32::from(*id))?
            .map(|v| {
                bincode::serde::decode_from_slice::<AccountEntry, _>(&v.value(), BINCODE_CONFIGURATION).map(|v| v.0)
            })
            .transpose()
            .map_err(|e| redb::Error::Corrupted(format!("account decoding failed: {e}")))
    }

    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error> {
        let id = {
            let read_tx = self.db.begin_read()?;
            let keys_to_id = read_tx.open_table(KEY_TO_ID)?;
            let packet_addr: [u8; OffchainPublicKey::SIZE] = (*key).into();
            let Some(id) = keys_to_id.get(packet_addr)?.map(|v| v.value()) else {
                return Ok(None);
            };
            id
        };

        self.get_account_by_id(&id.into())
    }

    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error> {
        let id = {
            let read_tx = self.db.begin_read()?;
            let address_to_id = read_tx.open_table(ADDRESS_TO_ID)?;
            let chain_key: [u8; Address::SIZE] = (*chain_key).into();
            let Some(id) = address_to_id.get(chain_key)?.map(|v| v.value()) else {
                return Ok(None);
            };
            id
        };

        self.get_account_by_id(&id.into())
    }

    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        let read_tx = self.db.begin_read()?;
        let accounts = read_tx.open_table(CHANNELS_TABLE_DEF)?;
        let id: [u8; ChannelId::SIZE] = (*id).into();
        accounts
            .get(id)?
            .map(|v| {
                bincode::serde::decode_from_slice::<ChannelEntry, _>(&v.value(), BINCODE_CONFIGURATION).map(|v| v.0)
            })
            .transpose()
            .map_err(|e| redb::Error::Corrupted(format!("channel decoding failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::tests::test_backend;

    #[test]
    fn test_tempdb() -> anyhow::Result<()> {
        let backend = TempDbBackend::new()?;
        test_backend(backend)
    }
}
