use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{AsyncReadExt, AsyncWriteExt, FutureExt, StreamExt};
use hopr_api::types::{
    crypto::{keypairs::ChainKeypair, prelude::Keypair},
    crypto_random::Randomizable,
    internal::{
        prelude::HoprPseudonym,
        routing::{DestinationRouting, RoutingOptions},
    },
    primitive::prelude::Address,
};
use hopr_protocol_app::prelude::ApplicationDataOut;
use hopr_protocol_app::v1::ApplicationDataIn;
use hopr_protocol_start::{StartChallenge, StartErrorReason, StartErrorType, StartProtocol};
use hopr_transport_session::{
    Capabilities, Capability, HoprSession, HoprSessionConfig, HoprStartProtocol, SESSION_APPLICATION_TAG, SessionId,
    SessionManager, SessionManagerConfig, SessionTarget,
};

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

/// Wrapper that makes a `flume::Sender` implement `futures::Sink`.
///
/// flume's `Sender` does not implement `Sink` directly; only `SendSink` does.
/// `SendSink` is not `Send`, so it can't satisfy `SessionManager`'s `S: ... + Send`
/// bound. This wrapper bridges the gap for benchmarking only.
#[cfg(feature = "benchmark")]
#[derive(Clone)]
pub struct FlumeSink<T>(flume::Sender<T>);

#[cfg(feature = "benchmark")]
impl<T: Send + 'static> futures::Sink<T> for FlumeSink<T> {
    type Error = flume::SendError<T>;
    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().0.send(item)
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// Builds a `SessionManager` for benchmarking without calling `start()`.
///
/// `start()` spawns tokio tasks that require a live runtime context, so we bypass
/// it entirely and inject the start-protocol sender directly. The session cache
/// is pre-populated with `num_sessions` entries.
///
/// Returns the manager, session IDs, the start-protocol receiver (keep alive for
/// the start_tx handle), and the session receivers (keep alive for session_tx).
/// Both receivers must be kept alive by the caller for the duration of the
/// benchmark — dropping them closes the corresponding channel and panics any
/// subsequent `try_send`.
fn make_manager_with_sessions(
    num_sessions: usize,
) -> (
    SessionManager<FlumeSink<(DestinationRouting, ApplicationDataOut)>>,
    Vec<SessionId>,
    flume::Receiver<(
        HoprPseudonym,
        StartProtocol<SessionId, SessionTarget, hopr_transport_session::ByteCapabilities>,
    )>,
    Vec<flume::Receiver<ApplicationDataIn>>,
) {
    // Unbounded so `try_send` never fills up during high-iteration benchmarks.
    let (start_tx, start_rx) = flume::unbounded::<(
        HoprPseudonym,
        StartProtocol<SessionId, SessionTarget, hopr_transport_session::ByteCapabilities>,
    )>();

    let cfg = SessionManagerConfig {
        maximum_sessions: 1000,
        ..Default::default()
    };
    let sm = SessionManager::new(cfg);

    // Inject the start-protocol sender directly without going through `start()`,
    // which would spawn tokio tasks and require a runtime context.
    sm.set_start_protocol_tx_for_benchmarking(start_tx);

    let session_ids: Vec<_> = (0..num_sessions).map(|_| HoprPseudonym::random()).collect();
    let session_receivers = sm.insert_session_slot_for_benchmarking_multi(&session_ids);
    sm.flush_pending_tasks_for_benchmarking();

    (sm, session_ids, start_rx, session_receivers)
}

