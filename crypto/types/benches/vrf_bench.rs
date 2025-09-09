use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::{
    keypairs::ChainKeypair,
    prelude::{Hash, Keypair},
    vrf::derive_vrf_parameters,
};

// Avoid musl's default allocator due to degraded performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const SAMPLE_SIZE: usize = 100_000;

pub fn vrf_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf_bench");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));

    group.bench_function("vrf_derive_bench", |b| {
        let ck = ChainKeypair::random();
        let hash = Hash::create(&[b"test"]);
        let dst = Hash::create(&[b"dst"]);
        b.iter(|| {
            derive_vrf_parameters(hash, &ck, dst.as_ref()).unwrap();
        })
    });
}

criterion_group!(benches, vrf_bench);
criterion_main!(benches);
