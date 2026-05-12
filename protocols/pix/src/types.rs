use std::{cmp::Ordering, ops::Add};

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
        typenum::{Sum, U4, Unsigned},
    },
};

use crate::{PixScalar, PixSpec, errors};

/// Type used to index Session Stealth Addresses (SSA).
///
/// Note that SSA Index starts with 1.
pub type SsaIndex = u16; // TODO: change this to NonZeroU16

/// Type used to index polynomials that reconstruct parts of a Session Stealth Addresses (SSA).
///
/// The index is 0-based.
pub type PolynomialIndex = u16;

/// Type used to index coefficients in a polynomial.
///
/// The index is 0-based.
pub type CoefficientIndex = u16;

/// Size of the [`SsaIndex`] and [`PolynomialIndex`] prefix prepended to the encrypted share.
pub type SsaPolyIndexPrefixSize = U4;

fn derive_ssa_encryption_key<S: PixSpec>(spi: &SsaPolynomialId<S>, ack: &HalfKey) -> errors::Result<S::Cipher> {
    let mut output = Blake3::new_derive_key(S::KEY_DERIVATION_CONTEXT)
        .update_reader(ack.as_ref())
        .and_then(|h| h.update_reader(spi.id.pseudonym.as_ref()))
        .and_then(|h| h.update_reader(spi.id.ssa_index.to_be_bytes().as_ref()))
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
/// 1. [`SsaIndex`] (big-endian, 2 bytes)
/// 2. [`PolynomialIndex`] (big-endian, 2 bytes)
/// 3. The encrypted scalar share ([`FieldBytesSize`] bytes)
///
/// This share can be [decrypted](EncryptedPartialSsaShare::decrypt) to [`PartialSsaShare`]
/// to be verified and used for reconstruction.
#[derive(Debug)]
pub struct EncryptedPartialSsaShare<S: PixSpec>(GenericArray<u8, EncShareSize<S>>)
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>;

impl<S: PixSpec> EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    /// [`SsaIndex`] embedded in this encrypted share.
    pub fn ssa_index(&self) -> SsaIndex {
        SsaIndex::from_be_bytes([self.0[0], self.0[1]])
    }

    /// [`PolynomialIndex`] embedded in this encrypted share.
    pub fn poly_index(&self) -> PolynomialIndex {
        PolynomialIndex::from_be_bytes([self.0[2], self.0[3]])
    }

    pub fn decrypt(self, pseudonym: &S::Pseudonym, key: &HalfKey) -> errors::Result<PartialSsaShare<S>> {
        let spi = SsaPolynomialId::new(SsaId::new(*pseudonym, self.ssa_index()), self.poly_index());
        let mut cipher = derive_ssa_encryption_key::<S>(&spi, key)?;
        let mut share = <PixScalar<S> as PrimeField>::Repr::default();
        share.as_mut().copy_from_slice(&self.0[SsaPolyIndexPrefixSize::USIZE..]);
        cipher.apply_keystream(share.as_mut());
        Ok(PartialSsaShare(share))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

impl<S: PixSpec> Clone for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: PixSpec> Copy for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
    <EncShareSize<S> as ArrayLength<u8>>::ArrayType: Copy,
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

impl<S: PixSpec> PartialEq for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S: PixSpec> Eq for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
}

impl<S: PixSpec> Default for EncryptedPartialSsaShare<S> {
    fn default() -> Self {
        Self(GenericArray::default())
    }
}

#[cfg(feature = "serde")]
impl<S: PixSpec> serde::Serialize for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de, S: PixSpec> serde::Deserialize<'de> for EncryptedPartialSsaShare<S>
where
    FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
    EncShareSize<S>: ArrayLength<u8>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EncryptedPartialSsaShareVisitor<S: PixSpec>(std::marker::PhantomData<S>);

        impl<'de, S: PixSpec> serde::de::Visitor<'de> for EncryptedPartialSsaShareVisitor<S>
        where
            FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
            EncShareSize<S>: ArrayLength<u8>,
        {
            type Value = EncryptedPartialSsaShare<S>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a byte array with the encrypted partial SSA share")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let expected = EncShareSize::<S>::USIZE;
                if v.len() != expected {
                    return Err(E::invalid_length(v.len(), &self));
                }

                let mut bytes = GenericArray::<u8, EncShareSize<S>>::default();
                bytes.copy_from_slice(v);
                Ok(EncryptedPartialSsaShare(bytes))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let expected = EncShareSize::<S>::USIZE;
                let mut bytes = GenericArray::<u8, EncShareSize<S>>::default();

                for i in 0..expected {
                    bytes[i] = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }

                if seq.next_element::<u8>()?.is_some() {
                    return Err(serde::de::Error::invalid_length(expected + 1, &self));
                }

                Ok(EncryptedPartialSsaShare(bytes))
            }
        }

        deserializer.deserialize_bytes(EncryptedPartialSsaShareVisitor::<S>(std::marker::PhantomData))
    }
}

