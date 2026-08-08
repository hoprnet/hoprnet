//! Exit-side (`SsaReconstructor`) benchmarks.
//!
//! ## Dimensions
//!
//! Everything here is measured at the deployed dimensions
//! [`PROD_POLYS_PER_SSA`] × [`PROD_THRESHOLD`] (mirroring `DEFAULT_PIX_POLYS_PER_SSA` /
//! `DEFAULT_PIX_SHARES_PER_POLY` in `transport/session/src/types.rs`), because the
//! reconstructor's cost is dominated by state that only exists at scale: the verifier
//! cache holds `polys` entries, the awaited-share cache holds one entry per in-flight
//! packet, and share verification is an MSM over `threshold` commitments. Smaller
//! parameter sweeps are gated behind the `all-benchmarks` feature.
//!
//! ## Why several groups reuse one reconstructor
//!
//! Installing a production commitment costs a `new_ssa_commitment` (seconds) plus
//! `polys × threshold` point decompressions. Paying that per criterion iteration would
//! mean minutes of untimed setup per sample. The acknowledgement groups therefore build
//! **one** reconstructor and drive it forward across iterations with `iter_custom`,
//! inserting fresh shares untimed before each timed batch. That is also closer to
//! production, where a single reconstructor absorbs a whole cycle's worth of shares.

#[path = "../tests/common.rs"]
mod common;

use std::time::{Duration, Instant};

use common::TestSpec;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{
    CONSTANT_TERM_COEFFICIENT, CoefficientIndex, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, EntryShareGenerator,
    ExitAcknowledgementShareProcessor, MAX_DEFERRED_ACKS_PER_CYCLE, MAX_DEFERRED_ACKS_PER_POLYNOMIAL, PixGroupRepr,
    PixParams, PolynomialIndex, ShareResolution, SsaCommitmentProof, SsaGeneratorConfig, SsaId, SsaIndex,
    SsaPartCommitment, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator, TaggedEncryptedPartialSsaShare,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::{Acknowledgement, VerifiedAcknowledgement},
};

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

/// Coefficient commitments carried by one `SsaCommit` message.
///
/// Mirrors `MIN_COMMITMENTS_PER_SSA_COMMIT_MSG` in `transport/session/src/manager.rs`:
/// `ApplicationData::PAYLOAD_SIZE` minus the fixed prefix and a CBOR session-id allowance,
/// divided by `size_of::<PolynomialIndex>() + size_of::<PixGroupRepr>()`. Hard-coded
/// because both `hopr-protocol-app` and `hopr-transport-session` sit above this crate.
const COMMITMENTS_PER_SSA_COMMIT_MSG: usize = 28;

/// Bytes of Session quota that one share corresponds to.
///
/// One SURB carries exactly one PIX share, so one verified share is one delivered packet's
/// worth of quota. Mirrors `HoprPacket::PAYLOAD_SIZE`; hard-coded because
/// `hopr-crypto-packet` sits above this crate.
const QUOTA_BYTES_PER_SHARE: u64 = 1038;

/// Acknowledgements in one realistic `acknowledge_shares` call.
///
/// An acknowledgement packet holds at most `MAX_ACKNOWLEDGEMENTS_BATCH_SIZE`
/// (`protocols/hopr/src/codec/encoder.rs`) acknowledgements, and the Exit ack pipeline
/// calls `acknowledge_shares` once per received packet, so this — not `threshold` — is the
/// production call shape.
const ACK_BATCH_SIZES: [usize; 2] = [1, 10];

/// Concurrency levels for the concurrent sustained-rate group.
///
/// 1 is the control (identical shape to the sequential group); 10 mirrors
/// `DEFAULT_ACK_INPUT_CONCURRENCY`, the width the Exit ack pipeline actually runs at. The third
/// point is one caller per core, which bounds what raising that config value could buy —
/// `verify_completed_share`'s internal rayon means the two do not simply multiply.
fn ack_concurrency() -> Vec<usize> {
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
    let mut levels = vec![1usize, 10];
    if cores > 10 {
        levels.push(cores);
    }
    levels
}

/// Batch size for the sustained-rate group.
///
/// Large enough that the per-call overhead of `acknowledge_shares` is amortised and the
/// result reflects the steady-state cost of absorbing shares, which is what decides the
/// achievable per-Session return-path throughput.
const SUSTAINED_BATCH: usize = 256;

/// Number of acknowledgements deferred in the deferred-ack groups.
///
/// Kept well below `MAX_DEFERRED_ACKS_PER_POLYNOMIAL × polys` so the cap is never the thing
/// being measured.
const DEFERRED_ACKS: usize = 1024;

