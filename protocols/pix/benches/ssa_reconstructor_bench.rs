#[path = "../tests/common.rs"]
mod common;

use common::TestSpec;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{
    DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, EntryShareGenerator, ExitAcknowledgementShareProcessor,
    ShareResolution, SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator,
    TaggedEncryptedPartialSsaShare,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::{Acknowledgement, VerifiedAcknowledgement},
};

/// Number of polynomials used by the single-polynomial `acknowledge_shares` benchmarks
/// (`single`, `partial`, `poly_part`).
///
/// Those scenarios only ever touch polynomial index 0, so the polynomial count has **no**
/// effect on what is measured; it only influences the (untimed) setup that generates and
/// commits the SSA. Keeping it small, therefore, makes the per-iteration setup cheap without
/// changing the benchmarked operation.
const SINGLE_POLY_BENCH_POLYS: u16 = 4;

/// Sets up the full commitment chain (`new_exit_commitment` + `insert_coefficient_commitments`)
/// and generates a batch of `num_shares` encrypted shares with their acknowledgements.
///
/// Each call uses a fresh pseudonym so the generator's per-pseudonym queue stays clean.
fn setup_and_generate_shares(
    reconstructor: &SsaReconstructor<TestSpec>,
    generator: &SsaShareGenerator<TestSpec>,
    peer: &OffchainKeypair,
    ssa_index: SsaIndex,
    threshold: usize,
    polynomials_per_ssa: usize,
    num_shares: usize,
) -> anyhow::Result<Vec<Acknowledgement>> {
    let pseudonym = SimplePseudonym::random();

    let ssa_id = SsaId::new(pseudonym, ssa_index);
    reconstructor.new_exit_commitment(ssa_id, polynomials_per_ssa, threshold)?;

    let commitment = generator.new_ssa_commitment(&pseudonym, ssa_index)?;
    for (coeff_index, poly_commitments) in commitment.verifiers {
        reconstructor.insert_coefficient_commitments(ssa_id, coeff_index, poly_commitments.into_iter())?;
    }

    let mut acks = Vec::with_capacity(num_shares);
    for i in 0..num_shares {
        let msg = i.to_be_bytes();
        let share = generator
            .next_share(&pseudonym, &msg)?
            .ok_or_else(|| anyhow::anyhow!("generator exhausted after {} shares", i))?;
        let ack = HalfKey::random();
        let ack_challenge = ack.to_challenge()?;
        let enc_share = share.share.encrypt(&share.id, &ack)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            ack_challenge,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc_share)?,
        )?;
        acks.push(VerifiedAcknowledgement::new(ack, peer).leak());
    }

    Ok(acks)
}

