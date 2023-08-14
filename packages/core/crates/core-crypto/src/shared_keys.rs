use blake2::Blake2s256;
use generic_array::{ArrayLength, GenericArray};
use std::marker::PhantomData;
use std::ops::Mul;

use crate::errors::CryptoError::CalculationError;
use hkdf::SimpleHkdf;

use crate::errors::Result;
use crate::keypairs::Keypair;
use crate::utils::SecretValue;

/// Represents a shared secret with a remote peer.
pub type SharedSecret = SecretValue<typenum::U32>;

/// Types representing a valid non-zero scalar an additive abelian group.
pub trait Scalar: Mul<Output = Self> + Sized {
    /// Generates a random scalar using a cryptographically secure RNG.
    fn random() -> Self;

    /// Create scalar from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>;

    /// Convert scalar to bytes.
    fn to_bytes(&self) -> Box<[u8]>;
}

/// Represents the Alpha value of a certain length in the Sphinx protocol
/// The length of the alpha value is directly dependent on the group element.
pub type Alpha<A> = GenericArray<u8, A>;

/// Generic additive abelian group element with an associated scalar type.
/// It also comes with the associated Alpha value size.
/// A group element is considered valid if it is not neutral or a torsion element of small order.
pub trait GroupElement<E: Scalar>: Clone + for<'a> Mul<&'a E, Output = Self> {
    /// Length of the Alpha value - a binary representation of the group element.
    type AlphaLen: ArrayLength<u8>;

    /// Converts the group element to a binary format suitable for representing the Alpha value.
    fn to_alpha(&self) -> Alpha<Self::AlphaLen>;

    /// Converts the group element from the binary format representing an Alpha value.
    fn from_alpha(alpha: Alpha<Self::AlphaLen>) -> Result<Self>;

    /// Create a group element using the group generator and the given scalar
    fn generate(scalar: &E) -> Self;

    /// Group element is considered valid if it is not a neutral element and also not a torsion element of small order.
    fn is_valid(&self) -> bool;

    /// Generates a random pair of group element and secret scalar.
    /// This is a convenience method that internally calls the `random` method of the associated Scalar
    /// and constructs the group element using `generate`.
    fn random_pair() -> (Self, E) {
        let scalar = E::random();
        (Self::generate(&scalar), scalar)
    }

    /// Extract a keying material from a group element using HKDF extract
    fn extract_key(&self, salt: &[u8]) -> SharedSecret {
        let ikm = self.to_alpha();
        SimpleHkdf::<Blake2s256>::extract(Some(salt), ikm.as_ref()).0.into()
    }

    /// Performs KDF expansion from the given group element using HKDF expand
    fn expand_key(&self, salt: &[u8]) -> SharedSecret {
        let mut out = GenericArray::default();
        let ikm = self.to_alpha();
        SimpleHkdf::<Blake2s256>::new(Some(salt), &ikm)
            .expand(b"", &mut out)
            .expect("invalid size of the shared secret output"); // Cannot panic, unless the constants are wrong

        out.into()
    }
}

/// Structure containing shared keys for peers using the Sphinx algorithm.
pub struct SharedKeys<E: Scalar, G: GroupElement<E>> {
    pub alpha: Alpha<G::AlphaLen>,
    pub secrets: Vec<SharedSecret>,
    _e: PhantomData<E>,
    _g: PhantomData<G>,
}

impl<E: Scalar, G: GroupElement<E>> SharedKeys<E, G> {
    /// Generates shared secrets given the group element of the peers.
    /// The order of the peer group elements is preserved for resulting shared keys.
    pub fn generate(peer_group_elements: Vec<G>) -> Result<SharedKeys<E, G>> {
        let mut shared_keys = Vec::new();

        // coeff_prev becomes: x * b_0 * b_1 * b_2 * ...
        // alpha_prev becomes: x * b_0 * b_1 * b_2 * ... * G
        let (mut alpha_prev, mut coeff_prev) = G::random_pair();

        // Save the part of the result
        let alpha = alpha_prev.to_alpha();

        // Iterate through all the given peer public keys
        let keys_len = peer_group_elements.len();
        for (i, group_element) in peer_group_elements.into_iter().enumerate() {
            // Try to decode the given public key point & multiply by the current coefficient
            let salt = group_element.to_alpha();
            let shared_secret = group_element.mul(&coeff_prev);

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            shared_keys.push(shared_secret.extract_key(&salt));

            // Stop here, we don't need to compute anything more
            if i == keys_len - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let b_k = shared_secret.expand_key(&alpha_prev.to_alpha());
            let b_k_checked = E::from_bytes(b_k.as_ref())?;

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
            _g: PhantomData,
        })
    }

    /// Calculates the forward transformation for the given the local private key.
    /// The `public_group_element` is a precomputed group element associated to the private key for efficiency.
    pub fn forward_transform(
        alpha: &Alpha<G::AlphaLen>,
        private_scalar: &E,
        public_group_element: &G,
    ) -> Result<(Alpha<G::AlphaLen>, SharedSecret)> {
        let alpha_point = G::from_alpha(alpha.clone())?;

        let s_k = alpha_point.clone().mul(private_scalar);

        let secret = s_k.extract_key(&public_group_element.to_alpha());

        let b_k = s_k.expand_key(alpha);

        let b_k_checked = E::from_bytes(b_k.as_ref())?;
        let alpha_new = alpha_point.mul(&b_k_checked);

        Ok((alpha_new.to_alpha(), secret))
    }
}

/// Represents an instantiation of the Spinx protocol using the given EC group and corresponding public key object.
pub trait SphinxSuite {
    /// Keypair corresponding to the EC group
    type P: Keypair;

    /// Scalar type supported by the EC group
    type E: Scalar + for<'a> From<&'a Self::P>;

    /// EC group element
    type G: GroupElement<Self::E> + for<'a> From<&'a <Self::P as Keypair>::Public>;

    /// Convenience function to generate shared keys from the path of public keys.
    fn new_shared_keys(public_keys: &[<Self::P as Keypair>::Public]) -> Result<SharedKeys<Self::E, Self::G>> {
        SharedKeys::generate(public_keys.iter().map(|pk| pk.into()).collect())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use subtle::ConstantTimeEq;

    pub fn generic_sphinx_suite_test<S: SphinxSuite>(node_count: usize) {
        let (pub_keys, priv_keys): (Vec<S::G>, Vec<S::E>) = (0..node_count).map(|_| S::G::random_pair()).unzip();

        // Now generate the key shares for the public keys
        let generated_shares = SharedKeys::<S::E, S::G>::generate(pub_keys.clone()).unwrap();
        assert_eq!(
            node_count,
            generated_shares.secrets.len(),
            "number of generated keys should be equal to the number of nodes"
        );

        let mut alpha_cpy = generated_shares.alpha.clone();
        for (i, priv_key) in priv_keys.into_iter().enumerate() {
            let (alpha, secret) =
                SharedKeys::<S::E, S::G>::forward_transform(&alpha_cpy, &priv_key, &pub_keys[i]).unwrap();

            assert_eq!(
                secret.ct_eq(&generated_shares.secrets[i]).unwrap_u8(),
                1,
                "forward transform should yield the same shared secret"
            );

            alpha_cpy = alpha;
        }
    }
}
