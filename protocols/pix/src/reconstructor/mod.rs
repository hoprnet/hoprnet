mod utils;

use std::{collections::HashMap, sync::Arc};

use hopr_types::{
    crypto::{
        crypto_traits::elliptic_curve::Field,
        prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    },
    internal::prelude::Acknowledgement,
};
use indexmap::IndexSet;
use utils::{AddShareOutcome, CommitmentResult, SsaBuilder, SsaCommitmentBuilder, SsaPartBuilder};

use crate::{
    CoefficientIndex, ExitAcknowledgementShareProcessor, Group, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, PixGroup,
    PixGroupRepr, PixScalar, PixSpec, PolynomialIndex, RecoveredSsa, ShareResolution, SsaCommitmentState,
    SsaPolynomialId, SsaRecoveryProgress, TaggedEncryptedPartialSsaShare, errors::PixError, types::SsaId,
};

/// Tracks polynomial verifier IDs per SSA so `retire_ssa` can remove all of them.
///
/// Uses TTL-only cache (same lifetime as `ssa_counters`) so entries auto-expire
/// if `retire_ssa` is not called.
type SsaVerifierMap<S> =
    moka::sync::Cache<SsaId<<S as PixSpec>::Pseudonym>, Vec<SsaPolynomialId<<S as PixSpec>::Pseudonym>>>;

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
    /// Time-to-live for the per-SSA counter entry (progress, faults).
    ///
    /// The counter cache uses **TTL only** (no TTI), so counters survive
    /// builder/verifier eviction. Callers must validate this value against their
    /// supervision horizon.
    ///
    /// Default is 7200 seconds (2 hours).
    #[default(7200)]
    #[validate(range(min = 60, max = 86400))]
    pub ssa_counter_lifetime_secs: u64,
}

type EncryptedShareCache<S> =
    moka::sync::Cache<HalfKeyChallenge, TaggedEncryptedPartialSsaShare<S, <S as PixSpec>::Pseudonym, PixScalar<S>>>;

/// Progress + fault snapshot for drain-eligibility decisions.
pub struct SsaDrainSnapshot<P> {
    /// Current recovery progress for the SSA.
    pub progress: SsaRecoveryProgress<P>,
    /// Absolute total of invalid shares observed for this SSA (cumulative).
    pub invalid_total: u64,
}

/// Per-SSA counter entry tracked in the dedicated TTL-only cache.
/// Counters are absolute — they represent the total observed so far for this SSA.
struct SsaCounterEntry {
    useful_shares: u64,
    target_useful_shares: u64,
    recovered_polynomials: u16,
    /// Cross-peer aggregate invalid-share total for this SSA.
    /// This is what gets emitted to the supervisor for limit enforcement.
    invalid_total: u64,
    /// Per-peer invalid share totals for this SSA.
    /// Values are absolute counts (not deltas). Retained for attribution/telemetry.
    invalid_by_peer: HashMap<OffchainPublicKey, u64>,
}

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
    /// Dedicated TTL-only cache for per-SSA counters (no TTI).
    /// Counters survive builder/verifier eviction.
    ssa_counters: moka::sync::Cache<SsaId<S::Pseudonym>, std::sync::Arc<parking_lot::Mutex<SsaCounterEntry>>>,
    ssa_to_verifier_ids: SsaVerifierMap<S>,
    cfg: SsaReconstructorConfig,
}

