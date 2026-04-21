#[cfg(feature = "rayon")]
use hopr_parallelize::cpu::rayon::prelude::*;
pub use hopr_types::crypto::prelude::SimplePseudonym;
use hopr_types::crypto::prelude::{Pseudonym, Sha3_256};
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Share, ShareElement, ShareVerifierGroup, ValueGroup,
    elliptic_curve::{
        CurveArithmetic, Group, PrimeCurve, PrimeField,
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
{
    /// Prime order elliptic curve use for commitments.
    type Curve: PrimeCurve + CurveArithmetic + GroupDigest;
    /// Pseudonym used to identify groups of SURBs.
    type Pseudonym: Pseudonym + Copy + Send + Sync + 'static;
}

pub type Scalar<S> = <<S as PixSpec>::Curve as CurveArithmetic>::Scalar;
pub type Element<S> = <<S as PixSpec>::Curve as CurveArithmetic>::ProjectivePoint;

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

/// Defines the index of a polynomial in a single use reply block (SURB).
///
/// The index consists of the following parts:
/// 1. The Pseudonym part of the `HoprSenderId`
/// 2. Index (i) of the Session Stealth Address (SSA)
/// 3. Index (j) of the polynomial used to reconstruct the portion of the SSA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct SurbPolynomialIndex<P = SimplePseudonym> {
    pseudonym: P,
    ssa_index: u32,
    poly_index: u32,
}

impl<P> SurbPolynomialIndex<P> {
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

pub(crate) fn msg_to_scalar<S: PixSpec>(pseudonym: &S::Pseudonym, msg: impl AsRef<[u8]>) -> errors::Result<Scalar<S>> {
    Ok(<S::Curve as GroupDigest>::hash_to_scalar::<ExpandMsgXmd<Sha3_256>>(
        &[msg.as_ref()],
        &[
            format!("{:?}_XMD:SHA3-256_SSWU_RO_", S::Curve::default()).as_bytes(),
            pseudonym.as_ref(),
        ],
    )?)
}

/// Verifier for shares of a polynomial with the given [`SurbPolynomialIndex`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaShareVerifier<S: PixSpec> {
    pub(crate) spi: SurbPolynomialIndex<S::Pseudonym>,
    pub(crate) poly_commitment: Vec<ShareVerifierGroup<Element<S>>>,
}

impl<S: PixSpec> SsaShareVerifier<S> {
    /// Returns the [`SurbPolynomialIndex`] of the polynomial corresponding to this verifier.
    #[inline]
    pub fn spi(&self) -> &SurbPolynomialIndex<S::Pseudonym> {
        &self.spi
    }

    /// Verifies that the given `share` corresponding to `msg` belongs to the polynomial associated with this verifier.
    pub fn verify(&self, share: &PartialSsaShare<S>, msg: impl AsRef<[u8]>) -> errors::Result<()> {
        let share: DefaultShare<IdentifierPrimeField<Scalar<S>>, IdentifierPrimeField<Scalar<S>>> = DefaultShare {
            identifier: msg_to_scalar::<S>(self.spi.pseudonym(), msg)?.into(),
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
    use hopr_types::crypto_random::Randomizable;
    use vsss_rs::{
        ParticipantIdGeneratorType,
        elliptic_curve::rand_core::{CryptoRng, RngCore},
        feldman,
    };

    use super::*;

    pub struct TestSpec;

    impl PixSpec for TestSpec {
        type Curve = k256::Secp256k1;
        type Pseudonym = SimplePseudonym;
    }

    fn standard_shamir_generate<S: PixSpec>(
        secret: &IdentifierPrimeField<>,
        t: usize,
        x: &[S],
        mut rng: impl RngCore + CryptoRng,
    ) -> anyhow::Result<(
        Vec<DefaultShare<IdentifierPrimeField<S>, IdentifierPrimeField<S>>>,
        Vec<ShareVerifierGroup<P>>,
    )>
    where
        S: PrimeField,
        P: Default + GroupEncoding + Group<Scalar = S>,
    {
        anyhow::ensure!(t > 0, "t must be greater than 0");
        anyhow::ensure!(x.len() >= t, "x must have at least t elements");

        let (shares, verifier_set) = feldman::split_secret_with_participant_generator::<
            DefaultShare<IdentifierPrimeField<S>, IdentifierPrimeField<S>>,
            ShareVerifierGroup<P>,
        >(
            t,
            x.len(),
            &secret,
            None,
            &mut rng,
            &[ParticipantIdGeneratorType::list(
                &x.into_iter().map(|x| (*x).into()).collect::<Vec<_>>(),
            )],
        )
        .map_err(anyhow::Error::msg)?;

        Ok((shares, verifier_set))
    }

    #[test]
    fn ssa_shared_verifier_must_correspond_to_standard() -> anyhow::Result<()> {
        let mut rng = vsss_rs::elliptic_curve::rand_core::OsRng;
        let secret = IdentifierPrimeField::<k256::Scalar>::random(&mut rng);

        let p = SimplePseudonym::random();
        let x = (0..20_u32)
            .map(|i| msg_to_scalar::<TestSpec>(&p, i.to_be_bytes()).unwrap())
            .collect::<Vec<_>>();

        let (shares, verifier) =
            standard_shamir_generate::<k256::Scalar, k256::ProjectivePoint>(&secret, 10, &x, &mut rng)?;

        let verifier: SsaShareVerifier<TestSpec> = SsaShareVerifier {
            spi: SurbPolynomialIndex::new(p, 1, 1),
            poly_commitment: verifier,
        };

        assert_eq!(shares.len(), x.len());

        for (i, s) in shares.into_iter().enumerate() {
            let share: PartialSsaShare<TestSpec> = PartialSsaShare(s.value.0.to_repr());
            verifier.verify(&share, i.to_be_bytes())?;
        }

        Ok(())
    }
}
