use crate::{
    errors::{KeyPairError, Result},
    keystore::{CipherparamsJson, CryptoJson, EthKeystore, KdfType, KdfparamsType, PrivateKeys},
};
use aes::{
    cipher::{self, InnerIvInit, KeyInit, StreamCipherCore},
    Aes128,
};
use hex;
use hopr_crypto_random::random_bytes;
use hopr_crypto_types::prelude::*;
use hopr_platform::file::native::{metadata, read_to_string, write};
use hopr_primitive_types::prelude::*;
use scrypt::{scrypt, Params as ScryptParams};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use serde_json::{from_str as from_json_string, to_string as to_json_string};
use sha3::{digest::Update, Digest, Keccak256};
use std::fmt::Debug;
use tracing::{error, info, warn};
use typenum::Unsigned;
use uuid::Uuid;

const HOPR_CIPHER: &str = "aes-128-ctr";
const HOPR_KEY_SIZE: usize = 32usize;
const HOPR_IV_SIZE: usize = 16usize;
const HOPR_KDF_PARAMS_DKLEN: u8 = 32u8;
const HOPR_KDF_PARAMS_LOG_N: u8 = 13u8;
const HOPR_KDF_PARAMS_R: u32 = 8u32;
const HOPR_KDF_PARAMS_P: u32 = 1u32;

const PACKET_KEY_LENGTH: usize = <OffchainKeypair as Keypair>::SecretLen::USIZE;
const CHAIN_KEY_LENGTH: usize = <ChainKeypair as Keypair>::SecretLen::USIZE;

const V1_PRIVKEY_LENGTH: usize = 32;
const V2_PRIVKEYS_LENGTH: usize = 172;

// Current version, deviates from pre 2.0
const VERSION: u32 = 2;

#[cfg(any(debug_assertions, test))]
const USE_WEAK_CRYPTO: bool = true;

#[cfg(all(not(debug_assertions), not(test)))]
const USE_WEAK_CRYPTO: bool = false;

struct Aes128Ctr {
    inner: ctr::CtrCore<Aes128, ctr::flavors::Ctr128BE>,
}

impl Aes128Ctr {
    fn new(key: &[u8], iv: &[u8]) -> std::result::Result<Self, cipher::InvalidLength> {
        let cipher = aes::Aes128::new_from_slice(key).unwrap();
        let inner = ctr::CtrCore::inner_iv_slice_init(cipher, iv).unwrap();
        Ok(Self { inner })
    }

    fn apply_keystream(self, buf: &mut [u8]) {
        self.inner.apply_keystream_partial(buf.into());
    }
}

pub enum IdentityRetrievalModes<'a> {
    /// hoprd starts with a previously generated identitiy file.
    /// If none is present, create a new one
    FromFile {
        /// Used encrypt / decrypt identity file
        password: &'a str,
        /// path to store / load identity file
        id_path: &'a str,
    },
    /// hoprd receives at startup a private which it will use
    /// for the entire runtime. The private stays in memory and
    /// thus remains fluent
    FromPrivateKey {
        /// hex string containing the private key
        private_key: &'a str,
    },
    /// takes a private key and create an identity file at the
    /// provided file destination
    #[cfg(any(feature = "hopli", test))]
    FromIdIntoFile {
        /// identifier of this keypair
        id: Uuid,
        /// Used encrypt / decrypt identity file
        password: &'a str,
        /// path to store / load identity file
        id_path: &'a str,
    },
}

pub struct HoprKeys {
    pub packet_key: OffchainKeypair,
    pub chain_key: ChainKeypair,
    id: Uuid,
}

impl Serialize for HoprKeys {
    /// Serialize without private keys
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("HoprKeys", 3)?;
        s.serialize_field("peer_id", self.packet_key.public().to_peerid_str().as_str())?;
        s.serialize_field("packet_key", self.packet_key.public().to_hex().as_str())?;
        s.serialize_field("chain_key", &self.chain_key.public().to_hex().as_str())?;
        s.serialize_field("native_address", &self.chain_key.public().to_address().to_string())?;
        s.serialize_field("uuid", &self.id)?;
        s.end()
    }
}

