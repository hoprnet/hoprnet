use std::collections::{HashMap, VecDeque};

#[cfg(feature = "rayon")]
use hopr_parallelize::cpu::rayon::prelude::*;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Polynomial, Share, ShareElement, ShareVerifierGroup,
    elliptic_curve::{
        Field, Group, PrimeField,
        rand_core::{CryptoRng, RngCore},
    },
};

use crate::{DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, PartialSsaShareVerifier, PixGroup, PixScalar, PixSpec, PolynomialIndex, errors, msg_to_scalar, types::{PartialSsaShare, SsaId, SsaIndex, SsaPolynomialId}, CoefficientIndex, PixGroupRepr};

type RawPolynomial<S> = Vec<DefaultShare<IdentifierPrimeField<PixScalar<S>>, IdentifierPrimeField<PixScalar<S>>>>;
type RawPolynomialVerifier<S> = Vec<ShareVerifierGroup<PixGroup<S>>>;

struct IndexedPolynomial<S: PixSpec> {
    spi: SsaPolynomialId<S>,
    raw: RawPolynomial<S>,
    shares_generated: usize,
    t: usize,
}

impl<S: PixSpec> IndexedPolynomial<S> {
    pub fn next_share(&mut self, x: PixScalar<S>) -> PartialSsaShare<S> {
        let eval = self.raw.evaluate(&x.into(), self.t);
        self.shares_generated += 1;
        PartialSsaShare(eval.0.to_repr())
    }
}

struct SsaPseudonymEntry<S: PixSpec> {
    ssa_index: SsaIndex,
    poly_queue: VecDeque<IndexedPolynomial<S>>,
}

fn new_polynomial_with_verifier<S: PixSpec>(
    secret: PixScalar<S>,
    t: usize,
    rng: impl RngCore + CryptoRng,
) -> errors::Result<(RawPolynomial<S>, RawPolynomialVerifier<S>)> {
    let mut polynomial = RawPolynomial::<S>::create(t);
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
    let g = ShareVerifierGroup::<PixGroup<S>>::one(); // The generator of the group of verifiers
    let one = IdentifierPrimeField::one();
    let verifier = once(&one) // The first verifier is the generator
        .chain(once(polynomial[0].value())) //
        .chain(coeffs_iter)
        .map(|c| g * c)
        .collect();

    Ok((polynomial, verifier))
}

/// Configuration for the [`SsaShareGenerator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaGeneratorConfig {
    /// The number of polynomials to generate per SSA commitment.
    ///
    /// Default is [`DEFAULT_POLYS_PER_SSA`], must be between 2 and 65535.
    #[default(DEFAULT_POLYS_PER_SSA)]
    #[validate(range(min = 2, max = 65535))]
    pub polynomials_per_ssa: usize,
    /// Minimum number of shares required to reconstruct each SSA polynomial.
    ///
    /// Default is [`DEFAULT_POLY_THRESHOLD`], must be between 2 and 1000.
    #[default(DEFAULT_POLY_THRESHOLD)]
    #[validate(range(min = 2, max = 1000))]
    pub threshold: usize,
    /// Additional number of shares to generate beyond the threshold for redundancy.
    ///
    /// Default is 20.
    #[default(20)]
    pub surplus_shares: usize,
}

/// Generator for Session Stealth Address (SSA) shares distributed over Single Use Reply Blocks (SURBs).
pub struct SsaShareGenerator<S: PixSpec> {
    polynomials:
        moka::sync::Cache<S::Pseudonym, std::sync::Arc<parking_lot::Mutex<SsaPseudonymEntry<S>>>, ahash::RandomState>,
    cfg: SsaGeneratorConfig,
}

