use crate::{
    errors::{
        CryptoError::{CalculationError, InvalidInputValue, InvalidParameterSize},
        Result,
    },
    parameters::{PACKET_TAG_LENGTH, PING_PONG_NONCE_SIZE, SECRET_KEY_LENGTH},
    primitives::{calculate_mac, DigestLike, SimpleDigest},
    random::{random_bytes, random_fill},
    types::HalfKey,
};
use blake2::Blake2s256;
use elliptic_curve::{
    hash2curve::{ExpandMsgXmd, GroupDigest},
    sec1::ToEncodedPoint,
    ScalarPrimitive,
};
use hkdf::SimpleHkdf;
use k256::{AffinePoint, Scalar, Secp256k1};
use utils_types::{primitives::Address, traits::BinarySerializable};

// Module-specific constants
const HASH_KEY_COMMITMENT_SEED: &str = "HASH_KEY_COMMITMENT_SEED";
const HASH_KEY_HMAC: &str = "HASH_KEY_HMAC";
const HASH_KEY_PACKET_TAG: &str = "HASH_KEY_PACKET_TAG";
const HASH_KEY_OWN_KEY: &str = "HASH_KEY_OWN_KEY";
const HASH_KEY_ACK_KEY: &str = "HASH_KEY_ACK_KEY";

/// Helper function to expand an already cryptographically strong key material using the HKDF expand function
fn hkdf_expand_from_prk<const OUT_LENGTH: usize>(secret: &[u8], tag: &[u8]) -> Result<[u8; OUT_LENGTH]> {
    // Create HKDF instance
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret).map_err(|_| InvalidParameterSize {
        name: "secret".into(),
        expected: SECRET_KEY_LENGTH,
    })?;

    // Expand the key to the required length
    let mut out = [0u8; OUT_LENGTH];
    hkdf.expand(tag, &mut out).map_err(|_| InvalidInputValue)?;

    Ok(out)
}

/// Derives a ping challenge (if no challenge is given) or a pong response to a ping challenge.
pub fn derive_ping_pong(challenge: Option<&[u8]>) -> Box<[u8]> {
    let mut ret = [0u8; PING_PONG_NONCE_SIZE];
    match challenge {
        None => random_fill(&mut ret),
        Some(chal) => {
            let mut digest = SimpleDigest::default();
            digest.update(chal);
            // Finalize requires enough space for the hash value, so this needs an extra copy
            let hash = digest.finalize();
            ret.copy_from_slice(&hash[0..PING_PONG_NONCE_SIZE]);
        }
    }
    ret.into()
}

/// Derives the commitment seed given the compressed private key representation
/// and the serialized channel information.
pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> Result<Box<[u8]>> {
    hkdf_expand_from_prk::<SECRET_KEY_LENGTH>(private_key, HASH_KEY_COMMITMENT_SEED.as_bytes())
        .and_then(|key| calculate_mac(&key, channel_info))
}

/// Derives the packet tag used during packet construction by expanding the given secret.
pub fn derive_packet_tag(secret: &[u8]) -> Result<Box<[u8]>> {
    hkdf_expand_from_prk::<PACKET_TAG_LENGTH>(secret, HASH_KEY_PACKET_TAG.as_bytes()).map(Box::from)
}

/// Derives a key for MAC calculation by expanding the given secret.
pub fn derive_mac_key(secret: &[u8]) -> Result<Box<[u8]>> {
    hkdf_expand_from_prk::<SECRET_KEY_LENGTH>(secret, HASH_KEY_HMAC.as_bytes()).map(Box::from)
}

/// Internal convenience function to generate key and IV from the given secret.
/// WARNING: The `iv_first` distinguishes if the IV should be sampled before or after the key is sampled.
pub(crate) fn generate_key_iv(secret: &[u8], info: &[u8], key: &mut [u8], iv: &mut [u8], iv_first: bool) -> Result<()> {
    if secret.len() != SECRET_KEY_LENGTH {
        return Err(InvalidParameterSize {
            name: "secret".into(),
            expected: SECRET_KEY_LENGTH,
        });
    }

    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret).map_err(|_| InvalidParameterSize {
        name: "secret".into(),
        expected: SECRET_KEY_LENGTH,
    })?;

    let mut out = vec![0u8; key.len() + iv.len()];
    hkdf.expand(info, &mut out).map_err(|_| InvalidInputValue)?;

    if iv_first {
        let (v_iv, v_key) = out.split_at(iv.len());
        iv.copy_from_slice(v_iv);
        key.copy_from_slice(v_key);
    } else {
        let (v_key, v_iv) = out.split_at(key.len());
        key.copy_from_slice(v_key);
        iv.copy_from_slice(v_iv);
    }

    Ok(())
}

/// Sample a random secp256k1 field element that can represent a valid secp256k1 point.
/// The implementation uses `hash_to_field` function as defined in
/// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-13.html#name-hashing-to-a-finite-field
/// The `tag` parameter will be used as an additional Domain Separation Tag.
pub fn sample_field_element(secret: &[u8], tag: &str) -> Result<HalfKey> {
    let scalar = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Sha3_256>>(
        &[secret],
        &[b"secp256k1_XMD:SHA3-256_SSWU_NU_", tag.as_bytes()],
    )
    .map_err(|_| CalculationError)?;
    Ok(HalfKey::new(scalar.to_bytes().as_ref()))
}

