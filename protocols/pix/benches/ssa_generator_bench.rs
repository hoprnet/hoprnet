#[path = "../tests/common.rs"]
mod common;

use common::TestSpec;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{EntryShareGenerator, GeneratedShare, SsaGeneratorConfig, SsaIndex, SsaShareGenerator};
use hopr_types::{crypto::prelude::SimplePseudonym, crypto_random::Randomizable};

// These values all correspond to a 512 MB quota (given ca. 1 kb HOPR packet payload capacity)
const THRESHOLDS: [u16; 4] = [8, 16, 32, 64];
const POLYNOMIALS: [u16; 4] = [65535, 32768, 16384, 8192];

fn bench_new_ssa_commitment(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::new_ssa_commitment");

    // Cap measurement time so larger parameter combinations don't blow up wall-clock time.
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(10);

    for &threshold in &THRESHOLDS {
        for &polynomials_per_ssa in &POLYNOMIALS {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("t{threshold}_p{polynomials_per_ssa}")),
                &(threshold, polynomials_per_ssa),
                |b, _| {
                    b.iter_batched(
                        || {
                            let cfg = SsaGeneratorConfig {
                                threshold,
                                polynomials_per_ssa,
                                ..Default::default()
                            };
                            (SsaShareGenerator::<TestSpec>::new(cfg), SimplePseudonym::random())
                        },
                        |(generator, pseudonym)| generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap(),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn bench_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareVerifier::verify");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(10);

    let pseudonym = SimplePseudonym::random();
    let x = hopr_types::crypto_random::random_bytes::<10>();

    let mut index = SsaIndex::MIN;
    for &threshold in &THRESHOLDS {
        let cfg = SsaGeneratorConfig {
            threshold,
            ..Default::default()
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);
        let c = generator.new_ssa_commitment(&pseudonym, index).unwrap();
        index = index.checked_add(1).unwrap();

        let GeneratedShare { share, .. } = generator.next_share(&pseudonym, &x).unwrap().unwrap();

        let verifiers = c.reconstruct_verifiers().unwrap();
        let verifier = &verifiers[0];

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("t{threshold}")),
            &threshold,
            |b, _| {
                b.iter(|| {
                    verifier.verify(&share, x).unwrap();
                });
            },
        );
    }
    group.finish();
}

fn bench_next_share(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::next_share");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(10);

    let pseudonym = SimplePseudonym::random();

    // Benchmark does not depend on polynomials_per_ssa
    for &threshold in &THRESHOLDS {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("t{threshold}_p2048")),
            &threshold,
            |b, _| {
                let mut index = SsaIndex::MIN;
                let mut counter: u64 = 0;

                b.iter_custom(|iters| {
                    let mut total = std::time::Duration::ZERO;

                    for _ in 0..iters {
                        // Create a fresh generator per iteration so that cache
                        // pressure does not grow across the run.
                        let cfg = SsaGeneratorConfig {
                            threshold,
                            polynomials_per_ssa: 2048,
                            ..Default::default()
                        };
                        let generator = SsaShareGenerator::<TestSpec>::new(cfg);
                        generator.new_ssa_commitment(&pseudonym, index).unwrap();
                        index = index.checked_add(1).unwrap();

                        let x = counter.to_be_bytes();
                        counter = counter.wrapping_add(1);

                        let start = std::time::Instant::now();
                        let res = generator.next_share(&pseudonym, &x).unwrap();
                        total += start.elapsed();

                        debug_assert!(res.is_some(), "freshly-committed generator should produce a share");
                    }
                    total
                });
            },
        );
    }
    group.finish();
}

fn bench_next_share_no_ssa(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::next_share_no_ssa");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(10);

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
