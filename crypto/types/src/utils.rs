use generic_array::{ArrayLength, GenericArray};
use hopr_crypto_random::random_array;
use k256::elliptic_curve::hash2curve::{ExpandMsgXmd, GroupDigest};
use k256::elliptic_curve::{Group, PrimeField};
use k256::Secp256k1;
use serde::{Deserializer, Serializer};
use sha3::Sha3_256;
use subtle::{Choice, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::errors::CryptoError;
use crate::errors::CryptoError::{CalculationError, InvalidInputValue, InvalidParameterSize};
use crate::prelude::{HalfKey, SecretKey};

/// Generates a random elliptic curve point on the secp256k1 curve (but not a point in infinity).
/// Returns the encoded secret scalar and the corresponding point.
pub(crate) fn random_group_element() -> ([u8; 32], crate::types::CurvePoint) {
    let mut scalar = k256::NonZeroScalar::from_uint(1u32.into()).unwrap();
    let mut point = k256::ProjectivePoint::IDENTITY;
    while point.is_identity().into() {
        scalar = k256::NonZeroScalar::random(&mut hopr_crypto_random::rng());
        point = k256::ProjectivePoint::GENERATOR * scalar.as_ref();
    }
    (scalar.to_bytes().into(), point.to_affine().into())
}

/// Creates X25519 secret scalar (also compatible with Ed25519 scalar) from the given bytes.
/// This function ensures the value is pre-multiplied by the curve's co-factor and already
/// reduced mod 2^255-19.
pub fn x25519_scalar_from_bytes(bytes: &[u8]) -> crate::errors::Result<curve25519_dalek::scalar::Scalar> {
    if bytes.len() == 32 {
        // Representation of the scalar is little-endian
        let mut clamped = [0u8; 32];
        clamped.copy_from_slice(&bytes[..32]);
        clamped[00] &= 0b1111_1000; // clear the 3 LSB bits (= multiply by Curve25519's co-factor)
        clamped[31] &= 0b0111_1111; // clear the 256-th bit
        clamped[31] |= 0b0100_0000; // make it 255-bit number

        Ok(curve25519_dalek::scalar::Scalar::from_bytes_mod_order(clamped))
    } else {
        Err(InvalidInputValue("bytes"))
    }
}

/// Creates secp256k1 secret scalar from the given bytes.
/// Note that this function allows zero scalars.
pub fn k256_scalar_from_bytes(bytes: &[u8]) -> crate::errors::Result<k256::Scalar> {
    Option::from(k256::Scalar::from_repr(*k256::FieldBytes::from_slice(bytes))).ok_or(InvalidInputValue("bytes"))
}

/// Sample a random secp256k1 field element that can represent a valid secp256k1 point.
/// The implementation uses `hash_to_field` function as defined in
/// `<https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-13.html#name-hashing-to-a-finite-field>`
/// The `secret` must be at least `SecretKey::LENGTH` long.
/// The `tag` parameter will be used as an additional Domain Separation Tag.
pub fn sample_secp256k1_field_element(secret: &[u8], tag: &str) -> crate::errors::Result<HalfKey> {
    if secret.len() >= SecretKey::LENGTH {
        let scalar = Secp256k1::hash_to_scalar::<ExpandMsgXmd<Sha3_256>>(
            &[secret],
            &[b"secp256k1_XMD:SHA3-256_SSWU_RO_", tag.as_bytes()],
        )
        .map_err(|_| CalculationError)?;
        Ok(HalfKey::try_from(scalar.to_bytes().as_ref())?)
    } else {
        Err(InvalidParameterSize {
            name: "secret",
            expected: SecretKey::LENGTH,
        })
    }
}

/// Represents a secret value of a fixed length that is zeroized on drop.
/// Secret values are always compared in constant time.
/// The default value is all zeroes.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretValue<L: ArrayLength>(GenericArray<u8, L>);

impl<L: ArrayLength> ConstantTimeEq for SecretValue<L> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl<L: ArrayLength> AsRef<[u8]> for SecretValue<L> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<L: ArrayLength> From<GenericArray<u8, L>> for SecretValue<L> {
    fn from(value: GenericArray<u8, L>) -> Self {
        Self(value)
    }
}

impl<'a, L: ArrayLength> From<&'a SecretValue<L>> for &'a GenericArray<u8, L> {
    fn from(value: &'a SecretValue<L>) -> Self {
        &value.0
    }
}

impl<L: ArrayLength> TryFrom<&[u8]> for SecretValue<L> {
    type Error = CryptoError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::LENGTH {
            Ok(Self(GenericArray::from_slice(value).clone()))
        } else {
            Err(InvalidInputValue("value"))
        }
    }
}

impl<L: ArrayLength> Default for SecretValue<L> {
    fn default() -> Self {
        // Ensure the default value is zeroized
        let mut ret = Self(GenericArray::default());
        ret.zeroize();
        ret
    }
}

impl<L: ArrayLength> AsMut<[u8]> for SecretValue<L> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl<L: ArrayLength> From<SecretValue<L>> for Box<[u8]> {
    fn from(value: SecretValue<L>) -> Self {
        value.as_ref().into()
    }
}

impl<L: ArrayLength> serde::Serialize for SecretValue<L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, L: ArrayLength> serde::Deserialize<'de> for SecretValue<L> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(GenericArray::deserialize(deserializer)?))
    }
}

impl<L: ArrayLength> SecretValue<L> {
    /// Length of the secret value in bytes.
    pub const LENGTH: usize = L::USIZE;

    /// Generates cryptographically strong random secret value.
    pub fn random() -> Self {
        Self(random_array())
    }
}