/// Awaited encrypted shares pre-loaded before measuring `insert_encrypted_share`.
///
/// Enough that the moka cache the insert touches is populated rather than empty. The insert
/// itself is O(1), so a deeper cache does not change the figure.
const AWAITING_ACKS_PRE_FILL: usize = 10_000;

/// Polynomial count used to source the pre-fill shares.
///
/// Only needs to cover [`AWAITING_ACKS_PRE_FILL`] shares at `threshold + surplus` each.
const PRE_FILL_POLYS: u16 = 256;

/// Polynomial count for the acknowledgement groups.
///
/// Installing verifiers costs `polys × threshold` commitment decodes, so a production-width
/// fixture is over a minute of untimed setup *per benchmark id*. The measured
/// per-acknowledgement cost is dominated by the `threshold`-term MSM and is insensitive to the
/// verifier-cache size, so a narrower fixture reports the same number. Production width is
/// still measured for the sustained-rate group under `all-benchmarks`.
const ACK_BENCH_POLYS: u16 = 512;

/// Polynomial count for the deferred-drain group.
///
/// Narrower than production on purpose: the cost isolated there is per deferred
/// acknowledgement, not per commitment, so it does not depend on how many polynomials the
/// cycle has — and the whole fixture is rebuilt per iteration.
const DEFERRED_DRAIN_POLYS: u16 = 512;

/// Negotiated dimensions for a benched cycle, at the shipping surplus.
///
/// Takes `usize` because every caller here already carries the dimensions that way; the narrowing
/// is checked by `try_new` rather than assumed.
fn bench_params(polys: usize, threshold: usize) -> PixParams {
    PixParams::try_new(
        polys.try_into().expect("bench polys must fit u16"),
        threshold.try_into().expect("bench threshold must fit u8"),
        SsaGeneratorConfig::default().surplus_shares,
    )
    .expect("bench dimensions must be in range")
}

/// Reconstructor configuration for benchmarks.
///
/// The expiry windows are stretched far beyond their defaults on purpose. At production
/// dimensions a single criterion iteration can take seconds, and the default
/// `max_ack_await_time` of 30 s would let awaited shares (and deferred acknowledgement
/// buckets) expire *during* a run — the measured call would then do nothing and the
/// benchmark would silently report the cost of an empty code path.
fn bench_recon_cfg(use_batch_verification: bool) -> SsaReconstructorConfig {
    SsaReconstructorConfig {
        use_batch_verification,
        max_ack_await_time: Duration::from_secs(3600),
        incomplete_commitment_lifetime: Duration::from_secs(3600),
        unused_verifier_lifetime: Duration::from_secs(3600),
        ..Default::default()
    }
}

fn gen_cfg(polys: u16, threshold: u8) -> SsaGeneratorConfig {
    SsaGeneratorConfig {
        threshold,
        polynomials_per_ssa: polys,
        ..Default::default()
    }
}

/// One `SsaCommit` message's worth of commitments: a coefficient index and a slice of
/// `(polynomial index, commitment)` pairs.
type CommitMessage<'a> = (CoefficientIndex, &'a [(PolynomialIndex, PixGroupRepr<TestSpec>)]);

/// A cycle's whole commitment set: one constant term per polynomial, sorted by polynomial index.
///
/// PIX commits to nothing else — the non-constant coefficient commitments were dropped, taking
/// `polys × (threshold - 1)` decodes per cycle with them.
type ConstantTerms = Vec<(PolynomialIndex, PixGroupRepr<TestSpec>)>;

/// A generated cycle: the generator holding the polynomials, the pseudonym it belongs to, the
/// commitments the Exit has to be fed, and the proof of knowledge that accompanies them —
/// without which the Exit refuses the cycle.
type GeneratedCycle = (
    SsaShareGenerator<TestSpec>,
    SimplePseudonym,
    ConstantTerms,
    SsaCommitmentProof<TestSpec>,
);

/// Chunks a cycle's commitments into the sequence of `SsaCommit` messages the Exit receives.
///
/// Replicates `SsaClientCommitmentMessage::new_multiple` (`protocols/start/src/lib.rs`), which
/// cannot be called from here because `hopr-protocol-start` depends on this crate. That is now a
/// single pass over the constant terms; the block-major second phase it used to have has nothing
/// left to emit.
///
/// `constant_terms` must already be sorted by polynomial index.
fn wire_order(constant_terms: &[(PolynomialIndex, PixGroupRepr<TestSpec>)]) -> Vec<CommitMessage<'_>> {
    constant_terms
        .chunks(COMMITMENTS_PER_SSA_COMMIT_MSG)
        .map(|chunk| (CONSTANT_TERM_COEFFICIENT, chunk))
        .collect()
}