pub async fn alice_send_data(
    data: &[u8],
    caps: impl Into<Capabilities> + std::fmt::Debug,
) -> impl futures::Stream<Item = ApplicationDataIn> + Send {
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (_bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    let dst: Address = (&ChainKeypair::random()).into();
    let id = HoprPseudonym::random();
    let cfg = HoprSessionConfig {
        capabilities: caps.into(),
        ..Default::default()
    };

    let mut alice_session = HoprSession::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into().unwrap())),
        cfg,
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
    let id = HoprPseudonym::random();
    let cfg = HoprSessionConfig {
        capabilities: caps.into(),
        ..Default::default()
    };

    let mut bob_session = HoprSession::new(
        id,
        DestinationRouting::Return(id.into()),
        cfg,
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

    group.sample_size(50000);
    group.measurement_time(std::time::Duration::from_secs(30));

    for size in if cfg!(feature = "all-benchmarks") {
        &[16 * KB, 64 * KB, 128 * KB, 1024 * KB][..]
    } else {
        &[1024 * KB][..]
    } {
        let mut alice_data = vec![0u8; *size];
        rand::fill(&mut alice_data[..]);

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

    group.sample_size(50000);
    group.measurement_time(std::time::Duration::from_secs(30));

    for size in if cfg!(feature = "all-benchmarks") {
        &[16 * KB, 64 * KB, 128 * KB, 1024 * KB][..]
    } else {
        &[1024 * KB][..]
    } {
        let mut alice_data = vec![0u8; *size];
        rand::fill(&mut alice_data[..]);

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

/// Benchmarks for SessionManager::dispatch_message — the hottest method in the
/// session stack, called on every incoming packet.
///
/// Three paths are measured independently:
/// - `cache_hit`: packet for an existing session → DashMap lookup + channel try_send
/// - `cache_miss`: packet for an unknown session pseudonym → DashMap lookup + error return
/// - `start_protocol`: Start protocol packet forwarded to the handler channel
fn dispatch_message_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_message");

    group.sample_size(10_000);
    group.measurement_time(std::time::Duration::from_secs(10));

    // Reusable input messages for the three benchmark paths.
    let session_app_data = ApplicationDataIn {
        data: hopr_protocol_app::v1::ApplicationData::new(SESSION_APPLICATION_TAG, &b""[..]).unwrap(),
        packet_info: Default::default(),
    };

    // Construct a valid encoded Start protocol error message for the start-protocol
    // tag path. `dispatch_message` decodes this via `HoprStartProtocol::try_from`.
    let start_error_msg = HoprStartProtocol::SessionError(StartErrorType {
        challenge: StartChallenge::MAX,
        reason: StartErrorReason::NoSlotsAvailable,
    });
    let start_error_bytes = &start_error_msg.encode().unwrap().1[..];
    let start_app_data = ApplicationDataIn {
        data: hopr_protocol_app::v1::ApplicationData::new(
            StartProtocol::<SessionId, SessionTarget, hopr_transport_session::ByteCapabilities>::START_PROTOCOL_MESSAGE_TAG,
            start_error_bytes,
        )
        .unwrap(),
        packet_info: Default::default(),
    };

    let (sm, session_ids, start_rx, session_rx) = make_manager_with_sessions(50);
    let unknown_pseudonym = HoprPseudonym::random();
    let known_pseudonym = session_ids[0];

    // ── Cache hit path ──────────────────────────────────────────────────────
    // The pseudonym is registered in the cache; dispatch delivers to the session
    // channel via try_send.
    group.bench_function("cache_hit", |b| {
        // Move sm, start_rx, and session_rx into the closure so both channels
        // stay open for the entire measurement loop.
        let sm = sm.clone();
        let start_rx = start_rx.clone();
        let session_rx = session_rx.clone();
        b.iter(|| {
            let _ = (&start_rx, &session_rx);
            sm.dispatch_message(known_pseudonym, session_app_data.clone())
                .expect("dispatch_message must succeed for known session")
        });
    });

    // ── Cache miss path ──────────────────────────────────────────────────────
    // The pseudonym is not in the cache; DashMap returns None and we return
    // TransportSessionError::UnknownData.
    group.bench_function("cache_miss", |b| {
        let sm = sm.clone();
        let start_rx = start_rx.clone();
        let session_rx = session_rx.clone();
        b.iter(|| {
            let _ = (&start_rx, &session_rx);
            sm.dispatch_message(unknown_pseudonym, session_app_data.clone())
                .expect_err("cache miss must return an error")
        });
    });

    // ── Start protocol tag path ──────────────────────────────────────────────
    // The packet has the Start protocol tag; it is forwarded via try_send to
    // the start_protocol_tx channel (no session lookup involved).
    group.bench_function("start_protocol_tag", |b| {
        let sm = sm.clone();
        let start_rx = start_rx.clone();
        let session_rx = session_rx.clone();
        b.iter(|| {
            let _ = (&start_rx, &session_rx);
            sm.dispatch_message(known_pseudonym, start_app_data.clone())
                .expect("dispatch_message must succeed for start protocol tag")
        });
    });

    // ── Vary cache size ───────────────────────────────────────────────────────
    // Compare cache_hit latency as session count grows: 10, 100, 500.
    // (n=0 is a cache miss and tested separately above.)
    for &n in &[10usize, 100, 500] {
        let (sm, ids, start_rx, session_rx) = make_manager_with_sessions(n);
        let known = ids[0];
        group.bench_with_input(BenchmarkId::new("cache_hit_n_sessions", n), &n, |b, _| {
            b.iter(|| {
                let _ = (&start_rx, &session_rx);
                sm.dispatch_message(known, session_app_data.clone())
                    .expect("dispatch_message must succeed for known session")
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    session_raw_benchmark,
    session_segmentation_benchmark,
    dispatch_message_benchmark
);
criterion_main!(benches);
