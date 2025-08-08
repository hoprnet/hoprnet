use std::{
    fmt::{Debug, Display, Formatter},
    hash,
    hash::Hasher,
    ops::Add,
    result,
    str::FromStr,
    sync::OnceLock,
};

use cipher::crypto_common::{Output, OutputSizeUser};
use curve25519_dalek::{
    edwards::{CompressedEdwardsY, EdwardsPoint},
    montgomery::MontgomeryPoint,
};
use digest::Digest;
use elliptic_curve::{NonZeroScalar, ProjectivePoint, sec1::EncodedPoint};
use hopr_crypto_random::Randomizable;
use hopr_primitive_types::{errors::GeneralError::ParseError, prelude::*};
use k256::{
    AffinePoint, Secp256k1,
    ecdsa::{
        self, RecoveryId, Signature as ECDSASignature, SigningKey, VerifyingKey,
        signature::{Verifier, hazmat::PrehashVerifier},
    },
    elliptic_curve::{
        self, CurveArithmetic,
        generic_array::GenericArray,
        group::prime::PrimeCurveAffine,
        sec1::{FromEncodedPoint, ToEncodedPoint},
    },
};
use libp2p_identity::PeerId;
use sha2::Sha512;
use sha3::Keccak256;
use tracing::warn;
use typenum::Unsigned;

use crate::{
    errors::{
        CryptoError::{self, CalculationError, InvalidInputValue},
        Result,
    },
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    utils::random_group_element,
};

/// Represents an elliptic curve point on the secp256k1 curve
/// It stores the compressed (and optionally also the uncompressed) form.
///
/// ```rust
/// # use hopr_crypto_types::prelude::*;
/// # use hex_literal::hex;
/// # use k256::{elliptic_curve::NonZeroScalar, Scalar, Secp256k1};
///
/// let a: [u8; 32] = hex!("876027a13900aad908842c3f79307cc8e96de5c3331090e91a24c315f2a8d43a");
/// let b: [u8; 32] = hex!("561d3fe2990e6a768b90f7d510b69e967e0922a3b61e8141398113aede8e1d3e");
///
/// let A = CurvePoint::from_exponent(&a).unwrap();
/// let B = CurvePoint::from_exponent(&b).unwrap();
///
/// // A_plus_B = a * G + b * G
/// let A_plus_B = CurvePoint::combine(&[&A, &B]);
///
/// let scalar_a: Scalar = *NonZeroScalar::<Secp256k1>::try_from(&a[..]).unwrap();
/// let scalar_b: Scalar = *NonZeroScalar::<Secp256k1>::try_from(&b[..]).unwrap();
///
/// // a_plus_b = (a + b) * G
/// let a_plus_b = CurvePoint::from_exponent(&(scalar_a + scalar_b).to_bytes()).unwrap();
///
/// // group homomorphism
/// // (a + b) * G = a * G + b * G
/// assert_eq!(A_plus_B, a_plus_b);
/// ```
#[derive(Clone, Debug)]
pub struct CurvePoint {
    pub(crate) affine: AffinePoint,
    compressed: EncodedPoint<Secp256k1>,
    uncompressed: OnceLock<EncodedPoint<Secp256k1>>,
}

impl CurvePoint {
    /// Size of the point if serialized via [`CurvePoint::as_compressed`].
    pub const SIZE_COMPRESSED: usize = 33;
    /// Size of the point if serialized via [`CurvePoint::as_uncompressed`].
    pub const SIZE_UNCOMPRESSED: usize = 65;

    /// Converts the uncompressed representation of the curve point to Ethereum address.
    pub fn to_address(&self) -> Address {
        let serialized = self.as_uncompressed();
        let hash = Hash::create(&[&serialized.as_bytes()[1..]]);
        Address::new(&hash.as_ref()[12..])
    }

    /// Creates a curve point from a non-zero scalar.
    /// The given exponent must represent a non-zero scalar and must result into
    /// a secp256k1 identity point.
    pub fn from_exponent(exponent: &[u8]) -> Result<Self> {
        PublicKey::from_privkey(exponent).map(CurvePoint::from)
    }

    /// Converts the curve point to a representation suitable for calculations.
    pub fn into_projective_point(self) -> ProjectivePoint<Secp256k1> {
        self.affine.into()
    }

    /// Converts the curve point into a compressed form. This is a cheap operation.
    pub fn as_compressed(&self) -> &EncodedPoint<Secp256k1> {
        &self.compressed
    }

    /// Converts the curve point into an uncompressed form. This is many cases an expensive operation.
    pub fn as_uncompressed(&self) -> &EncodedPoint<Secp256k1> {
        self.uncompressed.get_or_init(|| self.affine.to_encoded_point(false))
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

impl std::hash::Hash for CurvePoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.compressed.hash(state);
    }
}

impl Default for CurvePoint {
    fn default() -> Self {
        Self::from(AffinePoint::default())
    }
}

impl PartialEq for CurvePoint {
    fn eq(&self, other: &Self) -> bool {
        self.affine.eq(&other.affine)
    }
}

