mod utils;

use ahash::HashSetExt;
use hopr_types::{
    crypto::{
        crypto_traits::elliptic_curve::Field,
        prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    },
    internal::prelude::Acknowledgement,
};
use utils::{CommitmentResult, SsaBuilder, SsaCommitmentBuilder, SsaPartBuilder};
use validator::Validate;

/// Maximum number of concurrent SSA cycles (current + pipelined + sessions) that
/// can have verifiers in flight before eviction. Past this, late shares from slower
/// cycles get `MissingVerifier` and are silently lost.
const MAX_CONCURRENT_SSA_CYCLES: u64 = 8;

use crate::{
    CoefficientIndex, ExitAcknowledgementShareProcessor, Group, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, PixGroup,
    PixGroupRepr, PixScalar, PixSpec, PolynomialIndex, RecoveredSsa, ShareResolution, SsaCommitmentState,
    SsaPolynomialId, TaggedEncryptedPartialSsaShare, errors::PixError, types::SsaId,
};

/// Configuration for the SSA reconstructor.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, validator::Validate)]
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
    /// Fraction of reconstructed polynomials at which to emit an early recovery
    /// notification, triggering pipelined SSA request preparation.
    ///
    /// Range: 0.0..1.0. Default: 0.85.
    #[default(0.85)]
    #[validate(range(min = 0.0, max = 1.0))]
    pub early_recovery_threshold: f64,
}

type EncryptedShareCache<S> =
    moka::sync::Cache<HalfKeyChallenge, TaggedEncryptedPartialSsaShare<S, <S as PixSpec>::Pseudonym, PixScalar<S>>>;
type PendingAckPerPeerCache = moka::sync::Cache<HalfKeyChallenge, HalfKey>;

/// Allows server-side reconstruction of SSAs.
///
/// There are 3 inputs that reconstructor is dependent on (in order):
/// 1. SSA commitments from the Client (delivered via
///    [`insert_coefficient_commitments`](ExitAcknowledgementShareProcessor::insert_coefficient_commitments))
/// 2. Extraction of pending encrypted shares (added via
///    [`insert_encrypted_share`](ExitAcknowledgementShareProcessor::insert_encrypted_share)
/// 3. Decryption of pending encrypted shares via [`Acknowledgement`]s (via
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
    /// Cache of ack keys whose verifier was not yet available.
    /// `OffchainPublicKey → { HalfKeyChallenge → HalfKey }`. When a verified ack hits
    /// `MissingVerifier`, it is stored here so subsequent `acknowledge_shares` calls can
    /// retry in O(1) per-peer once the verifier arrives. Tied to `max_ack_await_time` in
    /// line with the awaiting_acks TTL.
    pending_ack_keys: moka::sync::Cache<OffchainPublicKey, PendingAckPerPeerCache>,
    cfg: SsaReconstructorConfig,
}

/// Result of processing a single verified acknowledgement in the SSA reconstructor.
enum ProcessedAckResult<S: PixSpec> {
    /// No SSA recovery progress — still waiting for more polynomial parts.
    NoProgress,
    /// The early recovery threshold was crossed (identified by SsaId).
    EarlyRecovery(SsaId<<S as PixSpec>::Pseudonym>),
    /// Full SSA was recovered.
    FullRecovery(RecoveredSsa<<S as PixSpec>::Pseudonym, <S as PixSpec>::AddressPrivateKey>),
}

impl<S: PixSpec + Clone> Default for SsaReconstructor<S> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<S: PixSpec + Clone> SsaReconstructor<S> {
    /// Creates a new SSA reconstructor from the given configuration.
    ///
    /// # Panics
    /// Panics if the configuration fails validation.
    pub fn new(cfg: SsaReconstructorConfig) -> Self {
        cfg.validate().expect("invalid SsaReconstructorConfig");
        Self {
            commitment_builder: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                .time_to_idle(cfg.incomplete_commitment_lifetime)
                .build(),
            ssa_builders: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                .time_to_idle(cfg.incomplete_ssa_lifetime)
                .build(),
            ssa_verifiers: moka::sync::CacheBuilder::new(MAX_CONCURRENT_SSA_CYCLES * (MAX_POLYS_PER_SSA as u64))
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            awaiting_acks: moka::sync::CacheBuilder::new(cfg.max_awaiting_acks as u64)
                .time_to_idle(cfg.max_ack_await_time)
                .build(),
            pending_ack_keys: moka::sync::CacheBuilder::new(MAX_CONCURRENT_SSA_CYCLES)
                .time_to_idle(cfg.max_ack_await_time)
                .build(),
            cfg,
        }
    }