/// Used in Proof of Relay to derive own half-key (S0)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
pub fn derive_own_key_share(secret: &[u8]) -> HalfKey {
    assert_eq!(SECRET_KEY_LENGTH, secret.len());

    sample_field_element(secret, HASH_KEY_OWN_KEY).expect("failed to sample own key share")
}

/// Used in Proof of Relay to derive the half-key of for the acknowledgement (S1)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
pub fn derive_ack_key_share(secret: &[u8]) -> HalfKey {
    assert_eq!(SECRET_KEY_LENGTH, secret.len());

    sample_field_element(secret, HASH_KEY_ACK_KEY).expect("failed to sample ack key share")
}

/// Bundles values given to the smart contract to prove that a ticket is a win.
///
/// The VRF is thereby needed because it generates on-demand determinitstic
/// entropy that can only be derived by the ticket redeemer.
pub struct VrfParameters {
    /// the pseudo-random point
    pub v: AffinePoint,
    pub h: Scalar,
    pub s: Scalar,
    /// helper value for smart contract
    pub h_v: AffinePoint,
    /// helper value for smart contract
    pub s_b: AffinePoint,
}

impl std::fmt::Display for VrfParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v_encoded = self.v.to_encoded_point(false);
        let h_v_encoded = self.h_v.to_encoded_point(false);
        let s_b_encoded = self.s_b.to_encoded_point(false);
        f.debug_struct("VrfParameters")
            .field(
                "V",
                &format!(
                    "({},{})",
                    hex::encode(v_encoded.x().unwrap()),
                    hex::encode(v_encoded.y().unwrap())
                ),
            )
            .field("h", &hex::encode(self.h.to_bytes()))
            .field("s", &hex::encode(self.s.to_bytes()))
            .field(
                "h_v",
                &format!(
                    "({},{})",
                    hex::encode(h_v_encoded.x().unwrap()),
                    hex::encode(h_v_encoded.y().unwrap())
                ),
            )
            .field(
                "s_b",
                &format!(
                    "({},{})",
                    hex::encode(s_b_encoded.x().unwrap()),
                    hex::encode(s_b_encoded.y().unwrap())
                ),
            )
            .finish()
    }
}

/// Takes a private key, the corresponding Ethereum address and a payload
/// and creates all parameters that are required by the smart contract
/// to prove that a ticket is a win.
pub fn derive_vrf_parameters<const T: usize>(
    msg: &[u8; T],
    secret: &[u8],
    chain_addr: &Address,
    dst: &[u8],
) -> Result<VrfParameters> {
    let b = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&chain_addr.to_bytes(), msg], &[dst])?;

    let a: Scalar = ScalarPrimitive::<Secp256k1>::from_slice(&secret)?.into();

    if a.is_zero().into() {
        return Err(crate::errors::CryptoError::InvalidSecretScalar);
    }

    let v = b * a;

    let r = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            &a.to_bytes(),
            &v.to_affine().to_encoded_point(false).as_bytes()[1..],
            &random_bytes::<64>(),
        ],
        &[dst],
    )?;

    let r_v = b * r;

    let h = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            &chain_addr.to_bytes(),
            &v.to_affine().to_encoded_point(false).as_bytes()[1..],
            &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
            msg,
        ],
        &[dst],
    )?;
    let s = r + h * a;

    Ok(VrfParameters {
        v: v.to_affine(),
        h,
        s,
        h_v: (v * h).to_affine(),
        s_b: (b * s).to_affine(),
    })
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;

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
    use super::*;
    use crate::types::PublicKey;
    use hex_literal::hex;

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

    #[test]
    fn test_sample_field_element() {
        let secret = [1u8; SECRET_KEY_LENGTH];
        assert!(sample_field_element(&secret, "TEST_TAG").is_ok());
    }

    #[test]
    fn test_vrf_parameter_generation() {
        let dst = b"some DST tag";
        let priv_key: [u8; 32] = hex!("f13233ff60e1f618525dac5f7d117bef0bad0eb0b0afb2459f9cbc57a3a987ba"); // dummy
        let message = hex!("f13233ff60e1f618525dac5f7d117bef0bad0eb0b0afb2459f9cbc57a3a987ba"); // dummy

        // vrf verification algorithm
        let pub_key = PublicKey::from_privkey(&priv_key).unwrap();

        let params = derive_vrf_parameters(&message, &priv_key, &pub_key.to_address(), dst).unwrap();

        let cap_b = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(
            &[&pub_key.to_address().to_bytes(), &message],
            &[dst],
        )
        .unwrap();

        assert_eq!(params.s_b, cap_b * params.s);

        let a: Scalar = ScalarPrimitive::<Secp256k1>::from_slice(&priv_key).unwrap().into();
        assert_eq!(params.h_v, cap_b * a * params.h);

        let r_v = cap_b * params.s - params.v * params.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
            &[
                &pub_key.to_address().to_bytes(),
                &params.v.to_encoded_point(false).as_bytes()[1..],
                &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
                &message,
            ],
            &[dst],
        )
        .unwrap();

        assert_eq!(h_check, params.h);
    }
}
