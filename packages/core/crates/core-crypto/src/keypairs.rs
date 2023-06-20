use zeroize::{Zeroize, ZeroizeOnDrop};
use ed25519_dalek::Sha512;
use curve25519_dalek::digest::Digest;
use utils_types::traits::BinarySerializable;
use crate::errors;
use crate::errors::CryptoError::InvalidInputValue;
use crate::random::{random_bytes, random_group_element};
use crate::shared_keys::Scalar;
use crate::types::{CompressedPublicKey, OffchainPublicKey, PublicKey};

/// Represents a generic key pair
/// The keypair contains a private key and public key.
/// The type must be zeroized on drop.
pub trait Keypair: ZeroizeOnDrop + Sized {
    /// Represents the type of the private (secret) key
    type Secret: Zeroize + AsRef<[u8]>;

    /// Represents the type of the public key
    type Public: BinarySerializable + Clone + PartialEq;

    /// Generates a new random keypair.
    fn random() -> Self;

    /// Creates a keypair from the given secret key.
    fn from_secret(bytes: &[u8]) -> errors::Result<Self>;

    /// Returns the private (secret) part of the keypair
    fn secret(&self) -> &Self::Secret;

    /// Returns the public part of the keypair
    fn public(&self) -> &Self::Public;

    /// Consumes the instance and produces separated private and public part
    fn unzip(self) -> (Self::Secret, Self::Public);
}

/// Represents a keypair consisting of an Ed25519 private and public key
#[derive(Debug, Clone, PartialEq, ZeroizeOnDrop)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct OffchainKeypair([u8; ed25519_dalek::SECRET_KEY_LENGTH], #[zeroize(skip)] OffchainPublicKey);

impl Keypair for OffchainKeypair {
    type Secret = [u8; ed25519_dalek::SECRET_KEY_LENGTH];
    type Public = OffchainPublicKey;

    fn random() -> Self {
        Self::from_secret(&random_bytes::<{ed25519_dalek::SECRET_KEY_LENGTH}>()).unwrap()
    }

    fn from_secret(bytes: &[u8]) -> errors::Result<Self> {
       Ok(Self(bytes.try_into().map_err(|_| InvalidInputValue)?, OffchainPublicKey::from_privkey(bytes)?))
    }

    fn secret(&self) -> &Self::Secret {
        &self.0
    }

    fn public(&self) -> &Self::Public {
        &self.1
    }

    fn unzip(self) -> (Self::Secret, Self::Public) {
        (self.0, self.1.clone())
    }
}

impl From<&OffchainKeypair> for curve25519_dalek::scalar::Scalar {
    /// Transforms the secret to be equivalent with the EdDSA public key used for signing.
    /// This is required, so that the secret keys used to generate Sphinx shared secrets
    /// are corresponding to the public keys we obtain from the Ed25519 peer ids.
    fn from(value: &OffchainKeypair) -> Self {
        let mut h: Sha512 = Sha512::new();
        h.update(&value.0);
        let hash = h.finalize();

        let mut ret = [0u8; ed25519_dalek::SECRET_KEY_LENGTH];
        ret.copy_from_slice(&hash[..32]);
        curve25519_dalek::scalar::Scalar::from_bytes(&ret).unwrap()
    }
}

/// Represents a keypair consisting of a secp256k1 private and public key
#[derive(Clone, PartialEq, ZeroizeOnDrop)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ChainKeypair([u8; 32], #[zeroize(skip)] CompressedPublicKey);

impl Keypair for ChainKeypair {
    type Secret = [u8; 32];
    type Public = CompressedPublicKey;

    fn random() -> Self {
        let (secret, public) = random_group_element();
        Self (secret, CompressedPublicKey(public.try_into().unwrap()))
    }

    fn from_secret(bytes: &[u8]) -> errors::Result<Self> {
        let compressed = PublicKey::from_privkey(bytes)
            .map(|pk| CompressedPublicKey(pk))?;

        Ok(Self(bytes.try_into().map_err(|_| InvalidInputValue)?, compressed))
    }

    fn secret(&self) -> &Self::Secret {
        &self.0
    }

    fn public(&self) -> &Self::Public {
        &self.1
    }

    fn unzip(self) -> (Self::Secret, Self::Public) {
        (self.0, self.1.clone())
    }
}

impl From<&ChainKeypair> for k256::Scalar {
    fn from(value: &ChainKeypair) -> Self {
        k256::Scalar::from_bytes(&value.0).unwrap()
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::keypairs::{ChainKeypair, Keypair, OffchainKeypair};

    #[wasm_bindgen]
    impl OffchainKeypair {
        #[wasm_bindgen(constructor)]
        pub fn _new(secret: &[u8]) -> JsResult<OffchainKeypair> {
            ok_or_jserr!(Self::from_secret(secret))
        }
    }

    #[wasm_bindgen]
    impl ChainKeypair {
        #[wasm_bindgen(constructor)]
        pub fn _new(secret: &[u8]) -> JsResult<ChainKeypair> {
            ok_or_jserr!(Self::from_secret(secret))
        }
    }
}