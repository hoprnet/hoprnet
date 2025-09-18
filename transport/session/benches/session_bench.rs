use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{AsyncReadExt, AsyncWriteExt, FutureExt, StreamExt};
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::{DestinationRouting, RoutingOptions};
use hopr_primitive_types::prelude::Address;
use hopr_protocol_app::{prelude::ApplicationDataOut, v1::ApplicationDataIn};
use hopr_transport_session::{Capabilities, Capability, HoprSession, HoprSessionConfig, SessionId};
use rand::{Rng, thread_rng};

// Avoid musl's default allocator due to degraded performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(any(target_env = "musl", target_env = "gnu"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

pub async fn alice_send_data(
    data: &[u8],
    caps: impl Into<Capabilities> + std::fmt::Debug,
) -> impl futures::Stream<Item = ApplicationDataIn> + Send {
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (_bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    let dst: Address = (&ChainKeypair::random()).into();
    let id = SessionId::new(1234_u64, HoprPseudonym::random());

    let mut alice_session = HoprSession::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into().unwrap())),
        HoprSessionConfig {
            capabilities: caps.into(),
            ..Default::default()
        },
        (
            alice_tx,
            alice_rx.map(|(_, data)| ApplicationDataIn {
                data: data.data,
                packet_info: Default::default(),
            }),
        ),
        None,
    )
    .unwrap();

    alice_session.write_all(data).await.unwrap();
    alice_session.flush().await.unwrap();
    alice_session.close().await.unwrap();

    bob_rx.map(|(_, data)| ApplicationDataIn {
        data: data.data,
        packet_info: Default::default(),
    })
}

pub async fn bob_receive_data(
    data: Vec<ApplicationDataIn>,
    caps: impl Into<Capabilities> + std::fmt::Debug,
) -> Vec<u8> {
    let (bob_tx, _alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let id = SessionId::new(1234_u64, HoprPseudonym::random());

    let mut bob_session = HoprSession::new(
        id,
        DestinationRouting::Return(id.pseudonym().into()),
        HoprSessionConfig {
            capabilities: caps.into(),
            ..Default::default()
        },
        (bob_tx, futures::stream::iter(data).map(|data| data)),
        None,
    )
    .unwrap();

    let mut vec = Vec::with_capacity(1024 * 1024);
    bob_session.read_to_end(&mut vec).await.unwrap();
    bob_session.close().await.unwrap();

    vec
}

pub fn session_raw_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_raw_benchmark");
    const KB: usize = 1024;

    group.sample_size(100000);
    group.measurement_time(std::time::Duration::from_secs(30));

    for size in [16 * KB, 64 * KB, 128 * KB, 1024 * KB].iter() {
        let mut alice_data = vec![0u8; *size];
        thread_rng().fill(&mut alice_data[..]);

        // Prepare data and make a sanity check
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let bob_data =
            runtime.block_on(alice_send_data(&alice_data, Capability::Segmentation).then(|rx| rx.collect::<Vec<_>>()));
        let bob_recv = runtime.block_on(bob_receive_data(bob_data.clone(), Capability::Segmentation));
        assert_eq!(alice_data, bob_recv);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("alice_tx", size), &alice_data, |b, data| {
            b.to_async(&runtime).iter(|| alice_send_data(data, None));
        });
        group.bench_with_input(BenchmarkId::new("bob_rx", size), &bob_data, |b, data| {
            b.to_async(&runtime).iter(|| bob_receive_data(data.clone(), None));
        });
    }
    group.finish();
}

pub fn session_segmentation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_segmentation_benchmark");
    const KB: usize = 1024;

    group.sample_size(100000);
    group.measurement_time(std::time::Duration::from_secs(30));

    for size in [16 * KB, 64 * KB, 128 * KB, 1024 * KB].iter() {
        let mut alice_data = vec![0u8; *size];
        thread_rng().fill(&mut alice_data[..]);

        // Prepare data and make a sanity check
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let bob_data =
            runtime.block_on(alice_send_data(&alice_data, Capability::Segmentation).then(|rx| rx.collect::<Vec<_>>()));
        let bob_recv = runtime.block_on(bob_receive_data(bob_data.clone(), Capability::Segmentation));
        assert_eq!(alice_data, bob_recv);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("alice_tx", size), &alice_data, |b, data| {
            b.to_async(&runtime)
                .iter(|| alice_send_data(data, Capability::Segmentation));
        });
        group.bench_with_input(BenchmarkId::new("bob_rx", size), &bob_data, |b, data| {
            b.to_async(&runtime)
                .iter(|| bob_receive_data(data.clone(), Capability::Segmentation));
        });
    }
    group.finish();
}

criterion_group!(benches, session_raw_benchmark, session_segmentation_benchmark);
criterion_main!(benches);
