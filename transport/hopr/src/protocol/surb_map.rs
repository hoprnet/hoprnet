//! A bounded concurrent map of minted SURBs that evicts a random record when it is full.
//!
//! # Why random eviction
//!
//! A SURB is remembered from the moment it is minted until its reply comes home, and a FIFO replier
//! spends its oldest SURB first, so the record whose reply is due next is always the *oldest* one.
//! Any policy that evicts by age or recency (LRU, FIFO) therefore evicts exactly the records about to
//! be needed: once the outstanding SURBs exceed the capacity, every record whose reply waits longer
//! than `capacity / mint rate` is gone before its reply arrives, so a Session with a long wait
//! matches none of its replies at all, and has no record left when a dead-relayer verdict needs to
//! find its SURBs.
//!
//! Evicting a uniformly random record degrades gracefully instead. Each mint into a full map evicts
//! a record chosen without regard to its age, so a record survives its wait with probability about
//! `(1 - 1/capacity)` raised to the number of evicting mints during that wait: the longer the wait,
//! the more it loses, smoothly (exponentially), and no Session drops to exactly zero, as one does
//! under eviction by age. When every Session waits alike, with `ρ` the outstanding SURBs divided by
//! the capacity, the share `h` of replies that still find their record solves `h = e^(-(1 - h) ρ)`:
//! about 91 % at `ρ = 1.05` and about 20 % at `ρ = 2`. Sessions whose waits differ lose unequally: in
//! the simulation of `under_overload_no_session_should_lose_all_its_replies`, a Session waiting three
//! times as long as its busy neighbour matches 83 % of its replies against the neighbour's 94 % at
//! `ρ = 1.05`, and 3 % against 32 % at `ρ = 2`. The newest record is always admitted: a full map
//! never refuses it.
//!
//! # Layout
//!
//! The map is split into shards, each a `parking_lot::Mutex` over an `IndexMap` (a dense vector
//! of records, so a random one is picked and removed in constant time) and the state of a small
//! xorshift RNG. Nothing is ever called while a shard lock is held except the caller's closure of
//! `for_each` and `update`, which must
//! not call back into the same map. An evicted record is handed back to the caller of
//! `insert`, who acts on it after the lock is released.

