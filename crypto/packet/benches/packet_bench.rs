use std::ops::Deref;

use anyhow::anyhow;
use bimap::BiHashMap;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_packet::{prelude::*, sphinx::prelude::SimpleBiMapper};
use hopr_protocol_pix::{
    DEFAULT_POLY_THRESHOLD, EntryShareGenerator, MAX_POLYS_PER_SSA, SsaGeneratorConfig, SsaIndex, SsaShareGenerator,
};
use hopr_types::{
    crypto::prelude::*,
    crypto_random::Randomizable,
    internal::prelude::*,
    primitive::prelude::{BytesEncodable, KeyIdent},
};

// Avoid musl's default allocator due to degraded performance
//
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(feature = "allocator-mimalloc", feature = "allocator-jemalloc"))]
compile_error!("feature \"allocator-jemalloc\" and feature \"allocator-mimalloc\" cannot be enabled at the same time");
#[cfg(all(target_os = "linux", feature = "allocator-mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(target_os = "linux", feature = "allocator-jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const SAMPLE_SIZE: usize = 100_000;

/// Pairs of (hops, surb_count) to benchmark.
#[cfg(feature = "all-benchmarks")]
const PACKET_BENCHMARK: &[(usize, usize)] = &[
    (0, 0), // 0-hop 0 SURBs = used for packet acknowledgements
    (1, 1), // 1-hop 1 SURB = common GnosisVPN use-case
    (1, 2), // 1-hop 2 SURBs = GnosisVPN use-case with asymmetric traffic (non-TCP)
    (2, 1), // 2-hop 1 SURB = common GnosisVPN use-case
    (2, 2), // 2-hop 2 SURBs = GnosisVPN use-case with asymmetric traffic (non-TCP)
    (3, 1), // 3-hop 1 SURB = common GnosisVPN use-case
    (3, 2), // 3-hop 2 SURBs = GnosisVPN use-case with asymmetric traffic (non-TCP)
];
#[cfg(not(feature = "all-benchmarks"))]
const PACKET_BENCHMARK: &[(usize, usize)] = &[
    (3, 2), // 3-hop 2 SURBs = worst case
];

/// Deployed polynomial threshold. `DEFAULT_PIX_SHARES_PER_POLY`
/// (`transport/session/src/types.rs`) is an alias of [`DEFAULT_POLY_THRESHOLD`], so this is
/// the negotiated value by construction.
///
/// This is what sets the per-share cost: `next_share` is a Horner evaluation over `threshold`
/// coefficients.
const PIX_THRESHOLD: u16 = DEFAULT_POLY_THRESHOLD;

/// Whether the PIX share generator has a committed SSA.
///
/// Without a commitment, `next_share` short-circuits to `Ok(None)`
/// (`protocols/pix/src/generator.rs`) and no share is embedded into a SURB, so `PixMode::Off`
/// is the cost of the packet path for a Session that does not use PIX. `PixMode::On` pays
/// one real share evaluation per SURB created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PixMode {
    Off,
    On,
}

impl PixMode {
    const ALL: [PixMode; 2] = [PixMode::Off, PixMode::On];

    fn as_str(&self) -> &'static str {
        match self {
            PixMode::Off => "pix_off",
            PixMode::On => "pix_on",
        }
    }

    fn generator(&self) -> &'static SsaShareGenerator<HoprPixSpec> {
        match self {
            PixMode::Off => &PIX_GEN_OFF,
            PixMode::On => &PIX_GEN_ON,
        }
    }
}

