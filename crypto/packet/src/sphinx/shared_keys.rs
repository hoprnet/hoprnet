use std::{marker::PhantomData, ops::Mul};

use hopr_types::{
    crypto::prelude::*,
    primitive::hybrid_array::{Array, ArraySize},
};

use super::derivation::{create_kdf_instance, generate_key_iv};

/// Represents a shared secret with a remote peer.
pub type SharedSecret = SecretValue<hopr_types::primitive::typenum::U32>;

/// Types representing a valid non-zero scalar of an additive abelian group.
pub trait Scalar: Mul<Output = Self> + Sized {
    /// Generates a random scalar using a cryptographically secure RNG.
    fn random() -> Self;

    /// Create scalar from bytes
    fn from_bytes(bytes: &[u8]) -> hopr_types::crypto::errors::Result<Self>;
}

/// Represents the Alpha value of a certain length in the Sphinx protocol
/// The length of the alpha value is directly dependent on the group element.
pub type Alpha<A> = Array<u8, A>;

/// Generic additive abelian group element with an associated scalar type.
/// It also comes with the associated Alpha value size.
/// A group element is considered valid if it is not neutral or a torsion element of small order.
pub trait GroupElement<E: Scalar>: Clone + for<'a> Mul<&'a E, Output = Self> {
    /// Length of the Alpha value - a binary representation of the group element.
    type AlphaLen: ArraySize;

    /// Converts the group element to a binary format suitable for representing the Alpha value.
    fn to_alpha(&self) -> Alpha<Self::AlphaLen>;

    /// Converts the group element from the binary format representing an Alpha value.
    fn from_alpha(alpha: Alpha<Self::AlphaLen>) -> hopr_types::crypto::errors::Result<Self>;

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
        Self::extract_key_from_alpha(&self.to_alpha(), context, salt)
    }

    /// As [`GroupElement::extract_key`], but from an already-computed Alpha encoding.
    ///
    /// Deriving two keys from the same point (the secret and the blinding factor) would otherwise
    /// call [`GroupElement::to_alpha`] twice, and on Edwards that encoding costs a field inversion.
    /// Both the sender (once per hop of every path) and the relayer (once per packet) do exactly
    /// that pair, so they encode the point once and reuse it.
    fn extract_key_from_alpha(alpha: &Alpha<Self::AlphaLen>, context: &str, salt: &[u8]) -> SharedSecret {
        let mut output = create_kdf_instance(alpha, context, Some(salt)).expect("invalid sphinx key length");
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
    ) -> hopr_types::crypto::errors::Result<SharedKeys<E, G>> {
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

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure.
            // The point is encoded to Alpha once and reused for the blinding factor below, since
            // `to_alpha` on Edwards costs a field inversion.
            let shared_secret_alpha = shared_secret.to_alpha();
            shared_keys.push(G::extract_key_from_alpha(
                &shared_secret_alpha,
                HASH_KEY_SPHINX_SECRET,
                salt.as_ref(),
            ));

            // Stop here, we don't need to compute anything more
            if i == keys_len - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let b_k = G::extract_key_from_alpha(&shared_secret_alpha, HASH_KEY_SPHINX_BLINDING, &alpha_prev.to_alpha());
            let b_k_checked = E::from_bytes(b_k.as_ref())?;

            // Update coeff_prev and alpha.
            //
            // `alpha_prev` is `coeff_prev * G` by construction, so the new alpha is the *fixed-base*
            // multiplication `coeff_prev_new * G`. Recomputing it that way instead of multiplying the
            // old point by `b_k` is the same point (scalar multiplication is associative:
            // `(coeff * G) * b = (coeff * b) * G`) but uses a precomputed base-point table, which is
            // several times cheaper than a variable-base multiplication. The sender pays this once per
            // hop on every packet it builds.
            coeff_prev = coeff_prev.mul(b_k_checked);
            alpha_prev = G::generate(&coeff_prev);

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
    ///
    /// Kept as the complete transform even though the packet decoder now uses
    /// [`SharedKeys::forward_transform_deferred`] and skips the alpha on the final hop: the suite test compares
    /// the two, and that comparison is what keeps the split from quietly diverging.
    #[cfg(test)]
    pub(crate) fn forward_transform(
        alpha: &Alpha<G::AlphaLen>,
        private_scalar: &E,
        public_element_alpha: &Alpha<G::AlphaLen>,
    ) -> hopr_types::crypto::errors::Result<(Alpha<G::AlphaLen>, SharedSecret)> {
        let (deferred, secret) = Self::forward_transform_deferred(alpha, private_scalar, public_element_alpha)?;
        Ok((deferred.finalize()?, secret))
    }

    /// The first half of [`SharedKeys::forward_transform`]: everything but the blinded alpha.
    ///
    /// Blinding the alpha is a second variable-base scalar multiplication, and a packet's *last* hop
    /// never forwards, so that point is never read there. Deferring it lets a decode pay only the
    /// multiplication that derives the shared secret when the packet is addressed to this node,
    /// which is the common case for an Exit and for any node's own traffic.
    pub(crate) fn forward_transform_deferred(
        alpha: &Alpha<G::AlphaLen>,
        private_scalar: &E,
        public_element_alpha: &Alpha<G::AlphaLen>,
    ) -> hopr_types::crypto::errors::Result<(DeferredAlpha<E, G>, SharedSecret)> {
        let alpha_point = G::from_alpha(alpha.clone())?;

        let s_k = alpha_point.clone().mul(private_scalar);

        // Reject a degenerate alpha here, on *every* hop, not only where the blinded alpha is
        // finalized. A packet whose alpha encodes the identity point makes `s_k = identity * sk`
        // independent of this node's private key: the shared secret - and so the header MAC the
        // packet is checked against - becomes computable by anyone, which would let a sender address
        // a *fully authenticated* packet to any node without knowing its key. The old eager
        // `forward_transform` rejected such a packet through its output-alpha check; deferring that
        // multiply must not move the guard off the final hop with it.
        //
        // On Edwards the two checks coincide for the identity input (multiplying it by `b_k` leaves
        // it the identity), so this preserves the old accept/reject set exactly rather than widening
        // it.
        if !s_k.is_valid() {
            return Err(CryptoError::CalculationError);
        }

        // Encode the shared point once: both keys below come from it, and `to_alpha` costs a field
        // inversion on Edwards. This is the per-packet cost of relaying, so the second encoding is
        // worth avoiding.
        let s_k_alpha = s_k.to_alpha();
        let secret = G::extract_key_from_alpha(&s_k_alpha, HASH_KEY_SPHINX_SECRET, public_element_alpha);

        let b_k = G::extract_key_from_alpha(&s_k_alpha, HASH_KEY_SPHINX_BLINDING, alpha);

        Ok((
            DeferredAlpha {
                alpha_point,
                b_k,
                _d: PhantomData,
            },
            secret,
        ))
    }
}