use std::{
    hash::{BuildHasherDefault, Hasher},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use hopr_api::types::internal::prelude::HoprSurbId;
use indexmap::{IndexMap, map::Entry};
use parking_lot::Mutex;

/// Most records one shard is meant to hold before the map is split further: a record walk or an
/// expiry sweep holds one shard lock at a time, so this bounds how long a lock is held.
const RECORDS_PER_SHARD: usize = 4096;

/// Most shards a map is split into.
const MAX_SHARDS: usize = 64;

/// The fraction of the shards one [`expire_step`](RandomEvictionMap::expire_step) sweeps, as a divisor:
/// every shard is swept once per this many steps.
const EXPIRY_SWEEP_STEPS: usize = 16;

/// A shard's record storage grows by at least a `1 / GROWTH_DIVISOR` of its length at a time, so
/// growing costs a bounded share of what is already held and never more than that is spare.
const GROWTH_DIVISOR: usize = 16;

/// The fewest records a shard's storage grows by at a time.
const MIN_GROWTH_STEP: usize = 16;

/// Bytes the index of one record costs, as a rule of thumb rather than a hard bound: the hash table
/// of the [`IndexMap`] keeps one `usize` and one control byte per bucket, its bucket count is a power
/// of two and it is never more than 7/8 full, which is up to about 2.3 buckets per record just after
/// it doubled. Under churn, hashbrown resizes instead of rehashing in place when a table full of
/// deleted-record tombstones holds more than half its capacity, which can double the index once more.
/// That was not seen in the measured churn (a million records, evictions and removals), where the
/// resident memory stayed within the accounted figure thanks to the other conservative terms of
/// [`record_bytes`] (the growth spare, and the figure being the worst case at a full shard).
const INDEX_BYTES_PER_RECORD: usize = (23 * (size_of::<usize>() + 1)).div_ceil(10);

/// The multiplier that picks a record's shard from its id (a golden-ratio constant), so that ids that
/// are not uniformly random (tests) still spread over the shards.
const SHARD_MULTIPLIER: u64 = 0x9E37_79B9_7F4A_7C15;

/// The multiplier of the id hash of a shard's table and of the RNG's output (Knuth's 64-bit LCG
/// constant and the xorshift64* multiplier).
const HASH_MULTIPLIER: u64 = 0x517C_C1B7_2722_0A95;
const RNG_MULTIPLIER: u64 = 0x2545_F491_4F6C_DD1D;

/// A stored record: the value and when it was last written (inserted or updated), for expiry.
#[derive(Debug, Clone, Copy)]
struct Slot<V> {
    value: V,
    written: Instant,
}

/// The layout of one entry of the [`IndexMap`] (hash, key, value), whose own type is private, so that
/// its size can be read at compile time.
#[allow(dead_code)]
struct EntryLayout<V> {
    hash: usize,
    key: HoprSurbId,
    slot: Slot<V>,
}

/// The memory one record of value type `V` costs, in bytes, in the worst case.
///
/// The entry itself, the spare of the storage that grows in steps (`1 / GROWTH_DIVISOR` of the
/// entries), and the index that finds it. A map holds `budget / record_bytes::<V>()` records, so each
/// map honours its byte budget from its own type instead of from a figure measured on another.
/// Allocator overhead and fragmentation are not included.
pub const fn record_bytes<V>() -> usize {
    let entry = size_of::<EntryLayout<V>>();
    entry + entry.div_ceil(GROWTH_DIVISOR) + INDEX_BYTES_PER_RECORD
}

/// A hasher for the 8 random bytes of a [`HoprSurbId`]: already uniform, so folding them in and
/// spreading the high bits down is all the table needs. Much cheaper than SipHash, with no use for
/// its resistance to chosen keys: the ids are minted here, not chosen by a peer.
#[derive(Default)]
struct IdHasher(u64);

impl IdHasher {
    fn fold(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(HASH_MULTIPLIER);
    }
}

impl Hasher for IdHasher {
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(size_of::<u64>()) {
            let mut word = [0u8; size_of::<u64>()];
            word[..chunk.len()].copy_from_slice(chunk);
            self.fold(u64::from_le_bytes(word));
        }
    }

    fn write_usize(&mut self, length: usize) {
        self.fold(length as u64);
    }

    fn finish(&self) -> u64 {
        // The table takes its bucket from the low bits and its tag from the high ones.
        self.0 ^ (self.0 >> 32)
    }
}

/// A shard: its records and the state of its eviction RNG.
struct Shard<V> {
    entries: IndexMap<HoprSurbId, Slot<V>, BuildHasherDefault<IdHasher>>,
    rng: u64,
}

impl<V> Shard<V> {
    /// xorshift64*: a few instructions under a lock already held, and no allocation or shared state.
    fn next_random(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        x.wrapping_mul(RNG_MULTIPLIER)
    }

    /// A uniform index below `len`, from the high bits of the RNG.
    fn random_below(&mut self, len: usize) -> usize {
        ((u128::from(self.next_random()) * len as u128) >> u64::BITS) as usize
    }

    /// Makes room for one more record without ever growing past `capacity`.
    ///
    /// The storage is grown in steps, not to `capacity` up front, because a map sized for the 4 GiB
    /// ceiling would otherwise ask for gigabytes at startup for a node that never fills it; and with
    /// `reserve_exact`, so that it never holds more than `capacity` either.
    fn make_room(&mut self, capacity: usize) {
        let len = self.entries.len();
        if len >= self.entries.capacity() {
            let step = (len / GROWTH_DIVISOR).max(MIN_GROWTH_STEP).min(capacity - len);
            self.entries.reserve_exact(step);
        }
    }
}

/// splitmix64, to turn one seed into a distinct non-zero RNG state per shard.
fn splitmix64(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A concurrent map of at most a fixed number of records keyed by [`HoprSurbId`], evicting a random
/// record to admit a new one when full. See the [module docs](self) for why.
pub struct RandomEvictionMap<V> {
    shards: Box<[Mutex<Shard<V>>]>,
    /// log2 of the number of shards.
    shard_bits: u32,
    /// Records one shard holds; the map holds at most `shards.len()` times this.
    shard_capacity: usize,
    /// Where the next [`expire_step`](Self::expire_step) starts.
    next_sweep: AtomicUsize,
}

impl<V> std::fmt::Debug for RandomEvictionMap<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RandomEvictionMap")
            .field("shards", &self.shards.len())
            .field("shard_capacity", &self.shard_capacity)
            .finish_non_exhaustive()
    }
}

