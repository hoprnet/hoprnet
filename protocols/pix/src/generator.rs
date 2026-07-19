use std::collections::VecDeque;

#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Polynomial, Share, ShareElement, ShareVerifierGroup,
    elliptic_curve::{Field, Group, PrimeField, group::GroupEncoding, rand_core::CryptoRng},
};

use crate::{
    CoefficientIndex, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA,
    PartialSsaShareVerifier, PixGroup, PixScalar, PixSpec, PolynomialIndex, errors,
    errors::PixError,
    traits::EntryShareGenerator,
    types::{GeneratedShare, PartialSsaShare, SsaCommitment, SsaId, SsaIndex, SsaPolynomialId, TransposedVerifiers},
};

type RawPolynomial<S> = Vec<DefaultShare<IdentifierPrimeField<PixScalar<S>>, IdentifierPrimeField<PixScalar<S>>>>;
type RawPolynomialVerifier<S> = Vec<ShareVerifierGroup<PixGroup<S>>>;

struct IndexedPolynomial<S: PixSpec> {
    spi: SsaPolynomialId<S::Pseudonym>,
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
    rng: impl CryptoRng,
) -> errors::Result<(RawPolynomial<S>, RawPolynomialVerifier<S>), S::Pseudonym> {
    let mut polynomial = RawPolynomial::<S>::create(t);
    polynomial.fill(&secret.into(), rng, t)?;

    #[cfg(not(feature = "rayon"))]
    use std::iter::once;

    #[cfg(feature = "rayon")]
    use hopr_utils::parallelize::cpu::rayon::iter::once;

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
    /// Default is [`DEFAULT_POLYS_PER_SSA`], must be between 2 and [`MAX_POLYS_PER_SSA`].
    #[default(DEFAULT_POLYS_PER_SSA)]
    #[validate(range(min = 2, max = MAX_POLYS_PER_SSA))]
    pub polynomials_per_ssa: u16,
    /// Minimum number of shares required to reconstruct each SSA polynomial.
    ///
    /// Default is [`DEFAULT_POLY_THRESHOLD`], must be between 2 and [`MAX_POLY_THRESHOLD`].
    #[default(DEFAULT_POLY_THRESHOLD)]
    #[validate(range(min = 2, max = MAX_POLY_THRESHOLD))]
    pub threshold: u16,
    /// Additional number of shares to generate beyond the threshold for redundancy.
    ///
    /// Default is 20.
    #[default(20)]
    pub surplus_shares: usize,
    /// When enabled, every generated share is corrupted (one byte XORed) so that
    /// Feldman verification at the Exit fails. Intended for integration tests.
    #[default(false)]
    pub corrupt_shares: bool,
}

/// Generator for Session Stealth Address (SSA) shares distributed over Single Use Reply Blocks (SURBs).
pub struct SsaShareGenerator<S: PixSpec> {
    polynomials:
        moka::sync::Cache<S::Pseudonym, std::sync::Arc<parking_lot::Mutex<SsaPseudonymEntry<S>>>, ahash::RandomState>,
    cfg: SsaGeneratorConfig,
}

impl<S: PixSpec> SsaShareGenerator<S> {
    /// Creates a new share generator with the provided configuration.
    pub fn new(cfg: SsaGeneratorConfig) -> Self {
        Self {
            polynomials: moka::sync::CacheBuilder::default()
                .initial_capacity(100_000)
                .time_to_idle(std::time::Duration::from_secs(1800))
                .build_with_hasher(ahash::RandomState::new()),
            cfg,
        }
    }

    /// Returns the configuration used to generate this [`SsaShareGenerator`].
    #[inline]
    pub fn config(&self) -> &SsaGeneratorConfig {
        &self.cfg
    }
}

impl<S: PixSpec> Default for SsaShareGenerator<S> {
    fn default() -> Self {
        Self::new(SsaGeneratorConfig::default())
    }
}

impl<S: PixSpec> EntryShareGenerator<S> for SsaShareGenerator<S> {
    type Error = PixError<S::Pseudonym>;

