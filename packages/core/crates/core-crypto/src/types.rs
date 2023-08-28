use crate::shared_keys::Scalar as _;
use curve25519_dalek::{
    edwards::{CompressedEdwardsY, EdwardsPoint},
    montgomery::MontgomeryPoint,
};
use elliptic_curve::{
    hash2curve::{ExpandMsgXmd, GroupDigest},
    sec1::EncodedPoint,
    NonZeroScalar, ProjectivePoint,
};
use k256::{
    ecdsa::{
        self,
        signature::{hazmat::PrehashVerifier, Verifier},
        RecoveryId, Signature as ECDSASignature, SigningKey, VerifyingKey,
    },
    elliptic_curve::{
        self,
        generic_array::GenericArray,
        sec1::{FromEncodedPoint, ToEncodedPoint},
        CurveArithmetic,
    },
    AffinePoint, Scalar, Secp256k1,
};

use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    ops::Add,
    str::FromStr,
};

use crate::{
    errors::{
        CryptoError::{self, CalculationError, InvalidInputValue},
        Result,
    },
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    primitives::{DigestLike, EthDigest},
    random::random_group_element,
};
use utils_log::warn;
use utils_types::{
    errors::GeneralError::{self, ParseError},
    primitives::{Address, EthereumChallenge, U256},
    traits::{BinarySerializable, PeerIdLike, ToHex},
};

/// Extend support for arbitrary array sizes in serde
///
/// Array of arbitrary sizes are not supported in serde due to backwards compatibility.
/// Read more in: https://github.com/serde-rs/serde/issues/1937
mod arrays {
    use std::{convert::TryInto, marker::PhantomData};

    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub fn serialize<S: Serializer, T: Serialize, const N: usize>(data: &[T; N], ser: S) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_tuple(N)?;
        for item in data {
            s.serialize_element(item)?;
        }
        s.end()
    }

    struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(&format!("an array of length {}", N))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // can be optimized using MaybeUninit
            let mut data = Vec::with_capacity(N);
            for _ in 0..N {
                match (seq.next_element())? {
                    Some(val) => data.push(val),
                    None => return Err(serde::de::Error::invalid_length(N, &self)),
                }
            }
            match data.try_into() {
                Ok(arr) => Ok(arr),
                Err(_) => unreachable!(),
            }
        }
    }
    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}

/// Represent an uncompressed elliptic curve point on the secp256k1 curve
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct CurvePoint {
    affine: AffinePoint,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl CurvePoint {
    /// Converts the uncompressed representation of the curve point to Ethereum address.
    pub fn to_address(&self) -> Address {
        let serialized = self.to_bytes();
        let hash = Hash::create(&[&serialized[1..]]).to_bytes();
        Address::new(&hash[12..])
    }
}

impl Default for CurvePoint {
    fn default() -> Self {
        CurvePoint::from_exponent(&U256::one().to_bytes()).unwrap()
    }
}

impl From<PublicKey> for CurvePoint {
    fn from(pubkey: PublicKey) -> Self {
        (*pubkey.key.as_affine()).into()
    }
}

impl From<&PublicKey> for CurvePoint {
    fn from(pubkey: &PublicKey) -> Self {
        (*pubkey.key.as_affine()).into()
    }
}

impl From<AffinePoint> for CurvePoint {
    fn from(affine: AffinePoint) -> Self {
        Self { affine }
    }
}

impl From<HalfKeyChallenge> for CurvePoint {
    fn from(value: HalfKeyChallenge) -> Self {
        CurvePoint::from_bytes(&value.hkc).unwrap()
    }
}

impl FromStr for CurvePoint {
    type Err = CryptoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(CurvePoint::from_bytes(&hex::decode(s).map_err(|_| ParseError)?)?)
    }
}

impl From<CurvePoint> for AffinePoint {
    fn from(value: CurvePoint) -> Self {
        value.affine
    }
}

impl BinarySerializable for CurvePoint {
    const SIZE: usize = 65; // Stores uncompressed data

    fn from_bytes(bytes: &[u8]) -> utils_types::errors::Result<Self> {
        // Deserializes both compressed and uncompressed
        elliptic_curve::sec1::EncodedPoint::<Secp256k1>::from_bytes(bytes)
            .map_err(|_| ParseError)
            .and_then(|encoded| Option::from(AffinePoint::from_encoded_point(&encoded)).ok_or(ParseError))
            .map(|affine| Self { affine })
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.affine.to_encoded_point(false).to_bytes()
    }
}

impl CurvePoint {
    /// Size of the point if serialized via `serialize_compressed`.
    pub const SIZE_COMPRESSED: usize = 33;

    /// Creates a curve point from a non-zero scalar.
    /// The given exponent must represent a non-zero scalar and must result into
    /// a secp256k1 identity point.
    pub fn from_exponent(exponent: &[u8]) -> Result<Self> {
        PublicKey::from_privkey(exponent).map(CurvePoint::from)
    }

    /// Converts the curve point to a representation suitable for calculations.
    pub fn to_projective_point(&self) -> ProjectivePoint<Secp256k1> {
        self.affine.into()
    }

    /// Serializes the curve point into a compressed form. This is a cheap operation.
    pub fn serialize_compressed(&self) -> Box<[u8]> {
        self.affine.to_encoded_point(true).to_bytes()
    }

    /// Sums all given curve points together, creating a new curve point.
    pub fn combine(summands: &[&CurvePoint]) -> CurvePoint {
        // Convert all public keys to EC points in the projective coordinates, which are
        // more efficient for doing the additions. Then finally make in an affine point
        let affine: AffinePoint = summands
            .iter()
            .map(|p| ProjectivePoint::<Secp256k1>::from(p.affine))
            .fold(<Secp256k1 as CurveArithmetic>::ProjectivePoint::IDENTITY, |acc, x| {
                acc.add(x)
            })
            .to_affine();

        affine.into()
    }
}

/// Natural extension of the Curve Point to the Proof-of-Relay challenge.
/// Proof-of-Relay challenge is a secp256k1 curve point.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Challenge {
    pub curve_point: CurvePoint,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Challenge {
    /// Converts the PoR challenge to an Ethereum challenge.
    /// This is a one-way (lossy) operation, since the corresponding curve point is hashed
    /// with the hash value then truncated.
    pub fn to_ethereum_challenge(&self) -> EthereumChallenge {
        EthereumChallenge::new(&self.curve_point.to_address().to_bytes())
    }
}

impl From<Challenge> for EthereumChallenge {
    fn from(challenge: Challenge) -> Self {
        challenge.to_ethereum_challenge()
    }
}

impl Challenge {
    /// Obtains the PoR challenge by adding the two EC points represented by the half-key challenges
    pub fn from_hint_and_share(own_share: &HalfKeyChallenge, hint: &HalfKeyChallenge) -> Result<Self> {
        let curve_point: CurvePoint = PublicKey::combine(&[
            &PublicKey::from_bytes(&own_share.hkc)?,
            &PublicKey::from_bytes(&hint.hkc)?,
        ])
        .into();
        Ok(curve_point.into())
    }

