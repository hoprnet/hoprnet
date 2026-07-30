use std::collections::VecDeque;

#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;
use validator::Validate;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Polynomial,
    elliptic_curve::{Field, Group, PrimeField, group::GroupEncoding, rand_core::CryptoRng},
};

use crate::{
    CONSTANT_TERM_COEFFICIENT, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA,
    PixGroup, PixScalar, PixSpec, PolynomialIndex, SsaPartCommitment, errors,
    errors::PixError,
    traits::EntryShareGenerator,
    types::{
        GeneratedShare, PartialSsaShare, SsaCommitment, SsaCommitmentProof, SsaId, SsaIndex, SsaPolynomialId,
        TransposedVerifiers,
    },
};

type RawPolynomial<S> = Vec<DefaultShare<IdentifierPrimeField<PixScalar<S>>, IdentifierPrimeField<PixScalar<S>>>>;

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

/// Builds a Shamir polynomial of degree `t - 1` over `secret` and commits to its constant term.
///
/// Only the constant term is committed to. The higher coefficients still exist — they are what
/// makes the shares hide the secret — but no commitment to them is published, so the Exit cannot
/// (and no longer needs to) check an individual share. See [`SsaPartCommitment`].
///
/// This is also why the Entry's per-cycle cost collapsed: committing to every coefficient was
/// `polys × threshold` fixed-base multiplications against an untabulated generator, over half a
/// million of them at production dimensions, all inside one blocking task at each cycle boundary.
fn new_polynomial_with_commitment<S: PixSpec>(
    secret: PixScalar<S>,
    t: usize,
    rng: impl CryptoRng,
) -> errors::Result<(RawPolynomial<S>, PixGroup<S>), S::Pseudonym> {
    let mut polynomial = RawPolynomial::<S>::create(t);
    polynomial.fill(&secret.into(), rng, t)?;

    Ok((polynomial, PixGroup::<S>::mul_by_generator(&secret)))
}

