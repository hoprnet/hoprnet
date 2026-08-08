use vsss_rs::{
    ReadableShareSet, Share, ShareElement,
    elliptic_curve::group::{Group, GroupEncoding},
};

use crate::{
    CONSTANT_TERM_COEFFICIENT, CoefficientIndex, CompletedShare, PartialSsaShare, PixGroup, PixGroupRepr, PixParams,
    PixScalar, PixSpec, PolynomialIndex, SsaCommitmentProof, SsaPartCommitment, SsaPolynomialId, SsaRecoveryProgress,
    errors, into_completed_share, types::SsaId,
};

/// Relaxed ordering suffices for every progress counter in [`SsaCycle`].
///
/// The counters are pure telemetry: they are never read to decide whether a share may be applied,
/// and the [supervisor](SsaRecoveryProgress) that consumes them keeps its own monotonic maximum and
/// treats a stale snapshot as benign. So no counter needs to be ordered against the reconstruction
/// state it describes, and a snapshot taken while a concurrent batch is mid-flight is allowed to
/// straddle it.
const PROGRESS_ORDERING: std::sync::atomic::Ordering = std::sync::atomic::Ordering::Relaxed;

/// All post-commitment state of one SSA cycle, held as a single unit.
///
/// ## Why one entry per cycle rather than one per polynomial
///
/// The accumulator and every part builder are produced by the same call — see
/// [`CommitmentProgress`] — needed by the same code path, and dead at the same moment. Splitting
/// them across caches previously required holding three lifetimes in lockstep by hand, and getting
/// that wrong was **H8**: the part builders were keyed per polynomial with an idle timer, so the
/// clock measured "time since a share for *this* polynomial arrived". Commitments land in a cycle's
/// opening moments while shares arrive polynomial-major across all of it, so any polynomial late in
/// the emission order had its builder reclaimed before its first share — unrecoverably, since the
/// commitment cannot be retransmitted.
///
/// Keyed by [`SsaId`], the idle timer measures "time since the *cycle* was active", which is the
/// property that actually matters and is correct at any line rate.
///
/// ## Locking
///
/// One mutex **per part**, plus one for the accumulator — never one around the whole cycle. A
/// single cycle-wide lock would serialise every share of a Session behind one mutex. Callers take
/// the part lock and the accumulator lock in that order, and never hold both.
pub struct SsaCycle<S: PixSpec> {
    id: SsaId<S::Pseudonym>,
    builder: parking_lot::Mutex<SsaBuilder<S>>,
    /// Part builders indexed by [`PolynomialIndex`]; length is always `num_polys`.
    parts: Box<[parking_lot::Mutex<SsaPartBuilder<S>>]>,
    /// Shares that advanced reconstruction: new, distinct, and below their part's threshold when
    /// they arrived. Duplicates, surplus shares and shares absorbed by a failed part are excluded,
    /// so this is the numerator of a progress ratio rather than a received-packet count.
    ///
    /// This is the *payment* counter. For liveness — whether the Entry is feeding this cycle at all —
    /// see `shares_seen`, and do not substitute one for the other.
    useful_shares: std::sync::atomic::AtomicU64,
    /// Shares that reached a part builder and were accepted as this cycle's, useful or not.
    ///
    /// The *liveness* counter, and always at least `useful_shares`. A conforming Entry emits
    /// `threshold + surplus` shares per polynomial, so a whole emission window ends with
    /// `surplus × window` consecutive shares that advance nothing — and a consumer watching
    /// `useful_shares` alone cannot tell that from an Entry that has stopped sending.
    shares_seen: std::sync::atomic::AtomicU64,
    /// Parts whose constant term has been reconstructed and opened its commitment.
    recovered_polynomials: std::sync::atomic::AtomicU32,
    /// Shares that failed verification, aggregated across every peer that relayed for this cycle.
    ///
    /// A failure is charged once per offending share, not once per polynomial: a part reports its
    /// failure exactly once and absorbs everything after it as
    /// [`AddShareOutcome::Absorbed`].
    invalid_shares: std::sync::atomic::AtomicU64,
    /// `num_polys × poly_threshold` — the useful-share count that constitutes full recovery.
    ///
    /// Fixed at construction, and must equal the dimensions negotiated at session establishment:
    /// the consumer treats a mismatch as a protocol violation rather than as drift.
    target_useful_shares: u64,
}

