mod utils;

use hopr_types::{
    crypto::{
        crypto_traits::elliptic_curve::Field,
        prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey},
    },
    internal::prelude::Acknowledgement,
};
use utils::{AddShareOutcome, SsaCommitmentBuilder, SsaCycle};
use validator::Validate;

use crate::{
    CoefficientIndex, ExitAcknowledgementShareProcessor, Group, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, PixGroup,
    PixGroupRepr, PixScalar, PixSpec, PolynomialIndex, RecoveredSsa, ShareResolution, SsaCommitmentProof,
    SsaCommitmentState, SsaPolynomialId, SsaRecoveryProgress, TaggedEncryptedPartialSsaShare, errors::PixError,
    types::SsaId,
};

/// Configuration for the SSA reconstructor.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, validator::Validate)]
pub struct SsaReconstructorConfig {
    /// Time until the complete commitment to an SSA must be received.
    ///
    /// Default is 2 minutes.
    #[default(Self::DEFAULT_INCOMPLETE_COMMITMENT_LIFETIME)]
    pub incomplete_commitment_lifetime: std::time::Duration,
    /// Maximum time an SSA cycle can go without progress before it is discarded.
    ///
    /// Measured from the last acknowledged share *anywhere* in the cycle, not per polynomial — see
    /// `SsaCycle` for why that distinction is load-bearing. A cycle that is still being served
    /// therefore never expires, whatever the line rate.
    ///
    /// Default is 30 minutes.
    #[default(Self::DEFAULT_UNUSED_VERIFIER_LIFETIME)]
    pub unused_verifier_lifetime: std::time::Duration,
    /// Maximum number of peers that can be tracked simultaneously with unacknowledged shares.
    ///
    /// Default is 2000, minimum is 10.
    ///
    /// This is a per-*peer* fan-out bound, and it guards the opposite concentration to
    /// [`max_awaiting_acks`](Self::max_awaiting_acks): traffic spread thinly across many
    /// first-relayers. The two cannot both be saturated at once, which is why their product is not
    /// the reconstructor's memory bound — see `PixReconstructorConfig` in `hopr-transport` for the
    /// bound that is.
    #[validate(range(min = 10))]
    #[default(Self::DEFAULT_MAX_TRACKED_PEERS)]
    pub max_tracked_peers: usize,
    /// Maximum number of awaited acknowledgements to extract a single share.
    ///
    /// This corresponds to the maximum number of unacknowledged HOPR packets awaiting acknowledgement.
    ///
    /// Default is 1 000 000, must be at least 10 000.
    ///
    /// Sizes one inner cache **per peer**, so it must cover the concentrated case: every Session on
    /// the node returning through a single first-relayer. At the operating point
    /// `tests/memory_profile.rs` models that is ~542 000 entries, which is what makes a cap of this
    /// order the right one rather than an oversight.
    #[default(Self::DEFAULT_MAX_AWAITING_ACKS)]
    #[validate(range(min = 10000))]
    pub max_awaiting_acks: usize,
    /// Maximum time an acknowledgement can be awaited before it is discarded.
    ///
    /// Default is 30 seconds.
    ///
    /// Multiplies the whole awaiting-ack buffer: the reachable state is the Exit's share-emission
    /// rate times this window, so it — not either cap above — is the dial that actually sizes the
    /// buffer.
    #[default(Self::DEFAULT_MAX_ACK_AWAIT_TIME)]
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
    /// **Settled against the concurrent pipeline shape**, which is what the Exit actually runs and
    /// what the sequential figures above could not speak to. `concurrent_quota_rate` on 48 cores,
    /// aggregate MiB/s of Session quota:
    ///
    /// | callers | unbatched | batched  |                              |
    /// | ------- | --------- | -------- | ---------------------------- |
    /// | 1       | **90.8**  | 47.7     | unbatched 1.90×              |
    /// | 10      | 137.5     | 133.4    | tie — confidence intervals overlap |
    /// | 48      | 150.4     | **162.6**| batched 1.08×                |
    ///
    /// So batching does eventually pay, but only *above* the concurrency the pipeline is
    /// configured for: `DEFAULT_ACK_INPUT_CONCURRENCY` is 10, which is precisely the row where the
    /// two are indistinguishable. `false` stays the default because it is far better at low
    /// concurrency and no worse at the configured one, making it the safer choice across the range
    /// an operator can set — not because batching is slower everywhere.
    ///
    /// Kept configurable for the operator who raises `ack_input_concurrency` well past its default,
    /// where the last row says the choice flips. Note that none of this binds capacity: 137 MiB/s
    /// at the production shape is 7.3× the 18.75 MiB/s that 100 concurrent Sessions demand.
    #[default(Self::DEFAULT_USE_BATCH_VERIFICATION)]
    pub use_batch_verification: bool,
    /// Fraction of reconstructed polynomials at which to emit an early recovery
    /// notification, triggering pipelined SSA request preparation.
    ///
    /// Range: 0.0..1.0. Default: 0.85.
    #[default(Self::DEFAULT_EARLY_RECOVERY_THRESHOLD)]
    #[validate(range(min = 0.0, max = 1.0))]
    pub early_recovery_threshold: f64,
    /// Ceiling on the total live state held in the awaiting-acknowledgement buffer, across every
    /// peer, in bytes.
    ///
    /// This is the *global* bound; [`max_tracked_peers`](Self::max_tracked_peers) and
    /// [`max_awaiting_acks`](Self::max_awaiting_acks) are per-dimension backstops and their product
    /// is not one. See [`SsaReconstructor::insert_encrypted_share`] for why the product overstates
    /// by roughly three thousandfold and why bounding it instead would lose shares.
    ///
    /// Enforced at insertion time rather than by validating a workload model. A model has to assume
    /// a Session count and a packet rate; the node enforces neither — `maximum_managed_sessions`
    /// validates to 100 000 and `SessionCapability::NoRateControl` removes the rate limiter
    /// entirely — so a configuration can be perfectly valid and still exceed any modelled budget.
    /// Counting what is actually held is indifferent to all of that.
    ///
    /// Default is 1 GiB, which at [`AWAITING_ACK_ENTRY_BYTES`] is ~2.68 M shares in flight.
    ///
    /// The minimum is 25 600 B — 64 entries. That is a sanity floor, not a sizing recommendation:
    /// its only job is to stop the budget rounding down to a handful of shares. Whether a given
    /// value is *adequate* depends on the node's traffic, which is exactly the thing this design
    /// stopped trying to predict, so the floor deliberately does not pretend to encode it. A node
    /// configured near it will drop shares and say so.
    #[default(Self::DEFAULT_MAX_ACK_BUFFER_BYTES)]
    #[validate(range(min = 25_600))]
    pub max_ack_buffer_bytes: usize,
}

/// Live heap one entry in the awaiting-acknowledgement buffer costs, in bytes.
///
/// **Measured, not derived.** `size_of` accounts for only 145 B of it — a 33 B `HalfKeyChallenge`
/// key and a 112 B [`TaggedEncryptedPartialSsaShare`] value, both inline arrays — and moka's
/// per-entry bookkeeping (hash map entry, LRU node, TTL timer-wheel node, the `Arc` around the
/// value) is the other 244 B. Run `awaiting_ack_entry_cost` in `tests/memory_profile.rs` to
/// re-derive it; at the time of writing it reports 383 B/entry at 20 000 entries and 389 B/entry at
/// 100 000. Rounded up, because understating it would let
/// [`max_ack_buffer_bytes`](SsaReconstructorConfig::max_ack_buffer_bytes) be exceeded.
///
/// Every entry is this size — the payload is fixed-width inline arrays with no indirection — which
/// is what lets the runtime bound count entries rather than weigh each one.
pub const AWAITING_ACK_ENTRY_BYTES: usize = 400;

/// The defaults, named so that a mirror can share them instead of restating them.
///
/// `hopr-transport`'s `PixReconstructorConfig` is the operator-facing shape of this struct, and a
/// second copy of these literals over there would be a second thing to keep true. Referencing them
/// from both `#[default(…)]` sites means the two cannot disagree by construction, which is a
/// stronger guarantee than any test comparing them after the fact.
impl SsaReconstructorConfig {
    /// Default [`early_recovery_threshold`](Self::early_recovery_threshold).
    pub const DEFAULT_EARLY_RECOVERY_THRESHOLD: f64 = 0.85;
    /// Default [`incomplete_commitment_lifetime`](Self::incomplete_commitment_lifetime).
    pub const DEFAULT_INCOMPLETE_COMMITMENT_LIFETIME: std::time::Duration = std::time::Duration::from_secs(120);
    /// Default [`max_ack_await_time`](Self::max_ack_await_time).
    pub const DEFAULT_MAX_ACK_AWAIT_TIME: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default [`max_ack_buffer_bytes`](Self::max_ack_buffer_bytes).
    pub const DEFAULT_MAX_ACK_BUFFER_BYTES: usize = 1024 * 1024 * 1024;
    /// Default [`max_awaiting_acks`](Self::max_awaiting_acks).
    pub const DEFAULT_MAX_AWAITING_ACKS: usize = 1_000_000;
    /// Default [`max_tracked_peers`](Self::max_tracked_peers).
    pub const DEFAULT_MAX_TRACKED_PEERS: usize = 2000;
    /// Default [`unused_verifier_lifetime`](Self::unused_verifier_lifetime).
    pub const DEFAULT_UNUSED_VERIFIER_LIFETIME: std::time::Duration = std::time::Duration::from_secs(1800);
    /// Default [`use_batch_verification`](Self::use_batch_verification).
    pub const DEFAULT_USE_BATCH_VERIFICATION: bool = false;
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
#[derive(Default)]
struct DeferredAcks {
    by_poly: std::collections::HashMap<PolynomialIndex, Vec<DeferredAck>>,
    /// Running sum of the `by_poly` lengths, maintained so the per-cycle cap is an O(1) check.
    ///
    /// Recomputing it would walk every sub-bucket inside the mutex on every deferral, and there
    /// can be one sub-bucket per entry: filling a bucket to
    /// [`MAX_DEFERRED_ACKS_PER_CYCLE`] would then cost ~33M map-entry visits, on the path every
    /// acknowledgement takes during the commitment window. The invariant
    /// `total == by_poly.values().map(Vec::len).sum()` has one increment site and one reset site,
    /// both under this mutex.
    total: usize,
    /// Set by the one drain this bucket will ever get, in the same critical section as the take.
    ///
    /// A bucket is reachable through two routes: the `pending_acks` key, and an `Arc` a
    /// [`defer_ack`](SsaReconstructor::defer_ack) already holds. The drain removes the first but
    /// cannot revoke the second, so an append that lands after it would sit in a bucket nothing
    /// will ever read again. This flag is how such an append notices — see
    /// [`Deferral::Orphaned`].
    drained: bool,
}

type DeferredAckBucket = std::sync::Arc<parking_lot::Mutex<DeferredAcks>>;

/// What became of an acknowledgement handed to [`defer_ack`](SsaReconstructor::defer_ack).
///
/// Fieldless — the caller still owns the acknowledgement, since [`DeferredAck`] is `Copy` — so this
/// stays a discriminant rather than carrying 264 bytes back out of the critical section.
#[derive(Clone, Copy)]
enum Deferral {
    /// Appended to a live bucket. The drain that installs the cycle will redeem it.
    Buffered,
    /// The bucket had already been drained, so no drain will come for this one. The caller must
    /// redeem it inline; parking it would be silent loss.
    Orphaned,
    /// Over one of the two caps. Already warned about, and deliberately discarded.
    Dropped,
}

/// Cap on deferred acknowledgements held for a single polynomial.
///
/// A conforming Entry emits `threshold + surplus` shares per polynomial — 96 at the default
/// dimensions — across all return paths combined, so at the defaults this cannot be reached without
/// the peer exceeding its own share budget. Anything above the cap is dropped rather than buffered.
///
/// It is *not* unreachable in general: both halves are a byte wide, so a conforming Entry may
/// legitimately announce up to `255 + 255` and have its excess deferrals silently discarded. Both
/// values now travel in [`PixParams`](crate::PixParams), so an Exit that cares can compare
/// `shares_per_poly + surplus_shares` against this cap when it accepts a Session, instead of
/// discovering the overflow one dropped acknowledgement at a time.
///
/// Public because it is observable behaviour, not an implementation detail: past the cap an
/// acknowledgement is discarded, so anything measuring or exercising the deferral path has to stay
/// underneath it or it silently measures the discard instead.
pub const MAX_DEFERRED_ACKS_PER_POLYNOMIAL: usize = 128;

/// Cap on deferred acknowledgements held across all polynomials of one cycle.
///
/// The per-polynomial cap alone leaves the cycle total at `num_polys × 128` — a million entries at
/// production dimensions, which is no bound at all. This one is derived rather than chosen: the
/// drain discards any acknowledgement whose share has already left `awaiting_acks`, and that cache
/// expires entries after
/// `max_ack_await_time` (30 s by default). An older deferral is therefore provably dead, so the
/// ceiling only has to cover the shares one cycle can receive inside that window: ~181 shares/s at
/// the deployed 1.5 Mbps per-Session cap, so ~5 400. 8192 leaves ~1.5× headroom and costs at most
/// ~786 KB per cycle.
///
/// Public for the same reason as [`MAX_DEFERRED_ACKS_PER_POLYNOMIAL`].
pub const MAX_DEFERRED_ACKS_PER_CYCLE: usize = 8192;

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
    /// speculatively. That is what keeps [`Self::acknowledge_shares`] free of retry work: it only
    /// ever *appends* to a bucket.
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
    /// to be picked up by the next [`Self::acknowledge_shares`] call.
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
    /// Running estimate of the entries live in [`awaiting_acks`](Self::awaiting_acks), summed over
    /// every peer, so the global budget costs one relaxed load per insertion.
    ///
    /// An *estimate*, and knowingly so — see [`Self::resync_ack_buffer`] for the one drift source
    /// that cannot be listened for and what keeps it from accumulating.
    ///
    /// Behind an `Arc` because each peer's inner cache decrements it from an eviction listener, and
    /// moka requires those to be `'static` — they cannot borrow the reconstructor that owns them.
    ack_buffer_entries: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    /// [`max_ack_buffer_bytes`](SsaReconstructorConfig::max_ack_buffer_bytes) in entries, divided
    /// once here rather than on every insertion.
    max_ack_buffer_entries: usize,
    /// When [`resync_ack_buffer`](Self::resync_ack_buffer) last ran, and the lock that keeps two
    /// from running at once.
    ///
    /// `None` until the first run, so a buffer that saturates immediately is not made to wait out
    /// an interval before its first ground-truth reading.
    ack_buffer_resync: parking_lot::Mutex<Option<std::time::Instant>>,
    cfg: SsaReconstructorConfig,
}

