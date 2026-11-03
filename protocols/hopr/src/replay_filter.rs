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
//! Keying on the derived tag rather than on the wire `alpha` is what makes the token *authenticated*: an attacker
//! can put any bytes at the start of a packet, but cannot produce a tag for a secret it has not derived. Inserting
//! tokens to age the filter out therefore costs the attacker the same per-packet work as sending a legitimate
//! packet.
//!
//! # Why a rolling filter rather than one Bloom filter that resets
//!
//! A Bloom filter is sized for a fixed number of items and cannot grow. The previous implementation answered
//! overflow by clearing the whole bitmap, which forgets *every* packet at once - and, because the clear happened on
//! every non-member insertion once full, a sustained flood left replay protection permanently off rather than
//! merely stale.
//!
//! Instead, each shard keeps two generations. When the current one fills it becomes the previous one and a fresh
//! current is allocated, so a token is remembered for at least one full generation - a floor that no insertion
//! pattern can take away, where the old design had none.
//!
//! Aging out is proportional to insert volume (a packet count, not wall time). The cost of shortening the window is
//! *decodable* packets, not paid ones: the decoder commits a token as soon as the Sphinx layer derives it, which
//! precedes ticket validation, and a final-hop packet carries no verifiable ticket at all. A direct neighbour can
//! therefore rotate generations with packets it does not pay for - but it must still present a valid tag for each,
//! so it pays the same elliptic-curve work per packet that a legitimate sender does, and the window it shortens is
//! bounded below by one full generation.
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
/// at least [`MIN_TRACKED_PACKETS`], and the capacity must be at least one token per shard. That last condition is
/// what the field-wise ranges cannot express on their own - `(1024, 4096)` satisfies both of them yet leaves a
/// shard with zero slots, whose Bloom filter could not be constructed.
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
    /// This is the retention floor: a token stays known for at least one full generation, so it stays known for at
    /// least `max_tracked_packets` insertions. Must be at least `shards`, so that every shard gets a slot.
    ///
    /// At the default false-positive rate the bitmap costs roughly three bytes per token per generation, so a
    /// decoder ultimately holds about this many megabytes, twice over once a shard has rotated. Generations are
    /// allocated on first insertion, so the footprint grows with traffic rather than being paid at construction;
    /// at the default it reaches a little under 32 MB per decoder, rising to about 63 MB after every shard has
    /// rotated once.
    ///
    /// Default is 10 000 000.
    #[default(default_max_tracked_packets())]
    #[validate(range(min = 1024, message = "max_tracked_packets must be at least 1024"))]
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

/// Lower bound on [`ReplayFilterConfig::max_tracked_packets`], enforced by the validator.
pub const MIN_TRACKED_PACKETS: usize = 1024;
/// Lower bound on [`ReplayFilterConfig::shards`], enforced by the validator.
pub const MIN_SHARDS: usize = 1;
/// Upper bound on [`ReplayFilterConfig::shards`], enforced by the validator.
pub const MAX_SHARDS: usize = 4096;
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

fn default_max_tracked_packets() -> usize {
    DEFAULT_MAX_TRACKED_PACKETS
}

fn default_shards() -> usize {
    DEFAULT_SHARDS
}

/// Rejects a configuration whose capacity does not give every shard at least one slot.
///
/// The field-wise ranges cannot express this: `max_tracked_packets` has a floor and `shards` has a ceiling, but
/// nothing stops the ceiling from exceeding the floor divided by the shard count. [`TagReplayFilter::new`] would
/// otherwise have to clamp, silently remembering fewer tokens than the configuration asked for.
fn validate_capacity_per_shard(cfg: &ReplayFilterConfig) -> Result<(), validator::ValidationError> {
    if cfg.shards > 0 && cfg.max_tracked_packets / cfg.shards == 0 {
        let mut err = validator::ValidationError::new("max_tracked_packets");
        err.message = Some(
            format!(
                "max_tracked_packets must be at least {MIN_TRACKED_PACKETS} and at least one per shard ({})",
                cfg.shards
            )
            .into(),
        );
        return Err(err);
    }
    Ok(())
}

