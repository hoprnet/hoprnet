//! Persistent encrypted store for Exit-side [`PixEvent::PrivateKeyRecovered`] events.
//!
//! # Encryption scheme
//!
//! ```text
//! Config password  ──→  scrypt(password, static_salt)  ──→  256-bit key
//!                                                                  │
//!                                               ┌──────────────────┤
//!                                               │                  │
//!                                               ▼                  ▼
//! PixAddressId  ──→  encode_key(id)  ──→  first 12 bytes     ChaCha20Poly1305
//!                                           = deterministic       │
//!                                             nonce per entry      │
//!                                                                  │
//!                                       Plaintext (32 bytes)  ──→  │
//!                                       Ciphertext (48 bytes)  ←───┘
//! ```
//!
//! **Key derivation.**  When the store is opened, the config-supplied password
//! is fed through scrypt (RFC 7914) with recommended parameters (log₂_N = 17,
//! r = 8, p = 1) and a **static application salt** (`b"hopr-pix-recovery-v1"`).
//! Static salt is acceptable here because the derived key is used for
//! *local-disk encryption*, not password verification — the secrecy rests on
//! the password itself.
//!
//! **Per-entry nonce.**  The [`PixAddressId`] (10-byte pseudonym + 4-byte
//! SSA index) doubles as the database key and as the source of a deterministic
//! 12-byte nonce (first 12 bytes of the encoded key).  Since each `PixAddressId`
//! is unique, the nonce is unique — no per-entry random IV needs to be stored.
//!
//! **Authenticated encryption.**  Each 32-byte private key is encrypted with
//! ChaCha20Poly1305 (RFC 8439).  The 16-byte authentication tag is appended,
//! producing a 48-byte stored value.  Any tampering with the stored ciphertext
//! (bit flips, truncation, etc.) is detected on decryption and reported as
//! [`PixRecoveryStoreError::Decryption`].
//!
//! **Same plaintext, different IDs → different ciphertexts.**  Because the
//! nonce changes with every `PixAddressId`, even identical private keys produce
//! distinct ciphertexts under the same encryption key.
//!
//! ## Threat model
//!
//! This protects against an attacker who obtains read-only access to the
//! database file (e.g. via a filesystem backup, exfiltration, or stolen disk).
//! Without the config password they cannot derive the encryption key, and
//! without the key they cannot decrypt any entry.
//!
//! It does **not** protect against:
//! - An attacker who also has the config (the password comes from the same config file).
//! - An attacker who can execute code as the node process (they can read the derived key from process memory).
//! - Side-channel or fault-analysis attacks.
//!
//! ## Recovery on restart
//!
//! On node restart the store is iterated and any entry whose on-chain balance
//! is still non-zero is reprocessed, providing crash recovery.
//!
//! The store is only opened in the Exit role (when
//! [`NonAnonymousPixStrategyConfig::pix_recovery_db_path`] and
//! [`NonAnonymousPixStrategyConfig::pix_recovery_password`] are both set).

use std::{num::NonZeroU32, path::Path, sync::Arc};

