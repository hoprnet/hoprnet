use std::marker::PhantomData;
use blake2::Blake2s256;
use std::ops::Mul;

use elliptic_curve::rand_core::{CryptoRng, RngCore};
use elliptic_curve::Group;

use k256::{NonZeroScalar, ProjectivePoint};

use crate::errors::CryptoError::{CalculationError, InvalidSecretScalar};
use hkdf::SimpleHkdf;
use libp2p_identity::PeerId;
use rand::rngs::OsRng;
use utils_types::traits::PeerIdLike;

use crate::errors::Result;
use crate::parameters::SECRET_KEY_LENGTH;
use crate::types::{CurvePoint, PublicKey, SecretKey};

/// Types representing a valid non-zero scalar an additive abelian group.
pub trait Scalar: Mul<Output = Self> + Sized {
    /// Type used to represent this type externally.
    type Repr: From<SecretKey>;

    /// Converts the scalar from its external representation.
    fn from_repr(repr: Self::Repr) -> Result<Self>;

    /// Converts the scalar to its external representation.
    fn to_repr(&self) -> Self::Repr;

    /// Generates a random scalar using a cryptographically secure RNG.
    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self;
}

/// Trait a representable group element needs to implement in order
/// to be encodable. The encoding should be space-efficient.
pub trait GroupEncoding {
    /// Encodes the group element representation. Encoding should be space efficient.
    fn encode(&self) -> Box<[u8]>;
}

/// Generic additive abelian group element with an associated scalar type.
/// A group element is considered valid if it is not neutral or a torsion element of small order.
pub trait GroupElement<E: Scalar>: Clone {
    /// Type used to represent the valid elements of this group.
    type Repr: GroupEncoding + Clone + PeerIdLike;

    /// Represents a valid group element. Panics if the element is not valid.
    fn to_repr(&self) -> Self::Repr;

    /// Creates a group element from a representation.
    fn from_repr(repr: Self::Repr) -> Result<Self>;

    /// Create a group element using the group generator and the given scalar
    fn generate(scalar: &E) -> Self;

    /// Performs the group law given number of times (i.e multiplication in additive groups)
    fn multiply(&self, scalar: &E) -> Self;

    /// Group element is considered valid if it is not a neutral element and also not a torsion element of small order.
    fn is_valid(&self) -> bool;

    fn random_pair(rng: &mut (impl CryptoRng + RngCore)) -> (Self, E) {
        let scalar = E::random(rng);
        (Self::generate(&scalar), scalar)
    }

    /// Extract a keying material from a group element using HKDF extract
    fn extract_key(&self, salt: &[u8]) -> SecretKey {
        SimpleHkdf::<Blake2s256>::extract(Some(salt), &self.to_repr().encode()).0.into()
    }

    /// Performs KDF expansion from the given group element using HKDF expand
    fn expand_key(&self, salt: &[u8]) -> SecretKey {
        let mut out = [0u8; SECRET_KEY_LENGTH];
        SimpleHkdf::<Blake2s256>::new(Some(salt), &self.to_repr().encode())
            .expand(b"", &mut out)
            .unwrap(); // Cannot panic, unless the constants are wrong

        out
    }
}

/// Structure containing shared keys for peers using the Sphinx algorithm.
pub struct SharedKeys<E: Scalar, G: GroupElement<E>> {
    pub alpha: G::Repr,
    pub secrets: Vec<SecretKey>,
    _type: PhantomData<E>
}

impl <E: Scalar, G: GroupElement<E>> SharedKeys<E, G> {
    /// Generates shared secrets for the given path of peers.
    pub fn new(path: &[&PeerId]) -> Result<Self> {
        Self::generate(
            &mut OsRng,
            path
                .iter()
                .map(|peer_id| G::Repr::from_peerid(peer_id))
                .collect::<utils_types::errors::Result<Vec<_>>>()?,
        )
    }

    /// Generates shared secrets given the group element of the peers.
    /// The order of the peer group elements is preserved for resulting shared keys.
    /// The specified random number generator will be used.
    pub fn generate(rng: &mut (impl CryptoRng + RngCore), peer_group_elements: Vec<G::Repr>) -> Result<SharedKeys<E, G>> {
        let mut shared_keys = Vec::new();

        // coeff_prev becomes: x * b_0 * b_1 * b_2 * ...
        // alpha_prev becomes: x * b_0 * b_1 * b_2 * ... * G
        let (mut alpha_prev, mut coeff_prev) = G::random_pair(rng);

        // Save the part of the result
        let alpha = alpha_prev.to_repr();

        // Iterate through all the given peer public keys
        let keys_len = peer_group_elements.len();
        for (i, ge_repr) in peer_group_elements.into_iter().enumerate() {
            let group_element = G::from_repr(ge_repr)?;

            // Try to decode the given public key point & multiply by the current coefficient
            let shared_secret = group_element.multiply(&coeff_prev);

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            shared_keys.push(shared_secret.extract_key(&group_element.to_repr().encode()));

            // Stop here, we don't need to compute anything more
            if i == keys_len - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let b_k = shared_secret.expand_key(&alpha_prev.to_repr().encode());
            let b_k_checked = E::from_repr(b_k.into())?;

            // Update coeff_prev and alpha
            alpha_prev = alpha_prev.multiply(&b_k_checked);
            coeff_prev = coeff_prev.mul(b_k_checked);

            if !alpha_prev.is_valid() {
                return Err(CalculationError);
            }
        }

        Ok(SharedKeys {
            alpha,
            secrets: shared_keys,
            _type: PhantomData,
        })
    }

