use std::ops::Mul;
use curve25519_dalek::EdwardsPoint;
use curve25519_dalek::traits::IsIdentity;
use rand::{CryptoRng, RngCore};
use utils_types::traits::BinarySerializable;
use crate::errors::CryptoError::CalculationError;
use crate::errors::Result;
use crate::shared_keys::{GroupElement, GroupEncoding, Scalar, SharedKeys};
use crate::types::OffchainPublicKey;

pub type EdScalar = curve25519_dalek::Scalar;

impl Scalar for EdScalar {
    type Repr = [u8; 32];

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        Ok(EdScalar::from_bits(repr))
    }

    fn to_repr(&self) -> Self::Repr {
        self.to_bytes()
    }

    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self {
        EdScalar::random(rng)
    }
}

impl GroupElement<EdScalar> for EdwardsPoint {
    type Repr = OffchainPublicKey;

    fn to_repr(&self) -> Self::Repr {
        OffchainPublicKey::from_bytes(self.compress().as_bytes())
                .expect("free-form operation resulted in invalid public key") // must not happen
    }

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        repr.key.decompress().ok_or(CalculationError)
    }

    fn generate(scalar: &curve25519_dalek::Scalar) -> Self {
        EdwardsPoint::mul_base(scalar)
    }

    fn multiply(&self, scalar: &curve25519_dalek::Scalar) -> Self {
        self.mul(scalar)
    }

    fn is_valid(&self) -> bool {
        !self.is_small_order() && !self.is_identity()
    }
}

impl GroupEncoding for OffchainPublicKey {
    fn encode(&self) -> Box<[u8]> {
        self.to_bytes()
    }
}

/// Instantiation of Sphinx shared keys generation using Ed25519 group
pub type Ed25519SharedKeys = SharedKeys<EdScalar, EdwardsPoint>;


#[cfg(test)]
pub mod tests {
    use curve25519_dalek::EdwardsPoint;
    use crate::shared_keys::tests::generic_test_shared_keys;

    #[test]
    fn test_peerid() {

    }

    #[test]
    fn test_secp256k1_shared_keys() {
        generic_test_shared_keys::<curve25519_dalek::Scalar, EdwardsPoint>()
    }
}

