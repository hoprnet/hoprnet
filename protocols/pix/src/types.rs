use std::{num::NonZero, ops::Add};

use hopr_types::{
    crypto::{
        crypto_traits::{self, KeyIvInit, StreamCipher},
        prelude::HalfKey,
        primitives::Blake3,
    },
    primitive::prelude::{BytesRepresentable, GeneralError},
};
use vsss_rs::elliptic_curve::{
    Curve, PrimeField,
    generic_array::{
        ArrayLength, GenericArray,
        typenum::{Sum, U6, Unsigned},
    },
};

use crate::{PartialSsaShareVerifier, PixGroup, PixScalar, PixSpec, errors, errors::PixError};

/// Raw zeroable SSA Index.
pub(crate) type RawSsaIndex = u32;

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

/// Size of the [`SsaIndex`] and [`PolynomialIndex`] prefix prepended to the encrypted share.
pub type SsaPolyIndexPrefixSize = U6;

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
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct PartialSsaShare<S: PixSpec>(pub(crate) <PixScalar<S> as PrimeField>::Repr);

impl<S: PixSpec> PartialSsaShare<S> {
    /// Encrypts this partial SSA share using the given acknowledgement [`HalfKey`].
    pub fn encrypt(
        mut self,
        spi: &SsaPolynomialId<S::Pseudonym>,
        ack_key: &HalfKey,
    ) -> errors::Result<EncryptedPartialSsaShare<S>>
    where
        FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
        EncShareSize<S>: ArrayLength<u8>,
    {
        let mut cipher = derive_ssa_encryption_key::<S>(spi, ack_key)?;
        cipher.apply_keystream(self.0.as_mut());

        let mut out = GenericArray::<u8, EncShareSize<S>>::default();
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
) -> errors::Result<S::Cipher> {
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
/// [`SsaPolyIndexPrefixSize`] + [`FieldBytesSize`].
pub type EncShareSize<S> = Sum<FieldBytesSize<S>, SsaPolyIndexPrefixSize>;

/// Contains an encrypted partial Session Stealth Address (SSA) share.
///
/// The internal byte layout is:
/// 1. [`SsaIndex`] (big-endian, 4 bytes)
/// 2. [`PolynomialIndex`] (big-endian, 2 bytes)
/// 3. The encrypted scalar share ([`FieldBytesSize`] bytes)
///
/// This share can be [decrypted](EncryptedPartialSsaShare::decrypt) to [`PartialSsaShare`]
/// to be verified and used for reconstruction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct EncryptedPartialSsaShare<S: PixSpec>(GenericArray<u8, EncShareSize<S>>)
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>;

impl<S: PixSpec> EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
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
    pub(crate) fn decrypt(self, pseudonym: &S::Pseudonym, ack_key: &HalfKey) -> errors::Result<PartialSsaShare<S>> {
        if let Some((ssa_index, poly_index)) = self.indices() {
            let spi = SsaPolynomialId::new(SsaId::new(*pseudonym, ssa_index), poly_index);
            let mut cipher = derive_ssa_encryption_key::<S>(&spi, ack_key)?;
            let mut share = <PixScalar<S> as PrimeField>::Repr::default();
            share.as_mut().copy_from_slice(&self.0[SsaPolyIndexPrefixSize::USIZE..]);
            cipher.apply_keystream(share.as_mut());
            Ok(PartialSsaShare(share))
        } else {
            Err(PixError::ShareIsEmpty)
        }
    }

    /// Returns true if the encrypted share is empty (all zeroes or zero SSA index).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == GenericArray::<u8, EncShareSize<S>>::default() || self.indices().is_none()
    }
}

impl<S: PixSpec + Copy> Copy for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
    GenericArray<u8, EncShareSize<S>>: Copy,
{
}

impl<S: PixSpec> AsRef<[u8]> for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a, S: PixSpec> TryFrom<&'a [u8]> for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() != Self::SIZE {
            return Err(GeneralError::ParseError("EncryptedPartialSsaShare.size".into()));
        }
        Ok(Self(GenericArray::clone_from_slice(value)))
    }
}

impl<S: PixSpec> BytesRepresentable for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    const SIZE: usize = EncShareSize::<S>::USIZE;
}

