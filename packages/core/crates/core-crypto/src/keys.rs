use std::ops::Mul;
use blake2::Blake2s256;

use elliptic_curve::{ProjectivePoint, PublicKey};
use elliptic_curve::rand_core::OsRng;
use elliptic_curve::sec1::ToEncodedPoint;

use generic_array::GenericArray;
use wasm_bindgen::prelude::*;

use k256::{AffinePoint, NonZeroScalar, Secp256k1};

use hkdf::SimpleHkdf;
use js_sys::Uint8Array;

use crate::parameters;
use crate::utils::as_jsvalue;

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
fn decode_public_key_to_point(encoded_public_key: &[u8]) -> Result<ProjectivePoint<Secp256k1>, JsValue> {
    PublicKey::<Secp256k1>::from_sec1_bytes(encoded_public_key)
        .map(|decoded| ProjectivePoint::<Secp256k1>::from(decoded))
        .map_err(as_jsvalue)
}

/// Checks if the given key bytes can form a scalar for EC point
fn to_checked_secret_scalar(secret_scalar: KeyBytes) -> Result<NonZeroScalar, JsValue> {
    let scalar = NonZeroScalar::from_repr(secret_scalar);
    match Option::from(scalar) {
        Some(s) => Ok(s),
        None => Err(JsValue::from("Invalid secret scalar resulting in EC point in infinity"))
    }
}