    /// Obtains the PoR challenge by converting the given HalfKey into a secp256k1 point and
    /// adding it with the given HalfKeyChallenge (which already represents a secp256k1 point).
    pub fn from_own_share_and_half_key(own_share: &HalfKeyChallenge, half_key: &HalfKey) -> Result<Self> {
        Self::from_hint_and_share(own_share, &half_key.to_challenge())
    }
}

impl From<CurvePoint> for Challenge {
    fn from(curve_point: CurvePoint) -> Self {
        Self { curve_point }
    }
}

impl From<Response> for Challenge {
    fn from(response: Response) -> Self {
        response.to_challenge()
    }
}

impl BinarySerializable for Challenge {
    const SIZE: usize = PublicKey::SIZE_COMPRESSED;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        // Accepts both compressed and uncompressed points
        CurvePoint::from_bytes(data).map(|curve_point| Challenge { curve_point })
    }

    fn to_bytes(&self) -> Box<[u8]> {
        // Serializes only compressed points
        self.curve_point.serialize_compressed()
    }
}

/// Represents a half-key used for Proof of Relay
/// Half-key is equivalent to a non-zero scalar in the field used by secp256k1, but the type
/// itself does not validate nor enforce this fact,
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HalfKey {
    hkey: [u8; Self::SIZE],
}

impl Default for HalfKey {
    fn default() -> Self {
        let mut ret = Self {
            hkey: [0u8; Self::SIZE],
        };
        ret.hkey.copy_from_slice(
            NonZeroScalar::<Secp256k1>::from_uint(1u16.into())
                .unwrap()
                .to_bytes()
                .as_slice(),
        );
        ret
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl HalfKey {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(half_key: &[u8]) -> Self {
        assert_eq!(half_key.len(), Self::SIZE, "invalid length");
        let mut ret = Self::default();
        ret.hkey.copy_from_slice(half_key);
        ret
    }

    /// Converts the non-zero scalar represented by this half-key into the half-key challenge.
    /// This operation naturally enforces the underlying scalar to be non-zero.
    pub fn to_challenge(&self) -> HalfKeyChallenge {
        CurvePoint::from_exponent(&self.hkey)
            .map(|cp| HalfKeyChallenge::new(&cp.serialize_compressed()))
            .expect("invalid public key")
    }
}

impl BinarySerializable for HalfKey {
    const SIZE: usize = 32;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = HalfKey::default();
            ret.hkey.copy_from_slice(data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.hkey.into()
    }
}

/// Represents a challenge for the half-key in Proof of Relay.
/// Half-key challenge is equivalent to a secp256k1 curve point.
/// Therefore, HalfKeyChallenge can be obtained from a HalfKey.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HalfKeyChallenge {
    #[serde(with = "arrays")]
    hkc: [u8; Self::SIZE],
}

impl Display for HalfKeyChallenge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.hkc))
    }
}

impl Default for HalfKeyChallenge {
    fn default() -> Self {
        // Note that the default HalfKeyChallenge is the identity point on secp256k1, therefore
        // will fail all public key checks, which is intended.
        let mut ret = Self { hkc: [0u8; Self::SIZE] };
        if let Some(b) = ret.hkc.last_mut() {
            *b = 1;
        }
        ret
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl HalfKeyChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(half_key_challenge: &[u8]) -> Self {
        assert_eq!(half_key_challenge.len(), Self::SIZE, "invalid length");
        let mut ret = Self::default();
        ret.hkc.copy_from_slice(half_key_challenge);
        ret
    }

    pub fn to_address(&self) -> Address {
        PublicKey::from_bytes(&self.hkc).expect("invalid half-key").to_address()
    }
}

impl BinarySerializable for HalfKeyChallenge {
    const SIZE: usize = PublicKey::SIZE_COMPRESSED; // Size of the compressed secp256k1 point.

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = HalfKeyChallenge::default();
            ret.hkc.copy_from_slice(data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.hkc.into()
    }
}

impl FromStr for HalfKeyChallenge {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_bytes(&hex::decode(s).map_err(|_| ParseError)?)
    }
}

impl From<HalfKey> for HalfKeyChallenge {
    fn from(half_key: HalfKey) -> Self {
        half_key.to_challenge()
    }
}

/// Represents an Ethereum 256-bit hash value
/// This implementation instantiates the hash via Keccak256 digest.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Hash {
    hash: [u8; Self::SIZE],
}

impl Default for Hash {
    fn default() -> Self {
        Self {
            hash: [0u8; Self::SIZE],
        }
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_hex().as_str())
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Hash {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(hash: &[u8]) -> Self {
        assert_eq!(hash.len(), Self::SIZE, "invalid length");
        let mut ret = Hash::default();
        ret.hash.copy_from_slice(hash);
        ret
    }

    /// Convenience method that creates a new hash by hashing this.
    pub fn hash(&self) -> Self {
        Self::create(&[&self.hash])
    }
}

impl BinarySerializable for Hash {
    const SIZE: usize = 32; // Defined by Keccak256.

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Self {
                hash: [0u8; Self::SIZE],
            };
            ret.hash.copy_from_slice(data);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.hash.into()
    }
}

impl Hash {
    /// Takes all the byte slices and computes hash of their concatenated value.
    /// Uses the Keccak256 digest.
    pub fn create(inputs: &[&[u8]]) -> Self {
        let mut hash = EthDigest::default();
        inputs.iter().for_each(|v| hash.update(v));
        let mut ret = Hash {
            hash: [0u8; Self::SIZE],
        };
        hash.finalize_into(&mut ret.hash);
        ret
    }
}

impl TryFrom<[u8; 32]> for Hash {
    type Error = GeneralError;

    fn try_from(value: [u8; 32]) -> std::result::Result<Self, Self::Error> {
        Hash::from_bytes(&value)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(value: Hash) -> Self {
        value.hash
    }
}

/// Represents an Ed25519 public key.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct OffchainPublicKey {
    compressed: CompressedEdwardsY,
}

impl BinarySerializable for OffchainPublicKey {
    const SIZE: usize = 32;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            Ok(Self {
                compressed: CompressedEdwardsY::from_slice(data).map_err(|_| ParseError)?,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.compressed.to_bytes().into()
    }
}

impl TryFrom<[u8; OffchainPublicKey::SIZE]> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: [u8; OffchainPublicKey::SIZE]) -> std::result::Result<Self, Self::Error> {
        OffchainPublicKey::from_bytes(&value)
    }
}

impl PeerIdLike for OffchainPublicKey {
    fn from_peerid(peer_id: &PeerId) -> utils_types::errors::Result<Self> {
        let mh = peer_id.as_ref();
        if mh.code() == 0 {
            libp2p_identity::PublicKey::try_decode_protobuf(mh.digest())
                .map_err(|_| ParseError)
                .and_then(|pk| pk.try_into_ed25519().map(|p| p.to_bytes()).map_err(|_| ParseError))
                .and_then(|pk| CompressedEdwardsY::from_slice(&pk).map_err(|_| ParseError))
                .map(|compressed| Self { compressed })
        } else {
            Err(ParseError)
        }
    }

