use std::{marker::PhantomData, ops::Mul};

use hopr_types::{
    crypto::prelude::*,
    primitive::hybrid_array::{Array, ArraySize},
};

use super::derivation::{create_kdf_instance, generate_key_iv};

/// Represents a shared secret with a remote peer.
pub type SharedSecret = SecretValue<hopr_types::primitive::typenum::U32>;

/// Types representing a scalar of an additive abelian group.
pub trait Scalar: Mul<Output = Self> + Sized {
    /// Generates a random scalar using a cryptographically secure RNG.
    fn random() -> Self;

    /// Create scalar from bytes
    fn from_bytes(bytes: &[u8]) -> hopr_types::crypto::errors::Result<Self>;

    /// Whether this scalar is the additive identity. `from_bytes` does not always exclude it:
    /// secp256k1 accepts an all-zero encoding, and zero blinds the alpha to the identity.
    ///
    /// Curve25519 clamping cannot produce it: clamping sets bit 254, so the value is a positive
    /// multiple of eight in `[2^254, 2^255)`, and the order `L` is odd, so its first multiple
    /// divisible by eight is `8L > 2^255`. Reduction modulo `L` is therefore never zero.
    fn is_zero(&self) -> bool;
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

    /// Whether a group element this crate *derived itself* from a validated scalar is usable.
    ///
    /// Distinct from [`GroupElement::is_valid`], which validates untrusted wire input: a point
    /// produced as `s * G` for an already-parsed scalar is in the prime-order subgroup by
    /// construction, so only the identity (a scalar that is a multiple of the group order) can be
    /// wrong. Suites whose `is_valid` is more than an identity test can override this to skip that
    /// extra work on the per-packet path.
    fn is_valid_derived(&self) -> bool {
        self.is_valid()
    }

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

            // Encoded once and reused for the blinding factor: `to_alpha` is a field inversion on
            // Edwards.
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
            let b_k_checked = Self::parse_blinding_scalar(&b_k)?;

            // Same point as `alpha_prev * b_k` by associativity, but as a fixed-base multiply
            // `(coeff * b) * G`, which uses the precomputed base-point table. Once per hop, per packet.
            coeff_prev = coeff_prev.mul(b_k_checked);
            alpha_prev = G::generate(&coeff_prev);

            // A multiple of the generator, so only the identity can be wrong - and `is_valid` would
            // not even establish that much, since it admits mixed-order points.
            if !alpha_prev.is_valid_derived() {
                return Err(CryptoError::CalculationError);
            }
        }

        Ok(SharedKeys {
            alpha,
            secrets: shared_keys,
            _d: PhantomData,
        })
    }

    /// The first half of the forward transform: everything but the blinded alpha.
    ///
    /// Blinding the alpha is a second variable-base scalar multiplication, and a packet's *last* hop
    /// never forwards, so that point is never read there. Deferring it lets a decode pay only the
    /// multiplication that derives the shared secret when the packet is addressed to this node,
    /// which is the common case for an Exit and for any node's own traffic.
    ///
    /// The caller that does forward must call [`DeferredAlpha::finalize`] to finish the transform.
    pub(crate) fn forward_transform_deferred(
        alpha: &Alpha<G::AlphaLen>,
        private_scalar: &E,
        public_element_alpha: &Alpha<G::AlphaLen>,
    ) -> hopr_types::crypto::errors::Result<(DeferredAlpha<E, G>, SharedSecret)> {
        let alpha_point = G::from_alpha(alpha.clone())?;

        // Reject a degenerate alpha *before* the private-scalar multiplication, on every hop.
        //
        // For alpha `P` of order `d`, `P * sk` takes at most `d` values (and for `d = 2`, `-P = P`,
        // so just `O` or `P`), however the key was chosen. A third party can compute those candidates
        // without the node's key, so the hop secret and header MAC stop being secret.
        //
        // The base validated the *blinded output* instead, which caught only the identity; it accepted
        // all 200 sampled keys for an order-2 input. Rejecting the whole class on the input is new.
        if !alpha_point.is_valid() {
            return Err(CryptoError::CalculationError);
        }

        let s_k = alpha_point.clone().mul(private_scalar);

        // Encode the shared point once: both keys below come from it, and `to_alpha` costs a field
        // inversion on Edwards. This is the per-packet cost of relaying, so the second encoding is
        // worth avoiding.
        let s_k_alpha = s_k.to_alpha();
        let secret = G::extract_key_from_alpha(&s_k_alpha, HASH_KEY_SPHINX_SECRET, public_element_alpha);

        let b_k = G::extract_key_from_alpha(&s_k_alpha, HASH_KEY_SPHINX_BLINDING, alpha);

        Self::parse_blinding_scalar(&b_k)?;

        Ok((
            DeferredAlpha {
                alpha_point,
                b_k,
                _d: PhantomData,
            },
            secret,
        ))
    }

    /// Validates a KDF output as a blinding scalar, in the first half of the transform on every hop.
    ///
    /// It cannot live in [`DeferredAlpha::finalize`]: a final hop never calls that, so a rejected KDF
    /// output would be accepted there. secp256k1 both rejects values at or above the group order and
    /// *accepts* all-zero, and zero blinds the alpha to the identity - which the eager transform
    /// refused on every hop, so the explicit zero test preserves that. (Curve25519 clamping cannot
    /// produce zero; see [`Scalar::is_zero`].)
    fn parse_blinding_scalar(b_k: &SharedSecret) -> hopr_types::crypto::errors::Result<E> {
        let b_k = E::from_bytes(b_k.as_ref())?;
        if b_k.is_zero() {
            return Err(CryptoError::CalculationError);
        }
        Ok(b_k)
    }
}