impl<S: PixSpec> SsaCycle<S> {
    /// Assembles a cycle from the accumulator and the full set of part builders.
    ///
    /// `parts` arrives in arbitrary order — it is drained from a `HashMap` — so each builder is
    /// placed by its own polynomial index rather than by iteration order. A missing or duplicated
    /// index is rejected here, which is what makes [`part`](Self::part) safe to index by an
    /// untrusted value later. Each builder is also checked to belong to `id`, so a part cannot be
    /// filed under a cycle it does not describe.
    pub fn new(
        id: SsaId<S::Pseudonym>,
        builder: SsaBuilder<S>,
        parts: Vec<SsaPartBuilder<S>>,
    ) -> errors::Result<Self, S::Pseudonym> {
        let num_polys = builder.num_polys();
        let mut slots: Vec<Option<SsaPartBuilder<S>>> = (0..num_polys).map(|_| None).collect();

        for part in parts {
            let spi = part.spi();
            if spi.as_ref() != &id {
                return Err(errors::PixError::InvalidInput);
            }
            let slot = slots
                .get_mut(spi.poly_index() as usize)
                .ok_or(errors::PixError::InvalidInput)?;
            if slot.replace(part).is_some() {
                return Err(errors::PixError::DuplicateCommitment);
            }
        }

        let parts = slots
            .into_iter()
            .map(|slot| slot.ok_or(errors::PixError::InvalidInput))
            .collect::<errors::Result<Vec<_>, S::Pseudonym>>()?;

        // Every part builder was handed the same `SsaCommitmentBuilder::poly_threshold`, so any of
        // them reports the negotiated threshold. Read it before the builders are wrapped in mutexes,
        // which would otherwise make this a lock acquisition.
        let poly_threshold = parts.first().ok_or(errors::PixError::InvalidInput)?.min_shares() as u64;

        Ok(Self {
            id,
            builder: parking_lot::Mutex::new(builder),
            parts: parts.into_iter().map(parking_lot::Mutex::new).collect(),
            useful_shares: Default::default(),
            shares_seen: Default::default(),
            recovered_polynomials: Default::default(),
            invalid_shares: Default::default(),
            target_useful_shares: num_polys as u64 * poly_threshold,
        })
    }

    /// Number of polynomials this cycle is composed of.
    pub fn num_polys(&self) -> usize {
        self.parts.len()
    }

    /// Records a share that advanced reconstruction — both a useful share and a share seen.
    pub fn record_useful_share(&self) {
        self.useful_shares.fetch_add(1, PROGRESS_ORDERING);
        self.shares_seen.fetch_add(1, PROGRESS_ORDERING);
    }

    /// Records a share that arrived for an already-reconstructed polynomial.
    ///
    /// Liveness only: the surplus a conforming Entry emits advances nothing, but it is still evidence
    /// that the Entry is serving this cycle, which is what the Exit's egress gate and recovery-idle
    /// deadline are asking about.
    pub fn record_surplus_share(&self) {
        self.shares_seen.fetch_add(1, PROGRESS_ORDERING);
    }

    /// Records a polynomial part that reconstructed and opened its commitment.
    pub fn record_completed_part(&self) {
        self.recovered_polynomials.fetch_add(1, PROGRESS_ORDERING);
    }

    /// Records a share that failed verification, returning the cycle's total afterwards.
    pub fn record_invalid_share(&self) -> u64 {
        self.invalid_shares.fetch_add(1, PROGRESS_ORDERING) + 1
    }

    /// Shares that have failed verification for this cycle so far, across all peers.
    #[cfg(test)]
    pub fn invalid_shares(&self) -> u64 {
        self.invalid_shares.load(PROGRESS_ORDERING)
    }