fn validate_shards(shards: usize) -> Result<(), validator::ValidationError> {
    if !(MIN_SHARDS..=MAX_SHARDS).contains(&shards) || !shards.is_power_of_two() {
        let mut err = validator::ValidationError::new("shards");
        err.message = Some(format!("shards must be a power of two between {MIN_SHARDS} and {MAX_SHARDS}").into());
        return Err(err);
    }
    Ok(())
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
    fn new(capacity: usize) -> Self {
        Self {
            current: None,
            previous: None,
            count: 0,
            capacity,
        }
    }

    fn generation(capacity: usize) -> bloomfilter::Bloom<ReplayToken> {
        bloomfilter::Bloom::new_for_fp_rate_with_seed(capacity, GENERATION_FALSE_POSITIVE_RATE, &random_bytes())
            .expect("bloom filter with the specified capacity is constructible")
    }

    fn contains(&self, token: &ReplayToken) -> bool {
        self.current.as_ref().is_some_and(|current| current.check(token))
            || self.previous.as_ref().is_some_and(|previous| previous.check(token))
    }

    /// Inserts `token` unless it is already known. Returns `true` if it was already known.
    fn check_and_set(&mut self, token: &ReplayToken) -> bool {
        // The retired generation is read-only, so it is checked here; the current one is checked as part of the
        // insertion below, which needs a single hash pass where a separate `check` plus `set` would need two.
        if self.previous.as_ref().is_some_and(|previous| previous.check(token)) {
            return true;
        }

        if self.count == self.capacity {
            // The current generation is full. The oldest one goes, the current one becomes the previous one, and
            // the token is recorded in a fresh generation.
            self.previous = Some(std::mem::replace(
                self.current.get_or_insert_with(|| Self::generation(self.capacity)),
                Self::generation(self.capacity),
            ));
            self.count = 0;
        }

        let current = self.current.get_or_insert_with(|| Self::generation(self.capacity));

        // A member of the current generation must not rotate the shard, and `check_and_set` reports it without
        // recording a second time.
        if current.check_and_set(token) {
            return true;
        }

        self.count += 1;
        false
    }
}

/// Replay-detecting filter over packet replay tokens.
///
/// See the [module documentation](self) for the design rationale.
#[derive(Debug)]
pub struct TagReplayFilter {
    /// Shards, each independently locked. The count is a power of two, so selection is a mask.
    shards: Box<[parking_lot::Mutex<ReplayShard>]>,
}

