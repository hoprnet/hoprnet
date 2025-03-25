use std::fmt::Formatter;
use typenum::Unsigned;

use crate::crypto_traits::{Iv, IvSizeUser, Key, KeyIvInit, KeySizeUser};
use crate::utils::SecretValue;

/// AES with 128-bit key in counter-mode (with big-endian counter).
pub type Aes128Ctr = ctr::Ctr64BE<aes::Aes128>;

// Re-exports of used cryptographic primitives
pub use blake2::Blake2s256;
pub use chacha20::ChaCha20;
pub use poly1305::Poly1305;
pub use sha3::Keccak256;
pub use sha3::Sha3_256;

/// Represents a 256-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey = SecretValue<typenum::U32>;

/// Represents a 128-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey16 = SecretValue<typenum::U16>;

/// Convenience container for IV and key of a given primitive `T`.
pub struct IvKey<T: KeyIvInit>(pub Iv<T>, pub Key<T>);

impl<T: KeyIvInit> KeySizeUser for IvKey<T> {
    type KeySize = T::KeySize;
}

impl<T: KeyIvInit> IvSizeUser for IvKey<T> {
    type IvSize = T::IvSize;
}

impl<T: KeyIvInit> KeyIvInit for IvKey<T> {
    fn new(key: &Key<Self>, iv: &Iv<Self>) -> Self {
        Self(iv.clone(), key.clone())
    }
}

impl<T: KeyIvInit> From<IvKey<T>> for (Iv<T>, Key<T>) {
    fn from(value: IvKey<T>) -> Self {
        (value.0, value.1)
    }
}

impl<T: KeyIvInit> Default for IvKey<T> {
    fn default() -> Self {
        Self(Iv::<T>::default(), Key::<T>::default())
    }
}

impl<T: KeyIvInit> Clone for IvKey<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T: KeyIvInit> std::fmt::Debug for IvKey<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IvKey")
            .field("key", &"<redacted>")
            .field("iv", &self.0)
            .finish()
    }
}

impl<T: KeyIvInit> PartialEq for IvKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<T: KeyIvInit> Eq for IvKey<T> {}

impl<T: KeyIvInit> IvKey<T> {
    /// Total size of the key and IV in bytes.
    pub const SIZE: usize = T::KeySize::USIZE + T::IvSize::USIZE;

    /// Turn this instance into another [`KeyIvInit`] with the same IV and key sizes.
    pub fn into_init<V>(self) -> V
    where
        V: KeyIvInit<KeySize = T::KeySize, IvSize = T::IvSize>,
    {
        V::new(&self.1, &self.0)
    }
}