/// Result of processing a single verified acknowledgement in the SSA reconstructor.
enum ProcessedAckResult<S: PixSpec> {
    /// No matching encrypted share found (ack for unknown challenge).
    NoProgress,
    /// Share was a duplicate evaluation identifier — no counters affected.
    Duplicate,
    /// Share arrived after the polynomial was already reconstructed — no counters affected.
    Surplus,
    /// A useful (new, verified, below-threshold) share was collected.
    UsefulShare(SsaId<<S as PixSpec>::Pseudonym>),
    /// A polynomial part was completed (share reached threshold).
    PolynomialPartComplete(SsaId<<S as PixSpec>::Pseudonym>),
    /// The early recovery threshold was crossed.
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
    pub fn new(cfg: SsaReconstructorConfig) -> Self {
        Self {
            commitment_builder: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                .time_to_idle(cfg.incomplete_commitment_lifetime)
                .build(),
            ssa_builders: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                .time_to_idle(cfg.incomplete_ssa_lifetime)
                .build(),
            ssa_verifiers: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA as u64) * 4)
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            awaiting_acks: moka::sync::CacheBuilder::new(cfg.max_awaiting_acks as u64)
                .time_to_idle(cfg.max_ack_await_time)
                .build(),
            ssa_counters: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                // Capacity is a generous per-SSA estimate. In practice the
                // key domain is (max concurrent sessions × SSAs per session),
                // which is far smaller than MAX_POLYS_PER_SSA (≈16k). The
                // TTL-only eviction means counters survive builder/verifier
                // eviction — they are never TTI-evicted while the session
                // is live.
                .time_to_live(std::time::Duration::from_secs(cfg.ssa_counter_lifetime_secs))
                .build(),
            ssa_to_verifier_ids: moka::sync::CacheBuilder::new((MAX_POLYS_PER_SSA + 1) as u64)
                // Same generous sizing + TTL-only policy as ssa_counters.
                .time_to_live(std::time::Duration::from_secs(cfg.ssa_counter_lifetime_secs))
                .build(),
            cfg,
        }
    }

    /// Returns the configuration of the reconstructor.
    #[inline]
    pub fn config(&self) -> &SsaReconstructorConfig {
        &self.cfg
    }

    /// Updates the per-SSA counter for a useful share.
    fn record_useful_share(&self, ssa_id: &SsaId<S::Pseudonym>) {
        if let Some(entry) = self.ssa_counters.get(ssa_id) {
            let mut entry = entry.lock();
            entry.useful_shares += 1;
        }
    }

    /// Updates the per-SSA counter for a completed polynomial part.
    fn record_completed_part(&self, ssa_id: &SsaId<S::Pseudonym>) {
        if let Some(entry) = self.ssa_counters.get(ssa_id) {
            let mut entry = entry.lock();
            entry.recovered_polynomials += 1;
        }
    }

    /// Updates the per-peer invalid-share counter for an SSA.
    fn record_invalid_share(&self, peer: &OffchainPublicKey, ssa_id: &SsaId<S::Pseudonym>) {
        if let Some(entry) = self.ssa_counters.get(ssa_id) {
            let mut entry = entry.lock();
            entry.invalid_total += 1;
            let _ = entry.invalid_by_peer.entry(*peer).and_modify(|c| *c += 1).or_insert(1);
        }
    }

    /// Returns a snapshot of current progress for the given SSA, if a counter entry exists.
    fn snapshot_progress(&self, ssa_id: &SsaId<S::Pseudonym>) -> Option<SsaRecoveryProgress<S::Pseudonym>> {
        let entry = self.ssa_counters.get(ssa_id)?;
        let e = entry.lock();
        Some(SsaRecoveryProgress {
            ssa_id: *ssa_id,
            useful_shares: e.useful_shares,
            target_useful_shares: e.target_useful_shares,
            recovered_polynomials: e.recovered_polynomials,
        })
    }

    /// Returns a combined progress + fault snapshot for drain-eligibility decisions.
    ///
    /// Returns `None` if no counter entry exists (e.g. after [`retire_ssa`](SsaReconstructor::retire_ssa)).
    pub fn drain_snapshot(&self, ssa_id: &SsaId<S::Pseudonym>) -> Option<SsaDrainSnapshot<S::Pseudonym>> {
        let entry = self.ssa_counters.get(ssa_id)?;
        let e = entry.lock();
        Some(SsaDrainSnapshot {
            progress: SsaRecoveryProgress {
                ssa_id: *ssa_id,
                useful_shares: e.useful_shares,
                target_useful_shares: e.target_useful_shares,
                recovered_polynomials: e.recovered_polynomials,
            },
            invalid_total: e.invalid_total,
        })
    }

    fn process_verified_ack(
        &self,
        _peer: &OffchainPublicKey,
        ack: HalfKey,
        ack_challenge: HalfKeyChallenge,
        awaiting_ack_from_peer: &moka::sync::Cache<HalfKeyChallenge, TaggedEncryptedPartialSsaShare<S>>,
    ) -> Result<ProcessedAckResult<S>, PixError<S::Pseudonym>> {
        let Some(share) = awaiting_ack_from_peer.remove(&ack_challenge) else {
            tracing::trace!(?ack_challenge, "received ack for unknown share");
            return Ok(ProcessedAckResult::NoProgress);
        };

        let spi = share.ssa_polynomial_id().ok_or(PixError::ShareIsEmpty)?;
        let ssa_id = *spi.as_ref();

        let reconstructor = self.ssa_verifiers.get(&spi).ok_or(PixError::MissingVerifier)?;

        // The share cannot be empty at this point because we prevent empty share insertions
        let partial_share = share.partial_share.decrypt(spi.pseudonym(), &ack)?;

        match reconstructor.lock().add_share(share.nonce, partial_share) {
            Ok(AddShareOutcome::Duplicate) => {
                tracing::trace!(%spi, "duplicate evaluation identifier — not useful");
                Ok(ProcessedAckResult::Duplicate)
            }
            Ok(AddShareOutcome::Surplus) => {
                tracing::trace!(%spi, "share after polynomial reconstruction — surplus");
                Ok(ProcessedAckResult::Surplus)
            }
            Ok(AddShareOutcome::Useful) => {
                tracing::trace!(%spi, "useful share collected, part not yet complete");
                Ok(ProcessedAckResult::UsefulShare(ssa_id))
            }
            Ok(AddShareOutcome::Completed(ssa_part)) => {
                tracing::trace!(%spi, "ssa part complete");
                let builder = self.ssa_builders.get(&ssa_id).ok_or(PixError::MissingSsaCommitment)?;

                let mut builder_guard = builder.lock();
                let ssa = builder_guard.add_recovered_ssa_part(spi.poly_index(), ssa_part)?;
                match ssa {
                    Some(scalar) => {
                        let Some(ssa) = S::scalar_to_private_key(scalar) else {
                            tracing::error!(%spi, "ssa reconstruction failed");
                            return Err(PixError::InvalidSsa);
                        };
                        tracing::info!(%ssa_id, "ssa recovered");
                        Ok(ProcessedAckResult::FullRecovery(RecoveredSsa { ssa_id, ssa }))
                    }
                    None => {
                        tracing::trace!(%spi, "ssa part complete, ssa not yet complete");
                        // Check early threshold while we hold the lock
                        if builder_guard.check_early_threshold(self.cfg.early_recovery_threshold) {
                            tracing::info!(%ssa_id, "early recovery threshold reached");
                            Ok(ProcessedAckResult::EarlyRecovery(ssa_id))
                        } else {
                            Ok(ProcessedAckResult::PolynomialPartComplete(ssa_id))
                        }
                    }
                }
            }
            Err(PixError::VsssError(vsss_rs::Error::InvalidShare)) => {
                tracing::error!(%spi, "share verification failed");
                Err(PixError::InvalidShare(*spi.pseudonym(), spi.ssa_index()))
            }
            Err(e) => Err(e),
        }
    }
}