impl Eq for CurvePoint {}

impl From<PublicKey> for CurvePoint {
    fn from(pubkey: PublicKey) -> Self {
        pubkey.0
    }
}

impl From<&PublicKey> for CurvePoint {
    fn from(pubkey: &PublicKey) -> Self {
        pubkey.0.clone()
    }
}

impl From<AffinePoint> for CurvePoint {
    fn from(affine: AffinePoint) -> Self {
        Self {
            affine,
            compressed: affine.to_encoded_point(true),
            uncompressed: OnceLock::new(),
        }
    }
}

impl TryFrom<HalfKeyChallenge> for CurvePoint {
    type Error = GeneralError;

    fn try_from(value: HalfKeyChallenge) -> std::result::Result<Self, Self::Error> {
        Self::try_from(value.0.as_ref())
    }
}

impl FromStr for CurvePoint {
    type Err = CryptoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(CurvePoint::try_from(
            hex::decode(s)
                .map_err(|_| GeneralError::ParseError("CurvePoint".into()))?
                .as_slice(),
        )?)
    }
}

impl From<CurvePoint> for AffinePoint {
    fn from(value: CurvePoint) -> Self {
        value.affine
    }
}

impl AsRef<[u8]> for CurvePoint {
    fn as_ref(&self) -> &[u8] {
        self.compressed.as_ref()
    }
}

impl TryFrom<&[u8]> for CurvePoint {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let ep = elliptic_curve::sec1::EncodedPoint::<Secp256k1>::from_bytes(value)
            .map_err(|_| GeneralError::ParseError("invalid secp256k1 encoded point".into()))?;
        Ok(Self {
            affine: Option::from(AffinePoint::from_encoded_point(&ep))
                .ok_or(GeneralError::ParseError("invalid secp256k1 encoded point".into()))?,
            // Compressing an uncompressed EC point is inexpensive
            compressed: if ep.is_compressed() { ep } else { ep.compress() },
            // If not directly uncompressed, defer the expensive operation for later
            uncompressed: if !ep.is_compressed() {
                ep.into()
            } else {
                OnceLock::new()
            },
        })
    }
}

impl BytesRepresentable for CurvePoint {
    const SIZE: usize = Self::SIZE_COMPRESSED;
}

/// Natural extension of the Curve Point to the Proof-of-Relay challenge.
/// Proof-of-Relay challenge is a secp256k1 curve point.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Challenge(CurvePoint);

impl Challenge {
    /// Converts the PoR challenge to an Ethereum challenge.
    /// This is a one-way (lossy) operation, since the corresponding curve point is hashed
    /// with the hash value then truncated.
    pub fn to_ethereum_challenge(&self) -> EthereumChallenge {
        EthereumChallenge::new(self.0.to_address().as_ref())
    }
}

impl From<Challenge> for EthereumChallenge {
    fn from(challenge: Challenge) -> Self {
        challenge.to_ethereum_challenge()
    }
}

impl Challenge {
    /// Gets the PoR challenge by adding the two EC points represented by the half-key challenges
    pub fn from_hint_and_share(own_share: &HalfKeyChallenge, hint: &HalfKeyChallenge) -> Result<Self> {
        let curve_point: CurvePoint = PublicKey::combine(&[
            &PublicKey::try_from(own_share.0.as_ref())?,
            &PublicKey::try_from(hint.0.as_ref())?,
        ])
        .into();
        Ok(curve_point.into())
    }

    /// Gets the PoR challenge by converting the given HalfKey into a secp256k1 point and
    /// adding it with the given HalfKeyChallenge (which already represents a secp256k1 point).
    pub fn from_own_share_and_half_key(own_share: &HalfKeyChallenge, half_key: &HalfKey) -> Result<Self> {
        Self::from_hint_and_share(own_share, &half_key.to_challenge())
    }
}

impl From<CurvePoint> for Challenge {
    fn from(curve_point: CurvePoint) -> Self {
        Self(curve_point)
    }
}

impl From<Response> for Challenge {
    fn from(response: Response) -> Self {
        response.to_challenge()
    }
}

impl AsRef<[u8]> for Challenge {
    fn as_ref(&self) -> &[u8] {
        // Serializes as compressed point
        self.0.as_ref()
    }
}

impl TryFrom<&[u8]> for Challenge {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        // Accepts both compressed and uncompressed points
        Ok(Self(value.try_into()?))
    }
}

impl BytesRepresentable for Challenge {
    const SIZE: usize = CurvePoint::SIZE_COMPRESSED;
}

/// Represents a half-key used for the Proof-of-Relay.
///
/// Half-key is equivalent to a non-zero scalar in the field used by secp256k1, but the type
/// itself does not validate nor enforce this fact.
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
    /// This operation naturally enforces the underlying scalar to be non-zero.
    pub fn to_challenge(&self) -> HalfKeyChallenge {
        CurvePoint::from_exponent(&self.0)
            .map(|cp| HalfKeyChallenge::new(cp.as_compressed().as_bytes()))
            .expect("invalid public key")
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
    /// Size of the secp256k1 secret scalar representing the half key.
    const SIZE: usize = 32;
}

