use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
    hash,
    hash::Hasher,
    marker::PhantomData,
    result,
    str::FromStr,
};

use cipher::crypto_common::OutputSizeUser;
use curve25519_dalek::{
    edwards::{CompressedEdwardsY, EdwardsPoint},
    montgomery::MontgomeryPoint,
};
use digest::Digest;
use elliptic_curve::NonZeroScalar;
use hopr_crypto_random::Randomizable;
use hopr_primitive_types::{errors::GeneralError::ParseError, prelude::*};
use k256::{
    AffinePoint, Secp256k1,
    elliptic_curve::{
        self,
        point::NonIdentity,
        sec1::{FromEncodedPoint, ToEncodedPoint},
    },
};
use libp2p_identity::PeerId;

use crate::{
    errors::{
        CryptoError::{self, CalculationError, InvalidInputValue},
        Result,
    },
    utils::random_group_element,
};

pub(crate) fn affine_point_from_bytes(bytes: &[u8]) -> Result<AffinePoint> {
    let ep = k256::EncodedPoint::from_bytes(bytes).map_err(|_| InvalidInputValue("affine_point_from_bytes"))?;
    AffinePoint::from_encoded_point(&ep)
        .into_option()
        .ok_or(InvalidInputValue("affine_point_from_bytes"))
}

pub(crate) fn affine_point_to_address(ap: &AffinePoint) -> Address {
    let serialized = ap.to_encoded_point(false);
    let hash = Hash::create(&[&serialized.as_ref()[1..]]);
    Address::new(&hash.as_ref()[12..])
}

/// Contains the complete Proof-of-Relay challenge is a secp256k1 curve point.
///
/// This is the elliptic curve point corresponding to the `Ticket` challenge.
#[derive(Clone, Copy)]
pub struct Challenge(NonIdentity<AffinePoint>);

impl Debug for Challenge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_encoded_point(true))
    }
}

impl PartialEq for Challenge {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other.0.as_ref())
    }
}

impl Eq for Challenge {}

impl Challenge {
    /// Converts the PoR challenge to an Ethereum challenge.
    ///
    /// This is a one-way (lossy) operation, since the corresponding curve point is hashed
    /// with the hash value then truncated.
    pub fn to_ethereum_challenge(&self) -> EthereumChallenge {
        EthereumChallenge(affine_point_to_address(&self.0))
    }
}

impl Challenge {
    /// Gets the PoR challenge by adding the two EC points represented by the half-key challenges.
    ///
    /// Note that this is an expensive operation that involves point decompression of the
    /// both [`HalfKeyChallenges`](HalfKeyChallenge).
    pub fn from_hint_and_share(own_share: &HalfKeyChallenge, hint: &HalfKeyChallenge) -> Result<Self> {
        #[cfg(not(feature = "rust-ecdsa"))]
        {
            let own_share = secp256k1::PublicKey::from_byte_array_compressed(own_share.0)
                .map_err(|_| ParseError("invalid half-key challenge".into()))?;

            let hint = secp256k1::PublicKey::from_byte_array_compressed(hint.0)
                .map_err(|_| ParseError("invalid half-key challenge".into()))?;

            let res = own_share.combine(&hint).map_err(|_| CalculationError)?;

            affine_point_from_bytes(&res.serialize_uncompressed())
                .and_then(|p| NonIdentity::new(p).into_option().ok_or(CryptoError::InvalidPublicKey))
                .map(Self)
        }

        #[cfg(feature = "rust-ecdsa")]
        {
            let own_share: k256::ProjectivePoint = affine_point_from_bytes(own_share.as_ref())?.into();

            let hint: k256::ProjectivePoint = affine_point_from_bytes(hint.as_ref())?.into();

            NonIdentity::new((own_share + hint).to_affine())
                .into_option()
                .ok_or(CalculationError)
                .map(Self)
        }
    }

