use digest::{FixedOutput, HashMarker, OutputSizeUser, crypto_common::BlockSizeUser};
#[cfg(feature = "rayon")]
use hopr_parallelize::cpu::rayon::prelude::*;
use hopr_types::crypto::prelude::Pseudonym;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Share, ShareElement, ShareVerifierGroup, ValueGroup,
    elliptic_curve::{
        CurveArithmetic, Group, PrimeCurve, PrimeField,
        consts::U256,
        generic_array::typenum::{IsLess, IsLessOrEqual},
        group::{GroupEncoding, cofactor::CofactorGroup},
        hash2curve::{ExpandMsgXmd, FromOkm, GroupDigest},
    },
};

mod errors;
mod generator;
mod reconstructor;

pub use generator::{SsaGeneratorConfig, SsaShareGenerator};

/// Specification of the Protocol for Incentivization of eXits (PIX).
pub trait PixSpec
where
    Scalar<Self>: PrimeField + FromOkm,
    Element<Self>: Group<Scalar = Scalar<Self>> + GroupEncoding + Default + CofactorGroup,
    <Digest<Self> as OutputSizeUser>::OutputSize: IsLess<U256>,
    <Digest<Self> as OutputSizeUser>::OutputSize: IsLessOrEqual<<Digest<Self> as BlockSizeUser>::BlockSize>,
{
    /// Prime order elliptic curve use for commitments.
    type Curve: PrimeCurve + CurveArithmetic + GroupDigest;
    /// Digest used for hashing operations.
    type Digest: BlockSizeUser + FixedOutput + std::fmt::Debug + Default + HashMarker;
    /// Pseudonym used to identify groups of SURBs.
    type Pseudonym: Pseudonym + Copy + Send + Sync + 'static;
}

pub type Scalar<S> = <<S as PixSpec>::Curve as CurveArithmetic>::Scalar;
pub type Element<S> = <<S as PixSpec>::Curve as CurveArithmetic>::ProjectivePoint;

pub type Digest<S> = <S as PixSpec>::Digest;

/// Type used to index Session Stealth Addresses (SSA).
///
/// Note that SSA Index starts with 1.
pub type SsaIndex = u32;

/// Share of a polynomial used to reconstruct a portion of the Session Stealth Address (SSA).
///
/// This corresponds to the `P_ij(X)` of the polynomial used to reconstruct the j-th portion of i-th SSA
/// at some value `X` (of type [`PixSpec::ShareId`]).
///
/// The `X` value is not held by the struct, and it's the responsibility of the user to determine its correct value.
#[derive(Clone, Default)]
pub struct PartialSsaShare<S: PixSpec>(<Scalar<S> as PrimeField>::Repr);

impl<S: PixSpec> std::fmt::Debug for PartialSsaShare<S>
where
    <Scalar<S> as PrimeField>::Repr: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PartialSsaShare").field(&self.0).finish()
    }
}

impl<S: PixSpec> PartialEq for PartialSsaShare<S>
where
    <Scalar<S> as PrimeField>::Repr: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S: PixSpec> Eq for PartialSsaShare<S> where <Scalar<S> as PrimeField>::Repr: Eq {}

/// Defines the index of a polynomial in a Session Stealth Address (SSA) corresponding
/// to a specific Session.
///
/// The index consists of the following parts:
/// 1. The Pseudonym part of the `HoprSenderId` - fixed for the given Session.
/// 2. Index (i) of the Session Stealth Address (SSA)
/// 3. Index (j) of the polynomial used to reconstruct the portion of the SSA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct SsaPolynomialIndex<P> {
    pseudonym: P,
    ssa_index: u32,
    poly_index: u32,
}

impl<P> SsaPolynomialIndex<P> {
    pub fn new(pseudonym: P, ssa_index: u32, poly_index: u32) -> Self {
        Self {
            pseudonym,
            ssa_index,
            poly_index,
        }
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

    /// Index (j-value) of the polynomial used to reconstruct the portion of the SSA.
    #[inline]
    pub fn poly_index(&self) -> u32 {
        self.poly_index
    }
}

pub(crate) fn msg_to_scalar<S: PixSpec>(
    spi: &SsaPolynomialIndex<S::Pseudonym>,
    msg: impl AsRef<[u8]>,
) -> errors::Result<Scalar<S>> {
    Ok(<S::Curve as GroupDigest>::hash_to_scalar::<ExpandMsgXmd<S::Digest>>(
        &[
            msg.as_ref(),
            spi.pseudonym().as_ref(),
            spi.ssa_index().to_be_bytes().as_ref(),
            spi.poly_index().to_be_bytes().as_ref(),
        ],
        &[format!("{:?}_XMD:{:?}_SSWU_RO_", S::Curve::default(), S::Digest::default()).as_bytes()],
    )?)
}

/// Verifier for shares of a polynomial with the given [`SsaPolynomialIndex`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaShareVerifier<S: PixSpec> {
    pub(crate) spi: SsaPolynomialIndex<S::Pseudonym>,
    pub(crate) poly_commitment: Vec<ShareVerifierGroup<Element<S>>>,
}

