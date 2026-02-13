use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{AsyncRead, AsyncWrite, io::Cursor};
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_network_types::utils::DuplexIO;
use hopr_protocol_session::{SessionSocketConfig, UnreliableSocket};
use tokio_util::compat::TokioAsyncReadCompatExt;

// Avoid musl's default allocator due to degraded performance
//
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(feature = "allocator-mimalloc", feature = "allocator-jemalloc"))]
compile_error!("feature \"allocator-jemalloc\" and feature \"allocator-mimalloc\" cannot be enabled at the same time");
#[cfg(all(target_os = "linux", feature = "allocator-mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(target_os = "linux", feature = "allocator-jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const MTU: usize = HoprPacket::PAYLOAD_SIZE;

pub async fn alice_send_data<S: AsyncRead + AsyncWrite + Send + Unpin + 'static>(data: &[u8], alice: S) {
    use futures::AsyncWriteExt;

    let mut alice_socket =
        UnreliableSocket::<MTU>::new_stateless("alice", alice, SessionSocketConfig::default()).unwrap();

    alice_socket.write_all(data).await.unwrap();
    alice_socket.flush().await.unwrap();
    alice_socket.close().await.unwrap();
}

pub async fn bob_receive_data(data: Vec<u8>, mut recv_data: Vec<u8>) -> Vec<u8> {
    use futures::AsyncReadExt;

    let mut bob_socket = UnreliableSocket::<MTU>::new_stateless(
        "bob",
        DuplexIO::from((futures::io::sink(), Cursor::new(data))),
        SessionSocketConfig::default(),
    )
    .unwrap();
    bob_socket.read_to_end(&mut recv_data).await.unwrap();

    recv_data
}

pub fn stateless_socket_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("stateless_socket_benchmark");
    const KB: usize = 1024;

    group.sample_size(100000);

    for size in [/* 16 * KB, 64 * KB, */ 128 * KB, 1024 * KB].iter() {
        let mut alice_data = vec![0u8; *size];

        hopr_crypto_random::random_fill(&mut alice_data);

        // Prepare data
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let (alice, mut bob) = tokio::io::duplex(2 * size);

        runtime.block_on(alice_send_data(&alice_data, alice.compat()));

        let mut wire_data = Vec::with_capacity(*size * 2);
        {
            use tokio::io::AsyncReadExt;
            let recv_size = runtime.block_on(bob.read_to_end(&mut wire_data)).unwrap();
            assert!(recv_size > *size);
        }

        // Make a sanity check
        assert_eq!(
            alice_data,
            runtime.block_on(bob_receive_data(wire_data.clone(), Vec::with_capacity(*size)))
        );

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("alice_tx", size), &alice_data, |b, data| {
            b.to_async(&runtime)
                .iter(|| alice_send_data(data, DuplexIO::from((futures::io::sink(), futures::io::empty()))));
        });
        group.bench_with_input(BenchmarkId::new("bob_rx", size), &wire_data, |b, data| {
            b.to_async(&runtime)
                .iter(|| bob_receive_data(data.clone(), Vec::with_capacity(*size)));
        });
    }
    group.finish();
}

criterion_group!(benches, stateless_socket_benchmark);
criterion_main!(benches);
