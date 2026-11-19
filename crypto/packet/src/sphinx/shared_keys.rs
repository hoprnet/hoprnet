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

            // `alpha_prev` is `coeff * G` for a scalar `coeff` that `E::from_bytes` already rejected
            // if non-canonical, so it is in the prime-order subgroup by construction. The full
            // `is_valid` (three doublings) would re-establish that; only the identity has to be
            // excluded here, because a `coeff` that is a multiple of the group order gives `O`. The
            // wire input is where the full check belongs, and `forward_transform_deferred` does it.
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
        // The old eager `forward_transform` rejected it through its check on the *output* alpha, which
        // the final hop no longer computes - but the reason to reject is the input. With a point of
        // small order (the identity above all), `alpha * sk` is a small-order point too, and one that
        // depends on `sk` only through the parity of the reduced scalar: for a clamped scalar the
        // four subgroups' points come out the same for half the keys and negated for the other half.
        // The KDF therefore sees only a couple of possible points, so the shared secret - and with it
        // the header MAC - can be derived by a third party from public information rather than by the
        // recipient alone. Checking the public input keeps that check on the final hop and also makes
        // it independent of which group the suite computes in.
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

        // Parse the blinding scalar here rather than in `finalize`, so that a KDF output the suite
        // rejects is refused on *every* hop, as the eager transform refused it. secp256k1 is the case
        // that matters: its `from_bytes` rejects a value at or above the group order, and zero is a
        // valid encoding there (it yields the identity), so a check that lived only in `finalize`
        // would let a final hop skip both the parse and the identity rejection. Parsing costs a
        // scalar conversion, not the multiplication, so the final hop still saves the multiplication.
        // Parsed here, discarded, and re-parsed in `finalize`: the parse validates the KDF output on
        // *every* hop (secp256k1 rejects a value at or above the order, and zero is a valid encoding
        // there), while the raw `SharedSecret` is what the struct retains so that it is zeroized.
        E::from_bytes(b_k.as_ref())?;

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
    /// Blinding factor `b_k`, kept in its raw KDF form.
    ///
    /// Kept as a [`SharedSecret`] rather than as a parsed `E` so that it is zeroized on drop: `E` is a
    /// bare curve scalar with no `ZeroizeOnDrop`. That it parses at all is checked in the first half of
    /// the transform, on every hop, so keeping the raw bytes here does not delay a malformed-scalar
    /// rejection to whoever finalizes.
    b_k: SharedSecret,
    _d: PhantomData<E>,
}

impl<E: Scalar, G: GroupElement<E>> DeferredAlpha<E, G> {
    /// Computes the blinded alpha, the second half of the forward transform.
    ///
    /// This is the multiplication [`SharedKeys::forward_transform_deferred`] leaves out; together the
    /// two are what the eager (complete) transform used to do in one call.
    pub(crate) fn finalize(self) -> hopr_types::crypto::errors::Result<Alpha<G::AlphaLen>> {
        let b_k_checked = E::from_bytes(self.b_k.as_ref())?;
        let alpha_new = self.alpha_point.mul(&b_k_checked);

        // As in `generate`: a scalar multiple of an already-validated point is in the prime-order
        // subgroup, so only the identity has to be excluded here.
        if !alpha_new.is_valid_derived() {
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
        let generated_shares = SharedKeys::<S::E, S::G>::generate(pub_keys_alpha.clone())?;
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

    /// Known-answer test pinning the wire format of the relay transform.
    ///
    /// The expected bytes were produced by the pre-change implementation (`ff0076efaa`) from the same
    /// fixed alpha and key pair, so every value here is opaque to the implementation under test: the
    /// per-hop shared secret and the blinded alpha handed to the next hop. The round-trip test above
    /// compares the deferral against itself and so cannot see a change to the KDF context or the
    /// blinding salt; this one does, because a different salt moves these bytes. A change that broke
    /// compatibility with existing nodes would fail here.
    ///
    /// The sender's `new_shared_keys` cannot be pinned the same way - it starts from a random alpha -
    /// so the alpha below is the one the pre-change implementation emitted, used as a fixed input.
    #[cfg(feature = "ed25519")]
    #[test]
    fn the_relay_transform_matches_the_pre_deferral_known_answers() -> anyhow::Result<()> {
        use hopr_types::crypto::keypairs::OffchainKeypair;
        use hopr_types::primitive::typenum::U32;

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

    /// The pre-deferral eager forward transform, spelled out.
    ///
    /// Deliberately re-derived here from the operations in
    /// [`SharedKeys::forward_transform_deferred`] plus [`DeferredAlpha::finalize`], rather than calling
    /// either, so that the test above has an expectation that does not move when the implementation
    /// moves.
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

    /// A **nonidentity** small-order alpha must be rejected too.
    ///
    /// The identity is not the only degenerate input: a point of order 2 or 4 also makes `alpha * sk`
    /// take only one or two values regardless of `sk`, so the shared secret, and the header MAC it
    /// authenticates, is derivable by a third party. The identity-only check the code used to have
    /// (`!is_identity()`) let these through. All eight small-order points are tried, so the check is
    /// exercised against each torsion class rather than against the identity alone.
    #[cfg(feature = "ed25519")]
    #[test]
    fn a_nonidentity_small_order_alpha_is_rejected() -> anyhow::Result<()> {
        use hopr_types::crypto::primitives::{Curve25519CompressedPoint, Curve25519Point};

        let (pub_key, priv_key) = <Curve25519Point as GroupElement<Curve25519Scalar>>::random_pair();

        // The eight points of the order-8 torsion subgroup, in compressed Edwards form. The first is
        // the identity; the rest are the nonidentity classes the old guard admitted.
        const SMALL_ORDER: [[u8; 32]; 8] = [
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            ],
            [
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
}
