use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hopr_crypto_packet::packet::HoprPacket;
use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::{TicketBuilder, PAYLOAD_SIZE};
use hopr_primitive_types::prelude::{Address, BytesEncodable};

const SAMPLE_SIZE: usize = 100_000;

pub fn packet_sending_bench(c: &mut Criterion) {
    assert!(
        !hopr_crypto_random::is_rng_fixed(),
        "RNG must not be fixed for bench tests"
    );

    let chain_key = ChainKeypair::random();
    let destination = Address::new(&hopr_crypto_random::random_bytes::<20>());

    let path = (0..=3)
        .map(|_| OffchainKeypair::random().public().clone())
        .collect::<Vec<_>>();
    let msg = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>();
    let dst = Hash::default();

    let mut group = c.benchmark_group("packet_sending");
    group.sample_size(SAMPLE_SIZE);

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hop} hop")), hop, |b, &hop| {
            b.iter(|| {
                // The number of hops for ticket creation does not matter for benchmark purposes
                let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);

                HoprPacket::into_outgoing(&msg, &path[0..=hop], &chain_key, tb, &dst)
            });
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
    let path = [relayer.public().clone(), recipient.public().clone()];

    let msg = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>();
    let dst = Hash::default();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);

    // Sender
    let packet = match HoprPacket::into_outgoing(&msg, &path, &chain_key, tb, &dst).unwrap() {
        HoprPacket::Outgoing { packet, ticket, .. } => {
            let mut ret = Vec::with_capacity(HoprPacket::SIZE);
            ret.extend_from_slice(packet.as_ref());
            ret.extend_from_slice(&ticket.clone().into_encoded());
            ret.into_boxed_slice()
        }
        _ => panic!("should not happen"),
    };

    // Benchmark the relayer
    let mut group = c.benchmark_group("packet_forwarding");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));
    group.bench_function("any hop", |b| {
        b.iter(|| {
            HoprPacket::from_incoming(&packet, &relayer, sender.public().clone()).unwrap();
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
    let path = [relayer.public().clone(), recipient.public().clone()];

    let msg = hopr_crypto_random::random_bytes::<PAYLOAD_SIZE>();
    let dst = Hash::default();

    // The number of hops for ticket creation does not matter for benchmark purposes
    let tb = TicketBuilder::zero_hop().direction(&(&chain_key).into(), &destination);

    // Sender
    let packet = match HoprPacket::into_outgoing(&msg, &path, &chain_key, tb, &dst).unwrap() {
        HoprPacket::Outgoing { packet, ticket, .. } => {
            let mut ret = Vec::with_capacity(HoprPacket::SIZE);
            ret.extend_from_slice(packet.as_ref());
            ret.extend_from_slice(&ticket.clone().into_encoded());
            ret.into_boxed_slice()
        }
        _ => panic!("should not happen"),
    };

    // Relayer
    let packet = match HoprPacket::from_incoming(&packet, &relayer, sender.public().clone()).unwrap() {
        HoprPacket::Forwarded { packet, ticket, .. } => {
            let mut ret = Vec::with_capacity(HoprPacket::SIZE);
            ret.extend_from_slice(packet.as_ref());
            ret.extend_from_slice(&ticket.clone().into_encoded());
            ret.into_boxed_slice()
        }
        _ => panic!("should not happen"),
    };

    // Benchmark the recipient
    let mut group = c.benchmark_group("packet_receiving");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));
    group.bench_function("any hop", |b| {
        b.iter(|| {
            HoprPacket::from_incoming(&packet, &recipient, relayer.public().clone()).unwrap();
        })
    });
}

criterion_group!(
    benches,
    packet_sending_bench,
    packet_forwarding_bench,
    packet_receiving_bench
);
criterion_main!(benches);
