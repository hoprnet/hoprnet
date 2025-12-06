use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::{
    prelude::{ChainKeypair, Keypair, OffchainKeypair, OffchainSignature, Signature},
    types::Hash,
};

// Avoid musl's default allocator due to degraded performance
//
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(feature = "allocator-mimalloc", feature = "allocator-jemalloc"))]
compile_error!("feature \"allocator-jemalloc\" and feature \"allocator-mimalloc\" cannot be enabled at the same time");
#[cfg(all(target_os = "linux", feature = "allocator-mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(target_os = "linux", feature = "allocator-jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const SAMPLE_SIZE: usize = 10_000;

pub fn chain_signature_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_signature_bench");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));

    group.bench_function("chain_signature_sign_hash", |b| {
        let ck = ChainKeypair::random();
        let hash = Hash::create(&[b"test"]);
        b.iter(|| {
            Signature::sign_hash(&hash, &ck);
        })
    });

    group.bench_function("chain_signature_verify_hash", |b| {
        let ck = ChainKeypair::random();
        let hash = Hash::create(&[b"test"]);
        let sig = Signature::sign_hash(&hash, &ck);
        b.iter(|| sig.verify_hash(&hash, ck.public()))
    });

    group.bench_function("chain_signature_recover_hash", |b| {
        let ck = ChainKeypair::random();
        let hash = Hash::create(&[b"test"]);
        let sig = Signature::sign_hash(&hash, &ck);
        b.iter(|| sig.recover_from_hash(&hash))
    });
}

pub fn offchain_signature_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("offchain_signature_bench");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(20));
    group.throughput(Throughput::Elements(1));

    group.bench_function("offchain_signature_sign_hash", |b| {
        let ck = OffchainKeypair::random();
        let msg = Hash::create(&[b"test"]).as_ref().to_vec();
        b.iter(|| OffchainSignature::sign_message(&msg, &ck))
    });

    group.bench_function("offchain_signature_verify_hash", |b| {
        let ck = OffchainKeypair::random();
        let msg = Hash::create(&[b"test"]).as_ref().to_vec();
        let sig = OffchainSignature::sign_message(&msg, &ck);
        b.iter(|| sig.verify_message(&msg, ck.public()))
    });

    group.finish();

    const BATCH_SIZE: usize = 100;
    let mut group = c.benchmark_group("offchain_signature_batch_verify_bench");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(20));
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));

    group.bench_function("offchain_signature_verify_batch", |b| {
        let msgs = (0..BATCH_SIZE)
            .map(|i| format!("test_msg_{i}").as_bytes().to_vec())
            .collect::<Vec<_>>();

        let kps = (0..BATCH_SIZE).map(|_| OffchainKeypair::random()).collect::<Vec<_>>();

        let tuples = (0..BATCH_SIZE)
            .map(|i| {
                let sig = OffchainSignature::sign_message(&msgs[i], &kps[i]);
                ((msgs[i].as_slice(), sig), kps[i].public())
            })
            .collect::<Vec<_>>();

        b.iter_batched(
            || tuples.clone(),
            |tuples| OffchainSignature::verify_batch(tuples),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, chain_signature_bench, offchain_signature_bench);
criterion_main!(benches);
