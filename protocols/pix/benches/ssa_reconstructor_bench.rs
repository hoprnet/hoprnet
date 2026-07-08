#[path = "../tests/common.rs"]
mod common;

use common::TestSpec;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_protocol_pix::{
    EntryShareGenerator, ExitAcknowledgementShareProcessor, SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor,
    SsaReconstructorConfig, SsaShareGenerator, TaggedEncryptedPartialSsaShare,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::{Acknowledgement, VerifiedAcknowledgement},
};

/// Sets up the full commitment chain (new_exit_commitment + insert_coefficient_commitments)
/// and generates a batch of encrypted shares with their acknowledgements.
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
    let mut group = c.benchmark_group("SsaReconstructor::new_exit_commitment");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50, 128];
    let polynomials_per_ssa = [128, 512, 1024];

    for &t in &thresholds {
        for &p in &polynomials_per_ssa {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("t{}_p{}", t, p)),
                &(t, p),
                |b, _| {
                    let cfg = SsaReconstructorConfig::default();
                    let pseudonym = SimplePseudonym::random();
                    let mut index = SsaIndex::MIN;

                    b.iter(|| {
                        let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
                        let ssa_id = SsaId::new(pseudonym, index);
                        reconstructor.new_exit_commitment(ssa_id, p, t).unwrap();
                        index = index.checked_add(1).unwrap();
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_insert_coefficient_commitments_partial(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::insert_coefficient_commitments/partial");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50];
    let polynomials_per_ssa = [128, 512];

    for &t in &thresholds {
        for &p in &polynomials_per_ssa {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("t{}_p{}", t, p)),
                &(t, p),
                |b, (t, p)| {
                    let cfg = SsaReconstructorConfig::default();
                    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                        threshold: *t,
                        polynomials_per_ssa: *p,
                        ..Default::default()
                    });
                    let pseudonym = SimplePseudonym::random();
                    let mut index = SsaIndex::MIN;

                    b.iter(|| {
                        let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
                        let ssa_id = SsaId::new(pseudonym, index);
                        reconstructor
                            .new_exit_commitment(ssa_id, *p as usize, *t as usize)
                            .unwrap();
                        let mut commitment = generator.new_ssa_commitment(&pseudonym, index).unwrap();
                        let constant_terms = commitment.verifiers.remove(&0).unwrap_or_default();
                        reconstructor
                            .insert_coefficient_commitments(ssa_id, 0, constant_terms.into_iter())
                            .unwrap();
                        index = index.checked_add(1).unwrap();
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_insert_coefficient_commitments_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::insert_coefficient_commitments/full");
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50];
    let polynomials_per_ssa = [128, 512];

    for &t in &thresholds {
        for &p in &polynomials_per_ssa {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("t{}_p{}", t, p)),
                &(t, p),
                |b, (t, p)| {
                    let cfg = SsaReconstructorConfig::default();
                    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                        threshold: *t,
                        polynomials_per_ssa: *p,
                        ..Default::default()
                    });
                    let pseudonym = SimplePseudonym::random();
                    let mut index = SsaIndex::MIN;

                    b.iter(|| {
                        let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
                        let ssa_id = SsaId::new(pseudonym, index);
                        reconstructor
                            .new_exit_commitment(ssa_id, *p as usize, *t as usize)
                            .unwrap();
                        let commitment = generator.new_ssa_commitment(&pseudonym, index).unwrap();
                        for (coeff_index, poly_commitments) in commitment.verifiers {
                            reconstructor
                                .insert_coefficient_commitments(ssa_id, coeff_index, poly_commitments.into_iter())
                                .unwrap();
                        }
                        index = index.checked_add(1).unwrap();
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_insert_encrypted_share(c: &mut Criterion) {
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

fn bench_acknowledge_shares_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/single");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50, 128];
    let polynomials_per_ssa = 512;
    let peer = OffchainKeypair::random();

    for &t in &thresholds {
        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, &t| {
            let cfg = SsaReconstructorConfig {
                use_batch_verification: false,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa,
                ..Default::default()
            });
            let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
            let mut index = SsaIndex::MIN;

            b.iter(|| {
                let acks = setup_and_generate_shares(
                    &reconstructor,
                    &generator,
                    &peer,
                    index,
                    t as usize,
                    polynomials_per_ssa as usize,
                    1,
                )
                .unwrap();
                reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                index = index.checked_add(1).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_acknowledge_shares_partial(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/partial");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50, 128];
    let polynomials_per_ssa = 512;
    let peer = OffchainKeypair::random();

    for &t in &thresholds {
        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, &t| {
            let cfg = SsaReconstructorConfig {
                use_batch_verification: false,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa,
                ..Default::default()
            });
            let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
            let mut index = SsaIndex::MIN;

            b.iter(|| {
                let acks = setup_and_generate_shares(
                    &reconstructor,
                    &generator,
                    &peer,
                    index,
                    t as usize,
                    polynomials_per_ssa as usize,
                    (t - 1) as usize,
                )
                .unwrap();
                reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                index = index.checked_add(1).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_acknowledge_shares_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/full");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50, 128];
    let polynomials_per_ssa = 512;
    let peer = OffchainKeypair::random();

    for &t in &thresholds {
        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, &t| {
            let cfg = SsaReconstructorConfig {
                use_batch_verification: false,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa,
                ..Default::default()
            });
            let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
            let mut index = SsaIndex::MIN;

            b.iter(|| {
                let acks = setup_and_generate_shares(
                    &reconstructor,
                    &generator,
                    &peer,
                    index,
                    t as usize,
                    polynomials_per_ssa as usize,
                    t as usize,
                )
                .unwrap();
                reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                index = index.checked_add(1).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_acknowledge_shares_single_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/single_batch");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50, 128];
    let polynomials_per_ssa = 512;
    let peer = OffchainKeypair::random();

    for &t in &thresholds {
        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, &t| {
            let cfg = SsaReconstructorConfig {
                use_batch_verification: true,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa,
                ..Default::default()
            });
            let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
            let mut index = SsaIndex::MIN;

            b.iter(|| {
                let acks = setup_and_generate_shares(
                    &reconstructor,
                    &generator,
                    &peer,
                    index,
                    t as usize,
                    polynomials_per_ssa as usize,
                    1,
                )
                .unwrap();
                reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                index = index.checked_add(1).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_acknowledge_shares_partial_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/partial_batch");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50, 128];
    let polynomials_per_ssa = 512;
    let peer = OffchainKeypair::random();

    for &t in &thresholds {
        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, &t| {
            let cfg = SsaReconstructorConfig {
                use_batch_verification: true,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa,
                ..Default::default()
            });
            let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
            let mut index = SsaIndex::MIN;

            b.iter(|| {
                let acks = setup_and_generate_shares(
                    &reconstructor,
                    &generator,
                    &peer,
                    index,
                    t as usize,
                    polynomials_per_ssa as usize,
                    (t - 1) as usize,
                )
                .unwrap();
                reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                index = index.checked_add(1).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_acknowledge_shares_full_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("SsaReconstructor::acknowledge_shares/full_batch");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(30);

    let thresholds = [10, 50, 128];
    let polynomials_per_ssa = 512;
    let peer = OffchainKeypair::random();

    for &t in &thresholds {
        group.bench_with_input(BenchmarkId::from_parameter(format!("t{}", t)), &t, |b, &t| {
            let cfg = SsaReconstructorConfig {
                use_batch_verification: true,
                ..Default::default()
            };
            let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
                threshold: t,
                polynomials_per_ssa,
                ..Default::default()
            });
            let reconstructor = SsaReconstructor::<TestSpec>::new(cfg);
            let mut index = SsaIndex::MIN;

            b.iter(|| {
                let acks = setup_and_generate_shares(
                    &reconstructor,
                    &generator,
                    &peer,
                    index,
                    t as usize,
                    polynomials_per_ssa as usize,
                    t as usize,
                )
                .unwrap();
                reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
                index = index.checked_add(1).unwrap();
            });
        });
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
    bench_acknowledge_shares_full,
    bench_acknowledge_shares_single_batch,
    bench_acknowledge_shares_partial_batch,
    bench_acknowledge_shares_full_batch
);
criterion_main!(benches);
