use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{EntryShareGenerator, GeneratedShare, PixSpec, SsaGeneratorConfig, SsaShareGenerator};
use hopr_types::{
    crypto::{
        prelude::SimplePseudonym,
        primitives::{Blake3, ChaCha20},
    },
    crypto_random::Randomizable,
};

pub struct TestSpec;

impl PixSpec for TestSpec {
    type Cipher = ChaCha20;
    type Curve = k256::Secp256k1;
    type Digest = Blake3;
    type Pseudonym = SimplePseudonym;
}

fn bench_new_ssa_commitment(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::new_ssa_commitment");

    // Cap measurement time so larger parameter combinations don't blow up wall-clock time.
    group.measurement_time(std::time::Duration::from_secs(30));

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
    let x = hopr_types::crypto_random::random_bytes::<10>();

    for &t in &thresholds {
        let cfg = SsaGeneratorConfig {
            threshold: t,
            ..Default::default()
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);
        let c = generator.new_ssa_commitment(&pseudonym).unwrap();
        let GeneratedShare { share, .. } = generator.next_share(&pseudonym, &x).unwrap().unwrap();

        let verifiers = c.reconstruct_verifiers().unwrap();
        let verifier = &verifiers[0];

        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, _| {
            b.iter(|| {
                verifier.verify(&share, x).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_next_share(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::next_share");
    group.throughput(Throughput::Elements(1));

    let thresholds = [10, 50, 100, 200];
    let polynomials_per_ssa = [2048]; // Benchmark does not depend on polynomials_per_ssa
    let pseudonym = SimplePseudonym::random();

    for &t in &thresholds {
        for &p in &polynomials_per_ssa {
            let cfg = SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa: p,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(cfg);
            // Prime the generator with an initial commitment, so the first iteration
            // has a polynomial to draw from.
            generator.new_ssa_commitment(&pseudonym).unwrap();

            group.bench_with_input(
                BenchmarkId::from_parameter(format!("t{}_p{}", t, p)),
                &(t, p),
                |b, _| {
                    // `next_share` mutates internal state, consuming one share per call,
                    // and requires that `msg` is unique for a given pseudonym. We use
                    // `iter_custom` so that whenever the underlying SSA gets exhausted
                    // (i.e. `next_share` yields `None`), we can transparently re-commit
                    // a new SSA without polluting the measured time.
                    let mut counter: u64 = 0;
                    b.iter_custom(|iters| {
                        let mut total = std::time::Duration::ZERO;
                        for _ in 0..iters {
                            let x = counter.to_be_bytes();
                            counter = counter.wrapping_add(1);

                            let start = std::time::Instant::now();
                            let res = generator.next_share(&pseudonym, &x).unwrap();
                            total += start.elapsed();

                            if res.is_none() {
                                // Refill the SSA pool outside of the measured region.
                                // This intentionally "wastes" any remaining shares from
                                // the previous SSA (there are none at this point) and
                                // ensures subsequent iterations have shares available.
                                generator.new_ssa_commitment(&pseudonym).unwrap();
                            }
                        }
                        total
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_next_share_no_ssa(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::next_share_no_ssa");
    group.throughput(Throughput::Elements(1));

    // Default configuration is enough; behavior under test does not depend on
    // threshold / polynomials_per_ssa because no SSA is ever committed.
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig::default());
    // Pseudonym that has never had `new_ssa_commitment` called for it, so every
    // call to `next_share` must take the early-return `Ok(None)` path.
    let pseudonym = SimplePseudonym::random();
    let msg = hopr_types::crypto_random::random_bytes::<10>();

    group.bench_function("no_commitment", |b| {
        b.iter(|| {
            let res = generator.next_share(&pseudonym, &msg).unwrap();
            debug_assert!(res.is_none());
            res
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_new_ssa_commitment,
    bench_verify,
    bench_next_share,
    bench_next_share_no_ssa
);
criterion_main!(benches);
