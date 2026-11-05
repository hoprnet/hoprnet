//! Replay detection for incoming HOPR packets.
//!
//! A packet is replayed when the same wire bytes are delivered twice. Recognising that matters: the alternative is
//! a header MAC verification, a payload permutation and a message parse for every duplicate an attacker cares to
//! send.
//!
//! The token is the packet's Sphinx authentication tag, derived from the shared secret with the local node. That
//! secret requires the elliptic-curve Diffie-Hellman, which cannot be avoided - but everything after it can, which
//! is why the tag is handed to the filter from inside the Sphinx decode rather than after it (see
//! [`HoprPacket::from_incoming_checked`](hopr_crypto_packet::prelude::HoprPacket::from_incoming_checked)).
//!
//! Keying on the derived tag rather than on the wire `alpha` matters because the tag depends on the node's
//! private key. Anybody can *compute* a tag from a chosen alpha and the node's public key, so the early
//! [`TagReplayFilter::contains`] check is not a proof of anything; what costs an attacker a well-formed packet is
//! getting a token *recorded*, which [`TagReplayFilter::commit`] does only after the header MAC has verified.
//! Rotating the filter out therefore needs packets the sender could actually authenticate, not merely bytes whose
//! first field the attacker chose.
//!
//! # Why a rolling filter rather than one Bloom filter that resets
//!
//! A Bloom filter is sized for a fixed number of items and cannot grow. The previous implementation answered
//! overflow by clearing the entire bitmap, which forgets *every* packet at once. It cleared once per capacity: the
//! insertion that found the filter full of other tags reset it, so the history of all peers was dropped on a
//! single packet.
//!
//! Instead, each shard keeps two generations. When the current one fills it becomes the previous one and a fresh
//! current is allocated, so a token is remembered for at least one full generation *of its shard* - a floor that no
//! insertion pattern can take away, where the old design had none. The floor is per shard rather than global: a
//! token's shard is chosen by a keyed hash and ages out with that shard alone, so the configured count is a bound on
//! total retention only when traffic spreads evenly across shards (see [`ReplayFilterConfig::max_tracked_packets`]).
//!
//! Aging out is proportional to insert volume (a packet count, not wall time). The cost of shortening the window is
//! *authenticated* packets, not paid ones: a token is committed once the header MAC verifies, which still precedes
//! ticket validation, so a direct neighbour can rotate generations with packets it does not pay for. Each must
//! still authenticate under the shared secret, which costs it the same elliptic-curve work a legitimate sender
//! pays, and the window it shortens is bounded below by one full generation.
//!
//! The shard a token lands in is chosen by a hash keyed per filter instance, so a sender - who computes the tag -
//! cannot aim its packets at one shard and evict only that shard's history.
//!
//! # Why sharded
//!
//! The filter is consulted by every concurrent decode task. A single mutex serialized them on one cache line;
//! sharding by token hash divides that contention by the shard count without changing the semantics, because the
//! filter is a set either way.
//!
//! The namespace is deliberately global rather than per previous hop: the token names the packet, not the sender,
//! so a replay injected through a different peer ID is still caught. The cost is that one high-volume peer
//! advances rotations for everyone; that residual is bounded by the generation floor rather than eliminated.

use hopr_api::types::crypto_random::random_bytes;

/// The token a replay filter is keyed on: the packet's Sphinx authentication tag.
///
/// The tag is derived from the shared secret with the local node, so only a party that can derive that secret can
/// produce one - an attacker cannot insert arbitrary tokens to age the filter out without paying the same
/// elliptic-curve work per packet that a legitimate sender does.
pub type ReplayToken = hopr_api::types::crypto::types::PacketTag;