    fn to_peerid(&self) -> PeerId {
        let k = libp2p_identity::ed25519::PublicKey::try_from_bytes(self.compressed.as_bytes()).unwrap();
        PeerId::from_public_key(&k.into())
    }
}

impl Display for OffchainPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl OffchainPublicKey {
    pub fn from_privkey(private_key: &[u8]) -> Result<Self> {
        let mut pk: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.try_into().map_err(|_| InvalidInputValue)?;
        let sk = libp2p_identity::ed25519::SecretKey::try_from_bytes(&mut pk).map_err(|_| InvalidInputValue)?;
        let kp: libp2p_identity::ed25519::Keypair = sk.into();
        Ok(Self {
            compressed: CompressedEdwardsY::from_slice(&kp.public().to_bytes()).map_err(|_| ParseError)?,
        })
    }
}

impl From<&OffchainPublicKey> for EdwardsPoint {
    fn from(value: &OffchainPublicKey) -> Self {
        value.compressed.decompress().unwrap()
    }
}

impl From<&OffchainPublicKey> for MontgomeryPoint {
    fn from(value: &OffchainPublicKey) -> Self {
        value.compressed.decompress().unwrap().to_montgomery()
    }
}

/// Represents a secp256k1 public key.
/// This is a "Schrödinger public key", both compressed and uncompressed to save some cycles.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PublicKey {
    key: elliptic_curve::PublicKey<Secp256k1>,
    pub(crate) compressed: EncodedPoint<Secp256k1>,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PublicKey {
    /// Generates a new random public key.
    /// Because the corresponding private key is discarded, this might be useful only for testing purposes.
    pub fn random() -> Self {
        let (_, pk) = Self::random_keypair();
        pk
    }

    /// Converts the public key to an Ethereum address
    pub fn to_address(&self) -> Address {
        let uncompressed = self.to_bytes(false);
        let serialized = Hash::create(&[&uncompressed[1..]]).to_bytes();
        Address::new(&serialized[12..])
    }

    /// Serializes the public key to a binary form.
    pub fn to_bytes(&self, compressed: bool) -> Box<[u8]> {
        if compressed {
            self.compressed.to_bytes()
        } else {
            self.key.as_affine().to_encoded_point(false).to_bytes()
        }
    }

    /// Serializes the public key to a binary form and converts it to hexadecimal string representation.
    pub fn to_hex(&self, compressed: bool) -> String {
        let offset = if compressed { 0 } else { 1 };
        format!("0x{}", hex::encode(&self.to_bytes(compressed)[offset..]))
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex(true))
    }
}

impl TryFrom<CurvePoint> for PublicKey {
    type Error = CryptoError;

    fn try_from(value: CurvePoint) -> std::result::Result<Self, Self::Error> {
        let key = elliptic_curve::PublicKey::<Secp256k1>::from_affine(value.affine).map_err(|_| InvalidInputValue)?;
        Ok(Self {
            key,
            compressed: key.to_encoded_point(true),
        })
    }
}

impl PublicKey {
    /// Size of the compressed public key in bytes
    pub const SIZE_COMPRESSED: usize = 33;

    pub const SIZE_UNCOMPRESSED_PLAIN: usize = 64;

    /// Size of the uncompressed public key in bytes
    pub const SIZE_UNCOMPRESSED: usize = 65;

    /// Generates new random keypair (private key, public key)
    pub fn random_keypair() -> ([u8; 32], PublicKey) {
        let (private, cp) = random_group_element();
        (private, cp.try_into().unwrap())
    }

    pub fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        match data.len() {
            Self::SIZE_UNCOMPRESSED => {
                // already has 0x04 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(data).map_err(|_| ParseError)?;

                Ok(PublicKey {
                    key,
                    compressed: key.to_encoded_point(true),
                })
            }
            Self::SIZE_UNCOMPRESSED_PLAIN => {
                // add 0x04 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(&[&[4u8], data].concat())
                    .map_err(|_| ParseError)?;

                Ok(PublicKey {
                    key,
                    compressed: key.to_encoded_point(true),
                })
            }
            Self::SIZE_COMPRESSED => {
                // has either 0x02 or 0x03 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(data).map_err(|_| ParseError)?;

                Ok(PublicKey {
                    key,
                    compressed: key.to_encoded_point(true),
                })
            }
            _ => Err(ParseError),
        }
    }

    pub fn from_privkey(private_key: &[u8]) -> Result<PublicKey> {
        let secret_scalar = NonZeroScalar::<Secp256k1>::try_from(private_key).map_err(|_| ParseError)?;

        let key = elliptic_curve::PublicKey::<Secp256k1>::from_secret_scalar(&secret_scalar);
        Ok(PublicKey {
            key,
            compressed: key.to_encoded_point(true),
        })
    }

    fn from_raw_signature<R>(msg: &[u8], r: &[u8], s: &[u8], v: u8, recovery_method: R) -> Result<PublicKey>
    where
        R: Fn(&[u8], &ECDSASignature, RecoveryId) -> std::result::Result<VerifyingKey, ecdsa::Error>,
    {
        let recid = RecoveryId::try_from(v).map_err(|_| ParseError)?;
        let signature =
            ECDSASignature::from_scalars(GenericArray::clone_from_slice(r), GenericArray::clone_from_slice(s))
                .map_err(|_| ParseError)?;
        let recovered_key = recovery_method(msg, &signature, recid).map_err(|_| CalculationError)?;

        Ok(Self::from_bytes(&recovered_key.to_encoded_point(false).to_bytes())?)
    }

    pub fn from_signature(msg: &[u8], signature: &Signature) -> Result<PublicKey> {
        Self::from_raw_signature(
            msg,
            &signature.signature[0..Signature::SIZE / 2],
            &signature.signature[Signature::SIZE / 2..],
            signature.recovery,
            VerifyingKey::recover_from_msg,
        )
    }

    pub fn from_signature_hash(hash: &[u8], signature: &Signature) -> Result<PublicKey> {
        Self::from_raw_signature(
            hash,
            &signature.signature[0..Signature::SIZE / 2],
            &signature.signature[Signature::SIZE / 2..],
            signature.recovery,
            VerifyingKey::recover_from_prehash,
        )
    }

    /// Sums all given public keys together, creating a new public key.
    /// Panics if reaches infinity (EC identity point), which is an invalid public key.
    pub fn combine(summands: &[&PublicKey]) -> PublicKey {
        let cps = summands.iter().map(|pk| CurvePoint::from(*pk)).collect::<Vec<_>>();
        let cps_ref = cps.iter().collect::<Vec<_>>();
        CurvePoint::combine(&cps_ref)
            .try_into()
            .expect("combination results in the ec identity (which is an invalid pub key)")
    }

    /// Adds the given public key with `tweak` times secp256k1 generator, producing a new public key.
    /// Panics if reaches infinity (EC identity point), which is an invalid public key.
    pub fn tweak_add(key: &PublicKey, tweak: &[u8]) -> PublicKey {
        let scalar = NonZeroScalar::<Secp256k1>::try_from(tweak).expect("zero tweak provided");

        let new_pk = (key.key.to_projective()
            + <Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar.as_ref())
        .to_affine();

        Self {
            key: elliptic_curve::PublicKey::<Secp256k1>::from_affine(new_pk)
                .expect("combination results into ec identity (which is an invalid pub key)"),
            compressed: new_pk.to_encoded_point(true),
        }
    }
}