    /// Gets the PoR challenge by converting the given HalfKey into a secp256k1 point and
    /// adding it with the given HalfKeyChallenge (which already represents a secp256k1 point).
    ///
    /// Note that this is an expensive operation that involves point decompression of the
    /// both [`HalfKeyChallenge`] and scalar multiplication of the [`HalfKey`] with the basepoint.
    pub fn from_own_share_and_half_key(own_share: &HalfKeyChallenge, half_key: &HalfKey) -> Result<Self> {
        Self::from_hint_and_share(own_share, &half_key.to_challenge()?)
    }
}

/// Represents a half-key used for the Proof-of-Relay.
///
/// Half-key is equivalent to a non-zero scalar in the field used by secp256k1, but the type
/// itself does not validate nor enforce this fact.
///
/// The type is internally represented as a byte-array of the secp256k1 field element.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HalfKey(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl Default for HalfKey {
    fn default() -> Self {
        let mut ret = Self([0u8; Self::SIZE]);

        ret.0.copy_from_slice(
            NonZeroScalar::<Secp256k1>::from_uint(1u16.into())
                .unwrap()
                .to_bytes()
                .as_slice(),
        );
        ret
    }
}

impl HalfKey {
    /// Converts the non-zero scalar represented by this half-key into the half-key challenge.
    ///
    /// Note that this is an expensive operation that involves scalar multiplication.
    ///
    /// Returns an error if the instance is a zero scalar.
    pub fn to_challenge(&self) -> Result<HalfKeyChallenge> {
        // This may return an error if the instance was deserialized (e.g., via serde) from a zero scalar
        Ok(PublicKey::from_privkey(&self.0)?.as_ref().try_into()?)
    }
}

impl Randomizable for HalfKey {
    fn random() -> Self {
        Self(random_group_element().0)
    }
}

impl AsRef<[u8]> for HalfKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for HalfKey {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ParseError("HalfKey".into()))?))
    }
}

impl BytesRepresentable for HalfKey {
    /// Size of the secp256k1 secret scalar representing the `HalfKey`.
    const SIZE: usize = 32;
}

/// Represents a challenge for the half-key in Proof of Relay.
///
/// Half-key challenge is equivalent to a secp256k1 curve point.
/// Therefore, `HalfKeyChallenge` can be [obtained](HalfKey::to_challenge) from a [`HalfKey`].
///
/// The value is internally stored as a compressed point encoded as a byte-array.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HalfKeyChallenge(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl Display for HalfKeyChallenge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for HalfKeyChallenge {
    fn default() -> Self {
        // Note that the default HalfKeyChallenge is the identity point on secp256k1, therefore,
        // will fail all public key checks, which is intended.
        let mut ret = Self([0u8; Self::SIZE]);
        ret.0[Self::SIZE - 1] = 1;
        ret
    }
}

impl HalfKeyChallenge {
    pub fn new(half_key_challenge: &[u8]) -> Self {
        let mut ret = Self::default();
        ret.0.copy_from_slice(half_key_challenge);
        ret
    }
}

impl AsRef<[u8]> for HalfKeyChallenge {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for HalfKeyChallenge {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            value.try_into().map_err(|_| ParseError("HalfKeyChallenge".into()))?,
        ))
    }
}

impl BytesRepresentable for HalfKeyChallenge {
    /// Size of the compressed secp256k1 point representing the Half Key Challenge.
    const SIZE: usize = PublicKey::SIZE_COMPRESSED;
}

impl FromStr for HalfKeyChallenge {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

const HASH_BASE_SIZE: usize = 32;

/// Represents a generic 256-bit hash value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HashBase<H>(
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; HASH_BASE_SIZE],
    PhantomData<H>,
);

impl<H> Clone for HashBase<H> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<H> Copy for HashBase<H> {}

impl<H> PartialEq for HashBase<H> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<H> Eq for HashBase<H> {}

impl<H> Default for HashBase<H> {
    fn default() -> Self {
        Self([0u8; HASH_BASE_SIZE], PhantomData)
    }
}

