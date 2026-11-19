use hopr_types::crypto::errors::Result;
#[cfg(feature = "x25519")]
use hopr_types::crypto::primitives::Curve25519MontgomeryPoint;
#[cfg(any(feature = "x25519", feature = "ed25519"))]
use hopr_types::crypto::primitives::{Curve25519CompressedPoint, Curve25519Point, Curve25519Scalar};
#[cfg(any(feature = "x25519", feature = "ed25519"))]
use hopr_types::crypto::types::{ExpandedOffchainPublicKey, OffchainPublicKey};
#[cfg(feature = "secp256k1")]
use hopr_types::crypto::{
    crypto_traits::elliptic_curve::{
        self, Group,
        field::FieldBytes,
        sec1::{FromSec1Point, Sec1Point, ToSec1Point},
    },
    keypairs::ChainKeypair,
    prelude::{CryptoError, Secp256k1},
    types::PublicKey,
};

use super::shared_keys::{Alpha, GroupElement, Scalar, SphinxSuite};

/// Newtype for secp256k1 scalars — avoids Rust 2024 coherence issues
/// that arise when using transparent type aliases to `elliptic_curve::Scalar<Secp256k1>`.
#[cfg(feature = "secp256k1")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256k1Scalar(pub elliptic_curve::Scalar<Secp256k1>);

/// Newtype for secp256k1 projective points — avoids Rust 2024 coherence issues
/// that arise when using transparent type aliases to `elliptic_curve::ProjectivePoint<Secp256k1>`.
#[cfg(feature = "secp256k1")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Secp256k1Point(pub elliptic_curve::ProjectivePoint<Secp256k1>);

#[cfg(feature = "secp256k1")]
impl Default for Secp256k1Point {
    fn default() -> Self {
        Self(elliptic_curve::ProjectivePoint::<Secp256k1>::IDENTITY)
    }
}

// Forwarding impls needed by SphinxSuite trait bounds
#[cfg(feature = "secp256k1")]
impl std::ops::Mul for Secp256k1Scalar {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

#[cfg(feature = "secp256k1")]
impl<'a> std::ops::Mul<&'a Secp256k1Scalar> for Secp256k1Point {
    type Output = Self;

