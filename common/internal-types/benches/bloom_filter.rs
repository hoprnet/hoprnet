use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use hopr_internal_types::prelude::TagBloomFilter;

fn tag_bloom_filter_bench(c: &mut Criterion) {
    // Fill up the Bloom filter first
    let mut bloom = TagBloomFilter::default();
    for _ in 1..bloom.capacity() {
        bloom.set(&hopr_crypto_random::random_bytes());
    }

    let tag = hopr_crypto_random::random_bytes();
    bloom.set(&tag);

    let mut group = c.benchmark_group("tag_bloom_filter");
    group.sample_size(100_000);
    group.throughput(Throughput::Elements(1));

    group.bench_function("check", |b| {
        b.iter(|| {
            bloom.check(black_box(&tag));
        })
    });
}

criterion_group!(benches, tag_bloom_filter_bench);
criterion_main!(benches);
