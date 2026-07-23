use std::{
    collections::{BTreeMap, HashMap},
    num::NonZero,
    ops::Add,
};

use hopr_types::{
    crypto::{
        crypto_traits::{
            self, KeyIvInit, StreamCipher,
            elliptic_curve::{Curve, PrimeField},
        },
        prelude::HalfKey,
        primitives::Blake3,
    },
    primitive::{
        hybrid_array::{
            Array, ArraySize,
            typenum::{Sum, U, Unsigned},
        },
        prelude::{BytesRepresentable, GeneralError},
    },
};

use crate::{
    ExitAcknowledgementShareProcessor, PartialSsaShareVerifier, PixGroup, PixGroupRepr, PixScalar, PixSpec, errors,
    errors::PixError,
};

/// Raw zeroable SSA Index.
pub type RawSsaIndex = u32;

/// Type used to index Session Stealth Addresses (SSA).
///
/// Note that SSA Index starts with 1.
pub type SsaIndex = NonZero<RawSsaIndex>;

/// Type used to index polynomials that reconstruct parts of a Session Stealth Addresses (SSA).
///
/// The index is 0-based.
pub type PolynomialIndex = u16;

/// Type used to index coefficients in a polynomial.
///
/// The index is 0-based.
pub type CoefficientIndex = u16;

/// Byte size of the [`SsaIndex`] when serialized as a big-endian prefix.
const SSA_INDEX_SIZE: usize = size_of::<SsaIndex>();

/// Byte size of the [`PolynomialIndex`] when serialized as a big-endian prefix.
const POLY_INDEX_SIZE: usize = size_of::<PolynomialIndex>();

/// [`typenum`]: hopr_types::primitive::hybrid_array::typenum
/// Size of the [`SsaIndex`] and [`PolynomialIndex`] prefix prepended to the encrypted share.
///
/// Derived at compile time from `size_of::<SsaIndex>() + size_of::<PolynomialIndex>()` via
/// `typenum`'s `Const`/`ToUInt` machinery. The matching runtime invariant is asserted by
/// the `size_of_indices_must_match` unit test below.
pub type SsaPolyIndexPrefixSize = Sum<U<SSA_INDEX_SIZE>, U<POLY_INDEX_SIZE>>;

/// Uniquely identifies a Session Stealth Address (SSA).
///
/// This consists of a pseudonym and [`SsaIndex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaId<P> {
    pseudonym: P,
    ssa_index: SsaIndex,
}

impl<P> SsaId<P> {
    /// Creates a new `SsaId` with the given pseudonym and SSA index.
    pub fn new(pseudonym: P, ssa_index: SsaIndex) -> Self {
        Self { pseudonym, ssa_index }
    }

    /// Pseudonym part of the `HoprSenderId`.
    #[inline]
    pub fn pseudonym(&self) -> &P {
        &self.pseudonym
    }

    /// Index (i-value) of the Session Stealth Address (SSA).
    #[inline]
    pub fn ssa_index(&self) -> SsaIndex {
        self.ssa_index
    }
}

impl<P: std::fmt::Display> std::fmt::Display for SsaId<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-ssa#{}", self.pseudonym, self.ssa_index)
    }
}

/// Uniquely identifies a polynomial that allows forming a Session Stealth Address (SSA) corresponding
/// to a specific Session.
///
/// The index consists of the following parts:
/// 1. The Pseudonym part of the `HoprSenderId` - fixed for the given Session.
/// 2. Index (i) of the Session Stealth Address (SSA)
/// 3. Index (j) of the polynomial used to reconstruct the portion of the SSA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaPolynomialId<P> {
    id: SsaId<P>,
    poly_index: PolynomialIndex,
}

/// Transposed verifiers of the partial SSA shares.
pub type TransposedVerifiers<S> = HashMap<CoefficientIndex, Vec<(PolynomialIndex, PixGroupRepr<S>)>>;

impl<P> SsaPolynomialId<P> {
    /// Creates a new `SsaPolynomialId` with the given `SsaId` and polynomial index.
    pub fn new(id: SsaId<P>, poly_index: PolynomialIndex) -> Self {
        Self { id, poly_index }
    }

    /// Pseudonym part of the `HoprSenderId`.
    #[inline]
    pub fn pseudonym(&self) -> &P {
        &self.id.pseudonym
    }