use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::Aead};
use hopr_api::{
    node::{PixAddressId, PixDepositSecret},
    types::internal::prelude::HoprPseudonym,
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use scrypt::{Params as ScryptParams, scrypt};

// Key = pseudonym (10 bytes) + ssa_index (4 bytes)
const KEY_SIZE: usize = 10 + 4;

// Ciphertext = plaintext (32 bytes) + Poly1305 tag (16 bytes)
const VALUE_SIZE: usize = 32 + 16;

const PIX_RECOVERED_KEYS: TableDefinition<[u8; KEY_SIZE], [u8; VALUE_SIZE]> =
    TableDefinition::new("pix_recovered_keys");

/// Application-specific salt for scrypt.  Since the derived key is used for
/// local-disk encryption (not password storage), a static salt is sufficient —
/// the secrecy depends on the password, not the salt.
const PIX_KDF_SALT: &[u8] = b"hopr-pix-recovery-v1";

/// Persistent encrypted recovery key store backed by `redb`.
#[derive(Clone)]
pub struct PixRecoveryStore {
    db: Arc<Database>,
    encryption_key: [u8; 32],
}

/// Errors from the PIX recovery store.
#[derive(Debug, thiserror::Error)]
pub enum PixRecoveryStoreError {
    #[error("database error: {0}")]
    Database(#[from] redb::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("encryption error")]
    Encryption,
    #[error("decryption error")]
    Decryption,
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

// ── Key / nonce encoding ───────────────────────────────────────────────────
// Key: 10 bytes HoprPseudonym + 4 bytes big-endian NonZeroU32 → 14 bytes.
// Nonce: first 12 bytes of the encoded key (10 pseudonym + 2 SSA index).

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

fn derive_nonce(id: &PixAddressId) -> Nonce {
    let encoded = encode_key(id);
    // First 12 bytes of the unique DB key → deterministic nonce.
    *Nonce::from_slice(&encoded[..12])
}

fn derive_key(password: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    #[allow(deprecated)]
    scrypt(
        password.as_bytes(),
        PIX_KDF_SALT,
        &ScryptParams::recommended(),
        &mut key,
    )
    .expect("scrypt with 32-byte output must succeed");
    key
}

// ── Encryption helpers ─────────────────────────────────────────────────────

fn encrypt(key: &[u8; 32], id: &PixAddressId, plaintext: &[u8; 32]) -> Result<[u8; VALUE_SIZE], PixRecoveryStoreError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = derive_nonce(id);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|_| PixRecoveryStoreError::Encryption)?;
    // ChaCha20Poly1305 output = plaintext_len + 16-byte tag
    Ok(ciphertext
        .as_slice()
        .try_into()
        .expect("ChaCha20Poly1305 ciphertext is 48 bytes"))
}

fn decrypt(
    key: &[u8; 32],
    id: &PixAddressId,
    ciphertext: &[u8; VALUE_SIZE],
) -> Result<[u8; 32], PixRecoveryStoreError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = derive_nonce(id);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_| PixRecoveryStoreError::Decryption)?;
    Ok(plaintext
        .as_slice()
        .try_into()
        .expect("ChaCha20Poly1305 plaintext is 32 bytes"))
}

impl PixRecoveryStore {
    /// Open (or create) the `redb` database at `path` with encryption key derived from `password`.
    pub fn open(path: impl AsRef<Path>, password: &str) -> Result<Self, PixRecoveryStoreError> {
        let encryption_key = derive_key(password);

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
        Ok(Self {
            db: Arc::new(db),
            encryption_key,
        })
    }

    /// Check whether a key has already been persisted.
    pub fn contains(&self, id: &PixAddressId) -> Result<bool, PixRecoveryStoreError> {
        let key = encode_key(id);
        let read_tx = self.db.begin_read()?;
        let table = read_tx.open_table(PIX_RECOVERED_KEYS)?;
        Ok(table.get(key)?.is_some())
    }

    /// Insert an encrypted recovered key.  Returns `true` if the entry was newly
    /// inserted, `false` if the key was already present.
    pub fn insert(&self, id: &PixAddressId, secret: &PixDepositSecret) -> Result<bool, PixRecoveryStoreError> {
        let key = encode_key(id);
        let plaintext: [u8; 32] = secret.0.as_ref().try_into().expect("PixDepositSecret is 32 bytes");
        let ciphertext = encrypt(&self.encryption_key, id, &plaintext)?;

        let write_tx = self.db.begin_write()?;
        let was_inserted = {
            let mut table = write_tx.open_table(PIX_RECOVERED_KEYS)?;
            table.insert(key, ciphertext)?.is_none()
        };
        write_tx.commit()?;
        Ok(was_inserted)
    }

    /// Iterate all stored `(id, secret)` pairs, decrypting each entry.
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
            let ciphertext: [u8; VALUE_SIZE] = value_bytes.value();
            let plaintext = decrypt(&self.encryption_key, &id, &ciphertext)?;
            let secret = PixDepositSecret(plaintext.into());
            results.push((id, secret));
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto_random::Randomizable;

    use super::*;

    const TEST_PASSWORD: &str = "test-password-for-unit-tests";

    fn make_id(index: u32) -> PixAddressId {
        (HoprPseudonym::random(), NonZeroU32::new(index).unwrap())
    }

    fn make_secret(byte: u8) -> PixDepositSecret {
        PixDepositSecret([byte; 32].into())
    }

