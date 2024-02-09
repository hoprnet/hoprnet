use blake2::{Blake2s256, Blake2sMac256};
use chacha20::cipher::KeyIvInit;
use chacha20::cipher::{IvSizeUser, KeySizeUser, StreamCipher, StreamCipherSeek};
use chacha20::ChaCha20;
use digest::{FixedOutputReset, KeyInit, Output, OutputSizeUser, Update};
use generic_array::GenericArray;
use sha3::Keccak256;
use typenum::Unsigned;
use zeroize::ZeroizeOnDrop;

use crate::utils::SecretValue;

/// Represents a secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey = SecretValue<typenum::U32>;

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
pub struct SimpleDigest(Blake2s256);

impl DigestLike<Blake2s256> for SimpleDigest {
    fn internal_state(&mut self) -> &mut Blake2s256 {
        &mut self.0
    }
}

/// Computation wrapper for a digest that's compatible with Ethereum digests.
/// Use `new`, `update` and `finalize` triplet to produce hash of arbitrary data.
/// Currently this instance is using Keccak256.
#[derive(Default, Clone)]
pub struct EthDigest(Keccak256);

impl DigestLike<Keccak256> for EthDigest {
    fn internal_state(&mut self) -> &mut Keccak256 {
        &mut self.0
    }
}

/// Simple Message Authentication Code (MAC) computation wrapper
/// Use `new`, `update` and `finalize` triplet to produce MAC of arbitrary data.
/// Currently instantiated using Blake2s256 MAC.
pub struct SimpleMac(Blake2sMac256); // TODO: add derive(ZeroizeOnDrop) once blake2 is updated to 0.11

impl SimpleMac {
    /// Create new instance of the MAC using the given secret key.
    pub fn new(key: &SecretKey) -> Self {
        Self(Blake2sMac256::new(key.into()))
    }
}

impl DigestLike<Blake2sMac256> for SimpleMac {
    fn internal_state(&mut self) -> &mut Blake2sMac256 {
        &mut self.0
    }
}

/// Simple stream cipher wrapper
/// Use `new` and `apply` (or `apply_copy`) to XOR the keystream on the plaintext or ciphertext.
/// Currently this instance is using ChaCha20.
#[derive(ZeroizeOnDrop)]
pub struct SimpleStreamCipher(ChaCha20);

impl SimpleStreamCipher {
    /// Size of the secret key
    pub const KEY_SIZE: usize = <ChaCha20 as KeySizeUser>::KeySize::USIZE;

    /// Size of the initialization vector
    pub const IV_SIZE: usize = <ChaCha20 as IvSizeUser>::IvSize::USIZE;

    /// Create new instance of the stream cipher initialized
    /// with the given secret key and IV.
    pub fn new(key: [u8; Self::KEY_SIZE], iv: [u8; Self::IV_SIZE]) -> Self {
        Self(ChaCha20::new(&key.into(), &iv.into()))
    }

    /// Seeks the keystream to the given block position
    pub fn set_block_counter(&mut self, counter: u32) {
        self.0.seek(counter as u64 * 64u64)
    }

    /// Apply keystream to the given data in-place.
    pub fn apply(&mut self, data: &mut [u8]) {
        self.0.apply_keystream(data);
    }

    /// Creates copy of the given data and applies the keystream to it.
    pub fn apply_copy(&mut self, data: &[u8]) -> Box<[u8]> {
        let mut ret = Vec::from(data);
        self.0.apply_keystream(ret.as_mut_slice());
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_chacha20() {
        let key = [0u8; 32];
        let mut iv = [0u8; 12];
        iv[11] = 2u8;

        let mut cipher = SimpleStreamCipher::new(key, iv);

        let mut data = [0u8; 64];
        cipher.apply(&mut data);

        let expected_ct = hex!("c2c64d378cd536374ae204b9ef933fcd1a8b2288b3dfa49672ab765b54ee27c78a970e0e955c14f3a88e741b97c286f75f8fc299e8148362fa198a39531bed6d");
        assert_eq!(expected_ct, data);
    }

    #[test]
    fn test_chacha20_iv_block_counter() {
        let key = hex!("a9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f3");
        let iv = hex!("6be504b26471dea53d688c4b");

        let mut cipher = SimpleStreamCipher::new(key, iv);

        cipher.set_block_counter(0xa5999171u32.to_be());

        let mut data = [0u8; 68];
        cipher.apply(&mut data);

        let expected_ct = hex!("abe088c198cb0a7b2591f1472fb1d0bd529a697a58a45d4ac5dc426ba6bf207deec4a5331149f93c6629d514ece8b0f49b4bc3eda74e07b78df5ac7d7f69fa75f611c926");
        assert_eq!(expected_ct, data);
    }

    #[test]
    fn test_mac() {
        let data = [0u8; 60];

        let mut mac = SimpleMac::new(&Default::default());
        mac.update(&data);

        let out = mac.finalize();
        let expected_out = hex!("29a11c3c01d102143a020fb562410902ba39966ec81b706945b8ed6d83498bb7");
        assert_eq!(expected_out, out.as_slice());
    }
}
