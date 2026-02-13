use std::{marker::PhantomData, ops::Mul};

use generic_array::{ArrayLength, GenericArray};
use hopr_crypto_types::prelude::*;

use crate::derivation::{create_kdf_instance, generate_key_iv};

/// Represents a shared secret with a remote peer.
pub type SharedSecret = SecretValue<typenum::U32>;

/// Types representing a valid non-zero scalar of an additive abelian group.
pub trait Scalar: Mul<Output = Self> + Sized {
    /// Generates a random scalar using a cryptographically secure RNG.
    fn random() -> Self;

    /// Create scalar from bytes
    fn from_bytes(bytes: &[u8]) -> hopr_crypto_types::errors::Result<Self>;
}

/// Represents the Alpha value of a certain length in the Sphinx protocol
/// The length of the alpha value is directly dependent on the group element.
pub type Alpha<A> = GenericArray<u8, A>;

/// Generic additive abelian group element with an associated scalar type.
/// It also comes with the associated Alpha value size.
/// A group element is considered valid if it is not neutral or a torsion element of small order.
pub trait GroupElement<E: Scalar>: Clone + for<'a> Mul<&'a E, Output = Self> {
    /// Length of the Alpha value - a binary representation of the group element.
    type AlphaLen: ArrayLength;

    /// Converts the group element to a binary format suitable for representing the Alpha value.
    fn to_alpha(&self) -> Alpha<Self::AlphaLen>;

    /// Converts the group element from the binary format representing an Alpha value.
    fn from_alpha(alpha: Alpha<Self::AlphaLen>) -> hopr_crypto_types::errors::Result<Self>;

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

    /// Extract a keying material from a group element using a KDF
    fn extract_key(&self, context: &str, salt: &[u8]) -> SharedSecret {
        let mut output = create_kdf_instance(&self.to_alpha(), context, Some(salt)).expect("invalid sphinx key length");
        let mut out = SharedSecret::default();
        output.fill(out.as_mut());
        out
    }
}

/// Structure containing shared keys for peers using the Sphinx algorithm.
pub struct SharedKeys<E: Scalar, G: GroupElement<E>> {
    pub alpha: Alpha<G::AlphaLen>,
    pub secrets: Vec<SharedSecret>,
    _d: PhantomData<(E, G)>,
}

const HASH_KEY_SPHINX_SECRET: &str = "HASH_KEY_SPHINX_SECRET";
const HASH_KEY_SPHINX_BLINDING: &str = "HASH_KEY_SPHINX_BLINDING";

impl<E: Scalar, G: GroupElement<E>> SharedKeys<E, G> {
    /// Generates shared secrets given the group element of the peers and their Alpha value encodings.
    ///
    /// The order of the peer group elements is preserved for resulting shared keys.
    pub(crate) fn generate(
        peer_group_elements: Vec<(G, &Alpha<G::AlphaLen>)>,
    ) -> hopr_crypto_types::errors::Result<SharedKeys<E, G>> {
        let mut shared_keys = Vec::with_capacity(peer_group_elements.len());

        // coeff_prev becomes: x * b_0 * b_1 * b_2 * ...
        // alpha_prev becomes: x * b_0 * b_1 * b_2 * ... * G
        let (mut alpha_prev, mut coeff_prev) = G::random_pair();

        // Save the part of the result
        let alpha = alpha_prev.to_alpha();

        // Iterate through all the given peer public keys
        let keys_len = peer_group_elements.len();
        for (i, (group_element, salt)) in peer_group_elements.into_iter().enumerate() {
            // Multiply the public key by the current coefficient
            let shared_secret = group_element.mul(&coeff_prev);

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            shared_keys.push(shared_secret.extract_key(HASH_KEY_SPHINX_SECRET, salt.as_ref()));

            // Stop here, we don't need to compute anything more
            if i == keys_len - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let b_k = shared_secret.extract_key(HASH_KEY_SPHINX_BLINDING, &alpha_prev.to_alpha());
            let b_k_checked = E::from_bytes(b_k.as_ref())?;

            // Update coeff_prev and alpha
            alpha_prev = alpha_prev.mul(&b_k_checked);
            coeff_prev = coeff_prev.mul(b_k_checked);

            if !alpha_prev.is_valid() {
                return Err(CryptoError::CalculationError);
            }
        }

        Ok(SharedKeys {
            alpha,
            secrets: shared_keys,
            _d: PhantomData,
        })
    }

