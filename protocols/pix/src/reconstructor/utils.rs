use vsss_rs::{
    ReadableShareSet,
    elliptic_curve::group::{Group, GroupEncoding},
};

use crate::{
    CoefficientIndex, CompletedShare, PartialSsaShare, PartialSsaShareVerifier, PixGroup, PixGroupRepr, PixScalar,
    PixSpec, PolynomialIndex, SsaCommitmentProof, SsaPolynomialId, errors, into_completed_share, types::SsaId,
};

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

/// Verifies shares and reconstructs a single SSA part from them.
pub struct SsaPartBuilder<S: PixSpec> {
    /// Verifier for this polynomial's shares.
    ///
    /// Its commitment vector is **released** the moment the part is reconstructed (see
    /// [`add_share`](Self::add_share)), so after that point only `spi` remains meaningful.
    /// The field is private for exactly that reason: nothing outside this module may reach
    /// through it to `min_shares()`, `constant_term()` or `verify*()`, since those read the
    /// released vector. Use [`spi`](Self::spi).
    verifier: PartialSsaShareVerifier<S>,
    /// Cached `verifier.min_shares()`.
    ///
    /// Must be cached rather than derived on demand: `min_shares()` is
    /// `poly_commitment.len() - 1`, which stops being meaningful once the commitment vector
    /// is released.
    min_shares: usize,
    shares: Vec<CompletedShare<S>>,
    reconstructed: Option<PixScalar<S>>,
}

impl<S: PixSpec> SsaPartBuilder<S> {
    pub fn new(verifier: PartialSsaShareVerifier<S>) -> Self {
        Self {
            min_shares: verifier.min_shares(),
            verifier,
            shares: Vec::new(),
            reconstructed: None,
        }
    }

    /// [`SsaPolynomialId`] of the polynomial this builder reconstructs.
    ///
    /// Remains valid after the verification state has been released.
    pub(crate) fn spi(&self) -> SsaPolynomialId<S::Pseudonym> {
        self.verifier.spi
    }

    /// Frees everything that was only needed to verify and combine shares.
    ///
    /// Called once the part is reconstructed, at which point neither the commitment vector nor
    /// the collected shares can be read again — the early return in
    /// [`add_share`](Self::add_share) short-circuits every later call before it touches them.
    ///
    /// This is the dominant term in reconstructor memory: at production dimensions the
    /// commitment vector is `(threshold + 1) × size_of::<PixGroup>()` and the share buffer is
    /// `threshold × size_of::<CompletedShare>()`, held for *every* one of the `polys`
    /// polynomials until the whole cycle is retired. Since the Entry emits shares
    /// polynomial-major, releasing here means only the polynomials still in flight hold any.
    ///
    /// Assigns fresh empty `Vec`s rather than `clear()`, so the backing allocations are
    /// actually returned instead of being retained at capacity.
    fn release_verification_state(&mut self) {
        self.verifier.poly_commitment = Vec::new();
        self.shares = Vec::new();
    }

    /// Number of commitments and collected shares still held for verification.
    ///
    /// Both drop to zero once the part is reconstructed.
    #[cfg(test)]
    pub(crate) fn verification_state_len(&self) -> (usize, usize) {
        (self.verifier.poly_commitment.len(), self.shares.len())
    }

    pub fn add_share(
        &mut self,
        msg: PixScalar<S>,
        share: PartialSsaShare<S>,
    ) -> errors::Result<Option<PixScalar<S>>, S::Pseudonym> {
        if let Some(reconstructed) = self.reconstructed {
            return Ok(Some(reconstructed));
        }

        let share = into_completed_share(msg, &share)?;

        // Reject duplicate shares — same identifier means the same X-coordinate,
        // which contributes no new information and can cause premature combination
        // attempts that will persistently fail.
        // Check this before verification to avoid expensive elliptic curve MSMs for redundant shares.
        if self.shares.iter().any(|s| s.identifier == share.identifier) {
            return Ok(None);
        }

        self.verifier.verify_completed_share(&share)?;

        self.shares.push(share);

        if self.shares.len() >= self.min_shares {
            let reconstructed = self.shares.combine()?.0;
            self.reconstructed = Some(reconstructed);
            self.release_verification_state();
            Ok(Some(reconstructed))
        } else {
            Ok(None)
        }
    }
}

/// Coefficient commitments of a single polynomial, keyed by coefficient index.
///
/// Commitments are stored **decoded**: each one is decompressed and subgroup-checked exactly once,
/// when it arrives in [`SsaCommitmentBuilder::add_transposed`]. Keeping the compressed
/// representation instead would force a second decompression when the verifiers are built, and at
/// production dimensions (`polys × threshold` commitments per SSA) that doubles the dominant cost
/// of the commitment phase.
type CommittedPolynomial<S> = std::collections::HashMap<CoefficientIndex, PixGroup<S>>;