    /// Index (i-value) of the Session Stealth Address (SSA).
    #[inline]
    pub fn ssa_index(&self) -> SsaIndex {
        self.id.ssa_index
    }

    /// Index (j-value) of the polynomial used to reconstruct the portion of the SSA.
    #[inline]
    pub fn poly_index(&self) -> PolynomialIndex {
        self.poly_index
    }
}

impl<P> AsRef<SsaId<P>> for SsaPolynomialId<P> {
    fn as_ref(&self) -> &SsaId<P> {
        &self.id
    }
}

impl<P: std::fmt::Display> std::fmt::Display for SsaPolynomialId<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.poly_index)
    }
}

/// Share of a polynomial used to reconstruct a portion of the Session Stealth Address (SSA).
///
/// This corresponds to the `P_ij(X)` of the polynomial used to reconstruct the j-th portion of i-th SSA
/// at some value `X`.
///
/// The struct does not hold the `X` value, as it is usually computed from the
/// [`nonce`](TaggedEncryptedPartialSsaShare).
///
/// See [`TaggedEncryptedPartialSsaShare`] and [`EncryptedPartialSsaShare`] for more details.
#[derive(Clone, Default, Hash, PartialEq, Eq)]
pub struct PartialSsaShare<S: PixSpec>(pub(crate) <PixScalar<S> as PrimeField>::Repr);

impl<S: PixSpec> std::fmt::Debug for PartialSsaShare<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartialSsaShare").finish_non_exhaustive()
    }
}

impl<S: PixSpec> PartialSsaShare<S> {
    /// Encrypts this partial SSA share using the given acknowledgement [`HalfKey`].
    pub fn encrypt(
        mut self,
        spi: &SsaPolynomialId<S::Pseudonym>,
        ack_key: &HalfKey,
    ) -> errors::Result<EncryptedPartialSsaShare<S>, S::Pseudonym>
    where
        FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
        EncShareSize<S>: ArraySize,
    {
        let mut cipher = derive_ssa_encryption_key::<S>(spi, ack_key)?;
        cipher.apply_keystream(self.0.as_mut());

        let mut out = Array::<u8, EncShareSize<S>>::default();
        out[0..size_of::<SsaIndex>()].copy_from_slice(&spi.ssa_index().get().to_be_bytes());
        out[size_of::<SsaIndex>()..size_of::<SsaIndex>() + size_of::<PolynomialIndex>()]
            .copy_from_slice(&spi.poly_index().to_be_bytes());
        out[size_of::<SsaIndex>() + size_of::<PolynomialIndex>()..].copy_from_slice(self.0.as_ref());
        Ok(EncryptedPartialSsaShare(out))
    }
}

impl<S: PixSpec> AsRef<<PixScalar<S> as PrimeField>::Repr> for PartialSsaShare<S> {
    fn as_ref(&self) -> &<PixScalar<S> as PrimeField>::Repr {
        &self.0
    }
}

fn derive_ssa_encryption_key<S: PixSpec>(
    spi: &SsaPolynomialId<S::Pseudonym>,
    ack: &HalfKey,
) -> errors::Result<S::Cipher, S::Pseudonym> {
    let mut output = Blake3::new_derive_key(S::KEY_DERIVATION_CONTEXT)
        .update_reader(ack.as_ref())
        .and_then(|h| h.update_reader(spi.id.pseudonym.as_ref()))
        .and_then(|h| h.update_reader(spi.id.ssa_index.get().to_be_bytes().as_ref()))
        .and_then(|h| h.update_reader(spi.poly_index.to_be_bytes().as_ref()))
        .map_err(|_| hopr_types::crypto::errors::CryptoError::InvalidInputValue("invalid ssa encryption key"))?
        .finalize_xof();

    let mut key = crypto_traits::Key::<S::Cipher>::default();
    let mut iv = crypto_traits::Iv::<S::Cipher>::default();

    let mut out = vec![0u8; key.len() + iv.len()];
    output.fill(&mut out);

    let (v_iv, v_key) = out.split_at(iv.len());
    iv.copy_from_slice(v_iv);
    key.copy_from_slice(v_key);

    Ok(S::Cipher::new(&key, &iv))
}

/// Size of the field-bytes portion of the encrypted share.
pub type FieldBytesSize<S> = <<S as PixSpec>::Curve as Curve>::FieldBytesSize;