/// Configuration of the [`TagReplayFilter`].
///
/// The two fields are validated together: `shards` must be a power of two in range, `max_tracked_packets` must be
/// at least [`MIN_TRACKED_PACKETS`], and the capacity must leave every shard at least [`MIN_CAPACITY_PER_SHARD`]
/// slots, i.e. `max_tracked_packets >= shards * MIN_CAPACITY_PER_SHARD`. That last condition is what the field-wise
/// ranges cannot express on their own - the floor capacity together with the shard ceiling satisfies both of them
/// yet leaves a shard with a single slot, which rotates on every distinct token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[validate(schema(function = "validate_capacity_per_shard"))]
pub struct ReplayFilterConfig {
    /// Number of tokens the filter remembers per generation, across all shards.
    ///
    /// This is the capacity of one generation, split evenly across the shards, and retention is **per shard**: a
    /// token stays known until its shard has taken `max_tracked_packets / shards` further distinct tokens, twice
    /// over (once to retire it, once more to drop the retired generation). So a token survives roughly this many
    /// *total* inserts only if traffic spreads evenly across shards - which a keyed shard hash makes the expected
    /// case, but it is not a hard global bound. Must leave every shard at least [`MIN_CAPACITY_PER_SHARD`] slots,
    /// i.e. be at least `shards * MIN_CAPACITY_PER_SHARD` (and at least [`MIN_TRACKED_PACKETS`]).
    ///
    /// At the default false-positive rate the bitmap costs roughly three bytes per token per generation, so a
    /// decoder ultimately holds about this many megabytes, twice over once a shard has rotated. Generations are
    /// allocated on first insertion, so the footprint grows with traffic rather than being paid at construction;
    /// at the default it reaches a little under 32 MB per decoder, rising to about 63 MB after every shard has
    /// rotated once.
    ///
    /// Default is 10 000 000.
    #[default(default_max_tracked_packets())]
    // Concrete numbers in the message: `validator` takes a literal, so the constants cannot be interpolated here.
    // The test `the_tracked_packet_floor_agrees_with_the_validator` keeps them in step.
    #[validate(
        range(
            min = MIN_TRACKED_PACKETS,
            max = MAX_TRACKED_PACKETS,
            message = "max_tracked_packets must be between 4096 and 100000000"
        )
    )]
    #[cfg_attr(feature = "serde", serde(default = "default_max_tracked_packets"))]
    pub max_tracked_packets: usize,
    /// Number of independent shards.
    ///
    /// Each shard has its own generation state and lock, so this divides both the memory high-water mark and the
    /// lock contention. It must be a power of two, which the validator enforces so that shard selection is a mask
    /// rather than a division.
    ///
    /// Default is 64.
    #[default(default_shards())]
    #[validate(custom(function = "validate_shards"))]
    #[cfg_attr(feature = "serde", serde(default = "default_shards"))]
    pub shards: usize,
}

/// Lower bound on the number of tokens a single shard holds per generation, enforced by the validator.
///
/// The check and the insertion are two separate lock acquisitions with the whole decode between them, so a token
/// committed by one decode can be lost to a second one only if the shard rotates *twice* in that window - that is,
/// if two full generations are inserted into one shard while a single packet is being decoded. One slot per shard
/// would make that two packets; this floor keeps the window at the same order as the default configuration's.
pub const MIN_CAPACITY_PER_SHARD: usize = 64;
/// Lower bound on [`ReplayFilterConfig::max_tracked_packets`], enforced by the validator.
///
/// Sized so that the default shard count gets [`MIN_CAPACITY_PER_SHARD`] slots each.
pub const MIN_TRACKED_PACKETS: usize = MIN_CAPACITY_PER_SHARD * DEFAULT_SHARDS;
/// Lower bound on [`ReplayFilterConfig::shards`], enforced by the validator.
pub const MIN_SHARDS: usize = 1;
/// Upper bound on [`ReplayFilterConfig::shards`], enforced by the validator.
pub const MAX_SHARDS: usize = 4096;
/// Upper bound on [`ReplayFilterConfig::max_tracked_packets`], enforced by the validator.
///
/// The bound exists because generations are allocated lazily: without it, a typo in the capacity would surface as a
/// failed allocation inside the decode path on the first packet for a shard, rather than as a rejected
/// configuration at startup. One hundred million tokens is about 320 MB per generation at the default rate, which
/// is the point past which a node's own memory, not its traffic, is the constraint.
pub const MAX_TRACKED_PACKETS: usize = 100_000_000;
/// Default for [`ReplayFilterConfig::shards`].
const DEFAULT_SHARDS: usize = 64;
/// Default for [`ReplayFilterConfig::max_tracked_packets`].
const DEFAULT_MAX_TRACKED_PACKETS: usize = 10_000_000;
/// False-positive rate of a single generation.
///
/// [`TagReplayFilter::contains`] consults two generations, so the rate a caller observes is the union of the two.
/// Halving the per-generation rate keeps that union at the 1e-5 the single-filter design used; because the bitmap
/// size grows with `ln(1/p)`, halving `p` costs only a few percent per generation. Holding two generations is what
/// doubles the total: a decoder now allocates roughly twice the bitmap it did, against a retention floor that is no
/// longer zero.
const GENERATION_FALSE_POSITIVE_RATE: f64 = 0.000_005;

