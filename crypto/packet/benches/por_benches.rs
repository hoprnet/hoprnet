#[allow(unused)]
#[path = "../src/errors.rs"]
mod errors;

#[allow(unused)]
#[path = "../src/por.rs"]
mod por;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_packet::HoprSphinxSuite;
use hopr_crypto_random::Randomizable;
use hopr_crypto_sphinx::prelude::{SharedSecret, SphinxSuite};
use hopr_crypto_types::{keypairs::Keypair, prelude::OffchainKeypair};
use por::{generate_proof_of_relay, pre_verify};

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

const SAMPLE_SIZE: usize = 100_000;

pub fn proof_of_relay_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_of_relay_bench");
    group.sample_size(SAMPLE_SIZE);

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("por_create_{hop}_hop")),
            hop,
            |b, &hop| {
                let secrets = HoprSphinxSuite::new_shared_keys(
                    &(0..=hop)
                        .map(|_| *OffchainKeypair::random().public())
                        .collect::<Vec<_>>(),
                )
                .unwrap()
                .secrets;

                b.iter(|| generate_proof_of_relay(&secrets).expect("failed to generate proof of relay"));
            },
        );
    }
    let secrets = (0..3).map(|_| SharedSecret::random()).collect::<Vec<_>>();
    let (pors, porv) = generate_proof_of_relay(&secrets).unwrap();

    group.bench_function("por_pre_verify", |b| {
        b.iter(|| {
            pre_verify(&secrets[0], &pors[0], &porv.ticket_challenge()).unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, proof_of_relay_bench);
criterion_main!(benches);
