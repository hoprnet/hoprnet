use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{AsyncRead, AsyncWrite, io::Cursor};
use hopr_crypto_packet::prelude::HoprPacket;
#[cfg(feature = "telemetry")]
use hopr_protocol_session::NoopTracker;
use hopr_protocol_session::{SessionSocketConfig, UnreliableSocket};
use hopr_utils::network_types::utils::DuplexIO;
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

    let mut alice_socket = UnreliableSocket::<MTU>::new_stateless(
        "alice",
        alice,
        SessionSocketConfig::default(),
        #[cfg(feature = "telemetry")]
        NoopTracker,
    )
    .unwrap();

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
        #[cfg(feature = "telemetry")]
        NoopTracker,
    )
    .unwrap();
    bob_socket.read_to_end(&mut recv_data).await.unwrap();

    recv_data
}

pub fn stateless_socket_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("stateless_socket_benchmark");
    const KB: usize = 1024;

    group.sample_size(50000);

    for size in if cfg!(feature = "all-benchmarks") {
        &[128 * KB, 1024 * KB][..]
    } else {
        &[1024 * KB][..]
    } {
        let mut alice_data = vec![0u8; *size];

        hopr_types::crypto_random::random_fill(&mut alice_data);

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

/// Records each `poll_write` as one wire packet, preserving packet boundaries
/// (the socket flushes each segment individually with the default zero
/// `max_buffered_segments`, so one write equals one segment).
#[derive(Clone, Default)]
struct PacketRecorder(std::sync::Arc<std::sync::Mutex<Vec<Box<[u8]>>>>);

impl AsyncWrite for PacketRecorder {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        self.0.lock().unwrap().push(buf.to_vec().into_boxed_slice());
        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// Writes `data` through a stateless socket, capturing the individual wire packets.
///
/// Unlike [`alice_send_data`], the socket is not closed: a terminating segment would
/// be shuffled in front of data segments by [`window_shuffle`] and abort the receive
/// pipeline early, which is an artifact rather than a datagram workload.
fn capture_wire_packets(data: &[u8], runtime: &tokio::runtime::Runtime) -> Vec<Box<[u8]>> {
    use futures::AsyncWriteExt;

    let recorder = PacketRecorder::default();
    let transport = DuplexIO::from((recorder.clone(), futures::io::empty()));

    runtime.block_on(async move {
        let mut alice_socket = UnreliableSocket::<MTU>::new_stateless(
            "alice",
            transport,
            SessionSocketConfig::default(),
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )
        .unwrap();
        alice_socket.write_all(data).await.unwrap();
        alice_socket.flush().await.unwrap();
    });

    let packets = recorder.0.lock().unwrap().drain(..).collect::<Vec<_>>();
    assert!(!packets.is_empty());
    packets
}

/// Deterministically reorders packets within a sliding window of the given size,
/// approximating the `buffer_unordered(window)` mixing used by the socket tests.
/// A window of 0 keeps the original order.
fn window_shuffle(packets: &[Box<[u8]>], window: usize, seed: [u8; 32]) -> Vec<u8> {
    use rand::{RngExt, SeedableRng, rngs::StdRng};

    let mut rng = StdRng::from_seed(seed);
    let mut out = Vec::with_capacity(packets.iter().map(|p| p.len()).sum());
    let mut buf: Vec<&[u8]> = Vec::with_capacity(window + 1);

    for packet in packets {
        if window == 0 {
            out.extend_from_slice(packet);
            continue;
        }
        buf.push(packet);
        if buf.len() > window {
            let i = rng.random_range(0..buf.len());
            out.extend_from_slice(buf.swap_remove(i));
        }
    }
    while !buf.is_empty() {
        let i = rng.random_range(0..buf.len());
        out.extend_from_slice(buf.swap_remove(i));
    }
    out
}

pub async fn bob_receive_data_with_cfg(data: Vec<u8>, mut recv_data: Vec<u8>, cfg: SessionSocketConfig) -> Vec<u8> {
    use futures::AsyncReadExt;

    let mut bob_socket = UnreliableSocket::<MTU>::new_stateless(
        "bob",
        DuplexIO::from((futures::io::sink(), Cursor::new(data))),
        cfg,
        #[cfg(feature = "telemetry")]
        NoopTracker,
    )
    .unwrap();
    bob_socket.read_to_end(&mut recv_data).await.unwrap();

    recv_data
}

/// Compares the ordered (sequencing) and unordered (arrival-order) receive pipelines
/// under packet reordering of increasing severity.
pub fn stateless_socket_reordering(c: &mut Criterion) {
    let mut group = c.benchmark_group("stateless_socket_reordering");
    const SIZE: usize = 1024 * 1024;

    group.sample_size(30);
    group.throughput(Throughput::Bytes(SIZE as u64));

    let mut alice_data = vec![0u8; SIZE];
    hopr_types::crypto_random::random_fill(&mut alice_data);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let packets = capture_wire_packets(&alice_data, &runtime);

    // Frame ids never time out in the ordered arm: all frames are present in the wire
    // data, so the sequencer only pays the re-sorting cost, never the gap discard.
    let ordered_cfg = SessionSocketConfig {
        frame_timeout: std::time::Duration::from_secs(10),
        ..Default::default()
    };
    let unordered_cfg = SessionSocketConfig {
        frame_timeout: std::time::Duration::from_secs(10),
        deliver_in_order: false,
        ..Default::default()
    };

    for mixing_factor in [0usize, 10, 32] {
        let wire_data = window_shuffle(&packets, mixing_factor, [0xEE; 32]);

        // Sanity: ordered mode restores the exact byte stream...
        let ordered_out = runtime.block_on(bob_receive_data_with_cfg(
            wire_data.clone(),
            Vec::with_capacity(SIZE),
            ordered_cfg,
        ));
        assert_eq!(alice_data, ordered_out, "ordered mode must restore the byte stream");

        // ...while unordered mode delivers all bytes, possibly permuted at frame level.
        let unordered_out = runtime.block_on(bob_receive_data_with_cfg(
            wire_data.clone(),
            Vec::with_capacity(SIZE),
            unordered_cfg,
        ));
        let (mut sent_sorted, mut recv_sorted) = (alice_data.clone(), unordered_out);
        sent_sorted.sort_unstable();
        recv_sorted.sort_unstable();
        assert_eq!(sent_sorted, recv_sorted, "unordered mode must deliver all bytes");

        group.bench_with_input(BenchmarkId::new("ordered", mixing_factor), &wire_data, |b, data| {
            b.to_async(&runtime)
                .iter(|| bob_receive_data_with_cfg(data.clone(), Vec::with_capacity(SIZE), ordered_cfg));
        });
        group.bench_with_input(BenchmarkId::new("unordered", mixing_factor), &wire_data, |b, data| {
            b.to_async(&runtime)
                .iter(|| bob_receive_data_with_cfg(data.clone(), Vec::with_capacity(SIZE), unordered_cfg));
        });
    }
    group.finish();
}

criterion_group!(benches, stateless_socket_benchmark, stateless_socket_reordering);
criterion_main!(benches);
