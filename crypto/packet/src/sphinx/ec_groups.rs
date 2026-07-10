use hopr_types::crypto::errors::Result;
#[cfg(feature = "x25519")]
use hopr_types::crypto::primitives::Curve25519MontgomeryPoint;
#[cfg(any(feature = "x25519", feature = "ed25519"))]
use hopr_types::crypto::primitives::{Curve25519CompressedPoint, Curve25519Point, Curve25519Scalar, IsIdentity};
#[cfg(feature = "secp256k1")]
use {
    hopr_types::crypto::crypto_traits::elliptic_curve::{
        self, AffinePoint, Group,
        field::FieldBytes,
        sec1::{FromSec1Point, Sec1Point, ToSec1Point},
    },
    hopr_types::crypto::prelude::{CryptoError, Secp256k1},
};

use super::shared_keys::{Alpha, GroupElement, Scalar, SphinxSuite};

#[cfg(any(feature = "x25519", feature = "ed25519"))]
impl Scalar for Curve25519Scalar {
    fn random() -> Self {
        let bytes = hopr_types::crypto_random::random_bytes::<32>();
        Self::from_bytes(&bytes).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        hopr_types::crypto::utils::x25519_scalar_from_bytes(bytes)
    }
}

#[cfg(feature = "secp256k1")]
impl Scalar for elliptic_curve::Scalar<Secp256k1> {
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
                return scalar;
            }
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        hopr_types::crypto::utils::k256_scalar_from_bytes(bytes)
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
        use IsIdentity;
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
        // Ed25519 scalars always come clamped (pre-multiplied by the curve's co-factor)
        // and therefore cannot result into points of small order.
        // See `x25519_scalar_from_bytes` for more details.
        use IsIdentity;
        !self.is_identity()
    }
}

#[cfg(feature = "secp256k1")]
impl GroupElement<elliptic_curve::Scalar<Secp256k1>> for elliptic_curve::ProjectivePoint<Secp256k1> {
    type AlphaLen = hopr_types::primitive::typenum::U33;

    fn to_alpha(&self) -> Alpha<hopr_types::primitive::typenum::U33> {
        let mut ret = Alpha::<hopr_types::primitive::typenum::U33>::default();
        // Copy only if the point is not the identity, we do not care here about constant-time here.
        if !bool::from(self.is_identity()) {
            ret.copy_from_slice(self.to_affine().to_sec1_point(true).as_ref());
        }
        ret
    }

    fn from_alpha(alpha: Alpha<hopr_types::primitive::typenum::U33>) -> Result<Self> {
        Sec1Point::<Secp256k1>::from_bytes(alpha)
            .map_err(|_| CryptoError::InvalidInputValue("alpha"))
            .and_then(|ep| {
                AffinePoint::from_sec1_point(&ep)
                    .into_option()
                    .ok_or(CryptoError::InvalidInputValue("alpha"))
            })
            .map(elliptic_curve::ProjectivePoint::<Secp256k1>::from)
    }

    fn generate(scalar: &elliptic_curve::Scalar<Secp256k1>) -> Self {
        elliptic_curve::ProjectivePoint::<Secp256k1>::mul_by_generator(scalar)
    }

    fn is_valid(&self) -> bool {
        !bool::from(self.is_identity())
    }
}

// TODO: invert this, so that each SphinxSuite takes this as a type argument
/// Default packet block size for the Sphinx protocol.
///
/// Currently, 1040 bytes.
pub type DefaultSphinxPacketSize = hopr_types::primitive::hybrid_array::sizes::U1040;

pub use hopr_types::primitive::typenum::Unsigned;

/// Represents an instantiation of the Sphinx protocol using secp256k1 elliptic curve and `ChainKeypair`
#[cfg(feature = "secp256k1")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Secp256k1Suite;

#[cfg(feature = "secp256k1")]
impl SphinxSuite for Secp256k1Suite {
    type E = elliptic_curve::Scalar<Secp256k1>;
    type G = elliptic_curve::ProjectivePoint<Secp256k1>;
    type P = hopr_types::crypto::keypairs::ChainKeypair;
    type PRP = hopr_types::crypto::lioness::LionessBlake3ChaCha20<DefaultSphinxPacketSize>;
}

/// Represents an instantiation of the Sphinx protocol using the ed25519 curve and `OffchainKeypair`
#[cfg(feature = "ed25519")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Suite;

#[cfg(feature = "ed25519")]
impl SphinxSuite for Ed25519Suite {
    type E = Curve25519Scalar;
    type G = Curve25519Point;
    type P = hopr_types::crypto::keypairs::OffchainKeypair;
    type PRP = hopr_types::crypto::lioness::LionessBlake3ChaCha20<DefaultSphinxPacketSize>;
}

/// Represents an instantiation of the Sphinx protocol using the Curve25519 curve and `OffchainKeypair`
#[cfg(feature = "x25519")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct X25519Suite;

#[cfg(feature = "x25519")]
impl SphinxSuite for X25519Suite {
    type E = Curve25519Scalar;
    type G = Curve25519MontgomeryPoint;
    type P = hopr_types::crypto::keypairs::OffchainKeypair;
    type PRP = hopr_types::crypto::lioness::LionessBlake3ChaCha20<DefaultSphinxPacketSize>;
}

#[cfg(test)]
mod tests {
    use parameterized::parameterized;

    use super::super::shared_keys::tests::generic_sphinx_suite_test;

    #[cfg(feature = "secp256k1")]
    #[test]
    fn test_extract_key_from_group_element() {
        use hopr_types::crypto::{crypto_traits::elliptic_curve, prelude::Secp256k1};

        use super::super::shared_keys::GroupElement;

        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = elliptic_curve::ProjectivePoint::<Secp256k1>::GENERATOR;

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

        use super::super::shared_keys::GroupElement;

        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = elliptic_curve::ProjectivePoint::<Secp256k1>::GENERATOR;

        let key = pt.extract_key("test", &salt);

        assert_eq!(
            "08112a22609819a4c698d6c92f404628ca925f3d731d53594126ffdf19ef6fa9",
            const_hex::encode(key)
        );
    }

    #[cfg(feature = "secp256k1")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_secp256k1_suite(nodes: usize) {
        generic_sphinx_suite_test::<super::Secp256k1Suite>(nodes)
    }

    #[cfg(feature = "ed25519")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_ed25519_shared_keys(nodes: usize) {
        generic_sphinx_suite_test::<crate::Ed25519Suite>(nodes)
    }

    #[cfg(feature = "x25519")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_montgomery_shared_keys(nodes: usize) {
        generic_sphinx_suite_test::<super::X25519Suite>(nodes)
    }
}