/// Generates a cycle's commitments once, sorted by polynomial index so they can be handed to
/// [`wire_order`].
///
/// Returns the pseudonym it was generated for, the generator (which now holds the
/// polynomials and can produce shares), and one constant-term commitment per polynomial.
fn generate_commitment_matrix(polys: u16, threshold: u8) -> GeneratedCycle {
    let generator = SsaShareGenerator::<TestSpec>::new(gen_cfg(polys, threshold));
    let pseudonym = SimplePseudonym::random();
    let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap();
    let proof = commitment.commitment_proof;

    let mut constant_terms = commitment
        .verifiers
        .get(&CONSTANT_TERM_COEFFICIENT)
        .cloned()
        .unwrap_or_default();
    constant_terms.sort_unstable_by_key(|(poly_index, _)| *poly_index);
    assert_eq!(polys as usize, constant_terms.len());

    (generator, pseudonym, constant_terms, proof)
}

/// Feeds a whole cycle's commitments into a reconstructor in wire order.
fn install_commitment(
    reconstructor: &SsaReconstructor<TestSpec>,
    ssa_id: SsaId<SimplePseudonym>,
    constant_terms: &[(PolynomialIndex, PixGroupRepr<TestSpec>)],
    proof: SsaCommitmentProof<TestSpec>,
) {
    for (coeff_index, chunk) in wire_order(constant_terms) {
        reconstructor
            .insert_coefficient_commitments(ssa_id, coeff_index, Some(proof), chunk.iter().copied())
            .unwrap();
    }
}

/// Produces `count` shares, caches them as awaited encrypted shares, and returns the
/// acknowledgements that would redeem them.
///
/// `counter` is threaded through so every `msg` stays unique for the pseudonym, which the
/// generator requires.
fn stage_shares(
    reconstructor: &SsaReconstructor<TestSpec>,
    generator: &SsaShareGenerator<TestSpec>,
    peer: &OffchainKeypair,
    pseudonym: SimplePseudonym,
    counter: &mut u64,
    count: usize,
) -> Vec<Acknowledgement> {
    let mut acks = Vec::with_capacity(count);
    for _ in 0..count {
        let msg = counter.to_be_bytes();
        *counter += 1;
        let share = generator
            .next_share(&pseudonym, &msg)
            .unwrap()
            .expect("generator must not be exhausted inside a benchmark");
        let ack = HalfKey::random();
        let ack_challenge = ack.to_challenge().unwrap();
        let enc_share = share.share.encrypt(&share.id, &ack).unwrap();
        reconstructor
            .insert_encrypted_share(
                peer.public(),
                ack_challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc_share).unwrap(),
            )
            .unwrap();
        acks.push(VerifiedAcknowledgement::new(ack, peer).leak());
    }
    acks
}

fn bench_decode_commitment(c: &mut Criterion) {
    // Isolates the per-commitment cost that dominates `insert_coefficient_commitments`:
    // decompressing a serialized group element and rejecting points outside the prime-order
    // subgroup. There are `polys × threshold` of these per cycle — over half a million — so
    // this is the figure to attribute before trying to make commitment ingest cheaper.
    let mut group = c.benchmark_group("SsaPartCommitment::decode_commitment");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(Duration::from_secs(5));

    // A small matrix is plenty: the cost is per element and independent of how many there are.
    let (_generator, _pseudonym, constant_terms, _proof) = generate_commitment_matrix(PRE_FILL_POLYS, PROD_THRESHOLD);
    let reprs: Vec<PixGroupRepr<TestSpec>> = constant_terms.iter().map(|(_, repr)| *repr).collect();
    assert!(!reprs.is_empty(), "the cycle must contain constant terms");

    group.bench_function("single_commitment", |b| {
        let mut i = 0usize;
        b.iter(|| {
            // Cycle through distinct commitments so the measurement cannot benefit from
            // repeatedly decoding one cached value.
            let repr = &reprs[i % reprs.len()];
            i += 1;
            SsaPartCommitment::<TestSpec>::decode_commitment(repr).unwrap()
        });
    });
    group.finish();
}