/// Total size of the [`EncryptedPartialSsaShare`] internal representation:
/// `SsaPolyIndexPrefixSize` + FieldBytesSize.
pub type EncShareSize<S> = Sum<FieldBytesSize<S>, SsaPolyIndexPrefixSize>;

/// Contains an encrypted partial Session Stealth Address (SSA) share.
///
/// The internal byte layout is:
/// 1. [`SsaIndex`] (big-endian, 4 bytes)
/// 2. [`PolynomialIndex`] (big-endian, 2 bytes)
/// 3. The encrypted scalar share (FieldBytesSize bytes)
///
/// This share can be decrypted to [`PartialSsaShare`]
/// to be verified and used for reconstruction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct EncryptedPartialSsaShare<S: PixSpec>(Array<u8, EncShareSize<S>>)
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize;

#[cfg(feature = "serde")]
impl<S: PixSpec> serde::Serialize for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize,
{
    fn serialize<Ser: serde::Serializer>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error> {
        serde_bytes::Bytes::new(self.0.as_ref()).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, S: PixSpec> serde::Deserialize<'de> for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = <&serde_bytes::Bytes as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_from(bytes.as_ref()).map_err(serde::de::Error::custom)
    }
}

impl<S: PixSpec> EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize,
{
    /// [`SsaIndex`] and [`PolynomialIndex`] embedded in this encrypted share.
    ///
    /// Returns `None` if the share is empty.
    pub fn indices(&self) -> Option<(SsaIndex, PolynomialIndex)> {
        let ssa_index: Option<SsaIndex> =
            RawSsaIndex::from_be_bytes(self.0[0..size_of::<RawSsaIndex>()].try_into().ok()?)
                .try_into()
                .ok();

        let poly_index = PolynomialIndex::from_be_bytes(
            self.0[size_of::<RawSsaIndex>()..size_of::<RawSsaIndex>() + size_of::<PolynomialIndex>()]
                .try_into()
                .ok()?,
        );

        ssa_index.map(|ssa_index| (ssa_index, poly_index))
    }

    /// Tries to decrypt the encrypted partial SSA share using the provided pseudonym and
    /// acknowledgement [`HalfKey`].
    ///
    /// NOTE: that the share must be verified by the reconstructor.
    pub(crate) fn decrypt(
        self,
        pseudonym: &S::Pseudonym,
        ack_key: &HalfKey,
    ) -> errors::Result<PartialSsaShare<S>, S::Pseudonym> {
        if let Some((ssa_index, poly_index)) = self.indices() {
            let spi = SsaPolynomialId::new(SsaId::new(*pseudonym, ssa_index), poly_index);
            let mut cipher = derive_ssa_encryption_key::<S>(&spi, ack_key)?;
            let mut share = <PixScalar<S> as PrimeField>::Repr::default();
            share.copy_from_slice(&self.0[SsaPolyIndexPrefixSize::USIZE..]);
            cipher.apply_keystream(share.as_mut());
            Ok(PartialSsaShare(share))
        } else {
            Err(PixError::ShareIsEmpty)
        }
    }

    /// Returns true if the encrypted share is empty (all zeroes or zero SSA index).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == Array::<u8, EncShareSize<S>>::default() || self.indices().is_none()
    }
}

impl<S: PixSpec + Copy> Copy for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize,
    Array<u8, EncShareSize<S>>: Copy,
{
}

impl<S: PixSpec> AsRef<[u8]> for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize,
{
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a, S: PixSpec> TryFrom<&'a [u8]> for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize,
{
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Array::try_from(value)
            .map(Self)
            .map_err(|_| GeneralError::ParseError("EncryptedPartialSsaShare.size".into()))
    }
}

impl<S: PixSpec> BytesRepresentable for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArraySize,
{
    const SIZE: usize = EncShareSize::<S>::USIZE;
}

/// This is a wrapped [`EncryptedPartialSsaShare`] extracted from a specific SURB (presumably along with the associated
/// `AcknowledgementChallenge`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "P: serde::Serialize, T: serde::Serialize, EncryptedPartialSsaShare<S>: serde::Serialize",
        deserialize = "P: serde::Deserialize<'de>, T: serde::Deserialize<'de>, EncryptedPartialSsaShare<S>: \
                       serde::Deserialize<'de>"
    ))
)]
pub struct TaggedEncryptedPartialSsaShare<S: PixSpec, P = <S as PixSpec>::Pseudonym, T = PixScalar<S>> {
    /// Pseudonym of the sender that created the encrypted partial SSA share.
    pub pseudonym: P,
    /// Nonce used to generate the encrypted partial SSA share.
    ///
    /// This must typically be convertible to [`PixScalar`] of `S`.
    pub nonce: T,
    /// Encrypted partial SSA share.
    pub partial_share: EncryptedPartialSsaShare<S>,
}