/// Configuration for the [`SsaShareGenerator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaGeneratorConfig {
    /// The number of polynomials to generate per SSA commitment.
    ///
    /// Default is [`DEFAULT_POLYS_PER_SSA`], must be between 1 and [`MAX_POLYS_PER_SSA`].
    #[default(DEFAULT_POLYS_PER_SSA)]
    #[validate(range(min = 1, max = MAX_POLYS_PER_SSA))]
    pub polynomials_per_ssa: u16,
    /// Minimum number of shares required to reconstruct each SSA polynomial.
    ///
    /// Default is [`DEFAULT_POLY_THRESHOLD`], must be between 2 and [`MAX_POLY_THRESHOLD`].
    #[default(DEFAULT_POLY_THRESHOLD)]
    #[validate(range(min = 2, max = MAX_POLY_THRESHOLD))]
    pub threshold: u16,
    /// Additional number of shares to generate beyond the threshold for redundancy.
    ///
    /// Covers *lost* shares only: the Exit reconstructs from the first `threshold` distinct shares
    /// that reach it, so any of the surplus can stand in for one that never arrives. It does not
    /// cover *corrupt* shares — nothing checks a share on arrival any more, so a bad one is only
    /// noticed once it has already poisoned the interpolation. See
    /// [`SsaPartCommitment`](crate::SsaPartCommitment).
    ///
    /// Default is 20, must be between 0 and 4096.
    #[default(20)]
    #[validate(range(min = 0, max = 4096))]
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
    ///
    /// # Panics
    /// Panics if the configuration fails validation.
    pub fn new(cfg: SsaGeneratorConfig) -> Self {
        cfg.validate().expect("invalid SsaGeneratorConfig");
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

        // Generate polynomial and constant-term commitment for each sub-secret
        let (raw_polynomials, raw_commitments): (Vec<RawPolynomial<S>>, Vec<PixGroup<S>>) = sub_secrets_iter
            .map(|secret| {
                new_polynomial_with_commitment::<S>(
                    secret,
                    self.cfg.threshold as usize,
                    hopr_types::crypto_random::rng(),
                )
            })
            .collect::<errors::Result<Vec<(RawPolynomial<S>, PixGroup<S>)>, S::Pseudonym>>()?
            .into_iter()
            .unzip();

        let mut commitments: Vec<SsaPartCommitment<S>> = Vec::with_capacity(raw_commitments.len());

        self.polynomials
            .entry_by_ref(pseudonym)
            .and_try_compute_with(|entry| match entry {
                None => {
                    commitments.extend(
                        raw_commitments
                            .into_iter()
                            .enumerate()
                            .map(|(poly_index, constant_term)| SsaPartCommitment {
                                spi: SsaPolynomialId::new(
                                    SsaId::new(*pseudonym, ssa_index),
                                    poly_index as PolynomialIndex,
                                ),
                                constant_term,
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

                        commitments.extend(raw_commitments.into_iter().enumerate().map(
                            |(poly_index, constant_term)| SsaPartCommitment {
                                spi: SsaPolynomialId::new(
                                    SsaId::new(*pseudonym, ssa_index),
                                    poly_index as PolynomialIndex,
                                ),
                                constant_term,
                            },
                        ));

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

        let ssa_id = *commitments[0].spi.as_ref();
        let ssa_commitment = PixGroup::<S>::generator() * our_commitment_secret;
        Ok(SsaCommitment {
            ssa_id,
            ssa_commitment,
            // Proves we know the sum of the sub-secrets. The recipient adds its own commitment to
            // ours to get the deposit key, and this is what stops us from having chosen our half so
            // that we — rather than nobody — know that sum. See `SsaCommitmentProof`.
            commitment_proof: SsaCommitmentProof::prove(&ssa_id, &our_commitment_secret, &ssa_commitment)?,
            verifiers: transposed_constant_terms(commitments),
        })
    }
}

/// Lays the per-polynomial constant-term commitments out in the coefficient-major form the wire
/// messages expect.
///
/// The result always holds exactly one key, [`CONSTANT_TERM_COEFFICIENT`]. The map shape is kept
/// rather than flattened to a plain `Vec` because it is what `SsaClientCommitmentMessage` splits
/// into packets, and the wire format still admits higher coefficient indices even though PIX no
/// longer produces any.
pub(crate) fn transposed_constant_terms<S: PixSpec>(commitments: Vec<SsaPartCommitment<S>>) -> TransposedVerifiers<S> {
    let mut transposed = TransposedVerifiers::<S>::new();
    transposed.insert(
        CONSTANT_TERM_COEFFICIENT,
        commitments
            .into_iter()
            .map(|c| (c.spi.poly_index(), c.constant_term.to_bytes()))
            .collect(),
    );
    transposed
}

#[cfg(test)]
mod tests {
    use hopr_types::{
        crypto::{crypto_traits, prelude::Secp256k1, types::SimplePseudonym},
        crypto_random::Randomizable,
    };
    use vsss_rs::ReadableShareSet;

    use super::*;
    use crate::{tests::TestSpec, traits::EntryShareGenerator};

    #[test]
    fn ssa_generator_should_generate_consecutive_spis() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
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

    /// Every polynomial's shares must interpolate back to the constant term the generator
    /// committed to — the single check the Exit performs, in place of the per-share Feldman
    /// verification that used to run `threshold` scalar multiplications per share.
    #[test]
    fn ssa_generator_parts_must_open_their_commitments() -> anyhow::Result<()> {
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);

        assert_eq!(&cfg, generator.config());

        let p = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p, 1.try_into()?)?;
        let commitments = c.reconstruct_part_commitments().map_err(anyhow::Error::msg)?;
        assert_eq!(cfg.polynomials_per_ssa as usize, commitments.len());

        for commitment in &commitments {
            // Only `threshold` shares are consumed; the surplus is drained afterwards so the
            // generator advances to the next polynomial.
            let mut shares = Vec::new();
            for i in 0..(cfg.threshold as usize + cfg.surplus_shares) {
                let x = hopr_types::crypto_random::random_bytes::<10>();
                let g = generator
                    .next_share(&p, &x)?
                    .ok_or(anyhow::anyhow!("failed to generate share"))?;
                assert_eq!(commitment.spi(), &g.id);

                if i < cfg.threshold as usize {
                    shares.push(completed_share(&g, &x)?);
                }
            }

            let reconstructed = shares.combine().map_err(anyhow::Error::msg)?.0;
            assert!(
                commitment.verify_reconstructed(&reconstructed),
                "polynomial {} did not open its commitment",
                commitment.spi().poly_index()
            );
        }

        Ok(())
    }

    #[test]
    fn ssa_generator_corresponds_to_standard_recoverer() -> anyhow::Result<()> {
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa: 10,
            threshold: 10,
            surplus_shares: 2,
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);

        let p = SimplePseudonym::random();
        let c = generator.new_ssa_commitment(&p, 1.try_into()?)?;
        let orig_commitment = c.ssa_commitment;

        let mut recovered_secret = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::default();
        for _ in 0..cfg.polynomials_per_ssa {
            let mut shares = Vec::new();
            for _ in 0..(cfg.threshold as usize + cfg.surplus_shares) {
                let x = hopr_types::crypto_random::random_bytes::<10>();
                let g = generator
                    .next_share(&p, &x)?
                    .ok_or(anyhow::anyhow!("failed to generate share"))?;
                shares.push(completed_share(&g, &x)?);
            }
            recovered_secret += shares.combine().map_err(anyhow::Error::msg)?.0;
        }

        assert_eq!(
            orig_commitment.to_affine(),
            (crypto_traits::elliptic_curve::ProjectivePoint::<Secp256k1>::GENERATOR * recovered_secret).to_affine()
        );

        Ok(())
    }

    /// Turns a generated share plus the nonce it was derived from into the `(x, y)` pair the
    /// interpolation consumes, exactly as the reconstructor does.
    fn completed_share(
        g: &GeneratedShare<TestSpec>,
        x: &impl AsRef<[u8]>,
    ) -> anyhow::Result<crate::CompletedShare<TestSpec>> {
        Ok(DefaultShare {
            identifier: TestSpec::msg_to_scalar(&g.id, x)?.into(),
            value: Option::from(crypto_traits::elliptic_curve::Scalar::<Secp256k1>::from_repr(g.share.0))
                .map(|s: PixScalar<TestSpec>| s.into())
                .ok_or(anyhow::anyhow!("share is not a field element"))?,
        })
    }
}
