use blake2::Blake2s256;
use std::ops::Mul;

use elliptic_curve::rand_core::{CryptoRng, RngCore};
use elliptic_curve::sec1::ToEncodedPoint;
use elliptic_curve::Group;

use k256::NonZeroScalar;

use crate::errors::CryptoError::{CalculationError, InvalidSecretScalar};
use hkdf::SimpleHkdf;
use libp2p_identity::PeerId;
use rand::rngs::OsRng;
use utils_types::traits::PeerIdLike;

use crate::parameters;

use crate::errors::Result;
use crate::types::{CurvePoint, PublicKey};

/// Type for the secret keys with fixed size
/// The GenericArray<..> is mostly deprecated since Rust 1.51 and it's introduction of const generics,
/// but we need to use it because elliptic_curves and all RustCrypto crates mostly expose it in their
/// public interfaces.
//pub type KeyBytes = GenericArray<u8, typenum::U32>;

/// Extract a keying material from an EC point using HKDF extract
fn extract_key_from_group_element(group_element: &CurvePoint, salt: &[u8]) -> Box<[u8]> {
    // Create the compressed EC point representation first
    let compressed_element = group_element.serialize_compressed();
    let ret = SimpleHkdf::<Blake2s256>::extract(Some(salt), &compressed_element).0;
    ret.as_slice().into()
}

/// Performs KDF expansion from the given EC point using HKDF expand
fn expand_key_from_group_element(group_element: &CurvePoint, salt: &[u8]) -> Box<[u8]> {
    // Create the compressed EC point representation first
    let compressed_element = group_element.serialize_compressed();

    let mut out = [0u8; parameters::SECRET_KEY_LENGTH];
    SimpleHkdf::<Blake2s256>::new(Some(salt), &compressed_element)
        .expand(b"", &mut out)
        .unwrap(); // Cannot panic, unless the constants are wrong

    out.into()
}

/// Checks if the given key bytes can form a scalar for EC point
fn to_checked_secret_scalar(secret_scalar: &[u8]) -> Result<NonZeroScalar> {
    NonZeroScalar::try_from(secret_scalar).map_err(|_| InvalidSecretScalar)
}

/// Structure containing shared keys for peers.
/// The members are exposed only using specialized methods.
pub struct SharedKeys {
    alpha: CurvePoint,
    secrets: Vec<Box<[u8]>>,
}

impl SharedKeys {
    /// Generates shared secrets for the given path of peers.
    pub fn new(path: &[&PeerId]) -> Result<Self> {
        Self::generate(
            &mut OsRng,
            &path
                .iter()
                .map(|peer_id| PublicKey::from_peerid(peer_id))
                .collect::<utils_types::errors::Result<Vec<_>>>()?,
        )
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
            let b_k_checked = to_checked_secret_scalar(&b_k)?;

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
    pub fn forward_transform(alpha: CurvePoint, private_key: &[u8]) -> Result<(CurvePoint, Box<[u8]>)> {
        let public_key = PublicKey::from_privkey(private_key)?;
        let private_scalar = to_checked_secret_scalar(private_key)?;
        let alpha_projective = alpha.to_projective_point();

        let s_k = (alpha_projective * private_scalar.as_ref()).to_affine();
        let secret = extract_key_from_group_element(&s_k.into(), &public_key.to_bytes(true));

        let b_k = expand_key_from_group_element(&s_k.into(), &alpha.serialize_compressed());
        let b_k_checked = to_checked_secret_scalar(&b_k)?;

        let alpha_new = (alpha_projective * b_k_checked.as_ref()).to_affine();

        Ok((alpha_new.into(), secret))
    }

    pub fn alpha(&self) -> &CurvePoint {
        &self.alpha
    }

    pub fn secrets(&self) -> Vec<&[u8]> {
        self.secrets.iter().map(Box::as_ref).collect()
    }

    pub fn secret(&self, idx: usize) -> Option<&[u8]> {
        self.secrets.get(idx).map(|x| x.as_ref())
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
        assert_eq!(res, key.as_ref());
    }

    #[test]
    fn test_expand_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = AffinePoint::generator();

        let key = expand_key_from_group_element(&pt.into(), &salt);
        assert_eq!(parameters::SECRET_KEY_LENGTH, key.len());

        let res = hex!("D138D9367474911F7124B95BE844D2F8A6D34E962694E37E8717BDBD3C15690B");
        assert_eq!(res, key.as_ref());
    }

    pub fn generate_random_keypairs(count: usize) -> (Vec<Box<[u8]>>, Vec<PublicKey>) {
        (0..count)
            .map(|_| NonZeroScalar::random(&mut OsRng))
            .map(|s| {
                (
                    s.to_bytes().as_slice().into(),
                    CurvePoint::from_exponent(s.to_bytes().as_slice())
                        .and_then(PublicKey::try_from)
                        .unwrap(),
                )
            })
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

        let mut alpha_cpy = generated_shares.alpha().clone();
        for (i, priv_key) in priv_keys.iter().enumerate() {
            let (alpha, secret) = SharedKeys::forward_transform(alpha_cpy, priv_key).unwrap();

            assert_eq!(secret.as_ref(), generated_shares.secrets()[i]);

            alpha_cpy = alpha;
        }
    }

    #[test]
    fn test_key_shares() {
        let pub_keys = [
            hex!("0253f6e72ad23de294466b830619448d6d9059a42050141cd83bac4e3ee82c3f1e"),
            hex!("035fc5660f59059c263d3946d7abaf33fa88181e27bf298fcc5a9fa493bec9110b"),
            hex!("038d2b50a77fd43eeae9b37856358c7f1aee773b3e3c9d26f30b8706c02cbbfbb6"),
        ]
        .into_iter()
        .map(|p| PublicKey::from_bytes(&p))
        .collect::<utils_types::errors::Result<Vec<_>>>()
        .unwrap();

        let keyshares = SharedKeys::generate(&mut OsRng, &pub_keys).unwrap();
        assert_eq!(3, keyshares.secrets().len());
    }
}