/// Result of processing a single verified acknowledgement in the SSA reconstructor.
///
/// The counters behind [`SsaRecoveryProgress`] are updated by `process_verified_ack` itself, on the
/// cycle it already holds, so these variants only have to say *whether* a snapshot is worth emitting
/// — not what changed. That is why a duplicate, a surplus share and an unmatched acknowledgement all
/// collapse into [`NoProgress`](Self::NoProgress): none of them moves a counter, so none of them can
/// make a snapshot differ from the last one sent.
enum ProcessedAckResult<S: PixSpec> {
    /// Nothing to report: the acknowledgement matched no pending share, or the share was a
    /// duplicate, a surplus, or absorbed by an already-failed polynomial.
    NoProgress,
    /// The share is valid but its polynomial's verifier is not installed yet, so it cannot be
    /// checked. Deferral, not failure: the ack is bucketed under this
    /// [`SsaPolynomialId`] and retried once the verifier arrives.
    VerifierNotReady(SsaPolynomialId<<S as PixSpec>::Pseudonym>),
    /// The share advanced reconstruction without finishing it.
    Progressed(SsaRecoveryProgress<<S as PixSpec>::Pseudonym>),
    /// The share failed verification. Carries the SSA's aggregate fault total across all peers.
    InvalidShare(SsaId<<S as PixSpec>::Pseudonym>, u64),
    /// The early recovery threshold was crossed.
    EarlyRecovery(SsaRecoveryProgress<<S as PixSpec>::Pseudonym>),
    /// Full SSA was recovered. Carries the cycle's final progress, captured before its state was
    /// released — afterwards there is nothing left to read it from.
    FullRecovery(
        RecoveredSsa<<S as PixSpec>::Pseudonym, <S as PixSpec>::AddressPrivateKey>,
        SsaRecoveryProgress<<S as PixSpec>::Pseudonym>,
    ),
}

/// Merges a snapshot into the batch's pending set, keeping the furthest-along one per SSA.
///
/// Concurrent batches share a cycle's counters, so snapshots taken microseconds apart can be
/// unordered. Keeping the maximum means one batch never reports its own SSA going backwards.
fn record_progress<P: PartialEq>(acc: &mut Vec<SsaRecoveryProgress<P>>, snapshot: SsaRecoveryProgress<P>) {
    match acc.iter_mut().find(|p| p.ssa_id == snapshot.ssa_id) {
        Some(existing) if existing.useful_shares >= snapshot.useful_shares => {}
        Some(existing) => *existing = snapshot,
        None => acc.push(snapshot),
    }
}

/// One SSA's fault observation, with the relayer that carried the offending share.
type FaultObservation<P> = (Box<OffchainPublicKey>, SsaId<P>, u64);

/// Merges a fault observation into the batch's pending set, keeping the highest total per SSA.
///
/// The relayer travels with the total rather than being filled in at emission time: a batch can also
/// carry faults redeemed from deferral, and those were relayed by whoever held the share at the time
/// — not by the peer whose acknowledgements are being processed now.
fn record_fault<P: PartialEq>(acc: &mut Vec<FaultObservation<P>>, observation: FaultObservation<P>) {
    match acc.iter_mut().find(|(_, id, _)| *id == observation.1) {
        Some(existing) if existing.2 >= observation.2 => {}
        Some(existing) => *existing = observation,
        None => acc.push(observation),
    }
}

/// Appends a resolution unless an equal one is already present.
fn push_unique<P: PartialEq, A>(acc: &mut Vec<ShareResolution<P, A>>, resolution: ShareResolution<P, A>) {
    if !acc.contains(&resolution) {
        acc.push(resolution);
    }
}

impl<S: PixSpec + Clone> Default for SsaReconstructor<S> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<S: PixSpec + Clone> SsaReconstructor<S> {
    /// Creates a new SSA reconstructor from the given configuration.
    ///
    /// Fails if the configuration does not validate. Prefer this over [`Self::new`] anywhere the
    /// configuration is assembled at runtime — a config built programmatically or read from a file
    /// is input, not a constant, and turning it into a panic makes it un-handleable by the caller.
    pub fn try_new(cfg: SsaReconstructorConfig) -> Result<Self, PixError<S::Pseudonym>> {
        cfg.validate()?;
        Ok(Self {
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
                // Dropping a peer entry drops its whole inner cache, and dropping a moka handle
                // does not run that cache's eviction listener — so without this, every entry the
                // peer still held would stay counted against the global budget forever. Invalidating
                // routes them through the inner listener instead.
                //
                // `run_pending_tasks()` is deliberately not called here: it is unbounded work on
                // whichever thread happened to trigger maintenance. That leaves the invalidation
                // best-effort, which is precisely why `resync_ack_buffer` exists — this narrows the
                // drift, it does not close it.
                .eviction_listener(|_, shares: EncryptedShareCache<S>, _| shares.invalidate_all())
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
            // Tombstone set. Its immediate job is the window between `retire_ssa` running and a
            // concurrent commitment completion publishing its cycle — but the TTL must outlive that
            // by a long way, because retirement is also permanent: a cycle re-registered at the same
            // `SsaId` after being abandoned must stay retired, which is what
            // `abandoning_a_live_cycle_retires_it_rather_than_just_releasing_it` asserts. Shortening
            // this to the width of the race would break that contract silently.
            //
            // Unbounded in count, deliberately for now: a size eviction here permits exactly the
            // resurrection the tombstone prevents, so a capacity has to be chosen against the
            // concurrent-Session budget rather than picked. That belongs with the global admission
            // control the memory work still owes.
            retired_ssas: moka::sync::Cache::builder()
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            ack_buffer_entries: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            // At least one, so a budget rounded below one entry refuses everything rather than
            // dividing to zero and admitting everything.
            max_ack_buffer_entries: (cfg.max_ack_buffer_bytes / AWAITING_ACK_ENTRY_BYTES).max(1),
            ack_buffer_resync: parking_lot::Mutex::new(None),
            cfg,
        })
    }

    /// Creates a new SSA reconstructor from the given configuration.
    ///
    /// # Panics
    /// Panics if the configuration fails validation. Use [`Self::try_new`] to handle that case
    /// instead.
    pub fn new(cfg: SsaReconstructorConfig) -> Self {
        Self::try_new(cfg).expect("invalid SsaReconstructorConfig")
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

        let ssa_id = *spi.as_ref();

        // The part lock is released before the accumulator is taken below. That order is the one
        // callers must keep, and neither lock is ever held across the other.
        let ssa_part = match part.lock().add_share(share.nonce, partial_share) {
            Ok(AddShareOutcome::Completed(share)) => {
                tracing::trace!(%spi, "ssa part complete");
                cycle.record_useful_share();
                cycle.record_completed_part();
                share
            }
            Ok(AddShareOutcome::Useful) => {
                tracing::trace!(%spi, "ssa part not yet complete, waiting for more shares");
                cycle.record_useful_share();
                // Snapshot here rather than making the caller look the cycle up again: this is the
                // steady-state outcome for all but one share in `threshold`, so a second cache get
                // would double the lookups on the hot path.
                return Ok(ProcessedAckResult::Progressed(cycle.progress()));
            }
            // Expected traffic, not a fault: a conforming Entry emits `threshold + surplus` shares
            // per polynomial, so every polynomial ends its life absorbing surplus.
            Ok(AddShareOutcome::Surplus) => {
                tracing::trace!(%spi, "share arrived after its polynomial was reconstructed");
                return Ok(ProcessedAckResult::NoProgress);
            }
            Ok(AddShareOutcome::Duplicate) => {
                tracing::trace!(%spi, "duplicate evaluation identifier");
                return Ok(ProcessedAckResult::NoProgress);
            }
            Ok(AddShareOutcome::Absorbed) => {
                tracing::trace!(%spi, "share for a polynomial that already failed its commitment");
                return Ok(ProcessedAckResult::NoProgress);
            }
            Err(PixError::VsssError(vsss_rs::Error::InvalidShare)) => {
                // Counted rather than raised: the caller reports it as a resolution, and the count
                // has to be taken here because the cycle that holds it is in hand.
                //
                // Almost always this means the polynomial's reconstructed constant term did not
                // open its commitment, in which case the offending share is one of the `threshold`
                // that went into it and cannot be singled out. The whole cycle is lost either way,
                // since the SSA needs every polynomial.
                let observed_total = cycle.record_invalid_share();
                tracing::error!(%spi, observed_total, "ssa part failed to open its commitment");
                return Ok(ProcessedAckResult::InvalidShare(ssa_id, observed_total));
            }
            Err(e) => return Err(e),
        };

        let mut builder_guard = cycle.builder().lock();
        let ssa = match builder_guard.add_recovered_ssa_part(spi.poly_index(), ssa_part) {
            Ok(ssa) => ssa,
            Err(error) => {
                // As terminal as `scalar_to_private_key` returning `None` below, and torn down the
                // same way. Propagating alone would leave the accumulator and every part builder in
                // place, and each further share for the cycle would refresh the idle timer that is
                // supposed to reclaim them — so a Session that keeps sending holds a cycle that can
                // never reconstruct for as long as it likes.
                //
                // The lock goes first: `remove_cycle` drops the last `Arc` to this very cycle.
                drop(builder_guard);
                tracing::error!(%spi, %error, "ssa part could not be added to its accumulator");
                self.remove_cycle(ssa_id);
                return Err(error);
            }
        };
        match ssa {
            Some(scalar) => {
                // Read the final progress while the cycle is still live: `remove_cycle` below drops
                // the counters along with everything else, so this is the last chance to report them.
                let progress = cycle.progress();
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
                Ok(ProcessedAckResult::FullRecovery(RecoveredSsa { ssa_id, ssa }, progress))
            }
            None => {
                tracing::trace!(%spi, "ssa not yet complete, waiting for more ssa parts");
                // Check early threshold while we hold the lock
                let early = builder_guard.check_early_threshold(self.cfg.early_recovery_threshold);
                drop(builder_guard);
                let progress = cycle.progress();
                if early {
                    tracing::info!(%ssa_id, "early recovery threshold reached");
                    Ok(ProcessedAckResult::EarlyRecovery(progress))
                } else {
                    Ok(ProcessedAckResult::Progressed(progress))
                }
            }
        }
    }

