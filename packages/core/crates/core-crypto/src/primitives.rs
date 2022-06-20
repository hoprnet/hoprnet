use blake2::Blake2s256;
use chacha20::ChaCha20;
use chacha20::cipher::{IvSizeUser, StreamCipher};
use hmac::{Mac, SimpleHmac};
use chacha20::cipher::KeyIvInit;
use digest::FixedOutputReset;

/// Simple Message Authentication Code (MAC) computation wrapper
/// Use `new`, `update` and `finalize` triplet to produce MAC of arbitrary data.
/// Currently this instance is computing HMAC based on Blake2s256
pub struct SimpleMac {
    instance: SimpleHmac<Blake2s256>
}

impl SimpleMac {
    pub fn new(key: &[u8]) -> Result<Self, String> {
        Ok(Self {
            instance: SimpleHmac::<Blake2s256>::new_from_slice(key).map_err(|e| e.to_string())?
        })
    }

    pub fn update(&mut self, data: &[u8]) {
        self.instance.update(data);
    }

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
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, String> {
        let chacha_iv_size = ChaCha20::iv_size();
        if iv.len() >= chacha_iv_size {
            Ok(Self {
                instance: ChaCha20::new_from_slices(key, &iv[0..chacha_iv_size]).map_err(|e| e.to_string())?
            })
        }
        else {
            Err("IV too small".into())
        }
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
pub fn calculate_mac(key: &[u8], data: &[u8]) -> Result<Box<[u8]>, String> {
    let mut mac = SimpleMac::new(key)?;
    mac.update(data);
    Ok(mac.finalize())
}

/// Functions and types exposed to WASM
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    use crate::utils::as_jsvalue;

    #[wasm_bindgen]
    pub fn calculate_mac(key: &[u8], data: &[u8]) -> Result<Box<[u8]>, JsValue> {
        super::calculate_mac(key, data).map_err(as_jsvalue)
    }
}