impl<V> RandomEvictionMap<V> {
    /// A map holding as many records as fit in `budget_bytes`, at [`record_bytes`] each, seeded from
    /// the system RNG. Always holds at least one.
    pub fn new(budget_bytes: u64) -> Self {
        let records = usize::try_from(budget_bytes / record_bytes::<V>() as u64).unwrap_or(usize::MAX);
        Self::with_seed(records, rand::random::<u64>())
    }

    /// A map holding at most `records` records (at least one), with the eviction RNG seeded by `seed`:
    /// the same inserts evict the same records, for deterministic tests.
    ///
    /// The map is split into a power of two of shards, about one per [`RECORDS_PER_SHARD`] records, so
    /// that a map of a few thousand records is a single shard and evicts uniformly over all of it.
    pub(crate) fn with_seed(records: usize, seed: u64) -> Self {
        let records = records.max(1);
        let shard_count = (1usize << (records / RECORDS_PER_SHARD).max(1).ilog2()).min(MAX_SHARDS);
        let shard_capacity = (records / shard_count).max(1);
        Self {
            shards: (0..shard_count)
                .map(|shard| {
                    Mutex::new(Shard {
                        entries: IndexMap::default(),
                        rng: splitmix64(seed.wrapping_add(shard as u64)) | 1,
                    })
                })
                .collect(),
            shard_bits: shard_count.ilog2(),
            shard_capacity,
            next_sweep: AtomicUsize::new(0),
        }
    }

    /// The most records the map holds.
    #[cfg_attr(not(any(test, feature = "testing")), allow(dead_code))]
    pub fn capacity(&self) -> usize {
        self.shards.len() * self.shard_capacity
    }

    fn shard_of(&self, id: &HoprSurbId) -> &Mutex<Shard<V>> {
        let index = if self.shard_bits == 0 {
            0
        } else {
            (u64::from_le_bytes(*id).wrapping_mul(SHARD_MULTIPLIER) >> (u64::BITS - self.shard_bits)) as usize
        };
        &self.shards[index]
    }

    /// Stores `value` under `id` and returns the record it evicted to make room, if the map was full.
    ///
    /// The new record is always admitted. Replacing the record under `id` evicts nothing. The evicted
    /// record is returned, not handled, so the caller acts on it after the lock is released.
    pub fn insert(&self, id: HoprSurbId, value: V) -> Option<(HoprSurbId, V)> {
        let slot = Slot {
            value,
            written: Instant::now(),
        };
        let mut shard = self.shard_of(&id).lock();
        let full = shard.entries.len() >= self.shard_capacity;
        match shard.entries.entry(id) {
            Entry::Occupied(mut occupied) => {
                occupied.insert(slot);
                return None;
            }
            Entry::Vacant(_) if full => {}
            Entry::Vacant(_) => {
                shard.make_room(self.shard_capacity);
                shard.entries.insert(id, slot);
                return None;
            }
        }
        let len = shard.entries.len();
        let victim = shard.random_below(len);
        let evicted = shard
            .entries
            .swap_remove_index(victim)
            .map(|(evicted_id, evicted)| (evicted_id, evicted.value));
        shard.entries.insert(id, slot);
        evicted
    }

    /// Takes the record under `id` out of the map.
    pub fn remove(&self, id: &HoprSurbId) -> Option<V> {
        self.shard_of(id)
            .lock()
            .entries
            .swap_remove(id)
            .map(|removed| removed.value)
    }

    /// Changes the record under `id` in place and returns what `change` returns, or `None` if there is
    /// no such record.
    ///
    /// The change happens under the shard's lock and never inserts, so a record that a concurrent
    /// [`remove`](Self::remove) took is never brought back by it.
    ///
    /// An update counts as a write: the record's expiry runs from it, as it did when the update was a
    /// re-insert. The registry opens every reply race through such an update, so after it the record
    /// lives a further full horizon, far past the race's own lifetime, and expiry can never drop a
    /// record while a race that only its reply can close is still open.
    pub fn update<R>(&self, id: &HoprSurbId, change: impl FnOnce(&mut V) -> R) -> Option<R> {
        self.shard_of(id).lock().entries.get_mut(id).map(|slot| {
            slot.written = Instant::now();
            change(&mut slot.value)
        })
    }