    /// Buckets an acknowledgement whose cycle's part builders have not been installed yet.
    ///
    /// O(1) — this is the entire cost the acknowledgement path pays for a deferral.
    fn defer_ack(&self, spi: SsaPolynomialId<S::Pseudonym>, deferred: DeferredAck) {
        let bucket = self.pending_acks.get_with(*spi.as_ref(), || {
            std::sync::Arc::new(parking_lot::Mutex::new(Default::default()))
        });
        self.defer_ack_into(&bucket, spi, deferred);
    }

    /// The bucket half of [`defer_ack`](Self::defer_ack), taking the bucket rather than looking it
    /// up.
    ///
    /// Split out so a test can hold a handle across the drain that invalidates the cache key,
    /// which is the interleaving this guards against and the one thing a single thread cannot
    /// otherwise produce — after the invalidate, `get_with` hands out a *fresh* bucket.
    fn defer_ack_into(&self, bucket: &DeferredAckBucket, spi: SsaPolynomialId<S::Pseudonym>, deferred: DeferredAck) {
        let ssa_id = *spi.as_ref();
        let outcome = {
            let mut bucket = bucket.lock();
            if bucket.drained {
                Deferral::Orphaned
            } else if bucket.total >= MAX_DEFERRED_ACKS_PER_CYCLE {
                // The cycle as a whole is holding more than the shares it could plausibly have
                // received inside `max_ack_await_time`, so the excess cannot be redeemable.
                tracing::warn!(
                    %ssa_id,
                    cap = MAX_DEFERRED_ACKS_PER_CYCLE,
                    "dropping deferred acknowledgement: cycle bucket is full"
                );
                Deferral::Dropped
            } else {
                // Reborrowed off the guard so `by_poly` and `total` are disjoint field borrows.
                let bucket = &mut *bucket;
                let per_poly = bucket.by_poly.entry(spi.poly_index()).or_default();
                if per_poly.len() >= MAX_DEFERRED_ACKS_PER_POLYNOMIAL {
                    // Only reachable if the peer emits more shares for one polynomial than its own
                    // `threshold + surplus` budget allows, so the excess is almost certainly
                    // duplicate.
                    tracing::warn!(
                        %spi,
                        cap = MAX_DEFERRED_ACKS_PER_POLYNOMIAL,
                        "dropping deferred acknowledgement: polynomial bucket is full"
                    );
                    Deferral::Dropped
                } else {
                    per_poly.push(deferred);
                    bucket.total += 1;
                    Deferral::Buffered
                }
            }
        };

        match outcome {
            // The drain took this bucket while we were on our way into it. Redeeming here is what
            // makes the mutex the whole synchronisation point: the drain's take and this append
            // are serialised by it, so exactly one of them owns the ack.
            Deferral::Orphaned => {
                tracing::trace!(%spi, "redeeming an acknowledgement deferred into a drained bucket");
                self.redeem_deferred_acks(&ssa_id, std::iter::once(deferred));
            }
            // Close the race against a concurrent installation. The decision to defer was made on
            // a cycle lookup that missed; if the cycle has appeared since, the drain that would
            // have redeemed this ack may already have run against a bucket we never saw.
            Deferral::Buffered => {
                if self.ssa_cycles.contains_key(&ssa_id) {
                    self.drain_deferred_acks(&ssa_id);
                }
            }
            Deferral::Dropped => {}
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

        // Take and tombstone in one critical section. A `defer_ack_into` that looked this bucket
        // up before the invalidate still holds an `Arc` to it; the flag is what stops its append
        // from disappearing into a bucket nothing will read again.
        let deferred = {
            let mut bucket = bucket.lock();
            bucket.drained = true;
            bucket.total = 0;
            std::mem::take(&mut bucket.by_poly)
        };
        if deferred.is_empty() {
            return;
        }

        self.redeem_deferred_acks(ssa_id, deferred.into_values().flatten());
    }

    /// Processes acknowledgements whose verifier has since been installed, parking whatever they
    /// resolve to.
    ///
    /// Shared by the two routes that can redeem a deferral — the drain on the commitment path and
    /// an [`orphaned`](Deferral::Orphaned) append — so both produce the same resolutions in the
    /// same order.
    fn redeem_deferred_acks(&self, ssa_id: &SsaId<S::Pseudonym>, deferred: impl IntoIterator<Item = DeferredAck>) {
        let mut resolved = Vec::new();
        // The furthest-along snapshot any redeemed ack produced. Shares recovered here would
        // otherwise be invisible to the consumer until some later batch happened to touch this same
        // SSA, since only `acknowledge_shares` emits snapshots.
        let mut progress = Vec::new();
        for (peer, challenge, ack) in deferred {
            // The share lives in the peer's own awaiting-acks cache; if the peer entry is gone the
            // share has expired with it and the ack is dead.
            let Some(awaiting) = self.awaiting_acks.get(&peer) else {
                continue;
            };
            match self.process_verified_ack(ack, challenge, &awaiting) {
                Ok(ProcessedAckResult::FullRecovery(ssa, snapshot)) => {
                    record_progress(&mut progress, snapshot);
                    resolved.push(ShareResolution::RecoveredSsa(ssa));
                }
                Ok(ProcessedAckResult::EarlyRecovery(snapshot)) => {
                    let id = snapshot.ssa_id;
                    record_progress(&mut progress, snapshot);
                    resolved.push(ShareResolution::AlmostRecoveredSsa(id));
                }
                Ok(ProcessedAckResult::Progressed(snapshot)) => record_progress(&mut progress, snapshot),
                Ok(ProcessedAckResult::InvalidShare(id, observed_total)) => {
                    tracing::error!(%id, observed_total, "deferred share could not be verified");
                    resolved.push(ShareResolution::InvalidShares {
                        peer: peer.into(),
                        ssa_id: id,
                        observed_total,
                    });
                }
                Ok(ProcessedAckResult::NoProgress) => {}
                Ok(ProcessedAckResult::VerifierNotReady(_)) => {
                    // The cycle was installed and then immediately withdrawn, which only the
                    // retirement path does. Re-bucketing would leak, so drop.
                    tracing::trace!(%ssa_id, "cycle withdrawn while draining deferred acknowledgements");
                }
                Err(error) => tracing::debug!(%ssa_id, %error, "failed to process deferred acknowledgement"),
            }
        }

        // Snapshots go in ahead of the terminal events they belong to, matching what
        // `acknowledge_shares` emits — the consumer sees one order regardless of which path resolved
        // a share.
        if !progress.is_empty() {
            let mut ordered = progress.into_iter().map(ShareResolution::Progress).collect::<Vec<_>>();
            ordered.append(&mut resolved);
            resolved = ordered;
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

    /// Minimum wall time between two [`resync_ack_buffer`](Self::resync_ack_buffer) passes.
    ///
    /// A saturated buffer would otherwise turn every rejected insertion into an
    /// `O(max_tracked_peers)` scan — the overload path amplifying its own cost, which is the shape
    /// of bug this budget exists to prevent.
    ///
    /// Derived from [`max_ack_await_time`](SsaReconstructorConfig::max_ack_await_time) rather than
    /// fixed, because what a resync reclaims is entries that have aged out of *that* window. A
    /// constant would be wrong at both ends: too slow for a short window, so a drained buffer keeps
    /// refusing shares long after it emptied, and needlessly eager for a long one.
    ///
    /// The resulting staleness — up to ~1.9 s at the 30 s default — only bites when the counter has
    /// drifted high *and* nothing is touching the caches, since redemption and any cache access
    /// drive moka's expiry maintenance and fire the listener directly. That is the traffic-stopped
    /// case, where refusing a share costs nothing.
    fn ack_buffer_resync_interval(&self) -> std::time::Duration {
        (self.cfg.max_ack_await_time / 16).max(std::time::Duration::from_millis(1))
    }

    /// Recomputes [`ack_buffer_entries`](Self::ack_buffer_entries) from what the caches actually
    /// hold.
    ///
    /// # Why a counter needs a backstop at all
    ///
    /// Entries leave the buffer four ways: redeemed by their acknowledgement, expired, size-evicted
    /// from their peer's cache, or dropped wholesale when the peer's entry leaves `awaiting_acks`.
    /// The inner eviction listener catches the first three exactly. The fourth cannot be caught:
    /// dropping a moka handle does not run its eviction listener, so the outer listener falls back
    /// to `invalidate_all`, which is best-effort and races an insertion landing on the very cache
    /// being discarded.
    ///
    /// Left alone, that residue only ever accumulates *upward*, and an over-count is far worse than
    /// an under-count: it would eventually refuse every share while the buffer sat empty, turning a
    /// memory ceiling into a permanent outage of the acknowledgement path. (The sibling
    /// `HoprUnacknowledgedTicketProcessor` in `hopr-protocol-hopr` has the same nesting and the same
    /// residue; there it only skews metrics.)
    ///
    /// So the counter is treated as a hint that is allowed to be wrong, and ground truth is
    /// consulted at the one moment being wrong would cost something — when it says the buffer is
    /// full. `try_lock` rather than `lock`: a caller that finds a resync already running should
    /// proceed on the current estimate, not queue up behind it.
    fn resync_ack_buffer(&self) {
        let Some(mut last_run) = self.ack_buffer_resync.try_lock() else {
            return;
        };
        if last_run.is_some_and(|at| at.elapsed() < self.ack_buffer_resync_interval()) {
            return;
        }

        let held = self.count_ack_buffer_entries();
        let previous = self.ack_buffer_entries.swap(held, std::sync::atomic::Ordering::Relaxed);
        *last_run = Some(std::time::Instant::now());

        if previous != held {
            tracing::debug!(
                previous,
                held,
                "resynchronised the awaiting-acknowledgement buffer count"
            );
        }
    }

    /// Ground truth: the entries actually held across every peer.
    ///
    /// Never reads [`ack_buffer_entries`](Self::ack_buffer_entries), so a test asserting the two
    /// agree is testing the counter rather than agreeing with it — the same reason
    /// `deferred_ack_count` recomputes from `by_poly` instead of reading `DeferredAcks::total`.
    ///
    /// `O(max_tracked_peers)`, and each `run_pending_tasks` is bounded by that cache's pending write
    /// queue rather than its size. Both callers keep it off the steady-state path.
    fn count_ack_buffer_entries(&self) -> usize {
        self.awaiting_acks.run_pending_tasks();
        self.awaiting_acks
            .iter()
            .map(|(_, shares)| {
                shares.run_pending_tasks();
                shares.entry_count() as usize
            })
            .sum()
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
    ///
    /// Recomputed from `by_poly` rather than read off `DeferredAcks::total` deliberately: a
    /// counter that has drifted from the map is exactly what this should catch, and an accessor
    /// reading the counter would agree with it whatever it said.
    #[cfg(test)]
    fn deferred_ack_count(&self, ssa_id: &SsaId<S::Pseudonym>) -> usize {
        self.pending_acks
            .get(ssa_id)
            .map(|b| b.lock().by_poly.values().map(Vec::len).sum())
            .unwrap_or(0)
    }
}

/// Ownership of an Exit SSA commitment, released when dropped.
///
/// Registering an Exit commitment is the first fallible step of many: the request still has to be
/// encoded, sent, and answered. Every early return between here and the point where a permanent
/// owner takes over would otherwise strand the commitment in the reconstructor until its own
/// lifetime expired, and a stranded commitment is not inert — its `SsaId` is occupied, so a retry at
/// the same index is rejected as a duplicate.
///
/// Move-only by design: no `Clone`, no `Copy`, so there is exactly one release point. A success path
/// hands ownership on with [`disarm`](Self::disarm) rather than letting the guard fall out of scope.
///
/// Dropping releases the registration **without** retiring the SSA, so the same index can be
/// requested again — see `SsaReconstructor::release_abandoned_commitment` for why that
/// distinction is load-bearing.
#[must_use = "dropping the guard immediately releases the SSA it owns"]
pub struct SsaCommitmentGuard<S: PixSpec + Clone> {
    /// `None` once disarmed, which is the only state in which `Drop` does nothing.
    owned: Option<OwnedCommitment<S>>,
}

/// What an [`SsaCommitmentGuard`] needs to release its SSA: where it is registered, and which one.
type OwnedCommitment<S> = (std::sync::Arc<SsaReconstructor<S>>, SsaId<<S as PixSpec>::Pseudonym>);

/// A registered Exit commitment, paired with ownership of its lifetime.
type GuardedExitCommitment<S> = (PixGroup<S>, SsaCommitmentGuard<S>);

impl<S: PixSpec + Clone> std::fmt::Debug for SsaCommitmentGuard<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SsaCommitmentGuard")
            .field("ssa_id", &self.owned.as_ref().map(|(_, id)| id))
            .finish()
    }
}

impl<S: PixSpec + Clone> SsaCommitmentGuard<S> {
    /// The SSA this guard owns, or `None` if it has been disarmed.
    pub fn ssa_id(&self) -> Option<&SsaId<S::Pseudonym>> {
        self.owned.as_ref().map(|(_, id)| id)
    }

    /// Gives up ownership without retiring, returning the SSA that is now the caller's to release.
    ///
    /// Returns `None` if the guard was already disarmed.
    pub fn disarm(mut self) -> Option<SsaId<S::Pseudonym>> {
        self.owned.take().map(|(_, ssa_id)| ssa_id)
    }
}

impl<S: PixSpec + Clone> Drop for SsaCommitmentGuard<S> {
    fn drop(&mut self) {
        if let Some((reconstructor, ssa_id)) = self.owned.take() {
            reconstructor.release_abandoned_commitment(ssa_id);
        }
    }
}

impl<S: PixSpec + Clone> SsaReconstructor<S> {
    /// Releases a commitment that was registered but never taken over by an owner.
    ///
    /// Deliberately **not** [`retire_ssa`](ExitAcknowledgementShareProcessor::retire_ssa), which
    /// additionally writes the resurrection tombstone. The tombstone is permanent for that `SsaId`
    /// for as long as it is retained, and it takes effect at the moment a cycle is *published* — so a
    /// retry at the same index would re-register, accept the peer's commitments, publish a deposit
    /// address, and then be silently undone at completion. The peer funds an SSA that can never be
    /// reconstructed, and nothing on either side reports a failure.
    ///
    /// Same-index retry is not a corner case: the SSA index is advanced only after every fallible
    /// step of a request has succeeded, so a request that failed keeps its index by design and the
    /// next attempt reuses it.
    ///
    /// Escalates to a full retirement if a cycle did go live, which means the peer was asked and
    /// answered — and therefore that ownership should already have been transferred with
    /// [`disarm`](SsaCommitmentGuard::disarm). That branch is a caller error, and retiring is the
    /// safe response to it, because a live cycle is exactly what the tombstone exists to protect.
    fn release_abandoned_commitment(&self, ssa_id: SsaId<S::Pseudonym>) {
        if self.ssa_cycles.contains_key(&ssa_id) {
            tracing::warn!(%ssa_id, "abandoned ssa commitment was already live — retiring it");
            self.retire_ssa(ssa_id);
        } else {
            tracing::debug!(%ssa_id, "releasing ssa commitment abandoned by its owner");
            self.remove_cycle(ssa_id);
        }
    }

    /// [`new_exit_commitment`](ExitAcknowledgementShareProcessor::new_exit_commitment), with the
    /// registration owned by an [`SsaCommitmentGuard`].
    ///
    /// No guard is produced on failure, so a rejected duplicate never retires the registration that
    /// caused the rejection.
    pub fn new_guarded_exit_commitment(
        self: &std::sync::Arc<Self>,
        id: SsaId<S::Pseudonym>,
        polys_per_ssa: usize,
        shares_per_poly: usize,
    ) -> Result<GuardedExitCommitment<S>, PixError<S::Pseudonym>> {
        let exit_commitment = self.new_exit_commitment(id, polys_per_ssa, shares_per_poly)?;
        Ok((
            exit_commitment,
            SsaCommitmentGuard {
                owned: Some((self.clone(), id)),
            },
        ))
    }
}

impl<S: PixSpec> Drop for SsaReconstructor<S> {
    /// Reports terminal resolutions that were never collected.
    ///
    /// `ready_resolutions` is a hand-off the *commitment* path fills and
    /// only `acknowledge_shares` empties, so delivery waits on the next acknowledgement batch from
    /// any peer. That is the common case and not the guaranteed one: a Session whose final cycle
    /// recovers through the deferred-ack drain, and which then stops sending because the cycle it
    /// was funding is complete, leaves the last resolution sitting here.
    ///
    /// Retirement is not the deadline — a retired cycle's resolution stays collectable, since the
    /// buffer is global and its entries name their own `SsaId`. This is, and nothing here can
    /// deliver: the commitment path has no route to the upper layer, which is why these were parked
    /// rather than returned. So the most that can be done is to refuse to lose them quietly. A
    /// `RecoveredSsa` reported here is a deposit key the Exit held and never handed on.
    ///
    /// The real fix is for the reconstructor to push rather than be pulled, which needs a sink on
    /// its constructor; that is bundled with threading a real `SsaReconstructorConfig` through the
    /// three sites in `hopr-transport` that hard-code `::default()`.
    fn drop(&mut self) {
        if self.ready_resolutions_len.load(std::sync::atomic::Ordering::Acquire) == 0 {
            return;
        }
        for resolution in self.ready_resolutions.lock().drain(..) {
            tracing::error!(
                ?resolution,
                "pix resolution was never collected and is lost with the reconstructor"
            );
        }
    }
}

impl<S: PixSpec + Clone> ExitAcknowledgementShareProcessor<S> for SsaReconstructor<S> {
    type Error = PixError<S::Pseudonym>;

    fn has_pending_shares(&self, peer: &OffchainPublicKey) -> bool {
        // The parked-resolution check is not redundant with the per-peer one. Callers use this to
        // skip `acknowledge_shares` entirely, and that is the only thing which ever collects
        // `ready_resolutions` — a buffer the *commitment* path fills, holding terminal events up
        // to and including a recovered deposit key. It is global rather than per-peer and its
        // contents name their own `SsaId`, so any batch can correctly carry it out; gating it
        // behind the producing peer's `awaiting_acks` entry, which expires on its own timer,
        // would strand it for no reason.
        self.ready_resolutions_len.load(std::sync::atomic::Ordering::Acquire) > 0
            || self.awaiting_acks.contains_key(peer)
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
            let cycle = SsaCycle::new(ssa_id, ssa_builder, progress.new_verifiers)?;
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

    /// Buffers an encrypted share until its acknowledgement arrives, subject to the global byte
    /// budget.
    ///
    /// # The bound is global, and it is not the product of the two caps
    ///
    /// [`max_tracked_peers`](SsaReconstructorConfig::max_tracked_peers) and
    /// [`max_awaiting_acks`](SsaReconstructorConfig::max_awaiting_acks) bound one dimension each,
    /// and their product — 2 000 × 1 000 000 by default, some 800 GB — is neither reachable nor the
    /// right thing to bound. Not reachable, because an entry exists only for a share this node has
    /// already *sent*, so filling it would take 66 M packets/s of egress inside the default 30 s
    /// window. Not right, because the two guard **mutually exclusive** concentrations:
    /// `max_awaiting_acks` sizes one cache per peer and has to cover every Session returning through
    /// a single first-relayer, while `max_tracked_peers` covers traffic spread thin. Squeezing the
    /// product would push one of them below what its own case needs, and a `max_awaiting_acks` set
    /// too low does not save memory — it size-evicts shares before their acknowledgements arrive.
    ///
    /// So the real bound is [`max_ack_buffer_bytes`](SsaReconstructorConfig::max_ack_buffer_bytes),
    /// counted here across all peers at once. Validating a workload model instead would not do:
    /// a model has to assume a Session count and a packet rate, and this node enforces neither.
    ///
    /// # Behaviour at the ceiling
    ///
    /// The newest share is refused, rather than the oldest evicted: there is no cheap global
    /// "oldest" across per-peer caches, and the oldest is nearest its TTL anyway. Either way a full
    /// buffer means share loss — the packet is already on the wire and its acknowledgement will find
    /// nothing — which is the honest cost of a hard ceiling. [`PixError::AckBufferFull`] is
    /// deliberately not an expected error so the caller logs it.
    ///
    /// The check and the insertion are not atomic. Concurrent inserters can overshoot the ceiling by
    /// their own number, which is the right trade: a lock on this path would cost more than the few
    /// hundred kilobytes of overshoot it would prevent.
    fn insert_encrypted_share(
        &self,
        peer: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        tagged_enc_share: TaggedEncryptedPartialSsaShare<S>,
    ) -> Result<(), Self::Error> {
        if tagged_enc_share.partial_share.is_empty() {
            return Err(PixError::ShareIsEmpty);
        }

        // One relaxed load in the steady state. Only a buffer that has actually reached its ceiling
        // pays for the ground-truth pass, and even then at most one caller at a time does.
        if self.ack_buffer_entries.load(std::sync::atomic::Ordering::Relaxed) >= self.max_ack_buffer_entries {
            self.resync_ack_buffer();
            if self.ack_buffer_entries.load(std::sync::atomic::Ordering::Relaxed) >= self.max_ack_buffer_entries {
                tracing::error!(
                    %peer,
                    budget_bytes = self.cfg.max_ack_buffer_bytes,
                    "awaiting-acknowledgement buffer is full — dropping share"
                );
                return Err(PixError::AckBufferFull);
            }
        }

        // Incremented unconditionally, including when this replaces an entry under the same
        // challenge. The inner listener fires for `Replaced` too, so the pair nets to zero.
        self.ack_buffer_entries
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.awaiting_acks
            .get_with_by_ref(peer, || {
                // Inner cache keyed by HalfKeyChallenge — each entry gets its own TTL
                // so a late-arriving share gets the full max_ack_await_time window.
                let released = self.ack_buffer_entries.clone();
                moka::sync::CacheBuilder::new(self.cfg.max_awaiting_acks as u64)
                    .time_to_live(self.cfg.max_ack_await_time)
                    // Every way an entry can leave this cache — redeemed by its acknowledgement,
                    // expired, or size-evicted — arrives here, which is what keeps the global count
                    // falling without the removal sites having to know about it.
                    .eviction_listener(move |_, _, _| {
                        released.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    })
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

        // Three accumulators rather than one deduplicating set, because the emission contract is an
        // ordering as well as a multiplicity: every SSA's counters precede its terminal event.
        //
        // All three are `Vec`s deduplicated by linear scan. A per-peer batch resolves shares for the
        // cycle that peer is relaying for — one, or a small handful while a Session pipelines the
        // next — so the scan is over a one- or two-element list and a hash set would cost more than
        // it saves.
        let mut progress: Vec<SsaRecoveryProgress<S::Pseudonym>> = Vec::new();
        let mut faults: Vec<FaultObservation<S::Pseudonym>> = Vec::new();
        let mut terminal: Vec<ShareResolution<S::Pseudonym, S::AddressPrivateKey>> = Vec::new();

        // Collect anything redeemed while verifiers were being installed. No retry scanning happens
        // here: a deferred ack is retried exactly once, by the installation of the verifier it was
        // waiting for (see `drain_deferred_acks`).
        for resolution in self.take_ready_resolutions() {
            match resolution {
                ShareResolution::Progress(snapshot) => record_progress(&mut progress, snapshot),
                ShareResolution::InvalidShares {
                    peer,
                    ssa_id,
                    observed_total,
                } => record_fault(&mut faults, (peer, ssa_id, observed_total)),
                other => push_unique(&mut terminal, other),
            }
        }

        for (ack, ack_challenge) in half_keys_challenges {
            match self.process_verified_ack(ack, ack_challenge, &awaiting_ack_from_peer) {
                Ok(ProcessedAckResult::FullRecovery(ssa, snapshot)) => {
                    record_progress(&mut progress, snapshot);
                    push_unique(&mut terminal, ShareResolution::RecoveredSsa(ssa));
                }
                Ok(ProcessedAckResult::EarlyRecovery(snapshot)) => {
                    let ssa_id = snapshot.ssa_id;
                    record_progress(&mut progress, snapshot);
                    push_unique(&mut terminal, ShareResolution::AlmostRecoveredSsa(ssa_id));
                }
                Ok(ProcessedAckResult::Progressed(snapshot)) => record_progress(&mut progress, snapshot),
                Ok(ProcessedAckResult::InvalidShare(ssa_id, observed_total)) => {
                    tracing::error!(%ssa_id, observed_total, "encountered share that could not be verified");
                    record_fault(&mut faults, (Box::new(peer), ssa_id, observed_total));
                }
                Ok(ProcessedAckResult::NoProgress) => {}
                Ok(ProcessedAckResult::VerifierNotReady(spi)) => {
                    // The share stays in `awaiting_acks`; bucket the ack under the polynomial whose
                    // verifier it needs, so installing that verifier redeems it.
                    tracing::trace!(%peer, %spi, "verifier not yet installed, deferring acknowledgement");
                    self.defer_ack(spi, (peer, ack_challenge, ack));
                }
                Err(PixError::ShareIsEmpty) => tracing::trace!(%peer, "received empty share"),
                Err(error) => {
                    tracing::error!(%error, "failed to process acknowledgement");
                }
            }
        }

        let mut res = Vec::with_capacity(progress.len() + faults.len() + terminal.len());
        res.extend(progress.into_iter().map(ShareResolution::Progress));
        res.extend(
            faults
                .into_iter()
                .map(|(peer, ssa_id, observed_total)| ShareResolution::InvalidShares {
                    peer,
                    ssa_id,
                    observed_total,
                }),
        );
        res.append(&mut terminal);
        Ok(res)
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

    #[test]
    fn ssa_reconstructor_try_new_should_reject_an_invalid_config_without_panicking() {
        let cfg = SsaReconstructorConfig {
            // Outside the validated 0.0..=1.0 range.
            early_recovery_threshold: 1.5,
            ..Default::default()
        };

        assert!(matches!(
            SsaReconstructor::<TestSpec>::try_new(cfg),
            Err(PixError::InvalidConfiguration(_))
        ));
    }

    #[test]
    #[should_panic(expected = "invalid SsaReconstructorConfig")]
    fn ssa_reconstructor_new_should_still_panic_on_an_invalid_config() {
        let _ = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            early_recovery_threshold: 1.5,
            ..Default::default()
        });
    }

    /// A distinct, insertable share and the acknowledgement that redeems it.
    ///
    /// The contents do not matter to the budget — only that each entry has its own key — so this
    /// skips the generator entirely and builds a `PartialSsaShare::default()` under a fresh
    /// acknowledgement key.
    fn budget_share(
        spi: &SsaPolynomialId<SimplePseudonym>,
    ) -> anyhow::Result<(HalfKey, HalfKeyChallenge, TaggedEncryptedPartialSsaShare<TestSpec>)> {
        let ack_key = HalfKey::random();
        let challenge = ack_key.to_challenge()?;
        Ok((
            ack_key,
            challenge,
            TaggedEncryptedPartialSsaShare {
                pseudonym: *spi.pseudonym(),
                nonce: crypto_traits::elliptic_curve::Scalar::<Secp256k1>::random(&mut hopr_types::crypto_random::rng()),
                partial_share: PartialSsaShare::default().encrypt(spi, &ack_key)?,
            },
        ))
    }

    /// A reconstructor whose acknowledgement buffer holds exactly `entries`.
    fn reconstructor_with_ack_budget(entries: usize) -> SsaReconstructor<TestSpec> {
        SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            max_ack_buffer_bytes: entries * AWAITING_ACK_ENTRY_BYTES,
            ..Default::default()
        })
    }

    #[test]
    fn ack_buffer_refuses_shares_past_its_byte_budget() -> anyhow::Result<()> {
        // The validated minimum, so the budget is the smallest one a real config can ask for.
        const BUDGET: usize = 64;

        let reconstructor = reconstructor_with_ack_budget(BUDGET);
        let spi = SsaPolynomialId::new(SsaId::new(SimplePseudonym::random(), 1.try_into()?), 0);
        let peer = OffchainKeypair::random();

        for i in 0..BUDGET {
            let (_, challenge, share) = budget_share(&spi)?;
            reconstructor
                .insert_encrypted_share(peer.public(), challenge, share)
                .map_err(|error| anyhow::anyhow!("insertion {i} must fit the budget: {error}"))?;
        }

        let (_, challenge, share) = budget_share(&spi)?;
        assert!(
            matches!(
                reconstructor.insert_encrypted_share(peer.public(), challenge, share),
                Err(PixError::AckBufferFull)
            ),
            "the share past the budget must be refused rather than buffered"
        );
        assert_eq!(
            BUDGET,
            reconstructor.count_ack_buffer_entries(),
            "the buffer must hold exactly its budget — no more, and the refusal must not have dropped any"
        );

        Ok(())
    }

    /// Redeeming an acknowledgement must give its budget back.
    ///
    /// This is the release path the inner eviction listener exists for: `process_verified_ack`
    /// calls `remove`, which moka reports as an `Explicit` removal. A listener that only handled
    /// expiry and size eviction would leak here, and the buffer would fill permanently on a node
    /// that was working perfectly.
    #[test]
    fn redeeming_an_acknowledgement_returns_its_budget() -> anyhow::Result<()> {
        const BUDGET: usize = 64;

        let reconstructor = reconstructor_with_ack_budget(BUDGET);
        let ssa_id = SsaId::new(SimplePseudonym::random(), 1.try_into()?);
        let spi = SsaPolynomialId::new(ssa_id, 0);
        let peer = OffchainKeypair::random();
        reconstructor.new_exit_commitment(ssa_id, DEFAULT_POLYS_PER_SSA as usize, DEFAULT_POLY_THRESHOLD as usize)?;

        let mut redeemable = None;
        for _ in 0..BUDGET {
            let (ack_key, challenge, share) = budget_share(&spi)?;
            reconstructor.insert_encrypted_share(peer.public(), challenge, share)?;
            redeemable = Some((ack_key, challenge));
        }

        let (ack_key, challenge) = redeemable.expect("the loop ran at least once");
        let (_, refused, share) = budget_share(&spi)?;
        assert!(
            matches!(
                reconstructor.insert_encrypted_share(peer.public(), refused, share),
                Err(PixError::AckBufferFull)
            ),
            "precondition: the buffer is full"
        );

        // The cycle is not published, so this defers rather than redeeming — and deliberately
        // leaves the share in place. Budget must therefore *not* be returned yet.
        let peer_cache = reconstructor
            .awaiting_acks
            .get(peer.public())
            .expect("the peer holds shares");
        assert!(matches!(
            reconstructor.process_verified_ack(ack_key, challenge, &peer_cache),
            Ok(ProcessedAckResult::VerifierNotReady(_))
        ));
        assert_eq!(
            BUDGET,
            reconstructor.count_ack_buffer_entries(),
            "a deferral retains the share, so it must still be charged for"
        );

        // An outright removal is the redemption path's effect on the buffer.
        peer_cache.remove(&challenge);
        assert_eq!(
            BUDGET - 1,
            reconstructor.count_ack_buffer_entries(),
            "redeeming must free the entry"
        );
        reconstructor
            .insert_encrypted_share(peer.public(), refused, share)
            .map_err(|error| anyhow::anyhow!("the freed slot must be reusable: {error}"))?;

        Ok(())
    }

    /// Expiry must give budget back too — the release path that runs constantly in production.
    ///
    /// Most shares are never acknowledged in the window that matters; they age out. If the TTL did
    /// not release budget, a busy Exit would fill its buffer once and never accept another share.
    #[test]
    fn expiring_shares_return_their_budget() -> anyhow::Result<()> {
        const BUDGET: usize = 64;
        const WINDOW: std::time::Duration = std::time::Duration::from_millis(100);

        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            max_ack_buffer_bytes: BUDGET * AWAITING_ACK_ENTRY_BYTES,
            max_ack_await_time: WINDOW,
            ..Default::default()
        });
        let spi = SsaPolynomialId::new(SsaId::new(SimplePseudonym::random(), 1.try_into()?), 0);
        let peer = OffchainKeypair::random();

        for _ in 0..BUDGET {
            let (_, challenge, share) = budget_share(&spi)?;
            reconstructor.insert_encrypted_share(peer.public(), challenge, share)?;
        }
        let (_, challenge, share) = budget_share(&spi)?;
        assert!(
            matches!(
                reconstructor.insert_encrypted_share(peer.public(), challenge, share),
                Err(PixError::AckBufferFull)
            ),
            "precondition: the buffer is full"
        );

        std::thread::sleep(WINDOW * 2);

        // moka expires lazily, so the entries are still charged for until maintenance runs. The
        // insertion path reaches that through `resync_ack_buffer`, which is what this asserts: a
        // buffer full of expired shares must let the next one in without any caller intervening.
        let (_, challenge, share) = budget_share(&spi)?;
        reconstructor
            .insert_encrypted_share(peer.public(), challenge, share)
            .map_err(|error| anyhow::anyhow!("expired entries must free their budget: {error}"))?;

        assert_eq!(
            1,
            reconstructor.count_ack_buffer_entries(),
            "only the share inserted after the window should remain"
        );

        Ok(())
    }