/// Represents a challenge for the half-key in Proof of Relay.
/// Half-key challenge is equivalent to a secp256k1 curve point.
/// Therefore, HalfKeyChallenge can be obtained from a HalfKey.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HalfKeyChallenge(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl Display for HalfKeyChallenge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Default for HalfKeyChallenge {
    fn default() -> Self {
        // Note that the default HalfKeyChallenge is the identity point on secp256k1, therefore
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

    pub fn to_address(&self) -> Address {
        PublicKey::try_from(self.0.as_ref())
            .expect("invalid half-key")
            .to_address()
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

impl std::hash::Hash for HalfKeyChallenge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl FromStr for HalfKeyChallenge {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl From<HalfKey> for HalfKeyChallenge {
    fn from(half_key: HalfKey) -> Self {
        half_key.to_challenge()
    }
}

/// Represents an Ethereum 256-bit hash value
/// This implementation instantiates the hash via Keccak256 digest.
#[derive(Clone, Copy, Eq, PartialEq, Default, PartialOrd, Ord, std::hash::Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hash(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl Debug for Hash {
    // Intentionally same as Display
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for Hash {
    type Err = GeneralError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Hash {
    /// Convenience method that creates a new hash by hashing this.
    pub fn hash(&self) -> Self {
        Self::create(&[&self.0])
    }

    /// Takes all the byte slices and computes hash of their concatenated value.
    /// Uses the Keccak256 digest.
    pub fn create(inputs: &[&[u8]]) -> Self {
        let mut output = Output::<Keccak256>::default();
        let mut hash = Keccak256::default();
        inputs.iter().for_each(|v| hash.update(v));
        hash.finalize_into(&mut output);
        Self(output.into())
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ParseError("Hash".into()))?))
    }
}

impl BytesRepresentable for Hash {
    /// Size of the digest is 32 bytes.
    const SIZE: usize = <Keccak256 as OutputSizeUser>::OutputSize::USIZE;
}

impl From<[u8; Self::SIZE]> for Hash {
    fn from(hash: [u8; Self::SIZE]) -> Self {
        Self(hash)
    }
}

impl From<Hash> for [u8; Hash::SIZE] {
    fn from(value: Hash) -> Self {
        value.0
    }
}

impl From<&Hash> for [u8; Hash::SIZE] {
    fn from(value: &Hash) -> Self {
        value.0
    }
}

impl From<Hash> for primitive_types::H256 {
    fn from(value: Hash) -> Self {
        value.0.into()
    }
}

impl From<primitive_types::H256> for Hash {
    fn from(value: primitive_types::H256) -> Self {
        Self(value.0)
    }
}

/// Represents an Ed25519 public key.
#[derive(Clone, Copy, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffchainPublicKey {
    compressed: CompressedEdwardsY,
    edwards: EdwardsPoint,
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

impl TryFrom<&PeerId> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: &PeerId) -> std::result::Result<Self, Self::Error> {
        let mh = value.as_ref();
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

impl TryFrom<PeerId> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: PeerId) -> std::result::Result<Self, Self::Error> {
        Self::try_from(&value)
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
/// ```rust
/// # use hex_literal::hex;
/// # use hopr_crypto_types::prelude::*;
/// # use k256::ecdsa::VerifyingKey;
///
/// const PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");
///
/// // compressed public keys start with `0x02` or `0x03``, depending on sign of y-component
/// const COMPRESSED: [u8; 33] = hex!("021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8");
///
/// // full public key without prefix
/// const UNCOMPRESSED_PLAIN: [u8; 64] = hex!("1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
///
/// // uncompressed public keys use `0x04` prefix
/// const UNCOMPRESSED: [u8; 65] = hex!("041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
///
/// let from_privkey = PublicKey::from_privkey(&PRIVATE_KEY).unwrap();
/// let from_compressed = PublicKey::try_from(COMPRESSED.as_ref()).unwrap();
/// let from_uncompressed_plain = PublicKey::try_from(UNCOMPRESSED_PLAIN.as_ref()).unwrap();
/// let from_uncompressed = PublicKey::try_from(UNCOMPRESSED.as_ref()).unwrap();
///
/// assert_eq!(from_privkey, from_uncompressed);
/// assert_eq!(from_compressed, from_uncompressed_plain);
/// assert_eq!(from_uncompressed_plain, from_uncompressed);
///
/// // also works from a signed Ethereum transaction
/// const TX_HASH: [u8; 32] = hex!("eff80b9f035b1d369c6a60f362ac7c8b8c3b61b76d151d1be535145ccaa3e83e");
///
/// const R: [u8; 32] = hex!("c8048d137fbb10ddffa1e4ba5141c300fcd19e4fb7d0a4354ca62a7694e46f9b");
/// const S: [u8; 32] = hex!("5bb43e23d8b430f17ba3649e38b5a94d02815f0bcfaaf171800c52d4794c3136");
/// const V: u8 = 1u8;
///
/// let mut r_and_s = Vec::<u8>::with_capacity(64);
/// r_and_s.extend_from_slice(&R);
/// r_and_s.extend_from_slice(&S);
///
/// let sig = Signature::new(&r_and_s, V);
///
/// let from_transaction_signature = PublicKey::from_signature_hash(&TX_HASH, &sig).unwrap();
///
/// assert_eq!(from_uncompressed, from_transaction_signature);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PublicKey(CurvePoint);

impl PublicKey {
    /// Size of the compressed public key in bytes
    pub const SIZE_COMPRESSED: usize = 33;
    /// Size of the uncompressed public key in bytes
    pub const SIZE_UNCOMPRESSED: usize = 65;
    pub const SIZE_UNCOMPRESSED_PLAIN: usize = 64;

