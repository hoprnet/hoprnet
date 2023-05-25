use std::fmt::Debug;

use crate::{
    errors::{KeyPairError, Result},
    keystore::{CipherparamsJson, CryptoJson, EthKeystore, KdfType, KdfparamsType},
};
use aes::{
    cipher::{self, InnerIvInit, KeyInit, StreamCipherCore},
    Aes128,
};
use core_crypto::types::PublicKey;
use getrandom::getrandom;
use scrypt::{scrypt, Params as ScryptParams};
use serde_json::{from_str as from_json_string, to_string as to_json_string};
use sha3::{digest::Update, Digest, Keccak256};
use uuid::Uuid;

#[cfg(all(feature = "wasm", not(test)))]
use real_base::file::wasm::{read_to_string, write};

#[cfg(any(not(feature = "wasm"), test))]
use real_base::file::native::{read_to_string, write};

const HOPR_CIPHER: &str = "aes-128-ctr";
const HOPR_KEY_SIZE: usize = 32usize;
const HOPR_IV_SIZE: usize = 16usize;
const HOPR_KDF_PARAMS_DKLEN: u8 = 32u8;
const HOPR_KDF_PARAMS_LOG_N: u8 = 13u8;
const HOPR_KDF_PARAMS_R: u32 = 8u32;
const HOPR_KDF_PARAMS_P: u32 = 1u32;

const PACKET_KEY_LENGTH: usize = 32;
const CHAIN_KEY_LENGTH: usize = 32;

pub type PacketKey = [u8; PACKET_KEY_LENGTH];
pub type ChainKey = [u8; CHAIN_KEY_LENGTH];

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

pub struct HoprKeys {
    packet_key: (PacketKey, PublicKey),
    chain_key: (ChainKey, PublicKey),
}

impl HoprKeys {
    pub fn new() -> Result<Self> {
        let mut packet_priv_key = [0u8; 32];
        let mut chain_priv_key = [0u8; 32];

        let packet_key_raw = PublicKey::random_keypair();
        packet_priv_key.copy_from_slice(packet_key_raw.0.as_ref());

        let chain_key_raw = PublicKey::random_keypair();
        chain_priv_key.copy_from_slice(chain_key_raw.0.as_ref());

        Ok(HoprKeys {
            packet_key: (packet_priv_key, packet_key_raw.1),
            chain_key: (chain_priv_key, chain_key_raw.1),
        })
    }

    pub fn read_eth_keystore(path: &str, password: &str) -> Result<Self> {
        let json_string = read_to_string(path)?;
        let keystore: EthKeystore = from_json_string(&json_string)?;

        let key = match keystore.crypto.kdfparams {
            KdfparamsType::Scrypt { dklen, n, p, r, salt } => {
                let mut key = vec![0u8; dklen as usize];
                let log_n = (n as f32).log2() as u8;
                let scrypt_params = ScryptParams::new(log_n, r, p, dklen.into())
                    .map_err(|err| KeyPairError::KeyDerivationError { err: err.to_string() })?;
                scrypt(password.as_ref(), &salt, &scrypt_params, key.as_mut_slice())
                    .map_err(|err| KeyPairError::KeyDerivationError { err: err.to_string() })?;
                key
            }
            _ => panic!("HOPR only supports scrypt"),
        };

        // Derive the MAC from the derived key and ciphertext.
        let derived_mac = Keccak256::new()
            .chain(&key[16..32])
            .chain(&keystore.crypto.ciphertext)
            .finalize();

        if derived_mac.as_slice() != keystore.crypto.mac.as_slice() {
            return Err(KeyPairError::MacMismatch);
        }

        // Decrypt the private key bytes using AES-128-CTR
        let decryptor = Aes128Ctr::new(&key[..16], &keystore.crypto.cipherparams.iv[..16]).expect("invalid length");

        let mut pk = keystore.crypto.ciphertext;

        if pk.len() != 64 {
            return Err(KeyPairError::InvalidEncryptedKeyLength {
                actual: pk.len(),
                expected: PACKET_KEY_LENGTH + CHAIN_KEY_LENGTH,
            });
        }

        decryptor.apply_keystream(&mut pk);

        let mut packet_key = [0u8; 32];
        packet_key.clone_from_slice(&pk.as_slice()[..32]);
        let mut chain_key = [0u8; 32];
        chain_key.clone_from_slice(&pk.as_slice()[32..64]);

        Ok(HoprKeys {
            packet_key: (packet_key, PublicKey::from_privkey(&packet_key[..]).unwrap()),
            // TODO: change this to off-chain privKey
            chain_key: (chain_key, PublicKey::from_privkey(&chain_key[..]).unwrap()),
        })
    }