    /// Absolute recovery progress for this cycle.
    pub fn progress(&self) -> SsaRecoveryProgress<S::Pseudonym> {
        SsaRecoveryProgress {
            ssa_id: self.id,
            useful_shares: self.useful_shares.load(PROGRESS_ORDERING),
            shares_seen: self.shares_seen.load(PROGRESS_ORDERING),
            target_useful_shares: self.target_useful_shares,
            // Bounded by `num_polys`, itself bounded by `MAX_POLYS_PER_SSA`, so the saturation
            // below is unreachable rather than lossy.
            recovered_polynomials: self.recovered_polynomials.load(PROGRESS_ORDERING).min(u16::MAX as u32) as u16,
        }
    }

    /// The part builder for one polynomial, or `None` if the index is out of range.
    ///
    /// The index originates from a peer-supplied share, so this is a checked lookup and must stay
    /// one.
    pub fn part(&self, poly_index: PolynomialIndex) -> Option<&parking_lot::Mutex<SsaPartBuilder<S>>> {
        self.parts.get(poly_index as usize)
    }

    /// The accumulator that sums recovered parts into the SSA scalar.
    pub fn builder(&self) -> &parking_lot::Mutex<SsaBuilder<S>> {
        &self.builder
    }
}

/// Reconstruct a single SSA from a set of SSA parts recovered from polynomials.
pub struct SsaBuilder<S: PixSpec> {
    pub full_commitment: PixGroup<S>,
    num_polys: usize,
    builder: PixScalar<S>,
    received_indices: ahash::HashSet<PolynomialIndex>,
    early_notified: bool,
}

impl<S: PixSpec> SsaBuilder<S> {
    pub fn new(full_commitment: PixGroup<S>, exit_secret_scalar: PixScalar<S>, num_polys: usize) -> Self {
        use ahash::HashSetExt;

        Self {
            full_commitment,
            builder: exit_secret_scalar,
            num_polys,
            received_indices: ahash::HashSet::with_capacity(num_polys),
            early_notified: false,
        }
    }

    /// Number of polynomials this SSA is composed of.
    pub fn num_polys(&self) -> usize {
        self.num_polys
    }

    /// Returns `true` once, when the number of received polynomial parts reaches
    /// `ceil(threshold * num_polys)` for the first time. Subsequent calls return
    /// `false` (idempotent guard — fires at most once per SSA lifecycle).
    pub fn check_early_threshold(&mut self, threshold: f64) -> bool {
        if self.early_notified {
            return false;
        }
        let needed = (threshold * self.num_polys as f64).ceil() as usize;
        if self.received_indices.len() >= needed {
            self.early_notified = true;
            true
        } else {
            false
        }
    }

    pub fn add_recovered_ssa_part(
        &mut self,
        index: PolynomialIndex,
        sub_secret: PixScalar<S>,
    ) -> errors::Result<Option<PixScalar<S>>, S::Pseudonym> {
        if !self.received_indices.insert(index) {
            return Ok(None);
        }

        self.builder += sub_secret;

        if self.received_indices.len() < self.num_polys {
            // SSA private scalar is not yet complete
            return Ok(None);
        }

        // This is computed only once when we have all the polynomials reconstructed
        if self.full_commitment == (PixGroup::<S>::generator() * self.builder) {
            self.early_notified = true;
            Ok(Some(self.builder))
        } else {
            Err(errors::PixError::InvalidSsa)
        }
    }
}

/// What a share contributed to its polynomial.
///
/// The four non-contributing outcomes are kept apart from one another because they are not
/// equivalent to a caller counting progress: a `Duplicate` says the peer re-sent an evaluation
/// point, a `Surplus` says it is still emitting for a polynomial that is already done (expected —
/// the Entry sends `threshold + surplus` shares per polynomial), a `SurplusOverBudget` says it has
/// emitted more of those than it negotiated, and `Absorbed` says the polynomial has already failed
/// and nothing can change that.
pub enum AddShareOutcome<S: PixSpec> {
    /// Same evaluation identifier as a share already collected for this polynomial.
    Duplicate,
    /// Arrived after the polynomial was already reconstructed, within the negotiated surplus.
    Surplus,
    /// Arrived after the polynomial was already reconstructed, past the negotiated surplus.
    ///
    /// Distinct from [`Surplus`](Self::Surplus) because only the latter is evidence the Entry is
    /// conforming. Once a peer is past its own budget, further shares for the same polynomial say
    /// nothing about whether it is still working on the cycle — see
    /// `SsaPartBuilder::max_credited_surplus`.
    SurplusOverBudget,
    /// Arrived after the polynomial failed to open its commitment, and was discarded.
    Absorbed,
    /// New and distinct, but the threshold is not reached yet.
    Useful,
    /// Reached the threshold; the constant term reconstructed and opened its commitment.
    Completed(PixScalar<S>),
}

