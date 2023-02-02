// TODO: All types specified in this module will be moved over to the core-crypto crate once merged.

use std::str::FromStr;
use k256::ecdsa::{SigningKey, Signature as ECDSASignature, signature::Signer, VerifyingKey};
use k256::{elliptic_curve, NonZeroScalar, Secp256k1};
use k256::ecdsa::signature::Verifier;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use libp2p_core::PeerId;
use crate::errors::{Result, GeneralError::ParseError};

#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PublicKey {
    key: elliptic_curve::PublicKey<Secp256k1>,
    compressed: Box<[u8]> // cache the compressed form to save some cycles
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PublicKey {
    pub fn eq(&self, other: &PublicKey) -> bool {
        // Needs reimplemented, because the trait impl is not available in WASM
        self.key.eq(&other.key) && self.compressed.eq(&other.compressed)
    }

    pub fn serialize(&self, compressed: bool) -> Box<[u8]> {
        if compressed {
            self.compressed.clone()
        } else {
            self.key.as_affine().to_encoded_point(false).to_bytes()
        }
    }

    pub fn to_hex(&self, compressed: bool) -> String {
        hex::encode(self.serialize(compressed))
    }

    pub fn to_peerid_str(&self) -> String {
        self.to_peerid().to_base58()
    }

}

impl PublicKey {
    pub fn to_peerid(&self) -> PeerId {
        PeerId::from_public_key(&libp2p_core::PublicKey::Secp256k1(
            libp2p_core::identity::secp256k1::PublicKey::decode(&self.compressed)
                .expect("cannot convert this public key to secp256k1 peer id")
        ))
    }

    pub fn deserialize(bytes: &[u8]) -> Result<PublicKey> {
        let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(bytes)
            .map_err(|_| ParseError)?;
        Ok(PublicKey{
            compressed: key.to_encoded_point(true).to_bytes(),
            key,
        })
    }

    pub fn from_peerid_str(peer_id: &str) -> Result<PublicKey> {
        Self::from_peerid(&PeerId::from_str(peer_id).map_err(|_|ParseError)?)
    }

    pub fn from_peerid(peer_id: &PeerId) -> Result<PublicKey> {
        // Here we explicitly assume non-RSA PeerId, so that multihash bytes are the actual public key
        let pid = peer_id.to_bytes();
        let (_, mh) = pid.split_at(6);
        Self::deserialize(mh)
    }

    pub fn from_privkey(private_key: &[u8]) -> Result<PublicKey> {
        let secret_scalar = NonZeroScalar::try_from(private_key)
            .map_err(|_| ParseError)?;

        let key = elliptic_curve::PublicKey::<Secp256k1>::from_secret_scalar(&secret_scalar);
        Ok(PublicKey {
            key,
            compressed: key.to_encoded_point(true).to_bytes()
        })
    }
}

// TODO: Move all Signature related stuff to core-crypto once merged
pub const SIGNATURE_LENGTH: usize = 64;

#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Signature {
    signature: [u8; SIGNATURE_LENGTH],
    recovery: u8,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Signature {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(raw_bytes: &[u8], recovery: u8) -> Signature {
        assert_eq!(raw_bytes.len(), SIGNATURE_LENGTH, "invalid length");
        assert!(recovery <= 1, "invalid recovery bit");
        let mut ret = Self {
            signature: [0u8; SIGNATURE_LENGTH],
            recovery
        };
        ret.signature.copy_from_slice(raw_bytes);
        ret
    }

    pub fn sign_message(message: &[u8], private_key: &[u8]) -> Signature {
        let key = SigningKey::from_bytes(private_key)
            .expect("invalid signing key");
        let signature: ECDSASignature = key.sign(message);
        Self::deserialize(signature.to_bytes().as_slice()).expect("signing failed")
    }

    pub fn verify(&self, message: &[u8], public_key: &[u8]) -> bool {
        let pub_key = VerifyingKey::from_sec1_bytes(public_key)
            .expect("invalid public key");
        let signature = ECDSASignature::try_from(self.signature.as_slice())
            .expect("invalid signature");

        pub_key.verify(message, &signature).is_ok()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.signature)
    }

    pub fn raw_signature(&self) -> Box<[u8]> {
        self.signature.into()
    }

    pub fn serialize(&self) -> Box<[u8]> {
        let mut compressed = Vec::from(self.signature);
        compressed[SIGNATURE_LENGTH/2] &= 0x7f;
        compressed[SIGNATURE_LENGTH/2] |= self.recovery << 7;
        compressed.into_boxed_slice()
    }
}

