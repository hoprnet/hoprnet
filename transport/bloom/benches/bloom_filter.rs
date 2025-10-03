use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::types::PACKET_TAG_LENGTH;
use hopr_transport_bloom::TagBloomFilter;

fn tag_bloom_filter_bench(c: &mut Criterion) {
    // Fill up the Bloom filter first
    let mut bloom = TagBloomFilter::default();
    for _ in 1..bloom.capacity() {
        let mut tag = hopr_crypto_random::random_bytes();
        tag[0] = 0xaa;
        bloom.set(&tag);
    }

    let mut existing_tag = hopr_crypto_random::random_bytes();
    existing_tag[0] = 0xaa;
    bloom.set(&existing_tag);

    let mut group = c.benchmark_group("tag_bloom_filter");
    group.sample_size(100_000);
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function("check_existing", |b| {
        b.iter(|| {
            bloom.check(black_box(&existing_tag));
        })
    });

    let non_existent_tag = [0u8; PACKET_TAG_LENGTH];
    group.bench_function("check_non_existent", |b| {
        b.iter(|| {
            bloom.check(black_box(&non_existent_tag));
        })
    });
}

criterion_group!(benches, tag_bloom_filter_bench);
criterion_main!(benches);
