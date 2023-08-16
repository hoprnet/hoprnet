use std::fmt::Debug;

use crate::errors;
use crate::errors::CryptoError::InvalidInputValue;
use crate::random::{random_bytes, random_group_element};
use crate::shared_keys::Scalar;
use crate::types::{CompressedPublicKey, OffchainPublicKey, PublicKey};
use crate::utils::SecretValue;
use digest::Digest;
use generic_array::{ArrayLength, GenericArray};
use sha2::Sha512;
use subtle::{Choice, ConstantTimeEq};
use utils_types::traits::BinarySerializable;
use zeroize::ZeroizeOnDrop;

/// Represents a generic key pair
/// The keypair contains a private key and public key.
/// Must be comparable in constant time and zeroized on drop.
pub trait Keypair: ConstantTimeEq + ZeroizeOnDrop + Sized {
    /// Represents the type of the private (secret) key
    type SecretLen: ArrayLength<u8>;

    /// Represents the type of the public key
    type Public: BinarySerializable + Clone + PartialEq;

    /// Generates a new random keypair.
    fn random() -> Self;

    /// Creates a keypair from the given secret key.
    fn from_secret(bytes: &[u8]) -> errors::Result<Self>;

    /// Returns the private (secret) part of the keypair
    fn secret(&self) -> &SecretValue<Self::SecretLen>;

    /// Returns the public part of the keypair
    fn public(&self) -> &Self::Public;

    /// Consumes the instance and produces separated private and public parts
    fn unzip(self) -> (SecretValue<Self::SecretLen>, Self::Public) {
        (self.secret().clone(), self.public().clone())
    }
}

/// Represents a keypair consisting of an Ed25519 private and public key
#[derive(Clone, ZeroizeOnDrop)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct OffchainKeypair(SecretValue<typenum::U32>, #[zeroize(skip)] OffchainPublicKey);

impl Keypair for OffchainKeypair {
    type SecretLen = typenum::U32;
    type Public = OffchainPublicKey;

    fn random() -> Self {
        Self::from_secret(&random_bytes::<{ ed25519_dalek::SECRET_KEY_LENGTH }>()).unwrap()
    }

    fn from_secret(bytes: &[u8]) -> errors::Result<Self> {
        Ok(Self(
            bytes.try_into().map_err(|_| InvalidInputValue)?,
            OffchainPublicKey::from_privkey(bytes)?,
        ))
    }

    fn secret(&self) -> &SecretValue<typenum::U32> {
        &self.0
    }

    fn public(&self) -> &Self::Public {
        &self.1
    }
}

impl Debug for OffchainKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OffchainKeypair").field(&self.1).finish()
    }
}

impl ConstantTimeEq for OffchainKeypair {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.secret().ct_eq(other.secret())
    }
}

impl From<&OffchainKeypair> for curve25519_dalek::scalar::Scalar {
    /// Transforms the secret to be equivalent with the EdDSA public key used for signing.
    /// This is required, so that the secret keys used to generate Sphinx shared secrets
    /// are corresponding to the public keys we obtain from the Ed25519 peer ids.
    fn from(value: &OffchainKeypair) -> Self {
        let mut h: Sha512 = Sha512::default();
        h.update(&value.0);
        let hash = h.finalize();

        let mut ret = [0u8; ed25519_dalek::SECRET_KEY_LENGTH];
        ret.copy_from_slice(&hash[..32]);
        curve25519_dalek::scalar::Scalar::from_bytes(&ret).unwrap()
    }
}

impl From<&OffchainKeypair> for libp2p_identity::Keypair {
    fn from(value: &OffchainKeypair) -> Self {
        libp2p_identity::Keypair::ed25519_from_bytes(value.0.clone()).expect("invalid offchain keypair")
        // must not happen
    }
}

/// Represents a keypair consisting of a secp256k1 private and public key
#[derive(Clone, ZeroizeOnDrop)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ChainKeypair(SecretValue<typenum::U32>, #[zeroize(skip)] CompressedPublicKey);

impl Keypair for ChainKeypair {
    type SecretLen = typenum::U32;
    type Public = CompressedPublicKey;

    fn random() -> Self {
        let (secret, public) = random_group_element();
        Self(
            GenericArray::from(secret).into(),
            CompressedPublicKey(public.try_into().unwrap()),
        )
    }

    fn from_secret(bytes: &[u8]) -> errors::Result<Self> {
        let compressed = PublicKey::from_privkey(bytes).map(CompressedPublicKey)?;

        Ok(Self(bytes.try_into().map_err(|_| InvalidInputValue)?, compressed))
    }

    fn secret(&self) -> &SecretValue<typenum::U32> {
        &self.0
    }