/// Incremental outcome of feeding coefficient commitments into an [`SsaCommitmentBuilder`].
///
/// Unlike an all-or-nothing result, this reports the two milestones independently, because they
/// are reached at different times and each unblocks something different:
///
/// * the **SSA commitment** becoming known (all constant terms in) yields the deposit address and the [`SsaBuilder`] —
///   from that point recovered polynomial parts have somewhere to go;
/// * an **individual polynomial's row** completing yields that polynomial's verifier — from that point its shares can
///   be verified, long before the rest of the cycle has arrived.
pub struct CommitmentProgress<S: PixSpec> {
    /// Full SSA commitment (Client + Exit), once every constant term has arrived.
    pub full_commitment: Option<PixGroup<S>>,
    /// The SSA part accumulator, yielded exactly once — on the call that first completes the
    /// constant-term set. The caller must publish it before any share can be reconstructed.
    pub ssa_builder: Option<SsaBuilder<S>>,
    /// Verifiers for polynomials whose rows completed on this call (or earlier, if they had to wait
    /// for the SSA commitment). Each is yielded exactly once.
    pub new_verifiers: Vec<SsaPartBuilder<S>>,
    /// `true` on the call that hands out the last polynomial's verifier.
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

/// Builds a complete SSA from the incoming client polynomial coefficient commitments of
/// SSA-part polynomials for a specific Session Stealth Address (SSA).
///
/// ## Progress tracking is O(1) per commitment
///
/// Completion used to be detected by scanning every polynomial's map on every inserted batch,
/// which is `O(polys)` per message — at production dimensions ~8192 iterations across ~18 700
/// messages per cycle. The counters below make each of the three questions ("how far along?",
/// "are all constant terms in?", "is this polynomial's row full?") a single comparison.
pub struct SsaCommitmentBuilder<S: PixSpec> {
    id: SsaId<S::Pseudonym>,
    poly_threshold: usize,
    num_polys: usize,
    committed_polynomials: std::collections::HashMap<PolynomialIndex, CommittedPolynomial<S>>,
    /// Cells filled so far, counted across polynomials that have since been handed out as
    /// verifiers. Used for the progress trace and for `is_empty`, both of which would otherwise
    /// have to walk `committed_polynomials`.
    total_committed: usize,
    /// Polynomials whose constant term (coefficient 0) has arrived. The SSA commitment is
    /// computable once this reaches `num_polys`.
    constant_terms_committed: usize,
    /// Polynomials whose row is complete but whose verifier has not been handed out yet.
    ///
    /// Only ever non-empty while the SSA commitment is still unknown: a verifier is useless
    /// until there is an [`SsaBuilder`] to receive the part it reconstructs, and the constant
    /// terms are what produce that. A conforming Entry sends the constant-term pass first, so
    /// this stays empty in practice — but correctness must not depend on the peer's send order.
    ready_polynomials: Vec<PolynomialIndex>,
    /// Verifiers handed out so far. The commitment is fully verifiable at `num_polys`.
    installed_polynomials: usize,
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
    pub fn new(
        id: SsaId<S::Pseudonym>,
        poly_threshold: usize,
        num_polys: usize,
        exit_commitment_secret: PixScalar<S>,
        exit_commitment_public: PixGroup<S>,
    ) -> Self {
        Self {
            id,
            poly_threshold,
            num_polys,
            exit_commitment_secret,
            exit_commitment_public,
            committed_polynomials: std::collections::HashMap::new(),
            total_committed: 0,
            constant_terms_committed: 0,
            ready_polynomials: Vec::new(),
            installed_polynomials: 0,
            complete: false,
            commitment_proof: None,
            full_ssa_commitment: None,
        }
    }

    /// `true` if not a single coefficient commitment has been received yet.
    ///
    /// Counts rather than inspecting `committed_polynomials`, which is drained as verifiers are
    /// handed out and would therefore report empty again mid-cycle.
    pub fn is_empty(&self) -> bool {
        self.total_committed == 0
    }

    pub fn get_deposit_address(&self) -> Option<&S::DepositAddress> {
        self.full_ssa_commitment.as_ref().map(|(_, a)| a)
    }

    /// Number of polynomials this SSA commitment is composed of.
    pub fn num_polys(&self) -> usize {
        self.num_polys
    }

