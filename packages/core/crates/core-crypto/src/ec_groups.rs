use curve25519_dalek::traits::IsIdentity;
use elliptic_curve::{Field, Group, PrimeField};
use libp2p_identity::PeerId;
use utils_types::traits::{BinarySerializable, PeerIdLike};
use crate::errors::CryptoError::InvalidInputValue;
use crate::errors::Result;
use crate::shared_keys::{Alpha, GroupElement, Scalar, SharedKeys};
use crate::types::{CurvePoint, PublicKey };

use curve25519_dalek as dalek;
use k256::FieldBytes;
use rand::{CryptoRng, RngCore};
use crate::random::random_bytes;

impl Scalar for dalek::scalar::Scalar {
    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self {
        let bytes = random_bytes::<64>();
        dalek::scalar::Scalar::from_bytes_mod_order_wide(&bytes)
    }

    fn from_bytes(sk: &[u8]) -> Result<Self> {
        dalek::scalar::Scalar::from_bytes(sk)
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Scalar for k256::Scalar {
    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self {
        <k256::Scalar as Field>::random(rng)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(k256::Scalar::from_repr(*FieldBytes::from_slice(bytes)).unwrap())
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl GroupElement<32, dalek::scalar::Scalar> for dalek::montgomery::MontgomeryPoint {
    fn to_alpha(&self) -> Alpha<32> {
        self.0.clone()
    }

    fn from_alpha(alpha: Alpha<32>) -> Result<Self> {
        Ok(dalek::montgomery::MontgomeryPoint(alpha))
    }

    fn from_peerid(peer_id: &PeerId) -> Result<Self> {
        let mh = peer_id.as_ref();
        if mh.code() == 0 {
            let value = &mh.digest()[4..];
            Ok(dalek::montgomery::MontgomeryPoint(value.try_into().map_err(|_| InvalidInputValue)?))
        } else {
            Err(InvalidInputValue)
        }
    }

    fn generate(scalar: &dalek::scalar::Scalar) -> Self {
        scalar * &curve25519_dalek::constants::X25519_BASEPOINT
    }

    fn is_valid(&self) -> bool {
        !self.is_identity()
    }
}

impl GroupElement<32, dalek::scalar::Scalar> for dalek::edwards::EdwardsPoint {
    fn to_alpha(&self) -> Alpha<32> {
        self.compress().0.clone()
    }

    fn from_alpha(alpha: Alpha<32>) -> Result<Self> {
        dalek::edwards::CompressedEdwardsY(alpha).decompress().ok_or(InvalidInputValue)
    }

    fn from_peerid(peer_id: &PeerId) -> Result<Self> {
        let mh = peer_id.as_ref();
        if mh.code() == 0 {
            let value = &mh.digest()[4..];
            dalek::edwards::CompressedEdwardsY(value.try_into().map_err(|_| InvalidInputValue)?)
                .decompress()
                .ok_or(InvalidInputValue)
        } else {
            Err(InvalidInputValue)
        }
    }

    fn generate(scalar: &dalek::scalar::Scalar) -> Self {
        scalar * &dalek::constants::ED25519_BASEPOINT_POINT
    }

    fn is_valid(&self) -> bool {
        self.is_torsion_free() && !self.is_identity()
    }
}

/// Secp256k1 additive group (via projective coordinates) represented as public keys
impl GroupElement<{CurvePoint::SIZE_COMPRESSED}, k256::Scalar> for k256::ProjectivePoint {

    fn to_alpha(&self) -> Alpha<{CurvePoint::SIZE_COMPRESSED}> {
        CurvePoint::from_affine(self.to_affine()).serialize_compressed().as_ref().try_into().unwrap()
    }

    fn from_alpha(alpha: Alpha<{CurvePoint::SIZE_COMPRESSED}>) -> Result<Self> {
        CurvePoint::from_bytes(&alpha)
            .map(|c| c.to_projective_point())
            .map_err(|_| InvalidInputValue)
    }

    fn from_peerid(peer_id: &PeerId) -> Result<Self> {
        PublicKey::from_peerid(peer_id).map(|pk| CurvePoint::from(pk)
            .to_projective_point())
            .map_err(|_| InvalidInputValue)
    }

    fn generate(scalar: &k256::Scalar) -> Self {
        k256::ProjectivePoint::GENERATOR * scalar
    }

    fn is_valid(&self) -> bool {
        self.is_identity().unwrap_u8() == 0
    }
}

// Instantiation of Sphinx shared keys generation using different EC groups

pub type Secp256k1SharedKeys = SharedKeys<k256::Scalar, {CurvePoint::SIZE_COMPRESSED}, k256::ProjectivePoint>;
pub type X25519SharedKeys = SharedKeys<dalek::scalar::Scalar, 32, dalek::montgomery::MontgomeryPoint>;
pub type Ed25519SharedKeys = SharedKeys<dalek::scalar::Scalar, 32, dalek::edwards::EdwardsPoint>;

#[cfg(test)]
mod tests {
    use crate::shared_keys::tests::generic_test_shared_keys;
    use curve25519_dalek as dalek;
    use hex_literal::hex;
    use crate::parameters::SECRET_KEY_LENGTH;
    use crate::shared_keys::GroupElement;
    use crate::types::CurvePoint;

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

    #[test]
    fn test_secp256k1_shared_keys() {
        generic_test_shared_keys::<k256::Scalar, {CurvePoint::SIZE_COMPRESSED}, k256::ProjectivePoint>()
    }

    #[test]
    fn test_ed25519_shared_keys() {
        generic_test_shared_keys::<dalek::scalar::Scalar, 32, dalek::edwards::EdwardsPoint>()
    }

    #[test]
    fn test_montgomery_shared_keys() {
        generic_test_shared_keys::<dalek::scalar::Scalar, 32, dalek::montgomery::MontgomeryPoint>()
    }
}