impl<S: PixSpec> SsaShareVerifier<S> {
    /// Returns the [`SsaPolynomialIndex`] of the polynomial corresponding to this verifier.
    #[inline]
    pub fn spi(&self) -> &SsaPolynomialIndex<S::Pseudonym> {
        &self.spi
    }

    /// Verifies that the given `share` corresponding to `msg` belongs to the polynomial associated with this verifier.
    pub fn verify(&self, share: &PartialSsaShare<S>, msg: impl AsRef<[u8]>) -> errors::Result<()> {
        let share: DefaultShare<IdentifierPrimeField<Scalar<S>>, IdentifierPrimeField<Scalar<S>>> = DefaultShare {
            identifier: msg_to_scalar::<S>(&self.spi, msg)?.into(),
            value: Option::from(Scalar::<S>::from_repr(share.0.clone()))
                .map(|s: Scalar<S>| s.into())
                .ok_or(vsss_rs::Error::InvalidShare)?,
        };

        if (share.value().is_zero() | share.identifier().is_zero()).into() {
            return Err(vsss_rs::Error::InvalidShare.into());
        }
        if self.poly_commitment[0].is_zero().into() {
            return Err(vsss_rs::Error::InvalidGenerator("generator is identity").into());
        }

        let mut i = IdentifierPrimeField::<Scalar<S>>::one();
        let mut scalars = Vec::with_capacity(self.poly_commitment.len() - 2);

        // The below multi-scalar multiplication method (MSM) is more efficient
        // for large polynomial degrees than Horner's method because it can be parallelized.

        // Computes x^1, x^2, x^3, ... x^t
        for _ in 0..self.poly_commitment.len() - 2 {
            *i.as_mut() *= share.identifier().as_ref();
            scalars.push(i);
        }

        #[cfg(feature = "rayon")]
        let scalars_iter = scalars.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let scalars_iter = scalars.into_iter();

        // v[1] + v[2]*x + v[3]*x^2 + ... + v[t]*x^t
        let rhs = self.poly_commitment[1].0
            + scalars_iter
                .enumerate()
                .map(|(i, c)| (self.poly_commitment[i + 2] * c).0)
                .sum::<Element<S>>();

        let rhs = ValueGroup::from(rhs);
        let lhs = self.poly_commitment[0] * share.value();

        let res = rhs - lhs;

        if res.is_zero().into() {
            Ok(())
        } else {
            Err(vsss_rs::Error::InvalidShare.into())
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use anyhow::Context;
    use hopr_types::crypto::prelude::SimplePseudonym;
    use vsss_rs::{
        ParticipantIdGeneratorType,
        elliptic_curve::{
            Field,
            rand_core::{CryptoRng, RngCore},
        },
        feldman,
    };

    use super::*;

    pub struct TestSpec;

    impl PixSpec for TestSpec {
        type Curve = k256::Secp256k1;
        type Digest = sha3::Sha3_256;
        type Pseudonym = SimplePseudonym;
    }

    type Share<S> = DefaultShare<IdentifierPrimeField<Scalar<S>>, IdentifierPrimeField<Scalar<S>>>;

    fn standard_shamir_generate<S: PixSpec>(
        secret: Scalar<S>,
        t: usize,
        x: &[Scalar<S>],
        mut rng: impl RngCore + CryptoRng,
    ) -> anyhow::Result<(Vec<Share<S>>, Vec<ShareVerifierGroup<Element<S>>>)> {
        anyhow::ensure!(t > 0, "t must be greater than 0");
        anyhow::ensure!(x.len() >= t, "x must have at least t elements");

        let (shares, verifier_set) =
            feldman::split_secret_with_participant_generator::<Share<S>, ShareVerifierGroup<Element<S>>>(
                t,
                x.len(),
                &secret.into(),
                None,
                &mut rng,
                &[ParticipantIdGeneratorType::list(
                    &x.iter().map(|x| (*x).into()).collect::<Vec<_>>(),
                )],
            )
            .map_err(anyhow::Error::msg)?;

        Ok((shares, verifier_set))
    }

    #[test]
    fn ssa_shared_verifier_must_correspond_to_standard() -> anyhow::Result<()> {
        let mut rng = vsss_rs::elliptic_curve::rand_core::OsRng;
        let secret = k256::Scalar::random(&mut rng);

        let spi = SsaPolynomialIndex::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1, 1);
        let x = (0..=20_u32)
            .map(|i| msg_to_scalar::<TestSpec>(&spi, i.to_be_bytes()).unwrap())
            .collect::<Vec<_>>();

        let (shares, verifier) = standard_shamir_generate::<TestSpec>(secret, 10, &x, &mut rng)?;

        let verifier: SsaShareVerifier<TestSpec> = SsaShareVerifier {
            spi,
            poly_commitment: verifier,
        };

        assert_eq!(shares.len(), x.len());

        for (i, s) in shares.into_iter().enumerate() {
            let share: PartialSsaShare<TestSpec> = PartialSsaShare(s.value.0.to_repr());
            verifier
                .verify(&share, (i as u32).to_be_bytes())
                .context(format!("Verification failed for share index {i}"))?;
        }

        Ok(())
    }
}