    fn public(&self) -> &Self::Public {
        &self.1
    }
}

impl Debug for ChainKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChainKeypair").field(&self.1).finish()
    }
}

impl ConstantTimeEq for ChainKeypair {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.secret().ct_eq(other.secret())
    }
}

impl From<&ChainKeypair> for k256::Scalar {
    fn from(value: &ChainKeypair) -> Self {
        k256::Scalar::from_bytes(value.0.as_ref()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use crate::types::{CompressedPublicKey, OffchainPublicKey, PublicKey};
    use subtle::ConstantTimeEq;
    use utils_types::traits::PeerIdLike;

    #[test]
    fn test_offchain_keypair() {
        let kp_1 = OffchainKeypair::random();

        let public = OffchainPublicKey::from_privkey(kp_1.secret().as_ref()).unwrap();
        assert_eq!(&public, kp_1.public(), "secret keys must yield compatible public keys");

        let kp_2 = OffchainKeypair::from_secret(kp_1.secret().as_ref()).unwrap();
        assert_eq!(
            kp_1.ct_eq(&kp_2).unwrap_u8(),
            1,
            "keypairs generated from secrets must be equal"
        );
        assert_eq!(&public, kp_2.public(), "secret keys must yield compatible public keys");
        assert_eq!(kp_1.public(), kp_2.public(), "keypair public keys must be equal");

        let (s1, p1) = kp_1.unzip();
        let (s2, p2) = kp_2.unzip();

        assert_eq!(s1.ct_eq(&s2).unwrap_u8(), 1);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_offchain_keypair_libp2p_compatibility() {
        let kp_1 = OffchainKeypair::random();

        let p2p_kp: libp2p_identity::Keypair = (&kp_1).into();
        assert_eq!(
            kp_1.public().to_peerid(),
            p2p_kp.public().to_peer_id(),
            "peer ids must be equal"
        );
    }

    #[test]
    fn test_chain_keypair() {
        let kp_1 = ChainKeypair::random();

        let public = CompressedPublicKey(PublicKey::from_privkey(kp_1.secret().as_ref()).unwrap());
        assert_eq!(&public, kp_1.public(), "secret keys must yield compatible public keys");

        let kp_2 = ChainKeypair::from_secret(kp_1.secret().as_ref()).unwrap();
        assert_eq!(
            kp_1.ct_eq(&kp_2).unwrap_u8(),
            1,
            "keypairs generated from secrets must be equal"
        );
        assert_eq!(&public, kp_2.public(), "secret keys must yield compatible public keys");
        assert_eq!(kp_1.public(), kp_2.public(), "keypair public keys must be equal");

        let (s1, p1) = kp_1.unzip();
        let (s2, p2) = kp_2.unzip();

        assert_eq!(s1.ct_eq(&s2).unwrap_u8(), 1);
        assert_eq!(p1, p2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use crate::types::{OffchainPublicKey, PublicKey};
    use js_sys::Uint8Array;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
    use utils_types::traits::PeerIdLike;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    impl OffchainKeypair {
        #[wasm_bindgen(constructor)]
        pub fn _new(secret: &[u8]) -> JsResult<OffchainKeypair> {
            ok_or_jserr!(Self::from_secret(secret))
        }

        #[wasm_bindgen(js_name = "random")]
        pub fn _random() -> OffchainKeypair {
            Self::random()
        }

        #[wasm_bindgen(js_name = "public")]
        pub fn _public(&self) -> OffchainPublicKey {
            self.1.clone()
        }

        #[wasm_bindgen(js_name = "secret")]
        pub fn _secret(&self) -> Uint8Array {
            self.0.as_ref().into()
        }

        #[wasm_bindgen(js_name = "to_peerid_str")]
        pub fn _to_peerid_str(&self) -> String {
            self.1.to_peerid_str()
        }
    }

    #[wasm_bindgen]
    impl ChainKeypair {
        #[wasm_bindgen(constructor)]
        pub fn _new(secret: &[u8]) -> JsResult<ChainKeypair> {
            ok_or_jserr!(Self::from_secret(secret))
        }

        #[wasm_bindgen(js_name = "random")]
        pub fn _random() -> ChainKeypair {
            Self::random()
        }

        #[wasm_bindgen(js_name = "public")]
        pub fn _public(&self) -> PublicKey {
            self.1 .0.clone()
        }

        #[wasm_bindgen(js_name = "secret")]
        pub fn _secret(&self) -> Uint8Array {
            self.0.as_ref().into()
        }

        #[wasm_bindgen(js_name = "to_address")]
        pub fn _to_address(&self) -> Address {
            self.1 .0.to_address()
        }
    }
}