    pub fn from_privkey(private_key: &[u8]) -> Result<PublicKey> {
        // This verifies that it is indeed a non-zero scalar, and thus represents a valid public key
        let secret_scalar = NonZeroScalar::<Secp256k1>::try_from(private_key)
            .map_err(|_| GeneralError::ParseError("PublicKey".into()))?;

        let key = elliptic_curve::PublicKey::<Secp256k1>::from_secret_scalar(&secret_scalar);
        Ok(key.into())
    }

    fn from_raw_signature<R>(msg: &[u8], r: &[u8], s: &[u8], v: u8, recovery_method: R) -> Result<PublicKey>
    where
        R: Fn(&[u8], &ECDSASignature, RecoveryId) -> std::result::Result<VerifyingKey, ecdsa::Error>,
    {
        let recid = RecoveryId::try_from(v).map_err(|_| GeneralError::ParseError("Signature".into()))?;
        let signature =
            ECDSASignature::from_scalars(GenericArray::clone_from_slice(r), GenericArray::clone_from_slice(s))
                .map_err(|_| GeneralError::ParseError("Signature".into()))?;

        let recovered_key = *recovery_method(msg, &signature, recid)
            .map_err(|_| CalculationError)?
            .as_affine();

        // Verify that it is a valid public key
        recovered_key.try_into()
    }

    pub fn from_signature(msg: &[u8], signature: &Signature) -> Result<PublicKey> {
        let (raw_signature, recovery) = signature.raw_signature();
        Self::from_raw_signature(
            msg,
            &raw_signature[0..Signature::SIZE / 2],
            &raw_signature[Signature::SIZE / 2..],
            recovery,
            VerifyingKey::recover_from_msg,
        )
    }

    pub fn from_signature_hash(hash: &[u8], signature: &Signature) -> Result<PublicKey> {
        let (raw_signature, recovery) = signature.raw_signature();
        Self::from_raw_signature(
            hash,
            &raw_signature[0..Signature::SIZE / 2],
            &raw_signature[Signature::SIZE / 2..],
            recovery,
            VerifyingKey::recover_from_prehash,
        )
    }

    /// Sums all given public keys together, creating a new public key.
    /// Panics if reaches infinity (EC identity point), which is an invalid public key.
    pub fn combine(summands: &[&PublicKey]) -> PublicKey {
        let cps = summands.iter().map(|pk| CurvePoint::from(*pk)).collect::<Vec<_>>();
        let cps_ref = cps.iter().collect::<Vec<_>>();

        // Verify that it is a valid public key
        CurvePoint::combine(&cps_ref)
            .try_into()
            .expect("combination results in the ec identity (which is an invalid pub key)")
    }

    /// Adds the given public key with `tweak` times secp256k1 generator, producing a new public key.
    /// Panics if reaches infinity (EC identity point), which is an invalid public key.
    pub fn tweak_add(key: &PublicKey, tweak: &[u8]) -> PublicKey {
        let scalar = NonZeroScalar::<Secp256k1>::try_from(tweak).expect("zero tweak provided");

        let new_pk = (key.0.clone().into_projective_point()
            + <Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar.as_ref())
        .to_affine();

        // Verify that it is a valid public key
        new_pk.try_into().expect("tweak add resulted in an invalid public key")
    }

    /// Converts the public key to an Ethereum address
    pub fn to_address(&self) -> Address {
        let uncompressed = self.to_bytes(false);
        let serialized = Hash::create(&[&uncompressed[1..]]);
        Address::new(&serialized.as_ref()[12..])
    }

    /// Serializes the public key to a binary form.
    pub fn to_bytes(&self, compressed: bool) -> Box<[u8]> {
        match compressed {
            true => self.0.as_compressed().to_bytes(),
            false => self.0.as_uncompressed().to_bytes(),
        }
    }

    /// Serializes the public key to a binary form and converts it to hexadecimal string representation.
    pub fn to_hex(&self, compressed: bool) -> String {
        let offset = if compressed { 0 } else { 1 };
        format!("0x{}", hex::encode(&self.to_bytes(compressed)[offset..]))
    }
}

impl Randomizable for PublicKey {
    /// Generates a new random public key.
    /// Because the corresponding private key is discarded, this might be useful only for testing purposes.
    fn random() -> Self {
        let (_, cp) = random_group_element();
        cp.try_into()
            .expect("random_group_element cannot generate identity points")
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex(true))
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

