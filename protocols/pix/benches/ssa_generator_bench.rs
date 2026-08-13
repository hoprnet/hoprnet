//! Entry-side (`SsaShareGenerator`) benchmarks.
//!
//! ## The operating box being modelled
//!
//! Deployments run a *range* on both dimensions, not a point:
//!
//! | | production |
//! | --------------------- | ---------- |
//! | polynomials per SSA   | 4 096 – 8 192 |
//! | threshold             | 16 – 64 |
//! | surplus shares        | `threshold/4`, covering 20 % share loss |
//! | SSAs in flight        | 2 – 3 per Session |
//! | per-Session rate      | 16 – 20 Mbps |
//! | clients per Exit      | 10 – 30 |
//!
//! Quota follows from the first three — `polys × (threshold + surplus) × PAYLOAD_SIZE`, per
//! `pix_params_to_quota` in `transport/session/src/types.rs` — and spans roughly 153 MB to 714 MB
//! across the box. It is an *output*, which is why the sweeps below cover the box rather than an
//! iso-quota diagonal: holding the product fixed models a trade no deployment actually makes.
//!
//! ## Dimensions
//!
//! The always-measured point is the deployed one: [`PROD_POLYS_PER_SSA`] × [`PROD_THRESHOLD`],
//! mirroring `DEFAULT_PIX_POLYS_PER_SSA` and `DEFAULT_PIX_SHARES_PER_POLY`. The wider sweeps are
//! gated behind the `all-benchmarks` feature and walk [`THRESHOLDS`] × [`POLYNOMIALS`], which is
//! the box above.

#[path = "../tests/common.rs"]
mod common;

use std::time::{Duration, Instant};

use common::TestSpec;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{
    DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, EntryShareGenerator, PixScalar, SsaGeneratorConfig, SsaIndex,
    SsaShareGenerator,
};
use hopr_types::{crypto::prelude::SimplePseudonym, crypto_random::Randomizable};

/// Deployed number of polynomials per SSA.
///
/// `DEFAULT_PIX_POLYS_PER_SSA` (`transport/session/src/types.rs`) is an alias of
/// [`DEFAULT_POLYS_PER_SSA`], so this is the negotiated value by construction.
const PROD_POLYS_PER_SSA: u16 = DEFAULT_POLYS_PER_SSA;

/// Deployed polynomial threshold (shares needed to reconstruct one polynomial).
///
/// `DEFAULT_PIX_SHARES_PER_POLY` (`transport/session/src/types.rs`) is an alias of
/// [`DEFAULT_POLY_THRESHOLD`], so this is the negotiated value by construction. It was a
/// literal 64 while the pix crate still said 128, which is exactly the drift the aliasing
/// removed.
const PROD_THRESHOLD: u8 = DEFAULT_POLY_THRESHOLD;

/// Surplus shares emitted per polynomial, derived from the threshold being swept.
///
/// A function rather than a constant, because the surplus *is* a ratio of the threshold — sized to
/// absorb 20 % share loss — and a sweep that varies the threshold while holding the surplus fixed
/// would silently vary the loss tolerance instead, from 20 % at one end to something else at the
/// other. See `default_surplus_for` in `hopr-protocol-pix`.
fn prod_surplus(threshold: u8) -> u8 {
    hopr_protocol_pix::default_surplus_for(threshold)
}

// The production operating box, per the module documentation. The last entry of each is the
// deployed value, so the sweep stays a superset of the default set.
//
// Every polynomial count here is inside [`MAX_POLYS_PER_SSA`] (16 192) — which the previous set was
// not. It read `[65535, 32768, 16384, 8192]`, and **three of those four points panicked**:
// `polynomials_per_ssa` is `#[validate(range(min = 1, max = MAX_POLYS_PER_SSA))]`, so only the
// deployed point was ever constructible and `--features all-benchmarks` aborted on the first entry.
//
// It was then briefly repointed at an iso-quota diagonal (`[15887, 13107, 10922, 8192]`), which
// compiled but modelled polynomial counts no deployment runs. Both mistakes have the same root:
// choosing the sweep from an idea about the parameter space rather than from what nodes are
// configured with.
#[cfg(feature = "all-benchmarks")]
const THRESHOLDS: [u8; 4] = [16, 32, 48, PROD_THRESHOLD];
#[cfg(feature = "all-benchmarks")]
const POLYNOMIALS: [u16; 3] = [4096, 6144, PROD_POLYS_PER_SSA];

