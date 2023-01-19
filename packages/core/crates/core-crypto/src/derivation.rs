use crate::parameters;

use blake2::Blake2s256;
use hkdf::SimpleHkdf;
use hmac::{Mac, SimpleHmac};
use crate::errors::CryptoError::{InvalidInputValue, InvalidParameterSize};

use crate::errors::Result;
use crate::parameters::{HASH_KEY_PACKET_TAG, PACKET_TAG_LENGTH, SECRET_KEY_LENGTH};

/// Derives the commitment seed given the compressed private key representation
/// and the serialized channel information.
pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> Result<Box<[u8]>> {

    // Create HKDF instance and call the `expand` on with the given private key
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(private_key)
        .map_err(|_| InvalidInputValue)?;

    let mut generated_key = [0u8; SECRET_KEY_LENGTH];

    hkdf.expand(parameters::HASH_KEY_COMMITMENT_SEED.as_bytes(), &mut generated_key)
        .map_err(|_| InvalidInputValue)?;

    // Create HMAC instance and derive the commitment seed
    let mut mac = SimpleHmac::<Blake2s256>::new_from_slice(&generated_key)
        .map_err(|_| InvalidInputValue)?;

    mac.update(channel_info);
    let mac_value = mac.finalize().into_bytes();

    Ok(mac_value.as_slice().into())
}

pub fn derive_packet_tag(secret: &[u8]) -> Result<Box<[u8]>> {
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret)
        .map_err(|_| InvalidParameterSize{name: "secret".into(), expected: SECRET_KEY_LENGTH})?;

    let mut out = vec![0u8; PACKET_TAG_LENGTH];
    hkdf.expand(HASH_KEY_PACKET_TAG.as_bytes(), &mut out)
        .map_err(|_| InvalidInputValue)?;

    Ok(out.into_boxed_slice())
}

pub fn generate_key_iv(secret: &[u8], info: &[u8], key: &mut [u8], iv: &mut [u8], iv_first: bool) -> Result<()> {
    if secret.len() != SECRET_KEY_LENGTH {
        return Err(InvalidParameterSize{name: "secret".into(), expected: SECRET_KEY_LENGTH})
    }

    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret)
        .map_err(|_| InvalidParameterSize{name: "secret".into(), expected: SECRET_KEY_LENGTH})?;

    let mut out = vec![0u8; key.len() + iv.len()];
    hkdf.expand(info, &mut out)
        .map_err(|_| InvalidInputValue)?;

    if iv_first {
        let (v_iv, v_key) = out.split_at(iv.len());
        iv.copy_from_slice(v_iv);
        key.copy_from_slice(v_key);
    }
    else {
        let (v_key, v_iv) = out.split_at(key.len());
        key.copy_from_slice(v_key);
        iv.copy_from_slice(v_iv);
    }

    Ok(())
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use crate::utils::{as_jsvalue, JsResult};

    #[wasm_bindgen]
    pub fn derive_packet_tag(secret: &[u8]) -> JsResult<Box<[u8]>> {
        super::derive_packet_tag(secret).map_err(as_jsvalue)
    }

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

        let priv_key = [0u8; SECRET_KEY_LENGTH];
        let chinfo = [0u8; SECRET_KEY_LENGTH];

        let res = derive_commitment_seed(&priv_key, &chinfo);
        assert_eq!(false, res.is_err());

        let r = hex!("6CBD916300C24CC0DA636490668A4D85A4F42113496FCB452099F76131A3662E");
        assert_eq!(r, res.unwrap().as_ref());
    }
}