impl<S: PixSpec> TaggedEncryptedPartialSsaShare<S, S::Pseudonym, PixScalar<S>> {
    /// Creates a new tagged encrypted partial SSA share from the
    /// [encrypted partial SSA share](TaggedEncryptedPartialSsaShare::partial_share) that must correspond
    /// to the given `pseudonym` and `nonce`.
    pub fn new(
        pseudonym: S::Pseudonym,
        nonce: &impl AsRef<[u8]>,
        partial_share: EncryptedPartialSsaShare<S>,
    ) -> errors::Result<Self, S::Pseudonym> {
        if let Some((ssa_index, poly_index)) = partial_share.indices() {
            Ok(Self {
                pseudonym,
                nonce: S::msg_to_scalar(
                    &SsaPolynomialId::new(SsaId::new(pseudonym, ssa_index), poly_index),
                    nonce.as_ref(),
                )?,
                partial_share,
            })
        } else {
            Err(PixError::ShareIsEmpty)
        }
    }
}

impl<S: PixSpec> TaggedEncryptedPartialSsaShare<S, S::Pseudonym, PixScalar<S>> {
    /// SSA ID this share corresponds to.
    ///
    /// Returns `None` if the share is empty.
    #[inline]
    pub fn ssa_id(&self) -> Option<SsaId<S::Pseudonym>> {
        self.partial_share
            .indices()
            .map(|(ssa_index, _)| SsaId::new(self.pseudonym, ssa_index))
    }

    /// SSA polynomial ID this share corresponds to.
    ///
    /// Returns `None` if the share is empty.
    #[inline]
    pub fn ssa_polynomial_id(&self) -> Option<SsaPolynomialId<S::Pseudonym>> {
        self.partial_share
            .indices()
            .map(|(ssa_index, poly_index)| SsaPolynomialId::new(SsaId::new(self.pseudonym, ssa_index), poly_index))
    }
}

impl<S: PixSpec + Copy, P: Copy, T: Copy> Copy for TaggedEncryptedPartialSsaShare<S, P, T> where
    EncryptedPartialSsaShare<S>: Copy
{
}

/// Contains a generated share from a specific previously committed SSA.
#[derive(Clone, PartialEq, Eq)]
pub struct GeneratedShare<S: PixSpec, P = <S as PixSpec>::Pseudonym> {
    /// ID of the polynomial corresponding to the partial SSA share.
    pub id: SsaPolynomialId<P>,
    /// Generated partial SSA share.
    pub share: PartialSsaShare<S>,
}

impl<S: PixSpec, P: std::fmt::Debug> std::fmt::Debug for GeneratedShare<S, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeneratedShare")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<S: PixSpec> GeneratedShare<S, S::Pseudonym> {
    /// Convenience method to [encrypt](PartialSsaShare::encrypt) the share using an acknowledgement [`HalfKey`].
    #[inline]
    pub fn encrypt(self, ack: &HalfKey) -> errors::Result<EncryptedPartialSsaShare<S>, S::Pseudonym> {
        self.share.encrypt(&self.id, ack)
    }
}

/// Contains commitment to a specific SSA and corresponding verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaCommitment<S: PixSpec, P = <S as PixSpec>::Pseudonym> {
    /// ID of the SSA that is being committed to.
    pub ssa_id: SsaId<P>,
    /// Commitment to the SSA.
    #[cfg_attr(feature = "serde", serde(with = "elliptic_curve_tools::group"))]
    pub ssa_commitment: PixGroup<S>,
    /// Verifiers of the partial SSA shares.
    #[cfg_attr(
        feature = "serde",
        serde(bound(
            serialize = "PixGroupRepr<S>: serde::Serialize",
            deserialize = "PixGroupRepr<S>: serde::Deserialize<'de>"
        ))
    )]
    pub verifiers: TransposedVerifiers<S>,
}

impl<S: PixSpec, P> IntoIterator for SsaCommitment<S, P> {
    type IntoIter = std::collections::hash_map::IntoIter<CoefficientIndex, Vec<(PolynomialIndex, PixGroupRepr<S>)>>;
    type Item = (CoefficientIndex, Vec<(PolynomialIndex, PixGroupRepr<S>)>);

