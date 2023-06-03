use std::ops::Mul;
use curve25519_dalek::EdwardsPoint;
use curve25519_dalek::traits::IsIdentity;
use rand::{CryptoRng, RngCore};
use k256::{NonZeroScalar, ProjectivePoint};
use hkdf::SimpleHkdf;
use blake2::Blake2s256;
use elliptic_curve::Group;
use utils_types::traits::{BinarySerializable, PeerIdLike};
use crate::errors::CryptoError::{CalculationError, InvalidSecretScalar};
use crate::errors::Result;
use crate::parameters::SECRET_KEY_LENGTH;
use crate::shared_keys::{GroupElement, GroupEncoding, Scalar, SharedKeys};
use crate::types::{CurvePoint, OffchainPublicKey, PublicKey, SecretKey};

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

/// Non-zero scalars in the underlying field for Secp256k1
impl Scalar for NonZeroScalar {
    type Repr = [u8; 32];

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        NonZeroScalar::try_from(repr.as_slice()).map_err(|_| InvalidSecretScalar)
    }

    fn to_repr(&self) -> Self::Repr {
        let mut ret = [0u8; 32];
        ret.copy_from_slice(self.to_bytes().as_slice());
        ret
    }

    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self {
        NonZeroScalar::random(rng)
    }
}

impl GroupEncoding for PublicKey {
    fn encode(&self) -> Box<[u8]> {
        self.to_bytes(true)
    }
}

/// Secp256k1 additive group (via projective coordinates) represented as public keys
impl GroupElement<NonZeroScalar> for ProjectivePoint {
    type Repr = PublicKey;

    fn to_repr(&self) -> Self::Repr {
        PublicKey::try_from(CurvePoint::from_affine(self.to_affine()))
            .expect("group element does not represent a valid public key")
    }

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        Ok(CurvePoint::from(repr).to_projective_point())
    }

    fn generate(scalar: &NonZeroScalar) -> Self {
        ProjectivePoint::GENERATOR * scalar.as_ref()
    }

    fn multiply(&self, scalar: &NonZeroScalar) -> Self {
        Mul::mul(self, scalar.as_ref())
    }

    fn is_valid(&self) -> bool {
        self.is_identity().unwrap_u8() == 0
    }
}

/// Instantiation of Sphinx shared keys generation using Secp256k1 group
pub type Secp256k1SharedKeys = SharedKeys<NonZeroScalar, ProjectivePoint>;
