use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{PixSpec, SsaGeneratorConfig, SsaShareGenerator};
use hopr_types::{crypto::prelude::SimplePseudonym, crypto_random::Randomizable};
use vsss_rs::elliptic_curve::Field;

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

fn bench_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareVerifier::verify");
    group.throughput(Throughput::Elements(1));

    let thresholds = [10, 50, 100, 200];
    let pseudonym = SimplePseudonym::random();
    let x = k256::Scalar::random(&mut vsss_rs::elliptic_curve::rand_core::OsRng);

    for &t in &thresholds {
        let cfg = SsaGeneratorConfig {
            threshold: t,
            ..Default::default()
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);
        let (_commitment, verifiers) = generator.new_ssa_commitment(&pseudonym).unwrap();
        let (_index, share) = generator.next_share(&pseudonym, x).unwrap();
        let verifier = &verifiers[0];

        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, _| {
            b.iter(|| {
                verifier.verify(&share, x).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_new_ssa_commitment, bench_verify);
criterion_main!(benches);