    fn open_temp_store() -> (PixRecoveryStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = PixRecoveryStore::open(dir.path().join("pix_recovery.db"), TEST_PASSWORD).unwrap();
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
        let store = PixRecoveryStore::open(&nested, TEST_PASSWORD).unwrap();
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
            let store = PixRecoveryStore::open(&path, TEST_PASSWORD).unwrap();
            store.insert(&id, &secret).unwrap();
        }

        // Re-open with same password and verify.
        let store = PixRecoveryStore::open(&path, TEST_PASSWORD).unwrap();
        assert!(store.contains(&id).unwrap());
        let entries = store.iter().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, id);
        assert_eq!(entries[0].1.0.as_ref(), secret.0.as_ref());
    }

    #[test]
    fn test_reopen_with_wrong_password_fails_decryption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pix.db");

        let id = make_id(7);
        let secret = make_secret(0x77);

        // Write with password A.
        {
            let store = PixRecoveryStore::open(&path, "password-a").unwrap();
            store.insert(&id, &secret).unwrap();
        }

        // Re-open with password B → iteration must fail decryption.
        let store = PixRecoveryStore::open(&path, "password-b").unwrap();
        assert!(store.contains(&id).unwrap(), "key should still be present");
        let result = store.iter();
        assert!(result.is_err(), "iter must fail with wrong password");
        assert!(
            matches!(result.unwrap_err(), PixRecoveryStoreError::Decryption),
            "expected Decryption error"
        );
    }

    #[test]
    fn test_key_uniqueness_per_pseudonym_and_index() {
        let (store, _dir) = open_temp_store();
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

    // ── Encryption-specific tests ──────────────────────────────────────────

    /// Helper: read the raw stored bytes for a given ID without decrypting.
    fn raw_stored_value(store: &PixRecoveryStore, id: &PixAddressId) -> Option<[u8; VALUE_SIZE]> {
        let key = encode_key(id);
        let read_tx = store.db.begin_read().unwrap();
        let table = read_tx.open_table(PIX_RECOVERED_KEYS).unwrap();
        table.get(key).unwrap().map(|g| g.value())
    }

    #[test]
    fn test_encrypted_ciphertext_differs_from_plaintext() {
        let (store, _dir) = open_temp_store();
        let id = make_id(1);
        let secret = make_secret(0xaa);
        assert!(store.insert(&id, &secret).unwrap());

        let stored = raw_stored_value(&store, &id).expect("should be present");
        assert_ne!(
            stored.as_slice(),
            secret.0.as_ref(),
            "ciphertext must differ from plaintext"
        );
    }

    #[test]
    fn test_same_plaintext_different_id_yields_different_ciphertext() {
        let (store, _dir) = open_temp_store();
        let id1 = make_id(1);
        let id2 = make_id(2);
        let secret = make_secret(0xaa);

        assert!(store.insert(&id1, &secret).unwrap());
        assert!(store.insert(&id2, &secret).unwrap());

        let c1 = raw_stored_value(&store, &id1).expect("entry 1");
        let c2 = raw_stored_value(&store, &id2).expect("entry 2");
        assert_ne!(
            c1, c2,
            "same plaintext under different IDs must produce different ciphertexts"
        );
    }

    #[test]
    fn test_tampered_ciphertext_fails_decryption() {
        let (store, _dir) = open_temp_store();
        let id = make_id(1);
        let secret = make_secret(0xaa);
        assert!(store.insert(&id, &secret).unwrap());

        // Read, tamper one byte of the ciphertext, write back directly.
        let mut corrupted = raw_stored_value(&store, &id).expect("should be present");
        corrupted[0] ^= 0xff; // flip all bits in the first byte

        // Write the corrupted ciphertext directly via raw redb.
        let key = encode_key(&id);
        let write_tx = store.db.begin_write().unwrap();
        {
            let mut table = write_tx.open_table(PIX_RECOVERED_KEYS).unwrap();
            table.insert(key, corrupted).unwrap();
        }
        write_tx.commit().unwrap();

        // iter() must detect the tampered AEAD tag and fail.
        let result = store.iter();
        assert!(
            matches!(result, Err(PixRecoveryStoreError::Decryption)),
            "tampered ciphertext must fail decryption"
        );
    }
}