    fn mul(self, rhs: &'a Secp256k1Scalar) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

#[cfg(feature = "secp256k1")]
impl From<&ChainKeypair> for Secp256k1Scalar {
    fn from(value: &ChainKeypair) -> Self {
        Self(elliptic_curve::Scalar::<Secp256k1>::from(value))
    }
}

#[cfg(feature = "secp256k1")]
impl From<&PublicKey> for Secp256k1Point {
    fn from(value: &PublicKey) -> Self {
        Self(elliptic_curve::ProjectivePoint::<Secp256k1>::from(value))
    }
}

#[cfg(any(feature = "x25519", feature = "ed25519"))]
impl Scalar for Curve25519Scalar {
    fn random() -> Self {
        let bytes = hopr_types::crypto_random::random_bytes::<32>();
        Self::from_bytes(&bytes).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        hopr_types::crypto::utils::x25519_scalar_from_bytes(bytes)
    }

    fn is_zero(&self) -> bool {
        // `PartialEq` on this scalar is constant-time (`ct_eq` under the hood).
        *self == Self::ZERO
    }
}

#[cfg(feature = "secp256k1")]
impl Scalar for Secp256k1Scalar {
    fn random() -> Self {
        // Beware, this is not constant-time
        let mut rng = hopr_types::crypto_random::rng();
        let mut bytes = FieldBytes::<Secp256k1>::default();
        use elliptic_curve::PrimeField;
        use hopr_types::crypto_random::Rng;
        // Needs manual implementation due to incompatible rand crates
        // elliptic_curve::Scalar::<Secp256k1>::generate_vartime(&mut hopr_types::crypto_random::rng())

        loop {
            rng.fill_bytes(&mut bytes);
            if let Some(scalar) = elliptic_curve::Scalar::<Secp256k1>::from_repr(bytes).into() {
                return Secp256k1Scalar(scalar);
            }
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Secp256k1Scalar(hopr_types::crypto::utils::k256_scalar_from_bytes(
            bytes,
        )?))
    }

    fn is_zero(&self) -> bool {
        // `k256_scalar_from_bytes` accepts an all-zero encoding, so this is reachable from a KDF
        // output and not only from an empty value. `Scalar::is_zero` here is `ff::Field::is_zero`,
        // which is constant-time.
        self.0.is_zero().into()
    }
}

#[cfg(feature = "x25519")]
impl GroupElement<Curve25519Scalar> for Curve25519MontgomeryPoint {
    type AlphaLen = hopr_types::primitive::typenum::U32;

    fn to_alpha(&self) -> Alpha<hopr_types::primitive::typenum::U32> {
        self.0.into()
    }

    fn from_alpha(alpha: Alpha<hopr_types::primitive::typenum::U32>) -> Result<Self> {
        Ok(Curve25519MontgomeryPoint(alpha.into()))
    }

    fn generate(scalar: &Curve25519Scalar) -> Self {
        Curve25519Point::mul_base(scalar).to_montgomery()
    }

    fn is_valid(&self) -> bool {
        // `GroupElement` requires a valid element to exclude small-order points, not only the
        // identity. A Montgomery coordinate also has no sign bit, so it may decode to a point of the
        // wrong Edwards sign; that is harmless for the ladder, and it does not change whether the
        // point is small order.
        //
        // `to_edwards` returning `None` means the coordinate lies on the twist rather than the curve,
        // for any `u` with no Edwards preimage - `u = 2` and `u = -1` are two such (dalek's own
        // `montgomery_to_edwards_rejects_twist` uses both; `u = -1` is only the exceptional case for
        // the birational map, not the only one). Rejecting every `None` rather than skipping it is
        // what the first version of this check got wrong when it treated `None` as valid.
        //
        // The order-2 point is a separate case, and it is *not* `u = -1`: dalek maps `u = 0` to the
        // Edwards order-2 point, so it decodes to `Some` and is caught by the `is_small_order` test
        // below rather than by the `None` arm.
        //
        // `is_small_order` (three doublings) rather than `is_torsion_free` (a full scalar
        // multiplication by the group order) for the same reason as on the Edwards side: this runs on
        // the per-packet path, and the points that must be rejected are exactly the eight of order
        // dividing 8.
        //
        // Note that this admits *mixed-order* points, i.e. a curve point with a nonzero prime-order
        // component and a nonzero small-order one. That is enough for what the protocol does with an
        // accepted alpha: multiplying it by a nonzero scalar keeps the prime-order component nonzero,
        // so the shared point does not collapse to a small set. See `DeferredAlpha::finalize`.
        self.to_edwards(0).is_some_and(|edwards| !edwards.is_small_order())
    }

    /// A derived point is a multiple of one this crate already validated, so only the identity can be
    /// wrong: the `to_edwards` conversion a wire coordinate needs is not required for our own
    /// arithmetic, and neither is the small-order test on a value that a nonzero scalar keeps
    /// out of reach.
    fn is_valid_derived(&self) -> bool {
        use hopr_types::crypto::primitives::IsIdentity;
        !self.is_identity()
    }
}

#[cfg(feature = "ed25519")]
impl GroupElement<Curve25519Scalar> for Curve25519Point {
    type AlphaLen = hopr_types::primitive::typenum::U32;

    fn to_alpha(&self) -> Alpha<hopr_types::primitive::typenum::U32> {
        self.compress().0.into()
    }

    fn from_alpha(alpha: Alpha<hopr_types::primitive::typenum::U32>) -> Result<Self> {
        Curve25519CompressedPoint(alpha.into())
            .decompress()
            .ok_or(hopr_types::crypto::errors::CryptoError::InvalidInputValue("alpha"))
    }

    fn generate(scalar: &Curve25519Scalar) -> Self {
        Curve25519Point::mul_base(scalar)
    }

    fn is_valid(&self) -> bool {
        // The scalar is clamped (a multiple of the co-factor 8), so `generate` cannot produce a
        // small-order point - but `is_valid` is also called on points that come in from the wire, in
        // particular the packet's alpha before it is multiplied. A sender controls that byte string,
        // so the check must be on the point.
        //
        // `is_small_order` rather than `is_torsion_free`: the latter multiplies by the group order
        // (a full scalar multiplication, measurable on the per-packet path), while the only inputs
        // that must be rejected here are the eight points whose order divides 8, and
        // `is_small_order` is three doublings plus an identity test. It covers the identity too, so
        // the explicit identity term is gone.
        //
        // This deliberately still admits *mixed-order* alphas (`P = Q + T` with `Q` of prime order and
        // `T` one of the eight torsion points), which `is_torsion_free` would exclude. That is the
        // intended input policy, not an oversight:
        // - secrecy is preserved. What secrecy needs is a bound on how many candidate alphas a fixed peer's shared
        //   point can stand for. `alpha * sk = sk*Q + sk*T`, so two alphas `Q + T` and `Q + T'` give the same shared
        //   point exactly when `sk*(T - T') = O`; the torsion points plus `O` form the order-8 group and `sk*(T - T')`
        //   stays in it, so this happens only when `sk` is a multiple of `T - T'`'s order. Measured over 40 random
        //   keys, the eight torsion variants of one alpha produced 8 distinct shared points for 20 keys, 4 for 9, and 2
        //   for 11
        //   - never 1, because a clamped scalar is never zero modulo 8 (bit 2 stays free while bits
        //   0-1 are cleared). So the hop secret is exposed by a small, fixed set of candidate alphas
        //   per peer, the same small-public-set property the identity class already has - not a
        //   collapse to one value regardless of the key.
        //   (Counting distinct outputs *across* random keys, as an earlier draft did, does not show
        //   this: those outputs differ in their prime-order parts, which a single peer's knowledge
        //   does not depend on.)
        // - rejecting them would change the wire format: a peer that sends a mixed-order alpha (this implementation
        //   never does, it derives alphas from the basepoint) would go from accepted to rejected on upgrade.
        // Enforcing prime-order membership would instead cost the variable-base multiply the deferral
        // exists to avoid, so the boundary stays at the class whose secret collapses for *every* key.
        !self.is_small_order()
    }

    /// As for the Montgomery implementation: a derived point is a scalar multiple of one already
    /// validated, so the three-doubling `is_small_order` check is not needed for it - only the
    /// identity, which a multiple of the group order produces.
    fn is_valid_derived(&self) -> bool {
        use hopr_types::crypto::primitives::IsIdentity;
        !self.is_identity()
    }
}

#[cfg(feature = "secp256k1")]
impl GroupElement<Secp256k1Scalar> for Secp256k1Point {
    type AlphaLen = hopr_types::primitive::typenum::U33;

    fn to_alpha(&self) -> Alpha<hopr_types::primitive::typenum::U33> {
        debug_assert!(self.is_valid(), "to_alpha called on identity point");
        let mut ret = Alpha::<hopr_types::primitive::typenum::U33>::default();
        // Copy only if the point is not the identity, we do not care here about constant-time here.
        if !bool::from(self.0.is_identity()) {
            ret.copy_from_slice(self.0.to_affine().to_sec1_point(true).as_ref());
        }
        ret
    }

    fn from_alpha(alpha: Alpha<hopr_types::primitive::typenum::U33>) -> Result<Self> {
        Sec1Point::<Secp256k1>::from_bytes(alpha)
            .map_err(|_| CryptoError::InvalidInputValue("alpha"))
            .and_then(|ep| {
                <Secp256k1 as elliptic_curve::CurveArithmetic>::AffinePoint::from_sec1_point(&ep)
                    .into_option()
                    .ok_or(CryptoError::InvalidInputValue("alpha"))
            })
            .map(|ap| Secp256k1Point(elliptic_curve::ProjectivePoint::<Secp256k1>::from(ap)))
    }

    fn generate(scalar: &Secp256k1Scalar) -> Self {
        Secp256k1Point(elliptic_curve::ProjectivePoint::<Secp256k1>::mul_by_generator(
            &scalar.0,
        ))
    }

    fn is_valid(&self) -> bool {
        !bool::from(self.0.is_identity())
    }
}

// TODO: invert this, so that each SphinxSuite takes this as a type argument
/// Default packet block size for the Sphinx protocol.
///
/// Currently, 3248 bytes.
pub type DefaultSphinxPacketSize = hopr_types::primitive::hybrid_array::sizes::U3248;

pub use hopr_types::primitive::typenum::Unsigned;

/// Represents an instantiation of the Sphinx protocol using secp256k1 elliptic curve and `ChainKeypair`
#[cfg(feature = "secp256k1")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Secp256k1Suite;

#[cfg(feature = "secp256k1")]
impl SphinxSuite for Secp256k1Suite {
    type E = Secp256k1Scalar;
    // `ChainKeypair::Public` already carries the curve point, so expansion is the identity.
    type ExpandedPublic = PublicKey;
    type G = Secp256k1Point;
    type P = hopr_types::crypto::keypairs::ChainKeypair;
    type PRP = hopr_types::crypto::lioness::LionessBlake3ChaCha20<DefaultSphinxPacketSize>;

    fn expand_public(public_key: &PublicKey) -> Result<Self::ExpandedPublic> {
        Ok(*public_key)
    }
}

/// Represents an instantiation of the Sphinx protocol using the ed25519 curve and `OffchainKeypair`
#[cfg(feature = "ed25519")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Suite;

#[cfg(feature = "ed25519")]
impl SphinxSuite for Ed25519Suite {
    type E = Curve25519Scalar;
    type ExpandedPublic = ExpandedOffchainPublicKey;
    type G = Curve25519Point;
    type P = hopr_types::crypto::keypairs::OffchainKeypair;
    type PRP = hopr_types::crypto::lioness::LionessBlake3ChaCha20<DefaultSphinxPacketSize>;

    fn expand_public(public_key: &OffchainPublicKey) -> Result<Self::ExpandedPublic> {
        Ok(ExpandedOffchainPublicKey::try_from(public_key)?)
    }
}

/// Represents an instantiation of the Sphinx protocol using the Curve25519 curve and `OffchainKeypair`
#[cfg(feature = "x25519")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct X25519Suite;

#[cfg(feature = "x25519")]
impl SphinxSuite for X25519Suite {
    type E = Curve25519Scalar;
    // Same expansion as `Ed25519Suite`, viewed as a Montgomery point instead.
    type ExpandedPublic = ExpandedOffchainPublicKey;
    type G = Curve25519MontgomeryPoint;
    type P = hopr_types::crypto::keypairs::OffchainKeypair;
    type PRP = hopr_types::crypto::lioness::LionessBlake3ChaCha20<DefaultSphinxPacketSize>;

    fn expand_public(public_key: &OffchainPublicKey) -> Result<Self::ExpandedPublic> {
        Ok(ExpandedOffchainPublicKey::try_from(public_key)?)
    }
}

#[cfg(test)]
mod tests {
    use parameterized::parameterized;

    use super::super::shared_keys::tests::generic_sphinx_suite_test;

    #[cfg(feature = "secp256k1")]
    #[test]
    fn test_extract_key_from_group_element() {
        use hopr_types::crypto::{crypto_traits::elliptic_curve, prelude::Secp256k1};

        use super::{super::shared_keys::GroupElement, Secp256k1Point};

        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = Secp256k1Point(elliptic_curve::ProjectivePoint::<Secp256k1>::GENERATOR);

        let key = pt.extract_key("test", &salt);
        assert_eq!(
            "08112a22609819a4c698d6c92f404628ca925f3d731d53594126ffdf19ef6fa9",
            const_hex::encode(key)
        );
    }

    #[cfg(feature = "secp256k1")]
    #[test]
    fn test_expand_key_from_group_element() {
        use hopr_types::crypto::{crypto_traits::elliptic_curve, prelude::Secp256k1};

        use super::{super::shared_keys::GroupElement, Secp256k1Point};

        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = Secp256k1Point(elliptic_curve::ProjectivePoint::<Secp256k1>::GENERATOR);

        let key = pt.extract_key("test", &salt);

        assert_eq!(
            "08112a22609819a4c698d6c92f404628ca925f3d731d53594126ffdf19ef6fa9",
            const_hex::encode(key)
        );
    }

    #[cfg(feature = "secp256k1")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_secp256k1_suite(nodes: usize) -> anyhow::Result<()> {
        generic_sphinx_suite_test::<crate::Secp256k1Suite>(nodes)?;
        Ok(())
    }

    #[cfg(feature = "ed25519")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_ed25519_shared_keys(nodes: usize) -> anyhow::Result<()> {
        generic_sphinx_suite_test::<crate::Ed25519Suite>(nodes)?;
        Ok(())
    }

