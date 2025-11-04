use std::{fmt::Formatter, marker::PhantomData};

use typenum::Unsigned;

use crate::{
    crypto_traits::{Iv, IvSizeUser, Key, KeyIvInit, KeySizeUser},
    utils::SecretValue,
};

/// AES with 128-bit key in counter-mode (with big-endian counter).
pub type Aes128Ctr = ctr::Ctr64BE<aes::Aes128>;

// Re-exports of used cryptographic primitives
pub use blake3::{Hasher as Blake3, OutputReader as Blake3Output, hash as blake3_hash};
pub use chacha20::ChaCha20;
pub use poly1305::Poly1305;
pub use sha3::{Keccak256, Sha3_256};

/// Represents a 256-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey = SecretValue<typenum::U32>;

/// Represents a 128-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey16 = SecretValue<typenum::U16>;

/// Convenience container for IV and key of a given primitive `T`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(zeroize::ZeroizeOnDrop)]
pub struct IvKey<T>(Box<[u8]>, PhantomData<T>);

impl<T: KeyIvInit> KeySizeUser for IvKey<T> {
    type KeySize = T::KeySize;
}

impl<T: KeyIvInit> IvSizeUser for IvKey<T> {
    type IvSize = T::IvSize;
}

impl<T: KeyIvInit> KeyIvInit for IvKey<T> {
    fn new(key: &Key<Self>, iv: &Iv<Self>) -> Self {
        let mut out = Vec::with_capacity(Self::SIZE);
        out.extend_from_slice(iv.as_ref());
        out.extend_from_slice(key.as_ref());
        Self(out.into_boxed_slice(), PhantomData)
    }
}

impl<T: KeyIvInit> Default for IvKey<T> {
    fn default() -> Self {
        Self(vec![0u8; Self::SIZE].into_boxed_slice(), PhantomData)
    }
}

impl<T> Clone for IvKey<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T> PartialEq for IvKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for IvKey<T> {}

impl<T: KeyIvInit> std::fmt::Debug for IvKey<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IvKey")
            .field("key", &"<redacted>")
            .field("iv", self.iv())
            .finish()
    }
}

#[allow(deprecated)] // Until the dependency updates to newer versions of `generic-array`
impl<T: KeyIvInit> IvKey<T> {
    /// Total size of the key and IV in bytes.
    pub const SIZE: usize = T::KeySize::USIZE + T::IvSize::USIZE;

    /// Returns the IV part.
    #[inline]
    pub fn iv(&self) -> &Iv<T> {
        Iv::<T>::from_slice(&self.0[0..T::IvSize::USIZE])
    }

    /// Returns IV as a mutable slice.
    #[inline]
    pub fn iv_mut(&mut self) -> &mut [u8] {
        &mut self.0[0..T::IvSize::USIZE]
    }

    /// Returns the key part.
    #[inline]
    pub fn key(&self) -> &Key<T> {
        Key::<T>::from_slice(&self.0[T::IvSize::USIZE..])
    }

    /// Returns the key as a mutable slice.
    #[inline]
    pub fn key_mut(&mut self) -> &mut [u8] {
        &mut self.0[T::IvSize::USIZE..]
    }

    /// Turn this instance into another [`KeyIvInit`] with the same IV and key sizes.
    #[inline]
    pub fn into_init<V>(self) -> V
    where
        V: KeyIvInit<KeySize = T::KeySize, IvSize = T::IvSize>,
    {
        V::new(self.key(), self.iv())
    }
}
