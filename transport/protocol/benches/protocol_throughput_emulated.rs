#[path = "../tests/common/mod.rs"]
mod common;

use common::{create_dbs, create_minimal_topology, random_packets_of_count, resolve_mock_path, PEERS, PEERS_CHAIN};
use core_path::path::TransportPath;
use criterion::{async_executor::AsyncExecutor, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use futures::StreamExt;
use hopr_crypto_types::keypairs::Keypair;
use hopr_internal_types::protocol::{Acknowledgement, ApplicationData};
use hopr_transport_protocol::msg::{
    mixer::MixerConfig,
    processor::{MsgSender, PacketInteractionConfig, PacketSendFinalizer},
};
use libp2p::PeerId;

const SAMPLE_SIZE: usize = 10;

pub fn protocol_throughput_sender(c: &mut Criterion) {
    const PAYLOAD_SIZE: usize = 490;
    const PEER_COUNT: usize = 3;
    const TESTED_PEER_ID: usize = 0;

    let mut group = c.benchmark_group("protocol_throughput_pipeline");
    group.sample_size(SAMPLE_SIZE);
    for bytes in [5 * 1024 * 2 * PAYLOAD_SIZE, 10 * 1024 * 2 * PAYLOAD_SIZE].iter() {
        group.throughput(Throughput::Bytes(*bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!(
                "random data with size {}",
                bytesize::ByteSize::b(*bytes as u64)
            )),
            bytes,
            |b, bytes| {
                let packets = random_packets_of_count(*bytes / PAYLOAD_SIZE);

                let runtime = criterion::async_executor::AsyncStdExecutor {};
                let dbs = runtime.block_on(async {
                    let mut dbs = create_dbs(PEER_COUNT).await.expect("DBs must be constructible");
                    create_minimal_topology(&mut dbs)
                        .await
                        .expect("topology must be constructible");
                    dbs
                });

                b.to_async(runtime).iter(|| {
                    let packets = packets.clone();
                    let dbs = dbs.clone();

                    async move {
                        let (_wire_ack_send_tx, wire_ack_send_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();
                        let (wire_ack_recv_tx, _wire_ack_recv_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();

                        let (_wire_msg_send_tx, wire_msg_send_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();
                        let (wire_msg_recv_tx, _wire_msg_recv_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();

                        let (api_send_tx, api_send_rx) =
                            futures::channel::mpsc::unbounded::<(ApplicationData, TransportPath, PacketSendFinalizer)>(
                            );
                        let (api_recv_tx, _api_recv_rx) = futures::channel::mpsc::unbounded::<ApplicationData>();

                        let cfg = PacketInteractionConfig {
                            check_unrealized_balance: true,
                            packet_keypair: (&PEERS[TESTED_PEER_ID]).clone(),
                            chain_keypair: (&PEERS_CHAIN[TESTED_PEER_ID]).clone(),
                            mixer: MixerConfig::default(),
                            outgoing_ticket_win_prob: 1.0,
                        };

                        let processes = hopr_transport_protocol::run_msg_ack_protocol(
                            cfg,
                            dbs[TESTED_PEER_ID].clone(),
                            None,
                            (wire_ack_recv_tx, wire_ack_send_rx),
                            (wire_msg_recv_tx, wire_msg_send_rx),
                            (api_recv_tx, api_send_rx),
                        )
                        .await;

                        let path = resolve_mock_path(
                            PEERS_CHAIN[TESTED_PEER_ID].public().to_address(),
                            PEERS[1..PEER_COUNT].iter().map(|p| p.public().into()).collect(),
                            PEERS_CHAIN[1..PEER_COUNT]
                                .iter()
                                .map(|key| key.public().to_address())
                                .collect(),
                        )
                        .await
                        .expect("path must be constructible");

                        let sender = MsgSender::new(api_send_tx);

                        futures::stream::iter(packets)
                            .map(|packet| {
                                let sender = sender.clone();
                                let path = path.clone();

                                async move {
                                    sender
                                        .send_packet(packet, path.clone())
                                        .await
                                        .expect("sending packet must succeed")
                                        .consume_and_wait(std::time::Duration::from_secs(1))
                                        .await
                                }
                            })
                            .for_each_concurrent(Some(40), |v| async {
                                assert!(v.await.is_ok());
                            })
                            .await;

                        for (_, jh) in processes {
                            jh.cancel().await;
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, protocol_throughput_sender,);
criterion_main!(benches);