impl<H> PartialOrd<Self> for HashBase<H> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<H> Ord for HashBase<H> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<H> std::hash::Hash for HashBase<H> {
    fn hash<H2: Hasher>(&self, state: &mut H2) {
        self.0.hash(state);
    }
}

impl<H> Debug for HashBase<H> {
    // Intentionally same as Display
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl<H> Display for HashBase<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl<H> FromStr for HashBase<H> {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl<H> HashBase<H>
where
    H: OutputSizeUser<OutputSize = typenum::U32> + Digest,
{
    /// Convenience method that creates a new hash by hashing this.
    pub fn hash(&self) -> Self {
        Self::create(&[&self.0])
    }

    /// Takes all the byte slices and computes hash of their concatenated value.
    pub fn create(inputs: &[&[u8]]) -> Self {
        let mut hash = H::new();
        inputs.iter().for_each(|v| hash.update(v));
        Self(hash.finalize().into(), PhantomData)
    }
}

impl<H> AsRef<[u8]> for HashBase<H> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<H> TryFrom<&[u8]> for HashBase<H> {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            value.try_into().map_err(|_| ParseError("Hash".into()))?,
            PhantomData,
        ))
    }
}

impl<H> BytesRepresentable for HashBase<H> {
    /// The size of the digest is 32 bytes.
    const SIZE: usize = HASH_BASE_SIZE;
}

impl<H> From<[u8; HASH_BASE_SIZE]> for HashBase<H> {
    fn from(hash: [u8; HASH_BASE_SIZE]) -> Self {
        Self(hash, PhantomData)
    }
}

impl<H> From<HashBase<H>> for [u8; HASH_BASE_SIZE] {
    fn from(value: HashBase<H>) -> Self {
        value.0
    }
}

impl<H> From<&HashBase<H>> for [u8; HASH_BASE_SIZE] {
    fn from(value: &HashBase<H>) -> Self {
        value.0
    }
}

impl<H> From<HashBase<H>> for primitive_types::H256 {
    fn from(value: HashBase<H>) -> Self {
        value.0.into()
    }
}

impl<H> From<primitive_types::H256> for HashBase<H> {
    fn from(value: primitive_types::H256) -> Self {
        Self(value.0, PhantomData)
    }
}

/// Represents an Ethereum 256-bit hash value.
///
/// This implementation instantiates the hash via Keccak256 digest.
pub type Hash = HashBase<sha3::Keccak256>;

/// Represents an alternative 256-bit hash value.
///
/// This implementation instantiates the hash via Blake3 digest.
pub type HashAlt = HashBase<blake3::Hasher>;

/// Represents an Ed25519 public key.
#[derive(Clone, Copy, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffchainPublicKey {
    compressed: CompressedEdwardsY,
    pub(crate) edwards: EdwardsPoint,
    montgomery: MontgomeryPoint,
}

impl std::fmt::Debug for OffchainPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Intentionally same as display
        write!(f, "{}", self.to_hex())
    }
}

impl std::hash::Hash for OffchainPublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.compressed.hash(state);
    }
}

impl PartialEq for OffchainPublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.compressed == other.compressed
    }
}

impl AsRef<[u8]> for OffchainPublicKey {
    fn as_ref(&self) -> &[u8] {
        self.compressed.as_bytes()
    }
}

impl TryFrom<&[u8]> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let compressed = CompressedEdwardsY::from_slice(value).map_err(|_| ParseError("OffchainPublicKey".into()))?;
        let edwards = compressed
            .decompress()
            .ok_or(ParseError("OffchainPublicKey.decompress".into()))?;
        Ok(Self {
            compressed,
            edwards,
            montgomery: edwards.to_montgomery(),
        })
    }
}

impl BytesRepresentable for OffchainPublicKey {
    /// Size of the public key (compressed Edwards Y coordinate)
    const SIZE: usize = 32;
}

