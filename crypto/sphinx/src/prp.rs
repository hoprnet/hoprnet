use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;

use hopr_crypto_types::primitives::SecretKey;
use zeroize::ZeroizeOnDrop;

use crate::derivation::generate_key_iv;

/// Abstraction for Pseudo-Random Permutation.
pub trait PRP: From<SecretKey> {
    /// Applies forward permutation on the given plaintext and modifies the given buffer in-place.
    fn forward_inplace(&mut self, plaintext: &mut [u8]) -> hopr_crypto_types::errors::Result<()>;

    /// Applies inverse permutation on the given ciphertext and modifies the given buffer in-place.
    fn inverse_inplace(&mut self, ciphertext: &mut [u8]) -> hopr_crypto_types::errors::Result<()>;
}

#[derive(ZeroizeOnDrop)]
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

const HASH_KEY_PRP: &str = "HASH_KEY_PRP";