/// Collects shares of a single polynomial and reconstructs its constant term.
///
/// ## Where verification happens
///
/// Nothing is checked per share beyond what interpolation itself requires (a non-zero, distinct
/// x-coordinate and a decodable y). The one cryptographic check is against
/// [`SsaPartCommitment`], run **once**, on the reconstructed constant term. See that type for why
/// this is sufficient here and what it costs — briefly: PIX has a single shareholder, so
/// "the recovered `a₀` is the committed one" is the whole property, and it is exact.
pub struct SsaPartBuilder<S: PixSpec> {
    commitment: SsaPartCommitment<S>,
    /// Shares needed to interpolate, i.e. the negotiated polynomial threshold.
    ///
    /// Comes from [`SsaCommitmentBuilder::poly_threshold`], never from the commitment: there is
    /// only one commitment per polynomial now, so its size says nothing about the degree.
    min_shares: usize,
    shares: Vec<CompletedShare<S>>,
    reconstructed: Option<PixScalar<S>>,
    /// Set when the part could not be reconstructed — either the interpolation itself failed, or
    /// the value it produced failed to open [`Self::commitment`].
    ///
    /// The failure is reported exactly once; every later share for this polynomial is absorbed
    /// silently. There is nothing to be gained from re-running the interpolation — the share set
    /// cannot be repaired without knowing *which* share is bad, and the cycle is already lost
    /// because [`SsaBuilder`] needs every polynomial. Both failure paths must therefore set this
    /// *and* release the share buffer, or the "exactly once" only holds for one of them.
    failed: bool,
    /// Post-reconstruction shares credited as liveness so far, bounded by
    /// [`Self::max_credited_surplus`].
    surplus_seen: usize,
    /// How many post-reconstruction shares this polynomial may contribute as liveness evidence.
    ///
    /// The negotiated `surplus_shares`, i.e. exactly what a conforming Entry emits per polynomial
    /// once the threshold is met — and, since `20845807ae`, exactly what the per-SSA quota charges
    /// for. Crediting to the negotiated figure therefore credits precisely the traffic that was paid
    /// for, and nothing beyond it.
    ///
    /// A bound is needed at all because reconstruction releases the collected share set, so
    /// duplicate detection is gone from that moment on: every later share for the polynomial is
    /// indistinguishable from every other. Without a cap an Entry could complete one polynomial,
    /// replay its shares forever, and keep the Exit's egress gate and idle deadline refreshed while
    /// touching none of the remaining `polys - 1`. The cap makes the free ride finite —
    /// `polys × surplus_shares` per cycle, which is the conforming volume.
    max_credited_surplus: usize,
}

impl<S: PixSpec> SsaPartBuilder<S> {
    pub fn new(commitment: SsaPartCommitment<S>, min_shares: usize, max_credited_surplus: usize) -> Self {
        Self {
            commitment,
            min_shares,
            shares: Vec::new(),
            reconstructed: None,
            failed: false,
            surplus_seen: 0,
            max_credited_surplus,
        }
    }

    /// [`SsaPolynomialId`] of the polynomial this builder reconstructs.
    ///
    /// Remains valid after the collected shares have been released.
    pub(crate) fn spi(&self) -> SsaPolynomialId<S::Pseudonym> {
        self.commitment.spi
    }

    /// Shares needed to interpolate this polynomial — the negotiated threshold.
    pub(crate) fn min_shares(&self) -> usize {
        self.min_shares
    }

    /// Frees the collected shares, which are only needed until the part is reconstructed.
    ///
    /// After that point they cannot be read again — the early returns in
    /// [`add_share`](Self::add_share) short-circuit every later call before it touches them.
    ///
    /// At production dimensions this buffer is `threshold × size_of::<CompletedShare>()` held for
    /// *every* one of the `polys` polynomials until the cycle is retired. Since the Entry emits
    /// shares polynomial-major, releasing here means only the polynomials still in flight hold any.
    ///
    /// Assigns a fresh empty `Vec` rather than `clear()`, so the backing allocation is actually
    /// returned instead of being retained at capacity.
    fn release_verification_state(&mut self) {
        self.shares = Vec::new();
    }

