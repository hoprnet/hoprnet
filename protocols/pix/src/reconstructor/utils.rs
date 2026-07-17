use vsss_rs::{
    ReadableShareSet,
    elliptic_curve::group::{Group, GroupEncoding},
};

use crate::{
    CoefficientIndex, CompletedShare, IdentifierPrimeField, PartialSsaShare, PartialSsaShareVerifier, PixGroup,
    PixGroupRepr, PixScalar, PixSpec, PolynomialIndex, SsaPolynomialId, errors, into_completed_share, types::SsaId,
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

    /// Number of polynomials in this SSA.
    pub fn num_polys(&self) -> usize {
        self.num_polys
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

/// Outcome of adding a share to an [`SsaPartBuilder`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddShareOutcome<S: PixSpec> {
    /// Share was a duplicate (same evaluation identifier already collected for this polynomial).
    Duplicate,
    /// Share arrived after the polynomial was already fully reconstructed.
    Surplus,
    /// Share was useful (new, verified) but threshold not yet reached.
    Useful,
    /// Polynomial threshold reached and the reconstructed value is available.
    Completed(PixScalar<S>),
}

/// Verifies shares and reconstructs a single SSA part from them.
pub struct SsaPartBuilder<S: PixSpec> {
    pub(crate) verifier: PartialSsaShareVerifier<S>,
    shares: Vec<CompletedShare<S>>,
    reconstructed: Option<PixScalar<S>>,
    /// Set of evaluation identifiers already collected for this polynomial.
    /// Used to detect duplicate shares.
    collected_identifiers: ahash::AHashSet<IdentifierPrimeField<PixScalar<S>>>,
}

impl<S: PixSpec> SsaPartBuilder<S> {
    pub fn new(verifier: PartialSsaShareVerifier<S>) -> Self {
        Self {
            verifier,
            shares: Vec::new(),
            reconstructed: None,
            collected_identifiers: ahash::AHashSet::new(),
        }
    }

    pub fn add_share(
        &mut self,
        msg: PixScalar<S>,
        share: PartialSsaShare<S>,
    ) -> errors::Result<AddShareOutcome<S>, S::Pseudonym> {
        // Surplus: polynomial already reconstructed
        if self.reconstructed.is_some() {
            return Ok(AddShareOutcome::Surplus);
        }

        let identifier: IdentifierPrimeField<PixScalar<S>> = msg.into();

        // Duplicate: evaluation identifier already collected for this polynomial
        if self.collected_identifiers.contains(&identifier) {
            return Ok(AddShareOutcome::Duplicate);
        }

        let share = into_completed_share(msg, &share)?;

        self.verifier.verify_completed_share(&share)?;
        self.collected_identifiers.insert(identifier);
        self.shares.push(share);

        if self.shares.len() >= self.verifier.min_shares() {
            let reconstructed = self.shares.combine()?.0;
            self.reconstructed = Some(reconstructed);
            Ok(AddShareOutcome::Completed(reconstructed))
        } else {
            Ok(AddShareOutcome::Useful)
        }
    }
}

type CommittedPolynomial<S> = std::collections::HashMap<CoefficientIndex, PixGroupRepr<S>>;

/// Result of building an SSA commitment.
pub enum CommitmentResult<S: PixSpec> {
    /// Not enough commitments have been received yet.
    NotEnoughCommitments,
    /// There are enough commitments to build at least the SSA commitment.
    SsaCommitmentDone(PixGroup<S>),
    /// There are enough commitments to build at least the SSA commitment, but not all coefficients are committed yet.
    StillIncomplete(PixGroup<S>),
    /// All coefficients have been committed.
    Completed(SsaBuilder<S>, Vec<SsaPartBuilder<S>>),
}

/// Builds a complete SSA from the incoming client polynomial coefficient commitments of
/// SSA-part polynomials for a specific Session Stealth Address (SSA).
pub struct SsaCommitmentBuilder<S: PixSpec> {
    id: SsaId<S::Pseudonym>,
    poly_threshold: usize,
    num_polys: usize,
    committed_polynomials: std::collections::HashMap<PolynomialIndex, CommittedPolynomial<S>>,
    complete: bool,
    exit_commitment_secret: PixScalar<S>,
    exit_commitment_public: PixGroup<S>,
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
            complete: false,
            full_ssa_commitment: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.committed_polynomials.is_empty()
    }

    pub fn get_deposit_address(&self) -> Option<&S::DepositAddress> {
        self.full_ssa_commitment.as_ref().map(|(_, a)| a)
    }

    pub fn add_transposed(
        &mut self,
        coeff_index: CoefficientIndex,
        polynomial_coeff_commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> errors::Result<CommitmentResult<S>, S::Pseudonym> {
        // Cannot add more commitments if we already have all
        if self.complete {
            return Err(errors::PixError::DuplicateCommitment);
        }

        if coeff_index >= self.poly_threshold as CoefficientIndex {
            return Err(errors::PixError::InvalidInput);
        }

        for (polynomial_index, polynomial_coeff_commitment) in polynomial_coeff_commitments {
            if polynomial_index >= self.num_polys as PolynomialIndex {
                return Err(errors::PixError::InvalidInput);
            }

            let polynomial = self.committed_polynomials.entry(polynomial_index).or_default();
            polynomial.entry(coeff_index).or_insert(polynomial_coeff_commitment);
        }

        tracing::trace!(
            id = %self.id,
            "SSA commitment is {:.2}% complete",
            self.committed_polynomials.values().map(|p| p.len()).sum::<usize>() as f64 * 100.0 / (self.num_polys * self.poly_threshold) as f64
        );

        // Check if we already have all the committed polynomials and all coefficient commitments in them
        self.complete = self.committed_polynomials.len() == self.num_polys
            && self
                .committed_polynomials
                .values()
                .all(|committed_poly| committed_poly.len() == self.poly_threshold);

        let all_constant_terms_committed = self.committed_polynomials.len() == self.num_polys
            && self
                .committed_polynomials
                .values()
                .all(|committed_poly| committed_poly.get(&0).is_some());

        if self.complete {
            tracing::debug!("SSA is fully committed for verification");

            let complete_ssa_verifier = self
                .committed_polynomials
                .drain()
                .map(|(polynomial_index, mut polynomial)| {
                    PartialSsaShareVerifier::from_serializable_commitments(
                        SsaPolynomialId::new(self.id, polynomial_index),
                        (0..self.poly_threshold as CoefficientIndex)
                            .map(|coeff_idx| {
                                polynomial
                                    .remove(&coeff_idx)
                                    .expect("polynomial coeffs must be already present")
                            })
                            .collect(),
                    )
                })
                .map(|v| v.map(SsaPartBuilder::new))
                .collect::<errors::Result<Vec<_>, S::Pseudonym>>()?;

            let full_commitment = match self.full_ssa_commitment.as_ref().map(|(c, _)| *c) {
                None => {
                    // Full client SSA commitment is the sum of all constant term commitments on all polynomials
                    let client_ssa_commitment: PixGroup<S> =
                        complete_ssa_verifier.iter().map(|v| v.verifier.constant_term()).sum();
                    tracing::debug!(id = %self.id, commitment = const_hex::encode(client_ssa_commitment.to_bytes()), "SSA client commitment");

                    let full_ssa_commitment = client_ssa_commitment + self.exit_commitment_public;

                    // Treat the failed conversion to deposit address as error
                    let deposit_addr =
                        S::group_to_deposit_address(full_ssa_commitment).ok_or(errors::PixError::InvalidInput)?;

                    self.full_ssa_commitment = Some((full_ssa_commitment, deposit_addr));
                    full_ssa_commitment
                }
                Some(commitment) => commitment,
            };

            Ok(CommitmentResult::Completed(
                SsaBuilder::new(full_commitment, self.exit_commitment_secret, self.num_polys),
                complete_ssa_verifier,
            ))
        } else if self.full_ssa_commitment.is_none() && all_constant_terms_committed {
            // Check if we already have at least all the constant term commitments on all polynomials.
            tracing::debug!("SSA commitment is complete");

            let client_ssa_commitment = self
                .committed_polynomials
                .values()
                .map(|p| p.get(&0).expect("constant term must be present"))
                .map(|const_term: &PixGroupRepr<S>| {
                    Option::<PixGroup<S>>::from(PixGroup::<S>::from_bytes(const_term))
                        .ok_or(errors::PixError::InvalidInput)
                })
                .sum::<errors::Result<PixGroup<S>, S::Pseudonym>>()?;
            tracing::debug!(id = %self.id, commitment = const_hex::encode(client_ssa_commitment.to_bytes()), "SSA client commitment");

            let full_ssa_commitment = client_ssa_commitment + self.exit_commitment_public;

            // Treat the failed conversion to deposit address as error
            let deposit_addr =
                S::group_to_deposit_address(full_ssa_commitment).ok_or(errors::PixError::InvalidInput)?;

            self.full_ssa_commitment = Some((full_ssa_commitment, deposit_addr));

            Ok(CommitmentResult::SsaCommitmentDone(full_ssa_commitment))
        } else if let Some(ssa_committed) = self.full_ssa_commitment.as_ref().map(|(c, _)| c) {
            Ok(CommitmentResult::StillIncomplete(*ssa_committed))
        } else {
            Ok(CommitmentResult::NotEnoughCommitments)
        }
    }
}
