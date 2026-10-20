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
/// Subtracts `n` from the quarantine SURB budget, clamping at zero instead of wrapping.
///
/// Every refund site reads a ring length outside the ring's own lock, so a stale reader can subtract
/// SURBs that another site already accounted: with a bare `fetch_sub`, that interleaving drives the
/// counter past zero and wraps it to near `usize::MAX`, after which `counter + len > budget` holds for
/// every quarantine insert and the store refuses unadmitted ingress for good — a self-inflicted
/// denial that no later drain can undo, because the drain subtracts rather than resets. Saturating
/// keeps the design's "no resync" stance (an over-refund still under-counts, which the cache's own
/// `max_capacity` bounds) while making the failure self-correcting: the counter bottoms out at zero
/// and the next refund-free insert tops it back up.
fn refund_quarantine_budget(budget: &std::sync::atomic::AtomicUsize, n: usize) {
    if n == 0 {
        return;
    }
    let mut current = budget.load(std::sync::atomic::Ordering::Relaxed);
    while let Err(observed) = budget.compare_exchange_weak(
        current,
        current.saturating_sub(n),
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
    ) {
        current = observed;
    }
}

/// Lower bound on [`SurbStoreConfig::eviction_report_interval`], enforced by the config validator.
pub const MINIMUM_EVICTION_REPORT_INTERVAL: Duration = Duration::from_secs(1);
const MINIMUM_OPENER_PSEUDONYMS: usize = 1000;
const MINIMUM_OPENERS_PER_PSEUDONYM: usize = 1000;
const MINIMUM_OPENER_LIFETIME: Duration = Duration::from_secs(60);
const MIN_SURB_RB_CAPACITY: usize = 1024;
/// Lower bound on [`SurbStoreConfig::quarantine_max_pseudonyms`], enforced by the config validator.
///
/// Deliberately far below the reply-opener caches' 1000-pseudonym floor: those are written only by
/// locally-originated sends, while the quarantine is written by whatever pseudonym an unauthenticated
/// peer chose, so its bound is a small multiple of a packet's worth of SURBs, not a Session census.
const MINIMUM_QUARANTINE_PSEUDONYMS: usize = 64;
/// Lower bound on [`SurbStoreConfig::quarantine_rb_capacity`], enforced by the config validator.
///
/// One: a single `HoprPacket::Final` may carry up to `MAX_SURBS_IN_PACKET` SURBs, and each must be
/// able to enter its ring without evicting a batch-mate, so the meaningful floor is "at least one
/// packet", not "at least a Session's buffer".
const MINIMUM_QUARANTINE_RB_CAPACITY: usize = 1;
/// Lower bound on [`SurbStoreConfig::quarantine_surb_budget`], enforced by the config validator.
///
/// A backstop well above what the quarantine's own entry count and per-ring cap already bound
/// (1024 × 16 by default), so it only bites if those two are raised out of proportion to each other.
///
/// Test-only: the validator spells the floor as a literal, matching every other range rule in this
/// file, and the config test pins the two together by asserting the boundary either side of this.
#[cfg(test)]
const QUARANTINE_SURB_BUDGET_FLOOR: usize = 4096;

/// Mints the number naming each admission of a pseudonym, process-wide.
///
/// Every [`SurbStore::admit`] call takes the next value, so two admissions of the same pseudonym can
/// never share one — which is the property the deferred retire leans on. The counter is process-wide
/// rather than per pseudonym so that an `admit` does not have to look up the pseudonym's previous
/// number to advance it; `u64` is inexhaustible at any conceivable admission rate.
///
/// Zero is deliberately never minted: a session-side `AdmissionToken::NONE` carries zero, and a
/// retire presenting it must match nothing — it is what a teardown of a never-admitted Session
/// passes through the seam.
static ADMISSION_EPOCHS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Takes the next admission number. Never zero — see [`ADMISSION_EPOCHS`].
fn next_admission_epoch() -> u64 {
    ADMISSION_EPOCHS.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn validate_pseudonyms_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_SURB_LIFETIME {
        Err(ValidationError::new("pseudonyms_lifetime is too low"))
    } else {
        Ok(())
    }
}

fn validate_quarantine_lifetime(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_SURB_LIFETIME {
        Err(ValidationError::new("quarantine_lifetime is too low"))
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

fn default_quarantine_rb_capacity() -> usize {
    16
}

fn default_quarantine_max_pseudonyms() -> usize {
    1024
}

fn default_quarantine_lifetime() -> Duration {
    Duration::from_secs(60)
}

fn default_quarantine_surb_budget() -> usize {
    65_536
}

fn default_admitted_max_pseudonyms() -> usize {
    10_000
}

fn default_admitted_headroom_percent() -> u32 {
    50
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
    /// Retained for configuration back-compat only; no longer bounds the SURB ring cache.
    ///
    /// This used to size the single receiving-side SURB cache. The store now splits the receiving
    /// side into the quarantine (bounded by [`SurbStoreConfig::quarantine_max_pseudonyms`]) and the
    /// admitted class (bounded by [`SurbStoreConfig::admitted_max_pseudonyms`]), and this field
    /// sizes nothing on the SURB path. Lowering it no longer bounds SURB memory — set those two
    /// fields instead. It keeps its validator and serde default so an operator file that spells it
    /// out still loads, and a later major version can remove it.
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
    /// Per-ring cap for **quarantined** pseudonyms — those no established Session stands behind.
    ///
    /// Every `HoprPacket::Final` carrying SURBs reaches the store with no ticket verification on the
    /// `Final` arm, so the pseudonym the packet names is chosen by the sender. An unadmitted batch
    /// lands here, capped per ring so one pseudonym cannot reserve more than a packet's worth of
    /// return path (`MAX_SURBS_IN_PACKET` is 8, so 16 leaves headroom). A pseudonym is promoted out
    /// of quarantine once a Session over it is established and [`SurbStore::admit`] fires; anything
    /// left here expires after [`SurbStoreConfig::quarantine_lifetime`].
    ///
    /// Default is 16.
    #[default(default_quarantine_rb_capacity())]
    #[validate(range(min = 1, message = "quarantine_rb_capacity must be at least 1"))]
    #[cfg_attr(feature = "serde", serde(default = "default_quarantine_rb_capacity"))]
    pub quarantine_rb_capacity: usize,
    /// Capacity of the quarantine cache: how many distinct unadmitted pseudonyms may hold a ring.
    ///
    /// The quarantine is the only store an unauthenticated peer can write into, so it — not the
    /// admitted cache below — is what pseudonym churn must be absorbed by. 1024 ×
    /// `quarantine_rb_capacity` × ~400 B is a few MB. Beyond this many distinct pseudonyms, the
    /// least-recently-used quarantine rings are shed; their SURBs are unadmitted, so nothing with a
    /// Session behind it is lost.
    ///
    /// Default is 1024.
    #[default(default_quarantine_max_pseudonyms())]
    #[validate(range(min = 64, message = "quarantine_max_pseudonyms must be at least 64"))]
    #[cfg_attr(feature = "serde", serde(default = "default_quarantine_max_pseudonyms"))]
    pub quarantine_max_pseudonyms: usize,
    /// How long a quarantined ring survives with no SURBs arriving for it.
    ///
    /// Bounds the unadmitted class from the time side, the way [`SurbStoreConfig::pseudonyms_lifetime`]
    /// bounds the admitted one. Must cover the worst-case Session-initiation window — the span between
    /// the first `Final` packet minting a quarantine ring and the Session over it being established
    /// (which promotes the ring) — i.e. roughly `2 × initiation_timeout_base ×
    /// RoutingOptions::MAX_INTERMEDIATE_HOPS`; the 60 s default clears that at both the 500 ms and the
    /// 5 s base for up to 3 intermediate hops.
    ///
    /// Default is 60 seconds.
    #[default(default_quarantine_lifetime())]
    #[validate(custom(function = "validate_quarantine_lifetime"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_quarantine_lifetime", with = "humantime_serde")
    )]
    pub quarantine_lifetime: Duration,
    /// Backstop on the total number of unadmitted SURBs held across the whole quarantine cache.
    ///
    /// The quarantine cache's own entry and per-ring caps are the structural bound
    /// (`quarantine_max_pseudonyms × quarantine_rb_capacity`); this counter is drained at insert time
    /// and, when exhausted, sheds whole quarantine rings before refusing the batch. It exists so a
    /// peer cannot rely on eviction timing: the accounting is exact at the mutation sites, so the two
    /// bounds together cap the unadmitted class regardless of eviction scheduling. A backstop above
    /// the cache's own bound needs no precision — a refused batch costs only unadmitted SURBs, never
    /// one an established Session is holding.
    ///
    /// Default is 65 536.
    #[default(default_quarantine_surb_budget())]
    #[validate(range(min = 4096, message = "quarantine_surb_budget must be at least 4096"))]
    // The floor above is spelled as a literal rather than interpolating `QUARANTINE_SURB_BUDGET_FLOOR`,
    // matching every other range validator in this file; the test asserts the two agree.
    #[cfg_attr(feature = "serde", serde(default = "default_quarantine_surb_budget"))]
    pub quarantine_surb_budget: usize,
    /// Capacity of the **admitted** cache: how many pseudonyms with an established Session may hold a
    /// ring.
    ///
    /// Admission goes through [`SurbStore::admit`], which only an established Session triggers, so
    /// unlike the quarantine this bound cannot be inflated by an unauthenticated peer. It is a
    /// structural ceiling, not a per-insert counter: an admitted insert is never refused and an
    /// admitted ring is never evicted to satisfy a counter. This value must be at least the node's
    /// `maximum_managed_sessions`; the transport logs an error at startup if it is not, and an
    /// overflow here is reported as an `admitted_overflow` metric event.
    ///
    /// Default is 10 000.
    #[default(default_admitted_max_pseudonyms())]
    #[validate(range(min = 1, message = "admitted_max_pseudonyms must be at least 1"))]
    #[cfg_attr(feature = "serde", serde(default = "default_admitted_max_pseudonyms"))]
    pub admitted_max_pseudonyms: usize,
    /// Headroom the store adds over a Session's announced SURB target when sizing its admitted ring.
    ///
    /// A Session announces the buffer target it was configured with; the store admits a ring of
    /// `expected × (1 + admitted_headroom_percent / 100)`, floored at `quarantine_rb_capacity` and
    /// ceiled at [`SurbStoreConfig::rb_capacity`]. The inverse relation to `surb_buffer_target_ceiling`
    /// in `hopr-transport` (which sets the announced target at `rb_capacity × 2/3`) means an Entry
    /// announcing its maximum yields a full-`rb_capacity` admitted ring at the default 50 %.
    ///
    /// Default is 50.
    #[default(default_admitted_headroom_percent())]
    #[validate(range(min = 0, message = "admitted_headroom_percent must not be negative"))]
    #[cfg_attr(feature = "serde", serde(default = "default_admitted_headroom_percent"))]
    pub admitted_headroom_percent: u32,
}

