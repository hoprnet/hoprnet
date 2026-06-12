mod utils;

use ahash::HashSetExt;
use hopr_types::{
    crypto::prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    internal::prelude::Acknowledgement,
};
use utils::{CommitmentResult, SsaBuilder, SsaCommitmentBuilder, SsaPartBuilder};
use vsss_rs::elliptic_curve::{Field, ops::MulByGenerator};

use crate::{
    CoefficientIndex, ExitAcknowledgementShareProcessor, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, PixGroup, PixGroupRepr,
    PixScalar, PixSpec, PolynomialIndex, RecoveredSsa, ShareResolution, SsaCommitmentState, SsaPolynomialId,
    TaggedEncryptedPartialSsaShare, errors::PixError, types::SsaId,
};

/// Configuration for the SSA reconstructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
pub struct SsaReconstructorConfig {
    /// Maximum time an SSA can be incomplete before it is discarded.
    ///
    /// Default is 10 minutes.
    #[default(std::time::Duration::from_secs(600))]
    pub incomplete_ssa_lifetime: std::time::Duration,
    /// Time until the complete commitment to an SSA must be received.
    ///
    /// Default is 2 minutes.
    #[default(std::time::Duration::from_secs(120))]
    pub incomplete_commitment_lifetime: std::time::Duration,
    /// Maximum time a verifier can be unused before it is discarded.
    ///
    /// Default is 30 minutes.
    #[default(std::time::Duration::from_secs(1800))]
    pub unused_verifier_lifetime: std::time::Duration,
    /// Maximum number of awaited acknowledgements to extract a single share.
    ///
    /// Default is 10 000 000, must be at least 10 000.
    #[default(10_000_000)]
    #[validate(range(min = 10000))]
    pub max_awaiting_acks: usize,
    /// Maximum time an acknowledgement can be awaited before it is discarded.
    ///
    /// Default is 30 seconds.
    #[default(std::time::Duration::from_secs(30))]
    pub max_ack_await_time: std::time::Duration,
    /// Indicates whether to use batch verification algorithm for acknowledgements.
    ///
    /// This has a positive performance impact on higher workloads.
    ///
    /// Default is true.
    #[default(true)]
    pub use_batch_verification: bool,
}

type EncryptedShareCache<S> =
    moka::sync::Cache<HalfKeyChallenge, TaggedEncryptedPartialSsaShare<S, <S as PixSpec>::Pseudonym, PixScalar<S>>>;

/// Allows server-side reconstruction of SSAs.
///
/// There are 3 inputs that reconstructor is dependent on (in order):
/// 1. SSA commitments from the Client (delivered via
///    [`insert_coefficient_commitments`](ExitAcknowledgementShareProcessor::insert_coefficient_commitments))
/// 2. Extraction of pending encrypted shares (added via
///    [`insert_encrypted_share`](ExitAcknowledgementShareProcessor::insert_encrypted_share)
/// 3. Decryption of pending encrypted shares via [`Acknowledgements`](Acknowledgements) (via
///    [`acknowledge_shares`](ExitAcknowledgementShareProcessor::acknowledge_shares))
///
/// It is able to track SSA for multiple different pseudonyms (Sessions).
pub struct SsaReconstructor<S: PixSpec> {
    commitment_builder:
        moka::sync::Cache<SsaId<S::Pseudonym>, std::sync::Arc<parking_lot::Mutex<SsaCommitmentBuilder<S>>>>,
    ssa_builders: moka::sync::Cache<SsaId<S::Pseudonym>, std::sync::Arc<parking_lot::Mutex<SsaBuilder<S>>>>,
    ssa_verifiers:
        moka::sync::Cache<SsaPolynomialId<S::Pseudonym>, std::sync::Arc<parking_lot::Mutex<SsaPartBuilder<S>>>>,
    awaiting_acks: moka::sync::Cache<OffchainPublicKey, EncryptedShareCache<S>>,
    cfg: SsaReconstructorConfig,
}

type MaybeRecoveredSsa<S> = Option<RecoveredSsa<<S as PixSpec>::Pseudonym, <S as PixSpec>::AddressPrivateKey>>;