impl From<elliptic_curve::PublicKey<Secp256k1>> for PublicKey {
    fn from(key: elliptic_curve::PublicKey<Secp256k1>) -> Self {
        Self {
            key,
            compressed: key.to_encoded_point(true),
        }
    }
}

impl From<&PublicKey> for k256::ProjectivePoint {
    fn from(value: &PublicKey) -> Self {
        value.key.to_projective()
    }
}

/// Represents a compressed serializable extension of the `PublicKey` using the secp256k1 curve.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CompressedPublicKey(pub PublicKey);

impl BinarySerializable for CompressedPublicKey {
    const SIZE: usize = PublicKey::SIZE_COMPRESSED;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        PublicKey::from_bytes(data).map(CompressedPublicKey)
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.0.to_bytes(true)
    }
}

impl From<PublicKey> for CompressedPublicKey {
    fn from(value: PublicKey) -> Self {
        Self(value)
    }
}

impl From<&CompressedPublicKey> for k256::ProjectivePoint {
    fn from(value: &CompressedPublicKey) -> Self {
        (&value.0).into()
    }
}

impl CompressedPublicKey {
    pub fn to_address(&self) -> Address {
        self.0.to_address()
    }
}

/// Bundles values given to the smart contract to prove that a ticket is a win.
///
/// The VRF is thereby needed because it generates on-demand determinitstic
/// entropy that can only be derived by the ticket redeemer.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VrfParameters {
    /// the pseudo-random point
    pub v: CurvePoint,
    pub h: Scalar,
    pub s: Scalar,
    /// helper value for smart contract
    pub h_v: CurvePoint,
    /// helper value for smart contract
    pub s_b: CurvePoint,
}

impl std::fmt::Display for VrfParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v_encoded = self.v.affine.to_encoded_point(false);
        let h_v_encoded = self.h_v.affine.to_encoded_point(false);
        let s_b_encoded = self.s_b.affine.to_encoded_point(false);
        f.debug_struct("VrfParameters")
            .field(
                "V",
                &format!(
                    "({},{})",
                    hex::encode(v_encoded.x().unwrap()),
                    hex::encode(v_encoded.y().unwrap())
                ),
            )
            .field("h", &hex::encode(self.h.to_bytes()))
            .field("s", &hex::encode(self.s.to_bytes()))
            .field(
                "h_v",
                &format!(
                    "({},{})",
                    hex::encode(h_v_encoded.x().unwrap()),
                    hex::encode(h_v_encoded.y().unwrap())
                ),
            )
            .field(
                "s_b",
                &format!(
                    "({},{})",
                    hex::encode(s_b_encoded.x().unwrap()),
                    hex::encode(s_b_encoded.y().unwrap())
                ),
            )
            .finish()
    }
}

impl BinarySerializable for VrfParameters {
    const SIZE: usize = CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE + CurvePoint::SIZE;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            Ok(VrfParameters {
                v: CurvePoint::from_bytes(&data[0..CurvePoint::SIZE]).unwrap(),
                h: k256::Scalar::from_bytes(&data[CurvePoint::SIZE..CurvePoint::SIZE + 32]).unwrap(),
                s: k256::Scalar::from_bytes(&data[CurvePoint::SIZE + 32..CurvePoint::SIZE + 32 + 32]).unwrap(),
                h_v: CurvePoint::from_bytes(
                    &data[CurvePoint::SIZE + 32 + 32..CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE],
                )
                .unwrap(),
                s_b: CurvePoint::from_bytes(
                    &data[CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE
                        ..CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE + CurvePoint::SIZE],
                )
                .unwrap(),
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.v.to_bytes());
        ret.extend_from_slice(&self.h.to_bytes());
        ret.extend_from_slice(&self.s.to_bytes());
        ret.extend_from_slice(&self.h_v.to_bytes());
        ret.extend_from_slice(&self.s_b.to_bytes());
        ret.into_boxed_slice()
    }
}

impl VrfParameters {
    /// Verifies that VRF values are valid
    pub fn verify<const T: usize>(&self, creator: &Address, msg: &[u8; T], dst: &[u8]) -> Result<()> {
        let cap_b =
            Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&creator.to_bytes(), msg], &[dst]).unwrap();

        // Check point witness
        if self.s_b.to_projective_point() != cap_b * self.s {
            return Err(CalculationError);
        }

        // Check point witness
        if self.h_v.to_projective_point() != self.v.to_projective_point() * self.h {
            return Err(CalculationError);
        }

        let r_v: ProjectivePoint<Secp256k1> = cap_b * self.s - self.v.to_projective_point() * self.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
            &[
                &creator.to_bytes(),
                &self.v.to_bytes()[1..],
                &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
                msg,
            ],
            &[dst],
        )
        .unwrap();

        if h_check != self.h {
            return Err(CalculationError);
        }

        Ok(())
    }
}

/// Contains a response upon ticket acknowledgement
/// It is equivalent to a non-zero secret scalar on secp256k1 (EC private key).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Response {
    response: [u8; Self::SIZE],
}

impl Default for Response {
    fn default() -> Self {
        let mut ret = Self {
            response: [0u8; Self::SIZE],
        };
        ret.response.copy_from_slice(
            NonZeroScalar::<Secp256k1>::from_uint(1u16.into())
                .unwrap()
                .to_bytes()
                .as_slice(),
        );
        ret
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_hex().as_str())
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Response {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), Self::SIZE);
        let mut ret = Self::default();
        ret.response.copy_from_slice(data);
        ret
    }

    /// Converts this response to the PoR challenge by turning the non-zero scalar
    /// represented by this response into a secp256k1 curve point (public key)
    pub fn to_challenge(&self) -> Challenge {
        Challenge {
            curve_point: CurvePoint::from_exponent(&self.response)
                .expect("response represents an invalid non-zero scalar"),
        }
    }
}

impl Response {
    /// Derives the response from two half-keys.
    /// This is done by adding the two non-zero scalars that the given half-keys represent.
    pub fn from_half_keys(first: &HalfKey, second: &HalfKey) -> Result<Self> {
        let res = NonZeroScalar::<Secp256k1>::try_from(HalfKey::to_bytes(first).as_ref())
            .and_then(|s1| {
                NonZeroScalar::<Secp256k1>::try_from(second.to_bytes().as_ref()).map(|s2| s1.as_ref() + s2.as_ref())
            })
            .map_err(|_| CalculationError)?; // One of the scalars was 0

        Ok(Response::new(res.to_bytes().as_slice()))
    }
}

