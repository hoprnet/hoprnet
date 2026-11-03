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

/// Build a filter that is at capacity in its current generation, so the numbers reflect the steady state rather
/// than a nearly empty bitmap.
fn saturated_filter(max_tracked_packets: usize) -> (TagReplayFilter, ReplayToken) {
    let filter = TagReplayFilter::new(&ReplayFilterConfig {
        max_tracked_packets,
        shards: 64,
    });

    for i in 0..max_tracked_packets {
        filter.commit(&synthetic_token(i as u64));
    }

    (filter, synthetic_token(max_tracked_packets as u64 - 1))
}

fn tag_replay_filter_bench(c: &mut Criterion) {
    // A fifth of the default, so the setup loop does not dominate: the per-operation cost does not depend on the
    // filter size, only on the number of hash probes.
    let (filter, existing) = saturated_filter(2_000_000);

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

    // Fresh tokens rotate shards as they are inserted, which is the steady state under live traffic.
    let mut counter = 0u64;
    group.bench_function("commit_new", |b| {
        b.iter(|| {
            counter = counter.wrapping_add(1);
            filter.commit(black_box(&synthetic_token(counter)))
        })
    });
}

criterion_group!(benches, tag_replay_filter_bench);
criterion_main!(benches);
