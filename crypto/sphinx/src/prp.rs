use blake2::Blake2bMac;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use digest::{FixedOutput, Mac};
use hopr_crypto_types::errors::CryptoError::InvalidParameterSize;
use hopr_crypto_types::primitives::{SecretKey, SimpleStreamCipher};
use hopr_crypto_types::utils;
use zeroize::ZeroizeOnDrop;

use crate::derivation::generate_key_iv;

/// Abstraction for Pseudo-Random Permutation.
pub trait PRP: From<SecretKey> {
    /// Applies forward permutation on the given plaintext and returns a new buffer
    /// containing the result.
    fn forward(&mut self, plaintext: &[u8]) -> hopr_crypto_types::errors::Result<Box<[u8]>> {
        let mut out = plaintext.to_vec();
        self.forward_inplace(&mut out)?;
        Ok(out.into_boxed_slice())
    }

    /// Applies forward permutation on the given plaintext and modifies the given buffer in-place.
    fn forward_inplace(&mut self, plaintext: &mut [u8]) -> hopr_crypto_types::errors::Result<()>;

    /// Applies inverse permutation on the given plaintext and returns a new buffer
    /// containing the result.
    fn inverse(&mut self, ciphertext: &[u8]) -> hopr_crypto_types::errors::Result<Box<[u8]>> {
        let mut out = ciphertext.to_vec();
        self.inverse_inplace(&mut out)?;
        Ok(out.into_boxed_slice())
    }

    /// Applies inverse permutation on the given ciphertext and modifies the given buffer in-place.
    fn inverse_inplace(&mut self, ciphertext: &mut [u8]) -> hopr_crypto_types::errors::Result<()>;
}

pub struct Chacha20PRP(ChaCha20);

impl From<SecretKey> for Chacha20PRP {
    fn from(value: SecretKey) -> Self {
        let mut key = chacha20::Key::default();
        let mut iv = chacha20::Nonce::default();
        generate_key_iv(&value, HASH_KEY_PRP.as_bytes(), &mut key, &mut iv, false);

        Self(ChaCha20::new(&key, &iv))
    }
}

impl PRP for Chacha20PRP {
    fn forward_inplace(&mut self, plaintext: &mut [u8]) -> hopr_crypto_types::errors::Result<()> {
        self.0.apply_keystream(plaintext);
        Ok(())
    }

    fn inverse_inplace(&mut self, ciphertext: &mut [u8]) -> hopr_crypto_types::errors::Result<()> {
        self.0.apply_keystream(ciphertext);
        Ok(())
    }
}

// TODO: remove the legacy stuff below

// Module-specific constants
const LIONESS_INTERMEDIATE_KEY_LENGTH: usize = 32;
const LIONESS_INTERMEDIATE_IV_LENGTH: usize = 16;
const LIONESS_PRP_KEY_LENGTH: usize = 4 * LIONESS_INTERMEDIATE_KEY_LENGTH;
const LIONESS_PRP_IV_LENGTH: usize = 4 * LIONESS_INTERMEDIATE_IV_LENGTH;

const HASH_KEY_PRP: &str = "HASH_KEY_PRP";

// The minimum input length must be at least size of the key, which is XORed with plaintext/ciphertext
pub const LIONESS_PRP_MIN_LENGTH: usize = LIONESS_INTERMEDIATE_KEY_LENGTH;

/// Parameters for the Pseudo-Random Permutation (PRP) function
/// This consists of IV and the raw secret key for use by the underlying cryptographic transformation.
#[derive(ZeroizeOnDrop)]
pub struct LionessPRPParameters {
    key: [u8; LIONESS_PRP_KEY_LENGTH],
    iv: [u8; LIONESS_PRP_IV_LENGTH],
}

impl Default for LionessPRPParameters {
    fn default() -> Self {
        Self {
            key: [0u8; LIONESS_PRP_KEY_LENGTH],
            iv: [0u8; LIONESS_PRP_IV_LENGTH],
        }
    }
}

impl LionessPRPParameters {
    /// Creates new parameters for the PRP by expanding the given
    /// keying material into the secret key and IV for the underlying cryptographic transformation.
    pub fn new(secret: SecretKey) -> Self {
        let mut ret = LionessPRPParameters::default();
        generate_key_iv(&secret, HASH_KEY_PRP.as_bytes(), &mut ret.key, &mut ret.iv, false);
        ret
    }
}