    /// Number of collected shares still held.
    ///
    /// Drops to zero once the part is reconstructed, or once it has failed its commitment.
    #[cfg(test)]
    pub(crate) fn verification_state_len(&self) -> usize {
        self.shares.len()
    }

    pub fn add_share(
        &mut self,
        msg: PixScalar<S>,
        share: PartialSsaShare<S>,
    ) -> errors::Result<AddShareOutcome<S>, S::Pseudonym> {
        // First, and before decoding: a failed part reports its fault exactly once, and every later
        // share for it is absorbed in silence. Validating ahead of this would turn each of those
        // into another `InvalidShare`, and the caller charges those against
        // `max_unverifiable_shares_per_ssa` — zero by default, so one already-doomed polynomial
        // would close the Session once per remaining share instead of once.
        if self.failed {
            return Ok(AddShareOutcome::Absorbed);
        }

        let share = into_completed_share(msg, &share)?;

        // A zero x-coordinate evaluates the polynomial at its constant term and would divide by
        // zero in the Lagrange basis; a zero y is degenerate in the same way. These used to be the
        // opening lines of the per-share Feldman check, and they are the part worth keeping —
        // unlike the check itself, they cost nothing.
        //
        // Ahead of the `reconstructed` check below, so that a share which could never have come
        // from the committed polynomial is rejected whether or not that polynomial is already done.
        // The other order made a completed polynomial into a laundering channel: every malformed
        // share addressed to it returned `Surplus`, which the Exit credits as evidence the Entry is
        // alive.
        if (share.value().is_zero() | share.identifier().is_zero()).into() {
            return Err(vsss_rs::Error::InvalidShare.into());
        }

        if self.reconstructed.is_some() {
            // Past the budget the share is real but no longer evidence of anything: see
            // `max_credited_surplus`.
            if self.surplus_seen >= self.max_credited_surplus {
                return Ok(AddShareOutcome::SurplusOverBudget);
            }
            self.surplus_seen += 1;
            return Ok(AddShareOutcome::Surplus);
        }

        // Reject duplicate shares — the same identifier is the same X-coordinate, which carries no
        // new information and makes the interpolation singular.
        //
        // Scanning the collected shares is sufficient to classify duplicates, and no separate set of
        // seen identifiers is needed: `self.shares` is only released once the part is reconstructed
        // or has failed, and both of those short-circuit above. So every call that reaches here
        // still has the full set in hand.
        if self.shares.iter().any(|s| s.identifier == share.identifier) {
            return Ok(AddShareOutcome::Duplicate);
        }

        self.shares.push(share);

        if self.shares.len() < self.min_shares {
            return Ok(AddShareOutcome::Useful);
        }

        let reconstructed = match self.shares.combine() {
            Ok(combined) => combined.0,
            Err(error) => {
                // Terminal, exactly like a failed commitment opening below, so it has to be
                // recorded the same way. Propagating with `?` alone would leave the part with a
                // full share set, no `reconstructed` and no `failed`, so neither early return
                // above would fire: every remaining share for this polynomial would be pushed and
                // re-run the interpolation over a larger set, and would re-report the same fault.
                self.release_verification_state();
                self.failed = true;
                return Err(error.into());
            }
        };
        self.release_verification_state();

        // The only elliptic curve operation on the share path: one fixed-base multiplication per
        // polynomial, against `threshold` per share previously.
        if !self.commitment.verify_reconstructed(&reconstructed) {
            self.failed = true;
            return Err(vsss_rs::Error::InvalidShare.into());
        }

        self.reconstructed = Some(reconstructed);
        Ok(AddShareOutcome::Completed(reconstructed))
    }
}

