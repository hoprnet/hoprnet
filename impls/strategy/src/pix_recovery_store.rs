//! Persistent store for Exit-side [`PixEvent::PrivateKeyRecovered`] events.
//!
//! Recovered private keys are written to a `redb` database before the
//! withdrawal transaction is submitted.  On node restart the store is
//! iterated and any entry whose on-chain balance is still non-zero is
//! reprocessed, providing crash recovery.
//!
//! The store is only opened in the Exit role (when
//! [`NonAnonymousPixStrategyConfig::pix_recovery_db_path`] is set).

use std::{num::NonZeroU32, path::Path, sync::Arc};

use hopr_api::{
    node::{PixAddressId, PixDepositSecret},
    types::internal::prelude::HoprPseudonym,
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

// Key = pseudonym (10 bytes) + ssa_index (4 bytes)
const KEY_SIZE: usize = 10 + 4;

const PIX_RECOVERED_KEYS: TableDefinition<[u8; KEY_SIZE], [u8; 32]> = TableDefinition::new("pix_recovered_keys");

/// Persistent recovery key store backed by `redb`.
#[derive(Clone)]
pub struct PixRecoveryStore {
    db: Arc<Database>,
}

/// Errors from the PIX recovery store.
#[derive(Debug, thiserror::Error)]
pub enum PixRecoveryStoreError {
    #[error("database error: {0}")]
    Database(#[from] redb::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<redb::DatabaseError> for PixRecoveryStoreError {
    fn from(error: redb::DatabaseError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::TransactionError> for PixRecoveryStoreError {
    fn from(error: redb::TransactionError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::TableError> for PixRecoveryStoreError {
    fn from(error: redb::TableError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::StorageError> for PixRecoveryStoreError {
    fn from(error: redb::StorageError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::CommitError> for PixRecoveryStoreError {
    fn from(error: redb::CommitError) -> Self {
        Self::Database(error.into())
    }
}

// ── Key encoding ─────────────────────────────────────────────────────────────
// Key = 32 bytes HoprPseudonym + 4 bytes big-endian NonZeroU32 → 36 bytes.

fn encode_key(id: &PixAddressId) -> [u8; KEY_SIZE] {
    let (pseudonym, ssa_index) = id;
    let pseudonym_bytes: &[u8] = pseudonym.as_ref();
    let ssa_bytes = ssa_index.get().to_be_bytes();
    let mut out = [0u8; KEY_SIZE];
    out[..10].copy_from_slice(pseudonym_bytes);
    out[10..].copy_from_slice(&ssa_bytes);
    out
}

fn decode_key(bytes: &[u8; KEY_SIZE]) -> PixAddressId {
    let pseudonym = HoprPseudonym::from(<[u8; 10]>::try_from(&bytes[..10]).expect("pix key: pseudonym slice"));
    let ssa_index = NonZeroU32::new(u32::from_be_bytes(
        <[u8; 4]>::try_from(&bytes[10..14]).expect("pix key: ssa index slice"),
    ))
    .expect("pix key: non-zero ssa index");
    (pseudonym, ssa_index)
}

impl PixRecoveryStore {
    /// Open (or create) the `redb` database at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PixRecoveryStoreError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(path.as_ref())?;
        // Create the table eagerly so read-only operations never fail on a fresh db.
        {
            let write_tx = db.begin_write()?;
            write_tx.open_table(PIX_RECOVERED_KEYS)?;
            write_tx.commit()?;
        }
        Ok(Self { db: Arc::new(db) })
    }

    /// Check whether a key has already been persisted.
    pub fn contains(&self, id: &PixAddressId) -> Result<bool, PixRecoveryStoreError> {
        let key = encode_key(id);
        let read_tx = self.db.begin_read()?;
        let table = read_tx.open_table(PIX_RECOVERED_KEYS)?;
        Ok(table.get(key)?.is_some())
    }

    /// Insert a recovered key.  Returns `true` if the entry was newly inserted,
    /// `false` if the key was already present.
    pub fn insert(&self, id: &PixAddressId, secret: &PixDepositSecret) -> Result<bool, PixRecoveryStoreError> {
        let key = encode_key(id);
        // SecretValue<U32> is 32 bytes — convert to fixed-size array
        let value: [u8; 32] = secret.0.as_ref().try_into().expect("PixDepositSecret is 32 bytes");
        let write_tx = self.db.begin_write()?;
        let was_inserted = {
            let mut table = write_tx.open_table(PIX_RECOVERED_KEYS)?;
            table.insert(key, value)?.is_none()
        };
        write_tx.commit()?;
        Ok(was_inserted)
    }

    /// Iterate all stored `(id, secret)` pairs, collected into a `Vec`.
    ///
    /// On startup the caller uses this to find entries whose on-chain balance
    /// is still non-zero and re-submit the withdrawal.
    pub fn iter(&self) -> Result<Vec<(PixAddressId, PixDepositSecret)>, PixRecoveryStoreError> {
        let read_tx = self.db.begin_read()?;
        let table = read_tx.open_table(PIX_RECOVERED_KEYS)?;
        let mut results = Vec::new();
        for result in table.iter()? {
            let (key_bytes, value_bytes) = result?;
            let id = decode_key(&key_bytes.value());
            let secret_bytes: [u8; 32] = value_bytes.value();
            let secret = PixDepositSecret(secret_bytes.into());
            results.push((id, secret));
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto_random::Randomizable;

    use super::*;

    fn make_id(index: u32) -> PixAddressId {
        (HoprPseudonym::random(), NonZeroU32::new(index).unwrap())
    }

    fn make_secret(byte: u8) -> PixDepositSecret {
        PixDepositSecret([byte; 32].into())
    }

    fn open_temp_store() -> (PixRecoveryStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = PixRecoveryStore::open(dir.path().join("pix_recovery.db")).unwrap();
        (store, dir)
    }

    #[test]
    fn test_open_creates_database_file() {
        let (_store, dir) = open_temp_store();
        assert!(dir.path().join("pix_recovery.db").exists());
    }

    #[test]
    fn test_open_with_missing_parent_creates_directories() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("pix.db");
        let store = PixRecoveryStore::open(&nested).unwrap();
        assert!(nested.exists());
        drop(store); // must not panic when dropped
    }

    #[test]
    fn test_insert_returns_true_for_new_entry() {
        let (store, _dir) = open_temp_store();
        let id = make_id(1);
        let secret = make_secret(0xaa);
        assert!(store.insert(&id, &secret).unwrap());
    }

    #[test]
    fn test_insert_returns_false_for_duplicate() {
        let (store, _dir) = open_temp_store();
        let id = make_id(1);
        let secret = make_secret(0xaa);
        assert!(store.insert(&id, &secret).unwrap());
        assert!(!store.insert(&id, &secret).unwrap());
    }

    #[test]
    fn test_contains_empty_store() {
        let (store, _dir) = open_temp_store();
        assert!(!store.contains(&make_id(1)).unwrap());
    }

    #[test]
    fn test_contains_after_insert() {
        let (store, _dir) = open_temp_store();
        let id = make_id(1);
        store.insert(&id, &make_secret(0xbb)).unwrap();
        assert!(store.contains(&id).unwrap());
    }

    #[test]
    fn test_iter_empty() {
        let (store, _dir) = open_temp_store();
        let entries = store.iter().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_iter_returns_inserted_entries() {
        let (store, _dir) = open_temp_store();
        let id1 = make_id(1);
        let id2 = make_id(2);
        let sec1 = make_secret(0x11);
        let sec2 = make_secret(0x22);

        assert!(store.insert(&id1, &sec1).unwrap());
        assert!(store.insert(&id2, &sec2).unwrap());

        let entries = store.iter().unwrap();
        assert_eq!(entries.len(), 2);

        // Verify both entries are present with correct data (order not guaranteed).
        assert!(
            entries
                .iter()
                .any(|(id, s)| id == &id1 && s.0.as_ref() == sec1.0.as_ref())
        );
        assert!(
            entries
                .iter()
                .any(|(id, s)| id == &id2 && s.0.as_ref() == sec2.0.as_ref())
        );
    }

    #[test]
    fn test_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pix.db");

        let id = make_id(7);
        let secret = make_secret(0x77);

        // Write and close.
        {
            let store = PixRecoveryStore::open(&path).unwrap();
            store.insert(&id, &secret).unwrap();
        }

        // Re-open and verify.
        let store = PixRecoveryStore::open(&path).unwrap();
        assert!(store.contains(&id).unwrap());
        let entries = store.iter().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, id);
        assert_eq!(entries[0].1.0.as_ref(), secret.0.as_ref());
    }

    #[test]
    fn test_key_uniqueness_per_pseudonym_and_index() {
        let (store, _dir) = open_temp_store();
        // Same pseudonym, different SSA indices → distinct entries.
        let pseudo = HoprPseudonym::random();
        let id_a = (pseudo, NonZeroU32::new(1).unwrap());
        let id_b = (pseudo, NonZeroU32::new(2).unwrap());
        let sec_a = make_secret(0xaa);
        let sec_b = make_secret(0xbb);

        assert!(store.insert(&id_a, &sec_a).unwrap());
        assert!(store.insert(&id_b, &sec_b).unwrap());
        assert!(store.contains(&id_a).unwrap());
        assert!(store.contains(&id_b).unwrap());
        assert_eq!(store.iter().unwrap().len(), 2);
    }
}
