use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::prelude::*;

const SAMPLE_SIZE: usize = 100_000;

pub fn hash_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_bench");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(10));
    group.throughput(Throughput::Elements(1));

    let data = hopr_crypto_random::random_bytes::<4096>();

    group.bench_function("hash_keccak256", |b| b.iter(|| Hash::create(&[&data])));

    group.bench_function("hash_fast_blake3", |b| b.iter(|| HashFast::create(&[&data])));
}

criterion_group!(benches, hash_bench);
criterion_main!(benches);
