#[cfg(feature = "rayon")]
use hopr_parallelize::cpu::rayon::prelude::*;
use hopr_types::crypto::prelude::Pseudonym;
pub use hopr_types::crypto::prelude::SimplePseudonym;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Share, ShareElement, ShareVerifierGroup, ValueGroup,
    elliptic_curve::{Group, PrimeField, group::GroupEncoding},
};

mod errors;
mod generator;
mod reconstructor;

pub use generator::{SsaGeneratorConfig, SsaShareGenerator};

/// Specification of the Protocol for Incentivization of eXits (PIX).
pub trait PixSpec {
    /// Scalar type used in the protocol (for polynomial coefficients)
    type Scalar: PrimeField;
    /// Element of a large prime order group used for commitments.
    type Element: Group<Scalar = Self::Scalar> + GroupEncoding + Default;
    /// Pseudonym used to identify groups of SURBs.
    type Pseudonym: Pseudonym + Copy + Send + Sync + 'static;
}

/// Type used to index Session Stealth Addresses (SSA).
///
/// Note that SSA Index starts with 1.
pub type SsaIndex = u32;

/// Share of a polynomial used to reconstruct a portion of the Session Stealth Address (SSA).
///
/// This corresponds to the `P_ij(X)` of the polynomial used to reconstruct the j-th portion of i-th SSA
/// at some value `X`, typically the hash of the corresponding SURB.
#[derive(Clone, Copy, Default)]
pub struct SsaPolyShare<S: PixSpec>(<<S as PixSpec>::Scalar as PrimeField>::Repr);

impl<S> std::fmt::Debug for SsaPolyShare<S>
where
    S: PixSpec,
    <<S as PixSpec>::Scalar as PrimeField>::Repr: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SsaPolyShare").field(&self.0).finish()
    }
}

impl<S> PartialEq for SsaPolyShare<S>
where
    S: PixSpec,
    <<S as PixSpec>::Scalar as PrimeField>::Repr: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S> Eq for SsaPolyShare<S>
where
    S: PixSpec,
    <<S as PixSpec>::Scalar as PrimeField>::Repr: Eq,
{
}

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

/// Verifier for shares of a polynomial with the given [`SurbPolynomialIndex`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaShareVerifier<S: PixSpec> {
    pub(crate) spi: SurbPolynomialIndex<S::Pseudonym>,
    pub(crate) poly_commitment: Vec<ShareVerifierGroup<S::Element>>,
}

impl<S: PixSpec> SsaShareVerifier<S> {
    pub fn spi(&self) -> &SurbPolynomialIndex<S::Pseudonym> {
        &self.spi
    }

    pub fn verify(&self, share: &SsaPolyShare<S>, x: S::Scalar) -> errors::Result<()> {
        let share: DefaultShare<IdentifierPrimeField<S::Scalar>, IdentifierPrimeField<S::Scalar>> = DefaultShare {
            identifier: x.into(),
            value: Option::from(S::Scalar::from_repr(share.0))
                .map(|s: S::Scalar| s.into())
                .ok_or(vsss_rs::Error::InvalidShare)?,
        };

        if (share.value().is_zero() | share.identifier().is_zero()).into() {
            return Err(vsss_rs::Error::InvalidShare.into());
        }
        if self.poly_commitment[0].is_zero().into() {
            return Err(vsss_rs::Error::InvalidGenerator("generator is identity").into());
        }

        let mut i = IdentifierPrimeField::<S::Scalar>::one();
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

        // v[1] + v[2]*x + v[3]*x^2 + ... + v[t]*x^t
        let rhs = self.poly_commitment[1].0
            + scalars_iter
                .enumerate()
                .map(|(i, c)| (self.poly_commitment[i + 2] * c).0)
                .sum::<S::Element>();

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