impl TryFrom<[u8; OffchainPublicKey::SIZE]> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: [u8; OffchainPublicKey::SIZE]) -> std::result::Result<Self, Self::Error> {
        let v: &[u8] = &value;
        v.try_into()
    }
}

impl From<OffchainPublicKey> for PeerId {
    fn from(value: OffchainPublicKey) -> Self {
        let k = libp2p_identity::ed25519::PublicKey::try_from_bytes(value.compressed.as_bytes()).unwrap();
        PeerId::from_public_key(&k.into())
    }
}

impl From<&OffchainPublicKey> for PeerId {
    fn from(value: &OffchainPublicKey) -> Self {
        (*value).into()
    }
}

impl Display for OffchainPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl OffchainPublicKey {
    /// Tries to create the public key from a Ed25519 private key.
    /// The length must be exactly `ed25519_dalek::SECRET_KEY_LENGTH`.
    pub fn from_privkey(private_key: &[u8]) -> Result<Self> {
        let mut pk: [u8; ed25519_dalek::SECRET_KEY_LENGTH] =
            private_key.try_into().map_err(|_| InvalidInputValue("private_key"))?;
        let sk = libp2p_identity::ed25519::SecretKey::try_from_bytes(&mut pk)
            .map_err(|_| InvalidInputValue("private_key"))?;
        let kp: libp2p_identity::ed25519::Keypair = sk.into();
        Ok(Self::try_from(kp.public().to_bytes())?)
    }

    /// Outputs the public key as PeerId represented as Base58 string.
    pub fn to_peerid_str(&self) -> String {
        PeerId::from(self).to_base58()
    }

    /// Tries to convert an Ed25519 `PeerId` to `OffchainPublicKey`.
    ///
    /// This is a CPU-intensive operation, as it performs Ed25519 point decompression
    /// and mapping to the Curve255919 point representation.
    pub fn from_peerid(peerid: &PeerId) -> std::result::Result<Self, GeneralError> {
        let mh = peerid.as_ref();
        if mh.code() == 0 {
            libp2p_identity::PublicKey::try_decode_protobuf(mh.digest())
                .map_err(|_| GeneralError::ParseError("invalid ed25519 peer id".into()))
                .and_then(|pk| {
                    pk.try_into_ed25519()
                        .map(|p| p.to_bytes())
                        .map_err(|_| GeneralError::ParseError("invalid ed25519 peer id".into()))
                })
                .and_then(Self::try_from)
        } else {
            Err(GeneralError::ParseError("invalid ed25519 peer id".into()))
        }
    }
}

impl From<&OffchainPublicKey> for EdwardsPoint {
    fn from(value: &OffchainPublicKey) -> Self {
        value.edwards
    }
}

impl From<&OffchainPublicKey> for MontgomeryPoint {
    fn from(value: &OffchainPublicKey) -> Self {
        value.montgomery
    }
}

/// Compact representation of [`OffchainPublicKey`] suitable for use in enums.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompactOffchainPublicKey(CompressedEdwardsY);

impl From<OffchainPublicKey> for CompactOffchainPublicKey {
    fn from(value: OffchainPublicKey) -> Self {
        Self(value.compressed)
    }
}

impl CompactOffchainPublicKey {
    /// Performs an **expensive** operation of converting back to the [`OffchainPublicKey`].
    pub fn into_offchain_public_key(self) -> OffchainPublicKey {
        let decompressed = self.0.decompress().expect("decompression must not fail");
        OffchainPublicKey {
            compressed: self.0,
            edwards: decompressed,
            montgomery: decompressed.to_montgomery(),
        }
    }
}

/// Length of a packet tag
pub const PACKET_TAG_LENGTH: usize = 16;

/// Represents a fixed size packet verification tag
pub type PacketTag = [u8; PACKET_TAG_LENGTH];