    /// Whether there is a record under `id`.
    pub fn contains_key(&self, id: &HoprSurbId) -> bool {
        self.shard_of(id).lock().entries.contains_key(id)
    }

    /// The number of records, summed over the shards one after the other, so approximate while the map
    /// is being changed.
    #[cfg_attr(not(any(test, feature = "testing")), allow(dead_code))]
    pub fn len(&self) -> usize {
        self.shards.iter().map(|shard| shard.lock().entries.len()).sum()
    }

    /// Whether the map holds no record, to the same approximation as [`len`](Self::len).
    #[cfg_attr(not(any(test, feature = "testing")), allow(dead_code))]
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|shard| shard.lock().entries.is_empty())
    }

    /// Calls `visit` for every record, shard by shard, holding one shard's lock at a time.
    ///
    /// A record moved or removed while the walk is on another shard may be visited twice or not at all,
    /// and `visit` must not call back into this map, which it would deadlock on: collect what it needs
    /// and act after the walk.
    pub fn for_each(&self, mut visit: impl FnMut(&HoprSurbId, &V)) {
        for shard in &self.shards {
            for (id, slot) in &shard.lock().entries {
                visit(id, &slot.value);
            }
        }
    }

    /// Drops the records last written longer than `horizon` ago from the next `1 / EXPIRY_SWEEP_STEPS` of
    /// the shards (at least one), so that every shard is swept once per that many calls, and returns
    /// how many it dropped.
    ///
    /// Called once per flush tick, which at a horizon of minutes sweeps every shard far more often than
    /// it must, and holds one shard lock at a time.
    pub fn expire_step(&self, horizon: Duration) -> usize {
        let per_step = self.shards.len().div_ceil(EXPIRY_SWEEP_STEPS);
        let start = self.next_sweep.fetch_add(per_step, Ordering::Relaxed);
        let mut expired = 0;
        for offset in 0..per_step {
            let mut shard = self.shards[(start + offset) % self.shards.len()].lock();
            let before = shard.entries.len();
            shard.entries.retain(|_, slot| slot.written.elapsed() <= horizon);
            expired += before - shard.entries.len();
        }
        expired
    }
}

