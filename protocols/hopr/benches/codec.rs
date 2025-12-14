#[path = "../src/utils.rs"]
mod utils;

use std::sync::Arc;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_chain_connector::{
    HoprBlockchainSafeConnector,
    testing::{BlokliTestClient, StaticState},
};
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_db_node::HoprNodeDb;
use hopr_internal_types::{
    path::ValidatedPath,
    prelude::{HoprPseudonym, VerifiedAcknowledgement},
};
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_protocol_hopr::{
    HoprCodecConfig, HoprDecoder, HoprEncoder, HoprTicketProcessor, HoprTicketProcessorConfig, MemorySurbStore,
    PacketDecoder, PacketEncoder, SurbStoreConfig,
};

use crate::utils::{Node, PEERS, create_blokli_client, create_node};

type TestEncoder = HoprEncoder<
    Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
    MemorySurbStore,
    HoprTicketProcessor<Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>, HoprNodeDb>,
>;

type TestDecoder = HoprDecoder<
    Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
    MemorySurbStore,
    HoprTicketProcessor<Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>, HoprNodeDb>,
>;

pub fn create_encoder(sender: &Node) -> TestEncoder {
    HoprEncoder::new(
        sender.chain_key.clone(),
        sender.chain_api.clone(),
        MemorySurbStore::new(SurbStoreConfig::default()),
        HoprTicketProcessor::new(
            sender.chain_api.clone(),
            sender.node_db.clone(),
            sender.chain_key.clone(),
            Hash::default(),
            HoprTicketProcessorConfig::default(),
        ),
        Hash::default(),
        HoprCodecConfig::default(),
    )
}

pub fn create_decoder(receiver: &Node) -> TestDecoder {
    HoprDecoder::new(
        (receiver.offchain_key.clone(), receiver.chain_key.clone()),
        receiver.chain_api.clone(),
        MemorySurbStore::new(SurbStoreConfig::default()),
        HoprTicketProcessor::new(
            receiver.chain_api.clone(),
            receiver.node_db.clone(),
            receiver.chain_key.clone(),
            Hash::default(),
            HoprTicketProcessorConfig::default(),
        ),
        Hash::default(),
        HoprCodecConfig::default(),
    )
}

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

fn hopr_encoder_bench(c: &mut Criterion) {
    let blokli_client = create_blokli_client().unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let sender = runtime.block_on(async { create_node(0, &blokli_client).await.unwrap() });

    let mut group = c.benchmark_group("hopr_encoder");
    group.throughput(Throughput::Elements(1));

    for (hops, rps) in PACKET_BENCHMARK {
        let path = runtime
            .block_on(async {
                ValidatedPath::new(
                    PEERS[0].0.public().to_address(),
                    PEERS[1..]
                        .iter()
                        .map(|p| p.0.public().to_address())
                        .take(hops + 1)
                        .collect::<Vec<_>>(),
                    &sender.chain_api.as_path_resolver(),
                )
                .await
            })
            .unwrap();

        let return_path = runtime.block_on(async {
            ValidatedPath::new(
                PEERS[4].0.public().to_address(),
                PEERS[0..=3]
                    .iter()
                    .rev()
                    .map(|p| p.0.public().to_address())
                    .take(hops + 1)
                    .collect::<Vec<_>>(),
                &sender.chain_api.as_path_resolver(),
            )
            .await
            .unwrap()
        });

        let routing = ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: path.clone(),
            return_paths: (0..rps).map(|_| return_path.clone()).collect::<Vec<_>>(),
        };

        let mut data = vec![0_u8; HoprPacket::max_message_with_surbs(rps)];
        hopr_crypto_random::random_fill(&mut data);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}hop_{}surbs_{}b", hops, rps, data.len())),
            &routing,
            |b, routing| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let encoder = create_encoder(&sender);
                b.to_async(runtime).iter_batched(
                    || (data.clone(), routing.clone()),
                    |(data, routing)| async { encoder.encode_packet(data, routing, None).await.unwrap() },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    let ack_recipient = *PEERS[1].1.public();
    for num_acks in [1, 2, 5, 10] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("ack_batch_{num_acks}")),
            &num_acks,
            |b, num_acks| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let acks = (0..*num_acks).map(|_| VerifiedAcknowledgement::random(&PEERS[0].1)).collect::<Vec<_>>();
                b.to_async(runtime).iter_batched(|| (create_encoder(&sender), acks.clone()), |(encoder, acks)| async move {
                    encoder.encode_acknowledgements(&acks, &ack_recipient).await.unwrap()
                },
                BatchSize::SmallInput)
            }
        );
    }
}

fn hopr_decoder_bench(c: &mut Criterion) {
    let blokli_client = create_blokli_client().unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (sender, relay, recipient) = runtime.block_on(async {
        (
            create_node(0, &blokli_client).await.unwrap(),
            create_node(1, &blokli_client).await.unwrap(),
            create_node(2, &blokli_client).await.unwrap(),
        )
    });

    let data = hopr_crypto_random::random_bytes::<1024>();

    let mut group = c.benchmark_group("hopr_decoder");
    group.throughput(Throughput::Elements(1));

    let path = runtime
        .block_on(async {
            ValidatedPath::new(
                PEERS[0].0.public().to_address(),
                PEERS[1..=2]
                    .iter()
                    .map(|p| p.0.public().to_address())
                    .collect::<Vec<_>>(),
                &sender.chain_api.as_path_resolver(),
            )
            .await
        })
        .unwrap();

    let routing = ResolvedTransportRouting::Forward {
        pseudonym: HoprPseudonym::random(),
        forward_path: path.clone(),
        return_paths: vec![],
    };

    let encoder = create_encoder(&sender);
    let relay_packet = runtime.block_on(async { encoder.encode_packet(data, routing, None).await.unwrap() });
    let ack_packet = runtime.block_on(async {
        let acks = (0..10)
            .map(|_| VerifiedAcknowledgement::random(&PEERS[0].1))
            .collect::<Vec<_>>();
        encoder
            .encode_acknowledgements(&acks, PEERS[1].1.public())
            .await
            .unwrap()
    });

    let prev_hop: PeerId = PEERS[0].1.public().into();
    group.bench_with_input("relay", &relay_packet, |b, packet| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.to_async(runtime).iter_batched(
            || (create_decoder(&relay), packet.data.clone()),
            |(decoder, data)| async move { decoder.decode(prev_hop, data).await.unwrap() },
            BatchSize::SmallInput,
        )
    });

    let prev_hop: PeerId = PEERS[0].1.public().into();
    group.bench_with_input("ack_recipient", &ack_packet, |b, packet| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.to_async(runtime).iter_batched(
            || (create_decoder(&relay), packet.data.clone()),
            |(decoder, data)| async move { decoder.decode(prev_hop, data).await.unwrap() },
            BatchSize::SmallInput,
        )
    });

    let decoder = create_decoder(&relay);
    let final_packet = runtime.block_on(async { decoder.decode(prev_hop, relay_packet.data).await.unwrap() });

    let prev_hop: PeerId = PEERS[1].1.public().into();
    group.bench_with_input("recipient", &final_packet, |b, packet| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.to_async(runtime).iter_batched(
            || {
                (
                    create_decoder(&recipient),
                    packet.try_as_forwarded_ref().unwrap().data.clone(),
                )
            },
            |(decoder, data)| async move { decoder.decode(prev_hop, data).await.unwrap() },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, hopr_encoder_bench, hopr_decoder_bench);
criterion_main!(benches);
