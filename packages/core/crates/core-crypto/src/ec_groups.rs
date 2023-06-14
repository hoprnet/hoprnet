use curve25519_dalek::traits::IsIdentity;
use elliptic_curve::{Group, PrimeField};
use utils_types::traits::BinarySerializable;
use crate::errors::CryptoError::InvalidInputValue;
use crate::errors::Result;
use crate::shared_keys::{Alpha, GroupElement, Scalar, SphinxSuite};
use crate::types::{CurvePoint, OffchainPublicKey, PublicKey};

use elliptic_curve::ops::MulByGenerator;
use crate::random::{random_bytes, random_fill};

impl Scalar for curve25519_dalek::scalar::Scalar {
    fn random() -> Self {
        let bytes = random_bytes::<64>();
        curve25519_dalek::scalar::Scalar::from_bytes_mod_order_wide(&bytes)
    }

    fn from_bytes(sk: &[u8]) -> Result<Self> {
        Ok(curve25519_dalek::scalar::Scalar::from_bits(sk.try_into().map_err(|_| InvalidInputValue)?))
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

impl GroupElement<typenum::U32, curve25519_dalek::scalar::Scalar> for curve25519_dalek::montgomery::MontgomeryPoint {
    fn to_alpha(&self) -> Alpha<typenum::U32> {
        self.0.into()
    }

    fn from_alpha(alpha: Alpha<typenum::U32>) -> Result<Self> {
        Ok(curve25519_dalek::montgomery::MontgomeryPoint(alpha.into()))
    }

    fn generate(scalar: &curve25519_dalek::scalar::Scalar) -> Self {
        scalar * &curve25519_dalek::constants::X25519_BASEPOINT
    }

    fn is_valid(&self) -> bool {
        !self.is_identity()
    }
}

impl GroupElement<typenum::U32, curve25519_dalek::scalar::Scalar> for curve25519_dalek::edwards::EdwardsPoint {

    fn to_alpha(&self) -> Alpha<typenum::U32>{
        self.compress().0.into()
    }

    fn from_alpha(alpha: Alpha<typenum::U32>) -> Result<Self> {
        curve25519_dalek::edwards::CompressedEdwardsY(alpha.into()).decompress().ok_or(InvalidInputValue)
    }

    fn generate(scalar: &curve25519_dalek::scalar::Scalar) -> Self {
        scalar * &curve25519_dalek::constants::ED25519_BASEPOINT_POINT
    }

    fn is_valid(&self) -> bool {
        self.is_torsion_free() && !self.is_identity()
    }
}

impl GroupElement<typenum::U33, k256::Scalar> for k256::ProjectivePoint {

    fn to_alpha(&self) -> Alpha<typenum::U33> {
        let mut ret = Alpha::<typenum::U33>::default();
        ret.copy_from_slice(CurvePoint::from_affine(self.to_affine()).serialize_compressed().as_ref());
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

/// Represents a compressed serializable extension of the `PublicKey` using the secp256k1 curve.
#[derive(PartialEq, Eq, Clone)]
pub struct CompressedPublicKey(pub PublicKey);

impl BinarySerializable<'_> for CompressedPublicKey {
    const SIZE: usize = PublicKey::SIZE_COMPRESSED;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        PublicKey::from_bytes(data).map(CompressedPublicKey)
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.0.to_bytes(true)
    }
}

impl From<CompressedPublicKey> for k256::ProjectivePoint {
    fn from(value: CompressedPublicKey) -> Self {
        value.0.into()
    }
}

/// Represents an instantiation of the Sphinx protocol using secp256k1 elliptic curve and `CompressedPublicKey`
pub struct Secp256k1Suite ;

impl SphinxSuite for Secp256k1Suite {
    type E = k256::Scalar;
    type A = typenum::U33;
    type G = k256::ProjectivePoint;
    type P = CompressedPublicKey;
}

/// Represents an instantiation of the Sphinx protocol using the ed25519 curve and `OffchainPublicKey`
pub struct Ed25519Suite ;

impl SphinxSuite for Ed25519Suite {
    type E = curve25519_dalek::scalar::Scalar;
    type A = typenum::U32;
    type G = curve25519_dalek::edwards::EdwardsPoint;
    type P = OffchainPublicKey;
}

/// Represents an instantiation of the Sphinx protocol using the Curve25519 curve and `OffchainPublicKey`
pub struct X25519Suite ;

impl SphinxSuite for X25519Suite {
    type E = curve25519_dalek::scalar::Scalar;
    type A = typenum::U32;
    type G = curve25519_dalek::montgomery::MontgomeryPoint;
    type P = OffchainPublicKey;
}

#[cfg(test)]
mod tests {
    use crate::shared_keys::tests::generic_sphinx_suite_test;
    use hex_literal::hex;
    use parameterized::parameterized;
    use crate::ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite};
    use crate::parameters::SECRET_KEY_LENGTH;
    use crate::shared_keys::GroupElement;

    #[test]
    fn test_extract_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = k256::ProjectivePoint::GENERATOR;

        let key = pt.extract_key(&salt);
        assert_eq!(SECRET_KEY_LENGTH, key.len());

        let res = hex!("54bf34178075e153f481ce05b113c1530ecc45a2f1f13a3366d4389f65470de6");
        assert_eq!(res, key.as_ref());
    }

    #[test]
    fn test_expand_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = k256::ProjectivePoint::GENERATOR;

        let key = pt.expand_key(&salt);
        assert_eq!(SECRET_KEY_LENGTH, key.len());

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