    /// The counter is a hint; this asserts it does not become a lying one.
    ///
    /// The churn below deliberately includes the drift source no listener can catch — evicting a
    /// peer's whole entry from `awaiting_acks`, which drops its inner cache rather than draining
    /// it. Over-counting is the dangerous direction: it would eventually refuse every share while
    /// the buffer sat empty. `count_ack_buffer_entries` recomputes from the caches instead of
    /// reading the counter, so this compares the counter against the truth rather than against
    /// itself.
    #[test]
    fn the_ack_buffer_counter_does_not_inflate_as_peers_churn() -> anyhow::Result<()> {
        const PEERS: usize = 40;
        const TRACKED: usize = 10;
        const PER_PEER: usize = 5;

        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            // Well above anything inserted here: the point is peer churn, not the ceiling.
            max_ack_buffer_bytes: 4096 * AWAITING_ACK_ENTRY_BYTES,
            // Forces the outer cache to evict peers — and with them, whole inner caches — four
            // times over.
            max_tracked_peers: TRACKED,
            ..Default::default()
        });
        let spi = SsaPolynomialId::new(SsaId::new(SimplePseudonym::random(), 1.try_into()?), 0);

        for _ in 0..PEERS {
            let peer = OffchainKeypair::random();
            for _ in 0..PER_PEER {
                let (_, challenge, share) = budget_share(&spi)?;
                reconstructor.insert_encrypted_share(peer.public(), challenge, share)?;
            }
        }

        let held = reconstructor.count_ack_buffer_entries();
        assert!(
            held <= TRACKED * PER_PEER,
            "the outer cache bounds what can be held: {held} entries across at most {TRACKED} peers"
        );

        // A resync is what makes the counter authoritative again; without the backstop the residue
        // from dropped inner caches would sit on it permanently.
        reconstructor.resync_ack_buffer();
        assert_eq!(
            held,
            reconstructor
                .ack_buffer_entries
                .load(std::sync::atomic::Ordering::Relaxed),
            "the counter must agree with what the caches actually hold after churn"
        );

        Ok(())
    }

    /// The bound no longer depends on a workload model — which is the whole of M2.
    ///
    /// Both caps are set so their product is orders of magnitude past the budget, exactly the
    /// configuration the old modelled validation would have waved through. What is held is decided
    /// by the byte budget and nothing else: no Session count, no assumed packet rate, no
    /// acknowledgement window.
    #[test]
    fn the_ack_buffer_budget_binds_regardless_of_the_configured_caps() -> anyhow::Result<()> {
        const BUDGET: usize = 64;
        /// Enough peers that neither cap is anywhere near binding, few enough that the test is not
        /// dominated by keypair generation.
        const PEERS: usize = 16;

        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            max_ack_buffer_bytes: BUDGET * AWAITING_ACK_ENTRY_BYTES,
            max_tracked_peers: 2000,
            max_awaiting_acks: 1_000_000,
            // An hour, against a 30 s default: the dial the modelled check keyed on, turned to a
            // value it would have rejected outright.
            max_ack_await_time: std::time::Duration::from_secs(3600),
            ..Default::default()
        });
        let spi = SsaPolynomialId::new(SsaId::new(SimplePseudonym::random(), 1.try_into()?), 0);

        // Spread across peers so neither per-peer nor per-dimension cap is anywhere near binding.
        let peers = (0..PEERS).map(|_| OffchainKeypair::random()).collect::<Vec<_>>();
        let mut accepted = 0;
        for i in 0..(BUDGET * 4) {
            let (_, challenge, share) = budget_share(&spi)?;
            if reconstructor
                .insert_encrypted_share(peers[i % PEERS].public(), challenge, share)
                .is_ok()
            {
                accepted += 1;
            }
        }

        assert_eq!(
            BUDGET, accepted,
            "the byte budget must be what binds, not the caps whose product is 2e9 entries"
        );
        Ok(())
    }

    /// Pulls from the generator until it yields a share for `poly_index`, discarding the rest.
    ///
    /// Shares are emitted round-robin across [`crate::SHARE_EMISSION_WINDOW`] polynomials, so a test
    /// that needs to drive one polynomial to its threshold cannot assume consecutive calls stay on
    /// it. Discarding the others is sound here: these tests assert on one polynomial's behaviour,
    /// and a polynomial that never receives its shares simply stays incomplete.
    fn next_share_for_poly(
        generator: &SsaShareGenerator<TestSpec>,
        pseudonym: &SimplePseudonym,
        poly_index: PolynomialIndex,
    ) -> anyhow::Result<([u8; 20], crate::GeneratedShare<TestSpec>)> {
        loop {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let share = generator
                .next_share(pseudonym, &msg)?
                .ok_or_else(|| anyhow::anyhow!("generator must yield a share"))?;
            if share.id.poly_index() == poly_index {
                return Ok((msg, share));
            }
        }
    }

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
            matches!(resolution1, ProcessedAckResult::Progressed(p) if p.useful_shares == 1),
            "first share should count as progress without completing the SSA"
        );

        // --- Step 4: Process the duplicate ---
        // The SsaPartBuilder has 1/2 shares. The duplicate share has the same identifier
        // (same X-coordinate from msg1), so it hits the
        // `any(|s| s.identifier == share.identifier)` check in SsaPartBuilder::add_share.
        //
        // The point of the assertion is the contrast with step 3: a duplicate is not merely
        // "incomplete", it is *not progress*, so it must leave the counters where they were.
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
            "duplicate share must not register as progress"
        );
        assert_eq!(
            1,
            reconstructor
                .cycle(&ssa_id)
                .ok_or(anyhow::anyhow!("cycle went away"))?
                .progress()
                .useful_shares,
            "the duplicate must not have advanced the useful-share count"
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
            matches!(resolution2, ProcessedAckResult::FullRecovery(ref r, _) if r.ssa_id == ssa_id),
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

    /// Sets up one fully committed cycle and returns every share's acknowledgement, unprocessed.
    ///
    /// Shares are inserted but not acknowledged, so the caller can feed acks in whatever grouping
    /// the test needs and read the cycle's counters in between.
    #[allow(clippy::type_complexity)]
    fn cycle_with_pending_acks(
        polys: u16,
        threshold: u8,
        surplus: u8,
        peer: &OffchainKeypair,
    ) -> anyhow::Result<(SsaReconstructor<TestSpec>, SsaId<SimplePseudonym>, Vec<Acknowledgement>)> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: polys,
            threshold,
            surplus_shares: surplus,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::default();
        reconstructor.new_exit_commitment(ssa_id, polys as usize, threshold as usize)?;
        commitment_msg.process_into_reconstructor(&reconstructor)?;

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
            acks.push(VerifiedAcknowledgement::new(ack, peer).leak());
        }

        Ok((reconstructor, ssa_id, acks))
    }

    #[test]
    fn progress_counts_only_the_shares_that_advance_reconstruction() -> anyhow::Result<()> {
        // 2 polynomials × threshold 2, plus 1 surplus share each: 6 shares on the wire, of which
        // only 4 can ever be useful. The gap between those two numbers is the whole point — a
        // consumer sizing a progress ratio against packets received would read 6/4.
        const POLYS: u16 = 2;
        const THRESHOLD: u8 = 2;
        let peer = OffchainKeypair::random();
        let (reconstructor, ssa_id, acks) = cycle_with_pending_acks(POLYS, THRESHOLD, 1, &peer)?;
        assert_eq!(
            6,
            acks.len(),
            "generator should emit (threshold + surplus) per polynomial"
        );

        let mut snapshots = Vec::new();
        let mut recovered = false;
        // One ack at a time, so each share's individual contribution is observable.
        for ack in acks {
            for resolution in reconstructor.acknowledge_shares(*peer.public(), vec![ack])? {
                match resolution {
                    ShareResolution::Progress(p) => snapshots.push(p),
                    ShareResolution::RecoveredSsa(r) => {
                        assert_eq!(ssa_id, r.ssa_id);
                        recovered = true;
                    }
                    _ => {}
                }
            }
        }

        assert!(recovered, "the cycle must reconstruct from its own shares");

        // Shares are emitted polynomial-major, so the useful ones are shares 1,2 and 4,5 — the
        // third share of each polynomial arrives after that polynomial is already reconstructed.
        assert_eq!(
            vec![1, 2, 3, 4],
            snapshots.iter().map(|p| p.useful_shares).collect::<Vec<_>>(),
            "each snapshot must advance by exactly one, and surplus shares must emit none at all"
        );
        let last = snapshots.last().ok_or(anyhow::anyhow!("no progress emitted"))?;
        assert_eq!(
            (POLYS as u64 * THRESHOLD as u64),
            last.target_useful_shares,
            "target must be polynomials × threshold, matching the negotiated dimensions"
        );
        assert_eq!(
            last.useful_shares, last.target_useful_shares,
            "a completed cycle must report itself as complete"
        );
        assert_eq!(
            POLYS, last.recovered_polynomials,
            "every polynomial must be accounted for"
        );

        Ok(())
    }

    #[test]
    fn progress_precedes_the_terminal_event_it_belongs_to() -> anyhow::Result<()> {
        // The emission order is a contract: a consumer that acts on RecoveredSsa must already have
        // been told the counters that justify it, including for the batch that completes the cycle —
        // the one whose snapshot is taken from a cycle that no longer exists by the time the batch
        // returns.
        let peer = OffchainKeypair::random();
        let (reconstructor, ssa_id, acks) = cycle_with_pending_acks(2, 2, 0, &peer)?;

        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks)?;

        let first_terminal = resolutions
            .iter()
            .position(|r| {
                matches!(
                    r,
                    ShareResolution::RecoveredSsa(_) | ShareResolution::AlmostRecoveredSsa(_)
                )
            })
            .ok_or(anyhow::anyhow!("expected a terminal resolution"))?;
        let last_progress = resolutions
            .iter()
            .rposition(|r| matches!(r, ShareResolution::Progress(_)))
            .ok_or(anyhow::anyhow!("expected a progress resolution"))?;
        assert!(
            last_progress < first_terminal,
            "every Progress must precede every terminal event, got {resolutions:?}"
        );

        // Exactly one snapshot per SSA per batch, even though four shares moved the counters.
        let progress = resolutions
            .iter()
            .filter_map(|r| match r {
                ShareResolution::Progress(p) => Some(p),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(1, progress.len(), "one snapshot per SSA per batch, got {progress:?}");
        assert_eq!(ssa_id, progress[0].ssa_id);
        assert_eq!(
            4, progress[0].useful_shares,
            "the surviving snapshot must be the furthest-along one"
        );

        Ok(())
    }

    #[test]
    fn an_abandoned_commitment_guard_releases_its_registration() -> anyhow::Result<()> {
        let reconstructor = std::sync::Arc::new(SsaReconstructor::<TestSpec>::default());
        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);

        let (_commitment, guard) = reconstructor.new_guarded_exit_commitment(ssa_id, 2, 2)?;
        assert_eq!(Some(&ssa_id), guard.ssa_id());
        assert!(
            reconstructor.contains_builder(&ssa_id),
            "the commitment must be registered while the guard holds it"
        );

        drop(guard);
        assert!(
            !reconstructor.contains_builder(&ssa_id),
            "dropping the guard must release the registration it owned"
        );

        Ok(())
    }

    /// Abandoning a guard whose cycle already went live escalates to a full retirement.
    ///
    /// Reaching this is a caller error — a live cycle means the peer was asked and answered, so
    /// ownership should already have been handed on with `disarm()`. Retiring is the safe response
    /// rather than the merely tidy one: the tombstone is what stops a commitment completion racing
    /// the teardown from republishing the cycle that was just dismantled.
    #[test]
    fn abandoning_a_live_cycle_retires_it_rather_than_just_releasing_it() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u8 = 2;
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });

        let reconstructor = std::sync::Arc::new(SsaReconstructor::<TestSpec>::default());
        let (_commitment, guard) =
            reconstructor.new_guarded_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;

        // Take the cycle live under the guard, which is the state a correct caller would have
        // disarmed out of.
        generator
            .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?
            .process_into_reconstructor(reconstructor.as_ref())?;
        assert!(reconstructor.cycle(&ssa_id).is_some(), "the cycle must be live");

        drop(guard);
        assert!(
            reconstructor.cycle(&ssa_id).is_none(),
            "abandoning a live cycle must tear it down"
        );

        // A tombstone was written, so this SsaId is spent: a fresh cycle at the same index is
        // published and then withdrawn at completion. That is the retirement contract, and the
        // contrast with `an_ssa_index_stays_usable_after_its_request_is_abandoned` is the point.
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        // A fresh generator: the original has already spent this pseudonym's index, and the identity
        // of the replacement commitment is irrelevant to what is being tested.
        let replacement = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        })
        .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let mut final_state = None;
        for (coeff_idx, coeffs) in replacement.verifiers.clone() {
            let proof = (coeff_idx == crate::CONSTANT_TERM_COEFFICIENT).then_some(replacement.commitment_proof);
            final_state =
                Some(reconstructor.insert_coefficient_commitments(ssa_id, coeff_idx, proof, coeffs.into_iter())?);
        }
        assert!(
            !final_state
                .ok_or(anyhow::anyhow!("commitment carried no batches"))?
                .is_verifiable,
            "a retired SsaId must stay retired"
        );

        Ok(())
    }

    #[test]
    fn a_disarmed_commitment_guard_leaves_its_ssa_alone() -> anyhow::Result<()> {
        let reconstructor = std::sync::Arc::new(SsaReconstructor::<TestSpec>::default());
        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);

        let (_commitment, guard) = reconstructor.new_guarded_exit_commitment(ssa_id, 2, 2)?;
        assert_eq!(Some(ssa_id), guard.disarm());
        assert!(
            reconstructor.contains_builder(&ssa_id),
            "a disarmed guard must not retire the commitment it handed on"
        );

        Ok(())
    }

    /// Abandoning a request must leave its SSA index usable, because that index is what the next
    /// attempt will use — it is advanced only once every fallible step has succeeded.
    ///
    /// Releasing through the full retirement path instead writes the resurrection tombstone, and the
    /// resulting failure is the quietest one in the protocol: the retry accepts the peer's
    /// commitments and publishes a deposit address, then has its cycle undone at completion. The peer
    /// funds an SSA that can never be reconstructed and neither side reports anything wrong.
    #[test]
    fn an_ssa_index_stays_usable_after_its_request_is_abandoned() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u8 = 2;
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let reconstructor = std::sync::Arc::new(SsaReconstructor::<TestSpec>::default());

        // First attempt is abandoned before the peer is ever asked for commitments.
        let (_commitment, guard) =
            reconstructor.new_guarded_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        drop(guard);

        // Retry at the same index, which is what a failed request leaves behind.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let mut final_state = None;
        for (coeff_idx, coeffs) in commitment.verifiers.clone() {
            let proof = (coeff_idx == crate::CONSTANT_TERM_COEFFICIENT).then_some(commitment.commitment_proof);
            final_state =
                Some(reconstructor.insert_coefficient_commitments(ssa_id, coeff_idx, proof, coeffs.into_iter())?);
        }
        let state = final_state.ok_or(anyhow::anyhow!("commitment carried no batches"))?;

        assert!(
            state.is_verifiable,
            "the retry must become verifiable — otherwise the peer is asked to fund a cycle that cannot reconstruct"
        );
        assert!(state.ssa_deposit_address.is_some(), "a verifiable cycle has an address");
        assert!(
            reconstructor.cycle(&ssa_id).is_some(),
            "the retry's cycle must be live, not published and then withdrawn"
        );

        Ok(())
    }

    #[test]
    fn a_rejected_duplicate_registration_does_not_retire_the_original() -> anyhow::Result<()> {
        // The failure path must not produce a guard: if it did, dropping the error's guard would
        // retire the very registration whose presence caused the rejection — turning a harmless
        // duplicate request into the loss of a live cycle.
        let reconstructor = std::sync::Arc::new(SsaReconstructor::<TestSpec>::default());
        let ssa_id = SsaId::new(SimplePseudonym::random(), SsaIndex::MIN);

        let (_commitment, guard) = reconstructor.new_guarded_exit_commitment(ssa_id, 2, 2)?;

        assert!(
            matches!(
                reconstructor.new_guarded_exit_commitment(ssa_id, 2, 2),
                Err(PixError::DuplicateCommitment)
            ),
            "a second registration at the same index must be rejected"
        );
        assert!(
            reconstructor.contains_builder(&ssa_id),
            "the rejected duplicate must leave the original registration intact"
        );

        drop(guard);
        Ok(())
    }

    /// **L2 regression.** A polynomial index repeated inside one batch must be rejected, not merely
    /// one repeated across batches.
    ///
    /// The two-phase check tested each entry against `committed_polynomials`, which holds only what
    /// earlier calls inserted, so two entries sharing an index both found the slot vacant. The
    /// second insert then rebound the first — the single-assignment invariant the two phases exist
    /// to enforce — and `total_committed` counted two occupants of one slot. The practical bite:
    /// a batch carrying every polynomial with a repeat among them can never complete the set, and
    /// the peer has no way to supply the one it displaced, because every retry is rejected as a
    /// duplicate against the slots the batch did fill.
    ///
    /// The wire decoder rejects intra-message duplicates today. This builder is not meant to depend
    /// on that, which is why the check is here.
    #[test]
    fn a_polynomial_repeated_within_one_batch_is_rejected() -> anyhow::Result<()> {
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

        // Polynomial 0 twice, and polynomial 1 not at all — the shape that would otherwise leave the
        // set permanently one short while reporting two commitments received.
        let mut batch = coefficient_of(&commitment, 0, Some(0))?;
        batch.push(batch[0]);
        let result =
            reconstructor.insert_coefficient_commitments(ssa_id, 0, proof_of(&commitment, 0), batch.into_iter());
        assert!(
            matches!(&result, Err(crate::errors::PixError::DuplicateCommitment)),
            "a repeat inside the batch must be rejected, got {result:?}"
        );

        // Rejected transactionally: neither entry was written, so the honest batch still lands.
        let retry = reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, None)?.into_iter(),
        )?;
        assert!(
            retry.is_verifiable && retry.ssa_deposit_address.is_some(),
            "the corrected batch must complete the commitment"
        );

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
    ///
    /// What this test covers is the *arrival point*, not the subgroup filter itself: `TestSpec` is
    /// secp256k1, cofactor 1, so no small-order point exists to feed it. The subgroup case is
    /// `pix_group_element_rejects_a_small_order_point` in `hopr-crypto-packet`, which also records
    /// why the filter cannot be isolated by any test — the Baby JubJub backend's own `from_bytes`
    /// already rejects, so `is_torsion_free` is defence in depth rather than the acting check.
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

        // Feed exactly enough shares to reconstruct polynomial 0, skipping the ones the round-robin
        // emission hands out for polynomial 1 in between.
        for _ in 0..2 {
            let (msg, share) = next_share_for_poly(&generator, &pseudonym, 0)?;

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

    /// A failed interpolation must leave the part terminal, exactly as a failed commitment opening
    /// does.
    ///
    /// The two paths were asymmetric: the commitment mismatch released the share buffer and set
    /// `failed`, while a `combine()` error propagated with `?` and did neither. That left the part
    /// holding a full share set with no terminal flag, so neither early return in `add_share`
    /// fired and every remaining share for the polynomial was pushed and re-ran the interpolation
    /// over a larger set — `O(threshold²)` per share, against a buffer that should already have
    /// been released, re-reporting the same fault each time.
    ///
    /// The interpolation is forced to fail here by giving the builder a one-share threshold, which
    /// `vsss_rs` rejects outright. How the failure arises is not the property under test; that the
    /// part is left terminal either way is.
    #[test]
    fn a_failed_interpolation_leaves_the_part_terminal() -> anyhow::Result<()> {
        use utils::{AddShareOutcome, SsaPartBuilder};

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 1,
            threshold: 2,
            surplus_shares: 2,
        });
        let pseudonym = SimplePseudonym::random();
        let spi = SsaPolynomialId::new(SsaId::new(pseudonym, SsaIndex::MIN), 0);
        generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        // The commitment is never reached — `verify_reconstructed` runs only on a value that
        // interpolated — so the generator is a stand-in for any well-formed constant term.
        let mut part = SsaPartBuilder::<TestSpec>::new(
            crate::SsaPartCommitment::from_decoded_commitment(spi, PixGroup::<TestSpec>::generator()),
            1,
        );

        let mut rng = hopr_types::crypto_random::rng();
        let (_, share) = next_share_for_poly(&generator, &pseudonym, 0)?;
        assert!(
            part.add_share(PixScalar::<TestSpec>::random(&mut rng), share.share)
                .is_err(),
            "one share is below what `combine` accepts, so the interpolation must fail"
        );
        assert_eq!(
            0,
            part.verification_state_len(),
            "a part that can never reconstruct must release its share buffer"
        );

        // Every later share is absorbed instead of re-running the interpolation and re-reporting.
        let (_, share) = next_share_for_poly(&generator, &pseudonym, 0)?;
        assert!(
            matches!(
                part.add_share(PixScalar::<TestSpec>::random(&mut rng), share.share)?,
                AddShareOutcome::Absorbed
            ),
            "a failed part must absorb later shares silently"
        );
        assert_eq!(0, part.verification_state_len());

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
                bucket.by_poly.get(&0).map(Vec::len).unwrap_or(0),
                "one sub-bucket holds both peers' acks"
            );
            assert_eq!(
                1,
                bucket.by_poly.get(&1).map(Vec::len).unwrap_or(0),
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
        const THRESHOLD: u8 = 4;

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
        const THRESHOLD: u8 = 2;

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
        const THRESHOLD: u8 = 4;
        const SURPLUS: u8 = 2;

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
        let mut invalid_reports: Vec<u64> = Vec::new();
        for i in 0..THRESHOLD as usize + SURPLUS as usize {
            let (msg, mut share) = next_share_for_poly(&generator, &pseudonym, 0)?;

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
                .filter_map(|r| match r {
                    ShareResolution::InvalidShares {
                        ssa_id: id,
                        observed_total,
                        ..
                    } if *id == ssa_id => Some(*observed_total),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if i + 1 < THRESHOLD as usize {
                assert!(
                    reported.is_empty(),
                    "share {i} must pass unremarked — nothing checks it yet"
                );
            }
            invalid_reports.extend(reported);
        }

        assert_eq!(
            vec![1],
            invalid_reports,
            "the corrupted set must be reported exactly once, at the threshold-th share, as the cycle's first fault"
        );

        Ok(())
    }

    /// The fault total is a property of the cycle, not of whoever relayed the offending share.
    ///
    /// A Session's shares reach the Exit through whichever relayer the return path happens to use,
    /// so a per-peer total would let an Entry stay under any limit by spreading its bad shares —
    /// and would make the number the consumer enforces on depend on routing rather than on conduct.
    #[test]
    fn fault_totals_aggregate_over_the_cycle_not_the_peer() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u8 = 2;

        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        // One relayer per polynomial, which is what shares arriving over different return paths
        // looks like from here.
        let relayers = [OffchainKeypair::random(), OffchainKeypair::random()];

        let reconstructor = SsaReconstructor::<TestSpec>::default();
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;
        generator
            .new_ssa_commitment(&pseudonym, SsaIndex::MIN)?
            .process_into_reconstructor(&reconstructor)?;

        // Emission is round-robin across the window, and with no surplus every share matters — so
        // the cycle's whole budget is drawn first and grouped, rather than filtered as it comes.
        let mut by_poly: std::collections::BTreeMap<PolynomialIndex, Vec<_>> = Default::default();
        for _ in 0..POLYS as usize * THRESHOLD as usize {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let share = generator
                .next_share(&pseudonym, &msg)?
                .ok_or_else(|| anyhow::anyhow!("generator must yield a share"))?;
            by_poly.entry(share.id.poly_index()).or_default().push((msg, share));
        }

        let mut totals = Vec::new();
        for poly in 0..POLYS {
            let relayer = &relayers[poly as usize];
            let shares = by_poly
                .remove(&poly)
                .ok_or_else(|| anyhow::anyhow!("no shares for polynomial {poly}"))?;
            assert_eq!(THRESHOLD as usize, shares.len());

            for (i, (msg, mut share)) in shares.into_iter().enumerate() {
                // Corrupt one share of each polynomial, so each one fails on its own threshold-th.
                if i == 0 {
                    AsMut::<[u8]>::as_mut(&mut share.share.0)[31] ^= 1;
                }

                let ack = HalfKey::random();
                let enc = share.share.clone().encrypt(&share.id, &ack)?;
                reconstructor.insert_encrypted_share(
                    relayer.public(),
                    ack.to_challenge()?,
                    TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
                )?;
                totals.extend(
                    reconstructor
                        .acknowledge_shares(
                            *relayer.public(),
                            vec![VerifiedAcknowledgement::new(ack, relayer).leak()],
                        )?
                        .into_iter()
                        .filter_map(|r| match r {
                            ShareResolution::InvalidShares { observed_total, .. } => Some(observed_total),
                            _ => None,
                        }),
                );
            }
        }

        assert_eq!(
            vec![1, 2],
            totals,
            "the second relayer's fault must be charged to the cycle's running total, not restart it"
        );
        assert_eq!(
            2,
            reconstructor
                .cycle(&ssa_id)
                .ok_or(anyhow::anyhow!("cycle went away"))?
                .invalid_shares(),
            "the cycle must hold the aggregate"
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
        const THRESHOLD: u8 = 2;

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
            (POLYS as usize * THRESHOLD as usize),
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

    /// A fault found while draining deferred acknowledgements must stay attributed to the relayer
    /// that carried the offending share.
    ///
    /// The drain runs on the commitment path, so its findings are handed to whichever
    /// `acknowledge_shares` call happens to collect them next — routinely a different peer's. Filling
    /// the relayer in at emission time would name that unrelated peer, which is worse than naming
    /// none: the field exists to attribute misbehaviour.
    #[test]
    fn a_fault_redeemed_from_deferral_keeps_its_own_relayer() -> anyhow::Result<()> {
        const POLYS: u16 = 1;
        const THRESHOLD: u8 = 2;

        // One surplus share, kept back to give `collector` an `awaiting_acks` entry.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 1,
        });
        let pseudonym = SimplePseudonym::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        // `carrier` relays the shares; `collector` is an unrelated peer whose later batch picks up
        // the parked resolutions.
        let carrier = OffchainKeypair::random();
        let collector = OffchainKeypair::random();

        let reconstructor = SsaReconstructor::<TestSpec>::default();
        reconstructor.new_exit_commitment(ssa_id, POLYS as usize, THRESHOLD as usize)?;

        // Shares arrive over `carrier` before any commitment, so they defer. One is corrupted, so the
        // drain is what discovers the fault.
        let mut acks = Vec::new();
        for i in 0..THRESHOLD {
            let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
            let mut share = generator
                .next_share(&pseudonym, &msg)?
                .ok_or_else(|| anyhow::anyhow!("generator must yield a share"))?;
            if i == 0 {
                AsMut::<[u8]>::as_mut(&mut share.share.0)[31] ^= 1;
            }
            let ack = HalfKey::random();
            let enc = share.share.clone().encrypt(&share.id, &ack)?;
            reconstructor.insert_encrypted_share(
                carrier.public(),
                ack.to_challenge()?,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
            )?;
            acks.push(VerifiedAcknowledgement::new(ack, &carrier).leak());
        }
        assert!(
            reconstructor.acknowledge_shares(*carrier.public(), acks)?.is_empty(),
            "shares must defer while the commitment is unknown"
        );

        // Completing the commitment installs the part builder and drains the bucket, which is where
        // the corrupted set is detected.
        reconstructor.insert_coefficient_commitments(
            ssa_id,
            0,
            proof_of(&commitment, 0),
            coefficient_of(&commitment, 0, None)?.into_iter(),
        )?;

        // The surplus share gives `collector` an entry in `awaiting_acks`, which is what makes its
        // batch acceptable. It is never acknowledged — the batch below is empty, so the only thing
        // `collector` contributes is being the wrong answer for the `peer` field.
        let msg: [u8; 20] = hopr_types::crypto_random::random_bytes();
        let surplus = generator
            .next_share(&pseudonym, &msg)?
            .ok_or_else(|| anyhow::anyhow!("generator must yield the surplus share"))?;
        let filler = HalfKey::random();
        reconstructor.insert_encrypted_share(
            collector.public(),
            filler.to_challenge()?,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, surplus.share.encrypt(&surplus.id, &filler)?)?,
        )?;

        let faults = reconstructor
            .acknowledge_shares(*collector.public(), Vec::new())?
            .into_iter()
            .filter_map(|r| match r {
                ShareResolution::InvalidShares { peer, ssa_id, .. } => Some((peer, ssa_id)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(1, faults.len(), "the drained fault must surface exactly once");
        assert_eq!(ssa_id, faults[0].1);
        assert_eq!(
            carrier.public(),
            faults[0].0.as_ref(),
            "the fault must name the relayer that carried the share, not the peer that collected it"
        );

        Ok(())
    }

    /// Deferring is decided on a cycle lookup that missed, so a cycle installing concurrently would
    /// leave the ack in a bucket whose one and only drain has already run — a share silently lost
    /// to a microsecond-wide window. `defer_ack` re-probes and drains itself in that case.
    #[test]
    fn deferring_against_an_installed_verifier_drains_immediately() -> anyhow::Result<()> {
        const POLYS: u16 = 2;
        const THRESHOLD: u8 = 2;

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

    /// An acknowledgement appended to a bucket the drain has already taken must still be redeemed.
    ///
    /// A bucket is reachable two ways — through the `pending_acks` key, and through an `Arc` a
    /// `defer_ack` obtained before the drain invalidated that key. The drain can only remove the
    /// first. The `ssa_cycles` re-probe was supposed to cover the resulting window, but it called
    /// `drain_deferred_acks`, which returns early on a cache miss — and a miss is exactly what this
    /// interleaving produces. The append landed in an orphaned bucket, and the share sat in
    /// `awaiting_acks` until `max_ack_await_time` discarded it. Silently: no error, no counter, and
    /// a polynomial losing more than `surplus_shares` this way strands the whole cycle without any
    /// check ever failing.
    ///
    /// Forcing the interleaving needs the stale handle, which is why `defer_ack_into` takes the
    /// bucket: after the invalidate, `defer_ack`'s own `get_with` would hand out a fresh one and
    /// the window would close by accident.
    #[test]
    fn an_acknowledgement_deferred_into_a_drained_bucket_is_still_redeemed() -> anyhow::Result<()> {
        // One polynomial at threshold 2, so the two deferred acknowledgements below are exactly
        // what the cycle needs: the second one landing makes the difference between a recovered
        // SSA and a stranded one.
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 1,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig::default());
        reconstructor.new_exit_commitment(ssa_id, 1, 2)?;

        // Both shares reach the Exit ahead of the commitment, so neither has a verifier yet.
        let mut pending = Vec::new();
        for _ in 0..2 {
            let (msg, share) = next_share_for_poly(&generator, &pseudonym, 0)?;
            let ack = HalfKey::random();
            let challenge = ack.to_challenge()?;
            let enc = share.share.encrypt(&share.id, &ack)?;
            reconstructor.insert_encrypted_share(
                peer.public(),
                challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
            )?;
            pending.push((share.id, challenge, ack));
        }

        // The first acknowledgement takes the ordinary deferral path and creates the bucket.
        let (_, _, first_ack) = pending[0];
        reconstructor.acknowledge_shares(
            *peer.public(),
            vec![VerifiedAcknowledgement::new(first_ack, &peer).leak()],
        )?;
        let bucket = reconstructor
            .pending_acks
            .get(&ssa_id)
            .ok_or_else(|| anyhow::anyhow!("the first acknowledgement must have created a bucket"))?;

        // Installing the cycle drains that bucket and invalidates its key. Our handle survives.
        commitment.process_into_reconstructor(&reconstructor)?;
        assert!(
            reconstructor.pending_acks.get(&ssa_id).is_none(),
            "the drain must have taken the cache key"
        );

        // The racing append: a `defer_ack` that looked the bucket up before the drain ran.
        let (spi, challenge, ack) = pending[1];
        reconstructor.defer_ack_into(&bucket, spi, (*peer.public(), challenge, ack));

        // Two shares interpolate the only polynomial, so the SSA is recovered — but only if the
        // second acknowledgement was redeemed rather than parked in the orphan.
        assert!(
            reconstructor
                .take_ready_resolutions()
                .iter()
                .any(|resolution| matches!(resolution, ShareResolution::RecoveredSsa(_))),
            "the acknowledgement must have been redeemed inline, not lost with the bucket"
        );

        Ok(())
    }

    /// Drives a cycle to full recovery entirely through the deferral path, so the `RecoveredSsa` it
    /// produces ends up parked in `ready_resolutions` rather than returned from `acknowledge_shares`.
    ///
    /// Both shares arrive before the commitment does, which is the ordering the emission window
    /// makes routine near a cycle boundary, so both acknowledgements defer and the drain that
    /// installs the cycle is what reconstructs.
    fn park_a_recovered_ssa() -> anyhow::Result<(SsaReconstructor<TestSpec>, SsaId<SimplePseudonym>)> {
        let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 1,
            threshold: 2,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let peer = OffchainKeypair::random();
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);

        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig::default());
        reconstructor.new_exit_commitment(ssa_id, 1, 2)?;

        for _ in 0..2 {
            let (msg, share) = next_share_for_poly(&generator, &pseudonym, 0)?;
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

        commitment.process_into_reconstructor(&reconstructor)?;
        assert_ne!(
            0,
            reconstructor
                .ready_resolutions_len
                .load(std::sync::atomic::Ordering::Acquire),
            "the drain must have parked the recovery"
        );

        Ok((reconstructor, ssa_id))
    }

    /// A parked resolution must not be gated behind the peer whose shares produced it.
    ///
    /// The pipelines call `has_pending_shares` to decide whether to hand a batch to
    /// `acknowledge_shares` at all, and `acknowledge_shares` is the only thing that ever collects
    /// `ready_resolutions`. Answering purely from `awaiting_acks` therefore made delivery of a
    /// recovered deposit key depend on the producing peer sending more traffic before its own cache
    /// entry idled out — while the buffer is global, and any batch could have carried it.
    #[test]
    fn a_parked_resolution_is_collectable_through_any_peer() -> anyhow::Result<()> {
        let (reconstructor, _) = park_a_recovered_ssa()?;

        // A peer this reconstructor has never seen: no shares, no `awaiting_acks` entry.
        let bystander = *OffchainKeypair::random().public();
        assert!(
            !reconstructor.awaiting_acks.contains_key(&bystander),
            "the bystander must have no pending shares of its own"
        );
        assert!(
            reconstructor.has_pending_shares(&bystander),
            "a parked resolution must let any batch through to collect it"
        );

        // And once collected, the guard goes back to answering per-peer.
        assert!(
            reconstructor
                .take_ready_resolutions()
                .iter()
                .any(|resolution| matches!(resolution, ShareResolution::RecoveredSsa(_)))
        );
        assert!(
            !reconstructor.has_pending_shares(&bystander),
            "with nothing parked the guard must not admit an unrelated peer"
        );

        Ok(())
    }

    /// Retiring a cycle must not consume the resolution it already produced.
    ///
    /// `ready_resolutions` is global and its entries name their own `SsaId`, so a parked
    /// `RecoveredSsa` stays collectable after its cycle is torn down — and it is worth collecting:
    /// the deposit key is what pays the Exit, whether or not the Session that earned it is still
    /// open. Draining at retirement would destroy that, and would take unrelated cycles'
    /// resolutions with it, since nothing in the buffer is keyed by cycle. The only point at which
    /// delivery genuinely becomes impossible is `Drop`, which reports whatever is left.
    #[test]
    fn retiring_a_cycle_leaves_its_resolution_collectable() -> anyhow::Result<()> {
        let (reconstructor, ssa_id) = park_a_recovered_ssa()?;

        reconstructor.retire_ssa(ssa_id);

        assert!(
            reconstructor
                .take_ready_resolutions()
                .iter()
                .any(|resolution| matches!(resolution, ShareResolution::RecoveredSsa(_))),
            "retirement must not swallow a recovered deposit key"
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
        const THRESHOLD: u8 = 4;
        /// Bounded on **both** sides, which is why this is not simply "as large as possible":
        ///
        /// * above `SHARE_SPACING`, or the cycle idles out between two consecutive shares and the test fails for a
        ///   reason unrelated to H8. Each iteration does rather more than sleep — polynomial evaluation, encryption,
        ///   insertion, and at the threshold a Lagrange combine and a scalar multiplication — so on a contended runner
        ///   the margin needs to be several times the sleep, not the 2× it used to be;
        /// * below `(THRESHOLD + 1) × SHARE_SPACING` ≈ 1250 ms, or the assertion below stops holding and the test
        ///   exercises nothing.
        ///
        /// 1000 ms sits inside that window with the slack on the side that grows under load:
        /// cumulative elapsed time only ever overshoots, while a single iteration would have to
        /// take four times its sleep to expire the cycle.
        const VERIFIER_LIFETIME: std::time::Duration = std::time::Duration::from_millis(1000);
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
        for i in 0..(POLYS as usize * THRESHOLD as usize) {
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
        const THRESHOLD: u8 = 2;
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
        const THRESHOLD: u8 = 2;

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
        const THRESHOLD: u8 = 2;

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
