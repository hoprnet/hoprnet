use bimap::BiHashMap;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_packet::prelude::*;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::{ChainKeypair, Hash, Keypair, OffchainKeypair, OffchainPublicKey};
use hopr_internal_types::prelude::*;
use hopr_path::TransportPath;
use hopr_primitive_types::prelude::{Address, BytesEncodable, KeyIdent};

const SAMPLE_SIZE: usize = 100_000;

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

    let msg = hopr_crypto_random::random_bytes::<{ HoprPacket::PAYLOAD_SIZE }>();
    let dst = Hash::default();

    let mut group = c.benchmark_group("packet_sending");
    group.sample_size(SAMPLE_SIZE);

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::BytesDecimal(msg.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hop}_hop")), hop, |b, &hop| {
            b.iter(|| {
                // The number of hops for ticket creation does not matter for benchmark purposes
                let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);
                let tp = TransportPath::new(path.iter().take(hop + 1).copied()).unwrap();

                HoprPacket::into_outgoing(
                    &msg,
                    &pseudonym,
                    PacketRouting::ForwardPath {
                        forward_path: tp,
                        return_paths: vec![],
                    },
                    &chain_key,
                    tb,
                    &mapper,
                    &dst,
                    None,
                )
                .unwrap();
            });
        });
    }
    group.finish();
}

pub fn packet_precompute_1rp_bench(c: &mut Criterion) {
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

    let mut group = c.benchmark_group("packet_precompute_1rp");
    group.sample_size(SAMPLE_SIZE);

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hop}_hop")), hop, |b, &hop| {
            b.iter(|| {
                // The number of hops for ticket creation does not matter for benchmark purposes
                let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);
                let tp = TransportPath::new(path.iter().take(hop + 1).copied()).unwrap();

                PartialHoprPacket::new(
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
            });
        });
    }
    group.finish();
}

pub fn packet_precompute_2rp_bench(c: &mut Criterion) {
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

    let mut group = c.benchmark_group("packet_precompute_2rp");
    group.sample_size(SAMPLE_SIZE);

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hop}_hop")), hop, |b, &hop| {
            b.iter(|| {
                // The number of hops for ticket creation does not matter for benchmark purposes
                let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);
                let tp = TransportPath::new(path.iter().take(hop + 1).copied()).unwrap();

                PartialHoprPacket::new(
                    &pseudonym,
                    PacketRouting::ForwardPath {
                        forward_path: tp.clone(),
                        return_paths: vec![tp.clone(), tp],
                    },
                    &chain_key,
                    tb,
                    &mapper,
                    &dst,
                )
                .unwrap();
            });
        });
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

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::BytesDecimal(msg.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hop}_hop")), hop, |b, &hop| {
            // The number of hops for ticket creation does not matter for benchmark purposes
            let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);
            let tp = TransportPath::new(path.iter().take(hop + 1).copied()).unwrap();
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
    group.throughput(Throughput::BytesDecimal(msg.len() as u64));
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

    let chain_key = ChainKeypair::random();
    let destination = Address::new(&hopr_crypto_random::random_bytes::<20>());

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
    group.throughput(Throughput::BytesDecimal(msg.len() as u64));
    group.bench_function("any_hop", |b| {
        b.iter(|| {
            HoprPacket::from_incoming(&packet, &recipient, *relayer.public(), &mapper, |_| None).unwrap();
        })
    });
}

criterion_group!(
    benches,
    packet_sending_bench,
    packet_precompute_1rp_bench,
    packet_precompute_2rp_bench,
    packet_sending_precomputed_bench,
    packet_forwarding_bench,
    packet_receiving_bench
);
criterion_main!(benches);
