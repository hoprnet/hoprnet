use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hopr_protocol_pix::{PixSpec, SsaGeneratorConfig, SsaShareGenerator};
use hopr_types::{crypto::prelude::SimplePseudonym, crypto_random::Randomizable};

pub struct TestSpec;

impl PixSpec for TestSpec {
    type Element = k256::ProjectivePoint;
    type Pseudonym = SimplePseudonym;
    type Scalar = k256::Scalar;
}

fn bench_new_ssa_commitment(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::new_ssa_commitment");
    let thresholds = [10, 50, 100, 200];
    let polynomials_per_ssa = [128, 512, 1024, 2048];
    let pseudonym = SimplePseudonym::random();

    for &t in &thresholds {
        for &p in &polynomials_per_ssa {
            let cfg = SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa: p,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(cfg);

            group.bench_with_input(
                BenchmarkId::from_parameter(format!("t{}_p{}", t, p)),
                &(t, p),
                |b, _| {
                    b.iter(|| {
                        generator.new_ssa_commitment(&pseudonym).unwrap();
                    });
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_new_ssa_commitment);
criterion_main!(benches);
