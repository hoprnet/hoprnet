use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::{
    prelude::{ChainKeypair, Keypair, Signature},
    types::Hash,
};

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

criterion_group!(benches, chain_signature_bench);
criterion_main!(benches);