/// Basic [`SurbStore`] implementation based on an in-memory cache.
///
/// This SURB store offers no persistence, and all SURBs and Reply Openers are lost once dropped.
///
/// The instance can be cheaply cloned.
#[derive(Clone)]
pub struct MemorySurbStore {
    pseudonym_openers: moka::sync::Cache<HoprPseudonym, moka::sync::Cache<HoprSurbId, ReplyOpener>>,
    /// Receiving-side rings for **admitted** pseudonyms: those an established Session stands behind,
    /// entered only through [`SurbStore::admit`] and left through [`SurbStore::retire`].
    ///
    /// This cache is also the single source of truth for admission: there is no separate
    /// admission-marker cache for `insert_surbs` to route on, because a marker in one cache pointing
    /// at a ring in another can desync (an eviction on one side, an insert on the other) and admits
    /// can race inserts. [`SurbStore::insert_surbs`] takes the entry here, so a pseudonym is
    /// admitted exactly as long as it holds one — even an empty one, which is what `admit` inserts
    /// so that standing is known before the first SURB arrives.
    ///
    /// The value carries the admission's number next to the ring, so a deferred
    /// [`SurbStore::retire`] can prove it is retiring the admission it was created for and not a
    /// successor's — see [`AdmittedRing`]. The number and the ring share one entry so the compare in
    /// `retire` and the removal are one atomic operation over the key.
    admitted_surbs_per_pseudonym: moka::sync::Cache<HoprPseudonym, AdmittedRing>,
    /// Receiving-side rings for **quarantined** pseudonyms: whatever pseudonym a `Final` packet chose
    /// and no Session has claimed.
    ///
    /// Physically separate from [`admitted_surbs_per_pseudonym`](Self::admitted_surbs_per_pseudonym)
    /// rather than a tier of it, so quarantine churn — pseudonym flooding, short TTI expiries, budget
    /// drains — can never evict an admitted ring. That is a structural guarantee: the two caches share
    /// no capacity or eviction machinery.
    quarantine_surbs: moka::sync::Cache<HoprPseudonym, SurbRingBuffer<HoprSurb>>,
    /// Count of SURBs currently held in [`quarantine_surbs`](Self::quarantine_surbs), across all its
    /// rings.
    ///
    /// Drained at the two quarantine mutation sites (insert and find) and corrected by the eviction
    /// listener, so it needs no resync pass — an opportunistic resync would be a CPU/lock amplifier
    /// running on exactly the attacker-flooded state. It is a backstop, not the structural bound: the
    /// cache's own `max_capacity` × `quarantine_rb_capacity` is the hard bound, and a few SURBs of
    /// drift between a `ring.len()` read and the listener cannot exceed it.
    ///
    /// Refunds saturate at zero rather than wrapping: the refund sites read ring lengths without the
    /// ring lock, so a stale reader could otherwise drive the counter past zero and permanently lock
    /// out every quarantine insert — see [`refund_quarantine_budget`].
    quarantine_budget: Arc<std::sync::atomic::AtomicUsize>,
    /// Relayers this node can no longer pay. Holds at most a handful of entries (our own closing
    /// channels), so a plain set behind an `RwLock` beats a concurrent map on this read-heavy path.
    invalidated_relayers: Arc<parking_lot::RwLock<std::collections::HashSet<HoprKeyIdent>>>,
    /// Current SURB-batch generation per pseudonym we originate for (sending side). Advanced on a
    /// return-path change so the replying side can drop SURBs for the superseded path.
    ///
    /// This is sending-side state, so it is retained like the reply openers
    /// ([`pseudonym_openers`](Self::pseudonym_openers)) — **not** like the receiving-side
    /// [`admitted_surbs_per_pseudonym`](Self::admitted_surbs_per_pseudonym). If this entry were evicted while the peer
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

// The `cache` label takes the names of `EvictedCache`: the five moka caches, and `ring_plain` / `ring_share` for the
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
    static ref METRIC_SURB_STORE_ADMISSIONS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_surb_store_admissions_count",
        "Admission-control events in the SURB store, by event kind",
        &["event"],
    )
    .unwrap();
    static ref METRIC_SURB_STORE_SURBS_HELD: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_surb_store_surbs_held",
        "SURBs currently held by the SURB store, by trust class",
        &["class"],
    )
    .unwrap();
}

#[cfg(all(feature = "telemetry", not(test)))]
/// Values of the `event` label on `hopr_surb_store_admissions_count`.
///
/// Bounded by construction: each variant is recorded from exactly one named code path, so the series
/// cardinality cannot grow with traffic.
#[derive(Clone, Copy, Debug, strum::Display, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum SurbAdmissionEvent {
    /// A Session over the pseudonym was established and it entered the admitted class.
    Admitted,
    /// On admission, a quarantine ring was promoted into the admitted class rather than re-read.
    Promoted,
    /// The Session over the pseudonym ended and it left the admitted class.
    Retired,
    /// A retire arrived whose token no longer named the pseudonym's current admission — a successor
    /// Session was admitted in the deferred-retire window, and its ring was left alone. Routine, and
    /// worth a series rather than silence: a stale retire that *could* touch a live ring would show up
    /// as this never rising while a Session's SURBs vanish.
    RetiredStale,
    /// An unadmitted batch was refused because the quarantine SURB budget was exhausted.
    QuarantineRefused,
    /// Quarantine rings were shed to get back under the quarantine SURB budget.
    QuarantineEvicted,
    /// The admitted cache overflowed `admitted_max_pseudonyms`: a Size eviction there means the
    /// config is below `maximum_managed_sessions`, an operator misconfiguration worth alerting on.
    AdmittedOverflow,
}

#[cfg(all(feature = "telemetry", not(test)))]
/// Values of the `class` label on `hopr_surb_store_surbs_held`.
#[derive(Clone, Copy, Debug, strum::Display, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum SurbHeldClass {
    /// SURBs behind an established Session.
    Admitted,
    /// Unadmitted SURBs, quarantined and time-limited.
    Quarantine,
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
    /// Replying side: a pseudonym's SURB ring buffer (`admitted_surbs_per_pseudonym`).
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
    /// Replying side: a pseudonym's quarantine SURB ring buffer (`quarantine_surbs`). Unadmitted
    /// SURBs only, so an eviction here costs nothing an established Session holds.
    QuarantineRing,
}