/// Keyed hash over `token`, used to pick a shard.
///
/// `SipHasher13` with an explicit key, not `std::hash::DefaultHasher`: the latter's keys are fixed, so a sender who
/// computes the tag could predict the shard exactly as well as with no hash at all.
fn keyed_hash(key: &[u8; 16], token: &ReplayToken) -> u64 {
    use std::hash::Hasher as _;
    let mut hasher = siphasher::sip::SipHasher13::new_with_key(key);
    hasher.write(token);
    hasher.finish()
}

/// `serde`/`SmartDefault` default for [`ReplayFilterConfig::max_tracked_packets`].
fn default_max_tracked_packets() -> usize {
    DEFAULT_MAX_TRACKED_PACKETS
}

/// `serde`/`SmartDefault` default for [`ReplayFilterConfig::shards`].
fn default_shards() -> usize {
    DEFAULT_SHARDS
}

/// Rejects a configuration whose capacity does not give every shard [`MIN_CAPACITY_PER_SHARD`] slots.
///
/// The field-wise ranges cannot express this: `max_tracked_packets` has a floor and `shards` has a ceiling, but
/// nothing stops the ceiling from exceeding the floor divided by the shard count. [`TagReplayFilter::new`] would
/// otherwise have to clamp, silently remembering fewer tokens than the configuration asked for.
///
/// The per-shard floor is more than "at least one": a shard with a single slot rotates on every distinct token, and
/// since the check and the commit are separate lock acquisitions, two rotations during one decode would let the
/// same packet through twice. See [`MIN_CAPACITY_PER_SHARD`].
fn validate_capacity_per_shard(cfg: &ReplayFilterConfig) -> Result<(), validator::ValidationError> {
    if cfg.shards == 0 {
        return Ok(());
    }

    let per_shard = cfg.max_tracked_packets / cfg.shards;
    if per_shard < MIN_CAPACITY_PER_SHARD {
        let required = cfg.shards * MIN_CAPACITY_PER_SHARD;
        let mut err = validator::ValidationError::new("max_tracked_packets");
        err.message = Some(
            format!(
                "max_tracked_packets must be at least {required} for {} shards ({} per shard), got {}",
                cfg.shards, MIN_CAPACITY_PER_SHARD, cfg.max_tracked_packets
            )
            .into(),
        );
        return Err(err);
    }
    Ok(())
}

/// Rejects a shard count that is out of range or not a power of two.
///
/// Power of two so [`TagReplayFilter::shard_for`] can mask rather than divide, and so every shard stays addressable.
fn validate_shards(shards: usize) -> Result<(), validator::ValidationError> {
    if !(MIN_SHARDS..=MAX_SHARDS).contains(&shards) || !shards.is_power_of_two() {
        let mut err = validator::ValidationError::new("shards");
        err.message = Some(format!("shards must be a power of two between {MIN_SHARDS} and {MAX_SHARDS}").into());
        return Err(err);
    }
    Ok(())
}

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    /// Counts shard rotations, i.e. how often a generation of replay history was dropped.
    ///
    /// Initialised lazily on first use; a sustained rate here means the replay window is shrinking faster than the
    /// configured capacity suggests it should.
    static ref METRIC_REPLAY_FILTER_ROTATIONS: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_replay_filter_rotations_total",
            "Replay filter generations dropped because their shard filled; each rotation shortens the window in which a packet is still recognised as a replay",
        )
        .unwrap();
}