#[cfg(not(feature = "all-benchmarks"))]
const THRESHOLDS: [u8; 1] = [PROD_THRESHOLD];
#[cfg(not(feature = "all-benchmarks"))]
const POLYNOMIALS: [u16; 1] = [PROD_POLYS_PER_SSA];

/// Compile-time check that every swept polynomial count is constructible.
///
/// The sweep has been wrong twice, both times by exceeding [`MAX_POLYS_PER_SSA`] and both times
/// discovered only by a panic partway through a benchmark run. A `const` assertion turns that into
/// a build failure, which is the whole difference between a benchmark configuration that is checked
/// and one that merely exists.
const _: () = {
    let mut i = 0;
    while i < POLYNOMIALS.len() {
        assert!(
            POLYNOMIALS[i] <= hopr_protocol_pix::MAX_POLYS_PER_SSA,
            "swept polynomial count exceeds MAX_POLYS_PER_SSA and would panic at run time"
        );
        i += 1;
    }
};

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
            // One "element" is one *commitment*, which post-M9 means one per polynomial — the
            // Entry commits to constant terms only. This used to report `polys × threshold`
            // elements, the pre-M9 Feldman matrix, which understated the per-commitment cost by
            // the threshold and made two shapes with the same product look identical when their
            // actual work differs by 4× across this sweep.
            group.throughput(Throughput::Elements(polynomials_per_ssa as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("t{threshold}_p{polynomials_per_ssa}")),
                &(threshold, polynomials_per_ssa),
                |b, _| {
                    b.iter_batched(
                        || {
                            let cfg = SsaGeneratorConfig {
                                threshold,
                                polynomials_per_ssa,
                                surplus_shares: prod_surplus(threshold),
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

/// Cost of opening one polynomial's commitment: the single elliptic curve operation the Exit
/// performs per polynomial, replacing the `threshold`-term MSM that used to run per *share*.
///
/// Parameterised by threshold only so the figures line up with the historical
/// `SsaShareVerifier::verify` series it supersedes; the cost is in fact threshold-independent,
/// which is the whole point.
fn bench_verify_reconstructed(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaPartCommitment::verify_reconstructed");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let pseudonym = SimplePseudonym::random();

    let mut index = SsaIndex::MIN;
    for &threshold in &THRESHOLDS {
        let cfg = SsaGeneratorConfig {
            threshold,
            polynomials_per_ssa: 1,
            surplus_shares: prod_surplus(threshold),
        };
        let generator = SsaShareGenerator::<TestSpec>::new(cfg);
        let c = generator.new_ssa_commitment(&pseudonym, index).unwrap();
        index = index.checked_add(1).unwrap();

        let commitments = c.reconstruct_part_commitments().unwrap();
        let commitment = &commitments[0];
        let secret = PixScalar::<TestSpec>::random(&mut hopr_types::crypto_random::rng());

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("t{threshold}")),
            &threshold,
            |b, _| {
                b.iter(|| {
                    std::hint::black_box(commitment.verify_reconstructed(&secret));
                });
            },
        );
    }
    group.finish();
}

/// The Entry's per-packet cost, and the half of the polys/threshold decision that was long assumed
/// away.
///
/// `IndexedPolynomial::next_share` evaluates a `threshold`-wide polynomial by Horner for every share
/// emitted, so unlike `new_ssa_commitment` — which a 4× threshold change moves by 7 % — this term is
/// genuinely threshold-dependent. Measured on 48 cores:
///
/// | threshold | µs/share |
/// | --------- | -------- |
/// | 16        | 0.90     |
/// | 32        | 1.20     |
/// | 48        | 1.51     |
/// | 64        | 1.82     |
///
/// **Requires `--features all-benchmarks` to say anything**: without it [`THRESHOLDS`] is the single
/// deployed point, and a one-point sweep cannot show a slope. That is how these figures came to be
/// missing from the calibration in the first place.
///
/// Against the Exit's 10.68 µs/share at the deployed threshold this is ~17 %, which is why the split
/// is decided on the Exit term — see [`hopr_protocol_pix::DEFAULT_POLY_THRESHOLD`] for the combined
/// tables and the stated objective.
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
                    surplus_shares: prod_surplus(threshold),
                };
                // A commitment yields `polys * (threshold + surplus)` shares before the
                // generator runs dry and starts returning `Ok(None)`.
                let shares_per_commitment =
                    NEXT_SHARE_BENCH_POLYS as usize * (threshold as usize + cfg.surplus_shares as usize);
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
    bench_verify_reconstructed,
    bench_next_share,
    bench_next_share_no_ssa
);
criterion_main!(benches);