/// Represents a secp256k1 public key.
///
/// The key is internally represented using an `AffinePoint` and the compressed encoding of it.
///
/// The `AsRef` implementation will always return the compressed representation.
/// However, the `TryFrom` byte slice accepts any representation.
#[derive(Copy, Clone)]
pub struct PublicKey(NonIdentity<AffinePoint>, [u8; Self::SIZE_COMPRESSED]);

impl PublicKey {
    /// Size of the compressed public key in bytes
    pub const SIZE_COMPRESSED: usize = 33;
    /// Size of the uncompressed public key in bytes
    pub const SIZE_UNCOMPRESSED: usize = 65;
    pub const SIZE_UNCOMPRESSED_PLAIN: usize = 64;

    /// Computes the public key from the given `private_key`.
    ///
    /// The private key must be a big-endian encoding of a non-zero scalar in the field
    /// of the `secp256k1` curve.
    pub fn from_privkey(private_key: &[u8]) -> Result<PublicKey> {
        #[cfg(feature = "rust-ecdsa")]
        {
            // This verifies that it is indeed a non-zero scalar, and thus represents a valid public key
            let secret_scalar = NonZeroScalar::<Secp256k1>::try_from(private_key)
                .map_err(|_| GeneralError::ParseError("PublicKey".into()))?;

            Ok(
                elliptic_curve::PublicKey::<Secp256k1>::from_secret_scalar(&secret_scalar)
                    .to_nonidentity()
                    .into(),
            )
        }

        #[cfg(not(feature = "rust-ecdsa"))]
        {
            let sk = secp256k1::SecretKey::from_byte_array(
                private_key
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("private_key.len".into()))?,
            )
            .map_err(|_| GeneralError::ParseError("private_key".into()))?;

            let pk = secp256k1::PublicKey::from_secret_key_global(&sk);
            affine_point_from_bytes(&pk.serialize_uncompressed())
                .and_then(|p| NonIdentity::new(p).into_option().ok_or(CryptoError::InvalidPublicKey))
                .map(Self::from)
        }
    }

    /// Converts the public key to an Ethereum address
    pub fn to_address(&self) -> Address {
        affine_point_to_address(self.0.as_ref())
    }

    /// Serializes the public key to a binary uncompressed form.
    pub fn to_uncompressed_bytes(&self) -> Box<[u8]> {
        self.0.to_encoded_point(false).to_bytes()
    }

    /// Serializes the public key to a binary uncompressed form and converts it to hexadecimal string representation.
    pub fn to_uncompressed_hex(&self) -> String {
        format!("0x{}", hex::encode(self.to_uncompressed_bytes()))
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}

impl Eq for PublicKey {}

impl hash::Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

impl Randomizable for PublicKey {
    /// Generates a new random public key.
    /// Because the corresponding private key is discarded, this might be useful only for testing purposes.
    fn random() -> Self {
        let (_, cp) = random_group_element();
        cp.into()
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        match value.len() {
            Self::SIZE_UNCOMPRESSED => {
                // already has 0x04 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(value)
                    .map_err(|_| GeneralError::ParseError("invalid secp256k1 point".into()))?;

                Ok(key.to_nonidentity().into())
            }
            Self::SIZE_UNCOMPRESSED_PLAIN => {
                // add 0x04 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(&[&[4u8], value].concat())
                    .map_err(|_| GeneralError::ParseError("invalid secp256k1 point".into()))?;

                Ok(key.to_nonidentity().into())
            }
            Self::SIZE_COMPRESSED => {
                // has either 0x02 or 0x03 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(value)
                    .map_err(|_| GeneralError::ParseError("invalid secp256k1 point".into()))?;

                Ok(key.to_nonidentity().into())
            }
            _ => Err(GeneralError::ParseError("invalid secp256k1 point".into())),
        }
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.1
    }
}

impl BytesRepresentable for PublicKey {
    const SIZE: usize = PublicKey::SIZE_COMPRESSED;
}