    pub fn write_eth_keystore(&self, path: &str, password: &str, use_weak_crypto: bool) -> Result<()> {
        // Generate a random salt.
        let mut salt = [0u8; HOPR_KEY_SIZE];

        getrandom(&mut salt)?;

        // Derive the key.
        let mut key = [0u8; HOPR_KDF_PARAMS_DKLEN as usize];
        let scrypt_params = ScryptParams::new(
            if use_weak_crypto { 1 } else { HOPR_KDF_PARAMS_LOG_N },
            HOPR_KDF_PARAMS_R,
            HOPR_KDF_PARAMS_P,
            HOPR_KDF_PARAMS_DKLEN.into(),
        )
        .map_err(|e| KeyPairError::KeyDerivationError { err: e.to_string() })?;

        scrypt(password.as_ref(), &salt, &scrypt_params, key.as_mut_slice())
            .map_err(|e| KeyPairError::KeyDerivationError { err: e.to_string() })?;

        // Encrypt the private key using AES-128-CTR.
        let mut iv = [0u8; HOPR_IV_SIZE];
        getrandom(&mut iv)?;

        let encryptor = Aes128Ctr::new(&key[..16], &iv[..16]).expect("invalid length");

        let mut ciphertext = [self.packet_key.0, self.chain_key.0].concat();
        encryptor.apply_keystream(&mut ciphertext);

        // Calculate the MAC.
        let mac = Keccak256::new().chain(&key[16..32]).chain(&ciphertext).finalize();

        // If a file name is not specified for the keystore, simply use the strigified uuid.
        let id = Uuid::new_v4();

        // Construct and serialize the encrypted JSON keystore.
        let keystore = EthKeystore {
            id,
            version: 3,
            crypto: CryptoJson {
                cipher: String::from(HOPR_CIPHER),
                cipherparams: CipherparamsJson { iv: iv.to_vec() },
                ciphertext: ciphertext.to_vec(),
                kdf: KdfType::Scrypt,
                kdfparams: KdfparamsType::Scrypt {
                    dklen: HOPR_KDF_PARAMS_DKLEN,
                    n: 2u32.pow(HOPR_KDF_PARAMS_LOG_N as u32),
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
}

impl Debug for HoprKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprKeys")
            .field(
                "packet_key",
                &format_args!("(priv_key: <REDACTED>, pub_key: {}", self.packet_key.1.to_hex(false)),
            )
            .field(
                "chain_key",
                &format_args!("(priv_key: <REDACTED>, pub_key: {}", self.chain_key.1.to_hex(false)),
            )
            .finish()
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_misc::ok_or_jserr;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct HoprKeys {
        w: super::HoprKeys,
    }

    #[wasm_bindgen]
    impl HoprKeys {
        #[wasm_bindgen(constructor)]
        pub fn new() -> std::result::Result<HoprKeys, JsValue> {
            let keys = ok_or_jserr!(super::HoprKeys::new())?;
            Ok(Self { w: keys })
        }

        #[wasm_bindgen]
        pub fn read_eth_keystore(path: &str, password: &str) -> Result<HoprKeys, JsValue> {
            let keys = ok_or_jserr!(super::HoprKeys::read_eth_keystore(path, password))?;
            Ok(Self { w: keys })
        }

        #[wasm_bindgen]
        pub fn write_eth_keystore(&self, path: &str, password: &str, use_weak_crypto: bool) -> Result<(), JsValue> {
            ok_or_jserr!(super::HoprKeys::write_eth_keystore(
                &self.w,
                path,
                password,
                use_weak_crypto
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HoprKeys;
    use std::fs::File;

    #[test]
    fn create_keys() {
        println!("{:?}", HoprKeys::new().unwrap())
    }
}
