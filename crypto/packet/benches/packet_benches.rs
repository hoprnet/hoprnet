use bimap::BiHashMap;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_packet::prelude::*;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::{ChainKeypair, Hash, Keypair, OffchainKeypair, OffchainPublicKey};
use hopr_internal_types::prelude::*;
use hopr_path::{Path, TransportPath};
use hopr_primitive_types::prelude::{Address, BytesEncodable, KeyIdent};

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

pub fn packet_sending_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let chain_key = ChainKeypair::random();
    let destination = Address::new(&hopr_crypto_random::random_bytes::<20>());
    let path = (0..=3).map(|_| *OffchainKeypair::random().public()).collect::<Vec<_>>();
    let mapper: bimap::BiMap<KeyIdent, OffchainPublicKey> = path
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from(i as u32), *k))
        .collect::<BiHashMap<_, _>>();
    let pseudonym = HoprPseudonym::random();

    let dst = Hash::default();

    let mut group = c.benchmark_group("packet_sending");
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
                        let mut payload = vec![0; HoprPacket::max_message_with_surbs(surb_count)];
                        hopr_crypto_random::random_fill(&mut payload);
                        (forward_path, return_paths, payload)
                    },
                    |(forward_path, return_paths, payload)| {
                        // The number of hops for ticket creation does not matter for benchmark purposes
                        let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);
                        HoprPacket::into_outgoing(
                            &payload,
                            &pseudonym,
                            PacketRouting::ForwardPath {
                                forward_path,
                                return_paths,
                            },
                            &chain_key,
                            tb,
                            &mapper,
                            &dst,
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
}

pub fn packet_precompute_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let chain_key = ChainKeypair::random();
    let destination = Address::new(&hopr_crypto_random::random_bytes::<20>());
    let path = (0..=3).map(|_| *OffchainKeypair::random().public()).collect::<Vec<_>>();
    let mapper: bimap::BiMap<KeyIdent, OffchainPublicKey> = path
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from(i as u32), *k))
        .collect::<BiHashMap<_, _>>();
    let pseudonym = HoprPseudonym::random();
    let dst = Hash::default();

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
                        (forward_path, return_paths)
                    },
                    |(forward_path, return_paths)| {
                        // The number of hops for ticket creation does not matter for benchmark purposes
                        let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);
                        PartialHoprPacket::new(
                            &pseudonym,
                            PacketRouting::ForwardPath {
                                forward_path,
                                return_paths,
                            },
                            &chain_key,
                            tb,
                            &mapper,
                            &dst,
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

pub fn packet_sending_precomputed_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let chain_key = ChainKeypair::random();
    let destination = Address::new(&hopr_crypto_random::random_bytes::<20>());
    let path = (0..=3).map(|_| *OffchainKeypair::random().public()).collect::<Vec<_>>();
    let mapper: bimap::BiMap<KeyIdent, OffchainPublicKey> = path
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from(i as u32), *k))
        .collect::<BiHashMap<_, _>>();
    let pseudonym = HoprPseudonym::random();

    let msg = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();
    let dst = Hash::default();

    let mut group = c.benchmark_group("packet_sending_precomputed");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    // This benchmark does not depend on the number of SURBs, because they are created in the precomputation step

    for hops in [0, 1, 2, 3] {
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hops}_hop")), &hops, |b, &hops| {
            // The number of hops for ticket creation does not matter for benchmark purposes
            let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);
            let tp = TransportPath::new(path.iter().take(hops + 1).copied()).unwrap();
            let precomputed = PartialHoprPacket::new(
                &pseudonym,
                PacketRouting::ForwardPath {
                    forward_path: tp,
                    return_paths: vec![],
                },
                &chain_key,
                tb,
                &mapper,
                &dst,
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

pub fn packet_forwarding_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let chain_key = ChainKeypair::random();
    let destination = Address::new(&hopr_crypto_random::random_bytes::<20>());

    let sender = OffchainKeypair::random();
    let relayer = OffchainKeypair::random();
    let recipient = OffchainKeypair::random();
    let path = [*relayer.public(), *recipient.public()];
    let mapper: bimap::BiMap<KeyIdent, OffchainPublicKey> = path
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from((i + 1) as u32), *k))
        .collect::<BiHashMap<_, _>>();
    let pseudonym = HoprPseudonym::random();

    let msg = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();
    let dst = Hash::default();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);

    // Sender
    let packet = match HoprPacket::into_outgoing(
        &msg,
        &pseudonym,
        PacketRouting::ForwardPath {
            forward_path: TransportPath::new(path).unwrap(),
            return_paths: vec![],
        },
        &chain_key,
        tb,
        &mapper,
        &dst,
        None,
    )
    .unwrap()
    {
        (HoprPacket::Outgoing(out), _) => {
            let mut ret = Vec::with_capacity(HoprPacket::SIZE);
            ret.extend_from_slice(out.packet.as_ref());
            ret.extend_from_slice(&out.ticket.clone().into_encoded());
            ret.into_boxed_slice()
        }
        _ => panic!("should not happen"),
    };

    // Benchmark the relayer
    let mut group = c.benchmark_group("packet_forwarding");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    group.bench_function("any_hop", |b| {
        b.iter(|| {
            HoprPacket::from_incoming(&packet, &relayer, *sender.public(), &mapper, |_| None).unwrap();
        })
    });
}

