use std::ops::Mul;
use blake2::Blake2s256;

use elliptic_curve::ProjectivePoint;
use elliptic_curve::rand_core::{CryptoRng, RngCore};
use elliptic_curve::sec1::ToEncodedPoint;

use generic_array::GenericArray;

use k256::{AffinePoint, NonZeroScalar, Secp256k1};

use hkdf::SimpleHkdf;
use crate::errors::CryptoError::InvalidSecretScalar;

use crate::parameters;

use crate::errors::Result;
use crate::types::CurvePoint;

/// Type for the secret keys with fixed size
/// The GenericArray<..> is mostly deprecated since Rust 1.51 and it's introduction of const generics,
/// but we need to use it because elliptic_curves and all RustCrypto crates mostly expose it in their
/// public interfaces.
pub type KeyBytes = GenericArray<u8, typenum::U32>;

/// Extract a keying material from an EC point using HKDF extract
fn extract_key_from_group_element(group_element: &AffinePoint, salt: &[u8]) -> KeyBytes {
    // Create the compressed EC point representation first
    let compressed_element = group_element.to_encoded_point(true);

    SimpleHkdf::<Blake2s256>::extract(Some(salt), compressed_element.as_bytes()).0
}

/// Performs KDF expansion from the given EC point using HKDF expand
fn expand_key_from_group_element(group_element: &AffinePoint, salt: &[u8]) -> KeyBytes {
    // Create the compressed EC point representation first
    let compressed_element = group_element.to_encoded_point(true);

    let mut out = [0u8; parameters::SECRET_KEY_LENGTH];
    SimpleHkdf::<Blake2s256>::new(Some(salt), compressed_element.as_bytes())
        .expand(b"", &mut out)
        .unwrap(); // Cannot panic, unless the constants are wrong

    KeyBytes::from(out)
}

/// Decodes the public key and converts it into an EC point in projective coordinates
fn decode_public_key_to_point(encoded_public_key: &[u8]) -> Result<ProjectivePoint<Secp256k1>> {
    Ok(crate::types::PublicKey::deserialize(encoded_public_key)
        .map(|pk| CurvePoint::from(pk).to_projective_point())?)
    /*PublicKey::<Secp256k1>::from_sec1_bytes(encoded_public_key)
        .map(|decoded| ProjectivePoint::<Secp256k1>::from(decoded))
        .map_err(|e| EllipticCurveError(e))*/
}

/// Checks if the given key bytes can form a scalar for EC point
fn to_checked_secret_scalar(secret_scalar: KeyBytes) -> Result<NonZeroScalar> {
    let scalar = NonZeroScalar::from_repr(secret_scalar);
    match Option::from(scalar) {
        Some(s) => Ok(s),
        None => Err(InvalidSecretScalar)
    }
}

/// Structure containing shared keys for peers.
/// The members are exposed only using specialized methods.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct SharedKeys {
    alpha: Vec<u8>,
    secrets: Vec<Vec<u8>>
}

impl SharedKeys {

    /// Generates shared secrets given the peer public keys array.
    /// The order of the peer public keys is preserved for resulting shared keys.
    /// The specified random number generator will be used.
    pub fn generate(rng: &mut (impl CryptoRng + RngCore), peer_public_keys: Vec<Box<[u8]>>) -> Result<SharedKeys> {

        let mut shared_keys = Vec::new();

        // This becomes: x * b_0 * b_1 * b_2 * ...
        let mut coeff_prev = NonZeroScalar::random(rng);

        // This becomes: x * b_0 * b_1 * b_2 * ... * G
        // We remain in projective coordinates to save some cycles
        let mut alpha_prev = k256::ProjectivePoint::GENERATOR * coeff_prev.as_ref();
        let alpha = alpha_prev.to_encoded_point(true);

        // Iterate through all the given peer public keys
        for (i, pk) in peer_public_keys.iter().enumerate() {
            // Try to decode the given public key point & multiply by the current coefficient
            let decoded_proj_point = decode_public_key_to_point(pk)?;
            let shared_secret = (decoded_proj_point * coeff_prev.as_ref()).to_affine();

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            let shared_pk = extract_key_from_group_element(&shared_secret, pk);
            shared_keys.push(shared_pk.to_vec());

            // Stop here, we don't need to compute anything more
            if i == peer_public_keys.len() - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let enc_alpha_prev = alpha_prev.to_encoded_point(true);
            let b_k = expand_key_from_group_element(&shared_secret, enc_alpha_prev.as_bytes());
            let b_k_checked = to_checked_secret_scalar(b_k)?;

            // Update coeff prev and alpha
            coeff_prev = coeff_prev.mul(b_k_checked);
            alpha_prev = alpha_prev * b_k_checked.as_ref();
        }

        Ok(SharedKeys {
            alpha: alpha.as_bytes().into(),
            secrets: shared_keys
        })
    }

    /// Calculates the forward transformation for the given peer public key.
    pub fn forward_transform(alpha: &[u8], public_key: &[u8], private_key: &[u8]) -> Result<SharedKeys> {

        let priv_key = to_checked_secret_scalar(KeyBytes::clone_from_slice(&private_key[0..private_key.len()]))?;
        let alpha_proj = decode_public_key_to_point(alpha)?;

        let s_k = (alpha_proj * priv_key.as_ref()).to_affine();
        let secret = extract_key_from_group_element(&s_k, public_key);

        let b_k = expand_key_from_group_element(&s_k, alpha);
        let b_k_checked = to_checked_secret_scalar(b_k)?;

        let alpha_new = (alpha_proj * b_k_checked.as_ref()).to_affine().to_encoded_point(true);

        Ok(SharedKeys {
            alpha: alpha_new.as_bytes().into(),
            secrets: vec![secret.to_vec()]
        })
    }
}

