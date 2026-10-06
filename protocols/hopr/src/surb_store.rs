use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use hopr_api::types::internal::{prelude::HoprPseudonym, routing::SurbMatcher};
use hopr_crypto_packet::prelude::*;
use hopr_protocol_pix::SsaIndex;
use moka::notification::RemovalCause;
use validator::ValidationError;

use crate::{FoundSurb, SurbInsertOutcome, traits::SurbStore};

/// Lower bound on [`SurbStoreConfig::pseudonyms_lifetime`], enforced by the config validator.
///
/// Public so that callers applying their own override can floor it identically, rather than
/// reaching a value the config file itself would have been rejected for.
pub const MINIMUM_SURB_LIFETIME: Duration = Duration::from_secs(30);
/// Lower bound on [`SurbStoreConfig::eviction_report_interval`], enforced by the config validator.
pub const MINIMUM_EVICTION_REPORT_INTERVAL: Duration = Duration::from_secs(1);
const MINIMUM_OPENER_PSEUDONYMS: usize = 1000;
const MINIMUM_OPENERS_PER_PSEUDONYM: usize = 1000;
const MINIMUM_SURBS_PER_PSEUDONYM: usize = 1000;
const MINIMUM_OPENER_LIFETIME: Duration = Duration::from_secs(60);
const MIN_SURB_RB_CAPACITY: usize = 1024;

fn validate_pseudonyms_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_SURB_LIFETIME {
        Err(ValidationError::new("pseudonyms_lifetime is too low"))
    } else {
        Ok(())
    }
}

fn validate_reply_opener_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_OPENER_LIFETIME {
        Err(ValidationError::new("reply_opener_lifetime is too low"))
    } else {
        Ok(())
    }
}

fn validate_eviction_report_interval(interval: &Duration) -> Result<(), ValidationError> {
    if interval < &MINIMUM_EVICTION_REPORT_INTERVAL {
        Err(ValidationError::new("eviction_report_interval is too low"))
    } else {
        Ok(())
    }
}

fn default_rb_capacity() -> usize {
    100_000
}

fn default_distress_threshold() -> usize {
    500
}

fn default_max_openers_per_pseudonym() -> usize {
    100_000
}

fn default_max_pseudonyms() -> usize {
    10_000
}

fn default_pseudonyms_lifetime() -> Duration {
    Duration::from_secs(600)
}

fn default_reply_opener_lifetime() -> Duration {
    Duration::from_secs(3600)
}

fn default_eviction_report_interval() -> Duration {
    Duration::from_secs(60)
}

fn default_eviction_report_threshold() -> u64 {
    1000
}

/// Which end of the per-pseudonym buffer a pop consumes from. Replying side only.
///
/// FIFO, oldest first, is the only order. SURBs that carry a PIX share are always consumed before
/// share-less ones (see "Share-bearing SURBs first" on the internal `SurbRingBuffer`), and within each
/// of those two tiers the oldest goes first, whether it is popped or evicted on overflow. The order is
/// independent of the per-SURB generation tag (`SurbReceiverInfo::generation`), which handles an
/// explicit return-path re-plan by clearing superseded SURBs wholesale on the next push.
///
/// There used to be a `Lifo` variant (hoprnet#8328), consuming newest first so that a return-path
/// change would apply before the buffered SURBs drained. It was removed because PIX cannot work with
/// it: a share reaches the reconstructor only when its SURB is *used*, so a LIFO store buries the
/// oldest shares of the current cycle, and an overflow then evicts exactly those, which loses them for
/// good. Every node may act as a PIX Exit and nothing in the configuration turns that off
/// (`enforce_pix` only makes it mandatory), so the option had no safe use — and what it was for is
/// what the generation tag does.
///
/// This enum, and the [`SurbStoreConfig::pop_order`] field carrying it, remain only so that existing
/// configurations (`pop_order: fifo`) and code naming [`SurbPopOrder::Fifo`] keep working; they are
/// candidates for removal in a later major version. `pop_order: lifo` is rejected as an unknown
/// variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SurbPopOrder {
    /// Oldest first. The only order, and the default.
    #[default]
    Fifo,
}

/// Configuration for the SURB cache.
///
/// The configuration options affect both the sending side (SURB creator) and the
/// replying side (SURB consumer).
///
/// In the classical scenario (`Entry - Relay 1 -... - Exit`), the sending side is
/// the `Entry` and the replying side is the `Exit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
pub struct SurbStoreConfig {
    /// Size of the SURB ring buffer per pseudonym.
    ///
    /// Affects only the replying side.
    ///
    /// This indicates how many SURBs can be at most held to be used to send a reply
    /// back to the sending side.
    ///
    /// Once the buffer is full, a push evicts share-less SURBs first (oldest first); only when none
    /// remain does it evict the oldest share-bearing SURB. An evicted SURB is never used. With PIX,
    /// evicting one that carries a share is not merely a wasted SURB: a partial SSA share is only
    /// delivered to the reconstructor when its SURB is *used*, so the share is lost for good. The
    /// capacity is therefore sized well above what any Session's SURB balancer targets — see
    /// `maximum_surb_buffer_size` in `hopr-transport`, which is derived from this value with
    /// headroom left for balancer overshoot.
    ///
    /// This is a ceiling rather than a reservation: the internal `SurbRingBuffer` allocates with
    /// occupancy, so a pseudonym holding three SURBs costs three, and only one that genuinely fills
    /// up pays `rb_capacity` × ~400 B. That property is what makes a large default safe here — see
    /// the "Why the capacity is a ceiling, not a reservation" section on `SurbRingBuffer`.
    ///
    /// Default is 100 000.
    // `SurbRingBuffer` is deliberately unlinked above: it is crate-private, so an intra-doc link
    // from this public field trips `rustdoc::private_intra_doc_links`, which CI builds as an error.
    #[default(default_rb_capacity())]
    #[validate(range(min = 1024, message = "rb_capacity must be at least 1024"))]
    #[cfg_attr(feature = "serde", serde(default = "default_rb_capacity"))]
    pub rb_capacity: usize,
    /// Pop order of the per-pseudonym buffer; see [`SurbPopOrder`].
    ///
    /// FIFO is the only order, so this setting changes nothing. It is retained for configuration
    /// compatibility — a file that spells out `pop_order: fifo` keeps loading — and is a candidate for
    /// removal in a later major version. `pop_order: lifo` is no longer accepted and fails to load as an
    /// unknown variant: PIX cannot work with LIFO, see [`SurbPopOrder`] for why.
    ///
    /// Affects only the replying side. Default is [`SurbPopOrder::Fifo`].
    #[cfg_attr(feature = "serde", serde(default))]
    pub pop_order: SurbPopOrder,
    /// Threshold for the number of SURBs in the ring buffer, below which it is
    /// considered low ("SURB distress").
    ///
    /// Default is 500.
    #[default(default_distress_threshold())]
    #[validate(range(min = 10, message = "distress_threshold must be at least 10"))]
    #[cfg_attr(feature = "serde", serde(default = "default_distress_threshold"))]
    pub distress_threshold: usize,
    /// Maximum number of reply openers (SURB counterparts) per pseudonym.
    ///
    /// Affects only the sending side when decrypting a received reply.
    ///
    /// This mostly affects Sessions, as they use a fixed pseudonym.
    /// It reflects how many reply openers the initiator-side of a Session can hold,
    /// until the oldest ones are dropped. If the other party uses a SURB corresponding
    /// to a dropped reply opener, the reply message will be undecryptable by the initiator-side.
    ///
    /// Default is 100 000.
    #[default(default_max_openers_per_pseudonym())]
    #[validate(range(min = 100, message = "max_openers_per_pseudonym must be at least 100"))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_openers_per_pseudonym"))]
    pub max_openers_per_pseudonym: usize,
    /// The maximum number of distinct pseudonyms for which we hold a SURB ringbuffer.
    ///
    /// Affects only the replying side.
    ///
    /// For each pseudonym, there is a ring-buffer with capacity `rb_capacity`.
    ///
    /// Default is 10 000.
    #[default(default_max_pseudonyms())]
    #[validate(range(min = 100, message = "max_pseudonyms must be at least 100"))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_pseudonyms"))]
    pub max_pseudonyms: usize,
    /// Maximum lifetime of ring-buffer for each pseudonym.
    ///
    /// # Effects on sending side
    /// This is the period for which we hold all reply openers for a pseudonym.
    /// If no more messages carrying SURBs are sent during this period, the entire stash of
    /// reply openers is dropped. Preventing receiving any more replies for that pseudonym.
    ///
    /// # Effects on replying side
    /// If a pseudonym has not received any SURBs for this period,
    /// the entire ring buffer with `rb_capacity` (= all SURBs for this pseudonym) is dropped.
    /// Preventing from sending any more replies for that pseudonym.
    ///
    /// Default is 600 seconds.
    #[default(default_pseudonyms_lifetime())]
    #[validate(custom(function = "validate_pseudonyms_lifetime"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_pseudonyms_lifetime", with = "humantime_serde")
    )]
    pub pseudonyms_lifetime: Duration,
    /// Maximum lifetime of a reply opener.
    ///
    /// Affects only the sending side.
    ///
    /// A reply opener is distinguished using [`HoprSurbId`] and a pseudonym it belongs to.
    /// If a reply opener is not used to decrypt the received packet within this period,
    /// it is dropped. If the replying side uses the corresponding SURB to send a reply,
    /// it won't be possible to decrypt it when received.
    ///
    /// Default is 3600 seconds.
    #[default(default_reply_opener_lifetime())]
    #[validate(custom(function = "validate_reply_opener_lifetime"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_reply_opener_lifetime", with = "humantime_serde")
    )]
    pub reply_opener_lifetime: Duration,
    /// How often cache evictions are summarised in the log, per cache and cause, instead of per entry.
    ///
    /// Affects both sides. Default is 60 seconds.
    #[default(default_eviction_report_interval())]
    #[validate(custom(function = "validate_eviction_report_interval"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_eviction_report_interval", with = "humantime_serde")
    )]
    pub eviction_report_interval: Duration,
    /// Evictions from one cache per report interval above which the summary is a warning rather than debug.
    ///
    /// Lost PIX shares (`ring_share`) are exempt: they are permanent, so any count warns, however high
    /// this is set. Dropped share-less SURBs (`ring_plain`) are not.
    ///
    /// Affects both sides. Default is 1000.
    #[default(default_eviction_report_threshold())]
    #[cfg_attr(feature = "serde", serde(default = "default_eviction_report_threshold"))]
    pub eviction_report_threshold: u64,
}

/// Basic [`SurbStore`] implementation based on an in-memory cache.
///
/// This SURB store offers no persistence, and all SURBs and Reply Openers are lost once dropped.
///
/// The instance can be cheaply cloned.
#[derive(Clone)]
pub struct MemorySurbStore {
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    surbs_per_pseudonym: moka::sync::Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
    /// Relayers this node can no longer pay. Holds at most a handful of entries (our own closing
    /// channels), so a plain set behind an `RwLock` beats a concurrent map on this read-heavy path.
    invalidated_relayers: Arc<parking_lot::RwLock<std::collections::HashSet<HoprKeyIdent>>>,
    /// Current SURB-batch generation per pseudonym we originate for (sending side). Advanced on a
    /// return-path change so the replying side can drop SURBs for the superseded path.
    ///
    /// This is sending-side state, so it is retained like the reply openers
    /// ([`pseudonym_openers`](Self::pseudonym_openers)) — **not** like the receiving-side
    /// [`surbs_per_pseudonym`](Self::surbs_per_pseudonym). If this entry were evicted while the peer
    /// still held SURBs of generation `N`, [`current_generation`](Self::current_generation) would
    /// fall back to `0`; the peer's [`SurbRingBuffer::push`] then reads `generation_is_newer(0, N)`
    /// as false (for `1 <= N <= 128`) and silently discards every fresh batch, stranding the reply
    /// path. Both dimensions are therefore taken from the sending-side reply-opener config, not the
    /// receiving-side SURB config: `reply_opener_lifetime` bounds how long we expect replies (hence
    /// outstanding SURBs) for a pseudonym, and it is `>=` the peer's SURB idle window, so the serial
    /// outlives the SURBs it numbers; and the capacity matches the reply-opener pseudonym bound
    /// (`max_openers_per_pseudonym`, which covers `maximum_managed_sessions`) so LRU pressure from the
    /// unrelated receiving-side `max_pseudonyms` cannot evict a live sender's generation.
    generations: moka::sync::Cache<HoprPseudonym, Arc<std::sync::atomic::AtomicU8>>,
    stats: Arc<EvictionStats>,
    cfg: Arc<SurbStoreConfig>,
}

// The `cache` label takes the names of `EvictedCache`: the four moka caches, and `ring_plain` / `ring_share` for the
// SURBs a full ring buffer drops or refuses, which are not cache entries but are counted alongside them. They are
// values of the label, not new series, so the names, descriptions and label keys below stay as METRICS.md has them.
#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_SURB_STORE_EVICTIONS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_surb_store_evictions_count",
        "Number of entries evicted from the SURB store caches, by cache and removal cause",
        &["cache", "cause"],
    )
    .unwrap();
    static ref METRIC_SURB_STORE_EVICTIONS_LAST_INTERVAL: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_surb_store_evictions_last_interval",
        "Entries evicted from the SURB store caches during the last completed report interval, by cache and removal cause",
        &["cache", "cause"],
    )
    .unwrap();
}

/// Which of the store's caches shed an entry. Doubles as the `cache` label on metrics and log lines.
///
/// The two `Ring*` variants are not caches in the moka sense: they count the SURBs that one
/// pseudonym's full ring buffer drops or refuses (always with [`RemovalCause::Size`]), split by what
/// that costs, so that the same per-interval summary can tell a routine loss from a permanent one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum EvictedCache {
    /// Sending side: all reply openers of one pseudonym (`pseudonym_openers`).
    ReplyOpenerBatch,
    /// Sending side: a single reply opener within a pseudonym's batch.
    ReplyOpener,
    /// Replying side: a pseudonym's SURB ring buffer (`surbs_per_pseudonym`).
    SurbRing,
    /// Replying side: a share-less SURB a full ring buffer dropped, or refused as a newcomer. A lost
    /// return path and nothing more, and routine while a sender that bypasses the balancer gate
    /// outproduces what this side spends.
    RingPlain,
    /// Replying side: a SURB carrying a PIX share that a full ring buffer dropped. A share reaches the
    /// reconstructor only when its SURB is *used*, so the share is lost for good.
    RingShare,
    /// Sending side: a pseudonym's SURB generation serial (`generations`).
    Generation,
}