    pub fn add_transposed(
        &mut self,
        coeff_index: CoefficientIndex,
        proof: Option<SsaCommitmentProof<S>>,
        polynomial_coeff_commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> errors::Result<CommitmentProgress<S>, S::Pseudonym> {
        // Cannot add more commitments if we already have all
        if self.complete {
            return Err(errors::PixError::DuplicateCommitment);
        }

        if coeff_index >= self.poly_threshold as CoefficientIndex {
            return Err(errors::PixError::InvalidInput);
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
        // what gets stored, so the verifier-building phase below never decompresses again.
        //
        // The check is `decode_commitment`, i.e. the exact same validation the verifiers apply
        // (decodable *and* inside the prime-order subgroup). It must not be weaker: a commitment
        // that passes here occupies its cell permanently, because re-insertion is rejected as a
        // duplicate. A weaker check would let a decodable-but-small-order point take a cell and
        // then fail unconditionally at completion, with no way to retransmit a correction.
        let mut validated: Vec<(PolynomialIndex, PixGroup<S>)> = Vec::new();
        for (polynomial_index, polynomial_coeff_commitment) in polynomial_coeff_commitments {
            if polynomial_index >= self.num_polys as PolynomialIndex {
                return Err(errors::PixError::InvalidInput);
            }
            validated.push((
                polynomial_index,
                PartialSsaShareVerifier::<S>::decode_commitment(&polynomial_coeff_commitment)?,
            ));
        }

        // Check for duplicate occupancy before any insertion (transactional).
        for (polynomial_index, _) in &validated {
            let polynomial = self.committed_polynomials.entry(*polynomial_index).or_default();
            if polynomial.contains_key(&coeff_index) {
                return Err(errors::PixError::DuplicateCommitment);
            }
        }

        // Second phase: insert into confirmed-vacant slots, maintaining the progress counters.
        for (polynomial_index, polynomial_coeff_commitment) in validated {
            let polynomial = self.committed_polynomials.entry(polynomial_index).or_default();
            polynomial.insert(coeff_index, polynomial_coeff_commitment);

            self.total_committed += 1;
            if coeff_index == 0 {
                self.constant_terms_committed += 1;
            }
            // A row is complete the moment it holds one commitment per coefficient. Cells can
            // never be overwritten (re-insertion is rejected as a duplicate above), so `len`
            // reaching the threshold happens exactly once per polynomial.
            if polynomial.len() == self.poly_threshold {
                self.ready_polynomials.push(polynomial_index);
            }
        }

        tracing::trace!(
            id = %self.id,
            "SSA commitment is {:.2}% complete",
            self.total_committed as f64 * 100.0 / (self.num_polys * self.poly_threshold) as f64
        );

        let mut progress = CommitmentProgress::empty();

        // Milestone 1: every constant term is in, so the SSA commitment — and with it the deposit
        // address and the part accumulator — becomes known. Must run before any polynomial is
        // handed out below, because handing one out removes its constant term from the map.
        if self.full_ssa_commitment.is_none() && self.constant_terms_committed == self.num_polys {
            // Constant terms are already decoded; summing them needs no decompression.
            let client_ssa_commitment = self
                .committed_polynomials
                .values()
                .map(|p| *p.get(&0).expect("constant term must be present"))
                .sum::<PixGroup<S>>();
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

            self.full_ssa_commitment = Some((full_ssa_commitment, deposit_addr));
            progress.ssa_builder = Some(SsaBuilder::new(
                full_ssa_commitment,
                self.exit_commitment_secret,
                self.num_polys,
            ));
        }

        progress.full_commitment = self.full_ssa_commitment.as_ref().map(|(c, _)| *c);

        // Milestone 2: hand out verifiers for every row that is complete, but only once there is
        // an `SsaBuilder` to receive what they reconstruct. Rows that completed earlier are still
        // queued in `ready_polynomials` and are released here in the same batch.
        // `from_decoded_commitments` rejects a commitment vector shorter than two entries, i.e. a
        // zero threshold. Checked here, before any state is moved out below, so that the release
        // loop cannot fail part-way through and strand the rows it has already taken.
        // `new_exit_commitment` enforces `shares_per_poly >= 2`, so this is defensive.
        if self.poly_threshold == 0 {
            return Err(errors::PixError::InvalidInput);
        }

        if self.full_ssa_commitment.is_some() && !self.ready_polynomials.is_empty() {
            // Every commitment was decoded and subgroup-checked on arrival above and stored as a
            // group element, so assembling a verifier involves no elliptic curve decompression.
            //
            // Each polynomial is *removed* from `committed_polynomials` as its verifier is built,
            // so the builder never holds a row and its verifier simultaneously. With rows released
            // as they complete, the live commitment set is a sliding window over the polynomials
            // still in flight rather than the whole `polys × threshold` matrix.
            let ready = std::mem::take(&mut self.ready_polynomials);
            progress.new_verifiers.reserve(ready.len());
            for poly_index in ready {
                let polynomial = self
                    .committed_polynomials
                    .remove(&poly_index)
                    .expect("a ready polynomial must still be present");
                let verifier = PartialSsaShareVerifier::from_decoded_commitments(
                    SsaPolynomialId::new(self.id, poly_index),
                    (0..self.poly_threshold as CoefficientIndex).map(|coeff_idx| {
                        *polynomial
                            .get(&coeff_idx)
                            .expect("polynomial coeffs must be already present")
                    }),
                )?;
                progress.new_verifiers.push(SsaPartBuilder::new(verifier));
            }

            self.installed_polynomials += progress.new_verifiers.len();
            if self.installed_polynomials == self.num_polys {
                tracing::debug!(id = %self.id, "SSA is fully committed for verification");
                self.complete = true;
                progress.fully_committed = true;
            }
        }

        Ok(progress)
    }
}