/// Implementation of Pseudo-Random Permutation (PRP).
/// Currently based on the Lioness wide-block cipher.
#[derive(ZeroizeOnDrop)]
pub struct LionessPRP {
    keys: [[u8; LIONESS_INTERMEDIATE_KEY_LENGTH]; 4],
    ivs: [[u8; LIONESS_INTERMEDIATE_IV_LENGTH]; 4],
}

impl From<SecretKey> for LionessPRP {
    fn from(value: SecretKey) -> Self {
        Self::from_parameters(LionessPRPParameters::new(value))
    }
}

impl LionessPRP {
    /// Creates new instance of the PRP using the raw key and IV.
    pub fn new(key: [u8; LIONESS_PRP_KEY_LENGTH], iv: [u8; LIONESS_PRP_IV_LENGTH]) -> Self {
        Self {
            keys: [
                key[..LIONESS_INTERMEDIATE_KEY_LENGTH].try_into().unwrap(),
                key[LIONESS_INTERMEDIATE_KEY_LENGTH..2 * LIONESS_INTERMEDIATE_KEY_LENGTH]
                    .try_into()
                    .unwrap(),
                key[2 * LIONESS_INTERMEDIATE_KEY_LENGTH..3 * LIONESS_INTERMEDIATE_KEY_LENGTH]
                    .try_into()
                    .unwrap(),
                key[3 * LIONESS_INTERMEDIATE_KEY_LENGTH..4 * LIONESS_INTERMEDIATE_KEY_LENGTH]
                    .try_into()
                    .unwrap(),
            ],
            ivs: [
                // NOTE: ChaCha20 takes only 12 byte IV
                iv[..LIONESS_INTERMEDIATE_IV_LENGTH].try_into().unwrap(),
                iv[LIONESS_INTERMEDIATE_IV_LENGTH..2 * LIONESS_INTERMEDIATE_IV_LENGTH]
                    .try_into()
                    .unwrap(),
                iv[2 * LIONESS_INTERMEDIATE_IV_LENGTH..3 * LIONESS_INTERMEDIATE_IV_LENGTH]
                    .try_into()
                    .unwrap(),
                iv[3 * LIONESS_INTERMEDIATE_IV_LENGTH..4 * LIONESS_INTERMEDIATE_IV_LENGTH]
                    .try_into()
                    .unwrap(),
            ],
        }
    }

    /// Creates a new PRP instance using the given parameters
    pub fn from_parameters(params: LionessPRPParameters) -> Self {
        Self::new(params.key, params.iv) // Parameter size checking taken care of by PRPParameters
    }

    // Internal helper functions

    fn xor_hash(data: &mut [u8], key: &[u8], iv: &[u8]) {
        let mut blake = Blake2bMac::<typenum::U32>::new_with_salt_and_personal(key, iv, &[])
            .expect("invalid intermediate key or iv size"); // should not happen
        blake.update(&data[LIONESS_PRP_MIN_LENGTH..]);

        utils::xor_inplace(data, &blake.finalize_fixed());
    }

    fn xor_keystream(data: &mut [u8], key: &[u8], iv: &[u8]) {
        let mut key_cpy = Vec::from(key);
        utils::xor_inplace(key_cpy.as_mut_slice(), &data[0..LIONESS_PRP_MIN_LENGTH]);

        let iv_cpy = &iv[4..iv.len()];

        let mut cipher = SimpleStreamCipher::new(
            key_cpy.try_into().expect("invalid keystream key size"),
            iv_cpy.try_into().expect("invalid keystream iv size"),
        );

        let block_counter = u32::from_le_bytes(iv[0..4].try_into().unwrap());
        cipher.set_block_counter(block_counter);

        cipher.apply(&mut data[LIONESS_PRP_MIN_LENGTH..]);
    }
}

