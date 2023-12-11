use crate::errors::CryptoError::InvalidInputValue;
use crate::errors::Result;
use crate::shared_keys::{Alpha, GroupElement, Scalar, SphinxSuite};
use crate::types::CurvePoint;
use curve25519_dalek::traits::IsIdentity;
use elliptic_curve::{Group, PrimeField};
use utils_types::traits::BinarySerializable;

use crate::keypairs::{ChainKeypair, OffchainKeypair};
use crate::random::{random_bytes, random_fill};
use elliptic_curve::ops::MulByGenerator;

impl Scalar for curve25519_dalek::scalar::Scalar {
    fn random() -> Self {
        let bytes = random_bytes::<32>();
        Self::from_bytes(&bytes).unwrap()
    }

    fn from_bytes(sk: &[u8]) -> Result<Self> {
        if sk.len() == 32 {
            // Representation of the scalar is little-endian
            let mut clamped = [0u8; 32];
            clamped.copy_from_slice(&sk[..32]);
            clamped[00] &= 0b1111_1000; // clear the 3 LSB bits (= multiply by Curve25519's co-factor)
            clamped[31] &= 0b0111_1111; // clear the 256-th bit
            clamped[31] |= 0b0100_0000; // make it 255-bit number

            Ok(curve25519_dalek::scalar::Scalar::from_bytes_mod_order(clamped))
        } else {
            Err(InvalidInputValue)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.to_bytes().into()
    }
}

impl Scalar for k256::Scalar {
    fn random() -> Self {
        // Beware this is not constant time
        let mut bytes = k256::FieldBytes::default();
        loop {
            random_fill(&mut bytes);
            if let Ok(scalar) = Self::from_bytes(&bytes) {
                return scalar;
            }
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(k256::Scalar::from_repr(*k256::FieldBytes::from_slice(bytes)).unwrap())
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let ret = self.to_bytes();
        Box::<[u8]>::from(ret.as_slice())
    }
}

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

impl GroupElement<curve25519_dalek::scalar::Scalar> for curve25519_dalek::edwards::EdwardsPoint {
    type AlphaLen = typenum::U32;

    fn to_alpha(&self) -> Alpha<typenum::U32> {
        self.compress().0.into()
    }

    fn from_alpha(alpha: Alpha<typenum::U32>) -> Result<Self> {
        curve25519_dalek::edwards::CompressedEdwardsY(alpha.into())
            .decompress()
            .ok_or(InvalidInputValue)
    }

    fn generate(scalar: &curve25519_dalek::scalar::Scalar) -> Self {
        scalar * curve25519_dalek::constants::ED25519_BASEPOINT_POINT
    }

    fn is_valid(&self) -> bool {
        self.is_torsion_free() && !self.is_identity()
    }
}

impl GroupElement<k256::Scalar> for k256::ProjectivePoint {
    type AlphaLen = typenum::U33;

    fn to_alpha(&self) -> Alpha<typenum::U33> {
        let mut ret = Alpha::<typenum::U33>::default();
        ret.copy_from_slice(CurvePoint::from(self.to_affine()).serialize_compressed().as_ref());
        ret
    }

    fn from_alpha(alpha: Alpha<typenum::U33>) -> Result<Self> {
        CurvePoint::from_bytes(&alpha)
            .map(|c| c.to_projective_point())
            .map_err(|_| InvalidInputValue)
    }

    fn generate(scalar: &k256::Scalar) -> Self {
        k256::ProjectivePoint::mul_by_generator(scalar)
    }

    fn is_valid(&self) -> bool {
        self.is_identity().unwrap_u8() == 0
    }
}

/// Represents an instantiation of the Sphinx protocol using secp256k1 elliptic curve and `ChainKeypair`
pub struct Secp256k1Suite;

impl SphinxSuite for Secp256k1Suite {
    type P = ChainKeypair;
    type E = k256::Scalar;
    type G = k256::ProjectivePoint;
}

/// Represents an instantiation of the Sphinx protocol using the ed25519 curve and `OffchainKeypair`
pub struct Ed25519Suite;

impl SphinxSuite for Ed25519Suite {
    type P = OffchainKeypair;
    type E = curve25519_dalek::scalar::Scalar;
    type G = curve25519_dalek::edwards::EdwardsPoint;
}

/// Represents an instantiation of the Sphinx protocol using the Curve25519 curve and `OffchainKeypair`
pub struct X25519Suite;

impl SphinxSuite for X25519Suite {
    type P = OffchainKeypair;
    type E = curve25519_dalek::scalar::Scalar;
    type G = curve25519_dalek::montgomery::MontgomeryPoint;
}

#[cfg(test)]
mod tests {
    use crate::ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite};
    use crate::shared_keys::tests::generic_sphinx_suite_test;
    use crate::shared_keys::GroupElement;
    use hex_literal::hex;
    use parameterized::parameterized;

    #[test]
    fn test_extract_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = k256::ProjectivePoint::GENERATOR;

        let key = pt.extract_key(&salt);

        let res = hex!("54bf34178075e153f481ce05b113c1530ecc45a2f1f13a3366d4389f65470de6");
        assert_eq!(res, key.as_ref());
    }

    #[test]
    fn test_expand_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = k256::ProjectivePoint::GENERATOR;

        let key = pt.expand_key(&salt);

        let res = hex!("d138d9367474911f7124b95be844d2f8a6d34e962694e37e8717bdbd3c15690b");
        assert_eq!(res, key.as_ref());
    }

    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_secp256k1_suite(nodes: usize) {
        generic_sphinx_suite_test::<Secp256k1Suite>(nodes)
    }

    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_ed25519_shared_keys(nodes: usize) {
        generic_sphinx_suite_test::<Ed25519Suite>(nodes)
    }

    #[parameterized(nodes = {4, 3, 2, 1})]
    fn test_montgomery_shared_keys(nodes: usize) {
        generic_sphinx_suite_test::<X25519Suite>(nodes)
    }
}