impl std::fmt::Display for HoprKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "packet_key: {}, chain_key: {} (Ethereum address: {})\nUUID: {}",
                self.packet_key.public().to_peerid_str(),
                self.chain_key.public().to_hex(),
                self.chain_key.public().0.to_address(),
                self.id
            )
            .as_str(),
        )
    }
}

impl TryFrom<&str> for HoprKeys {
    type Error = KeyPairError;

    /// Deserializes HoprKeys from string
    ///
    /// ```rust
    /// use hoprd_keypair::key_pair::HoprKeys;
    ///
    /// let priv_keys = "0x56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52ae";
    /// assert!(HoprKeys::try_from(priv_keys).is_ok());
    /// ```
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let maybe_priv_key = s.strip_prefix("0x").unwrap_or(s);

        if maybe_priv_key.len() != 2 * (PACKET_KEY_LENGTH + CHAIN_KEY_LENGTH) {
            return Err(KeyPairError::InvalidPrivateKeySize {
                actual: maybe_priv_key.len(),
                expected: 2 * (PACKET_KEY_LENGTH + CHAIN_KEY_LENGTH),
            });
        }

        let mut priv_key_raw = [0u8; PACKET_KEY_LENGTH + CHAIN_KEY_LENGTH];
        hex::decode_to_slice(maybe_priv_key, &mut priv_key_raw[..])?;

        priv_key_raw.try_into()
    }
}

impl TryFrom<[u8; PACKET_KEY_LENGTH + CHAIN_KEY_LENGTH]> for HoprKeys {
    type Error = KeyPairError;
    /// Deserializes HoprKeys from binary string
    ///
    /// ```rust
    /// use hoprd_keypair::key_pair::HoprKeys;
    ///
    /// let priv_keys = [
    ///     0x56,0xb2,0x9c,0xef,0xcd,0xf5,0x76,0xee,0xa3,0x06,0xba,0x2f,0xd5,0xf3,0x2e,0x65,
    ///     0x1c,0x09,0xe0,0xab,0xbc,0x01,0x8c,0x47,0xbd,0xc6,0xef,0x44,0xf6,0xb7,0x50,0x6f,
    ///     0x10,0x50,0xf9,0x51,0x37,0x77,0x04,0x78,0xf5,0x0b,0x45,0x62,0x67,0xf7,0x61,0xf1,
    ///     0xb8,0xb3,0x41,0xa1,0x3d,0xa6,0x8b,0xc3,0x2e,0x5c,0x96,0x98,0x4f,0xcd,0x52,0xae
    /// ];
    /// assert!(HoprKeys::try_from(priv_keys).is_ok());
    /// ```
    fn try_from(value: [u8; CHAIN_KEY_LENGTH + PACKET_KEY_LENGTH]) -> std::result::Result<Self, Self::Error> {
        let mut packet_key = [0u8; PACKET_KEY_LENGTH];
        packet_key.copy_from_slice(&value[0..32]);
        let mut chain_key = [0u8; CHAIN_KEY_LENGTH];
        chain_key.copy_from_slice(&value[32..64]);

        (packet_key, chain_key).try_into()
    }
}

impl TryFrom<([u8; PACKET_KEY_LENGTH], [u8; CHAIN_KEY_LENGTH])> for HoprKeys {
    type Error = KeyPairError;

    /// Deserializes HoprKeys from tuple of two binary private keys
    ///
    /// ```rust
    /// use hoprd_keypair::key_pair::HoprKeys;
    ///
    /// let priv_keys = (
    /// [
    ///     0x56,0xb2,0x9c,0xef,0xcd,0xf5,0x76,0xee,0xa3,0x06,0xba,0x2f,0xd5,0xf3,0x2e,0x65,
    ///     0x1c,0x09,0xe0,0xab,0xbc,0x01,0x8c,0x47,0xbd,0xc6,0xef,0x44,0xf6,0xb7,0x50,0x6f,
    /// ], [
    ///     0x10,0x50,0xf9,0x51,0x37,0x77,0x04,0x78,0xf5,0x0b,0x45,0x62,0x67,0xf7,0x61,0xf1,
    ///     0xb8,0xb3,0x41,0xa1,0x3d,0xa6,0x8b,0xc3,0x2e,0x5c,0x96,0x98,0x4f,0xcd,0x52,0xae
    /// ]);
    /// assert!(HoprKeys::try_from(priv_keys).is_ok());
    /// ```
    fn try_from(value: ([u8; PACKET_KEY_LENGTH], [u8; CHAIN_KEY_LENGTH])) -> std::result::Result<Self, Self::Error> {
        Ok(HoprKeys {
            packet_key: OffchainKeypair::from_secret(&value.0)?,
            chain_key: ChainKeypair::from_secret(&value.1)?,
            id: Uuid::new_v4(),
        })
    }
}