/// The blinded alpha a [`SharedKeys::forward_transform_deferred`] has yet to compute.
///
/// Holds the point and the blinding factor it would be multiplied by, so that a caller who learns
/// the packet is final need not pay for the multiplication whose result is then discarded.
pub(crate) struct DeferredAlpha<E: Scalar, G: GroupElement<E>> {
    alpha_point: G,
    /// Blinding factor `b_k`, kept in its raw KDF form so that [`Scalar`] need not be `Clone`.
    b_k: SharedSecret,
    _d: PhantomData<E>,
}

impl<E: Scalar, G: GroupElement<E>> DeferredAlpha<E, G> {
    /// Computes the blinded alpha, as [`SharedKeys::forward_transform`] would have.
    pub(crate) fn finalize(self) -> hopr_types::crypto::errors::Result<Alpha<G::AlphaLen>> {
        let b_k_checked = E::from_bytes(self.b_k.as_ref())?;
        let alpha_new = self.alpha_point.mul(&b_k_checked);

        if !alpha_new.is_valid() {
            return Err(hopr_types::crypto::errors::CryptoError::CalculationError);
        }

        Ok(alpha_new.to_alpha())
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

    /// Expanded form of [`Self::P`]'s public key, suitable for EC computations.
    ///
    /// Deriving it from the compact [`Keypair::Public`] can be CPU-intensive, which is why it is
    /// a type of its own: a path can be expanded once and reused across several packets.
    type ExpandedPublic;

    /// EC group element
    type G: GroupElement<Self::E> + for<'a> From<&'a Self::ExpandedPublic>;

    /// Pseudo-Random Permutation used to encrypt and decrypt packet payload
    type PRP: crypto_traits::PRP + crypto_traits::KeyIvInit;

    /// Expands a compact public key into [`Self::ExpandedPublic`].
    ///
    /// Can be CPU-intensive. Fails if `public_key` is not a valid point on the curve.
    fn expand_public(
        public_key: &<Self::P as Keypair>::Public,
    ) -> hopr_types::crypto::errors::Result<Self::ExpandedPublic>;

    /// Expands an entire path. See [`SphinxSuite::expand_public`].
    fn expand_path(
        public_keys: &[<Self::P as Keypair>::Public],
    ) -> hopr_types::crypto::errors::Result<Vec<Self::ExpandedPublic>> {
        public_keys.iter().map(|pk| Self::expand_public(pk)).collect()
    }

    /// Convenience function to generate shared keys from the path of public keys.
    ///
    /// Expands every key on the path; use [`SphinxSuite::new_shared_keys_expanded`] when the path
    /// has already been expanded, so that the cost is not paid once per packet.
    fn new_shared_keys<'a>(
        public_keys: &'a [<Self::P as Keypair>::Public],
    ) -> hopr_types::crypto::errors::Result<SharedKeys<Self::E, Self::G>>
    where
        &'a Alpha<<Self::G as GroupElement<Self::E>>::AlphaLen>: From<&'a <Self::P as Keypair>::Public>,
    {
        let expanded = Self::expand_path(public_keys)?;

        SharedKeys::generate(
            expanded
                .iter()
                .zip(public_keys)
                .map(|(expanded, compact)| (expanded.into(), compact.into()))
                .collect(),
        )
    }

