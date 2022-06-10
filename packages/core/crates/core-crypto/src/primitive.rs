use blake2::Blake2s256;
use hmac::{Mac, SimpleHmac};
use js_sys::Uint8Array;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::utils::as_jsvalue;

/// Calculates ordinary MAC using the given key and data.
#[wasm_bindgen]
pub fn create_mac(key: &[u8], data: &[u8]) -> Result<Uint8Array, JsValue> {

    let mut mac = SimpleHmac::<Blake2s256>::new_from_slice(key)
        .map_err(as_jsvalue)?;

    mac.update(data);
    let mac_value = mac.finalize().into_bytes();

    Ok(Uint8Array::from(mac_value.as_slice()))
}