use std::hash::Hash;

use hopr_types::{
    crypto::prelude::{HalfKeyChallenge, OffchainPublicKey},
    internal::prelude::Acknowledgement,
};

use crate::{
    CoefficientIndex, GeneratedShare, PixGroup, PixGroupRepr, PixSpec, PolynomialIndex, RecoveredSsa, SsaCommitment,
    SsaCommitmentProof, SsaCommitmentState, SsaId, SsaIndex, SsaRecoveryProgress, TaggedEncryptedPartialSsaShare,
};

/// Possible resolutions of a received acknowledgement that might be bound to decrypt
/// an encrypted PIX share.
///
/// `P` is the pseudonym type, `A` is the private key type for SSA.
///
/// ## Ordering and multiplicity within one batch
///
/// [`acknowledge_shares`](ExitAcknowledgementShareProcessor::acknowledge_shares) emits at most one
/// [`Progress`](Self::Progress) and at most one [`InvalidShares`](Self::InvalidShares) per SSA per
/// call, and orders them ahead of the terminal
/// [`AlmostRecoveredSsa`](Self::AlmostRecoveredSsa)/[`RecoveredSsa`](Self::RecoveredSsa) for the
/// same SSA. A consumer can therefore act on a terminal event knowing the counters it would have
/// wanted first have already been delivered.
#[derive(Clone, strum::EnumTryAs)]
pub enum ShareResolution<P, A> {
    /// Full SSA was recovered.
    RecoveredSsa(RecoveredSsa<P, A>),
    /// The early recovery threshold was reached (SSA almost complete).
    AlmostRecoveredSsa(SsaId<P>),
    /// Absolute recovery progress for one SSA after this batch.
    Progress(SsaRecoveryProgress<P>),
    /// Invalid (unverifiable) shares were encountered for an SSA.
    ///
    /// `observed_total` is the **cross-peer aggregate** for the SSA, not a delta and not this peer's
    /// share of it: the cycle is a single unit of accounting and any relayer can carry shares for it.
    /// `peer` identifies who relayed the share that triggered this emission, for attribution only.
    InvalidShares {
        /// Relayer whose acknowledgement carried the offending share.
        peer: Box<OffchainPublicKey>,
        /// SSA the offending share belongs to.
        ssa_id: SsaId<P>,
        /// Total invalid shares seen for this SSA across all peers.
        observed_total: u64,
    },
}

impl<P: PartialEq, A> PartialEq for ShareResolution<P, A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::RecoveredSsa(a), Self::RecoveredSsa(b)) => a == b,
            (Self::AlmostRecoveredSsa(a), Self::AlmostRecoveredSsa(b)) => a == b,
            (Self::Progress(a), Self::Progress(b)) => a == b,
            (
                Self::InvalidShares {
                    peer: p1,
                    ssa_id: id1,
                    observed_total: t1,
                },
                Self::InvalidShares {
                    peer: p2,
                    ssa_id: id2,
                    observed_total: t2,
                },
            ) => p1 == p2 && id1 == id2 && t1 == t2,
            _ => false,
        }
    }
}

impl<P: Eq, A> Eq for ShareResolution<P, A> {}

impl<P: std::fmt::Debug, A> std::fmt::Debug for ShareResolution<P, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecoveredSsa(ssa) => f.debug_tuple("RecoveredSsa").field(ssa).finish(),
            Self::AlmostRecoveredSsa(id) => f.debug_tuple("AlmostRecoveredSsa").field(id).finish(),
            Self::Progress(progress) => f.debug_tuple("Progress").field(progress).finish(),
            Self::InvalidShares {
                peer,
                ssa_id,
                observed_total,
            } => f
                .debug_struct("InvalidShares")
                .field("peer", peer)
                .field("ssa_id", ssa_id)
                .field("observed_total", observed_total)
                .finish(),
        }
    }
}

impl<P: std::hash::Hash, A> Hash for ShareResolution<P, A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::RecoveredSsa(recovered) => recovered.hash(state),
            Self::AlmostRecoveredSsa(id) => id.hash(state),
            Self::Progress(progress) => progress.hash(state),
            Self::InvalidShares {
                peer,
                ssa_id,
                observed_total,
            } => {
                peer.hash(state);
                ssa_id.hash(state);
                observed_total.hash(state);
            }
        }
    }
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

    /// Returns `true` if the peer has pending encrypted shares awaiting an acknowledgement.
    ///
    /// This is a cheap, non-blocking check used by the pipeline to avoid spawning a
    /// blocking thread-pool task for `acknowledge_shares` when there are no pending shares
    /// for the peer. Implementations should return `false` when no shares are pending.
    ///
    /// The default implementation returns `true` for safety — callers must still handle
    /// the case where `acknowledge_shares` returns no results.
    fn has_pending_shares(&self, _peer: &OffchainPublicKey) -> bool {
        true
    }

    /// Returns `true` if the given error is an expected "not for us" skip (e.g. no
    /// acknowledgements from the peer were expected), so the caller can log it at a
    /// lower severity.
    ///
    /// The default implementation returns `false`. Implementations with a concrete
    /// error type should override this to identify expected error variants.
    fn is_expected_error(&self, _error: &Self::Error) -> bool {
        false
    }

    /// Releases all reconstructor state for a finished or torn-down SSA cycle.
    ///
    /// Called on full recovery and on session closure. Implementations must be
    /// idempotent — retiring an unknown or already-retired cycle is a no-op.
    fn retire_ssa(&self, ssa_id: SsaId<S::Pseudonym>);

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
    ///
    /// `proof` carries the sender's [`SsaCommitmentProof`] and is expected on messages delivering
    /// constant terms (`index == 0`), since those are what determine the commitment it opens. The
    /// first one supplied for an SSA is kept and the rest ignored — any single valid proof suffices.
    /// A cycle whose constant terms are all present but which never carried a valid proof is
    /// rejected, and no deposit address is published for it.
    fn insert_coefficient_commitments(
        &self,
        ssa_id: SsaId<S::Pseudonym>,
        index: CoefficientIndex,
        proof: Option<SsaCommitmentProof<S>>,
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

#[cfg(test)]
mod tests {
    use hopr_types::{
        crypto::{
            keypairs::Keypair,
            prelude::{ChainKeypair, SimplePseudonym},
        },
        crypto_random::Randomizable,
    };

    use super::*;

    #[test]
    fn debug_redaction_share_resolution_recovered_ssa() {
        // ShareResolution::RecoveredSsa wraps RecoveredSsa; the nested Debug
        // must preserve the secret redaction.
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into().unwrap());
        let dummy_key = ChainKeypair::random();
        let recovered = RecoveredSsa { ssa_id, ssa: dummy_key };
        let recovered_debug = format!("{:?}", recovered);
        let resolution = ShareResolution::RecoveredSsa(recovered);
        let debug = format!("{:?}", resolution);

        assert!(debug.contains("RecoveredSsa"));
        // The outer tuple wraps the inner RecoveredSsa Debug, which redacts ssa
        assert_eq!(
            debug,
            format!("RecoveredSsa({recovered_debug})"),
            "ShareResolution::RecoveredSsa Debug must perfectly delegate to RecoveredSsa Debug"
        );
    }
}