                // Already verified, this is a valid public key
                Ok(key.into())
            }
            Self::SIZE_UNCOMPRESSED_PLAIN => {
                // add 0x04 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(&[&[4u8], value].concat())
                    .map_err(|_| GeneralError::ParseError("invalid secp256k1 point".into()))?;

                // Already verified, this is a valid public key
                Ok(key.into())
            }
            Self::SIZE_COMPRESSED => {
                // has either 0x02 or 0x03 prefix
                let key = elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(value)
                    .map_err(|_| GeneralError::ParseError("invalid secp256k1 point".into()))?;

                // Already verified, this is a valid public key
                Ok(key.into())
            }
            _ => Err(GeneralError::ParseError("invalid secp256k1 point".into())),
        }
    }
}

impl TryFrom<AffinePoint> for PublicKey {
    type Error = CryptoError;

    fn try_from(value: AffinePoint) -> std::result::Result<Self, Self::Error> {
        if value.is_identity().into() {
            return Err(CryptoError::InvalidPublicKey);
        }
        Ok(Self(value.into()))
    }
}

impl TryFrom<CurvePoint> for PublicKey {
    type Error = CryptoError;

    fn try_from(value: CurvePoint) -> std::result::Result<Self, Self::Error> {
        if value.affine.is_identity().into() {
            return Err(CryptoError::InvalidPublicKey);
        }
        Ok(Self(value))
    }
}

impl From<elliptic_curve::PublicKey<Secp256k1>> for PublicKey {
    fn from(key: elliptic_curve::PublicKey<Secp256k1>) -> Self {
        Self((*key.as_affine()).into())
    }
}

// TODO: make this `for &k256::ProjectivePoint`
impl From<&PublicKey> for k256::ProjectivePoint {
    fn from(value: &PublicKey) -> Self {
        value.0.clone().into_projective_point()
    }
}

/// Represents a compressed serializable extension of the `PublicKey` using the secp256k1 curve.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct CompressedPublicKey(pub PublicKey);

impl TryFrom<&[u8]> for CompressedPublicKey {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(PublicKey::try_from(value)?.into())
    }
}

impl AsRef<[u8]> for CompressedPublicKey {
    fn as_ref(&self) -> &[u8] {
        // CurvePoint::as_ref() returns a compressed representation of the curve point
        self.0.0.as_ref()
    }
}

impl BytesRepresentable for CompressedPublicKey {
    const SIZE: usize = PublicKey::SIZE_COMPRESSED;
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
        f.write_str(self.to_hex().as_str())
    }
}

impl Response {
    /// Converts this response to the PoR challenge by turning the non-zero scalar
    /// represented by this response into a secp256k1 curve point (public key)
    pub fn to_challenge(&self) -> Challenge {
        Challenge(CurvePoint::from_exponent(&self.0).expect("response represents an invalid non-zero scalar"))
    }

    /// Derives the response from two half-keys.
    /// This is done by adding together the two non-zero scalars that the given half-keys represent.
    /// Returns an error if any of the given scalars is zero.
    pub fn from_half_keys(first: &HalfKey, second: &HalfKey) -> Result<Self> {
        let res = NonZeroScalar::<Secp256k1>::try_from(first.as_ref())
            .and_then(|s1| NonZeroScalar::<Secp256k1>::try_from(second.as_ref()).map(|s2| s1.as_ref() + s2.as_ref()))
            .map_err(|_| CalculationError)?; // One of the scalars was 0

        Ok(Response::try_from(res.to_bytes().as_slice())?)
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
    /// Fixed size of the PoR challenge response.
    const SIZE: usize = 32;
}

impl From<[u8; Self::SIZE]> for Response {
    fn from(value: [u8; Self::SIZE]) -> Self {
        Self(value)
    }
}

/// Represents an EdDSA signature using Ed25519 Edwards curve.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffchainSignature(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl OffchainSignature {
    /// Sign the given message using the [OffchainKeypair].
    pub fn sign_message(msg: &[u8], signing_keypair: &OffchainKeypair) -> Self {
        // Expand the SK from the given keypair
        let expanded_sk = ed25519_dalek::hazmat::ExpandedSecretKey::from(
            &ed25519_dalek::SecretKey::try_from(signing_keypair.secret().as_ref()).expect("invalid private key"),
        );

        // Get the verifying key from the SAME keypair, avoiding Double Public Key Signing Function Oracle Attack on
        // Ed25519 See https://github.com/MystenLabs/ed25519-unsafe-libs for details
        let verifying = ed25519_dalek::VerifyingKey::from(signing_keypair.public().edwards);

        ed25519_dalek::hazmat::raw_sign::<Sha512>(&expanded_sk, msg, &verifying).into()
    }

    /// Verify this signature of the given message and [OffchainPublicKey].
    pub fn verify_message(&self, msg: &[u8], public_key: &OffchainPublicKey) -> bool {
        let sgn = ed25519_dalek::Signature::from_slice(&self.0).expect("corrupted OffchainSignature");
        let pk = ed25519_dalek::VerifyingKey::from(public_key.edwards);
        pk.verify_strict(msg, &sgn).is_ok()
    }
}