    /// Generate the next [`PartialSsaShare`] for the given pseudonym and message `msg`.
    ///
    /// IMPORTANT: Each `msg` MUST be unique for a given pseudonym.
    ///
    /// Returns `None` if all polynomials for the given pseudonym have been used up.
    /// This signals that a new SSA must be committed.
    fn next_share(
        &self,
        pseudonym: &S::Pseudonym,
        msg: &impl AsRef<[u8]>,
    ) -> errors::Result<Option<GeneratedShare<S>>, S::Pseudonym> {
        if let Some(entry) = self.polynomials.get(pseudonym) {
            let polys = &mut entry.lock().poly_queue;
            while !polys.is_empty() {
                if let Some(poly) = polys.front_mut()
                    && poly.shares_generated < self.cfg.threshold as usize + self.cfg.surplus_shares
                {
                    let x = S::msg_to_scalar(&poly.spi, msg)?;
                    // Zero would disclose the secret, so we disallow it.
                    // The chance is practically impossible.
                    if x.is_zero().into() {
                        return Err(errors::PixError::InvalidInput);
                    }

                    return Ok(Some({
                        let mut share = poly.next_share(x);

                        #[cfg(feature = "test-utils")]
                        if self.cfg.corrupt_shares {
                            if let Some(byte) = share.as_mut_inner().first_mut() {
                                *byte ^= 0xFF;
                            }
                        }

                        GeneratedShare { id: poly.spi, share }
                    }));
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
    fn new_ssa_commitment(
        &self,
        pseudonym: &S::Pseudonym,
        ssa_index: SsaIndex,
    ) -> errors::Result<SsaCommitment<S>, S::Pseudonym> {
        let mut rng = hopr_types::crypto_random::rng();

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
            .map(|secret| {
                new_polynomial_with_verifier::<S>(secret, self.cfg.threshold as usize, hopr_types::crypto_random::rng())
            })
            .collect::<errors::Result<Vec<(RawPolynomial<S>, RawPolynomialVerifier<S>)>, S::Pseudonym>>()?
            .into_iter()
            .unzip();

        let mut verifiers: Vec<PartialSsaShareVerifier<S>> = Vec::with_capacity(raw_verifiers.len());

        self.polynomials
            .entry_by_ref(pseudonym)
            .and_try_compute_with(|entry| match entry {
                None => {
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
                    Ok::<_, PixError<S::Pseudonym>>(moka::ops::compute::Op::Put(std::sync::Arc::new(
                        parking_lot::Mutex::new(SsaPseudonymEntry {
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
                                    t: self.cfg.threshold as usize,
                                })
                                .collect(),
                        }),
                    )))
                }
                Some(value) => {
                    let value = value.into_value();
                    {
                        let mut entry = value.lock();
                        if ssa_index <= entry.ssa_index {
                            return Err(PixError::InvalidInput);
                        }
                        entry.ssa_index = ssa_index;

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
                                    t: self.cfg.threshold as usize,
                                }
                            }));
                    }

                    Ok(moka::ops::compute::Op::Nop)
                }
            })?;

        let ssa_id = *verifiers[0].spi.as_ref();
        Ok(SsaCommitment {
            ssa_id,
            ssa_commitment: PixGroup::<S>::generator() * our_commitment_secret,
            verifiers: transpose_commitments(verifiers),
        })
    }
}

/// Transposes the commitments returned by the [`SsaShareGenerator::new_ssa_commitment`] to a representation
/// where verifiers are represented by coefficient index first.
pub(crate) fn transpose_commitments<S: PixSpec>(
    commitments: Vec<PartialSsaShareVerifier<S>>,
) -> TransposedVerifiers<S> {
    let mut transposed = TransposedVerifiers::<S>::new();
    commitments.into_iter().for_each(|c| {
        let spi = c.spi;
        c.poly_commitment
            .into_iter()
            .skip(1) // Skip generator
            .enumerate()
            .for_each(|(coeff_id, coeff)| {
                transposed
                    .entry(coeff_id as CoefficientIndex)
                    .or_default()
                    .push((spi.poly_index(), coeff.0.to_bytes()));
            });
    });
    transposed
}

#[cfg(test)]
mod tests {
    use hopr_types::{
        crypto::{crypto_traits, prelude::Secp256k1, types::SimplePseudonym},
        crypto_random::Randomizable,
    };
    use vsss_rs::{FeldmanVerifierSet, ReadableShareSet};

    use super::*;
    use crate::{tests::TestSpec, traits::EntryShareGenerator};

