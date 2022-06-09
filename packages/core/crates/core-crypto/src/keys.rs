use std::borrow::Borrow;
use blake2::Blake2s256;
use elliptic_curve::{ProjectivePoint, PublicKey};
use elliptic_curve::rand_core::OsRng;
use elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use elliptic_curve::subtle::CtOption;
use wasm_bindgen::prelude::*;

use js_sys;
use k256;
use k256::{EncodedPoint, NonZeroScalar, Scalar, Secp256k1, U256};

use hkdf;
use hkdf::SimpleHkdf;
use js_sys::Uint8Array;

use crate::constants;

#[wasm_bindgen]
pub struct SharedKeys {
    alpha: Box<[u8]>,
    secrets: Box<[js_sys::Uint8Array]>
}

fn extract_key_from_group_element(group_element: &PublicKey<Secp256k1>, salt: &[u8]) -> Box<[u8]> {
    // Create the compressed EC point representation first
    let compressed_element = group_element.to_encoded_point(true);

    let out = SimpleHkdf::<Blake2s256>::extract(Some(salt), compressed_element.as_bytes()).0;

    out.to_vec().into_boxed_slice()
}

fn full_kdf(group_element: &PublicKey<Secp256k1>, salt: &[u8]) -> Box<[u8]> {
    // Create the compressed EC point representation first
    let compressed_element = group_element.to_encoded_point(true);

    let mut out = [0u8; constants::SECRET_KEY_LENGTH];
    SimpleHkdf::<Blake2s256>::new(Some(salt), compressed_element.as_bytes())
        .expand(b"", &mut out)
        .unwrap(); // Cannot panic, unless the constants are wrong

    Box::new(out)
}

#[wasm_bindgen]
pub fn generate_shared_keys(peer_pubkeys: Vec<Uint8Array>) -> SharedKeys {

    let mut shared_keys = Vec::new();

    let mut coeff_prev = NonZeroScalar::random(&mut OsRng);
    let mut alpha_prev = PublicKey::<Secp256k1>::from_secret_scalar(&coeff_prev);

     for (i, pk) in peer_pubkeys.iter().map(|ppk| ppk.to_vec()).enumerate() {
         // Try to decode the given point
         if let Ok(decoded_point) = EncodedPoint::from_bytes(pk.as_slice())
             .map(|p| PublicKey::from_encoded_point(&p))
             .map(|o: CtOption<PublicKey<Secp256k1>>| Option::<PublicKey<Secp256k1>>::from(o)) // We don't care about constant-time comparison here
         {
                let decoded_proj_point = ProjectivePoint::<Secp256k1>::from(decoded_point.unwrap());
                let shared_secret = (decoded_proj_point * coeff_prev.borrow().as_ref()).to_affine();

                let affine_pk = PublicKey::from_affine(shared_secret).unwrap();
                let shared_pk= extract_key_from_group_element(&affine_pk, pk.as_slice());
                shared_keys.push(Uint8Array::from(shared_pk.as_ref()));

                // Stop here, we don't need to compute anything more
                if i == peer_pubkeys.len()-1 {
                    break;
                }

                let enc_alpha = alpha_prev.to_encoded_point(true);
                let b_k = full_kdf(&affine_pk, enc_alpha.as_bytes());

                //coeff_prev = coeff_prev.mul(U256::from_le_slice(b_k.as_ref()))
         }
    }


    SharedKeys {
        alpha: Box::new([0u8]),
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