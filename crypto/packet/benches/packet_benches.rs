use std::ops::Deref;

use anyhow::anyhow;
use bimap::BiHashMap;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_packet::prelude::*;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{BytesEncodable, KeyIdent};

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
const PACKET_BENCHMARK: [(usize, usize); 7] = [
    (0, 0), // 0-hop 0 SURBs = used for packet acknowledgements
    (1, 1), // 1-hop 1 SURB = common GnosisVPN use-case
    (1, 2), // 1-hop 2 SURBs = GnosisVPN use-case with asymmetric traffic (non-TCP)
    (2, 1), // 2-hop 1 SURB = common GnosisVPN use-case
    (2, 2), // 2-hop 2 SURBs = GnosisVPN use-case with asymmetric traffic (non-TCP)
    (3, 1), // 3-hop 1 SURB = common GnosisVPN use-case
    (3, 2), // 3-hop 2 SURBs = GnosisVPN use-case with asymmetric traffic (non-TCP)
];

lazy_static::lazy_static! {
    static ref CHAIN_KEYS: [ChainKeypair; 5] = (0..5).map(|_| ChainKeypair::random()).collect::<Vec<_>>().try_into().unwrap();
    static ref OFFCHAIN_KEYS: [OffchainKeypair; 5] = (0..5).map(|_| OffchainKeypair::random()).collect::<Vec<_>>().try_into().unwrap();
    static ref MAPPER: bimap::BiMap<KeyIdent, OffchainPublicKey> = OFFCHAIN_KEYS
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from(i as u32), *k.public()))
        .collect::<BiHashMap<_, _>>();
    static ref PSEUDONYM: HoprPseudonym = HoprPseudonym::random();
    static ref DST: Hash = Hash::default();
}

pub fn packet_sending_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = OFFCHAIN_KEYS.iter().take(4).map(|k| *k.public()).collect::<Vec<_>>();

    let mut group = c.benchmark_group("packet_sending_no_precomputation");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    for (hops, surb_count) in PACKET_BENCHMARK {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{hops}_hop_{surb_count}_surbs")),
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
                        hopr_crypto_random::random_fill(&mut payload);
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
                            None,
                        )
                        .unwrap();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("packet_sending_precomputed");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    let msg = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();

    // This benchmark does not depend on the number of SURBs, because they are created in the precomputation step

    for hops in [0, 1, 2, 3] {
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hops}_hop")), &hops, |b, &hops| {
            // The number of hops for ticket creation does not matter for benchmark purposes
            let tb = TicketBuilder::zero_hop().counterparty(destination_chain.public().to_address());
            let forward_path = TransportPath::new(path.iter().take(hops + 1).copied()).unwrap();
            let precomputed = PartialHoprPacket::new(
                &PSEUDONYM,
                PacketRouting::ForwardPath {
                    forward_path,
                    return_paths: vec![],
                },
                sender_chain,
                tb,
                MAPPER.deref(),
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
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = OFFCHAIN_KEYS.iter().take(4).map(|k| *k.public()).collect::<Vec<_>>();

    let mut group = c.benchmark_group("packet_precompute");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(30));

    for (hops, surb_count) in PACKET_BENCHMARK {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{hops}_hop_{surb_count}_surbs")),
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
                            &DST,
                        )
                        .unwrap();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

pub fn packet_forwarding_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = [*OFFCHAIN_KEYS[1].public(), *OFFCHAIN_KEYS[2].public()];

    let msg = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().counterparty(destination_chain.public().to_address());

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
        None,
    )
    .map_err(anyhow::Error::new)
    .and_then(|(packet, _)| packet.try_as_outgoing().ok_or(anyhow!("packet is not outgoing")))
    .map(|data| {
        let mut ret = Vec::with_capacity(HoprPacket::SIZE);
        ret.extend_from_slice(data.packet.as_ref());
        ret.extend_from_slice(&data.ticket.clone().into_encoded());
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
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain = &CHAIN_KEYS[0];
    let destination_chain = &CHAIN_KEYS[4];
    let path = [*OFFCHAIN_KEYS[1].public(), *OFFCHAIN_KEYS[2].public()];

    let msg = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().counterparty(destination_chain.public().to_address());

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
        None,
    )
    .map_err(anyhow::Error::new)
    .and_then(|(packet, _)| packet.try_as_outgoing().ok_or(anyhow!("packet is not outgoing")))
    .map(|data| {
        let mut ret = Vec::with_capacity(HoprPacket::SIZE);
        ret.extend_from_slice(data.packet.as_ref());
        ret.extend_from_slice(&data.ticket.clone().into_encoded());
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
        ret.extend_from_slice(&data.outgoing.ticket.clone().into_encoded());
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
