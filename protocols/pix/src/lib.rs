use hopr_types::crypto::prelude::Pseudonym;
pub use hopr_types::crypto::prelude::SimplePseudonym;
use k256::elliptic_curve::{Group, PrimeField, group::GroupEncoding};
use vsss_rs::elliptic_curve::rand_core::{CryptoRng, RngCore};
use vsss_rs::{DefaultShare, IdentifierPrimeField, ShareVerifierGroup};

mod generator;
mod reconstructor;
mod errors;

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
pub type SSAIndex = u32;

/// Share of a polynomial used to reconstruct a portion of the Session Stealth Address (SSA).
///
/// This corresponds to the `P_ij(X)` of the polynomial used to reconstruct the j-th portion of i-th SSA
/// at some value `X`, typically the hash of the corresponding SURB.
#[derive(Clone, Copy, Default)]
pub struct SsaPolyShare<S: PixSpec>(<<S as PixSpec>::Scalar as PrimeField>::Repr);

impl<S> std::fmt::Debug for SsaPolyShare<S>
where S: PixSpec, <<S as PixSpec>::Scalar as PrimeField>::Repr: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SsaPolyShare").field(&self.0).finish()
    }
}

impl<S> PartialEq for SsaPolyShare<S>
where S: PixSpec, <<S as PixSpec>::Scalar as PrimeField>::Repr: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S> Eq for SsaPolyShare<S>
where S: PixSpec, <<S as PixSpec>::Scalar as PrimeField>::Repr: Eq
{ }

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
        Self { pseudonym, ssa_index, poly_index }
    }

    /// Pseudonym part of the `HoprSenderId`.
    #[inline]
    pub fn pseudonym(&self) -> &P {
        &self.pseudonym
    }

    /// Index (i-value) of the Session Stealth Address (SSA).
    #[inline]
    pub fn ssa_index(&self) -> SSAIndex {
        self.ssa_index
    }

    /// Index (j-value) of the polynomial used to reconstruct the portion of the SSA.
    #[inline]
    pub fn poly_index(&self) -> u32 {
        self.poly_index
    }
}