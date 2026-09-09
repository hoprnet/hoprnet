use std::{collections::VecDeque, sync::Arc, time::Duration};

use hopr_api::types::internal::{prelude::HoprPseudonym, routing::SurbMatcher};
use hopr_crypto_packet::prelude::*;
use moka::notification::RemovalCause;
use validator::ValidationError;

use crate::{FoundSurb, traits::SurbStore};

/// Lower bound on [`SurbStoreConfig::pseudonyms_lifetime`], enforced by the config validator.
///
/// Public so that callers applying their own override can floor it identically, rather than
/// reaching a value the config file itself would have been rejected for.
pub const MINIMUM_SURB_LIFETIME: Duration = Duration::from_secs(30);
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

/// Which end of the per-pseudonym buffer a pop consumes from. Replying side only.
///
/// Chooses the consumption order *within* a SURB generation. It is orthogonal to the per-SURB
/// generation tag (`SurbReceiverInfo::generation`), which handles an explicit return-path re-plan by
/// clearing superseded SURBs wholesale on the next push. Overflow always evicts the oldest SURB, in
/// either order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumString, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SurbPopOrder {
    /// Oldest first. Default; preserves the historical behaviour.
    #[default]
    Fifo,
    /// Newest first, so a return-path change applies sooner instead of only after the buffered SURBs
    /// drain. Stale ones are shed from the other end on overflow.
    Lifo,
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
    /// Once the buffer is full, a push overwrites the oldest SURBs, which are then never used.
    /// With PIX that is not merely a wasted SURB: each one carries a partial SSA share that is only
    /// delivered to the reconstructor when the SURB is *used*, so an overwrite is a permanently
    /// lost share. The capacity is therefore sized well above what any Session's SURB balancer
    /// targets — see `maximum_surb_buffer_size` in `hopr-transport`, which is derived from this
    /// value with headroom left for balancer overshoot.
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
    /// Which end of the per-pseudonym buffer a pop consumes from; see [`SurbPopOrder`].
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
    cfg: Arc<SurbStoreConfig>,
}

