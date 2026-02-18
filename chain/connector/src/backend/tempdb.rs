use hopr_api::chain::HoprKeyIdent;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::{
    account::AccountEntry,
    channels::{ChannelEntry, ChannelId},
};
use hopr_primitive_types::prelude::{Address, BytesRepresentable};
use redb::{ReadableDatabase, TableDefinition};

/// Errors from the temporary database backend.
#[derive(Debug, thiserror::Error)]
pub enum TempDbError {
    #[error("database error: {0}")]
    Database(#[from] redb::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<redb::DatabaseError> for TempDbError {
    fn from(error: redb::DatabaseError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::TransactionError> for TempDbError {
    fn from(error: redb::TransactionError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::TableError> for TempDbError {
    fn from(error: redb::TableError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::StorageError> for TempDbError {
    fn from(error: redb::StorageError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::CommitError> for TempDbError {
    fn from(error: redb::CommitError) -> Self {
        Self::Database(error.into())
    }
}

/// A backend that is implemented via [`redb`](https://docs.rs/redb/latest/redb/) database stored in a temporary file.
///
/// The database file is dropped once the last instance is dropped.
#[derive(Clone)]
pub struct TempDbBackend {
    // `db` is declared before `_tmp` so the database is closed before the temp file is deleted
    // (Rust drops fields in declaration order).
    db: std::sync::Arc<redb::Database>,
    _tmp: std::sync::Arc<tempfile::NamedTempFile>,
}

impl TempDbBackend {
    pub fn new() -> Result<Self, TempDbError> {
        let file = tempfile::NamedTempFile::new()?;

        tracing::info!(path = %file.path().display(), "opened temporary redb database");

        let db = redb::Database::create(file.path())?;

        // Create all tables eagerly so that read-only lookups on a fresh
        // database return Ok(None) instead of a TableError.
        {
            let write_tx = db.begin_write()?;
            write_tx.open_table(ACCOUNTS_TABLE_DEF)?;
            write_tx.open_table(CHANNELS_TABLE_DEF)?;
            write_tx.open_table(ADDRESS_TO_ID)?;
            write_tx.open_table(KEY_TO_ID)?;
            write_tx.commit()?;
        }

        Ok(Self {
            db: std::sync::Arc::new(db),
            _tmp: std::sync::Arc::new(file),
        })
    }
}

const ACCOUNTS_TABLE_DEF: TableDefinition<u32, Vec<u8>> = TableDefinition::new("id_accounts");
const CHANNELS_TABLE_DEF: TableDefinition<[u8; ChannelId::SIZE], Vec<u8>> = TableDefinition::new("id_channels");
const ADDRESS_TO_ID: TableDefinition<[u8; Address::SIZE], u32> = TableDefinition::new("address_to_id");
const KEY_TO_ID: TableDefinition<[u8; OffchainPublicKey::SIZE], u32> = TableDefinition::new("key_to_id");

impl super::Backend for TempDbBackend {
    type Error = TempDbError;

    fn insert_account(&self, account: AccountEntry) -> Result<Option<AccountEntry>, Self::Error> {
        let write_tx = self.db.begin_write()?;
        let old_value = {
            let mut accounts = write_tx.open_table(ACCOUNTS_TABLE_DEF)?;
            let old_value = accounts
                .insert(u32::from(account.key_id), postcard::to_allocvec(&account)?)?
                .map(|v| postcard::from_bytes::<AccountEntry>(&v.value()))
                .transpose()?;

            let mut address_to_id = write_tx.open_table(ADDRESS_TO_ID)?;
            let mut key_to_id = write_tx.open_table(KEY_TO_ID)?;

            // Remove old account entry references not to create stale mappings if keys changed
            if let Some(old_entry) = &old_value {
                let chain_addr: [u8; Address::SIZE] = old_entry.chain_addr.into();
                let packet_addr: [u8; OffchainPublicKey::SIZE] = old_entry.public_key.into();
                address_to_id.remove(&chain_addr)?;
                key_to_id.remove(&packet_addr)?;
            }

            let chain_addr: [u8; Address::SIZE] = account.chain_addr.into();
            address_to_id.insert(chain_addr, u32::from(account.key_id))?;

            let packet_addr: [u8; OffchainPublicKey::SIZE] = account.public_key.into();
            key_to_id.insert(packet_addr, u32::from(account.key_id))?;

            old_value
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
                .insert(channel_id, postcard::to_allocvec(&channel)?)?
                .map(|v| postcard::from_bytes::<ChannelEntry>(&v.value()))
                .transpose()?
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
            .map(|v| postcard::from_bytes::<AccountEntry>(&v.value()))
            .transpose()
            .map_err(TempDbError::from)
    }

    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error> {
        let read_tx = self.db.begin_read()?;
        let keys_to_id = read_tx.open_table(KEY_TO_ID)?;
        let packet_addr: [u8; OffchainPublicKey::SIZE] = (*key).into();
        let Some(id) = keys_to_id.get(packet_addr)?.map(|v| v.value()) else {
            return Ok(None);
        };
        let accounts = read_tx.open_table(ACCOUNTS_TABLE_DEF)?;
        accounts
            .get(id)?
            .map(|v| postcard::from_bytes::<AccountEntry>(&v.value()))
            .transpose()
            .map_err(TempDbError::from)
    }

    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error> {
        let read_tx = self.db.begin_read()?;
        let address_to_id = read_tx.open_table(ADDRESS_TO_ID)?;
        let chain_key: [u8; Address::SIZE] = (*chain_key).into();
        let Some(id) = address_to_id.get(chain_key)?.map(|v| v.value()) else {
            return Ok(None);
        };
        let accounts = read_tx.open_table(ACCOUNTS_TABLE_DEF)?;
        accounts
            .get(id)?
            .map(|v| postcard::from_bytes::<AccountEntry>(&v.value()))
            .transpose()
            .map_err(TempDbError::from)
    }

    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        let read_tx = self.db.begin_read()?;
        let channels = read_tx.open_table(CHANNELS_TABLE_DEF)?;
        let id: [u8; ChannelId::SIZE] = (*id).into();
        channels
            .get(id)?
            .map(|v| postcard::from_bytes::<ChannelEntry>(&v.value()))
            .transpose()
            .map_err(TempDbError::from)
    }
}

#[cfg(test)]
mod tests {
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_internal_types::{
        account::AccountType,
        channels::{ChannelStatus, generate_channel_id},
    };
    use hopr_primitive_types::balance::HoprBalance;

    use super::*;
    use crate::{Backend, backend::tests::test_backend};

    #[test]
    fn test_tempdb() -> anyhow::Result<()> {
        let backend = TempDbBackend::new()?;
        test_backend(backend)
    }

    #[test]
    fn upsert_cleans_up_stale_index_entries() -> anyhow::Result<()> {
        let backend = TempDbBackend::new()?;

        let kp_a = OffchainKeypair::random();
        let cp = ChainKeypair::random();

        let account = AccountEntry {
            public_key: (*kp_a.public()),
            chain_addr: cp.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: 1.into(),
        };
        backend.insert_account(account)?;

        // Update same ID with a different offchain key
        let kp_b = OffchainKeypair::random();
        let updated = AccountEntry {
            public_key: (*kp_b.public()),
            chain_addr: cp.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: 1.into(),
        };
        backend.insert_account(updated.clone())?;

        // Old key should no longer resolve
        assert!(backend.get_account_by_key(kp_a.public())?.is_none());
        // New key should resolve
        assert_eq!(backend.get_account_by_key(kp_b.public())?, Some(updated));

        Ok(())
    }

    #[test]
    fn upsert_cleans_up_stale_address_entries() -> anyhow::Result<()> {
        let backend = TempDbBackend::new()?;

        let kp = OffchainKeypair::random();
        let cp_a = ChainKeypair::random();

        let account = AccountEntry {
            public_key: (*kp.public()),
            chain_addr: cp_a.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: 1.into(),
        };
        backend.insert_account(account)?;

        // Update same ID with a different chain address
        let cp_b = ChainKeypair::random();
        let updated = AccountEntry {
            public_key: (*kp.public()),
            chain_addr: cp_b.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: 1.into(),
        };
        backend.insert_account(updated.clone())?;

        // Old address should no longer resolve
        assert!(backend.get_account_by_address(&cp_a.public().to_address())?.is_none());
        // New address should resolve
        assert_eq!(
            backend.get_account_by_address(&cp_b.public().to_address())?,
            Some(updated)
        );

        Ok(())
    }

    #[test]
    fn channel_upsert_returns_old_value() -> anyhow::Result<()> {
        let backend = TempDbBackend::new()?;

        let src = Address::new(&[1u8; 20]);
        let dst = Address::new(&[2u8; 20]);

        let channel_v1 = ChannelEntry::new(
            src,
            dst,
            HoprBalance::new_base(100),
            1u32.into(),
            ChannelStatus::Open,
            1u32,
        );

        let first_insert = backend.insert_channel(channel_v1)?;
        assert!(first_insert.is_none(), "first insert should return None");

        let channel_v2 = ChannelEntry::new(
            src,
            dst,
            HoprBalance::new_base(200),
            2u32.into(),
            ChannelStatus::Open,
            2u32,
        );

        let second_insert = backend.insert_channel(channel_v2)?;
        assert_eq!(second_insert, Some(channel_v1), "second insert should return old value");

        Ok(())
    }

    #[test]
    fn lookup_nonexistent_returns_none() -> anyhow::Result<()> {
        let backend = TempDbBackend::new()?;

        let kp = OffchainKeypair::random();
        let cp = ChainKeypair::random();
        let channel_id = generate_channel_id(&Address::new(&[1u8; 20]), &Address::new(&[2u8; 20]));

        assert!(backend.get_account_by_id(&42.into())?.is_none());
        assert!(backend.get_account_by_key(kp.public())?.is_none());
        assert!(backend.get_account_by_address(&cp.public().to_address())?.is_none());
        assert!(backend.get_channel_by_id(&channel_id)?.is_none());

        Ok(())
    }

    #[test]
    fn index_lookup_uses_single_transaction() -> anyhow::Result<()> {
        let backend = TempDbBackend::new()?;

        let kp = OffchainKeypair::random();
        let cp = ChainKeypair::random();

        let account = AccountEntry {
            public_key: (*kp.public()),
            chain_addr: cp.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: 5.into(),
        };
        backend.insert_account(account.clone())?;

        // Verify lookups via both index paths return the correct account
        let by_key = backend.get_account_by_key(kp.public())?;
        assert_eq!(by_key, Some(account.clone()));

        let by_addr = backend.get_account_by_address(&cp.public().to_address())?;
        assert_eq!(by_addr, Some(account));

        Ok(())
    }
}
