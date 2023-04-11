use blake2::Blake2s256;
use std::ops::Mul;

use elliptic_curve::rand_core::{CryptoRng, RngCore};
use elliptic_curve::sec1::ToEncodedPoint;
use elliptic_curve::{Group};

use generic_array::GenericArray;

use k256::{NonZeroScalar};

use crate::errors::CryptoError::{CalculationError, InvalidSecretScalar};
use hkdf::SimpleHkdf;
use libp2p_identity::PeerId;
use rand::rngs::OsRng;
use utils_types::traits::{BinarySerializable, PeerIdLike};

use crate::parameters;

use crate::errors::Result;
use crate::types::{CurvePoint, PublicKey};

/// Type for the secret keys with fixed size
/// The GenericArray<..> is mostly deprecated since Rust 1.51 and it's introduction of const generics,
/// but we need to use it because elliptic_curves and all RustCrypto crates mostly expose it in their
/// public interfaces.
pub type KeyBytes = GenericArray<u8, typenum::U32>;

/// Extract a keying material from an EC point using HKDF extract
fn extract_key_from_group_element(group_element: &CurvePoint, salt: &[u8]) -> KeyBytes {
    // Create the compressed EC point representation first
    let compressed_element = group_element.serialize_compressed();
    SimpleHkdf::<Blake2s256>::extract(Some(salt), &compressed_element).0
}

/// Performs KDF expansion from the given EC point using HKDF expand
fn expand_key_from_group_element(group_element: &CurvePoint, salt: &[u8]) -> KeyBytes {
    // Create the compressed EC point representation first
    let compressed_element = group_element.serialize_compressed();

    let mut out = [0u8; parameters::SECRET_KEY_LENGTH];
    SimpleHkdf::<Blake2s256>::new(Some(salt), &compressed_element)
        .expand(b"", &mut out)
        .unwrap(); // Cannot panic, unless the constants are wrong

    out.into()
}

/// Checks if the given key bytes can form a scalar for EC point
fn to_checked_secret_scalar(secret_scalar: KeyBytes) -> Result<NonZeroScalar> {
    Option::from(NonZeroScalar::from_repr(secret_scalar)).ok_or(InvalidSecretScalar)
}

/// Structure containing shared keys for peers.
/// The members are exposed only using specialized methods.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct SharedKeys {
    alpha: CurvePoint,
    secrets: Vec<Box<[u8]>>,
}

impl SharedKeys {
    /// Generates shared secrets for the given path of peers.
    pub fn new(path: &[&PeerId]) -> Result<Self> {
        Self::generate(&mut OsRng, &path
            .iter()
            .map(|peer_id| PublicKey::from_peerid(peer_id))
            .collect::<utils_types::errors::Result<Vec<_>>>()?)
    }

    /// Generates shared secrets given the peer public keys array.
    /// The order of the peer public keys is preserved for resulting shared keys.
    /// The specified random number generator will be used.
    pub fn generate(rng: &mut (impl CryptoRng + RngCore), peer_public_keys: &[PublicKey]) -> Result<SharedKeys> {
        let mut shared_keys = Vec::new();

        // This becomes: x * b_0 * b_1 * b_2 * ...
        let mut coeff_prev = NonZeroScalar::random(rng);

        // This becomes: x * b_0 * b_1 * b_2 * ... * G
        // We remain in projective coordinates to save some cycles
        let mut alpha_prev = k256::ProjectivePoint::GENERATOR * coeff_prev.as_ref();
        let alpha = alpha_prev;

        // Iterate through all the given peer public keys
        for (i, cp) in peer_public_keys.iter().map(CurvePoint::from).enumerate() {
            // Try to decode the given public key point & multiply by the current coefficient
            let shared_secret = (cp.to_projective_point() * coeff_prev.as_ref()).to_affine();

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            let shared_pk = extract_key_from_group_element(&shared_secret.into(), &cp.serialize_compressed());
            shared_keys.push(shared_pk.to_vec().into_boxed_slice());

            // Stop here, we don't need to compute anything more
            if i == peer_public_keys.len() - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let enc_alpha_prev = alpha_prev.to_encoded_point(true);
            let b_k = expand_key_from_group_element(&shared_secret.into(), enc_alpha_prev.as_bytes());
            let b_k_checked = to_checked_secret_scalar(b_k)?;

            // Update coeff prev and alpha
            coeff_prev = coeff_prev.mul(b_k_checked);
            alpha_prev = alpha_prev * b_k_checked.as_ref();

            if alpha_prev.is_identity().unwrap_u8() != 0 {
                return Err(CalculationError);
            }
        }

        Ok(SharedKeys {
            alpha: alpha.to_affine().into(),
            secrets: shared_keys,
        })
    }

    /// Calculates the forward transformation for the given peer public key.
    pub fn forward_transform(alpha: &[u8], private_key: &[u8]) -> Result<SharedKeys> {
        let public_key = PublicKey::from_privkey(private_key)?;
        let private_scalar = to_checked_secret_scalar(KeyBytes::clone_from_slice(&private_key[..]))?;
        let alpha_projective = CurvePoint::deserialize(alpha)?.to_projective_point();

        let s_k = (alpha_projective * private_scalar.as_ref()).to_affine();
        let secret = extract_key_from_group_element(&s_k.into(), &public_key.serialize(true));

        let b_k = expand_key_from_group_element(&s_k.into(), alpha);
        let b_k_checked = to_checked_secret_scalar(b_k)?;

        let alpha_new = (alpha_projective * b_k_checked.as_ref()).to_affine();

        Ok(SharedKeys {
            alpha: alpha_new.into(),
            secrets: vec![secret.to_vec().into_boxed_slice()],
        })
    }