impl MemorySurbStore {
    /// Creates a new instance with the given configuration.
    pub fn new(cfg: SurbStoreConfig) -> Self {
        Self {
            // Reply openers are indexed by entire Sender IDs (Pseudonym + SURB ID)
            // in a cascade fashion, allowing the entire batches (by Pseudonym) to be evicted
            // if not used.
            pseudonym_openers: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|sender_id, _reply_opener, cause| {
                    tracing::warn!(?sender_id, ?cause, "evicting reply opener for pseudonym");
                })
                .max_capacity(cfg.max_openers_per_pseudonym.max(MINIMUM_OPENER_PSEUDONYMS) as u64)
                .build(),
            // SURBs are indexed only by Pseudonyms, which have longer lifetimes.
            // For each Pseudonym, there's an RB of SURBs and their IDs.
            surbs_per_pseudonym: moka::sync::Cache::builder()
                .time_to_idle(cfg.pseudonyms_lifetime.max(MINIMUM_SURB_LIFETIME))
                .eviction_policy(moka::policy::EvictionPolicy::lru())
                .eviction_listener(|pseudonym, _reply_opener, cause| {
                    tracing::warn!(%pseudonym, ?cause, "evicting surb for pseudonym");
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
                .eviction_listener(|pseudonym, _generation, cause| {
                    // Under normal operation minting keeps this entry warm; an eviction here can
                    // reset the generation serial and have the peer reject fresh SURBs.
                    tracing::warn!(%pseudonym, ?cause, "evicting SURB generation for pseudonym");
                })
                .max_capacity(cfg.max_openers_per_pseudonym.max(MINIMUM_OPENER_PSEUDONYMS) as u64)
                .build(),
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
            // The following code intentionally only checks the SURB at the popping end of the
            // ring buffer and does not search the entire RB.
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
    fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize {
        // A batch is one packet's worth of SURBs, minted by the creator at a single generation, so
        // the generation of any one of them stands for the whole batch. An empty batch carries no
        // generation and must not create or disturb the buffer.
        let Some(generation) = surbs
            .first()
            .map(|(_, surb)| surb.additional_data_receiver.generation())
        else {
            return self.surbs_per_pseudonym.get(&pseudonym).map(|rb| rb.len()).unwrap_or(0);
        };

        self.surbs_per_pseudonym
            .entry_by_ref(&pseudonym)
            .or_insert_with(|| SurbRingBuffer::new(self.cfg.rb_capacity.max(MIN_SURB_RB_CAPACITY), self.cfg.pop_order))
            .value()
            .push(surbs, generation)
    }

    #[tracing::instrument(skip_all, level = "trace", fields(?sender_id))]
    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener) {
        let opener_lifetime = self.cfg.reply_opener_lifetime.max(MINIMUM_OPENER_LIFETIME);
        let max_openers_per_pseudonym = self.cfg.max_openers_per_pseudonym.max(MINIMUM_OPENERS_PER_PSEUDONYM);
        self.pseudonym_openers
            .get_with(sender_id.pseudonym(), move || {
                moka::sync::Cache::builder()
                    .time_to_live(opener_lifetime)
                    .eviction_listener(move |id: Arc<HoprSurbId>, _, cause| {
                        if cause != RemovalCause::Explicit {
                            tracing::warn!(
                                pseudonym = %sender_id.pseudonym(),
                                surb_id = const_hex::encode(id.as_slice()),
                                ?cause,
                                "evicting reply opener for sender id"
                            );
                        }
                    })
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

/// Ring buffer of SURBs and their IDs, all belonging to one pseudonym and therefore identified only
/// by [`HoprSurbId`].
///
/// Backed by a [`VecDeque`] that is never allowed to exceed `capacity`: a push into a full buffer
/// evicts the oldest element first. [`SurbPopOrder`] picks which end a pop consumes from; overflow
/// always evicts the oldest, in either order.
///
/// ## Generations: dropping SURBs for a superseded return path
///
/// A return path that dies deep in a multi-hop route is invisible to this replying side — nothing
/// here can tell a stale SURB from a live one. The SURB creator can: it stamps every SURB of a
/// batch with a generation (`SurbReceiverInfo::generation`) and bumps it whenever it changes the
/// return path. This buffer keeps only the highest generation it has seen: the first push carrying a
/// newer generation **clears the buffer wholesale** before inserting, so a return-path change takes
/// effect on the very next reply rather than only once the stale SURBs drain — and stale SURBs are
/// never handed out. A push carrying an older generation (a late/reordered batch) is discarded.
/// Clearing is a per-path-change O(n) sweep, so pops need no per-SURB generation check.
///
/// ## Why the capacity is a ceiling, not a reservation
///
/// The deque grows with occupancy rather than being sized at construction. The distinction matters
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
/// Growth is geometric and amortised, and the deque never shrinks below its high-water mark, so a
/// pseudonym that genuinely fills up still ends at the same footprint — and stops reallocating
/// there, however long the steady-state overflow runs. It just has to earn it.
#[derive(Clone, Debug)]
pub struct SurbRingBuffer<S> {
    inner: Arc<parking_lot::Mutex<GenerationalBuffer<S>>>,
    /// Ceiling on the number of retained SURBs; the oldest are dropped once a push would exceed it.
    capacity: usize,
    /// Which end a pop consumes from; overflow always evicts the oldest regardless.
    pop_order: SurbPopOrder,
}

/// The mutex-protected state of a [`SurbRingBuffer`]: the SURBs and the highest generation seen.
///
/// Both live under one lock so that clearing the deque and advancing the generation on a newer
/// batch is atomic against a concurrent pop.
#[derive(Debug)]
struct GenerationalBuffer<S> {
    surbs: VecDeque<(HoprSurbId, S)>,
    /// Highest generation seen; `None` until the first push.
    generation: Option<u8>,
}

impl<S> GenerationalBuffer<S> {
    /// The end a pop consumes from under `order` (front for FIFO, back for LIFO). The single place
    /// the order-to-end polarity is decided, so [`pop_end`](Self::pop_end) and its peek stay in step.
    fn peek_end(&self, order: SurbPopOrder) -> Option<&(HoprSurbId, S)> {
        match order {
            SurbPopOrder::Fifo => self.surbs.front(),
            SurbPopOrder::Lifo => self.surbs.back(),
        }
    }

    /// Removes and returns the SURB at the end `order` consumes from.
    fn pop_end(&mut self, order: SurbPopOrder) -> Option<(HoprSurbId, S)> {
        match order {
            SurbPopOrder::Fifo => self.surbs.pop_front(),
            SurbPopOrder::Lifo => self.surbs.pop_back(),
        }
    }
}

impl<S> SurbRingBuffer<S> {
    /// Creates a buffer holding at most `capacity` (min 1, so a push is never a no-op) SURBs,
    /// popped in the given order.
    pub fn new(capacity: usize, pop_order: SurbPopOrder) -> Self {
        Self {
            inner: Arc::new(parking_lot::Mutex::new(GenerationalBuffer {
                surbs: VecDeque::new(),
                generation: None,
            })),
            // A zero capacity would make the eviction below pop from an empty deque and then push
            // past the bound. Callers already clamp to `MIN_SURB_RB_CAPACITY`; this is belt-and-braces.
            capacity: capacity.max(1),
            pop_order,
        }
    }

    /// Pushes all SURBs of one batch, stamped with `generation`, evicting the oldest past capacity.
    ///
    /// A batch is minted at a single generation by the creator (one packet's worth of SURBs), so a
    /// single `generation` covers the whole `surbs` iterator. Relative to the highest generation
    /// seen so far:
    /// - **newer** → the buffer is cleared before inserting, so SURBs for the superseded return path are dropped at
    ///   once rather than lingering until they drain;
    /// - **equal** → the batch is appended (an ordinary refill);
    /// - **older** → the batch is discarded as a late/reordered leftover.
    ///
    /// Once at capacity, each insert evicts the oldest SURB. Under PIX that is a lost SSA share, not
    /// merely a lost SURB — see [`SurbStoreConfig::rb_capacity`].
    ///
    /// Returns the number of elements held after the push.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I, generation: u8) -> usize {
        let mut inner = self.inner.lock();

        match inner.generation {
            Some(current) if generation == current => {} // ordinary refill: append below
            Some(current) if generation_is_newer(generation, current) => {
                // Return path changed: everything held is for the superseded path. Drop it wholesale
                // so the newer batch is all that remains and the next reply uses the live path.
                inner.surbs.clear();
                inner.generation = Some(generation);
            }
            Some(_) => {
                // Older than what we already hold: a late or reordered batch for a path the creator
                // has already moved on from. Discard it rather than reintroduce stale SURBs.
                return inner.surbs.len();
            }
            None => inner.generation = Some(generation),
        }

        for surb in surbs {
            // Evict before inserting, so the length never exceeds the ceiling and the backing
            // allocation stops growing once the high-water mark is reached.
            if inner.surbs.len() >= self.capacity {
                inner.surbs.pop_front();
            }
            inner.surbs.push_back(surb);
        }
        inner.surbs.len()
    }

    /// Pops the next SURB that `is_valid` accepts, in the buffer's [`SurbPopOrder`].
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
                let (id, surb) = inner.pop_end(self.pop_order)?;
                (id, surb, inner.surbs.len())
            };

            if is_valid(&id, &surb) {
                return Some(PoppedSurb { id, surb, remaining });
            }
        }
    }

    /// Number of SURBs currently held.
    fn len(&self) -> usize {
        self.inner.lock().surbs.len()
    }

    /// Pops the next SURB (in the buffer's [`SurbPopOrder`]) only if it has the given ID.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Option<PoppedSurb<S>> {
        let mut inner = self.inner.lock();

        if inner.peek_end(self.pop_order).is_some_and(|(surb_id, _)| surb_id == id) {
            let (id, surb) = inner.pop_end(self.pop_order)?;
            Some(PoppedSurb {
                id,
                surb,
                remaining: inner.surbs.len(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto::crypto_traits::Randomizable;
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

    /// Eviction always removes the oldest, so both orders see the same surviving set {2,3,4}, but
    /// consume it from opposite ends (all within one generation).
    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo, [[2u8; 8], [3u8; 8], [4u8; 8]])]
    #[case::lifo(SurbPopOrder::Lifo, [[4u8; 8], [3u8; 8], [2u8; 8]])]
    fn surb_ring_buffer_should_drop_oldest_items_when_capacity_is_reached(
        #[case] order: SurbPopOrder,
        #[case] expected: [HoprSurbId; 3],
    ) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3, order);
        rb.push([([1u8; 8], 0)], 0);
        rb.push([([2u8; 8], 0)], 0);
        rb.push([([3u8; 8], 0)], 0);
        rb.push([([4u8; 8], 0)], 0);

        for (i, expected_id) in expected.into_iter().enumerate() {
            let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
            assert_eq!(expected_id, popped.id, "unexpected id at index {i}");
            assert_eq!(expected.len() - 1 - i, popped.remaining, "unexpected remaining");
        }

        assert!(rb.pop_any().is_none(), "buffer should be drained");

        Ok(())
    }

    /// Two SURBs pushed as {1, 2}: FIFO hands out 1 then 2, LIFO hands out 2 then 1 — and the same
    /// order holds for a fresh batch pushed after the buffer drains. (That FIFO is the *default* is
    /// asserted separately by `surb_store_config_should_default_to_fifo`.)
    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo, [[1u8; 8], [2u8; 8]])]
    #[case::lifo(SurbPopOrder::Lifo, [[2u8; 8], [1u8; 8]])]
    fn surb_ring_buffer_should_consume_from_the_configured_end(
        #[case] order: SurbPopOrder,
        #[case] expected: [HoprSurbId; 2],
    ) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, order);

        assert_eq!(1, rb.push([([1u8; 8], 0)], 0));
        assert_eq!(2, rb.push([([2u8; 8], 0)], 0));

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!(expected[0], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!(expected[1], popped.id);
        assert_eq!(0, popped.remaining);

        // A fresh batch after draining consumes from the same end.
        assert_eq!(2, rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0));
        assert_eq!(expected[0], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!(expected[1], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo)]
    #[case::lifo(SurbPopOrder::Lifo)]
    fn surb_ring_buffer_should_skip_entries_failing_the_predicate(#[case] order: SurbPopOrder) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, order);
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
        let rb = SurbRingBuffer::new(5, SurbPopOrder::Lifo);
        rb.push([([1u8; 8], 0), ([2u8; 8], 0)], 0);

        assert!(rb.pop_next_valid(|_, _| false).is_none());
        // The buffer is fully drained by the exhaustive search.
        assert!(rb.pop_any().is_none());

        Ok(())
    }

    /// The buffer grows with occupancy, so it does reallocate on the way up to its ceiling — that
    /// is the point of
    /// [`surb_ring_buffer_must_allocate_with_occupancy_not_capacity`]. What must not happen is
    /// churn *afterwards*: once a pseudonym has filled its buffer, an unbounded stream of
    /// pushes and pops must not keep reallocating. So the high-water mark is reached first and
    /// sampled there, rather than at construction.
    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo)]
    #[case::lifo(SurbPopOrder::Lifo)]
    fn surb_ring_buffer_should_not_reallocate_under_steady_overflow(#[case] order: SurbPopOrder) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(8, order);

        for i in 0..8u32 {
            rb.push([(((i as u64).to_be_bytes()), 0)], 0);
        }
        let settled_capacity = rb.inner.lock().surbs.capacity();
        assert!(settled_capacity >= 8, "8 SURBs must actually fit");

        for i in 0..1_000u32 {
            rb.push([(((i as u64).to_be_bytes()), 0)], 0);
            if i % 3 == 0 {
                rb.pop_any();
            }
            assert!(rb.inner.lock().surbs.len() <= 8, "length exceeded capacity");
        }

        assert_eq!(settled_capacity, rb.inner.lock().surbs.capacity(), "buffer reallocated");

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
        let rb = SurbRingBuffer::<u64>::new(CAPACITY, SurbPopOrder::default());

        let empty = rb.inner.lock().surbs.capacity();
        assert!(empty < CAPACITY / 100, "a buffer holding nothing reserved for {empty}");

        for i in 0..10u64 {
            rb.push([([i as u8; 8], i)], 0);
        }

        let allocated = rb.inner.lock().surbs.capacity();
        assert!(allocated >= 10, "10 SURBs must actually fit");
        assert!(
            allocated < CAPACITY / 100,
            "10 SURBs reserved for {allocated} — allocation is tracking capacity, not occupancy"
        );
    }

    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo)]
    #[case::lifo(SurbPopOrder::Lifo)]
    fn surb_ring_buffer_should_not_pop_if_id_does_not_match(#[case] order: SurbPopOrder) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, order);

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

    /// `pop_one_if_has_id` checks only the popping end — FIFO the front (oldest), LIFO the back
    /// (newest). Given {1, 2}, a match for the id at the *other* end is not popped; the id at the
    /// consuming end is.
    #[rstest]
    #[case::fifo(SurbPopOrder::Fifo, [2u8; 8], [1u8; 8])]
    #[case::lifo(SurbPopOrder::Lifo, [1u8; 8], [2u8; 8])]
    fn surb_ring_buffer_should_check_the_popping_end_for_an_exact_id(
        #[case] order: SurbPopOrder,
        #[case] other_end_id: HoprSurbId,
        #[case] popping_end_id: HoprSurbId,
    ) -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5, order);
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
        let rb = SurbRingBuffer::new(64, SurbPopOrder::Fifo);
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
        let rb = SurbRingBuffer::new(64, SurbPopOrder::Fifo);
        rb.push([([1u8; 8], 0)], 7);
        rb.push([([2u8; 8], 0)], 7);
        assert_eq!(2, rb.len(), "same-generation batches accumulate");
        assert_eq!([1u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!([2u8; 8], rb.pop_any().ok_or(anyhow::anyhow!("expected pop"))?.id);
        Ok(())
    }

    #[test]
    fn surb_ring_buffer_should_discard_a_stale_older_generation_batch() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(64, SurbPopOrder::Fifo);
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
        let rb = SurbRingBuffer::new(64, SurbPopOrder::Fifo);
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
        let rb = SurbRingBuffer::new(64, SurbPopOrder::Fifo);

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
        let rb = SurbRingBuffer::new(64, SurbPopOrder::Fifo);
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
        let rb = SurbRingBuffer::new(64, SurbPopOrder::Fifo);
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
        let rb = SurbRingBuffer::new(3, SurbPopOrder::Fifo);
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

    /// A configured `pop_order = lifo` is honoured end-to-end at the store: within a generation, the
    /// newest buffered SURB is handed out first.
    #[test]
    fn memory_surb_store_should_consume_lifo_when_configured() -> anyhow::Result<()> {
        let cfg = SurbStoreConfig {
            pop_order: SurbPopOrder::Lifo,
            ..Default::default()
        };

        let store = MemorySurbStore::new(cfg);
        let pseudonym = HoprPseudonym::random();
        let relayer = HoprKeyIdent::from(1u32);
        store.insert_surbs(
            pseudonym,
            vec![
                ([1u8; 8], surb_via(relayer, TWO_HOP)?),
                ([2u8; 8], surb_via(relayer, TWO_HOP)?),
            ],
        );
        let found = store
            .find_surb(SurbMatcher::Pseudonym(pseudonym))
            .ok_or(anyhow::anyhow!("expected a usable SURB"))?;
        assert_eq!(
            [2u8; 8],
            found.sender_id.surb_id(),
            "LIFO must hand out the newest SURB first"
        );

        Ok(())
    }
}
