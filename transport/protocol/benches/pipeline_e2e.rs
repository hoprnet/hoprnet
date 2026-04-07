#[path = "../tests/common/mod.rs"]
mod common;

use std::{str::FromStr, time::Duration};

use common::{PEERS, PEERS_CHAIN, random_packets_of_count};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{
    SinkExt, StreamExt,
    channel::{mpsc, oneshot},
};
use hopr_api::types::{
    crypto::keypairs::Keypair,
    crypto_random::Randomizable,
    internal::{path::ValidatedPath, prelude::*, routing::ResolvedTransportRouting},
    primitive::prelude::HoprBalance,
};
use hopr_crypto_packet::{HoprSurb, prelude::HoprPacket};
use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};
use hopr_protocol_hopr::{
    HoprCodecConfig, HoprDecoder, HoprEncoder, HoprUnacknowledgedTicketProcessor, MemorySurbStore, PacketEncoder,
    SurbStoreConfig,
};
use hopr_test_stubs::{StubChainApi, StubPathResolver};
use hopr_ticket_manager::{HoprTicketFactory, MemoryStore};
use hopr_transport_mixer::config::MixerConfig;
use hopr_transport_protocol::TicketEvent;
use libp2p::PeerId;

const SAMPLE_SIZE: usize = 30;
const MEASUREMENT_TIME_SECS: u64 = 30;
const SENDER_IDX: usize = 0;
const PAYLOAD_SIZE: usize = HoprPacket::PAYLOAD_SIZE;
const CHANNEL_CAPACITY: usize = 2048;

const HOPS: [usize; 4] = [0, 1, 2, 3];
const PACKET_COUNTS: [usize; 3] = [1_000, 2_500, 5_000];
const WIN_PROB: f64 = 0.01;

/// Drains encoded packets from the mixer output and feeds pre-generated ack
/// packets back through the wire-in channel. Signals completion once `expected`
/// packets have been received.
async fn network_stub(
    mut wire_out_rx: hopr_transport_mixer::channel::Receiver<(PeerId, Box<[u8]>)>,
    mut wire_in_tx: mpsc::Sender<(PeerId, Box<[u8]>)>,
    ack_buffer: Vec<(PeerId, Box<[u8]>)>,
    expected: usize,
    hops: usize,
    done_tx: oneshot::Sender<()>,
) {
    let mut received = 0;
    let mut ack_idx = 0;

    while let Some((_peer, _data)) = wire_out_rx.next().await {
        received += 1;

        if hops > 0 && !ack_buffer.is_empty() {
            let ack = ack_buffer[ack_idx % ack_buffer.len()].clone();
            let _ = wire_in_tx.send(ack).await;
            ack_idx += 1;
        }

        if received >= expected {
            let _ = done_tx.send(());
            return;
        }
    }

    // Stream closed before reaching `expected` — signal completion to prevent hangs.
    let _ = done_tx.send(());
}

fn build_channel(src_idx: usize, dst_idx: usize) -> ChannelEntry {
    ChannelEntry::builder()
        .between(
            PEERS_CHAIN[src_idx].public().to_address(),
            PEERS_CHAIN[dst_idx].public().to_address(),
        )
        .balance(HoprBalance::from_str("100 wxHOPR").unwrap())
        .ticket_index(0)
        .status(ChannelStatus::Open)
        .epoch(1)
        .build()
        .unwrap()
}

fn build_chain_api() -> StubChainApi {
    let mut builder = StubChainApi::builder().me(PEERS_CHAIN[SENDER_IDX].public().to_address());

    for (offchain, chain) in PEERS.iter().zip(PEERS_CHAIN.iter()) {
        builder = builder.peer(offchain.public(), chain.public().to_address());
    }

    // Create bidirectional channels between consecutive peers
    // (forward for the outgoing path, reverse for the return path)
    for i in 0..PEERS_CHAIN.len() - 1 {
        builder = builder.channel(build_channel(i, i + 1));
        builder = builder.channel(build_channel(i + 1, i));
    }

    builder
        .ticket_price(HoprBalance::from_str("0.1 wxHOPR").expect("valid balance"))
        .win_prob(WinningProbability::try_from_f64(WIN_PROB).expect("valid win prob"))
        .build()
}

