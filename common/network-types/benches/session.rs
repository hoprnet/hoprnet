use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::{AsyncReadExt, AsyncWriteExt};
use hopr_network_types::prelude::state::{SessionConfig, SessionSocket};
use hopr_network_types::utils::DuplexIO;
use rand::{thread_rng, Rng};
use std::collections::HashSet;

#[cfg(not(feature = "testing"))]
compile_error!("Must specify the 'testing' feature");

use hopr_network_types::prelude::{FaultyNetwork, FaultyNetworkConfig};

const MTU: usize = 466;

fn setup_network<const MTU: usize>(
    network_cfg: FaultyNetworkConfig,
    session_cfg: SessionConfig,
) -> (SessionSocket<MTU>, SessionSocket<MTU>) {
    let (alice_reader, alice_writer) = FaultyNetwork::<MTU>::new(network_cfg, None).split();
    let (bob_reader, bob_writer) = FaultyNetwork::<MTU>::new(network_cfg, None).split();

    (
        SessionSocket::<MTU>::new("alice", DuplexIO(alice_reader, bob_writer), session_cfg.clone()),
        SessionSocket::<MTU>::new("bob", DuplexIO(bob_reader, alice_writer), session_cfg.clone()),
    )
}

async fn send_one_way(network_cfg: FaultyNetworkConfig, session_cfg: SessionConfig, data: &[u8]) -> anyhow::Result<()> {
    let (mut a_to_b, mut b_to_a) = setup_network::<MTU>(network_cfg, session_cfg);

    a_to_b.write_all(&data).await?;

    let mut data_out = vec![0u8; data.len()];
    b_to_a.read_exact(&mut data_out[..]).await?;

    a_to_b.close().await?;
    b_to_a.close().await?;

    Ok(())
}

pub fn session_one_way_reliable_send_recv_benchmark(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("session_one_way_reliable_send_recv");

    group.sample_size(1000);

    let network_cfg = FaultyNetworkConfig::default();
    let session_cfg = SessionConfig {
        enabled_features: HashSet::new(),
        ..Default::default()
    };

    for size in [16 * KB, 64 * KB, 128 * KB, 1024 * KB].iter() {
        let mut data = vec![0u8; *size];
        thread_rng().fill(&mut data[..]);

        let runtime = tokio::runtime::Runtime::new().unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.to_async(&runtime)
                .iter(|| send_one_way(network_cfg, session_cfg.clone(), data));
        });
    }
    group.finish();
}

criterion_group!(benches, session_one_way_reliable_send_recv_benchmark,);
criterion_main!(benches);