impl PRP for LionessPRP {
    /// Applies forward permutation on the given plaintext and modifies the given buffer in-place.
    fn forward_inplace(&mut self, plaintext: &mut [u8]) -> hopr_crypto_types::errors::Result<()> {
        if plaintext.len() >= LIONESS_PRP_MIN_LENGTH {
            Self::xor_keystream(plaintext, &self.keys[0], &self.ivs[0]);
            Self::xor_hash(plaintext, &self.keys[1], &self.ivs[1]);
            Self::xor_keystream(plaintext, &self.keys[2], &self.ivs[2]);
            Self::xor_hash(plaintext, &self.keys[3], &self.ivs[3]);
            Ok(())
        } else {
            Err(InvalidParameterSize {
                name: "plaintext",
                expected: LIONESS_PRP_MIN_LENGTH,
            })
        }
    }
    /// Applies inverse permutation on the given ciphertext and modifies the given buffer in-place.
    fn inverse_inplace(&mut self, ciphertext: &mut [u8]) -> hopr_crypto_types::errors::Result<()> {
        if ciphertext.len() >= LIONESS_PRP_MIN_LENGTH {
            Self::xor_hash(ciphertext, &self.keys[3], &self.ivs[3]);
            Self::xor_keystream(ciphertext, &self.keys[2], &self.ivs[2]);
            Self::xor_hash(ciphertext, &self.keys[1], &self.ivs[1]);
            Self::xor_keystream(ciphertext, &self.keys[0], &self.ivs[0]);
            Ok(())
        } else {
            Err(InvalidParameterSize {
                name: "ciphertext",
                expected: LIONESS_PRP_MIN_LENGTH,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;

    #[test]
    fn test_prp_fixed() {
        let mut prp = LionessPRP::new([0u8; 4 * 32], [0u8; 4 * 16]);

        let data = [1u8; 278];

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_eq!(&data, pt.as_ref());
    }

    #[test]
    fn test_prp_random() {
        let mut prp = LionessPRP::new(random_bytes(), random_bytes());
        let data: [u8; 278] = random_bytes();

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_ne!(&data, ct.as_ref(), "ciphertext must be different than plaintext");
        assert_eq!(&data, pt.as_ref(), "plaintexts must be the same");
    }

    #[test]
    fn test_prp_random_inplace() {
        let mut prp = LionessPRP::new(random_bytes(), random_bytes());
        let mut data: [u8; 278] = random_bytes();
        let data_old = data;

        prp.forward_inplace(&mut data).unwrap();
        assert_ne!(&data_old, data.as_ref(), "buffer must be encrypted in-place");

        prp.inverse_inplace(&mut data).unwrap();
        assert_eq!(&data_old, data.as_ref(), "buffer must be decrypted in-place");
    }

    #[test]
    fn test_prp_parameters() {
        let expected_key = hex!("a9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f35cb715a26e268fc3ce68680e16767581de4e2cb3944c563d1f1a0cc077f3e788a12f31ae07111d77a876a66de5bdd6176bdaa2e07d1cb2e36e428afafdebb2109f70ce8422c8821233053bdd5871523ffb108f1e0f86809999a99d407590df25");
        let expected_iv = hex!("a59991716be504b26471dea53d688c4bab8e910328e54ebb6ebf07b49e6d12eacfc56e0935ba2300559b43ede25aa09eee7e8a2deea5f0bdaee2e859834edd38");

        let params = LionessPRPParameters::new(SecretKey::default());

        assert_eq!(expected_key, params.key);
        assert_eq!(expected_iv, params.iv)
    }

    #[test]
    fn test_prp_ciphertext_from_params() {
        let params = LionessPRPParameters::new(SecretKey::default());

        let expected_key = hex!("a9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f35cb715a26e268fc3ce68680e16767581de4e2cb3944c563d1f1a0cc077f3e788a12f31ae07111d77a876a66de5bdd6176bdaa2e07d1cb2e36e428afafdebb2109f70ce8422c8821233053bdd5871523ffb108f1e0f86809999a99d407590df25");
        let expected_iv = hex!("a59991716be504b26471dea53d688c4bab8e910328e54ebb6ebf07b49e6d12eacfc56e0935ba2300559b43ede25aa09eee7e8a2deea5f0bdaee2e859834edd38");
        assert_eq!(expected_key, params.key);
        assert_eq!(expected_iv, params.iv);

        let mut prp = LionessPRP::from_parameters(params);

        let pt = [0u8; 100];
        let ct = prp.forward(&pt).unwrap();

        assert_eq!([0u8; 100], pt, "plain text must not change for in-place operation");
        assert_ne!(&pt, ct.as_ref(), "plain text must be different from ciphertext");
    }
}