/// One generation pair of a shard.
#[derive(Debug)]
struct ReplayShard {
    /// The generation currently accepting insertions.
    ///
    /// Allocated on the first insertion, not when the filter is built: at the default configuration a generation
    /// is tens of megabytes, and a node that has not yet received a packet should not have paid for every shard of
    /// it - nor should every `HoprDecoder` construction in a test or a bench.
    current: Option<bloomfilter::Bloom<ReplayToken>>,
    /// The generation that was current until the last rotation, if any.
    ///
    /// Allocated lazily too: a shard that never fills its first generation never pays for a second bitmap.
    previous: Option<bloomfilter::Bloom<ReplayToken>>,
    /// Number of tokens inserted into [`Self::current`].
    count: usize,
    /// Number of tokens a single generation holds before the shard rotates.
    capacity: usize,
}

impl ReplayShard {
    /// Creates an empty shard that will hold `capacity` tokens per generation.
    fn new(capacity: usize) -> Self {
        Self {
            current: None,
            previous: None,
            count: 0,
            capacity,
        }
    }

    /// Allocates one generation sized for `capacity` tokens.
    fn generation(capacity: usize) -> bloomfilter::Bloom<ReplayToken> {
        bloomfilter::Bloom::new_for_fp_rate_with_seed(capacity, GENERATION_FALSE_POSITIVE_RATE, &random_bytes())
            .expect("bloom filter with the specified capacity is constructible")
    }

    /// Whether either generation holds `token`.
    fn contains(&self, token: &ReplayToken) -> bool {
        self.current.as_ref().is_some_and(|current| current.check(token))
            || self.previous.as_ref().is_some_and(|previous| previous.check(token))
    }

    /// Inserts `token` unless it is already known. Returns `true` if it was already known.
    fn check_and_set(&mut self, token: &ReplayToken) -> bool {
        // Both generations are consulted *before* any rotation. Checking them after would mis-handle the exact
        // case where it matters most: a duplicate that is the token which filled the generation is in the
        // outgoing `current`, which the rotation is about to move into `previous` - so a check made before the
        // rotation would miss it and re-admit it into the fresh generation as a brand-new packet.
        if self.contains(token) {
            return true;
        }

        if self.count == self.capacity {
            // The current generation is full. The oldest one goes, the current one becomes the previous one, and
            // the token is recorded in a fresh generation.
            //
            // A rotation is normal in this design, so it is `debug`: at the default capacity a busy node rotates
            // every few tens of seconds, and warning on each would be noise. The counter is the signal an operator
            // alerts on; this line is for chasing a single rotation, and it is logged while the shard is locked.
            tracing::debug!(
                capacity = self.capacity,
                "replay filter shard rotated; its oldest generation was dropped"
            );
            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_REPLAY_FILTER_ROTATIONS.increment();
            self.previous = Some(std::mem::replace(
                self.current.get_or_insert_with(|| Self::generation(self.capacity)),
                Self::generation(self.capacity),
            ));
            self.count = 0;
        }

        self.current
            .get_or_insert_with(|| Self::generation(self.capacity))
            .set(token);
        self.count += 1;
        false
    }
}

/// Replay-detecting filter over packet replay tokens.
///
/// Each of the shards keeps two generations of a Bloom filter. When the current one fills it becomes the previous
/// one and a fresh current is allocated, so a token is remembered for at least one full generation of its shard -
/// the floor the previous single-filter design did not have, since it answered overflow by clearing every tag at
/// once. See [`ReplayFilterConfig::max_tracked_packets`] for what that means for the configured total.
///
/// Sharding by token divides both the lock contention of the decode path and the memory high-water mark. The
/// namespace is global rather than per previous hop: the token names the packet, not the sender, so a replay
/// injected through a different peer ID is still caught.
#[derive(Debug)]
pub struct TagReplayFilter {
    /// Shards, each independently locked. The count is a power of two, so selection is a mask.
    shards: Box<[parking_lot::Mutex<ReplayShard>]>,
    /// Key for shard selection, derived per filter instance.
    ///
    /// The tag is computed by the packet's creator, so indexing shards on its raw bytes would let a sender aim
    /// every packet at one shard - 1/64 of the traffic would evict that shard's history. Hashing with a
    /// per-instance key makes the shard unpredictable until a token has been seen, which restores the spread.
    shard_key: [u8; 16],
}