impl PartialEq for HoprKeys {
    fn eq(&self, other: &Self) -> bool {
        self.packet_key.public().eq(other.packet_key.public()) && self.chain_key.public().eq(other.chain_key.public())
    }
}

impl HoprKeys {
    /// Creates two new keypairs, one for off-chain affairs and
    /// another one to be used within the smart contract
    pub fn random() -> Self {
        Self {
            packet_key: OffchainKeypair::random(),
            chain_key: ChainKeypair::random(),
            id: Uuid::new_v4(),
        }
    }

    /// Used to derive a UUID as used as a filename in case none
    /// is specified
    #[cfg(any(feature = "hopli", test))]
    pub fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
    }

    /// Initializes HoprKeys using the provided retrieval mode
    pub fn init(retrieval_mode: IdentityRetrievalModes) -> Result<Self> {
        match retrieval_mode {
            IdentityRetrievalModes::FromFile { password, id_path } => {
                let identity_file_exists = metadata(&id_path).is_ok();

                if identity_file_exists {
                    info!("identity file exists at {}", id_path);

                    match HoprKeys::read_eth_keystore(&id_path, &password) {
                        Ok((keys, needs_migration)) => {
                            info!("migration needed = {}", needs_migration);
                            if needs_migration {
                                keys.write_eth_keystore(&id_path, &password)?
                            }
                            Ok(keys)
                        }
                        Err(e) => {
                            error!("{}", e.to_string());

                            Err(KeyPairError::GeneralError(format!("An identity file is present at {} but the provided password <REDACTED> is not sufficient to decrypt it", id_path)))
                        }
                    }
                } else {
                    let keys = HoprKeys::random();

                    info!("created a new set of keypairs at {}", id_path);
                    info!("{}", keys);

                    keys.write_eth_keystore(&id_path, &password)?;
                    Ok(keys)
                }
            }
            IdentityRetrievalModes::FromPrivateKey { private_key } => {
                info!("initializing HoprKeys with provided private keys <REDACTED>");

                private_key.try_into()
            }
            #[cfg(any(feature = "hopli", test))]
            IdentityRetrievalModes::FromIdIntoFile { id, password, id_path } => {
                let identity_file_exists = metadata(&id_path).is_ok();

                if identity_file_exists {
                    info!("identity file exists at {}", id_path);

                    Err(KeyPairError::GeneralError(format!(
                        "Cannot create identity file at {} because the file already exists.",
                        id_path
                    )))
                } else {
                    let keys: HoprKeys = HoprKeys {
                        id,
                        packet_key: OffchainKeypair::random(),
                        chain_key: ChainKeypair::random(),
                    };

                    keys.write_eth_keystore(&id_path, &password)?;

                    Ok(keys)
                }
            }
        }
    }

    /// Reads a keystore file using custom FS operations
    ///
    /// Highly inspired by `<https://github.com/roynalnaruto/eth-keystore-rs>`
    fn read_eth_keystore(path: &str, password: &str) -> Result<(Self, bool)> {
        let json_string = read_to_string(path)?;
        let keystore: EthKeystore = from_json_string(&json_string)?;

        let key = match keystore.crypto.kdfparams {
            KdfparamsType::Scrypt { dklen, n, p, r, salt } => {
                let mut key = vec![0u8; dklen as usize];
                let log_n = (n as f32).log2() as u8;
                let scrypt_params = ScryptParams::new(log_n, r, p, dklen.into())
                    .map_err(|err| KeyPairError::KeyDerivationError(err.to_string()))?;
                // derive "master key" and store it in `key`
                scrypt(password.as_ref(), &salt, &scrypt_params, &mut key)
                    .map_err(|err| KeyPairError::KeyDerivationError(err.to_string()))?;
                key
            }
            _ => panic!("HOPR only supports scrypt"),
        };

        // Derive the MAC from the derived key and ciphertext.
        let derived_mac = Keccak256::new()
            .chain(&key[16..32])
            .chain(&keystore.crypto.ciphertext)
            .finalize();

        if *derived_mac != *keystore.crypto.mac {
            return Err(KeyPairError::MacMismatch);
        }

        // Decrypt the private key bytes using AES-128-CTR
        let decryptor = Aes128Ctr::new(&key[..16], &keystore.crypto.cipherparams.iv[..16]).expect("invalid length");

        let mut pk = keystore.crypto.ciphertext;

        match pk.len() {
            V1_PRIVKEY_LENGTH => {
                decryptor.apply_keystream(&mut pk);

                let packet_key: [u8; PACKET_KEY_LENGTH] = random_bytes();

                let mut chain_key = [0u8; CHAIN_KEY_LENGTH];
                chain_key.clone_from_slice(&pk.as_slice()[0..CHAIN_KEY_LENGTH]);

                let ret: HoprKeys = (packet_key, chain_key).try_into().unwrap();

                Ok((ret, true))
            }
            V2_PRIVKEYS_LENGTH => {
                decryptor.apply_keystream(&mut pk);

                let private_keys = serde_json::from_slice::<PrivateKeys>(&pk)?;

                if private_keys.packet_key.len() != PACKET_KEY_LENGTH {
                    return Err(KeyPairError::InvalidEncryptedKeyLength {
                        actual: private_keys.packet_key.len(),
                        expected: PACKET_KEY_LENGTH,
                    });
                }

                if private_keys.chain_key.len() != CHAIN_KEY_LENGTH {
                    return Err(KeyPairError::InvalidEncryptedKeyLength {
                        actual: private_keys.chain_key.len(),
                        expected: CHAIN_KEY_LENGTH,
                    });
                }

                let mut packet_key = [0u8; PACKET_KEY_LENGTH];
                packet_key.clone_from_slice(private_keys.packet_key.as_slice());

                let mut chain_key = [0u8; CHAIN_KEY_LENGTH];
                chain_key.clone_from_slice(private_keys.chain_key.as_slice());

                Ok((
                    HoprKeys {
                        packet_key: OffchainKeypair::from_secret(&packet_key).unwrap(),
                        chain_key: ChainKeypair::from_secret(&chain_key).unwrap(),
                        id: keystore.id,
                    },
                    false,
                ))
            }
            _ => Err(KeyPairError::InvalidEncryptedKeyLength {
                actual: pk.len(),
                expected: V2_PRIVKEYS_LENGTH,
            }),
        }
    }

    /// Writes a keystore file using custom FS operation and custom entropy source
    ///
    /// Highly inspired by `<https://github.com/roynalnaruto/eth-keystore-rs>`
    fn write_eth_keystore(&self, path: &str, password: &str) -> Result<()> {
        if USE_WEAK_CRYPTO {
            warn!("USING WEAK CRYPTO -> this build is not meant for production!");
        }
        // Generate a random salt.
        let salt: [u8; HOPR_KEY_SIZE] = random_bytes();

        // Derive the key.
        let mut key = [0u8; HOPR_KDF_PARAMS_DKLEN as usize];
        let scrypt_params = ScryptParams::new(
            if USE_WEAK_CRYPTO { 1 } else { HOPR_KDF_PARAMS_LOG_N },
            HOPR_KDF_PARAMS_R,
            HOPR_KDF_PARAMS_P,
            HOPR_KDF_PARAMS_DKLEN.into(),
        )
        .map_err(|e| KeyPairError::KeyDerivationError(e.to_string()))?;

        // derive "master key" and store it in `key`
        scrypt(password.as_ref(), &salt, &scrypt_params, &mut key)
            .map_err(|e| KeyPairError::KeyDerivationError(e.to_string()))?;

        // Encrypt the private key using AES-128-CTR.
        let iv: [u8; HOPR_IV_SIZE] = random_bytes();

        let encryptor = Aes128Ctr::new(&key[..16], &iv[..16]).expect("invalid length");

        let private_keys = PrivateKeys {
            chain_key: self.chain_key.secret().as_ref().to_vec(),
            packet_key: self.packet_key.secret().as_ref().to_vec(),
            version: VERSION,
        };

        let mut ciphertext = serde_json::to_vec(&private_keys)?;
        encryptor.apply_keystream(&mut ciphertext);

        // Calculate the MAC.
        let mac = Keccak256::new().chain(&key[16..32]).chain(&ciphertext).finalize();

        // Construct and serialize the encrypted JSON keystore.
        let keystore = EthKeystore {
            id: self.id,
            version: 3,
            crypto: CryptoJson {
                cipher: String::from(HOPR_CIPHER),
                cipherparams: CipherparamsJson { iv: iv.to_vec() },
                ciphertext,
                kdf: KdfType::Scrypt,
                kdfparams: KdfparamsType::Scrypt {
                    dklen: HOPR_KDF_PARAMS_DKLEN,
                    n: 2u32.pow(if USE_WEAK_CRYPTO { 1 } else { HOPR_KDF_PARAMS_LOG_N } as u32),
                    p: HOPR_KDF_PARAMS_P,
                    r: HOPR_KDF_PARAMS_R,
                    salt: salt.to_vec(),
                },
                mac: mac.to_vec(),
            },
        };

        let serialized = to_json_string(&keystore)?;

        write(path, serialized).map_err(|e| e.into())
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}

