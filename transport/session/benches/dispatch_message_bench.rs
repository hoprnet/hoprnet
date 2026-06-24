use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hopr_api::types::{
    crypto_random::Randomizable,
    internal::{prelude::HoprPseudonym, routing::DestinationRouting},
};
use hopr_protocol_app::{prelude::ApplicationDataOut, v1::ApplicationDataIn};
use hopr_protocol_start::{StartChallenge, StartErrorReason, StartErrorType, StartProtocol};
use hopr_transport_session::{
    HoprStartProtocol, SESSION_APPLICATION_TAG, SessionId, SessionManager, SessionManagerConfig, SessionTarget,
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
/// `flume::Sender` does not implement `Sink` directly; only `SendSink` does.
/// `SendSink` owns the sender and is `Send`, but it is not publicly constructible
/// from outside the `flume` crate, and its lifetime parameter makes it
/// inexpressible as a return type here.  This thin zero-cost wrapper bridges the
/// gap for benchmarking only.
#[cfg(feature = "benchmark")]
#[derive(Clone)]
pub struct FlumeSink<T>(pub flume::Sender<T>);

#[cfg(feature = "benchmark")]
impl<T: Send + 'static> futures::Sink<T> for FlumeSink<T> {
    type Error = flume::SendError<T>;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().0.send(item)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// Builds a `SessionManager` for benchmarking without calling `start()`.
///
/// `start()` spawns tokio tasks that require a live runtime context, so we bypass
/// it entirely and inject the start-protocol sender directly.  The session cache
/// is pre-populated with `num_sessions` entries.
///
/// Returns the manager, session IDs, the start-protocol receiver (keep alive for
/// the start_tx handle), and the session receivers (keep alive for session_tx).
/// A background drainer is spawned to prevent unbounded channel accumulation.
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
    sm.set_active_sessions_for_benchmarking(num_sessions);
    sm.flush_pending_tasks_for_benchmarking();

    // Drain the channels in the background so messages from dispatch_message calls
    // do not accumulate indefinitely (unbounded queues distort steady-state timing).
    // Clone before moving into the thread since the originals are returned.
    let start_rx_clone = start_rx.clone();
    let session_receivers_clone = session_receivers.clone();
    std::thread::spawn(move || {
        loop {
            let start_drained = start_rx_clone.try_iter().count();
            let session_drained: usize =
                session_receivers_clone.iter().map(|r| r.try_iter().count()).sum();
            if start_drained == 0 && session_drained == 0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    });

    (sm, session_ids, start_rx, session_receivers)
}

/// Benchmarks for SessionManager::dispatch_message — the hottest method in the
/// session stack, called on every incoming packet.
///
/// Three paths are measured independently:
/// - `cache_hit`: packet for an existing session → DashMap lookup + channel try_send
/// - `cache_miss`: packet for an unknown session pseudonym → DashMap lookup + error return
/// - `start_protocol`: Start protocol packet forwarded to the handler channel
pub fn dispatch_message_benchmark(c: &mut Criterion) {
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

criterion_group!(benches, dispatch_message_benchmark);
criterion_main!(benches);