    #[test]
    fn ssa_generator_should_generate_consecutive_spis() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
            ..Default::default()
        });

        let p1 = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p1, 1.try_into()?)?;
        assert_eq!(c.ssa_id.pseudonym(), &p1);
        assert_eq!(c.ssa_id.ssa_index(), 1.try_into()?);

        let c = generator.new_ssa_commitment(&p1, 2.try_into()?)?;
        assert_eq!(c.ssa_id.pseudonym(), &p1);
        assert_eq!(c.ssa_id.ssa_index(), 2.try_into()?);

        let p2 = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p2, 1.try_into()?)?;
        assert_eq!(c.ssa_id.pseudonym(), &p2);
        assert_eq!(c.ssa_id.ssa_index(), 1.try_into()?);

        let c = generator.new_ssa_commitment(&p1, 3.try_into()?)?;
        assert_eq!(c.ssa_id.pseudonym(), &p1);
        assert_eq!(c.ssa_id.ssa_index(), 3.try_into()?);

        let c = generator.new_ssa_commitment(&p2, 2.try_into()?)?;
        assert_eq!(c.ssa_id.pseudonym(), &p2);
        assert_eq!(c.ssa_id.ssa_index(), 2.try_into()?);

        // Repeated SSA index
        assert!(generator.new_ssa_commitment(&p2, 2.try_into()?).is_err());

        Ok(())
    }

    #[test]
    fn ssa_generator_should_return_shares_in_order() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 3,
            threshold: 3,
            surplus_shares: 1,
            ..Default::default()
        });

        let p1 = SimplePseudonym::random();
        generator.new_ssa_commitment(&p1, 1.try_into()?)?;

        for i in 0..12_u16 {
            let g = generator
                .next_share(&p1, &i.to_be_bytes())?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(g.id.pseudonym(), &p1);
            assert_eq!(g.id.ssa_index(), 1.try_into()?);
            assert_eq!(g.id.poly_index(), i / 4);
        }
        assert!(generator.next_share(&p1, &20_u32.to_be_bytes())?.is_none());

        generator.new_ssa_commitment(&p1, 2.try_into()?)?;

        for i in 0..12_u16 {
            let g = generator
                .next_share(&p1, &i.to_be_bytes())?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(g.id.pseudonym(), &p1);
            assert_eq!(g.id.ssa_index(), 2.try_into()?);
            assert_eq!(g.id.poly_index(), i / 4);
        }
        assert!(generator.next_share(&p1, &20_u32.to_be_bytes())?.is_none());

        Ok(())
    }

    #[test]
    fn ssa_generator_shares_must_be_verifiable() -> anyhow::Result<()> {
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
            ..Default::default()
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);

        assert_eq!(&cfg, generator.config());

        let p = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p, 1.try_into()?)?;
        let verifiers = c.reconstruct_verifiers().map_err(anyhow::Error::msg)?;

        for verifier in verifiers.iter().take(cfg.polynomials_per_ssa as usize) {
            for _ in 0..(cfg.threshold as usize + cfg.surplus_shares) {
                let x = hopr_types::crypto_random::random_bytes::<10>();

                let g = generator
                    .next_share(&p, &x)?
                    .ok_or(anyhow::anyhow!("failed to generate share"))?;

                verifier.verify(&g.share, x)?;
            }
        }

        Ok(())
    }

    #[test]
    fn ssa_generator_corresponds_to_standard_verifier_and_recoverer() -> anyhow::Result<()> {
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
            ..Default::default()
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);

        let p = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p, 1.try_into()?)?;
        let orig_commitment = c.ssa_commitment;
        let verifiers = c.reconstruct_verifiers().map_err(anyhow::Error::msg)?;
        let vs = verifiers.into_iter().map(|v| v.poly_commitment).collect::<Vec<_>>();

        let mut recovered_secret = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::default();
        for v in vs.iter().take(cfg.polynomials_per_ssa as usize) {
            let mut shares = Vec::new();
            for _ in 0..(cfg.threshold as usize + cfg.surplus_shares) {
                let x = hopr_types::crypto_random::random_bytes::<10>();

                let g = generator
                    .next_share(&p, &x)?
                    .ok_or(anyhow::anyhow!("failed to generate share"))?;
                let complete_share = DefaultShare {
                    identifier: TestSpec::msg_to_scalar(&g.id, x)?.into(),
                    value: crypto_traits::elliptic_curve::Scalar::<Secp256k1>::from_repr(g.share.0)
                        .unwrap()
                        .into(),
                };

                v.verify_share(&complete_share)
                    .map_err(|_| anyhow::anyhow!("invalid share"))?;
                shares.push(complete_share);
            }
            recovered_secret += shares.combine().map_err(anyhow::Error::msg)?.0;
        }

        assert_eq!(
            orig_commitment.to_affine(),
            (crypto_traits::elliptic_curve::ProjectivePoint::<Secp256k1>::GENERATOR * recovered_secret).to_affine()
        );

        Ok(())
    }
}
