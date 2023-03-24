use blake2::Blake2s256;
use hkdf::SimpleHkdf;
use crate::errors::CryptoError::{InvalidInputValue, InvalidParameterSize};

use crate::errors::Result;
use crate::parameters::{PACKET_TAG_LENGTH, SECRET_KEY_LENGTH};
use crate::primitives::calculate_mac;

// Module-specific constants
const HASH_KEY_COMMITMENT_SEED: &str = "HASH_KEY_COMMITMENT_SEED";
const HASH_KEY_HMAC: &str = "HASH_KEY_HMAC";
const HASH_KEY_PACKET_TAG: &str = "HASH_KEY_PACKET_TAG";

/// Helper function to expand an already cryptographically strong key material using the HKDF expand function
fn hkdf_expand_from_prk<const OUT_LENGTH: usize>(secret: &[u8], tag: &[u8]) -> Result<[u8; OUT_LENGTH]> {
    // Create HKDF instance
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret)
        .map_err(|_| InvalidParameterSize{name: "secret".into(), expected: SECRET_KEY_LENGTH})?;

    // Expand the key to the required length
    let mut out = [0u8; OUT_LENGTH];
    hkdf.expand(tag, &mut out)
        .map_err(|_| InvalidInputValue)?;

    Ok(out)
}

/// Derives the commitment seed given the compressed private key representation
/// and the serialized channel information.
pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> Result<Box<[u8]>> {
    hkdf_expand_from_prk::<SECRET_KEY_LENGTH>(private_key, HASH_KEY_COMMITMENT_SEED.as_bytes())
        .and_then(|key| calculate_mac(&key, channel_info))
}

/// Derives the packet tag used during packet construction by expanding the given secret.
pub fn derive_packet_tag(secret: &[u8]) -> Result<Box<[u8]>> {
    hkdf_expand_from_prk::<PACKET_TAG_LENGTH>(secret, HASH_KEY_PACKET_TAG.as_bytes())
        .map(Box::from)
}

/// Derives a key for MAC calculation by expanding the given secret.
pub fn derive_mac_key(secret: &[u8]) -> Result<Box<[u8]>> {
    hkdf_expand_from_prk::<SECRET_KEY_LENGTH>(secret, HASH_KEY_HMAC.as_bytes())
        .map(Box::from)
}

pub(crate) fn generate_key_iv(secret: &[u8], info: &[u8], key: &mut [u8], iv: &mut [u8], iv_first: bool) -> Result<()> {
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
    use utils_misc::utils::wasm::JsResult;
    use utils_misc::ok_or_jserr;

    #[wasm_bindgen]
    pub fn derive_packet_tag(secret: &[u8]) -> JsResult<Box<[u8]>> {
        ok_or_jserr!(super::derive_packet_tag(secret))
    }

    #[wasm_bindgen]
    pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> JsResult<Box<[u8]>> {
        ok_or_jserr!(super::derive_commitment_seed(private_key, channel_info))
    }

    #[wasm_bindgen]
    pub fn derive_mac_key(secret: &[u8]) -> JsResult<Box<[u8]>> {
        ok_or_jserr!(super::derive_mac_key(secret))
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

        let res = derive_commitment_seed(&priv_key, &chinfo).unwrap();

        let r = hex!("6CBD916300C24CC0DA636490668A4D85A4F42113496FCB452099F76131A3662E");
        assert_eq!(r, res.as_ref());
    }

    #[test]
    fn test_derive_packet_tag() {
        let secret = [0u8; SECRET_KEY_LENGTH];
        let tag = derive_packet_tag(&secret).unwrap();

        let r = hex!("e0cf0fb82ea5a541b0367b376eb36a60");
        assert_eq!(r, tag.as_ref());
    }

    #[test]
    fn test_derive_mac_key() {
        let secret = [0u8; SECRET_KEY_LENGTH];
        let tag = derive_mac_key(&secret).unwrap();

        let r = hex!("7f656daaf7c2e64bcfc1386f8af273890e863dec63b410967a5652630617b09b");
        assert_eq!(r, tag.as_ref());
    }
}

