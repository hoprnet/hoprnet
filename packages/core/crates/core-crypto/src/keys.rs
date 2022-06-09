
use std::ops::Mul;
use blake2::Blake2s256;

use elliptic_curve::{ProjectivePoint, PublicKey};
use elliptic_curve::rand_core::OsRng;
use elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::CtOption;

use generic_array::GenericArray;
use wasm_bindgen::prelude::*;

use k256::{AffinePoint, EncodedPoint, NonZeroScalar, Secp256k1};

use hkdf::SimpleHkdf;
use js_sys::Uint8Array;

use crate::constants;

/// Type for the secret keys with fixed size
/// The GenericArray<..> is mostly deprecated since Rust 1.51 and it's introduction of const generics,
/// but we need to use it because elliptic_curves and all RustCrypto crates mostly expose it in their
/// public interfaces.
pub type KeyBytes = GenericArray<u8, typenum::U32>;

/// Structure containing shared keys for peers.
#[wasm_bindgen]
pub struct SharedKeys {
    alpha: Box<[u8]>,
    secrets: Box<[Uint8Array]>
}

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

    let mut out = [0u8; constants::SECRET_KEY_LENGTH];
    SimpleHkdf::<Blake2s256>::new(Some(salt), compressed_element.as_bytes())
        .expand(b"", &mut out)
        .unwrap(); // Cannot panic, unless the constants are wrong

    KeyBytes::from(out)
}

/// Decodes the public key and converts it into an EC point in projective coordinates
fn decode_public_key_to_point(encoded_public_key: &[u8]) -> Result<ProjectivePoint<Secp256k1>, String> {
    EncodedPoint::from_bytes(encoded_public_key)
        .map(|p| PublicKey::from_encoded_point(&p))
        .map(|o: CtOption<PublicKey<Secp256k1>>| Option::<PublicKey<Secp256k1>>::from(o)) // We don't care about constant-time comparison here
        .map(|decoded| ProjectivePoint::<Secp256k1>::from(decoded.unwrap()))
        .map_err(|err| err.to_string())
}

/// Generates shared keys for all the given public keys of the peers.
#[wasm_bindgen]
pub fn generate_shared_keys(peer_pubkeys: Vec<Uint8Array>) -> SharedKeys {

    let mut shared_keys = Vec::new();

    // This becomes: x * b_0 * b_1 * b_2 * ...
    let mut coeff_prev = NonZeroScalar::random(&mut OsRng);

    // This becomes: x * b_0 * b_1 * b_2 * ... * G
    // We remain in projective coordinates to save some cycles
    let mut alpha_prev = k256::ProjectivePoint::GENERATOR * coeff_prev.as_ref();

    // Iterate through all the given peer public keys
    for (i, pk) in peer_pubkeys.iter().map(|ppk| ppk.to_vec()).enumerate() {
        // Try to decode the given point
        if let Ok(decoded_proj_point) = decode_public_key_to_point(pk.as_slice()) {

            // Multiply the decoded public key point using the current coefficient
            let shared_secret = (decoded_proj_point * coeff_prev.as_ref()).to_affine();

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            let shared_pk = extract_key_from_group_element(&shared_secret, pk.as_slice());
            shared_keys.push(Uint8Array::from(shared_pk.as_ref()));

            // Stop here, we don't need to compute anything more
            if i == peer_pubkeys.len() - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let enc_alpha_prev = alpha_prev.to_encoded_point(true);
            let b_k = expand_key_from_group_element(&shared_secret, enc_alpha_prev.as_bytes());

            let b_k_checked: NonZeroScalar = Option::from(NonZeroScalar::from_repr(b_k))
                .expect("Key derivation resulted in an EC point in infinity!"); // Extremely unlikely...

            // Update coeff prev
            coeff_prev = coeff_prev.mul(b_k_checked);
            alpha_prev = alpha_prev * b_k_checked.as_ref();
        }
    }

    SharedKeys {
        alpha: alpha_prev.to_encoded_point(true).to_bytes(),
        secrets: shared_keys.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {

    use wasm_bindgen_test::*;
    use super::*;

    #[wasm_bindgen_test]
    fn test_extract_key_from_group_element() {

    }

    #[wasm_bindgen_test]
    fn test_full_kdf() {

    }

    #[wasm_bindgen_test]
    fn test_generate_shared_keys() {

    }
}