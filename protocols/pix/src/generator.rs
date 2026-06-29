use std::collections::VecDeque;

use elliptic_curve::{
    Field, Group, PrimeField,
    rand_core::{CryptoRng, OsRng, RngCore},
};

use crate::{
    CoefficientIndex, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA,
    PartialSsaShareVerifier, PixGroup, PixScalar, PixSpec, PolynomialIndex, ShareVerifierGroup,
    combine::RawPolynomial,
    errors,
    errors::PixError,
    traits::EntryShareGenerator,
    types::{GeneratedShare, PartialSsaShare, SsaCommitment, SsaId, SsaIndex, SsaPolynomialId, TransposedVerifiers},
};

type RawPolynomialVerifier<S> = Vec<ShareVerifierGroup<PixGroup<S>>>;

type PolynomialWithVerifier<S> = (RawPolynomial<PixScalar<S>>, RawPolynomialVerifier<S>);

struct IndexedPolynomial<S: PixSpec> {
    spi: SsaPolynomialId<S::Pseudonym>,
    raw: RawPolynomial<PixScalar<S>>,
    shares_generated: usize,
    t: usize,
}

impl<S: PixSpec> IndexedPolynomial<S> {
    pub fn next_share(&mut self, x: PixScalar<S>) -> PartialSsaShare<S> {
        let eval = self.raw.evaluate(&x, self.t);
        self.shares_generated += 1;
        PartialSsaShare(eval.value().to_repr())
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
) -> errors::Result<PolynomialWithVerifier<S>, S::Pseudonym> {
    // Create a polynomial with degree t-1 (t coefficients: secret + t-1 random)
    // This matches the threshold behavior where t shares are needed for reconstruction
    let polynomial = RawPolynomial::create_with_threshold(t).fill(&secret, rng, t.saturating_sub(1));

    // Compute commitments to the coefficients of the polynomial
    let g = PixGroup::<S>::generator();
    // Include generator as first entry, followed by coefficient commitments
    // Total: 1 (generator) + polynomial coefficients (degree + 1)
    let mut verifier = Vec::with_capacity(1 + polynomial.0.len());
    verifier.push(ShareVerifierGroup::from(g)); // Generator first
    for coeff in polynomial.0.iter() {
        verifier.push(ShareVerifierGroup::from(g * coeff.value()));
    }

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
    pub polynomials_per_ssa: usize,
    /// Minimum number of shares required to reconstruct each SSA polynomial.
    ///
    /// Default is [`DEFAULT_POLY_THRESHOLD`], must be between 2 and [`MAX_POLY_THRESHOLD`].
    #[default(DEFAULT_POLY_THRESHOLD)]
    #[validate(range(min = 2, max = MAX_POLY_THRESHOLD))]
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
                    && poly.shares_generated < self.cfg.threshold + self.cfg.surplus_shares
                {
                    let x = S::msg_to_scalar(&poly.spi, msg)?;
                    // Zero would disclose the secret, so we disallow it.
                    // The chance is practically impossible.
                    if x.is_zero().into() {
                        return Err(errors::PixError::InvalidInput);
                    }

                    return Ok(Some(GeneratedShare {
                        id: poly.spi,
                        share: poly.next_share(x),
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
    ) -> errors::Result<SsaCommitment<S>, S::Pseudonym> {
        let mut rng = OsRng;

        // Generate polynomial and verifier for each sub-secret
        let mut raw_polynomials = Vec::with_capacity(self.cfg.polynomials_per_ssa);
        let mut raw_verifiers = Vec::with_capacity(self.cfg.polynomials_per_ssa);
        let mut our_commitment_secret = PixScalar::<S>::ZERO;

        for _ in 0..self.cfg.polynomials_per_ssa {
            let secret = <PixScalar<S> as Field>::random(&mut rng);
            our_commitment_secret += secret;
            let (poly, verifier) = new_polynomial_with_verifier::<S>(secret, self.cfg.threshold, &mut rng)?;
            raw_polynomials.push(poly);
            raw_verifiers.push(verifier);
        }

        let ssa_index = if self.polynomials.contains_key(pseudonym) {
            let entry = self.polynomials.get(pseudonym).unwrap(); // Arc clone, stays in cache
            let mut entry = entry.lock();
            let ssa_index = entry.ssa_index.checked_add(1).ok_or(PixError::SsaIndexOverflow)?;
            entry.ssa_index = ssa_index;
            entry.poly_queue.extend(raw_polynomials.into_iter().enumerate().map(|(poly_index, raw)| {
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
            ssa_index
        } else {
            let ssa_index = SsaIndex::MIN;
            self.polynomials.insert(
                *pseudonym,
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
                })),
            );
            ssa_index
        };

        let mut verifiers: Vec<PartialSsaShareVerifier<S>> = Vec::with_capacity(raw_verifiers.len());
        verifiers.extend(raw_verifiers.into_iter().enumerate().map(|(poly_index, poly_commitment)| {
            PartialSsaShareVerifier {
                spi: SsaPolynomialId::new(
                    SsaId::new(*pseudonym, ssa_index),
                    poly_index as PolynomialIndex,
                ),
                poly_commitment,
            }
        }));

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
                    .push((spi.poly_index(), coeff.to_bytes()));
            });
    });
    transposed
}

#[cfg(test)]
mod tests {
    use elliptic_curve::rand_core::OsRng;
    use hopr_types::{crypto::types::SimplePseudonym, crypto_random::Randomizable};
    use vsss_rs::{
        DefaultShare, IdentifierPrimeField, ParticipantIdGeneratorType, ReadableShareSet, Share, ShareVerifierGroup,
        ValuePrimeField, feldman,
    };

