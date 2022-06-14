use blake2::Blake2s256;
use chacha20::ChaCha20;
use chacha20::cipher::StreamCipher;
use hmac::{Mac, SimpleHmac};
use chacha20::cipher::KeyIvInit;
use digest::FixedOutput;
use crate::shared_keys::KeyBytes;

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

    pub fn finalize(&self) -> Box<[u8]> {
        let mut ret = [0u8; crate::parameters::SECRET_KEY_LENGTH];
        self.instance.finalize_into(KeyBytes::from_mut_slice(&mut ret));
        ret.into()
    }
}

pub struct SimpleStreamCipher {
    instance: ChaCha20
}

impl SimpleStreamCipher {
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, String> {
        Ok(Self {
            instance: ChaCha20::new_from_slices(key, iv).map_err(|e| e.to_string())?
        })
    }

    pub fn apply(&mut self, data: &mut [u8]) {
        self.instance.apply_keystream(data);
    }

    pub fn apply_copy(&mut self, data: &[u8]) -> Box<[u8]> {
        let mut ret = Vec::from(data);
        self.instance.apply_keystream(ret.as_mut_slice());
        ret.into_boxed_slice()
    }
}

/// Calculates ordinary MAC using the given key and data.
pub fn calculate_mac(key: &[u8], data: &[u8]) -> Result<Box<[u8]>, String> {
    let mut mac = SimpleMac::new(key)?;
    mac.update(data);
    Ok(mac.finalize())
}

pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    use crate::utils::as_jsvalue;

    #[wasm_bindgen]
    pub fn calculate_mac(key: &[u8], data: &[u8]) -> Result<Box<[u8]>, JsValue> {
        super::calculate_mac(key, data).map_err(as_jsvalue)
    }
}