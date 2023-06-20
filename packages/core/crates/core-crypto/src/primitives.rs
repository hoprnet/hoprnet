use crate::derivation::derive_mac_key;
use blake2::Blake2s256;
use chacha20::cipher::KeyIvInit;
use chacha20::cipher::{IvSizeUser, KeySizeUser, StreamCipher, StreamCipherSeek};
use chacha20::ChaCha20;
use digest::{FixedOutputReset, Output, OutputSizeUser, Update};
use generic_array::GenericArray;
use sha3::Keccak256;
use typenum::Unsigned;

use crate::errors::CryptoError::{InvalidInputValue, InvalidParameterSize};
use crate::errors::Result;
use crate::parameters::SECRET_KEY_LENGTH;

/// Represents a secret key of fixed length
pub type SecretKey = [u8; SECRET_KEY_LENGTH];

/// Generalization of digest-like operation (MAC, Digest,...)
/// Defines the `update` and `finalize` operations to produce digest value of arbitrary data.
pub trait DigestLike<T>
where
    T: Update + FixedOutputReset + OutputSizeUser,
{
    /// Length of the digest in bytes
    const SIZE: usize = T::OutputSize::USIZE;

    /// Access to the internal state of the digest-like operation.
    fn internal_state(&mut self) -> &mut T;

    /// Update the internal state of the digest-like using the given input data.
    fn update(&mut self, data: &[u8]) {
        self.internal_state().update(data);
    }

    /// Retrieve the final digest value into a prepared buffer and reset this instance so it could be reused for
    /// a new computation.
    fn finalize_into(&mut self, out: &mut [u8]) {
        assert_eq!(Self::SIZE, out.len(), "invalid output size");
        let output = Output::<T>::from_mut_slice(out);
        self.internal_state().finalize_into_reset(output);
    }

    /// Retrieve the final digest value and reset this instance so it could be reused for
    /// a new computation.
    fn finalize(&mut self) -> GenericArray<u8, T::OutputSize> {
        let mut output = Output::<T>::default();
        self.finalize_into(&mut output);
        output
    }
}

/// Simple digest computation wrapper.
/// Use `new`, `update` and `finalize` triplet to produce hash of arbitrary data.
/// Currently this instance is using Blake2s256.
#[derive(Default, Clone)]
pub struct SimpleDigest {
    instance: Blake2s256,
}

impl DigestLike<Blake2s256> for SimpleDigest {
    fn internal_state(&mut self) -> &mut Blake2s256 {
        &mut self.instance
    }
}

/// Computation wrapper for a digest that's compatible with Ethereum digests.
/// Use `new`, `update` and `finalize` triplet to produce hash of arbitrary data.
/// Currently this instance is using Keccak256.
#[derive(Default, Clone)]
pub struct EthDigest {
    instance: Keccak256,
}

impl DigestLike<Keccak256> for EthDigest {
    fn internal_state(&mut self) -> &mut Keccak256 {
        &mut self.instance
    }
}

/// Simple Message Authentication Code (MAC) computation wrapper
/// Use `new`, `update` and `finalize` triplet to produce MAC of arbitrary data.
/// Currently this instance implemented `Blake2s256(key || HOPR_BLAKE_BASED_MAC || message)`
#[derive(Clone)]
pub struct SimpleMac {
    instance: Blake2s256,
}

impl SimpleMac {
    const DOMAIN_TAG: &'static str = "HOPR_BLAKE_BASED_MAC";

    /// Create new instance of the MAC using the given secret key.
    /// The key size can be arbitrary, but must be at least `SECRET_KEY_LENGTH`
    pub fn new(key: &[u8]) -> Self {
        assert!(key.len() >= SECRET_KEY_LENGTH, "mac key too short");

        let mut instance = Blake2s256::default();
        digest::Digest::update(&mut instance, &key);
        digest::Digest::update(&mut instance, Self::DOMAIN_TAG.as_bytes());
        Self { instance }
    }
}

impl DigestLike<Blake2s256> for SimpleMac {
    fn internal_state(&mut self) -> &mut Blake2s256 {
        &mut self.instance
    }
}

/// Simple stream cipher wrapper
/// Use `new` and `apply` (or `apply_copy`) to XOR the keystream on the plaintext or ciphertext.
/// Currently this instance is using ChaCha20.
pub struct SimpleStreamCipher {
    instance: ChaCha20,
}

