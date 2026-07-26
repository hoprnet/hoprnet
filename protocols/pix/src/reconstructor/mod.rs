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
    /// Maximum number of peers that can be tracked simultaneously with unacknowledged shares.
    ///
    /// Default is 2000, minimum is 10.
    #[validate(range(min = 10))]
    #[default(2000)]
    pub max_tracked_peers: usize,
    /// Maximum number of awaited acknowledgements to extract a single share.
    ///
    /// This corresponds to the maximum number of unacknowledged HOPR packets awaiting acknowledgement.
    ///
    /// Default is 1 000 000, must be at least 10 000.
    #[default(1_000_000)]
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
    /// Liveness map: records `num_polys` for every completed SSA cycle so that
    /// `retire_ssa` can remove all verifier/builder state even when both builders
    /// have been TTL-evicted.
    ///
    /// ## Why a separate map? (builder TTL guard is insufficient for retirement)
    ///
    /// The builder TTL guard at construction (`max(builder_ttl, verifier_ttl)`) only
    /// guarantees *starting* TTL parity. It cannot prevent the TTL window we call
    /// the "verifier-widow":
    ///
    /// `process_verified_ack` (the ack processing hot path) accesses the verifier
    /// `self.ssa_verifiers.get()` *before* the builder `self.ssa_builders.get()`.
    /// When an ack arrives just after the builder has expired:
    ///  1. The verifier `.get()` at line 213 succeeds and **resets** the verifier's idle-timer to the full
    ///     `unused_verifier_lifetime` (e.g. another 30 min).
    ///  2. The builder `.get()` at line 221 fails → `MissingSsaCommitment`.
    ///  3. The builder is now gone forever, but the verifier lives for another 30 minutes with no way to learn
    ///     `num_polys`.
    ///
    /// `ssa_num_polys` bridges this widow. It is populated alongside the verifiers
    /// at `CommitmentResult::Completed` time, shares their TTL (so it stays alive
    /// as long as what it shadows), and is explicitly invalidated in `remove_cycle`
    /// so it does not outlive the cleanup it enables.
    ///
    /// Populated at `CommitmentResult::Completed` time in
    /// `insert_coefficient_commitments`; cleaned up by `remove_cycle`.
    ssa_num_polys: moka::sync::Cache<SsaId<S::Pseudonym>, usize>,
    /// Tombstone set: SsaIds that have been retired.  The commitment completion
    /// path checks this after inserting verifiers but before publishing the
    /// builder/liveness entry, preventing resurrection when `retire_ssa` runs
    /// between verifier installation and publication.
    retired_ssas: moka::sync::Cache<SsaId<S::Pseudonym>, ()>,
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
            commitment_builder: moka::sync::Cache::builder()
                .time_to_idle(cfg.incomplete_commitment_lifetime)
                .build(),
            // The builder must not be reclaimed *before* its verifiers (I1): a live
            // verifier paired with an expired builder makes a recovered share permanently
            // unreachable (`process_verified_ack` returns `MissingSsaCommitment` and the
            // ack is dropped). Clamping the builder TTL to at least the verifier TTL
            // prevents this *during ack processing* — the two `.get()` calls in
            // `process_verified_ack` are sequential (verifier first, builder second), so
            // a builder that started with ≥ the verifier's TTL will never expire between
            // them in a single invocation.
            //
            // NOTE: This TTL guard alone is *not* sufficient for `retire_ssa` cleanup
            // (see `ssa_num_polys` docstring — the "verifier-widow"). The guard prevents
            // mid-request expiration, but across requests the verifier's idle timestamp
            // can be independently refreshed while the builder is not, creating an
            // asymmetric window where the builder has expired but the verifier has not.
            //
            // Both builder caches intentionally have NO max_capacity (same as
            // `ssa_verifiers`): active removal happens via `remove_cycle` on full
            // recovery and `retire_ssa` on session teardown, and TTL is the backstop.
            // A hard capacity would silently evict a builder while its verifiers remain
            // live, permanently stranding the SSA cycle.
            ssa_builders: moka::sync::Cache::builder()
                .time_to_idle(cfg.incomplete_ssa_lifetime.max(cfg.unused_verifier_lifetime))
                .build(),
            // Indispensable per-cycle state: never size-evicted. Built without a
            // `max_capacity`, so only `time_to_idle` reclaims it (see H1). Explicit
            // `retire_ssa` removes verifiers on full recovery and session teardown.
            ssa_verifiers: moka::sync::Cache::builder()
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            awaiting_acks: moka::sync::CacheBuilder::new(cfg.max_tracked_peers as u64)
                .time_to_idle(cfg.max_ack_await_time)
                .build(),
            pending_ack_keys: moka::sync::CacheBuilder::new(cfg.max_tracked_peers as u64)
                .time_to_idle(cfg.max_ack_await_time)
                .build(),
            // Liveness map for retirement: TTL must cover the verifier lifetime so the
            // entry survives as long as the verifiers it shadows. No max_capacity because
            // entries are explicitly invalidated in remove_cycle.
            ssa_num_polys: moka::sync::Cache::builder()
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            // Tombstone set: short TTL since it only needs to cover the window between
            // verifier insertion and builder/liveness publication.  Once the builder
            // is published, retire_ssa can find the cycle via the liveness map.
            retired_ssas: moka::sync::Cache::builder()
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            cfg,
        }
    }

    /// Returns the configuration of the reconstructor.
    #[inline]
    pub fn config(&self) -> &SsaReconstructorConfig {
        &self.cfg
    }

    /// Returns `true` if the reconstructor still holds a builder (SSA-part
    /// builder or commitment builder) for the given cycle.  Used by tests to
    /// verify that [`retire_ssa`](ExitAcknowledgementShareProcessor::retire_ssa)
    /// cleaned up the expected state.
    pub fn contains_builder(&self, ssa_id: &SsaId<S::Pseudonym>) -> bool {
        self.ssa_builders.contains_key(ssa_id) || self.commitment_builder.contains_key(ssa_id)
    }

    /// Removes all reconstructor state for a single SSA cycle whose polynomial
    /// count is already known. At commitment completion the polynomial indices are
    /// contiguous `0..num_polys`, so those are exactly the verifier keys to drop.
    /// Idempotent: invalidating an absent key is a no-op.
    fn remove_cycle(&self, ssa_id: SsaId<S::Pseudonym>, num_polys: usize) {
        for poly_index in 0..num_polys as PolynomialIndex {
            self.ssa_verifiers.invalidate(&SsaPolynomialId::new(ssa_id, poly_index));
        }
        self.ssa_builders.invalidate(&ssa_id);
        self.commitment_builder.invalidate(&ssa_id);
        self.ssa_num_polys.invalidate(&ssa_id);
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

        // Guard: confirm the builder exists (and refresh its idle TTL) before consuming
        // the share. The builder has a shorter TTL (10 min) than the verifier (30 min),
        // so acks for other polynomials can keep the verifier alive long after the
        // builder has expired — without this guard, the recovered part would be dropped
        // with no retry path. Hold the Arc to skip a redundant cache lookup later.
        let builder = self
            .ssa_builders
            .get(spi.as_ref())
            .ok_or(PixError::MissingSsaCommitment)?;

        // Verifier and builder confirmed — safe to consume the share.
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

        let mut builder_guard = builder.lock();
        let ssa = builder_guard.add_recovered_ssa_part(spi.poly_index(), ssa_part)?;
        match ssa {
            Some(scalar) => {
                let ssa_id = *spi.as_ref();
                // Capture what we need and release the builder lock before retiring,
                // so `remove_cycle` does not re-enter this same mutex.
                let num_polys = builder_guard.num_polys();
                drop(builder_guard);
                let Some(ssa) = S::scalar_to_private_key(scalar) else {
                    tracing::error!(%spi, "ssa reconstruction failed");
                    self.remove_cycle(ssa_id, num_polys);
                    return Err(PixError::InvalidSsa);
                };
                // Full recovery: this cycle's verifier state is no longer needed.
                self.remove_cycle(ssa_id, num_polys);
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

    fn is_expected_error(&self, error: &Self::Error) -> bool {
        matches!(error, PixError::UnexpectedShare)
    }

    fn retire_ssa(&self, ssa_id: SsaId<S::Pseudonym>) {
        // Mark tombstone BEFORE removing state so the commitment completion path
        // can detect retirement and skip builder/liveness publication.
        self.retired_ssas.insert(ssa_id, ());

        // Prefer the liveness map: it retains num_polys even after both builders
        // have been TTL-evicted, enabling full verifier cleanup.
        let num_polys = self.ssa_num_polys.get(&ssa_id).or_else(|| {
            // Fallback: num_polys is also available from either builder while it exists.
            self.ssa_builders
                .get(&ssa_id)
                .map(|b| b.lock().num_polys())
                .or_else(|| self.commitment_builder.get(&ssa_id).map(|b| b.lock().num_polys()))
        });
        match num_polys {
            Some(num_polys) => self.remove_cycle(ssa_id, num_polys),
            None => {
                // No builder and no liveness entry: any lingering verifiers fall to
                // the idle-TTL backstop.
                self.ssa_builders.invalidate(&ssa_id);
                self.commitment_builder.invalidate(&ssa_id);
            }
        }
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
                let num_polys = ssa_builder.num_polys();
                let full_ssa_commitment = ssa_builder.full_commitment;
                // Insert verifiers BEFORE checking the tombstone, so that
                // a concurrent retire_ssa can find and remove them via the
                // liveness map once we publish below.  If we checked the
                // tombstone first, retire_ssa might run between the check
                // and the insert, leaving verifiers that were never published
                // to any cache and are thus invisible to cleanup.
                let spis: Vec<SsaPolynomialId<S::Pseudonym>> =
                    ssa_reconstructors.iter().map(|r| r.verifier.spi).collect();
                for ssa_reconstructor in ssa_reconstructors {
                    self.ssa_verifiers.insert(
                        ssa_reconstructor.verifier.spi,
                        std::sync::Arc::new(parking_lot::Mutex::new(ssa_reconstructor)),
                    );
                }
                // Check the tombstone: if retire_ssa ran during verifier
                // insertion, clean up the verifiers and skip publication.
                if self.retired_ssas.contains_key(&ssa_id) {
                    for spi in &spis {
                        self.ssa_verifiers.invalidate(spi);
                    }
                    tracing::trace!(%ssa_id, "ssa commitment completed but cycle was retired — dropped verifiers");
                } else {
                    self.ssa_builders
                        .insert(ssa_id, std::sync::Arc::new(parking_lot::Mutex::new(ssa_builder)));
                    self.ssa_num_polys.insert(ssa_id, num_polys);
                    res.ssa_deposit_address =
                        Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);
                    res.is_verifiable = true;
                }

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
                        .get_with(peer, || {
                            moka::sync::CacheBuilder::new(self.cfg.max_awaiting_acks as u64).build()
                        })
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
    fn reconstructor_duplicate_per_cell_commitment() -> anyhow::Result<()> {
        // Regression test for the per-cell duplicate check inside add_transposed.
        // Previously the same (poly_index, coeff_index) slot silently overwrote;
        // now it returns DuplicateCommitment.
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);

        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // Insert coeff_index=0 for poly 0
        let mut poly_map_1 = HashMap::new();
        poly_map_1.insert(0 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        reconstructor.insert_coefficient_commitments(ssa_id, 0, poly_map_1.into_iter())?;

        // Insert coeff_index=0 for the same poly 0 again — should now fail
        let mut poly_map_2 = HashMap::new();
        poly_map_2.insert(0 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        let result = reconstructor.insert_coefficient_commitments(ssa_id, 0, poly_map_2.into_iter());
        assert!(matches!(result, Err(PixError::DuplicateCommitment)));

        // But coeff_index=1 for the same poly should still succeed
        let mut poly_map_3 = HashMap::new();
        poly_map_3.insert(0 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        assert!(
            reconstructor
                .insert_coefficient_commitments(ssa_id, 1, poly_map_3.into_iter())
                .is_ok()
        );

        Ok(())
    }

    #[test]
    fn reconstructor_missing_verifier_retains_share() -> anyhow::Result<()> {
        // Regression test for the share-loss race:
        // When `process_verified_ack` encounters MissingVerifier, the share must
        // NOT be removed from the awaiting_acks cache — it should remain available
        // for a later retry when the verifier arrives.
        //
        // The implementation guarantees this: `process_verified_ack` looks the share
        // up with `.get()` and only `.remove()`s it after the verifier lookup
        // succeeds, so a MissingVerifier error leaves the share in place. This test
        // asserts that retention.
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

        // The share MUST NOT be destroyed by the MissingVerifier error: the
        // implementation only removes it after the verifier lookup succeeds, so it
        // stays available for a later retry.
        let peer_cache_after = reconstructor.awaiting_acks.get(peer.public());
        assert!(
            peer_cache_after.is_some(),
            "share must be retained after MissingVerifier"
        );
        assert!(
            peer_cache_after.as_ref().unwrap().contains_key(&challenge),
            "share must be retained after MissingVerifier"
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
    fn malformed_commitment_does_not_poison_corrected_retransmission() -> anyhow::Result<()> {
        // Regression test for M2: a malformed coefficient that fails EC point
        // decoding must NOT leave the commitment builder permanently poisoned.
        // After submitting malformed bytes, a retry with correct bytes must
        // succeed and complete the SSA.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 1,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let mut commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        // Extract the constant term (coeff 0) and the final coefficient (coeff 1).
        let constant_term = commitment
            .verifiers
            .remove(&0)
            .ok_or_else(|| anyhow::anyhow!("missing constant-term commitment"))?
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(constant_term.len(), 1);

        let final_coefficient = commitment
            .verifiers
            .remove(&1)
            .ok_or_else(|| anyhow::anyhow!("missing final coefficient commitment"))?
            .into_iter()
            .collect::<Vec<_>>();

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, 1, 2)?;

        // Step 1: Submit constant term — must succeed, produce a deposit address,
        // but NOT be verifiable (coefficient 1 is still missing).
        let partial = reconstructor.insert_coefficient_commitments(ssa_id, 0, constant_term.into_iter())?;
        assert!(
            partial.ssa_deposit_address.is_some(),
            "constant term alone should yield a deposit address"
        );
        assert!(!partial.is_verifiable, "not yet complete");

        // Step 2: Submit a malformed coefficient (bytes with an invalid EC
        // compressed-point prefix of 0xff) — must return InvalidInput.
        let mut malformed = PixGroupRepr::<TestSpec>::default(); // zero-filled
        AsMut::<[u8]>::as_mut(&mut malformed).fill(0xff);
        let malformed_result = reconstructor.insert_coefficient_commitments(ssa_id, 1, [(0, malformed)].into_iter());
        assert!(
            matches!(&malformed_result, Err(crate::errors::PixError::InvalidInput)),
            "malformed commitment must be rejected, got {malformed_result:?}"
        );

        // Step 3: Retry with the correct bytes — must succeed and produce a
        // verifiable SSA commitment.
        let retry = reconstructor.insert_coefficient_commitments(ssa_id, 1, final_coefficient.into_iter());
        assert!(
            matches!(&retry, Ok(state) if state.is_verifiable),
            "corrected retransmission must complete the SSA, got {retry:?}"
        );

        Ok(())
    }

    #[test]
    fn full_recovery_retires_all_reconstructor_state() -> anyhow::Result<()> {
        // 4 polynomials, threshold 4, no surplus → 16 shares, fully recoverable.
        // Multiple polynomials on purpose: this exercises the whole `0..num_polys`
        // cleanup loop, so a wrong-key or off-by-one in `remove_cycle` would surface.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 4,
            threshold: 4,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig::default());
        reconstructor.new_exit_commitment(ssa_id, 4, 4)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // Precondition: the completed cycle holds its verifier + builder state.
        reconstructor.ssa_verifiers.run_pending_tasks();
        assert_eq!(
            reconstructor.ssa_verifiers.entry_count(),
            4,
            "4 verifiers present after completion"
        );
        assert!(
            reconstructor.ssa_builders.contains_key(&ssa_id),
            "ssa builder present after completion"
        );

        // Drive full recovery: generate every share, insert it encrypted, acknowledge.
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
        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;
        assert!(
            resolutions
                .iter()
                .any(|r| matches!(r, ShareResolution::RecoveredSsa(r) if r.ssa_id == ssa_id)),
            "cycle must fully recover"
        );

        // Behaviour under test: full recovery must retire ALL of the cycle's
        // reconstructor state, rather than leave it to linger until the idle TTL.
        reconstructor.ssa_verifiers.run_pending_tasks();
        reconstructor.ssa_builders.run_pending_tasks();
        reconstructor.commitment_builder.run_pending_tasks();
        assert_eq!(
            reconstructor.ssa_verifiers.entry_count(),
            0,
            "verifiers must be retired on full recovery"
        );
        assert!(
            !reconstructor.ssa_builders.contains_key(&ssa_id),
            "ssa builder must be retired on full recovery"
        );
        assert!(
            !reconstructor.commitment_builder.contains_key(&ssa_id),
            "commitment builder must be retired on full recovery"
        );

        Ok(())
    }

    #[test]
    fn retire_ssa_removes_cycle_state_and_is_idempotent() -> anyhow::Result<()> {
        // 3 polynomials so the cleanup loop is again exercised over several keys.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 3,
            threshold: 4,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig::default());
        reconstructor.new_exit_commitment(ssa_id, 3, 4)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        reconstructor.ssa_verifiers.run_pending_tasks();
        assert_eq!(
            reconstructor.ssa_verifiers.entry_count(),
            3,
            "3 verifiers present after completion"
        );
        assert!(reconstructor.ssa_builders.contains_key(&ssa_id));

        // Explicit retirement (as invoked on session teardown) drops everything.
        reconstructor.retire_ssa(ssa_id);
        reconstructor.ssa_verifiers.run_pending_tasks();
        reconstructor.ssa_builders.run_pending_tasks();
        reconstructor.commitment_builder.run_pending_tasks();
        assert_eq!(
            reconstructor.ssa_verifiers.entry_count(),
            0,
            "verifiers must be removed by retire_ssa"
        );
        assert!(!reconstructor.ssa_builders.contains_key(&ssa_id));
        assert!(!reconstructor.commitment_builder.contains_key(&ssa_id));

        // Idempotent: retiring the same (now-empty) cycle again is a harmless no-op.
        reconstructor.retire_ssa(ssa_id);

        // `None` fallback: retiring a cycle that was never created must not panic and
        // must leave the caches untouched.
        let never_seen = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        reconstructor.retire_ssa(never_seen);
        reconstructor.ssa_verifiers.run_pending_tasks();
        assert_eq!(reconstructor.ssa_verifiers.entry_count(), 0);

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

    /// Verifies that the builder caches accept more entries than the old
    /// `MAX_POLYS_PER_SSA` size bound. After removing the hard capacity, only
    /// TTL governs eviction.  Also verifies that fully-committed IDs populate
    /// `ssa_builders` and remain cached.
    #[test]
    fn builder_caches_accept_more_entries_than_max_polys_per_ssa() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let pseudonym = SimplePseudonym::random();
        let exceed = MAX_POLYS_PER_SSA as usize + 5;
        let mut ids = Vec::with_capacity(exceed);
        for i in 0..exceed {
            let ssa_id = SsaId::new(pseudonym, (1u32 + i as u32).try_into()?);
            reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
            ids.push(ssa_id);
        }
        reconstructor.commitment_builder.run_pending_tasks();
        for ssa_id in &ids {
            assert!(
                reconstructor.contains_builder(ssa_id),
                "commitment builder must retain every accepted SsaId ({ssa_id:?})"
            );
        }

        // Complete the first few IDs through the full commitment path to populate
        // ssa_builders.  Use a 2-poly 2-threshold generator so that
        // insert_coefficient_commitments eventually hits CommitmentResult::Completed
        // and pushes into ssa_builders.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        });
        for ssa_id in ids.iter().take(3) {
            let commit = generator.new_ssa_commitment(&pseudonym, ssa_id.ssa_index())?;
            commit.process_into_reconstructor(&reconstructor)?;
            reconstructor.commitment_builder.run_pending_tasks();
            reconstructor.ssa_builders.run_pending_tasks();

            assert!(
                reconstructor.ssa_builders.contains_key(ssa_id),
                "ssa_builders must contain completed SsaId index {} ({ssa_id:?})",
                ssa_id.ssa_index().get(),
            );
            assert!(
                reconstructor.ssa_num_polys.contains_key(ssa_id),
                "ssa_num_polys must retain the completed SsaId ({ssa_id:?})",
            );
        }

        Ok(())
    }

    #[test]
    fn retire_ssa_tombstone_prevents_resurrection_in_insert_coefficient_commitments() -> anyhow::Result<()> {
        // Regression test for the M9 tombstone fix: when a concurrent `retire_ssa`
        // runs between verifier insertion and builder/liveness publication inside
        // `insert_coefficient_commitments`'s `CommitmentResult::Completed` arm, the
        // tombstone must prevent resurrection.
        //
        // The race window is:
        //  1. commitment_builder entry is completed
        //  2. verifiers are inserted (line 443-447)
        //  3. retire_ssa runs concurrently (sets tombstone + removes builder/verifier)
        //  4. tombstone check (line 451) intercepts and skips publication
        //
        // We simulate step 3 by setting the tombstone directly (the `retired_ssas`
        // cache) between the two coefficient insertions.  The first insertion makes
        // the commitment almost-complete; the second triggers Completed.
        //
        // The test uses a 2-coefficient generator so that two separate
        // insert_coefficient_commitments calls are needed to complete the SSA.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 1,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, 1, 2)?;

        // Insert first coefficient — makes the builder almost-complete.
        // Builders may be lazily evaluated; run pending tasks to flush.
        {
            let coeff_0: Vec<_> = commitment
                .verifiers
                .get(&0)
                .ok_or_else(|| anyhow::anyhow!("missing coeff 0"))?
                .iter()
                .map(|(pi, repr)| (*pi, *repr))
                .collect();
            let partial = reconstructor.insert_coefficient_commitments(ssa_id, 0, coeff_0.into_iter())?;
            assert!(!partial.is_verifiable, "not yet complete after coeff 0");
        }
        reconstructor.commitment_builder.run_pending_tasks();

        // Simulate concurrent retire_ssa by setting only the tombstone.
        // In the real race, retire_ssa would also remove builders/verifiers,
        // but setting the tombstone is sufficient to test the critical check.
        reconstructor.retired_ssas.insert(ssa_id, ());

        // Insert final coefficient — triggers Completed; tombstone must intercept.
        reconstructor.commitment_builder.run_pending_tasks();
        {
            let coeff_1: Vec<_> = commitment
                .verifiers
                .get(&1)
                .ok_or_else(|| anyhow::anyhow!("missing coeff 1"))?
                .iter()
                .map(|(pi, repr)| (*pi, *repr))
                .collect();
            let result = reconstructor.insert_coefficient_commitments(ssa_id, 1, coeff_1.into_iter())?;

            // The tombstone path drops verifiers and skips builder/liveness
            // publication even though the builder completed.
            // Note: ssa_deposit_address is populated at line 409 from the
            // commitment_builder's internal state (set before the tombstone
            // check), so it may still be Some.  The key invariant is
            // is_verifiable=false — the builder was NOT published.
            assert!(!result.is_verifiable, "tombstone prevented verifiable=true");
        }

        // Flush lazy caches.
        reconstructor.ssa_builders.run_pending_tasks();

        // Builder and liveness caches must be empty.
        assert!(
            !reconstructor.ssa_builders.contains_key(&ssa_id),
            "tombstone prevented builder publication"
        );
        assert!(
            !reconstructor.ssa_num_polys.contains_key(&ssa_id),
            "tombstone prevented liveness publication"
        );

        // The verifiers should also have been invalidated by the tombstone path.
        for poly_idx in 0..1 {
            let spi = SsaPolynomialId::new(ssa_id, poly_idx);
            assert!(
                !reconstructor.ssa_verifiers.contains_key(&spi),
                "tombstone must also remove verifier for spi {spi:?}"
            );
        }

        Ok(())
    }
}