    /// Calculates the forward transformation for the given the local private key.
    pub fn forward_transform(alpha: G::Repr, private_key: E::Repr) -> Result<(G::Repr, SecretKey)> {
        let private_scalar = E::from_repr(private_key)?;
        let public_key = G::generate(&private_scalar);
        let alpha_point = G::from_repr(alpha)?;

        let s_k = alpha_point.multiply(&private_scalar);
        let secret = s_k.extract_key(&public_key.to_repr().encode());

        let b_k = s_k.expand_key(&alpha_point.to_repr().encode());
        let b_k_checked = E::from_repr(b_k.into())?;

        let alpha_new = alpha_point.multiply(&b_k_checked);

        Ok((alpha_new.to_repr(), secret))
    }

}

/// Instantiation of Sphinx shared keys generation using Secp256k1 group
pub type Secp256k1SharedKeys = SharedKeys<NonZeroScalar, ProjectivePoint>;

/// Non-zero scalars in the underlying field for Secp256k1
impl Scalar for NonZeroScalar {
    type Repr = [u8; 32];

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        NonZeroScalar::try_from(repr.as_slice()).map_err(|_| InvalidSecretScalar)
    }

    fn to_repr(&self) -> Self::Repr {
        let mut ret = [0u8; 32];
        ret.copy_from_slice(self.to_bytes().as_slice());
        ret
    }

    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self {
        NonZeroScalar::random(rng)
    }
}

impl GroupEncoding for PublicKey {
    fn encode(&self) -> Box<[u8]> {
        self.to_bytes(true)
    }
}

/// Secp256k1 additive group (via projective coordinates) represented as public keys
impl GroupElement<NonZeroScalar> for ProjectivePoint {
    type Repr = PublicKey;

    fn to_repr(&self) -> Self::Repr {
        PublicKey::try_from(CurvePoint::from_affine(self.to_affine()))
            .expect("group element does not represent a valid public key")
    }

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        Ok(CurvePoint::from(repr).to_projective_point())
    }

    fn generate(scalar: &NonZeroScalar) -> Self {
        ProjectivePoint::GENERATOR * scalar.as_ref()
    }

    fn multiply(&self, scalar: &NonZeroScalar) -> Self {
        Mul::mul(self, scalar.as_ref())
    }

    fn is_valid(&self) -> bool {
        self.is_identity().unwrap_u8() == 0
    }
}

/// Unit tests of the Rust code
#[cfg(test)]
pub mod tests {
    use super::*;
    use elliptic_curve::rand_core::OsRng;
    use hex_literal::hex;

    #[test]
    fn test_extract_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = ProjectivePoint::GENERATOR;

        let key = pt.extract_key(&salt);
        assert_eq!(SECRET_KEY_LENGTH, key.len());

        let res = hex!("54bf34178075e153f481ce05b113c1530ecc45a2f1f13a3366d4389f65470de6");
        assert_eq!(res, key.as_ref());
    }

    #[test]
    fn test_expand_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = ProjectivePoint::GENERATOR;

        let key = pt.expand_key(&salt);
        assert_eq!(SECRET_KEY_LENGTH, key.len());

        let res = hex!("d138d9367474911f7124b95be844d2f8a6d34e962694e37e8717bdbd3c15690b");
        assert_eq!(res, key.as_ref());
    }

    pub fn generate_random_keypairs<E: Scalar, G: GroupElement<E>>(count: usize) -> (Vec<G::Repr>, Vec<E::Repr>) {
        (0..count)
            .map(|_| G::random_pair(&mut OsRng))
            .map(|(a, b)| (a.to_repr(), b.to_repr()))
            .unzip()
    }

    pub fn generic_test_shared_keys<E: Scalar, G: GroupElement<E>>() {
        const COUNT_KEYPAIRS: usize = 3;
        let (pub_keys, priv_keys) = generate_random_keypairs::<E, G>(COUNT_KEYPAIRS);

        // Now generate the key shares for the public keys
        let generated_shares = SharedKeys::<E, G>::generate(&mut OsRng, pub_keys).unwrap();
        assert_eq!(COUNT_KEYPAIRS, generated_shares.secrets.len());

        let mut alpha_cpy = generated_shares.alpha.clone();
        for (i, priv_key) in priv_keys.into_iter().enumerate() {
            let (alpha, secret) = SharedKeys::<E, G>::forward_transform(alpha_cpy, priv_key).unwrap();

            assert_eq!(secret.as_ref(), generated_shares.secrets[i]);

            alpha_cpy = alpha;
        }
    }

    #[test]
    fn test_secp256k1_shared_keys() {
        generic_test_shared_keys::<NonZeroScalar, ProjectivePoint>()
    }

}