/// RAII guard that owns an Exit SSA commitment in the reconstructor.
///
/// Created by [`SsaReconstructor::new_guarded_exit_commitment`]. On [`Drop`],
/// the SSA is retired from all internal caches via [`SsaReconstructor::retire_ssa`],
/// preventing orphaned commitments on any error path or task cancellation.
///
/// This is a move-only type (no `Clone`, no `Copy`) to maintain single-ownership
/// of the commitment lifecycle.
#[must_use = "SsaCommitmentGuard does nothing if unused — the SSA will be immediately retired"]
pub struct SsaCommitmentGuard<S: PixSpec + Clone>(Option<(Arc<SsaReconstructor<S>>, SsaId<S::Pseudonym>)>);

impl<S: PixSpec + Clone> std::fmt::Debug for SsaCommitmentGuard<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SsaCommitmentGuard")
            .field("ssa_id", &self.0.as_ref().map(|(_, id)| id))
            .finish()
    }
}

impl<S: PixSpec + Clone> SsaCommitmentGuard<S> {
    /// Creates a new guard that will retire the SSA on drop.
    fn new(reconstructor: Arc<SsaReconstructor<S>>, ssa_id: SsaId<S::Pseudonym>) -> Self {
        Self(Some((reconstructor, ssa_id)))
    }

    /// Returns the [`SsaId`] tracked by this guard.
    pub fn ssa_id(&self) -> &SsaId<S::Pseudonym> {
        &self
            .0
            .as_ref()
            .expect("SsaCommitmentGuard invariant: Option is always Some until disarm")
            .1
    }

    /// Consumes the guard without retiring the SSA, returning the tracked [`SsaId`].
    ///
    /// Use this when ownership of the commitment is transferred to another owner
    /// (e.g., `SsaRetirementGuard`).
    pub fn disarm(mut self) -> SsaId<S::Pseudonym> {
        let (_, ssa_id) = self.0.take().expect("SsaCommitmentGuard disarmed twice");
        ssa_id
    }
}

impl<S: PixSpec + Clone> Drop for SsaCommitmentGuard<S> {
    fn drop(&mut self) {
        if let Some((ref reconstructor, ref ssa_id)) = self.0 {
            reconstructor.retire_ssa(ssa_id);
        }
    }
}