impl EvictedCache {
    const ALL: [EvictedCache; 6] = [
        Self::ReplyOpenerBatch,
        Self::ReplyOpener,
        Self::SurbRing,
        Self::RingPlain,
        Self::RingShare,
        Self::Generation,
    ];
}

#[cfg(all(feature = "telemetry", not(test)))]
fn cause_label(cause: RemovalCause) -> &'static str {
    match cause {
        RemovalCause::Expired => "expired",
        RemovalCause::Size => "size",
        RemovalCause::Replaced => "replaced",
        RemovalCause::Explicit => "explicit",
    }
}

/// Evictions from one cache in one report interval, by cause.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EvictionCounts {
    expired: u64,
    size: u64,
    replaced: u64,
}

impl EvictionCounts {
    /// Counts `n` evictions for `cause`.
    ///
    /// `Explicit` removals are deliberate (a used opener, an invalidation), not evictions, so they are skipped.
    fn bump(&mut self, cause: RemovalCause, n: u64) {
        match cause {
            RemovalCause::Expired => self.expired += n,
            RemovalCause::Size => self.size += n,
            RemovalCause::Replaced => self.replaced += n,
            RemovalCause::Explicit => {}
        }
    }

    fn total(&self) -> u64 {
        self.expired + self.size + self.replaced
    }
}

/// The interval being counted right now.
#[derive(Debug)]
struct IntervalState {
    started_at: Instant,
    /// Parallel to [`EvictedCache::ALL`].
    counts: [EvictionCounts; EvictedCache::ALL.len()],
}

/// Eviction counts per cache and cause, reported once per interval (GNO-793: per-entry lines flooded the log).
struct EvictionStats {
    interval: Duration,
    warn_threshold: u64,
    state: parking_lot::Mutex<IntervalState>,
}

impl EvictionStats {
    fn new(cfg: &SurbStoreConfig) -> Self {
        Self {
            interval: cfg.eviction_report_interval.max(MINIMUM_EVICTION_REPORT_INTERVAL),
            warn_threshold: cfg.eviction_report_threshold,
            state: parking_lot::Mutex::new(IntervalState {
                started_at: Instant::now(),
                counts: Default::default(),
            }),
        }
    }

    fn record(&self, cache: EvictedCache, cause: RemovalCause) {
        self.record_many(cache, cause, 1);
    }

    /// Counts `n` evictions at once, for a caller that learns of many together (one push dropping several
    /// SURBs) and should not take the lock once per entry.
    fn record_many(&self, cache: EvictedCache, cause: RemovalCause, n: u64) {
        if let Some(report) = self.record_many_at(cache, cause, n, Instant::now()) {
            report.log();
        }
    }

    /// Closes the interval if `now` is past it, then counts `n` evictions; returns the closed interval's counts.
    ///
    /// A count of zero is not an eviction: it returns at once, without taking the lock or closing the
    /// interval. That keeps the common case of a push that evicts nothing off the lock.
    fn record_many_at(&self, cache: EvictedCache, cause: RemovalCause, n: u64, now: Instant) -> Option<EvictionReport> {
        if n == 0 {
            return None;
        }

        // One lock for check, drain and count, so a rollover cannot split an eviction from its interval.
        let mut state = self.state.lock();
        let report = self.close_interval_if_elapsed(&mut state, now);
        state.counts[cache as usize].bump(cause, n);
        drop(state);

        #[cfg(all(feature = "telemetry", not(test)))]
        if cause != RemovalCause::Explicit {
            METRIC_SURB_STORE_EVICTIONS.increment_by(&[cache.into(), cause_label(cause)], n);
        }
        report
    }

    /// No runtime here, so the first eviction past the interval closes it: a quiet store reports late, not never.
    fn close_interval_if_elapsed(&self, state: &mut IntervalState, now: Instant) -> Option<EvictionReport> {
        if now.duration_since(state.started_at) < self.interval {
            return None;
        }
        state.started_at = now;
        let counts = std::mem::take(&mut state.counts);
        Some(EvictionReport {
            interval: self.interval,
            warn_threshold: self.warn_threshold,
            per_cache: EvictedCache::ALL.map(|cache| (cache, counts[cache as usize])),
        })
    }
}

/// The counts of one closed report interval, ready to be logged.
#[derive(Debug)]
struct EvictionReport {
    interval: Duration,
    warn_threshold: u64,
    per_cache: [(EvictedCache, EvictionCounts); EvictedCache::ALL.len()],
}

impl EvictionReport {
    fn counts(&self, cache: EvictedCache) -> EvictionCounts {
        self.per_cache[cache as usize].1
    }

    /// Whether `cache` evicted enough in the interval for its summary to be a warning: more than the
    /// configured threshold, except for lost shares, which warn at any count.
    ///
    /// A lost share is always worth a warning. It is permanent, and the redundancy budget that absorbs
    /// such losses is finite, so each one brings a cycle closer to being unrecoverable. A dropped
    /// share-less SURB, in contrast, is routine under gate bypass (the sender outproduces what this side
    /// spends), so it keeps the configured threshold like every other cache.
    fn exceeds_threshold(&self, cache: EvictedCache) -> bool {
        let threshold = match cache {
            EvictedCache::RingShare => 0,
            _ => self.warn_threshold,
        };
        self.counts(cache).total() > threshold
    }

    fn log(&self) {
        let interval_secs = self.interval.as_secs();
        for (cache, counts) in self.per_cache {
            #[cfg(all(feature = "telemetry", not(test)))]
            for (cause, value) in [
                (RemovalCause::Expired, counts.expired),
                (RemovalCause::Size, counts.size),
                (RemovalCause::Replaced, counts.replaced),
            ] {
                METRIC_SURB_STORE_EVICTIONS_LAST_INTERVAL.set(&[cache.into(), cause_label(cause)], value as f64);
            }

            if counts.total() == 0 {
                continue;
            }
            if self.exceeds_threshold(cache) {
                tracing::warn!(
                    %cache,
                    expired = counts.expired,
                    size = counts.size,
                    replaced = counts.replaced,
                    interval_secs,
                    "SURB store evicted entries in the last interval"
                );
            } else {
                tracing::debug!(
                    %cache,
                    expired = counts.expired,
                    size = counts.size,
                    replaced = counts.replaced,
                    interval_secs,
                    "SURB store evicted entries in the last interval"
                );
            }
        }
    }
}