impl TagReplayFilter {
    /// Creates a filter holding `cfg.max_tracked_packets` tokens per generation, split evenly across `cfg.shards`.
    ///
    /// A validated configuration gives every shard at least [`MIN_CAPACITY_PER_SHARD`] slots, and the total is then
    /// `capacity / shards * shards`, i.e. within `shards - 1` of the configured value. Only an unvalidated
    /// configuration can land here with a smaller share, which the `.max(1)` below keeps from producing a
    /// zero-sized Bloom filter.
    pub fn new(cfg: &ReplayFilterConfig) -> Self {
        // Clamped first, because an unvalidated `shards` - `new` is public and `HoprDecoder::new` does not
        // validate - could otherwise overflow `next_power_of_two` (panic in debug, zero in release) or try to
        // allocate an unbounded number of shards.
        //
        // Then rounded up to a power of two: `shard_for`'s mask addresses only the first `2^k` shards, so an odd
        // count would silently waste the rest of the configured capacity.
        let shards = cfg.shards.clamp(MIN_SHARDS, MAX_SHARDS).next_power_of_two();
        // A validated configuration is already in range; an unvalidated one - `new` is public and the decoder does
        // not validate - is clamped here rather than left to fail allocating a generation inside the decode path.
        let capacity_per_shard = (cfg.max_tracked_packets.min(MAX_TRACKED_PACKETS) / shards).max(1);

        Self {
            shards: (0..shards)
                .map(|_| parking_lot::Mutex::new(ReplayShard::new(capacity_per_shard)))
                .collect(),
            shard_key: random_bytes(),
        }
        .with_metrics_registered()
    }

    /// Registers this type's telemetry at construction, so the series exists before the first rotation.
    ///
    /// Registering lazily on first use would leave `hopr_replay_filter_rotations_total` absent until a rotation
    /// happened - and `rate()`/`increase()` do not count the first increment of a series that did not exist before
    /// it - and would run the registration, and its `unwrap`, inside the decode path while a shard is locked.
    fn with_metrics_registered(self) -> Self {
        #[cfg(all(feature = "telemetry", not(test)))]
        lazy_static::initialize(&METRIC_REPLAY_FILTER_ROTATIONS);
        self
    }

    /// Number of shards, for tests and telemetry.
    pub fn shards(&self) -> usize {
        self.shards.len()
    }

    /// Total number of tokens the filter remembers per generation.
    pub fn capacity(&self) -> usize {
        self.shards.iter().map(|shard| shard.lock().capacity).sum()
    }

    /// Number of tokens inserted into the current generations, summed across shards, for tests and telemetry.
    pub fn count(&self) -> usize {
        self.shards.iter().map(|shard| shard.lock().count).sum()
    }

    /// Returns `true` if `token` is known to the filter. False positives are possible, false negatives are not
    /// until the token's generation rotates out.
    ///
    /// A hit here means "seen before" up to the filter's false-positive rate; a miss means "new" without exception.
    /// The value is only meaningful once a header MAC has verified it - a tag is computable by anyone from the
    /// node's public key, so an *unrecorded* tag proves nothing.
    pub fn contains(&self, token: &ReplayToken) -> bool {
        self.shard_for(token).lock().contains(token)
    }

    /// Records `token` and returns whether it was already known.
    ///
    /// A `true` return means the token was inserted by a concurrent decode of the same packet between the
    /// [`Self::contains`] check and this call, so the caller holds the duplicate.
    pub fn commit(&self, token: &ReplayToken) -> bool {
        self.shard_for(token).lock().check_and_set(token)
    }

    /// The shard `token` is assigned to, under the instance's keyed hash.
    fn shard_for(&self, token: &ReplayToken) -> &parking_lot::Mutex<ReplayShard> {
        // Keyed hash, so the sender - who computes the tag - cannot choose the shard. The mask is enough because
        // the shard count is a power of two.
        let index = keyed_hash(&self.shard_key, token) as usize & (self.shards.len() - 1);
        &self.shards[index]
    }
}