impl BinarySerializable for Response {
    const SIZE: usize = 32;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            Ok(Response::new(data))
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.response.into()
    }
}

/// Represents an EdDSA signature using Ed25519 Edwards curve.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct OffchainSignature {
    signature: ed25519_dalek::Signature,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl OffchainSignature {
    pub fn sign_message(msg: &[u8], signing_keypair: &OffchainKeypair) -> Self {
        let expanded_sk = ed25519_dalek::ExpandedSecretKey::from(
            &ed25519_dalek::SecretKey::from_bytes(signing_keypair.secret().as_ref()).expect("invalid private key"),
        );

        let verifying = ed25519_dalek::PublicKey::from_bytes(signing_keypair.public().compressed.as_bytes()).unwrap();
        Self {
            signature: expanded_sk.sign(msg, &verifying),
        }
    }

    pub fn verify_message(&self, msg: &[u8], public_key: &OffchainPublicKey) -> bool {
        let pk = ed25519_dalek::PublicKey::from_bytes(public_key.compressed.as_bytes()).unwrap();
        pk.verify_strict(msg, &self.signature).is_ok()
    }
}

impl BinarySerializable for OffchainSignature {
    const SIZE: usize = ed25519_dalek::Signature::BYTE_SIZE;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        ed25519_dalek::Signature::from_bytes(data)
            .map_err(|_| ParseError)
            .map(|signature| Self { signature })
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.signature.to_bytes().into()
    }
}

impl TryFrom<([u8; 32], [u8; 32])> for OffchainSignature {
    type Error = GeneralError;

    fn try_from(value: ([u8; 32], [u8; 32])) -> std::result::Result<Self, Self::Error> {
        let mut sig = [0u8; OffchainSignature::SIZE];

        sig[0..32].copy_from_slice(&value.0);
        sig[32..64].copy_from_slice(&value.1);

        OffchainSignature::from_bytes(&sig)
    }
}

/// Represents an ECDSA signature based on the secp256k1 curve with recoverable public key.
/// This signature encodes the 2-bit recovery information into the
/// upper-most bits of MSB of the S value, which are never used by this ECDSA
/// instantiation over secp256k1.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Signature {
    #[serde(with = "arrays")]
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
            recovery,
        };
        ret.signature.copy_from_slice(raw_bytes);
        ret
    }

    fn sign<S>(data: &[u8], private_key: &[u8], signing_method: S) -> Signature
    where
        S: Fn(&SigningKey, &[u8]) -> ecdsa::signature::Result<(ECDSASignature, RecoveryId)>,
    {
        let key = SigningKey::from_bytes(private_key.into()).expect("invalid signing key");
        let (sig, rec) = signing_method(&key, data).expect("signing failed");

        let mut ret = Signature {
            signature: [0u8; Self::SIZE],
            recovery: rec.to_byte(),
        };
        ret.signature.copy_from_slice(&sig.to_vec());
        ret
    }

    /// Signs the given message using the chain private key.
    pub fn sign_message(message: &[u8], chain_keypair: &ChainKeypair) -> Signature {
        Self::sign(
            message,
            chain_keypair.secret().as_ref(),
            |k: &SigningKey, data: &[u8]| k.sign_recoverable(data),
        )
    }

    /// Signs the given hash using the raw private key.
    pub fn sign_hash(hash: &[u8], chain_keypair: &ChainKeypair) -> Signature {
        Self::sign(hash, chain_keypair.secret().as_ref(), |k: &SigningKey, data: &[u8]| {
            k.sign_prehash_recoverable(data)
        })
    }

    fn verify<V>(&self, message: &[u8], public_key: &[u8], verifier: V) -> bool
    where
        V: Fn(&VerifyingKey, &[u8], &ECDSASignature) -> ecdsa::signature::Result<()>,
    {
        let pub_key = VerifyingKey::from_sec1_bytes(public_key).expect("invalid public key");

        if let Ok(signature) = ECDSASignature::try_from(self.signature.as_slice()) {
            verifier(&pub_key, message, &signature).is_ok()
        } else {
            warn!("un-parseable signature encountered");
            false
        }
    }

    /// Verifies this signature against the given message and a public key object
    pub fn verify_message(&self, message: &[u8], public_key: &PublicKey) -> bool {
        self.verify(message, &public_key.to_bytes(false), |k, msg, sgn| k.verify(msg, sgn))
    }

    /// Verifies this signature against the given hash and a public key object
    pub fn verify_hash(&self, hash: &[u8], public_key: &PublicKey) -> bool {
        self.verify(hash, &public_key.to_bytes(false), |k, msg, sgn| {
            k.verify_prehash(msg, sgn)
        })
    }

    /// Returns the raw signature, without the encoded public key recovery bit.
    pub fn raw_signature(&self) -> Box<[u8]> {
        self.signature.into()
    }
}

impl BinarySerializable for Signature {
    const SIZE: usize = 64;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            // Read & clear the top-most bit in S
            let mut ret = Signature {
                signature: [0u8; Self::SIZE],
                recovery: (data[Self::SIZE / 2] & 0x80 != 0).into(),
            };
            ret.signature.copy_from_slice(data);
            ret.signature[Self::SIZE / 2] &= 0x7f;

            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut compressed = Vec::from(self.signature);
        compressed[Self::SIZE / 2] &= 0x7f;
        compressed[Self::SIZE / 2] |= self.recovery << 7;
        compressed.into_boxed_slice()
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.signature.eq(&other.signature)
    }
}

impl Eq for Signature {}

/// A method that turns all lower-cased hexadecimal address to a checksum-ed address
/// according to https://eips.ethereum.org/EIPS/eip-55
pub trait ToChecksum {
    /// Checksum of self according to https://eips.ethereum.org/EIPS/eip-55
    fn to_checksum(&self) -> String;
}

impl ToChecksum for Address {
    fn to_checksum(&self) -> String {
        let address_hex = &self.to_hex()[2..];

        let mut hasher = EthDigest::default();
        hasher.update(address_hex.as_bytes());
        let hash = hasher.finalize();

        let mut ret = String::with_capacity(Self::SIZE * 2 + 2);
        ret.push_str("0x");

        for (i, c) in address_hex.chars().enumerate() {
            let nibble = hash[i / 2] >> (((i + 1) % 2) * 4) & 0xf;
            if nibble >= 8 {
                ret.push(c.to_ascii_uppercase());
            } else {
                ret.push(c);
            }
        }
        ret
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        derivation::derive_vrf_parameters,
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        random::random_group_element,
        types::{
            Challenge, CurvePoint, HalfKey, HalfKeyChallenge, Hash, OffchainPublicKey, OffchainSignature, PublicKey,
            Response, Signature, ToChecksum, VrfParameters,
        },
    };
    use ed25519_dalek::Signer;
    use hex_literal::hex;
    use k256::{
        ecdsa::VerifyingKey,
        elliptic_curve::{sec1::ToEncodedPoint, CurveArithmetic},
        {NonZeroScalar, Secp256k1, U256},
    };
    use libp2p_identity::PeerId;
    use std::str::FromStr;
    use utils_types::{
        primitives::Address,
        traits::{BinarySerializable, PeerIdLike, ToHex},
    };

