#[allow(unused)]
#[path = "../src/session/utils/test.rs"]
mod test;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{AsyncReadExt, AsyncWriteExt};
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_network_types::{
    session::{SessionSocketConfig, StatelessSocket},
    utils::DuplexIO,
};
use rand::{Rng, thread_rng};

use crate::test::{FaultyNetwork, FaultyNetworkConfig};

const MTU: usize = HoprPacket::PAYLOAD_SIZE;

fn setup_network<const MTU: usize>(
    network_cfg: FaultyNetworkConfig,
    session_cfg: SessionSocketConfig,
) -> (StatelessSocket<MTU>, StatelessSocket<MTU>) {
    let (alice_reader, alice_writer) = FaultyNetwork::<MTU>::new(network_cfg.clone(), None).split();
    let (bob_reader, bob_writer) = FaultyNetwork::<MTU>::new(network_cfg, None).split();

    (
        StatelessSocket::<MTU>::new_stateless("alice", DuplexIO(alice_reader, bob_writer), session_cfg.clone())
            .expect("socket creation failed"),
        StatelessSocket::<MTU>::new_stateless("bob", DuplexIO(bob_reader, alice_writer), session_cfg.clone())
            .expect("socket creation failed"),
    )
}

async fn send_one_way(
    network_cfg: FaultyNetworkConfig,
    session_cfg: SessionSocketConfig,
    data: &[u8],
) -> anyhow::Result<()> {
    let (mut a_to_b, mut b_to_a) = setup_network::<MTU>(network_cfg, session_cfg);

    a_to_b.write_all(data).await?;

    let mut data_out = vec![0u8; data.len()];
    b_to_a.read_exact(&mut data_out[..]).await?;

    a_to_b.close().await?;
    b_to_a.close().await?;

    Ok(())
}

pub fn session_one_way_unreliable_send_recv_benchmark(c: &mut Criterion) {
    const KB: usize = 1024;

    let mut group = c.benchmark_group("session_one_way_reliable_send_recv");

    group.sample_size(1000);

    let network_cfg = FaultyNetworkConfig::default();
    let session_cfg = SessionSocketConfig { ..Default::default() };

    let runtime = tokio::runtime::Runtime::new().unwrap();

    for size in [16 * KB, 64 * KB, 128 * KB, 1024 * KB].iter() {
        let mut data = vec![0u8; *size];
        thread_rng().fill(&mut data[..]);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(bytesize::ByteSize::b(*size as u64).to_string().replace(" ", "_")),
            &data,
            |b, data| {
                b.to_async(&runtime)
                    .iter(|| send_one_way(network_cfg.clone(), session_cfg.clone(), data));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, session_one_way_unreliable_send_recv_benchmark);
criterion_main!(benches);