impl EvictedCache {
    const ALL: [EvictedCache; 7] = [
        Self::ReplyOpenerBatch,
        Self::ReplyOpener,
        Self::SurbRing,
        Self::RingPlain,
        Self::RingShare,
        Self::Generation,
        Self::QuarantineRing,
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
            lazy_static::initialize(&METRIC_SURB_STORE_ADMISSIONS);
            lazy_static::initialize(&METRIC_SURB_STORE_SURBS_HELD);
        }
        let stats = Arc::new(EvictionStats::new(&cfg));
        let budget = Arc::new(std::sync::atomic::AtomicUsize::new(0));
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
            // Admitted SURB rings: entered only through `admit`, i.e. only once an established Session
            // stands behind the pseudonym. Sized by `admitted_max_pseudonyms`, which the operator sets
            // at or above `maximum_managed_sessions` — an established Session per pseudonym is the only
            // way to fill this cache, so its ceiling is the operator's own session census, not anything
            // an unauthenticated peer can inflate. Its listener logs at ERROR on a Size removal: that is
            // the one cause that says the config is wrong, since the cache cannot then hold one ring per
            // live Session — an `admitted_overflow` event, worth alerting on.
            //
            // Deliberately no `time_to_idle`. The entry count is already bounded by
            // `admitted_max_pseudonyms`, and every Session teardown retires its entry through
            // `SurbStore::retire`, so a ring's lifetime is exactly its admission's lifetime — the
            // invariant the whole design rests on, which a TTI can only violate. A Session's idle
            // timeout is unbounded above and separate from `pseudonyms_lifetime`, and even at the
            // defaults a Session can sit alive without receiving inbound SURBs for longer than the
            // 600 s TTI. A ring reclaimed by time then silently falls back to the quarantine, whose
            // 16-SURB cap drops the Session's later SURBs and PIX shares — the exact loss admission
            // exists to prevent — and `SessionManager::ping_session` refreshes the session cache but
            // does not re-admit, so nothing would restore it.
            admitted_surbs_per_pseudonym: moka::sync::Cache::builder()
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener({
                    let stats = stats.clone();
                    move |_pseudonym, _surbs, cause| {
                        // Size is the only cache-driven removal left — the entry otherwise leaves by
                        // `retire`'s explicit removal — so it is the one cause that says the config is
                        // wrong: the cache cannot hold one ring per live Session.
                        if cause == RemovalCause::Size {
                            #[cfg(all(feature = "telemetry", not(test)))]
                            METRIC_SURB_STORE_ADMISSIONS
                                .increment_by(&[SurbAdmissionEvent::AdmittedOverflow.into()], 1);
                            tracing::error!(
                                ?cause,
                                "an admitted SURB ring was evicted for size: admitted_max_pseudonyms is below the \
                                 number of live sessions"
                            );
                        }
                        stats.record(EvictedCache::SurbRing, cause)
                    }
                })
                .max_capacity(cfg.admitted_max_pseudonyms.max(1) as u64)
                .build(),
            // Quarantine rings: whatever pseudonym a `Final` packet chose and no Session has claimed.
            // Capped and aged far below the admitted class — short TTI, few pseudonyms, few SURBs per
            // ring — so unauthenticated churn is absorbed here and, being a physically separate cache,
            // can never evict an admitted ring. The listener keeps the quarantine SURB budget honest for
            // cache-driven removals (expiry, LRU, size): an explicit removal is a promotion or a
            // deliberate take-over, which the promoting code path accounts for itself.
            quarantine_surbs: moka::sync::Cache::builder()
                .time_to_idle(cfg.quarantine_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener({
                    let stats = stats.clone();
                    let budget = Arc::clone(&budget);
                    move |_pseudonym, ring: SurbRingBuffer<HoprSurb>, cause| {
                        if cause != RemovalCause::Explicit {
                            refund_quarantine_budget(&budget, ring.len());
                        }
                        stats.record(EvictedCache::QuarantineRing, cause)
                    }
                })
                .max_capacity(cfg.quarantine_max_pseudonyms.max(MINIMUM_QUARANTINE_PSEUDONYMS) as u64)
                .build(),
            invalidated_relayers: Default::default(),
            generations: moka::sync::Cache::builder()
                // Sending-side state, so it is sized and aged like the sibling sending-side
                // `pseudonym_openers` cache — NOT like the receiving-side admitted/quarantine SURB
                // caches.
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
            quarantine_budget: budget,
            stats,
            cfg: cfg.into(),
        }
    }

    /// Whether `relayer` is currently unusable as a return path's first hop.
    pub fn is_relayer_invalidated(&self, relayer: &HoprKeyIdent) -> bool {
        self.invalidated_relayers.read().contains(relayer)
    }

    /// Capacity for a newly created admitted ring, derived from the Session's announced SURB target.
    ///
    /// The target gets [`SurbStoreConfig::admitted_headroom_percent`] of headroom — the balancer aims
    /// below the ring's ceiling, so a ring exactly the size of the target would overflow on ordinary
    /// overshoot — then is floored at `quarantine_rb_capacity` and ceiled at the operator's
    /// `rb_capacity`. An announced target of `0` means "no announcement" and gets the floor: a ring of
    /// one is a trap the very next packet overflows, and nothing cheaper than the quarantine floor is
    /// worth admitting.
    fn admitted_ring_capacity(&self, expected_surbs: usize) -> usize {
        let floor = self.cfg.quarantine_rb_capacity.max(MINIMUM_QUARANTINE_RB_CAPACITY);
        if expected_surbs == 0 {
            return floor;
        }
        let headroom = expected_surbs.saturating_mul(100 + self.cfg.admitted_headroom_percent as usize) / 100;
        headroom.max(floor).min(self.cfg.rb_capacity.max(MIN_SURB_RB_CAPACITY))
    }

    /// SURBs held for `pseudonym` across both classes.
    ///
    /// Sums the two caches rather than asking one, so the brief dual-ring state around a promotion —
    /// a batch that routed to the quarantine while `admit` was re-checking — cannot under-report what
    /// the pseudonym holds.
    fn ring_len(&self, pseudonym: &HoprPseudonym) -> usize {
        let admitted = self
            .admitted_surbs_per_pseudonym
            .get(pseudonym)
            .map(|rb| rb.len())
            .unwrap_or(0);
        let quarantined = self.quarantine_surbs.get(pseudonym).map(|rb| rb.len()).unwrap_or(0);
        admitted + quarantined
    }

    /// Books what a ring-buffer push dropped into the eviction summary, split by what it cost.
    fn record_ring_overflow(&self, outcome: &SurbInsertOutcome) {
        // The total that `num_evicted_surbs` carries upwards, and the Exit books into its flow
        // estimate, cannot tell a lost share from a lost return path, and only the former always
        // warrants a warning. Recording a count of zero is free, so the usual push, which overflows
        // nothing, takes no lock here.
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
    }

    /// Pushes a batch into an admitted ring, after folding in whatever a quarantine ring for the same
    /// pseudonym still holds.
    ///
    /// Shared by both admitted routes in [`SurbStore::insert_surbs`]: the straight one, where the
    /// admission pre-dates the batch, and the admit-window one, where `admit` landed between the
    /// routing check and the re-check. The fold-in is re-done there rather than left to `admit`,
    /// because a concurrent insert for the same pseudonym can have created a quarantine ring *after*
    /// `admit`'s own promotion — and because `admit` itself must not be called again: it would mint a
    /// fresh admission number the manager's token no longer names, leaving the Session's deferred
    /// retire matching nothing while the entry it stamped holds a number no retire will ever present.
    ///
    /// The fold-in is what keeps the two routes from diverging, and it is why only `admit` creates an
    /// admitted entry: an insert that minted one itself would have no admission number to stamp, and a
    /// ring stamped with the sentinel is one no retire ever removes.
    /// `remove` is an explicit removal the listener skips, so the budget is decremented here at this
    /// one mutation site.
    fn push_into_admitted(
        &self,
        pseudonym: &HoprPseudonym,
        admitted: &AdmittedRing,
        surbs: Vec<(HoprSurbId, HoprSurb)>,
        generation: u8,
    ) -> SurbInsertOutcome {
        if let Some(qring) = self.quarantine_surbs.remove(pseudonym) {
            refund_quarantine_budget(&self.quarantine_budget, qring.len());
            let (qgeneration, held) = qring.take_all();
            if let Some(qgeneration) = qgeneration {
                let outcome = admitted.push(held, qgeneration);
                self.record_ring_overflow(&outcome);
            }
        }

        let outcome = admitted.push(surbs, generation);
        self.record_ring_overflow(&outcome);
        outcome
    }

    /// The admitted arm of [`SurbStore::insert_surbs`], split out so a test can drive it directly:
    /// it is reached when `insert_surbs`' admission lookup finds an entry, which is only ever the one
    /// `admit` installed — no insert may create one — and cannot be forced from outside.
    ///
    /// `admitted` is the entry the real `admit` installed, carrying the admission number the
    /// SessionManager threads back to `retire`. It is used as it stands rather than `admit` being
    /// called again: a second `admit` would mint a fresh number the manager's token no longer names,
    /// and the deferred retire would then match nothing, leaking the Session's ring until its teardown
    /// never comes. What is re-done here is only the fold-in of a quarantine ring, which a concurrent
    /// insert for the same pseudonym can have created after `admit`'s own promotion.
    fn insert_surbs_into_admitted(
        &self,
        pseudonym: &HoprPseudonym,
        admitted: AdmittedRing,
        surbs: Vec<(HoprSurbId, HoprSurb)>,
        generation: u8,
    ) -> SurbInsertOutcome {
        self.push_into_admitted(pseudonym, &admitted, surbs, generation)
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
        // The admitted ring is consulted first: it is the one an established Session stands behind,
        // so a SURB there is worth more than one held on speculation. The quarantine is consulted only
        // because a pseudonym can legitimately receive SURBs before its Session is established (probe
        // replies, the one-RTT window before `admit` fires) — a hit there is handed out, but it is
        // *not* promoted: which pseudonym deserves admission is decided by the Session lifecycle alone,
        // and `matcher` names a pseudonym an unauthenticated counterparty chose, so promoting on a pop
        // would let that counterparty mint itself into the admitted class.
        let is_quarantine = self.admitted_surbs_per_pseudonym.get(&pseudonym).is_none();
        let surbs_for_pseudonym = if is_quarantine {
            self.quarantine_surbs.get(&pseudonym)?
        } else {
            // Deref'd to the ring: the caller below sees a `SurbRingBuffer` either way, and the
            // admission number riding on the entry is this method's business only insofar as it must
            // not touch it — a pop must not disturb the guard the deferred retire checks.
            self.admitted_surbs_per_pseudonym.get(&pseudonym)?.ring
        };
        let held_before = surbs_for_pseudonym.len();

        let pop = match matcher {
            // SURBs whose return path no longer has a usable first edge are dropped on the way,
            // rather than handed out only to have the reply fail to be paid for.
            SurbMatcher::Pseudonym(_) => surbs_for_pseudonym
                .pop_next_valid(|_, surb| self.is_surb_usable(surb))
                .map(|popped_surb| (popped_surb.id, popped_surb.surb, popped_surb.remaining)),
            // The following code intentionally only checks the oldest SURB of the tier that would be
            // consumed next (share-bearing SURBs first, share-less ones once none are left) and does
            // not search the entire RB.
            // This is because the exact match use-case is suited only for situations
            // when there is a single SURB in the RB.
            SurbMatcher::Exact(id) => surbs_for_pseudonym
                .pop_one_if_has_id(&id.surb_id())
                .map(|popped_surb| (popped_surb.id, popped_surb.surb, popped_surb.remaining)), // = likely 0
        };

        if is_quarantine {
            // A pop is a removal as far as the quarantine budget is concerned, and `pop_next_valid`
            // discards rather than skips the SURBs failing the usability test, so the whole delta
            // between the lengths before and after is unadmitted SURBs that left the store. The
            // accounting must precede the `pop` short-circuit below: a lookup whose SURBs are all
            // discarded empties the ring and yields `None`, and those SURBs left the store all the
            // same — skipping the decrement there would pin the budget at its cap and refuse every
            // later fresh SURB.
            let dropped = held_before.saturating_sub(surbs_for_pseudonym.len());
            refund_quarantine_budget(&self.quarantine_budget, dropped);
        }

        let (id, surb, remaining) = pop?;

        Some(FoundSurb {
            sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, id),
            surb,
            remaining,
        })
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
                retained: self.ring_len(&pseudonym),
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

        // Route on the admitted cache itself: it holds an entry — often an empty ring — for exactly
        // as long as a Session stands behind the pseudonym, so this one lookup is both the admission
        // test and the source of truth. A separate marker cache would have to be kept in step with
        // the ring across evictions and admits, and a desync there either loses a live ring to the
        // quarantine's tiny cap or lets a superseded marker resurrect it.
        //
        // The lookup *takes* the entry rather than inserting one: only `admit` may create an admitted
        // entry. `entry_by_ref(...).or_insert_with(...)` would recreate an epoch-0 ring whenever a
        // deferred `retire` removed the key between the admission test and the entry lookup, and that
        // ring would then never leave by its teardown — `retire` ignores token 0 — so it would leak
        // until size eviction, while a successor `admit` finding it would only re-stamp the epoch and
        // leave it at the floor capacity, so later inserts overflow it and lose SURBs and PIX shares.
        // An absent entry therefore means the pseudonym is not admitted (or was just retired), and the
        // batch belongs to the quarantine like any other unadmitted one.
        if let Some(admitted) = self.admitted_surbs_per_pseudonym.get(&pseudonym) {
            return self.insert_surbs_into_admitted(&pseudonym, admitted, surbs, generation);
        }

        // The quarantine is the only class an unauthenticated peer can write into, so the budget
        // applies here and only here. Refusing an admitted batch to protect it would trade live
        // PIX shares for untrusted ones.
        if self.quarantine_budget.load(std::sync::atomic::Ordering::Relaxed) + surbs.len()
            > self.cfg.quarantine_surb_budget
        {
            // Evict whole quarantine rings to get back under the budget. The victims are uniformly
            // untrusted, so eviction order is arbitrary — moka's `iter` gives no order to lean on.
            // Never `invalidate` inside a live `iter`: moka holds the cache lock for the iteration,
            // so the invalidation would deadlock against itself. The rings are collected with their
            // keys so the budget can be decremented by what is actually removed: `invalidate` is an
            // explicit removal, which the listener skips, so this drain is the only place that can
            // account for the SURBs it sheds — missing it here pins the counter at the cap and
            // refuses every later fresh SURB for good.
            const VICTIMS: usize = 64;
            let victims: Vec<(HoprPseudonym, SurbRingBuffer<HoprSurb>)> = self
                .quarantine_surbs
                .iter()
                .take(VICTIMS)
                .map(|(key, ring)| (*key, ring.clone()))
                .collect();
            let evicted_rings = victims.len();
            let evicted_surbs: usize = victims.iter().map(|(_, ring)| ring.len()).sum();
            for (key, _) in &victims {
                self.quarantine_surbs.invalidate(key);
            }
            refund_quarantine_budget(&self.quarantine_budget, evicted_surbs);
            self.quarantine_surbs.run_pending_tasks();

            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_SURB_STORE_ADMISSIONS
                .increment_by(&[SurbAdmissionEvent::QuarantineEvicted.into()], evicted_rings as u64);
            #[cfg(not(all(feature = "telemetry", not(test))))]
            let _ = evicted_rings;

            // Still over? Then the cache's own bound is the wall and the batch is refused whole:
            // it is unadmitted traffic, so turning it away costs return paths, not shares held for
            // a Session. `retained` reports what the pseudonym still holds in either class, and
            // `evicted_shares` the shares the refused batch actually carried — the caller above
            // reads `evicted` as "SURBs that left or never entered the store", which a refusal is.
            let over = self.quarantine_budget.load(std::sync::atomic::Ordering::Relaxed) + surbs.len()
                > self.cfg.quarantine_surb_budget;
            if over {
                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_SURB_STORE_ADMISSIONS.increment_by(&[SurbAdmissionEvent::QuarantineRefused.into()], 1);
                tracing::debug!(
                    %pseudonym,
                    num_surbs = surbs.len(),
                    budget = self.cfg.quarantine_surb_budget,
                    "refusing unadmitted SURBs: the quarantine SURB budget is exhausted"
                );
                return SurbInsertOutcome {
                    retained: self.ring_len(&pseudonym),
                    evicted: surbs.len(),
                    evicted_shares: surbs.iter().filter(|(_, surb)| surb.ssa_index().is_some()).count(),
                };
            }
        }

        // Captured *after* the drain above, not before it: the drain may have evicted this very
        // pseudonym's ring, and the listener already subtracted that ring's SURBs, so the delta
        // against a pre-drain `before` would subtract them a second time.
        let before = self.ring_len(&pseudonym);

        let outcome = self
            .quarantine_surbs
            .entry_by_ref(&pseudonym)
            .or_insert_with(|| SurbRingBuffer::new(self.cfg.quarantine_rb_capacity))
            .value()
            .push(surbs, generation);

        // The delta is exact at this one mutation site, so the budget needs no resync pass: a
        // `push` that evicted and re-inserted in the same call nets out here, and the eviction
        // listener above corrects for the cache's own removals.
        let delta = outcome.retained as isize - before as isize;
        if delta >= 0 {
            self.quarantine_budget
                .fetch_add(delta as usize, std::sync::atomic::Ordering::Relaxed);
        } else {
            refund_quarantine_budget(&self.quarantine_budget, (-delta) as usize);
        }

        self.record_ring_overflow(&outcome);
        outcome
    }

    fn tier_lens(&self, pseudonym: &HoprPseudonym) -> Option<(usize, usize)> {
        self.admitted_surbs_per_pseudonym
            .get(pseudonym)
            .map(|rb| rb.tier_lens())
            .or_else(|| self.quarantine_surbs.get(pseudonym).map(|rb| rb.tier_lens()))
    }

    /// Admits `pseudonym` into the store's admitted class, behind an established Session.
    ///
    /// Called once the Session over the pseudonym is established, and never on a mere packet's say-so,
    /// so the admitted class stays out of any unauthenticated peer's reach. `expected_surbs` is the
    /// Session's announced SURB target; the store adds its own headroom and floors at
    /// `quarantine_rb_capacity` — an unknown target (`0`) is treated as "no announcement" and gets the
    /// floor, not a one-SURB ring.
    ///
    /// Returns the number naming this admission, which the caller threads to
    /// [`SurbStore::retire`]. Every call mints a fresh one and invalidates the previous, so a
    /// retire issued on behalf of a superseded admission matches nothing — that is the guard that
    /// makes the deferred retire safe to schedule.
    ///
    /// Any quarantine ring for the pseudonym is promoted rather than left behind to expire: the SURBs
    /// arrived for this very Session. A ring arrives here only while the pseudonym is unadmitted
    /// (`insert_surbs` routes on admission), so an admitted entry this finds already in place is one a
    /// previous admission left behind — `insert_surbs` never creates one — and the promotion fills it
    /// rather than discarding what it holds.
    ///
    /// The budget is accounted here, not by the listener: the promotion removes the quarantine entry
    /// explicitly, which the listener skips, so the SURBs it carried are decremented at this one
    /// mutation site.
    fn admit(&self, pseudonym: &HoprPseudonym, expected_surbs: usize) -> u64 {
        let capacity = self.admitted_ring_capacity(expected_surbs);
        // Minted before anything is written, so a retire racing this admit always carries a number
        // strictly older than the one being installed here.
        let epoch = next_admission_epoch();

        // Any quarantine ring is promoted rather than left behind to expire: the SURBs arrived for
        // this very Session. Lifted out before the compute below so the compute stays a single
        // decision about the admitted entry; the promoted SURBs are folded into whichever ring that
        // decision leaves in place.
        let promoted = self.quarantine_surbs.remove(pseudonym);
        if let Some(qring) = &promoted {
            refund_quarantine_budget(&self.quarantine_budget, qring.len());
        }

        // One atomic compute over the key for both halves of the admission — the ring and its number.
        // Splitting them (install the ring, then stamp the number) would reopen the very window the
        // token exists to close: a deferred retire carrying the *previous* admission's token matches
        // the not-yet-restamped entry and removes it, and the stamp then lands on nothing — the new
        // Session would go on inserting its SURBs into the small un-admitted quarantine with nothing
        // logged anywhere. `and_compute_with` holds the key-level lock across both, the same lock the
        // retire's compare-and-remove runs under, so the two cannot interleave.
        let stamped = self
            .admitted_surbs_per_pseudonym
            .entry_by_ref(pseudonym)
            .and_compute_with(|entry: Option<moka::Entry<HoprPseudonym, AdmittedRing>>| match entry {
                // Re-admitting a pseudonym (a Session reopened on the same one) must not discard what
                // the resident ring already holds, and must not replace the entry wholesale either:
                // `Replaced` trips the admitted cache's listener, which books it as an eviction the
                // ring never actually suffered. Stamping in place changes the number and nothing else.
                Some(entry) => {
                    entry.value().stamp_epoch(epoch);
                    moka::ops::compute::Op::Nop
                }
                // No ring to keep: admit an empty one, so standing is knowable before the first SURB
                // arrives, or `insert_surbs` would route the Session's early batches to quarantine.
                None => moka::ops::compute::Op::Put(AdmittedRing::new(SurbRingBuffer::new(capacity), epoch)),
            });

        let resident = match stamped.into_entry() {
            // Both arms above leave an entry behind: `Put` inserted it, `Nop` kept the one it found.
            Some(entry) => entry.value().ring.clone(),
            None => {
                // Unreachable by the same argument; reached only if a concurrent retire removed an
                // entry this compute had already decided to keep, which the key-level lock rules out.
                // Refusing to admit here degrades to the quarantine, which is the safe direction.
                tracing::error!(?pseudonym, "an admitted ring vanished while it was being admitted");
                return epoch;
            }
        };

        if let Some(qring) = promoted {
            // `with_capacity` deep-copies the ring at the admitted ceiling — a plain clone shares
            // the inner buffer and would keep the quarantine cap on what is now an admitted ring.
            let promoted = qring.with_capacity(capacity);
            let (generation, held) = promoted.take_all();
            if let Some(generation) = generation {
                let outcome = resident.push(held, generation);
                self.record_ring_overflow(&outcome);
            }

            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_SURB_STORE_ADMISSIONS.increment_by(&[SurbAdmissionEvent::Promoted.into()], 1);
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_SURB_STORE_ADMISSIONS.increment_by(&[SurbAdmissionEvent::Admitted.into()], 1);

        epoch
    }

    /// Retires `pseudonym` from the admitted class: its Session is over.
    ///
    /// Only the admitted class is touched — a rolled-back or unestablished Session's preload SURBs
    /// belong to the quarantine, where their short TTI expires them without any teardown bookkeeping.
    /// Idempotent: retiring a pseudonym that holds no admitted ring is a no-op.
    ///
    /// `token` names the admission the teardown is retiring, and the retire is honoured only while it
    /// still matches the pseudonym's current one. The retire runs deferred, past the packet pipeline's
    /// SURB-resolution window, because the closing Session's own return-routed notification still
    /// needs its ring — long enough for a successor Session to have been admitted on the same
    /// client-chosen pseudonym. Dropping the successor's ring there would silently degrade its return
    /// path with nothing logged anywhere, which is exactly what the token exists to prevent: a stale
    /// retire leaves the successor's ring, and its epoch, exactly as they are.
    fn retire(&self, pseudonym: &HoprPseudonym, token: u64) {
        // `token == 0` never matches: it is what a teardown of a never-admitted Session passes
        // through the seam, and no admission mints it.
        if token == 0 {
            return;
        }

        // One atomic compute over the key, so the epoch read and the removal cannot be split by a
        // successor `admit`: done as a `get` then a conditional `invalidate`, an admit landing between
        // the two would leave this retire removing the *successor's* ring. `and_compute_with` holds
        // the key-level lock across both, and `Op::Nop` leaves the entry — the successor's ring and
        // its number — untouched when the numbers disagree.
        let retired = matches!(
            self.admitted_surbs_per_pseudonym
                .entry_by_ref(pseudonym)
                .and_compute_with(|entry: Option<moka::Entry<HoprPseudonym, AdmittedRing>>| match entry {
                    // A matching number names this very ring: `admit` stamps the entry only after the
                    // ring is in place, under the same key lock this compare runs under. Dropping the
                    // entry is the retire.
                    Some(entry) if entry.value().epoch() == token => moka::ops::compute::Op::Remove,
                    // No ring, or a ring whose number has moved on: the pseudonym was re-admitted since
                    // this token was minted, and the live ring is not this retire's to release.
                    _ => moka::ops::compute::Op::Nop,
                }),
            moka::ops::compute::CompResult::Removed(_)
        );

        if retired {
            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_SURB_STORE_ADMISSIONS.increment_by(&[SurbAdmissionEvent::Retired.into()], 1);
        } else {
            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_SURB_STORE_ADMISSIONS.increment_by(&[SurbAdmissionEvent::RetiredStale.into()], 1);
        }
    }

    /// Total SURBs held across both classes. Test and observability seam, not a hot path: it walks the
    /// admitted cache and reads the quarantine budget.
    fn surbs_held(&self) -> usize {
        // Read the quarantine budget exactly once: it is an atomic that a concurrent insert or refund
        // can move between two loads, and deriving the admitted half as `held - quarantined` from two
        // separate reads would then underflow. One read makes the two halves consistent by construction.
        let quarantined = self.quarantine_budget.load(std::sync::atomic::Ordering::Relaxed);
        let admitted = self
            .admitted_surbs_per_pseudonym
            .iter()
            .map(|(_, rb)| rb.len())
            .sum::<usize>();

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            METRIC_SURB_STORE_SURBS_HELD.set(&[SurbHeldClass::Admitted.into()], admitted as f64);
            METRIC_SURB_STORE_SURBS_HELD.set(&[SurbHeldClass::Quarantine.into()], quarantined as f64);
        }

        admitted + quarantined
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

/// An admitted ring together with the number naming the admission that installed it.
///
/// The pairing is what a deferred [`SurbStore::retire`] checks: it removes the entry only when its
/// token still equals the number stored here, so a retire that arrives after a successor Session was
/// admitted on the same pseudonym is a no-op rather than a destruction of the successor's ring. The
/// two live in one cache entry so the compare in `retire` and the removal are a single atomic
/// operation over the key — split across two caches, an `admit` could land between the compare and
/// the removal and the retire would destroy the successor's ring.
///
/// `epoch` is behind an `Arc` so it can be re-stamped in place: `admit` of a re-admitted pseudonym
/// must keep the resident ring (it must not discard what it already holds), and replacing the entry
/// wholesale to carry a new number would trip moka's `Replaced` path — which the admitted cache's
/// listener books as an eviction the ring never actually suffered. Storing through the `Arc` changes
/// only the number, and it happens inside [`SurbStore::admit`]'s `and_compute_with`, under the same
/// key-level lock `retire` compares under — so an entry is never observable with one admission's
/// ring and a later admission's number, nor with no number at all.
#[derive(Clone, Debug)]
struct AdmittedRing {
    ring: SurbRingBuffer<HoprSurb>,
    /// The number [`next_admission_epoch`] minted for the admission that installed `ring`. Zero is
    /// never minted, so an entry created by a racing [`SurbStore::insert_surbs`], whose Session has
    /// not been admitted yet, matches no retire token at all.
    epoch: Arc<std::sync::atomic::AtomicU64>,
}

impl AdmittedRing {
    fn new(ring: SurbRingBuffer<HoprSurb>, epoch: u64) -> Self {
        Self {
            ring,
            epoch: Arc::new(std::sync::atomic::AtomicU64::new(epoch)),
        }
    }

    fn epoch(&self) -> u64 {
        self.epoch.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn stamp_epoch(&self, epoch: u64) {
        self.epoch.store(epoch, std::sync::atomic::Ordering::Relaxed);
    }
}

impl std::ops::Deref for AdmittedRing {
    type Target = SurbRingBuffer<HoprSurb>;

    fn deref(&self) -> &Self::Target {
        &self.ring
    }
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

    /// A copy of this buffer holding at most `capacity` SURBs, everything else carried over.
    ///
    /// This is a deep copy of both tiers and the generation into a fresh buffer, not an `Arc` clone:
    /// `capacity` is a plain field read by [`push`](Self::push), so a shared inner would leave the
    /// copy bound by the *old* ceiling no matter what the caller passes. It is how the store raises a
    /// quarantine ring to its admitted capacity on promotion — rebuilding via `push` would re-run the
    /// eviction order on SURBs that already earned their slots, and would lose the generation the
    /// batches were minted at.
    ///
    /// Insertion order is preserved within each tier, so what pops first afterwards is what would
    /// have popped first before.
    pub fn with_capacity(&self, capacity: usize) -> Self
    where
        S: Clone,
    {
        let inner = self.inner.lock();
        Self {
            inner: Arc::new(parking_lot::Mutex::new(GenerationalBuffer {
                shares: inner.shares.iter().cloned().collect(),
                plain: inner.plain.iter().cloned().collect(),
                generation: inner.generation,
            })),
            capacity: capacity.max(1),
        }
    }

    /// Removes every SURB held and returns them with the generation they were pushed at.
    ///
    /// Used to move a ring between the store's two classes: the caller pushes the returned batch into
    /// the destination ring, whose own generation logic then decides how the two merge (append, supersede
    /// or discard). The returned order is pop order — share-bearing SURBs first, each tier oldest
    /// first — not the arrival order, which is the order that matters to a consumer.
    fn take_all(&self) -> (Option<u8>, Vec<(HoprSurbId, S)>) {
        let mut inner = self.inner.lock();
        let generation = inner.generation;
        let shares = std::mem::take(&mut inner.shares);
        let plain = std::mem::take(&mut inner.plain);
        (generation, shares.into_iter().chain(plain).collect())
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
        // The exit's ring only reaches `rb_capacity` for an admitted pseudonym; unadmitted SURBs land
        // in the 16-cap quarantine, which a 200-reply flood would exhaust long before the test's
        // eviction math is exercised.
        exit.admit(&pseudonym, rb_capacity);

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
        // The opener and generation caches are floored at 1000 pseudonyms, and the quarantine at 64,
        // so 1500 distinct ones overflow all three. Fresh-pseudonym SURBs land in the quarantine: the
        // admitted cache is entered only through `admit`, which this flood never triggers, so what the
        // churn sheds is unadmitted rings — which is the point of the split.
        let store = MemorySurbStore::new(SurbStoreConfig {
            max_openers_per_pseudonym: 100,
            quarantine_max_pseudonyms: 100,
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
        store.quarantine_surbs.run_pending_tasks();
        store.generations.run_pending_tasks();

        for cache in [
            EvictedCache::ReplyOpenerBatch,
            EvictedCache::QuarantineRing,
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
        // The full-ring arithmetic below is admitted-ring arithmetic; unadmitted SURBs would cap out
        // at the quarantine's 16.
        store.admit(&pseudonym, capacity);
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

    /// The range rules on the admission-control fields, checked without serde so they are exercised
    /// under the default feature set — the serde-gated companion below covers only the loading half.
    #[test]
    fn surb_store_config_should_reject_out_of_range_admission_fields() {
        use validator::Validate;

        // Every field at its boundary is valid. The budget's range rule spells its floor as a literal
        // (as every other range validator here does), so the assertions below — which reject one below
        // `QUARANTINE_SURB_BUDGET_FLOOR` and accept it exactly — are what pin the two together.
        assert!(
            SurbStoreConfig {
                quarantine_rb_capacity: MINIMUM_QUARANTINE_RB_CAPACITY,
                quarantine_max_pseudonyms: MINIMUM_QUARANTINE_PSEUDONYMS,
                quarantine_surb_budget: QUARANTINE_SURB_BUDGET_FLOOR,
                admitted_max_pseudonyms: 1,
                admitted_headroom_percent: 0,
                ..Default::default()
            }
            .validate()
            .is_ok(),
            "each admission field at its floor must validate"
        );

        for (name, cfg) in [
            (
                "quarantine_rb_capacity",
                SurbStoreConfig {
                    quarantine_rb_capacity: MINIMUM_QUARANTINE_RB_CAPACITY - 1,
                    ..Default::default()
                },
            ),
            (
                "quarantine_max_pseudonyms",
                SurbStoreConfig {
                    quarantine_max_pseudonyms: MINIMUM_QUARANTINE_PSEUDONYMS - 1,
                    ..Default::default()
                },
            ),
            (
                "quarantine_surb_budget",
                SurbStoreConfig {
                    quarantine_surb_budget: QUARANTINE_SURB_BUDGET_FLOOR - 1,
                    ..Default::default()
                },
            ),
            (
                "admitted_max_pseudonyms",
                SurbStoreConfig {
                    admitted_max_pseudonyms: 0,
                    ..Default::default()
                },
            ),
        ] {
            assert!(cfg.validate().is_err(), "{name} below its floor must be rejected");
        }

        // The lifetime uses a custom rule rather than a range, so it is checked on its own.
        assert!(
            SurbStoreConfig {
                quarantine_lifetime: MINIMUM_SURB_LIFETIME - Duration::from_millis(1),
                ..Default::default()
            }
            .validate()
            .is_err(),
            "a quarantine lifetime below the SURB lifetime floor must be rejected"
        );
    }

    // --- admission control -------------------------------------------------------------------------

    /// The issue's regression: a flood of fresh pseudonyms used to LRU-evict a live Session's ring out
    /// of the single SURB cache. Now the flood is confined to the quarantine — and the test is not
    /// vacuous about it: it first asserts the quarantine actually shed rings under the flood, and only
    /// then that the admitted ring survived.
    #[test]
    fn admitted_ring_survives_a_fresh_pseudonym_flood() -> anyhow::Result<()> {
        // The quarantine's own floor: the flood must be able to shed something.
        let store = MemorySurbStore::new(SurbStoreConfig {
            quarantine_max_pseudonyms: MINIMUM_QUARANTINE_PSEUDONYMS,
            ..Default::default()
        });
        let live = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        store.admit(&live, MIN_SURB_RB_CAPACITY);
        store.insert_surbs(live, vec![([7u8; 8], surb_via(relayer, DIRECT)?)]);

        // 65 fresh pseudonyms against a 64-entry quarantine: at least one quarantine ring must go.
        for i in 0..(MINIMUM_QUARANTINE_PSEUDONYMS + 1) {
            store.insert_surbs(
                HoprPseudonym::random(),
                vec![([i as u8; 8], surb_via(relayer, DIRECT)?)],
            );
        }
        store.quarantine_surbs.run_pending_tasks();
        store.admitted_surbs_per_pseudonym.run_pending_tasks();

        let shed = store.stats.state.lock().counts[EvictedCache::QuarantineRing as usize].size;
        assert!(
            shed > 0,
            "precondition: the flood must actually shed quarantine rings, got {shed}"
        );
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(live)).is_some(),
            "the admitted ring must survive a flood that shed {shed} quarantine rings"
        );

        Ok(())
    }

    /// The v1 budget refused admitted inserts, dropping live PIX shares to protect untrusted SURBs —
    /// the defect the two-class split exists to remove. With the quarantine budget saturated by
    /// churn, a second, legitimate Session's insert must still be accepted and retained.
    #[test]
    fn a_second_legitimate_session_can_still_insert_under_quarantine_pressure() -> anyhow::Result<()> {
        // 512 pseudonyms × 16 SURBs of fresh churn against the 4096 budget floor: the drain sheds
        // rings as it runs, so the counter ends wherever the last drain landed rather than at the
        // cap — the flood puts the quarantine under sustained pressure, and the store below pins the
        // counter to the cap so the admitted insert is decided against a saturated budget.
        let store = MemorySurbStore::new(SurbStoreConfig {
            quarantine_surb_budget: 4096,
            ..Default::default()
        });
        let relayer = HoprKeyIdent::from(1u32);
        let surb = surb_via(relayer, DIRECT).context("fixture SURB")?;
        for i in 0..512u32 {
            let batch: Vec<_> = (0..16u32)
                .map(|j| ([((i * 16 + j) % 256) as u8; 8], surb.clone()))
                .collect();
            store.insert_surbs(HoprPseudonym::random(), batch);
        }
        store.quarantine_surbs.run_pending_tasks();

        // Pin the counter at its cap: the point under test is an admitted insert decided while the
        // budget has nothing left, and reaching that state through churn alone is timing-dependent —
        // every drain the churn triggers hands the counter back headroom before the next insert.
        store
            .quarantine_budget
            .store(store.cfg.quarantine_surb_budget, Ordering::Relaxed);

        let second = HoprPseudonym::random();
        store.admit(&second, MIN_SURB_RB_CAPACITY);
        let outcome = store.insert_surbs(second, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);

        assert_eq!(
            1, outcome.retained,
            "an admitted insert must be retained, whatever the quarantine budget is doing"
        );
        assert_eq!(
            0,
            store
                .find_surb(SurbMatcher::Pseudonym(second))
                .context("expected a SURB")?
                .remaining,
            "and it must still be there to be spent"
        );

        Ok(())
    }

    /// The budget drain evicts whole rings as explicit removals, which the quarantine listener skips,
    /// so the drain is the only place that can refund what it sheds. Missing that decrement pinned the
    /// counter at the cap under exactly the fresh-pseudonym churn the budget exists to bound: the
    /// quarantine emptied out while the counter stayed full, and every later fresh SURB was refused
    /// for good. After a flood that drove the drain, the counter must read what the quarantine
    /// actually holds, and a fresh SURB must be accepted again.
    #[test]
    fn the_quarantine_budget_recovers_after_the_budget_drain_empties_the_quarantine() -> anyhow::Result<()> {
        // Full 16-SURB rings against the budget floor: 512 pseudonyms × 16 = 8192 unadmitted SURBs,
        // well past the 4096 budget, while staying inside the 1024-pseudonym cache — so every ring the
        // store sheds is shed by the budget drain, not by the cache's own LRU.
        let store = MemorySurbStore::new(SurbStoreConfig {
            quarantine_surb_budget: 4096,
            ..Default::default()
        });
        let relayer = HoprKeyIdent::from(1u32);
        let inserted = 512 * 16;

        // Sustained churn: the drain must run repeatedly and the counter must come back down every
        // time, not ratchet up to the cap.
        let surb = surb_via(relayer, DIRECT).context("fixture SURB")?;
        for i in 0..512u32 {
            let batch: Vec<_> = (0..16u32)
                .map(|j| ([((i * 16 + j) % 256) as u8; 8], surb.clone()))
                .collect();
            store.insert_surbs(HoprPseudonym::random(), batch);
        }
        store.quarantine_surbs.run_pending_tasks();

        // The counter is the only record of what the quarantine holds, so it must agree with the
        // rings actually resident — a pinned counter over-reports `surbs_held` and refuses forever.
        let resident: usize = store.quarantine_surbs.iter().map(|(_, rb)| rb.len()).sum();
        assert!(
            resident < inserted,
            "precondition: the budget drain must have shed rings, resident {resident} of {inserted}"
        );
        assert_eq!(
            resident,
            store.quarantine_budget.load(Ordering::Relaxed),
            "the budget counter must track the {resident} SURBs the quarantine actually holds"
        );

        // Whatever the drain left, a brand-new pseudonym must be able to write into the quarantine.
        let fresh = HoprPseudonym::random();
        let outcome = store.insert_surbs(fresh, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);
        assert_eq!(
            1, outcome.retained,
            "a fresh SURB must be accepted after the flood, not refused by a pinned counter"
        );
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(fresh)).is_some(),
            "the accepted SURB must still be spendable"
        );

        Ok(())
    }

    /// A lookup that discards every SURB it examines — here because their first relayer was
    /// invalidated — empties the ring and yields `None`. The refund used to sit behind that `None`
    /// short-circuit, so the discarded SURBs were never subtracted and the counter pinned at the cap
    /// the same way the drain leak pinned it.
    #[test]
    fn the_quarantine_budget_refunds_surbs_discarded_by_a_failing_lookup() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        // Two quarantine SURBs returning via a relayer that is invalidated before the lookup: both
        // are discarded and the lookup comes up empty.
        store.insert_surbs(
            pseudonym,
            vec![
                ([1u8; 8], surb_via(relayer, TWO_HOP)?),
                ([2u8; 8], surb_via(relayer, TWO_HOP)?),
            ],
        );
        store.invalidate_relayer(&relayer);

        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none(),
            "precondition: the lookup must discard both SURBs and yield nothing"
        );
        assert_eq!(
            0,
            store.quarantine_budget.load(Ordering::Relaxed),
            "both discarded SURBs must be refunded to the budget"
        );
        assert_eq!(0, store.surbs_held(), "nothing of the pseudonym remains held");

        // The refund must leave the quarantine usable for the next pseudonym.
        let fresh = HoprPseudonym::random();
        let outcome = store.insert_surbs(fresh, vec![([3u8; 8], surb_via(relayer, DIRECT)?)]);
        assert_eq!(
            1, outcome.retained,
            "a fresh SURB must still be accepted after the discard emptied the budget"
        );

        Ok(())
    }

    /// Admission used to be bounded by `max_pseudonyms`, which the operator cannot raise past the
    /// Session census without also unbounding the reply openers. Admitted rings must outlive that
    /// value now: `max_pseudonyms` no longer sizes any receiving-side cache.
    #[test]
    fn admitted_pseudonyms_above_max_pseudonyms_do_not_degrade() -> anyhow::Result<()> {
        // `max_pseudonyms` is floored at 1000, so 1500 admissions must exceed it while staying well
        // under `admitted_max_pseudonyms`.
        let store = MemorySurbStore::new(SurbStoreConfig {
            max_pseudonyms: 100,
            admitted_max_pseudonyms: 2000,
            ..Default::default()
        });
        let relayer = HoprKeyIdent::from(1u32);

        for i in 0..1500 {
            let pseudonym = HoprPseudonym::random();
            store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);
            store.insert_surbs(pseudonym, vec![([i as u8; 8], surb_via(relayer, DIRECT)?)]);
        }
        store.admitted_surbs_per_pseudonym.run_pending_tasks();

        let shed = store.stats.state.lock().counts[EvictedCache::SurbRing as usize].size;
        assert_eq!(
            0, shed,
            "1500 admitted pseudonyms must all be held; the admitted cache saw {shed} size evictions"
        );
        let readable = store
            .admitted_surbs_per_pseudonym
            .iter()
            .filter(|(_, rb)| rb.len() == 1)
            .count();
        assert_eq!(
            1500, readable,
            "every admitted ring must still be readable, not just the newest ones"
        );

        Ok(())
    }

    /// A ring that accumulated in quarantine before the Session over it was established must move
    /// into the admitted class on admission — at the admitted capacity, not the quarantine one — and
    /// the transfer must not double-count a single SURB.
    #[test]
    fn quarantine_surbs_should_be_promoted_on_admission() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        for i in 0..10 {
            store.insert_surbs(pseudonym, vec![([i as u8; 8], surb_via(relayer, DIRECT)?)]);
        }
        let before = store.surbs_held();
        assert_eq!(
            10,
            store
                .quarantine_surbs
                .get(&pseudonym)
                .context("precondition: ring in quarantine")?
                .len(),
            "precondition: the unadmitted ring holds all ten"
        );

        store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);

        let admitted = store
            .admitted_surbs_per_pseudonym
            .get(&pseudonym)
            .context("the admitted ring must exist after admission")?;
        assert_eq!(10, admitted.len(), "the promoted ring carries all ten SURBs");
        assert!(
            store.quarantine_surbs.get(&pseudonym).is_none(),
            "the quarantine entry must be gone"
        );
        assert_eq!(
            before,
            store.surbs_held(),
            "a promotion moves SURBs, it does not create or lose them"
        );

        Ok(())
    }

    /// Re-admitting a fixed pseudonym — a Session reopened on the same one — must not discard what
    /// the admitted ring already holds.
    #[test]
    fn admit_should_not_clobber_an_existing_admitted_ring() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);
        store.insert_surbs(pseudonym, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);
        store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);

        assert_eq!(
            1,
            store
                .admitted_surbs_per_pseudonym
                .get(&pseudonym)
                .context("the admitted ring must still exist")?
                .len(),
            "the second admission must have left the ring's contents alone"
        );
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_some(),
            "and the SURB must still be spendable"
        );

        Ok(())
    }

    /// The admitted arm: a batch for an admitted pseudonym must land in the entry `admit` installed,
    /// used as it stands — the real `admit` already promoted the quarantine ring and stamped it with
    /// the number the SessionManager threads back to `retire`. Calling `admit` again would mint a
    /// fresh number the manager's token no longer names, and the Session's deferred retire would then
    /// match nothing: the ring would leak until a teardown that never comes, and the retire would be
    /// booked as stale.
    ///
    /// The test drives the arm directly with the entry `admit` produced, and then checks the
    /// properties that matter: the epoch the manager holds is unchanged, the SURB landed in the
    /// admitted ring, and a retire with the manager's token still releases it.
    #[test]
    fn the_admitted_arm_must_reuse_the_admission_the_session_manager_holds() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        // The Session is established; `admit` mints the number the manager stores in the slot's
        // admission token and promotes anything the quarantine already held.
        let token = store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);

        // The insert finds the entry `admit` installed and hands the batch to the admitted arm with
        // it — the path the old code took through a second `admit`.
        let admitted = store
            .admitted_surbs_per_pseudonym
            .get(&pseudonym)
            .context("the Session's admitted entry must exist")?;
        // `AdmittedRing::clone`, not the deref'd ring's: the arm takes the whole entry, epoch and all.
        let admitted = AdmittedRing::clone(&admitted);
        let outcome =
            store.insert_surbs_into_admitted(&pseudonym, admitted, vec![([1u8; 8], surb_via(relayer, DIRECT)?)], 0);
        assert_eq!(
            SurbInsertOutcome {
                retained: 1,
                evicted: 0,
                evicted_shares: 0
            },
            outcome,
            "the batch must be retained in the admitted ring, not strained against the quarantine cap"
        );

        let admitted = store
            .admitted_surbs_per_pseudonym
            .get(&pseudonym)
            .context("the admitted entry must still exist")?;
        assert_eq!(
            token,
            admitted.epoch(),
            "the admission number the manager holds must be untouched — a second `admit` here would have minted a \
             fresh one its retire can never match"
        );
        assert_eq!(1, admitted.len(), "the racing batch must be held in the admitted ring");
        assert!(
            store.quarantine_surbs.get(&pseudonym).is_none(),
            "nothing may be left behind in the quarantine the promotion already drained"
        );

        // The teardown, carrying the manager's token, still releases the ring: the leak the second
        // `admit` would have caused is what this asserts away.
        store.retire(&pseudonym, token);
        assert!(
            store.admitted_surbs_per_pseudonym.get(&pseudonym).is_none(),
            "the manager's own token must still retire the Session's ring"
        );

        Ok(())
    }

    /// Retiring a Session releases only what the admitted class holds: the SURBs go with the Session,
    /// but a pseudonym that was never admitted is untouched — as is anything still sitting in the
    /// quarantine, which expires on its own short TTI without any teardown bookkeeping.
    #[test]
    fn retire_should_release_only_the_admitted_class() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let other = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        let token = store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);
        store.insert_surbs(pseudonym, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);
        store.insert_surbs(other, vec![([2u8; 8], surb_via(relayer, DIRECT)?)]);

        store.retire(&pseudonym, token);

        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_none(),
            "retired SURBs must not be handed out"
        );
        assert!(
            store.admitted_surbs_per_pseudonym.get(&pseudonym).is_none(),
            "the admitted ring must be gone"
        );
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(other)).is_some(),
            "an unadmitted pseudonym's SURBs are not the retired Session's to release"
        );
        // Retiring again must be a no-op, not a panic: teardown paths can overlap.
        store.retire(&pseudonym, token);

        Ok(())
    }

    /// The regression test for the deferred-retire hazard: a teardown whose retire was pushed past the
    /// packet pipeline's SURB-resolution window must not destroy a ring a successor Session admitted
    /// on the same client-chosen pseudonym in the meantime.
    ///
    /// Re-admitting `pseudonym` between the first admission and the retire stands in for the successor;
    /// the stale token from the first admission then stands in for the retire the teardown defers.
    /// The successor's ring must survive it, contents and all — anything less and the successor would
    /// go on inserting its SURBs into the small un-admitted quarantine with nothing logged anywhere.
    #[test]
    fn a_stale_retire_must_not_clobber_a_successor_admissions_ring() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        let stale_token = store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);

        // The successor's admission, standing in for the Session that was established while the first
        // Session's teardown was still waiting out the SURB-resolution window. It mints a fresh token
        // and keeps what the admitted entry already holds, so the SURB below survives it.
        let successor_token = store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);
        store.insert_surbs(pseudonym, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_some(),
            "the successor's SURB must resolve before the stale retire fires"
        );

        // Re-insert, since the probe above spent the ring's only SURB.
        store.insert_surbs(pseudonym, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);
        store.retire(&pseudonym, stale_token);

        assert_ne!(
            stale_token, successor_token,
            "two admissions of one pseudonym must not share a token, or no retire could tell them apart"
        );
        assert!(
            store
                .admitted_surbs_per_pseudonym
                .get(&pseudonym)
                .context("the successor's admitted ring must still exist")?
                .epoch()
                == successor_token,
            "the stale retire must not have disturbed the successor's admission number"
        );
        assert_eq!(
            1,
            store
                .admitted_surbs_per_pseudonym
                .get(&pseudonym)
                .context("the successor's admitted ring must still exist")?
                .len(),
            "the successor's SURB must still be held"
        );
        assert!(
            store.find_surb(SurbMatcher::Pseudonym(pseudonym)).is_some(),
            "and it must still be spendable — the deferred retire of the Session before it must not have invalidated \
             the successor's return path"
        );

        // The successor's own teardown, carrying its own token, still works.
        store.retire(&pseudonym, successor_token);
        assert!(
            store.admitted_surbs_per_pseudonym.get(&pseudonym).is_none(),
            "a retire whose token matches the current admission must still release the ring"
        );

        Ok(())
    }

    /// A retire whose token names no admission must be a no-op: a rolled-back Session was never
    /// admitted, and its teardown still runs the retire through the same seam as every other teardown.
    #[test]
    fn a_retire_with_an_unknown_token_must_not_touch_any_ring() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        let token = store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);
        store.insert_surbs(pseudonym, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);
        let wrong_token = store.admit(&HoprPseudonym::random(), MIN_SURB_RB_CAPACITY);

        store.retire(&pseudonym, wrong_token);

        assert!(
            store
                .admitted_surbs_per_pseudonym
                .get(&pseudonym)
                .context("the admitted ring must still exist")?
                .epoch()
                == token,
            "a token minted for another pseudonym must not release this one's ring"
        );
        assert_eq!(
            1,
            store
                .admitted_surbs_per_pseudonym
                .get(&pseudonym)
                .context("the admitted ring must still exist")?
                .len(),
            "and its SURB must still be held"
        );

        Ok(())
    }

    /// Only `admit` may create an admitted entry; an insert that finds none must route to the
    /// quarantine.
    ///
    /// The failure this rules out: a deferred `retire` removes the key between `insert_surbs`'
    /// admission test and its entry lookup, and an `or_insert_with` there would recreate the ring at
    /// epoch `0` — which no retire matches, so the entry leaks until size eviction, and a successor
    /// `admit` finding it would only re-stamp the epoch, leaving it at the floor capacity so later
    /// inserts overflow it and lose SURBs and PIX shares.
    #[test]
    fn an_insert_must_not_create_an_admitted_entry() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        // A Session was admitted and retired before this batch arrived: the admitted entry is gone.
        let token = store.admit(&pseudonym, MIN_SURB_RB_CAPACITY);
        store.retire(&pseudonym, token);
        assert!(
            store.admitted_surbs_per_pseudonym.get(&pseudonym).is_none(),
            "the retire must have removed the admitted entry"
        );

        let outcome = store.insert_surbs(pseudonym, vec![([1u8; 8], surb_via(relayer, DIRECT)?)]);

        assert_eq!(
            SurbInsertOutcome {
                retained: 1,
                evicted: 0,
                evicted_shares: 0
            },
            outcome,
            "the batch must be retained, in the quarantine"
        );
        assert!(
            store.admitted_surbs_per_pseudonym.get(&pseudonym).is_none(),
            "an insert must never resurrect an admitted entry — only `admit` may create one"
        );
        assert!(
            store.quarantine_surbs.get(&pseudonym).is_some(),
            "the unadmitted batch belongs to the quarantine"
        );

        Ok(())
    }

    /// Probe replies resolve against unadmitted pseudonyms (a probe names whatever pseudonym it
    /// replies to), so `find_surb` must consult the quarantine — and must *not* promote what it finds
    /// there: which pseudonym deserves admission is decided by the Session lifecycle, and the matcher
    /// names a pseudonym the counterparty chose.
    #[test]
    fn find_surb_should_consult_quarantine_for_probe_replies() -> anyhow::Result<()> {
        let store = MemorySurbStore::default();
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);

        store.insert_surbs(pseudonym, vec![([3u8; 8], surb_via(relayer, DIRECT)?)]);

        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .context("a quarantined SURB must still resolve")?;
        assert_eq!([3u8; 8], found.sender_id.surb_id());
        assert!(
            store.admitted_surbs_per_pseudonym.get(&pseudonym).is_none(),
            "a pop must not promote the pseudonym into the admitted class"
        );
        assert!(
            store.quarantine_surbs.get(&pseudonym).is_some(),
            "and the pseudonym stays in the quarantine it came from"
        );

        Ok(())
    }

    /// The new config fields each enforce their floor, and a configuration file that predates them —
    /// with none of the six present — still loads, because every one carries its own serde default
    /// under `deny_unknown_fields`.
    #[cfg(feature = "serde")]
    #[test]
    fn surb_store_config_should_reject_out_of_range_quarantine_and_budget_fields() -> anyhow::Result<()> {
        use serde::{
            Deserialize,
            de::value::{Error as SerdeError, MapDeserializer},
        };
        use validator::Validate;

        // The config is deserialized from a key-value map, which is all a configuration file is to
        // serde — the same approach `surb_store_config_should_reject_a_lifo_pop_order` uses. The map
        // values are typed the way a file would carry them: numbers as `u64`, durations as humantime
        // strings.
        let load = |pairs: Vec<(&'static str, u64)>| -> anyhow::Result<SurbStoreConfig> {
            // The range checks are validator rules, not serde ones: they run when the loaded config
            // is validated, exactly as a configuration file's would be.
            let cfg = SurbStoreConfig::deserialize(MapDeserializer::<_, SerdeError>::new(pairs.into_iter()))
                .context("config must load")?;
            cfg.validate().context("config must validate")?;
            Ok(cfg)
        };

        assert!(
            load(vec![("quarantine_rb_capacity", 0)]).is_err(),
            "a zero quarantine ring capacity cannot hold even one packet"
        );
        assert!(
            load(vec![("quarantine_max_pseudonyms", 63)]).is_err(),
            "below the quarantine pseudonym floor must be rejected"
        );
        assert!(
            load(vec![("quarantine_surb_budget", 4095)]).is_err(),
            "below the quarantine budget floor must be rejected"
        );
        assert!(
            load(vec![("admitted_max_pseudonyms", 0)]).is_err(),
            "an admitted cache of zero cannot hold even one session"
        );

        // Each field at its boundary must load and validate.
        load(vec![
            ("quarantine_rb_capacity", 1),
            ("quarantine_max_pseudonyms", 64),
            ("quarantine_surb_budget", 4096),
            ("admitted_max_pseudonyms", 1),
            ("admitted_headroom_percent", 0),
        ])
        .context("boundary values must load and validate")?;

        // A lifetime is validated by the custom rule floored at the SURB lifetime.
        let short = SurbStoreConfig {
            quarantine_lifetime: MINIMUM_SURB_LIFETIME - Duration::from_millis(1),
            ..Default::default()
        };
        assert!(
            short.validate().is_err(),
            "a quarantine lifetime below the SURB lifetime floor must be rejected"
        );

        // A file that predates the new fields must still load: every field is defaulted.
        let legacy = load(vec![("rb_capacity", 100_000), ("distress_threshold", 500)])
            .context("a pre-admission config must still load")?;
        assert_eq!(16, legacy.quarantine_rb_capacity, "the quarantine cap is defaulted");
        assert_eq!(
            10_000, legacy.admitted_max_pseudonyms,
            "the admitted census is defaulted"
        );

        Ok(())
    }
}
