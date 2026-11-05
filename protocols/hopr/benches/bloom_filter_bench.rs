use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_hopr::{ReplayFilterConfig, ReplayToken, TagReplayFilter};

fn synthetic_token(value: u64) -> ReplayToken {
    let mut token = ReplayToken::default();
    for (i, byte) in token.iter_mut().enumerate() {
        *byte = ((value >> (i % 8 * 8)) as u8).wrapping_add((i as u8).wrapping_mul(31));
    }
    token
}

/// Build a filter whose current generations are exactly full, so the numbers reflect the steady state rather than
/// a nearly empty bitmap.
///
/// Fills by counting successful inserts, so a Bloom false positive cannot leave a generation short. The count is
/// kept locally rather than read back from `TagReplayFilter::count`, which locks every shard - doing that once per
/// insert made the setup quadratic. Returns the filter and a token the current generation holds.
fn saturated_filter(max_tracked_packets: usize) -> (TagReplayFilter, ReplayToken) {
    let filter = TagReplayFilter::new(&ReplayFilterConfig {
        max_tracked_packets,
        shards: 64,
    });

    let target = filter.capacity();
    let mut inserted = 0usize;
    let mut candidate = 0u64;
    let mut present = synthetic_token(0);
    while inserted < target {
        let token = synthetic_token(candidate);
        candidate += 1;
        // `commit` reports a token it already knows; those do not fill a slot.
        if !filter.commit(&token) {
            inserted += 1;
            present = token;
        }
    }

    (filter, present)
}

fn tag_replay_filter_bench(c: &mut Criterion) {
    // A fraction of the default, so the setup loop does not dominate: the per-operation cost depends on the number
    // of hash probes, not on the filter size.
    let (filter, existing) = saturated_filter(200_000);

    let mut group = c.benchmark_group("tag_replay_filter");
    group.sample_size(100);
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));

    group.bench_function("contains_existing", |b| {
        b.iter(|| filter.contains(black_box(&existing)))
    });

    let non_existent = synthetic_token(u64::MAX);
    group.bench_function("contains_non_existent", |b| {
        b.iter(|| filter.contains(black_box(&non_existent)))
    });

    group.bench_function("commit_existing", |b| b.iter(|| filter.commit(black_box(&existing))));

    // Fresh tokens rotate shards as they are inserted, which is the steady state under live traffic. The counter
    // starts past everything `saturated_filter` inserted, so these really are new tokens rather than the cheap
    // already-present path.
    let mut counter = u64::MAX / 2;
    group.bench_function("commit_new", |b| {
        b.iter(|| {
            counter = counter.wrapping_add(1);
            filter.commit(black_box(&synthetic_token(counter)))
        })
    });
}

criterion_group!(benches, tag_replay_filter_bench);
criterion_main!(benches);