    const PUBLIC_KEY: [u8; 33] = hex!("021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8");
    const PUBLIC_KEY_UNCOMPRESSED_PLAIN: [u8; 64] = hex!("1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
    const PUBLIC_KEY_UNCOMPRESSED: [u8; 65] = hex!("041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
    const PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    fn signature_signing_test() {
        let msg = b"test12345";
        let kp = ChainKeypair::from_secret(&PRIVATE_KEY).unwrap();
        let sgn = Signature::sign_message(msg, &kp);

        let expected_pk = PublicKey::from_bytes(&PUBLIC_KEY).unwrap();
        assert!(sgn.verify_message(msg, &expected_pk));

        let extracted_pk = PublicKey::from_signature(msg, &sgn).unwrap();
        assert_eq!(expected_pk, extracted_pk, "key extracted from signature does not match");
    }

    #[test]
    fn offchain_signature_signing_test() {
        let msg = b"test12345";
        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        let key = ed25519_dalek::SecretKey::from_bytes(&PRIVATE_KEY).unwrap();
        let pk: ed25519_dalek::PublicKey = (&key).into();
        let kp = ed25519_dalek::Keypair {
            secret: key,
            public: pk.clone(),
        };

        let sgn = kp.sign(msg);
        assert!(pk.verify_strict(msg, &sgn).is_ok(), "blomp");

        let sgn_1 = OffchainSignature::sign_message(msg, &keypair);
        let sgn_2 = OffchainSignature::from_bytes(&sgn_1.to_bytes()).unwrap();

        assert!(
            sgn_1.verify_message(msg, keypair.public()),
            "cannot verify message via sig 1"
        );
        assert!(
            sgn_2.verify_message(msg, keypair.public()),
            "cannot verify message via sig 2"
        );
        assert_eq!(sgn_1, sgn_2, "signatures must be equal");
    }

    #[test]
    fn signature_serialize_test() {
        let msg = b"test000000";
        let kp = ChainKeypair::from_secret(&PRIVATE_KEY).unwrap();
        let sgn = Signature::sign_message(msg, &kp);

        let deserialized = Signature::from_bytes(&sgn.to_bytes()).unwrap();
        assert_eq!(sgn, deserialized, "signatures don't match");
    }

    #[test]
    fn offchain_signature() {
        let msg = b"test12345";
        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        let key = ed25519_dalek::SecretKey::from_bytes(&PRIVATE_KEY).unwrap();
        let pk: ed25519_dalek::PublicKey = (&key).into();
        let kp = ed25519_dalek::Keypair {
            secret: key,
            public: pk.clone(),
        };

        let sgn = kp.sign(msg);
        assert!(pk.verify_strict(msg, &sgn).is_ok(), "blomp");

        let sgn_1 = OffchainSignature::sign_message(msg, &keypair);
        let sgn_2 = OffchainSignature::from_bytes(&sgn_1.to_bytes()).unwrap();

        assert!(
            sgn_1.verify_message(msg, keypair.public()),
            "cannot verify message via sig 1"
        );
        assert!(
            sgn_2.verify_message(msg, keypair.public()),
            "cannot verify message via sig 2"
        );
        assert_eq!(sgn_1, sgn_2, "signatures must be equal");
        // let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        // let sig = OffchainSignature::sign_message("my test msg".as_bytes(), &keypair);

        // assert!(sig.verify_message("my test msg".as_bytes(), keypair.public()));
    }

    #[test]
    fn public_key_to_hex() {
        let pk = PublicKey::from_privkey(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .unwrap();

        assert_eq!("0x39d1bc2291826eaed86567d225cf243ebc637275e0a5aedb0d6b1dc82136a38e428804340d4c949a029846f682711d046920b4ca8b8ebeb9d1192b5bdaa54dba",
            pk.to_hex(false));
        assert_eq!(
            "0x0239d1bc2291826eaed86567d225cf243ebc637275e0a5aedb0d6b1dc82136a38e",
            pk.to_hex(true)
        );
    }

    #[test]
    fn public_key_recover_test() {
        let address = Address::from_str("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();

        let r = hex!("bcae4d37e3a1cd869984d1d68f9242291773cd33d26f1e754ecc1a9bfaee7d17");
        let s = hex!("0b755ab5f6375595fc7fc245c45f6598cc873719183733f4c464d63eefd8579b");
        let v = 1u8;

        let hash = hex!("fac7acad27047640b069e8157b61623e3cb6bb86e6adf97151f93817c291f3cf");

        assert_eq!(
            address,
            PublicKey::from_raw_signature(&hash, &r, &s, v, VerifyingKey::recover_from_prehash)
                .unwrap()
                .to_address()
        );
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

        let kp = ChainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        let signature1 = Signature::sign_message(&msg, &kp);
        let signature2 = Signature::sign_hash(&msg, &kp);

        let pub_key1 = PublicKey::from_privkey(&PRIVATE_KEY).unwrap();
        let pub_key2 = PublicKey::from_signature(&msg, &signature1).unwrap();
        let pub_key3 = PublicKey::from_signature_hash(&msg, &signature2).unwrap();

        assert_eq!(pub_key1, kp.public().0);
        assert_eq!(pub_key1, pub_key2, "recovered public key does not match");
        assert_eq!(pub_key1, pub_key3, "recovered public key does not match");

        assert!(
            signature1.verify_message(&msg, &pub_key1),
            "signature 1 verification failed with pub key 1"
        );
        assert!(
            signature1.verify_message(&msg, &pub_key2),
            "signature 1 verification failed with pub key 2"
        );
        assert!(
            signature1.verify_message(&msg, &pub_key3),
            "signature 1 verification failed with pub key 3"
        );

        assert!(
            signature2.verify_hash(&msg, &pub_key1),
            "signature 2 verification failed with pub key 1"
        );
        assert!(
            signature2.verify_hash(&msg, &pub_key2),
            "signature 2 verification failed with pub key 2"
        );
        assert!(
            signature2.verify_hash(&msg, &pub_key3),
            "signature 2 verification failed with pub key 3"
        );
    }

    #[test]
    fn public_key_serialize_test() {
        let pk1 = PublicKey::from_bytes(&PUBLIC_KEY).expect("failed to deserialize 1");
        let pk2 = PublicKey::from_bytes(&pk1.to_bytes(true)).expect("failed to deserialize 2");
        let pk3 = PublicKey::from_bytes(&pk1.to_bytes(false)).expect("failed to deserialize 3");

        assert_eq!(pk1, pk2, "pub keys 1 2 don't match");
        assert_eq!(pk2, pk3, "pub keys 2 3 don't match");

        let pk1 = PublicKey::from_bytes(&PUBLIC_KEY).expect("failed to deserialize");
        let pk2 = PublicKey::from_bytes(&PUBLIC_KEY_UNCOMPRESSED).expect("failed to deserialize");
        let pk3 = PublicKey::from_bytes(&PUBLIC_KEY_UNCOMPRESSED_PLAIN).expect("failed to deserialize");

        assert_eq!(pk1, pk2, "pubkeys don't match");
        assert_eq!(pk2, pk3, "pubkeys don't match");

        assert_eq!(PublicKey::SIZE_COMPRESSED, pk1.to_bytes(true).len());
        assert_eq!(PublicKey::SIZE_UNCOMPRESSED, pk1.to_bytes(false).len());

        let shorter = hex!("f85e38b056284626a7aed0acc5d474605a408e6cccf76d7241ec7b4dedb31929b710e034f4f9a7dba97743b01e1cc35a45a60bebb29642cb0ba6a7fe8433316c");
        let s1 = PublicKey::from_bytes(&shorter).unwrap();
        let s2 = PublicKey::from_bytes(&s1.to_bytes(false)).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn public_key_curve_point() {
        let cp1: CurvePoint = PublicKey::from_bytes(&PUBLIC_KEY).unwrap().into();
        let cp2 = CurvePoint::from_bytes(&cp1.to_bytes()).unwrap();
        assert_eq!(cp1, cp2);
    }

    #[test]
    fn public_key_from_privkey() {
        let pk1 = PublicKey::from_privkey(&PRIVATE_KEY).expect("failed to convert from private key");
        let pk2 = PublicKey::from_bytes(&PUBLIC_KEY).expect("failed to deserialize");

        assert_eq!(pk1, pk2, "failed to match deserialized pub key");
    }

    #[test]
    fn offchain_public_key_test() {
        let (s, pk1) = OffchainKeypair::random().unzip();

        let pk2 = OffchainPublicKey::from_privkey(s.as_ref()).unwrap();
        assert_eq!(pk1, pk2, "from privkey failed");

        let pk3 = OffchainPublicKey::from_bytes(&pk1.to_bytes()).unwrap();
        assert_eq!(pk1, pk3, "from bytes failed");
    }

    #[test]
    fn offchain_public_key_peerid_test() {
        let (_, pk1) = OffchainKeypair::random().unzip();
        let pk2 =
            OffchainPublicKey::from_peerid_str(pk1.to_peerid_str().as_str()).expect("peer id serialization failed");
        assert_eq!(pk1, pk2, "pubkeys don't match");
        assert_eq!(pk1.to_peerid_str(), pk2.to_peerid_str(), "peer id strings don't match");

        let valid_peerid = PeerId::from_str("12D3KooWLYKsvDB4xEELYoHXxeStj2gzaDXjra2uGaFLpKCZkJHs").unwrap();
        let valid = OffchainPublicKey::from_peerid(&valid_peerid).unwrap();
        assert_eq!(valid_peerid, valid.to_peerid(), "must work with ed25519 peer ids");

        let invalid_peerid = PeerId::from_str("16Uiu2HAmPHGyJ7y1Rj3kJ64HxJQgM9rASaeT2bWfXF9EiX3Pbp3K").unwrap();
        let invalid = OffchainPublicKey::from_peerid(&invalid_peerid);
        assert!(invalid.is_err(), "must not work with secp256k1 peer ids");

        let invalid_peerid_2 = PeerId::from_str("QmWvEwidPYBbLHfcZN6ATHdm4NPM4KbUx72LZnZRoRNKEN").unwrap();
        let invalid_2 = OffchainPublicKey::from_peerid(&invalid_peerid_2);
        assert!(invalid_2.is_err(), "must not work with rsa peer ids");
    }

    #[test]
    pub fn response_test() {
        let r1 = Response::new(&[0u8; Response::SIZE]);
        let r2 = Response::from_bytes(&r1.to_bytes()).unwrap();
        assert_eq!(r1, r2, "deserialized response does not match");
    }

    #[test]
    fn curve_point_test() {
        let scalar = NonZeroScalar::from_uint(U256::from_u8(100)).unwrap();
        let test_point = (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar.as_ref()).to_affine();

        let cp1 = CurvePoint::from_str(hex::encode(test_point.to_encoded_point(false).to_bytes()).as_str()).unwrap();

        let cp2 = CurvePoint::from_bytes(&cp1.to_bytes()).unwrap();

        assert_eq!(cp1, cp2, "failed to match deserialized curve point");

        let pk = PublicKey::from_privkey(&scalar.to_bytes()).unwrap();

        assert_eq!(
            cp1.to_address(),
            pk.to_address(),
            "failed to match curve point address with pub key address"
        );

        let ch1 = Challenge { curve_point: cp1 };
        let ch2 = Challenge { curve_point: cp2 };

        assert_eq!(ch1.to_ethereum_challenge(), ch2.to_ethereum_challenge());
        assert_eq!(ch1, ch2, "failed to match ethereum challenges from curve points");

        // Must be able to create from compressed and uncompressed data
        let scalar2 = NonZeroScalar::from_uint(U256::from_u8(123)).unwrap();
        let test_point2 = (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar2.as_ref()).to_affine();
        let uncompressed = test_point2.to_encoded_point(false);
        assert!(!uncompressed.is_compressed(), "given point is compressed");

        let compressed = uncompressed.compress();
        assert!(compressed.is_compressed(), "failed to compress points");

        let cp3 = CurvePoint::from_bytes(uncompressed.as_bytes()).unwrap();
        let cp4 = CurvePoint::from_bytes(compressed.as_bytes()).unwrap();

        assert_eq!(
            cp3, cp4,
            "failed to match curve point from compressed and uncompressed source"
        );
    }

    #[test]
    fn half_key_test() {
        let hk1 = HalfKey::new(&[0u8; HalfKey::SIZE]);
        let hk2 = HalfKey::from_bytes(&hk1.to_bytes()).unwrap();

        assert_eq!(hk1, hk2, "failed to match deserialized half-key");
    }

    #[test]
    fn half_key_challenge_test() {
        let hkc1 = HalfKeyChallenge::from_bytes(&PUBLIC_KEY).unwrap();
        let hkc2 = HalfKeyChallenge::from_bytes(&hkc1.to_bytes()).unwrap();
        assert_eq!(hkc1, hkc2, "failed to match deserialized half key challenge");
    }

    #[test]
    fn hash_test() {
        let hash1 = Hash::create(&[b"msg"]);
        assert_eq!(
            "0x92aef1b955b9de564fc50e31a55b470b0c8cdb931f186485d620729fb03d6f2c",
            hash1.to_hex(),
            "hash test vector failed to match"
        );

        let hash2 = Hash::from_bytes(&hash1.to_bytes()).unwrap();
        assert_eq!(hash1, hash2, "failed to match deserialized hash");

        assert_eq!(
            hash1.hash(),
            Hash::new(&hex!(
                "1c4d8d521eccee7225073ea180e0fa075a6443afb7ca06076a9566b07d29470f"
            ))
        );
    }

    #[test]
    fn address_to_checksum_test_all_caps() {
        let addr_1 = Address::from_str("52908400098527886e0f7030069857d2e4169ee7").unwrap();
        let value_1 = addr_1.to_checksum();
        let addr_2 = Address::from_str("8617e340b3d01fa5f11f306f4090fd50e238070d").unwrap();
        let value_2 = addr_2.to_checksum();

        assert_eq!(
            value_1, "0x52908400098527886E0F7030069857D2E4169EE7",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
            "checksumed address does not match"
        );
    }

    #[test]
    fn address_to_checksum_test_all_lower() {
        let addr_1 = Address::from_str("de709f2102306220921060314715629080e2fb77").unwrap();
        let value_1 = addr_1.to_checksum();
        let addr_2 = Address::from_str("27b1fdb04752bbc536007a920d24acb045561c26").unwrap();
        let value_2 = addr_2.to_checksum();

        assert_eq!(
            value_1, "0xde709f2102306220921060314715629080e2fb77",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0x27b1fdb04752bbc536007a920d24acb045561c26",
            "checksumed address does not match"
        );
    }

    #[test]
    fn address_to_checksum_test_all_normal() {
        let addr_1 = Address::from_str("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
        let addr_2 = Address::from_str("fb6916095ca1df60bb79ce92ce3ea74c37c5d359").unwrap();
        let addr_3 = Address::from_str("dbf03b407c01e7cd3cbea99509d93f8dddc8c6fb").unwrap();
        let addr_4 = Address::from_str("d1220a0cf47c7b9be7a2e6ba89f429762e7b9adb").unwrap();

        let value_1 = addr_1.to_checksum();
        let value_2 = addr_2.to_checksum();
        let value_3 = addr_3.to_checksum();
        let value_4 = addr_4.to_checksum();

        assert_eq!(
            value_1, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "checksumed address does not match"
        );
        assert_eq!(
            value_3, "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "checksumed address does not match"
        );
        assert_eq!(
            value_4, "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
            "checksumed address does not match"
        );
    }

    #[test]
    fn vrf_values_serialize_deserialize() {
        let keypair = ChainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        let test_msg: [u8; 32] = hex!("8248a966b9215e154c8f673cb154da030916be3fb31af3b1220419a1c98eeaed");

        let vrf_values = derive_vrf_parameters(&test_msg, &keypair, &Hash::default().to_bytes()).unwrap();

        assert_eq!(vrf_values, VrfParameters::from_bytes(&vrf_values.to_bytes()).unwrap());
    }

    #[test]
    fn vrf_values_crypto() {
        let keypair = ChainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        let test_msg: [u8; 32] = hex!("8248a966b9215e154c8f673cb154da030916be3fb31af3b1220419a1c98eeaed");

        let vrf_values = derive_vrf_parameters(&test_msg, &keypair, &Hash::default().to_bytes()).unwrap();

        assert!(vrf_values
            .verify(&keypair.public().to_address(), &test_msg, &Hash::default().to_bytes())
            .is_ok());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::Uint8Array;
    use k256::ecdsa::VerifyingKey;
    use sha3::{digest::DynDigest, Keccak256};
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};
    use wasm_bindgen::prelude::*;

    use crate::types::{
        Challenge, CurvePoint, HalfKey, HalfKeyChallenge, Hash, OffchainPublicKey, OffchainSignature, PublicKey,
        Response, Signature,
    };

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

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(bytes: &[u8]) -> JsResult<CurvePoint> {
            ok_or_jserr!(Self::from_bytes(bytes))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "serialize_compressed")]
        pub fn _serialize_compressed(&self) -> Box<[u8]> {
            self.serialize_compressed()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &CurvePoint) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
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

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Challenge> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl HalfKey {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<HalfKey> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &HalfKey) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
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

        #[wasm_bindgen(js_name = "from_str")]
        pub fn _from_str(str: &str) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(Self::from_str(str))
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(HalfKeyChallenge::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            *self
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Hash {
        #[wasm_bindgen(js_name = "create")]
        pub fn _create(inputs: Vec<Uint8Array>) -> Self {
            let mut hash = Keccak256::default();
            inputs.into_iter().map(|a| a.to_vec()).for_each(|v| hash.update(&v));

            let mut ret = Hash {
                hash: [0u8; Self::SIZE],
            };
            hash.finalize_into(&mut ret.hash).unwrap();
            ret
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Hash> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Hash) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            *self
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl OffchainPublicKey {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(bytes: &[u8]) -> JsResult<OffchainPublicKey> {
            ok_or_jserr!(OffchainPublicKey::from_bytes(bytes))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "from_peerid_str")]
        pub fn _from_peerid_str(peer_id: &str) -> JsResult<OffchainPublicKey> {
            ok_or_jserr!(OffchainPublicKey::from_peerid_str(peer_id))
        }

        #[wasm_bindgen(js_name = "to_peerid_str")]
        pub fn _to_peerid_str(&self) -> String {
            self.to_peerid_str()
        }

        #[wasm_bindgen(js_name = "from_privkey")]
        pub fn _from_privkey(private_key: &[u8]) -> JsResult<OffchainPublicKey> {
            ok_or_jserr!(OffchainPublicKey::from_privkey(private_key))
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &OffchainPublicKey) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen(js_name = "to_string")]
        pub fn _to_string(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen(getter_with_clone)]
    pub struct KeyPair {
        pub private: Box<[u8]>,
        pub public: PublicKey,
    }

    #[wasm_bindgen]
    impl PublicKey {
        #[wasm_bindgen(js_name = "random_keypair")]
        pub fn _random_keypair() -> KeyPair {
            let (private, public) = Self::random_keypair();
            KeyPair {
                private: private.into(),
                public,
            }
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(bytes: &[u8]) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::from_bytes(bytes))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self, compressed: bool) -> Box<[u8]> {
            self.to_bytes(compressed)
        }

        #[wasm_bindgen(js_name = "from_signature")]
        pub fn _from_raw_signature(hash: &[u8], r: &[u8], s: &[u8], v: u8) -> JsResult<PublicKey> {
            ok_or_jserr!(PublicKey::from_raw_signature(
                hash,
                r,
                s,
                v,
                VerifyingKey::recover_from_msg
            ))
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

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }
    }

    #[wasm_bindgen]
    impl Response {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Response> {
            ok_or_jserr!(Response::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "from_half_keys")]
        pub fn _from_half_keys(first: &HalfKey, second: &HalfKey) -> JsResult<Response> {
            ok_or_jserr!(Response::from_half_keys(first, second))
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Signature {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(signature: &[u8]) -> JsResult<Signature> {
            ok_or_jserr!(Signature::from_bytes(signature))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl OffchainSignature {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(signature: &[u8]) -> JsResult<OffchainSignature> {
            ok_or_jserr!(OffchainSignature::from_bytes(signature))
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        #[wasm_bindgen]
        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