    fn into_iter(self) -> Self::IntoIter {
        self.verifiers.into_iter()
    }
}

impl<S: PixSpec> SsaCommitment<S, S::Pseudonym> {
    /// Reconstructs the verifiers from the internal transposed representation.
    pub fn reconstruct_verifiers(self) -> errors::Result<Vec<PartialSsaShareVerifier<S>>, S::Pseudonym> {
        let mut poly_coeffs: BTreeMap<PolynomialIndex, BTreeMap<CoefficientIndex, PixGroupRepr<S>>> = BTreeMap::new();
        for (coeff_idx, coeffs) in self.verifiers {
            for (poly_idx, commitment) in coeffs {
                poly_coeffs.entry(poly_idx).or_default().insert(coeff_idx, commitment);
            }
        }

        poly_coeffs
            .into_iter()
            .map(|(poly_idx, coeffs)| {
                let spi = SsaPolynomialId::new(self.ssa_id, poly_idx);
                let sorted_coeffs: Vec<_> = coeffs.into_values().collect();
                PartialSsaShareVerifier::from_serializable_commitments(spi, sorted_coeffs)
            })
            .collect()
    }

    /// Shorthand to pass all the coefficient commitments into the [reconstructor](ExitAcknowledgementShareProcessor).
    pub fn process_into_reconstructor<R: ExitAcknowledgementShareProcessor<S>>(
        self,
        reconstructor: &R,
    ) -> Result<(), R::Error> {
        for (coeff_idx, coeffs) in self.verifiers {
            reconstructor.insert_coefficient_commitments(self.ssa_id, coeff_idx, coeffs.into_iter())?;
        }
        Ok(())
    }
}

/// Represents the current state of a specific SSA commitment on an Exit node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaCommitmentState<P, A> {
    /// ID of the SSA that is being committed to.
    pub ssa_id: SsaId<P>,
    /// Commitment to the SSA, if it's already known.
    pub ssa_deposit_address: Option<A>,
    /// Whether the commitment is fully committed and therefore its partial shares are verifiable.
    pub is_verifiable: bool,
    /// Whether this SSA was encountered for the first time.
    pub is_first_encountered: bool,
    /// Whether the SSA deposit address has been discovered.
    pub deposit_address_first_encountered: bool,
}

impl<P, A> SsaCommitmentState<P, A> {
    /// Creates a new SsaCommitmentState for the given SSA ID.
    ///
    /// It has no associated commitment and is not verifiable initially.
    pub fn new(ssa_id: SsaId<P>) -> Self {
        Self {
            ssa_id,
            ssa_deposit_address: None,
            is_verifiable: false,
            is_first_encountered: true,
            deposit_address_first_encountered: false,
        }
    }
}

/// Contains the already recovered secret scalar corresponding to a specific SSA.
///
/// `P` is the pseudonym type, `A` is the private key type for SSA.
#[derive(Clone, Copy)]
pub struct RecoveredSsa<P, A> {
    /// ID of the SSA that was recovered.
    pub ssa_id: SsaId<P>,
    /// Recovered secret scalar (private key corresponding to the SSA deposit address).
    pub ssa: A,
}

impl<P: std::fmt::Debug, A> std::fmt::Debug for RecoveredSsa<P, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveredSsa")
            .field("ssa_id", &self.ssa_id)
            .finish_non_exhaustive()
    }
}

impl<P: PartialEq, A> PartialEq for RecoveredSsa<P, A> {
    fn eq(&self, other: &Self) -> bool {
        self.ssa_id == other.ssa_id
    }
}

impl<P: Eq, A> Eq for RecoveredSsa<P, A> {}

