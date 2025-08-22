use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{AsyncReadExt, AsyncWriteExt, FutureExt, StreamExt};
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::{DestinationRouting, RoutingOptions};
use hopr_primitive_types::prelude::Address;
use hopr_protocol_app::prelude::ApplicationData;
use hopr_transport_session::{Capabilities, Capability, Session, SessionId};
use rand::{Rng, thread_rng};

pub async fn alice_send_data(
    data: &[u8],
    caps: impl Into<Capabilities> + std::fmt::Debug,
) -> impl futures::Stream<Item = Box<[u8]>> + Send {
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationData)>();
    let (_bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationData)>();

    let dst: Address = (&ChainKeypair::random()).into();
    let id = SessionId::new(1234_u64, HoprPseudonym::random());

    let mut alice_session = Session::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into().unwrap())),
        caps,
        (alice_tx, alice_rx.map(|(_, data)| data.plain_text)),
        None,
    )
    .unwrap();

    alice_session.write_all(data).await.unwrap();
    alice_session.flush().await.unwrap();
    alice_session.close().await.unwrap();

    bob_rx.map(|(_, data)| data.plain_text)
}

pub async fn bob_receive_data(data: Vec<Box<[u8]>>, caps: impl Into<Capabilities> + std::fmt::Debug) -> Vec<u8> {
    let (bob_tx, _alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationData)>();
    let id = SessionId::new(1234_u64, HoprPseudonym::random());

    let mut bob_session = Session::new(
        id,
        DestinationRouting::Return(id.pseudonym().into()),
        caps,
        (bob_tx, futures::stream::iter(data)),
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
