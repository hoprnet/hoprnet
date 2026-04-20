use std::collections::VecDeque;

#[cfg(feature = "rayon")]
use hopr_parallelize::cpu::rayon::prelude::*;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Polynomial, Share, ShareElement, ShareVerifierGroup,
    elliptic_curve::{
        Field, Group, PrimeField,
        rand_core::{CryptoRng, RngCore},
    },
};

use crate::{PixSpec, SsaIndex, SsaPolyShare, SurbPolynomialIndex, errors};

type RawPolynomial<S> = Vec<DefaultShare<IdentifierPrimeField<S>, IdentifierPrimeField<S>>>;
type RawPolynomialVerifier<E> = Vec<ShareVerifierGroup<E>>;

struct SecretPolynomial<S: PixSpec> {
    spi: SurbPolynomialIndex<S::Pseudonym>,
    raw: RawPolynomial<S::Scalar>,
    shares_generated: usize,
    t: usize,
}

impl<S: PixSpec> SecretPolynomial<S> {
    pub fn next_share(&mut self, x: S::Scalar) -> SsaPolyShare<S> {
        let eval = self.raw.evaluate(&x.into(), self.t);
        self.shares_generated += 1;
        SsaPolyShare(eval.0.to_repr())
    }
}

struct SsaPseudonymEntry<S: PixSpec> {
    ssa_index: SsaIndex,
    poly_queue: VecDeque<SecretPolynomial<S>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaShareVerifier<S: PixSpec> {
    spi: SurbPolynomialIndex<S::Pseudonym>,
    poly_commitment: Vec<ShareVerifierGroup<S::Element>>,
}

fn new_polynomial_with_verifier<S: PixSpec>(
    secret: S::Scalar,
    t: usize,
    rng: impl RngCore + CryptoRng,
) -> errors::Result<(RawPolynomial<S::Scalar>, RawPolynomialVerifier<S::Element>)> {
    let mut polynomial = RawPolynomial::create(t);
    polynomial.fill(&secret.into(), rng, t)?;

    #[cfg(not(feature = "rayon"))]
    use std::iter::once;

    #[cfg(feature = "rayon")]
    use hopr_parallelize::cpu::rayon::iter::once;

    #[cfg(feature = "rayon")]
    let coeffs_iter = polynomial[1..].par_iter().map(|c| c.identifier());

    #[cfg(not(feature = "rayon"))]
    let coeffs_iter = polynomial[1..].iter().map(|c| c.identifier());

    // Compute commitments to the coefficients of the polynomial
    let g = ShareVerifierGroup::<S::Element>::one(); // The generator of the group of verifiers
    let one = IdentifierPrimeField::one();
    let verifier = once(&one) // The first verifier is the generator
        .chain(once(polynomial[0].value())) //
        .chain(coeffs_iter)
        .map(|c| g * c)
        .collect();

    Ok((polynomial, verifier))
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
pub struct SsaShareGenerator<S: PixSpec> {
    polynomials:
        moka::sync::Cache<S::Pseudonym, std::sync::Arc<parking_lot::Mutex<SsaPseudonymEntry<S>>>, ahash::RandomState>,
    cfg: SsaGeneratorConfig,
}

impl<S: PixSpec + 'static> SsaShareGenerator<S> {
    pub fn new(cfg: SsaGeneratorConfig) -> Self {
        Self {
            polynomials: moka::sync::CacheBuilder::default()
                .initial_capacity(100_000)
                .time_to_idle(std::time::Duration::from_secs(1800))
                .build_with_hasher(ahash::RandomState::new()),
            cfg,
        }
    }

    /// Generate the next [`SsaPolyShare`] for the given pseudonym.
    ///
    /// Returns `None` if all polynomials for the given pseudonym have been used up.
    pub fn next_share(
        &self,
        pseudonym: &S::Pseudonym,
        x: S::Scalar,
    ) -> Option<(SurbPolynomialIndex<S::Pseudonym>, SsaPolyShare<S>)> {
        self.polynomials.get(pseudonym).and_then(|entry| {
            let polys = &mut entry.lock().poly_queue;
            while !polys.is_empty() {
                if let Some(poly) = polys.front_mut() {
                    if poly.shares_generated < self.cfg.threshold + self.cfg.surplus_shares {
                        return Some((poly.spi, poly.next_share(x)));
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
    pub fn new_ssa_commitment(
        &self,
        pseudonym: &S::Pseudonym,
    ) -> errors::Result<(S::Element, Vec<SsaShareVerifier<S>>)> {
        let mut rng = vsss_rs::elliptic_curve::rand_core::OsRng;

        // Generate sub-secrets for each polynomial
        let sub_secrets = (0..self.cfg.polynomials_per_ssa)
            .map(|_| <S::Scalar as Field>::random(&mut rng))
            .collect::<Vec<_>>();

        // Overall commitment secret is the sum of all sub-secrets
        let our_commitment_secret = sub_secrets.iter().sum::<S::Scalar>();

        #[cfg(not(feature = "rayon"))]
        let sub_secrets_iter = sub_secrets.into_iter();

        #[cfg(feature = "rayon")]
        let sub_secrets_iter = sub_secrets.into_par_iter();

        // Generate polynomial and verifier for each sub-secret
        let (raw_polynomials, raw_verifiers): (Vec<RawPolynomial<S::Scalar>>, Vec<RawPolynomialVerifier<S::Element>>) =
            sub_secrets_iter
                .map(|secret| new_polynomial_with_verifier::<S>(secret, self.cfg.threshold, rng))
                .collect::<errors::Result<Vec<(RawPolynomial<S::Scalar>, RawPolynomialVerifier<S::Element>)>>>()?
                .into_iter()
                .unzip();

        let mut verifiers = Vec::with_capacity(raw_verifiers.len());

        self.polynomials
            .entry_by_ref(pseudonym)
            .and_upsert_with(|entry| match entry {
                None => {
                    let new_index = 1;
                    verifiers.extend(
                        raw_verifiers
                            .into_iter()
                            .enumerate()
                            .map(|(poly_index, poly_commitment)| SsaShareVerifier {
                                spi: SurbPolynomialIndex::new(*pseudonym, new_index, poly_index as u32),
                                poly_commitment,
                            }),
                    );
                    std::sync::Arc::new(parking_lot::Mutex::new(SsaPseudonymEntry {
                        ssa_index: 1,
                        poly_queue: raw_polynomials
                            .into_iter()
                            .enumerate()
                            .map(|(poly_index, raw)| SecretPolynomial {
                                spi: SurbPolynomialIndex::new(*pseudonym, new_index, poly_index as u32),
                                raw,
                                shares_generated: 0,
                                t: self.cfg.threshold,
                            })
                            .collect(),
                    }))
                }
                Some(value) => {
                    let value = value.into_value();
                    {
                        let mut entry = value.lock();
                        entry.ssa_index += 1;

                        let new_index = entry.ssa_index;
                        verifiers.extend(
                            raw_verifiers
                                .into_iter()
                                .enumerate()
                                .map(|(poly_index, poly_commitment)| SsaShareVerifier {
                                    spi: SurbPolynomialIndex::new(*pseudonym, new_index, poly_index as u32),
                                    poly_commitment,
                                }),
                        );

                        entry
                            .poly_queue
                            .extend(raw_polynomials.into_iter().enumerate().map(|(poly_index, raw)| {
                                SecretPolynomial {
                                    spi: SurbPolynomialIndex::new(*pseudonym, new_index, poly_index as u32),
                                    raw,
                                    shares_generated: 0,
                                    t: self.cfg.threshold,
                                }
                            }));
                    }

                    value
                }
            });

        Ok((S::Element::generator() * our_commitment_secret, verifiers))
    }
}

#[cfg(test)]
mod tests {
    use hopr_types::{crypto::types::SimplePseudonym, crypto_random::Randomizable};

    use super::*;

    pub struct TestSpec;

    impl PixSpec for TestSpec {
        type Element = k256::ProjectivePoint;
        type Pseudonym = SimplePseudonym;
        type Scalar = k256::Scalar;
    }

    #[test]
    fn ssa_generator_should_generate_consecutive_spis() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
        });

        let p1 = SimplePseudonym::random();
        let (_, v) = generator.new_ssa_commitment(&p1)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p1);
            assert_eq!(v[i].spi.ssa_index(), 1);
            assert_eq!(v[i].spi.poly_index(), i as u32);
        }

        let (_, v) = generator.new_ssa_commitment(&p1)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p1);
            assert_eq!(v[i].spi.ssa_index(), 2);
            assert_eq!(v[i].spi.poly_index(), i as u32);
        }

        let p2 = SimplePseudonym::random();
        let (_, v) = generator.new_ssa_commitment(&p2)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p2);
            assert_eq!(v[i].spi.ssa_index(), 1);
            assert_eq!(v[i].spi.poly_index(), i as u32);
        }

        let (_, v) = generator.new_ssa_commitment(&p1)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p1);
            assert_eq!(v[i].spi.ssa_index(), 3);
            assert_eq!(v[i].spi.poly_index(), i as u32);
        }

        let (_, v) = generator.new_ssa_commitment(&p2)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p2);
            assert_eq!(v[i].spi.ssa_index(), 2);
            assert_eq!(v[i].spi.poly_index(), i as u32);
        }

        Ok(())
    }

    #[test]
    fn ssa_generator_should_return_shares_in_order() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 3,
            threshold: 3,
            surplus_shares: 1,
        });

        let p1 = SimplePseudonym::random();
        generator.new_ssa_commitment(&p1)?;

        for i in 0..12_u32 {
            let (spi, _) = generator
                .next_share(&p1, k256::Scalar::random(vsss_rs::elliptic_curve::rand_core::OsRng))
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(spi.pseudonym(), &p1);
            assert_eq!(spi.ssa_index(), 1);
            assert_eq!(spi.poly_index(), i / 4);
        }
        assert!(
            generator
                .next_share(&p1, k256::Scalar::random(vsss_rs::elliptic_curve::rand_core::OsRng))
                .is_none()
        );

        generator.new_ssa_commitment(&p1)?;

        for i in 0..12_u32 {
            let (spi, _) = generator
                .next_share(&p1, k256::Scalar::random(vsss_rs::elliptic_curve::rand_core::OsRng))
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(spi.pseudonym(), &p1);
            assert_eq!(spi.ssa_index(), 2);
            assert_eq!(spi.poly_index(), i / 4);
        }
        assert!(
            generator
                .next_share(&p1, k256::Scalar::random(vsss_rs::elliptic_curve::rand_core::OsRng))
                .is_none()
        );

        Ok(())
    }
}