impl<S: PixSpec + Clone> SsaReconstructor<S> {
    /// Like [`ExitAcknowledgementShareProcessor::new_exit_commitment`] but returns an RAII
    /// [`SsaCommitmentGuard`] that retires the SSA on drop.
    ///
    /// The guard ensures the commitment is always cleaned up if the caller fails to
    /// transfer it to a permanent owner (e.g., the PIX action driver's retirement guard).
    ///
    /// On duplicate, returns [`PixError::DuplicateCommitment`] — no guard is created.
    pub fn new_guarded_exit_commitment(
        self: &Arc<Self>,
        id: SsaId<S::Pseudonym>,
        polys_per_ssa: usize,
        shares_per_poly: usize,
    ) -> Result<(PixGroup<S>, SsaCommitmentGuard<S>), PixError<S::Pseudonym>> {
        let exit_commitment = self.new_exit_commitment(id, polys_per_ssa, shares_per_poly)?;
        let guard = SsaCommitmentGuard::new(self.clone(), id);
        Ok((exit_commitment, guard))
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
        if polys_per_ssa > MAX_POLYS_PER_SSA as usize || !(2..=MAX_POLY_THRESHOLD as usize).contains(&shares_per_poly) {
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
                let polys = ssa_builder.num_polys() as u16;
                let threshold = ssa_reconstructors.first().map(|r| r.verifier.min_shares()).unwrap_or(0);
                let target = polys as u64 * threshold as u64;
                self.ssa_builders
                    .insert(ssa_id, std::sync::Arc::new(parking_lot::Mutex::new(ssa_builder)));

                // Track verifier IDs for this SSA so retire_ssa can invalidate them
                let mut verifier_ids = Vec::with_capacity(ssa_reconstructors.len());
                for ssa_reconstructor in ssa_reconstructors {
                    let spi = ssa_reconstructor.verifier.spi;
                    verifier_ids.push(spi);
                    self.ssa_verifiers
                        .insert(spi, std::sync::Arc::new(parking_lot::Mutex::new(ssa_reconstructor)));
                }
                self.ssa_to_verifier_ids.insert(ssa_id, verifier_ids);

                // Initialize the counter entry when the commitment becomes verifiable.
                // Use get_with so a pre-existing counter (e.g. after builder TTI eviction
                // and re-commitment) is not clobbered — counters must survive across
                // builder invalidation for the full TTL duration.
                self.ssa_counters.get_with(ssa_id, || {
                    std::sync::Arc::new(parking_lot::Mutex::new(SsaCounterEntry {
                        useful_shares: 0,
                        target_useful_shares: target,
                        recovered_polynomials: 0,
                        invalid_total: 0,
                        invalid_by_peer: HashMap::new(),
                    }))
                });

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

        // Ordered aggregation per SSA: collect terminal events for each touched SSA,
        // using IndexSet for deterministic first-seen ordering.
        let mut progress_touched: IndexSet<SsaId<S::Pseudonym>> = IndexSet::new();
        let mut terminal_events: Vec<ShareResolution<S::Pseudonym, S::AddressPrivateKey>> = Vec::new();
        let mut invalid_ssas: IndexSet<SsaId<S::Pseudonym>> = IndexSet::new();

        for (ack, ack_challenge) in half_keys_challenges {
            match self.process_verified_ack(&peer, ack, ack_challenge, &awaiting_ack_from_peer) {
                Ok(ProcessedAckResult::FullRecovery(ssa)) => {
                    // Record the share that triggered full recovery.
                    self.record_useful_share(&ssa.ssa_id);
                    self.record_completed_part(&ssa.ssa_id);
                    progress_touched.insert(ssa.ssa_id);
                    terminal_events.push(ShareResolution::RecoveredSsa(ssa));
                }
                Ok(ProcessedAckResult::EarlyRecovery(ssa_id)) => {
                    // Record the share that triggered early recovery.
                    self.record_useful_share(&ssa_id);
                    self.record_completed_part(&ssa_id);
                    progress_touched.insert(ssa_id);
                    terminal_events.push(ShareResolution::AlmostRecoveredSsa(ssa_id));
                }
                Ok(ProcessedAckResult::UsefulShare(ssa_id)) => {
                    progress_touched.insert(ssa_id);
                    self.record_useful_share(&ssa_id);
                }
                Ok(ProcessedAckResult::PolynomialPartComplete(ssa_id)) => {
                    progress_touched.insert(ssa_id);
                    self.record_useful_share(&ssa_id);
                    self.record_completed_part(&ssa_id);
                }
                Ok(ProcessedAckResult::Duplicate)
                | Ok(ProcessedAckResult::Surplus)
                | Ok(ProcessedAckResult::NoProgress) => {}
                Err(PixError::ShareIsEmpty) => tracing::trace!(%peer, "received empty share"),
                Err(PixError::InvalidShare(pseudonym, ssa_index)) => {
                    let ssa_id = SsaId::new(pseudonym, ssa_index);
                    progress_touched.insert(ssa_id);
                    invalid_ssas.insert(ssa_id);
                    self.record_invalid_share(&peer, &ssa_id);
                }
                Err(error) => {
                    tracing::error!(%error, "failed to process acknowledgement");
                }
            }
        }

        // Build the result in deterministic order:
        // 1. One Progress snapshot per touched SSA (final post-batch counters)
        // 2. One InvalidShares per (peer, SSA) with absolute total
        // 3. Terminal events (AlmostRecoveredSsa, RecoveredSsa) that same-batch progressed
        let mut res = Vec::with_capacity(progress_touched.len() + terminal_events.len());

        for ssa_id in &progress_touched {
            if let Some(progress) = self.snapshot_progress(ssa_id) {
                res.push(ShareResolution::Progress(progress));
            }
        }

        // Emit one InvalidShares per touched SSA where faults were observed
        for ssa_id in &invalid_ssas {
            let aggregate_total = self
                .ssa_counters
                .get(ssa_id)
                .map(|entry| {
                    let e = entry.lock();
                    e.invalid_total
                })
                .unwrap_or(0);
            // Only emit if we have a positive count (should always be true here)
            if aggregate_total > 0 {
                res.push(ShareResolution::InvalidShares {
                    peer: Box::new(peer),
                    ssa_id: *ssa_id,
                    // Emit the cross-peer aggregate total so the supervisor
                    // correctly tracks limit enforcement. The peer field is
                    // retained for diagnostics/logging.
                    observed_total: aggregate_total,
                });
            }
        }

        // Append terminal events (their progress snapshot already precedes them)
        res.extend(terminal_events);

        Ok(res)
    }

    fn retire_ssa(&self, ssa_id: &SsaId<S::Pseudonym>) {
        // Remove commitment builder
        self.commitment_builder.invalidate(ssa_id);
        // Remove SSA builder
        self.ssa_builders.invalidate(ssa_id);
        // Remove all verifiers for this SSA's polynomials
        if let Some(ids) = self.ssa_to_verifier_ids.get(ssa_id) {
            for id in &ids {
                self.ssa_verifiers.invalidate(id);
            }
        }
        self.ssa_to_verifier_ids.invalidate(ssa_id);
        // Remove counter entry
        self.ssa_counters.invalidate(ssa_id);
        tracing::trace!(%ssa_id, "ssa retired");
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
            peer.public(),
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
    fn retire_ssa_removes_builders_verifiers_and_counters() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        // Create a commitment so builders, verifiers, and counters are populated
        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        let mut poly_map = HashMap::new();
        for poly in 0..2 {
            poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        }
        reconstructor.insert_coefficient_commitments(ssa_id, 0, poly_map.into_iter())?;
        let mut poly_map2 = HashMap::new();
        for poly in 0..2 {
            poly_map2.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        }
        reconstructor.insert_coefficient_commitments(ssa_id, 1, poly_map2.into_iter())?;

        // Verify SSA builder, counters, and verifier map are populated
        assert!(
            reconstructor.ssa_builders.get(&ssa_id).is_some(),
            "ssa_builders should exist"
        );
        assert!(
            reconstructor.ssa_counters.get(&ssa_id).is_some(),
            "ssa_counters should exist"
        );
        assert!(
            reconstructor.ssa_to_verifier_ids.get(&ssa_id).is_some(),
            "ssa_to_verifier_ids should exist"
        );

        // Capture verifier IDs before retiring.
        let spis: Vec<_> = reconstructor.ssa_to_verifier_ids.get(&ssa_id).unwrap();

        // Now retire the SSA
        reconstructor.retire_ssa(&ssa_id);

        // All entries must be gone
        assert!(reconstructor.ssa_builders.get(&ssa_id).is_none());
        assert!(reconstructor.ssa_counters.get(&ssa_id).is_none());
        assert!(reconstructor.ssa_to_verifier_ids.get(&ssa_id).is_none());

        // Individual polynomial verifiers must also be cleaned up.
        for spi in &spis {
            assert!(
                reconstructor.ssa_verifiers.get(spi).is_none(),
                "verifier {spi:?} should have been retired"
            );
        }

        Ok(())
    }

    #[test]
    fn retire_and_recreate_same_pseudonym() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        for coeff in 0..2 {
            let mut poly_map = HashMap::new();
            for poly in 0..2 {
                poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
            }
            reconstructor.insert_coefficient_commitments(ssa_id, coeff as CoefficientIndex, poly_map.into_iter())?;
        }

        // Capture verifier IDs before retiring.
        let spis: Vec<_> = reconstructor.ssa_to_verifier_ids.get(&ssa_id).unwrap();
        reconstructor.retire_ssa(&ssa_id);

        // Verifiers for the retired SSA must be cleaned up.
        for spi in &spis {
            assert!(
                reconstructor.ssa_verifiers.get(spi).is_none(),
                "verifier {spi:?} should have been retired"
            );
        }

        // Re-create a new SSA with a different index
        let ssa_id2 = SsaId::new(pseudonym, 2.try_into()?);
        reconstructor.new_exit_commitment(ssa_id2, 2, 2)?;
        for coeff in 0..2 {
            let mut poly_map = HashMap::new();
            for poly in 0..2 {
                poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
            }
            reconstructor.insert_coefficient_commitments(ssa_id2, coeff as CoefficientIndex, poly_map.into_iter())?;
        }

        // New SSA should have its own tracker entries
        assert!(
            reconstructor.ssa_builders.get(&ssa_id2).is_some(),
            "new SSA should have builders"
        );
        assert!(
            reconstructor.ssa_to_verifier_ids.get(&ssa_id2).is_some(),
            "new SSA should have verifier map"
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Counter correctness tests
    // -----------------------------------------------------------------------

    #[test]
    fn progress_increments_once_per_unique_verified_share() -> anyhow::Result<()> {
        // 3 polynomials, threshold=2, surplus=0 → 6 shares total.
        // After first 3 useful shares → useful_shares=3.
        // After all 6             → useful_shares=6.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 3,
            threshold: 2,
            surplus_shares: 0,
        });

        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 3, 2)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // Batch 1: 3 useful shares
        let mut acks = Vec::new();
        for _ in 0..3 {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let gs = generator.next_share(&pseudonym, &msg)?.unwrap();
            let ack = HalfKey::random();
            let ack_challenge = ack.to_challenge()?;
            let enc = gs.share.encrypt(&gs.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                ack_challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
            )?;
            acks.push(VerifiedAcknowledgement::new(ack, &peer).leak());
        }

