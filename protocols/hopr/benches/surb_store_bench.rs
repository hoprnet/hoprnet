//! A/B benchmark for the SURB store's two hot operations.
//!
//! `insert_surbs` runs once per incoming packet that carries SURBs; `find_surb` runs once per
//! reply. Both are on the packet hot path and both changed shape when the store began bucketing
//! share-bearing SURBs by emission window.
//!
//! # Why this is written the awkward way
//!
//! Two traps make the obvious version measure the fixture instead of the operation:
//!
//! 1. **Per-iteration setup.** A single pop is tens of nanoseconds. Built inside the timed region, the fixture
//!    dominates. So each sample performs [`OPS`] operations and the trapezoid is amortised away.
//! 2. **Teardown.** `iter_batched` drops the input batch after the routine, and dropping a `MemorySurbStore` holding
//!    100 000 SURBs costs more than the operations themselves — it scales with occupancy exactly like the numbers did,
//!    which is how the first version of this file ended up reporting ~6 µs per pop. [`Criterion::iter_custom`] lets the
//!    fixture be built and dropped outside the measured span.
//!
//! Only the public `SurbStore` trait is used, so the identical source compiles against the baseline
//! and the branch and measures the same operations in both.

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_api::types::{
    crypto::{crypto_traits::Randomizable, prelude::*},
    internal::{prelude::*, routing::SurbMatcher},
    primitive::prelude::BytesRepresentable,
};
use hopr_crypto_packet::{prelude::*, sphinx::prelude::SphinxHeaderSpec};
use hopr_protocol_hopr::{MemorySurbStore, SurbStore, SurbStoreConfig};

const SSA_INDEX: u32 = 1;
/// Operations per timed sample.
const OPS: usize = 256;

fn surb_at(poly_index: u16) -> HoprSurb {
    let mut bytes = vec![0u8; HoprSurb::SIZE];
    let key_id_size = HoprSphinxHeaderSpec::KEY_ID_SIZE.get();
    bytes[..key_id_size].copy_from_slice(HoprKeyIdent::from(1u32).as_ref());
    bytes[HoprSurb::SIZE - HoprSphinxHeaderSpec::SURB_RECEIVER_DATA_SIZE] = 1;
    let share_at = HoprSurb::SIZE - 1 - HoprEncryptedPartialSsaShare::SIZE;
    bytes[share_at..share_at + 4].copy_from_slice(&SSA_INDEX.to_be_bytes());
    bytes[share_at + 4..share_at + 6].copy_from_slice(&poly_index.to_be_bytes());
    HoprSurb::try_from(bytes.as_slice()).expect("fixture")
}

fn plain_surb() -> HoprSurb {
    let mut bytes = vec![0u8; HoprSurb::SIZE];
    let key_id_size = HoprSphinxHeaderSpec::KEY_ID_SIZE.get();
    bytes[..key_id_size].copy_from_slice(HoprKeyIdent::from(1u32).as_ref());
    bytes[HoprSurb::SIZE - HoprSphinxHeaderSpec::SURB_RECEIVER_DATA_SIZE] = 1;
    HoprSurb::try_from(bytes.as_slice()).expect("fixture")
}

fn populated(fill: usize, share_bearing: bool, capacity: usize) -> (MemorySurbStore, SimplePseudonym) {
    let store = MemorySurbStore::new(SurbStoreConfig {
        rb_capacity: capacity,
        ..Default::default()
    });
    let pseudonym = SimplePseudonym::random();
    // Batched, because 100 000 single-SURB inserts is setup cost, not something under test.
    let batch: Vec<(HoprSurbId, HoprSurb)> = (0..fill)
        .map(|i| {
            let sid: HoprSurbId = (i as u64).to_be_bytes();
            let surb = if share_bearing {
                surb_at((i % 4096) as u16)
            } else {
                plain_surb()
            };
            (sid, surb)
        })
        .collect();
    store.insert_surbs(pseudonym, batch);
    (store, pseudonym)
}

