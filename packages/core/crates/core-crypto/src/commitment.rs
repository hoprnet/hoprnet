use wasm_bindgen::prelude::*;

use crate::parameters;
use crate::utils::as_jsvalue;

use blake2::Blake2s256;
use hkdf::SimpleHkdf;
use hmac::{SimpleHmac, Mac};
use js_sys::Uint8Array;

#[wasm_bindgen]
pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> Result<Uint8Array, JsValue> {

    // Create HKDF instance and call the `expand` on with the given private key
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(private_key)
        .map_err(as_jsvalue)?;

    let mut generated_key = [0u8; parameters::SECRET_KEY_LENGTH];

    hkdf.expand(parameters::HASH_KEY_COMMITMENT_SEED.as_bytes(), &mut generated_key)
        .map_err(as_jsvalue)?;

    // Create HMAC instance and derive the commitment seed
    let mut mac = SimpleHmac::<Blake2s256>::new_from_slice(&generated_key)
        .map_err(as_jsvalue)?;

    mac.update(channel_info);
    let mac_value = mac.finalize().into_bytes();

    Ok(Uint8Array::from(mac_value.as_slice()))
}

#[cfg(test)]
mod tests {

    use wasm_bindgen_test::*;
    use super::*;

    #[wasm_bindgen_test]
    fn test_derive_commitment_seed() {

        let priv_key = [0u8; parameters::SECRET_KEY_LENGTH];
        let chinfo = [0u8; parameters::SECRET_KEY_LENGTH];

        let res = derive_commitment_seed(&priv_key, &chinfo);

        assert_eq!(false, res.is_err());
    }
}