impl MemorySurbStore {
    /// Creates a new instance with the given configuration.
    pub fn new(cfg: SurbStoreConfig) -> Self {
        #[cfg(all(feature = "telemetry", not(test)))]
        {
            lazy_static::initialize(&METRIC_SURB_STORE_EVICTIONS);
            lazy_static::initialize(&METRIC_SURB_STORE_EVICTIONS_LAST_INTERVAL);
        }
        let stats = Arc::new(EvictionStats::new(&cfg));
        Self {
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener({
                    let stats = stats.clone();
                    move |_pseudonym, _openers, cause| stats.record(EvictedCache::ReplyOpenerBatch, cause)
                })
                .max_capacity(cfg.max_openers_per_pseudonym.max(MINIMUM_OPENER_PSEUDONYMS) as u64)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener({
                    let stats = stats.clone();
                    move |_pseudonym, _surbs, cause| stats.record(EvictedCache::SurbRing, cause)
                })
                .max_capacity(cfg.max_pseudonyms.max(MINIMUM_SURBS_PER_PSEUDONYM) as u64)
                .build(),
            invalidated_relayers: Default::default(),
            generations: moka::sync::Cache::builder()
                // Sending-side state, so it is sized and aged like the sibling sending-side
                // `pseudonym_openers` cache — NOT like the receiving-side `surbs_per_pseudonym`.
                // Retained for the reply-opener window (>= the peer's SURB idle window) so the serial
                // cannot reset to 0 while the peer still holds SURBs it numbers, and bounded by the
                // reply-opener pseudonym capacity (which covers `maximum_managed_sessions`) so LRU
                // pressure from the receiving-side `max_pseudonyms` cannot evict a live sender's
                // generation. See the field doc for why an early eviction would strand the reply path.
                .time_to_idle(cfg.reply_opener_lifetime.max(MINIMUM_OPENER_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener({
                    let stats = stats.clone();
                    move |pseudonym, _generation, cause| {
                        // Minting keeps this warm, so each eviction earns a line: a reset serial has the peer reject
                        // SURBs.
                        tracing::warn!(%pseudonym, ?cause, "evicting SURB generation for pseudonym");
                        stats.record(EvictedCache::Generation, cause);
                    }
                })
                .max_capacity(cfg.max_openers_per_pseudonym.max(MINIMUM_OPENER_PSEUDONYMS) as u64)
                .build(),
            stats,
            cfg: cfg.into(),
        }
    }

    /// Whether `relayer` is currently unusable as a return path's first hop.
    pub fn is_relayer_invalidated(&self, relayer: &HoprKeyIdent) -> bool {
        self.invalidated_relayers.read().contains(relayer)
    }

    /// Whether a stored SURB can still be used to reply: its first relayer must still be payable.
    ///
    /// A direct return path is exempt — its "first relayer" is the final recipient, which needs no
    /// channel (RFC-0003 §3.2, RFC-0006 §6.1). Without that exemption, closing an unrelated channel
    /// to a session's originator would discard perfectly good SURBs.
    fn is_surb_usable(&self, surb: &HoprSurb) -> bool {
        match surb.additional_data_receiver.proof_of_relay_values().chain_length() {
            // Direct return path: the "first relayer" is the final recipient, which needs no channel.
            1 => true,
            // A chain length is hops + 1, so 0 cannot occur on a well-formed SURB. Refuse it rather
            // than let a malformed value pass as "direct" and bypass the check below.
            //
            // The length is an unvalidated byte off a SURB minted by the counterparty, so this is a
            // statement about that peer, not a local fault we could act on: `warn`, not `error`.
            0 => {
                tracing::warn!(
                    first_relayer = %surb.first_relayer,
                    "refusing a malformed SURB declaring a zero-length return path"
                );
                false
            }
            _ => {
                let usable = !self.invalidated_relayers.read().contains(&surb.first_relayer);
                if !usable {
                    tracing::trace!(
                        first_relayer = %surb.first_relayer,
                        "refusing a SURB whose first relayer is invalidated"
                    );
                }
                usable
            }
        }
    }
}

impl Default for MemorySurbStore {
    fn default() -> Self {
        Self::new(SurbStoreConfig::default())
    }
}

impl SurbStore for MemorySurbStore {
    #[tracing::instrument(skip_all, level = "trace", fields(?matcher), ret)]
    fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb> {
        let pseudonym = matcher.pseudonym();
        let surbs_for_pseudonym = self.surbs_per_pseudonym.get(&pseudonym)?;

        match matcher {
            // SURBs whose return path no longer has a usable first edge are dropped on the way,
            // rather than handed out only to have the reply fail to be paid for.
            SurbMatcher::Pseudonym(_) => surbs_for_pseudonym
                .pop_next_valid(|_, surb| self.is_surb_usable(surb))
                .map(|popped_surb| FoundSurb {
                    sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                    surb: popped_surb.surb,
                    remaining: popped_surb.remaining,
                }),
            // The following code intentionally only checks the oldest SURB of the tier that would be
            // consumed next (share-bearing SURBs first, share-less ones once none are left) and does
            // not search the entire RB.
            // This is because the exact match use-case is suited only for situations
            // when there is a single SURB in the RB.
            SurbMatcher::Exact(id) => {
                surbs_for_pseudonym
                    .pop_one_if_has_id(&id.surb_id())
                    .map(|popped_surb| FoundSurb {
                        sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                        surb: popped_surb.surb,
                        remaining: popped_surb.remaining, // = likely 0
                    })
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace", fields(%pseudonym, num_surbs = surbs.len()))]
    fn insert_surbs(&self, pseudonym: HoprPseudonym, mut surbs: Vec<(HoprSurbId, HoprSurb)>) -> SurbInsertOutcome {
        // A batch is one packet's worth of SURBs, minted by the creator at a single generation, so
        // the generation of the first stands for the whole batch. An empty batch carries no
        // generation and must not create or disturb the buffer.
        let Some(generation) = surbs
            .first()
            .map(|(_, surb)| surb.additional_data_receiver.generation())
        else {
            return SurbInsertOutcome {
                retained: self.surbs_per_pseudonym.get(&pseudonym).map(|rb| rb.len()).unwrap_or(0),
                evicted: 0,
                evicted_shares: 0,
            };
        };

        // That "single generation" holds by construction only for a batch minted by an honest
        // creator: `PacketRouting::ForwardPath` stamps one generation into every SURB it mints. On
        // this side the batch is parsed out of a counterparty-controlled payload, so enforce it
        // rather than assume it — otherwise a mixed batch smuggles SURBs for a superseded return
        // path into a buffer the push below labels with the newer generation, where no later push
        // can clear them. The first SURB always survives, so the buffer is never created empty.
        let mixed = surbs.len();
        surbs.retain(|(_, surb)| surb.additional_data_receiver.generation() == generation);
        let dropped = mixed - surbs.len();
        if dropped > 0 {
            // A statement about the peer that minted the batch, not a local fault: `warn`, not
            // `error`. These are not capacity pressure, so they stay out of the `evicted` count.
            tracing::warn!(
                %pseudonym,
                dropped,
                generation,
                "discarding SURBs whose generation disagrees with the rest of their batch"
            );
        }

        let outcome = self
            .surbs_per_pseudonym
            .entry_by_ref(&pseudonym)
            .or_insert_with(|| SurbRingBuffer::new(self.cfg.rb_capacity.max(MIN_SURB_RB_CAPACITY)))
            .value()
            .push(surbs, generation);

        // What a full buffer dropped goes into the same interval summary as every other eviction, split
        // by what it cost. The total that `num_evicted_surbs` carries upwards, and the Exit books into
        // its flow estimate, cannot tell a lost share from a lost return path, and only the former
        // always warrants a warning. Recording a count of zero is free, so the usual push, which
        // overflows nothing, takes no lock here.
        self.stats.record_many(
            EvictedCache::RingPlain,
            RemovalCause::Size,
            outcome.evicted.saturating_sub(outcome.evicted_shares) as u64,
        );
        self.stats.record_many(
            EvictedCache::RingShare,
            RemovalCause::Size,
            outcome.evicted_shares as u64,
        );

        outcome
    }

    fn tier_lens(&self, pseudonym: &HoprPseudonym) -> Option<(usize, usize)> {
        self.surbs_per_pseudonym.get(pseudonym).map(|rb| rb.tier_lens())
    }

    #[tracing::instrument(skip_all, level = "trace", fields(?sender_id))]
    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        let opener_lifetime = self.cfg.reply_opener_lifetime.max(MINIMUM_OPENER_LIFETIME);
        let max_openers_per_pseudonym = self.cfg.max_openers_per_pseudonym.max(MINIMUM_OPENERS_PER_PSEUDONYM);
        let stats = self.stats.clone();
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), move || {
                moka::sync::Cache::builder()
                    .time_to_live(opener_lifetime)
                    // Keep the newest openers, not the stalest. Reply openers are written once and
                    // never read before they are used, so under the default TinyLFU policy every
                    // entry ties at frequency zero and incumbents win admission: a full cache then
                    // freezes the oldest openers and drops every newer one. But the counterparty's
                    // SURB ring always hands back the newest SURBs, whose openers are exactly those
                    // dropped — so replies stop decrypting once the cache fills. LRU on this
                    // write-only workload evicts by insertion order, i.e. it sheds the oldest and
                    // keeps the newest, mirroring the SURB ring buffer (and the outer cache).
                    .eviction_policy(moka::policy::EvictionPolicy::lru())
                    .eviction_listener(move |_id, _opener, cause| stats.record(EvictedCache::ReplyOpener, cause))
                    .max_capacity(max_openers_per_pseudonym as u64)
                    .build()
            })
            .insert(sender_id.surb_id(), opener);
    }

    #[tracing::instrument(skip_all, level = "trace")]
    fn invalidate_relayer(&self, relayer: &HoprKeyIdent) {
        if self.invalidated_relayers.write().insert(*relayer) {
            tracing::info!(
                %relayer,
                "invalidating stored SURBs whose return path starts with this relayer"
            );
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    fn revalidate_relayer(&self, relayer: &HoprKeyIdent) {
        if self.invalidated_relayers.write().remove(relayer) {
            tracing::info!(%relayer, "relayer is usable again for SURB return paths");
        }
    }

    #[tracing::instrument(skip_all, level = "trace", fields(?sender_id), ret)]
    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener> {
        self.pseudonym_openers
            .get(&sender_id.pseudonym())
            .and_then(|cache| cache.remove(&sender_id.surb_id()))
    }

    fn current_generation(&self, pseudonym: &HoprPseudonym) -> u8 {
        self.generations
            .get(pseudonym)
            .map(|g| g.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0)
    }

    #[tracing::instrument(skip_all, level = "trace", fields(%pseudonym), ret)]
    fn bump_generation(&self, pseudonym: &HoprPseudonym) -> u8 {
        // `fetch_add` returns the previous value; the new generation is one past it. A `u8` serial
        // wraps cleanly (255 -> 0) and the replying side compares with RFC-1982 arithmetic.
        self.generations
            .get_with(*pseudonym, || Arc::new(std::sync::atomic::AtomicU8::new(0)))
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .wrapping_add(1)
    }
}

/// Represents a single SURB along with its ID popped from the [`SurbRingBuffer`].
#[derive(Debug, Clone)]
pub struct PoppedSurb<S> {
    /// Complete SURB sender ID.
    pub id: HoprSurbId,
    /// The popped SURB.
    pub surb: S,
    /// Number of SURBs left in the RB after the pop.
    pub remaining: usize,
}

/// RFC-1982 serial-number comparison over a `u8` generation: is `a` strictly newer than `b`?
///
/// SURB generations are minted monotonically by the creator and only ever compared across the two
/// adjacent generations that can be in flight at once, so a `u8` serial (window 128) is ample and
/// wraps cleanly: 255 → 0 reads as newer.
fn generation_is_newer(a: u8, b: u8) -> bool {
    a != b && a.wrapping_sub(b) < 128
}

/// Which SSA the PIX share sealed in a SURB belongs to.
///
/// This is what lets [`SurbRingBuffer`] tell share-bearing SURBs from share-less ones on insertion. The
/// SSA index travels in plaintext in the SURB's receiver-only block, which only the node the SURB was
/// sent to can read and no relay ever sees, so classifying on it reveals nothing that node did not
/// already hold. Only *whether* there is a share is used for now; the index itself is returned so that a
/// finer ordering, or a trace line, can name the SSA without another look into the block.
pub trait SurbShareInfo {
    /// The SSA index of the share this SURB carries, or `None` for a share-less SURB.
    fn ssa_index(&self) -> Option<SsaIndex>;
}

impl SurbShareInfo for HoprSurb {
    fn ssa_index(&self) -> Option<SsaIndex> {
        let share = self.additional_data_receiver.encrypted_partial_ssa_share();
        // The very test the packet builder applies when it decides whether a reply carries this share
        // (`!is_empty()`), so what the store calls share-bearing cannot drift from what the Exit sends.
        if share.is_empty() {
            None
        } else {
            share.indices().map(|(ssa_index, _)| ssa_index)
        }
    }
}

/// Ring buffer of SURBs and their IDs, all belonging to one pseudonym and therefore identified only
/// by [`HoprSurbId`].
///
/// Backed by two [`VecDeque`] tiers that together are never allowed to exceed `capacity`: a push into
/// a full buffer evicts first (see below for which SURB goes). Each tier is first in, first out: a pop
/// consumes the oldest SURB of the tier it draws from, and so does an eviction. See [`SurbPopOrder`]
/// for why that is the only order.
///
/// ## Share-bearing SURBs first
///
/// A PIX share is sealed into a SURB when the Entry mints it, and reaches the Exit's reconstructor only
/// when the Exit *uses* that SURB. The Entry emits shares strictly in SSA-index order and has none left
/// to seal once the committed cycle is exhausted, so every SURB it mints from then on is share-less. The
/// next cycle's shares are minted only after the Exit has requested that cycle, which is to say *behind*
/// every share-less SURB minted in between. While the Entry produces SURBs faster than the Exit spends
/// them — `max_surbs_per_data_packet > 1` bypasses the balancer's gate, so a 512 B write carries about
/// six against the one a reply spends — that share-less backlog grows faster than it drains. In a single
/// FIFO queue the next cycle's shares are then reached late or never, and the supervisor closes the
/// Session on `RecoveryIdle` (hoprnet#8466).
///
/// So the buffer keeps two tiers and always hands out the share-bearing one first. [`SurbShareInfo`]
/// sorts each SURB into `shares` or `plain` as it arrives, and a pop takes from `shares` for as long as
/// it holds anything and from `plain` after that, the oldest SURB of either first. The Entry emits
/// shares in SSA-index order, so FIFO inside `shares` is also lowest-SSA-first.
///
/// Eviction follows the same priority, because a share-less SURB is only a return path while a
/// share-bearing one is a return path *and* a share nothing can replace. A full buffer gives up share-less
/// SURBs first, oldest first. A share-less newcomer is refused rather than evict a share. Only when
/// nothing but shares is held does the oldest share go, as the oldest SURB always used to. A push reports
/// how many SURBs it dropped and, of those, how many were shares, so that the loss that matters can be told
/// from the routine one (see [`SurbInsertOutcome`]).
///
/// What this deliberately does *not* do: order shares across SSAs beyond these two tiers, retire the
/// leftovers of an already-recovered SSA (the supervisor budgets for those, see its
/// `paid_recovery_tail`), or insist that a batch belong to one SSA — a packet's SURBs legitimately
/// straddle a cycle boundary, or the step from carrying shares to carrying none.
///
/// ## Generations: dropping SURBs for a superseded return path
///
/// A return path that dies deep in a multi-hop route is invisible to this replying side — nothing
/// here can tell a stale SURB from a live one. The SURB creator can: it stamps every SURB of a
/// batch with a generation (`SurbReceiverInfo::generation`) and bumps it whenever it changes the
/// return path. This buffer keeps only the highest generation it has seen: the first push carrying a
/// newer generation **clears the buffer wholesale** (both tiers) before inserting, so a return-path
/// change takes effect on the very next reply rather than only once the stale SURBs drain — and stale
/// SURBs are never handed out. A push carrying an older generation (a late/reordered batch) is
/// discarded. Clearing is a per-path-change O(n) sweep, so pops need no per-SURB generation check.
///
/// ## Why the capacity is a ceiling, not a reservation
///
/// The deques grow with occupancy rather than being sized at construction. The distinction matters
/// because the pseudonym a buffer is filed under is chosen by whoever sent the packet: any
/// `HoprPacket::Final` carrying a SURB reaches `insert_surbs`, which mints a buffer for a pseudonym
/// it has never seen, with no Session or handshake behind it.
///
/// A structure that took its whole capacity upfront therefore let an unauthenticated peer reserve
/// `rb_capacity` × `size_of::<(HoprSurbId, S)>()` of address space per pseudonym it invented —
/// ~16.8 MB each at the default capacity, and `max_pseudonyms` of those. Resident memory was never
/// the problem (untouched pages cost nothing), but the reservation is real to anything that
/// accounts address space: strict overcommit, `ulimit -v`, `vm.max_map_count`.
///
/// Growth is geometric and amortised, and a deque never shrinks below its high-water mark, so a
/// pseudonym that genuinely fills up still ends at the same footprint — and stops reallocating
/// there, however long the steady-state overflow runs. It just has to earn it. Each tier keeps its own
/// mark, so a pseudonym that has been filled with share-less SURBs and then with shares holds both:
/// at most twice that footprint, and only after the buffer has been filled twice over.
#[derive(Clone, Debug)]
pub struct SurbRingBuffer<S> {
    inner: Arc<parking_lot::Mutex<GenerationalBuffer<S>>>,
    /// Ceiling on the number of retained SURBs across both tiers; one is dropped (a share-less SURB if
    /// there is one) for every SURB that arrives once it is reached.
    capacity: usize,
}

/// The mutex-protected state of a [`SurbRingBuffer`]: the SURBs, in their two tiers, and the highest
/// generation seen.
///
/// All of it lives under one lock so that clearing both tiers and advancing the generation on a newer
/// batch is atomic against a concurrent pop, and so that the combined length the capacity bounds is
/// never seen or changed half-way.
#[derive(Debug)]
struct GenerationalBuffer<S> {
    /// SURBs carrying a PIX share, in insertion order. The Entry emits shares in SSA-index order, so
    /// FIFO here is also lowest-SSA-first.
    shares: VecDeque<(HoprSurbId, S)>,
    /// SURBs carrying no share: pure return paths.
    plain: VecDeque<(HoprSurbId, S)>,
    /// Highest generation seen; `None` until the first push.
    generation: Option<u8>,
}

impl<S> GenerationalBuffer<S> {
    /// SURBs held across both tiers: the quantity the capacity bounds.
    fn len(&self) -> usize {
        self.shares.len() + self.plain.len()
    }

    /// Drops every SURB of both tiers.
    fn clear(&mut self) {
        self.shares.clear();
        self.plain.clear();
    }

    /// The tier a pop takes from: share-bearing SURBs for as long as there are any, share-less ones only
    /// once there are none. The single place that priority is decided, so peeking and popping — and
    /// with them the exact-ID pop — cannot disagree about which SURB is next.
    fn next_tier(&self) -> &VecDeque<(HoprSurbId, S)> {
        if self.shares.is_empty() {
            &self.plain
        } else {
            &self.shares
        }
    }

    /// As [`next_tier`](Self::next_tier), for removing from it.
    fn next_tier_mut(&mut self) -> &mut VecDeque<(HoprSurbId, S)> {
        if self.shares.is_empty() {
            &mut self.plain
        } else {
            &mut self.shares
        }
    }

    /// The SURB the next pop would take: the oldest of the next tier. Peeking and popping both go
    /// through [`next_tier`](Self::next_tier), so they stay in step.
    fn peek_next(&self) -> Option<&(HoprSurbId, S)> {
        self.next_tier().front()
    }

    /// Removes and returns the oldest SURB of the next tier.
    fn pop_next(&mut self) -> Option<(HoprSurbId, S)> {
        self.next_tier_mut().pop_front()
    }

    /// Makes room in a full buffer for one more SURB by evicting the oldest share-less one.
    ///
    /// Failing that, the oldest share-bearing one goes if the newcomer carries a share too. A share-less
    /// newcomer is turned away instead — a return path is not worth a share nothing can replace — and
    /// [`MadeRoom::Refused`] says so: there is no room for it, and the caller must not insert it.
    ///
    /// What happened is returned rather than just whether there is room, because the cost differs: a
    /// share-less SURB is only a lost return path, a share-bearing one is a share nothing can replace.
    fn make_room_for(&mut self, newcomer_carries_share: bool) -> MadeRoom {
        if self.plain.pop_front().is_some() {
            MadeRoom::EvictedPlain
        } else if newcomer_carries_share && self.shares.pop_front().is_some() {
            MadeRoom::EvictedShare
        } else {
            MadeRoom::Refused
        }
    }
}

/// What [`GenerationalBuffer::make_room_for`] did to a full buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MadeRoom {
    /// The oldest share-less SURB was evicted; the newcomer may be inserted.
    EvictedPlain,
    /// The oldest share-bearing SURB was evicted, and its PIX share with it; the newcomer, which carries
    /// a share too, may be inserted.
    EvictedShare,
    /// Nothing was evicted: the buffer holds nothing but shares and the newcomer carries none. There is
    /// no room for it, and it must not be inserted.
    Refused,
}

impl<S> SurbRingBuffer<S> {
    /// Creates a buffer holding at most `capacity` (min 1, so a push is never a no-op) SURBs, popped
    /// oldest first within each tier.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(parking_lot::Mutex::new(GenerationalBuffer {
                shares: VecDeque::new(),
                plain: VecDeque::new(),
                generation: None,
            })),
            // A zero capacity would make the eviction below pop from an empty buffer and then push
            // past the bound. Callers already clamp to `MIN_SURB_RB_CAPACITY`; this is belt-and-braces.
            capacity: capacity.max(1),
        }
    }

    /// Pushes all SURBs of one batch, stamped with `generation`, evicting past capacity.
    ///
    /// A batch is minted at a single generation by the creator (one packet's worth of SURBs), so a
    /// single `generation` covers the whole `surbs` iterator. Relative to the highest generation
    /// seen so far:
    /// - **newer** → both tiers are cleared before inserting, so SURBs for the superseded return path are dropped at
    ///   once rather than lingering until they drain;
    /// - **equal** → the batch is appended (an ordinary refill);
    /// - **older** → the batch is discarded as a late/reordered leftover.
    ///
    /// Each SURB is sorted into the share-bearing or the share-less tier by [`SurbShareInfo`]. Once at
    /// capacity, every one that arrives costs the buffer one SURB, share-less before share-bearing:
    /// - if a share-less SURB is held, the oldest of them is evicted;
    /// - else, a share-less newcomer is refused: it is only a return path, and a share-bearing SURB is that plus a
    ///   share nothing can replace;
    /// - else, only shares are held and the newcomer carries one too, so the oldest share-bearing SURB is evicted.
    ///
    /// A refused newcomer counts as `evicted` all the same: it is a SURB the buffer had no room for. Under
    /// PIX an evicted share-bearing SURB is a lost SSA share, not merely a lost SURB — see
    /// [`SurbStoreConfig::rb_capacity`] — so those are counted on their own, as `evicted_shares`: the
    /// exact number of shares the push lost to overflow, a subset of `evicted`.
    ///
    /// Returns what the push did; the eviction count is what lets a caller notice the overflow at
    /// all, since dropping a SURB is otherwise indistinguishable from a clean insert.
    /// It counts *capacity* overflow only: SURBs dropped because a newer generation superseded them
    /// were already unusable, and reporting them as pressure would ask the creator to slow down
    /// because it re-planned its own return path.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I, generation: u8) -> SurbInsertOutcome
    where
        S: SurbShareInfo,
    {
        let mut inner = self.inner.lock();

        match inner.generation {
            Some(current) if generation == current => {} // ordinary refill: append below
            Some(current) if generation_is_newer(generation, current) => {
                // Return path changed: everything held is for the superseded path. Drop it wholesale
                // so the newer batch is all that remains and the next reply uses the live path.
                inner.clear();
                inner.generation = Some(generation);
            }
            Some(_) => {
                // Older than what we already hold: a late or reordered batch for a path the creator
                // has already moved on from. Discard it rather than reintroduce stale SURBs.
                return SurbInsertOutcome {
                    retained: inner.len(),
                    evicted: 0,
                    evicted_shares: 0,
                };
            }
            None => inner.generation = Some(generation),
        }

        let mut evicted = 0;
        let mut evicted_shares = 0;
        for (id, surb) in surbs {
            let carries_share = surb.ssa_index().is_some();

            // Make room before inserting, so the length never exceeds the ceiling and the backing
            // allocations stop growing once their high-water marks are reached. Either a resident makes
            // way or the newcomer is turned away; both are one SURB dropped for want of room.
            if inner.len() >= self.capacity {
                evicted += 1;
                match inner.make_room_for(carries_share) {
                    MadeRoom::EvictedPlain => {}
                    MadeRoom::EvictedShare => evicted_shares += 1,
                    MadeRoom::Refused => continue,
                }
            }

            let tier = if carries_share {
                &mut inner.shares
            } else {
                &mut inner.plain
            };
            tier.push_back((id, surb));
        }
        SurbInsertOutcome {
            retained: inner.len(),
            evicted,
            evicted_shares,
        }
    }

    /// Pops the next SURB that `is_valid` accepts: a share-bearing one while any is held, the oldest of
    /// each tier first.
    ///
    /// **Destructive:** rejected entries are discarded, not skipped, so an unusable SURB neither is
    /// handed out nor blocks those behind it. Pass only a validity test — a selective predicate
    /// (say, a routing preference) would drain the buffer. `None` once it is exhausted without a
    /// match.
    ///
    /// `is_valid` runs *outside* the lock: it is caller-supplied and may take locks of its own, so
    /// calling it inside the critical section would invite lock-order inversion.
    pub fn pop_next_valid<F>(&self, is_valid: F) -> Option<PoppedSurb<S>>
    where
        F: Fn(&HoprSurbId, &S) -> bool,
    {
        loop {
            let (id, surb, remaining) = {
                let mut inner = self.inner.lock();
                let (id, surb) = inner.pop_next()?;
                (id, surb, inner.len())
            };

            if is_valid(&id, &surb) {
                return Some(PoppedSurb { id, surb, remaining });
            }
        }
    }

    /// Number of SURBs currently held, across both tiers.
    fn len(&self) -> usize {
        self.inner.lock().len()
    }

    /// Number of SURBs currently held in each tier, as `(share-bearing, share-less)`: [`len`](Self::len)
    /// split the way a pop sees it, share-bearing first. Both are read under one lock, so they are a
    /// consistent pair.
    fn tier_lens(&self) -> (usize, usize) {
        let inner = self.inner.lock();
        (inner.shares.len(), inner.plain.len())
    }

    /// Pops the next SURB (the one [`pop_next_valid`](Self::pop_next_valid) would take first) only if it has
    /// the given ID.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Option<PoppedSurb<S>> {
        let mut inner = self.inner.lock();

        if inner.peek_next().is_some_and(|(surb_id, _)| surb_id == id) {
            let (id, surb) = inner.pop_next()?;
            Some(PoppedSurb {
                id,
                surb,
                remaining: inner.len(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use anyhow::Context;
    use hopr_api::types::{
        crypto::{crypto_traits::Randomizable, prelude::SecretKey16},
        primitive::prelude::BytesRepresentable,
    };
    use hopr_crypto_packet::sphinx::prelude::SphinxHeaderSpec;
    use rstest::rstest;

    use super::*;

    impl<S> SurbRingBuffer<S> {
        /// Pops the next SURB regardless of validity — the buffer-ordering tests below are about
        /// which end is consumed, not about which SURBs are usable.
        fn pop_any(&self) -> Option<PoppedSurb<S>> {
            self.pop_next_valid(|_, _| true)
        }

        /// Snapshot of the highest generation the buffer has seen (test-only accessor).
        fn generation(&self) -> Option<u8> {
            self.inner.lock().generation
        }

        /// Slots allocated across both tiers (test-only accessor).
        fn allocated(&self) -> usize {
            let inner = self.inner.lock();
            inner.shares.capacity() + inner.plain.capacity()
        }
    }

    /// The ordering and accounting tests below push bare integers where only the buffer mechanics
    /// matter; they stand in for SURBs that carry no PIX share.
    impl SurbShareInfo for i32 {
        fn ssa_index(&self) -> Option<SsaIndex> {
            None
        }
    }

    impl SurbShareInfo for u64 {
        fn ssa_index(&self) -> Option<SsaIndex> {
            None
        }
    }

    /// Builds a SURB with the given first relayer, PoR chain length, and batch generation.
    ///
    /// Only those fields are read by the store, so the SURB is assembled straight from its wire
    /// layout — `first_relayer | alpha | header | sender_key | additional_data_receiver` — whose
    /// parser performs no cryptographic validation. That avoids a full Sphinx key exchange per
    /// fixture and keeps the fields exactly controllable.
    fn surb_gen(first_relayer: HoprKeyIdent, chain_length: u8, generation: u8) -> anyhow::Result<HoprSurb> {
        let mut bytes = vec![0u8; HoprSurb::SIZE];

        let key_id_size = HoprSphinxHeaderSpec::KEY_ID_SIZE.get();
        bytes[..key_id_size].copy_from_slice(first_relayer.as_ref());

        // The chain length is the leading byte of the receiver's proof-of-relay values, which lead
        // the trailing `additional_data_receiver` block; the generation is that block's last byte.
        bytes[HoprSurb::SIZE - HoprSphinxHeaderSpec::SURB_RECEIVER_DATA_SIZE] = chain_length;
        bytes[HoprSurb::SIZE - 1] = generation;

        let surb = HoprSurb::try_from(bytes.as_slice())?;

        // Guard the hand-rolled layout: a wrong offset would silently yield the wrong field and make
        // the assertions below pass for the wrong reason.
        assert_eq!(first_relayer, surb.first_relayer, "fixture: wrong first relayer");
        assert_eq!(
            chain_length,
            surb.additional_data_receiver.proof_of_relay_values().chain_length(),
            "fixture: wrong chain length"
        );
        assert_eq!(
            generation,
            surb.additional_data_receiver.generation(),
            "fixture: wrong generation"
        );

        Ok(surb)
    }

    /// A generation-0 SURB, for tests that do not exercise generations.
    fn surb_via(first_relayer: HoprKeyIdent, chain_length: u8) -> anyhow::Result<HoprSurb> {
        surb_gen(first_relayer, chain_length, 0)
    }

    /// A return path with one intermediate relayer: `me -> relayer -> recipient`.
    const TWO_HOP: u8 = 2;
    /// A return path straight to the recipient, which needs no payment channel.
    const DIRECT: u8 = 1;

    /// A two-hop, generation-0 SURB that carries the PIX share of SSA `ssa_index`, which must be non-zero
    /// (a zero index is what marks a share-less SURB).
    ///
    /// The share block sits directly before the trailing generation byte of the receiver-only data, laid
    /// out `ssa_index: u32 BE | poly_index: u16 BE | scalar`. The store reads only the SSA index, so that is
    /// the only part written; the polynomial index and the scalar stay zero. Like [`surb_gen`], the SURB is
    /// built from its wire layout, whose parser performs no cryptographic validation.
    fn surb_share(first_relayer: HoprKeyIdent, ssa_index: u32) -> anyhow::Result<HoprSurb> {
        let ssa = SsaIndex::new(ssa_index).context("fixture: the SSA index must be non-zero")?;

        let mut bytes = surb_via(first_relayer, TWO_HOP)?.into_boxed().into_vec();
        let share_at = HoprSurb::SIZE - 1 - HoprEncryptedPartialSsaShare::SIZE;
        bytes[share_at..share_at + size_of::<SsaIndex>()].copy_from_slice(&ssa.get().to_be_bytes());

        let surb = HoprSurb::try_from(bytes.as_slice())?;

        // Guard the hand-rolled layout, as `surb_gen` does: a wrong offset would silently yield a
        // share-less SURB, or the wrong SSA, and make the tests below pass for the wrong reason.
        let share = surb.additional_data_receiver.encrypted_partial_ssa_share();
        assert!(!share.is_empty(), "fixture: the share must not read as empty");
        assert_eq!(
            Some((ssa, 0)),
            share.indices(),
            "fixture: wrong SSA or polynomial index"
        );
        assert_eq!(first_relayer, surb.first_relayer, "fixture: wrong first relayer");
        assert_eq!(
            0,
            surb.additional_data_receiver.generation(),
            "fixture: wrong generation"
        );

        Ok(surb)
    }

    /// A share-less `(id, SURB)` pair for the tier tests. The id is `[n; 8]`, so `n` names the entry in
    /// assertions (see [`drain_ids`]).
    fn plain_surb(n: u8) -> anyhow::Result<(HoprSurbId, HoprSurb)> {
        Ok(([n; 8], surb_via(HoprKeyIdent::from(1u32), TWO_HOP)?))
    }

    /// As [`plain_surb`], but carrying the share of SSA `n` (so `n` must be at least 1).
    fn share_surb(n: u8) -> anyhow::Result<(HoprSurbId, HoprSurb)> {
        Ok(([n; 8], surb_share(HoprKeyIdent::from(1u32), u32::from(n))?))
    }

    /// Pops everything left, regardless of validity, and names each entry by its repeated id byte.
    fn drain_ids(rb: &SurbRingBuffer<HoprSurb>) -> Vec<u8> {
        std::iter::from_fn(|| rb.pop_any().map(|popped| popped.id[0])).collect()
    }

    #[test]
    fn memory_surb_store_should_skip_surbs_whose_first_relayer_was_invalidated() -> anyhow::Result<()> {
        let (dead, alive) = (HoprKeyIdent::from(1u32), HoprKeyIdent::from(2u32));

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        // Two SURBs return via the dead relay, one via a healthy one; all are two-hop.
        store.insert_surbs(
            pseudonym,
            vec![
                ([1u8; 8], surb_via(dead, TWO_HOP)?),
                ([2u8; 8], surb_via(dead, TWO_HOP)?),
                ([3u8; 8], surb_via(alive, TWO_HOP)?),
            ],
        );

        store.invalidate_relayer(&dead);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("expected a usable SURB"))?;
        assert_eq!([3u8; 8], found.sender_id.surb_id(), "must skip past the dead relayer");
        assert_eq!(
            0, found.remaining,
            "the invalidated SURBs must be discarded, not left behind"
        );

        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none(),
            "no usable SURB should remain"
        );

        Ok(())
    }

    #[test]
    fn memory_surb_store_should_not_invalidate_surbs_with_a_direct_return_path() -> anyhow::Result<()> {
        // A single-element path means the "first relayer" is the final recipient, which needs no
        // payment channel — closing a channel to it must not discard the SURB.
        let recipient = HoprKeyIdent::from(1u32);

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(pseudonym, vec![([7u8; 8], surb_via(recipient, DIRECT)?)]);
        store.invalidate_relayer(&recipient);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("a direct-return-path SURB must stay usable"))?;
        assert_eq!([7u8; 8], found.sender_id.surb_id());

        Ok(())
    }

    #[test]
    fn memory_surb_store_should_reject_a_surb_with_a_malformed_chain_length() -> anyhow::Result<()> {
        // A chain length is hops + 1, so 0 is malformed. It must not pass as "direct" and thereby
        // skip the invalidation check.
        let relayer = HoprKeyIdent::from(1u32);

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(pseudonym, vec![([5u8; 8], surb_via(relayer, 0)?)]);
        store.invalidate_relayer(&relayer);

        assert!(store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none());

        Ok(())
    }

    #[test]
    fn memory_surb_store_should_make_a_relayer_usable_again_after_revalidation() -> anyhow::Result<()> {
        let relayer = HoprKeyIdent::from(1u32);

        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(pseudonym, vec![([9u8; 8], surb_via(relayer, TWO_HOP)?)]);

        store.invalidate_relayer(&relayer);
        store.revalidate_relayer(&relayer);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("expected the revalidated SURB"))?;
        assert_eq!([9u8; 8], found.sender_id.surb_id());

        Ok(())
    }

    #[test]
    fn surb_store_config_should_default_to_fifo() {
        assert_eq!(SurbPopOrder::Fifo, SurbStoreConfig::default().pop_order);
        assert_eq!(SurbPopOrder::Fifo, SurbPopOrder::default());
    }

    /// LIFO was removed because PIX cannot work with it, so asking for it must be an error rather than
    /// a quiet fall back to FIFO.
    #[test]
    fn surb_pop_order_should_reject_lifo() -> anyhow::Result<()> {
        use std::str::FromStr;

        assert_eq!(
            SurbPopOrder::Fifo,
            SurbPopOrder::from_str("fifo").context("fifo must still parse")?
        );
        assert!(
            SurbPopOrder::from_str("lifo").is_err(),
            "LIFO was removed, so it must no longer parse"
        );

        Ok(())
    }

    /// The configuration path: `pop_order: lifo` fails to load as an unknown variant, while `fifo`,
    /// which existing configurations spell out, still loads. The config is deserialized from a map,
    /// which is all a configuration file is to serde, so no data-format crate is needed.
    #[cfg(feature = "serde")]
    #[test]
    fn surb_store_config_should_reject_a_lifo_pop_order() -> anyhow::Result<()> {
        use serde::{
            Deserialize,
            de::value::{Error, MapDeserializer},
        };

        let load = |pop_order: &'static str| {
            SurbStoreConfig::deserialize(MapDeserializer::<_, Error>::new([("pop_order", pop_order)].into_iter()))
        };

        assert_eq!(
            SurbPopOrder::Fifo,
            load("fifo").context("fifo must still load")?.pop_order
        );

        let Err(error) = load("lifo") else {
            anyhow::bail!("LIFO was removed, so a config asking for it must not load");
        };
        assert!(
            error.to_string().contains("unknown variant `lifo`"),
            "unexpected error: {error}"
        );

        Ok(())
    }

    /// Eviction removes the oldest, so the surviving set is {2,3,4}, which is then consumed oldest
    /// first (all within one generation).
    #[test]
    fn surb_ring_buffer_should_drop_oldest_items_when_capacity_is_reached() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3);
        rb.push([([1u8; 8], 0)], 0);
        rb.push([([2u8; 8], 0)], 0);
        rb.push([([3u8; 8], 0)], 0);
        rb.push([([4u8; 8], 0)], 0);

        let expected: [HoprSurbId; 3] = [[2u8; 8], [3u8; 8], [4u8; 8]];
        for (i, expected_id) in expected.into_iter().enumerate() {
            let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
            assert_eq!(expected_id, popped.id, "unexpected id at index {i}");
            assert_eq!(expected.len() - 1 - i, popped.remaining, "unexpected remaining");
        }

        assert!(rb.pop_any().is_none(), "buffer should be drained");

        Ok(())
    }

    /// Two SURBs pushed as {1, 2} are handed out 1 then 2 — and the same order holds for a fresh
    /// batch pushed after the buffer drains. (That the configuration still reports FIFO is asserted
    /// separately by `surb_store_config_should_default_to_fifo`.)
    #[test]
    fn surb_ring_buffer_should_consume_oldest_first() -> anyhow::Result<()> {
        let expected: [HoprSurbId; 2] = [[1u8; 8], [2u8; 8]];
        let rb = SurbRingBuffer::new(5);

        assert_eq!(1, rb.push([([1u8; 8], 0)], 0).retained);
        assert_eq!(2, rb.push([([2u8; 8], 0)], 0).retained);

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!(expected[0], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!(expected[1], popped.id);
        assert_eq!(0, popped.remaining);

        // A fresh batch after draining is consumed in the same order.
        assert_eq!(2, rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0).retained);
        assert_eq!(expected[0], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!(expected[1], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_skip_entries_failing_the_predicate() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0), ([3u8; 8], 0)], 0);

        // Only the middle entry is acceptable, so the two rejected ones must be discarded.
        let popped = rb
            .pop_next_valid(|id, _| id == &[2u8; 8])
            .ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);

        // The rejected entries are gone, not merely skipped over.
        assert_eq!(1, popped.remaining);
        assert!(rb.pop_next_valid(|id, _| id == &[2u8; 8]).is_none());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_return_none_when_no_entry_satisfies_the_predicate() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0);

        assert!(rb.pop_next_valid(|_, _| false).is_none());
        // The buffer is fully drained by the exhaustive search.
        assert!(rb.pop_any().is_none());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_report_no_eviction_below_capacity() {
        let rb = SurbRingBuffer::new(4);

        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 0,
                evicted_shares: 0
            },
            rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0)
        );
        assert_eq!(
            SurbInsertOutcome {
                retained: 4,
                evicted: 0,
                evicted_shares: 0
            },
            rb.push([([3u8; 8], 0), ([4u8; 8], 0)], 0)
        );
    }

    /// Overflow is otherwise entirely silent — the buffer drops its oldest entry and the caller sees
    /// only a successful push. The count is what lets the layers above notice that SURBs (and the
    /// PIX shares riding on them) are being destroyed on arrival, so it has to be exact.
    #[test]
    fn surb_ring_buffer_should_count_evictions_past_capacity() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(2);

        let outcome = rb.push([([1u8; 8], 0), ([2u8; 8], 0), ([3u8; 8], 0)], 0);
        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 1,
                evicted_shares: 0
            },
            outcome,
            "a 3-element push into a 2-slot buffer drops exactly one, and it carried no share"
        );

        // The *oldest* is the one gone.
        let ids: Vec<_> = std::iter::from_fn(|| rb.pop_any().map(|p| p.id)).collect();
        assert!(
            !ids.contains(&[1u8; 8]),
            "the oldest entry must be the evicted one, got {ids:?}"
        );

        Ok(())
    }

    /// A buffer already at capacity evicts one per element pushed, however the pushes are grouped —
    /// the steady-state overflow that a counterparty producing faster than this side drains creates.
    #[test]
    fn surb_ring_buffer_should_count_evictions_across_separate_pushes() {
        let rb = SurbRingBuffer::new(2);
        assert_eq!(
            0,
            rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0).evicted,
            "precondition: full"
        );

        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 1,
                evicted_shares: 0
            },
            rb.push([([3u8; 8], 0)], 0)
        );
        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 2,
                evicted_shares: 0
            },
            rb.push([([4u8; 8], 0), ([5u8; 8], 0)], 0)
        );
    }

    /// A newer generation drops the SURBs it supersedes, but those were already unusable — the
    /// creator re-planned its own return path. Counting them as evictions would report capacity
    /// pressure that does not exist, and the count feeds the eviction summary (and the
    /// counterparty-facing `num_evicted_surbs`) that exists to flag over-production. The same holds for
    /// a late older-generation batch, which is discarded without ever entering the buffer.
    #[test]
    fn surb_ring_buffer_should_not_count_superseded_generations_as_evictions() {
        let rb = SurbRingBuffer::new(2);
        assert_eq!(
            0,
            rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0).evicted,
            "precondition: full at generation 0"
        );

        // A newer generation clears the full buffer before inserting: two SURBs go, none of them to
        // capacity pressure.
        assert_eq!(
            SurbInsertOutcome {
                retained: 1,
                evicted: 0,
                evicted_shares: 0
            },
            rb.push([([3u8; 8], 0)], 1),
            "a supersede-and-clear is not an overflow"
        );

        // A late batch from the superseded generation is dropped whole, again without overflowing.
        assert_eq!(
            SurbInsertOutcome {
                retained: 1,
                evicted: 0,
                evicted_shares: 0
            },
            rb.push([([4u8; 8], 0), ([5u8; 8], 0)], 0),
            "a discarded older-generation batch is not an overflow"
        );
    }

    /// The buffer grows with occupancy, so it does reallocate on the way up to its ceiling — that
    /// is the point of
    /// [`surb_ring_buffer_must_allocate_with_occupancy_not_capacity`]. What must not happen is
    /// churn *afterwards*: once a pseudonym has filled its buffer, an unbounded stream of
    /// pushes and pops must not keep reallocating. So the high-water mark is reached first and
    /// sampled there, rather than at construction.
    #[test]
    fn surb_ring_buffer_should_not_reallocate_under_steady_overflow() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(8);

        for i in 0..8u32 {
            rb.push([(((i as u64).to_be_bytes()), 0)], 0);
        }
        let settled_capacity = rb.allocated();
        assert!(settled_capacity >= 8, "8 SURBs must actually fit");

        for i in 0..1_000u32 {
            rb.push([(((i as u64).to_be_bytes()), 0)], 0);
            if i % 3 == 0 {
                rb.pop_any();
            }
            assert!(rb.len() <= 8, "length exceeded capacity");
        }

        assert_eq!(settled_capacity, rb.allocated(), "buffer reallocated");

        Ok(())
    }

    /// The pseudonym a buffer is filed under is chosen by whoever sent the packet, and
    /// `insert_surbs` mints a buffer for any pseudonym that arrives carrying a SURB. So the cost
    /// must track what a buffer holds, not what it is allowed to hold — otherwise an
    /// unauthenticated peer reserves `rb_capacity` worth of memory per pseudonym it invents.
    ///
    /// Guards against a swap back to a structure that sizes itself at construction.
    #[test]
    fn surb_ring_buffer_must_allocate_with_occupancy_not_capacity() {
        const CAPACITY: usize = 100_000;
        let rb = SurbRingBuffer::<u64>::new(CAPACITY);

        let empty = rb.allocated();
        assert!(empty < CAPACITY / 100, "a buffer holding nothing reserved for {empty}");

        for i in 0..10u64 {
            rb.push([([i as u8; 8], i)], 0);
        }

        let allocated = rb.allocated();
        assert!(allocated >= 10, "10 SURBs must actually fit");
        assert!(
            allocated < CAPACITY / 100,
            "10 SURBs reserved for {allocated} — allocation is tracking capacity, not occupancy"
        );
    }

    #[test]
    fn surb_ring_buffer_should_not_pop_if_id_does_not_match() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        rb.push([([1u8; 8], 0)], 0);

        assert!(rb.pop_one_if_has_id(&[2u8; 8]).is_none());
        assert_eq!(
            [1u8; 8],
            rb.pop_one_if_has_id(&[1u8; 8])
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        Ok(())
    }

    /// A reply opener whose contents don't matter — the tests only ask whether one is *present* —
    /// so it is built once and cloned for every SURB.
    fn cheap_opener() -> ReplyOpener {
        ReplyOpener {
            sender_key: SecretKey16::random(),
            shared_secrets: Vec::new(),
        }
    }

    /// Floods `flood` reply openers (ids `0..flood`) for one pseudonym into a fresh client store
    /// capped at `max_openers`, then forces moka's lazy size-driven evictions to land so the
    /// retained set is observable. Returns the store and its pseudonym.
    fn flooded_client(max_openers: usize, flood: u64) -> (MemorySurbStore, HoprPseudonym) {
        let client = MemorySurbStore::new(SurbStoreConfig {
            max_openers_per_pseudonym: max_openers,
            ..Default::default()
        });
        let pseudonym = HoprPseudonym::random();
        let opener = cheap_opener();
        for i in 0..flood {
            let id: HoprSurbId = i.to_be_bytes();
            client.insert_reply_opener(HoprSenderId::from_pseudonym_and_id(&pseudonym, id), opener.clone());
        }
        client.pseudonym_openers.run_pending_tasks();
        if let Some(inner) = client.pseudonym_openers.get(&pseudonym) {
            inner.run_pending_tasks();
        }
        (client, pseudonym)
    }

    /// Upload-only return-path model.
    ///
    /// One session pseudonym uploads hard: it keeps minting SURBs (one reply opener stored per SURB
    /// on the *client*, one SURB stored on the *exit*) while the exit barely replies, so nothing is
    /// consumed and both stores run to capacity and start evicting. `flood` pairs are pushed, then
    /// the exit answers `replies` times — each answer pops a SURB the way the exit would and tries
    /// to open it on the client. Returns how many of those replies land on a reply opener the client
    /// has already evicted, i.e. how many replies the client cannot decrypt.
    ///
    /// The two stores are separate instances with their own configs, mirroring the two ends: the
    /// client cares only about `max_openers` (its reply-opener cache), the exit only about
    /// `rb_capacity` (its SURB ring).
    fn undecryptable_replies_after_upload_flood(
        max_openers: usize,
        rb_capacity: usize,
        flood: usize,
        replies: usize,
    ) -> usize {
        let (client, pseudonym) = flooded_client(max_openers, flood as u64);
        let exit = MemorySurbStore::new(SurbStoreConfig {
            rb_capacity,
            ..Default::default()
        });

        // A direct return path (chain length 1) is always usable, so `find_surb` never skips one for
        // an unrelated reason — the only thing under test is the opener's presence. The SURB value is
        // invariant across the flood, so build it once and clone (`HoprSurb` is a memcpy).
        let surb = surb_via(HoprKeyIdent::from(1u32), DIRECT).expect("valid surb fixture");
        for i in 0..flood as u64 {
            exit.insert_surbs(pseudonym, vec![(i.to_be_bytes(), surb.clone())]);
        }

        (0..replies)
            .filter(|_| {
                let found = exit
                    .find_surb(SurbMatcher::Pseudonym(pseudonym))
                    .expect("the exit still holds SURBs to reply with");
                client.find_reply_opener(&found.sender_id).is_none()
            })
            .count()
    }

    /// Baseline: while the reply-opener cache has not overflowed, every reply opens. Confirms the
    /// two-store plumbing (matching pseudonym + SURB id) before the overflow tests read anything
    /// into an eviction.
    #[test]
    fn replies_stay_decryptable_while_the_opener_cache_has_not_overflowed() {
        const MAX_OPENERS: usize = 7000;
        // Flood == capacity: nothing is evicted.
        let undecryptable = undecryptable_replies_after_upload_flood(MAX_OPENERS, 1050, MAX_OPENERS, 200);
        assert_eq!(
            0, undecryptable,
            "no reply should be undecryptable before the cache overflows"
        );
    }

    /// The fix, at the store level: on overflow the reply-opener cache sheds the stalest openers and
    /// keeps the newest, matching the exit's newest-SURB ring (the `lru()` rationale is on
    /// [`MemorySurbStore::insert_reply_opener`]). This asserts that directly — after overflowing a
    /// 7 000-cap cache with 21 000 openers, the oldest ids are gone and the newest are present.
    #[test]
    fn a_sustained_upload_keeps_the_newest_reply_openers_and_sheds_the_stalest() {
        const MAX_OPENERS: usize = 7000;
        const FLOOD: u64 = 21_000;
        const EDGE: u64 = 1_050; // a slice at each end of the id range

        let (client, pseudonym) = flooded_client(MAX_OPENERS, FLOOD);
        let inner = client
            .pseudonym_openers
            .get(&pseudonym)
            .expect("the pseudonym's opener cache exists");

        let present =
            |lo: u64, hi: u64| -> u64 { (lo..hi).filter(|i| inner.contains_key(&i.to_be_bytes())).count() as u64 };

        assert_eq!(0, present(0, EDGE), "the stalest openers are shed");
        assert_eq!(
            EDGE,
            present(FLOOD - EDGE, FLOOD),
            "the freshest openers — the ones the exit's newest SURBs need — are kept"
        );
    }

    /// End-to-end at the store level: with the opener cache keeping its newest entries, a sustained
    /// upload no longer strands the return path at the deployed configuration. At production
    /// proportions (opener cache larger than the exit's SURB ring) every reply opens.
    ///
    /// The one case that still strands replies is a misconfiguration: an exit whose SURB ring is
    /// *larger* than the opener cache, so it pops the oldest SURBs whose openers fall outside the
    /// smaller opener window. It is neither deployed nor sane, and is asserted here to document the
    /// boundary of the fix rather than to endorse it. (Before the fix, every one of these cases
    /// stranded all `REPLIES`; see this test's history.)
    ///
    /// In the inverted case the exit retains the newest `rb_capacity` SURBs and the client the newest
    /// `max_openers` openers (`flood` overflows both), so the two ranges overlap only on the newest
    /// `max_openers` ids. The exit pops from the oldest end, so the first `rb_capacity - max_openers`
    /// pops fall outside that opener window; since `REPLIES <= rb_capacity - max_openers`
    /// (200 ≤ 14 000), every tested reply is undecryptable.
    #[rstest]
    #[case::production_fifo(7000, 1050, 21_000, 0)]
    #[case::inverted_fifo(1000, 15_000, 20_000, 200)]
    fn a_sustained_upload_keeps_the_return_path_alive_at_the_deployed_config(
        #[case] max_openers: usize,
        #[case] rb_capacity: usize,
        #[case] flood: usize,
        #[case] expected_undecryptable: usize,
    ) {
        const REPLIES: usize = 200;
        let undecryptable = undecryptable_replies_after_upload_flood(max_openers, rb_capacity, flood, REPLIES);
        assert_eq!(
            expected_undecryptable, undecryptable,
            "unexpected undecryptable-reply count at openers={max_openers}, rb={rb_capacity}"
        );
    }

    /// `pop_one_if_has_id` checks only the popping end, which is the front (oldest). Given {1, 2}, a
    /// match for the id behind it is not popped; the id at the front is.
    #[test]
    fn surb_ring_buffer_should_check_the_popping_end_for_an_exact_id() -> anyhow::Result<()> {
        let (other_end_id, popping_end_id) = ([2u8; 8], [1u8; 8]);
        let rb = SurbRingBuffer::new(5);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0);

        assert!(rb.pop_one_if_has_id(&other_end_id).is_none());
        assert_eq!(
            popping_end_id,
            rb.pop_one_if_has_id(&popping_end_id)
                .ok_or(anyhow::anyhow!("expected pop"))?
                .id
        );

        Ok(())
    }

    // --- generation-tagged consumption -----------------------------------------------------------

    #[test]
    fn surb_ring_buffer_should_drop_the_previous_generation_when_a_newer_one_arrives() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 3);
        assert_eq!(2, rb.len());

        // A newer generation supersedes the old one: the buffer is cleared before inserting, so only
        // the new-generation SURB remains and it is what the next pop returns.
        rb.push([([9u8; 8], 0)], 4);
        assert_eq!(1, rb.len(), "the superseded generation must be dropped, not retained");
        assert_eq!(Some(4), rb.generation());

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([9u8; 8], popped.id, "only the new generation may be handed out");
        assert!(rb.pop_any().is_none(), "no stale SURB may remain");

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_append_within_the_same_generation() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64);
        rb.push([([1u8; 8], 0)], 7);
        rb.push([([2u8; 8], 0)], 7);
        assert_eq!(2, rb.len(), "same-generation batches accumulate");
        assert_eq!([1u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!([2u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_discard_a_stale_older_generation_batch() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64);
        rb.push([([9u8; 8], 0)], 5);

        // A late/reordered batch from an older generation must not reintroduce stale SURBs.
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 4);
        assert_eq!(1, rb.len(), "the older-generation batch must be discarded");
        assert_eq!(Some(5), rb.generation());
        assert_eq!([9u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    /// A generation is an RFC-1982 `u8` serial, so it wraps 255 -> 0. The wrap must trigger the same
    /// supersede-and-clear as any other newer generation, not be misread as an older batch.
    #[test]
    fn surb_ring_buffer_should_switch_generations_across_the_u8_wrap() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 255);
        assert_eq!(2, rb.len());

        // 0 is newer than 255 across the wrap: drop the old generation and switch to the new one.
        rb.push([([9u8; 8], 0)], 0);
        assert_eq!(
            1,
            rb.len(),
            "the wrapped-around newer generation must supersede the previous one"
        );
        assert_eq!(Some(0), rb.generation());
        assert_eq!([9u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert!(rb.pop_any().is_none(), "no stale SURB may survive the wrap");

        Ok(())
    }

    /// Several return-path re-plans in a row, walking up to and across the wrap (253 -> 1). Each newer
    /// generation supersedes the one before, so the eldest is dropped at every step and the buffer
    /// holds only the newest batch — re-plans alone can never accumulate stale generations and so can
    /// never overflow capacity.
    #[test]
    fn surb_ring_buffer_should_supersede_across_consecutive_replans_including_the_wrap() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64);

        for (i, generation) in [253u8, 254, 255, 0, 1].into_iter().enumerate() {
            rb.push([([i as u8; 8], 0)], generation);
            assert_eq!(
                1,
                rb.len(),
                "each re-plan must leave only its own batch, dropping the previous"
            );
            assert_eq!(Some(generation), rb.generation());
        }

        // Only the final generation's SURB (index 4, generation 1) survives.
        assert_eq!([4u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert!(rb.pop_any().is_none());

        Ok(())
    }

    /// With several re-plans in flight at once, their batches can arrive out of order. The buffer
    /// keeps the highest generation it has seen and discards a later-arriving older batch, so the exit
    /// never falls back to a superseded path even when three generations touch the buffer.
    #[test]
    fn surb_ring_buffer_should_keep_the_highest_generation_when_replans_arrive_out_of_order() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64);
        rb.push([([1u8; 8], 0)], 7); // generation 7
        rb.push([([2u8; 8], 0)], 9); // generation 9 (two re-plans later) supersedes 7
        rb.push([([3u8; 8], 0)], 8); // generation 8 arrives late, out of order -> discarded as older
        assert_eq!(1, rb.len(), "a late older-generation batch must not be reintroduced");
        assert_eq!(Some(9), rb.generation());
        assert_eq!([2u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    /// The exit's discard decision must be defined right at the serial-space boundary. A batch a full
    /// half-space ahead (+128) is deliberately NOT taken as newer, so the exit keeps its current SURBs
    /// rather than switch to an ambiguously-ordered generation. This cannot arise while adjacent
    /// generations are in flight (the design's premise); the test pins the boundary so it stays
    /// intentional rather than accidental.
    #[test]
    fn surb_ring_buffer_should_not_switch_on_a_half_serial_space_jump() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64);
        rb.push([([1u8; 8], 0)], 10);

        rb.push([([2u8; 8], 0)], 10u8.wrapping_add(128));
        assert_eq!(1, rb.len(), "a half-space jump must not be taken as newer");
        assert_eq!(
            Some(10),
            rb.generation(),
            "the current generation is retained at the boundary"
        );
        assert_eq!(
            [1u8; 8],
            rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id,
            "the exit keeps its current SURB rather than the ambiguous one"
        );

        Ok(())
    }

    /// A newer generation clears the buffer *before* inserting, so even a batch large enough to
    /// overflow capacity holds only new-generation SURBs — a superseded generation can never occupy a
    /// slot the live return path needs.
    #[test]
    fn surb_ring_buffer_should_clear_before_capacity_eviction_on_a_newer_generation() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0), ([3u8; 8], 0)], 0); // fill to capacity at generation 0
        assert_eq!(3, rb.len());

        // Newer generation, a full batch: generation 0 is cleared first, then the new batch fills
        // from scratch — no generation-0 SURB survives to consume a slot.
        rb.push([([4u8; 8], 0), ([5u8; 8], 0), ([6u8; 8], 0)], 1);
        assert_eq!(3, rb.len(), "capacity is respected and no superseded SURB survives");
        assert_eq!(Some(1), rb.generation());
        for expected in [[4u8; 8], [5u8; 8], [6u8; 8]] {
            assert_eq!(expected, rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        }

        Ok(())
    }

    #[test]
    fn generation_serial_should_wrap_around() {
        // RFC-1982: 0 is newer than 255, and 255 is not newer than 0.
        assert!(generation_is_newer(0, 255), "0 must be newer than 255 across the wrap");
        assert!(
            !generation_is_newer(255, 0),
            "255 must not be newer than 0 across the wrap"
        );
        assert!(generation_is_newer(4, 3));
        assert!(!generation_is_newer(3, 3), "a generation is not newer than itself");

        // The comparison window is half the serial space: +127 is still newer, but +128 sits on the
        // ambiguity boundary and is deliberately NOT treated as newer. This is the cap on how far the
        // sender may advance between two batches the exit actually sees; adjacent generations — the
        // only case in flight — are nowhere near it, so the exit's discard decision stays well-defined.
        assert!(
            generation_is_newer(10u8.wrapping_add(127), 10),
            "+127 is inside the window"
        );
        assert!(
            !generation_is_newer(10u8.wrapping_add(128), 10),
            "+128 is the boundary and must not read as newer"
        );
    }

    /// The sending-side generation serial starts at 0 for an unseen pseudonym and each bump advances
    /// it by one (the value the encoder stamps onto the next minted batch).
    #[test]
    fn memory_surb_store_generation_should_start_at_zero_and_advance_on_bump() {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        assert_eq!(
            0,
            store.current_generation(&pseudonym),
            "an unseen pseudonym starts at generation 0"
        );
        assert_eq!(
            1,
            store.bump_generation(&pseudonym),
            "the first bump returns the new generation 1"
        );
        assert_eq!(
            1,
            store.current_generation(&pseudonym),
            "current_generation reflects the last bump"
        );
        assert_eq!(2, store.bump_generation(&pseudonym), "each bump advances by one");
        assert_eq!(2, store.current_generation(&pseudonym));
    }

    /// End-to-end at the store: a newer generation for a pseudonym drops the SURBs held for the old
    /// one, so a return-path change takes effect on the next reply (one-packet recovery).
    #[test]
    fn memory_surb_store_should_switch_to_the_newest_generation() -> anyhow::Result<()> {
        let relayer = HoprKeyIdent::from(1u32);
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(
            pseudonym,
            vec![
                ([1u8; 8], surb_gen(relayer, TWO_HOP, 0)?),
                ([2u8; 8], surb_gen(relayer, TWO_HOP, 0)?),
            ],
        );
        // The client re-plans the return path and mints a fresh batch at the next generation.
        store.insert_surbs(pseudonym, vec![([3u8; 8], surb_gen(relayer, TWO_HOP, 1)?)]);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("expected a usable SURB"))?;
        assert_eq!(
            [3u8; 8],
            found.sender_id.surb_id(),
            "must hand out the newest generation"
        );
        assert_eq!(0, found.remaining, "the superseded generation must have been dropped");
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none(),
            "no stale SURB may remain"
        );

        Ok(())
    }

    /// A batch is minted at a single generation by an honest creator, but it is parsed out of a
    /// counterparty-controlled payload. A batch whose SURBs disagree must not smuggle a superseded
    /// return path into the buffer the batch's generation labels: only the first SURB's generation
    /// is kept.
    #[test]
    fn memory_surb_store_should_reject_surbs_that_disagree_with_their_batch() -> anyhow::Result<()> {
        let relayer = HoprKeyIdent::from(1u32);
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        // The peer had already delivered a generation-0 batch that the mixed batch below supersedes.
        store.insert_surbs(pseudonym, vec![([1u8; 8], surb_gen(relayer, TWO_HOP, 0)?)]);

        let outcome = store.insert_surbs(
            pseudonym,
            vec![
                ([2u8; 8], surb_gen(relayer, TWO_HOP, 1)?),
                // Stale: belongs to the generation the batch itself supersedes.
                ([3u8; 8], surb_gen(relayer, TWO_HOP, 0)?),
            ],
        );
        assert_eq!(1, outcome.retained, "only the SURB matching the batch may be stored");
        assert_eq!(
            0, outcome.evicted,
            "a mismatched SURB is not capacity pressure and must not be reported as eviction"
        );
        assert_eq!(0, outcome.evicted_shares, "and it cannot have cost a share either");

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("expected a usable SURB"))?;
        assert_eq!(
            [2u8; 8],
            found.sender_id.surb_id(),
            "must hand out the SURB of the batch's own generation"
        );
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none(),
            "no SURB of a superseded generation may remain"
        );

        Ok(())
    }

    // --- share-bearing SURBs first -----------------------------------------------------------------

    /// The Entry emits the PIX shares of a cycle first and share-less SURBs after, and a share reaches
    /// the reconstructor only when its SURB is *used*. So shares go out ahead of share-less SURBs
    /// however the two were interleaved on arrival, each tier oldest first.
    #[test]
    fn surb_ring_buffer_should_pop_share_bearing_surbs_before_share_less_ones() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(8);
        // One batch, as a packet delivers them: each SURB is classified on its own.
        rb.push([plain_surb(1)?, share_surb(2)?, plain_surb(3)?, share_surb(4)?], 0);

        assert_eq!(vec![2, 4, 1, 3], drain_ids(&rb));

        Ok(())
    }

    /// A share-less SURB is only a return path, so it is what a full buffer gives up first, oldest
    /// first — even when a share-bearing SURB is older still.
    #[test]
    fn surb_ring_buffer_should_evict_share_less_surbs_before_shares() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3);
        assert_eq!(
            SurbInsertOutcome {
                retained: 3,
                evicted: 0,
                evicted_shares: 0
            },
            rb.push([share_surb(1)?, plain_surb(2)?, share_surb(3)?], 0)
        );

        assert_eq!(
            SurbInsertOutcome {
                retained: 3,
                evicted: 1,
                evicted_shares: 0
            },
            rb.push([share_surb(4)?], 0),
            "the share-less SURB goes, not the older share, so no share is lost"
        );

        assert_eq!(vec![1, 3, 4], drain_ids(&rb));

        Ok(())
    }

    /// A share-bearing SURB is a return path *and* a share nothing can replace, so a share-less
    /// newcomer is refused rather than evict one. The refusal is capacity pressure all the same, so
    /// it is reported as an eviction — but not as a lost share, because none was lost.
    #[test]
    fn surb_ring_buffer_should_refuse_a_share_less_surb_rather_than_evict_a_share() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(2);
        rb.push([share_surb(1)?, share_surb(2)?], 0);

        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 1,
                evicted_shares: 0
            },
            rb.push([plain_surb(3)?], 0)
        );

        assert_eq!(vec![1, 2], drain_ids(&rb), "both shares must survive");

        Ok(())
    }

    /// With nothing share-less left to give up, shares fall back to the historical rule: the oldest
    /// goes. That is the one case where an eviction destroys a share, and it is counted as such.
    #[test]
    fn surb_ring_buffer_should_evict_the_oldest_share_once_no_share_less_surbs_remain() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(2);
        rb.push([share_surb(1)?, share_surb(2)?], 0);

        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 1,
                evicted_shares: 1
            },
            rb.push([share_surb(3)?], 0)
        );

        assert_eq!(vec![2, 3], drain_ids(&rb));

        Ok(())
    }

    /// The eviction order is applied per SURB, not per batch: a packet's SURBs can straddle the
    /// share to share-less transition, and each one meets the buffer as the one before left it.
    /// Of the two evictions in the second batch only the one that takes a share is a lost share.
    #[test]
    fn surb_ring_buffer_should_apply_the_eviction_order_to_each_surb_of_a_batch() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(2);

        // Share 3 evicts the share-less 1 that sits ahead of it.
        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 1,
                evicted_shares: 0
            },
            rb.push([plain_surb(1)?, share_surb(2)?, share_surb(3)?], 0)
        );

        // Only shares are held now: the share-less 4 is refused, then share 5 evicts the oldest share.
        assert_eq!(
            SurbInsertOutcome {
                retained: 2,
                evicted: 2,
                evicted_shares: 1
            },
            rb.push([plain_surb(4)?, share_surb(5)?], 0)
        );

        assert_eq!(vec![3, 5], drain_ids(&rb));

        Ok(())
    }

    /// `remaining` counts the whole buffer, not the tier the SURB came from: the SURB distress flag is
    /// raised from it, and must not depend on how the held SURBs happen to be split between the tiers.
    #[test]
    fn surb_ring_buffer_should_report_remaining_across_both_tiers() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(8);
        rb.push([share_surb(1)?, plain_surb(2)?, plain_surb(3)?], 0);
        assert_eq!(3, rb.len());

        for expected_remaining in [2, 1, 0] {
            let popped = rb.pop_any().context("expected pop")?;
            assert_eq!(expected_remaining, popped.remaining);
        }

        Ok(())
    }

    /// The tier sizes are what `len` is made of, reported as `(share-bearing, share-less)`: the first
    /// is how many shares are still to be delivered, the second how many plain return paths sit
    /// behind them.
    #[test]
    fn surb_ring_buffer_should_report_the_size_of_each_tier() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(8);
        assert_eq!((0, 0), rb.tier_lens(), "an empty buffer holds nothing in either tier");

        rb.push([plain_surb(1)?, share_surb(2)?, plain_surb(3)?, plain_surb(4)?], 0);
        assert_eq!((1, 3), rb.tier_lens());
        assert_eq!(4, rb.len(), "the tiers add up to the buffer's length");

        // Shares are handed out first, so the share tier empties before the share-less one is touched.
        rb.pop_any().context("expected the share-bearing SURB")?;
        assert_eq!((0, 3), rb.tier_lens());
        rb.pop_any().context("expected a share-less SURB")?;
        assert_eq!((0, 2), rb.tier_lens());

        // A newer generation supersedes both tiers.
        rb.push([share_surb(5)?], 1);
        assert_eq!((1, 0), rb.tier_lens());

        Ok(())
    }

    /// The store exposes the split per pseudonym, so a layer that only holds a [`SurbStore`] can trace
    /// it; a pseudonym without a buffer has none to report.
    #[test]
    fn memory_surb_store_should_report_the_size_of_each_tier_per_pseudonym() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        assert_eq!(None, store.tier_lens(&pseudonym), "no SURBs, no buffer, no tiers");

        store.insert_surbs(pseudonym, vec![plain_surb(1)?, share_surb(2)?, plain_surb(3)?]);
        assert_eq!(Some((1, 2)), store.tier_lens(&pseudonym));
        assert_eq!(
            None,
            store.tier_lens(&HoprPseudonym::random()),
            "another pseudonym has its own buffer"
        );

        // The decoder holds the store behind an `Arc`, and the trait's defaulted method must still reach
        // the store's own implementation through it rather than answer `None`.
        let shared = Arc::new(store);
        assert_eq!(Some((1, 2)), SurbStore::tier_lens(&shared, &pseudonym));

        Ok(())
    }

    /// An exact-ID pop inspects only the end the next pop would take. That is now the share-bearing
    /// tier while it holds anything, so a share-less SURB is not poppable by ID ahead of a share.
    #[test]
    fn surb_ring_buffer_should_check_the_popping_end_of_the_next_tier_for_an_exact_id() -> anyhow::Result<()> {
        let (plain_id, share_id) = ([1u8; 8], [2u8; 8]);
        let rb = SurbRingBuffer::new(8);
        rb.push([plain_surb(1)?, share_surb(2)?], 0);

        assert!(
            rb.pop_one_if_has_id(&plain_id).is_none(),
            "the share-bearing SURB is next, so the share-less one is not"
        );
        assert_eq!(
            share_id,
            rb.pop_one_if_has_id(&share_id)
                .context("the share-bearing SURB is next")?
                .id
        );
        assert_eq!(
            plain_id,
            rb.pop_one_if_has_id(&plain_id)
                .context("with the share gone, the share-less SURB is next")?
                .id
        );

        Ok(())
    }

    /// A newer generation supersedes the SURBs of *both* tiers: a share-bearing SURB for the old return
    /// path is as unusable as a share-less one.
    #[test]
    fn surb_ring_buffer_should_clear_both_tiers_on_a_newer_generation() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(8);
        rb.push([plain_surb(1)?, share_surb(2)?], 0);
        assert_eq!(2, rb.len());

        rb.push([share_surb(3)?], 1);

        assert_eq!(1, rb.len(), "neither tier may keep a superseded SURB");
        assert_eq!(vec![3], drain_ids(&rb), "only the newer generation may be handed out");

        Ok(())
    }

    /// End-to-end at the store: SURBs are classified by the share sealed in them, so the one that
    /// carries a share is handed out first although it did not arrive first.
    #[test]
    fn memory_surb_store_should_hand_out_share_bearing_surbs_first() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();

        store.insert_surbs(pseudonym, vec![plain_surb(1)?, share_surb(2)?, plain_surb(3)?]);

        let first = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .context("expected a usable SURB")?;
        assert_eq!([2u8; 8], first.sender_id.surb_id(), "the share-bearing SURB goes first");
        assert_eq!(2, first.remaining);

        let mut order_seen = vec![first.sender_id.surb_id()[0]];
        while let Some(found) = store.find_surb(SurbMatcher::Pseudonym(pseudonym)) {
            order_seen.push(found.sender_id.surb_id()[0]);
        }
        assert_eq!(vec![2, 1, 3], order_seen);

        Ok(())
    }

    /// The classification reads the share block of the SURB's receiver-only data: an all-zero block
    /// is a share-less SURB, and anything else names the SSA its share belongs to.
    #[test]
    fn hopr_surb_share_info_should_treat_a_zero_share_as_share_less() -> anyhow::Result<()> {
        let relayer = HoprKeyIdent::from(1u32);

        assert_eq!(
            None,
            surb_via(relayer, TWO_HOP)?.ssa_index(),
            "a zeroed share block carries no share"
        );
        assert_eq!(SsaIndex::new(7), surb_share(relayer, 7)?.ssa_index());

        Ok(())
    }

    const REPORT_INTERVAL: Duration = Duration::from_secs(60);

    impl EvictionStats {
        /// Counts one eviction at the given time rather than now, and returns the closed interval's counts
        /// if it closed one. Most tests count evictions singly, against a clock they control.
        fn record_at(&self, cache: EvictedCache, cause: RemovalCause, now: Instant) -> Option<EvictionReport> {
            self.record_many_at(cache, cause, 1, now)
        }
    }

    fn eviction_stats(warn_threshold: u64) -> EvictionStats {
        EvictionStats::new(&SurbStoreConfig {
            eviction_report_interval: REPORT_INTERVAL,
            eviction_report_threshold: warn_threshold,
            ..Default::default()
        })
    }

    #[test]
    fn eviction_stats_should_not_report_before_the_interval_elapses() {
        let stats = eviction_stats(0);
        let t0 = Instant::now();
        for _ in 0..10 {
            assert!(
                stats
                    .record_at(EvictedCache::ReplyOpener, RemovalCause::Expired, t0)
                    .is_none()
            );
        }

        let just_before = t0 + REPORT_INTERVAL - Duration::from_millis(1);
        assert!(
            stats
                .record_at(EvictedCache::ReplyOpener, RemovalCause::Size, just_before)
                .is_none()
        );
    }

    #[test]
    fn eviction_stats_should_report_counts_per_cache_and_cause_once_the_interval_elapses() -> anyhow::Result<()> {
        let stats = eviction_stats(0);
        let t0 = Instant::now();
        for _ in 0..3 {
            stats.record_at(EvictedCache::ReplyOpener, RemovalCause::Expired, t0);
        }
        for _ in 0..2 {
            stats.record_at(EvictedCache::ReplyOpener, RemovalCause::Size, t0);
        }
        stats.record_at(EvictedCache::SurbRing, RemovalCause::Replaced, t0);
        stats.record_at(EvictedCache::SurbRing, RemovalCause::Explicit, t0);

        let report = stats
            .record_at(EvictedCache::Generation, RemovalCause::Expired, t0 + REPORT_INTERVAL)
            .context("the interval has elapsed, so a report is due")?;

        assert_eq!(
            report.counts(EvictedCache::ReplyOpener),
            EvictionCounts {
                expired: 3,
                size: 2,
                replaced: 0
            }
        );
        assert_eq!(
            report.counts(EvictedCache::SurbRing),
            EvictionCounts {
                expired: 0,
                size: 0,
                replaced: 1
            },
            "explicit removals are not evictions"
        );
        assert_eq!(
            report.counts(EvictedCache::Generation),
            EvictionCounts::default(),
            "the eviction that closes an interval belongs to the next one"
        );
        assert_eq!(report.counts(EvictedCache::ReplyOpenerBatch), EvictionCounts::default());

        let next = stats
            .record_at(EvictedCache::SurbRing, RemovalCause::Explicit, t0 + 2 * REPORT_INTERVAL)
            .context("second interval has elapsed")?;
        assert_eq!(
            next.counts(EvictedCache::Generation),
            EvictionCounts {
                expired: 1,
                size: 0,
                replaced: 0
            }
        );
        Ok(())
    }

    #[test]
    fn eviction_stats_should_start_a_fresh_interval_after_reporting() -> anyhow::Result<()> {
        let stats = eviction_stats(0);
        let t0 = Instant::now();
        let t1 = t0 + REPORT_INTERVAL;
        stats.record_at(EvictedCache::SurbRing, RemovalCause::Expired, t0);
        stats
            .record_at(EvictedCache::SurbRing, RemovalCause::Expired, t1)
            .context("first report")?;

        assert!(
            stats
                .record_at(EvictedCache::SurbRing, RemovalCause::Expired, t1)
                .is_none(),
            "the new interval has only just started"
        );
        let report = stats
            .record_at(EvictedCache::SurbRing, RemovalCause::Expired, t1 + REPORT_INTERVAL)
            .context("second report")?;
        assert_eq!(
            report.counts(EvictedCache::SurbRing).total(),
            2,
            "only evictions since the last report count"
        );
        Ok(())
    }

    /// A push can drop many SURBs at once, and each is one eviction: `record_many` books them under
    /// the one lock instead of taking it once per SURB.
    #[test]
    fn eviction_stats_should_count_many_evictions_recorded_at_once() -> anyhow::Result<()> {
        let stats = eviction_stats(0);
        let t0 = Instant::now();
        stats.record_many_at(EvictedCache::RingPlain, RemovalCause::Size, 7, t0);
        stats.record_many_at(EvictedCache::RingPlain, RemovalCause::Size, 3, t0);
        stats.record_many_at(EvictedCache::RingShare, RemovalCause::Size, 2, t0);
        stats.record_many_at(EvictedCache::RingShare, RemovalCause::Explicit, 5, t0);
        stats.record_many_at(EvictedCache::SurbRing, RemovalCause::Size, 0, t0);

        let report = stats
            .record_at(EvictedCache::SurbRing, RemovalCause::Explicit, t0 + REPORT_INTERVAL)
            .context("the interval has elapsed, so a report is due")?;

        assert_eq!(
            report.counts(EvictedCache::RingPlain),
            EvictionCounts {
                expired: 0,
                size: 10,
                replaced: 0
            }
        );
        assert_eq!(
            report.counts(EvictedCache::RingShare),
            EvictionCounts {
                expired: 0,
                size: 2,
                replaced: 0
            },
            "explicit removals are not evictions, however many of them there are"
        );
        assert_eq!(
            report.counts(EvictedCache::SurbRing),
            EvictionCounts::default(),
            "recording zero evictions counts nothing"
        );
        Ok(())
    }

    /// `insert_surbs` reports every push, evictions or not, and most pushes evict nothing: a count of
    /// zero is not an eviction, so it neither takes the lock nor closes an interval.
    #[test]
    fn eviction_stats_should_ignore_a_count_of_zero() {
        let stats = eviction_stats(0);
        let t0 = Instant::now();
        assert!(
            stats
                .record_many_at(EvictedCache::RingPlain, RemovalCause::Size, 0, t0 + REPORT_INTERVAL)
                .is_none(),
            "nothing was evicted, so nothing closes the interval"
        );
        assert_eq!(
            EvictionCounts::default(),
            stats.state.lock().counts[EvictedCache::RingPlain as usize]
        );
    }

    /// The ring-buffer caches are new *values* of the `cache` label, so the metric names, their
    /// descriptions and their label keys — all that METRICS.md documents — are unchanged.
    #[test]
    fn evicted_cache_should_label_ring_overflows_by_tier() {
        assert_eq!("ring_plain", <&'static str>::from(EvictedCache::RingPlain));
        assert_eq!("ring_share", <&'static str>::from(EvictedCache::RingShare));
        assert_eq!("ring_share", EvictedCache::RingShare.to_string());
    }

    /// The interval counters are indexed by `cache as usize`, so every cache must be listed in
    /// [`EvictedCache::ALL`] exactly once, at the slot its discriminant names.
    #[test]
    fn evicted_cache_all_should_list_every_cache_at_its_own_slot() {
        let slots: Vec<usize> = EvictedCache::ALL.iter().map(|cache| *cache as usize).collect();
        assert_eq!((0..EvictedCache::ALL.len()).collect::<Vec<_>>(), slots);
    }

    /// A lost share is permanent, and the redundancy budget that absorbs losses is finite, so even
    /// one is worth a warning. Dropping share-less SURBs is routine under gate bypass (the sender
    /// outproduces what the Exit spends), so those keep the configured threshold like every other cache.
    #[rstest]
    #[case::a_single_lost_share(EvictedCache::RingShare, 1, true)]
    #[case::share_less_at_the_threshold(EvictedCache::RingPlain, 5, false)]
    #[case::share_less_above_the_threshold(EvictedCache::RingPlain, 6, true)]
    fn eviction_report_should_warn_about_any_lost_share_but_only_about_many_share_less_drops(
        #[case] cache: EvictedCache,
        #[case] evictions: u64,
        #[case] expected: bool,
    ) -> anyhow::Result<()> {
        let stats = eviction_stats(5);
        let t0 = Instant::now();
        stats.record_many_at(cache, RemovalCause::Size, evictions, t0);
        let report = stats
            .record_at(EvictedCache::SurbRing, RemovalCause::Explicit, t0 + REPORT_INTERVAL)
            .context("report due")?;

        assert_eq!(expected, report.exceeds_threshold(cache));

        let recorder = Arc::new(RecordingSubscriber::default());
        tracing::subscriber::with_default(recorder.clone(), || report.log());
        assert_eq!(u64::from(expected), recorder.warnings.load(Ordering::Relaxed));
        assert_eq!(u64::from(!expected), recorder.debugs.load(Ordering::Relaxed));
        Ok(())
    }

    #[rstest]
    #[case::at_threshold(5, false)]
    #[case::above_threshold(6, true)]
    fn eviction_report_should_flag_only_caches_above_the_warn_threshold(
        #[case] evictions: u64,
        #[case] expected: bool,
    ) -> anyhow::Result<()> {
        let stats = eviction_stats(5);
        let t0 = Instant::now();
        for _ in 0..evictions {
            stats.record_at(EvictedCache::ReplyOpener, RemovalCause::Expired, t0);
        }

        let report = stats
            .record_at(EvictedCache::SurbRing, RemovalCause::Explicit, t0 + REPORT_INTERVAL)
            .context("report due")?;

        assert_eq!(report.exceeds_threshold(EvictedCache::ReplyOpener), expected);
        assert!(!report.exceeds_threshold(EvictedCache::SurbRing));
        assert!(
            !report.exceeds_threshold(EvictedCache::RingPlain) && !report.exceeds_threshold(EvictedCache::RingShare),
            "a cache that saw nothing is never flagged, not even one whose threshold is zero"
        );
        Ok(())
    }

    /// Counts events per level; `enabled` is always true so the logging macros' bodies actually run.
    #[derive(Default)]
    struct RecordingSubscriber {
        warnings: AtomicU64,
        debugs: AtomicU64,
    }

    impl tracing::Subscriber for RecordingSubscriber {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }

        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }

        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

        fn event(&self, event: &tracing::Event<'_>) {
            match *event.metadata().level() {
                tracing::Level::WARN => self.warnings.fetch_add(1, Ordering::Relaxed),
                tracing::Level::DEBUG => self.debugs.fetch_add(1, Ordering::Relaxed),
                _ => 0,
            };
        }

        fn enter(&self, _: &tracing::span::Id) {}

        fn exit(&self, _: &tracing::span::Id) {}
    }

    #[test]
    fn eviction_report_should_log_one_line_per_cache_that_saw_evictions() -> anyhow::Result<()> {
        let stats = eviction_stats(5);
        let t0 = Instant::now();
        for _ in 0..6 {
            stats.record_at(EvictedCache::ReplyOpener, RemovalCause::Expired, t0);
        }
        stats.record_at(EvictedCache::SurbRing, RemovalCause::Size, t0);
        stats.record_many_at(EvictedCache::RingPlain, RemovalCause::Size, 2, t0);
        stats.record_at(EvictedCache::RingShare, RemovalCause::Size, t0);
        let report = stats
            .record_at(EvictedCache::SurbRing, RemovalCause::Explicit, t0 + REPORT_INTERVAL)
            .context("report due")?;

        let recorder = Arc::new(RecordingSubscriber::default());
        tracing::subscriber::with_default(recorder.clone(), || report.log());

        assert_eq!(
            recorder.warnings.load(Ordering::Relaxed),
            2,
            "the opener cache exceeded the threshold, and a lost share always warrants a warning"
        );
        assert_eq!(
            recorder.debugs.load(Ordering::Relaxed),
            2,
            "two caches saw evictions below the threshold; caches without evictions log nothing"
        );
        Ok(())
    }

    #[test]
    fn eviction_stats_should_log_when_a_real_eviction_closes_the_interval() -> anyhow::Result<()> {
        let stats = eviction_stats(0);
        {
            let mut state = stats.state.lock();
            state.started_at = Instant::now()
                .checked_sub(2 * REPORT_INTERVAL)
                .context("monotonic clock must be at least two intervals old")?;
            state.counts[EvictedCache::SurbRing as usize].expired += 1;
        }

        let recorder = Arc::new(RecordingSubscriber::default());
        tracing::subscriber::with_default(recorder.clone(), || {
            stats.record(EvictedCache::SurbRing, RemovalCause::Expired)
        });

        assert_eq!(recorder.warnings.load(Ordering::Relaxed), 1);
        Ok(())
    }

    #[test]
    fn memory_surb_store_should_count_evictions_from_every_cache() -> anyhow::Result<()> {
        // Both caps are floored at 1000 pseudonyms per cache, so 1500 distinct ones overflow all three.
        let store = MemorySurbStore::new(SurbStoreConfig {
            max_openers_per_pseudonym: 100,
            max_pseudonyms: 100,
            ..Default::default()
        });
        let relayer = HoprKeyIdent::from(1u32);
        let opener = cheap_opener();
        for _ in 0..1500 {
            let pseudonym = HoprPseudonym::random();
            store.insert_reply_opener(
                HoprSenderId::from_pseudonym_and_id(&pseudonym, [0u8; 8]),
                opener.clone(),
            );
            store.insert_surbs(pseudonym, vec![([0u8; 8], surb_via(relayer, DIRECT)?)]);
            store.bump_generation(&pseudonym);
        }
        store.pseudonym_openers.run_pending_tasks();
        store.surbs_per_pseudonym.run_pending_tasks();
        store.generations.run_pending_tasks();

        for cache in [
            EvictedCache::ReplyOpenerBatch,
            EvictedCache::SurbRing,
            EvictedCache::Generation,
        ] {
            let size_evictions = store.stats.state.lock().counts[cache as usize].size;
            assert!(
                size_evictions > 0,
                "{cache} overflowed, so its size evictions must be counted"
            );
        }
        Ok(())
    }

    /// Ring overflows go into the same per-interval summary as the other evictions, split by what
    /// they cost: a share-less SURB dropped or refused is only a return path (`ring_plain`), a
    /// share-bearing one dropped is a PIX share lost for good (`ring_share`).
    #[test]
    fn memory_surb_store_should_count_ring_overflows_by_tier() -> anyhow::Result<()> {
        // The ring capacity is floored at `MIN_SURB_RB_CAPACITY`, so that is the size to overflow.
        let capacity = MIN_SURB_RB_CAPACITY;
        let store = MemorySurbStore::new(SurbStoreConfig {
            rb_capacity: capacity,
            ..Default::default()
        });
        let pseudonym = HoprPseudonym::random();
        let (plain, share) = (plain_surb(1)?, share_surb(2)?);

        // A full ring: all shares but for one share-less SURB. Nothing overflows yet.
        let mut fill = vec![share.clone(); capacity - 1];
        fill.push(plain.clone());
        assert_eq!(
            SurbInsertOutcome {
                retained: capacity,
                evicted: 0,
                evicted_shares: 0
            },
            store.insert_surbs(pseudonym, fill)
        );

        // The share-less SURB makes way for a share: a return path lost, no share.
        assert_eq!(
            SurbInsertOutcome {
                retained: capacity,
                evicted: 1,
                evicted_shares: 0
            },
            store.insert_surbs(pseudonym, vec![share.clone()])
        );

        // Nothing but shares is held now, so two more shares evict the two oldest ones.
        assert_eq!(
            SurbInsertOutcome {
                retained: capacity,
                evicted: 2,
                evicted_shares: 2
            },
            store.insert_surbs(pseudonym, vec![share.clone(); 2])
        );

        // Share-less newcomers are refused rather than evict a share: three more return paths lost.
        assert_eq!(
            SurbInsertOutcome {
                retained: capacity,
                evicted: 3,
                evicted_shares: 0
            },
            store.insert_surbs(pseudonym, vec![plain; 3])
        );

        let report = store
            .stats
            .record_at(
                EvictedCache::SurbRing,
                RemovalCause::Explicit,
                Instant::now() + 2 * REPORT_INTERVAL,
            )
            .context("the interval has elapsed, so a report is due")?;
        assert_eq!(
            EvictionCounts {
                expired: 0,
                size: 4,
                replaced: 0
            },
            report.counts(EvictedCache::RingPlain),
            "one share-less SURB evicted and three refused"
        );
        assert_eq!(
            EvictionCounts {
                expired: 0,
                size: 2,
                replaced: 0
            },
            report.counts(EvictedCache::RingShare),
            "two shares lost"
        );
        assert!(
            report.exceeds_threshold(EvictedCache::RingShare) && !report.exceeds_threshold(EvictedCache::RingPlain),
            "any lost share warrants a warning, four dropped return paths do not"
        );

        Ok(())
    }

    #[test]
    fn memory_surb_store_should_count_size_evictions_of_flooded_reply_openers() {
        let (client, _) = flooded_client(MINIMUM_OPENERS_PER_PSEUDONYM, 3 * MINIMUM_OPENERS_PER_PSEUDONYM as u64);

        let size_evictions = client.stats.state.lock().counts[EvictedCache::ReplyOpener as usize].size;
        assert!(
            size_evictions > 0,
            "the flood overflowed the opener cache, so its size evictions must be counted"
        );
    }

    #[test]
    fn surb_store_config_should_reject_an_eviction_report_interval_below_the_minimum() {
        use validator::Validate;

        let too_low = SurbStoreConfig {
            eviction_report_interval: MINIMUM_EVICTION_REPORT_INTERVAL - Duration::from_millis(1),
            ..Default::default()
        };
        assert!(too_low.validate().is_err());
        assert!(SurbStoreConfig::default().validate().is_ok());
    }
}