pub fn packet_receiving_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let sender_chain_key = ChainKeypair::random();
    let recipient_chain_key = ChainKeypair::random();
    let destination = recipient_chain_key.public().to_address();

    let sender = OffchainKeypair::random();
    let relayer = OffchainKeypair::random();
    let recipient = OffchainKeypair::random();
    let path = [*relayer.public(), *recipient.public()];
    let mapper: bimap::BiMap<KeyIdent, OffchainPublicKey> = path
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from(i as u32), *k))
        .collect::<BiHashMap<_, _>>();
    let pseudonym = HoprPseudonym::random();

    let msg = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();
    let dst = Hash::default();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().direction(&(&sender_chain_key).into(), &destination);

    // Sender
    let forward_path = TransportPath::new(path).unwrap();
    let packet = match HoprPacket::into_outgoing(
        &msg,
        &pseudonym,
        PacketRouting::ForwardPath {
            forward_path,
            return_paths: vec![],
        },
        &sender_chain_key,
        tb,
        &mapper,
        &dst,
        None,
    )
    .unwrap()
    {
        (HoprPacket::Outgoing(out), _) => {
            let mut ret = Vec::with_capacity(HoprPacket::SIZE);
            ret.extend_from_slice(out.packet.as_ref());
            ret.extend_from_slice(&out.ticket.clone().into_encoded());
            ret.into_boxed_slice()
        }
        _ => panic!("should not happen"),
    };

    // Relayer
    let packet = match HoprPacket::from_incoming(&packet, &relayer, *sender.public(), &mapper, |_| None).unwrap() {
        HoprPacket::Forwarded(fwd) => {
            let mut ret = Vec::with_capacity(HoprPacket::SIZE);
            ret.extend_from_slice(fwd.outgoing.packet.as_ref());
            ret.extend_from_slice(&fwd.outgoing.ticket.clone().into_encoded());
            ret.into_boxed_slice()
        }
        _ => panic!("should not happen"),
    };

    // Benchmark the recipient
    let mut group = c.benchmark_group("packet_receiving");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));
    group.bench_function("any_hop", |b| {
        b.iter(|| {
            HoprPacket::from_incoming(&packet, &recipient, *relayer.public(), &mapper, |_| None).unwrap();
        })
    });
}

criterion_group!(
    benches,
    packet_sending_bench,
    packet_precompute_bench,
    packet_sending_precomputed_bench,
    packet_forwarding_bench,
    packet_receiving_bench
);
criterion_main!(benches);
