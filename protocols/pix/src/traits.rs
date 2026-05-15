use hopr_types::{
    crypto::prelude::{HalfKeyChallenge, OffchainPublicKey},
    internal::prelude::Acknowledgement,
};

use crate::{
    CoefficientIndex, PixGroupRepr, PixScalar, PixSpec, PolynomialIndex, SsaId, TaggedEncryptedPartialSsaShare,
};

#[auto_impl::auto_impl(&, Arc, Box)]
pub trait ExitAcknowledgementShareProcessor<S: PixSpec> {
    type Error: std::error::Error + Send + Sync + 'static;

    fn insert_coefficient_commitments(
        &self,
        index: CoefficientIndex,
        commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> Result<CommitmentInsertionResult<S>, Self::Error>;

    fn insert_encrypted_share<T: Into<PixScalar<S>>>(
        &self,
        peer: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        tagged_enc_share: TaggedEncryptedPartialSsaShare<S, T>,
    ) -> Result<(), Self::Error>;

    fn acknowledge_shares(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>,
    ) -> Result<Vec<PixScalar<S>>, Self::Error>;
}

/// Possible results of [`ExitAcknowledgementShareProcessor::insert_coefficient_commitments`].
#[derive(strum::EnumTryAs, strum::EnumIs)]
pub enum CommitmentInsertionResult<S: PixSpec> {
    /// The commitment was successfully inserted but triggered no additional action.
    NoAction,
    /// Emitted when a new SSA is encountered for the first time.
    NewSsa(SsaId<S>),
    /// Emitted when the commitment to an SSA is known and can be checked for deposit already.
    SsaCommitmentKnown(SsaId<S>, PixGroupRepr<S>),
    /// Emitted when a new SSA is completely committed to by the client and can therefore
    /// be used to for RP traffic.
    FullyCommitted(SsaId<S>, PixGroupRepr<S>),
}