fn bench_new_exit_commitment(c: &mut Criterion) {
    // `new_exit_commitment` performs a constant amount of work regardless of the threshold
    // and polynomials-per-SSA: an input range check, one random scalar, one scalar
    // multiplication, and an O(1) `SsaCommitmentBuilder` construction (the parameters are
    // merely stored). Sweeping those parameters would measure the same thing repeatedly, so
    // a single representative (production-default) case is used.
    let mut group = c.benchmark_group("SsaReconstructor::new_exit_commitment");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    let reconstructor = SsaReconstructor::<TestSpec>::new(bench_recon_cfg(true));
    let (polys, threshold) = (PROD_POLYS_PER_SSA as usize, PROD_THRESHOLD as usize);

    group.bench_function(BenchmarkId::from_parameter(format!("t{threshold}_p{polys}")), |b| {
        b.iter_batched(
            || SsaId::new(SimplePseudonym::random(), SsaIndex::MIN),
            |ssa_id| {
                reconstructor
                    .new_exit_commitment(ssa_id, bench_params(polys, threshold))
                    .unwrap();
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// Parameter points for the commitment-ingest group.
///
/// A whole cycle is now only `polys` commitments, so the production width is cheap enough to
/// always measure — it used to be `polys × threshold`, which at ~150 µs each ran for a quarter
/// of an hour and had to be narrowed.
fn whole_cycle_points() -> Vec<(u16, u8)> {
    let mut points = vec![(PROD_POLYS_PER_SSA, PROD_THRESHOLD)];
    if cfg!(feature = "all-benchmarks") {
        points.extend([(128u16, PROD_THRESHOLD), (512, PROD_THRESHOLD), (2048, PROD_THRESHOLD)]);
    }
    points
}

fn bench_insert_coefficient_commitments(c: &mut Criterion) {
    // Inserts a whole cycle's commitments in the order the Exit receives them: one constant
    // term per polynomial, chunked into packet-sized messages. Completing the set is what lets
    // the Exit derive the SSA commitment, publish the part accumulator and install every part
    // builder — all on the final message.
    //
    // Reported per commitment, so dividing by `COMMITMENTS_PER_SSA_COMMIT_MSG` gives the cost of
    // handling one `SsaCommit` packet, and the group total is the whole-cycle cost.
    let mut group = c.benchmark_group("SsaReconstructor::insert_coefficient_commitments");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for (polys, threshold) in whole_cycle_points() {
        let (_generator, pseudonym, constant_terms, proof) = generate_commitment_matrix(polys, threshold);

        // One "element" is one coefficient commitment.
        group.throughput(Throughput::Elements(polys as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("t{threshold}_p{polys}")),
            &(polys, threshold),
            |b, _| {
                b.iter_batched(
                    || {
                        let reconstructor = SsaReconstructor::<TestSpec>::new(bench_recon_cfg(true));
                        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
                        reconstructor
                            .new_exit_commitment(ssa_id, bench_params(polys as usize, threshold as usize))
                            .unwrap();
                        (reconstructor, ssa_id)
                    },
                    |(reconstructor, ssa_id)| {
                        install_commitment(&reconstructor, ssa_id, &constant_terms, proof);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_insert_encrypted_share(c: &mut Criterion) {
    // Measures a single `insert_encrypted_share` call: caching one already-encrypted, tagged
    // partial share under its `(peer, ack_challenge)` key while it awaits acknowledgement.
    //
    // No commitment is installed: this path only touches the `awaiting_acks` cache and never
    // consults a verifier. The cache is pre-filled instead, so the measured insert contends
    // with a populated moka cache rather than an empty one.
    let mut group = c.benchmark_group("SsaReconstructor::insert_encrypted_share");
    group.throughput(Throughput::Elements(1));

    let reconstructor = SsaReconstructor::<TestSpec>::new(bench_recon_cfg(true));
    let peer = OffchainKeypair::random();
    let (generator, pseudonym, _matrix, _proof) = generate_commitment_matrix(PRE_FILL_POLYS, PROD_THRESHOLD);

    let mut counter: u64 = 0;
    let _prefill = stage_shares(
        &reconstructor,
        &generator,
        &peer,
        pseudonym,
        &mut counter,
        AWAITING_ACKS_PRE_FILL,
    );

    // Pre-generate a real encrypted share outside the benchmark loop.
    let ack = HalfKey::random();
    let ack_challenge = ack.to_challenge().unwrap();
    let msg = b"benchmark_msg";
    let share = generator.next_share(&pseudonym, msg).unwrap().unwrap();
    let enc_share = share.share.encrypt(&share.id, &ack).unwrap();
    let tagged_share = TaggedEncryptedPartialSsaShare::new(pseudonym, msg, enc_share).unwrap();

    group.bench_function("single_share", |b| {
        b.iter(|| {
            reconstructor
                .insert_encrypted_share(peer.public(), ack_challenge, tagged_share)
                .unwrap();
        });
    });
    group.finish();
}

/// Drives `acknowledge_shares` against a long-lived reconstructor with `polys` verifiers
/// installed.
///
/// Each criterion sample stages `batch` fresh shares untimed, then times exactly the
/// `acknowledge_shares` call. Because the generator hands out a polynomial's shares
/// consecutively, a part is reconstructed roughly every `threshold + surplus` shares, so
/// the measured average legitimately includes the Lagrange combines that production also
/// pays.
///
/// `polys` defaults to [`ACK_BENCH_POLYS`] rather than the production width because
/// installing the verifiers costs `polys × threshold` commitment decodes — over a minute at
/// production width, paid once per benchmark id. The per-acknowledgement cost is dominated by
/// the `threshold`-term MSM in `verify_completed_share` and is not sensitive to how many
/// verifiers sit in the (O(1)-lookup) cache, so the narrower fixture reports the same figure.
fn bench_acknowledge_batch(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    id: BenchmarkId,
    polys: u16,
    batch: usize,
    use_batch_verification: bool,
) {
    let peer = OffchainKeypair::random();
    let reconstructor = SsaReconstructor::<TestSpec>::new(bench_recon_cfg(use_batch_verification));

    // One commitment yields `polys * (threshold + surplus)` shares. How many a run consumes
    // depends on the measured cost, so the budget has to be topped up rather than assumed:
    // a fresh cycle is generated and installed *outside* the timed section, exactly as the
    // Exit does between cycles in production.
    let shares_per_cycle =
        polys as usize * (PROD_THRESHOLD as usize + SsaGeneratorConfig::default().surplus_shares as usize);
    let new_cycle = || {
        let (generator, pseudonym, constant_terms, proof) = generate_commitment_matrix(polys, PROD_THRESHOLD);
        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
        reconstructor
            .new_exit_commitment(ssa_id, bench_params(polys as usize, PROD_THRESHOLD as usize))
            .unwrap();
        install_commitment(&reconstructor, ssa_id, &constant_terms, proof);
        (generator, pseudonym)
    };

    group.bench_with_input(id, &batch, |b, &batch| {
        let (mut generator, mut pseudonym) = new_cycle();
        let mut remaining = shares_per_cycle;
        let mut counter: u64 = 0;
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                if remaining < batch {
                    let (g, p) = new_cycle();
                    generator = g;
                    pseudonym = p;
                    remaining = shares_per_cycle;
                    counter = 0;
                }

                let acks = stage_shares(&reconstructor, &generator, &peer, pseudonym, &mut counter, batch);
                remaining -= batch;

                let start = Instant::now();
                let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                total += start.elapsed();

                // A resolution set containing an invalid share would mean the benchmark is
                // measuring the error path rather than real reconstruction work.
                assert!(
                    !resolutions
                        .iter()
                        .any(|r| matches!(r, ShareResolution::InvalidShares { .. })),
                    "shares must verify"
                );
            }
            total
        });
    });
}

fn bench_acknowledge_shares(c: &mut Criterion) {
    // Per-call cost at the production call shape: one `acknowledge_shares` per received
    // acknowledgement packet, at production verifier/awaited-share occupancy.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/per_call");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for batch in ACK_BATCH_SIZES {
        for (mode, use_batch_verification) in [("per_ack", false), ("batch", true)] {
            group.throughput(Throughput::Elements(batch as u64));
            bench_acknowledge_batch(
                &mut group,
                BenchmarkId::from_parameter(format!("n{batch}/{mode}")),
                ACK_BENCH_POLYS,
                batch,
                use_batch_verification,
            );
        }
    }
    group.finish();
}

fn bench_acknowledge_shares_sustained(c: &mut Criterion) {
    // Sustained rate: how fast the Exit can absorb shares in steady state. Reported in
    // bytes of Session quota (one share == one delivered packet), so the result reads
    // directly as the per-Session return-path ceiling imposed by PIX.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/sustained_quota_rate");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);
    group.throughput(Throughput::Bytes(SUSTAINED_BATCH as u64 * QUOTA_BYTES_PER_SHARE));

    let widths: &[u16] = if cfg!(feature = "all-benchmarks") {
        &[ACK_BENCH_POLYS, PROD_POLYS_PER_SSA]
    } else {
        &[ACK_BENCH_POLYS]
    };

    for &polys in widths {
        for (mode, use_batch_verification) in [("per_ack", false), ("batch", true)] {
            bench_acknowledge_batch(
                &mut group,
                BenchmarkId::from_parameter(format!("p{polys}/{mode}")),
                polys,
                SUSTAINED_BATCH,
                use_batch_verification,
            );
        }
    }
    group.finish();
}

fn bench_acknowledge_shares_concurrent(c: &mut Criterion) {
    // The sequential sustained-rate group above measures one `acknowledge_shares` at a time,
    // but the Exit pipeline runs `for_each_concurrent(ack_input_concurrency)` around
    // `spawn_fifo_blocking` (`transport/hopr/src/protocol/pipeline/mod.rs`). That distinction
    // decides the per-Exit Session capacity: `verify_completed_share` parallelises *within* one
    // share's MSM via rayon, so if a single call already saturates the machine, concurrency buys
    // nothing and the sequential figure is the machine ceiling. If it does not, aggregate
    // throughput scales and the ceiling is several times higher.
    //
    // Reported in aggregate bytes of Session quota, so the ids are directly comparable to the
    // sequential group: flat throughput means saturated, rising throughput means it was not.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/concurrent_quota_rate");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let polys = ACK_BENCH_POLYS;
    let shares_per_cycle =
        polys as usize * (PROD_THRESHOLD as usize + SsaGeneratorConfig::default().surplus_shares as usize);

    for concurrency in ack_concurrency() {
        group.throughput(Throughput::Bytes(
            (concurrency * SUSTAINED_BATCH) as u64 * QUOTA_BYTES_PER_SHARE,
        ));

        let peer = OffchainKeypair::random();
        let reconstructor = SsaReconstructor::<TestSpec>::new(bench_recon_cfg(true));
        let new_cycle = || {
            let (generator, pseudonym, constant_terms, proof) = generate_commitment_matrix(polys, PROD_THRESHOLD);
            let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
            reconstructor
                .new_exit_commitment(ssa_id, bench_params(polys as usize, PROD_THRESHOLD as usize))
                .unwrap();
            install_commitment(&reconstructor, ssa_id, &constant_terms, proof);
            (generator, pseudonym)
        };

        group.bench_function(BenchmarkId::from_parameter(format!("c{concurrency}")), |b| {
            let (mut generator, mut pseudonym) = new_cycle();
            let mut remaining = shares_per_cycle;
            let mut counter: u64 = 0;
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let needed = concurrency * SUSTAINED_BATCH;
                    if remaining < needed {
                        let (g, p) = new_cycle();
                        generator = g;
                        pseudonym = p;
                        remaining = shares_per_cycle;
                        counter = 0;
                    }

                    // Stage every thread's batch untimed, so the timed region is exactly the
                    // concurrent `acknowledge_shares` wave.
                    let batches: Vec<_> = (0..concurrency)
                        .map(|_| {
                            stage_shares(
                                &reconstructor,
                                &generator,
                                &peer,
                                pseudonym,
                                &mut counter,
                                SUSTAINED_BATCH,
                            )
                        })
                        .collect();
                    remaining -= needed;

                    let recon = &reconstructor;
                    let peer_ref = &peer;
                    let start = Instant::now();
                    std::thread::scope(|scope| {
                        for acks in batches {
                            scope.spawn(move || {
                                recon.acknowledge_shares(*peer_ref.public(), acks).unwrap();
                            });
                        }
                    });
                    total += start.elapsed();
                }
                total
            });
        });
    }
    group.finish();
}

fn bench_acknowledge_shares_deferred(c: &mut Criterion) {
    // Acknowledgements that arrive before their polynomial's verifier is installed. Only
    // the constant terms are inserted, so the part accumulator exists but no verifier
    // does, and every acknowledgement takes the `VerifierNotReady` -> `defer_ack` path.
    // This is the hot path during the commitment window of every cycle.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/deferred");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);
    group.throughput(Throughput::Elements(ACK_BATCH_SIZES[ACK_BATCH_SIZES.len() - 1] as u64));

    let batch = ACK_BATCH_SIZES[ACK_BATCH_SIZES.len() - 1];
    let peer = OffchainKeypair::random();
    // Bounded awaited-share cache: deferred shares are never redeemed here, so with the
    // production 1 000 000 they would accumulate for the whole run. This still leaves the
    // cache far larger than the number of shares any single sample stages, so the moka
    // occupancy the measured call contends with stays realistic.
    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        max_awaiting_acks: 100_000,
        ..bench_recon_cfg(true)
    });
    let (generator, pseudonym, _constant_terms, _proof) =
        generate_commitment_matrix(PROD_POLYS_PER_SSA, PROD_THRESHOLD);
    let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
    // Register the Exit side but install *no* commitment: the constant-term set stays incomplete, so
    // no cycle is published and every acknowledgement below has to defer. Installing it — as this
    // benchmark used to — makes the first cycle's shares resolve normally instead, which is the path
    // the sustained group already measures. That went unnoticed because a resolving share below its
    // polynomial's threshold produced no resolution at all, so the `is_empty` guard below held for
    // the wrong reason.
    reconstructor
        .new_exit_commitment(
            ssa_id,
            bench_params(PROD_POLYS_PER_SSA as usize, PROD_THRESHOLD as usize),
        )
        .unwrap();

    let shares_per_commitment =
        PROD_POLYS_PER_SSA as usize * (PROD_THRESHOLD as usize + SsaGeneratorConfig::default().surplus_shares as usize);

    // How many acknowledgements one deferral bucket absorbs before the scenario rotates onto a fresh
    // one.
    //
    // `defer_ack` buckets by `SsaId` and drops anything past `MAX_DEFERRED_ACKS_PER_CYCLE`, logging
    // a warning — a far cheaper path than a real deferral. This group used to rotate its bucket only
    // when the *generator's* share budget ran dry, which is `shares_per_commitment` = 688 128
    // acknowledgements at these dimensions, so the cap was reached 84× earlier and every iteration
    // after the first ~8 192 measured the drop branch. The `is_empty` guard below could not catch it:
    // a dropped acknowledgement produces no resolution either, which is the same shape of false
    // reassurance the comment above records having already been caught once here.
    //
    // Half the cap, so a full batch always fits underneath it. The per-polynomial cap cannot bind
    // first: emission is round-robin over `SHARE_EMISSION_WINDOW` polynomials at
    // `threshold + surplus` shares each, so no single polynomial accumulates more than 84 of these.
    const ROTATE_BUCKET_AT: usize = MAX_DEFERRED_ACKS_PER_CYCLE / 2;
    assert!(
        batch <= ROTATE_BUCKET_AT && ROTATE_BUCKET_AT + batch <= MAX_DEFERRED_ACKS_PER_CYCLE,
        "a batch must fit inside the deferral budget, or this group measures the drop path"
    );
    assert!(
        PROD_THRESHOLD as usize + SsaGeneratorConfig::default().surplus_shares as usize
            <= MAX_DEFERRED_ACKS_PER_POLYNOMIAL,
        "a conforming Entry's per-polynomial share budget must fit under the per-polynomial cap"
    );

    group.bench_function(BenchmarkId::from_parameter(format!("n{batch}")), |b| {
        let mut counter: u64 = 0;
        let mut pseudonym = pseudonym;
        let mut remaining = shares_per_commitment;
        let mut deferred_in_bucket = 0usize;
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                // Rotate onto a fresh pseudonym — and therefore a fresh `SsaId`, and a fresh
                // deferral bucket — before the current one reaches the cap. Rotating the pseudonym
                // rather than the SSA index is what actually moves the bucket: `new_ssa_commitment`
                // *appends* to the pseudonym's polynomial queue, so shares keep coming from the old
                // cycle until it is drained, and the `SsaId` they carry would not change.
                //
                // Both sides are set up outside the timed section. The Exit commitment is registered
                // but never completed, so no cycle is published and every acknowledgement defers.
                if deferred_in_bucket + batch > ROTATE_BUCKET_AT || remaining < batch {
                    pseudonym = SimplePseudonym::random();
                    generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap();
                    reconstructor
                        .new_exit_commitment(
                            SsaId::new(pseudonym, SsaIndex::MIN),
                            bench_params(PROD_POLYS_PER_SSA as usize, PROD_THRESHOLD as usize),
                        )
                        .unwrap();
                    remaining = shares_per_commitment;
                    deferred_in_bucket = 0;
                }

                let acks = stage_shares(&reconstructor, &generator, &peer, pseudonym, &mut counter, batch);
                remaining -= batch;

                let start = Instant::now();
                let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                total += start.elapsed();
                deferred_in_bucket += batch;

                // Nothing can resolve while the verifiers are missing; a non-empty result
                // would mean the scenario is not actually exercising the deferral path.
                assert!(resolutions.is_empty(), "no share can resolve without a verifier");
            }
            total
        });
    });
    group.finish();
}