impl From<NonIdentity<AffinePoint>> for PublicKey {
    fn from(value: NonIdentity<AffinePoint>) -> Self {
        let mut compressed = [0u8; PublicKey::SIZE_COMPRESSED];
        compressed.copy_from_slice(value.to_encoded_point(true).as_bytes());
        Self(value, compressed)
    }
}

impl From<PublicKey> for NonIdentity<AffinePoint> {
    fn from(value: PublicKey) -> Self {
        value.0
    }
}

impl TryFrom<AffinePoint> for PublicKey {
    type Error = CryptoError;

    fn try_from(value: AffinePoint) -> std::result::Result<Self, Self::Error> {
        Ok(NonIdentity::new(value)
            .into_option()
            .ok_or(CryptoError::InvalidPublicKey)?
            .into())
    }
}

// TODO: make this `for &k256::ProjectivePoint`
impl From<&PublicKey> for k256::ProjectivePoint {
    fn from(value: &PublicKey) -> Self {
        (*value.0.as_ref()).into()
    }
}

/// Contains a response upon ticket acknowledgement
/// It is equivalent to a non-zero secret scalar on secp256k1 (EC private key).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Response(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl Default for Response {
    fn default() -> Self {
        Self(HalfKey::default().0)
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Response {
    /// Converts this response to the PoR challenge by turning the non-zero scalar
    /// represented by this response into a secp256k1 curve point (public key).
    ///
    /// Note that this is an expensive operation involving scalar multiplication.
    ///
    /// Error is returned when this `Response` contains an invalid value.
    pub fn to_challenge(&self) -> Result<Challenge> {
        // This may return an error if the instance was deserialized (e.g., via serde) from a zero scalar
        PublicKey::from_privkey(&self.0).map(|pk| Challenge(pk.into()))
    }

    /// Derives the response from two half-keys.
    ///
    /// This is done by adding together the two non-zero scalars that the given half-keys represent.
    /// Returns an error if any of the given scalars is zero.
    pub fn from_half_keys(first: &HalfKey, second: &HalfKey) -> Result<Self> {
        let first = NonZeroScalar::<Secp256k1>::try_from(first.as_ref()).map_err(|_| InvalidInputValue("first"))?;
        let second = NonZeroScalar::<Secp256k1>::try_from(second.as_ref()).map_err(|_| InvalidInputValue("second"))?;

        // This addition is modulo order the order of the secp256k1 prime field
        let res = first.as_ref() + second.as_ref();
        if res.is_zero().into() {
            return Err(InvalidInputValue("invalid half-key"));
        }

        let mut ret = [0u8; Self::SIZE];
        ret.copy_from_slice(res.to_bytes().as_slice());
        Ok(Self(ret))
    }
}

impl AsRef<[u8]> for Response {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Response {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ParseError("Response".into()))?))
    }
}

impl BytesRepresentable for Response {
    /// Size of the PoR challenge response.
    const SIZE: usize = 32;
}

impl From<[u8; Self::SIZE]> for Response {
    fn from(value: [u8; Self::SIZE]) -> Self {
        Self(value)
    }
}

/// Pseudonym used to identify the creator of a `SURB`.
/// This allows indexing `SURB` and `LocalSURBEntry` at both parties.
///
/// To maintain anonymity, this must be something else than the sender's
/// public key or public key identifier.
pub trait Pseudonym: BytesRepresentable + hash::Hash + Eq + Display + Randomizable {}

/// Represents a simple UUID-like pseudonym consisting of 10 bytes.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimplePseudonym(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] pub [u8; Self::SIZE]);

impl Display for SimplePseudonym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Debug for SimplePseudonym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl BytesRepresentable for SimplePseudonym {
    const SIZE: usize = 10;
}

impl AsRef<[u8]> for SimplePseudonym {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for SimplePseudonym {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> result::Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| ParseError("SimplePseudonym".into()))
    }
}

