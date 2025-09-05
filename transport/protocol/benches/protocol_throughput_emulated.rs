#[path = "../tests/common/mod.rs"]
mod common;
use common::{PEERS, PEERS_CHAIN, create_dbs, create_minimal_topology, random_packets_of_count, resolve_mock_path};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::StreamExt;
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::keypairs::Keypair;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::prelude::HoprBalance;
use hopr_protocol_app::prelude::ApplicationData;
use hopr_transport_protocol::processor::{MsgSender, PacketInteractionConfig, PacketSendFinalizer};
use libp2p::PeerId;

const SAMPLE_SIZE: usize = 50;

pub fn protocol_throughput_sender(c: &mut Criterion) {
    const PAYLOAD_SIZE: usize = HoprPacket::PAYLOAD_SIZE;
    const PEER_COUNT: usize = 3;
    const TESTED_PEER_ID: usize = 0;

    let mut group = c.benchmark_group("protocol_throughput_pipeline");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    for bytes in [5 * 1024 * 2 * PAYLOAD_SIZE, 10 * 1024 * 2 * PAYLOAD_SIZE].iter() {
        group.throughput(Throughput::Bytes(*bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!(
                "random_data_size_{}",
                bytesize::ByteSize::b(*bytes as u64).to_string().replace(" ", "_")
            )),
            bytes,
            |b, bytes| {
                let packets = random_packets_of_count(*bytes / PAYLOAD_SIZE);

                let runtime = tokio::runtime::Runtime::new().expect("tokio runtime must be constructible");
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
                        let (wire_msg_send_tx, wire_msg_send_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();

                        let (_wire_msg_recv_tx, wire_msg_recv_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();

                        let (api_send_tx, api_send_rx) = futures::channel::mpsc::unbounded::<(
                            ApplicationData,
                            ResolvedTransportRouting,
                            PacketSendFinalizer,
                        )>();
                        let (api_recv_tx, _api_recv_rx) =
                            futures::channel::mpsc::unbounded::<(HoprPseudonym, ApplicationData)>();

                        let cfg = PacketInteractionConfig {
                            packet_keypair: PEERS[TESTED_PEER_ID].clone(),
                            outgoing_ticket_win_prob: Some(WinningProbability::ALWAYS),
                            outgoing_ticket_price: Some(HoprBalance::from(1)),
                        };

                        let processes = hopr_transport_protocol::run_msg_ack_protocol(
                            cfg,
                            dbs[TESTED_PEER_ID].clone(),
                            None,
                            (wire_msg_send_tx, wire_msg_recv_rx),
                            (api_recv_tx, api_send_rx),
                        )
                        .await;

                        let path = resolve_mock_path(
                            PEERS_CHAIN[TESTED_PEER_ID].public().to_address(),
                            PEERS[1..PEER_COUNT].iter().map(|p| *p.public()).collect(),
                            PEERS_CHAIN[1..PEER_COUNT]
                                .iter()
                                .map(|key| key.public().to_address())
                                .collect(),
                        )
                        .await
                        .expect("path must be constructible");

                        let sender = MsgSender::new(api_send_tx);
                        let routing = ResolvedTransportRouting::Forward {
                            pseudonym: HoprPseudonym::random(),
                            forward_path: path,
                            return_paths: vec![],
                        };

                        let count = packets.len();
                        futures::stream::iter(packets)
                            .map(|packet| {
                                let sender = sender.clone();
                                let path = routing.clone();

                                async move { sender.send_packet(packet, path.clone()).await }
                            })
                            .for_each_concurrent(Some(50), |v| async {
                                assert!(v.await.is_ok());
                            })
                            .await;

                        assert_eq!(wire_msg_send_rx.take(count).count().await, count);

                        for (_, jh) in processes {
                            jh.abort();
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