impl<S: PixSpec + Clone> SsaReconstructor<S> {
    pub fn new(cfg: SsaReconstructorConfig) -> Self {
        Self {
            commitment_builder: moka::sync::CacheBuilder::new(10)
                .time_to_idle(cfg.incomplete_commitment_lifetime)
                .build(),
            ssa_builders: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                .time_to_idle(cfg.incomplete_ssa_lifetime)
                .build(),
            ssa_verifiers: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            awaiting_acks: moka::sync::CacheBuilder::new(cfg.max_awaiting_acks as u64)
                .time_to_live(cfg.max_ack_await_time)
                .build(),
            cfg,
        }
    }

    fn process_verified_ack(
        &self,
        ack: HalfKey,
        ack_challenge: HalfKeyChallenge,
        awaiting_ack_from_peer: &moka::sync::Cache<HalfKeyChallenge, TaggedEncryptedPartialSsaShare<S>>,
    ) -> Result<MaybeRecoveredSsa<S>, PixError<S::Pseudonym>> {
        let Some(share) = awaiting_ack_from_peer.remove(&ack_challenge) else {
            tracing::trace!(?ack_challenge, "received ack for unknown share");
            return Ok(None);
        };

        let spi = share.ssa_polynomial_id().ok_or(PixError::ShareIsEmpty)?;

        let reconstructor = self.ssa_verifiers.get(&spi).ok_or(PixError::MissingVerifier)?;

        // The share cannot be empty at this point, because we prevent empty share insertions
        let partial_share = share.partial_share.decrypt(spi.pseudonym(), &ack)?;

        let ssa_part = match reconstructor.lock().add_share(share.nonce, partial_share) {
            Ok(Some(share)) => {
                tracing::trace!(%spi, "ssa part complete");
                share
            }
            Ok(None) => {
                tracing::trace!(%spi, "ssa part not yet complete, waiting for more shares");
                return Ok(None);
            }
            Err(PixError::VsssError(vsss_rs::Error::InvalidShare)) => {
                // We need to treat this error differently, because it is critical
                // and may be differently handled by the upper-layer components
                tracing::error!(%spi, "share verification failed");
                return Err(PixError::InvalidShare(*spi.pseudonym(), spi.ssa_index()));
            }
            Err(e) => return Err(e),
        };

        let builder = self
            .ssa_builders
            .get(spi.as_ref())
            .ok_or(PixError::MissingSsaCommitment)?;
        let Some(ssa) = builder.lock().add_recovered_ssa_part(ssa_part)? else {
            tracing::trace!(%spi, "ssa not yet complete, waiting for more ssa parts");
            return Ok(None);
        };

        let Some(ssa) = S::scalar_to_private_key(ssa) else {
            tracing::error!(%spi, "ssa reconstruction failed");
            return Err(PixError::InvalidSsa);
        };

        let ssa_id = *spi.as_ref();
        tracing::info!(%ssa_id, "ssa recovered");

        Ok(Some(RecoveredSsa { ssa_id, ssa }))
    }
}

impl<S: PixSpec + Clone> ExitAcknowledgementShareProcessor<S> for SsaReconstructor<S> {
    type Error = PixError<S::Pseudonym>;

    fn new_exit_commitment(
        &self,
        id: SsaId<S::Pseudonym>,
        polys_per_ssa: usize,
        shares_per_poly: usize,
    ) -> Result<PixGroup<S>, Self::Error> {
        if polys_per_ssa > MAX_POLYS_PER_SSA || !(2..=MAX_POLY_THRESHOLD).contains(&shares_per_poly) {
            return Err(PixError::InvalidInput);
        }

        let exit_commitment_secret = PixScalar::<S>::random(vsss_rs::elliptic_curve::rand_core::OsRng);
        let exit_commitment_public = PixGroup::<S>::mul_by_generator(&exit_commitment_secret);

        self.commitment_builder
            .entry(id)
            .and_try_compute_with(|entry| match entry {
                Some(_) => Err(PixError::DuplicateCommitment),
                None => Ok(moka::ops::compute::Op::Put(std::sync::Arc::new(
                    parking_lot::Mutex::new(SsaCommitmentBuilder::new(
                        id,
                        shares_per_poly,
                        polys_per_ssa,
                        exit_commitment_secret,
                        exit_commitment_public,
                    )),
                ))),
            })?;

        Ok(exit_commitment_public)
    }

