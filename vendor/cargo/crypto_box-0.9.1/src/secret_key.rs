use crate::{PublicKey, KEY_SIZE};
use core::{
    array::TryFromSliceError,
    fmt::{self, Debug},
};
use curve25519_dalek::{
    scalar::{clamp_integer, Scalar},
    MontgomeryPoint,
};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

#[cfg(feature = "rand_core")]
use aead::rand_core::CryptoRngCore;

#[cfg(feature = "seal")]
use {
    crate::{get_seal_nonce, SalsaBox},
    aead::Aead,
    alloc::vec::Vec,
};

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

/// A `crypto_box` secret key.
#[derive(Clone)]
pub struct SecretKey {
    pub(crate) bytes: [u8; KEY_SIZE],
    pub(crate) scalar: Scalar,
}

impl SecretKey {
    /// Initialize [`SecretKey`] from a byte array.
    #[inline]
    pub fn from_bytes(bytes: [u8; KEY_SIZE]) -> Self {
        let scalar = Scalar::from_bytes_mod_order(clamp_integer(bytes));
        Self { bytes, scalar }
    }

    /// Initialize [`SecretKey`] from a byte slice.
    ///
    /// Returns [`TryFromSliceError`] if the slice length is not exactly equal
    /// to [`KEY_SIZE`].
    pub fn from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
        slice.try_into().map(Self::from_bytes)
    }

    /// Generate a random [`SecretKey`].
    #[cfg(feature = "rand_core")]
    pub fn generate(csprng: &mut impl CryptoRngCore) -> Self {
        let mut bytes = [0u8; KEY_SIZE];
        csprng.fill_bytes(&mut bytes);
        bytes.into()
    }

    /// Get the [`PublicKey`] which corresponds to this [`SecretKey`]
    pub fn public_key(&self) -> PublicKey {
        PublicKey(MontgomeryPoint::mul_base(&self.scalar))
    }

    /// Serialize [`SecretKey`] to bytes.
    ///
    /// # ⚠️Warning
    ///
    /// The serialized bytes are secret key material. Please treat them with
    /// the care they deserve!
    ///
    /// # `Scalar` conversion notes
    ///
    /// If you are using the `From<Scalar>` impl on [`SecretKey`] (as opposed
    /// to using [`SecretKey::from_bytes`] or one of the other methods that
    /// decodes a secret key from bytes), this method will return the same
    /// value as `Scalar::to_bytes`, which may reflect "clamping" if it was
    /// applied to the original `Scalar`.
    ///
    /// In such cases, it may be undesirable to call this method, since such a
    /// value may not reflect the original scalar prior to clamping. We suggest
    /// you don't call this method when using `From<Scalar>` unless you know
    /// what you're doing.
    ///
    /// Calling [`SecretKey::to_scalar`] can be used to safely round-trip the
    /// scalar value in such cases.
    pub fn to_bytes(&self) -> [u8; KEY_SIZE] {
        self.bytes
    }

    /// Obtain the inner [`Scalar`] value of this [`SecretKey`].
    ///
    /// # ⚠️Warning
    ///
    /// This value is key material. Please treat it with the care it deserves!
    pub fn to_scalar(&self) -> Scalar {
        self.scalar
    }

    /// Implementation of `crypto_box_seal_open` function from [libsodium "sealed boxes"].
    ///
    /// Sealed boxes are designed to anonymously send messages to a recipient given their public key.
    ///
    /// [libsodium "sealed boxes"]: https://doc.libsodium.org/public-key_cryptography/sealed_boxes
    #[cfg(feature = "seal")]
    pub fn unseal(&self, ciphertext: &[u8]) -> Result<Vec<u8>, aead::Error> {
        if ciphertext.len() <= KEY_SIZE {
            return Err(aead::Error);
        }

        let ephemeral_sk: [u8; KEY_SIZE] = ciphertext[..KEY_SIZE].try_into().unwrap();
        let ephemeral_pk = ephemeral_sk.into();
        let nonce = get_seal_nonce(&ephemeral_pk, &self.public_key());
        let salsabox = SalsaBox::new(&ephemeral_pk, self);
        salsabox.decrypt(&nonce, &ciphertext[KEY_SIZE..])
    }
}

impl Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretKey").finish_non_exhaustive()
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.scalar.zeroize();
    }
}

impl Eq for SecretKey {}

impl From<Scalar> for SecretKey {
    fn from(scalar: Scalar) -> Self {
        let bytes = scalar.to_bytes();
        SecretKey { bytes, scalar }
    }
}

impl From<[u8; KEY_SIZE]> for SecretKey {
    fn from(bytes: [u8; KEY_SIZE]) -> SecretKey {
        Self::from_bytes(bytes)
    }
}

impl PartialEq for SecretKey {
    fn eq(&self, other: &Self) -> bool {
        self.scalar.ct_eq(&other.scalar).into()
    }
}

impl TryFrom<&[u8]> for SecretKey {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, TryFromSliceError> {
        Self::from_slice(slice)
    }
}

#[cfg(feature = "serde")]
impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serdect::array::serialize_hex_upper_or_bin(&self.to_bytes(), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut bytes = [0u8; KEY_SIZE];
        serdect::array::deserialize_hex_or_bin(&mut bytes, deserializer)?;
        Ok(SecretKey::from(bytes))
    }
}