fn bench_new_exit_commitment(c: &mut Criterion) {
    // `new_exit_commitment` performs a constant amount of work regardless of the threshold
    // and polynomials-per-SSA: an input range check, one random scalar, one scalar
    // multiplication, and an O(1) `SsaCommitmentBuilder` construction (the parameters are
    // merely stored). Sweeping those parameters would measure the same thing repeatedly, so
    // a single representative (production-default) case is used.
    //
    // The reconstructor is built once, outside the timed loop; each iteration only times the
    // method itself, and a fresh (random) SSA id is produced in the untimed setup closure.
    let mut group = c.benchmark_group("SsaReconstructor::new_exit_commitment");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let threshold = DEFAULT_POLY_THRESHOLD as usize;
    let polys = DEFAULT_POLYS_PER_SSA as usize;
    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig::default());

    group.bench_function(BenchmarkId::from_parameter(format!("t{threshold}_p{polys}")), |b| {
        b.iter_batched(
            || SsaId::new(SimplePseudonym::random(), SsaIndex::MIN),
            |ssa_id| {
                reconstructor.new_exit_commitment(ssa_id, polys, threshold).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_insert_coefficient_commitments_partial(c: &mut Criterion) {
    // Inserts only the constant terms (coefficient 0) of every polynomial, which does not
    // complete the commitment, so no verifiers are built yet. The expensive commitment
    // generation is performed in the untimed setup closure; only the insertion is timed.
    let mut group = c.benchmark_group("SsaReconstructor::insert_coefficient_commitments/partial");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10u16, 50];
    let polynomials_per_ssa = [128u16, 512];

    for &t in &thresholds {
        for &p in &polynomials_per_ssa {
            let recon_cfg = SsaReconstructorConfig::default();
            let gen_cfg = SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa: p,
                ..Default::default()
            };
            group.bench_with_input(BenchmarkId::from_parameter(format!("t{t}_p{p}")), &(t, p), |b, _| {
                b.iter_batched(
                    || {
                        let reconstructor = SsaReconstructor::<TestSpec>::new(recon_cfg);
                        let generator = SsaShareGenerator::<TestSpec>::new(gen_cfg);
                        let pseudonym = SimplePseudonym::random();
                        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
                        reconstructor
                            .new_exit_commitment(ssa_id, p as usize, t as usize)
                            .unwrap();
                        let mut commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap();
                        let constant_terms = commitment.verifiers.remove(&0).unwrap_or_default();
                        (reconstructor, ssa_id, constant_terms)
                    },
                    |(reconstructor, ssa_id, constant_terms)| {
                        reconstructor
                            .insert_coefficient_commitments(ssa_id, 0, constant_terms.into_iter())
                            .unwrap();
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

fn bench_insert_coefficient_commitments_full(c: &mut Criterion) {
    // Inserts *all* coefficient commitments, which completes the commitment and therefore
    // triggers building all polynomial verifiers (`from_serializable_commitments`) — the work
    // this benchmark is meant to measure. The commitment generation happens in the untimed
    // setup closure.
    let mut group = c.benchmark_group("SsaReconstructor::insert_coefficient_commitments/full");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10u16, 50];
    let polynomials_per_ssa = [128u16, 512];

    for &t in &thresholds {
        for &p in &polynomials_per_ssa {
            let recon_cfg = SsaReconstructorConfig::default();
            let gen_cfg = SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa: p,
                ..Default::default()
            };
            group.bench_with_input(BenchmarkId::from_parameter(format!("t{t}_p{p}")), &(t, p), |b, _| {
                b.iter_batched(
                    || {
                        let reconstructor = SsaReconstructor::<TestSpec>::new(recon_cfg);
                        let generator = SsaShareGenerator::<TestSpec>::new(gen_cfg);
                        let pseudonym = SimplePseudonym::random();
                        let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
                        reconstructor
                            .new_exit_commitment(ssa_id, p as usize, t as usize)
                            .unwrap();
                        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap();
                        (reconstructor, ssa_id, commitment.verifiers)
                    },
                    |(reconstructor, ssa_id, verifiers)| {
                        for (coeff_index, poly_commitments) in verifiers {
                            reconstructor
                                .insert_coefficient_commitments(ssa_id, coeff_index, poly_commitments.into_iter())
                                .unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

fn bench_insert_encrypted_share(c: &mut Criterion) {
    // Measures a single `insert_encrypted_share` call: caching one already-encrypted, tagged
    // partial share under its `(peer, ack_challenge)` key while it awaits acknowledgement. The
    // share is generated once, outside the timed loop, so only the insertion is measured. The
    // same key is reused on every iteration, so this reflects a hot single-slot cache update
    // rather than growing-occupancy behaviour.
    let mut group = c.benchmark_group("SsaReconstructor::insert_encrypted_share");
    group.throughput(Throughput::Elements(1));

    let cfg = SsaReconstructorConfig::default();
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        threshold: 10,
        polynomials_per_ssa: 10,
        ..Default::default()
    });
    let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
    let pseudonym = SimplePseudonym::random();
    let peer = OffchainKeypair::random();

    // Pre-generate a real encrypted share outside the benchmark loop.
    // Uses a fresh pseudonym so the generator queue is clean.
    let (tagged_share, ack_challenge) = {
        let ssa_index = SsaIndex::MIN;
        let ssa_id = SsaId::new(pseudonym, ssa_index);
        reconstructor.new_exit_commitment(ssa_id, 10, 10).unwrap();
        let commitment = generator.new_ssa_commitment(&pseudonym, ssa_index).unwrap();
        for (coeff_index, poly_commitments) in commitment.verifiers {
            reconstructor
                .insert_coefficient_commitments(ssa_id, coeff_index, poly_commitments.into_iter())
                .unwrap();
        }

        let ack = HalfKey::random();
        let ack_challenge = ack.to_challenge().unwrap();
        let msg = b"benchmark_msg";
        let share = generator.next_share(&pseudonym, msg).unwrap().unwrap();
        let enc_share = share.share.encrypt(&share.id, &ack).unwrap();
        let tagged = TaggedEncryptedPartialSsaShare::new(pseudonym, msg, enc_share).unwrap();
        (tagged, ack_challenge)
    };

    group.bench_function("single_share", |b| {
        b.iter(|| {
            reconstructor
                .insert_encrypted_share(peer.public(), ack_challenge, tagged_share)
                .unwrap();
        });
    });
    group.finish();
}

/// Benchmarks a single `acknowledge_shares` scenario.
///
/// A fresh reconstructor and a fresh set of `num_shares` inserted (and acknowledgeable) shares
/// are built in the *untimed* setup closure via [`setup_and_generate_shares`], so only the
/// `acknowledge_shares` call itself is measured. Because `acknowledge_shares` consumes the
/// awaited shares (mutating reconstructor state), the setup must produce a brand-new
/// reconstructor for every batch.
fn bench_acknowledge_case<M: criterion::measurement::Measurement>(
    group: &mut criterion::BenchmarkGroup<'_, M>,
    id: BenchmarkId,
    peer: &OffchainKeypair,
    recon_cfg: SsaReconstructorConfig,
    gen_cfg: SsaGeneratorConfig,
    num_shares: usize,
) {
    group.throughput(Throughput::Elements(num_shares as u64));
    group.bench_with_input(id, &(), |b, _| {
        b.iter_batched(
            || {
                let reconstructor = SsaReconstructor::<TestSpec>::new(recon_cfg);
                let generator = SsaShareGenerator::<TestSpec>::new(gen_cfg);
                let acks = setup_and_generate_shares(
                    &reconstructor,
                    &generator,
                    peer,
                    SsaIndex::MIN,
                    gen_cfg.threshold as usize,
                    gen_cfg.polynomials_per_ssa as usize,
                    num_shares,
                )
                .unwrap();
                (reconstructor, acks)
            },
            |(reconstructor, acks)| {
                reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_acknowledge_shares_single(c: &mut Criterion) {
    // A single acknowledgement: one cache removal + one decryption + one share verification,
    // with no Lagrange reconstruction. There is intentionally no batch variant: a single ack
    // is below `MIN_BATCH_SIZE`, so `Acknowledgement::verify_batch` transparently falls back to
    // per-ack verification and would measure this exact same path.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/single");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let peer = OffchainKeypair::random();
    let recon_cfg = SsaReconstructorConfig {
        use_batch_verification: false,
        ..Default::default()
    };
    for t in [10u16, 50, 128] {
        let gen_cfg = SsaGeneratorConfig {
            threshold: t,
            polynomials_per_ssa: SINGLE_POLY_BENCH_POLYS,
            ..Default::default()
        };
        bench_acknowledge_case(
            &mut group,
            BenchmarkId::from_parameter(format!("t{t}")),
            &peer,
            recon_cfg,
            gen_cfg,
            1,
        );
    }
    group.finish();
}

fn bench_acknowledge_shares_partial(c: &mut Criterion) {
    // `threshold - 1` acknowledgements: one short of the threshold, so the polynomial part is
    // *not* reconstructed (no Lagrange `combine`), but every ack is still decrypted and its
    // share verified. Each threshold is measured with both per-ack and batch verification so
    // the (signature-verification-only) difference between the two modes is isolated;
    // `threshold - 1 >= MIN_BATCH_SIZE` holds for every threshold used here.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/partial");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let peer = OffchainKeypair::random();
    for t in [10u16, 50, 128] {
        for (mode, use_batch_verification) in [("per_ack", false), ("batch", true)] {
            let recon_cfg = SsaReconstructorConfig {
                use_batch_verification,
                ..Default::default()
            };
            let gen_cfg = SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa: SINGLE_POLY_BENCH_POLYS,
                ..Default::default()
            };
            bench_acknowledge_case(
                &mut group,
                BenchmarkId::from_parameter(format!("t{t}/{mode}")),
                &peer,
                recon_cfg,
                gen_cfg,
                (t - 1) as usize,
            );
        }
    }
    group.finish();
}

fn bench_acknowledge_shares_poly_part(c: &mut Criterion) {
    // Exactly `threshold` acknowledgements for a single polynomial: just enough to reconstruct
    // **one** SSA part (one Lagrange `combine`). Note this does *not* recover a full SSA — that
    // requires all `polynomials_per_ssa` parts (see `acknowledge_shares/full_ssa`). Measured
    // with both verification modes.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/poly_part");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let peer = OffchainKeypair::random();
    for t in [10u16, 50, 128] {
        for (mode, use_batch_verification) in [("per_ack", false), ("batch", true)] {
            let recon_cfg = SsaReconstructorConfig {
                use_batch_verification,
                ..Default::default()
            };
            let gen_cfg = SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa: SINGLE_POLY_BENCH_POLYS,
                ..Default::default()
            };
            bench_acknowledge_case(
                &mut group,
                BenchmarkId::from_parameter(format!("t{t}/{mode}")),
                &peer,
                recon_cfg,
                gen_cfg,
                t as usize,
            );
        }
    }
    group.finish();
}

fn bench_acknowledge_shares_full_ssa(c: &mut Criterion) {
    // Recovers an *entire* SSA. Unlike the single-polynomial cases above, this drives every
    // polynomial part to completion, so it is the only `acknowledge_shares` benchmark that
    // exercises the final reconstruction path (`ShareResolution::RecoveredSsa`,
    // `scalar_to_private_key`, and the full-commitment check).
    //
    // `surplus_shares = 0` makes the generator emit exactly `threshold` shares per polynomial,
    // so `polynomials_per_ssa * threshold` acknowledgements recover the full SSA with no
    // redundant work. The parameter pairs are kept small so a full recovery stays tractable.
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/full_ssa");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let peer = OffchainKeypair::random();
    let recon_cfg = SsaReconstructorConfig {
        use_batch_verification: false,
        ..Default::default()
    };
    for (polys, t) in [(4u16, 10u16), (16, 10), (4, 50)] {
        let num_shares = polys as usize * t as usize;
        let gen_cfg = SsaGeneratorConfig {
            threshold: t,
            polynomials_per_ssa: polys,
            surplus_shares: 0,
        };

        // Sanity check (outside the timed loop) that this configuration really recovers a full
        // SSA, so the benchmark provably exercises the completion path it claims to.
        {
            let reconstructor = SsaReconstructor::<TestSpec>::new(recon_cfg);
            let generator = SsaShareGenerator::<TestSpec>::new(gen_cfg);
            let acks = setup_and_generate_shares(
                &reconstructor,
                &generator,
                &peer,
                SsaIndex::MIN,
                t as usize,
                polys as usize,
                num_shares,
            )
            .unwrap();
            let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
            assert!(
                resolutions
                    .iter()
                    .any(|r| matches!(r, ShareResolution::RecoveredSsa(_))),
                "full_ssa scenario must recover a full SSA (polys={polys}, t={t})"
            );
        }

        bench_acknowledge_case(
            &mut group,
            BenchmarkId::from_parameter(format!("p{polys}_t{t}")),
            &peer,
            recon_cfg,
            gen_cfg,
            num_shares,
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_new_exit_commitment,
    bench_insert_coefficient_commitments_partial,
    bench_insert_coefficient_commitments_full,
    bench_insert_encrypted_share,
    bench_acknowledge_shares_single,
    bench_acknowledge_shares_partial,
    bench_acknowledge_shares_poly_part,
    bench_acknowledge_shares_full_ssa
);
criterion_main!(benches);