/// Incremental outcome of feeding coefficient commitments into an [`SsaCommitmentBuilder`].
///
/// Everything happens on one call: the constant-term set completing is simultaneously the moment
/// the SSA commitment becomes known *and* the moment every polynomial becomes reconstructible,
/// since a polynomial's whole commitment is its constant term. The fields stay separate because
/// the caller must publish them in a specific order — see `insert_coefficient_commitments`.
pub struct CommitmentProgress<S: PixSpec> {
    /// Full SSA commitment (Client + Exit), once every constant term has arrived.
    pub full_commitment: Option<PixGroup<S>>,
    /// The SSA part accumulator, yielded exactly once — on the call that completes the
    /// constant-term set. The caller must publish it before any share can be reconstructed.
    pub ssa_builder: Option<SsaBuilder<S>>,
    /// Per-polynomial part builders, all yielded together on that same call.
    pub new_verifiers: Vec<SsaPartBuilder<S>>,
    /// `true` on the call that hands the part builders out.
    pub fully_committed: bool,
}

impl<S: PixSpec> CommitmentProgress<S> {
    fn empty() -> Self {
        Self {
            full_commitment: None,
            ssa_builder: None,
            new_verifiers: Vec::new(),
            fully_committed: false,
        }
    }
}

/// Builds a complete SSA from the incoming client constant-term commitments of the SSA-part
/// polynomials for a specific Session Stealth Address (SSA).
///
/// One commitment per polynomial arrives, so "the SSA commitment is known" and "every polynomial
/// is reconstructible" are the same event: `committed_polynomials.len() == num_polys`.
pub struct SsaCommitmentBuilder<S: PixSpec> {
    id: SsaId<S::Pseudonym>,
    /// Shares needed to reconstruct one polynomial, as negotiated at session establishment.
    ///
    /// The commitments no longer carry the degree, so this is the only source for it. It is
    /// handed to every [`SsaPartBuilder`] as its `min_shares`.
    poly_threshold: usize,
    /// Shares the peer said it would emit per polynomial *beyond* [`Self::poly_threshold`], as
    /// negotiated at session establishment.
    ///
    /// Handed to every [`SsaPartBuilder`] as its `max_credited_surplus`. Kept alongside the
    /// threshold rather than derived from it: the two are independent halves of the negotiated
    /// [`PixParams`](crate::PixParams) word, and the ratio between them is an operator choice.
    poly_surplus: usize,
    num_polys: usize,
    /// Constant-term commitments received so far, **decoded**: each is decompressed and
    /// subgroup-checked exactly once, on arrival. Keeping the compressed representation instead
    /// would force a second decompression when the part builders are created, and decompression
    /// is the dominant per-commitment cost.
    ///
    /// Drained when the part builders are handed out.
    committed_polynomials: std::collections::HashMap<PolynomialIndex, PixGroup<S>>,
    /// Commitments received so far, counted across polynomials that have since been handed out.
    /// Used for `is_empty`, which would otherwise report empty again after the drain.
    total_committed: usize,
    complete: bool,
    exit_commitment_secret: PixScalar<S>,
    exit_commitment_public: PixGroup<S>,
    /// First [`SsaCommitmentProof`] the peer supplied, checked once the constant-term set is
    /// complete — that is the point at which the commitment it opens becomes known.
    ///
    /// Only the first is kept: any single valid proof is sufficient, and Schnorr proofs are
    /// randomised, so there is nothing to reconcile between several of them.
    commitment_proof: Option<SsaCommitmentProof<S>>,
    full_ssa_commitment: Option<(PixGroup<S>, S::DepositAddress)>,
}

impl<S: PixSpec> SsaCommitmentBuilder<S> {
    /// Takes the whole negotiated triple rather than the two fields it used to read.
    ///
    /// The surplus is the third, and it was the one previously dropped on the floor at the call
    /// site: the Session layer held a [`PixParams`] and destructured two fields out of it. Keeping
    /// the type intact is what makes the per-polynomial surplus budget available down here — see
    /// [`SsaPartBuilder::max_credited_surplus`].
    pub fn new(
        id: SsaId<S::Pseudonym>,
        params: PixParams,
        exit_commitment_secret: PixScalar<S>,
        exit_commitment_public: PixGroup<S>,
    ) -> Self {
        Self {
            id,
            poly_threshold: params.shares_per_poly() as usize,
            poly_surplus: params.surplus_shares() as usize,
            num_polys: params.polys_per_ssa() as usize,
            exit_commitment_secret,
            exit_commitment_public,
            committed_polynomials: std::collections::HashMap::new(),
            total_committed: 0,
            complete: false,
            commitment_proof: None,
            full_ssa_commitment: None,
        }
    }

