use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::{
    keypairs::Keypair,
    prelude::{OffchainKeypair, OffchainPublicKey},
};
use libp2p_identity::PeerId;

// Avoid musl's default allocator due to degraded performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const SAMPLE_SIZE: usize = 100_000;

pub fn offchain_public_key_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("offchain_public_key_bench");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));

    group.bench_function("offchain_public_key_from_peer_id", |b| {
        let peer_id = PeerId::from(OffchainKeypair::random().public());
        b.iter(|| OffchainPublicKey::from_peerid(&peer_id))
    });
}

criterion_group!(benches, offchain_public_key_bench);
criterion_main!(benches);