    /// Returns the configuration of the reconstructor.
    #[inline]
    pub fn config(&self) -> &SsaReconstructorConfig {
        &self.cfg
    }

    fn process_verified_ack(
        &self,
        ack: HalfKey,
        ack_challenge: HalfKeyChallenge,
        awaiting_ack_from_peer: &moka::sync::Cache<HalfKeyChallenge, TaggedEncryptedPartialSsaShare<S>>,
    ) -> Result<ProcessedAckResult<S>, PixError<S::Pseudonym>> {
        let Some(share) = awaiting_ack_from_peer.get(&ack_challenge) else {
            tracing::trace!(?ack_challenge, "received ack for unknown share");
            return Ok(ProcessedAckResult::NoProgress);
        };

        let spi = share.ssa_polynomial_id().ok_or(PixError::ShareIsEmpty)?;

        let reconstructor = self.ssa_verifiers.get(&spi).ok_or(PixError::MissingVerifier)?;

        // Verifier confirmed — safe to remove the share from the pending cache.
        awaiting_ack_from_peer.remove(&ack_challenge);

        // The share cannot be empty at this point because we prevent empty share insertions
        let partial_share = share.partial_share.decrypt(spi.pseudonym(), &ack)?;

        let ssa_part = match reconstructor.lock().add_share(share.nonce, partial_share) {
            Ok(Some(share)) => {
                tracing::trace!(%spi, "ssa part complete");
                share
            }
            Ok(None) => {
                tracing::trace!(%spi, "ssa part not yet complete, waiting for more shares");
                return Ok(ProcessedAckResult::NoProgress);
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

        let mut builder_guard = builder.lock();
        let ssa = builder_guard.add_recovered_ssa_part(spi.poly_index(), ssa_part)?;
        match ssa {
            Some(scalar) => {
                let Some(ssa) = S::scalar_to_private_key(scalar) else {
                    tracing::error!(%spi, "ssa reconstruction failed");
                    return Err(PixError::InvalidSsa);
                };
                let ssa_id = *spi.as_ref();
                tracing::info!(%ssa_id, "ssa recovered");
                Ok(ProcessedAckResult::FullRecovery(RecoveredSsa { ssa_id, ssa }))
            }
            None => {
                tracing::trace!(%spi, "ssa not yet complete, waiting for more ssa parts");
                // Check early threshold while we hold the lock
                if builder_guard.check_early_threshold(self.cfg.early_recovery_threshold) {
                    let ssa_id = *spi.as_ref();
                    tracing::info!(%ssa_id, "early recovery threshold reached");
                    Ok(ProcessedAckResult::EarlyRecovery(ssa_id))
                } else {
                    Ok(ProcessedAckResult::NoProgress)
                }
            }
        }
    }
}

impl<S: PixSpec + Clone> ExitAcknowledgementShareProcessor<S> for SsaReconstructor<S> {
    type Error = PixError<S::Pseudonym>;

    fn has_pending_shares(&self, peer: &OffchainPublicKey) -> bool {
        self.awaiting_acks.contains_key(peer)
    }

    fn new_exit_commitment(
        &self,
        id: SsaId<S::Pseudonym>,
        polys_per_ssa: usize,
        shares_per_poly: usize,
    ) -> Result<PixGroup<S>, Self::Error> {
        if !(1..=MAX_POLYS_PER_SSA as usize).contains(&polys_per_ssa)
            || !(2..=MAX_POLY_THRESHOLD as usize).contains(&shares_per_poly)
        {
            return Err(PixError::InvalidInput);
        }

        let exit_commitment_secret = PixScalar::<S>::random(&mut hopr_types::crypto_random::rng());
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
            res.ssa_deposit_address = builder.get_deposit_address().copied();
            builder.add_transposed(index, commitments)?
        };

        res.deposit_address_first_encountered = res.ssa_deposit_address.is_none();

        match maybe_complete_ssa_commitment {
            CommitmentResult::NotEnoughCommitments => {
                res.deposit_address_first_encountered = false; // Not yet encountered
                tracing::trace!(%ssa_id, "ssa commitment not yet complete, waiting for more data");
            }
            CommitmentResult::SsaCommitmentDone(full_ssa_commitment) => {
                res.ssa_deposit_address =
                    Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);

                tracing::trace!(%ssa_id, "ssa commitment done");
            }
            CommitmentResult::StillIncomplete(full_ssa_commitment) => {
                res.ssa_deposit_address =
                    Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);

                tracing::trace!(%ssa_id, "ssa commitment still incomplete");
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

                res.ssa_deposit_address =
                    Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);
                res.is_verifiable = true;

                tracing::trace!(%ssa_id, "ssa commitment completed");
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
                // Inner cache keyed by HalfKeyChallenge — each entry gets its own TTL
                // so a late-arriving share gets the full max_ack_await_time window.
                moka::sync::CacheBuilder::new(self.cfg.max_awaiting_acks as u64)
                    .time_to_live(self.cfg.max_ack_await_time)
                    .build()
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

        // Drain pending retries from previous calls — the verifier may have
        // been inserted since the last acknowledge_shares invocation.
        if let Some(per_peer) = self.pending_ack_keys.get(&peer) {
            let stashed: Vec<(HalfKeyChallenge, HalfKey)> = per_peer.iter().map(|entry| (*entry.0, entry.1)).collect();
            for (challenge, ack) in &stashed {
                if !awaiting_ack_from_peer.contains_key(challenge) {
                    // Share was already consumed (e.g. by the main loop in a prior call).
                    per_peer.invalidate(challenge);
                    continue;
                }
                match self.process_verified_ack(*ack, *challenge, &awaiting_ack_from_peer) {
                    Ok(ProcessedAckResult::FullRecovery(ssa)) => {
                        per_peer.invalidate(challenge);
                        res.insert(ShareResolution::RecoveredSsa(ssa));
                    }
                    Ok(ProcessedAckResult::EarlyRecovery(ssa_id)) => {
                        per_peer.invalidate(challenge);
                        res.insert(ShareResolution::AlmostRecoveredSsa(ssa_id));
                    }
                    Ok(ProcessedAckResult::NoProgress) => {
                        per_peer.invalidate(challenge);
                    }
                    Err(PixError::MissingVerifier) => {
                        // Verifier still not available — leave in pending_ack_keys for the next call.
                        tracing::trace!(%peer, "verifier not yet available, share retained in pending cache");
                    }
                    Err(_) => {
                        // Permanent failure — don't retry.
                        per_peer.invalidate(challenge);
                    }
                }
            }

            // Clean up empty per-peer entries so the outer index does not leak.
            if per_peer.weighted_size() == 0 {
                self.pending_ack_keys.invalidate(&peer);
            }
        }

        for (ack, ack_challenge) in half_keys_challenges {
            match self.process_verified_ack(ack, ack_challenge, &awaiting_ack_from_peer) {
                Ok(ProcessedAckResult::FullRecovery(ssa)) => {
                    res.insert(ShareResolution::RecoveredSsa(ssa));
                }
                Ok(ProcessedAckResult::EarlyRecovery(ssa_id)) => {
                    res.insert(ShareResolution::AlmostRecoveredSsa(ssa_id));
                }
                Ok(ProcessedAckResult::NoProgress) => {}
                Err(PixError::ShareIsEmpty) => tracing::trace!(%peer, "received empty share"),
                Err(PixError::InvalidShare(pseudonym, ssa_index)) => {
                    tracing::error!(%pseudonym, ssa_index, "encountered share that could not be verified");
                    res.insert(ShareResolution::InvalidShare(
                        peer.into(),
                        SsaId::new(pseudonym, ssa_index),
                    ));
                }
                Err(PixError::MissingVerifier) => {
                    // Share retained in awaiting_acks (process_verified_ack now uses .get()).
                    // Stash the ack so a subsequent call can retry once the verifier arrives.
                    tracing::trace!(%peer, "verifier not yet available, stashing ack for retry");
                    self.pending_ack_keys
                        .get_with(peer, || moka::sync::CacheBuilder::new(8).build())
                        .insert(ack_challenge, ack);
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

    use hopr_types::{
        crypto::{crypto_traits, prelude::*},
        crypto_random::Randomizable,
        internal::prelude::VerifiedAcknowledgement,
    };
    use vsss_rs::elliptic_curve::Field;

    use super::*;
    use crate::{
        DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, PartialSsaShare, SsaGeneratorConfig, SsaIndex,
        SsaShareGenerator,
        tests::TestSpec,
        traits::{EntryShareGenerator, ExitAcknowledgementShareProcessor},
    };

    #[test]
    fn reconstructor_rejects_invalid_exit_commitment_inputs() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let make_ssa_id = || SsaId::new(SimplePseudonym::random(), 1.try_into().unwrap());

        // polys_per_ssa == 0
        assert!(matches!(
            reconstructor.new_exit_commitment(make_ssa_id(), 0, 2),
            Err(PixError::InvalidInput)
        ));

        // polys_per_ssa exceeds MAX
        assert!(matches!(
            reconstructor.new_exit_commitment(make_ssa_id(), MAX_POLYS_PER_SSA as usize + 1, 2),
            Err(PixError::InvalidInput)
        ));

        // shares_per_poly == 0
        assert!(matches!(
            reconstructor.new_exit_commitment(make_ssa_id(), 2, 0),
            Err(PixError::InvalidInput)
        ));

        // shares_per_poly == 1 (below minimum of 2)
        assert!(matches!(
            reconstructor.new_exit_commitment(make_ssa_id(), 2, 1),
            Err(PixError::InvalidInput)
        ));

        // shares_per_poly exceeds MAX
        assert!(matches!(
            reconstructor.new_exit_commitment(make_ssa_id(), 2, MAX_POLY_THRESHOLD as usize + 1),
            Err(PixError::InvalidInput)
        ));

        // Valid inputs still work
        assert!(reconstructor.new_exit_commitment(make_ssa_id(), 2, 2).is_ok());

        Ok(())
    }

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
    fn reconstructor_missing_verifier_destroys_share() -> anyhow::Result<()> {
        // Regression test for the share-loss race:
        // When `process_verified_ack` encounters MissingVerifier, the share must
        // NOT be removed from the awaiting_acks cache — it should remain available
        // for a later retry when the verifier arrives.
        //
        // Current buggy behavior: `awaiting_ack_from_peer.remove()` at the top of
        // `process_verified_ack` consumes the share before the verifier lookup,
        // so MissingVerifier silently destroys it. This assertion will fail until
        // the ordering is fixed (use .get() first, .remove() after verifier is confirmed).
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig { ..Default::default() });

        let ack_key = HalfKey::random();
        let challenge = ack_key.to_challenge()?;

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);
        let spi = SsaPolynomialId::new(ssa_id, 0);

        let partial_share = PartialSsaShare::default().encrypt(&spi, &ack_key)?;
        let peer = OffchainKeypair::random();
        let nonce = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::random(&mut hopr_types::crypto_random::rng());

        reconstructor.new_exit_commitment(ssa_id, DEFAULT_POLYS_PER_SSA as usize, DEFAULT_POLY_THRESHOLD as usize)?;

        reconstructor.insert_encrypted_share(
            peer.public(),
            challenge,
            TaggedEncryptedPartialSsaShare {
                pseudonym: *spi.pseudonym(),
                nonce,
                partial_share,
            },
        )?;

        // Verify the share exists before processing
        let peer_cache = reconstructor.awaiting_acks.get(peer.public());
        assert!(peer_cache.is_some(), "share must be inserted before processing");
        assert!(
            peer_cache.as_ref().unwrap().contains_key(&challenge),
            "share must be present in the peer cache before processing"
        );

        // Process the ack — this should return MissingVerifier
        let peer_cache_ref = reconstructor.awaiting_acks.get(peer.public()).unwrap();
        let result = reconstructor.process_verified_ack(ack_key, challenge, &peer_cache_ref);
        assert!(matches!(result, Err(PixError::MissingVerifier)));

        // The share MUST NOT be destroyed by the MissingVerifier error.
        // BUG: the current code uses .remove() before checking the verifier,
        // so this assertion will fail, demonstrating the regression.
        let peer_cache_after = reconstructor.awaiting_acks.get(peer.public());
        assert!(
            peer_cache_after.is_some(),
            "BUG: share was permanently removed despite MissingVerifier"
        );
        assert!(
            peer_cache_after.as_ref().unwrap().contains_key(&challenge),
            "BUG: share was permanently removed despite MissingVerifier"
        );

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
        let nonce = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::random(&mut hopr_types::crypto_random::rng());

        reconstructor.new_exit_commitment(ssa_id, DEFAULT_POLYS_PER_SSA as usize, DEFAULT_POLY_THRESHOLD as usize)?;

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
    fn reconstructor_rejects_duplicate_share_via_different_challenges() -> anyhow::Result<()> {
        // 1 poly, threshold=2 → need 2 shares per polynomial to reconstruct.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 1,
            threshold: 2,
            surplus_shares: 0,
        });

        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 1, 2)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // --- Step 1: Generate the first share ---
        let msg1: [u8; 20] = hopr_types::crypto_random::random_bytes();
        let Some(first) = generator.next_share(&pseudonym, &msg1)? else {
            anyhow::bail!("expected first share");
        };
        // Clone the PartialSsaShare so we can re-encrypt it as a duplicate later
        let first_share = first.share.clone();
        let ack1 = HalfKey::random();
        let challenge1 = ack1.to_challenge()?;
        let enc1 = first.share.encrypt(&first.id, &ack1)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            challenge1,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg1, enc1)?,
        )?;

        // --- Step 2: Re-encrypt the SAME share under a different challenge (true duplicate) ---
        // The PartialSsaShare retains the same scalar value and derives the same identifier
        // (X-coordinate) from msg1, so it will be recognised as a duplicate at share-insertion time.
        let dup_ack = HalfKey::random();
        let dup_challenge = dup_ack.to_challenge()?;
        let enc_dup = first_share.encrypt(&first.id, &dup_ack)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            dup_challenge,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg1, enc_dup)?,
        )?;

        // --- Step 3: Process the first ack — share accepted, not yet complete ---
        let resolution1 = reconstructor.process_verified_ack(
            ack1,
            challenge1,
            reconstructor
                .awaiting_acks
                .get(peer.public())
                .as_ref()
                .ok_or(anyhow::anyhow!("missing peer"))?,
        )?;
        assert!(
            matches!(resolution1, ProcessedAckResult::NoProgress),
            "first share should not yet complete the SSA"
        );

        // --- Step 4: Process the duplicate ---
        // The SsaPartBuilder has 1/2 shares. The duplicate share has the same identifier
        // (same X-coordinate from msg1), so it hits the
        // `any(|s| s.identifier == share.identifier)` check in SsaPartBuilder::add_share
        // and returns Ok(None), which surfaces as NoProgress.
        let resolution_dup = reconstructor.process_verified_ack(
            dup_ack,
            dup_challenge,
            reconstructor
                .awaiting_acks
                .get(peer.public())
                .as_ref()
                .ok_or(anyhow::anyhow!("missing peer"))?,
        )?;
        assert!(
            matches!(resolution_dup, ProcessedAckResult::NoProgress),
            "duplicate share must return NoProgress during active reconstruction"
        );

        // --- Step 5: Generate and process the second distinct share ---
        let msg2: [u8; 20] = hopr_types::crypto_random::random_bytes();
        let Some(second) = generator.next_share(&pseudonym, &msg2)? else {
            anyhow::bail!("expected second share");
        };
        let ack2 = HalfKey::random();
        let challenge2 = ack2.to_challenge()?;
        let enc2 = second.share.encrypt(&second.id, &ack2)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            challenge2,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg2, enc2)?,
        )?;

        let resolution2 = reconstructor.process_verified_ack(
            ack2,
            challenge2,
            reconstructor
                .awaiting_acks
                .get(peer.public())
                .as_ref()
                .ok_or(anyhow::anyhow!("missing peer"))?,
        )?;
        assert!(
            matches!(resolution2, ProcessedAckResult::FullRecovery(ref r) if r.ssa_id == ssa_id),
            "second unique share should complete SSA reconstruction"
        );

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

    // -----------------------------------------------------------------------
    // early_recovery_threshold tests
    // -----------------------------------------------------------------------

    /// Helper: create an SsaBuilder that accepts zero-valued sub-secrets.
    fn make_builder(num_polys: usize) -> SsaBuilder<TestSpec> {
        let exit_secret = PixScalar::<TestSpec>::default();
        let full_commitment = PixGroup::<TestSpec>::default();
        SsaBuilder::new(full_commitment, exit_secret, num_polys)
    }

    /// Helper: add `n` zero-valued polynomial parts to `builder`, returning
    /// the result of each call.
    fn add_parts(
        builder: &mut SsaBuilder<TestSpec>,
        n: usize,
    ) -> crate::errors::Result<Vec<Option<PixScalar<TestSpec>>>, <TestSpec as PixSpec>::Pseudonym> {
        let mut results = Vec::with_capacity(n);
        for i in 0..n {
            let sub = PixScalar::<TestSpec>::default();
            results.push(builder.add_recovered_ssa_part(i as PolynomialIndex, sub)?);
        }
        Ok(results)
    }

    #[test]
    fn ssa_builder_early_threshold_below() -> anyhow::Result<()> {
        // num_polys=10, threshold=0.85 → ceil(0.85×10)=9.
        // Adding 8 parts should NOT reach the threshold.
        let mut builder = make_builder(10);
        add_parts(&mut builder, 8)?;
        assert!(!builder.check_early_threshold(0.85));
        Ok(())
    }

    #[test]
    fn ssa_builder_early_threshold_hits_ceil_at_9() -> anyhow::Result<()> {
        // num_polys=10, threshold=0.85 → ceil(0.85×10)=9.
        // Adding 9 parts SHOULD fire on the first check.
        let mut builder = make_builder(10);
        add_parts(&mut builder, 9)?;
        assert!(builder.check_early_threshold(0.85));
        // Second call must return false (idempotent guard).
        assert!(!builder.check_early_threshold(0.85));
        Ok(())
    }

    #[test]
    fn ssa_builder_threshold_1_dot_0_fires_at_full_recovery() -> anyhow::Result<()> {
        // num_polys=10, threshold=1.0 → ceil(1.0×10)=10.
        // Only fires when ALL 10 polynomial parts are received.
        let mut builder = make_builder(10);
        add_parts(&mut builder, 9)?;
        assert!(!builder.check_early_threshold(1.0));
        add_parts(&mut builder, 1)?; // 10th part → completes SSA
        // After full recovery, early_notified is set by add_recovered_ssa_part.
        // check_early_threshold should still report false.
        assert!(!builder.check_early_threshold(1.0));
        Ok(())
    }

    #[test]
    fn process_verified_ack_emits_early_and_full_recovery() -> anyhow::Result<()> {
        // Use a small SSA config where we can observe both events.
        // 4 polynomials, threshold=4, surplus=0 → 16 shares total.
        // early_recovery_threshold=0.5 → ceil(0.5×4)=2.
        // After 2 polynomial parts → EarlyRecovery.
        // After all 4         → FullRecovery.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 4,
            threshold: 4,
            surplus_shares: 0,
        });

        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            early_recovery_threshold: 0.5,
            ..Default::default()
        });

        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 4, 4)?;

        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // No shares inserted yet — has_pending_shares must be false.
        assert!(
            !reconstructor.has_pending_shares(peer.public()),
            "no shares inserted yet"
        );

        // Generate and insert all 16 encrypted shares
        let mut acks = Vec::new();
        while let Some((msg, share)) = {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            generator.next_share(&pseudonym, &msg).map(|v| v.map(|u| (msg, u)))
        }? {
            let ack = HalfKey::random();
            let ack_challenge = ack.to_challenge()?;
            let enc_share = share.share.encrypt(&share.id, &ack)?;

            reconstructor.insert_encrypted_share(
                peer.public(),
                ack_challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc_share)?,
            )?;
            acks.push(VerifiedAcknowledgement::new(ack, &peer).leak());
        }

        // After inserting encrypted shares, the peer must have pending shares.
        assert!(
            reconstructor.has_pending_shares(peer.public()),
            "shares were just inserted"
        );

        // Process all acks in one batch
        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;

        // Both events MUST be present
        let has_early = resolutions
            .iter()
            .any(|r| matches!(r, ShareResolution::AlmostRecoveredSsa(id) if *id == ssa_id));
        let has_full = resolutions
            .iter()
            .any(|r| matches!(r, ShareResolution::RecoveredSsa(r) if r.ssa_id == ssa_id));

        assert!(has_early, "expected AlmostRecoveredSsa event");
        assert!(has_full, "expected RecoveredSsa event");

        Ok(())
    }

    #[test]
    fn pending_ack_cache_isolates_stashed_acks_by_peer() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig { ..Default::default() });

        let ack_a = HalfKey::random();
        let challenge_a = ack_a.to_challenge()?;
        let peer_a = OffchainKeypair::random();

        let ack_b = HalfKey::random();
        let challenge_b = ack_b.to_challenge()?;
        let peer_b = OffchainKeypair::random();

        // Stash acks for two different peers — the nested cache must keep them isolated.
        reconstructor
            .pending_ack_keys
            .get_with(*peer_a.public(), || moka::sync::CacheBuilder::new(8).build())
            .insert(challenge_a, ack_a);
        reconstructor
            .pending_ack_keys
            .get_with(*peer_b.public(), || moka::sync::CacheBuilder::new(8).build())
            .insert(challenge_b, ack_b);

        // peer_a's per-peer cache contains only challenge_a
        let cache_a = reconstructor.pending_ack_keys.get(peer_a.public()).unwrap();
        assert!(cache_a.contains_key(&challenge_a), "peer_a should have its stash");
        assert!(
            !cache_a.contains_key(&challenge_b),
            "peer_a must not see peer_b's stash"
        );

        // peer_b's per-peer cache contains only challenge_b
        let cache_b = reconstructor.pending_ack_keys.get(peer_b.public()).unwrap();
        assert!(cache_b.contains_key(&challenge_b), "peer_b should have its stash");
        assert!(
            !cache_b.contains_key(&challenge_a),
            "peer_b must not see peer_a's stash"
        );

        // Invalidating by challenge from peer_a's cache does not affect peer_b
        reconstructor
            .pending_ack_keys
            .get(peer_a.public())
            .unwrap()
            .invalidate(&challenge_a);
        assert!(
            reconstructor
                .pending_ack_keys
                .get(peer_a.public())
                .unwrap()
                .weighted_size()
                == 0,
            "peer_a cache should be empty after invalidate"
        );
        assert!(
            reconstructor
                .pending_ack_keys
                .get(peer_b.public())
                .unwrap()
                .contains_key(&challenge_b),
            "peer_b's stash must survive peer_a's invalidation"
        );

        Ok(())
    }
}