impl Default for TagReplayFilter {
    /// A filter at the default configuration.
    fn default() -> Self {
        Self::new(&ReplayFilterConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use anyhow::Context;

    use super::*;

    /// Builds a config from the two raw fields.
    fn config(max_tracked_packets: usize, shards: usize) -> ReplayFilterConfig {
        ReplayFilterConfig {
            max_tracked_packets,
            shards,
        }
    }

    /// Commits exactly `inserts` distinct fresh tokens into a single-shard filter, skipping Bloom false positives.
    ///
    /// A false positive makes `commit` report a fresh token as known, which would leave the generation short of
    /// full and silently skip the rotation a test depends on. Skipping such tokens keeps the arithmetic exact.
    /// Returns the token of the final successful insert.
    fn commit_exactly(filter: &TagReplayFilter, inserts: usize) -> ReplayToken {
        let mut candidate = 0u64;
        let mut inserted = 0;
        let mut last = token(0);

        while inserted < inserts {
            let t = token(candidate);
            candidate += 1;
            if filter.commit(&t) {
                continue; // false positive: already "known", so it did not take a slot
            }
            last = t;
            inserted += 1;
        }

        last
    }

    /// A deterministic synthetic token whose bytes vary with `value`.
    fn token(value: u64) -> ReplayToken {
        // Spread the counter across the token so that shard selection and the Bloom hashes both see variety.
        let mut token: ReplayToken = hopr_api::types::crypto::types::PacketTag::default();
        for (i, byte) in token.iter_mut().enumerate() {
            *byte = ((value >> (i % 8 * 8)) as u8).wrapping_add((i as u8).wrapping_mul(31));
        }
        token
    }

    #[test]
    fn a_committed_token_is_then_known() {
        let filter = TagReplayFilter::new(&config(1024, 4));
        let t = token(1);

        assert!(!filter.contains(&t));
        assert!(!filter.commit(&t), "first commit must see a new token");
        assert!(filter.contains(&t));
        assert!(filter.commit(&t), "second commit must see the token");
        assert_eq!(1, filter.count(), "a duplicate must not be counted twice");
    }

    /// The property the old clear-on-overflow design lacked: after a shard rotates, a token from the generation
    /// just retired is still known.
    #[test]
    fn a_token_survives_exactly_one_rotation() {
        // A single shard, so every token lands in it and the generations turn over deterministically.
        let filter = TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 1));
        let capacity = filter.capacity();

        let target = token(0);
        assert!(!filter.commit(&target), "the target is fresh");

        // Fill the generation, which rotates it once. Counting successful inserts keeps a Bloom false positive
        // from leaving the generation short and skipping the rotation.
        commit_exactly(&filter, capacity);
        assert!(filter.contains(&target), "the retired generation still remembers it");

        // Fill again, which rotates the retired generation out.
        commit_exactly(&filter, capacity);
        assert!(
            !filter.contains(&target),
            "after two rotations the token may age out; the floor is one generation"
        );
    }