/// `insert_surbs` on a buffer held at the balancer's 7 000-SURB target.
///
/// The fixture is rebuilt for EVERY sample, not once per benchmark. A store that is only built once
/// accumulates across criterion's samples until it reaches `rb_capacity`, after which the timed
/// inserts are evictions rather than steady-state insertions — a different thing being measured
/// under the same name. Rebuilding in the setup closure costs an allocation the timer never sees.
fn insert_into_populated_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("surb_store/insert");
    group.throughput(Throughput::Elements(OPS as u64));

    for (label, share_bearing) in [("share_bearing", true), ("share_less", false)] {
        group.bench_function(BenchmarkId::from_parameter(label), |b| {
            let mut id: u64 = 1_000_000;
            b.iter_custom(|iters| {
                let mut elapsed = std::time::Duration::ZERO;
                for _ in 0..iters {
                    // Fresh buffer at the target for each sample, built outside the timer.
                    let (store, pseudonym) = populated(7_000, share_bearing, 100_000);
                    let started = Instant::now();
                    for _ in 0..OPS {
                        let sid: HoprSurbId = id.to_be_bytes();
                        let surb = if share_bearing {
                            surb_at((id % 4096) as u16)
                        } else {
                            plain_surb()
                        };
                        id += 1;
                        std::hint::black_box(store.insert_surbs(pseudonym, vec![(sid, surb)]));
                    }
                    elapsed += started.elapsed();
                    // Dropped outside the measured span.
                    drop(store);
                }
                elapsed
            });
        });
    }
    group.finish();
}

/// `find_surb` — the reply path, one pop per reply packet.
///
/// Every successful lookup CONSUMES a SURB, so a fixture filled once would run dry and later
/// samples would time the empty path. The buffer is therefore topped back up to the advertised
/// occupancy between samples.
///
/// The refill happens OUTSIDE the timer, and the store itself is built once per `iter_custom`
/// call rather than once per sample: criterion drives `iters` samples per call, and rebuilding a
/// 100 000-SURB fixture inside that loop would cost far more than the operations being measured
/// (it is what made an earlier version of this benchmark report ~400 ns per pop at 100k while the
/// same code measured ~165 ns at 7k).
fn find_surb_from_populated_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("surb_store/find");
    group.throughput(Throughput::Elements(OPS as u64));

    for (label, fill, share_bearing, capacity) in [
        ("7k_share_bearing", 7_000usize, true, 100_000usize),
        ("7k_share_less", 7_000, false, 100_000),
        ("100k_share_bearing", 100_000, true, 1_000_000),
    ] {
        group.bench_function(BenchmarkId::from_parameter(label), |b| {
            b.iter_custom(|iters| {
                let (store, pseudonym) = populated(fill, share_bearing, capacity);
                let mut elapsed = std::time::Duration::ZERO;
                for sample in 0..iters {
                    // Top back up to `fill` before the timer starts. `OPS` pops per sample need at
                    // least that many resident; refilling only when needed keeps the fixture
                    // rebuild out of the vast majority of the timed iterations.
                    if sample > 0 {
                        let missing = OPS;
                        let base = sample * OPS as u64 + fill as u64;
                        let surbs = (0..missing)
                            .map(|i| {
                                let id = base + i as u64;
                                let sid: HoprSurbId = id.to_be_bytes();
                                let surb = if share_bearing {
                                    surb_at((id % 4096) as u16)
                                } else {
                                    plain_surb()
                                };
                                (sid, surb)
                            })
                            .collect::<Vec<_>>();
                        store.insert_surbs(pseudonym, surbs);
                    }
                    let started = Instant::now();
                    for _ in 0..OPS {
                        std::hint::black_box(store.find_surb(SurbMatcher::Pseudonym(pseudonym)));
                    }
                    elapsed += started.elapsed();
                }
                elapsed
            });
        });
    }
    group.finish();
}

criterion_group!(benches, insert_into_populated_store, find_surb_from_populated_store);
criterion_main!(benches);