/// Structure containing shared keys for peers.
/// The members are exposed only using specialized methods.
#[wasm_bindgen]
pub struct SharedKeys {
    alpha: Vec<u8>,
    secrets: Vec<Vec<u8>>
}

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

    /// Generates shared secrets given the peer public keys array.
    /// The order of the peer public keys is preserved for resulting shared keys.
    pub fn generate(peer_pubkeys: Vec<Uint8Array>) -> Result<SharedKeys, JsValue> {

        let mut shared_keys = Vec::new();

        // This becomes: x * b_0 * b_1 * b_2 * ...
        let mut coeff_prev = NonZeroScalar::random(&mut OsRng);

        // This becomes: x * b_0 * b_1 * b_2 * ... * G
        // We remain in projective coordinates to save some cycles
        let mut alpha_prev = k256::ProjectivePoint::GENERATOR * coeff_prev.as_ref();

        // Iterate through all the given peer public keys
        for (i, pk) in peer_pubkeys.iter().map(|ppk| ppk.to_vec()).enumerate() {
            // Try to decode the given point
            let decoded_proj_point = decode_public_key_to_point(pk.as_slice())?;

            // Multiply the decoded public key point using the current coefficient
            let shared_secret = (decoded_proj_point * coeff_prev.as_ref()).to_affine();

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            let shared_pk = extract_key_from_group_element(&shared_secret, pk.as_slice());
            shared_keys.push(shared_pk.to_vec());

            // Stop here, we don't need to compute anything more
            if i == peer_pubkeys.len() - 1 {
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

        // Compress alpha
        let alpha_comp = alpha_prev.to_encoded_point(true);
        Ok(SharedKeys {
            alpha: alpha_comp.as_bytes().into(),
            secrets: shared_keys
        })
    }

    pub fn forward_transform(alpha: &[u8], public_key: &[u8], private_key: &[u8]) -> Result<SharedKeys, JsValue> {

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

mod dummy_rng {

    use elliptic_curve::rand_core::{CryptoRng, Error, RngCore};
    use hex_literal::hex;

    const RNG_BYTES: [u8; 512] = hex!("cc6cb43c4928eea3c31e0d3bfcf8563f85d4bcc771e8efc4792fe3422a09f08a
    36dd22e648fce34edcd20439d9075075073f6da33d344430a45e7e2dfd297890
    7975caa9619afec8b43b3da891ec2369710a61d9630fbdfcd5509da466139c5f7
    a3c91f01fd6fac3665ad229def29873a2b0498bfefadbcb95f946bbea2a3f7657
    701dceffd55c5ce032ae663e1298e041c6b4350ef0e4ec921bedb0c5982709dc2
    b5939053317e14a63ed2f4dccea56145b256667fee63fdc037a7540bd6e16c238
    ff5d9ccfd9acfc9f91755b123b81106a1e3ec6bcc569063cdc78bda0e780aea6d
    06b20c784a295e700f429ee37508a62d98ab3634cfd6ba1c60e40d5c822d8cade
    c591ada0091ea9eae7422980e3defc89ca13ca2bc0de0c8397c5f9abb7b51e373
    7764cab0cfb2faf11e898de2fcc0e1df8fd96a2b1208111420e3aab3953329247
    aeb5416751b120466f41d8e5c094a4cdf6afee1143f42dba102529a0ebac44ced
    199341cc319b533429858c4ac159f7057aad9c2c9211b82c8c227439ec16a4883
    f50c24ee05a3f938e617fb40b7e56dff9a0536b9b7a3b70c607e76086ee61bd05
    d626878acfd5d7ca093d75fd152a00de1ebcd06788b9f6bfa2b289799b75b31c5
    bb8cfb2c005c7de64fb7c8f08613fafe824f1cbebd869aae560299d771f2b896b
    26fcf9a70b0ea3066531ac1a9190b52eb12cc10997aca62d7ce");

    /// Dummy RNG that cyclically outputs same set of random bytes
    #[derive(Clone, Copy, Debug, Default)]
    pub struct DummyFixedRng {
        ptr: usize
    }

    impl DummyFixedRng {
        pub fn new() -> Self {
            DummyFixedRng { ptr: 0 }
        }

        fn read_raw_byte(&mut self) -> u8 {
            if self.ptr >=  RNG_BYTES.len() {
                self.ptr = 0
            }
            else if self.ptr != 0 {
                self.ptr += 1;
            }

            RNG_BYTES[self.ptr]
        }

        fn read_bytes(&mut self, data: &mut [u8]) {
            for i in 0..data.len() {
                data[i] = self.read_raw_byte()
            }
        }
    }

    impl RngCore for DummyFixedRng {

        fn next_u32(&mut self) -> u32 {
            let mut data = [0u8; 4];
            self.read_bytes(&mut data);
            u32::from_ne_bytes(data)
        }

        fn next_u64(&mut self) -> u64 {
            let mut data = [0u8; 8];
            self.read_bytes(&mut data);
            u64::from_ne_bytes(data)
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.read_bytes(dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    impl CryptoRng for DummyFixedRng { }
}

#[cfg(test)]
mod tests {

    use wasm_bindgen_test::*;
    use super::*;
    use super::dummy_rng;


    #[wasm_bindgen_test]
    fn test_extract_key_from_group_element() {

    }

    #[wasm_bindgen_test]
    fn test_expand_key_from_group_element() {

    }

    #[wasm_bindgen_test]
    fn test_shared_keys() {

        const COUNT_KEYPAIRS: usize = 3;

        let (priv_keys, pub_keys): (Vec<Uint8Array>, Vec<Uint8Array>) = (0..COUNT_KEYPAIRS)
            .map(|_i| NonZeroScalar::random(&mut dummy_rng::DummyFixedRng::new()))
            .map(|s| (s, k256::ProjectivePoint::GENERATOR * s.as_ref()))
            .map(|p| (p.0, p.1.to_encoded_point(true)))
            .map(|p| (p.0.to_bytes(), p.1))
            .map(|p| (p.0.as_slice().into(), p.1.as_bytes().into()))
            .unzip();

        let generated_shares = SharedKeys::generate(pub_keys.clone()).unwrap();

        let mut alpha_cpy = generated_shares.alpha.clone();

        for i in 0..COUNT_KEYPAIRS {
            let priv_key = priv_keys[i].to_vec();
            let pub_key = pub_keys[i].to_vec();

            let shared_key = SharedKeys::forward_transform(alpha_cpy.as_slice(),
                                          pub_key.as_slice(),
                                          priv_key.as_slice()).unwrap();

            let a = shared_key.get_peer_shared_key(0).unwrap().to_vec();
            let b = generated_shares.get_peer_shared_key(i).unwrap().to_vec();

            assert_eq!(a, b);

            alpha_cpy = shared_key.alpha.clone();
        }

    }
}