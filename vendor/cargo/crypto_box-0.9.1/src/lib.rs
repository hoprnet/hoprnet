#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms)]

//! ## Usage
//!
#![cfg_attr(all(feature = "getrandom", feature = "std"), doc = "```")]
#![cfg_attr(not(all(feature = "getrandom", feature = "std")), doc = "```ignore")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use crypto_box::{
//!     aead::{Aead, AeadCore, OsRng},
//!     SalsaBox, PublicKey, SecretKey
//! };
//!
//! //
//! // Encryption
//! //
//!
//! // Generate a random secret key.
//! // NOTE: The secret key bytes can be accessed by calling `secret_key.as_bytes()`
//! let alice_secret_key = SecretKey::generate(&mut OsRng);
//!
//! // Get the public key for the secret key we just generated
//! let alice_public_key_bytes = alice_secret_key.public_key().as_bytes().clone();
//!
//! // Obtain your recipient's public key.
//! let bob_public_key = PublicKey::from([
//!    0xe8, 0x98, 0xc, 0x86, 0xe0, 0x32, 0xf1, 0xeb,
//!    0x29, 0x75, 0x5, 0x2e, 0x8d, 0x65, 0xbd, 0xdd,
//!    0x15, 0xc3, 0xb5, 0x96, 0x41, 0x17, 0x4e, 0xc9,
//!    0x67, 0x8a, 0x53, 0x78, 0x9d, 0x92, 0xc7, 0x54,
//! ]);
//!
//! // Create a `SalsaBox` by performing Diffie-Hellman key agreement between
//! // the two keys.
//! let alice_box = SalsaBox::new(&bob_public_key, &alice_secret_key);
//!
//! // Get a random nonce to encrypt the message under
//! let nonce = SalsaBox::generate_nonce(&mut OsRng);
//!
//! // Message to encrypt
//! let plaintext = b"Top secret message we're encrypting";
//!
//! // Encrypt the message using the box
//! let ciphertext = alice_box.encrypt(&nonce, &plaintext[..])?;
//!
//! //
//! // Decryption
//! //
//!
//! // Either side can encrypt or decrypt messages under the Diffie-Hellman key
//! // they agree upon. The example below shows Bob's side.
//! let bob_secret_key = SecretKey::from([
//!     0xb5, 0x81, 0xfb, 0x5a, 0xe1, 0x82, 0xa1, 0x6f,
//!     0x60, 0x3f, 0x39, 0x27, 0xd, 0x4e, 0x3b, 0x95,
//!     0xbc, 0x0, 0x83, 0x10, 0xb7, 0x27, 0xa1, 0x1d,
//!     0xd4, 0xe7, 0x84, 0xa0, 0x4, 0x4d, 0x46, 0x1b
//! ]);
//!
//! // Deserialize Alice's public key from bytes
//! let alice_public_key = PublicKey::from(alice_public_key_bytes);
//!
//! // Bob can compute the same `SalsaBox` as Alice by performing the
//! // key agreement operation.
//! let bob_box = SalsaBox::new(&alice_public_key, &bob_secret_key);
//!
//! // Decrypt the message, using the same randomly generated nonce
//! let decrypted_plaintext = bob_box.decrypt(&nonce, &ciphertext[..])?;
//!
//! assert_eq!(&plaintext[..], &decrypted_plaintext[..]);
//! # Ok(())
//! # }
//! ```
//!
//! ## Choosing [`ChaChaBox`] vs [`SalsaBox`]
//!
//! The `crypto_box` construction was originally specified using [`SalsaBox`].
//!
//! However, the newer [`ChaChaBox`] construction is also available, which
//! provides better security and performance.
//!
//! To use it, enable the `chacha20` feature.
//!
#![cfg_attr(
    all(feature = "chacha20", feature = "getrandom", feature = "std"),
    doc = "```"
)]
#![cfg_attr(
    not(all(feature = "chacha20", feature = "getrandom", feature = "std")),
    doc = "```ignore"
)]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use crypto_box::{
//!     aead::{Aead, AeadCore, Payload, OsRng},
//!     ChaChaBox, PublicKey, SecretKey
//! };
//!
//! let alice_secret_key = SecretKey::generate(&mut OsRng);
//! let alice_public_key_bytes = alice_secret_key.public_key().as_bytes().clone();
//! let bob_public_key = PublicKey::from([
//!    0xe8, 0x98, 0xc, 0x86, 0xe0, 0x32, 0xf1, 0xeb,
//!    0x29, 0x75, 0x5, 0x2e, 0x8d, 0x65, 0xbd, 0xdd,
//!    0x15, 0xc3, 0xb5, 0x96, 0x41, 0x17, 0x4e, 0xc9,
//!    0x67, 0x8a, 0x53, 0x78, 0x9d, 0x92, 0xc7, 0x54,
//! ]);
//! let alice_box = ChaChaBox::new(&bob_public_key, &alice_secret_key);
//! let nonce = ChaChaBox::generate_nonce(&mut OsRng);
//!
//! // Message to encrypt
//! let plaintext = b"Top secret message we're encrypting".as_ref();
//!
//! // Encrypt the message using the box
//! let ciphertext = alice_box.encrypt(&nonce, plaintext).unwrap();
//!
//! //
//! // Decryption
//! //
//!
//! let bob_secret_key = SecretKey::from([
//!     0xb5, 0x81, 0xfb, 0x5a, 0xe1, 0x82, 0xa1, 0x6f,
//!     0x60, 0x3f, 0x39, 0x27, 0xd, 0x4e, 0x3b, 0x95,
//!     0xbc, 0x0, 0x83, 0x10, 0xb7, 0x27, 0xa1, 0x1d,
//!     0xd4, 0xe7, 0x84, 0xa0, 0x4, 0x4d, 0x46, 0x1b
//! ]);
//! let alice_public_key = PublicKey::from(alice_public_key_bytes);
//! let bob_box = ChaChaBox::new(&alice_public_key, &bob_secret_key);
//!
//! // Decrypt the message, using the same randomly generated nonce
//! let decrypted_plaintext = bob_box.decrypt(&nonce, ciphertext.as_slice()).unwrap();
//!
//! assert_eq!(&plaintext[..], &decrypted_plaintext[..]);
//! # Ok(())
//! # }
//! ```
//!
//! ## In-place Usage (eliminates `alloc` requirement)
//!
//! This crate has an optional `alloc` feature which can be disabled in e.g.
//! microcontroller environments that don't have a heap.
//!
//! The [`AeadInPlace::encrypt_in_place`] and [`AeadInPlace::decrypt_in_place`]
//! methods accept any type that impls the [`aead::Buffer`] trait which
//! contains the plaintext for encryption or ciphertext for decryption.
//!
//! Note that if you enable the `heapless` feature of this crate,
//! you will receive an impl of `aead::Buffer` for [`heapless::Vec`]
//! (re-exported from the `aead` crate as `aead::heapless::Vec`),
//! which can then be passed as the `buffer` parameter to the in-place encrypt
//! and decrypt methods.
//!
//! A `heapless` usage example can be found in the documentation for the
//! `xsalsa20poly1305` crate:
//!
//! <https://docs.rs/xsalsa20poly1305/latest/xsalsa20poly1305/#in-place-usage-eliminates-alloc-requirement>
//!
//! [NaCl]: https://nacl.cr.yp.to/
//! [`crypto_box`]: https://nacl.cr.yp.to/box.html
//! [X25519]: https://cr.yp.to/ecdh.html
//! [XSalsa20Poly1305]: https://nacl.cr.yp.to/secretbox.html
//! [ECIES]: https://en.wikipedia.org/wiki/Integrated_Encryption_Scheme
//! [`heapless::Vec`]: https://docs.rs/heapless/latest/heapless/struct.Vec.html

#[cfg(feature = "seal")]
extern crate alloc;

mod public_key;
mod secret_key;

pub use crate::{public_key::PublicKey, secret_key::SecretKey};
pub use aead;
pub use crypto_secretbox::Nonce;

use aead::{
    consts::{U0, U16, U24, U32, U8},
    generic_array::GenericArray,
    AeadCore, AeadInPlace, Buffer, Error, KeyInit,
};
use crypto_secretbox::{
    cipher::{IvSizeUser, KeyIvInit, KeySizeUser, StreamCipher},
    Kdf, SecretBox,
};
use zeroize::Zeroizing;

#[cfg(feature = "chacha20")]
use chacha20::ChaCha20Legacy as ChaCha20;

#[cfg(feature = "salsa20")]
use salsa20::Salsa20;

/// Size of a `crypto_box` public or secret key in bytes.
pub const KEY_SIZE: usize = 32;

/// Poly1305 tag.
///
/// Implemented as an alias for [`GenericArray`].
pub type Tag = GenericArray<u8, U16>;

/// Size of a Poly1305 tag in bytes.
#[cfg(feature = "seal")]
const TAG_SIZE: usize = 16;

#[cfg(feature = "seal")]
/// Extra bytes for the ciphertext of a `crypto_box_seal` compared to the plaintext
pub const SEALBYTES: usize = KEY_SIZE + TAG_SIZE;

/// [`CryptoBox`] instantiated with the ChaCha20 stream cipher.
#[cfg(feature = "chacha20")]
pub type ChaChaBox = CryptoBox<ChaCha20>;

/// [`CryptoBox`] instantiated with with the Salsa20 stream cipher.
#[cfg(feature = "salsa20")]
pub type SalsaBox = CryptoBox<Salsa20>;

/// Public-key encryption scheme based on the [X25519] Elliptic Curve
/// Diffie-Hellman function and the [crypto_secretbox] authenticated encryption
/// cipher.
///
/// This type impls the [`aead::Aead`] trait, and otherwise functions as a
/// symmetric Authenticated Encryption with Associated Data (AEAD) cipher
/// once instantiated.
///
/// Note that additional associated data (AAD) is not supported and encryption
/// operations will return [`aead::Error`] if it is provided as an argument.
///
/// [X25519]: https://cr.yp.to/ecdh.html
/// [crypto_secretbox]: https://github.com/RustCrypto/nacl-compat/tree/master/crypto_secretbox
#[derive(Clone)]
pub struct CryptoBox<C> {
    secretbox: SecretBox<C>,
}

impl<C> CryptoBox<C> {
    /// Create a new [`CryptoBox`], performing X25519 Diffie-Hellman to derive
    /// a shared secret from the provided public and secret keys.
    pub fn new(public_key: &PublicKey, secret_key: &SecretKey) -> Self
    where
        C: Kdf,
    {
        let shared_secret = Zeroizing::new(secret_key.scalar * public_key.0);

        // Use HChaCha20 to create a uniformly random key from the shared secret
        let key = Zeroizing::new(C::kdf(
            GenericArray::from_slice(&shared_secret.0),
            &GenericArray::default(),
        ));

        Self {
            secretbox: SecretBox::<C>::new(&*key),
        }
    }
}

impl<C> AeadCore for CryptoBox<C> {
    type NonceSize = U24;
    type TagSize = U16;
    type CiphertextOverhead = U0;
}

impl<C> AeadInPlace for CryptoBox<C>
where
    C: Kdf + KeyIvInit + KeySizeUser<KeySize = U32> + IvSizeUser<IvSize = U8> + StreamCipher,
{
    fn encrypt_in_place(
        &self,
        nonce: &GenericArray<u8, Self::NonceSize>,
        associated_data: &[u8],
        buffer: &mut dyn Buffer,
    ) -> Result<(), Error> {
        self.secretbox
            .encrypt_in_place(nonce, associated_data, buffer)
    }

    fn encrypt_in_place_detached(
        &self,
        nonce: &GenericArray<u8, Self::NonceSize>,
        associated_data: &[u8],
        buffer: &mut [u8],
    ) -> Result<Tag, Error> {
        self.secretbox
            .encrypt_in_place_detached(nonce, associated_data, buffer)
    }

    fn decrypt_in_place(
        &self,
        nonce: &GenericArray<u8, Self::NonceSize>,
        associated_data: &[u8],
        buffer: &mut dyn Buffer,
    ) -> Result<(), Error> {
        self.secretbox
            .decrypt_in_place(nonce, associated_data, buffer)
    }

    fn decrypt_in_place_detached(
        &self,
        nonce: &GenericArray<u8, Self::NonceSize>,
        associated_data: &[u8],
        buffer: &mut [u8],
        tag: &Tag,
    ) -> Result<(), Error> {
        self.secretbox
            .decrypt_in_place_detached(nonce, associated_data, buffer, tag)
    }
}

#[cfg(feature = "seal")]
fn get_seal_nonce(ephemeral_pk: &PublicKey, recipient_pk: &PublicKey) -> Nonce {
    use blake2::{Blake2b, Digest};
    let mut hasher = Blake2b::<U24>::new();
    hasher.update(ephemeral_pk.as_bytes());
    hasher.update(recipient_pk.as_bytes());
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn test_public_key_serialization() {
        use aead::rand_core::RngCore;

        // Random PK bytes
        let mut public_key_bytes = [0; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut public_key_bytes);

        // Create public key
        let public_key = PublicKey::from(public_key_bytes);

        // Round-trip serialize with bincode
        let serialized = bincode::serialize(&public_key).unwrap();
        let deserialized: PublicKey = bincode::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, public_key,);

        // Round-trip serialize with rmp (msgpack)
        let serialized = rmp_serde::to_vec_named(&public_key).unwrap();
        let deserialized: PublicKey = rmp_serde::from_slice(&serialized).unwrap();
        assert_eq!(deserialized, public_key,);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_secret_key_serialization() {
        use aead::rand_core::RngCore;

        // Random SK bytes
        let mut secret_key_bytes = [0; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut secret_key_bytes);

        // Create secret key
        let secret_key = SecretKey::from(secret_key_bytes);

        // Round-trip serialize with bincode
        let serialized = bincode::serialize(&secret_key).unwrap();
        let deserialized: SecretKey = bincode::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.to_bytes(), secret_key.to_bytes());

        // Round-trip serialize with rmp (msgpack)
        let serialized = rmp_serde::to_vec_named(&secret_key).unwrap();
        let deserialized: SecretKey = rmp_serde::from_slice(&serialized).unwrap();
        assert_eq!(deserialized.to_bytes(), secret_key.to_bytes());
    }

    #[test]
    fn test_public_key_from_slice() {
        let array = [0; 40];

        // Returns None for empty array
        assert!(PublicKey::from_slice(&[]).is_err());

        // Returns None for length <32
        for i in 1..=31 {
            assert!(PublicKey::from_slice(&array[..i]).is_err());
        }

        // Succeeds for length 32
        assert!(PublicKey::from_slice(&array[..32]).is_ok());

        // Returns None for length >32
        for i in 33..=40 {
            assert!(PublicKey::from_slice(&array[..i]).is_err());
        }
    }
}
