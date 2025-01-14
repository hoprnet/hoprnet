#[path = "../tests/common/mod.rs"]
mod common;

use common::{
    emulate_channel_communication, peer_setup_for, random_packets_of_count, resolve_mock_path, PEERS, PEERS_CHAIN,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use futures::StreamExt;
use hopr_crypto_types::keypairs::Keypair;
use hopr_internal_types::protocol::ApplicationData;
use hopr_transport_protocol::msg::processor::MsgSender;

const SAMPLE_SIZE: usize = 100;

async fn send_continuous_load(items: Vec<ApplicationData>, sender: MsgSender, size: usize) {
    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS[1..size].iter().map(|p| p.public().into()).collect(),
        PEERS_CHAIN[1..size]
            .iter()
            .map(|key| key.public().to_address())
            .collect(),
    )
    .await
    .expect("path must be constructible");

    futures::stream::iter(items)
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
            let _ = v.await;
        })
        .await;
}

pub fn protocol_throughput_emulated_channel(c: &mut Criterion) {
    const ITERATIONS: usize = 2 * 1024 * 10; // 10MB
    const PEER_COUNT: usize = 3;

    let packets = random_packets_of_count(ITERATIONS);

    let mut group = c.benchmark_group("protocol_throughput_emulated");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));
    group.bench_with_input(
        BenchmarkId::from_parameter(format!(
            "random data with size {}",
            bytesize::ByteSize::b((packets.len() * 500) as u64)
        )),
        &packets,
        |b, data| {
            let runtime = tokio::runtime::Runtime::new().expect("Runtime must be constructible");

            let (wire_apis, apis, _ticket_channels) = runtime.block_on(async {
                peer_setup_for(PEER_COUNT)
                    .await
                    .expect("test setup must be constructible")
            });

            let channel = runtime.spawn(emulate_channel_communication(packets.len(), wire_apis));
            let sender = MsgSender::new(apis.first().expect("first element must exist").0.clone());

            b.to_async(runtime).iter(move || {
                let sender = sender.clone();

                send_continuous_load(data.clone(), sender, PEER_COUNT)
            });

            channel.abort();
        },
    );

    group.finish();
}

criterion_group!(benches, protocol_throughput_emulated_channel,);
criterion_main!(benches);