    /// As [`SphinxSuite::new_shared_keys`], but for a path that is already expanded.
    ///
    /// Kept next to [`SphinxSuite::new_shared_keys`] so the two cannot diverge.
    fn new_shared_keys_expanded<'a>(
        public_keys: &'a [Self::ExpandedPublic],
    ) -> hopr_types::crypto::errors::Result<SharedKeys<Self::E, Self::G>>
    where
        &'a Alpha<<Self::G as GroupElement<Self::E>>::AlphaLen>: From<&'a Self::ExpandedPublic>,
    {
        SharedKeys::generate(public_keys.iter().map(|pk| (pk.into(), pk.into())).collect())
    }

    /// Instantiates a new Pseudo-Random Permutation IV and key for general packet data.
    fn new_prp_init(secret: &SecretKey) -> hopr_types::crypto::errors::Result<IvKey<Self::PRP>> {
        generate_key_iv(secret, HASH_KEY_PRP, None)
    }

    /// Instantiates a new Pseudo-Random Permutation IV and key for reply data.
    fn new_reply_prp_init(secret: &SecretKey16, salt: &[u8]) -> hopr_types::crypto::errors::Result<IvKey<Self::PRP>> {
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
            // Both halves of the split transform are exercised here: the deferred form must yield the
            // same secret as the eager one, and its `finalize` must yield the same alpha.
            //
            // Note the limits of this check: `forward_transform` now *delegates* to
            // `forward_transform_deferred`, so this compares the deferral round-trip against itself,
            // not against the pre-deferral implementation. Byte-equivalence with the old code was
            // established separately (an A/B harness against the parent commit, all three suites);
            // what this test pins is that the loop stays self-consistent and that no hop's alpha is
            // dropped or reordered.
            let (deferred, secret) =
                SharedKeys::<S::E, S::G>::forward_transform_deferred(&alpha_cpy, &priv_key, &pub_keys[i].1).unwrap();

            assert_eq!(
                secret.ct_eq(&generated_shares.secrets[i]).unwrap_u8(),
                1,
                "forward transform should yield the same shared secret"
            );

            let (eager_alpha, eager_secret) =
                SharedKeys::<S::E, S::G>::forward_transform(&alpha_cpy, &priv_key, &pub_keys[i].1).unwrap();
            assert_eq!(
                eager_secret.ct_eq(&secret).unwrap_u8(),
                1,
                "deferring the alpha must not change the secret"
            );

            let alpha = deferred.finalize().unwrap();
            assert_eq!(alpha, eager_alpha, "deferring the alpha must not change it");

            alpha_cpy = alpha;
        }
    }

    /// A degenerate (identity) alpha must be rejected on the **deferred** path too.
    ///
    /// This is the regression the deferral invites: the blinded alpha used to be checked on every
    /// hop, and moving that multiply behind the final-hop branch would silently move the check off
    /// the final hop with it. With an identity alpha, `s_k = identity * sk` is independent of the
    /// private key, so the shared secret - and the header MAC it authenticates - is computable by
    /// anyone: a sender could then address a fully authenticated packet to a node whose key it does
    /// not know. The check must therefore live in the first half of the transform, not in `finalize`.
    #[cfg(feature = "ed25519")]
    #[test]
    fn a_degenerate_alpha_is_rejected_on_the_deferred_path() -> anyhow::Result<()> {
        use hopr_types::crypto::primitives::{Curve25519CompressedPoint, Curve25519Point};

        let (pub_key, priv_key) = <Curve25519Point as GroupElement<Curve25519Scalar>>::random_pair();

        // Ed25519 compressed encoding of the identity point: y = 1, sign bit 0.
        let mut identity_bytes = [0u8; 32];
        identity_bytes[0] = 1;
        let identity_alpha = Curve25519CompressedPoint(identity_bytes)
            .decompress()
            .unwrap()
            .to_alpha();

        let result = SharedKeys::<Curve25519Scalar, Curve25519Point>::forward_transform_deferred(
            &identity_alpha,
            &priv_key,
            &pub_key.to_alpha(),
        );

        assert!(
            result.is_err(),
            "the deferred transform must reject an identity alpha, as the eager transform did"
        );
        Ok(())
    }
}