    #[test]
    fn rotation_does_not_lose_the_newest_tokens() {
        let filter = TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 1));

        // Two generations' worth, so at least one rotation happens mid-run.
        let mut recent = Vec::new();
        for i in 0..(2 * filter.capacity() as u64) {
            let t = token(i);
            if filter.commit(&t) {
                continue; // a false positive; it did not take a slot, so it does not count as recent
            }
            recent.push(t);
            if recent.len() > 100 {
                recent.remove(0);
            }
        }
        assert!(!recent.is_empty(), "the run must have inserted tokens");

        for t in recent {
            assert!(
                filter.contains(&t),
                "the current generation must remember its own inserts"
            );
        }
    }

    #[test]
    fn commit_is_a_single_winner_under_concurrency() {
        let filter = std::sync::Arc::new(TagReplayFilter::new(&config(1024, 4)));
        let t = token(7);

        let winners = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        std::thread::scope(|scope| {
            for _ in 0..16 {
                let filter = filter.clone();
                let winners = winners.clone();
                scope.spawn(move || {
                    if !filter.commit(&t) {
                        winners.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                });
            }
        });

        assert_eq!(
            1,
            winners.load(std::sync::atomic::Ordering::SeqCst),
            "exactly one commit may see a fresh token"
        );
    }

    /// The token that fills a generation must be reported by the next `commit`, not re-admitted.
    ///
    /// It sits in the outgoing `current` at that moment, so a check made only against `previous` - or made after
    /// the rotation using a stale view - misses it, rotates it into the retired generation, and inserts it into the
    /// fresh one as a new packet. That is the boundary at which two concurrent decodes of one packet could both
    /// succeed with a single rotation.
    #[test]
    fn a_token_that_filled_the_generation_must_not_be_readmitted() {
        let filter = TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 1));
        let filling = commit_exactly(&filter, filter.capacity());
        assert_eq!(filter.capacity(), filter.count(), "the generation is exactly full");

        assert!(
            filter.commit(&filling),
            "the token that filled the generation must be reported, not re-admitted"
        );
        assert_eq!(
            filter.capacity(),
            filter.count(),
            "a duplicate must neither rotate the shard nor add to it"
        );
    }

    /// A token that lives only in the retired generation must still be reported by `commit`, not just by
    /// `contains`. The decoder calls `commit` directly, so this is the path a duplicate takes there.
    #[test]
    fn commit_should_report_a_token_from_the_retired_generation() {
        // One shard, so every token lands in it and the rotation is deterministic.
        let filter = TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 1));
        let ours = token(0);
        filter.commit(&ours);

        // Fill the generation so that `ours` is retired into `previous`.
        commit_exactly(&filter, filter.capacity());
        assert!(filter.contains(&ours), "the retired generation still knows it");

        assert!(
            filter.commit(&ours),
            "commit must report a token only the retired generation knows"
        );
        assert_eq!(
            filter.count(),
            1,
            "a commit that found the token in the retired generation must not also insert it"
        );
    }

    /// `count` reports the current generations only: it is reset by a rotation and never includes the retired one.
    #[test]
    fn count_should_track_the_current_generation_only() {
        let filter = TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 1));
        assert_eq!(0, filter.count(), "a new filter has inserted nothing");

        let capacity = filter.capacity();
        commit_exactly(&filter, capacity);
        assert_eq!(capacity, filter.count(), "a full generation is counted");

        // One more distinct token rotates the shard and starts a fresh generation.
        filter.commit(&token(u64::MAX));
        assert_eq!(1, filter.count(), "the count restarts with the new generation");
        assert_eq!(capacity, filter.capacity(), "capacity is unaffected by rotation");
    }

    #[test]
    fn shard_selection_spreads_tokens() {
        let filter = TagReplayFilter::new(&config(1024, 64));
        let mut used = HashSet::new();
        for i in 0..10_000u64 {
            used.insert(filter.shard_for(&token(i)) as *const _ as usize);
        }
        assert!(used.len() > 32, "tokens must reach most shards, saw {}", used.len());
    }

    /// A shard count that is not a power of two is rounded up rather than leaving part of the filter unaddressed,
    /// since `TagReplayFilter::new` is public and the decoder does not validate its configuration.
    #[test]
    fn an_odd_shard_count_should_still_use_the_whole_filter() {
        let filter = TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 3));
        assert_eq!(4, filter.shards(), "3 shards must round up to 4");

        let mut used = HashSet::new();
        for i in 0..10_000u64 {
            used.insert(filter.shard_for(&token(i)) as *const _ as usize);
        }
        assert_eq!(4, used.len(), "every shard must be addressable, saw {}", used.len());
    }

    /// The bounds and their message are spelled out as literals in the `range` validator, because `validator`
    /// cannot interpolate constants. This keeps the exported constants, the enforced bounds and the numbers an
    /// operator reads in the error message from drifting apart.
    #[test]
    fn the_tracked_packet_bounds_agree_with_the_validator() -> anyhow::Result<()> {
        let mut cfg = config(MIN_TRACKED_PACKETS, 64);
        validator::Validate::validate(&cfg).context("the floor itself must be accepted")?;

        cfg.max_tracked_packets = MIN_TRACKED_PACKETS - 1;
        let err = validator::Validate::validate(&cfg)
            .err()
            .context("one below the exported floor must be rejected")?;
        let message = format!("{}", err.field_errors()["max_tracked_packets"][0]);
        assert!(
            message.contains(&MIN_TRACKED_PACKETS.to_string()),
            "the message must state the real lower bound, got: {message}"
        );

        cfg.max_tracked_packets = MAX_TRACKED_PACKETS + 1;
        let err = validator::Validate::validate(&cfg)
            .err()
            .context("one above the exported ceiling must be rejected")?;
        let message = format!("{}", err.field_errors()["max_tracked_packets"][0]);
        assert!(
            message.contains(&MAX_TRACKED_PACKETS.to_string()),
            "the message must state the real upper bound, got: {message}"
        );
        Ok(())
    }

    /// The field-wise ranges accept the floor capacity together with the shard ceiling, yet that leaves far fewer
    /// slots per shard than [`MIN_CAPACITY_PER_SHARD`]. The config-level check is what rejects it - and it must not
    /// reject a configuration whose capacity merely does not divide evenly.
    #[test]
    fn a_configuration_with_too_little_capacity_per_shard_must_be_rejected() {
        assert!(
            validator::Validate::validate(&config(MIN_TRACKED_PACKETS, MAX_SHARDS)).is_err(),
            "the floor capacity split across the shard ceiling leaves too few slots per shard"
        );
        assert!(
            validator::Validate::validate(&config(MIN_CAPACITY_PER_SHARD * 63, 64)).is_err(),
            "one below the per-shard floor must be rejected"
        );
        assert!(
            validator::Validate::validate(&config(MIN_CAPACITY_PER_SHARD * 64, 64)).is_ok(),
            "exactly the per-shard floor must be accepted"
        );
        assert!(
            validator::Validate::validate(&config(MIN_CAPACITY_PER_SHARD * 64 + 1, 64)).is_ok(),
            "an uneven split must still be accepted"
        );
    }

    /// Every accepted configuration gives each shard the per-shard floor, so the filter's capacity is the configured
    /// value up to the shard count, and never larger than it.
    #[test]
    fn an_accepted_configuration_never_widens_the_capacity() -> anyhow::Result<()> {
        for (packets, shards) in [
            (MIN_TRACKED_PACKETS, 64),
            (MIN_TRACKED_PACKETS + 1, 64),
            (MIN_CAPACITY_PER_SHARD * MAX_SHARDS, MAX_SHARDS),
        ] {
            let cfg = config(packets, shards);
            validator::Validate::validate(&cfg).context("must be a valid configuration")?;
            let filter = TagReplayFilter::new(&cfg);
            assert_eq!(
                packets / shards * shards,
                filter.capacity(),
                "unexpected capacity for {cfg:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn shard_count_must_be_a_power_of_two() -> anyhow::Result<()> {
        // The capacity is generous in both cases, so only the shard rule can decide.
        let err = validator::Validate::validate(&config(65_536, 3))
            .err()
            .context("3 shards must be rejected")?;
        assert!(err.field_errors().contains_key("shards"));
        validator::Validate::validate(&config(65_536, 64)).context("64 shards must be accepted")?;
        Ok(())
    }

    #[test]
    fn capacity_is_shared_evenly_across_shards() {
        let filter = TagReplayFilter::new(&config(1024, 64));
        assert_eq!(1024, filter.capacity());
        assert_eq!(64, filter.shards());
    }

    /// A token this filter accepts must be reported as a replay the next time it is offered, however much traffic
    /// has passed through in between.
    ///
    /// This is the property a per-peer insertion allowance broke: past its allowance a peer's packets were accepted
    /// without being recorded, so the same packet was accepted again. Exercised through the whole filter rather than
    /// a single shard, over several rotations.
    #[test]
    fn a_token_accepted_now_must_be_a_replay_later() {
        let filter = TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 4));

        // Far more than a generation's worth, so several rotations happen before the token of interest.
        for i in 0..(3 * filter.capacity() as u64) {
            filter.commit(&token(i));
        }

        let recent = token(3 * filter.capacity() as u64 + 1);
        assert!(!filter.commit(&recent), "the token is fresh");
        assert!(
            filter.commit(&recent),
            "a token that was just accepted must be reported as a replay when repeated"
        );
    }

    /// An unvalidated `shards` must not overflow or allocate unboundedly: `new` is public and the decoder does not
    /// validate its configuration.
    #[test]
    fn an_unvalidated_shard_count_is_clamped() {
        assert_eq!(
            MAX_SHARDS,
            TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, usize::MAX)).shards(),
            "an oversized shard count must clamp to the ceiling, not overflow"
        );
        assert_eq!(
            MIN_SHARDS,
            TagReplayFilter::new(&config(MIN_TRACKED_PACKETS, 0)).shards(),
            "a zero shard count must clamp to the floor"
        );
    }

    #[test]
    fn a_new_filter_knows_nothing() {
        let filter = TagReplayFilter::default();
        assert_eq!(0, filter.count());
        assert!(!filter.contains(&token(0)));
    }
}