impl<P: std::hash::Hash, A> std::hash::Hash for RecoveredSsa<P, A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ssa_id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use hopr_types::{crypto::types::SimplePseudonym, crypto_random::Randomizable};
    use vsss_rs::elliptic_curve::Field;

    use super::*;
    use crate::tests::TestSpec;

    #[test]
    fn test_tagged_encrypted_partial_ssa_share_traits() {
        let pseudonym = SimplePseudonym::random();
        let nonce = 42u64;
        let partial_share = EncryptedPartialSsaShare::<TestSpec>::default();
        let tagged = TaggedEncryptedPartialSsaShare {
            pseudonym,
            nonce,
            partial_share,
        };

        // Test Clone
        #[allow(clippy::clone_on_copy)]
        let cloned = tagged.clone();
        assert_eq!(tagged, cloned);

        // Test Copy
        let copied = tagged;
        assert_eq!(tagged, copied);

        // Test Debug
        let debug_str = format!("{:?}", tagged);
        assert!(debug_str.contains("TaggedEncryptedPartialSsaShare"));
        assert!(debug_str.contains("pseudonym"));
        assert!(debug_str.contains("nonce"));
        assert!(debug_str.contains("partial_share"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_tagged_encrypted_partial_ssa_share_serde() {
        let pseudonym = SimplePseudonym::random();
        let nonce = 42u64;
        let partial_share = EncryptedPartialSsaShare::<TestSpec>::default();
        let _tagged = TaggedEncryptedPartialSsaShare {
            pseudonym,
            nonce,
            partial_share,
        };
        // Compile-time check only for now as no serde_json is available in this crate
    }

    #[test]
    fn size_of_indices_must_match() {
        let actual = size_of::<SsaIndex>() + size_of::<PolynomialIndex>();
        let expected = SsaPolyIndexPrefixSize::USIZE;
        assert_eq!(actual, expected);
    }

    #[test]
    fn default_enc_share_is_empty() {
        assert!(EncryptedPartialSsaShare::<TestSpec>::default().is_empty());
        assert!(EncryptedPartialSsaShare::<TestSpec>::default().indices().is_none());
    }

    #[test]
    fn ssa_part_shares_should_encrypt_and_decrypt() -> anyhow::Result<()> {
        let key = HalfKey::random();
        let spi = SsaPolynomialId::<SimplePseudonym>::new(SsaId::new(SimplePseudonym::random(), 1.try_into()?), 0);
        let scalar = PixScalar::<TestSpec>::random(&mut hopr_types::crypto_random::rng());

        let share_1 = PartialSsaShare::<TestSpec>(scalar.to_repr());
        let share_2 = share_1.clone().encrypt(&spi, &key)?.decrypt(spi.pseudonym(), &key)?;

        assert_eq!(share_1, share_2);
        Ok(())
    }

    #[test]
    fn debug_redaction_partial_ssa_share() {
        let scalar = PixScalar::<TestSpec>::random(&mut hopr_types::crypto_random::rng());
        let share = PartialSsaShare::<TestSpec>(scalar.to_repr());
        let debug = format!("{:?}", share);
        assert!(debug.contains("PartialSsaShare"));
        // The scalar repr should not appear in Debug output
        assert_eq!(
            debug, "PartialSsaShare { .. }",
            "PartialSsaShare Debug must exactly match the redacted format"
        );
    }

    #[test]
    fn debug_redaction_generated_share() {
        use crate::tests::TestSpec;

        let scalar = PixScalar::<TestSpec>::random(&mut hopr_types::crypto_random::rng());
        let share = PartialSsaShare::<TestSpec>(scalar.to_repr());
        let id = SsaPolynomialId::new(
            SsaId::new(
                SimplePseudonym::try_from([0u8; 10].as_ref()).unwrap(),
                1.try_into().unwrap(),
            ),
            0,
        );
        let generated = GeneratedShare { id, share };
        let debug = format!("{:?}", generated);

        // Must include the public ID field
        assert!(debug.contains("GeneratedShare"));
        assert!(debug.contains("id"));

        // Must NOT include the secret share field — only the public id
        assert_eq!(
            debug,
            format!("GeneratedShare {{ id: {:?}, .. }}", id),
            "GeneratedShare Debug must expose only the id field"
        );
    }

    #[test]
    fn debug_redaction_recovered_ssa() {
        use hopr_types::crypto::{keypairs::Keypair, prelude::ChainKeypair};

        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into().unwrap());
        let dummy_key = ChainKeypair::random();
        let recovered = RecoveredSsa { ssa_id, ssa: dummy_key };
        let debug = format!("{:?}", recovered);

        // Must include the public ssa_id field
        assert!(debug.contains("RecoveredSsa"));
        assert!(debug.contains("ssa_id"));

        // Must NOT include the secret ssa field — only the public ssa_id
        assert_eq!(
            debug,
            format!("RecoveredSsa {{ ssa_id: {:?}, .. }}", ssa_id),
            "RecoveredSsa Debug must expose only the ssa_id field"
        );
    }
}