#[cfg(test)]
impl<V: Clone> RandomEvictionMap<V> {
    /// A copy of the record under `id`.
    pub(crate) fn get(&self, id: &HoprSurbId) -> Option<V> {
        self.shard_of(id).lock().entries.get(id).map(|slot| slot.value.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};

    use anyhow::Context;

    use super::*;

    const SEED: u64 = 0x5EED;

    fn id(n: u32) -> HoprSurbId {
        let mut id = [0u8; 8];
        id[..4].copy_from_slice(&n.to_le_bytes());
        id
    }

    #[test]
    fn a_map_should_never_hold_more_than_its_capacity() {
        let map = RandomEvictionMap::<u32>::with_seed(100, SEED);
        for n in 0..1_000 {
            map.insert(id(n), n);
            assert!(map.len() <= 100, "{} records after {} inserts", map.len(), n + 1);
        }
        assert_eq!(100, map.len(), "and it fills up to it");
    }

    #[test]
    fn a_sharded_map_should_never_hold_more_than_its_capacity() {
        let map = RandomEvictionMap::<u32>::with_seed(5 * RECORDS_PER_SHARD, SEED);
        assert!(map.shards.len() > 1, "large enough to be split");
        assert!(map.capacity() <= 5 * RECORDS_PER_SHARD);
        for n in 0..4 * map.capacity() as u32 {
            map.insert(id(n), n);
        }
        assert!(map.len() <= map.capacity());
    }

    #[test]
    fn the_newest_record_should_always_be_present_after_it_is_inserted() {
        let map = RandomEvictionMap::<u32>::with_seed(8, SEED);
        for n in 0..200 {
            map.insert(id(n), n);
            assert_eq!(Some(n), map.get(&id(n)), "record {n} must not be refused");
        }
    }

    #[test]
    fn inserting_into_a_full_map_should_return_the_evicted_record() -> anyhow::Result<()> {
        let map = RandomEvictionMap::<u32>::with_seed(4, SEED);
        for n in 0..4 {
            assert!(map.insert(id(n), n).is_none(), "nothing is evicted while there is room");
        }
        let (evicted_id, evicted) = map.insert(id(4), 4).context("a full map evicts")?;
        assert_eq!(id(evicted), evicted_id);
        assert!(!map.contains_key(&evicted_id), "and it is gone");
        assert_eq!(4, map.len());
        Ok(())
    }

    #[test]
    fn replacing_a_record_should_evict_nothing() {
        let map = RandomEvictionMap::<u32>::with_seed(4, SEED);
        for n in 0..4 {
            map.insert(id(n), n);
        }
        assert!(map.insert(id(2), 99).is_none());
        assert_eq!(Some(99), map.get(&id(2)));
        assert_eq!(4, map.len());
    }

    #[test]
    fn updating_a_removed_record_should_find_nothing_and_not_bring_it_back() {
        let map = RandomEvictionMap::<u32>::with_seed(4, SEED);
        map.insert(id(1), 1);
        assert_eq!(Some(1), map.remove(&id(1)));
        assert!(map.update(&id(1), |value| *value += 1).is_none());
        assert!(!map.contains_key(&id(1)), "an update never inserts");
        assert!(map.is_empty());

        map.insert(id(2), 5);
        assert_eq!(
            Some(6),
            map.update(&id(2), |value| {
                *value += 1;
                *value
            })
        );
        assert_eq!(Some(6), map.get(&id(2)));
    }

    /// The race this map exists to close: a record consumed by one thread while another updates it
    /// must stay consumed, which a read-compute-insert update cannot promise.
    #[test]
    fn a_record_removed_while_it_is_updated_should_never_come_back() -> anyhow::Result<()> {
        const ROUNDS: usize = 20_000;
        let map = Arc::new(RandomEvictionMap::<u32>::with_seed(16, SEED));
        let barrier = Arc::new(Barrier::new(3));
        let key = id(7);

        let remover = {
            let (map, barrier) = (map.clone(), barrier.clone());
            std::thread::spawn(move || {
                for _ in 0..ROUNDS {
                    barrier.wait();
                    map.remove(&key);
                    barrier.wait();
                }
            })
        };
        let updater = {
            let (map, barrier) = (map.clone(), barrier.clone());
            std::thread::spawn(move || {
                for _ in 0..ROUNDS {
                    barrier.wait();
                    map.update(&key, |value| *value += 1);
                    barrier.wait();
                }
            })
        };
        for round in 0..ROUNDS {
            map.insert(key, 0);
            barrier.wait();
            barrier.wait();
            assert!(!map.contains_key(&key), "round {round}: the removed record came back");
        }
        remover.join().map_err(|_| anyhow::anyhow!("remover thread panicked"))?;
        updater.join().map_err(|_| anyhow::anyhow!("updater thread panicked"))?;
        Ok(())
    }

    #[test]
    fn expiring_should_drop_only_the_records_past_the_horizon() {
        const HORIZON: Duration = Duration::from_millis(20);
        let map = RandomEvictionMap::<u32>::with_seed(16, SEED);
        map.insert(id(1), 1);
        map.insert(id(2), 2);
        std::thread::sleep(2 * HORIZON);
        map.insert(id(3), 3);

        assert_eq!(2, map.expire_step(HORIZON));
        assert!(!map.contains_key(&id(1)) && !map.contains_key(&id(2)));
        assert!(map.contains_key(&id(3)), "a younger record stays");
        assert_eq!(0, map.expire_step(HORIZON));
    }

    /// An update restarts the record's expiry. The registry opens a reply race by updating the partner's
    /// record, which may be nearly a horizon old: expiring it by its mint time would drop it while the
    /// race it just opened is live, and its reply could then settle nothing. The registry's horizon is
    /// the const `PENDING_SURB_HORIZON` of 15 minutes, too long for a test of its own, so this is the
    /// test of that guarantee, with a short horizon.
    #[test]
    fn an_update_should_restart_the_expiry_of_the_record() {
        // Long enough that each check has 400 ms of slack on either side, so a loaded machine that
        // stalls the test between a sleep and a check does not fail it.
        const HORIZON: Duration = Duration::from_secs(1);
        let map = RandomEvictionMap::<u32>::with_seed(16, SEED);
        map.insert(id(1), 1);
        std::thread::sleep(HORIZON * 6 / 10);
        assert_eq!(Some(()), map.update(&id(1), |value| *value += 1));
        // The insert is now past the horizon, the update is not.
        std::thread::sleep(HORIZON * 6 / 10);
        assert_eq!(0, map.expire_step(HORIZON), "expiry counts from the update");
        assert_eq!(Some(2), map.get(&id(1)));
        std::thread::sleep(HORIZON * 6 / 10);
        assert_eq!(1, map.expire_step(HORIZON), "and it expires a horizon after the update");
    }

    #[test]
    fn expiring_should_reach_every_shard_within_a_cycle_of_steps() {
        const HORIZON: Duration = Duration::from_millis(10);
        let map = RandomEvictionMap::<u32>::with_seed(MAX_SHARDS * RECORDS_PER_SHARD, SEED);
        assert_eq!(MAX_SHARDS, map.shards.len());
        for n in 0..2_000 {
            map.insert(id(n), n);
        }
        std::thread::sleep(2 * HORIZON);
        let first = map.expire_step(HORIZON);
        assert!(first < 2_000, "one step sweeps only part of the shards");
        let rest: usize = (1..EXPIRY_SWEEP_STEPS).map(|_| map.expire_step(HORIZON)).sum();
        assert_eq!(2_000, first + rest);
        assert!(map.is_empty());
    }

    /// Past capacity a record survives a mint with probability `1 - 1/capacity` whatever its age:
    /// after as many mints again, a record of the first batch is still there with probability
    /// `(1 - 1/n)^n ≈ 1/e` — not 0, as for an eviction by age.
    #[test]
    fn eviction_should_not_favour_the_young_or_the_old() {
        const N: u32 = 1_000;
        let map = RandomEvictionMap::<u32>::with_seed(N as usize, SEED);
        for n in 0..N {
            map.insert(id(n), n);
        }
        for n in N..2 * N {
            map.insert(id(n), n);
        }
        let old = (0..N).filter(|n| map.contains_key(&id(*n))).count();
        let young = (N..2 * N).filter(|n| map.contains_key(&id(*n))).count();
        assert_eq!(N as usize, old + young);
        // 1/e of 1000 is 368.
        assert!((300..440).contains(&old), "{old} of the old batch survived");
    }

    #[test]
    fn a_walk_should_visit_every_record_once() {
        let map = RandomEvictionMap::<u32>::with_seed(2 * RECORDS_PER_SHARD, SEED);
        for n in 0..1_000 {
            map.insert(id(n), n);
        }
        let mut seen = std::collections::HashSet::new();
        map.for_each(|id, _| assert!(seen.insert(*id)));
        assert_eq!(1_000, seen.len());
    }

    #[test]
    fn storage_should_grow_in_steps_and_never_past_the_capacity() {
        const CAPACITY: usize = 1_000;
        let map = RandomEvictionMap::<u32>::with_seed(CAPACITY, SEED);
        assert_eq!(
            0,
            map.shards[0].lock().entries.capacity(),
            "nothing is reserved up front"
        );
        map.insert(id(0), 0);
        assert!(map.shards[0].lock().entries.capacity() < CAPACITY / 4);
        for n in 1..3 * CAPACITY as u32 {
            map.insert(id(n), n);
            assert!(map.shards[0].lock().entries.capacity() <= CAPACITY);
        }
        assert_eq!(CAPACITY, map.shards[0].lock().entries.capacity());
    }

    #[test]
    fn the_shard_count_should_follow_the_capacity() {
        let shards = |records| RandomEvictionMap::<u32>::with_seed(records, SEED).shards.len();
        assert_eq!(1, shards(0));
        assert_eq!(1, shards(1));
        assert_eq!(1, shards(2 * RECORDS_PER_SHARD - 1));
        assert_eq!(2, shards(2 * RECORDS_PER_SHARD));
        assert_eq!(MAX_SHARDS, shards(usize::MAX / 2));
    }

    #[test]
    fn a_budget_should_buy_records_at_the_cost_of_its_own_type() {
        let budget = 1_000 * record_bytes::<u64>() as u64;
        assert_eq!(1_000, RandomEvictionMap::<u64>::new(budget).capacity());
        assert!(record_bytes::<[u8; 512]>() > record_bytes::<u64>());
        assert_eq!(1, RandomEvictionMap::<u64>::new(0).capacity(), "never fewer than one");
    }
}
