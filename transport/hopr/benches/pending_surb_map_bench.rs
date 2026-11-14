//! The map that remembers minted SURBs until their reply comes home: `RandomEvictionMap` against the
//! `moka` cache it replaces, configured as it was (a byte capacity, a constant weigher, LRU eviction
//! and an eviction listener).
//!
//! Run with `cargo bench -p hopr-transport --features testing --bench pending_surb_map_bench`.
//! Records are the size of a pending SURB, and both maps hold the same number of them.
//!
//! Which map is registered first alternates between the groups, since running one right after the other
//! can favour either (cache and allocator state). Every function can also be run alone with a criterion
//! filter, e.g. `-- insert_at_capacity/moka_lru`, which removes the order effect entirely.
//!
//! moka applies size eviction and its listener in deferred maintenance, so `insert_at_capacity` runs
//! `run_pending_tasks` inside the timed region every `MAINTENANCE_EVERY` inserts: the work it defers is
//! part of what an insert costs, as `RandomEvictionMap`'s synchronous eviction is. The concurrent runs
//! time only the map operations: the threads are spawned before the clock starts and released together.

use std::{
    hint::black_box,
    sync::{
        Arc, Barrier,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use criterion::{Criterion, Throughput, criterion_group, criterion_main, measurement::WallTime};
use hopr_transport::protocol::surb_map::{RandomEvictionMap, record_bytes};

/// A stand-in for a pending SURB's record, of its size.
type Record = [u8; 504];

type SurbId = [u8; 8];

/// Records each map holds.
const CAPACITY: u64 = 100_000;

/// What the replaced cache charged for one record.
const MOKA_RECORD_BYTES: u64 = 850;

/// Threads in the concurrent runs, and the records each handles per iteration.
const THREADS: u64 = 8;
const PER_THREAD: u64 = 10_000;

/// Inserts between the `run_pending_tasks` calls timed into moka's `insert_at_capacity`.
const MAINTENANCE_EVERY: u64 = 1_000;

type Group<'a> = criterion::BenchmarkGroup<'a, WallTime>;

/// The record that mints `n`: an id spread over the whole key space, as random ones are.
fn id(n: u64) -> SurbId {
    n.wrapping_mul(0x9E37_79B9_7F4A_7C15).to_le_bytes()
}

fn random_eviction_map() -> RandomEvictionMap<Record> {
    RandomEvictionMap::new(CAPACITY * record_bytes::<Record>() as u64)
}

/// The cache this map replaces, as it was configured.
fn moka_cache() -> (moka::sync::Cache<SurbId, Record>, Arc<AtomicU64>) {
    let lost = Arc::new(AtomicU64::new(0));
    let counter = lost.clone();
    let cache = moka::sync::Cache::builder()
        .max_capacity(CAPACITY * MOKA_RECORD_BYTES)
        .weigher(|_, _| MOKA_RECORD_BYTES as u32)
        .eviction_policy(moka::policy::EvictionPolicy::lru())
        .eviction_listener(move |_, _, cause| {
            if cause == moka::notification::RemovalCause::Size {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        })
        .build();
    (cache, lost)
}

/// Mint then spend a record, the map staying below capacity: what a healthy node does per SURB.
fn insert_and_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_and_remove");
    group.throughput(Throughput::Elements(1));
    random_insert_and_remove(&mut group);
    moka_insert_and_remove(&mut group);
    group.finish();
}

fn random_insert_and_remove(group: &mut Group) {
    let map = random_eviction_map();
    let mut n = 0u64;
    group.bench_function("random_eviction_map", |b| {
        b.iter(|| {
            n += 1;
            map.insert(id(n), [0u8; 504]);
            black_box(map.remove(&id(n)))
        })
    });
}

fn moka_insert_and_remove(group: &mut Group) {
    let (cache, _lost) = moka_cache();
    let mut n = 0u64;
    group.bench_function("moka_lru", |b| {
        b.iter(|| {
            n += 1;
            cache.insert(id(n), [0u8; 504]);
            black_box(cache.remove(&id(n)))
        })
    });
}

/// Mint into a full map: every insert evicts a record. Moka first in this group.
fn insert_at_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_at_capacity");
    group.throughput(Throughput::Elements(1));
    let (cache, _lost) = moka_cache();
    for n in 0..CAPACITY {
        cache.insert(id(n), [0u8; 504]);
    }
    cache.run_pending_tasks();
    let mut n = CAPACITY;
    group.bench_function("moka_lru", |b| {
        b.iter(|| {
            n += 1;
            cache.insert(id(n), [0u8; 504]);
            // Its size eviction and listener are deferred: time them in.
            if n.is_multiple_of(MAINTENANCE_EVERY) {
                cache.run_pending_tasks();
            }
        })
    });
    let map = random_eviction_map();
    for n in 0..CAPACITY {
        map.insert(id(n), [0u8; 504]);
    }
    let mut n = CAPACITY;
    group.bench_function("random_eviction_map", |b| {
        b.iter(|| {
            n += 1;
            black_box(map.insert(id(n), [0u8; 504]))
        })
    });
    group.finish();
}

/// Runs `work(thread index)` on `THREADS` threads released together, and times from the release to the
/// last one finishing: spawning and joining are outside the clock.
fn timed_threads(work: impl Fn(u64) + Sync) -> Duration {
    let (start, end) = (Barrier::new(THREADS as usize + 1), Barrier::new(THREADS as usize + 1));
    thread::scope(|scope| {
        for t in 0..THREADS {
            let (start, end, work) = (&start, &end, &work);
            scope.spawn(move || {
                start.wait();
                work(t);
                end.wait();
            });
        }
        start.wait();
        let began = Instant::now();
        end.wait();
        began.elapsed()
    })
}

/// Several threads minting and spending at once, as the encoder and decoder halves do.
fn concurrent_insert_and_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_insert_and_remove");
    group.throughput(Throughput::Elements(THREADS * PER_THREAD));
    let map = random_eviction_map();
    group.bench_function("random_eviction_map", |b| {
        b.iter_custom(|iters| {
            (0..iters)
                .map(|_| {
                    timed_threads(|t| {
                        for n in t * PER_THREAD..(t + 1) * PER_THREAD {
                            map.insert(id(n), [0u8; 504]);
                            black_box(map.remove(&id(n)));
                        }
                    })
                })
                .sum()
        })
    });
    let (cache, _lost) = moka_cache();
    group.bench_function("moka_lru", |b| {
        b.iter_custom(|iters| {
            (0..iters)
                .map(|_| {
                    timed_threads(|t| {
                        for n in t * PER_THREAD..(t + 1) * PER_THREAD {
                            cache.insert(id(n), [0u8; 504]);
                            black_box(cache.remove(&id(n)));
                        }
                    })
                })
                .sum()
        })
    });
    group.finish();
}

/// Walk every record, as the write-off and the invalid count do once per flush tick. Moka first.
fn walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("walk");
    group.throughput(Throughput::Elements(CAPACITY));
    let (cache, _lost) = moka_cache();
    for n in 0..CAPACITY {
        cache.insert(id(n), [n as u8; 504]);
    }
    cache.run_pending_tasks();
    group.bench_function("moka_lru", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for (_, record) in cache.iter() {
                sum += u64::from(record[0]);
            }
            black_box(sum)
        })
    });
    let map = random_eviction_map();
    for n in 0..CAPACITY {
        map.insert(id(n), [n as u8; 504]);
    }
    group.bench_function("random_eviction_map", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            map.for_each(|_, record| sum += u64::from(record[0]));
            black_box(sum)
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    insert_and_remove,
    insert_at_capacity,
    concurrent_insert_and_remove,
    walk
);
criterion_main!(benches);