/// Unit tests of the Rust code
#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use elliptic_curve::group::prime::PrimeCurveAffine;
    use elliptic_curve::rand_core::OsRng;

   #[test]
    fn test_decode_point() {
        let point = hex!("0253f6e72ad23de294466b830619448d6d9059a42050141cd83bac4e3ee82c3f1e");
        decode_public_key_to_point(&point).unwrap();
    }

    #[test]
    fn test_extract_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef ];
        let pt = AffinePoint::generator();

        let key = extract_key_from_group_element(&pt, &salt);
        assert_eq!(parameters::SECRET_KEY_LENGTH, key.len());

        let res = hex!("54BF34178075E153F481CE05B113C1530ECC45A2F1F13A3366D4389F65470DE6");
        assert_eq!(res, key.as_slice());
    }

    #[test]
    fn test_expand_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef ];
        let pt = AffinePoint::generator();

        let key = expand_key_from_group_element(&pt, &salt);
        assert_eq!(parameters::SECRET_KEY_LENGTH, key.len());

        let res = hex!("D138D9367474911F7124B95BE844D2F8A6D34E962694E37E8717BDBD3C15690B");
        assert_eq!(res, key.as_slice());
    }

    #[test]
    fn test_shared_keys() {
        // DummyRng is useful for deterministic debugging
        //let mut used_rng = crate::dummy_rng::DummyFixedRng::new();

        const COUNT_KEYPAIRS: usize = 3;
        let mut used_rng = OsRng;

        // Generate some random key pairs
        let (priv_keys, pub_keys): (Vec<Box<[u8]>>, Vec<Box<[u8]>>) = (0..COUNT_KEYPAIRS)
            .map(|_i| NonZeroScalar::random(&mut used_rng))
            .map(|s| (s, k256::ProjectivePoint::GENERATOR * s.as_ref()))
            .map(|p| (p.0, p.1.to_encoded_point(true)))
            .map(|p| (p.0.to_bytes(), p.1))
            .map(|p| (Box::from(p.0.as_slice()), Box::from(p.1.as_bytes())))
            .unzip();

        // Now generate the key shares for the public keys
        let generated_shares = SharedKeys::generate(&mut used_rng, pub_keys.clone()).unwrap();

        let mut alpha_cpy = generated_shares.alpha.clone();
        for i in 0..COUNT_KEYPAIRS {
            let priv_key = priv_keys[i].to_vec();
            let pub_key = pub_keys[i].to_vec();

            let shared_key = SharedKeys::forward_transform(alpha_cpy.as_slice(),
                                                           pub_key.as_slice(),
                                                           priv_key.as_slice()).unwrap();

            assert_eq!(&shared_key.secrets[0], &generated_shares.secrets[i]);

            alpha_cpy = shared_key.alpha.clone();
        }
    }

    #[test]
    fn test_key_shares() {
        let pub_keys: Vec<Box<[u8]>> = vec![
            Box::new(hex!("0253f6e72ad23de294466b830619448d6d9059a42050141cd83bac4e3ee82c3f1e")),
            Box::new(hex!("035fc5660f59059c263d3946d7abaf33fa88181e27bf298fcc5a9fa493bec9110b")),
            Box::new(hex!("038d2b50a77fd43eeae9b37856358c7f1aee773b3e3c9d26f30b8706c02cbbfbb6"))
        ];

        let keyshares = SharedKeys::generate(&mut OsRng, pub_keys).unwrap();
        assert_eq!(3, keyshares.secrets.len());
    }
}

/// This module contains wrapper for the Rust code
/// to be properly called from the JS.
/// Code in this module does not need to be unit tested, as it already
/// wraps code that has been unit tested in pure Rust.
#[cfg(feature = "wasm")]
pub mod wasm {
    use elliptic_curve::rand_core::OsRng;
    use wasm_bindgen::prelude::*;
    use js_sys::Uint8Array;
    use utils_misc::utils::wasm::JsResult;
    use utils_misc::ok_or_jserr;
    use crate::shared_keys::SharedKeys;

    #[wasm_bindgen]
    impl SharedKeys {
        /// Get the `alpha` value of the derived shared secrets.
        pub fn get_alpha(&self) -> Uint8Array {
            self.alpha.as_slice().into()
        }

        /// Gets the shared secret of the peer on the given index.
        /// The indices are assigned in the same order as they were given to the
        /// [`generate`] function.
        pub fn get_peer_shared_key(&self, peer_idx: usize) -> Option<Uint8Array> {
            if peer_idx < self.secrets.len() {
                Some(self.secrets[peer_idx].as_slice().into())
            }
            else {
                None
            }
        }

        /// Returns the number of shared keys generated in this structure.
        pub fn count_shared_keys(&self) -> usize {
            self.secrets.len()
        }

        #[wasm_bindgen(js_name = "forward_transform")]
        pub fn _forward_transform(alpha: &[u8], public_key: &[u8], private_key: &[u8]) -> JsResult<SharedKeys> {
            ok_or_jserr!(super::SharedKeys::forward_transform(alpha, public_key, private_key))
        }

        /// Generate shared keys given the peer public keys
        #[wasm_bindgen(js_name = "generate")]
        pub fn _generate(peer_public_keys: Vec<Uint8Array>) -> JsResult<SharedKeys> {
            ok_or_jserr!(super::SharedKeys::generate(&mut OsRng, peer_public_keys.iter().map(|v| v.to_vec().into_boxed_slice()).collect()))
        }
    }
}