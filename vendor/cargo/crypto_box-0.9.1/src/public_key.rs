use crate::{SecretKey, KEY_SIZE};
use core::{array::TryFromSliceError, cmp::Ordering};
use curve25519_dalek::MontgomeryPoint;

#[cfg(feature = "seal")]
use {
    crate::{get_seal_nonce, SalsaBox, TAG_SIZE},
    aead::{rand_core::CryptoRngCore, Aead},
    alloc::vec::Vec,
};

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

/// A `crypto_box` public key.
///
/// This type can be serialized if the `serde` feature is enabled.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PublicKey(pub(crate) MontgomeryPoint);

impl PublicKey {
    /// Initialize [`PublicKey`] from a byte array.
    pub fn from_bytes(bytes: [u8; KEY_SIZE]) -> Self {
        PublicKey(MontgomeryPoint(bytes))
    }

    /// Initialize [`PublicKey`] from a byte slice.
    ///
    /// Returns [`TryFromSliceError`] if the slice length is not exactly equal
    /// to [`KEY_SIZE`].
    pub fn from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
        slice.try_into().map(Self::from_bytes)
    }

    /// Borrow the public key as bytes.
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        self.0.as_bytes()
    }

    /// Serialize this public key as bytes.
    pub fn to_bytes(&self) -> [u8; KEY_SIZE] {
        self.0.to_bytes()
    }

    /// Implementation of `crypto_box_seal` function from [libsodium "sealed boxes"].
    ///
    /// Sealed boxes are designed to anonymously send messages to a recipient given their public key.
    ///
    /// [libsodium "sealed boxes"]: https://doc.libsodium.org/public-key_cryptography/sealed_boxes
    #[cfg(feature = "seal")]
    pub fn seal(
        &self,
        csprng: &mut impl CryptoRngCore,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, aead::Error> {
        let mut out = Vec::with_capacity(KEY_SIZE + TAG_SIZE + plaintext.len());
        let ephemeral_sk = SecretKey::generate(csprng);
        let ephemeral_pk = ephemeral_sk.public_key();
        out.extend_from_slice(ephemeral_pk.as_bytes());

        let nonce = get_seal_nonce(&ephemeral_pk, self);
        let salsabox = SalsaBox::new(self, &ephemeral_sk);
        let encrypted = salsabox.encrypt(&nonce, plaintext)?;
        out.extend_from_slice(&encrypted);

        Ok(out)
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<&SecretKey> for PublicKey {
    fn from(secret_key: &SecretKey) -> PublicKey {
        secret_key.public_key()
    }
}

impl From<[u8; KEY_SIZE]> for PublicKey {
    fn from(bytes: [u8; KEY_SIZE]) -> PublicKey {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, TryFromSliceError> {
        Self::from_slice(slice)
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl From<MontgomeryPoint> for PublicKey {
    fn from(value: MontgomeryPoint) -> Self {
        PublicKey(value)
    }
}

#[cfg(feature = "serde")]
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serdect::array::serialize_hex_upper_or_bin(self.as_bytes(), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut bytes = [0u8; KEY_SIZE];
        serdect::array::deserialize_hex_or_bin(&mut bytes, deserializer)?;
        Ok(PublicKey::from(bytes)) // TODO(tarcieri): validate key
    }
}