    fn insert_coefficient_commitments(
        &self,
        ssa_id: SsaId<S::Pseudonym>,
        index: CoefficientIndex,
        commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> Result<SsaCommitmentState<S::Pseudonym, S::DepositAddress>, Self::Error> {
        let mut res = SsaCommitmentState::new(ssa_id);

        // The Server commitment must be present first
        let Some(builder) = self.commitment_builder.get(&ssa_id) else {
            return Err(PixError::MissingSsaCommitment);
        };

        let maybe_complete_ssa_commitment = {
            let mut builder = builder.lock();
            res.is_first_encountered = builder.is_empty();
            builder.add_transposed(index, commitments)?
        };

        match maybe_complete_ssa_commitment {
            CommitmentResult::NotEnoughCommitments => {
                tracing::trace!(%ssa_id, "ssa commitment not yet complete, waiting for more data");
            }
            CommitmentResult::SsaCommitmentDone(full_ssa_commitment) => {
                res.is_deposit_address_fresh_known = true;
                res.ssa_deposit_address =
                    Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);
            }
            CommitmentResult::StillIncomplete(full_ssa_commitment) => {
                res.is_deposit_address_fresh_known = res.ssa_deposit_address.is_none();
                res.ssa_deposit_address =
                    Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);
            }
            CommitmentResult::Completed(ssa_builder, ssa_reconstructors) => {
                let full_ssa_commitment = ssa_builder.full_commitment;
                self.ssa_builders
                    .insert(ssa_id, std::sync::Arc::new(parking_lot::Mutex::new(ssa_builder)));

                for ssa_reconstructor in ssa_reconstructors {
                    self.ssa_verifiers.insert(
                        ssa_reconstructor.verifier.spi,
                        std::sync::Arc::new(parking_lot::Mutex::new(ssa_reconstructor)),
                    );
                }

                res.is_deposit_address_fresh_known = res.ssa_deposit_address.is_none();
                res.ssa_deposit_address =
                    Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);
                res.is_verifiable = true;
            }
        }

        Ok(res)
    }

    fn insert_encrypted_share(
        &self,
        peer: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        tagged_enc_share: TaggedEncryptedPartialSsaShare<S>,
    ) -> Result<(), Self::Error> {
        if tagged_enc_share.partial_share.is_empty() {
            return Err(PixError::ShareIsEmpty);
        }

        self.awaiting_acks
            .get_with_by_ref(peer, || {
                moka::sync::CacheBuilder::new(self.cfg.max_awaiting_acks as u64).build()
            })
            .insert(challenge, tagged_enc_share);

        Ok(())
    }

    fn acknowledge_shares(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>,
    ) -> Result<Vec<ShareResolution<S::Pseudonym, S::AddressPrivateKey>>, Self::Error> {
        let Some((awaiting_ack_from_peer, half_keys_challenges)) = crate::ack_verify::verify_expected_acknowledgements(
            peer,
            acks,
            &self.awaiting_acks,
            self.cfg.use_batch_verification,
        ) else {
            return Err(PixError::UnexpectedShare);
        };

        // Feed output into HashSet, that deduplicates
        let mut res = ahash::HashSet::with_capacity(half_keys_challenges.len());
        for (ack, ack_challenge) in half_keys_challenges {
            match self.process_verified_ack(ack, ack_challenge, &awaiting_ack_from_peer) {
                Ok(Some(ssa)) => {
                    res.insert(ShareResolution::RecoveredSsa(ssa));
                }
                Ok(None) => {}
                Err(PixError::ShareIsEmpty) => tracing::trace!(%peer, "received empty share"),
                Err(PixError::InvalidShare(pseudonym, ssa_index)) => {
                    tracing::error!(%pseudonym, ssa_index, "encountered share that could not be verified");
                    res.insert(ShareResolution::InvalidShare(peer.into(), pseudonym, ssa_index));
                }
                Err(error) => {
                    tracing::error!(%error, "failed to process acknowledgement");
                }
            }
        }

        Ok(res.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hopr_types::{crypto::prelude::*, crypto_random::Randomizable};
    use k256::elliptic_curve::Field;

    use super::*;
    use crate::{DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, PartialSsaShare, tests::TestSpec};

    #[test]
    fn reconstructor_invalid_commitment_inputs() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);

        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // 1. Invalid coefficient index (>= threshold)
        let result = reconstructor.insert_coefficient_commitments(
            ssa_id,
            2, // threshold is 2, so 2 is invalid
            HashMap::new().into_iter(),
        );
        assert!(matches!(result, Err(PixError::InvalidInput)));

        // 2. Invalid polynomial index (>= polys_per_ssa)
        let mut invalid_poly_map = HashMap::new();
        invalid_poly_map.insert(2 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        let result = reconstructor.insert_coefficient_commitments(ssa_id, 0, invalid_poly_map.into_iter());
        assert!(matches!(result, Err(PixError::InvalidInput)));

        Ok(())
    }

    #[test]
    fn reconstructor_should_not_accept_client_commitments_without_priod_exit_commitment() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);

        let mut poly_map = HashMap::new();
        for poly in 0..2 {
            poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        }

        let res = reconstructor.insert_coefficient_commitments(ssa_id, 0, poly_map.into_iter());

        assert!(matches!(res, Err(PixError::MissingSsaCommitment)));

        Ok(())
    }

    #[test]
    fn reconstructor_duplicate_commitments() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);

        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // Fill all commitments
        for coeff in 0..2 {
            let mut poly_map = HashMap::new();
            for poly in 0..2 {
                poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
            }
            reconstructor.insert_coefficient_commitments(ssa_id, coeff as CoefficientIndex, poly_map.into_iter())?;
        }

        // Now adding more should fail with DuplicateCommitment
        let result = reconstructor.insert_coefficient_commitments(ssa_id, 0, HashMap::new().into_iter());
        assert!(matches!(result, Err(PixError::DuplicateCommitment)));

        Ok(())
    }

    #[test]
    fn reconstructor_missing_verifier() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig { ..Default::default() });

        let ack_key = HalfKey::random();
        let challenge = ack_key.to_challenge()?;

        // Add a pending share but NO commitment (so no verifier is created)
        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);
        let spi = SsaPolynomialId::new(ssa_id, 0);

        // We need a valid-looking encrypted share even if it's junk.
        // EncryptedPartialSsaShare is basically a wrapper around bytes.
        let partial_share = PartialSsaShare::default().encrypt(&spi, &ack_key)?;

        let peer = OffchainKeypair::random();
        let nonce = k256::Scalar::random(&mut vsss_rs::elliptic_curve::rand_core::OsRng);

        reconstructor.new_exit_commitment(ssa_id, DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD)?;

        reconstructor.insert_encrypted_share(
            peer.public(),
            challenge,
            TaggedEncryptedPartialSsaShare {
                pseudonym: *spi.pseudonym(),
                nonce,
                partial_share,
            },
        )?;

        let result = reconstructor.process_verified_ack(
            ack_key,
            challenge,
            reconstructor
                .awaiting_acks
                .get(peer.public())
                .as_ref()
                .ok_or(anyhow::anyhow!("missing peer"))?,
        );
        assert!(matches!(result, Err(PixError::MissingVerifier)));

        Ok(())
    }

    #[test]
    fn reconstructor_must_not_accept_empty_encrypted_share() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig { ..Default::default() });

        let ack_key = HalfKey::random();
        let challenge = ack_key.to_challenge()?;

        let peer = OffchainKeypair::random();

        assert!(
            reconstructor
                .insert_encrypted_share(
                    peer.public(),
                    challenge,
                    TaggedEncryptedPartialSsaShare {
                        pseudonym: SimplePseudonym::random(),
                        nonce: Default::default(),
                        partial_share: Default::default(),
                    }
                )
                .is_err()
        );

        Ok(())
    }
}