    use super::*;
    use crate::combine::Share as OurShare;
    use crate::{combine::ReadableShareSet as OurReadableShareSet, tests::TestSpec, traits::EntryShareGenerator};

    #[test]
    fn ssa_generator_should_generate_consecutive_spis() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
        });

        let p1 = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p1)?;
        assert_eq!(c.ssa_id.pseudonym(), &p1);
        assert_eq!(c.ssa_id.ssa_index(), 1.try_into()?);

        let c = generator.new_ssa_commitment(&p1)?;
        assert_eq!(c.ssa_id.pseudonym(), &p1);
        assert_eq!(c.ssa_id.ssa_index(), 2.try_into()?);

        let p2 = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p2)?;
        assert_eq!(c.ssa_id.pseudonym(), &p2);
        assert_eq!(c.ssa_id.ssa_index(), 1.try_into()?);

        let c = generator.new_ssa_commitment(&p1)?;
        assert_eq!(c.ssa_id.pseudonym(), &p1);
        assert_eq!(c.ssa_id.ssa_index(), 3.try_into()?);

        let c = generator.new_ssa_commitment(&p2)?;
        assert_eq!(c.ssa_id.pseudonym(), &p2);
        assert_eq!(c.ssa_id.ssa_index(), 2.try_into()?);

        // With auto-increment, p2's index continues from 1 → 2 → 3 → … independently
        // of p1's sequence. The overflow (SsaIndexOverflow) only triggers when a
        // pseudonym's ssa_index reaches u32::MAX, which is not reachable in this test.

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
            let g = generator
                .next_share(&p1, &i.to_be_bytes())?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(g.id.pseudonym(), &p1);
            assert_eq!(g.id.ssa_index(), 1.try_into()?);
            assert_eq!(g.id.poly_index(), i / 4);
        }
        assert!(generator.next_share(&p1, &20_u32.to_be_bytes())?.is_none());

        generator.new_ssa_commitment(&p1)?;

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
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);

        assert_eq!(&cfg, generator.config());

        let p = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p)?;
        let verifiers = c.reconstruct_verifiers().map_err(anyhow::Error::msg)?;

        for verifier in verifiers.iter().take(cfg.polynomials_per_ssa) {
            for _ in 0..(cfg.threshold + cfg.surplus_shares) {
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
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);

        let p = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p)?;
        let orig_commitment = c.ssa_commitment;
        let verifiers = c.reconstruct_verifiers().map_err(anyhow::Error::msg)?;

        let mut recovered_secret = k256::Scalar::default();
        for v in verifiers.iter().take(cfg.polynomials_per_ssa) {
            let mut shares = Vec::new();
            for _ in 0..(cfg.threshold + cfg.surplus_shares) {
                let x = hopr_types::crypto_random::random_bytes::<10>();

                let g = generator
                    .next_share(&p, &x)?
                    .ok_or(anyhow::anyhow!("failed to generate share"))?;
                let identifier = TestSpec::msg_to_scalar(&g.id, x)?;
                let value = k256::Scalar::from_repr(g.share.0).unwrap();
                let complete_share =
                    OurShare::new(identifier, value).ok_or_else(|| anyhow::anyhow!("invalid share"))?;

                // Verify using our verifier
                v.verify_completed_share(&complete_share)
                    .map_err(|_| anyhow::anyhow!("invalid share"))?;
                shares.push(complete_share);
            }
            // Use our combine function
            recovered_secret += OurReadableShareSet::combine(&shares)?.0;
        }

        assert_eq!(
            orig_commitment.to_affine(),
            (k256::ProjectivePoint::GENERATOR * recovered_secret).to_affine()
        );

        Ok(())
    }

    /// Test that verifies our polynomial implementation generates valid evaluations
    /// and correctly reconstructs the secret using Lagrange interpolation.
    #[test]
    fn polynomial_generation_and_reconstruction_works() -> anyhow::Result<()> {
        use crate::combine::ReadableShareSet;

        let mut rng = OsRng;
        let secret = k256::Scalar::random(&mut rng);
        let t = 5;

        // Generate polynomial using our implementation
        // create_with_threshold(t) creates capacity for t coefficients (degree t-1)
        // fill(t-1) pushes 1 (secret) + (t-1) random = t coefficients
        let our_poly = RawPolynomial::create_with_threshold(t).fill(&secret, &mut rng, t - 1);

        // Generate shares at 5 points
        let x_values = vec![
            k256::Scalar::from(1u32),
            k256::Scalar::from(2u32),
            k256::Scalar::from(3u32),
            k256::Scalar::from(4u32),
            k256::Scalar::from(5u32),
        ];

        // Create shares using our polynomial evaluation
        let our_shares: Vec<OurShare> = x_values
            .iter()
            .map(|x| {
                let share = our_poly.evaluate(x, t);
                OurShare::new(*x, *share.value()).unwrap()
            })
            .collect();

        // Verify we have the correct number of coefficients
        assert_eq!(our_poly.len(), t);
        assert_eq!(our_poly.0[0].value, secret);

        // Reconstruct the secret using Lagrange interpolation at x=0
        let reconstructed: Vec<OurShare> = our_shares.clone();
        let recovered_secret = <Vec<OurShare> as ReadableShareSet<k256::Scalar>>::combine(&reconstructed)?.0;

        // Verify reconstruction matches original secret
        assert_eq!(recovered_secret, secret);

        // Verify that combining any t shares reconstructs the secret
        for i in 0..(our_shares.len() - t + 1) {
            let subset: Vec<OurShare> = our_shares[i..i + t].to_vec();
            let subset_recovered = <Vec<OurShare> as ReadableShareSet<k256::Scalar>>::combine(&subset)?.0;
            assert_eq!(
                subset_recovered, secret,
                "Reconstruction failed with subset starting at index {}",
                i
            );
        }

        Ok(())
    }

    /// Test that our combine matches vsss-rs combine
    #[test]
    fn combine_matches_vsss_rs() -> anyhow::Result<()> {
        let mut rng = OsRng;
        let secret = k256::Scalar::random(&mut rng);
        let t = 3;

        // Define share and verifier types
        type VsssShare = DefaultShare<IdentifierPrimeField<k256::Scalar>, ValuePrimeField<k256::Scalar>>;
        type VsssVerifier = ShareVerifierGroup<k256::ProjectivePoint>;

        // Generate shares using vsss-rs
        let (vsss_shares, _verifier) = feldman::split_secret_with_participant_generator::<VsssShare, VsssVerifier>(
            t,
            5,
            &ValuePrimeField::from(secret),
            None,
            &mut rng,
            &[ParticipantIdGeneratorType::list(&[
                k256::Scalar::from(1u32).into(),
                k256::Scalar::from(2u32).into(),
                k256::Scalar::from(3u32).into(),
                k256::Scalar::from(4u32).into(),
                k256::Scalar::from(5u32).into(),
            ])],
        )
        .map_err(anyhow::Error::msg)?;

        // Convert to our share type
        let our_shares: Vec<OurShare> = vsss_shares
            .iter()
            .map(|s| {
                let id_scalar = s.identifier().0;
                let val_scalar = s.value().0;
                OurShare::new(id_scalar, val_scalar).unwrap()
            })
            .collect();

        // Combine using our implementation
        let our_result = OurReadableShareSet::combine(&our_shares)?;

        // Combine using vsss-rs
        let vsss_result = vsss_shares.combine().map_err(anyhow::Error::msg)?;

        // Both should give the same secret
        assert_eq!(our_result.0, vsss_result.0);

        Ok(())
    }

    #[test]
    fn ssa_generator_benchmark_exact_repro() -> anyhow::Result<()> {
        let cfg = SsaGeneratorConfig {
            threshold: 10,
            polynomials_per_ssa: 128,
            ..Default::default()
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);
        let pseudonym = SimplePseudonym::random();

        for _ in 0..5 {
            generator.new_ssa_commitment(&pseudonym)?;
        }
        Ok(())
    }
}