fn pipeline_e2e_forward(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime must be constructible");

    let chain_api = build_chain_api();

    let codec_config = HoprCodecConfig {
        outgoing_ticket_price: Some(HoprBalance::from_str("0.1 wxHOPR").expect("valid balance")),
        outgoing_win_prob: Some(WinningProbability::try_from_f64(WIN_PROB).expect("valid win prob")),
        ..Default::default()
    };

    let resolver = StubPathResolver::from_chain_api(&chain_api);

    // Pre-generate ack buffer using an encoder for PEERS[1] (next hop)
    let max_packets = *PACKET_COUNTS.last().unwrap();
    let ack_buffer: Vec<(PeerId, Box<[u8]>)> = runtime.block_on(async {
        let ack_encoder = HoprEncoder::new(
            PEERS_CHAIN[1].clone(),
            chain_api.clone(),
            MemorySurbStore::new(SurbStoreConfig::default()),
            HoprTicketFactory::new(MemoryStore::default()),
            Default::default(),
            HoprCodecConfig::default(),
        );

        let sender_peer_id = PeerId::from(*PEERS[1].public());
        let mut buffer = Vec::with_capacity(max_packets);

        for _ in 0..max_packets {
            let ack = VerifiedAcknowledgement::random(&PEERS[SENDER_IDX]);
            let packet = ack_encoder
                .encode_acknowledgements(&[ack], PEERS[SENDER_IDX].public())
                .expect("ack encoding must succeed");
            buffer.push((sender_peer_id, packet.data));
        }
        buffer
    });

    // Pre-resolve forward and return paths for each hop count.
    // Forward: PEERS[0] → PEERS[1..=hops].
    // Return:  PEERS[hops] → PEERS[hops-1..=0] (symmetric hop count).
    let paths: Vec<(ValidatedPath, ValidatedPath)> = runtime.block_on(async {
        let mut paths = Vec::with_capacity(HOPS.len());
        for &hops in &HOPS {
            let dest_idx = hops + 1; // index of the destination peer

            let forward = ValidatedPath::new(
                PEERS_CHAIN[SENDER_IDX].public().to_address(),
                PEERS_CHAIN[1..=dest_idx]
                    .iter()
                    .map(|key| key.public().to_address())
                    .collect::<Vec<_>>(),
                &resolver,
            )
            .await
            .expect("forward path must be constructible");

            let return_path = ValidatedPath::new(
                PEERS_CHAIN[dest_idx].public().to_address(),
                PEERS_CHAIN[..dest_idx]
                    .iter()
                    .rev()
                    .map(|key| key.public().to_address())
                    .collect::<Vec<_>>(),
                &resolver,
            )
            .await
            .expect("return path must be constructible");

            paths.push((forward, return_path));
        }
        paths
    });

    let mixer_cfg = MixerConfig {
        min_delay: Duration::from_nanos(1),
        delay_range: Duration::from_nanos(1),
        capacity: 20_000,
        ..Default::default()
    };

    let mut group = c.benchmark_group("pipeline_e2e_forward");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(Duration::from_secs(MEASUREMENT_TIME_SECS));

    for (hop_idx, &hops) in HOPS.iter().enumerate() {
        for &packet_count in &PACKET_COUNTS {
            let total_bytes = (packet_count * PAYLOAD_SIZE) as u64;
            let id = format!("{hops}hop_{packet_count}pkt");

            group.throughput(Throughput::Bytes(total_bytes));

            let (fwd_path, ret_path) = paths[hop_idx].clone();
            let ack_buf = ack_buffer[..packet_count.min(ack_buffer.len())].to_vec();
            let chain_api = chain_api.clone();

            group.bench_with_input(BenchmarkId::from_parameter(&id), &packet_count, |b, &packet_count| {
                b.to_async(&runtime).iter_batched(
                    || {
                        // -- SETUP (not timed) --
                        let packets = random_packets_of_count(packet_count);

                        let (mixer_tx, mixer_rx) = hopr_transport_mixer::channel::<(PeerId, Box<[u8]>)>(mixer_cfg);

                        let (wire_in_tx, wire_in_rx) = mpsc::channel::<(PeerId, Box<[u8]>)>(CHANNEL_CAPACITY);

                        let (api_send_tx, api_send_rx) =
                            mpsc::channel::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>(CHANNEL_CAPACITY);

                        let (api_recv_tx, _api_recv_rx) =
                            mpsc::channel::<(HoprPseudonym, ApplicationDataIn)>(CHANNEL_CAPACITY);

                        let (ticket_events_tx, _ticket_events_rx) = mpsc::channel::<TicketEvent>(CHANNEL_CAPACITY);

                        let unack_proc = MemorySurbStore::new(SurbStoreConfig::default());

                        let ticket_proc = HoprUnacknowledgedTicketProcessor::new(
                            chain_api.clone(),
                            PEERS_CHAIN[SENDER_IDX].clone(),
                            Default::default(),
                            Default::default(),
                        );

                        let ticket_mgr = std::sync::Arc::new(HoprTicketFactory::new(MemoryStore::default()));

                        let encoder = HoprEncoder::new(
                            PEERS_CHAIN[SENDER_IDX].clone(),
                            chain_api.clone(),
                            unack_proc.clone(),
                            ticket_mgr.clone(),
                            Default::default(),
                            codec_config,
                        );

                        let decoder = HoprDecoder::new(
                            (PEERS[SENDER_IDX].clone(), PEERS_CHAIN[SENDER_IDX].clone()),
                            chain_api.clone(),
                            unack_proc,
                            ticket_mgr.clone(),
                            Default::default(),
                            codec_config,
                        );

                        let processes = hopr_transport_protocol::run_packet_pipeline(
                            PEERS[SENDER_IDX].clone(),
                            (mixer_tx, wire_in_rx),
                            (encoder, decoder),
                            ticket_proc,
                            ticket_events_tx,
                            Default::default(),
                            (api_recv_tx, api_send_rx),
                            Default::default(),
                        );

                        let (done_tx, done_rx) = oneshot::channel::<()>();

                        let routing = ResolvedTransportRouting::Forward {
                            pseudonym: HoprPseudonym::random(),
                            forward_path: fwd_path.clone(),
                            return_paths: vec![ret_path.clone()],
                        };

                        // Spawn the network stub
                        tokio::spawn(network_stub(
                            mixer_rx,
                            wire_in_tx,
                            ack_buf.clone(),
                            packet_count,
                            hops,
                            done_tx,
                        ));

                        (packets, api_send_tx, routing, done_rx, processes)
                    },
                    |(packets, mut api_send_tx, routing, done_rx, processes)| {
                        // -- TIMED --
                        async move {
                            for pkt in packets {
                                api_send_tx
                                    .send((routing.clone(), ApplicationDataOut::with_no_packet_info(pkt)))
                                    .await
                                    .expect("send must succeed");
                            }
                            done_rx.await.expect("completion signal must arrive");
                            processes.abort_all();
                        }
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, pipeline_e2e_forward);
criterion_main!(benches);