/// This is a wrapped [`EncryptedPartialSsaShare`] extracted from a specific SURB (presumably along with the associated
/// `AcknowledgementChallenge`).
#[derive(Clone, Debug, PartialEq, Eq)]
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
    ) -> errors::Result<Self> {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedShare<S: PixSpec, P = <S as PixSpec>::Pseudonym> {
    /// ID of the polynomial corresponding to the partial SSA share.
    pub id: SsaPolynomialId<P>,
    /// Generated partial SSA share.
    pub share: PartialSsaShare<S>,
}

impl<S: PixSpec> GeneratedShare<S, S::Pseudonym> {
    /// Convenience method to [encrypt](PartialSsaShare::encrypt) the share using an acknowledgement [`HalfKey`].
    #[inline]
    pub fn encrypt(self, ack: &HalfKey) -> errors::Result<EncryptedPartialSsaShare<S>> {
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
    pub verifiers: Vec<PartialSsaShareVerifier<S, P>>,
}

/// Represents the current state of a specific SSA commitment on an Exit node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaCommitmentState<S: PixSpec, P = <S as PixSpec>::Pseudonym> {
    /// ID of the SSA that is being committed to.
    pub ssa_id: SsaId<P>,
    /// Commitment to the SSA, if it's already known.
    #[cfg_attr(feature = "serde", serde(with = "option_group"))]
    pub ssa_commitment: Option<PixGroup<S>>,
    /// Whether the commitment is fully committed and therefore its partial shares are verifiable.
    pub is_verifiable: bool,
    /// Whether this SSA was encountered for the first time.
    pub is_first_encountered: bool,
}

impl<S: PixSpec> SsaCommitmentState<S, S::Pseudonym> {
    /// Creates a new SsaCommitmentState for the given SSA ID.
    ///
    /// It has no associated commitment and is not verifiable initially.
    pub fn new(ssa_id: SsaId<S::Pseudonym>) -> Self {
        Self {
            ssa_id,
            ssa_commitment: None,
            is_verifiable: false,
            is_first_encountered: true,
        }
    }
}

/// Contains the already recovered secret scalar corresponding to a specific SSA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveredSsa<S: PixSpec, P = <S as PixSpec>::Pseudonym> {
    /// ID of the SSA that was recovered.
    pub ssa_id: SsaId<P>,
    /// Recovered secret scalar (private key corresponding to the SSA).
    pub ssa: PixScalar<S>,
}

/// Serde adapter for `Option<G>` where `G` is a curve group element, delegating
/// to `elliptic_curve_tools::group` for the inner element.
///
/// Workaround for `serde_with` not supporting `Option<elliptic_curve_tools::group>`
/// (the latter is a serde-module helper, not a `SerializeAs`/`DeserializeAs` impl).
#[cfg(feature = "serde")]
mod option_group {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use vsss_rs::elliptic_curve::{Group, group::GroupEncoding};

    pub fn serialize<G, S>(value: &Option<G>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        G: Group + GroupEncoding,
    {
        struct Wrapper<'a, G: Group + GroupEncoding>(&'a G);

        impl<G: Group + GroupEncoding> Serialize for Wrapper<'_, G> {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                elliptic_curve_tools::group::serialize(self.0, s)
            }
        }

        match value {
            Some(g) => serializer.serialize_some(&Wrapper(g)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, G, D>(deserializer: D) -> Result<Option<G>, D::Error>
    where
        D: Deserializer<'de>,
        G: Group + GroupEncoding,
    {
        #[derive(Deserialize)]
        struct Wrapper<G: Group + GroupEncoding>(#[serde(with = "elliptic_curve_tools::group")] G);

        Ok(Option::<Wrapper<G>>::deserialize(deserializer)?.map(|w| w.0))
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
        let scalar = PixScalar::<TestSpec>::random(vsss_rs::elliptic_curve::rand_core::OsRng);

        let share_1 = PartialSsaShare::<TestSpec>(scalar.to_repr());
        let share_2 = share_1.clone().encrypt(&spi, &key)?.decrypt(spi.pseudonym(), &key)?;

        assert_eq!(share_1, share_2);
        Ok(())
    }
}
