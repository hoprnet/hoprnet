use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::{
    prelude::{ChainKeypair, Keypair, Signature},
    types::Hash,
};
use hopr_crypto_types::prelude::{OffchainKeypair, OffchainSignature};

const SAMPLE_SIZE: usize = 100_000;

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
    group.throughput(Throughput::Elements(1));

    group.bench_function("offchain_signature_sign_hash", |b| {
        let ck = OffchainKeypair::random();
        let msg = Hash::create(&[b"test"]).as_ref().to_vec();
        b.iter(|| {
            OffchainSignature::sign_message(&msg, &ck)
        })
    });

    group.bench_function("offchain_signature_verify_hash", |b| {
        let ck = OffchainKeypair::random();
        let msg = Hash::create(&[b"test"]).as_ref().to_vec();
        let sig = OffchainSignature::sign_message(&msg, &ck);
        b.iter(|| sig.verify_message(&msg, ck.public()) )
    });
}

criterion_group!(benches, chain_signature_bench, offchain_signature_bench);
criterion_main!(benches);