/// Share of a polynomial used to reconstruct a portion of the Session Stealth Address (SSA).
///
/// This corresponds to the `P_ij(X)` of the polynomial used to reconstruct the j-th portion of i-th SSA
/// at some value `X` (of type [`PixSpec::ShareId`]).
///
/// The `X` value is not held by the struct, and it's the responsibility of the user to determine its correct value.
#[derive(Default)]
pub struct PartialSsaShare<S: PixSpec>(pub(crate) <PixScalar<S> as PrimeField>::Repr);

impl<S: PixSpec> PartialSsaShare<S> {
    pub fn encrypt(mut self, spi: &SsaPolynomialId<S>, key: &HalfKey) -> errors::Result<EncryptedPartialSsaShare<S>>
    where
        FieldBytesSize<S>: Add<SsaPolyIndexPrefixSize>,
        EncShareSize<S>: ArrayLength<u8>,
    {
        let mut cipher = derive_ssa_encryption_key::<S>(spi, key)?;
        cipher.apply_keystream(self.0.as_mut());

        let mut out = GenericArray::<u8, EncShareSize<S>>::default();
        out[0..size_of::<SsaIndex>()].copy_from_slice(&spi.ssa_index().to_be_bytes());
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
impl<S: PixSpec> Clone for PartialSsaShare<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: PixSpec> std::fmt::Debug for PartialSsaShare<S>
where
    <PixScalar<S> as PrimeField>::Repr: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PartialSsaShare").field(&self.0).finish()
    }
}

impl<S: PixSpec> PartialEq for PartialSsaShare<S>
where
    <PixScalar<S> as PrimeField>::Repr: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S: PixSpec> Eq for PartialSsaShare<S> where <PixScalar<S> as PrimeField>::Repr: Eq {}

/// Uniquely identifies a polynomial that allows forming a Session Stealth Address (SSA) corresponding
/// to a specific Session.
///
/// The index consists of the following parts:
/// 1. The Pseudonym part of the `HoprSenderId` - fixed for the given Session.
/// 2. Index (i) of the Session Stealth Address (SSA)
/// 3. Index (j) of the polynomial used to reconstruct the portion of the SSA.
pub struct SsaPolynomialId<S: PixSpec> {
    id: SsaId<S>,
    poly_index: PolynomialIndex,
}

impl<S: PixSpec> SsaPolynomialId<S> {
    pub fn new(id: SsaId<S>, poly_index: PolynomialIndex) -> Self {
        Self { id, poly_index }
    }

    /// Pseudonym part of the `HoprSenderId`.
    #[inline]
    pub fn pseudonym(&self) -> &S::Pseudonym {
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

impl<S: PixSpec> AsRef<SsaId<S>> for SsaPolynomialId<S> {
    fn as_ref(&self) -> &SsaId<S> {
        &self.id
    }
}

impl<S: PixSpec> std::fmt::Display for SsaPolynomialId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.poly_index)
    }
}

// Manual trait implementations are required because they are not dependent on the generic S

impl<S: PixSpec> std::fmt::Debug for SsaPolynomialId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SsaPolynomialIndex")
            .field("id", &self.id)
            .field("poly_index", &self.poly_index)
            .finish()
    }
}