fn bench_drain_deferred_acks(c: &mut Criterion) {
    // The other half of the deferral path: the commitment insertion that installs the
    // verifiers also drains the buckets that were waiting on them, verifying those shares
    // on the commitment path rather than the acknowledgement path. Measured as the extra
    // cost on top of an identical insertion with no deferred acknowledgements, so the two
    // ids in this group are directly comparable.
    let mut group = c.benchmark_group("SsaReconstructor::insert_coefficient_commitments/deferred_drain");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);
    group.throughput(Throughput::Elements(DEFERRED_ACKS as u64));

    let peer = OffchainKeypair::random();
    let polys = DEFERRED_DRAIN_POLYS as usize;

    for (label, deferred) in [("with_deferred", DEFERRED_ACKS), ("baseline", 0)] {
        group.bench_function(BenchmarkId::from_parameter(label), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    // The whole fixture is rebuilt per iteration. Installing part builders and
                    // draining buckets are both one-shot, so reconstructor state cannot be
                    // reused; and the staged shares must belong to the same cycle as the
                    // commitments being installed, so the generator cannot be reused either.
                    // At this width that costs a fraction of the timed section.
                    let (generator, pseudonym, constant_terms, proof) =
                        generate_commitment_matrix(DEFERRED_DRAIN_POLYS, PROD_THRESHOLD);
                    let reconstructor = SsaReconstructor::<TestSpec>::new(bench_recon_cfg(true));
                    let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
                    reconstructor
                        .new_exit_commitment(ssa_id, bench_params(polys, PROD_THRESHOLD as usize))
                        .unwrap();

                    // Everything but the final message, so the cycle is still incomplete and no
                    // part builder exists — every share staged below has to be deferred.
                    let msgs = wire_order(&constant_terms);
                    let (last, leading) = msgs.split_last().expect("at least one commit message");
                    for (coeff_index, chunk) in leading {
                        reconstructor
                            .insert_coefficient_commitments(ssa_id, *coeff_index, Some(proof), chunk.iter().copied())
                            .unwrap();
                    }

                    if deferred > 0 {
                        let mut counter: u64 = 0;
                        let acks = stage_shares(&reconstructor, &generator, &peer, pseudonym, &mut counter, deferred);
                        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                        assert!(resolutions.is_empty(), "acks must have been deferred");
                    }

                    // Time only the closing message: that is where every part builder gets
                    // installed and the deferred buckets get drained.
                    let start = Instant::now();
                    reconstructor
                        .insert_coefficient_commitments(ssa_id, last.0, Some(proof), last.1.iter().copied())
                        .unwrap();
                    total += start.elapsed();
                }
                total
            });
        });
    }
    group.finish();
}