/// The blinded alpha a [`SharedKeys::forward_transform_deferred`] has yet to compute.
///
/// A final hop discards it, so it never pays the multiplication.
pub(crate) struct DeferredAlpha<E: Scalar, G: GroupElement<E>> {
    alpha_point: G,
    /// Blinding factor `b_k` in raw KDF form, so it is zeroized on drop (`E` has no `ZeroizeOnDrop`).
    /// Already parsed and checked nonzero in the first half.
    b_k: SharedSecret,
    _d: PhantomData<E>,
}

impl<E: Scalar, G: GroupElement<E>> DeferredAlpha<E, G> {
    /// Computes the blinded alpha - the multiplication the first half leaves out.
    pub(crate) fn finalize(self) -> hopr_types::crypto::errors::Result<Alpha<G::AlphaLen>> {
        let b_k_checked = SharedKeys::<E, G>::parse_blinding_scalar(&self.b_k)?;
        let alpha_new = self.alpha_point.mul(&b_k_checked);

        // Not the prime-order subgroup: `is_valid` admits mixed-order points. But for `P = T + Q`
        // with `T` small order and `Q` of prime order, `b * P` keeps a nonzero `b * Q` for any
        // nonzero `b mod L`, so this can never be the identity. The check is defensive.
        if !alpha_new.is_valid_derived() {
            return Err(hopr_types::crypto::errors::CryptoError::CalculationError);
        }

        Ok(alpha_new.to_alpha())
    }