    pub fn alpha(&self) -> Box<[u8]> {
        self.alpha.serialize_compressed()
    }

    pub fn secrets(&self) -> Vec<&[u8]> {
        self.secrets.iter().map(Box::as_ref).collect()
    }

    pub fn secret(&self, idx: usize) -> &[u8] {
        &self.secrets[idx]
    }
}

/// Unit tests of the Rust code
#[cfg(test)]
pub mod tests {
    use super::*;
    use elliptic_curve::group::prime::PrimeCurveAffine;
    use elliptic_curve::rand_core::OsRng;
    use hex_literal::hex;
    use k256::AffinePoint;

    #[test]
    fn test_extract_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = AffinePoint::generator();

        let key = extract_key_from_group_element(&pt.into(), &salt);
        assert_eq!(parameters::SECRET_KEY_LENGTH, key.len());

        let res = hex!("54BF34178075E153F481CE05B113C1530ECC45A2F1F13A3366D4389F65470DE6");
        assert_eq!(res, key.as_slice());
    }

    #[test]
    fn test_expand_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = AffinePoint::generator();

        let key = expand_key_from_group_element(&pt.into(), &salt);
        assert_eq!(parameters::SECRET_KEY_LENGTH, key.len());

        let res = hex!("D138D9367474911F7124B95BE844D2F8A6D34E962694E37E8717BDBD3C15690B");
        assert_eq!(res, key.as_slice());
    }

    pub fn generate_random_keypairs(count: usize) -> (Vec<Box<[u8]>>, Vec<PublicKey>) {
        (0..count)
            .map(|_| NonZeroScalar::random(&mut OsRng))
            .map(|s|(s.to_bytes().as_slice().into(), CurvePoint::from_exponent(s.to_bytes().as_slice()).and_then(PublicKey::try_from).unwrap()))
            .unzip()
    }

    #[test]
    fn test_shared_keys() {
        // DummyRng is useful for deterministic debugging
        //let mut used_rng = crate::dummy_rng::DummyFixedRng::new();

        const COUNT_KEYPAIRS: usize = 3;
        let mut used_rng = OsRng;

        // Generate some random key pairs
        let (priv_keys, pub_keys) = generate_random_keypairs(COUNT_KEYPAIRS);

        // Now generate the key shares for the public keys
        let generated_shares = SharedKeys::generate(&mut used_rng, &pub_keys).unwrap();

        let mut alpha_cpy: Box<[u8]> = generated_shares.alpha();
        for (i, priv_key) in priv_keys.iter().enumerate() {

            let shared_key =
                SharedKeys::forward_transform(&alpha_cpy, priv_key).unwrap();

            assert_eq!(&shared_key.secrets()[0], &generated_shares.secrets()[i]);

            alpha_cpy = shared_key.alpha();
        }
    }

    #[test]
    fn test_key_shares() {
        let pub_keys =
        [
            hex!("0253f6e72ad23de294466b830619448d6d9059a42050141cd83bac4e3ee82c3f1e"),
            hex!("035fc5660f59059c263d3946d7abaf33fa88181e27bf298fcc5a9fa493bec9110b"),
            hex!("038d2b50a77fd43eeae9b37856358c7f1aee773b3e3c9d26f30b8706c02cbbfbb6"),
        ]
        .into_iter()
        .map(|p|PublicKey::deserialize(&p))
        .collect::<utils_types::errors::Result<Vec<_>>>()
        .unwrap();

        let keyshares = SharedKeys::generate(&mut OsRng, &pub_keys).unwrap();
        assert_eq!(3, keyshares.secrets().len());
    }
}

/// This module contains wrapper for the Rust code
/// to be properly called from the JS.
/// Code in this module does not need to be unit tested, as it already
/// wraps code that has been unit tested in pure Rust.
#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::shared_keys::SharedKeys;
    use elliptic_curve::rand_core::OsRng;
    use js_sys::Uint8Array;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;
    use crate::types::PublicKey;

    #[wasm_bindgen]
    impl SharedKeys {
        /// Get the `alpha` value of the derived shared secrets.
        pub fn get_alpha(&self) -> Uint8Array {
            self.alpha().as_ref().into()
        }

        /// Gets the shared secret of the peer on the given index.
        /// The indices are assigned in the same order as they were given to the
        /// [`generate`] function.
        pub fn get_peer_shared_key(&self, peer_idx: usize) -> Option<Uint8Array> {
            self.secrets.get(peer_idx).map(|k| Uint8Array::from(k.as_ref()))
        }

        /// Returns the number of shared keys generated in this structure.
        pub fn count_shared_keys(&self) -> usize {
            self.secrets.len()
        }

        #[wasm_bindgen(js_name = "forward_transform")]
        pub fn _forward_transform(alpha: &[u8], private_key: &[u8]) -> JsResult<SharedKeys> {
            ok_or_jserr!(super::SharedKeys::forward_transform(alpha, private_key))
        }

        /// Generate shared keys given the peer public keys
        #[wasm_bindgen(js_name = "generate")]
        pub fn _generate(peer_public_keys: Vec<Uint8Array>) -> JsResult<SharedKeys> {
            let public_keys = ok_or_jserr!(peer_public_keys.into_iter()
                    .map(|v| PublicKey::deserialize(&v.to_vec()))
                    .collect::<utils_types::errors::Result<Vec<PublicKey>>>())?;
            ok_or_jserr!(super::SharedKeys::generate(&mut OsRng, &public_keys))
        }
    }
}