impl Debug for HoprKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprKeys")
            .field(
                "packet_key",
                &format_args!("(priv_key: <REDACTED>, pub_key: {}", self.packet_key.public().to_hex()),
            )
            .field(
                "chain_key",
                &format_args!("(priv_key: <REDACTED>, pub_key: {}", self.chain_key.public().to_hex()),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use hopr_crypto_types::prelude::*;
    use tempfile::tempdir;

    use super::HoprKeys;

    const DEFAULT_PASSWORD: &str = "dummy password for unit testing";

    #[test]
    fn create_keys() {
        println!("{:?}", super::HoprKeys::random())
    }

    #[test]
    fn store_keys_and_read_them() {
        let tmp = tempdir().unwrap();

        let identity_dir = tmp.path().join("hopr-unit-test-identity");

        let keys = super::HoprKeys::random();

        keys.write_eth_keystore(identity_dir.to_str().unwrap(), DEFAULT_PASSWORD)
            .unwrap();

        let (deserialized, needs_migration) =
            super::HoprKeys::read_eth_keystore(identity_dir.to_str().unwrap(), DEFAULT_PASSWORD).unwrap();

        assert!(!needs_migration);
        assert_eq!(deserialized, keys);
    }

    #[test]
    fn test_migration() {
        let tmp = tempdir().unwrap();

        let identity_dir = tmp.path().join("hopr-unit-test-identity");

        let old_keystore_file = r#"{"id":"8e5fe142-6ef9-4fbb-aae8-5de32b680e31","version":3,"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"04141354edb9dfb0c65e6905a3a0b9dd"},"ciphertext":"74f12f72cf2d3d73ff09f783cb9b57995b3808f7d3f71aa1fa1968696aedfbdd","kdf":"scrypt","kdfparams":{"salt":"f5e3f04eaa0c9efffcb5168c6735d7e1fe4d96f48a636c4f00107e7c34722f45","n":1,"dklen":32,"p":1,"r":8},"mac":"d0daf0e5d14a2841f0f7221014d805addfb7609d85329d4c6424a098e50b6fbe"}}"#;

        fs::write(identity_dir.to_str().unwrap(), old_keystore_file.as_bytes()).unwrap();

        let (deserialized, needs_migration) =
            super::HoprKeys::read_eth_keystore(identity_dir.to_str().unwrap(), "local").unwrap();

        assert!(needs_migration);
        assert_eq!(
            deserialized.chain_key.public().0.to_address().to_string(),
            "0x826a1bf3d51fa7f402a1e01d1b2c8a8bac28e666"
        );
    }

    #[test]
    fn test_auto_migration() {
        let tmp = tempdir().unwrap();
        let identity_dir = tmp.path().join("hopr-unit-test-identity");
        let identity_path: &str = identity_dir.to_str().unwrap();

        let old_keystore_file = r#"{"id":"8e5fe142-6ef9-4fbb-aae8-5de32b680e31","version":3,"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"04141354edb9dfb0c65e6905a3a0b9dd"},"ciphertext":"74f12f72cf2d3d73ff09f783cb9b57995b3808f7d3f71aa1fa1968696aedfbdd","kdf":"scrypt","kdfparams":{"salt":"f5e3f04eaa0c9efffcb5168c6735d7e1fe4d96f48a636c4f00107e7c34722f45","n":1,"dklen":32,"p":1,"r":8},"mac":"d0daf0e5d14a2841f0f7221014d805addfb7609d85329d4c6424a098e50b6fbe"}}"#;
        fs::write(identity_path, old_keystore_file.as_bytes()).unwrap();

        assert!(super::HoprKeys::init(super::IdentityRetrievalModes::FromFile {
            password: "local".into(),
            id_path: identity_path.into()
        })
        .is_ok());

        let (deserialized, needs_migration) = super::HoprKeys::read_eth_keystore(identity_path, "local").unwrap();

        assert!(!needs_migration);
        assert_eq!(
            deserialized.chain_key.public().0.to_address().to_string(),
            "0x826a1bf3d51fa7f402a1e01d1b2c8a8bac28e666"
        );
    }

    #[test]
    fn test_should_not_overwrite_existing() {
        let tmp = tempdir().unwrap();
        let identity_dir = tmp.path().join("hopr-unit-test-identity");
        let identity_path: &str = identity_dir.to_str().unwrap();

        fs::write(identity_path, "".as_bytes()).unwrap();

        // Overwriting existing keys must not be possible
        assert!(super::HoprKeys::init(super::IdentityRetrievalModes::FromFile {
            password: "local".into(),
            id_path: identity_path.into()
        })
        .is_err());
    }
    #[test]
    fn test_from_privatekey() {
        let private_key = "0x56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52ae";

        let from_private_key = HoprKeys::init(super::IdentityRetrievalModes::FromPrivateKey { private_key }).unwrap();

        let private_key_without_prefix = "56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52ae";

        let from_private_key_without_prefix = HoprKeys::init(super::IdentityRetrievalModes::FromPrivateKey {
            private_key: private_key_without_prefix,
        })
        .unwrap();

        // both ways should work
        assert_eq!(from_private_key, from_private_key_without_prefix);

        let too_short_private_key = "0xb29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52ae";

        assert!(HoprKeys::init(super::IdentityRetrievalModes::FromPrivateKey {
            private_key: too_short_private_key
        })
        .is_err());

        let too_long_private_key = "0x56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52aeae";

        assert!(HoprKeys::init(super::IdentityRetrievalModes::FromPrivateKey {
            private_key: too_long_private_key
        })
        .is_err());

        let non_hex_private = "this is the story of hopr: in 2018 ...";
        assert!(HoprKeys::init(super::IdentityRetrievalModes::FromPrivateKey {
            private_key: non_hex_private
        })
        .is_err());
    }

    #[test]
    fn test_from_privatekey_into_file() {
        let tmp = tempdir().unwrap();
        let identity_dir = tmp.path().join("hopr-unit-test-identity");
        let identity_path = identity_dir.to_str().unwrap();
        let id = super::HoprKeys::get_random_uuid();

        assert!(super::HoprKeys::init(super::IdentityRetrievalModes::FromIdIntoFile {
            password: "local",
            id_path: identity_path,
            id
        })
        .is_ok());

        let (deserialized, needs_migration) = super::HoprKeys::read_eth_keystore(identity_path, "local").unwrap();

        assert!(!needs_migration);
        assert_eq!(
            deserialized.chain_key.public().0.to_address().to_string(),
            "0x05801834fd089afe070dab49b2111b92a4725893"
        );

        // Overwriting existing keys must not be possible
        assert!(super::HoprKeys::init(super::IdentityRetrievalModes::FromIdIntoFile {
            password: "local",
            id_path: identity_path,
            id
        })
        .is_err());
    }
}