lazy_static::lazy_static! {
    static ref CHAIN_KEYS: [ChainKeypair; 5] = (0..5).map(|_| ChainKeypair::random()).collect::<Vec<_>>().try_into().unwrap();
    static ref OFFCHAIN_KEYS: [OffchainKeypair; 5] = (0..5).map(|_| OffchainKeypair::random()).collect::<Vec<_>>().try_into().unwrap();
    static ref MAPPER: SimpleBiMapper::<HoprSphinxSuite, HoprSphinxHeaderSpec> = OFFCHAIN_KEYS
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from(i as u32), *k.public()))
        .collect::<BiHashMap<_, _>>()
        .into();
    static ref PSEUDONYM: HoprPseudonym = HoprPseudonym::random();
    static ref DST: Hash = Hash::default();

    /// Generator with no committed SSA, so `next_share` always short-circuits.
    static ref PIX_GEN_OFF: SsaShareGenerator<HoprPixSpec> =
        SsaShareGenerator::new(SsaGeneratorConfig::default());

    /// Generator holding a committed SSA for [`PSEUDONYM`], so every SURB created during the
    /// benchmark carries a real PIX share.
    ///
    /// `polynomials_per_ssa` is set to the maximum rather than the deployed 8192 purely to
    /// widen the share budget: a commitment yields `polys * (threshold + surplus)` shares,
    /// and the benchmark makes hundreds of thousands of SURBs. The polynomial count does not
    /// affect the per-share cost — only [`PIX_THRESHOLD`] does — so this does not change what
    /// is measured. Committing costs several seconds, paid once on first use.
    static ref PIX_GEN_ON: SsaShareGenerator<HoprPixSpec> = {
        let cfg = SsaGeneratorConfig {
            threshold: PIX_THRESHOLD,
            polynomials_per_ssa: MAX_POLYS_PER_SSA,
            ..Default::default()
        };
        let generator = SsaShareGenerator::new(cfg);
        // Explicit deref: `new_ssa_commitment` takes `&S::Pseudonym` in a generic position,
        // where the lazy_static wrapper would not deref-coerce on its own.
        generator
            .new_ssa_commitment(&*PSEUDONYM, SsaIndex::MIN)
            .expect("pix commitment must succeed");
        generator
    };
}

/// Asserts that the `pix_on` generator still has shares left.
///
/// If the budget ran out mid-run, `next_share` starts returning `Ok(None)` and `pix_on`
/// silently degrades into `pix_off` — the comparison would then report that PIX is free.
fn assert_pix_budget_remaining(context: &str) {
    let probe = hopr_types::crypto_random::random_bytes::<16>();
    assert!(
        PIX_GEN_ON
            .next_share(&*PSEUDONYM, &probe)
            .expect("probe must not error")
            .is_some(),
        "pix_on share budget exhausted during {context}; results are not comparable"
    );
}