        let res = reconstructor.acknowledge_shares(*peer.public(), acks)?;
        let progress: Vec<_> = res
            .iter()
            .filter_map(|r| {
                if let ShareResolution::Progress(p) = r {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(progress.len(), 1, "expected one Progress per SSA per batch");
        assert_eq!(progress[0].useful_shares, 3);

        // Batch 2: remaining 3 shares
        let mut acks2 = Vec::new();
        for _ in 0..3 {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let gs = generator.next_share(&pseudonym, &msg)?.unwrap();
            let ack = HalfKey::random();
            let ack_challenge = ack.to_challenge()?;
            let enc = gs.share.encrypt(&gs.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                ack_challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
            )?;
            acks2.push(VerifiedAcknowledgement::new(ack, &peer).leak());
        }

        let res2 = reconstructor.acknowledge_shares(*peer.public(), acks2)?;
        let progress2: Vec<_> = res2
            .iter()
            .filter_map(|r| {
                if let ShareResolution::Progress(p) = r {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(progress2.len(), 1, "expected one Progress per SSA per batch");
        assert_eq!(progress2[0].useful_shares, 6);

        Ok(())
    }

    #[test]
    fn duplicate_share_is_not_useful() -> anyhow::Result<()> {
        // 2 polys, threshold=2, surplus=0 → 4 shares total when fully drawn.
        // Insert: poly0:share1, poly0:share1 (duplicate), poly1:share1.
        // useful_shares must be 2 (the duplicate does not count).
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });

        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // Generate shares: poly0:1, poly0:2(completing), poly1:3, poly1:4(completing)
        let msg1: [u8; 20] = hopr_types::crypto_random::random_bytes();
        let share1 = generator.next_share(&pseudonym, &msg1)?.unwrap(); // poly0
        // Skip poly0's completing share — we want the duplicate BEFORE the poly completes
        let msg2: [u8; 20] = hopr_types::crypto_random::random_bytes();
        let share2 = generator.next_share(&pseudonym, &msg2)?.unwrap(); // poly1

        // Insert share1 (poly0) encrypted with ack1 — legitimate first copy
        let ack1 = HalfKey::random();
        let challenge1 = ack1.to_challenge()?;
        let enc1 = share1.share.clone().encrypt(&share1.id, &ack1)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            challenge1,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg1, enc1)?,
        )?;

        // Insert the SAME share1 (poly0) encrypted with ack2 — duplicate identifier
        let ack2 = HalfKey::random();
        let challenge2 = ack2.to_challenge()?;
        let enc2 = share1.share.encrypt(&share1.id, &ack2)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            challenge2,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg1, enc2)?,
        )?;