    /// Lets a test build the state that makes `finalize` fail (an identity point).
    #[cfg(test)]
    pub(crate) fn for_test(alpha_point: G, b_k: SharedSecret) -> Self {
        Self {
            alpha_point,
            b_k,
            _d: PhantomData,
        }
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
    use anyhow::Context;
    use subtle::ConstantTimeEq;

    use super::*;

    #[allow(clippy::type_complexity)]
    pub fn generic_sphinx_suite_test<S: SphinxSuite>(node_count: usize) -> anyhow::Result<()> {
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
        let generated_shares =
            SharedKeys::<S::E, S::G>::generate(pub_keys_alpha.clone()).context("sender: generate the shared keys")?;
        assert_eq!(
            node_count,
            generated_shares.secrets.len(),
            "number of generated keys should be equal to the number of nodes"
        );

        let mut alpha_cpy = generated_shares.alpha.clone();
        for (i, priv_key) in priv_keys.into_iter().enumerate() {
            // Both halves of the split transform are exercised here, against a *reference*
            // implementation of the pre-deferral eager transform written out below. A comparison
            // against a helper that merely calls `forward_transform_deferred` + `finalize` would not
            // do: a change to the blinding salt or the alpha encoding would move both sides and leave
            // the assertion passing.
            let (expected_alpha, expected_secret) =
                reference_forward_transform::<S>(&alpha_cpy, &priv_key, &pub_keys[i].1)
                    .with_context(|| format!("hop {i}: reference transform"))?;

            let (deferred, secret) =
                SharedKeys::<S::E, S::G>::forward_transform_deferred(&alpha_cpy, &priv_key, &pub_keys[i].1)
                    .with_context(|| format!("hop {i}: deferred transform"))?;

            assert_eq!(
                secret.ct_eq(&generated_shares.secrets[i]).unwrap_u8(),
                1,
                "hop {i}: forward transform should yield the same shared secret"
            );
            assert_eq!(
                secret.ct_eq(&expected_secret).unwrap_u8(),
                1,
                "hop {i}: the deferred secret must match the reference transform"
            );

            let alpha = deferred.finalize().with_context(|| format!("hop {i}: finalize"))?;
            assert_eq!(
                alpha, expected_alpha,
                "hop {i}: the finalized alpha must match the reference transform"
            );

            alpha_cpy = alpha;
        }
        Ok(())
    }

    /// Pins the relay transform's wire bytes against values `ff0076efaa` emitted.
    ///
    /// The round-trip test compares the deferral against itself and cannot see a changed KDF context
    /// or blinding salt; these bytes can.
    #[cfg(feature = "ed25519")]
    #[test]
    fn the_relay_transform_matches_the_pre_deferral_known_answers() -> anyhow::Result<()> {
        use hopr_types::{crypto::keypairs::OffchainKeypair, primitive::typenum::U32};

        // Alpha and key pair from the pre-change run; all three are inputs, not derived here.
        let alpha: Alpha<U32> = const_hex::decode("a8c8f046aea14953474ca0787e7eb3d908439f2fd66ef019d54cfa92301483bc")
            .context("alpha hex")?
            .as_slice()
            .try_into()
            .expect("alpha is 32 bytes");
        let keys: Vec<OffchainKeypair> = [[0x11u8; 32], [0x22u8; 32]]
            .iter()
            .map(|s| OffchainKeypair::from_secret(s).expect("valid fixed secret"))
            .collect();

        let expected = [
            (
                "7f639766b036db9f885db049f738e45448e125395b0f4ffab2dc6df62fa8c96c",
                "0e3558196aa341329cd97a7d1c880ecc34d96f715fa094bd5c44d6273c318860",
            ),
            (
                "b0402a50cd8cb78afdd812d5f5dcd09bc0270ac69b4f439de72248df151c4ebe",
                "ad91af7c908b70209abbe42b770b08871482f8f33d512893188c960509c66ba8",
            ),
        ];

        let mut alpha = alpha;
        for (hop, (kp, (want_alpha, want_secret))) in keys.iter().zip(expected).enumerate() {
            let pubkey = *kp.public();
            let pub_alpha: &Alpha<U32> = (&pubkey).into();
            let (deferred, secret) = SharedKeys::<Curve25519Scalar, Curve25519Point>::forward_transform_deferred(
                &alpha,
                &kp.into(),
                pub_alpha,
            )
            .with_context(|| format!("hop {hop}: deferred transform"))?;
            assert_eq!(
                const_hex::encode(secret.as_ref()),
                want_secret,
                "hop {hop}: shared secret changed"
            );
            alpha = deferred.finalize().with_context(|| format!("hop {hop}: finalize"))?;
            assert_eq!(
                const_hex::encode(AsRef::<[u8]>::as_ref(&alpha)),
                want_alpha,
                "hop {hop}: blinded alpha changed"
            );
        }
        Ok(())
    }

    /// The pre-deferral eager transform, re-derived rather than calling the code under test.
    #[allow(clippy::type_complexity)] // the alpha type is spelled out by the trait, not chosen here
    fn reference_forward_transform<S: SphinxSuite>(
        alpha: &Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
        private_scalar: &S::E,
        public_element_alpha: &Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
    ) -> anyhow::Result<(Alpha<<S::G as GroupElement<S::E>>::AlphaLen>, SharedSecret)> {
        let alpha_point = S::G::from_alpha(alpha.clone()).context("reference: decode the alpha")?;
        let s_k = alpha_point.clone().mul(private_scalar);
        let secret = s_k.extract_key(HASH_KEY_SPHINX_SECRET, public_element_alpha);
        let b_k = s_k.extract_key(HASH_KEY_SPHINX_BLINDING, alpha);
        let b_k = S::E::from_bytes(b_k.as_ref()).context("reference: parse the blinding scalar")?;
        let alpha_new = alpha_point.mul(&b_k);
        anyhow::ensure!(alpha_new.is_valid(), "reference transform produced an invalid alpha");
        Ok((alpha_new.to_alpha(), secret))
    }

    /// An identity alpha must be rejected on the deferred path, where `finalize` no longer runs.
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

    /// Each of the eight small-order classes must be rejected, not just the identity.
    #[cfg(feature = "ed25519")]
    #[test]
    fn a_nonidentity_small_order_alpha_is_rejected() -> anyhow::Result<()> {
        use hopr_types::crypto::primitives::{Curve25519CompressedPoint, Curve25519Point};

        let (pub_key, priv_key) = <Curve25519Point as GroupElement<Curve25519Scalar>>::random_pair();

        // The eight points of the order-8 torsion subgroup, in compressed Edwards form. The first is
        // the identity; the rest are the nonidentity classes the old guard admitted.
        const SMALL_ORDER: [[u8; 32]; 8] = [
            [
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            [
                0x26, 0xe8, 0x95, 0x8f, 0xc2, 0xb2, 0x27, 0xb0, 0x45, 0xc3, 0xf4, 0x89, 0xf2, 0xef, 0x98, 0xf0, 0xd5,
                0xdf, 0xac, 0x05, 0xd3, 0xc6, 0x33, 0x39, 0xb1, 0x38, 0x02, 0x88, 0x6d, 0x53, 0xfc, 0x05,
            ],
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            [
                0xc7, 0x17, 0x6a, 0x70, 0x3d, 0x4d, 0xd8, 0x4f, 0xba, 0x3c, 0x0b, 0x76, 0x0d, 0x10, 0x67, 0x0f, 0x2a,
                0x20, 0x53, 0xfa, 0x2c, 0x39, 0xcc, 0xc6, 0x4e, 0xc7, 0xfd, 0x77, 0x92, 0xac, 0x03, 0x7a,
            ],
            [
                0xec, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
            ],
            [
                0xc7, 0x17, 0x6a, 0x70, 0x3d, 0x4d, 0xd8, 0x4f, 0xba, 0x3c, 0x0b, 0x76, 0x0d, 0x10, 0x67, 0x0f, 0x2a,
                0x20, 0x53, 0xfa, 0x2c, 0x39, 0xcc, 0xc6, 0x4e, 0xc7, 0xfd, 0x77, 0x92, 0xac, 0x03, 0xfa,
            ],
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            ],
            [
                0x26, 0xe8, 0x95, 0x8f, 0xc2, 0xb2, 0x27, 0xb0, 0x45, 0xc3, 0xf4, 0x89, 0xf2, 0xef, 0x98, 0xf0, 0xd5,
                0xdf, 0xac, 0x05, 0xd3, 0xc6, 0x33, 0x39, 0xb1, 0x38, 0x02, 0x88, 0x6d, 0x53, 0xfc, 0x85,
            ],
        ];

        // The fixture must be eight *distinct* canonical encodings, or the loop silently tests fewer
        // torsion classes than it claims: duplicate entries would make it pass while leaving a class
        // uncovered.
        let distinct: std::collections::BTreeSet<_> = SMALL_ORDER.iter().collect();
        assert_eq!(
            distinct.len(),
            8,
            "the torsion fixtures must be eight distinct encodings"
        );
        for (idx, bytes) in SMALL_ORDER.iter().enumerate() {
            // Canonicality: re-compressing a decompressed point must give the same bytes back.
            let p = Curve25519CompressedPoint(*bytes)
                .decompress()
                .with_context(|| format!("small-order point {idx} must decompress"))?;
            assert_eq!(
                p.compress().0,
                *bytes,
                "small-order fixture {idx} is not the canonical encoding of its point"
            );
        }

        for (idx, bytes) in SMALL_ORDER.iter().enumerate() {
            let point = Curve25519CompressedPoint(*bytes)
                .decompress()
                .with_context(|| format!("small-order point {idx} must decompress"))?;
            assert!(
                !<Curve25519Point as GroupElement<Curve25519Scalar>>::is_valid(&point),
                "small-order point {idx} must be invalid"
            );

            let result = SharedKeys::<Curve25519Scalar, Curve25519Point>::forward_transform_deferred(
                &point.to_alpha(),
                &priv_key,
                &pub_key.to_alpha(),
            );
            assert!(
                result.is_err(),
                "small-order alpha {idx} must be rejected before any key derivation"
            );
        }
        Ok(())
    }

    /// A zero blinding scalar must be rejected on every hop, including the final one, where
    /// `finalize` no longer runs to catch the identity it produces.
    #[cfg(feature = "secp256k1")]
    #[test]
    fn a_zero_blinding_scalar_is_rejected() -> anyhow::Result<()> {
        use crate::sphinx::ec_groups::{Secp256k1Point, Secp256k1Scalar};

        // The premise: `from_bytes` really does accept an all-zero encoding, so the value is
        // reachable from a KDF output rather than only from an uninitialised scalar.
        let zero_bytes = SharedSecret::default();
        let parsed = <Secp256k1Scalar as Scalar>::from_bytes(zero_bytes.as_ref())
            .expect("secp256k1 accepts an all-zero scalar encoding");
        assert!(parsed.is_zero(), "an all-zero encoding must parse to zero");
        assert!(Secp256k1Scalar::default().is_zero(), "the default scalar must be zero");

        // The check under test must reject it. This is the assertion the fix exists for: without the
        // `is_zero` term the parse alone succeeds and a final hop accepts the packet.
        assert!(
            SharedKeys::<Secp256k1Scalar, Secp256k1Point>::parse_blinding_scalar(&zero_bytes).is_err(),
            "a zero blinding scalar must be rejected on every hop"
        );

        // And a nonzero scalar must still be accepted, so the guard does not reject everything.
        // k256 scalars are big-endian, so the scalar one is a trailing 1.
        let mut one_bytes = [0u8; 32];
        one_bytes[31] = 1;
        let one = <Secp256k1Scalar as Scalar>::from_bytes(&one_bytes).context("one must parse")?;
        assert!(!one.is_zero(), "a nonzero scalar must not report as zero");
        let one_secret = SharedSecret::try_from(one_bytes.as_slice()).context("one is 32 bytes")?;
        assert!(
            SharedKeys::<Secp256k1Scalar, Secp256k1Point>::parse_blinding_scalar(&one_secret).is_ok(),
            "a nonzero blinding scalar must be accepted"
        );

        // Zero really does blind the alpha away to the identity, which is what makes it unacceptable:
        // `P * 0 = O` for every alpha.
        use hopr_types::primitive::typenum::U33;

        let mut alpha_bytes = Alpha::<U33>::default();
        alpha_bytes.copy_from_slice(
            const_hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                .context("secp256k1 generator in compressed SEC1 form")?
                .as_slice(),
        );
        let generator = <Secp256k1Point as GroupElement<Secp256k1Scalar>>::from_alpha(alpha_bytes)
            .context("the generator must decode")?;
        let blinded = generator.mul(&parsed);
        assert!(
            !blinded.is_valid_derived(),
            "multiplying by a zero scalar must give the identity, which the derived check rejects"
        );
        Ok(())
    }

    /// The Montgomery *transform* must reject small-order alphas too: its `from_alpha` accepts any
    /// 32-byte coordinate, so the rejection has to come from the guard inside the transform.
    #[cfg(feature = "x25519")]
    #[test]
    fn a_small_order_montgomery_alpha_is_rejected_by_the_transform() -> anyhow::Result<()> {
        use hopr_types::crypto::primitives::{Curve25519CompressedPoint, Curve25519MontgomeryPoint};

        let (pub_key, priv_key) = <Curve25519MontgomeryPoint as GroupElement<Curve25519Scalar>>::random_pair();

        // The eight Edwards torsion points, projected onto their Montgomery `u` coordinates. The
        // projection collapses `P` and `-P`, so this yields four distinct coordinates covering every
        // torsion class; taking them through the Edwards side keeps the fixture tied to the group
        // structure rather than to hand-copied constants.
        const EDWARDS_TORSION: [[u8; 32]; 8] = [
            [
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            ],
            [
                0x26, 0xe8, 0x95, 0x8f, 0xc2, 0xb2, 0x27, 0xb0, 0x45, 0xc3, 0xf4, 0x89, 0xf2, 0xef, 0x98, 0xf0, 0xd5,
                0xdf, 0xac, 0x05, 0xd3, 0xc6, 0x33, 0x39, 0xb1, 0x38, 0x02, 0x88, 0x6d, 0x53, 0xfc, 0x05,
            ],
            [
                0x26, 0xe8, 0x95, 0x8f, 0xc2, 0xb2, 0x27, 0xb0, 0x45, 0xc3, 0xf4, 0x89, 0xf2, 0xef, 0x98, 0xf0, 0xd5,
                0xdf, 0xac, 0x05, 0xd3, 0xc6, 0x33, 0x39, 0xb1, 0x38, 0x02, 0x88, 0x6d, 0x53, 0xfc, 0x85,
            ],
            [
                0xc7, 0x17, 0x6a, 0x70, 0x3d, 0x4d, 0xd8, 0x4f, 0xba, 0x3c, 0x0b, 0x76, 0x0d, 0x10, 0x67, 0x0f, 0x2a,
                0x20, 0x53, 0xfa, 0x2c, 0x39, 0xcc, 0xc6, 0x4e, 0xc7, 0xfd, 0x77, 0x92, 0xac, 0x03, 0x7a,
            ],
            [
                0xc7, 0x17, 0x6a, 0x70, 0x3d, 0x4d, 0xd8, 0x4f, 0xba, 0x3c, 0x0b, 0x76, 0x0d, 0x10, 0x67, 0x0f, 0x2a,
                0x20, 0x53, 0xfa, 0x2c, 0x39, 0xcc, 0xc6, 0x4e, 0xc7, 0xfd, 0x77, 0x92, 0xac, 0x03, 0xfa,
            ],
            [
                0xec, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
            ],
            // The two signs at y = 0 are the order-four pair; they project to one Montgomery coordinate.
            [0x00; 32],
        ];

        let mut coords = std::collections::BTreeSet::new();
        let mut edwards_points = std::collections::BTreeSet::new();
        for (idx, bytes) in EDWARDS_TORSION.into_iter().enumerate() {
            let edwards = Curve25519CompressedPoint(bytes)
                .decompress()
                .with_context(|| format!("torsion fixture {idx} must decompress"))?;
            // Count the decompressed points, not just the projected coordinates: the four-`u` check
            // below cannot see a duplicated Edwards entry that projects onto a coordinate the table
            // already has, which is how an earlier version of this fixture silently held seven.
            edwards_points.insert(edwards.compress().0);
            let montgomery = edwards.to_montgomery();
            coords.insert(montgomery.0);

            let result = SharedKeys::<Curve25519Scalar, Curve25519MontgomeryPoint>::forward_transform_deferred(
                &montgomery.to_alpha(),
                &priv_key,
                &pub_key.to_alpha(),
            );
            assert!(
                result.is_err(),
                "small-order Montgomery alpha from fixture {idx} must be rejected by the transform"
            );
        }
        assert_eq!(
            edwards_points.len(),
            8,
            "the fixture must be eight distinct Edwards torsion points"
        );
        assert_eq!(
            coords.len(),
            4,
            "the eight torsion points must project onto four distinct u coordinates"
        );

        // Control: an alpha this suite generates must still go through.
        let (alpha, _) = <Curve25519MontgomeryPoint as GroupElement<Curve25519Scalar>>::random_pair();
        assert!(
            SharedKeys::<Curve25519Scalar, Curve25519MontgomeryPoint>::forward_transform_deferred(
                &alpha.to_alpha(),
                &priv_key,
                &pub_key.to_alpha(),
            )
            .is_ok(),
            "a generated Montgomery alpha must still be accepted"
        );
        Ok(())
    }
}
