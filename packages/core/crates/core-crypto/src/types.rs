use std::ops::Add;
use std::str::FromStr;
use elliptic_curve::ProjectivePoint;
use k256::ecdsa::{SigningKey, Signature as ECDSASignature, VerifyingKey, RecoveryId};
use k256::{AffinePoint, ecdsa, elliptic_curve, NonZeroScalar, Secp256k1};
use k256::ecdsa::signature::hazmat::PrehashVerifier;
use k256::ecdsa::signature::Verifier;
use k256::elliptic_curve::CurveArithmetic;
use k256::elliptic_curve::generic_array::GenericArray;
use k256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use sha3::{Keccak256, digest::DynDigest};
use libp2p_identity::{PeerId, PublicKey as lp2p_PublicKey, secp256k1::PublicKey as lp2p_k256_PublicKey};
use utils_log::warn;
use utils_types::errors::GeneralError;
use utils_types::errors::GeneralError::{Other, ParseError};

use utils_types::primitives::{Address, EthereumChallenge};
use utils_types::traits::{BinarySerializable, PeerIdLike};

use crate::errors::{Result, CryptoError, CryptoError::CalculationError};
use crate::errors::CryptoError::InvalidInputValue;

/// Represent an uncompressed elliptic curve point on the secp256k1 curve
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct CurvePoint {
    affine: AffinePoint
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl CurvePoint {
    pub fn to_address(&self) -> Address {
        let serialized = self.serialize();
        let hash = Hash::create(&[&serialized[1..]]).serialize();
        Address::new(&hash[12..])
    }
}

impl From<PublicKey> for CurvePoint {
    fn from(pubkey: PublicKey) -> Self {
        CurvePoint::from_affine(pubkey.key.as_affine().clone())
    }
}

impl FromStr for CurvePoint {
    type Err = CryptoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(CurvePoint::deserialize(&hex::decode(s).map_err(|_| ParseError)?)?)
    }
}

impl PeerIdLike for CurvePoint {
    fn from_peerid(peer_id: &PeerId) -> utils_types::errors::Result<Self> {
        Ok(CurvePoint::deserialize(&PublicKey::from_peerid(peer_id)?.serialize(false))?)
    }

    fn to_peerid(&self) -> PeerId {
        PublicKey::deserialize(&self.serialize()).unwrap().to_peerid()
    }
}

impl BinarySerializable for CurvePoint {
    const SIZE: usize = 65;

    fn deserialize(bytes: &[u8]) -> utils_types::errors::Result<Self> {
        elliptic_curve::sec1::EncodedPoint::<Secp256k1>::from_bytes(bytes)
            .map_err(|_| ParseError)
            .and_then(|encoded| Option::from(AffinePoint::from_encoded_point(&encoded))
                .ok_or(ParseError))
            .map(|affine| Self { affine })
    }

    fn serialize(&self) -> Box<[u8]> {
        self.affine.to_encoded_point(false).to_bytes()
    }
}

impl CurvePoint {
    /// Creates a curve point from a non-zero scalar.
    pub fn from_exponent(exponent: &[u8]) -> Result<Self> {
        PublicKey::from_privkey(exponent)
            .map(CurvePoint::from)
    }

    pub fn from_affine(affine: AffinePoint) -> Self {
        Self { affine }
    }

    /// Converts the curve point to a representation suitable for calculations
    pub fn to_projective_point(&self) -> ProjectivePoint<Secp256k1> {
        ProjectivePoint::<Secp256k1>::from(&self.affine)
    }
}