pub fn packet_sending_bench(c: &mut Criterion) {
    assert!(
        !hopr_types::crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = OFFCHAIN_KEYS.iter().take(4).map(|k| *k.public()).collect::<Vec<_>>();

    let mut group = c.benchmark_group("packet_sending_no_precomputation");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    // A PIX share is embedded once per SURB created (`create_surb_for_path` ->
    // `next_share`), so this group — which builds the return paths inline — is where the
    // Entry-side PIX cost lands.
    for &(hops, surb_count) in PACKET_BENCHMARK {
        for pix in PixMode::ALL {
            let pix_gen = pix.generator();
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{hops}_hop_{surb_count}_surbs/{}", pix.as_str())),
                &(hops, surb_count),
                |b, &(hops, surb_count)| {
                    b.iter_batched(
                        || {
                            let forward_path = TransportPath::new(path.iter().take(hops + 1).copied()).unwrap();
                            let return_paths = (0..surb_count)
                                .map(|_| forward_path.clone().invert().unwrap())
                                .collect::<Vec<_>>();
                            let addrs = (
                                sender_chain.public().to_address(),
                                destination_chain.public().to_address(),
                            );
                            let mut payload = vec![0; HoprPacket::max_message_with_surbs(surb_count)];
                            hopr_types::crypto_random::random_fill(&mut payload);
                            (addrs, forward_path, return_paths, payload)
                        },
                        |((_sender_addr, destination_addr), forward_path, return_paths, payload)| {
                            // The number of hops for ticket creation does not matter for benchmark purposes
                            let tb = TicketBuilder::zero_hop().counterparty(destination_addr);
                            HoprPacket::into_outgoing(
                                &payload,
                                &PSEUDONYM,
                                PacketRouting::ForwardPath {
                                    forward_path,
                                    return_paths,
                                },
                                sender_chain,
                                tb,
                                MAPPER.deref(),
                                &DST,
                                pix_gen,
                                None,
                            )
                            .unwrap();
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    assert_pix_budget_remaining("packet_sending_no_precomputation");
    group.finish();

    let mut group = c.benchmark_group("packet_sending_precomputed");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    let msg = hopr_types::crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();

    // This benchmark does not depend on the number of SURBs, because they are created in the precomputation step.
    // For the same reason there is no PIX dimension here: shares ride on SURBs, `return_paths` is
    // empty, and `next_share` is therefore never reached. The PIX cost of precomputation is
    // measured in `packet_precompute_bench` instead.

    for &hops in if cfg!(feature = "all-benchmarks") {
        &[0, 1, 2, 3][..]
    } else {
        &[3][..]
    } {
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hops}_hop")), &hops, |b, &hops| {
            // The number of hops for ticket creation does not matter for benchmark purposes
            let tb = TicketBuilder::zero_hop().counterparty(destination_chain.public().to_address());
            let forward_path = TransportPath::new(path.iter().take(hops + 1).copied()).unwrap();
            let ssa_gen = PixMode::Off.generator();
            let precomputed = PartialHoprPacket::new(
                &PSEUDONYM,
                PacketRouting::ForwardPath {
                    forward_path,
                    return_paths: vec![],
                },
                sender_chain,
                tb,
                MAPPER.deref(),
                ssa_gen,
                &DST,
            )
            .unwrap();

            b.iter_batched(
                || precomputed.clone(),
                |p| p.into_hopr_packet(&msg, None).unwrap(),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

pub fn packet_precompute_bench(c: &mut Criterion) {
    assert!(
        !hopr_types::crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = OFFCHAIN_KEYS.iter().take(4).map(|k| *k.public()).collect::<Vec<_>>();

    let mut group = c.benchmark_group("packet_precompute");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(30));

    // Precomputation is where the SURBs — and therefore the PIX shares — are built, so this
    // group carries the Entry-side PIX cost for the precomputed sending path.
    for &(hops, surb_count) in PACKET_BENCHMARK {
        for pix in PixMode::ALL {
            let pix_gen = pix.generator();
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{hops}_hop_{surb_count}_surbs/{}", pix.as_str())),
                &(hops, surb_count),
                |b, &(hops, surb_count)| {
                    b.iter_batched(
                        || {
                            let forward_path = TransportPath::new(path.iter().take(hops + 1).copied()).unwrap();
                            let return_paths = (0..surb_count)
                                .map(|_| forward_path.clone().invert().unwrap())
                                .collect::<Vec<_>>();
                            let addrs = (
                                sender_chain.public().to_address(),
                                destination_chain.public().to_address(),
                            );
                            (addrs, forward_path, return_paths)
                        },
                        |((_sender_addr, destination_addr), forward_path, return_paths)| {
                            // The number of hops for ticket creation does not matter for benchmark purposes
                            let tb = TicketBuilder::zero_hop().counterparty(destination_addr);
                            PartialHoprPacket::new(
                                &PSEUDONYM,
                                PacketRouting::ForwardPath {
                                    forward_path,
                                    return_paths,
                                },
                                sender_chain,
                                tb,
                                MAPPER.deref(),
                                pix_gen,
                                &DST,
                            )
                            .unwrap();
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    assert_pix_budget_remaining("packet_precompute");
    group.finish();
}

pub fn packet_forwarding_bench(c: &mut Criterion) {
    assert!(
        !hopr_types::crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = [*OFFCHAIN_KEYS[1].public(), *OFFCHAIN_KEYS[2].public()];

    let msg = hopr_types::crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().counterparty(destination_chain.public().to_address());

    // Only needed to build the fixture packet: the measured operation (relaying) never
    // touches the generator, and the fixture carries no SURBs, so PIX cannot apply here.
    let ssa_gen = PixMode::Off.generator();

    // Sender
    let packet = HoprPacket::into_outgoing(
        &msg,
        &PSEUDONYM,
        PacketRouting::ForwardPath {
            forward_path: TransportPath::new(path.to_vec()).unwrap(),
            return_paths: vec![],
        },
        sender_chain,
        tb,
        MAPPER.deref(),
        &DST,
        ssa_gen,
        None,
    )
    .map_err(anyhow::Error::new)
    .and_then(|(packet, _)| packet.try_as_outgoing().ok_or(anyhow!("packet is not outgoing")))
    .map(|data| {
        let mut ret = Vec::with_capacity(HoprPacket::SIZE);
        ret.extend_from_slice(data.packet.as_ref());
        ret.extend_from_slice(&data.ticket.into_encoded());
        ret.into_boxed_slice()
    })
    .unwrap();

    // Benchmark the Relayer
    let mut group = c.benchmark_group("packet_forwarding");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    group.bench_function("any_hop", |b| {
        b.iter(|| {
            HoprPacket::from_incoming(
                &packet,
                &OFFCHAIN_KEYS[1],
                *OFFCHAIN_KEYS[0].public(),
                MAPPER.deref(),
                |_| None,
            )
            .unwrap();
        })
    });
}

pub fn packet_receiving_bench(c: &mut Criterion) {
    assert!(
        !hopr_types::crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = [*OFFCHAIN_KEYS[1].public(), *OFFCHAIN_KEYS[2].public()];

    let msg = hopr_types::crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().counterparty(destination_chain.public().to_address());

    // Only needed to build the fixture packet: the measured operation (receiving) never
    // touches the generator, and the fixture carries no SURBs, so PIX cannot apply here.
    let ssa_gen = PixMode::Off.generator();

    // Sender
    let forward_path = TransportPath::new(path).unwrap();
    let packet = HoprPacket::into_outgoing(
        &msg,
        &PSEUDONYM,
        PacketRouting::ForwardPath {
            forward_path,
            return_paths: vec![],
        },
        sender_chain,
        tb,
        MAPPER.deref(),
        &DST,
        ssa_gen,
        None,
    )
    .map_err(anyhow::Error::new)
    .and_then(|(packet, _)| packet.try_as_outgoing().ok_or(anyhow!("packet is not outgoing")))
    .map(|data| {
        let mut ret = Vec::with_capacity(HoprPacket::SIZE);
        ret.extend_from_slice(data.packet.as_ref());
        ret.extend_from_slice(&data.ticket.into_encoded());
        ret.into_boxed_slice()
    })
    .unwrap();

    // Relayer
    let packet = HoprPacket::from_incoming(
        &packet,
        &OFFCHAIN_KEYS[1],
        *OFFCHAIN_KEYS[0].public(),
        MAPPER.deref(),
        |_| None,
    )
    .map_err(anyhow::Error::new)
    .and_then(|packet| packet.try_as_forwarded().ok_or(anyhow!("packet is not forwarded")))
    .map(|data| {
        let mut ret = Vec::with_capacity(HoprPacket::SIZE);
        ret.extend_from_slice(data.outgoing.packet.as_ref());
        ret.extend_from_slice(&data.outgoing.ticket.into_encoded());
        ret.into_boxed_slice()
    })
    .unwrap();

    // Benchmark the Destination
    let mut group = c.benchmark_group("packet_receiving");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));
    group.bench_function("any_hop", |b| {
        b.iter(|| {
            HoprPacket::from_incoming(
                &packet,
                &OFFCHAIN_KEYS[2],
                *OFFCHAIN_KEYS[1].public(),
                MAPPER.deref(),
                |_| None,
            )
            .unwrap();
        })
    });
}

criterion_group!(
    benches,
    packet_sending_bench,
    packet_precompute_bench,
    packet_forwarding_bench,
    packet_receiving_bench
);
criterion_main!(benches);