impl SimpleStreamCipher {
    /// Create new instance of the stream cipher initialized
    /// with the given secret key and IV.
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self> {
        let chacha_iv_size = ChaCha20::iv_size();
        if iv.len() != chacha_iv_size {
            return Err(InvalidParameterSize {
                name: "iv".into(),
                expected: chacha_iv_size,
            });
        }

        let chacha_key_size = ChaCha20::key_size();
        if key.len() != chacha_key_size {
            return Err(InvalidParameterSize {
                name: "key".into(),
                expected: chacha_key_size,
            });
        }

        Ok(Self {
            instance: ChaCha20::new_from_slices(key, iv).map_err(|_| InvalidInputValue)?,
        })
    }

    pub fn set_block_counter(&mut self, counter: u32) {
        self.instance.seek(counter as u64 * 64u64)
    }

    /// Apply keystream to the given data in-place.
    pub fn apply(&mut self, data: &mut [u8]) {
        self.instance.apply_keystream(data);
    }

    /// Creates copy of the given data and applies the keystream to it.
    pub fn apply_copy(&mut self, data: &[u8]) -> Box<[u8]> {
        let mut ret = Vec::from(data);
        self.instance.apply_keystream(ret.as_mut_slice());
        ret.into_boxed_slice()
    }
}

/// Convenience function that calculates MAC using the given raw key and data.
/// The key size can be arbitrary, but has to be at least `SECRET_KEY_LENGTH`.
/// The function is based on `SimpleMac`.
pub fn calculate_mac(key: &[u8], data: &[u8]) -> [u8; SimpleMac::SIZE] {
    let mut mac = SimpleMac::new(key);
    mac.update(data);
    mac.finalize().into()
}

/// Calculates a message authentication code with fixed key tag (HASH_KEY_HMAC)
/// The given secret is first transformed using HKDF before the MAC calculation is performed.
/// Based on `SimpleMac`
pub fn create_tagged_mac(secret: &[u8], data: &[u8]) -> Result<[u8; SimpleMac::SIZE]> {
    derive_mac_key(secret).map(|key| calculate_mac(&key, data))
}

#[cfg(test)]
mod tests {
    use crate::parameters::SECRET_KEY_LENGTH;
    use crate::primitives::{create_tagged_mac, DigestLike, SimpleMac, SimpleStreamCipher};
    use hex_literal::hex;

    #[test]
    fn test_chacha20() {
        let key = [0u8; 32];
        let mut iv = [0u8; 12];
        iv[11] = 2u8;

        let mut cipher = SimpleStreamCipher::new(&key, &iv).unwrap();

        let mut data = [0u8; 64];
        cipher.apply(&mut data);

        let expected_ct = hex!("c2c64d378cd536374ae204b9ef933fcd1a8b2288b3dfa49672ab765b54ee27c78a970e0e955c14f3a88e741b97c286f75f8fc299e8148362fa198a39531bed6d");
        assert_eq!(expected_ct, data);
    }

    #[test]
    fn test_chacha20_iv_block_counter() {
        let key = hex!("a9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f3");
        let iv = hex!("6be504b26471dea53d688c4b");

        let mut cipher = SimpleStreamCipher::new(&key, &iv).unwrap();

        cipher.set_block_counter(0xa5999171u32.to_be());

        let mut data = [0u8; 68];
        cipher.apply(&mut data);

        let expected_ct = hex!("abe088c198cb0a7b2591f1472fb1d0bd529a697a58a45d4ac5dc426ba6bf207deec4a5331149f93c6629d514ece8b0f49b4bc3eda74e07b78df5ac7d7f69fa75f611c926");
        assert_eq!(expected_ct, data);
    }

    #[test]
    fn test_mac() {
        let key = [1u8; SECRET_KEY_LENGTH];
        let data = [2u8; 64];
        let mac = create_tagged_mac(&key, &data).unwrap();

        let expected = hex!("a52161fd19f576948f13effe9fb66b5705607e626f5a6621c20c828495639d04");
        assert_eq!(SimpleMac::SIZE, mac.len());
        assert_eq!(expected, mac.as_ref())
    }
}