    /// Calculates the forward transformation given the local private key.
    ///
    /// Efficiency note: the `public_element_alpha` is a precomputed group element Alpha encoding associated with the
    /// `private_scalar`.
    pub(crate) fn forward_transform(
        alpha: &Alpha<G::AlphaLen>,
        private_scalar: &E,
        public_element_alpha: &Alpha<G::AlphaLen>,
    ) -> hopr_crypto_types::errors::Result<(Alpha<G::AlphaLen>, SharedSecret)> {
        let alpha_point = G::from_alpha(alpha.clone())?;

        let s_k = alpha_point.clone().mul(private_scalar);

        let secret = s_k.extract_key(HASH_KEY_SPHINX_SECRET, public_element_alpha);

        let b_k = s_k.extract_key(HASH_KEY_SPHINX_BLINDING, alpha);

        let b_k_checked = E::from_bytes(b_k.as_ref())?;
        let alpha_new = alpha_point.mul(&b_k_checked);

        Ok((alpha_new.to_alpha(), secret))
    }
}

const HASH_KEY_PRP: &str = "HASH_KEY_PRP";

const HASH_KEY_REPLY_PRP: &str = "HASH_KEY_REPLY_PRP";

/// Represents an instantiation of the Spinx protocol using the given EC group and corresponding public key object.
pub trait SphinxSuite {
    /// Keypair corresponding to the EC group
    type P: Keypair;

    /// Scalar type supported by the EC group
    type E: Scalar + for<'a> From<&'a Self::P>;

    /// EC group element
    type G: GroupElement<Self::E> + for<'a> From<&'a <Self::P as Keypair>::Public>;

    /// Pseudo-Random Permutation used to encrypt and decrypt packet payload
    type PRP: crypto_traits::PRP + crypto_traits::KeyIvInit;

    /// Convenience function to generate shared keys from the path of public keys.
    fn new_shared_keys<'a>(
        public_keys: &'a [<Self::P as Keypair>::Public],
    ) -> hopr_crypto_types::errors::Result<SharedKeys<Self::E, Self::G>>
    where
        &'a Alpha<<Self::G as GroupElement<Self::E>>::AlphaLen>: From<&'a <Self::P as Keypair>::Public>,
    {
        SharedKeys::generate(public_keys.iter().map(|pk| (pk.into(), pk.into())).collect())
    }

    /// Instantiates a new Pseudo-Random Permutation IV and key for general packet data.
    fn new_prp_init(secret: &SecretKey) -> hopr_crypto_types::errors::Result<IvKey<Self::PRP>> {
        generate_key_iv(secret, HASH_KEY_PRP, None)
    }

    /// Instantiates a new Pseudo-Random Permutation IV and key for reply data.
    fn new_reply_prp_init(secret: &SecretKey16, salt: &[u8]) -> hopr_crypto_types::errors::Result<IvKey<Self::PRP>> {
        generate_key_iv(secret, HASH_KEY_REPLY_PRP, Some(salt))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use subtle::ConstantTimeEq;

    use super::*;

    #[allow(clippy::type_complexity)]
    pub fn generic_sphinx_suite_test<S: SphinxSuite>(node_count: usize) {
        let (pub_keys, priv_keys): (Vec<(S::G, Alpha<<S::G as GroupElement<S::E>>::AlphaLen>)>, Vec<S::E>) = (0
            ..node_count)
            .map(|_| {
                let pair = S::G::random_pair();
                ((pair.0.clone(), pair.0.to_alpha()), pair.1)
            })
            .unzip();

        let pub_keys_alpha = pub_keys
            .iter()
            .map(|(pk, alpha)| (pk.clone(), alpha))
            .collect::<Vec<_>>();

        // Now generate the key shares for the public keys
        let generated_shares = SharedKeys::<S::E, S::G>::generate(pub_keys_alpha.clone()).unwrap();
        assert_eq!(
            node_count,
            generated_shares.secrets.len(),
            "number of generated keys should be equal to the number of nodes"
        );

        let mut alpha_cpy = generated_shares.alpha.clone();
        for (i, priv_key) in priv_keys.into_iter().enumerate() {
            let (alpha, secret) =
                SharedKeys::<S::E, S::G>::forward_transform(&alpha_cpy, &priv_key, &pub_keys[i].1).unwrap();

            assert_eq!(
                secret.ct_eq(&generated_shares.secrets[i]).unwrap_u8(),
                1,
                "forward transform should yield the same shared secret"
            );

            alpha_cpy = alpha;
        }
    }
}
