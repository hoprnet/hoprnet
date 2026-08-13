use std::collections::VecDeque;

#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;
use validator::Validate;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Polynomial,
    elliptic_curve::{Field, Group, PrimeField, group::GroupEncoding, rand_core::CryptoRng},
};

use crate::{
    CONSTANT_TERM_COEFFICIENT, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, DEFAULT_SURPLUS_SHARES,
    MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, MIN_POLY_THRESHOLD, PixGroup, PixParams, PixScalar, PixSpec,
    PolynomialIndex, SsaPartCommitment, errors,
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
    /// Shares emitted so far for the cycle currently at the front of `poly_queue`.
    ///
    /// Reset in lockstep with `front_run`, so it measures the front cycle only. Where
    /// `highest_emitted` says *which* cycle is being served, this says *how far into it* — and the
    /// difference is the whole admission decision: `highest_emitted` reaches the last committed index
    /// on that cycle's very first share, which is 0 % of the way through the batch, not the ~85 % at
    /// which a conforming Exit asks for the next one.
    front_emitted: u64,
}

/// How far share emission has progressed for one pseudonym.
///
/// Returned by [`SsaShareGenerator::emission_progress`], and the input to the Entry's half of the
/// successor gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmissionProgress {
    /// Highest SSA index a share has been emitted for, `None` until the first one goes out.
    pub highest_emitted: Option<SsaIndex>,
    /// Highest SSA index committed to.
    pub last_committed: SsaIndex,
    /// Shares emitted for the cycle at the front of the queue.
    pub front_emitted: u64,
    /// Shares a whole cycle emits, i.e. `polynomials_per_ssa × (threshold + surplus)`.
    ///
    /// Never zero: `polynomials_per_ssa` is at least 1 and the threshold at least
    /// [`MIN_POLY_THRESHOLD`].
    pub shares_per_cycle: u64,
}

impl EmissionProgress {
    /// `true` when emission has reached the last cycle this pseudonym is committed to.
    ///
    /// Necessary for admitting a successor batch, and nowhere near sufficient — it becomes true on
    /// that cycle's first share. See [`front_fraction`](Self::front_fraction).
    #[inline]
    pub fn is_serving_last_committed(&self) -> bool {
        self.highest_emitted == Some(self.last_committed)
    }

    /// Fraction of the front cycle's shares that have been emitted, in `0.0..=1.0`.
    #[inline]
    pub fn front_fraction(&self) -> f64 {
        if self.shares_per_cycle == 0 {
            return 0.0;
        }
        (self.front_emitted as f64 / self.shares_per_cycle as f64).min(1.0)
    }
}

