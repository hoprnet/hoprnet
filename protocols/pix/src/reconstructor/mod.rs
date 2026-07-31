mod utils;

use ahash::HashSetExt;
use hopr_types::{
    crypto::{
        crypto_traits::elliptic_curve::Field,
        prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    },
    internal::prelude::Acknowledgement,
};
use utils::{SsaCommitmentBuilder, SsaCycle};
use validator::Validate;

use crate::{
    CoefficientIndex, ExitAcknowledgementShareProcessor, Group, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, PixGroup,
    PixGroupRepr, PixScalar, PixSpec, PolynomialIndex, RecoveredSsa, ShareResolution, SsaCommitmentProof,
    SsaCommitmentState, SsaPolynomialId, TaggedEncryptedPartialSsaShare, errors::PixError, types::SsaId,
};

/// Configuration for the SSA reconstructor.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, validator::Validate)]
pub struct SsaReconstructorConfig {
    /// Time until the complete commitment to an SSA must be received.
    ///
    /// Default is 2 minutes.
    #[default(std::time::Duration::from_secs(120))]
    pub incomplete_commitment_lifetime: std::time::Duration,
    /// Maximum time an SSA cycle can go without progress before it is discarded.
    ///
    /// Measured from the last acknowledged share *anywhere* in the cycle, not per polynomial — see
    /// [`SsaCycle`] for why that distinction is load-bearing. A cycle that is still being served
    /// therefore never expires, whatever the line rate.
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
    /// Default is false.
    ///
    /// Batching only covers the acknowledgement *signature* check. While each share also cost a
    /// `threshold`-term multi-scalar multiplication, that MSM dominated and the choice was
    /// immaterial — measured 2.46 MiB/s batched against 2.50 MiB/s unbatched. Committing to the
    /// constant term alone removed the MSM, and the batching overhead is no longer hidden by it:
    /// the same benchmark then measures 50.2 MiB/s batched against 92.8 MiB/s unbatched, so
    /// batching costs 46 % of the sustained rate.
    ///
    /// Left configurable rather than removed: the figures above come from the sequential
    /// `sustained_quota_rate` group, and the Exit runs acknowledgements through a concurrent
    /// pipeline, where amortising the batch setup across more callers may yet pay.
    #[default(false)]
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

/// Deferred acknowledgements for one cycle, drained in one shot when its part builders install.
///
/// Plain `Vec`s under one mutex rather than nested caches: a bucket is only ever appended to and
/// then drained whole, so per-entry cache bookkeeping (and its ~200 B overhead per entry) buys
/// nothing. The mutex serialises deferrals *within one cycle* only, and deferral is O(1) work off
/// the steady-state path.
type DeferredAckBucket =
    std::sync::Arc<parking_lot::Mutex<std::collections::HashMap<PolynomialIndex, Vec<DeferredAck>>>>;

/// Cap on deferred acknowledgements held for a single polynomial.
///
/// A conforming Entry emits `threshold + surplus` shares per polynomial — 96 at the default
/// dimensions — across all return paths combined, so this cannot be reached without the peer
/// exceeding its own share budget. Anything above the cap is dropped rather than buffered.
const MAX_DEFERRED_ACKS_PER_POLYNOMIAL: usize = 128;

/// Cap on deferred acknowledgements held across all polynomials of one cycle.
///
/// The per-polynomial cap alone leaves the cycle total at `num_polys × 128` — a million entries at
/// production dimensions, which is no bound at all. This one is derived rather than chosen:
/// [`drain_deferred_acks`](SsaReconstructor::drain_deferred_acks) discards any acknowledgement
/// whose share has already left `awaiting_acks`, and that cache expires entries after
/// `max_ack_await_time` (30 s by default). An older deferral is therefore provably dead, so the
/// ceiling only has to cover the shares one cycle can receive inside that window: ~181 shares/s at
/// the deployed 1.5 Mbps per-Session cap, so ~5 400. 8192 leaves ~1.5× headroom and costs at most
/// ~786 KB per cycle.
const MAX_DEFERRED_ACKS_PER_CYCLE: usize = 8192;

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
    /// Post-commitment state of every live cycle: the part accumulator and all part builders,
    /// published and reclaimed as one unit. See [`SsaCycle`].
    ssa_cycles: moka::sync::Cache<SsaId<S::Pseudonym>, std::sync::Arc<SsaCycle<S>>>,
    awaiting_acks: moka::sync::Cache<OffchainPublicKey, EncryptedShareCache<S>>,
    /// Acknowledgements that arrived before their cycle's part builders were installed, bucketed by
    /// cycle and then by polynomial.
    ///
    /// ## Why bucketed at all
    ///
    /// The bucket key is exactly the thing whose arrival unblocks the entries inside it, so a
    /// bucket is drained once, by the installation of its own cycle, and never scanned
    /// speculatively. That is what keeps [`acknowledge_shares`] free of retry work: it only ever
    /// *appends* to a bucket.
    ///
    /// The original per-peer stash had to be re-scanned in full on every `acknowledge_shares` call,
    /// because a per-peer key says nothing about which entries have become viable. That is
    /// quadratic in the number of acks received while a cycle's commitments are in flight, and the
    /// per-peer key aggregates across every Session sharing a first-relayer.
    ///
    /// ## Why keyed by cycle, sub-bucketed by polynomial
    ///
    /// Part builders are installed for a whole cycle at once, so a per-polynomial *key* no longer
    /// buys selective draining — the drain would just walk every polynomial of the cycle. Keying by
    /// cycle makes it one lookup. The per-polynomial sub-bucket is kept because its cap is what
    /// bounds a misbehaving peer (see [`MAX_DEFERRED_ACKS_PER_POLYNOMIAL`]).
    ///
    /// The capacity unit is cycles, not polynomials. Keyed per polynomial it was `2 *
    /// MAX_POLYS_PER_SSA` entries — which one cycle can exhaust on its own, so past roughly four
    /// concurrent cycles node-wide, moka began LRU-evicting buckets and silently dropping real
    /// shares. A size eviction here is share loss, so the headroom is deliberate and the
    /// `max_ack_await_time` TTL is the operative bound.
    pending_acks: moka::sync::Cache<SsaId<S::Pseudonym>, DeferredAckBucket>,
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
    /// Tombstone set: SsaIds that have been retired. The commitment completion path checks this
    /// after publishing the cycle, preventing resurrection when `retire_ssa` runs concurrently.
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
            // Indispensable per-cycle state: never size-evicted. Built without a `max_capacity`,
            // so only `time_to_idle` reclaims it. Active removal happens via `remove_cycle` on
            // full recovery and `retire_ssa` on session teardown; the TTL is the backstop.
            // A hard capacity would silently strand a live cycle.
            //
            // The idle timer is refreshed by an acknowledgement for *any* polynomial of the cycle,
            // because the whole cycle is one entry. That is what makes reclamation correct at any
            // line rate — see `SsaCycle`.
            ssa_cycles: moka::sync::Cache::builder()
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            awaiting_acks: moka::sync::CacheBuilder::new(cfg.max_tracked_peers as u64)
                .time_to_idle(cfg.max_ack_await_time)
                .build(),
            // One bucket per cycle, expiring on the same clock as the shares it belongs to: an ack
            // whose share has left `awaiting_acks` can never be used again, so there is nothing to
            // keep. `time_to_live`, not idle — appending to a bucket must not extend the life of
            // entries already in it.
            //
            // The capacity is in cycles. `MAX_POLYS_PER_SSA` is reused only as a generous count of
            // concurrently deferring cycles; a size eviction here is share loss, so it is
            // deliberately far above the pipelining factor and the TTL is the operative bound.
            pending_acks: moka::sync::CacheBuilder::new(MAX_POLYS_PER_SSA as u64)
                .time_to_live(cfg.max_ack_await_time)
                .build(),
            ready_resolutions: parking_lot::Mutex::new(Vec::new()),
            ready_resolutions_len: std::sync::atomic::AtomicUsize::new(0),
            // Tombstone set: only needs to cover the window between `retire_ssa` running and a
            // concurrent commitment completion publishing its cycle.
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
        self.ssa_cycles.contains_key(ssa_id) || self.commitment_builder.contains_key(ssa_id)
    }

    /// Removes all reconstructor state for a single SSA cycle.
    ///
    /// Idempotent: invalidating an absent key is a no-op.
    fn remove_cycle(&self, ssa_id: SsaId<S::Pseudonym>) {
        self.ssa_cycles.invalidate(&ssa_id);
        // Deferred acks for a retired cycle can never be redeemed — their part builders will not
        // come back and their shares are about to expire.
        self.pending_acks.invalidate(&ssa_id);
        self.commitment_builder.invalidate(&ssa_id);
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

        // One lookup for the whole cycle: the part builders and the accumulator are published and
        // reclaimed together, so there is no state in which one is reachable and the other is not.
        // The lookup also refreshes the cycle's idle timer, which is what keeps a cycle that is
        // still being served from being reclaimed underneath itself.
        let Some(cycle) = self.ssa_cycles.get(spi.as_ref()) else {
            // Not an error: the constant-term set is still incomplete, so no part builder exists
            // yet. Leave the share in `awaiting_acks` and hand the caller the key it needs to
            // bucket the ack.
            return Ok(ProcessedAckResult::VerifierNotReady(spi));
        };

        // The polynomial index comes from the peer's own share, so it is untrusted. Once the cycle
        // is known its dimensions are too, which makes an out-of-range index definitively invalid
        // rather than merely early — there is no later state in which it becomes meaningful.
        let Some(part) = cycle.part(spi.poly_index()) else {
            tracing::error!(%spi, num_polys = cycle.num_polys(), "share names a polynomial outside the cycle");
            return Err(PixError::InvalidInput);
        };

        // Cycle confirmed — safe to consume the share.
        awaiting_ack_from_peer.remove(&ack_challenge);

        // The share cannot be empty at this point because we prevent empty share insertions
        let partial_share = share.partial_share.decrypt(spi.pseudonym(), &ack)?;

        // The part lock is released before the accumulator is taken below. That order is the one
        // callers must keep, and neither lock is ever held across the other.
        let ssa_part = match part.lock().add_share(share.nonce, partial_share) {
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

        let mut builder_guard = cycle.builder().lock();
        let ssa = builder_guard.add_recovered_ssa_part(spi.poly_index(), ssa_part)?;
        match ssa {
            Some(scalar) => {
                let ssa_id = *spi.as_ref();
                // Release the accumulator lock before retiring, so `remove_cycle` — which drops
                // the last `Arc` to this very cycle — does not run while it is held.
                drop(builder_guard);
                let Some(ssa) = S::scalar_to_private_key(scalar) else {
                    tracing::error!(%spi, "ssa reconstruction failed");
                    self.remove_cycle(ssa_id);
                    return Err(PixError::InvalidSsa);
                };
                // Full recovery: this cycle's state is no longer needed.
                self.remove_cycle(ssa_id);
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

    /// Buckets an acknowledgement whose cycle's part builders have not been installed yet.
    ///
    /// O(1) — this is the entire cost the acknowledgement path pays for a deferral.
    fn defer_ack(&self, spi: SsaPolynomialId<S::Pseudonym>, deferred: DeferredAck) {
        let ssa_id = *spi.as_ref();
        let bucket = self.pending_acks.get_with(ssa_id, || {
            std::sync::Arc::new(parking_lot::Mutex::new(Default::default()))
        });
        {
            let mut bucket = bucket.lock();
            if bucket.values().map(Vec::len).sum::<usize>() >= MAX_DEFERRED_ACKS_PER_CYCLE {
                // The cycle as a whole is holding more than the shares it could plausibly have
                // received inside `max_ack_await_time`, so the excess cannot be redeemable.
                tracing::warn!(
                    %ssa_id,
                    cap = MAX_DEFERRED_ACKS_PER_CYCLE,
                    "dropping deferred acknowledgement: cycle bucket is full"
                );
                return;
            }
            let per_poly = bucket.entry(spi.poly_index()).or_default();
            if per_poly.len() >= MAX_DEFERRED_ACKS_PER_POLYNOMIAL {
                // Only reachable if the peer emits more shares for one polynomial than its own
                // `threshold + surplus` budget allows, so the excess is almost certainly duplicate.
                tracing::warn!(
                    %spi,
                    cap = MAX_DEFERRED_ACKS_PER_POLYNOMIAL,
                    "dropping deferred acknowledgement: polynomial bucket is full"
                );
                return;
            }
            per_poly.push(deferred);
        }

        // Close the race against a concurrent installation. The decision to defer was made on a
        // cycle lookup that missed; if the cycle has appeared since, the drain that would have
        // redeemed this ack has already run and nothing else will come for it.
        if self.ssa_cycles.contains_key(&ssa_id) {
            self.drain_deferred_acks(&ssa_id);
        }
    }

    /// Redeems the acknowledgements that were waiting for this cycle's part builders.
    ///
    /// Called from the commitment path immediately after the cycle is installed, so each bucket is
    /// processed exactly once and never speculatively re-scanned. Resolutions are parked in
    /// [`ready_resolutions`](Self::ready_resolutions) for the next `acknowledge_shares` call, since
    /// the commitment path has no route to the upper layer.
    fn drain_deferred_acks(&self, ssa_id: &SsaId<S::Pseudonym>) {
        let Some(bucket) = self.pending_acks.get(ssa_id) else {
            return;
        };
        self.pending_acks.invalidate(ssa_id);

        let deferred = std::mem::take(&mut *bucket.lock());
        if deferred.is_empty() {
            return;
        }

        let mut resolved = Vec::new();
        for (peer, challenge, ack) in deferred.into_values().flatten() {
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
                    // The cycle was installed and then immediately withdrawn, which only the
                    // retirement path does. Re-bucketing would leak, so drop.
                    tracing::trace!(%ssa_id, "cycle withdrawn while draining deferred acknowledgements");
                }
                Err(PixError::InvalidShare(pseudonym, ssa_index)) => {
                    tracing::error!(%pseudonym, ssa_index, "deferred share could not be verified");
                    resolved.push(ShareResolution::InvalidShare(
                        peer.into(),
                        SsaId::new(pseudonym, ssa_index),
                    ));
                }
                Err(error) => tracing::debug!(%ssa_id, %error, "failed to process deferred acknowledgement"),
            }
        }

        if !resolved.is_empty() {
            tracing::debug!(%ssa_id, num = resolved.len(), "redeemed deferred acknowledgements");
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

    /// The published cycle, if it is still live.
    #[cfg(test)]
    fn cycle(&self, ssa_id: &SsaId<S::Pseudonym>) -> Option<std::sync::Arc<SsaCycle<S>>> {
        self.ssa_cycles.get(ssa_id)
    }

    /// Number of live cycles across all Sessions.
    #[cfg(test)]
    fn live_cycles(&self) -> u64 {
        self.ssa_cycles.run_pending_tasks();
        self.ssa_cycles.entry_count()
    }

    /// Number of part builders installed for a cycle, or `0` if the cycle is not live.
    ///
    /// The per-polynomial cache entry count used to express this. It has to be asked of the cycle
    /// now, because the cache holds one entry per cycle rather than one per polynomial — so
    /// `entry_count()` alone can no longer tell "every part installed" from "one part installed".
    #[cfg(test)]
    fn installed_parts(&self, ssa_id: &SsaId<S::Pseudonym>) -> usize {
        self.ssa_cycles.get(ssa_id).map(|c| c.num_polys()).unwrap_or(0)
    }

    /// Total deferred acknowledgements bucketed for a cycle.
    #[cfg(test)]
    fn deferred_ack_count(&self, ssa_id: &SsaId<S::Pseudonym>) -> usize {
        self.pending_acks
            .get(ssa_id)
            .map(|b| b.lock().values().map(Vec::len).sum())
            .unwrap_or(0)
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
        // Mark tombstone BEFORE removing state so the commitment completion path can detect
        // retirement and undo its publication.
        self.retired_ssas.insert(ssa_id, ());

        // Every key is the SsaId itself, so there is nothing to enumerate and no way for part of a
        // cycle to survive the removal of the rest.
        self.remove_cycle(ssa_id);
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

        // The accumulator and every part builder go in as one entry. There is deliberately no
        // ordering to get right here: a share can never observe a cycle in which one is reachable
        // and the other is not, which used to be a permanent-drop hazard.
        let installed = if let Some(ssa_builder) = progress.ssa_builder {
            let num_polys = ssa_builder.num_polys();
            let cycle = SsaCycle::new(ssa_builder, progress.new_verifiers)?;
            self.ssa_cycles.insert(ssa_id, std::sync::Arc::new(cycle));
            tracing::debug!(%ssa_id, num_polys, "ssa commitment known — cycle is live");
            true
        } else {
            false
        };

        // Tombstone checked *after* publishing, so that retirement racing this call cannot slip
        // between a check and a write. If it did run, undo what this call published — the cycle's
        // state was already torn down and republishing it would resurrect it.
        if installed && self.retired_ssas.contains_key(&ssa_id) {
            self.remove_cycle(ssa_id);
            tracing::trace!(%ssa_id, "ssa commitment progressed but cycle was retired — dropped published state");
            res.deposit_address_first_encountered = false;
            return Ok(res);
        }

        // Installing the cycle unblocks every acknowledgement bucketed under it. Doing this here
        // rather than on the acknowledgement path is what keeps `acknowledge_shares` free of retry
        // scanning.
        if installed {
            self.drain_deferred_acks(&ssa_id);
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

    use super::{utils::SsaBuilder, *};
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
        let cycle = reconstructor
            .cycle(&ssa_id)
            .ok_or_else(|| anyhow::anyhow!("cycle must be live"))?;
        assert_eq!(
            0,
            cycle
                .part(poly_0.poly_index())
                .ok_or_else(|| anyhow::anyhow!("part builder for polynomial 0 must be installed"))?
                .lock()
                .verification_state_len()
        );
        drop(cycle);

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
        let cycle = reconstructor
            .cycle(&ssa_id)
            .ok_or_else(|| anyhow::anyhow!("a reconstructed polynomial must keep its slot"))?;
        assert_eq!(
            0,
            cycle
                .part(poly_0.poly_index())
                .ok_or_else(|| anyhow::anyhow!("polynomial 0 must keep its slot"))?
                .lock()
                .verification_state_len(),
            "a reconstructed polynomial must hold no shares"
        );

        // Polynomial 1 is untouched and must still be installed, awaiting its own shares.
        assert!(
            cycle.part(1).is_some(),
            "part builder for polynomial 1 must be installed"
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

        // Precondition: the completed cycle holds every part builder and its accumulator.
        assert_eq!(
            4,
            reconstructor.installed_parts(&ssa_id),
            "4 part builders present after completion"
        );
        assert_eq!(1, reconstructor.live_cycles(), "the cycle is live after completion");

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
        reconstructor.commitment_builder.run_pending_tasks();
        assert_eq!(
            0,
            reconstructor.live_cycles(),
            "the cycle must be retired on full recovery"
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

        assert_eq!(
            3,
            reconstructor.installed_parts(&ssa_id),
            "3 part builders present after completion"
        );

        // Explicit retirement (as invoked on session teardown) drops everything.
        reconstructor.retire_ssa(ssa_id);
        reconstructor.commitment_builder.run_pending_tasks();
        assert_eq!(
            0,
            reconstructor.live_cycles(),
            "the cycle must be removed by retire_ssa"
        );
        assert!(!reconstructor.commitment_builder.contains_key(&ssa_id));

        // Idempotent: retiring the same (now-empty) cycle again is a harmless no-op.
        reconstructor.retire_ssa(ssa_id);

        // Retiring a cycle that was never created must not panic and must leave the caches
        // untouched.
        let never_seen = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        reconstructor.retire_ssa(never_seen);
        assert_eq!(0, reconstructor.live_cycles());

        Ok(())
    }

    /// Deferred acknowledgements are bucketed by the cycle they are waiting for, sub-bucketed by
    /// polynomial, and never by peer.
    ///
    /// All three halves matter. Keying by cycle is what lets a bucket be drained by exactly one
    /// event (its own cycle installing) instead of being rescanned speculatively. The
    /// per-polynomial sub-bucket is what the cap is expressed against. And a sub-bucket
    /// deliberately holding several peers' acks is not an accident: one polynomial's shares are
    /// spread across return paths, hence across first-relayers, so the peer has to be carried per
    /// entry rather than being the key.
    #[test]
    fn deferred_acks_are_bucketed_by_cycle_and_polynomial_across_peers() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        let other_ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        let spi_0 = SsaPolynomialId::new(ssa_id, 0);
        let spi_1 = SsaPolynomialId::new(ssa_id, 1);
        let spi_other = SsaPolynomialId::new(other_ssa_id, 0);

        let peer_a = OffchainKeypair::random();
        let peer_b = OffchainKeypair::random();
        let ack_a = HalfKey::random();
        let ack_b = HalfKey::random();
        let ack_other_poly = HalfKey::random();
        let ack_other_cycle = HalfKey::random();

        // Two peers defer for the same polynomial; a third ack belongs to another polynomial of the
        // same cycle; a fourth to a different cycle entirely.
        reconstructor.defer_ack(spi_0, (*peer_a.public(), ack_a.to_challenge()?, ack_a));
        reconstructor.defer_ack(spi_0, (*peer_b.public(), ack_b.to_challenge()?, ack_b));
        reconstructor.defer_ack(
            spi_1,
            (*peer_a.public(), ack_other_poly.to_challenge()?, ack_other_poly),
        );
        reconstructor.defer_ack(
            spi_other,
            (*peer_a.public(), ack_other_cycle.to_challenge()?, ack_other_cycle),
        );

        let bucket = reconstructor
            .pending_acks
            .get(&ssa_id)
            .ok_or_else(|| anyhow::anyhow!("missing bucket for the cycle"))?;
        {
            let bucket = bucket.lock();
            assert_eq!(
                2,
                bucket.get(&0).map(Vec::len).unwrap_or(0),
                "one sub-bucket holds both peers' acks"
            );
            assert_eq!(
                1,
                bucket.get(&1).map(Vec::len).unwrap_or(0),
                "a different polynomial keeps its own sub-bucket"
            );
        }
        assert_eq!(3, reconstructor.deferred_ack_count(&ssa_id));
        assert_eq!(1, reconstructor.deferred_ack_count(&other_ssa_id));

        // Draining one cycle's bucket must not touch another's. No share exists in `awaiting_acks`,
        // so nothing is redeemed — the point is the bucket bookkeeping.
        reconstructor.drain_deferred_acks(&ssa_id);
        assert!(
            !reconstructor.pending_acks.contains_key(&ssa_id),
            "a drained bucket is removed, with all of its polynomials"
        );
        assert!(
            reconstructor.pending_acks.contains_key(&other_ssa_id),
            "draining one cycle must not disturb another"
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

            let expected = if last { POLYS as usize } else { 0 };
            assert_eq!(
                expected,
                reconstructor.installed_parts(&ssa_id),
                "after polynomial {poly} there must be {expected} part builders installed"
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
        assert_eq!(
            (POLYS * THRESHOLD) as usize,
            reconstructor.deferred_ack_count(&ssa_id),
            "every early ack must be bucketed under its own cycle"
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

        // The bucket is consumed by the drain, not left to be re-scanned.
        assert!(
            !reconstructor.pending_acks.contains_key(&ssa_id),
            "the cycle's bucket must be consumed by the drain"
        );

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

    /// Deferring is decided on a cycle lookup that missed, so a cycle installing concurrently would
    /// leave the ack in a bucket whose one and only drain has already run — a share silently lost
    /// to a microsecond-wide window. `defer_ack` re-probes and drains itself in that case.
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

        // Every part builder is installed and every bucket already drained.
        let spi = SsaPolynomialId::new(ssa_id, 0);
        assert!(reconstructor.cycle(&ssa_id).is_some_and(|c| c.part(0).is_some()));

        // Simulate the racing path: an ack deferred *after* its cycle appeared.
        let ack = HalfKey::random();
        reconstructor.defer_ack(spi, (*peer.public(), ack.to_challenge()?, ack));

        assert!(
            !reconstructor.pending_acks.contains_key(&ssa_id),
            "a bucket created after its cycle installed must drain itself, not linger unclaimed"
        );

        Ok(())
    }

    /// The per-polynomial sub-cap must hold, so a peer cannot make the Exit buffer without bound by
    /// emitting more shares for one polynomial than its own share budget permits.
    #[test]
    fn deferred_ack_buckets_are_capped() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        let spi = SsaPolynomialId::new(ssa_id, 0);
        let peer = OffchainKeypair::random();

        for _ in 0..MAX_DEFERRED_ACKS_PER_POLYNOMIAL + 32 {
            let ack = HalfKey::random();
            reconstructor.defer_ack(spi, (*peer.public(), ack.to_challenge()?, ack));
        }

        assert_eq!(
            MAX_DEFERRED_ACKS_PER_POLYNOMIAL,
            reconstructor.deferred_ack_count(&ssa_id),
            "a polynomial's sub-bucket must not grow past the cap"
        );

        Ok(())
    }

    /// The per-cycle ceiling must hold even when the peer spreads its acknowledgements across many
    /// polynomials, each of which stays under the per-polynomial sub-cap.
    ///
    /// Without it the cycle total is `num_polys × 128` — a million entries at production
    /// dimensions, which is no bound at all.
    #[test]
    fn deferred_ack_buckets_are_capped_per_cycle() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);
        let peer = OffchainKeypair::random();

        // Spread over enough polynomials that no sub-bucket ever reaches its own cap, so the
        // per-cycle ceiling is provably the thing doing the work.
        let polys = (MAX_DEFERRED_ACKS_PER_CYCLE / (MAX_DEFERRED_ACKS_PER_POLYNOMIAL / 2)) + 8;
        'outer: for poly in 0..polys as PolynomialIndex {
            let spi = SsaPolynomialId::new(ssa_id, poly);
            for _ in 0..MAX_DEFERRED_ACKS_PER_POLYNOMIAL / 2 {
                let ack = HalfKey::random();
                reconstructor.defer_ack(spi, (*peer.public(), ack.to_challenge()?, ack));
                if reconstructor.deferred_ack_count(&ssa_id) > MAX_DEFERRED_ACKS_PER_CYCLE {
                    break 'outer;
                }
            }
        }

        assert_eq!(
            MAX_DEFERRED_ACKS_PER_CYCLE,
            reconstructor.deferred_ack_count(&ssa_id),
            "the cycle total must not grow past the ceiling"
        );

        Ok(())
    }

    /// **H8 regression.** Reclamation is scoped to the cycle, so a share for *any* polynomial keeps
    /// the whole cycle alive.
    ///
    /// This used to fail. The part builders were keyed per polynomial with an idle timer, so the
    /// clock measured "time since a share for *this* polynomial arrived". Commitments are a
    /// fraction of a percent of a cycle's bytes, so every builder was installed in the opening
    /// moments and then waited, while shares arrive polynomial-major and spread across the whole
    /// cycle. Any polynomial late in the emission order had its builder evicted before its first
    /// share landed — unrecoverably, since the commitment cannot be retransmitted and a deferred
    /// ack's only drain is an installation that had already happened. The SSA never completed and
    /// the deposit burned.
    ///
    /// The condition was `quota / line_rate > unused_verifier_lifetime`. At the deployed 1.5 Mbps
    /// per-Session cap a 519 MiB cycle runs 48.4 minutes against a 30-minute default, so every
    /// polynomial past ≈62 % of the cycle was lost.
    ///
    /// Scaled down here to two polynomials and a half-second lifetime; the shape is identical.
    ///
    /// The essential geometry is that the cycle is *continuously* busy while any single polynomial
    /// is not. Shares are spaced at half the lifetime, so the cycle never goes idle, but the four
    /// shares of polynomial 0 take twice the lifetime to arrive — long enough that polynomial 1's
    /// builder, untouched since installation, would have expired under the old per-polynomial key.
    #[test]
    fn a_cycle_stays_live_while_a_single_polynomial_goes_untouched() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u16 = 4;
        const VERIFIER_LIFETIME: std::time::Duration = std::time::Duration::from_millis(500);
        /// Comfortably inside the lifetime, so no *cycle* is ever idle long enough to expire.
        const SHARE_SPACING: std::time::Duration = std::time::Duration::from_millis(250);

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            unused_verifier_lifetime: VERIFIER_LIFETIME,
            ..Default::default()
        });
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;

        // The whole commitment set lands up front, as it does in production: every part builder is
        // installed now, and the later ones then wait out most of the cycle.
        generator
            .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?
            .process_into_reconstructor(&reconstructor)?;

        let installed_at = std::time::Instant::now();
        let mut recovered = false;
        for i in 0..(POLYS * THRESHOLD) as usize {
            std::thread::sleep(SHARE_SPACING);

            // Shares are emitted polynomial-major, so this is the hand-over to polynomial 1 — the
            // point at which its builder has been idle for the whole of polynomial 0's run.
            if i == THRESHOLD as usize {
                assert!(
                    installed_at.elapsed() > VERIFIER_LIFETIME,
                    "the test must actually outlast the lifetime before polynomial 1's first share, otherwise it \
                     exercises nothing"
                );
            }

            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let share = generator
                .next_share(&pseudonym, &msg)?
                .ok_or_else(|| anyhow::anyhow!("generator must yield a share"))?;

            let ack = HalfKey::random();
            let enc = share.share.encrypt(&share.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                ack.to_challenge()?,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
            )?;
            recovered |= reconstructor
                .acknowledge_shares(*peer.public(), vec![VerifiedAcknowledgement::new(ack, &peer).leak()])?
                .iter()
                .any(|r| matches!(r, ShareResolution::RecoveredSsa(r) if r.ssa_id == ssa_id));
        }

        if !recovered {
            // Name the mechanism, so a regression reports the cause and not just the symptom. Under
            // H8 the late polynomial's builder was gone and its shares were stranded in a bucket
            // whose only drain had already run.
            assert_eq!(
                0,
                reconstructor.deferred_ack_count(&ssa_id),
                "shares were stranded in a deferred-ack bucket — H8 has regressed"
            );
            panic!("the cycle failed to recover even though it was continuously busy");
        }

        Ok(())
    }

    /// The idle timer was not simply disabled: a cycle that receives no shares at all is still
    /// reclaimed on schedule.
    ///
    /// This is the other half of the H8 regression. Widening the reclamation scope is only correct
    /// if reclamation still happens — otherwise an abandoned cycle is pinned until session
    /// teardown.
    #[test]
    fn a_cycle_with_no_shares_at_all_still_expires() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u16 = 2;
        const VERIFIER_LIFETIME: std::time::Duration = std::time::Duration::from_millis(500);

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            unused_verifier_lifetime: VERIFIER_LIFETIME,
            ..Default::default()
        });
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        generator
            .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?
            .process_into_reconstructor(&reconstructor)?;

        assert_eq!(POLYS as usize, reconstructor.installed_parts(&ssa_id));

        std::thread::sleep(VERIFIER_LIFETIME * 3);

        assert_eq!(
            0,
            reconstructor.live_cycles(),
            "a cycle that never received a share must still be reclaimed"
        );

        Ok(())
    }

    /// The polynomial index travels inside a peer-supplied share, so it reaches the slot lookup as
    /// untrusted input. Once the cycle is known its dimensions are too, which makes an out-of-range
    /// index definitively invalid rather than merely early — and it must not index the slot array.
    #[test]
    fn a_share_naming_a_polynomial_outside_the_cycle_is_rejected() -> anyhow::Result<()> {
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

        // Take a real share and re-label it for a polynomial the cycle does not have.
        let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
        let mut share = generator
            .next_share(&pseudonym, &msg)?
            .ok_or_else(|| anyhow::anyhow!("generator must yield a share"))?;
        share.id = SsaPolynomialId::new(ssa_id, POLYS as PolynomialIndex + 5);

        let ack = HalfKey::random();
        let enc = share.share.encrypt(&share.id, &ack)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            ack.to_challenge()?,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
        )?;

        // `acknowledge_shares` logs and swallows the error, so the observable contract is that
        // nothing resolves, nothing is deferred, and the process is still standing.
        let resolutions =
            reconstructor.acknowledge_shares(*peer.public(), vec![VerifiedAcknowledgement::new(ack, &peer).leak()])?;
        assert!(resolutions.is_empty(), "an out-of-range share must resolve to nothing");
        assert_eq!(
            0,
            reconstructor.deferred_ack_count(&ssa_id),
            "an out-of-range share must be rejected outright, not deferred forever"
        );
        assert!(
            reconstructor.cycle(&ssa_id).is_some(),
            "the cycle itself must be unharmed"
        );

        Ok(())
    }

    /// Shares for different polynomials of one cycle must be able to run concurrently.
    ///
    /// The cycle is a single cache entry, which makes collapsing it to a single mutex an easy and
    /// invisible mistake — and one that would serialise every share of a Session. This holds one
    /// polynomial's lock and asserts another's is still free.
    #[test]
    fn parts_of_one_cycle_lock_independently() -> anyhow::Result<()> {
        const POLYS: u16 = 4;
        const THRESHOLD: u16 = 2;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        generator
            .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?
            .process_into_reconstructor(&reconstructor)?;

        let cycle = reconstructor
            .cycle(&ssa_id)
            .ok_or_else(|| anyhow::anyhow!("cycle must be live"))?;

        let held = cycle.part(0).ok_or_else(|| anyhow::anyhow!("missing part 0"))?.lock();
        for poly in 1..POLYS as PolynomialIndex {
            assert!(
                cycle
                    .part(poly)
                    .ok_or_else(|| anyhow::anyhow!("missing part {poly}"))?
                    .try_lock()
                    .is_some(),
                "polynomial {poly} must not be blocked by polynomial 0 — the cycle must not share one mutex"
            );
        }
        // The accumulator is a separate lock too, so a part in flight does not block recovery
        // accounting for another part.
        assert!(
            cycle.builder().try_lock().is_some(),
            "the accumulator must lock separately"
        );
        drop(held);

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

        // Complete the first few IDs through the full commitment path to populate `ssa_cycles`.
        // Use a 2-poly 2-threshold generator so that insert_coefficient_commitments reaches the
        // completion milestone and publishes a cycle.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        });
        for ssa_id in ids.iter().take(3) {
            let commit = generator.new_ssa_commitment(&pseudonym, ssa_id.ssa_index())?;
            commit.process_into_reconstructor(&reconstructor)?;
            reconstructor.commitment_builder.run_pending_tasks();

            assert_eq!(
                2,
                reconstructor.installed_parts(ssa_id),
                "ssa_cycles must contain completed SsaId index {} ({ssa_id:?})",
                ssa_id.ssa_index().get(),
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

        assert_eq!(
            0,
            reconstructor.live_cycles(),
            "tombstone must prevent cycle publication"
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
        assert_eq!(
            0,
            reconstructor.live_cycles(),
            "no part builder may be installed while the SSA commitment is unknown"
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

        assert_eq!(
            0,
            reconstructor.live_cycles(),
            "tombstone must withdraw the cycle published after retirement — accumulator and every part builder"
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
        assert_eq!(0, reconstructor.live_cycles(), "no cycle may be published");

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
        // The published cycle is what makes an SSA live and able to accept recovered shares.
        // `commitment_builder` legitimately still exists — `new_exit_commitment` created it.
        assert_eq!(
            0,
            reconstructor.live_cycles(),
            "no cycle must be published for an unproven commitment"
        );

        Ok(())
    }
}