impl<S: PixSpec> Clone for SsaPolynomialId<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: PixSpec> Copy for SsaPolynomialId<S> {}

impl<S: PixSpec> PartialEq for SsaPolynomialId<S> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.poly_index == other.poly_index
    }
}

impl<S: PixSpec> Eq for SsaPolynomialId<S> {}

impl<S: PixSpec> std::hash::Hash for SsaPolynomialId<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.poly_index.hash(state);
    }
}

impl<S> PartialOrd for SsaPolynomialId<S>
where
    S: PixSpec,
    S::Pseudonym: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> Ord for SsaPolynomialId<S>
where
    S: PixSpec,
    S::Pseudonym: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.poly_index.cmp(&other.poly_index))
    }
}

/// Uniquely identifies a Session Stealth Address (SSA).
///
/// This consists of a pseudonym and [`SsaIndex`].
pub struct SsaId<S: PixSpec> {
    pseudonym: S::Pseudonym,
    ssa_index: SsaIndex,
}

impl<S: PixSpec> SsaId<S> {
    pub fn new(pseudonym: S::Pseudonym, ssa_index: SsaIndex) -> Self {
        Self { pseudonym, ssa_index }
    }

    #[inline]
    pub fn pseudonym(&self) -> &S::Pseudonym {
        &self.pseudonym
    }

    #[inline]
    pub fn ssa_index(&self) -> SsaIndex {
        self.ssa_index
    }
}

// Manual trait implementations are required because they are not dependent on the generic S

impl<S: PixSpec> Clone for SsaId<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: PixSpec> Copy for SsaId<S> {}

impl<S: PixSpec> PartialEq for SsaId<S> {
    fn eq(&self, other: &Self) -> bool {
        self.pseudonym == other.pseudonym && self.ssa_index == other.ssa_index
    }
}

impl<S: PixSpec> Eq for SsaId<S> {}

impl<S: PixSpec> std::hash::Hash for SsaId<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pseudonym.hash(state);
        self.ssa_index.hash(state);
    }
}

impl<S: PixSpec> std::fmt::Display for SsaId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-ssa#{}", self.pseudonym, self.ssa_index)
    }
}

impl<S: PixSpec> std::fmt::Debug for SsaId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SsaId")
            .field("pseudonym", &self.pseudonym.to_string())
            .field("ssa_index", &self.ssa_index)
            .finish()
    }
}

impl<S: PixSpec> PartialOrd<Self> for SsaId<S>
where
    S::Pseudonym: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: PixSpec> Ord for SsaId<S>
where
    S::Pseudonym: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.pseudonym
            .cmp(&other.pseudonym)
            .then_with(|| self.ssa_index.cmp(&other.ssa_index))
    }
}

#[cfg(test)]
mod tests {
    use hopr_types::{crypto::types::SimplePseudonym, crypto_random::Randomizable};
    use vsss_rs::elliptic_curve::Field;

    use super::*;
    use crate::tests::TestSpec;

    #[test]
    fn size_of_indices_must_match() {
        let actual = size_of::<SsaIndex>() + size_of::<PolynomialIndex>();
        let expected = SsaPolyIndexPrefixSize::USIZE;
        assert_eq!(actual, expected);
    }

    #[test]
    fn default_enc_share_is_empty() {
        assert!(EncryptedPartialSsaShare::<TestSpec>::default().is_empty());
    }

    #[test]
    fn ssa_part_shares_should_encrypt_and_decrypt() -> anyhow::Result<()> {
        let key = HalfKey::random();
        let spi = SsaPolynomialId::<TestSpec>::new(SsaId::new(SimplePseudonym::random(), 1), 0);
        let scalar = PixScalar::<TestSpec>::random(vsss_rs::elliptic_curve::rand_core::OsRng);

        let share_1 = PartialSsaShare::<TestSpec>(scalar.to_repr());
        let share_2 = share_1.clone().encrypt(&spi, &key)?.decrypt(spi.pseudonym(), &key)?;

        assert_eq!(share_1, share_2);
        Ok(())
    }
}