    #[cfg(feature = "x25519")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_montgomery_shared_keys(nodes: usize) -> anyhow::Result<()> {
        generic_sphinx_suite_test::<crate::X25519Suite>(nodes)?;
        Ok(())
    }
}

#[cfg(all(test, feature = "x25519"))]
mod montgomery_is_valid {
    use super::*;

    #[test]
    fn small_order_montgomery_points_are_rejected() {
        let small: [[u8; 32]; 4] = [
            [0u8; 32],
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            [
                236, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127,
            ],
            [
                238, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127,
            ],
        ];
        // The Montgomery ladder has no sign bit, so these come from the compressed `u`-coordinate
        // classes directly; the identity, order-2 and the two order-4 classes are covered.
        for (i, s) in small.into_iter().enumerate() {
            let p = Curve25519MontgomeryPoint(s);
            assert!(!p.is_valid(), "small-order Montgomery point {i} passed is_valid");
        }

        // The order-8 classes are most reliably reached by *projecting* the Edwards eight-torsion set
        // onto Montgomery coordinates, which also exercises the `to_montgomery`/`to_edwards` pair the
        // check relies on. Each projected point must be rejected.
        let edwards_torsion: [[u8; 32]; 8] = [
            // identity, order 2, four of order 4/8 on the main curve (canonical compressed forms)
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            [0u8; 32],
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
            [
                0xee, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
            ],
        ];
        let mut projected = std::collections::BTreeSet::new();
        for (i, bytes) in edwards_torsion.into_iter().enumerate() {
            let edwards = Curve25519CompressedPoint(bytes)
                .decompress()
                .unwrap_or_else(|| panic!("torsion fixture {i} must decompress"));
            assert!(edwards.is_small_order(), "fixture {i} must be small order");
            let montgomery = edwards.to_montgomery();
            projected.insert(montgomery.0);
            assert!(
                !montgomery.is_valid(),
                "projected torsion point {i} passed the Montgomery is_valid"
            );
        }
        // The eight Edwards torsion points project onto four distinct `u` values, because `u` has no
        // sign: P and -P collapse. Asserting the count keeps a broken fixture from silently shrinking
        // the set.
        assert_eq!(
            projected.len(),
            4,
            "the torsion subgroup must project onto four distinct u values"
        );
        let good = Curve25519MontgomeryPoint([
            9u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        assert!(good.is_valid(), "the Montgomery basepoint must be valid");

        // A successful conversion: an alpha this suite produces (a real curve point) must pass. This is
        // the case `is_valid` is called with on the packet path, and it exercises the `Some` arm.
        let (alpha, _) = <Curve25519MontgomeryPoint as GroupElement<Curve25519Scalar>>::random_pair();
        assert!(alpha.is_valid(), "a generated Montgomery alpha must be valid");

        // A failed conversion: a twist coordinate must be rejected, not skipped. `to_edwards` returns
        // `None` for `u = -1`, whose right-hand side `486660` is a non-square, so it lies on the twist
        // rather than the curve. Treating `None` as valid is what the first version of this check got
        // wrong.
        let mut u_minus_one = [0xffu8; 32];
        u_minus_one[0] = 0xec;
        u_minus_one[31] = 0x7f;
        let twist_coordinate = Curve25519MontgomeryPoint(u_minus_one);
        assert!(
            twist_coordinate.to_edwards(0).is_none(),
            "u = -1 is a twist coordinate, so it has no Edwards representative"
        );
        assert!(!twist_coordinate.is_valid(), "a twist coordinate must be invalid");

        // `u = -1` is not the only twist coordinate. `u = 2` is another (dalek's own conversion test
        // uses the pair), so the rejection must be for `None` generally, not for that special case.
        let mut u_two = [0u8; 32];
        u_two[0] = 2;
        let u_two = Curve25519MontgomeryPoint(u_two);
        assert!(
            u_two.to_edwards(0).is_none(),
            "u = 2 is also a twist coordinate, so it has no Edwards representative"
        );
        assert!(!u_two.is_valid(), "a twist coordinate must be invalid");

        // The order-2 point is reached through `u = 0`, and it takes the `Some` arm: dalek maps it to
        // the Edwards order-2 point, so it is rejected by `is_small_order` rather than by `None`.
        let order_two = Curve25519MontgomeryPoint([0u8; 32]);
        let edwards = order_two
            .to_edwards(0)
            .expect("u = 0 must map to the Edwards order-2 point");
        assert!(edwards.is_small_order(), "u = 0 must be a small-order point");
        assert!(!order_two.is_valid(), "u = 0 must be invalid");
    }
}
