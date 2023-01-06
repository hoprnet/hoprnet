use blake2::Blake2s256;
use chacha20::ChaCha20;
use chacha20::cipher::{IvSizeUser, KeySizeUser, StreamCipher};
use hmac::{Mac, SimpleHmac};
use chacha20::cipher::KeyIvInit;
use digest::FixedOutputReset;

use crate::errors::Result;
use crate::errors::CryptoError::{InvalidInputSize, InvalidParameterSize};
use crate::parameters::SECRET_KEY_LENGTH;

/// Simple Message Authentication Code (MAC) computation wrapper
/// Use `new`, `update` and `finalize` triplet to produce MAC of arbitrary data.
/// Currently this instance is computing HMAC based on Blake2s256
pub struct SimpleMac {
    instance: SimpleHmac<Blake2s256>
}

impl SimpleMac {

    /// Create new instance of the MAC using the given secret key.
    pub fn new(key: &[u8]) -> Result<Self> {
        Ok(Self {
            instance: SimpleHmac::<Blake2s256>::new_from_slice(key).map_err(|_| InvalidParameterSize {name: "key".into(), expected: SECRET_KEY_LENGTH})?
        })
    }

    /// Update the internal state of the MAC using the given input data.
    pub fn update(&mut self, data: &[u8]) {
        self.instance.update(data);
    }

    /// Retrieve the final MAC and reset this instance so it could be reused for
    /// a new MAC computation.
    pub fn finalize(&mut self) -> Box<[u8]> {
        self.instance.finalize_fixed_reset().to_vec().into_boxed_slice()
    }
}

/// Simple stream cipher wrapper
/// Use `new` and `apply` (or `apply_copy`) to XOR the keystream on the plaintext or ciphertext.
/// Currently this instance is using ChaCha20.
pub struct SimpleStreamCipher {
    instance: ChaCha20
}

impl SimpleStreamCipher {

    /// Create new instance of the stream cipher initialized
    /// with the given secret key and IV.
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self> {
        let chacha_iv_size = ChaCha20::iv_size();
        if iv.len() < chacha_iv_size {
            return Err(InvalidParameterSize {name: "iv".into(), expected: chacha_iv_size})
        }

        let chacha_key_size = ChaCha20::key_size();
        if key.len() < chacha_key_size {
            return Err(InvalidParameterSize {name: "key".into(), expected: chacha_key_size})
        }

        Ok(Self {
            instance: ChaCha20::new_from_slices(key, &iv[0..chacha_iv_size]).map_err(|_| InvalidInputSize)?
        })
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

/// Calculates MAC using the given key and data.
/// Uses HMAC based on Blake2s256.
pub fn calculate_mac(key: &[u8], data: &[u8]) -> Result<Box<[u8]>> {
    let mut mac = SimpleMac::new(key)?;
    mac.update(data);
    Ok(mac.finalize())
}

/// Functions and types exposed to WASM
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use crate::utils::{as_jsvalue, JsResult};

    #[wasm_bindgen]
    pub fn calculate_mac(key: &[u8], data: &[u8]) -> JsResult<Box<[u8]>> {
        super::calculate_mac(key, data).map_err(as_jsvalue)
    }
}