impl TagReplayFilter {
    /// Creates a filter holding `cfg.max_tracked_packets` tokens per generation, split evenly across `cfg.shards`.
    ///
    /// A validated configuration gives every shard at least one slot, and the total is then `capacity / shards *
    /// shards`, i.e. within `shards - 1` of the configured value. Only an unvalidated configuration can land here
    /// with a smaller share, which the `.max(1)` below keeps from producing a zero-sized Bloom filter.
    pub fn new(cfg: &ReplayFilterConfig) -> Self {
        let shards = cfg.shards.max(MIN_SHARDS);
        let capacity_per_shard = (cfg.max_tracked_packets / shards).max(1);

        Self {
            shards: (0..shards)
                .map(|_| parking_lot::Mutex::new(ReplayShard::new(capacity_per_shard)))
                .collect(),
        }
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
    /// The tag it is given is authenticated by construction, so a hit cannot be provoked by a third party without
    /// that party deriving the same shared secret.
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

    fn shard_for(&self, token: &ReplayToken) -> &parking_lot::Mutex<ReplayShard> {
        // The token is already uniformly distributed (it is a slice of a curve point), so no mixing is needed;
        // take the leading bytes and mask. `shards.len()` is a power of two or 1.
        let index = usize::from_ne_bytes(token[..size_of::<usize>()].try_into().expect("token is long enough"))
            & (self.shards.len() - 1);
        &self.shards[index]
    }
}

impl Default for TagReplayFilter {
    fn default() -> Self {
        Self::new(&ReplayFilterConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn config(max_tracked_packets: usize, shards: usize) -> ReplayFilterConfig {
        ReplayFilterConfig {
            max_tracked_packets,
            shards,
        }
    }

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
        // Four shards of 16 tokens each; insert only shard-local tokens by using a single shard.
        let filter = TagReplayFilter::new(&config(1024, 1));
        assert_eq!(1024, filter.capacity());

        let target = token(0);
        filter.commit(&target);

        // Fill the generation, which rotates it once.
        for i in 1..=1024 {
            filter.commit(&token(i as u64));
        }

        assert!(filter.contains(&target), "the retired generation still remembers it");

        // Fill again, which rotates the retired generation out.
        for i in 1025..=2048 {
            filter.commit(&token(i as u64));
        }

        assert!(
            !filter.contains(&target),
            "after two rotations the token may age out; the floor is one generation"
        );
    }

    #[test]
    fn rotation_does_not_lose_the_newest_tokens() {
        let filter = TagReplayFilter::new(&config(1024, 1));

        let mut recent = Vec::new();
        for i in 0..2048u64 {
            let t = token(i);
            filter.commit(&t);
            recent.push(t);
            if recent.len() > 100 {
                recent.remove(0);
            }
        }

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

    #[test]
    fn shard_selection_spreads_tokens() {
        let filter = TagReplayFilter::new(&config(1024, 64));
        let mut used = HashSet::new();
        for i in 0..10_000u64 {
            used.insert(filter.shard_for(&token(i)) as *const _ as usize);
        }
        assert!(used.len() > 32, "tokens must reach most shards, saw {}", used.len());
    }

    /// The floor is spelled as a literal in the `range` validator, matching the other configs in this crate; this
    /// keeps the exported constant and the enforced bound from drifting apart.
    #[test]
    fn the_tracked_packet_floor_agrees_with_the_validator() {
        let mut cfg = config(MIN_TRACKED_PACKETS, 64);
        validator::Validate::validate(&cfg).expect("the floor itself must be accepted");
        cfg.max_tracked_packets = MIN_TRACKED_PACKETS - 1;
        assert!(
            validator::Validate::validate(&cfg).is_err(),
            "one below the exported floor must be rejected"
        );
    }

    /// The field-wise ranges accept `(1024, 4096)` - the floor and the shard ceiling - yet that leaves every shard
    /// with zero slots. The config-level check is what rejects it, and it must not reject a configuration whose
    /// capacity merely does not divide evenly.
    #[test]
    fn a_capacity_smaller_than_the_shard_count_must_be_rejected() {
        assert!(
            validator::Validate::validate(&config(MIN_TRACKED_PACKETS, MAX_SHARDS)).is_err(),
            "a capacity below one per shard must be rejected"
        );
        assert!(
            validator::Validate::validate(&config(MIN_TRACKED_PACKETS, MIN_TRACKED_PACKETS / 2)).is_ok(),
            "one slot per shard is enough"
        );
        validator::Validate::validate(&config(MIN_TRACKED_PACKETS + 1, 64))
            .expect("an uneven split must still be accepted");
    }

    /// Every accepted configuration gives each shard a slot, so the filter's capacity is the configured value up to
    /// the shard count, and never larger than it.
    #[test]
    fn an_accepted_configuration_never_widens_the_capacity() {
        for (packets, shards) in [(MIN_TRACKED_PACKETS, 64), (MIN_TRACKED_PACKETS + 1, 64), (4096, 4096)] {
            let cfg = config(packets, shards);
            validator::Validate::validate(&cfg).expect("must be a valid configuration");
            let filter = TagReplayFilter::new(&cfg);
            assert_eq!(
                packets / shards * shards,
                filter.capacity(),
                "unexpected capacity for {cfg:?}"
            );
        }
    }

    #[test]
    fn shard_count_must_be_a_power_of_two() {
        let err = validator::Validate::validate(&config(1024, 3)).expect_err("3 shards must be rejected");
        assert!(err.field_errors().contains_key("shards"));
        validator::Validate::validate(&config(1024, 64)).expect("64 shards must be accepted");
    }

    #[test]
    fn capacity_is_shared_evenly_across_shards() {
        let filter = TagReplayFilter::new(&config(1024, 64));
        assert_eq!(1024, filter.capacity());
        assert_eq!(64, filter.shards());
    }

    #[test]
    fn a_new_filter_knows_nothing() {
        let filter = TagReplayFilter::default();
        assert_eq!(0, filter.count());
        assert!(!filter.contains(&token(0)));
    }
}
