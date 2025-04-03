use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hopr_crypto_packet::prelude::generate_proof_of_relay;
use hopr_crypto_packet::HoprSphinxSuite;
use hopr_crypto_sphinx::prelude::SphinxSuite;
use hopr_crypto_types::keypairs::Keypair;
use hopr_crypto_types::prelude::OffchainKeypair;

const SAMPLE_SIZE: usize = 100_000;

pub fn por_creation_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("por_creation");
    group.sample_size(SAMPLE_SIZE);

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hop} hop")), hop, |b, &hop| {
            let secrets = HoprSphinxSuite::new_shared_keys(
                &(0..=hop)
                    .map(|_| OffchainKeypair::random().public().clone())
                    .collect::<Vec<_>>(),
            )
            .unwrap()
            .secrets;

            b.iter(|| generate_proof_of_relay(&secrets).expect("failed to generate proof of relay"));
        });
    }
    group.finish();
}

/*pub fn por_pre_verification(c: &mut Criterion) {

}*/

criterion_group!(
    benches,
    por_creation_bench,
    //por_pre_verification,
);
criterion_main!(benches);
