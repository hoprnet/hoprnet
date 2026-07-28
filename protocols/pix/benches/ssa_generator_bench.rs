//! Entry-side (`SsaShareGenerator`) benchmarks.
//!
//! ## Dimensions
//!
//! The always-measured point is the deployed one: `polys × threshold` =
//! [`PROD_POLYS_PER_SSA`] × [`PROD_THRESHOLD`], mirroring `DEFAULT_PIX_POLYS_PER_SSA` and
//! `DEFAULT_PIX_SHARES_PER_POLY` in `transport/session/src/types.rs`. The wider sweeps are
//! gated behind the `all-benchmarks` feature: every `(threshold, polynomials)` pair in
//! [`THRESHOLDS`] × [`POLYNOMIALS`] has roughly the same product, so each one costs about
//! the same as the production point and the full grid is ~16× the runtime for little
//! extra signal.

#[path = "../tests/common.rs"]
mod common;

use std::time::{Duration, Instant};

use common::TestSpec;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{
    DEFAULT_POLYS_PER_SSA, EntryShareGenerator, GeneratedShare, SsaGeneratorConfig, SsaIndex, SsaShareGenerator,
};
use hopr_types::{crypto::prelude::SimplePseudonym, crypto_random::Randomizable};

/// Deployed number of polynomials per SSA.
///
/// Mirrors `DEFAULT_PIX_POLYS_PER_SSA` (`transport/session/src/types.rs`), which currently
/// coincides with the pix crate's own [`DEFAULT_POLYS_PER_SSA`].
const PROD_POLYS_PER_SSA: u16 = DEFAULT_POLYS_PER_SSA;

/// Deployed polynomial threshold (shares needed to reconstruct one polynomial).
///
/// Mirrors `DEFAULT_PIX_SHARES_PER_POLY` (`transport/session/src/types.rs`). Deliberately
/// **not** the pix crate's `DEFAULT_POLY_THRESHOLD`, which is still 128 — the negotiated
/// session value is what nodes actually run with.
const PROD_THRESHOLD: u16 = 64;

// These values all correspond to a 512 MB quota (given ca. 1 kb HOPR packet payload capacity).
// The last entry of each is the deployed value, so the sweep is a superset of the default set.
#[cfg(feature = "all-benchmarks")]
const THRESHOLDS: [u16; 4] = [8, 16, 32, PROD_THRESHOLD];
#[cfg(feature = "all-benchmarks")]
const POLYNOMIALS: [u16; 4] = [65535, 32768, 16384, PROD_POLYS_PER_SSA];

#[cfg(not(feature = "all-benchmarks"))]
const THRESHOLDS: [u16; 1] = [PROD_THRESHOLD];
#[cfg(not(feature = "all-benchmarks"))]
const POLYNOMIALS: [u16; 1] = [PROD_POLYS_PER_SSA];

/// Number of `next_share` calls timed as one criterion sample.
///
/// `next_share` is tens of microseconds, while the `new_ssa_commitment` that has to precede
/// it costs seconds. Timing one call per iteration would make criterion pay a commitment
/// per measurement; batching amortises the commitment over [`NEXT_SHARE_BATCH`] calls
/// instead. Same rationale as `transport/session/benches/dispatch_bench.rs`.
const NEXT_SHARE_BATCH: usize = 4096;

/// Polynomials per SSA used by the `next_share` benchmark.
///
/// `next_share` evaluates a single polynomial (Horner over `threshold` coefficients), so
/// its cost is independent of how many polynomials are queued. A smaller value keeps the
/// untimed re-commitments cheap when the share budget runs dry mid-run.
const NEXT_SHARE_BENCH_POLYS: u16 = 1024;

fn bench_new_ssa_commitment(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaShareGenerator::new_ssa_commitment");

    // Cap measurement time so larger parameter combinations don't blow up wall-clock time.
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for &threshold in &THRESHOLDS {
        for &polynomials_per_ssa in &POLYNOMIALS {
            // One "element" is one coefficient commitment, so the reported throughput is
            // directly comparable across parameter shapes with the same product.
            group.throughput(Throughput::Elements(polynomials_per_ssa as u64 * threshold as u64));
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
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let pseudonym = SimplePseudonym::random();
    let x = hopr_types::crypto_random::random_bytes::<10>();

    let mut index = SsaIndex::MIN;
    for &threshold in &THRESHOLDS {
        // Verification touches exactly one polynomial's commitments, so a single polynomial
        // is enough. Committing at production width here would spend seconds per threshold
        // building 8191 verifiers that are never used.
        let cfg = SsaGeneratorConfig {
            threshold,
            polynomials_per_ssa: 1,
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
    group.throughput(Throughput::Elements(NEXT_SHARE_BATCH as u64));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let pseudonym = SimplePseudonym::random();

    // Benchmark does not depend on polynomials_per_ssa
    for &threshold in &THRESHOLDS {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("t{threshold}_p{NEXT_SHARE_BENCH_POLYS}")),
            &threshold,
            |b, _| {
                let cfg = SsaGeneratorConfig {
                    threshold,
                    polynomials_per_ssa: NEXT_SHARE_BENCH_POLYS,
                    ..Default::default()
                };
                // A commitment yields `polys * (threshold + surplus)` shares before the
                // generator runs dry and starts returning `Ok(None)`.
                let shares_per_commitment = NEXT_SHARE_BENCH_POLYS as usize * (threshold as usize + cfg.surplus_shares);
                assert!(
                    shares_per_commitment >= NEXT_SHARE_BATCH,
                    "one commitment must cover a whole batch"
                );

                let generator = SsaShareGenerator::<TestSpec>::new(cfg);
                let mut ssa_index = SsaIndex::MIN;
                generator.new_ssa_commitment(&pseudonym, ssa_index).unwrap();
                let mut remaining = shares_per_commitment;
                let mut counter: u64 = 0;

                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;

                    for _ in 0..iters {
                        // Top up the share budget *outside* the timed section, so the
                        // measurement only ever covers calls that really produce a share.
                        // `new_ssa_commitment` appends to the pseudonym's polynomial queue,
                        // so the budget accumulates.
                        if remaining < NEXT_SHARE_BATCH {
                            ssa_index = ssa_index.checked_add(1).unwrap();
                            generator.new_ssa_commitment(&pseudonym, ssa_index).unwrap();
                            remaining += shares_per_commitment;
                        }

                        let mut produced = 0usize;
                        let start = Instant::now();
                        for _ in 0..NEXT_SHARE_BATCH {
                            // Each `msg` must be unique for a given pseudonym.
                            let x = counter.to_be_bytes();
                            counter = counter.wrapping_add(1);
                            produced += generator.next_share(&pseudonym, &x).unwrap().is_some() as usize;
                        }
                        total += start.elapsed();

                        assert_eq!(
                            produced, NEXT_SHARE_BATCH,
                            "generator ran dry inside a timed batch, measurement is not comparable"
                        );
                        remaining -= NEXT_SHARE_BATCH;
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
    group.measurement_time(Duration::from_secs(10));
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