impl Randomizable for SimplePseudonym {
    /// Generates a random pseudonym.
    fn random() -> Self {
        let mut data = vec![0u8; Self::SIZE];
        hopr_crypto_random::random_fill(&mut data);
        Self::try_from(data.as_slice()).unwrap()
    }
}

impl Pseudonym for SimplePseudonym {}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_primitive_types::prelude::*;
    use k256::AffinePoint;
    use libp2p_identity::PeerId;

    use crate::{
        keypairs::{Keypair, OffchainKeypair},
        types::{Challenge, HalfKey, HalfKeyChallenge, Hash, OffchainPublicKey, PublicKey, Response},
    };

    const PUBLIC_KEY: [u8; 33] = hex!("021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8");
    const PUBLIC_KEY_UNCOMPRESSED_PLAIN: [u8; 64] = hex!("1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
    const PUBLIC_KEY_UNCOMPRESSED: [u8; 65] = hex!("041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
    const PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    fn test_public_key_to_hex() -> anyhow::Result<()> {
        let pk = PublicKey::from_privkey(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        assert_eq!("0x0439d1bc2291826eaed86567d225cf243ebc637275e0a5aedb0d6b1dc82136a38e428804340d4c949a029846f682711d046920b4ca8b8ebeb9d1192b5bdaa54dba",
            pk.to_uncompressed_hex());
        assert_eq!(
            "0x0239d1bc2291826eaed86567d225cf243ebc637275e0a5aedb0d6b1dc82136a38e",
            pk.to_hex()
        );

        Ok(())
    }

    #[test]
    fn test_public_key_serialize() -> anyhow::Result<()> {
        let pk1 = PublicKey::try_from(PUBLIC_KEY.as_ref())?;
        let pk2 = PublicKey::try_from(pk1.as_ref())?;
        let pk3 = PublicKey::try_from(pk1.to_uncompressed_bytes().as_ref())?;

        assert_eq!(pk1, pk2, "pub keys 1 2 don't match");
        assert_eq!(pk2, pk3, "pub keys 2 3 don't match");

        let pk1 = PublicKey::try_from(PUBLIC_KEY.as_ref())?;
        let pk2 = PublicKey::try_from(PUBLIC_KEY_UNCOMPRESSED.as_ref())?;
        let pk3 = PublicKey::try_from(PUBLIC_KEY_UNCOMPRESSED_PLAIN.as_ref())?;

        assert_eq!(pk1, pk2, "pubkeys don't match");
        assert_eq!(pk2, pk3, "pubkeys don't match");

        assert_eq!(PublicKey::SIZE_COMPRESSED, pk1.as_ref().len());
        assert_eq!(PublicKey::SIZE_UNCOMPRESSED, pk1.to_uncompressed_bytes().len());

        let shorter = hex!("f85e38b056284626a7aed0acc5d474605a408e6cccf76d7241ec7b4dedb31929b710e034f4f9a7dba97743b01e1cc35a45a60bebb29642cb0ba6a7fe8433316c");
        let s1 = PublicKey::try_from(shorter.as_ref())?;
        let s2 = PublicKey::try_from(s1.to_uncompressed_bytes().as_ref())?;
        assert_eq!(s1, s2);

        Ok(())
    }

    #[test]
    fn test_public_key_should_not_accept_identity() -> anyhow::Result<()> {
        PublicKey::try_from(AffinePoint::IDENTITY).expect_err("must fail for identity point");
        Ok(())
    }

    #[test]
    fn test_public_key_from_privkey() -> anyhow::Result<()> {
        let pk1 = PublicKey::from_privkey(&PRIVATE_KEY)?;
        let pk2 = PublicKey::try_from(PUBLIC_KEY.as_ref())?;

        assert_eq!(pk1, pk2, "failed to match deserialized pub key");

        Ok(())
    }

    #[test]
    fn test_offchain_public_key() -> anyhow::Result<()> {
        let (s, pk1) = OffchainKeypair::random().unzip();

        let pk2 = OffchainPublicKey::from_privkey(s.as_ref())?;
        assert_eq!(pk1, pk2, "from privkey failed");

        let pk3 = OffchainPublicKey::try_from(pk1.as_ref())?;
        assert_eq!(pk1, pk3, "from bytes failed");

        Ok(())
    }

    #[test]
    fn test_offchain_public_key_peerid() -> anyhow::Result<()> {
        let valid_peerid = PeerId::from_str("12D3KooWLYKsvDB4xEELYoHXxeStj2gzaDXjra2uGaFLpKCZkJHs")?;
        let valid = OffchainPublicKey::from_peerid(&valid_peerid)?;
        assert_eq!(valid_peerid, valid.into(), "must work with ed25519 peer ids");

        let invalid_peerid = PeerId::from_str("16Uiu2HAmPHGyJ7y1Rj3kJ64HxJQgM9rASaeT2bWfXF9EiX3Pbp3K")?;
        let invalid = OffchainPublicKey::from_peerid(&invalid_peerid);
        assert!(invalid.is_err(), "must not work with secp256k1 peer ids");

        let invalid_peerid_2 = PeerId::from_str("QmWvEwidPYBbLHfcZN6ATHdm4NPM4KbUx72LZnZRoRNKEN")?;
        let invalid_2 = OffchainPublicKey::from_peerid(&invalid_peerid_2);
        assert!(invalid_2.is_err(), "must not work with rsa peer ids");

        Ok(())
    }

    #[test]
    pub fn test_response() -> anyhow::Result<()> {
        let r1 = Response([0u8; Response::SIZE]);
        let r2 = Response::try_from(r1.as_ref())?;
        assert_eq!(r1, r2, "deserialized response does not match");

        Ok(())
    }

    #[test]
    fn test_half_key() -> anyhow::Result<()> {
        let hk1 = HalfKey([0u8; HalfKey::SIZE]);
        let hk2 = HalfKey::try_from(hk1.as_ref())?;

        assert_eq!(hk1, hk2, "failed to match deserialized half-key");

        Ok(())
    }

    #[test]
    fn test_half_key_challenge() -> anyhow::Result<()> {
        let hkc1 = HalfKeyChallenge::try_from(PUBLIC_KEY.as_ref())?;
        let hkc2 = HalfKeyChallenge::try_from(hkc1.as_ref())?;
        assert_eq!(hkc1, hkc2, "failed to match deserialized half key challenge");

        Ok(())
    }

    #[test]
    fn test_challenge_response_flow() -> anyhow::Result<()> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let response = Response::from_half_keys(&hk1, &hk2)?;

        let half_chal1 = hk1.to_challenge()?;
        let half_chal2 = hk2.to_challenge()?;

        let challenge1 = Challenge::from_hint_and_share(&half_chal1, &half_chal2)?;
        assert_eq!(challenge1, Challenge::from_hint_and_share(&half_chal2, &half_chal1)?);
        assert_eq!(challenge1, Challenge::from_own_share_and_half_key(&half_chal1, &hk2)?);

        let challenge2 = response.to_challenge()?;
        assert_eq!(challenge1, challenge2);
        assert_eq!(challenge1.to_ethereum_challenge(), challenge2.to_ethereum_challenge());
        Ok(())
    }

    #[test]
    fn test_hash() -> anyhow::Result<()> {
        let hash1 = Hash::create(&[b"msg"]);
        assert_eq!(
            "0x92aef1b955b9de564fc50e31a55b470b0c8cdb931f186485d620729fb03d6f2c",
            hash1.to_hex(),
            "hash test vector failed to match"
        );

        let hash2 = Hash::try_from(hash1.as_ref())?;
        assert_eq!(hash1, hash2, "failed to match deserialized hash");

        assert_eq!(
            hash1.hash(),
            Hash::try_from(hex!("1c4d8d521eccee7225073ea180e0fa075a6443afb7ca06076a9566b07d29470f").as_ref())?
        );

        Ok(())
    }
}