    /// `true` if not a single coefficient commitment has been received yet.
    ///
    /// Counts rather than inspecting `committed_polynomials`, which is drained when the part
    /// builders are handed out and would therefore report empty again afterwards.
    pub fn is_empty(&self) -> bool {
        self.total_committed == 0
    }

    pub fn get_deposit_address(&self) -> Option<&S::DepositAddress> {
        self.full_ssa_commitment.as_ref().map(|(_, a)| a)
    }

    pub fn add_transposed(
        &mut self,
        coeff_index: CoefficientIndex,
        proof: Option<SsaCommitmentProof<S>>,
        polynomial_coeff_commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> errors::Result<CommitmentProgress<S>, S::Pseudonym> {
        // Commitments to non-constant coefficients carry nothing this side can use: shares are
        // checked in aggregate, against the constant term, once the part is reconstructed (see
        // `SsaPartCommitment`). Ignore rather than reject, so a peer that still sends the full
        // Feldman matrix merely wastes its own bandwidth.
        //
        // Deliberately ahead of the `complete` guard below: such a peer sends the bulk of them
        // *after* the constant-term pass has finished, and those must not be mistaken for a
        // duplicate-commitment attack. Nothing is decoded here either, so the ~152 µs per
        // commitment is not spent on data that is about to be dropped.
        if coeff_index != CONSTANT_TERM_COEFFICIENT {
            tracing::debug!(
                id = %self.id,
                coeff_index,
                "ignoring commitments to a non-constant polynomial coefficient"
            );
            return Ok(CommitmentProgress {
                full_commitment: self.full_ssa_commitment.as_ref().map(|(c, _)| *c),
                // Report the state as it stands; ignoring a message must not make a completed
                // cycle look incomplete.
                fully_committed: self.complete,
                ..CommitmentProgress::empty()
            });
        }

        // Cannot add more commitments if we already have all
        if self.complete {
            return Err(errors::PixError::DuplicateCommitment);
        }

        // Retain the first proof offered. It cannot be checked yet: the commitment it opens is the
        // sum of *all* constant terms, so verification waits for the milestone below. Recorded
        // before the transactional insert so that a batch which later bails on a duplicate still
        // leaves the proof available — it is not part of the state the duplicate check protects.
        if self.commitment_proof.is_none() {
            self.commitment_proof = proof;
        }

        // Collect and validate all items before mutating state (transactional).
        //
        // Decoding here is the *only* decode of each commitment: the resulting group element is
        // what gets stored, so building the part builders below never decompresses again.
        //
        // The check is `decode_commitment` — decodable *and* inside the prime-order subgroup. It
        // must not be weaker: a commitment that passes here occupies its slot permanently, because
        // re-insertion is rejected as a duplicate. A weaker check would let a
        // decodable-but-small-order point take a slot and then fail unconditionally at completion,
        // with no way to retransmit a correction.
        let mut validated: Vec<(PolynomialIndex, PixGroup<S>)> = Vec::new();
        for (polynomial_index, polynomial_coeff_commitment) in polynomial_coeff_commitments {
            if polynomial_index >= self.num_polys as PolynomialIndex {
                return Err(errors::PixError::InvalidInput);
            }
            validated.push((
                polynomial_index,
                SsaPartCommitment::<S>::decode_commitment(&polynomial_coeff_commitment)?,
            ));
        }

        // Check for duplicate occupancy before any insertion (transactional).
        //
        // A repeat *within* the batch counts. Testing only against `committed_polynomials` would let
        // two entries sharing a polynomial index both see a vacant slot: the second insert would
        // silently rebind the first — the single-assignment invariant this two-phase check exists to
        // enforce — and `total_committed` would count two occupants of one slot, so a batch of
        // `num_polys` entries containing a repeat could never complete the set and every retry would
        // be rejected as a duplicate against the slots it did fill. The wire decoder rejects
        // intra-message duplicates today, but this builder is not meant to depend on that.
        let mut seen = std::collections::HashSet::with_capacity(validated.len());
        for (polynomial_index, _) in &validated {
            if self.committed_polynomials.contains_key(polynomial_index) || !seen.insert(*polynomial_index) {
                return Err(errors::PixError::DuplicateCommitment);
            }
        }

        // Second phase: insert into confirmed-vacant slots, maintaining the progress counter.
        for (polynomial_index, polynomial_coeff_commitment) in validated {
            self.committed_polynomials
                .insert(polynomial_index, polynomial_coeff_commitment);
            self.total_committed += 1;
        }

        tracing::trace!(
            id = %self.id,
            "SSA commitment is {:.2}% complete",
            self.total_committed as f64 * 100.0 / self.num_polys as f64
        );

        let mut progress = CommitmentProgress::empty();

        // The one milestone: every constant term is in, so the SSA commitment — and with it the
        // deposit address, the part accumulator and every polynomial's part builder — becomes
        // known. Reading the map must happen before the drain below empties it.
        if self.full_ssa_commitment.is_none() && self.committed_polynomials.len() == self.num_polys {
            // Constant terms are already decoded; summing them needs no decompression.
            let client_ssa_commitment = self.committed_polynomials.values().copied().sum::<PixGroup<S>>();
            tracing::debug!(id = %self.id, commitment = const_hex::encode(client_ssa_commitment.to_bytes()), "SSA client commitment");

            // The client commitment is now known, so its proof of knowledge can finally be checked.
            //
            // This gate must sit *before* `full_ssa_commitment` is recorded and before the
            // `SsaBuilder` is handed out: those are what produce the deposit address and make the
            // cycle live. Rejecting here means an unproven commitment never reaches the deposit
            // path at all, which is the whole point — a peer that does not know the discrete
            // logarithm of what it published may know the discrete logarithm of the *sum* with our
            // own commitment, and could then sweep the deposit itself.
            if !self
                .commitment_proof
                .as_ref()
                .is_some_and(|proof| proof.verify(&self.id, &client_ssa_commitment))
            {
                tracing::error!(id = %self.id, "client ssa commitment has no valid proof of knowledge");
                return Err(errors::PixError::UnprovenSsaCommitment);
            }

            let full_ssa_commitment = client_ssa_commitment + self.exit_commitment_public;

            // Treat the failed conversion to deposit address as error
            let deposit_addr =
                S::group_to_deposit_address(full_ssa_commitment).ok_or(errors::PixError::InvalidInput)?;

            // A zero threshold would make every part builder reconstruct from no shares at all.
            // `PixParams` cannot hold one — `try_new` enforces `MIN_POLY_THRESHOLD` — so this is
            // defensive, but it must be checked before the drain below, so a failure cannot strand
            // the commitments it has already taken.
            if self.poly_threshold == 0 {
                return Err(errors::PixError::InvalidInput);
            }

            self.full_ssa_commitment = Some((full_ssa_commitment, deposit_addr));
            progress.ssa_builder = Some(SsaBuilder::new(
                full_ssa_commitment,
                self.exit_commitment_secret,
                self.num_polys,
            ));

            // Hand out every polynomial's part builder in this same call. Each commitment was
            // decoded and subgroup-checked on arrival, so this costs no elliptic curve work, and
            // the map is drained as it goes so the builder never holds both representations.
            //
            // They all become available at once because a polynomial's entire commitment *is* its
            // constant term: there is no partially committed row to wait on. Shares that arrived
            // before this point were deferred by the reconstructor and are redeemed by the caller
            // right after these are installed.
            progress.new_verifiers.reserve(self.committed_polynomials.len());
            for (poly_index, constant_term) in self.committed_polynomials.drain() {
                progress.new_verifiers.push(SsaPartBuilder::new(
                    SsaPartCommitment::from_decoded_commitment(
                        SsaPolynomialId::new(self.id, poly_index),
                        constant_term,
                    ),
                    self.poly_threshold,
                    self.poly_surplus,
                ));
            }

            tracing::debug!(id = %self.id, "SSA is fully committed for verification");
            self.complete = true;
            progress.fully_committed = true;
        }

        progress.full_commitment = self.full_ssa_commitment.as_ref().map(|(c, _)| *c);

        Ok(progress)
    }
}
