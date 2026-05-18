use vsss_rs::{
    ReadableShareSet,
    elliptic_curve::group::{Group, GroupEncoding},
};

use crate::{
    CoefficientIndex, CompletedShare, PartialSsaShare, PartialSsaShareVerifier, PixGroup, PixGroupRepr, PixScalar,
    PixSpec, PolynomialIndex, SsaPolynomialId, errors, into_completed_share, types::SsaId,
};

/// Reconstruct a single SSA from a set of SSA parts recovered from polynomials.
pub struct SsaBuilder<S: PixSpec> {
    pub commitment: PixGroupRepr<S>,
    num_polys: usize,
    builder: PixScalar<S>,
}

impl<S: PixSpec> SsaBuilder<S> {
    pub fn new(commitment: PixGroup<S>, num_polys: usize) -> Self {
        Self {
            commitment: commitment.to_bytes(),
            builder: PixScalar::<S>::default(),
            num_polys,
        }
    }

    pub fn add_recovered_ssa_part(&mut self, sub_secret: PixScalar<S>) -> errors::Result<Option<PixScalar<S>>> {
        if let Some(n) = self.num_polys.checked_sub(1) {
            self.num_polys = n;
            self.builder += sub_secret;
            if n > 0 {
                // SSA private scalar is not yet complete
                return Ok(None);
            }
        }

        if self.commitment == (PixGroup::<S>::generator() * self.builder).to_bytes() {
            Ok(Some(self.builder))
        } else {
            Err(errors::PixError::InvalidSsa)
        }
    }
}

/// Verifies shares and reconstructs a single SSA part from them.
pub struct SsaPartBuilder<S: PixSpec> {
    pub verifier: PartialSsaShareVerifier<S>,
    shares: Vec<CompletedShare<S>>,
}

impl<S: PixSpec> SsaPartBuilder<S> {
    pub fn new(verifier: PartialSsaShareVerifier<S>) -> Self {
        Self {
            verifier,
            shares: Vec::new(),
        }
    }

    pub fn add_share(&mut self, msg: PixScalar<S>, share: PartialSsaShare<S>) -> errors::Result<Option<PixScalar<S>>> {
        let share = into_completed_share(msg, &share)?;

        self.verifier.verify_completed_share(&share)?;
        self.shares.push(share);

        if self.shares.len() >= self.verifier.min_shares() {
            Ok(Some(self.shares.combine()?.0))
        } else {
            Ok(None)
        }
    }
}

type CommittedPolynomial<S> = std::collections::HashMap<CoefficientIndex, PixGroupRepr<S>>;

/// Result of building an SSA commitment.
pub enum CommitmentResult<S: PixSpec> {
    /// Not enough commitments have been received yet.
    NotEnoughCommitments,
    /// There are enough commitments to build at least the SSA commitment.
    SsaCommitmentDone(PixGroupRepr<S>),
    /// There are enough commitments to build at least the SSA commitment, but not all coefficients are committed yet.
    StillIncomplete(PixGroupRepr<S>),
    /// All coefficients have been committed.
    Completed(SsaBuilder<S>, Vec<SsaPartBuilder<S>>),
}

/// Builds [`CommittedSsa`] from the incoming client polynomial coefficient commitments of
/// SSA-part polynomials for a specific Session Stealth Address (SSA).
pub struct SsaCommitmentBuilder<S: PixSpec> {
    id: SsaId<S>,
    poly_threshold: usize,
    num_polys: usize,
    committed_polynomials: std::collections::HashMap<PolynomialIndex, CommittedPolynomial<S>>,
    complete: bool,
    ssa_committed: Option<PixGroupRepr<S>>,
}

impl<S: PixSpec> SsaCommitmentBuilder<S> {
    pub fn new(id: SsaId<S>, poly_threshold: usize, num_polys: usize) -> Self {
        Self {
            id,
            poly_threshold,
            num_polys,
            committed_polynomials: std::collections::HashMap::new(),
            complete: false,
            ssa_committed: None,
        }
    }

    pub fn add_transposed(
        &mut self,
        coeff_index: CoefficientIndex,
        polynomial_coeff_commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> errors::Result<CommitmentResult<S>> {
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
            tracing::debug!("SSA is fully committed");

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
                .collect::<errors::Result<Vec<_>>>()?;

            // Full SSA commitment is the sum of all constant term commitments on all polynomials
            let full_ssa_commitment: PixGroup<S> =
                complete_ssa_verifier.iter().map(|v| v.verifier.constant_term()).sum();
            tracing::debug!(id = %self.id, commitment = hex::encode(full_ssa_commitment.to_bytes()), "SSA client commitment");

            Ok(CommitmentResult::Completed(
                SsaBuilder::new(full_ssa_commitment, self.num_polys),
                complete_ssa_verifier,
            ))
        } else if self.ssa_committed.is_none() && all_constant_terms_committed {
            // Check if we already have at least all the constant term commitments on all polynomials.
            tracing::debug!("SSA commitment is complete");

            let full_ssa_commitment = self
                .committed_polynomials
                .values()
                .map(|p| p.get(&0).expect("constant term must be present"))
                .map(|const_term: &PixGroupRepr<S>| {
                    Option::<PixGroup<S>>::from(PixGroup::<S>::from_bytes(const_term))
                        .ok_or(errors::PixError::InvalidInput)
                })
                .sum::<errors::Result<PixGroup<S>>>()?
                .to_bytes();

            self.ssa_committed = Some(full_ssa_commitment);
            Ok(CommitmentResult::SsaCommitmentDone(full_ssa_commitment))
        } else if let Some(ssa_committed) = self.ssa_committed.as_ref() {
            Ok(CommitmentResult::StillIncomplete(*ssa_committed))
        } else {
            Ok(CommitmentResult::NotEnoughCommitments)
        }
    }
}
