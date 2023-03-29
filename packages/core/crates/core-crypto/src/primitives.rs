use blake2::Blake2s256;
use chacha20::ChaCha20;
use chacha20::cipher::{IvSizeUser, KeySizeUser, StreamCipher, StreamCipherSeek};
use hmac::{Mac, SimpleHmac};
use chacha20::cipher::KeyIvInit;
use digest::FixedOutputReset;
use crate::derivation::derive_mac_key;

use crate::errors::Result;
use crate::errors::CryptoError::{InvalidInputValue, InvalidParameterSize};
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
        if iv.len() != chacha_iv_size {
            return Err(InvalidParameterSize {name: "iv".into(), expected: chacha_iv_size})
        }

        let chacha_key_size = ChaCha20::key_size();
        if key.len() != chacha_key_size {
            return Err(InvalidParameterSize {name: "key".into(), expected: chacha_key_size})
        }

        Ok(Self {
            instance: ChaCha20::new_from_slices(key, iv).map_err(|_| InvalidInputValue)?
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

/// Calculates MAC using the given raw key and data.
/// Uses HMAC based on Blake2s256.
pub fn calculate_mac(key: &[u8], data: &[u8]) -> Result<Box<[u8]>> {
    let mut mac = SimpleMac::new(key)?;
    mac.update(data);
    Ok(mac.finalize())
}

/// Calculates a message authentication code with fixed key tag (HASH_KEY_HMAC)
/// The given secret is first transformed using HKDF before the MAC calculation is performed.
/// Uses HMAC based on Blake2s256.
pub fn create_tagged_mac(secret: &[u8], data: &[u8]) -> Result<Box<[u8]>> {
    calculate_mac(&derive_mac_key(secret)?, data)
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use crate::parameters::SECRET_KEY_LENGTH;
    use crate::primitives::SimpleStreamCipher;
    use crate::primitives::wasm::create_tagged_mac;

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
        assert_eq!(expected, mac.as_ref())
    }
}

/// Functions and types exposed to WASM
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use utils_misc::utils::wasm::JsResult;
    use utils_misc::ok_or_jserr;

    #[wasm_bindgen]
    pub fn calculate_mac(key: &[u8], data: &[u8]) -> JsResult<Box<[u8]>> {
        ok_or_jserr!(super::calculate_mac(key, data))
    }

    #[wasm_bindgen]
    pub fn create_tagged_mac(secret: &[u8], data: &[u8]) -> JsResult<Box<[u8]>> {
        ok_or_jserr!(super::create_tagged_mac(secret, data))
    }
}