/// Fewest shares that can be emitted for a cycle before an Exit's early-recovery signal can fire.
///
/// An Entry uses this to tell a legitimate request for the next batch from one that arrives before
/// the current batch has been earned. Nothing below the returned count can have produced an honest
/// [`SsaReconstructorConfig::early_recovery_threshold`](crate::SsaReconstructorConfig::early_recovery_threshold)
/// signal, at any dimensions, on a lossless link — and loss only pushes the real signal later.
///
/// ## Why it is not `early_threshold × shares_per_cycle`
///
/// Two independent reasons, and they pull the answer *up*, so the naive estimate is unsafe rather
/// than merely imprecise.
///
/// The Exit's threshold counts **reconstructed polynomials**, not shares — `check_early_threshold`
/// tests `received_indices.len()` against `ceil(threshold × num_polys)`. A polynomial reconstructs on
/// its `threshold`-th useful share and its surplus is emitted afterwards, so shares and polynomials
/// do not advance in proportion.
///
/// And emission is **windowed**: [`SHARE_EMISSION_WINDOW`] polynomials advance in lockstep, and a
/// window emits its entire surplus before the next window starts. So every window before the one
/// holding the boundary has already spent `threshold + surplus` per polynomial, not `threshold`.
///
/// At the deployed 8192 × 64 (+16) that is 27 whole windows — 552 960 shares — plus 63 full passes
/// and 52 shares of the 64th in the 28th window, i.e. **86.8 % of the cycle**. Dividing the Exit's
/// 0.85 by the 1.25× surplus factor gives 68 %, and admitting there would hand out the next deposit
/// roughly 122 MiB of payload before it could possibly have been earned.
///
/// ## Which threshold to pass
///
/// [`MIN_EARLY_RECOVERY_THRESHOLD`](crate::MIN_EARLY_RECOVERY_THRESHOLD), not the caller's own
/// `early_recovery_threshold`. The value that decides when the request actually goes out belongs to
/// the *peer* and does not travel on the wire, and the direction of a mismatch is unforgiving: a peer
/// configured lower asks earlier than a gate built on the local value admits, and its one-shot
/// request is dropped with no retry path. Computing the gate at the protocol floor admits every
/// conforming peer, and the floor is what makes "conforming" checkable locally.
pub fn min_emission_for_early_recovery(params: &PixParams, early_threshold: f64) -> u64 {
    let polys = params.polys_per_ssa() as u64;
    let threshold = params.shares_per_poly() as u64;
    let emitted_per_poly = params.emitted_shares_per_poly() as u64;
    // Clamped because a fraction outside `0..=1` would make the arithmetic below either underflow or
    // exceed the cycle — and a non-finite one fails *closed*, to the whole cycle, rather than being
    // clamped. `f64::clamp` propagates `NaN`, `NaN.ceil()` is `NaN`, and casting that to an integer
    // saturates to zero, so the one input that should be refused outright would instead return a
    // boundary of zero shares and open the gate completely. This function is only ever called to
    // decide whether a deposit has been earned; when its input is meaningless the answer is "no".
    let early_threshold = if early_threshold.is_finite() {
        early_threshold.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let needed = (early_threshold * polys as f64).ceil() as u64;
    if needed == 0 {
        return 0;
    }

    let window = (SHARE_EMISSION_WINDOW as u64).min(polys);
    // `needed - 1` so that a boundary landing exactly on a window edge is attributed to the window
    // that completes it rather than to the next one.
    let prior_windows = (needed - 1) / window;
    let prior_polys = prior_windows * window;
    // The last window may be narrower than `SHARE_EMISSION_WINDOW` when `polys` is not a multiple of
    // it; a narrower window reaches the same pass in fewer shares.
    let current_width = window.min(polys - prior_polys);
    let in_window = needed - prior_polys;

    // Whole windows are exhausted, surplus included. Inside the current one, the `needed`-th
    // polynomial completes partway through the `threshold`-th pass — after `threshold - 1` full
    // passes plus `in_window` shares of that pass. That is the earliest instant, not the average.
    prior_polys * emitted_per_poly + (threshold - 1) * current_width + in_window
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

/// Rejects a surplus larger than the threshold it insures.
///
/// The bound is deliberately loose — twice the emitted shares a polynomial needs — because the
/// surplus is legitimately a deployment choice about return-path loss, and over-insuring a bad path
/// is a reasonable thing to want. What it forbids is the case where the insurance costs more than
/// the thing insured: since H5 the surplus travels in the negotiated
/// [`PixParams`](crate::PixParams) and is billed on purchase rather than on claim, so a surplus
/// above the threshold means an Entry paying for more redundancy than payload in every deposit.
///
/// This is a *configuration* bound, not a wire one. `PixParams` packs the surplus as a byte and
/// accepts the whole range, and a peer offering an extravagant surplus is already caught where it
/// should be — by the Exit's `quota_range`, since the surplus inflates the quota.
fn surplus_must_not_exceed_threshold(cfg: &SsaGeneratorConfig) -> Result<(), validator::ValidationError> {
    if cfg.surplus_shares > cfg.threshold {
        return Err(validator::ValidationError::new(
            "surplus_shares must not exceed threshold — the surplus is billed, so this pays for more redundancy than \
             payload",
        ));
    }
    Ok(())
}

/// Configuration for the [`SsaShareGenerator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[validate(schema(function = "surplus_must_not_exceed_threshold", skip_on_field_errors = false))]
pub struct SsaGeneratorConfig {
    /// The number of polynomials to generate per SSA commitment.
    ///
    /// Default is [`DEFAULT_POLYS_PER_SSA`], must be between 1 and [`MAX_POLYS_PER_SSA`].
    #[default(DEFAULT_POLYS_PER_SSA)]
    #[validate(range(min = 1, max = MAX_POLYS_PER_SSA))]
    pub polynomials_per_ssa: u16,
    /// Minimum number of shares required to reconstruct each SSA polynomial.
    ///
    /// Default is [`DEFAULT_POLY_THRESHOLD`], must be at least [`MIN_POLY_THRESHOLD`]. The upper
    /// bound [`MAX_POLY_THRESHOLD`] is the width of the field.
    #[default(DEFAULT_POLY_THRESHOLD)]
    #[validate(range(min = MIN_POLY_THRESHOLD, max = MAX_POLY_THRESHOLD))]
    pub threshold: u8,
    /// Additional number of shares to generate beyond the threshold for redundancy.
    ///
    /// Covers *lost* shares only: the Exit reconstructs from the first `threshold` distinct shares
    /// that reach it, so any of the surplus can stand in for one that never arrives. It does not
    /// cover *corrupt* shares — nothing checks a share on arrival any more, so a bad one is only
    /// noticed once it has already poisoned the interpolation. See
    /// [`SsaPartCommitment`].
    ///
    /// Emitting them is unconditional: a polynomial leaves the queue at `threshold + surplus`
    /// shares, whether or not any were lost, so the Exit serves this many packets per polynomial in
    /// every case. That is why the surplus is part of the per-SSA quota rather than free service —
    /// the Entry is buying insurance, and insurance is paid for whether or not it is claimed.
    ///
    /// Default is [`DEFAULT_SURPLUS_SHARES`] — but prefer [`default_surplus_for`] wherever the
    /// threshold is known, because this is a *ratio* of it and the constant can only be the ratio
    /// evaluated at the default threshold.
    ///
    /// Bounded by `threshold` rather than by the byte the wire gives it: see
    /// [`surplus_must_not_exceed_threshold`]. It shares the lower half of the negotiated
    /// [`PixParams`](crate::PixParams) word with `threshold`, so a byte is all that fits there — but
    /// what is *representable* and what is sane to configure are different questions, and this field
    /// used to be documented as needing no validator on the strength of the first.
    #[default(DEFAULT_SURPLUS_SHARES)]
    pub surplus_shares: u8,
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
    /// Fails if the configuration does not validate. Prefer this over [`Self::new`] anywhere the
    /// configuration is assembled at runtime — a config built programmatically or read from a file
    /// is input, not a constant, and turning it into a panic makes it un-handleable by the caller.
    pub fn try_new(cfg: SsaGeneratorConfig) -> errors::Result<Self, S::Pseudonym> {
        cfg.validate()?;
        Ok(Self {
            polynomials: moka::sync::CacheBuilder::default()
                .initial_capacity(100_000)
                .time_to_idle(std::time::Duration::from_secs(1800))
                .build_with_hasher(ahash::RandomState::new()),
            cfg,
        })
    }

    /// Creates a new share generator with the provided configuration.
    ///
    /// # Panics
    /// Panics if the configuration fails validation. Use [`Self::try_new`] to handle that case
    /// instead.
    pub fn new(cfg: SsaGeneratorConfig) -> Self {
        Self::try_new(cfg).expect("invalid SsaGeneratorConfig")
    }

    /// Returns the configuration used to generate this [`SsaShareGenerator`].
    #[inline]
    pub fn config(&self) -> &SsaGeneratorConfig {
        &self.cfg
    }

    /// Discards all polynomial state held for `pseudonym`.
    ///
    /// The same observable state the cache's own idle retention produces, reached deliberately: no
    /// further share can be emitted for any cycle of this pseudonym, and
    /// [`emission_progress`](Self::emission_progress) goes back to `None`.
    ///
    /// The distinction that state has been *lost* rather than never created is not recoverable from
    /// here — an evicted entry leaves nothing behind — so a caller that needs it has to hold the fact
    /// itself, for as long as it needs it to mean something. `hopr-transport-session` keeps it per
    /// Session, because a successor `SsaRequest` arriving against lost state is indistinguishable from
    /// the opening one otherwise, and the two must not be answered alike.
    pub fn forget(&self, pseudonym: &S::Pseudonym) {
        self.polynomials.invalidate(pseudonym);
    }

    /// How far share emission has got for `pseudonym`.
    ///
    /// `None` when nothing has been committed for the pseudonym yet.
    ///
    /// Exists so an Entry can refuse to commit to another batch while emission has not yet reached
    /// the *end* of the batch it is already serving — an Exit asking earlier is asking for deposits
    /// it cannot have earned yet. Emission rather than reconstruction, because the Entry never learns
    /// what the Exit actually recovered.
    ///
    /// Because the window is clamped to one cycle (see [`SHARE_EMISSION_WINDOW`]),
    /// [`is_serving_last_committed`](EmissionProgress::is_serving_last_committed) means every earlier
    /// cycle is exhausted — exactly, at every dimension, rather than approximately near a boundary.
    /// It becomes true on the last cycle's *first* share, though, so it answers "which cycle" and not
    /// "how far in"; [`front_fraction`](EmissionProgress::front_fraction) is the second half, and an
    /// admission rule needs both.
    pub fn emission_progress(&self, pseudonym: &S::Pseudonym) -> Option<EmissionProgress> {
        let entry = self.polynomials.get(pseudonym)?;
        let entry = entry.lock();
        Some(EmissionProgress {
            highest_emitted: entry.highest_emitted,
            last_committed: entry.ssa_index,
            front_emitted: entry.front_emitted,
            shares_per_cycle: self.cfg.polynomials_per_ssa as u64
                * (self.cfg.threshold as u64 + self.cfg.surplus_shares as u64),
        })
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
            front_emitted,
            ..
        } = &mut *entry;
        let max_shares_per_poly = self.cfg.threshold as usize + self.cfg.surplus_shares as usize;

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
                // A new cycle takes the front, so the emission counter restarts with it. This is the
                // only place it is cleared, which is what keeps it measuring the front cycle alone.
                *front_emitted = 0;
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
            *front_emitted += 1;
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
        // Monotonicity is checked here, ahead of the generation below, and again under the cache's
        // own lock at the end. Doing it twice is not redundant: this one is the cheap rejection, and
        // the one below is the authoritative one, because the entry can be advanced by a concurrent
        // caller in between.
        //
        // Without it, every racing request pays for `polynomials_per_ssa` polynomials — hundreds of
        // thousands of EC operations at the deployed dimensions, a second or more of CPU — and then
        // all but one throw the result away. Since a stale or malicious `SsaRequest` is exactly the
        // shape that loses that race, the wasted work was reachable from a single inbound packet.
        if let Some(entry) = self.polynomials.get(pseudonym)
            && ssa_index <= entry.lock().ssa_index
        {
            return Err(PixError::InvalidInput);
        }

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
                            front_emitted: 0,
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

        // Built from the parameters rather than read back off `commitments[0]`: every element above
        // was constructed with exactly this `SsaId`, and indexing would have made the whole function
        // depend on `polynomials_per_ssa >= 1` holding — which is a validation invariant enforced
        // three call layers away, not something visible here.
        let ssa_id = SsaId::new(*pseudonym, ssa_index);
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
        const THRESHOLD: u8 = 2;
        const SURPLUS: u8 = 1;
        const BATCH: u32 = 3;
        let per_poly = THRESHOLD as usize + SURPLUS as usize;

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
        const THRESHOLD: u8 = 2;

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
        let progress = generator.emission_progress(&p).expect("committed");
        assert_eq!(
            (None, SsaIndex::new(3)),
            (progress.highest_emitted, Some(progress.last_committed)),
            "the whole batch is committed, but nothing has been emitted yet"
        );
        assert_eq!(
            POLYS as u64 * THRESHOLD as u64,
            progress.shares_per_cycle,
            "a cycle's supply is polys x (threshold + surplus), and the surplus is zero here"
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
        let progress = generator.emission_progress(&p).expect("committed");
        assert_eq!(
            Some(1.try_into()?),
            progress.highest_emitted,
            "emission is still inside the first cycle of the batch"
        );
        assert!(
            !progress.is_serving_last_committed(),
            "the first cycle of a batch of three is not the last"
        );
        assert_eq!(10, progress.front_emitted);

        // Drain the first cycle: every polynomial emits exactly `threshold` shares at zero surplus.
        emit(POLYS as u32 * THRESHOLD as u32, &mut sent)?;
        let progress = generator.emission_progress(&p).expect("committed");
        assert_eq!(
            Some(2.try_into()?),
            progress.highest_emitted,
            "emission moves to the next cycle only once its predecessor is drained"
        );

        emit(POLYS as u32 * THRESHOLD as u32, &mut sent)?;
        let progress = generator.emission_progress(&p).expect("committed");
        assert!(
            progress.is_serving_last_committed(),
            "emission has reached the last cycle of the batch"
        );

        // And this is the distinction the whole admission rule turns on. Reaching the last cycle says
        // nothing about how far into it emission has got: `is_serving_last_committed` became true on
        // that cycle's *first* share, ~0 % of the way through, while a conforming Exit does not ask
        // until ~85 %. An Entry admitting a successor batch on the index alone admits it a whole cycle
        // early — which is the same unfunded exposure the batch gate exists to prevent.
        assert!(
            progress.front_fraction() < 0.05,
            "reaching the last cycle must not imply being far into it, got {}",
            progress.front_fraction()
        );

        emit(POLYS as u32 * THRESHOLD as u32 * 9 / 10, &mut sent)?;
        let progress = generator.emission_progress(&p).expect("committed");
        assert!(
            progress.is_serving_last_committed() && progress.front_fraction() >= 0.85,
            "nine tenths into the last cycle is where an Exit may legitimately ask for the next batch, got {}",
            progress.front_fraction()
        );

        Ok(())
    }

    /// The early-recovery boundary must be derived, not estimated from the surplus factor.
    ///
    /// Pins the deployed number, because that is what the Entry's successor gate compares against and
    /// it is not something a reader can check by inspection. The naive
    /// `early_threshold / surplus_factor` gives 68 %; the truth is 86.8 %, and the gap is a whole
    /// batch's worth of deposit exposure.
    #[test]
    fn early_recovery_boundary_is_computed_from_the_emission_windows() -> anyhow::Result<()> {
        let params =
            PixParams::try_new_for::<TestSpec>(DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD, DEFAULT_SURPLUS_SHARES)?;
        let cycle = params.polys_per_ssa() as u64 * params.emitted_shares_per_poly() as u64;
        assert_eq!(655_360, cycle);

        // 27 whole windows (552 960 shares), then 63 full passes of the 28th plus 52 shares of its
        // 64th — the instant the 6964th polynomial reaches its threshold.
        let boundary = min_emission_for_early_recovery(&params, 0.85);
        assert_eq!(552_960 + 63 * 256 + 52, boundary);

        let fraction = boundary as f64 / cycle as f64;
        assert!(
            (0.868..0.869).contains(&fraction),
            "the deployed boundary must be ~86.8% of emission, got {fraction}"
        );
        assert!(
            fraction > 0.85 / 1.25,
            "the naive surplus-factor estimate must be demonstrably below the real boundary"
        );

        Ok(())
    }

    /// The boundary must hold at dimensions that do not divide the emission window evenly, and at
    /// dimensions narrower than one window — where there is no windowing to account for at all.
    #[test]
    fn early_recovery_boundary_handles_partial_and_narrow_windows() -> anyhow::Result<()> {
        // Narrower than one window: a single lockstep block, so the boundary is simply the pass on
        // which the `needed`-th polynomial completes.
        let narrow = PixParams::try_new_for::<TestSpec>(10, 4, 2)?;
        // ceil(0.85 * 10) = 9 polynomials; 3 full passes of 10, then 9 shares of the 4th.
        assert_eq!(3 * 10 + 9, min_emission_for_early_recovery(&narrow, 0.85));

        // Not a multiple of the window: the last window is narrower and reaches a pass sooner.
        let ragged = PixParams::try_new_for::<TestSpec>(SHARE_EMISSION_WINDOW as u16 + 100, 4, 2)?;
        let polys = ragged.polys_per_ssa() as u64;
        let needed = (0.85 * polys as f64).ceil() as u64;
        let boundary = min_emission_for_early_recovery(&ragged, 0.85);
        assert!(
            needed > SHARE_EMISSION_WINDOW as u64,
            "the boundary must fall in the second, narrower window for this case to mean anything"
        );
        assert_eq!(
            SHARE_EMISSION_WINDOW as u64 * 6 + 3 * 100 + (needed - SHARE_EMISSION_WINDOW as u64),
            boundary,
            "one whole window at threshold+surplus, then 3 passes of the 100-wide remainder"
        );

        // Degenerate thresholds are answerable rather than panicking: everything, and nothing.
        assert_eq!(0, min_emission_for_early_recovery(&narrow, 0.0));
        let all = min_emission_for_early_recovery(&narrow, 1.0);
        assert_eq!(
            3 * 10 + 10,
            all,
            "every polynomial completes on the last pass of the threshold"
        );

        Ok(())
    }

    /// A non-finite threshold must close the gate, not open it.
    ///
    /// The clamp this replaced propagated `NaN`, and every step after it quietly agreed: `NaN * polys`
    /// is `NaN`, `NaN.ceil()` is `NaN`, and casting that to an integer saturates to *zero*. The
    /// early-exit for `needed == 0` then returned a boundary of zero shares — so the one input with no
    /// meaning at all produced the single most permissive answer this function can give, on the
    /// calculation the Entry uses to decide whether a deposit has been earned.
    ///
    /// `SsaReconstructorConfig` now refuses a non-finite threshold outright, so this is defence in
    /// depth rather than the only guard. It is worth having because the function is public, takes a
    /// bare `f64`, and every one of the steps above is silent.
    #[test]
    fn a_non_finite_early_recovery_threshold_closes_the_emission_gate() -> anyhow::Result<()> {
        let params =
            PixParams::try_new_for::<TestSpec>(DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD, DEFAULT_SURPLUS_SHARES)?;
        let whole_cycle = min_emission_for_early_recovery(&params, 1.0);

        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                whole_cycle,
                min_emission_for_early_recovery(&params, bad),
                "{bad} must demand the whole cycle, not zero shares"
            );
        }

        Ok(())
    }

    /// A stale index must be refused before any polynomial is generated.
    ///
    /// Generation is the expensive half of `new_ssa_commitment` — `polynomials_per_ssa` polynomials
    /// and their commitments, hundreds of thousands of EC operations at the deployed dimensions —
    /// and it used to run *before* the entry was taken and the index checked. Every request that
    /// lost a race, and every stale or replayed `SsaRequest`, paid it in full and threw the result
    /// away, all reachable from one inbound packet.
    ///
    /// Timing is the observable here, so the assertion is deliberately loose: dimensions large enough
    /// that generation dominates by orders of magnitude, and a bound far above the rejection's real
    /// cost but far below one generation. A tighter bound would be a flaky test rather than a
    /// stricter one.
    #[test]
    fn a_stale_index_is_refused_before_any_polynomial_is_generated() -> anyhow::Result<()> {
        const POLYS: u16 = 512;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: 8,
            surplus_shares: 2,
        });
        let p = SimplePseudonym::random();

        let started = std::time::Instant::now();
        generator.new_ssa_commitment(&p, 2.try_into()?)?;
        let generation = started.elapsed();

        // Every index at or below the last one: a replay, a duplicate, and a reordered predecessor.
        let started = std::time::Instant::now();
        for stale in [1u32, 2] {
            assert!(
                matches!(
                    generator.new_ssa_commitment(&p, stale.try_into()?),
                    Err(PixError::InvalidInput)
                ),
                "index {stale} is not above the last committed and must be refused"
            );
        }
        let rejections = started.elapsed();

        assert!(
            rejections * 4 < generation,
            "two rejections took {rejections:?} against one generation's {generation:?} — the index check must \
             short-circuit ahead of the polynomial generation, not after it"
        );

        // And the refusals left the generator's own state alone: the next legitimate index still works.
        generator.new_ssa_commitment(&p, 3.try_into()?)?;

        Ok(())
    }

    /// The surplus is a ratio of the threshold, and the ratio is a loss-rate tolerance.
    ///
    /// Pinned as the tolerance rather than as four literals, because the tolerance is the property
    /// with a physical meaning — `surplus/(threshold + surplus)` is the fraction of a polynomial's
    /// shares that may be lost before it cannot reconstruct. A change to
    /// `SURPLUS_LOSS_TOLERANCE_DIVISOR` has to restate what it did to that number.
    ///
    /// Swept over the *whole* accepted threshold range rather than the deployed multiples of four.
    /// Restricting it to those was what let the ratio round down unnoticed: every sampled point
    /// divided exactly, so integer division and the intended ratio agreed on all of them and
    /// disagreed on everything else.
    #[test]
    fn the_default_surplus_covers_a_fifth_of_a_polynomial_being_lost() {
        for threshold in MIN_POLY_THRESHOLD..=MAX_POLY_THRESHOLD {
            let surplus = crate::default_surplus_for(threshold);
            let emitted = threshold as f64 + surplus as f64;
            let tolerated = surplus as f64 / emitted;

            // A floor, not an approximation: the surplus may over-cover, never under-cover.
            assert!(
                tolerated >= 0.20,
                "threshold {threshold} + surplus {surplus} tolerates only {tolerated:.4} loss"
            );
            // And it over-covers by less than one share, which is what makes rounding up cheap.
            assert!(
                surplus as f64 - 1.0 < threshold as f64 / crate::SURPLUS_LOSS_TOLERANCE_DIVISOR as f64,
                "threshold {threshold} buys {surplus} surplus shares, more than one above the ratio"
            );
            // Zero surplus is zero loss tolerance. Rounding down produced it at thresholds 2 and 3.
            assert!(surplus > 0, "threshold {threshold} derives no surplus at all");
        }

        // Where the threshold divides exactly the tolerance is the documented 20 % on the nose, and
        // the deployed threshold is one of those.
        for threshold in [16u8, 32, 48, 64] {
            let surplus = crate::default_surplus_for(threshold);
            let tolerated = surplus as f64 / (threshold as f64 + surplus as f64);
            assert!(
                (0.19..=0.21).contains(&tolerated),
                "threshold {threshold} + surplus {surplus} tolerates {tolerated:.3} loss, expected ~0.20"
            );
        }

        assert_eq!(
            DEFAULT_SURPLUS_SHARES,
            crate::default_surplus_for(DEFAULT_POLY_THRESHOLD),
            "the constant must stay the ratio evaluated at the default threshold, not drift from it"
        );
    }

    /// A derived surplus must never fail the validator it is derived under.
    ///
    /// `default_surplus_for` rounds up and `surplus_must_not_exceed_threshold` bounds the surplus by
    /// the threshold, so the two meet at the smallest threshold there is: at
    /// [`MIN_POLY_THRESHOLD`] the derived surplus is 1 against a threshold of 2. Any further
    /// rounding-up would collide with the bound rather than merely over-insure.
    #[test]
    fn every_derived_surplus_passes_the_bound_it_is_derived_under() {
        for threshold in MIN_POLY_THRESHOLD..=MAX_POLY_THRESHOLD {
            let cfg = SsaGeneratorConfig {
                polynomials_per_ssa: 16,
                threshold,
                surplus_shares: crate::default_surplus_for(threshold),
            };
            assert!(
                cfg.validate().is_ok(),
                "the surplus derived for threshold {threshold} fails its own validator"
            );
        }
    }

    /// A surplus above the threshold pays for more redundancy than payload, and is billed for it.
    ///
    /// The boundary rather than an arbitrary over-large value: the rule is exactly "not more than
    /// the thing it insures", so the interesting cases are on either side of it. A flat surplus of
    /// 20 — what deployments used before this became a ratio — is what fails at threshold 16.
    #[test]
    fn a_surplus_larger_than_the_threshold_is_rejected() {
        let at_bound = SsaGeneratorConfig {
            polynomials_per_ssa: 16,
            threshold: 16,
            surplus_shares: 16,
        };
        assert!(
            SsaShareGenerator::<TestSpec>::try_new(at_bound).is_ok(),
            "a surplus equal to the threshold must be allowed — over-insuring a lossy path is a real choice"
        );

        let past_bound = SsaGeneratorConfig {
            surplus_shares: 17,
            ..at_bound
        };
        assert!(matches!(
            SsaShareGenerator::<TestSpec>::try_new(past_bound),
            Err(PixError::InvalidConfiguration(_))
        ));

        let flat_twenty_at_low_threshold = SsaGeneratorConfig {
            surplus_shares: 20,
            ..at_bound
        };
        assert!(
            SsaShareGenerator::<TestSpec>::try_new(flat_twenty_at_low_threshold).is_err(),
            "the configuration this rule exists to catch: 20 shares of insurance against 16 of payload"
        );
    }

    #[test]
    fn ssa_generator_try_new_should_reject_an_invalid_config_without_panicking() {
        // Zero polynomials is the case the rest of the generator quietly relies on being impossible
        // — `new_ssa_commitment` builds its `SsaId` from the parameters precisely so that it does
        // not have to index into an empty commitment vector.
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa: 0,
            threshold: 10,
            surplus_shares: 2,
        };

        assert!(matches!(
            SsaShareGenerator::<TestSpec>::try_new(cfg),
            Err(PixError::InvalidConfiguration(_))
        ));
    }

    #[test]
    #[should_panic(expected = "invalid SsaGeneratorConfig")]
    fn ssa_generator_new_should_still_panic_on_an_invalid_config() {
        // `new` stays panicking on purpose: it is what the benches and tests use, where a bad
        // constant should abort rather than be threaded through a `Result`.
        let _ = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 0,
            threshold: 10,
            surplus_shares: 2,
        });
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
                .all(|n| *n == cfg.threshold as usize + cfg.surplus_shares as usize)
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
            assert_eq!(cfg.threshold as usize + cfg.surplus_shares as usize, shares.len());

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
        let expected = cfg.polynomials_per_ssa as usize * (cfg.threshold as usize + cfg.surplus_shares as usize);
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