        // Insert share2 (poly1) — legitimate
        let ack3 = HalfKey::random();
        let challenge3 = ack3.to_challenge()?;
        let enc3 = share2.share.encrypt(&share2.id, &ack3)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            challenge3,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg2, enc3)?,
        )?;

        let acks = vec![
            VerifiedAcknowledgement::new(ack1, &peer).leak(),
            VerifiedAcknowledgement::new(ack2, &peer).leak(),
            VerifiedAcknowledgement::new(ack3, &peer).leak(),
        ];
        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;

        let progress: Vec<_> = resolutions
            .iter()
            .filter_map(|r| {
                if let ShareResolution::Progress(p) = r {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(progress.len(), 1);
        assert_eq!(
            progress[0].useful_shares, 2,
            "duplicate must not increment useful_shares"
        );

        Ok(())
    }

    #[test]
    fn surplus_share_is_not_useful() -> anyhow::Result<()> {
        // 2 polys, threshold=2, surplus_shares=1 → 3 shares per poly (6 total).
        // Submit poly0:3 shares: 2 reach threshold (completing poly0 → 2 useful),
        // the 3rd arrives after polynomial is done → Surplus.
        // Submit poly1:1 share (useful).
        // Expected useful_shares = 3 (the surplus is excluded).
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        });

        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // Generate 6 shares (poly0:3, poly1:3)
        let mut shares_data: Vec<([u8; 20], crate::types::GeneratedShare<TestSpec>)> = Vec::new();
        for _ in 0..6 {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let gs = generator.next_share(&pseudonym, &msg)?.unwrap();
            shares_data.push((msg, gs));
        }

        // Insert first 4 shares: poly0:3 (1st, 2nd=completing, 3rd=surplus) + poly1:1 (1st=useful)
        let mut acks_and_data = Vec::new();
        for (msg, gs) in &shares_data[..4] {
            let ack = HalfKey::random();
            let challenge = ack.to_challenge()?;
            let enc = gs.share.clone().encrypt(&gs.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, msg, enc)?,
            )?;
            acks_and_data.push((ack, challenge));
        }

        let acks: Vec<_> = acks_and_data
            .iter()
            .map(|(ack, _)| VerifiedAcknowledgement::new(*ack, &peer).leak())
            .collect();
        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;

        let progress: Vec<_> = resolutions
            .iter()
            .filter_map(|r| {
                if let ShareResolution::Progress(p) = r {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].useful_shares, 3, "surplus share must not count");
        assert_eq!(progress[0].target_useful_shares, 4, "2 polys × 2 threshold");

        Ok(())
    }

    #[test]
    fn invalid_shares_reports_absolute_totals_per_ssa() -> anyhow::Result<()> {
        // Submit a share with a zero (invalid) scalar that fails vsss verification.
        // The reconstructor must emit InvalidShares with observed_total=1.
        // Submit a second invalid share for the same (peer, SSA) → observed_total=2.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        });

        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // Helper: insert a bogus encrypted share (zero scalar) for the given SPI
        let insert_bad_share = |spi: SsaPolynomialId<SimplePseudonym>,
                                reconstructor: &SsaReconstructor<TestSpec>,
                                peer: &OffchainKeypair|
         -> anyhow::Result<HalfKey> {
            let ack = HalfKey::random();
            let challenge = ack.to_challenge()?;
            let bad_msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let bad_enc = PartialSsaShare::<TestSpec>::default().encrypt(&spi, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &bad_msg, bad_enc)?,
            )?;
            Ok(ack)
        };

        let spi0 = SsaPolynomialId::new(ssa_id, 0);
        let ack1 = insert_bad_share(spi0, &reconstructor, &peer)?;
        let ack2 = insert_bad_share(spi0, &reconstructor, &peer)?;

        let acks = vec![
            VerifiedAcknowledgement::new(ack1, &peer).leak(),
            VerifiedAcknowledgement::new(ack2, &peer).leak(),
        ];
        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;

        let invalid: Vec<_> = resolutions
            .iter()
            .filter_map(|r| {
                if let ShareResolution::InvalidShares {
                    peer: p,
                    ssa_id: id,
                    observed_total,
                } = r
                {
                    Some((**p, *id, *observed_total))
                } else {
                    None
                }
            })
            .collect();
        // Should be exactly one InvalidShares per (peer, SSA) per batch with absolute total
        assert_eq!(invalid.len(), 1, "one InvalidShares entry per (peer, SSA) per batch");
        assert_eq!(invalid[0].0, *peer.public());
        assert_eq!(invalid[0].1, ssa_id);
        assert_eq!(
            invalid[0].2, 2,
            "observed_total must be absolute (2 after two bad shares)"
        );

        Ok(())
    }

    #[test]
    fn invalid_shares_cross_peer_aggregation() -> anyhow::Result<()> {
        // Two distinct peers each submit invalid shares for the same SSA.
        // The reconstructor must emit the cross-peer aggregate total, not a
        // per-peer count.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        });

        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        let peer_a = OffchainKeypair::random();
        let peer_b = OffchainKeypair::random();
        let spi = SsaPolynomialId::new(ssa_id, 0);

        // Helper: insert a bogus encrypted share for the given peer.
        let insert_bad_share = |spi: SsaPolynomialId<SimplePseudonym>,
                                reconstructor: &SsaReconstructor<TestSpec>,
                                peer: &OffchainKeypair|
         -> anyhow::Result<HalfKey> {
            let ack = HalfKey::random();
            let challenge = ack.to_challenge()?;
            let bad_msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let bad_enc = PartialSsaShare::<TestSpec>::default().encrypt(&spi, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &bad_msg, bad_enc)?,
            )?;
            Ok(ack)
        };

        // Peer A submits 2 invalid shares.
        let a_a1 = insert_bad_share(spi, &reconstructor, &peer_a)?;
        let a_a2 = insert_bad_share(spi, &reconstructor, &peer_a)?;
        let acks_a = vec![
            VerifiedAcknowledgement::new(a_a1, &peer_a).leak(),
            VerifiedAcknowledgement::new(a_a2, &peer_a).leak(),
        ];
        let resolutions_a = reconstructor.acknowledge_shares(*peer_a.public(), acks_a)?;
        let invalid_a: Vec<_> = resolutions_a
            .iter()
            .filter_map(|r| {
                if let ShareResolution::InvalidShares { observed_total, .. } = r {
                    Some(*observed_total)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(invalid_a.len(), 1, "expected one InvalidShares event after peer a");
        assert_eq!(invalid_a[0], 2, "peer a's 2 bad shares → observed_total 2");

        // Peer B submits 1 invalid share → observed_total must be 3 (2 + 1).
        let b_ack = insert_bad_share(spi, &reconstructor, &peer_b)?;
        let acks_b = vec![VerifiedAcknowledgement::new(b_ack, &peer_b).leak()];
        let resolutions_b = reconstructor.acknowledge_shares(*peer_b.public(), acks_b)?;
        let invalid_b: Vec<_> = resolutions_b
            .iter()
            .filter_map(|r| {
                if let ShareResolution::InvalidShares { observed_total, .. } = r {
                    Some(*observed_total)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(invalid_b.len(), 1, "expected one InvalidShares event after peer b");
        assert_eq!(invalid_b[0], 3, "total must be cross-peer sum (2+1 = 3)");

        Ok(())
    }

    #[test]
    fn recovered_polynomials_tracks_completed_parts() -> anyhow::Result<()> {
        // 2 polys, threshold=2, surplus=0 → 4 shares total (poly0:1, poly0:2, poly1:1, poly1:2).
        // The generator exhausts each poly before moving to the next, so the
        // share order is: poly0:1, poly0:2(completes), poly1:1, poly1:2(completes).
        //
        // Batch 1: 1 share  (poly0:1)         → useful=1, recovered_polynomials=0
        // Batch 2: 3 shares (poly0:2+poly1:1+poly1:2) → useful=4, recovered_polynomials=2
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });

        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let _server_commitment = reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

        // Helper: generate, encrypt, insert and return acks for N shares
        fn prepare_acks(
            generator: &SsaShareGenerator<TestSpec>,
            pseudonym: &SimplePseudonym,
            reconstructor: &SsaReconstructor<TestSpec>,
            peer: &OffchainKeypair,
            count: usize,
        ) -> anyhow::Result<Vec<HalfKey>> {
            let mut half_keys = Vec::new();
            for _ in 0..count {
                let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
                let gs = generator.next_share(pseudonym, &msg)?.unwrap();
                let ack = HalfKey::random();
                let challenge = ack.to_challenge()?;
                let enc = gs.share.encrypt(&gs.id, &ack)?;
                reconstructor.insert_encrypted_share(
                    peer.public(),
                    challenge,
                    TaggedEncryptedPartialSsaShare::new(*pseudonym, &msg, enc)?,
                )?;
                half_keys.push(ack);
            }
            Ok(half_keys)
        }

        // Batch 1: 1 share — first polynomial not yet complete
        let hk1 = prepare_acks(&generator, &pseudonym, &reconstructor, &peer, 1)?;
        let acks1: Vec<_> = hk1
            .iter()
            .map(|k| VerifiedAcknowledgement::new(*k, &peer).leak())
            .collect();
        let res1 = reconstructor.acknowledge_shares(*peer.public(), acks1)?;
        let p1: Vec<_> = res1
            .iter()
            .filter_map(|r| {
                if let ShareResolution::Progress(p) = r {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(p1.len(), 1);
        assert_eq!(p1[0].useful_shares, 1);
        assert_eq!(
            p1[0].recovered_polynomials, 0,
            "first share: no polynomial should be complete yet"
        );

        // Batch 2: remaining 3 shares — both polys complete
        let hk2 = prepare_acks(&generator, &pseudonym, &reconstructor, &peer, 3)?;
        let acks2: Vec<_> = hk2
            .iter()
            .map(|k| VerifiedAcknowledgement::new(*k, &peer).leak())
            .collect();
        let res2 = reconstructor.acknowledge_shares(*peer.public(), acks2)?;
        let p2: Vec<_> = res2
            .iter()
            .filter_map(|r| {
                if let ShareResolution::Progress(p) = r {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(p2.len(), 1);
        assert_eq!(p2[0].useful_shares, 4);
        assert_eq!(p2[0].recovered_polynomials, 2, "both polynomials should be complete");

        // Full SSA must be recovered
        assert!(
            res2.iter()
                .any(|r| matches!(r, ShareResolution::RecoveredSsa(r) if r.ssa_id == ssa_id)),
            "expected RecoveredSsa"
        );

        Ok(())
    }

    #[test]
    fn counters_survive_builder_eviction() -> anyhow::Result<()> {
        // ssa_counters lives in a TTL-only cache, independent of ssa_builders (TTI).
        // After invalidating just the builder, the counter entry must still exist.
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        let mut poly_map = HashMap::new();
        for poly in 0..2 {
            poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        }
        reconstructor.insert_coefficient_commitments(ssa_id, 0, poly_map.into_iter())?;
        let mut poly_map2 = HashMap::new();
        for poly in 0..2 {
            poly_map2.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        }
        reconstructor.insert_coefficient_commitments(ssa_id, 1, poly_map2.into_iter())?;

        // Confirm both builder and counter exist
        assert!(reconstructor.ssa_builders.get(&ssa_id).is_some());
        assert!(reconstructor.ssa_counters.get(&ssa_id).is_some());

        // Record progress before eviction to demonstrate non-zero values survive.
        reconstructor.record_useful_share(&ssa_id);
        reconstructor.record_completed_part(&ssa_id);

        // Evict only the builder (simulating moka TTI eviction)
        reconstructor.ssa_builders.invalidate(&ssa_id);
        assert!(
            reconstructor.ssa_builders.get(&ssa_id).is_none(),
            "builder must be evicted"
        );

        // Counter must survive independently
        assert!(
            reconstructor.ssa_counters.get(&ssa_id).is_some(),
            "counter must survive builder eviction (TTL-only cache)"
        );

        // Verify counter contents survived with recorded values
        let entry = reconstructor.ssa_counters.get(&ssa_id).unwrap();
        let e = entry.lock();
        assert_eq!(
            e.useful_shares, 1,
            "recorded useful share must survive builder eviction"
        );
        assert_eq!(
            e.recovered_polynomials, 1,
            "recorded completed part must survive builder eviction"
        );

        Ok(())
    }
}