/// Tuple consisting of the SSA polynomial index and an SSA share from the corresponding polynomial.
pub type GeneratedShare<S> = (SsaPolynomialId<S>, PartialSsaShare<S>);

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

    /// Generate the next [`PartialSsaShare`] for the given pseudonym and message `msg`.
    ///
    /// IMPORTANT: Each `msg` MUST be unique for a given pseudonym.
    ///
    /// Returns `None` if all polynomials for the given pseudonym have been used up.
    /// This signals that a new SSA must be committed.
    pub fn next_share(
        &self,
        pseudonym: &S::Pseudonym,
        msg: &impl AsRef<[u8]>,
    ) -> errors::Result<Option<GeneratedShare<S>>> {
        if let Some(entry) = self.polynomials.get(pseudonym) {
            let polys = &mut entry.lock().poly_queue;
            while !polys.is_empty() {
                if let Some(poly) = polys.front_mut()
                    && poly.shares_generated < self.cfg.threshold + self.cfg.surplus_shares
                {
                    let x = msg_to_scalar::<S>(&poly.spi, msg)?;
                    // Zero would disclose the secret, so we disallow it.
                    // The chance is practically impossible.
                    if x.is_zero().into() {
                        return Err(errors::PixError::InvalidInput);
                    }

                    return Ok(Some((poly.spi, poly.next_share(x))));
                }
                // If we replaced VecDeque with a lock-free alternative, we could remove
                // the mutex, but the alternative would need to effectively deallocate,
                // so the polynomials do not grow indefinitely when new commitments are
                // being added.
                polys.pop_front();
            }
        }
        Ok(None)
    }

    /// Generates a new SSA commitment from the sender side, for the given `pseudonym`.
    ///
    /// Returns the new random SSA-commitment and the corresponding SSA share verifier.
    pub fn new_ssa_commitment(
        &self,
        pseudonym: &S::Pseudonym,
    ) -> errors::Result<(PixGroup<S>, Vec<PartialSsaShareVerifier<S>>)> {
        let mut rng = vsss_rs::elliptic_curve::rand_core::OsRng;

        // Generate sub-secrets for each polynomial
        let sub_secrets = (0..self.cfg.polynomials_per_ssa)
            .map(|_| <PixScalar<S> as Field>::random(&mut rng))
            .collect::<Vec<_>>();

        // Overall commitment secret is the sum of all sub-secrets
        let our_commitment_secret = sub_secrets.iter().sum::<PixScalar<S>>();

        #[cfg(not(feature = "rayon"))]
        let sub_secrets_iter = sub_secrets.into_iter();

        #[cfg(feature = "rayon")]
        let sub_secrets_iter = sub_secrets.into_par_iter();

        // Generate polynomial and verifier for each sub-secret
        let (raw_polynomials, raw_verifiers): (Vec<RawPolynomial<S>>, Vec<RawPolynomialVerifier<S>>) = sub_secrets_iter
            .map(|secret| new_polynomial_with_verifier::<S>(secret, self.cfg.threshold, rng))
            .collect::<errors::Result<Vec<(RawPolynomial<S>, RawPolynomialVerifier<S>)>>>()?
            .into_iter()
            .unzip();

        let mut verifiers = Vec::with_capacity(raw_verifiers.len());

        self.polynomials
            .entry_by_ref(pseudonym)
            .and_upsert_with(|entry| match entry {
                None => {
                    let ssa_index = 1;
                    verifiers.extend(
                        raw_verifiers
                            .into_iter()
                            .enumerate()
                            .map(|(poly_index, poly_commitment)| PartialSsaShareVerifier {
                                spi: SsaPolynomialId::new(
                                    SsaId::new(*pseudonym, ssa_index),
                                    poly_index as PolynomialIndex,
                                ),
                                poly_commitment,
                            }),
                    );
                    std::sync::Arc::new(parking_lot::Mutex::new(SsaPseudonymEntry {
                        ssa_index,
                        poly_queue: raw_polynomials
                            .into_iter()
                            .enumerate()
                            .map(|(poly_index, raw)| IndexedPolynomial {
                                spi: SsaPolynomialId::new(
                                    SsaId::new(*pseudonym, ssa_index),
                                    poly_index as PolynomialIndex,
                                ),
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

                        let ssa_index = entry.ssa_index;
                        verifiers.extend(
                            raw_verifiers
                                .into_iter()
                                .enumerate()
                                .map(|(poly_index, poly_commitment)| PartialSsaShareVerifier {
                                    spi: SsaPolynomialId::new(
                                        SsaId::new(*pseudonym, ssa_index),
                                        poly_index as PolynomialIndex,
                                    ),
                                    poly_commitment,
                                }),
                        );

                        entry
                            .poly_queue
                            .extend(raw_polynomials.into_iter().enumerate().map(|(poly_index, raw)| {
                                IndexedPolynomial {
                                    spi: SsaPolynomialId::new(
                                        SsaId::new(*pseudonym, ssa_index),
                                        poly_index as PolynomialIndex,
                                    ),
                                    raw,
                                    shares_generated: 0,
                                    t: self.cfg.threshold,
                                }
                            }));
                    }

                    value
                }
            });

        Ok((PixGroup::<S>::generator() * our_commitment_secret, verifiers))
    }
}

/// Transposes the commitments returned by the [`SsaShareGenerator::new_ssa_commitment`] to a representation
/// where verifiers are represented by coefficient index first.
pub fn transpose_commitments<S: PixSpec>(commitments: Vec<PartialSsaShareVerifier<S>>) -> HashMap<CoefficientIndex, HashMap<PolynomialIndex, PixGroupRepr<S>>> {
    let mut transposed = HashMap::<CoefficientIndex, HashMap<PolynomialIndex, PixGroupRepr<S>>>::new();
    commitments
        .into_iter()
        .map(|c| c.into_serializable_commitments())
        .for_each(|(spi, committed_polynomial)| {
            for (coeff_id, coeff) in committed_polynomial.into_iter().enumerate() {
                transposed
                    .entry(coeff_id as CoefficientIndex)
                    .or_default()
                    .insert(spi.poly_index(), coeff);
            }
        });
    transposed
}

#[cfg(test)]
mod tests {
    use hopr_types::{crypto::types::SimplePseudonym, crypto_random::Randomizable};
    use vsss_rs::{FeldmanVerifierSet, ReadableShareSet};

    use super::*;
    use crate::{msg_to_scalar, tests::TestSpec};

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
            assert_eq!(v[i].spi.poly_index(), i as PolynomialIndex);
        }

        let (_, v) = generator.new_ssa_commitment(&p1)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p1);
            assert_eq!(v[i].spi.ssa_index(), 2);
            assert_eq!(v[i].spi.poly_index(), i as PolynomialIndex);
        }

        let p2 = SimplePseudonym::random();
        let (_, v) = generator.new_ssa_commitment(&p2)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p2);
            assert_eq!(v[i].spi.ssa_index(), 1);
            assert_eq!(v[i].spi.poly_index(), i as PolynomialIndex);
        }

        let (_, v) = generator.new_ssa_commitment(&p1)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p1);
            assert_eq!(v[i].spi.ssa_index(), 3);
            assert_eq!(v[i].spi.poly_index(), i as PolynomialIndex);
        }

        let (_, v) = generator.new_ssa_commitment(&p2)?;
        for i in 0..generator.cfg.polynomials_per_ssa {
            assert_eq!(v[i].spi.pseudonym(), &p2);
            assert_eq!(v[i].spi.ssa_index(), 2);
            assert_eq!(v[i].spi.poly_index(), i as PolynomialIndex);
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

        for i in 0..12_u16 {
            let (spi, _) = generator
                .next_share(&p1, &i.to_be_bytes())?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(spi.pseudonym(), &p1);
            assert_eq!(spi.ssa_index(), 1);
            assert_eq!(spi.poly_index(), i / 4);
        }
        assert!(generator.next_share(&p1, &20_u32.to_be_bytes())?.is_none());

        generator.new_ssa_commitment(&p1)?;

        for i in 0..12_u16 {
            let (spi, _) = generator
                .next_share(&p1, &i.to_be_bytes())?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(spi.pseudonym(), &p1);
            assert_eq!(spi.ssa_index(), 2);
            assert_eq!(spi.poly_index(), i / 4);
        }
        assert!(generator.next_share(&p1, &20_u32.to_be_bytes())?.is_none());

        Ok(())
    }

    #[test]
    fn ssa_generator_shares_must_be_verifiable() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
        });

        let p = SimplePseudonym::random();
        let (_, vs) = generator.new_ssa_commitment(&p)?;

        for poly_index in 0..10 {
            for _ in 0..12 {
                let x = hopr_types::crypto_random::random_bytes::<10>();

                let (_, share) = generator
                    .next_share(&p, &x)?
                    .ok_or(anyhow::anyhow!("failed to generate share"))?;

                vs[poly_index].verify(&share, x)?;
            }
        }

        Ok(())
    }

    #[test]
    fn ssa_generator_corresponds_to_standard_verifier_and_recoverer() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
        });

        let p = SimplePseudonym::random();
        let (orig_commitment, vs) = generator.new_ssa_commitment(&p)?;
        let vs = vs.into_iter().map(|v| v.poly_commitment).collect::<Vec<_>>();

        let mut recovered_secret = k256::Scalar::default();
        for poly_index in 0..10 {
            let mut shares = Vec::new();
            for _ in 0..12 {
                let x = hopr_types::crypto_random::random_bytes::<10>();

                let (spi, share) = generator
                    .next_share(&p, &x)?
                    .ok_or(anyhow::anyhow!("failed to generate share"))?;
                let complete_share = DefaultShare {
                    identifier: msg_to_scalar::<TestSpec>(&spi, x)?.into(),
                    value: k256::Scalar::from_repr(share.0).unwrap().into(),
                };

                vs[poly_index]
                    .verify_share(&complete_share)
                    .map_err(|_| anyhow::anyhow!("invalid share"))?;
                shares.push(complete_share);
            }
            recovered_secret += shares.combine().map_err(anyhow::Error::msg)?.0;
        }

        assert_eq!(
            orig_commitment.to_affine(),
            (k256::ProjectivePoint::GENERATOR * recovered_secret).to_affine()
        );

        Ok(())
    }
}
