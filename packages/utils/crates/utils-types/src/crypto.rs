// TODO: All types specified in this module will be moved over to the core-crypto crate once merged.

use std::str::FromStr;
use k256::ecdsa::{SigningKey, Signature as ECDSASignature, signature::Signer, VerifyingKey};
use k256::{elliptic_curve, NonZeroScalar, Secp256k1};
use k256::ecdsa::signature::Verifier;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use libp2p_core::PeerId;
use sha3::{Keccak256, digest::DynDigest};
use crate::errors::{Result, GeneralError::ParseError};
use crate::primitives::Address;

/// Represent an uncompressed elliptic curve point on the secp256k1 curve
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct CurvePoint {
    uncompressed: [u8; Self::SIZE]
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl CurvePoint {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE, "invalid length");
        let mut ret = CurvePoint {
            uncompressed: [0u8; Self::SIZE]
        };
        ret.uncompressed.copy_from_slice(bytes);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.uncompressed)
    }

    pub fn to_address(&self) -> Address {
        Address::new(&Hash::create(&[&self.uncompressed[1..]]).serialize())
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.uncompressed.into()
    }

    pub fn eq(&self, other: &CurvePoint) -> bool {
        self.uncompressed.eq(&other.uncompressed)
    }

    pub fn to_peerid_str(&self) -> String {
        self.to_peerid().to_base58()
    }
}

impl CurvePoint {
    /// Size of the uncompressed elliptic curve point
    pub const SIZE: usize = 64;

    pub fn from_exponent(exponent: &[u8]) -> Result<Self> {
        Ok(CurvePoint::new(&PublicKey::from_privkey(exponent)?
               .serialize(false)
        ))
    }

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(CurvePoint::new(&hex::decode(s).map_err(|_| ParseError)?))
    }

    pub fn from_peerid(peer_id: &PeerId) -> Result<Self> {
        Ok(CurvePoint::new(&PublicKey::from_peerid(peer_id)?.serialize(false)))
    }

    pub fn from_peerid_str(peer_id: &str) -> Result<Self> {
        Self::from_peerid(&PeerId::from_str(peer_id).map_err(|_|ParseError)?)
    }

    pub fn to_peerid(&self) -> PeerId {
        PublicKey::deserialize(&self.uncompressed).unwrap().to_peerid()
    }
}

/// Represents a half-key used for Proof of Relay
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HalfKey {
    hkey: [u8; Self::SIZE]
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl HalfKey {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(half_key: &[u8]) -> Self {
        assert_eq!(half_key.len(), Self::SIZE, "invalid length");
        let mut ret = Self {
            hkey: [0u8; Self::SIZE]
        };
        ret.hkey.copy_from_slice(&half_key);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.hkey)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.hkey.into()
    }

    pub fn eq(&self, other: &HalfKey) -> bool {
        self.hkey.eq(&other.hkey)
    }

    pub fn clone_halfkey(&self) -> HalfKey {
        self.clone()
    }
}

impl HalfKey {
    /// Size of the half-key
    pub const SIZE: usize = 32;

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Self {
                hkey: [0u8; Self::SIZE]
            };
            ret.hkey.copy_from_slice(&data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }
}

/// Represents a challange for the half-key in Proof of Relay
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HalfKeyChallenge {
    hkc: [u8; Self::SIZE]
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl HalfKeyChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(half_key_challenge: &[u8]) -> Self {
        assert_eq!(half_key_challenge.len(), Self::SIZE, "invalid length");
        let mut ret = Self {
            hkc: [0u8; Self::SIZE]
        };
        ret.hkc.copy_from_slice(&half_key_challenge);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.hkc)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.hkc.into()
    }

    pub fn eq(&self, other: &HalfKeyChallenge) -> bool {
        self.hkc.eq(&other.hkc)
    }

    pub fn clone_halfkey_challenge(&self) -> HalfKeyChallenge {
        self.clone()
    }

    pub fn to_address(&self) -> Address {
        PublicKey::deserialize(&self.hkc)
            .expect("invalid half-key")
            .to_address()
    }

    pub fn to_peerid_str(&self) -> String {
        self.to_peerid().to_base58()
    }
}

impl HalfKeyChallenge {
    /// Size of the half-key challenge is the size of the compressed secp256k1 point
    pub const SIZE: usize = 33;

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Self {
                hkc: [0u8; Self::SIZE]
            };
            ret.hkc.copy_from_slice(&data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        Self::deserialize(&hex::decode(str).map_err(|_| ParseError)?)
    }

    pub fn from_peerid(peer_id: &PeerId) -> Result<Self> {
        HalfKeyChallenge::deserialize(&PublicKey::from_peerid(peer_id)?.serialize(true))
    }

    pub fn from_peerid_str(peer_id: &str) -> Result<Self> {
        Self::from_peerid(&PeerId::from_str(peer_id).map_err(|_|ParseError)?)
    }

    pub fn to_peerid(&self) -> PeerId {
        PublicKey::deserialize(&self.hkc)
            .expect("invalid half-key")
            .to_peerid()
    }

}

