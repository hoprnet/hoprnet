use curve25519_dalek::traits::IsIdentity;
use hopr_crypto_types::errors::Result;
#[cfg(feature = "secp256k1")]
use {
    elliptic_curve::{
        Group,
        ops::MulByGenerator,
        sec1::{FromEncodedPoint, ToEncodedPoint},
    },
    hopr_crypto_types::prelude::CryptoError,
    k256::{AffinePoint, EncodedPoint},
};

use crate::shared_keys::{Alpha, GroupElement, Scalar, SphinxSuite};

#[cfg(any(feature = "x25519", feature = "ed25519"))]
impl Scalar for curve25519_dalek::scalar::Scalar {
    fn random() -> Self {
        let bytes = hopr_crypto_random::random_bytes::<32>();
        Self::from_bytes(&bytes).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        hopr_crypto_types::utils::x25519_scalar_from_bytes(bytes)
    }
}

#[cfg(feature = "secp256k1")]
impl Scalar for k256::Scalar {
    fn random() -> Self {
        // Beware this is not constant time
        let mut bytes = k256::FieldBytes::default();
        loop {
            hopr_crypto_random::random_fill(&mut bytes);
            if let Ok(scalar) = Self::from_bytes(&bytes) {
                return scalar;
            }
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        hopr_crypto_types::utils::k256_scalar_from_bytes(bytes)
    }
}

#[cfg(feature = "x25519")]
impl GroupElement<curve25519_dalek::scalar::Scalar> for curve25519_dalek::montgomery::MontgomeryPoint {
    type AlphaLen = typenum::U32;

    fn to_alpha(&self) -> Alpha<typenum::U32> {
        self.0.into()
    }

    fn from_alpha(alpha: Alpha<typenum::U32>) -> Result<Self> {
        Ok(curve25519_dalek::montgomery::MontgomeryPoint(alpha.into()))
    }

    fn generate(scalar: &curve25519_dalek::scalar::Scalar) -> Self {
        scalar * curve25519_dalek::constants::X25519_BASEPOINT
    }

    fn is_valid(&self) -> bool {
        !self.is_identity()
    }
}

#[cfg(feature = "ed25519")]
impl GroupElement<curve25519_dalek::scalar::Scalar> for curve25519_dalek::edwards::EdwardsPoint {
    type AlphaLen = typenum::U32;

    fn to_alpha(&self) -> Alpha<typenum::U32> {
        self.compress().0.into()
    }

    fn from_alpha(alpha: Alpha<typenum::U32>) -> Result<Self> {
        curve25519_dalek::edwards::CompressedEdwardsY(alpha.into())
            .decompress()
            .ok_or(hopr_crypto_types::errors::CryptoError::InvalidInputValue("alpha"))
    }

    fn generate(scalar: &curve25519_dalek::scalar::Scalar) -> Self {
        scalar * curve25519_dalek::constants::ED25519_BASEPOINT_POINT
    }

    fn is_valid(&self) -> bool {
        self.is_torsion_free() && !self.is_identity()
    }
}

#[cfg(feature = "secp256k1")]
impl GroupElement<k256::Scalar> for k256::ProjectivePoint {
    type AlphaLen = typenum::U33;

    fn to_alpha(&self) -> Alpha<typenum::U33> {
        let mut ret = Alpha::<typenum::U33>::default();
        ret.copy_from_slice(self.to_affine().to_encoded_point(true).as_ref());
        ret
    }

    fn from_alpha(alpha: Alpha<typenum::U33>) -> Result<Self> {
        EncodedPoint::from_bytes(&alpha)
            .map_err(|_| CryptoError::InvalidInputValue("alpha"))
            .and_then(|ep| {
                AffinePoint::from_encoded_point(&ep)
                    .into_option()
                    .ok_or(CryptoError::InvalidInputValue("alpha"))
            })
            .map(k256::ProjectivePoint::from)
    }

    fn generate(scalar: &k256::Scalar) -> Self {
        k256::ProjectivePoint::mul_by_generator(scalar)
    }

    fn is_valid(&self) -> bool {
        self.is_identity().unwrap_u8() == 0
    }
}

/// Represents an instantiation of the Sphinx protocol using secp256k1 elliptic curve and `ChainKeypair`
#[cfg(feature = "secp256k1")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Secp256k1Suite;

#[cfg(feature = "secp256k1")]
impl SphinxSuite for Secp256k1Suite {
    type E = k256::Scalar;
    type G = k256::ProjectivePoint;
    type P = hopr_crypto_types::keypairs::ChainKeypair;
    type PRP = hopr_crypto_types::lioness::LionessBlake3ChaCha20<typenum::U1022>;
}

/// Represents an instantiation of the Sphinx protocol using the ed25519 curve and `OffchainKeypair`
#[cfg(feature = "ed25519")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Suite;

#[cfg(feature = "ed25519")]
impl SphinxSuite for Ed25519Suite {
    type E = curve25519_dalek::scalar::Scalar;
    type G = curve25519_dalek::edwards::EdwardsPoint;
    type P = hopr_crypto_types::keypairs::OffchainKeypair;
    type PRP = hopr_crypto_types::lioness::LionessBlake3ChaCha20<typenum::U1022>;
}

/// Represents an instantiation of the Sphinx protocol using the Curve25519 curve and `OffchainKeypair`
#[cfg(feature = "x25519")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct X25519Suite;

#[cfg(feature = "x25519")]
impl SphinxSuite for X25519Suite {
    type E = curve25519_dalek::scalar::Scalar;
    type G = curve25519_dalek::montgomery::MontgomeryPoint;
    type P = hopr_crypto_types::keypairs::OffchainKeypair;
    type PRP = hopr_crypto_types::lioness::LionessBlake3ChaCha20<typenum::U1022>;
}

#[cfg(test)]
mod tests {
    use parameterized::parameterized;

    use crate::shared_keys::tests::generic_sphinx_suite_test;

    #[cfg(feature = "secp256k1")]
    #[test]
    fn test_extract_key_from_group_element() {
        use crate::shared_keys::GroupElement;

        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = k256::ProjectivePoint::GENERATOR;

        let key = pt.extract_key("test", &salt);
        assert_eq!(
            "08112a22609819a4c698d6c92f404628ca925f3d731d53594126ffdf19ef6fa9",
            hex::encode(key)
        );
    }

    #[cfg(feature = "secp256k1")]
    #[test]
    fn test_expand_key_from_group_element() {
        use crate::shared_keys::GroupElement;

        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = k256::ProjectivePoint::GENERATOR;

        let key = pt.extract_key("test", &salt);

        assert_eq!(
            "08112a22609819a4c698d6c92f404628ca925f3d731d53594126ffdf19ef6fa9",
            hex::encode(key)
        );
    }

    #[cfg(feature = "secp256k1")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_secp256k1_suite(nodes: usize) {
        generic_sphinx_suite_test::<crate::ec_groups::Secp256k1Suite>(nodes)
    }

    #[cfg(feature = "ed25519")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_ed25519_shared_keys(nodes: usize) {
        generic_sphinx_suite_test::<crate::ec_groups::Ed25519Suite>(nodes)
    }

    #[cfg(feature = "x25519")]
    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_montgomery_shared_keys(nodes: usize) {
        generic_sphinx_suite_test::<crate::ec_groups::X25519Suite>(nodes)
    }
}
