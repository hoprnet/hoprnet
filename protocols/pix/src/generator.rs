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
    /// Position within the emission window, i.e. into the first
    /// [`SHARE_EMISSION_WINDOW`] entries of `poly_queue`.
    cursor: usize,
    /// Highest SSA index a share has actually been emitted for, `None` until the first one goes out.
    ///
    /// Monotone. Equal to the index of the cycle at the front of `poly_queue` while that cycle is
    /// being served, since the window no longer straddles cycle boundaries — see `front_run`. Kept as
    /// its own field rather than derived, because it must survive the front cycle being drained and
    /// removed. Read by [`SsaShareGenerator::emission_progress`].
    highest_emitted: Option<SsaIndex>,
    /// Polynomials of the *front* cycle still in `poly_queue`, or `0` when that cycle is drained and
    /// the next one has yet to be picked up.
    ///
    /// This is what confines the emission window to a single cycle. Refreshed lazily in
    /// [`EntryShareGenerator::next_share`] rather than at commit time: removals only ever touch the
    /// front cycle, so when it hits `0` the new front cycle is necessarily untouched and its run is a
    /// full `polynomials_per_ssa` — which holds whether or not further cycles were committed in the
    /// meantime.
    front_run: usize,
}

/// Number of polynomials the generator emits shares for concurrently.
///
/// Shares are emitted round-robin across the first `SHARE_EMISSION_WINDOW` polynomials of the
/// queue rather than draining one polynomial to exhaustion before starting the next. Both orderings
/// emit exactly the same shares — every share carries its own [`SsaPolynomialId`], which the Exit
/// files by, so arrival order is irrelevant to reconstruction — but they fail very differently.
///
/// A share only reaches the reconstructor when the Exit *uses* the SURB carrying it, so a SURB
/// dropped from the Exit's per-pseudonym ring buffer is a permanently lost share. That buffer
/// overwrites its oldest entries, which is a *contiguous* run of the emission order. Draining one
/// polynomial at a time makes such a run land on a single polynomial: lose more than
/// `surplus_shares` of it and it can never reach `threshold`, and since the SSA is the sum of
/// *every* polynomial's constant term, the whole cycle becomes unrecoverable — silently, because a
/// starved polynomial never fails a check, it simply never completes. Round-robin spreads the same
/// run across the window, so a contiguous loss of up to `surplus_shares × SHARE_EMISSION_WINDOW`
/// shares is absorbed by the surplus that exists for exactly this purpose.
///
/// The window is bounded rather than spanning the whole SSA because the Exit holds a part builder's
/// collected shares until that part reconstructs (`release_verification_state`). One polynomial at
/// a time keeps one part live; the full 8192 would keep every part live at once, `polys × threshold`
/// shares of peak memory. 256 keeps that peak around a megabyte while covering a contiguous loss
/// far larger than the ring buffer's entire overshoot allowance.
///
/// ## The window never crosses a cycle boundary
///
/// It is clamped to the polynomials of the cycle at the front of the queue, so no share for cycle
/// `k + 1` is ever emitted while any polynomial of cycle `k` is unexhausted. That is what lets an
/// Exit treat progress on a later cycle as misbehaviour rather than as a boundary artefact: a batch
/// of `n` cycles is served as `n` separate quotas, so the Exit is exposed to one cycle's worth of
/// unpaid traffic at a time instead of `n`. An Entry free to spread shares across a whole batch could
/// take `n × quota` of service while leaving every cycle short of recovery — and since an SSA is the
/// sum of *every* polynomial's constant term, a cycle short of recovery is worth nothing at all.
///
/// The clamp costs no loss tolerance while `polynomials_per_ssa` is a multiple of the window, which
/// the deployed 8192 is: polynomials in the window advance in lockstep, so a cycle drains as
/// `polys / window` full-width blocks and the last one is swept out inside a single `next_share`
/// call. A cycle whose polynomial count is *not* a multiple runs its final partial block at that
/// remainder's width, absorbing a proportionally shorter contiguous loss.
pub const SHARE_EMISSION_WINDOW: usize = 256;

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
    /// [`SsaPartCommitment`].
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

    /// How far share emission has got for `pseudonym`, as
    /// `(highest SSA index a share was emitted for, highest SSA index committed to)`.
    ///
    /// `None` when nothing has been committed for the pseudonym yet; the first element is `None`
    /// until the first share goes out. Both are monotone for the life of the cache entry.
    ///
    /// Exists so an Entry can refuse to commit to another batch while emission has not yet reached
    /// the last cycle of the batch it is already serving — an Exit asking that early is asking for
    /// deposits it cannot have earned yet. Emission rather than reconstruction because the Entry never
    /// learns what the Exit actually recovered.
    ///
    /// Because the window is clamped to one cycle (see [`SHARE_EMISSION_WINDOW`]), the first element
    /// reaching the second means every earlier cycle is exhausted — exactly, at every dimension,
    /// rather than approximately near a boundary. What it does *not* say is how much of that last
    /// cycle is left, so the signal is "the Exit could legitimately be near the end of the batch",
    /// not "it is".
    pub fn emission_progress(&self, pseudonym: &S::Pseudonym) -> Option<(Option<SsaIndex>, SsaIndex)> {
        let entry = self.polynomials.get(pseudonym)?;
        let entry = entry.lock();
        Some((entry.highest_emitted, entry.ssa_index))
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
        let Some(entry) = self.polynomials.get(pseudonym) else {
            return Ok(None);
        };

        // If we replaced VecDeque with a lock-free alternative, we could remove the mutex, but the
        // alternative would need to effectively deallocate, so the polynomials do not grow
        // indefinitely when new commitments are being added.
        let mut entry = entry.lock();
        let SsaPseudonymEntry {
            poly_queue,
            cursor,
            highest_emitted,
            front_run,
            ..
        } = &mut *entry;
        let max_shares_per_poly = self.cfg.threshold as usize + self.cfg.surplus_shares;

        while !poly_queue.is_empty() {
            // The window is always the front of the queue: `new_ssa_commitment` appends, and an
            // exhausted polynomial is removed in place so its immediate successor shifts in.
            //
            // It never straddles a cycle boundary, though — see `SHARE_EMISSION_WINDOW` for why the
            // Exit's exposure depends on that. `front_run` counts what is left of the front cycle,
            // and the window cannot reach past it, so the next cycle's first share waits until this
            // cycle's last polynomial is exhausted.
            //
            // The refresh happens here rather than at commit time because removals only ever touch
            // the front cycle: reaching `0` therefore means the new front cycle is untouched and
            // still has all `polynomials_per_ssa` of its polynomials queued.
            if *front_run == 0 {
                *front_run = (self.cfg.polynomials_per_ssa as usize).min(poly_queue.len());
                *cursor = 0;
            }

            let window = (*front_run).min(SHARE_EMISSION_WINDOW).min(poly_queue.len()).max(1);
            if *cursor >= window {
                *cursor = 0;
            }
            let idx = *cursor;

            if poly_queue[idx].shares_generated >= max_shares_per_poly {
                // O(window) element moves, but paid once per polynomial rather than once per share.
                poly_queue.remove(idx);
                *front_run = front_run.saturating_sub(1);
                continue;
            }

            let poly = &mut poly_queue[idx];
            let x = S::msg_to_scalar(&poly.spi, msg)?;
            // Zero would disclose the secret, so we disallow it.
            // The chance is practically impossible.
            if x.is_zero().into() {
                return Err(errors::PixError::InvalidInput);
            }

            let generated = GeneratedShare {
                id: poly.spi,
                share: poly.next_share(x),
            };
            let emitted_for = poly.spi.as_ref().ssa_index();
            if highest_emitted.is_none_or(|seen| emitted_for > seen) {
                *highest_emitted = Some(emitted_for);
            }
            *cursor = (idx + 1) % window;
            return Ok(Some(generated));
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
                            cursor: 0,
                            highest_emitted: None,
                            // Picked up lazily by `next_share`, like every later cycle's.
                            front_run: 0,
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

    /// No share for a cycle may be emitted while any polynomial of an earlier cycle is unexhausted.
    ///
    /// The dimensions here are the adversarial ones for the clamp: `polynomials_per_ssa` far below
    /// [`SHARE_EMISSION_WINDOW`], so an unclamped positional window would span the *whole* batch and
    /// serve all three cycles in lockstep from the first share — leaving the Exit `3 × quota` of
    /// traffic short of recovering any single one of them.
    ///
    /// Also pins that the clamp did not degrade to one polynomial at a time: within a cycle the
    /// emission is still round-robin, which is what spreads a contiguous SURB loss across the window
    /// instead of starving one polynomial.
    #[test]
    fn emission_never_crosses_a_cycle_boundary_early() -> anyhow::Result<()> {
        const POLYS: u16 = 4;
        const THRESHOLD: u16 = 2;
        const SURPLUS: usize = 1;
        const BATCH: u32 = 3;
        let per_poly = THRESHOLD as usize + SURPLUS;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: SURPLUS,
        });

        let p = SimplePseudonym::random();
        for idx in 1..=BATCH {
            generator.new_ssa_commitment(&p, idx.try_into()?)?;
        }
        assert!(
            POLYS as usize * BATCH as usize <= SHARE_EMISSION_WINDOW,
            "the whole batch must fit one window, or this test is not exercising the clamp"
        );

        // Drain everything, recording which cycle each share belonged to, in order.
        let mut emitted: Vec<(u32, u16)> = Vec::new();
        for msg in 0..(POLYS as u32 * BATCH * per_poly as u32) {
            let share = generator
                .next_share(&p, &msg.to_be_bytes())?
                .ok_or_else(|| anyhow::anyhow!("share {msg} must be available"))?;
            emitted.push((share.id.ssa_index().get(), share.id.poly_index()));
        }
        assert!(
            generator.next_share(&p, &u32::MAX.to_be_bytes())?.is_none(),
            "the batch must be exactly spent"
        );

        let cycles: Vec<u32> = emitted.iter().map(|(ssa, _)| *ssa).collect();
        assert!(
            cycles.windows(2).all(|w| w[0] <= w[1]),
            "cycle indices must never go backwards, got {cycles:?}"
        );
        for cycle in 1..=BATCH {
            assert_eq!(
                cycles.iter().filter(|c| **c == cycle).count(),
                POLYS as usize * per_poly,
                "cycle {cycle} must emit its whole share supply"
            );
        }

        // Round-robin within a cycle: the first `POLYS` shares of a cycle cover every polynomial once.
        let first_cycle: Vec<u16> = emitted
            .iter()
            .filter(|(ssa, _)| *ssa == 1)
            .take(POLYS as usize)
            .map(|(_, poly)| *poly)
            .collect();
        let mut distinct = first_cycle.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            POLYS as usize,
            "the first pass must touch every polynomial once, not drain one at a time — got {first_cycle:?}"
        );

        Ok(())
    }

    /// `emission_progress` must lag the commitment index across a batch, and catch up in order.
    ///
    /// `polynomials_per_ssa` is deliberately above [`SHARE_EMISSION_WINDOW`], which is the deployed
    /// shape: the window then sits entirely inside one cycle, so emission reaches a cycle only once
    /// its predecessors are drained. That ordering is what an Entry relies on to tell "the Exit is
    /// asking for the next batch on time" from "the Exit is asking a whole batch early".
    #[test]
    fn emission_progress_lags_the_commitment_index_across_a_batch() -> anyhow::Result<()> {
        const POLYS: u16 = SHARE_EMISSION_WINDOW as u16 + 44;
        const THRESHOLD: u16 = 2;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });

        let p = SimplePseudonym::random();
        assert!(
            generator.emission_progress(&p).is_none(),
            "an unknown pseudonym has no progress to report"
        );

        // One batch of three, as an Exit with `ssas_per_request = 3` would ask for.
        for idx in 1..=3u32 {
            generator.new_ssa_commitment(&p, idx.try_into()?)?;
        }
        assert_eq!(
            generator.emission_progress(&p),
            Some((None, 3.try_into()?)),
            "the whole batch is committed, but nothing has been emitted yet"
        );

        let mut sent = 0u32;
        let emit = |n: u32, sent: &mut u32| -> anyhow::Result<()> {
            for _ in 0..n {
                *sent += 1;
                generator.next_share(&p, &sent.to_be_bytes())?;
            }
            Ok(())
        };

        emit(10, &mut sent)?;
        assert_eq!(
            generator.emission_progress(&p),
            Some((Some(1.try_into()?), 3.try_into()?)),
            "emission is still inside the first cycle of the batch"
        );

        // Drain the first cycle: every polynomial emits exactly `threshold` shares at zero surplus.
        emit(POLYS as u32 * THRESHOLD as u32, &mut sent)?;
        assert_eq!(
            generator.emission_progress(&p),
            Some((Some(2.try_into()?), 3.try_into()?)),
            "emission moves to the next cycle only once its predecessor is drained"
        );

        emit(POLYS as u32 * THRESHOLD as u32, &mut sent)?;
        assert_eq!(
            generator.emission_progress(&p),
            Some((Some(3.try_into()?), 3.try_into()?)),
            "emission has reached the last cycle of the batch — the point an Exit may ask for the next"
        );

        Ok(())
    }

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

    /// With fewer polynomials than [`SHARE_EMISSION_WINDOW`] the whole SSA is one window, so
    /// emission cycles through every polynomial before returning to any of them.
    #[test]
    fn ssa_generator_should_round_robin_within_the_emission_window() -> anyhow::Result<()> {
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
            assert_eq!(g.id.poly_index(), i % 3);
        }
        assert!(generator.next_share(&p1, &20_u32.to_be_bytes())?.is_none());

        // A new cycle is appended, and the window does not reach into it until the previous one is
        // fully emitted — so indices restart from the beginning of the new SSA.
        generator.new_ssa_commitment(&p1, 2.try_into()?)?;

        for i in 0..12_u16 {
            let g = generator
                .next_share(&p1, &i.to_be_bytes())?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            assert_eq!(g.id.pseudonym(), &p1);
            assert_eq!(g.id.ssa_index(), 2.try_into()?);
            assert_eq!(g.id.poly_index(), i % 3);
        }
        assert!(generator.next_share(&p1, &20_u32.to_be_bytes())?.is_none());

        Ok(())
    }

    /// The property that actually protects an SSA cycle from SURB ring-buffer eviction.
    ///
    /// Evictions take a *contiguous* run of the emission order, and a polynomial dies if it loses
    /// more than `surplus_shares`. Spreading every run across the window is what keeps a burst from
    /// concentrating on one polynomial — with more polynomials than the window, any run of `n`
    /// shares must touch at least `min(n, window)` distinct ones.
    #[test]
    fn contiguous_runs_must_spread_across_the_emission_window() -> anyhow::Result<()> {
        // Deliberately more polynomials than the window, so the window is the binding constraint.
        let polynomials_per_ssa = (SHARE_EMISSION_WINDOW * 2) as u16;
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa,
            threshold: 2,
            surplus_shares: 1,
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);

        let p = SimplePseudonym::random();
        generator.new_ssa_commitment(&p, 1.try_into()?)?;

        let mut emitted = Vec::new();
        for i in 0..(polynomials_per_ssa as usize * 3) {
            let g = generator
                .next_share(&p, &(i as u32).to_be_bytes())?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            emitted.push(g.id.poly_index());
        }
        assert!(generator.next_share(&p, &u32::MAX.to_be_bytes())?.is_none());

        for run in [2_usize, 17, SHARE_EMISSION_WINDOW] {
            for window_start in (0..emitted.len().saturating_sub(run)).step_by(run.max(1)) {
                let distinct = emitted[window_start..window_start + run]
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len();
                assert_eq!(
                    run, distinct,
                    "a contiguous run of {run} shares starting at {window_start} hit only {distinct} distinct \
                     polynomials; an eviction of that run would concentrate on too few of them"
                );
            }
        }

        // Every polynomial still receives exactly `threshold + surplus` shares.
        let mut per_poly = std::collections::HashMap::new();
        for poly_index in &emitted {
            *per_poly.entry(*poly_index).or_insert(0_usize) += 1;
        }
        assert_eq!(polynomials_per_ssa as usize, per_poly.len());
        assert!(
            per_poly
                .values()
                .all(|n| *n == cfg.threshold as usize + cfg.surplus_shares)
        );

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

        // Shares are emitted round-robin across the window, so they are grouped by polynomial here
        // rather than assumed to arrive in consecutive runs.
        let by_poly = drain_shares_by_polynomial(&generator, &p, &cfg)?;
        assert_eq!(cfg.polynomials_per_ssa as usize, by_poly.len());

        for commitment in &commitments {
            let shares = by_poly
                .get(&commitment.spi().poly_index())
                .ok_or(anyhow::anyhow!("no shares for polynomial"))?;
            assert_eq!(cfg.threshold as usize + cfg.surplus_shares, shares.len());

            // Only `threshold` shares are needed; the surplus stands in for any that are lost.
            let reconstructed = shares[..cfg.threshold as usize]
                .to_vec()
                .combine()
                .map_err(anyhow::Error::msg)?
                .0;
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

        let by_poly = drain_shares_by_polynomial(&generator, &p, &cfg)?;
        assert_eq!(cfg.polynomials_per_ssa as usize, by_poly.len());

        let mut recovered_secret = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::default();
        for shares in by_poly.values() {
            recovered_secret += shares[..cfg.threshold as usize]
                .to_vec()
                .combine()
                .map_err(anyhow::Error::msg)?
                .0;
        }

        assert_eq!(
            orig_commitment.to_affine(),
            (crypto_traits::elliptic_curve::ProjectivePoint::<Secp256k1>::GENERATOR * recovered_secret).to_affine()
        );

        Ok(())
    }

    /// Turns a generated share plus the nonce it was derived from into the `(x, y)` pair the
    /// interpolation consumes, exactly as the reconstructor does.
    /// Drains the generator and buckets every share by its polynomial index.
    ///
    /// Emission is round-robin across [`SHARE_EMISSION_WINDOW`], so a polynomial's shares are not
    /// contiguous in the output stream; anything reconstructing a polynomial has to group first.
    /// Within a bucket the order is preserved, which is what lets a caller take the first
    /// `threshold` and treat the rest as surplus.
    #[allow(clippy::type_complexity)]
    fn drain_shares_by_polynomial(
        generator: &SsaShareGenerator<TestSpec>,
        p: &SimplePseudonym,
        cfg: &SsaGeneratorConfig,
    ) -> anyhow::Result<std::collections::BTreeMap<PolynomialIndex, Vec<crate::CompletedShare<TestSpec>>>> {
        let expected = cfg.polynomials_per_ssa as usize * (cfg.threshold as usize + cfg.surplus_shares);
        let mut by_poly: std::collections::BTreeMap<PolynomialIndex, Vec<_>> = std::collections::BTreeMap::new();

        for _ in 0..expected {
            let x = hopr_types::crypto_random::random_bytes::<10>();
            let g = generator
                .next_share(p, &x)?
                .ok_or(anyhow::anyhow!("failed to generate share"))?;
            by_poly
                .entry(g.id.poly_index())
                .or_default()
                .push(completed_share(&g, &x)?);
        }
        // The generator must be exhausted after exactly the expected number of shares.
        anyhow::ensure!(
            generator.next_share(p, &u32::MAX.to_be_bytes())?.is_none(),
            "generator emitted more shares than the configured dimensions allow"
        );

        Ok(by_poly)
    }

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