/// Natural extension of the Curve Point to the PoR challenge
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Challenge {
    pub curve_point: CurvePoint
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Challenge {
    pub fn to_ethereum_challenge(&self) -> EthereumChallenge {
        EthereumChallenge::new(&self.curve_point.to_address().serialize())
    }
}

impl Challenge {
    pub fn from_hint_and_share(own_share: &HalfKeyChallenge, hint: &HalfKeyChallenge) -> Result<Self> {
        let curve_point: CurvePoint = PublicKey::combine( & [
            & PublicKey::deserialize( & own_share.hkc)?,
            & PublicKey::deserialize( & hint.hkc)?
        ]).into();
        Ok(Self { curve_point })
    }

    pub fn from_own_share_and_half_key(own_share: &HalfKeyChallenge, half_key: &HalfKey) -> Result<Self> {
        let curve_point: CurvePoint = PublicKey::tweak_add(
            &PublicKey::deserialize(&own_share.hkc)?,
            &half_key.serialize()
        ).into();
        Ok(Self { curve_point })
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
}

impl BinarySerializable for HalfKey {
    const SIZE: usize = 32;

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
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

    fn serialize(&self) -> Box<[u8]> {
        self.hkey.into()
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

    pub fn to_address(&self) -> Address {
        PublicKey::deserialize(&self.hkc)
            .expect("invalid half-key")
            .to_address()
    }
}

impl BinarySerializable for HalfKeyChallenge {
    const SIZE: usize = 33; // Size of the compressed secp256k1 point.

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
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

    fn serialize(&self) -> Box<[u8]> {
        self.hkc.into()
    }
}

impl PeerIdLike for HalfKeyChallenge {
    fn from_peerid(peer_id: &PeerId) -> utils_types::errors::Result<Self> {
        HalfKeyChallenge::deserialize(&PublicKey::from_peerid(peer_id)?.serialize(true))
    }

    fn to_peerid(&self) -> PeerId {
        PublicKey::deserialize(&self.hkc)
            .expect("invalid half-key")
            .to_peerid()
    }
}

impl FromStr for HalfKeyChallenge {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::deserialize(&hex::decode(s).map_err(|_| ParseError)?)
    }
}

/// Represents a 256-bit hash value
/// This implementation instantiates the hash via Keccak256 digest.
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
}

impl BinarySerializable for Hash {
    const SIZE: usize = 32; // Defined by Keccak256.

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
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

    fn serialize(&self) -> Box<[u8]> {
        self.hash.into()
    }
}

impl Hash {
    /// Takes all the byte slices and computes hash of their concatenated value.
    /// Uses the Keccak256 digest.
    pub fn create(inputs: &[&[u8]]) -> Self {
        let mut hash = Keccak256::default();
        inputs.into_iter().for_each(|v| hash.update(*v));
        let mut ret = Hash {
            hash: [0u8; Self::SIZE]
        };
        hash.finalize_into(&mut ret.hash).unwrap();
        ret
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
    pub fn to_address(&self) -> Address {
        let uncompressed = self.serialize(false);
        let serialized = Hash::create(&[&uncompressed[1..]]).serialize();
        Address::new(&serialized[12..])
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
}

impl PeerIdLike for PublicKey {
    fn from_peerid(peer_id: &PeerId) -> utils_types::errors::Result<Self> {
        // Workaround for the missing public key API on PeerIds
        let peer_id_str = peer_id.to_base58();
        if peer_id_str.starts_with("16U") {
            // Here we explicitly assume non-RSA PeerId, so that multihash bytes are the actual public key
            let pid = peer_id.to_bytes();
            let (_, mh) = pid.split_at(6);
            Self::deserialize(mh).map_err(|e| Other(e.into()))
        } else if peer_id_str.starts_with("12D") {
            // TODO: support for Ed25519 peer ids needs to be added here
            warn!("Ed25519-based peer id not yet supported");
            Err(ParseError)
        } else {
            // RSA-based peer ID might never going to be supported by HOPR
            warn!("RSA-based peer id encountered");
            Err(ParseError)
        }
    }

    // TODO: Once the enum is made opaque as described in the deprecation note, a workaround must be done.
    // Possibly by constructing directly the protobuf structure and then parsing it via l2p_PublicKey::from_protobuf_encoding
    #[allow(deprecated)]
    fn to_peerid(&self) -> PeerId {
        PeerId::from_public_key(&lp2p_PublicKey::Secp256k1(
            lp2p_k256_PublicKey::decode(&self.compressed)
                .expect("cannot convert this public key to secp256k1 peer id")
        ))
    }
}

impl TryFrom<CurvePoint> for PublicKey {
    type Error = CryptoError;

    fn try_from(value: CurvePoint) -> std::result::Result<Self, Self::Error> {
        let key = elliptic_curve::PublicKey::<Secp256k1>::from_affine(value.affine)
            .map_err(|_| InvalidInputValue)?;
        Ok(Self {
            key,
            compressed: key.to_encoded_point(true).to_bytes()
        })
    }
}

impl PublicKey {
    /// Size of the compressed public key in bytes
    pub const SIZE_COMPRESSED: usize = 33;

    /// Size of the uncompressed public key in bytes
    pub const SIZE_UNCOMPRESSED: usize = 65;

    pub fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
        let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(data)
            .map_err(|_| ParseError)?;
        Ok(PublicKey{
            key,
            compressed: key.to_encoded_point(true).to_bytes()
        })
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

    fn from_raw_signature<R>(msg: &[u8], r: &[u8], s: &[u8], v: u8, recovery_method: R) -> Result<PublicKey>
    where R: Fn(&[u8], &ECDSASignature, RecoveryId) -> std::result::Result<VerifyingKey, ecdsa::Error> {
        let recid = RecoveryId::try_from(v).map_err(|_| ParseError)?;
        let signature = ECDSASignature::from_scalars(
            GenericArray::clone_from_slice(r),GenericArray::clone_from_slice(s))
            .map_err(|_| ParseError)?;
        let recovered_key = recovery_method(
            msg,
            &signature,
            recid
        ).map_err(|_| CalculationError)?;

        Ok(Self::deserialize(&recovered_key.to_encoded_point(false).to_bytes())?)
    }

    pub fn from_signature(msg: &[u8], signature: &Signature) -> Result<PublicKey> {
        Self::from_raw_signature(msg,
                                 &signature.signature[0..Signature::SIZE/2],
                                 &signature.signature[Signature::SIZE/2..],
                                 signature.recovery,
                                 VerifyingKey::recover_from_msg)
    }

    pub fn from_signature_hash(hash: &[u8], signature: &Signature) -> Result<PublicKey> {
        Self::from_raw_signature(hash,
                                 &signature.signature[0..Signature::SIZE/2],
                                 &signature.signature[Signature::SIZE/2..],
                                 signature.recovery,
                                 VerifyingKey::recover_from_prehash)
    }

    /// Sums all given public keys together, creating a new public key.
    /// Panics if reaches infinity (EC identity point), which is an invalid public key.
    pub fn combine(summands: &[&PublicKey]) -> PublicKey {
        // Convert all public keys to EC points in the projective coordinates, which are
        // more efficient for doing the additions. Then finally make in an affine point
        let affine: AffinePoint = summands
            .iter()
            .map(|p| p.key.to_projective())
            .fold(<Secp256k1 as CurveArithmetic>::ProjectivePoint::IDENTITY, |acc, x| acc.add(x))
            .to_affine();

        Self {
            key: elliptic_curve::PublicKey::<Secp256k1>::from_affine(affine)
                .expect("combination results in the ec identity (which is an invalid pub key)"),
            compressed: affine.to_encoded_point(true).to_bytes()
        }
    }

    /// Adds the public key with tweak times generator, producing a new public key.
    /// Panics if reaches infinity (EC identity point), which is an invalid public key.
    pub fn tweak_add(key: &PublicKey, tweak: &[u8]) -> PublicKey {
        let scalar = NonZeroScalar::try_from(tweak)
            .expect("zero tweak provided");

        let new_pk = (key.key.to_projective() + <Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar.as_ref())
            .to_affine();

        Self {
            key: elliptic_curve::PublicKey::<Secp256k1>::from_affine(new_pk)
                .expect("combination results into ec identity (which is an invalid pub key)"),
            compressed: new_pk.to_encoded_point(true).to_bytes()
        }
    }
}

/// Represents an ECDSA signature based on the secp256k1 curve with recoverable public key.
/// This signature encodes the 2-bit recovery information into the
/// upper-most bits of MSB of the S value, which are never used by this ECDSA
/// instantiation over secp256k1.
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

    fn sign<S>(data: &[u8], private_key: &[u8], signing_method: S) -> Signature
    where
        S: Fn(&SigningKey, &[u8]) -> ecdsa::signature::Result<(ECDSASignature, RecoveryId)> {
        let key = SigningKey::from_bytes(private_key.into())
            .expect("invalid signing key");
        let (sig, rec) = signing_method(&key, data)
            .expect("signing failed");

        let mut ret = Signature {
            signature: [0u8; Self::SIZE],
            recovery: rec.to_byte()
        };
        ret.signature.copy_from_slice(&sig.to_vec());
        ret
    }

    /// Signs the given message using the raw private key.
    pub fn sign_message(message: &[u8], private_key: &[u8]) -> Signature {
        Self::sign(message, private_key,|k: &SigningKey, data: &[u8]| { k.sign_recoverable(data) })
    }

    /// Signs the given hash using the raw private key.
    pub fn sign_hash(hash: &[u8], private_key: &[u8]) -> Signature {
        Self::sign(hash,
                   private_key,
                   |k: &SigningKey, data: &[u8]| { k.sign_prehash_recoverable(data) })
    }

    fn verify<V>(&self, message: &[u8], public_key: &[u8], verifier: V) -> bool
    where
        V: Fn(&VerifyingKey, &[u8], &ECDSASignature) -> ecdsa::signature::Result<()> {
        let pub_key = VerifyingKey::from_sec1_bytes(public_key)
            .expect("invalid public key");

        if let Ok(signature) = ECDSASignature::try_from(self.signature.as_slice()) {
            verifier(&pub_key, message, &signature).is_ok()
        } else {
            warn!("un-parseable signature encountered");
            false
        }
    }

    /// Verifies this signature against the given message and a public key (compressed or uncompressed)
    pub fn verify_message(&self, message: &[u8], public_key: &[u8]) -> bool {
        self.verify(message, public_key, |k, msg, sgn| k.verify(msg, sgn))
    }

    /// Verifies this signature against the given message and a public key object
    pub fn verify_message_with_pubkey(&self, message: &[u8], public_key: &PublicKey) -> bool {
        self.verify_message(message, &public_key.serialize(false))
    }

    /// Verifies this signature against the given hash and a public key (compressed or uncompressed)
    pub fn verify_hash(&self, hash: &[u8], public_key: &[u8]) -> bool {
        self.verify(hash, public_key, |k, msg, sgn| k.verify_prehash(msg, sgn))
    }

    /// Verifies this signature against the given message and a public key object
    pub fn verify_hash_with_pubkey(&self, message: &[u8], public_key: &PublicKey) -> bool {
        self.verify_hash(message, &public_key.serialize(false))
    }

    pub fn raw_signature(&self) -> Box<[u8]> {
        self.signature.into()
    }
}

impl BinarySerializable for Signature {
    const SIZE: usize = 64;

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            // Read & clear the top-most bit in S
            let mut ret = Signature {
                signature: [0u8; Self::SIZE],
                recovery: if data[Self::SIZE/2]&0x80 != 0 { 1 } else { 0 }
            };
            ret.signature.copy_from_slice(data);
            ret.signature[Self::SIZE/2] &= 0x7f;

            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut compressed = Vec::from(self.signature);
        compressed[Self::SIZE/2] &= 0x7f;
        compressed[Self::SIZE/2] |= self.recovery << 7;
        compressed.into_boxed_slice()
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use hex_literal::hex;
    use k256::elliptic_curve::CurveArithmetic;
    use k256::{NonZeroScalar, Secp256k1, U256};
    use k256::ecdsa::VerifyingKey;
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    use utils_types::primitives::Address;
    use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};
    use crate::random::random_group_element;

    use crate::types::{Challenge, CurvePoint, HalfKey, HalfKeyChallenge, Hash, PublicKey, Signature};

    const PUBLIC_KEY: [u8; 33] = hex!("021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8");
    const PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    fn signature_signing_test() {
        let msg = b"test12345";
        let sgn = Signature::sign_message(msg, &PRIVATE_KEY);

        assert!(sgn.verify_message(msg, &PUBLIC_KEY));

        let extracted_pk = PublicKey::from_signature(msg, &sgn).unwrap();
        let expected_pk = PublicKey::deserialize(&PUBLIC_KEY).unwrap();
        assert_eq!(expected_pk, extracted_pk, "key extracted from signature does not match");
    }

    #[test]
    fn signature_serialize_test() {
        let msg = b"test000000";
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
    fn public_key_recover_test() {
        let address = Address::from_str("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();

        let r = hex!("bcae4d37e3a1cd869984d1d68f9242291773cd33d26f1e754ecc1a9bfaee7d17");
        let s = hex!("0b755ab5f6375595fc7fc245c45f6598cc873719183733f4c464d63eefd8579b");
        let v = 1u8;

        let hash = hex!("fac7acad27047640b069e8157b61623e3cb6bb86e6adf97151f93817c291f3cf");

        assert_eq!(address, PublicKey::from_raw_signature(&hash, &r, &s, v, VerifyingKey::recover_from_prehash).unwrap().to_address());
    }

    #[test]
    fn public_key_combine_tweak() {
        let (scalar1, point1) = random_group_element();
        let (scalar2, point2) = random_group_element();

        let pk1 = PublicKey::try_from(point1).unwrap();
        let pk2 = PublicKey::try_from(point2).unwrap();

        let sum = PublicKey::combine(&[&pk1, &pk2]);
        let tweak1 = PublicKey::tweak_add(&pk1, &scalar2);
        assert_eq!(sum, tweak1);

        let tweak2 = PublicKey::tweak_add(&pk2, &scalar1);
        assert_eq!(sum, tweak2);
    }

    #[test]
    fn sign_and_recover_test() {
        let msg = hex!("eff80b9f035b1d369c6a60f362ac7c8b8c3b61b76d151d1be535145ccaa3e83e");

        let signature1 = Signature::sign_message(&msg, &PRIVATE_KEY);
        let signature2 = Signature::sign_hash(&msg, &PRIVATE_KEY);

        let pub_key1 = PublicKey::from_privkey(&PRIVATE_KEY).unwrap();
        let pub_key2 = PublicKey::from_signature(&msg, &signature1).unwrap();
        let pub_key3 = PublicKey::from_signature_hash(&msg, &signature2).unwrap();

        assert_eq!(pub_key1, pub_key2, "recovered public key does not match");
        assert_eq!(pub_key1, pub_key3, "recovered public key does not match");

        assert!(signature1.verify_message_with_pubkey(&msg, &pub_key1), "signature 1 verification failed with pub key 1");
        assert!(signature1.verify_message_with_pubkey(&msg, &pub_key2), "signature 1 verification failed with pub key 2");
        assert!(signature1.verify_message_with_pubkey(&msg, &pub_key3), "signature 1 verification failed with pub key 3");

        assert!(signature2.verify_hash_with_pubkey(&msg, &pub_key1), "signature 2 verification failed with pub key 1");
        assert!(signature2.verify_hash_with_pubkey(&msg, &pub_key2), "signature 2 verification failed with pub key 2");
        assert!(signature2.verify_hash_with_pubkey(&msg, &pub_key3), "signature 2 verification failed with pub key 3");
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
    fn public_key_curve_point() {
        let cp1: CurvePoint = PublicKey::deserialize(&PUBLIC_KEY).unwrap().into();
        let cp2 = CurvePoint::deserialize(&cp1.serialize()).unwrap();
        assert_eq!(cp1, cp2);
    }

    #[test]
    fn public_key_from_privkey() {
        let pk1 = PublicKey::from_privkey(&PRIVATE_KEY)
            .expect("failed to convert from private key");
        let pk2 = PublicKey::deserialize(&PUBLIC_KEY)
            .expect("failed to deserialize");

        assert_eq!(pk1, pk2, "failed to match deserialized pub key");
    }

    #[test]
    fn curve_point_test() {
        let scalar = NonZeroScalar::from_uint(U256::from_u8(100)).unwrap();
        let test_point = (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar.as_ref())
            .to_affine();

        let cp1 = CurvePoint::from_str(hex::encode(test_point.to_encoded_point(false).to_bytes()).as_str())
            .unwrap();

        let cp2 = CurvePoint::deserialize(&cp1.serialize())
            .unwrap();

        assert_eq!(cp1, cp2, "failed to match deserialized curve point");

        let pk = PublicKey::from_privkey(&scalar.to_bytes()).unwrap();

        assert_eq!(cp1.to_address(), pk.to_address(), "failed to match curve point address with pub key address");

        let ch1 = Challenge { curve_point: cp1 };
        let ch2 = Challenge { curve_point: cp2 };

        assert_eq!(ch1.to_ethereum_challenge(), ch2.to_ethereum_challenge());
        assert_eq!(ch1, ch2, "failed to match ethereum challenges from curve points");

        // Must be able to create from compressed and uncompressed data
        let scalar2 = NonZeroScalar::from_uint(U256::from_u8(123)).unwrap();
        let test_point2 = (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar2.as_ref())
            .to_affine();
        let uncompressed = test_point2.to_encoded_point(false);
        assert!(!uncompressed.is_compressed(), "given point is compressed");

        let compressed = uncompressed.compress();
        assert!(compressed.is_compressed(), "failed to compress points");

        let cp3 = CurvePoint::deserialize(uncompressed.as_bytes()).unwrap();
        let cp4 = CurvePoint::deserialize(compressed.as_bytes()).unwrap();

        assert_eq!(cp3, cp4, "failed to match curve point from compressed and uncompressed source");
    }

    #[test]
    fn half_key_test() {
        let hk1 = HalfKey::new(&[0u8; HalfKey::SIZE]);
        let hk2 = HalfKey::deserialize(&hk1.serialize()).unwrap();

        assert_eq!(hk1, hk2, "failed to match deserialized half-key");
    }

    #[test]
    fn half_key_challenge_test() {
        let peer_id = PublicKey::deserialize(&PUBLIC_KEY).unwrap().to_peerid();
        let hkc1 = HalfKeyChallenge::from_peerid(&peer_id).unwrap();
        let hkc2 = HalfKeyChallenge::deserialize(&hkc1.serialize()).unwrap();
        assert_eq!(hkc1, hkc2, "failed to match deserialized half key challenge");
        assert_eq!(peer_id, hkc2.to_peerid(), "failed to match half-key challenge peer id");
    }

    #[test]
    fn hash_test() {
        let hash1 = Hash::create(&[b"msg"]);
        assert_eq!("92aef1b955b9de564fc50e31a55b470b0c8cdb931f186485d620729fb03d6f2c", hash1.to_hex(), "hash test vector failed to match");

        let hash2 = Hash::deserialize(&hash1.serialize()).unwrap();
        assert_eq!(hash1, hash2, "failed to match deserialized hash");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::str::FromStr;
    use js_sys::Uint8Array;
    use k256::ecdsa::VerifyingKey;
    use sha3::{Keccak256, digest::DynDigest};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};
    use wasm_bindgen::prelude::*;

    use crate::types::{Challenge, CurvePoint, HalfKey, HalfKeyChallenge, Hash, PublicKey, Signature};

    #[wasm_bindgen]
    impl CurvePoint {
        #[wasm_bindgen(js_name = "from_exponent")]
        pub fn _from_exponent(exponent: &[u8]) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::from_exponent(exponent))
        }

        #[wasm_bindgen(js_name = "from_str")]
        pub fn _from_str(str: &str) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::from_str(str))
        }

        #[wasm_bindgen(js_name = "from_peerid_str")]
        pub fn _from_peerid_str(peer_id: &str) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::from_peerid_str(peer_id))
        }

        #[wasm_bindgen(js_name = "to_peerid_str")]
        pub fn _to_peerid_str(&self) -> String {
            self.to_peerid_str()
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(bytes: &[u8]) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::deserialize(bytes))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &CurvePoint) -> bool {
            self.eq(&other)
        }

        pub fn size() -> u32 { Self::SIZE as u32 }
    }

    #[wasm_bindgen]
    impl Challenge {
        #[wasm_bindgen(js_name = "from_hint_and_share")]
        pub fn _from_hint_and_share(own_share: &HalfKeyChallenge, hint: &HalfKeyChallenge) -> JsResult<Challenge> {
            ok_or_jserr!(Self::from_hint_and_share(own_share, hint))
        }

        #[wasm_bindgen(js_name = "from_own_share_and_half_key")]
        pub fn _from_own_share_and_half_key(own_share: &HalfKeyChallenge, half_key: &HalfKey) -> JsResult<Challenge> {
            ok_or_jserr!(Self::from_own_share_and_half_key(own_share, half_key))
        }
    }

    #[wasm_bindgen]
    impl HalfKey {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<HalfKey> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> HalfKey {
            self.clone()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &HalfKey) -> bool {
            self.eq(other)
        }

        pub fn size() -> u32 { Self::SIZE as u32 }
    }

    #[wasm_bindgen]
    impl HalfKeyChallenge {
        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &HalfKeyChallenge) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> HalfKeyChallenge {
            self.clone()
        }

        #[wasm_bindgen(js_name = "to_peerid_str")]
        pub fn _to_peerid_str(&self) -> String {
            self.to_peerid_str()
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "from_str")]
        pub fn _from_str(str: &str) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(Self::from_str(str))
        }

        #[wasm_bindgen(js_name = "from_peerid_str")]
        pub fn _from_peerid_str(peer_id: &str) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(Self::from_peerid_str(peer_id))
        }

        pub fn size() -> u32 { Self::SIZE as u32 }
    }

    #[wasm_bindgen]
    impl Hash {
        #[wasm_bindgen(js_name = "create")]
        pub fn _create(inputs: Vec<Uint8Array>) -> Self {
            let mut hash = Keccak256::default();
            inputs.into_iter().map(|a| a.to_vec()).for_each(|v| hash.update(&v));

            let mut ret = Hash {
                hash: [0u8; Self::SIZE]
            };
            hash.finalize_into(&mut ret.hash).unwrap();
            ret
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Hash> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Hash) -> bool {
            self.eq(other)
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl PublicKey {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(bytes: &[u8]) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::deserialize(bytes))
        }

        #[wasm_bindgen(js_name = "from_peerid_str")]
        pub fn _from_peerid_str(peer_id: &str) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::from_peerid_str(peer_id))
        }

        #[wasm_bindgen(js_name = "to_peerid_str")]
        pub fn _to_peerid_str(&self) -> String {
            self.to_peerid_str()
        }

        #[wasm_bindgen(js_name = "from_signature")]
        pub fn _from_raw_signature(hash: &[u8], r: &[u8], s: &[u8], v: u8) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::from_raw_signature(hash, r, s, v, VerifyingKey::recover_from_msg))
        }

        #[wasm_bindgen(js_name = "from_privkey")]
        pub fn _from_privkey(private_key: &[u8]) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::from_privkey(private_key))
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &PublicKey) -> bool {
            self.eq(other)
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
        pub fn _deserialize(signature: &[u8]) -> JsResult<Signature> {
            ok_or_jserr!(Signature::deserialize(signature))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}