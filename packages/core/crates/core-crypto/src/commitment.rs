use crate::parameters;

use blake2::Blake2s256;
use hkdf::SimpleHkdf;
use hmac::{SimpleHmac, Mac};
use crate::errors::CryptoError::InvalidInputSize;

use crate::errors::Result;

/// Derives the commitment seed given the compressed private key representation
/// and the serialized channel information.
pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> Result<Box<[u8]>> {

    // Create HKDF instance and call the `expand` on with the given private key
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(private_key)
        .map_err(|_| InvalidInputSize)?;

    let mut generated_key = [0u8; parameters::SECRET_KEY_LENGTH];

    hkdf.expand(parameters::HASH_KEY_COMMITMENT_SEED.as_bytes(), &mut generated_key)
        .map_err(|_| InvalidInputSize)?;

    // Create HMAC instance and derive the commitment seed
    let mut mac = SimpleHmac::<Blake2s256>::new_from_slice(&generated_key)
        .map_err(|_| InvalidInputSize)?;

    mac.update(channel_info);
    let mac_value = mac.finalize().into_bytes();

    Ok(mac_value.as_slice().into())
}

pub mod wasm {
    use wasm_bindgen::prelude::*;
    use crate::utils::{as_jsvalue, JsResult};

    #[wasm_bindgen]
    pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> JsResult<Box<[u8]>> {
        super::derive_commitment_seed(private_key, channel_info).map_err(as_jsvalue)
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use super::*;

    #[test]
    fn test_derive_commitment_seed() {

        let priv_key = [0u8; parameters::SECRET_KEY_LENGTH];
        let chinfo = [0u8; parameters::SECRET_KEY_LENGTH];

        let res = derive_commitment_seed(&priv_key, &chinfo);
        assert_eq!(false, res.is_err());

        let r = hex!("6CBD916300C24CC0DA636490668A4D85A4F42113496FCB452099F76131A3662E");
        assert_eq!(r, res.unwrap().as_ref());
    }
}