impl AsRef<[u8]> for OffchainSignature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for OffchainSignature {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(ed25519_dalek::Signature::from_slice(value)
            .map_err(|_| ParseError("OffchainSignature".into()))?
            .into())
    }
}

impl BytesRepresentable for OffchainSignature {
    /// Size of the EdDSA signature using Ed25519.
    const SIZE: usize = ed25519_dalek::Signature::BYTE_SIZE;
}

impl From<ed25519_dalek::Signature> for OffchainSignature {
    fn from(value: ed25519_dalek::Signature) -> Self {
        let mut ret = Self([0u8; Self::SIZE]);
        ret.0.copy_from_slice(value.to_bytes().as_ref());
        ret
    }
}

impl TryFrom<([u8; 32], [u8; 32])> for OffchainSignature {
    type Error = GeneralError;

    fn try_from(value: ([u8; 32], [u8; 32])) -> std::result::Result<Self, Self::Error> {
        Ok(ed25519_dalek::Signature::from_components(value.0, value.1).into())
    }
}

/// Represents an ECDSA signature based on the secp256k1 curve with a recoverable public key.
/// This signature encodes the 2-bit recovery information into the
/// uppermost bits from MSB of the S value, which are never used by this ECDSA
/// instantiation over secp256k1.
/// The instance holds the byte array consisting of `R` and `S` values with the recovery bit
/// already embedded in S.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Signature(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl Signature {
    pub fn new(raw_bytes: &[u8], recovery: u8) -> Signature {
        assert!(recovery <= 1, "invalid recovery bit");

        let mut ret = Self([0u8; Self::SIZE]);
        ret.0.copy_from_slice(raw_bytes);
        ret.embed_recovery_bit(recovery);
        ret
    }

    fn sign<S>(data: &[u8], private_key: &[u8], signing_method: S) -> Signature
    where
        S: FnOnce(&SigningKey, &[u8]) -> ecdsa::signature::Result<(ECDSASignature, RecoveryId)>,
    {
        let key = SigningKey::from_bytes(private_key.into()).expect("invalid signing key");
        let (sig, rec) = signing_method(&key, data).expect("signing failed");

        Self::new(&sig.to_vec(), rec.to_byte())
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
        V: FnOnce(&VerifyingKey, &[u8], &ECDSASignature) -> ecdsa::signature::Result<()>,
    {
        let pub_key = VerifyingKey::from_sec1_bytes(public_key).expect("invalid public key");

        if let Ok(signature) = ECDSASignature::try_from(self.raw_signature().0.as_ref()) {
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

    /// Returns the raw signature, without the encoded public key recovery bit and
    /// the recovery bit as a separate value.
    pub fn raw_signature(&self) -> ([u8; Self::SIZE], u8) {
        let mut raw_sig = self.0;
        let recovery: u8 = (raw_sig[Self::SIZE / 2] & 0x80 != 0).into();
        raw_sig[Self::SIZE / 2] &= 0x7f;
        (raw_sig, recovery)
    }

    fn embed_recovery_bit(&mut self, recovery: u8) {
        self.0[Self::SIZE / 2] &= 0x7f;
        self.0[Self::SIZE / 2] |= recovery << 7;
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ParseError("Signature".into()))?))
    }
}

impl BytesRepresentable for Signature {
    const SIZE: usize = 64;
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Signature {}

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

    use ed25519_dalek::Signer;
    use hex_literal::hex;
    use hopr_primitive_types::prelude::*;
    use k256::{
        AffinePoint, NonZeroScalar, Secp256k1, U256,
        ecdsa::VerifyingKey,
        elliptic_curve::{CurveArithmetic, sec1::ToEncodedPoint},
    };
    use libp2p_identity::PeerId;

