mod utils;

use ahash::HashSetExt;
use hopr_types::{
    crypto::{
        crypto_traits::elliptic_curve::Field,
        prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    },
    internal::prelude::Acknowledgement,
};
use utils::{SsaBuilder, SsaCommitmentBuilder, SsaPartBuilder};
use validator::Validate;

use crate::{
    CoefficientIndex, ExitAcknowledgementShareProcessor, Group, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, PixGroup,
    PixGroupRepr, PixScalar, PixSpec, PolynomialIndex, RecoveredSsa, ShareResolution, SsaCommitmentProof,
    SsaCommitmentState, SsaPolynomialId, TaggedEncryptedPartialSsaShare, errors::PixError, types::SsaId,
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

/// An acknowledgement that arrived before its polynomial's verifier was installed.
///
/// The peer is carried per entry because a polynomial's shares are spread across return paths, and
/// therefore across first-relayers: one bucket can hold deferred acks from several peers.
type DeferredAck = (OffchainPublicKey, HalfKeyChallenge, HalfKey);

/// Deferred acknowledgements for one polynomial, drained in one shot when its verifier installs.
///
/// A plain `Vec` rather than a nested cache: the bucket is only ever appended to and then drained
/// whole, so per-entry cache bookkeeping (and its ~200 B overhead per entry) buys nothing.
type DeferredAckBucket = std::sync::Arc<parking_lot::Mutex<Vec<DeferredAck>>>;

/// Cap on deferred acknowledgements held for a single polynomial.
///
/// A conforming Entry emits `threshold + surplus` shares per polynomial — 96 at the default
/// dimensions — across all return paths combined, so this cannot be reached without the peer
/// exceeding its own share budget. Anything above the cap is dropped rather than buffered.
const MAX_DEFERRED_ACKS_PER_POLYNOMIAL: usize = 128;

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
    /// Acknowledgements that arrived before their polynomial's verifier was installed, bucketed by
    /// polynomial.
    ///
    /// ## Why bucketed by polynomial
    ///
    /// The bucket key is exactly the thing whose arrival unblocks the entries inside it, so a
    /// bucket is drained once, by the installation of its own verifier, and never scanned
    /// speculatively. That is what keeps [`acknowledge_shares`] free of retry work: it only ever
    /// *appends* to a bucket.
    ///
    /// The previous per-peer stash had to be re-scanned in full on every `acknowledge_shares` call,
    /// because a per-peer key says nothing about which entries have become viable. That is
    /// quadratic in the number of acks received while a cycle's commitments are in flight, and the
    /// per-peer key aggregates across every Session sharing a first-relayer.
    ///
    /// `max_capacity` allows two full cycles' worth of polynomials — the pipelining factor — so a
    /// polynomial can never be denied a bucket by another polynomial's traffic. That headroom is
    /// deliberate: a size eviction here silently drops real shares, and only the surplus absorbs
    /// that. In practice the `max_ack_await_time` TTL is the operative bound, since a bucket only
    /// exists at all when shares outrun the constant-term pass that makes their polynomial
    /// reconstructible.
    pending_acks: moka::sync::Cache<SsaPolynomialId<S::Pseudonym>, DeferredAckBucket>,
    /// Resolutions produced by draining deferred-ack buckets at verifier-installation time, waiting
    /// to be picked up by the next [`acknowledge_shares`] call.
    ///
    /// Draining happens on the commitment path (`insert_coefficient_commitments`), which is where
    /// the verifier that unblocks the acks is installed. That deliberately keeps the share
    /// verification off the acknowledgement hot path, but it also means the resolutions surface
    /// somewhere that has no route to the upper layer — hence this hand-off. Acks flow continuously
    /// while a Session is live, so pickup latency is one ack batch.
    ready_resolutions: parking_lot::Mutex<Vec<ShareResolution<S::Pseudonym, S::AddressPrivateKey>>>,
    /// Length of [`ready_resolutions`](Self::ready_resolutions), so the common case (nothing to pick
    /// up) costs one relaxed load instead of a mutex acquisition on every ack batch.
    ready_resolutions_len: std::sync::atomic::AtomicUsize,
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
    /// The share is valid but its polynomial's verifier is not installed yet, so it cannot be
    /// checked. Deferral, not failure: the ack is bucketed under this
    /// [`SsaPolynomialId`] and retried once the verifier arrives.
    VerifierNotReady(SsaPolynomialId<<S as PixSpec>::Pseudonym>),
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
            // One bucket per polynomial of a cycle, expiring on the same clock as the shares they
            // belong to: an ack whose share has left `awaiting_acks` can never be used again, so
            // there is nothing to keep. `time_to_live`, not idle — appending to a bucket must not
            // extend the life of entries already in it.
            pending_acks: moka::sync::CacheBuilder::new(2 * MAX_POLYS_PER_SSA as u64)
                .time_to_live(cfg.max_ack_await_time)
                .build(),
            ready_resolutions: parking_lot::Mutex::new(Vec::new()),
            ready_resolutions_len: std::sync::atomic::AtomicUsize::new(0),
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
            let spi = SsaPolynomialId::new(ssa_id, poly_index);
            self.ssa_verifiers.invalidate(&spi);
            // Deferred acks for a retired cycle can never be redeemed — their verifier will not
            // come back and their shares are about to expire.
            self.pending_acks.invalidate(&spi);
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

        let Some(reconstructor) = self.ssa_verifiers.get(&spi) else {
            // Not an error: the constant-term set is still incomplete, so no part builder exists
            // yet. Leave the share in `awaiting_acks` and hand the caller the key it needs to
            // bucket the ack.
            return Ok(ProcessedAckResult::VerifierNotReady(spi));
        };

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
                // and may be differently handled by the upper-layer components.
                //
                // Almost always this means the polynomial's reconstructed constant term did not
                // open its commitment, in which case the offending share is one of the `threshold`
                // that went into it and cannot be singled out. The whole cycle is lost either way,
                // since the SSA needs every polynomial.
                tracing::error!(%spi, "ssa part failed to open its commitment");
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

    /// Buckets an acknowledgement whose polynomial verifier has not been installed yet.
    ///
    /// O(1) — this is the entire cost the acknowledgement path pays for a deferral.
    fn defer_ack(&self, spi: SsaPolynomialId<S::Pseudonym>, deferred: DeferredAck) {
        let bucket = self
            .pending_acks
            .get_with(spi, || std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())));
        {
            let mut bucket = bucket.lock();
            if bucket.len() >= MAX_DEFERRED_ACKS_PER_POLYNOMIAL {
                // Only reachable if the peer emits more shares for one polynomial than its own
                // `threshold + surplus` budget allows, so the excess is almost certainly duplicate.
                tracing::warn!(
                    %spi,
                    cap = MAX_DEFERRED_ACKS_PER_POLYNOMIAL,
                    "dropping deferred acknowledgement: polynomial bucket is full"
                );
                return;
            }
            bucket.push(deferred);
        }

        // Close the race against a concurrent installation. The decision to defer was made on a
        // verifier lookup that missed; if the verifier has appeared since, the drain that would have
        // redeemed this ack has already run and nothing else will come for it. Probing with
        // `contains_key` rather than `get` keeps this from refreshing the verifier's idle timer.
        if self.ssa_verifiers.contains_key(&spi) {
            self.drain_deferred_acks(&spi);
        }
    }

    /// Redeems the acknowledgements that were waiting for this polynomial's verifier.
    ///
    /// Called from the commitment path immediately after the verifier is installed, so each bucket
    /// is processed exactly once and never speculatively re-scanned. Resolutions are parked in
    /// [`ready_resolutions`](Self::ready_resolutions) for the next `acknowledge_shares` call, since
    /// the commitment path has no route to the upper layer.
    fn drain_deferred_acks(&self, spi: &SsaPolynomialId<S::Pseudonym>) {
        let Some(bucket) = self.pending_acks.get(spi) else {
            return;
        };
        self.pending_acks.invalidate(spi);

        let deferred = std::mem::take(&mut *bucket.lock());
        if deferred.is_empty() {
            return;
        }

        let mut resolved = Vec::new();
        for (peer, challenge, ack) in deferred {
            // The share lives in the peer's own awaiting-acks cache; if the peer entry is gone the
            // share has expired with it and the ack is dead.
            let Some(awaiting) = self.awaiting_acks.get(&peer) else {
                continue;
            };
            match self.process_verified_ack(ack, challenge, &awaiting) {
                Ok(ProcessedAckResult::FullRecovery(ssa)) => resolved.push(ShareResolution::RecoveredSsa(ssa)),
                Ok(ProcessedAckResult::EarlyRecovery(ssa_id)) => {
                    resolved.push(ShareResolution::AlmostRecoveredSsa(ssa_id))
                }
                Ok(ProcessedAckResult::NoProgress) => {}
                Ok(ProcessedAckResult::VerifierNotReady(_)) => {
                    // The verifier was installed and then immediately withdrawn, which only the
                    // retirement path does. Re-bucketing would leak, so drop.
                    tracing::trace!(%spi, "verifier withdrawn while draining deferred acknowledgements");
                }
                Err(PixError::InvalidShare(pseudonym, ssa_index)) => {
                    tracing::error!(%pseudonym, ssa_index, "deferred share could not be verified");
                    resolved.push(ShareResolution::InvalidShare(
                        peer.into(),
                        SsaId::new(pseudonym, ssa_index),
                    ));
                }
                Err(error) => tracing::debug!(%spi, %error, "failed to process deferred acknowledgement"),
            }
        }

        if !resolved.is_empty() {
            tracing::debug!(%spi, num = resolved.len(), "redeemed deferred acknowledgements");
            let mut ready = self.ready_resolutions.lock();
            ready.extend(resolved);
            self.ready_resolutions_len
                .store(ready.len(), std::sync::atomic::Ordering::Release);
        }
    }

    /// Takes any resolutions parked by [`drain_deferred_acks`](Self::drain_deferred_acks).
    ///
    /// One relaxed load in the common case — the buckets are empty whenever the Entry finishes the
    /// constant-term pass before the shares that reference it arrive.
    fn take_ready_resolutions(&self) -> Vec<ShareResolution<S::Pseudonym, S::AddressPrivateKey>> {
        if self.ready_resolutions_len.load(std::sync::atomic::Ordering::Acquire) == 0 {
            return Vec::new();
        }
        let mut ready = self.ready_resolutions.lock();
        self.ready_resolutions_len
            .store(0, std::sync::atomic::Ordering::Release);
        std::mem::take(&mut *ready)
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
        proof: Option<SsaCommitmentProof<S>>,
        commitments: impl Iterator<Item = (PolynomialIndex, PixGroupRepr<S>)>,
    ) -> Result<SsaCommitmentState<S::Pseudonym, S::DepositAddress>, Self::Error> {
        let mut res = SsaCommitmentState::new(ssa_id);

        // The Server commitment must be present first
        let Some(builder) = self.commitment_builder.get(&ssa_id) else {
            return Err(PixError::MissingSsaCommitment);
        };

        let progress = {
            let mut builder = builder.lock();
            res.is_first_encountered = builder.is_empty();
            res.ssa_deposit_address = builder.get_deposit_address().copied();
            builder.add_transposed(index, proof, commitments)?
        };

        // `ssa_deposit_address` was read *before* the insertion, so it being absent here means the
        // address (if any) was discovered by this very call.
        res.deposit_address_first_encountered = res.ssa_deposit_address.is_none();

        let Some(full_ssa_commitment) = progress.full_commitment else {
            res.deposit_address_first_encountered = false; // Not yet encountered
            tracing::trace!(%ssa_id, "ssa commitment not yet complete, waiting for more constant terms");
            return Ok(res);
        };
        res.ssa_deposit_address = Some(S::group_to_deposit_address(full_ssa_commitment).ok_or(PixError::InvalidSsa)?);

        // Publish the part accumulator BEFORE installing any verifier, and never the other way
        // round. `process_verified_ack` reads the verifier first and the builder second, so a share
        // that finds a verifier but no builder fails with `MissingSsaCommitment` — which is a
        // permanent drop, with no deferral path to recover it. Reversed, a share that finds no
        // verifier yet is simply deferred and redeemed by the drain below. Only one of the two
        // orderings has a recovery path.
        //
        // Both publications are also what makes a concurrent `retire_ssa` able to see this cycle:
        // the liveness map now exists before the verifiers it accounts for, so retirement can
        // always enumerate them.
        if let Some(ssa_builder) = progress.ssa_builder {
            let num_polys = ssa_builder.num_polys();
            self.ssa_builders
                .insert(ssa_id, std::sync::Arc::new(parking_lot::Mutex::new(ssa_builder)));
            self.ssa_num_polys.insert(ssa_id, num_polys);
            tracing::debug!(%ssa_id, num_polys, "ssa commitment known — cycle is live");
        }

        let installed: Vec<SsaPolynomialId<S::Pseudonym>> = progress.new_verifiers.iter().map(|v| v.spi()).collect();
        for verifier in progress.new_verifiers {
            self.ssa_verifiers
                .insert(verifier.spi(), std::sync::Arc::new(parking_lot::Mutex::new(verifier)));
        }

        // Tombstone checked *after* publishing, so that retirement racing this call cannot slip
        // between a check and a write. If it did run, undo everything this call published — the
        // cycle's state was already torn down and republishing it would resurrect it.
        if self.retired_ssas.contains_key(&ssa_id) {
            for spi in &installed {
                self.ssa_verifiers.invalidate(spi);
                self.pending_acks.invalidate(spi);
            }
            self.ssa_builders.invalidate(&ssa_id);
            self.ssa_num_polys.invalidate(&ssa_id);
            tracing::trace!(%ssa_id, "ssa commitment progressed but cycle was retired — dropped published state");
            res.deposit_address_first_encountered = false;
            return Ok(res);
        }

        // Each freshly installed verifier unblocks exactly the acknowledgements bucketed under its
        // own polynomial. Doing this here rather than on the acknowledgement path is what keeps
        // `acknowledge_shares` free of retry scanning.
        for spi in &installed {
            self.drain_deferred_acks(spi);
        }

        res.is_verifiable = progress.fully_committed;
        if progress.fully_committed {
            tracing::trace!(%ssa_id, "ssa commitment completed");
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

        // Collect anything redeemed while verifiers were being installed. No retry scanning happens
        // here: a deferred ack is retried exactly once, by the installation of the verifier it was
        // waiting for (see `drain_deferred_acks`).
        res.extend(self.take_ready_resolutions());

        for (ack, ack_challenge) in half_keys_challenges {
            match self.process_verified_ack(ack, ack_challenge, &awaiting_ack_from_peer) {
                Ok(ProcessedAckResult::FullRecovery(ssa)) => {
                    res.insert(ShareResolution::RecoveredSsa(ssa));
                }
                Ok(ProcessedAckResult::EarlyRecovery(ssa_id)) => {
                    res.insert(ShareResolution::AlmostRecoveredSsa(ssa_id));
                }
                Ok(ProcessedAckResult::NoProgress) => {}
                Ok(ProcessedAckResult::VerifierNotReady(spi)) => {
                    // The share stays in `awaiting_acks`; bucket the ack under the polynomial whose
                    // verifier it needs, so installing that verifier redeems it.
                    tracing::trace!(%peer, %spi, "verifier not yet installed, deferring acknowledgement");
                    self.defer_ack(spi, (peer, ack_challenge, ack));
                }
                Err(PixError::ShareIsEmpty) => tracing::trace!(%peer, "received empty share"),
                Err(PixError::InvalidShare(pseudonym, ssa_index)) => {
                    tracing::error!(%pseudonym, ssa_index, "encountered share that could not be verified");
                    res.insert(ShareResolution::InvalidShare(
                        peer.into(),
                        SsaId::new(pseudonym, ssa_index),
                    ));
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
        DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, GroupEncoding, PartialSsaShare, SsaGeneratorConfig, SsaIndex,
        SsaShareGenerator,
        tests::TestSpec,
        traits::{EntryShareGenerator, ExitAcknowledgementShareProcessor},
    };

    /// The proof a generated commitment carries, attached only to the constant-term batch — the
    /// shape the wire uses, since that batch is what determines the commitment being opened.
    fn proof_of(
        commitment: &crate::SsaCommitment<TestSpec>,
        coeff_index: CoefficientIndex,
    ) -> Option<SsaCommitmentProof<TestSpec>> {
        (coeff_index == 0).then_some(commitment.commitment_proof)
    }

    /// Proof matching the all-identity commitment sets some fixtures use as filler: their sum is the
    /// identity, whose discrete logarithm is zero, so the proof is honest rather than a bypass.
    fn identity_proof(ssa_id: &SsaId<SimplePseudonym>) -> SsaCommitmentProof<TestSpec> {
        let zero = <PixScalar<TestSpec> as Field>::ZERO;
        SsaCommitmentProof::prove(ssa_id, &zero, &PixGroup::<TestSpec>::mul_by_generator(&zero))
            .expect("identity proof must be constructible")
    }

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

        // 1. Any non-constant coefficient index is ignored rather than rejected, whether or not it is within the
        //    threshold — PIX commits to nothing but the constant term, and a peer that still sends a full Feldman
        //    matrix must not be treated as hostile.
        for coeff_index in [1, 2, CoefficientIndex::MAX] {
            let ignored =
                reconstructor.insert_coefficient_commitments(ssa_id, coeff_index, None, HashMap::new().into_iter())?;
            assert!(ignored.ssa_deposit_address.is_none());
            assert!(!ignored.is_verifiable);
        }
        assert!(
            reconstructor
                .commitment_builder
                .get(&ssa_id)
                .ok_or_else(|| anyhow::anyhow!("missing builder"))?
                .lock()
                .is_empty(),
            "ignored commitments must not enter the builder's state"
        );

        // 2. Invalid polynomial index (>= polys_per_ssa)
        let mut invalid_poly_map = HashMap::new();
        invalid_poly_map.insert(2 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        let result = reconstructor.insert_coefficient_commitments(ssa_id, 0, None, invalid_poly_map.into_iter());
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

        let res = reconstructor.insert_coefficient_commitments(ssa_id, 0, None, poly_map.into_iter());

        assert!(matches!(res, Err(PixError::MissingSsaCommitment)));

        Ok(())
    }

    #[test]
    fn reconstructor_duplicate_commitments() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);

        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // Fill every constant term, which is the whole commitment
        let mut poly_map = HashMap::new();
        for poly in 0..2 {
            poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        }
        reconstructor.insert_coefficient_commitments(ssa_id, 0, Some(identity_proof(&ssa_id)), poly_map.into_iter())?;

        // Now adding more should fail with DuplicateCommitment
        let result = reconstructor.insert_coefficient_commitments(ssa_id, 0, None, HashMap::new().into_iter());
        assert!(matches!(result, Err(PixError::DuplicateCommitment)));

        // A trailing non-constant coefficient, on the other hand, is simply ignored: a peer that
        // still emits the full Feldman matrix sends the bulk of it *after* the constant-term pass
        // has completed, and that must not read as a duplicate-commitment attack.
        let mut trailing = HashMap::new();
        trailing.insert(0 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        let ignored = reconstructor.insert_coefficient_commitments(ssa_id, 1, None, trailing.into_iter())?;
        assert!(
            ignored.is_verifiable,
            "ignoring a message must not make a completed cycle look incomplete"
        );

        Ok(())
    }

    #[test]
    fn reconstructor_duplicate_per_polynomial_commitment() -> anyhow::Result<()> {
        // Regression test for the per-polynomial duplicate check inside add_transposed.
        // Previously the same polynomial's slot silently overwrote; now it returns
        // DuplicateCommitment.
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);

        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // Insert the constant term of poly 0
        let mut poly_map_1 = HashMap::new();
        poly_map_1.insert(0 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        reconstructor.insert_coefficient_commitments(ssa_id, 0, None, poly_map_1.into_iter())?;

        // Insert the constant term of poly 0 again — must fail
        let mut poly_map_2 = HashMap::new();
        poly_map_2.insert(0 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        let result = reconstructor.insert_coefficient_commitments(ssa_id, 0, None, poly_map_2.into_iter());
        assert!(matches!(result, Err(PixError::DuplicateCommitment)));

        // Poly 1's constant term is a different slot and must still be accepted
        let mut poly_map_3 = HashMap::new();
        poly_map_3.insert(1 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        assert!(
            reconstructor
                .insert_coefficient_commitments(ssa_id, 0, Some(identity_proof(&ssa_id)), poly_map_3.into_iter())
                .is_ok()
        );

        Ok(())
    }

    #[test]
    fn reconstructor_missing_verifier_retains_share() -> anyhow::Result<()> {
        // Regression test for the share-loss race:
        // When the polynomial's verifier is not installed yet, the share must NOT be removed
        // from the awaiting_acks cache — it must remain available for the retry that happens
        // when the verifier arrives.
        //
        // The implementation guarantees this: `process_verified_ack` looks the share up with
        // `.get()` and only `.remove()`s it after the verifier lookup succeeds, so the
        // `VerifierNotReady` deferral leaves the share in place. This test asserts that retention.
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

        // Process the ack — the verifier is not installed, so this defers rather than failing,
        // and reports the polynomial the ack must be bucketed under.
        let peer_cache_ref = reconstructor.awaiting_acks.get(peer.public()).unwrap();
        let result = reconstructor.process_verified_ack(ack_key, challenge, &peer_cache_ref);
        assert!(
            matches!(result, Ok(ProcessedAckResult::VerifierNotReady(reported)) if reported == spi),
            "expected deferral naming polynomial {spi:?}"
        );

        // The share MUST NOT be destroyed by the deferral: the implementation only removes it
        // after the verifier lookup succeeds, so it stays available for the retry.
        let peer_cache_after = reconstructor.awaiting_acks.get(peer.public());
        assert!(peer_cache_after.is_some(), "share must be retained when deferred");
        assert!(
            peer_cache_after.as_ref().unwrap().contains_key(&challenge),
            "share must be retained when deferred"
        );

        Ok(())
    }

    #[test]
    fn reconstructor_defers_ack_when_verifier_is_not_installed() -> anyhow::Result<()> {
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
        assert!(
            matches!(result, Ok(ProcessedAckResult::VerifierNotReady(reported)) if reported == spi),
            "an ack with no installed verifier must be deferred, not failed"
        );

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
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // Step 1: Submit polynomial 0's constant term — must succeed, but the SSA commitment needs
        // every constant term, so no deposit address yet.
        let partial = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(0))?.into_iter(),
        )?;
        assert!(
            partial.ssa_deposit_address.is_none(),
            "the deposit address needs every constant term"
        );
        assert!(!partial.is_verifiable, "not yet complete");

        // Step 2: Submit a malformed constant term for polynomial 1 (bytes with an invalid EC
        // compressed-point prefix of 0xff) — must return InvalidInput.
        let mut malformed = PixGroupRepr::<TestSpec>::default(); // zero-filled
        AsMut::<[u8]>::as_mut(&mut malformed).fill(0xff);
        let malformed_result = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            [(1, malformed)].into_iter(),
        );
        assert!(
            matches!(&malformed_result, Err(crate::errors::PixError::InvalidInput)),
            "malformed commitment must be rejected, got {malformed_result:?}"
        );

        // Step 3: Retry with the correct bytes — must succeed and complete the SSA commitment.
        let retry = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(1))?.into_iter(),
        );
        assert!(
            matches!(&retry, Ok(state) if state.is_verifiable && state.ssa_deposit_address.is_some()),
            "corrected retransmission must complete the SSA, got {retry:?}"
        );

        Ok(())
    }

    /// Regression test for M13: the validation applied when a commitment *arrives* is the only
    /// validation it ever gets, so it must include the prime-order-subgroup test.
    ///
    /// A commitment that passes occupies its polynomial's slot permanently — re-insertion is
    /// rejected as a duplicate. A decodable-but-small-order point admitted here would take the
    /// slot and then make every reconstruction of that polynomial fail, with no way to retransmit
    /// a correction. This matters in practice: the default build uses BabyJubJub, whose cofactor is
    /// 8, so small-order points do exist and do pass a plain on-curve check.
    #[test]
    fn decode_commitment_is_the_single_validation_point() {
        use vsss_rs::elliptic_curve::group::GroupEncoding;

        use crate::SsaPartCommitment;

        // A well-formed commitment (the generator) decodes.
        let generator_repr = PixGroup::<TestSpec>::generator().to_bytes();
        assert!(
            SsaPartCommitment::<TestSpec>::decode_commitment(&generator_repr).is_ok(),
            "the generator is a valid constant-term commitment (scalar coefficient 1)"
        );

        // Undecodable bytes are rejected.
        let mut malformed = PixGroupRepr::<TestSpec>::default();
        AsMut::<[u8]>::as_mut(&mut malformed).fill(0xff);
        assert!(
            matches!(
                SsaPartCommitment::<TestSpec>::decode_commitment(&malformed),
                Err(PixError::InvalidInput)
            ),
            "undecodable bytes must be rejected"
        );
    }

    /// A reconstructed polynomial part must release its collected shares, while **keeping its
    /// cache entry**.
    ///
    /// That buffer dominates reconstructor memory — `threshold` shares held for every one of
    /// `polys` polynomials — and it cannot be read again once the part is reconstructed.
    ///
    /// Retaining the (now-stripped) cache entry is equally deliberate: evicting it would make every
    /// late or surplus share for that polynomial look like a not-yet-installed verifier, so the ack
    /// would be deferred into a bucket that nothing will ever drain. The stripped builder keeps the
    /// cheap already-reconstructed path instead.
    #[test]
    fn reconstructed_polynomial_releases_verification_state_but_keeps_its_entry() -> anyhow::Result<()> {
        // 2 polynomials, threshold 2, no surplus: finishing polynomial 0 does *not* complete the
        // SSA, so the cycle is still live and its per-polynomial state is observable.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let poly_0 = SsaPolynomialId::new(ssa_id, 0);

        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig::default());
        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;
        commitment.process_into_reconstructor(&reconstructor)?;

        // Freshly installed: no shares collected yet.
        let builder = reconstructor
            .ssa_verifiers
            .get(&poly_0)
            .ok_or_else(|| anyhow::anyhow!("verifier for polynomial 0 must be installed"))?;
        assert_eq!(0, builder.lock().verification_state_len());
        drop(builder);

        // Feed exactly enough shares to reconstruct polynomial 0. The generator drains the front
        // polynomial first, so the first `threshold` shares all belong to polynomial 0.
        for _ in 0..2 {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let share = generator
                .next_share(&pseudonym, &msg)?
                .ok_or_else(|| anyhow::anyhow!("generator must yield a share"))?;
            assert_eq!(0, share.id.poly_index(), "shares must arrive polynomial-major");

            let ack = HalfKey::random();
            let challenge = ack.to_challenge()?;
            let enc = share.share.encrypt(&share.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
            )?;
            reconstructor.acknowledge_shares(*peer.public(), vec![VerifiedAcknowledgement::new(ack, &peer).leak()])?;
        }

        // The SSA as a whole is not recovered (polynomial 1 is untouched), so the cycle — and
        // polynomial 0's cache entry — must still be there.
        assert!(
            reconstructor.contains_builder(&ssa_id),
            "the cycle must still be live while polynomial 1 is outstanding"
        );
        let builder = reconstructor
            .ssa_verifiers
            .get(&poly_0)
            .ok_or_else(|| anyhow::anyhow!("a reconstructed polynomial must keep its cache entry"))?;
        assert_eq!(
            0,
            builder.lock().verification_state_len(),
            "a reconstructed polynomial must hold no shares"
        );

        // Polynomial 1 is untouched and must still be installed, awaiting its own shares.
        assert!(
            reconstructor
                .ssa_verifiers
                .contains_key(&SsaPolynomialId::new(ssa_id, 1)),
            "verifier for polynomial 1 must be installed"
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

    /// Deferred acknowledgements are bucketed by the polynomial they are waiting for, *not* by peer.
    ///
    /// Both halves matter. Isolation by polynomial is what lets a bucket be drained by exactly one
    /// event (its own verifier installing) instead of being rescanned speculatively. And a single
    /// bucket deliberately holding several peers' acks is not an accident: one polynomial's shares
    /// are spread across return paths, hence across first-relayers, so the peer has to be carried
    /// per entry rather than being the key.
    #[test]
    fn deferred_acks_are_bucketed_by_polynomial_across_peers() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        let spi_0 = SsaPolynomialId::new(ssa_id, 0);
        let spi_1 = SsaPolynomialId::new(ssa_id, 1);

        let peer_a = OffchainKeypair::random();
        let peer_b = OffchainKeypair::random();
        let ack_a = HalfKey::random();
        let ack_b = HalfKey::random();
        let ack_other = HalfKey::random();

        // Two peers defer for the same polynomial; a third ack belongs to another polynomial.
        reconstructor.defer_ack(spi_0, (*peer_a.public(), ack_a.to_challenge()?, ack_a));
        reconstructor.defer_ack(spi_0, (*peer_b.public(), ack_b.to_challenge()?, ack_b));
        reconstructor.defer_ack(spi_1, (*peer_a.public(), ack_other.to_challenge()?, ack_other));

        let bucket_0 = reconstructor
            .pending_acks
            .get(&spi_0)
            .ok_or_else(|| anyhow::anyhow!("missing bucket for polynomial 0"))?;
        assert_eq!(bucket_0.lock().len(), 2, "one bucket holds both peers' acks");
        assert_eq!(
            reconstructor
                .pending_acks
                .get(&spi_1)
                .ok_or_else(|| anyhow::anyhow!("missing bucket for polynomial 1"))?
                .lock()
                .len(),
            1,
            "a different polynomial keeps its own bucket"
        );

        // Draining one polynomial's bucket must not touch the other's. Neither share exists in
        // `awaiting_acks`, so nothing is redeemed — the point is the bucket bookkeeping.
        reconstructor.drain_deferred_acks(&spi_0);
        assert!(
            !reconstructor.pending_acks.contains_key(&spi_0),
            "a drained bucket is removed"
        );
        assert!(
            reconstructor.pending_acks.contains_key(&spi_1),
            "draining one polynomial must not disturb another"
        );

        Ok(())
    }

    /// Every polynomial becomes reconstructible on the call that completes the constant-term set,
    /// and none before it.
    ///
    /// A polynomial's whole commitment *is* its constant term, so there is no partially committed
    /// row to wait on; but a part builder is still useless until the [`SsaBuilder`] exists to
    /// receive what it reconstructs, and that needs the constant terms of *all* polynomials.
    /// The two therefore coincide, and this pins that they do.
    #[test]
    fn verifiers_are_installed_when_the_constant_term_set_completes() -> anyhow::Result<()> {
        const POLYS: u16 = 6;
        const THRESHOLD: u16 = 4;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;

        // One polynomial's constant term per call. Nothing is published until the last one.
        for poly in 0..POLYS as PolynomialIndex {
            let state = reconstructor.insert_coefficient_commitments(
                ssa_id,
                0,
                proof_of(&commitment, 0),
                coefficient_of(&commitment, 0, Some(poly))?.into_iter(),
            )?;

            let last = poly == POLYS as PolynomialIndex - 1;
            assert_eq!(
                state.ssa_deposit_address.is_some(),
                last,
                "the deposit address is the sum of every constant term (poly {poly})"
            );
            assert_eq!(
                state.is_verifiable, last,
                "the cycle becomes verifiable exactly when the constant-term set closes (poly {poly})"
            );

            reconstructor.ssa_verifiers.run_pending_tasks();
            let expected = if last { POLYS as u64 } else { 0 };
            assert_eq!(
                reconstructor.ssa_verifiers.entry_count(),
                expected,
                "after polynomial {poly} there must be {expected} verifiers installed"
            );
        }

        Ok(())
    }

    /// An Entry that still emits the full Feldman matrix must not break the cycle: the Exit ignores
    /// every non-constant coefficient, whether it arrives before, during or after the constant-term
    /// pass, and recovery proceeds unaffected.
    #[test]
    fn non_constant_coefficients_are_ignored_wherever_they_arrive() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u16 = 2;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            early_recovery_threshold: 1.0,
            ..Default::default()
        });
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;

        // Stand-in for what a Feldman-emitting Entry would send: a well-formed commitment under a
        // non-constant coefficient index, for every polynomial.
        let higher: Vec<(PolynomialIndex, PixGroupRepr<TestSpec>)> = (0..POLYS as PolynomialIndex)
            .map(|poly| {
                (
                    poly,
                    PixGroup::<TestSpec>::mul_by_generator(&PixScalar::<TestSpec>::random(
                        &mut hopr_types::crypto_random::rng(),
                    ))
                    .to_bytes(),
                )
            })
            .collect();

        // Before the constant-term pass.
        reconstructor.insert_coefficient_commitments(ssa_id, 1, None, higher.clone().into_iter())?;

        // Interleaved with it.
        reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(0))?.into_iter(),
        )?;
        reconstructor.insert_coefficient_commitments(ssa_id, 1, None, higher.clone().into_iter())?;
        let state = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(1))?.into_iter(),
        )?;
        assert!(state.is_verifiable, "the constant-term pass alone completes the cycle");

        // And after it — the bulk of what such an Entry sends. Must not read as a duplicate.
        reconstructor.insert_coefficient_commitments(ssa_id, 1, None, higher.into_iter())?;

        // Recovery is unaffected.
        let mut acks = Vec::new();
        while let Some((msg, share)) = {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            generator.next_share(&pseudonym, &msg).map(|v| v.map(|u| (msg, u)))
        }? {
            let ack = HalfKey::random();
            let enc_share = share.share.encrypt(&share.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                ack.to_challenge()?,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc_share)?,
            )?;
            acks.push(VerifiedAcknowledgement::new(ack, &peer).leak());
        }

        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;
        assert!(
            resolutions
                .iter()
                .any(|r| matches!(r, ShareResolution::RecoveredSsa(r) if r.ssa_id == ssa_id)),
            "the SSA must recover from the constant terms alone, got {resolutions:?}"
        );

        Ok(())
    }

    /// A single corrupted share is invisible until the polynomial is interpolated — that is the
    /// price of dropping the per-coefficient commitments — and it must surface exactly then, once,
    /// with no further noise from the shares that follow it.
    #[test]
    fn a_corrupted_share_is_reported_once_at_the_threshold_th_share() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u16 = 4;
        const SURPLUS: usize = 2;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: SURPLUS,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        generator
            .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?
            .process_into_reconstructor(&reconstructor)?;

        // Corrupt the very first share of polynomial 0, then feed its whole budget one at a time.
        let mut invalid_reports = 0;
        for i in 0..THRESHOLD as usize + SURPLUS {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let mut share = generator
                .next_share(&pseudonym, &msg)?
                .ok_or_else(|| anyhow::anyhow!("generator must yield a share"))?;
            assert_eq!(0, share.id.poly_index(), "shares arrive polynomial-major");

            if i == 0 {
                // Flip a low bit of the share value. It stays a valid field element, so nothing
                // rejects it on arrival — only the interpolation notices.
                AsMut::<[u8]>::as_mut(&mut share.share.0)[31] ^= 1;
            }

            let ack = HalfKey::random();
            let enc = share.share.clone().encrypt(&share.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                ack.to_challenge()?,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
            )?;
            let resolutions = reconstructor
                .acknowledge_shares(*peer.public(), vec![VerifiedAcknowledgement::new(ack, &peer).leak()])?;

            let reported = resolutions
                .iter()
                .filter(|r| matches!(r, ShareResolution::InvalidShare(_, id) if *id == ssa_id))
                .count();
            if i + 1 < THRESHOLD as usize {
                assert_eq!(0, reported, "share {i} must pass unremarked — nothing checks it yet");
            }
            invalid_reports += reported;
        }

        assert_eq!(
            1, invalid_reports,
            "the corrupted set must be reported exactly once, at the threshold-th share"
        );

        Ok(())
    }

    /// End-to-end deferral: shares that arrive before their polynomial's row is committed must be
    /// redeemed when the verifier installs, and the resulting resolutions must reach the caller.
    ///
    /// This is the path that used to be a full stash re-scan on every acknowledgement batch.
    #[test]
    fn shares_arriving_before_their_verifier_are_redeemed_on_installation() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u16 = 2;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;

        // Only part of the constant-term pass so far, so the SSA commitment is still unknown and no
        // part builder can be installed — a recovered part would have nowhere to go.
        reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(0))?.into_iter(),
        )?;

        // All shares for the whole cycle arrive now — every one of them ahead of its verifier.
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

        // Nothing can resolve yet, and every ack must have been bucketed rather than dropped.
        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;
        assert!(
            resolutions.is_empty(),
            "no share can resolve before its polynomial is committed"
        );
        let deferred: usize = (0..POLYS as PolynomialIndex)
            .filter_map(|poly| reconstructor.pending_acks.get(&SsaPolynomialId::new(ssa_id, poly)))
            .map(|bucket| bucket.lock().len())
            .sum();
        assert_eq!(
            deferred,
            (POLYS * THRESHOLD) as usize,
            "every early ack must be bucketed under its own polynomial"
        );

        // The last constant term closes the set, which installs every part builder and redeems the
        // deferred acknowledgements.
        let state = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(1))?.into_iter(),
        )?;
        assert!(state.is_verifiable);

        // Buckets are consumed by the drain, not left to be re-scanned.
        for poly in 0..POLYS as PolynomialIndex {
            assert!(
                !reconstructor
                    .pending_acks
                    .contains_key(&SsaPolynomialId::new(ssa_id, poly)),
                "bucket for polynomial {poly} must be consumed by the drain"
            );
        }

        // The redeemed resolutions surface on the next acknowledgement batch. An empty ack batch is
        // enough — the work is already done, only the hand-off remains.
        let resolutions = reconstructor.acknowledge_shares(*peer.public(), Vec::new())?;
        assert!(
            resolutions
                .iter()
                .any(|r| matches!(r, ShareResolution::RecoveredSsa(r) if r.ssa_id == ssa_id)),
            "deferred shares must recover the SSA once their verifiers install, got {resolutions:?}"
        );

        Ok(())
    }

    /// Deferring is decided on a verifier lookup that missed, so a verifier installing concurrently
    /// would leave the ack in a bucket whose one and only drain has already run — a share silently
    /// lost to a microsecond-wide window. `defer_ack` re-probes and drains itself in that case.
    #[test]
    fn deferring_against_an_installed_verifier_drains_immediately() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u16 = 2;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        generator
            .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?
            .process_into_reconstructor(&reconstructor)?;

        // Every verifier is installed and every bucket already drained.
        let spi = SsaPolynomialId::new(ssa_id, 0);
        assert!(reconstructor.ssa_verifiers.contains_key(&spi));

        // Simulate the racing path: an ack deferred *after* its verifier appeared.
        let ack = HalfKey::random();
        reconstructor.defer_ack(spi, (*peer.public(), ack.to_challenge()?, ack));

        assert!(
            !reconstructor.pending_acks.contains_key(&spi),
            "a bucket created after its verifier installed must drain itself, not linger unclaimed"
        );

        Ok(())
    }

    /// The per-polynomial bucket cap must hold, so a peer cannot make the Exit buffer without
    /// bound by emitting more shares for one polynomial than its own share budget permits.
    #[test]
    fn deferred_ack_buckets_are_capped() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let spi = SsaPolynomialId::new(SsaId::new(SimplePseudonym::random(), SsaIndex::MIN), 0);
        let peer = OffchainKeypair::random();

        for _ in 0..MAX_DEFERRED_ACKS_PER_POLYNOMIAL + 32 {
            let ack = HalfKey::random();
            reconstructor.defer_ack(spi, (*peer.public(), ack.to_challenge()?, ack));
        }

        assert_eq!(
            reconstructor
                .pending_acks
                .get(&spi)
                .ok_or_else(|| anyhow::anyhow!("missing bucket"))?
                .lock()
                .len(),
            MAX_DEFERRED_ACKS_PER_POLYNOMIAL,
            "bucket must not grow past the cap"
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

    /// Extracts one coefficient's commitments from a generated SSA commitment, optionally narrowed
    /// to a single polynomial, in the shape `insert_coefficient_commitments` expects.
    fn coefficient_of(
        commitment: &crate::SsaCommitment<TestSpec>,
        coeff_index: CoefficientIndex,
        only_poly: Option<PolynomialIndex>,
    ) -> anyhow::Result<Vec<(PolynomialIndex, PixGroupRepr<TestSpec>)>> {
        Ok(commitment
            .verifiers
            .get(&coeff_index)
            .ok_or_else(|| anyhow::anyhow!("missing coefficient {coeff_index}"))?
            .iter()
            .filter(|(poly_index, _)| only_poly.is_none_or(|wanted| *poly_index == wanted))
            .map(|(poly_index, repr)| (*poly_index, *repr))
            .collect())
    }

    /// Tombstone guard on the *first* publication point: the SSA becoming live.
    ///
    /// Since verifiers are now installed as individual polynomials complete, the part accumulator
    /// has to be published earlier — the moment the constant terms yield the SSA commitment. A
    /// `retire_ssa` racing that publication must still leave nothing behind.
    #[test]
    fn retire_ssa_tombstone_prevents_builder_publication() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // Constant term of polynomial 0 only — the SSA commitment is still unknown.
        let state = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(0))?.into_iter(),
        )?;
        assert!(
            state.ssa_deposit_address.is_none(),
            "deposit address must not be derivable from a partial constant-term set"
        );

        // Simulate `retire_ssa` racing the completion by setting only the tombstone.
        reconstructor.retired_ssas.insert(ssa_id, ());

        // Constant term of polynomial 1 completes the set, so this call would publish the builder.
        let state = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(1))?.into_iter(),
        )?;
        assert!(!state.is_verifiable, "a retired cycle is never verifiable");

        reconstructor.ssa_builders.run_pending_tasks();
        assert!(
            !reconstructor.ssa_builders.contains_key(&ssa_id),
            "tombstone must prevent builder publication"
        );
        assert!(
            !reconstructor.ssa_num_polys.contains_key(&ssa_id),
            "tombstone must prevent liveness publication"
        );

        Ok(())
    }

    /// Tombstone guard on verifier installation.
    ///
    /// Verifiers and the part accumulator are now published by the same call, so retirement racing
    /// it must withdraw both. The guard is checked *after* publishing — so that retirement cannot
    /// slip between a check and a write — which means the withdrawal path is what this pins.
    #[test]
    fn retire_ssa_tombstone_prevents_verifier_installation() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        // Polynomial 0's constant term: nothing is published yet, the set is incomplete.
        reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(0))?.into_iter(),
        )?;
        reconstructor.ssa_verifiers.run_pending_tasks();
        assert_eq!(
            reconstructor.ssa_verifiers.entry_count(),
            0,
            "no verifier may be installed while the SSA commitment is unknown"
        );

        // Retirement lands here.
        reconstructor.retired_ssas.insert(ssa_id, ());

        // Polynomial 1's constant term closes the set, so this call would install both verifiers.
        let state = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, Some(1))?.into_iter(),
        )?;
        assert!(!state.is_verifiable, "a retired cycle is never verifiable");

        reconstructor.ssa_verifiers.run_pending_tasks();
        assert_eq!(
            reconstructor.ssa_verifiers.entry_count(),
            0,
            "tombstone must withdraw every verifier installed after retirement"
        );
        reconstructor.ssa_builders.run_pending_tasks();
        assert!(
            !reconstructor.ssa_builders.contains_key(&ssa_id),
            "tombstone must withdraw the part accumulator too"
        );

        Ok(())
    }

    /// The commitment proof must bind everything it claims to: both of its own components, the SSA
    /// index it was issued for, and the commitment it opens.
    #[test]
    fn commitment_proof_must_bind_its_components_the_ssa_index_and_the_commitment() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let proof = commitment.commitment_proof;

        assert!(
            proof.verify(&ssa_id, &commitment.ssa_commitment),
            "the generator's own proof must verify"
        );

        // Flipping a bit anywhere breaks it, whether it lands in the nonce commitment or in the
        // response. Some flips make the component unparseable, which is equally a rejection.
        let bytes = proof.to_bytes();
        assert_eq!(SsaCommitmentProof::<TestSpec>::SIZE, bytes.len());
        for position in [0, bytes.len() / 2, bytes.len() - 1] {
            let mut tampered = bytes.clone();
            tampered[position] ^= 1;
            if let Ok(tampered) = SsaCommitmentProof::<TestSpec>::try_from_bytes(&tampered) {
                assert!(
                    !tampered.verify(&ssa_id, &commitment.ssa_commitment),
                    "a proof with byte {position} flipped must not verify"
                );
            }
        }

        // Bound to the SSA index, so it cannot be replayed onto another cycle even if the commitment
        // were somehow reused.
        let other_index = SsaId::new(pseudonym, SsaIndex::new(SsaIndex::MIN.get() + 1).expect("non-zero"));
        assert!(
            !proof.verify(&other_index, &commitment.ssa_commitment),
            "a proof must not verify against a different SSA index"
        );

        // And bound to the commitment: this is the property the whole thing exists for.
        let unrelated = PixGroup::<TestSpec>::mul_by_generator(&PixScalar::<TestSpec>::random(
            &mut hopr_types::crypto_random::rng(),
        ));
        assert!(
            !proof.verify(&ssa_id, &unrelated),
            "a proof must not verify against a commitment it does not open"
        );

        // A truncated or over-long encoding is refused outright.
        assert!(SsaCommitmentProof::<TestSpec>::try_from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(SsaCommitmentProof::<TestSpec>::try_from_bytes(&[bytes.as_slice(), &[0u8]].concat()).is_err());

        Ok(())
    }

    /// A constant-term set that carries no proof at all is refused, exactly as one carrying an
    /// invalid proof is: the Exit cannot tell the difference, and both mean the cycle is unusable.
    #[test]
    fn constant_terms_arriving_without_a_proof_are_refused() -> anyhow::Result<()> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, 2, 2)?;

        let refused = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            None,
            coefficient_of(&commitment, 0, None)?.into_iter(),
        );
        assert!(
            matches!(refused, Err(PixError::UnprovenSsaCommitment)),
            "constant terms without a proof must be refused, got {refused:?}"
        );
        assert!(
            !reconstructor.ssa_builders.contains_key(&ssa_id),
            "no part accumulator may be published"
        );

        Ok(())
    }

    /// A client commitment crafted so the Entry alone knows the *combined* deposit key must be
    /// refused, and must not produce a deposit address.
    ///
    /// The deposit key is `s + e`, where `s` is the sum of the Entry's polynomial constant terms and
    /// `e` is the Exit's commitment secret. Neither party is supposed to know the sum. But
    /// `SsaRequest` hands `e·G` to the Entry *before* it chooses its own constant terms
    /// (`protocols/start/src/lib.rs:305`, consumed at `transport/session/src/manager.rs:2879-2883`),
    /// so an Entry that is made to prove nothing can pick `w`, publish constant terms summing to
    /// `w·G − e·G`, and end up with a combined commitment of `w·G` — a key it can sweep alone.
    ///
    /// It cannot then produce shares for the polynomial whose constant term it does not know, so the
    /// Exit never recovers the SSA and is never paid. But the Entry controls emission order
    /// (`generator.rs` builds `poly_queue`, `next_share` drains `front_mut()`) and puts that
    /// polynomial last, so it is served nearly the whole cycle before the Exit notices — by which
    /// time it has already swept the deposit. Note this is distinct from the by-design burn
    /// semantics: there *neither* party can recover, whereas here the party that owes can.
    ///
    /// Before [`SsaCommitmentProof`] existed this construction was accepted, and the Exit published
    /// `addr(w·G)` as the address to watch — verified by asserting exactly that. The proof cannot be
    /// forged here because producing it would require `dlog(w·G − e·G)`, and knowing that together
    /// with `w` yields `e`. So the assertion is now inverted.
    #[test]
    fn exit_refuses_a_client_commitment_whose_deposit_key_the_entry_knows() -> anyhow::Result<()> {
        const POLYS: usize = 3;
        const THRESHOLD: usize = 2;

        let mut rng = hopr_types::crypto_random::rng();
        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig::default());

        // The Exit reveals its half. This is exactly what `SsaRequest` carries to the Entry.
        let exit_public = reconstructor.new_exit_commitment(ssa_id, POLYS, THRESHOLD)?;

        // The attacker picks the deposit key it wants to end up holding.
        let w = PixScalar::<TestSpec>::random(&mut rng);
        let target = PixGroup::<TestSpec>::mul_by_generator(&w);

        // Honest constant terms for every polynomial but the last.
        let honest: Vec<PixGroup<TestSpec>> = (0..POLYS - 1)
            .map(|_| PixGroup::<TestSpec>::mul_by_generator(&PixScalar::<TestSpec>::random(&mut rng)))
            .collect();
        let honest_sum: PixGroup<TestSpec> = honest.iter().copied().sum();

        // The last one is obtained by group subtraction. The attacker never learns its discrete log
        // — that would require the Exit's secret — and does not need to.
        let rogue = target - exit_public - honest_sum;

        let mut constant_terms: HashMap<PolynomialIndex, PixGroupRepr<TestSpec>> = honest
            .iter()
            .enumerate()
            .map(|(poly_index, c0)| (poly_index as PolynomialIndex, c0.to_bytes()))
            .collect();
        constant_terms.insert((POLYS - 1) as PolynomialIndex, rogue.to_bytes());

        // The best the attacker can offer is a proof over the client commitment it actually
        // published, using the only scalar it knows — which is not that commitment's discrete log.
        let bogus_proof = SsaCommitmentProof::prove(&ssa_id, &w, &(target - exit_public))?;

        let rejected =
            reconstructor.insert_coefficient_commitments(ssa_id, 0, Some(bogus_proof), constant_terms.into_iter());
        assert!(
            matches!(rejected, Err(PixError::UnprovenSsaCommitment)),
            "a commitment whose discrete logarithm the sender does not know must be refused, got {rejected:?}"
        );

        // Nothing about the cycle may have been published: with no deposit address the strategy is
        // never asked to fund an SSA the Entry could reclaim.
        assert!(
            reconstructor
                .commitment_builder
                .get(&ssa_id)
                .is_some_and(|b| b.lock().get_deposit_address().is_none()),
            "no deposit address may be derived from an unproven commitment"
        );
        // The part accumulator is what makes a cycle live and able to accept recovered shares.
        // `commitment_builder` legitimately still exists — `new_exit_commitment` created it.
        assert!(
            !reconstructor.ssa_builders.contains_key(&ssa_id),
            "the part accumulator must not be published for an unproven commitment"
        );

        Ok(())
    }
}