fn bench_acknowledge_shares_full_ssa(c: &mut Criterion) {
    // Recovers an *entire* SSA, so this is the only group that exercises the final
    // reconstruction path (`ShareResolution::RecoveredSsa`, `scalar_to_private_key`, and
    // the full-commitment check).
    //
    // `surplus_shares = 0` makes the generator emit exactly `threshold` shares per
    // polynomial, so `polys * threshold` acknowledgements recover the full SSA with no
    // redundant work. The dimensions are kept small because a production-width full
    // recovery is hundreds of seconds per iteration — the production number is obtained by
    // scaling the sustained-rate group instead.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/full_ssa");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let peer = OffchainKeypair::random();
    let recon_cfg = bench_recon_cfg(false);
    for (polys, threshold) in [(4u16, 10u8), (16, 10), (4, PROD_THRESHOLD)] {
        let num_shares = polys as usize * threshold as usize;
        let generator_cfg = SsaGeneratorConfig {
            threshold,
            polynomials_per_ssa: polys,
            surplus_shares: 0,
        };

        group.throughput(Throughput::Elements(num_shares as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("p{polys}_t{threshold}")),
            &(polys, threshold),
            |b, _| {
                let mut recovered_at_least_once = false;
                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;
                    for _ in 0..iters {
                        // A full recovery consumes the cycle, so both the reconstructor and
                        // the generator have to be rebuilt for every iteration.
                        let reconstructor = SsaReconstructor::<TestSpec>::new(recon_cfg);
                        let generator = SsaShareGenerator::<TestSpec>::new(generator_cfg);
                        let pseudonym = SimplePseudonym::random();
                        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
                        reconstructor
                            .new_exit_commitment(ssa_id, bench_params(polys as usize, threshold as usize))
                            .unwrap();
                        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap();
                        let proof = commitment.commitment_proof;
                        for (coeff_index, poly_commitments) in commitment.verifiers {
                            reconstructor
                                .insert_coefficient_commitments(
                                    ssa_id,
                                    coeff_index,
                                    (coeff_index == 0).then_some(proof),
                                    poly_commitments.into_iter(),
                                )
                                .unwrap();
                        }

                        let mut counter: u64 = 0;
                        let acks = stage_shares(&reconstructor, &generator, &peer, pseudonym, &mut counter, num_shares);

                        let start = Instant::now();
                        let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                        total += start.elapsed();

                        recovered_at_least_once |= resolutions
                            .iter()
                            .any(|r| matches!(r, ShareResolution::RecoveredSsa(_)));
                    }
                    total
                });
                assert!(
                    recovered_at_least_once,
                    "full_ssa scenario must recover a full SSA at least once"
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_decode_commitment,
    bench_new_exit_commitment,
    bench_insert_coefficient_commitments,
    bench_insert_encrypted_share,
    bench_acknowledge_shares,
    bench_acknowledge_shares_sustained,
    bench_acknowledge_shares_concurrent,
    bench_acknowledge_shares_deferred,
    bench_drain_deferred_acks,
    bench_acknowledge_shares_full_ssa
);
criterion_main!(benches);
