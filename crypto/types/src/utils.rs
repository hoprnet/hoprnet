use crate::errors::CryptoError;
use crate::errors::CryptoError::InvalidInputValue;
use generic_array::{ArrayLength, GenericArray};
use hopr_crypto_random::random_array;
use k256::elliptic_curve::{Group, PrimeField};
use subtle::{Choice, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Convenience method to XOR one slice onto other.
pub fn xor_inplace(a: &mut [u8], b: &[u8]) {
    let bound = a.len().min(b.len());

    // TODO: use portable_simd here
    for i in 0..bound {
        a[i] ^= b[i];
    }
}

/// Generates a random elliptic curve point on secp256k1 curve (but not a point in infinity).
/// Returns the encoded secret scalar and the corresponding point.
pub(crate) fn random_group_element() -> ([u8; 32], crate::types::CurvePoint) {
    let mut scalar = k256::NonZeroScalar::from_uint(1u32.into()).unwrap();
    let mut point = k256::ProjectivePoint::IDENTITY;
    while point.is_identity().into() {
        scalar = k256::NonZeroScalar::random(&mut hopr_crypto_random::OsRng);
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
        Err(InvalidInputValue)
    }
}

/// Creates secp256k1 secret scalar from the given bytes.
/// Note that this function allows zero scalars.
pub fn k256_scalar_from_bytes(bytes: &[u8]) -> crate::errors::Result<k256::Scalar> {
    Ok(Option::from(k256::Scalar::from_repr(*k256::FieldBytes::from_slice(bytes))).ok_or(InvalidInputValue)?)
}

/// Represents a secret value of a fixed length that is zeroized on drop.
/// Secret values are always compared in constant time.
/// The default value is all zeroes.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretValue<L: ArrayLength<u8>>(GenericArray<u8, L>);

impl<L: ArrayLength<u8>> ConstantTimeEq for SecretValue<L> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.as_ref().ct_eq(other.0.as_ref())
    }
}

impl<L: ArrayLength<u8>> AsRef<[u8]> for SecretValue<L> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<L: ArrayLength<u8>> From<GenericArray<u8, L>> for SecretValue<L> {
    fn from(value: GenericArray<u8, L>) -> Self {
        Self(value)
    }
}

impl<'a, L: ArrayLength<u8>> From<&'a SecretValue<L>> for &'a GenericArray<u8, L> {
    fn from(value: &'a SecretValue<L>) -> Self {
        &value.0
    }
}

impl<L: ArrayLength<u8>> TryFrom<&[u8]> for SecretValue<L> {
    type Error = CryptoError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::LENGTH {
            Ok(Self(GenericArray::from_slice(value).clone()))
        } else {
            Err(InvalidInputValue)
        }
    }
}

impl<L: ArrayLength<u8>> Default for SecretValue<L> {
    fn default() -> Self {
        // Ensure the default value is zeroized
        let mut ret = Self(GenericArray::default());
        ret.zeroize();
        ret
    }
}

impl<L: ArrayLength<u8>> AsMut<[u8]> for SecretValue<L> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl<L: ArrayLength<u8>> From<SecretValue<L>> for Box<[u8]> {
    fn from(value: SecretValue<L>) -> Self {
        value.as_ref().into()
    }
}

impl<L: ArrayLength<u8>> SecretValue<L> {
    /// Length of the secret value in bytes.
    pub const LENGTH: usize = L::USIZE;

    /// Generates cryptographically strong random secret value.
    pub fn random() -> Self {
        Self(random_array())
    }
}
