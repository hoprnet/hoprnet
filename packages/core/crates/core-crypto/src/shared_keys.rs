use std::marker::PhantomData;
use blake2::Blake2s256;
use std::ops::Mul;

use crate::errors::CryptoError::CalculationError;
use hkdf::SimpleHkdf;
use libp2p_identity::PeerId;
use rand::{CryptoRng, RngCore};
use rand::rngs::OsRng;

use crate::errors::Result;
use crate::parameters::SECRET_KEY_LENGTH;
use crate::types::SecretKey;

pub type Alpha<const Size: usize> = [u8; Size];

pub type SharedSecret = [u8; SECRET_KEY_LENGTH];

/// Types representing a valid non-zero scalar an additive abelian group.
pub trait Scalar: Mul<Output = Self> + Sized {
    /// Generates a random scalar using a cryptographically secure RNG.
    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self;

    /// Create scalar from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>;

    /// Represent scalar as bytes
    fn as_bytes(&self) -> &[u8];
}

/// Generic additive abelian group element with an associated scalar type.
/// It also comes with the associated Alpha value size.
/// A group element is considered valid if it is not neutral or a torsion element of small order.
pub trait GroupElement<const A: usize, E: Scalar>: Clone + for <'a> Mul<&'a E, Output = Self> {
    /// Converts the group element to a binary format suitable for representing the Alpha value.
    fn to_alpha(&self) -> Alpha<A>;

    /// Converts the group element from the binary format representin an Alpha value.
    fn from_alpha(alpha: Alpha<A>) -> Result<Self>;

    /// Converts peer id to the group element
    fn from_peerid(peer_id: &PeerId) -> Result<Self>;

    /// Create a group element using the group generator and the given scalar
    fn generate(scalar: &E) -> Self;

    /// Group element is considered valid if it is not a neutral element and also not a torsion element of small order.
    fn is_valid(&self) -> bool;

    fn random_pair(rng: &mut (impl CryptoRng + RngCore)) -> (Self, E) {
        let scalar = E::random(rng);
        (Self::generate(&scalar), scalar)
    }

    /// Extract a keying material from a group element using HKDF extract
    fn extract_key(&self, salt: &[u8]) -> SharedSecret {
        SimpleHkdf::<Blake2s256>::extract(Some(salt), &self.to_alpha()).0.into()
    }

    /// Performs KDF expansion from the given group element using HKDF expand
    fn expand_key(&self, salt: &[u8]) -> SharedSecret {
        let mut out = [0u8; SECRET_KEY_LENGTH];
        SimpleHkdf::<Blake2s256>::new(Some(salt), &self.to_alpha())
            .expand(b"", &mut out)
            .unwrap(); // Cannot panic, unless the constants are wrong

        out
    }
}

/// Structure containing shared keys for peers using the Sphinx algorithm.
pub struct SharedKeys<E: Scalar, const A: usize, G: GroupElement<A, E>> {
    pub alpha: Alpha<A>,
    pub secrets: Vec<SharedSecret>,
    _e: PhantomData<E>,
    _g: PhantomData<G>,
}

impl <E: Scalar, const A: usize, G: GroupElement<A, E>> SharedKeys<E, A, G> {
    /// Generates shared secrets for the given path of peers.
    pub fn new(path: &[&PeerId]) -> Result<Self> {
        Self::generate(
            &mut OsRng,
            path
                .iter()
                .map(|peer_id| G::from_peerid(peer_id))
                .collect::<Result<Vec<_>>>()?,
        )
    }

    /// Generates shared secrets given the group element of the peers.
    /// The order of the peer group elements is preserved for resulting shared keys.
    /// The specified random number generator will be used.
    pub fn generate(rng: &mut (impl CryptoRng + RngCore), peer_group_elements: Vec<G>) -> Result<SharedKeys<E, A, G>> {
        let mut shared_keys = Vec::new();

        // coeff_prev becomes: x * b_0 * b_1 * b_2 * ...
        // alpha_prev becomes: x * b_0 * b_1 * b_2 * ... * G
        let (mut alpha_prev, mut coeff_prev) = G::random_pair(rng);

        // Save the part of the result
        let alpha = alpha_prev.to_alpha();

        // Iterate through all the given peer public keys
        let keys_len = peer_group_elements.len();
        for (i, group_element) in peer_group_elements.into_iter().enumerate() {

            // Try to decode the given public key point & multiply by the current coefficient
            let shared_secret = group_element.mul(&coeff_prev);

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            shared_keys.push(shared_secret.extract_key(&group_element.to_alpha()));

            // Stop here, we don't need to compute anything more
            if i == keys_len - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let b_k = shared_secret.expand_key(&alpha_prev.to_alpha());
            let b_k_checked = E::from_bytes(&b_k)?;

            // Update coeff_prev and alpha
            alpha_prev = alpha_prev.mul(&b_k_checked);
            coeff_prev = coeff_prev.mul(b_k_checked);

            if !alpha_prev.is_valid() {
                return Err(CalculationError);
            }
        }

        Ok(SharedKeys {
            alpha,
            secrets: shared_keys,
            _e: PhantomData,
            _g: PhantomData
        })
    }

    /// Calculates the forward transformation for the given the local private key.
    pub fn forward_transform(alpha: Alpha<A>, private_key: &[u8], public_key: &G) -> Result<(Alpha<A>, SecretKey)> {
        let private_scalar = E::from_bytes(private_key)?;
        let alpha_point = G::from_alpha(alpha)?;

        let s_k = alpha_point.mul(&private_scalar);

        let secret = s_k.extract_key(&public_key.to_alpha());

        let b_k = s_k.expand_key(&alpha_point.to_alpha());

        let b_k_checked = E::from_bytes(&b_k)?;
        let alpha_new = alpha_point.mul(&b_k_checked);

        Ok((alpha_new.to_alpha(), secret))
    }

}

/// Unit tests of the Rust code
#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::rngs::OsRng;

    pub fn generate_random_keypairs<E: Scalar, const A: usize, G: GroupElement<A, E>>(count: usize) -> (Vec<G>, Vec<E>) {
        (0..count)
            .map(|_| G::random_pair(&mut OsRng))
            .unzip()
    }

    pub fn generic_test_shared_keys<E: Scalar, const A: usize, G: GroupElement<A, E>>() {
        const COUNT_KEYPAIRS: usize = 3;
        let (pub_keys, priv_keys) = generate_random_keypairs::<E, A, G>(COUNT_KEYPAIRS);

        // Now generate the key shares for the public keys
        let generated_shares = SharedKeys::<E, A, G>::generate(&mut OsRng, pub_keys.clone()).unwrap();
        assert_eq!(COUNT_KEYPAIRS, generated_shares.secrets.len());

        let mut alpha_cpy = generated_shares.alpha.clone();
        for (i, priv_key) in priv_keys.into_iter().enumerate() {
            let (alpha, secret) = SharedKeys::<E, A, G>::forward_transform(alpha_cpy, priv_key.as_bytes(), &pub_keys[i]).unwrap();

            assert_eq!(secret.as_ref(), generated_shares.secrets[i]);

            alpha_cpy = alpha;
        }
    }

}
