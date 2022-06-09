use std::fmt::Display;
use wasm_bindgen::prelude::*;

use crate::constants;

use blake2::Blake2s256;
use hkdf::SimpleHkdf;
use hmac::{SimpleHmac, Mac};

fn as_jsvalue<T>(v: T) -> JsValue where T: Display {
    JsValue::from(v.to_string())
}

#[wasm_bindgen]
pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> Result<Box<[u8]>, JsValue> {

    // Create HKDF instance and call the `expand` on with the given private key
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(private_key)
        .map_err(as_jsvalue)?;

    let mut generated_key = [0u8; constants::SECRET_KEY_LENGTH];

    hkdf.expand(constants::HASH_KEY_COMMITMENT_SEED.as_bytes(), &mut generated_key)
        .map_err(as_jsvalue)?;

    // Create HMAC instance and derive the commitment seed
    let mut mac = SimpleHmac::<Blake2s256>::new_from_slice(&generated_key)
        .map_err(as_jsvalue)?;

    mac.update(channel_info);
    let mac_value = mac.finalize().into_bytes();

    Ok(Vec::<u8>::from_iter(mac_value.into_iter()).into_boxed_slice())
}

#[cfg(test)]
mod tests {

    use wasm_bindgen_test::*;
    use super::*;

    #[wasm_bindgen_test]
    fn test_derive_commitment_seed() {

        let priv_key = [0u8; constants::SECRET_KEY_LENGTH];
        let chinfo = [0u8; constants::SECRET_KEY_LENGTH];

        let res = derive_commitment_seed(&priv_key, &chinfo);

        assert_eq!(false, res.is_err());
    }
}