impl Signature {
    pub fn deserialize(signature: &[u8]) -> Result<Signature> {
        if signature.len() == SIGNATURE_LENGTH {
            // Read & clear the top-most bit in S
            let mut ret = Signature {
                signature: [0u8; SIGNATURE_LENGTH],
                recovery: if signature[SIGNATURE_LENGTH/2]&0x80 != 0 { 1 } else { 0 }
            };
            ret.signature.copy_from_slice(signature);
            ret.signature[SIGNATURE_LENGTH/2] &= 0x7f;

            Ok(ret)
        } else {
            Err(ParseError)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use lazy_static::lazy_static;
    use crate::crypto::{PublicKey, Signature};

    lazy_static! {
        static ref PUBLIC_KEY: Vec<u8>  = hex::decode("021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8").unwrap();
        static ref PRIVATE_KEY: Vec<u8> = hex::decode("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8").unwrap();
    }

    #[test]
    fn signature_signing_test() {
        let msg = b"test";
        let sgn = Signature::sign_message(msg, &PRIVATE_KEY);

        assert!(sgn.verify(msg, &PUBLIC_KEY))
    }

    #[test]
    fn signature_serialize_test() {
        let msg = b"test";
        let sgn = Signature::sign_message(msg, &PRIVATE_KEY);

        let deserialized = Signature::deserialize(&sgn.serialize()).unwrap();

        assert_eq!(sgn, deserialized, "signatures don't match");
    }

    #[test]
    fn public_key_peerid_test() {
        let pk1 = PublicKey::deserialize(&PUBLIC_KEY)
            .expect("failed to deserialize");

        let pk2 = PublicKey::from_peerid_str(pk1.to_peerid_str().as_str())
            .expect("peer id serialization failed");

        assert_eq!(pk1, pk2, "pubkeys don't match");
        assert_eq!(pk1.to_peerid_str(), pk2.to_peerid_str(), "peer id strings don't match");
    }

    #[test]
    fn public_key_serialize_test() {
        let pk1 = PublicKey::deserialize(&PUBLIC_KEY)
            .expect("failed to deserialize 1");
        let pk2 = PublicKey::deserialize(&pk1.serialize(true))
            .expect("failed to deserialize 2");
        let pk3 = PublicKey::deserialize(&pk1.serialize(false))
            .expect("failed to deserialize 3");

        assert_eq!(pk1, pk2, "pub keys 1 2 don't match");
        assert_eq!(pk2, pk3, "pub keys 2 3 don't match");
    }

    #[test]
    fn public_key_from_privkey() {
        let pk1 = PublicKey::from_privkey(&PRIVATE_KEY)
            .expect("failed to convert from private key");
        let pk2 = PublicKey::deserialize(&PUBLIC_KEY)
            .expect("failed to deserialize");

        assert_eq!(pk1, pk2);
        assert_eq!(pk1, pk2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;
    use crate::crypto::{PublicKey, Signature};

    #[wasm_bindgen]
    impl Signature {
        pub fn deserialize_signature(signature: &[u8]) -> JsResult<Signature> {
            ok_or_jserr!(Signature::deserialize(signature))
        }
    }

    #[wasm_bindgen]
    impl PublicKey {
        pub fn deserialize_public_key(bytes: &[u8]) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::deserialize(bytes))
        }

        pub fn public_key_from_peerid_str(peer_id: &str) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::from_peerid_str(peer_id))
        }
    }
}