/// Represents a 256-bit hash value
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Hash {
    hash: [u8; Self::SIZE],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Hash {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(hash: &[u8]) -> Self {
        assert_eq!(hash.len(), Self::SIZE, "invalid length");
        let mut ret = Hash {
            hash: [0u8; Self::SIZE]
        };
        ret.hash.copy_from_slice(hash);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.hash.into()
    }

    pub fn eq(&self, other: &Hash) -> bool {
        self.hash.eq(&other.hash)
    }
}

impl Hash {
    /// Size of the hash value in bytes
    pub const SIZE: usize = 32;

    pub fn create(inputs: &[&[u8]]) -> Self {
        let mut hash = Keccak256::default();
        inputs.into_iter().for_each(|v| hash.update(*v));
        let mut ret = Hash {
            hash: [0u8; Self::SIZE]
        };
        hash.finalize_into(&mut ret.hash).unwrap();
        ret
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Self {
                hash: [0u8; Self::SIZE]
            };
            ret.hash.copy_from_slice(data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }
}

/// Represents a secp256k1 public key.
/// This is a "Schrödinger public key", both compressed and uncompressed to save some cycles.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PublicKey {
    key: elliptic_curve::PublicKey<Secp256k1>,
    compressed: Box<[u8]>
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PublicKey {
    pub fn eq(&self, other: &PublicKey) -> bool {
        // Needs to be re-implemented here, because the trait impl is not available in WASM
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

    pub fn to_address(&self) -> Address {
        let uncompressed = self.serialize(false);
        let serialized = Hash::create(&[&uncompressed[1..]]).serialize();
        Address::new(&serialized[12..])
    }

}

impl PublicKey {
    /// Size of the compressed public key in bytes
    pub const SIZE_COMPRESSED: usize = 33;

    /// Size of the uncompressed public key in bytes
    pub const SIZE_UNCOMPRESSED: usize = 65;

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

/// Represents an ECDSA signature based on the secp256k1 curve.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Signature {
    signature: [u8; Self::SIZE],
    recovery: u8,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Signature {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(raw_bytes: &[u8], recovery: u8) -> Signature {
        assert_eq!(raw_bytes.len(), Self::SIZE, "invalid length");
        assert!(recovery <= 1, "invalid recovery bit");
        let mut ret = Self {
            signature: [0u8; Self::SIZE],
            recovery
        };
        ret.signature.copy_from_slice(raw_bytes);
        ret
    }

    /// Signs the given message using the raw private key.
    pub fn sign_message(message: &[u8], private_key: &[u8]) -> Signature {
        let key = SigningKey::from_bytes(private_key)
            .expect("invalid signing key");
        let signature: ECDSASignature = key.sign(message);
        Self::deserialize(signature.to_bytes().as_slice()).expect("signing failed")
    }

    /// Verifies this signature against the given message and a public key (compressed or uncompressed)
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
        compressed[Self::SIZE/2] &= 0x7f;
        compressed[Self::SIZE/2] |= self.recovery << 7;
        compressed.into_boxed_slice()
    }
}

impl Signature {
    /// Size of the signature in bytes
    pub const SIZE: usize = 64;
    
    pub fn deserialize(signature: &[u8]) -> Result<Signature> {
        if signature.len() == Self::SIZE {
            // Read & clear the top-most bit in S
            let mut ret = Signature {
                signature: [0u8; Self::SIZE],
                recovery: if signature[Self::SIZE/2]&0x80 != 0 { 1 } else { 0 }
            };
            ret.signature.copy_from_slice(signature);
            ret.signature[Self::SIZE/2] &= 0x7f;

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

    #[test]
    fn curve_point_test() {

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::Uint8Array;
    use sha3::{Keccak256, digest::DynDigest};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;
    use crate::crypto::{CurvePoint, HalfKey, HalfKeyChallenge, Hash, PublicKey, Signature};

    #[wasm_bindgen]
    impl CurvePoint {
        #[wasm_bindgen(js_name = "from_exponent")]
        pub fn create_from_exponent(exponent: &[u8]) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::from_exponent(exponent))
        }

        #[wasm_bindgen(js_name = "from_str")]
        pub fn create_from_str(str: &str) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::from_str(str))
        }

        #[wasm_bindgen(js_name = "from_peerid_str")]
        pub fn create_from_peerid_str(peer_id: &str) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::from_peerid_str(peer_id))
        }

        pub fn size() -> u32 { Self::SIZE as u32 }
    }

    #[wasm_bindgen]
    impl HalfKey {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_halfkey(data: &[u8]) -> JsResult<HalfKey> {
            ok_or_jserr!(Self::deserialize(data))
        }

        pub fn size() -> u32 { Self::SIZE as u32 }
    }

    #[wasm_bindgen]
    impl HalfKeyChallenge {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_halfkey_challenge(data: &[u8]) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "from_str")]
        pub fn create_from_str(str: &str) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(Self::from_str(str))
        }

        #[wasm_bindgen(js_name = "from_peerid_str")]
        pub fn create_from_peerid_str(peer_id: &str) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(Self::from_peerid_str(peer_id))
        }

        pub fn size() -> u32 { Self::SIZE as u32 }
    }

    #[wasm_bindgen]
    impl Hash {
        #[wasm_bindgen(js_name = "create")]
        pub fn create_from(inputs: Vec<Uint8Array>) -> Self {
            let mut hash = Keccak256::default();
            inputs.into_iter().map(|a| a.to_vec()).for_each(|v| hash.update(&v));

            let mut ret = Hash {
                hash: [0u8; Self::SIZE]
            };
            hash.finalize_into(&mut ret.hash).unwrap();
            ret
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_hash(data: &[u8]) -> JsResult<Hash> {
            ok_or_jserr!(Self::deserialize(data))
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl PublicKey {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_public_key(bytes: &[u8]) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::deserialize(bytes))
        }

        #[wasm_bindgen(js_name = "from_peerid_str")]
        pub fn public_key_from_peerid_str(peer_id: &str) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::from_peerid_str(peer_id))
        }

        pub fn size_compressed() -> u32 {
            Self::SIZE_COMPRESSED as u32
        }

        pub fn size_uncompressed() -> u32 {
            Self::SIZE_UNCOMPRESSED as u32
        }
    }

    #[wasm_bindgen]
    impl Signature {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn deserialize_signature(signature: &[u8]) -> JsResult<Signature> {
            ok_or_jserr!(Signature::deserialize(signature))
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}