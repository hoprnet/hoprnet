#[path = "../tests/common/mod.rs"]
mod common;

use std::{sync::Arc, time::Duration};

use common::{CHAIN_DATA, PEERS, PEERS_CHAIN, random_packets_of_count, resolve_mock_path};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{SinkExt, StreamExt};
use hopr_chain_connector::create_trustful_hopr_blokli_connector;
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::keypairs::Keypair;
use hopr_db_node::HoprNodeDb;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::prelude::HoprBalance;
use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};
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
                let (node_dbs, connectors) = runtime.block_on(async {
                    let mut node_dbs = Vec::new();
                    let mut connectors = Vec::new();
                    for i in 0..PEER_COUNT {
                        let node_db = HoprNodeDb::new_in_memory()
                            .await
                            .expect("node db must be constructible");
                        node_dbs.push(node_db);

                        let mut connector = create_trustful_hopr_blokli_connector(
                            &PEERS_CHAIN[i],
                            Default::default(),
                            CHAIN_DATA.clone().build_static_client(),
                            Default::default(),
                        )
                        .await
                        .expect("connector must be constructible");

                        connector
                            .connect(Duration::from_secs(3))
                            .await
                            .expect("connector must be connected");
                        connectors.push(Arc::new(connector));
                    }
                    (node_dbs, connectors)
                });

                b.to_async(runtime).iter(|| {
                    let packets = packets.clone();
                    let node_dbs = node_dbs.clone();
                    let connectors = connectors.clone();

                    async move {
                        let (wire_msg_send_tx, wire_msg_send_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();

                        let (_wire_msg_recv_tx, wire_msg_recv_rx) =
                            futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();

                        let (api_send_tx, api_send_rx) =
                            futures::channel::mpsc::unbounded::<(ResolvedTransportRouting, ApplicationDataOut)>();
                        let (api_recv_tx, _api_recv_rx) =
                            futures::channel::mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

                        let cfg = PacketInteractionConfig {
                            packet_keypair: PEERS[TESTED_PEER_ID].clone(),
                            outgoing_ticket_win_prob: Some(WinningProbability::ALWAYS),
                            outgoing_ticket_price: Some(HoprBalance::from(1)),
                        };

                        let processes = hopr_transport_protocol::run_msg_ack_protocol(
                            cfg,
                            node_dbs[TESTED_PEER_ID].clone(),
                            connectors[TESTED_PEER_ID].clone(),
                            (wire_msg_send_tx, wire_msg_recv_rx),
                            (api_recv_tx, api_send_rx),
                        )
                        .await;

                        let path = resolve_mock_path(
                            PEERS_CHAIN[TESTED_PEER_ID].public().to_address(),
                            PEERS_CHAIN[1..PEER_COUNT]
                                .iter()
                                .map(|key| key.public().to_address())
                                .collect(),
                        )
                        .await
                        .expect("path must be constructible");

                        let routing = ResolvedTransportRouting::Forward {
                            pseudonym: HoprPseudonym::random(),
                            forward_path: path,
                            return_paths: vec![],
                        };

                        let count = packets.len();
                        futures::stream::iter(packets)
                            .map(|packet| {
                                let mut sender = api_send_tx.clone();
                                let path = routing.clone();

                                async move {
                                    sender
                                        .send((path.clone(), ApplicationDataOut::with_no_packet_info(packet)))
                                        .await
                                }
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