    use crate::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{
            Challenge, CurvePoint, HalfKey, HalfKeyChallenge, Hash, OffchainPublicKey, OffchainSignature, PublicKey,
            Response, Signature,
        },
        utils::random_group_element,
    };

    const PUBLIC_KEY: [u8; 33] = hex!("021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8");
    const PUBLIC_KEY_UNCOMPRESSED_PLAIN: [u8; 64] = hex!("1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
    const PUBLIC_KEY_UNCOMPRESSED: [u8; 65] = hex!("041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e");
    const PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    fn test_signature_signing() -> anyhow::Result<()> {
        let msg = b"test12345";
        let kp = ChainKeypair::from_secret(&PRIVATE_KEY)?;
        let sgn = Signature::sign_message(msg, &kp);

        let expected_pk = PublicKey::try_from(PUBLIC_KEY.as_ref())?;
        assert!(sgn.verify_message(msg, &expected_pk));

        let extracted_pk = PublicKey::from_signature(msg, &sgn)?;
        assert_eq!(expected_pk, extracted_pk, "key extracted from signature does not match");

        Ok(())
    }

    #[test]
    fn test_offchain_signature_signing() -> anyhow::Result<()> {
        let msg = b"test12345";
        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY)?;

        let key = ed25519_dalek::SecretKey::try_from(PRIVATE_KEY)?;
        let kp = ed25519_dalek::SigningKey::from_bytes(&key);
        let pk = ed25519_dalek::VerifyingKey::from(&kp);

        let sgn = kp.sign(msg);
        assert!(pk.verify_strict(msg, &sgn).is_ok(), "blomp");

        let sgn_1 = OffchainSignature::sign_message(msg, &keypair);
        let sgn_2 = OffchainSignature::try_from(sgn_1.as_ref())?;

        assert!(
            sgn_1.verify_message(msg, keypair.public()),
            "cannot verify message via sig 1"
        );
        assert!(
            sgn_2.verify_message(msg, keypair.public()),
            "cannot verify message via sig 2"
        );
        assert_eq!(sgn_1, sgn_2, "signatures must be equal");

        Ok(())
    }

    #[test]
    fn test_signature_serialize() -> anyhow::Result<()> {
        let msg = b"test000000";
        let kp = ChainKeypair::from_secret(&PRIVATE_KEY)?;
        let sgn = Signature::sign_message(msg, &kp);

        let deserialized = Signature::try_from(sgn.as_ref())?;
        assert_eq!(sgn, deserialized, "signatures don't match");

        Ok(())
    }

    #[test]
    fn test_offchain_signature() -> anyhow::Result<()> {
        let msg = b"test12345";
        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY)?;

        let key = ed25519_dalek::SecretKey::try_from(PRIVATE_KEY)?;
        let kp = ed25519_dalek::SigningKey::from_bytes(&key);
        let pk = ed25519_dalek::VerifyingKey::from(&kp);

        let sgn = kp.sign(msg);
        assert!(pk.verify_strict(msg, &sgn).is_ok(), "blomp");

        let sgn_1 = OffchainSignature::sign_message(msg, &keypair);
        let sgn_2 = OffchainSignature::try_from(sgn_1.as_ref())?;

        assert!(
            sgn_1.verify_message(msg, keypair.public()),
            "cannot verify message via sig 1"
        );
        assert!(
            sgn_2.verify_message(msg, keypair.public()),
            "cannot verify message via sig 2"
        );
        assert_eq!(sgn_1, sgn_2, "signatures must be equal");
        // let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY)?;

        // let sig = OffchainSignature::sign_message("my test msg".as_bytes(), &keypair);

        // assert!(sig.verify_message("my test msg".as_bytes(), keypair.public()));

        Ok(())
    }

    #[test]
    fn test_public_key_to_hex() -> anyhow::Result<()> {
        let pk = PublicKey::from_privkey(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        assert_eq!("0x39d1bc2291826eaed86567d225cf243ebc637275e0a5aedb0d6b1dc82136a38e428804340d4c949a029846f682711d046920b4ca8b8ebeb9d1192b5bdaa54dba",
            pk.to_hex(false));
        assert_eq!(
            "0x0239d1bc2291826eaed86567d225cf243ebc637275e0a5aedb0d6b1dc82136a38e",
            pk.to_hex(true)
        );

        Ok(())
    }

    #[test]
    fn test_public_key_recover() -> anyhow::Result<()> {
        let address = Address::from_str("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266")?;

        let r = hex!("bcae4d37e3a1cd869984d1d68f9242291773cd33d26f1e754ecc1a9bfaee7d17");
        let s = hex!("0b755ab5f6375595fc7fc245c45f6598cc873719183733f4c464d63eefd8579b");
        let v = 1u8;

        let hash = hex!("fac7acad27047640b069e8157b61623e3cb6bb86e6adf97151f93817c291f3cf");

        assert_eq!(
            address,
            PublicKey::from_raw_signature(&hash, &r, &s, v, VerifyingKey::recover_from_prehash)?.to_address()
        );

        Ok(())
    }

    #[test]
    fn test_public_key_combine_tweak() -> anyhow::Result<()> {
        let (scalar1, point1) = random_group_element();
        let (scalar2, point2) = random_group_element();

        let pk1 = PublicKey::try_from(point1)?;
        let pk2 = PublicKey::try_from(point2)?;

        let sum = PublicKey::combine(&[&pk1, &pk2]);
        let tweak1 = PublicKey::tweak_add(&pk1, &scalar2);
        assert_eq!(sum, tweak1);

        let tweak2 = PublicKey::tweak_add(&pk2, &scalar1);
        assert_eq!(sum, tweak2);

        Ok(())
    }

    #[test]
    fn test_sign_and_recover() -> anyhow::Result<()> {
        let msg = hex!("eff80b9f035b1d369c6a60f362ac7c8b8c3b61b76d151d1be535145ccaa3e83e");

        let kp = ChainKeypair::from_secret(&PRIVATE_KEY)?;

        let signature1 = Signature::sign_message(&msg, &kp);
        let signature2 = Signature::sign_hash(&msg, &kp);

        let pub_key1 = PublicKey::from_privkey(&PRIVATE_KEY)?;
        let pub_key2 = PublicKey::from_signature(&msg, &signature1)?;
        let pub_key3 = PublicKey::from_signature_hash(&msg, &signature2)?;

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

        Ok(())
    }

    #[test]
    fn test_public_key_serialize() -> anyhow::Result<()> {
        let pk1 = PublicKey::try_from(PUBLIC_KEY.as_ref())?;
        let pk2 = PublicKey::try_from(pk1.to_bytes(true).as_ref())?;
        let pk3 = PublicKey::try_from(pk1.to_bytes(false).as_ref())?;

        assert_eq!(pk1, pk2, "pub keys 1 2 don't match");
        assert_eq!(pk2, pk3, "pub keys 2 3 don't match");

        let pk1 = PublicKey::try_from(PUBLIC_KEY.as_ref())?;
        let pk2 = PublicKey::try_from(PUBLIC_KEY_UNCOMPRESSED.as_ref())?;
        let pk3 = PublicKey::try_from(PUBLIC_KEY_UNCOMPRESSED_PLAIN.as_ref())?;

        assert_eq!(pk1, pk2, "pubkeys don't match");
        assert_eq!(pk2, pk3, "pubkeys don't match");

        assert_eq!(PublicKey::SIZE_COMPRESSED, pk1.to_bytes(true).len());
        assert_eq!(PublicKey::SIZE_UNCOMPRESSED, pk1.to_bytes(false).len());

        let shorter = hex!("f85e38b056284626a7aed0acc5d474605a408e6cccf76d7241ec7b4dedb31929b710e034f4f9a7dba97743b01e1cc35a45a60bebb29642cb0ba6a7fe8433316c");
        let s1 = PublicKey::try_from(shorter.as_ref())?;
        let s2 = PublicKey::try_from(s1.to_bytes(false).as_ref())?;
        assert_eq!(s1, s2);

        Ok(())
    }

    #[test]
    fn test_public_key_should_not_accept_identity() -> anyhow::Result<()> {
        let cp: CurvePoint = AffinePoint::IDENTITY.into();

        PublicKey::try_from(cp).expect_err("must fail for identity point");
        PublicKey::try_from(AffinePoint::IDENTITY).expect_err("must fail for identity point");

        Ok(())
    }

    #[test]
    fn test_public_key_curve_point() -> anyhow::Result<()> {
        let cp1: CurvePoint = PublicKey::try_from(PUBLIC_KEY.as_ref())?.into();
        let cp2 = CurvePoint::try_from(cp1.as_ref())?;
        assert_eq!(cp1, cp2);

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
        let valid = OffchainPublicKey::try_from(valid_peerid)?;
        assert_eq!(valid_peerid, valid.into(), "must work with ed25519 peer ids");

        let invalid_peerid = PeerId::from_str("16Uiu2HAmPHGyJ7y1Rj3kJ64HxJQgM9rASaeT2bWfXF9EiX3Pbp3K")?;
        let invalid = OffchainPublicKey::try_from(invalid_peerid);
        assert!(invalid.is_err(), "must not work with secp256k1 peer ids");

        let invalid_peerid_2 = PeerId::from_str("QmWvEwidPYBbLHfcZN6ATHdm4NPM4KbUx72LZnZRoRNKEN")?;
        let invalid_2 = OffchainPublicKey::try_from(invalid_peerid_2);
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
    fn test_curve_point() -> anyhow::Result<()> {
        let scalar = NonZeroScalar::from_uint(U256::from_u8(100)).expect("should hold a value");
        let test_point = (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar.as_ref()).to_affine();

        let cp1 = CurvePoint::from_str(hex::encode(test_point.to_encoded_point(false).to_bytes()).as_str())?;

        let cp2 = CurvePoint::try_from(cp1.as_ref())?;

        assert_eq!(cp1, cp2, "failed to match deserialized curve point");

        let pk = PublicKey::from_privkey(&scalar.to_bytes())?;

        assert_eq!(
            cp1.to_address(),
            pk.to_address(),
            "failed to match curve point address with pub key address"
        );

        let ch1 = Challenge(cp1);
        let ch2 = Challenge(cp2);

        assert_eq!(ch1.to_ethereum_challenge(), ch2.to_ethereum_challenge());
        assert_eq!(ch1, ch2, "failed to match ethereum challenges from curve points");

        // Must be able to create from compressed and uncompressed data
        let scalar2 = NonZeroScalar::from_uint(U256::from_u8(123)).expect("should hold a value");
        let test_point2 = (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * scalar2.as_ref()).to_affine();
        let uncompressed = test_point2.to_encoded_point(false);
        assert!(!uncompressed.is_compressed(), "given point is compressed");

        let compressed = uncompressed.compress();
        assert!(compressed.is_compressed(), "failed to compress points");

        let cp3 = CurvePoint::try_from(uncompressed.as_bytes())?;
        let cp4 = CurvePoint::try_from(compressed.as_bytes())?;

        assert_eq!(
            cp3, cp4,
            "failed to match curve point from compressed and uncompressed source"
        );

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
