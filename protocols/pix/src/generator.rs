use std::collections::VecDeque;
use k256::elliptic_curve::{PrimeField};
use vsss_rs::{DefaultShare, IdentifierPrimeField, Polynomial, ShareElement, ShareVerifierGroup};

use crate::{PixSpec, SsaPolyShare, SurbPolynomialIndex, errors};

struct SecretPolynomial<S: PixSpec> {
    spi: SurbPolynomialIndex<S::Pseudonym>,
    coeffs: Vec<DefaultShare<IdentifierPrimeField<S::Scalar>, IdentifierPrimeField<S::Scalar>>>,
    shares_generated: usize,
    t: usize,
}

impl<S: PixSpec> SecretPolynomial<S> {

    pub fn next_share(&mut self, x: S::Scalar) -> SsaPolyShare<S> {
        let eval = self.coeffs.evaluate(&x.into(), self.t);
        self.shares_generated += 1;
        SsaPolyShare(eval.0.to_repr())
    }

    pub fn shares_generated(&self) -> usize {
        self.shares_generated
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaShareVerifier<S: PixSpec> {
    poly_commitment: Vec<ShareVerifierGroup<S::Element>>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaGeneratorConfig {
    #[default(1024)]
    pub polynomials_per_ssa: usize,
    #[default(200)]
    pub threshold: usize,
    #[default(20)]
    pub surplus_shares: usize,
}

/// Generator for Session Stealth Address (SSA) shares distributed over Single Use Reply Blocks (SURBs).
pub struct SurbSsaGenerator<S: PixSpec> {
    polynomials: moka::sync::Cache<S::Pseudonym, std::sync::Arc<parking_lot::Mutex<VecDeque<SecretPolynomial<S>>>>>,
    cfg: SsaGeneratorConfig,
}

impl<S: PixSpec + 'static> SurbSsaGenerator<S> {
    /// Generate the next [`SsaPolyShare`] for the given pseudonym.
    ///
    /// Returns `None` if all polynomials for the given pseudonym have been used up.
    pub fn next_share(&self, pseudonym: &S::Pseudonym, x: S::Scalar) -> Option<(SurbPolynomialIndex<S::Pseudonym>, SsaPolyShare<S>)> {
        self.polynomials
            .get(pseudonym)
            .and_then(|polys| {
                let mut polys = polys.lock();
                while !polys.is_empty() {
                    if let Some(poly) = polys.front_mut() {
                        if poly.shares_generated < self.cfg.threshold + self.cfg.surplus_shares {
                            return Some((poly.spi, poly.next_share(x)))
                        }
                    }
                    polys.pop_front();
                }
                None
            })
    }

    /// Generates a new SSA commitment from the sender side, for the given `pseudonym`.
    ///
    /// Returns the new random SSA-commitment and the corresponding SSA share verifier.
    pub fn commit_ssa_part(&self, pseudonym: &S::Pseudonym) -> errors::Result<(S::Element, SsaShareVerifier<S>)> {
        // The generator of the group of verifiers
        let g = ShareVerifierGroup::<S::Element>::one();

        let mut rng = vsss_rs::elliptic_curve::rand_core::OsRng;

        let sub_secrets = (0..self.cfg.polynomials_per_ssa)
            .map(|_| IdentifierPrimeField::<S::Scalar>::random(&mut rng))
            .collect::<Vec<_>>();

        let our_commitment_secret = sub_secrets
            .iter()
            .map(|s| s.0)
            .sum::<S::Scalar>();

        todo!()
    }
}