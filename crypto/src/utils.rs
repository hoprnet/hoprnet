use crate::errors::CryptoError;
use crate::errors::CryptoError::InvalidInputValue;
use crate::random::random_array;
use generic_array::{ArrayLength, GenericArray};
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

/// Convenience function to efficiently copy slices of unequal sizes.
#[allow(dead_code)]
pub fn copy_nonequal(target: &mut [u8], source: &[u8]) {
    let sz = target.len().min(source.len());
    target[0..sz].copy_from_slice(&source[0..sz]);
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
