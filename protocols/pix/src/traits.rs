use hopr_types::{
    crypto::prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    internal::prelude::Acknowledgement,
};

use crate::{
    CoefficientIndex, EncryptedPartialSsaShare, PartialSsaShare, PartialSsaShareVerifier, PixGroup, PixGroupRepr,
    PixScalar, PixSpec, PolynomialIndex, SsaId, SsaPolynomialId, TaggedEncryptedPartialSsaShare, errors,
};

pub struct SsaCommitmentState<S: PixSpec> {
    pub ssa_id: SsaId<S>,
    pub ssa_commitment: Option<PixGroupRepr<S>>,
    pub is_fully_committed: bool,
    pub is_first_encountered: bool,
}

impl<S: PixSpec> SsaCommitmentState<S> {
    pub fn new(ssa_id: SsaId<S>) -> Self {
        Self {
            ssa_id,
            ssa_commitment: None,
            is_fully_committed: false,
            is_first_encountered: true,
        }
    }
}

pub struct RecoveredSsa<S: PixSpec> {
    pub ssa_id: SsaId<S>,
    pub ssa: PixScalar<S>,
}

/// Allows reconstruction of SSAs at the Exit node.
///
/// There are 3 inputs that the implementor is dependent on (in order):
/// 1. SSA commitments from the Client (delivered via
///    [`insert_coefficient_commitments`](ExitAcknowledgementShareProcessor::insert_coefficient_commitments))
/// 2. Extraction of pending encrypted shares (added via
///    [`insert_encrypted_share`](ExitAcknowledgementShareProcessor::insert_encrypted_share)
/// 3. Decryption of pending encrypted shares via [`Acknowledgements`](Acknowledgements) (via
///    [`acknowledge_shares`](ExitAcknowledgementShareProcessor::acknowledge_shares))
#[auto_impl::auto_impl(&, Arc, Box)]
pub trait ExitAcknowledgementShareProcessor<S: PixSpec> {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Adds the client commitment data.
    ///
    /// Each "data packet" should contain an `ssa_id` of the corresponding SSA. The `index` is
    /// the polynomial coefficient index that is common to all the polynomial coefficient commitments included in
    /// `commitments`. In other words, the `commitments` argument contains commitments to
    /// the same polynomial coefficients across multiple polynomials (each one with its own polynomial index).
    fn insert_coefficient_commitments(
        &self,
        ssa_id: SsaId<S>,
        index: CoefficientIndex,
        commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> Result<SsaCommitmentState<S>, Self::Error>;

    /// Adds an encrypted partial SSA share awaiting acknowledgement from `peer` to be decrypted.
    ///
    /// The `challenge` is the acknowledgement challenge that must correspond to the
    /// acknowledgement that will be awaited.
    fn insert_encrypted_share(
        &self,
        peer: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        tagged_enc_share: TaggedEncryptedPartialSsaShare<S>,
    ) -> Result<(), Self::Error>;

    /// Finds and acknowledges previously inserted encrypted share, using incoming [`Acknowledgements`](Acknowledgement)
    /// from the upstream [`peer`](OffchainPublicKey).
    ///
    /// Function should first check if any acknowledgements are expected from the given `peer`.
    ///
    /// Furthermore, the function must verify each given acknowledgement and find if it evaluates to any solutions
    /// to challenges of previously [inserted encrypted
    /// shares](ExitAcknowledgementShareProcessor::insert_encrypted_share).
    ///
    /// On success, the [resolutions](RecoveredSsa) contain any fully recovered SSA shares that were completed as result
    /// of the given acknowledgements.
    ///
    /// Challenges for which tickets were not found are skipped.
    ///
    /// Must return an error if no `Acknowledgements` from the given `peer` were expected.
    ///
    /// This operation is expected to be somewhat long-running and significantly blocking.
    fn acknowledge_shares(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>,
    ) -> Result<Vec<RecoveredSsa<S>>, Self::Error>;
}

/// Contains a generated share from a specific previously committed SSA.
pub struct GeneratedShare<S: PixSpec> {
    pub id: SsaPolynomialId<S>,
    pub share: PartialSsaShare<S>,
}

impl<S: PixSpec> GeneratedShare<S> {
    #[inline]
    pub fn encrypt(self, ack: &HalfKey) -> errors::Result<EncryptedPartialSsaShare<S>> {
        self.share.encrypt(&self.id, ack)
    }
}

/// Contains commitment to a specific SSA and corresponding verifier.
pub struct SsaCommitment<S: PixSpec> {
    pub ssa_id: SsaId<S>,
    pub ssa_commitment: PixGroup<S>,
    pub verifiers: Vec<PartialSsaShareVerifier<S>>,
}

#[auto_impl::auto_impl(&, Arc, Box)]
pub trait EntryShareGenerator<S: PixSpec> {
    type Error: std::error::Error + Send + Sync + 'static;

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
    ) -> Result<Option<GeneratedShare<S>>, Self::Error>;
    /// Generates a new SSA commitment from the sender side, for the given `pseudonym`.
    ///
    /// Returns the new random SSA-commitment and the corresponding SSA share verifier.
    fn new_ssa_commitment(&self, pseudonym: &S::Pseudonym) -> Result<SsaCommitment<S>, Self::Error>;
}
