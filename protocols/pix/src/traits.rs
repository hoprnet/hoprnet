use hopr_types::{
    crypto::prelude::{HalfKeyChallenge, OffchainPublicKey},
    internal::prelude::Acknowledgement,
};

use crate::{
    CoefficientIndex, GeneratedShare, PixGroup, PixGroupRepr, PixSpec, PolynomialIndex, RecoveredSsa, SsaCommitment,
    SsaCommitmentState, SsaId, SsaIndex, SsaRecoveryProgress, TaggedEncryptedPartialSsaShare,
};

/// Possible resolutions of a received acknowledgement that might be bound to decrypt
/// an encrypted PIX share.
///
/// `P` is the pseudonym type, `A` is the private key type for SSA.
///
/// ## Equality semantics
///
/// [`Progress`] and [`InvalidShares`] carry absolute counters and are never
/// deduplicated — the reconstructor emits at most one of each per (peer, SSA) per
/// batch, so derive-based `PartialEq`/`Eq`/`Hash` is fine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::EnumTryAs)]
pub enum ShareResolution<P, A> {
    /// Full SSA was recovered.
    RecoveredSsa(RecoveredSsa<P, A>),
    /// The early recovery threshold was reached (SSA almost complete).
    AlmostRecoveredSsa(SsaId<P>),
    /// Absolute per-SSA recovery progress after processing this batch.
    Progress(SsaRecoveryProgress<P>),
    /// Invalid (unverifiable) shares encountered for a given peer and SSA.
    ///
    /// `observed_total` is the **cross-peer aggregate** count of invalid shares
    /// observed for this SSA so far (not a batch delta). The `peer` field is
    /// retained for attribution/telemetry; limit enforcement uses the aggregate.
    InvalidShares {
        peer: Box<OffchainPublicKey>,
        ssa_id: SsaId<P>,
        observed_total: u64,
    },
}

/// Type alias for a collection of [`ShareResolution`]s.
pub type ShareResolutions<S> = Vec<ShareResolution<<S as PixSpec>::Pseudonym, <S as PixSpec>::AddressPrivateKey>>;

/// Allows reconstruction of SSAs at the Exit node.
///
/// There are 3 inputs that the implementor is dependent on (in order):
/// 1. SSA commitments from the Client (delivered via
///    [`insert_coefficient_commitments`](ExitAcknowledgementShareProcessor::insert_coefficient_commitments))
/// 2. Extraction of pending encrypted shares (added via
///    [`insert_encrypted_share`](ExitAcknowledgementShareProcessor::insert_encrypted_share)
/// 3. Decryption of pending encrypted shares via [`Acknowledgement`]s (via
///    [`acknowledge_shares`](ExitAcknowledgementShareProcessor::acknowledge_shares))
#[auto_impl::auto_impl(&, Arc, Box)]
pub trait ExitAcknowledgementShareProcessor<S: PixSpec> {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Generates a new random Exit SSA commitment and registers it internally under the given `id`.
    fn new_exit_commitment(
        &self,
        id: SsaId<S::Pseudonym>,
        polys_per_ssa: usize,
        shares_per_poly: usize,
    ) -> Result<PixGroup<S>, Self::Error>;

    /// Adds the client commitment data.
    ///
    /// Each "data packet" should contain an `ssa_id` of the corresponding SSA. The `index` is
    /// the polynomial coefficient index that is common to all the polynomial coefficient commitments included in
    /// `commitments`. In other words, the `commitments` argument contains commitments to
    /// the same polynomial coefficients across multiple polynomials (each one with its own polynomial index).
    fn insert_coefficient_commitments(
        &self,
        ssa_id: SsaId<S::Pseudonym>,
        index: CoefficientIndex,
        commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> Result<SsaCommitmentState<S::Pseudonym, S::DepositAddress>, Self::Error>;

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

    /// Finds and acknowledges previously inserted encrypted share, using incoming [`Acknowledgement`]s
    /// from the upstream [`peer`](OffchainPublicKey).
    ///
    /// Function should first check if any acknowledgements are expected from the given `peer`.
    ///
    /// Furthermore, the function must verify each given acknowledgement and find if it evaluates to any solutions
    /// to challenges of previously
    /// [inserted encrypted shares](ExitAcknowledgementShareProcessor::insert_encrypted_share).
    ///
    /// On success, the [resolutions](ShareResolution) contain any fully recovered SSA shares that were completed as
    /// result of the given acknowledgements, or particular cases that lead to invalid (unverifiable) share. That
    /// might indicate faulty behavior of the Entry, or a malicious attempt to disrupt the protocol.
    ///
    /// Challenges for which encrypted shares were not found are skipped.
    ///
    /// Must return an error if no acknowledgements from the given `peer` were expected.
    ///
    /// This operation is expected to be somewhat long-running and significantly blocking.
    fn acknowledge_shares(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>,
    ) -> Result<ShareResolutions<S>, Self::Error>;

    /// Retires (invalidates) all internal state for the given `ssa_id`.
    ///
    /// Removes the SSA commitment builder, all polynomial verifiers, and the
    /// per-SSA counter entry. Idempotent — calling twice or for an unknown SSA
    /// is a no-op.
    fn retire_ssa(&self, ssa_id: &SsaId<S::Pseudonym>);
}

#[auto_impl::auto_impl(&, Arc, Box)]
pub trait EntryShareGenerator<S: PixSpec> {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Generate the next [`crate::PartialSsaShare`] for the given pseudonym and message `msg`.
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
    fn new_ssa_commitment(
        &self,
        pseudonym: &S::Pseudonym,
        ssa_index: SsaIndex,
    ) -> Result<SsaCommitment<S>, Self::Error>;
}
